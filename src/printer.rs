use std::process::Command;
use uuid::Uuid;

#[derive(Debug)]
pub enum PrinterError {
    CupsError(String),
    PrinterNotFound(String),
    InvalidText(String),
    SystemError(String),
}

impl std::fmt::Display for PrinterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrinterError::CupsError(msg) => write!(f, "CUPS error: {}", msg),
            PrinterError::PrinterNotFound(msg) => write!(f, "Printer not found: {}", msg),
            PrinterError::InvalidText(msg) => write!(f, "Invalid text: {}", msg),
            PrinterError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for PrinterError {}

pub async fn print_lines(lines: &[Option<String>], printer_name: Option<&str>, label_size: Option<&str>) -> Result<String, PrinterError> {
    if lines.is_empty() || lines[0].is_none() || lines[0].as_ref().unwrap().is_empty() {
        return Err(PrinterError::InvalidText("First line cannot be empty".to_string()));
    }

    let printer = match printer_name {
        Some(name) => name.to_string(),
        None => find_dymo_printer().await?,
    };

    let job_id = Uuid::new_v4().to_string();
    let lines_owned: Vec<Option<String>> = lines.to_vec();
    let label_size_owned = label_size.map(|s| s.to_string());

    let result = tokio::task::spawn_blocking(move || {
        print_via_lp_lines(&lines_owned, &printer, &job_id, label_size_owned.as_deref())
    }).await
    .map_err(|e| PrinterError::SystemError(format!("Task join error: {}", e)))?;

    result
}

pub async fn list_printers() -> Result<Vec<String>, PrinterError> {
    tokio::task::spawn_blocking(|| {
        let output = Command::new("lpstat")
            .arg("-p")
            .output()
            .map_err(|e| PrinterError::SystemError(format!("Failed to run lpstat: {}", e)))?;

        if !output.status.success() {
            return Err(PrinterError::CupsError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let printers: Vec<String> = output_str
            .lines()
            .filter_map(|line| {
                if line.starts_with("printer ") {
                    line.split_whitespace().nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(printers)
    }).await
    .map_err(|e| PrinterError::SystemError(format!("Task join error: {}", e)))?
}

async fn find_dymo_printer() -> Result<String, PrinterError> {
    let printers = list_printers().await?;

    for printer in &printers {
        if printer.to_lowercase().contains("dymo") {
            return Ok(printer.clone());
        }
    }

    if printers.is_empty() {
        Err(PrinterError::PrinterNotFound("No printers found".to_string()))
    } else {
        Err(PrinterError::PrinterNotFound(format!(
            "No Dymo printer found. Available printers: {}",
            printers.join(", ")
        )))
    }
}

fn print_via_lp_lines(lines: &[Option<String>], printer: &str, job_id: &str, label_size: Option<&str>) -> Result<String, PrinterError> {
    let media_size = label_size.unwrap_or("30252");

    let mut cmd = Command::new("lp");
    cmd.arg("-d").arg(printer)
       .arg("-t").arg(format!("Label-{}", job_id))
       .arg("-o").arg(format!("media={}", media_size))
       .arg("-o").arg("fit-to-page")
       .arg("-o").arg("orientation-requested=4");

    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| PrinterError::SystemError(format!("Failed to start lp command: {}", e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;

        // Format the text from multiple lines
        let formatted_text = format!("\n\n{}", lines
            .iter()
            .filter_map(|line| line.as_ref())
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<String>>()
            .join("\n"));

        stdin.write_all(formatted_text.as_bytes())
            .map_err(|e| PrinterError::SystemError(format!("Failed to write to lp stdin: {}", e)))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| PrinterError::SystemError(format!("Failed to wait for lp command: {}", e)))?;

    if !output.status.success() {
        return Err(PrinterError::CupsError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().next() {
        if line.contains("request id is") {
            if let Some(actual_job_id) = line.split_whitespace().last() {
                return Ok(actual_job_id.to_string());
            }
        }
    }

    Ok(job_id.to_string())
}