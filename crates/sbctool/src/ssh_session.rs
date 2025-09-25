use anyhow::Result;
use ssh2::Session;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use std::io::Read;

pub struct SSHSession {
    session: Arc<Mutex<Session>>,
    target: String,
}

impl SSHSession {
    pub async fn new(target: &str) -> Result<Self> {
        let (user, host) = Self::parse_target(target).await?;
        println!("SSH Session: Connecting to {}@{}", user, host);
        
        // Connect to the remote host
        let tcp = TcpStream::connect(&host)?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
        
        // Create SSH session
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        
        // Authenticate (try public key first, then password)
        // For now, we'll use a simple approach - in production you'd want proper key handling
        if sess.userauth_agent(&user).is_err() {
            // If agent auth fails, try password (this is a simplified approach)
            // In production, you might want to handle this more gracefully
            return Err(anyhow::anyhow!("SSH authentication failed"));
        }
        
        Ok(SSHSession {
            session: Arc::new(Mutex::new(sess)),
            target: target.to_string(),
        })
    }
    
    async fn parse_target(target: &str) -> Result<(String, String)> {
        if let Some((user, host)) = target.split_once('@') {
            Ok((user.to_string(), host.to_string()))
        } else {
            // Try to resolve alias using ssh -G
            use std::process::Command;
            
            if let Ok(output) = Command::new("ssh").arg("-G").arg(target).output() {
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
                    
                    let host = resolved_host.unwrap_or(target.to_string());
                    let user = resolved_user.unwrap_or_else(|| {
                        std::env::var("USER").unwrap_or_else(|_| "root".to_string())
                    });
                    Ok((user, host))
                } else {
                    Err(anyhow::anyhow!("Failed to resolve SSH alias: {}", target))
                }
            } else {
                Err(anyhow::anyhow!("SSH command not available"))
            }
        }
    }
    
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        let session = self.session.lock().await;
        
        // Create a new channel for this command
        let mut channel = session.channel_session()?;
        channel.exec(command)?;
        
        // Read the output
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        
        // Wait for the command to complete
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        
        if exit_status == 0 {
            Ok(output.trim().to_string())
        } else {
            Err(anyhow::anyhow!("Command failed with exit status: {}", exit_status))
        }
    }
    
    pub async fn execute_multiple_commands(&self, commands: &[&str]) -> Result<Vec<String>> {
        let mut results = Vec::new();
        
        for command in commands {
            match self.execute_command(command).await {
                Ok(output) => results.push(output),
                Err(e) => {
                    // Log error but continue with other commands
                    eprintln!("Command '{}' failed: {}", command, e);
                    results.push(format!("Error: {}", e));
                }
            }
        }
        
        Ok(results)
    }
    
    pub async fn start_log_stream(&self, log_sender: Arc<Mutex<Vec<crate::tui::LogEntry>>>) -> Result<()> {
        let session = self.session.lock().await;
        
        // Start journalctl -f for real-time logs
        let mut channel = session.channel_session()?;
        channel.exec("journalctl -f --no-hostname --output=short-iso")?;
        
        // Stream logs in a loop
        let mut buffer = [0; 1024];
        loop {
            match channel.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    for line in data.lines() {
                        if let Some(entry) = self.parse_journald_log_line(line) {
                            let mut logs = log_sender.lock().await;
                            logs.push(entry);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading log stream: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_journald_log_line(&self, line: &str) -> Option<crate::tui::LogEntry> {
        // Example: "2025-09-05T18:49:25+0000 hostname systemd[1]: Started Session 1 of User stulluk."
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 { return None; }

        let timestamp = parts[0].split('T').nth(1).unwrap_or("").split('+').next().unwrap_or("").to_string();
        let message = parts[2].to_string();

        let level = if message.to_lowercase().contains("error") || message.to_lowercase().contains("fail") {
            "ERROR"
        } else if message.to_lowercase().contains("warn") || message.to_lowercase().contains("warning") {
            "WARN"
        } else if message.to_lowercase().contains("info") || message.to_lowercase().contains("start") || message.to_lowercase().contains("started") {
            "INFO"
        } else if message.to_lowercase().contains("debug") {
            "DEBUG"
        } else {
            "UNKNOWN"
        }.to_string();

        Some(crate::tui::LogEntry { timestamp, level, message })
    }
}
