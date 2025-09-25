use anyhow::Result;
use crate::tui::LogEntry;
use crate::ssh_session::SSHSession;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

pub struct LogCollector {
    connection_type: String,
    target: String,
    is_android: bool,
    ssh_session: Option<Arc<SSHSession>>,
}

impl LogCollector {
    pub fn new(connection_type: &str, target: &str, is_android: bool) -> Self {
        Self {
            connection_type: connection_type.to_string(),
            target: target.to_string(),
            is_android,
            ssh_session: None,
        }
    }
    
    pub fn new_with_ssh_session(connection_type: &str, target: &str, is_android: bool, ssh_session: Arc<SSHSession>) -> Self {
        Self {
            connection_type: connection_type.to_string(),
            target: target.to_string(),
            is_android,
            ssh_session: Some(ssh_session),
        }
    }

    pub async fn start_log_collection(&self, log_sender: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>) {
        if self.is_android {
            self.collect_android_logs(log_sender).await;
        } else {
            self.collect_linux_logs(log_sender).await;
        }
    }

    async fn collect_android_logs(&self, log_sender: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>) {
        loop {
            match self.get_android_logs().await {
                Ok(logs) => {
                    if let Ok(mut sender) = log_sender.lock() {
                        for log in logs {
                            sender.push(log);
                        }
                    }
                }
                Err(e) => {
                    let error_log = LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        level: "ERROR".to_string(),
                        message: format!("Failed to get Android logs: {}", e),
                    };
                    if let Ok(mut sender) = log_sender.lock() {
                        sender.push(error_log);
                    }
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn collect_linux_logs(&self, log_sender: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>) {
        // Try journald first
        if self.has_journald().await {
            self.collect_journald_logs(log_sender).await;
        } else {
            self.collect_syslog_logs(log_sender).await;
        }
    }

    async fn has_journald(&self) -> bool {
        match self.execute_command("which journalctl").await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn collect_journald_logs(&self, log_sender: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>) {
        loop {
            match self.get_journald_logs().await {
                Ok(logs) => {
                    if let Ok(mut sender) = log_sender.lock() {
                        for log in logs {
                            sender.push(log);
                        }
                    }
                }
                Err(e) => {
                    let error_log = LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        level: "ERROR".to_string(),
                        message: format!("Failed to get journald logs: {}", e),
                    };
                    if let Ok(mut sender) = log_sender.lock() {
                        sender.push(error_log);
                    }
                }
            }
            sleep(Duration::from_secs(3)).await;
        }
    }

    async fn collect_syslog_logs(&self, log_sender: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>) {
        loop {
            match self.get_syslog_logs().await {
                Ok(logs) => {
                    if let Ok(mut sender) = log_sender.lock() {
                        for log in logs {
                            sender.push(log);
                        }
                    }
                }
                Err(e) => {
                    let error_log = LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        level: "ERROR".to_string(),
                        message: format!("Failed to get syslog: {}", e),
                    };
                    if let Ok(mut sender) = log_sender.lock() {
                        sender.push(error_log);
                    }
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn get_android_logs(&self) -> Result<Vec<LogEntry>> {
        let output = self.execute_command("logcat -d -v time").await?;
        let mut logs = Vec::new();

        for line in output.lines() {
            if let Some(log_entry) = self.parse_android_log_line(line) {
                logs.push(log_entry);
            }
        }

        // Return last 20 entries
        logs.reverse();
        logs.truncate(20);
        Ok(logs)
    }

    async fn get_journald_logs(&self) -> Result<Vec<LogEntry>> {
        let output = self.execute_command("journalctl --no-pager -n 20 -o short-iso").await?;
        let mut logs = Vec::new();

        for line in output.lines() {
            if let Some(log_entry) = self.parse_journald_log_line(line) {
                logs.push(log_entry);
            }
        }

        Ok(logs)
    }

    async fn get_syslog_logs(&self) -> Result<Vec<LogEntry>> {
        let syslog_paths = vec![
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/kern.log",
        ];

        for path in syslog_paths {
            if let Ok(output) = self.execute_command(&format!("tail -n 20 {}", path)).await {
                let mut logs = Vec::new();
                for line in output.lines() {
                    if let Some(log_entry) = self.parse_syslog_line(line) {
                        logs.push(log_entry);
                    }
                }
                if !logs.is_empty() {
                    return Ok(logs);
                }
            }
        }

        Err(anyhow::anyhow!("No syslog files found"))
    }

    fn parse_android_log_line(&self, line: &str) -> Option<LogEntry> {
        // Android logcat format: MM-DD HH:MM:SS.fff PID TID LEVEL TAG: MESSAGE
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }

        let timestamp = format!("{} {}", parts[0], parts[1]);
        let level = parts[4].to_uppercase();
        let message = parts[5..].join(" ");

        Some(LogEntry {
            timestamp,
            level,
            message,
        })
    }

    fn parse_journald_log_line(&self, line: &str) -> Option<LogEntry> {
        // journald format: YYYY-MM-DDTHH:MM:SS+ZZ:ZZ HOSTNAME SERVICE[PID]: MESSAGE
        if let Some(space_pos) = line.find(' ') {
            let timestamp_part = &line[..space_pos];
            let rest = &line[space_pos + 1..];
            
            if let Some(colon_pos) = rest.find(':') {
                let _service_part = &rest[..colon_pos];
                let message = &rest[colon_pos + 1..].trim();
                
                let level = if message.to_lowercase().contains("error") {
                    "ERROR"
                } else if message.to_lowercase().contains("warn") {
                    "WARN"
                } else if message.to_lowercase().contains("info") {
                    "INFO"
                } else {
                    "DEBUG"
                };

                return Some(LogEntry {
                    timestamp: timestamp_part.to_string(),
                    level: level.to_string(),
                    message: message.to_string(),
                });
            }
        }
        None
    }

    fn parse_syslog_line(&self, line: &str) -> Option<LogEntry> {
        // syslog format: MMM DD HH:MM:SS HOSTNAME SERVICE: MESSAGE
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }

        let timestamp = format!("{} {} {}", parts[0], parts[1], parts[2]);
        let service = parts[4].trim_end_matches(':');
        let message = parts[5..].join(" ");

        let level = if message.to_lowercase().contains("error") {
            "ERROR"
        } else if message.to_lowercase().contains("warn") {
            "WARN"
        } else if message.to_lowercase().contains("info") {
            "INFO"
        } else {
            "DEBUG"
        };

        Some(LogEntry {
            timestamp,
            level: level.to_string(),
            message: format!("{}: {}", service, message),
        })
    }

    async fn execute_command(&self, command: &str) -> Result<String> {
        match self.connection_type.as_str() {
            "ssh" => self.execute_ssh_command(command).await,
            "adb" => self.execute_adb_command(command).await,
            _ => Err(anyhow::anyhow!("Unknown connection type: {}", self.connection_type)),
        }
    }

    async fn execute_ssh_command(&self, command: &str) -> Result<String> {
        use std::process::Command;
        
        // Parse target to get user@host
        let (user, host) = if let Some((u, h)) = self.target.split_once('@') {
            (u.to_string(), h.to_string())
        } else {
            // Try to resolve alias using ssh -G
            if let Ok(output) = Command::new("ssh").arg("-G").arg(&self.target).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut resolved_host = None;
                    let mut resolved_user = None;
                    
                    for line in stdout.lines() {
                        if let Some(rest) = line.strip_prefix("hostname ") {
                            resolved_host = Some(rest.trim().to_string());
                        } else if let Some(rest) = line.strip_prefix("user ") {
                            resolved_user = Some(rest.trim().to_string());
                        }
                    }
                    
                    let host = resolved_host.unwrap_or(self.target.clone());
                    let user = resolved_user.unwrap_or_else(|| {
                        std::env::var("USER").unwrap_or_else(|_| "root".to_string())
                    });
                    (user, host)
                } else {
                    return Err(anyhow::anyhow!("Failed to resolve SSH alias: {}", self.target));
                }
            } else {
                return Err(anyhow::anyhow!("SSH command not available"));
            }
        };

        // Execute command via SSH
        let output = Command::new("ssh")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-o")
            .arg("ServerAliveInterval=2")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(&format!("{}@{}", user, host))
            .arg(command)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow::anyhow!("SSH command failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    async fn execute_adb_command(&self, command: &str) -> Result<String> {
        use std::process::Command;
        
        if self.target == "auto" {
            // Try to find a device automatically
            if let Ok(output) = Command::new("adb").arg("devices").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                
                // Find first device that's not "List of devices attached"
                for line in lines {
                    if line.contains("\tdevice") {
                        if let Some(serial) = line.split('\t').next() {
                            if !serial.is_empty() {
                                return self.execute_adb_command_with_serial(serial, command).await;
                            }
                        }
                    }
                }
            }
            Err(anyhow::anyhow!("No ADB devices found"))
        } else {
            self.execute_adb_command_with_serial(&self.target, command).await
        }
    }

    async fn execute_adb_command_with_serial(&self, serial: &str, command: &str) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("adb")
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg(command)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow::anyhow!("ADB command failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}
