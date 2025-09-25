use anyhow::Result;
use crate::tui::SystemInfo;
use crate::ssh_session::SSHSession;
use std::sync::Arc;

pub struct SystemInfoCollector {
    connection_type: String,
    target: String,
    ssh_session: Option<Arc<SSHSession>>,
}

impl SystemInfoCollector {
    pub fn new(connection_type: &str, target: &str) -> Self {
        Self {
            connection_type: connection_type.to_string(),
            target: target.to_string(),
            ssh_session: None,
        }
    }
    
    pub async fn new_with_ssh_session(connection_type: &str, target: &str) -> Result<Self> {
        let mut collector = Self::new(connection_type, target);
        
        if connection_type == "ssh" {
            let ssh_session = SSHSession::new(target).await?;
            collector.ssh_session = Some(Arc::new(ssh_session));
        }
        
        Ok(collector)
    }

    pub async fn collect_system_info(&self) -> Result<SystemInfo> {
        // If we have a persistent SSH session, use batch commands for better performance
        if let Some(ssh_session) = &self.ssh_session {
            self.collect_system_info_batch(ssh_session).await
        } else {
            self.collect_system_info_sequential().await
        }
    }
    
    async fn collect_system_info_batch(&self, ssh_session: &SSHSession) -> Result<SystemInfo> {
        // Execute multiple commands in batch for better performance
        let commands = vec![
            "uname -a",
            "hostname", 
            "cat /proc/device-tree/model 2>/dev/null || echo 'No model'",
            "cat /proc/device-tree/compatible 2>/dev/null || echo 'No compatible'",
            "cat /proc/cpuinfo",
            "cat /proc/meminfo",
            "cat /proc/uptime",
            "cat /etc/os-release 2>/dev/null || echo 'No os-release'"
        ];
        
        let results = ssh_session.execute_multiple_commands(&commands).await?;
        
        // Parse results
        let uname_output = &results[0];
        let hostname = results[1].trim().to_string();
        
        // Parse uname output
        let parts: Vec<&str> = uname_output.split_whitespace().collect();
        let kernel = if parts.len() > 2 {
            format!("{} {}", parts[0], parts[2])
        } else {
            parts[0].to_string()
        };
        
        let architecture = if parts.len() > 12 {
            parts[12].to_string()
        } else {
            "unknown".to_string()
        };

        // Parse chip info from device tree
        let chip = self.parse_chip_from_batch_results(&results[2], &results[3], &results[4]);
        
        // Parse CPU info
        let cpu_info = self.parse_cpu_from_cpuinfo(&results[4]);
        
        // Parse memory info
        let memory = self.parse_memory_from_meminfo(&results[5]);
        
        // Parse uptime
        let uptime = self.parse_uptime_from_proc(&results[6]);
        
        // Parse OS info
        let os_info = self.parse_os_from_release(&results[7]);

        Ok(SystemInfo {
            hostname,
            kernel,
            architecture,
            chip,
            cpu_info,
            memory,
            uptime,
            os_info,
        })
    }
    
    async fn collect_system_info_sequential(&self) -> Result<SystemInfo> {
        let uname_output = self.execute_command("uname -a").await?;
        let hostname = self.execute_command("hostname").await?.trim().to_string();
        
        // Parse uname output
        let parts: Vec<&str> = uname_output.split_whitespace().collect();
        let kernel = if parts.len() > 2 {
            format!("{} {}", parts[0], parts[2])
        } else {
            parts[0].to_string()
        };
        
        let architecture = if parts.len() > 12 {
            parts[12].to_string()
        } else {
            "unknown".to_string()
        };

        // Get chip information from device tree
        let chip = self.get_chip_info().await.ok();

        // Get CPU information
        let cpu_info = self.get_cpu_info().await.unwrap_or_else(|_| "Unknown".to_string());

        // Get memory information
        let memory = self.get_memory_info().await.unwrap_or_else(|_| "Unknown".to_string());

        // Get uptime
        let uptime = self.get_uptime().await.unwrap_or_else(|_| "Unknown".to_string());

        // Get OS information
        let os_info = self.get_os_info().await.unwrap_or_else(|_| "Unknown".to_string());

        Ok(SystemInfo {
            hostname,
            kernel,
            architecture,
            chip,
            cpu_info,
            memory,
            uptime,
            os_info,
        })
    }

    async fn execute_command(&self, command: &str) -> Result<String> {
        match self.connection_type.as_str() {
            "ssh" => {
                if let Some(ssh_session) = &self.ssh_session {
                    // Use persistent SSH session
                    ssh_session.execute_command(command).await
                } else {
                    // Fallback to old method
                    self.execute_ssh_command(command).await
                }
            },
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

        // Execute command via SSH with timeout and terminal reset
        let output = Command::new("ssh")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-o")
            .arg("ServerAliveInterval=2")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("RequestTTY=no")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("UserKnownHostsFile=/dev/null")
            .arg(&format!("{}@{}", user, host))
            .arg(&format!("timeout 30 bash -c '{}'", command))
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

    async fn get_chip_info(&self) -> Result<String> {
        if self.connection_type == "adb" {
            // For Android, get device info from properties
            let mut chip_info = String::new();
            
            // Get manufacturer
            if let Ok(manufacturer) = self.execute_command("getprop ro.product.manufacturer").await {
                chip_info.push_str(&manufacturer.trim());
            }
            
            // Get model
            let mut model = String::new();
            if let Ok(model_result) = self.execute_command("getprop ro.product.model").await {
                model = model_result.trim().to_string();
                if !chip_info.is_empty() {
                    chip_info.push_str(" ");
                }
                chip_info.push_str(&model);
            }
            
            // Get board info as additional detail
            if let Ok(board) = self.execute_command("getprop ro.product.board").await {
                let board_trimmed = board.trim();
                if !board_trimmed.is_empty() && board_trimmed != model {
                    if !chip_info.is_empty() {
                        chip_info.push_str(" (");
                        chip_info.push_str(board_trimmed);
                        chip_info.push_str(")");
                    }
                }
            }
            
            if !chip_info.is_empty() {
                return Ok(chip_info);
            }
            
            // Fallback to board info only
            if let Ok(board) = self.execute_command("getprop ro.product.board").await {
                return Ok(board.trim().to_string());
            }
        } else {
            // For Linux systems, try to read from device tree first
            if let Ok(model) = self.execute_command("cat /proc/device-tree/model 2>/dev/null").await {
                let model_trimmed = model.trim();
                if !model_trimmed.is_empty() && model_trimmed != "No model" {
                    return Ok(model_trimmed.to_string());
                }
            }
            
            // Try compatible string
            if let Ok(compatible) = self.execute_command("cat /proc/device-tree/compatible 2>/dev/null").await {
                let compatible_trimmed = compatible.trim();
                if !compatible_trimmed.is_empty() && compatible_trimmed != "No compatible" {
                    if let Some(chip) = self.parse_chip_from_compatible(&compatible_trimmed) {
                        return Ok(chip);
                    }
                }
            }

            // Fallback to cpuinfo parsing
            if let Ok(cpuinfo) = self.execute_command("cat /proc/cpuinfo").await {
                if let Some(chip) = self.parse_chip_from_cpuinfo(&cpuinfo) {
                    return Ok(chip);
                }
            }
        }

        Err(anyhow::anyhow!("Could not determine chip information"))
    }

    fn parse_chip_from_output(&self, output: &str) -> Option<String> {
        let output = output.trim();
        
        // Common SBC patterns
        if output.contains("Raspberry Pi") {
            return Some("Raspberry Pi".to_string());
        }
        if output.contains("Orange Pi") {
            return Some("Orange Pi".to_string());
        }
        if output.contains("Banana Pi") {
            return Some("Banana Pi".to_string());
        }
        if output.contains("NanoPi") {
            return Some("NanoPi".to_string());
        }
        if output.contains("Khadas") {
            return Some("Khadas".to_string());
        }
        if output.contains("Rockchip") {
            return Some("Rockchip".to_string());
        }
        if output.contains("Allwinner") {
            return Some("Allwinner".to_string());
        }
        if output.contains("Broadcom") {
            return Some("Broadcom".to_string());
        }
        if output.contains("Amlogic") {
            return Some("Amlogic".to_string());
        }

        None
    }

    fn parse_chip_from_compatible(&self, compatible: &str) -> Option<String> {
        // Parse device tree compatible string
        // Example: "rockchip,rk3399-rockpro64\0rockchip,rk3399"
        let compatible = compatible.replace('\0', " ");
        
        // Common SBC patterns
        if compatible.contains("rockchip") {
            if compatible.contains("rk3399") {
                return Some("Rockchip RK3399".to_string());
            } else if compatible.contains("rk3568") {
                return Some("Rockchip RK3568".to_string());
            } else if compatible.contains("rk3588") {
                return Some("Rockchip RK3588".to_string());
            } else {
                return Some("Rockchip".to_string());
            }
        }
        
        if compatible.contains("amlogic") {
            if compatible.contains("g12") {
                return Some("Amlogic G12".to_string());
            } else if compatible.contains("s905") {
                return Some("Amlogic S905".to_string());
            } else if compatible.contains("s922") {
                return Some("Amlogic S922".to_string());
            } else {
                return Some("Amlogic".to_string());
            }
        }
        
        if compatible.contains("allwinner") {
            return Some("Allwinner".to_string());
        }
        
        if compatible.contains("broadcom") {
            return Some("Broadcom".to_string());
        }
        
        if compatible.contains("qualcomm") {
            return Some("Qualcomm".to_string());
        }
        
        if compatible.contains("nvidia") {
            return Some("Nvidia Jetson".to_string());
        }
        
        None
    }

    fn parse_chip_from_cpuinfo(&self, cpuinfo: &str) -> Option<String> {
        for line in cpuinfo.lines() {
            if line.starts_with("Hardware") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    let hardware = parts[1].trim();
                    if !hardware.is_empty() && hardware != "BCM2835" {
                        return Some(hardware.to_string());
                    }
                }
            }
            if line.starts_with("CPU implementer") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    let implementer = parts[1].trim();
                    match implementer {
                        "0x41" => return Some("ARM".to_string()),
                        "0x42" => return Some("Broadcom".to_string()),
                        "0x51" => return Some("Qualcomm".to_string()),
                        _ => {}
                    }
                }
            }
        }
        None
    }

    async fn get_cpu_info(&self) -> Result<String> {
        let cpuinfo = self.execute_command("cat /proc/cpuinfo").await?;
        
        // Try to get model name first
        for line in cpuinfo.lines() {
            if line.starts_with("model name") || line.starts_with("Processor") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return Ok(parts[1].trim().to_string());
                }
            }
        }

        // For ARM devices, try to get CPU implementer info
        let mut implementer = None;
        let mut architecture = None;
        let mut processor_count = 0;
        
        for line in cpuinfo.lines() {
            if line.starts_with("processor") {
                processor_count += 1;
            } else if line.starts_with("CPU implementer") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    implementer = Some(parts[1].trim());
                }
            } else if line.starts_with("CPU architecture") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    architecture = Some(parts[1].trim());
                }
            }
        }

        // Build CPU description
        let mut cpu_desc = String::new();
        if let Some(impl_code) = implementer {
            match impl_code {
                "0x41" => cpu_desc.push_str("ARM"),
                "0x42" => cpu_desc.push_str("Broadcom"),
                "0x51" => cpu_desc.push_str("Qualcomm"),
                _ => cpu_desc.push_str("Unknown"),
            }
        }
        
        if let Some(arch) = architecture {
            if !cpu_desc.is_empty() {
                cpu_desc.push_str(" ");
            }
            cpu_desc.push_str(&format!("v{}", arch));
        }
        
        if processor_count > 0 {
            if !cpu_desc.is_empty() {
                cpu_desc.push_str(" ");
            }
            cpu_desc.push_str(&format!("({} cores)", processor_count));
        }
        
        if cpu_desc.is_empty() {
            cpu_desc = format!("{} cores", processor_count);
        }
        
        Ok(cpu_desc)
    }

    async fn get_memory_info(&self) -> Result<String> {
        if self.connection_type == "adb" {
            // For Android, try to use the free command first
            if let Ok(free_output) = self.execute_command("free").await {
                // Parse free output: "total        used        free      shared     buffers"
                // "Mem:       2005991424  1791692800   214298624     2093056    36106240"
                for line in free_output.lines() {
                    if line.starts_with("Mem:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            if let Ok(total_kb) = parts[1].parse::<u64>() {
                                let mb = total_kb / 1024;
                                let gb = mb / 1024;
                                
                                if gb > 0 {
                                    return Ok(format!("{} GB", gb));
                                } else {
                                    return Ok(format!("{} MB", mb));
                                }
                            }
                        }
                    }
                }
            }
            
            // Fallback to /proc/meminfo
            if let Ok(meminfo) = self.execute_command("cat /proc/meminfo").await {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            let kb: u64 = parts[1].parse().unwrap_or(0);
                            let mb = kb / 1024;
                            let gb = mb / 1024;
                            
                            if gb > 0 {
                                return Ok(format!("{} GB", gb));
                            } else {
                                return Ok(format!("{} MB", mb));
                            }
                        }
                    }
                }
            }
            
            return Ok("Unknown".to_string());
        } else {
            // For Linux systems
            let meminfo = self.execute_command("cat /proc/meminfo").await?;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let kb: u64 = parts[1].parse().unwrap_or(0);
                        let mb = kb / 1024;
                        let gb = mb / 1024;
                        
                        if gb > 0 {
                            return Ok(format!("{} GB", gb));
                        } else {
                            return Ok(format!("{} MB", mb));
                        }
                    }
                }
            }
            
            Ok("Unknown".to_string())
        }
    }

    async fn get_uptime(&self) -> Result<String> {
        if self.connection_type == "adb" {
            // For Android, try to use the uptime command first
            if let Ok(uptime_output) = self.execute_command("uptime").await {
                // Parse uptime output: "18:57:16 up  1:42,  0 users,  load average: 1.09, 1.06, 1.02"
                if let Some(up_part) = uptime_output.split("up").nth(1) {
                    if let Some(time_part) = up_part.split(',').next() {
                        let time_str = time_part.trim();
                        if !time_str.is_empty() {
                            return Ok(time_str.to_string());
                        }
                    }
                }
            }
            
            // Fallback to /proc/uptime
            if let Ok(uptime) = self.execute_command("cat /proc/uptime").await {
                let parts: Vec<&str> = uptime.split_whitespace().collect();
                
                if let Some(seconds_str) = parts.first() {
                    if let Ok(seconds) = seconds_str.parse::<f64>() {
                        let days = (seconds / 86400.0) as u32;
                        let hours = ((seconds % 86400.0) / 3600.0) as u32;
                        let minutes = ((seconds % 3600.0) / 60.0) as u32;
                        
                        if days > 0 {
                            return Ok(format!("{}d {}h {}m", days, hours, minutes));
                        } else if hours > 0 {
                            return Ok(format!("{}h {}m", hours, minutes));
                        } else {
                            return Ok(format!("{}m", minutes));
                        }
                    }
                }
            }
            
            return Ok("Unknown".to_string());
        } else {
            // For Linux systems
            let uptime = self.execute_command("cat /proc/uptime").await?;
            let parts: Vec<&str> = uptime.split_whitespace().collect();
            
            if let Some(seconds_str) = parts.first() {
                if let Ok(seconds) = seconds_str.parse::<f64>() {
                    let days = (seconds / 86400.0) as u32;
                    let hours = ((seconds % 86400.0) / 3600.0) as u32;
                    let minutes = ((seconds % 3600.0) / 60.0) as u32;
                    
                    if days > 0 {
                        return Ok(format!("{}d {}h {}m", days, hours, minutes));
                    } else if hours > 0 {
                        return Ok(format!("{}h {}m", hours, minutes));
                    } else {
                        return Ok(format!("{}m", minutes));
                    }
                }
            }
            
            Ok("Unknown".to_string())
        }
    }

    async fn get_os_info(&self) -> Result<String> {
        if self.connection_type == "adb" {
            // For Android, get build info
            let mut os_info = String::new();
            
            // Get Android version
            if let Ok(version) = self.execute_command("getprop ro.build.version.release").await {
                os_info.push_str(&format!("Android {}", version.trim()));
            }
            
            // Get build number
            if let Ok(build) = self.execute_command("getprop ro.build.display.id").await {
                if !build.trim().is_empty() {
                    if !os_info.is_empty() {
                        os_info.push_str(" ");
                    }
                    os_info.push_str(&format!("({})", build.trim()));
                }
            }
            
            if !os_info.is_empty() {
                return Ok(os_info);
            }
        } else {
            // For Linux systems, try to get OS info from various sources
            let sources = vec![
                ("cat /etc/os-release", "PRETTY_NAME"),
                ("cat /etc/lsb-release", "DISTRIB_DESCRIPTION"),
                ("uname -o", ""),
            ];

            for (command, key) in sources {
                if let Ok(output) = self.execute_command(command).await {
                    if key.is_empty() {
                        return Ok(output.trim().to_string());
                    } else {
                        for line in output.lines() {
                            if line.starts_with(key) {
                                let parts: Vec<&str> = line.split('=').collect();
                                if parts.len() > 1 {
                                    let value = parts[1].trim_matches('"');
                                    return Ok(value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok("Unknown".to_string())
    }
    
    // Batch parsing methods for better performance
    fn parse_chip_from_batch_results(&self, model: &str, compatible: &str, cpuinfo: &str) -> Option<String> {
        // Try device tree model first
        if !model.trim().is_empty() && model.trim() != "No model" {
            return Some(model.trim().to_string());
        }
        
        // Try compatible string
        if !compatible.trim().is_empty() && compatible.trim() != "No compatible" {
            if let Some(chip) = self.parse_chip_from_compatible(compatible.trim()) {
                return Some(chip);
            }
        }
        
        // Fallback to cpuinfo
        self.parse_chip_from_cpuinfo(cpuinfo)
    }
    
    fn parse_cpu_from_cpuinfo(&self, cpuinfo: &str) -> String {
        // Try to get model name first
        for line in cpuinfo.lines() {
            if line.starts_with("model name") || line.starts_with("Processor") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return parts[1].trim().to_string();
                }
            }
        }

        // For ARM devices, try to get CPU implementer info
        let mut implementer = None;
        let mut architecture = None;
        let mut processor_count = 0;
        
        for line in cpuinfo.lines() {
            if line.starts_with("processor") {
                processor_count += 1;
            } else if line.starts_with("CPU implementer") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    implementer = Some(parts[1].trim());
                }
            } else if line.starts_with("CPU architecture") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    architecture = Some(parts[1].trim());
                }
            }
        }

        // Build CPU description
        let mut cpu_desc = String::new();
        if let Some(impl_code) = implementer {
            match impl_code {
                "0x41" => cpu_desc.push_str("ARM"),
                "0x42" => cpu_desc.push_str("Broadcom"),
                "0x51" => cpu_desc.push_str("Qualcomm"),
                _ => cpu_desc.push_str("Unknown"),
            }
        }
        
        if let Some(arch) = architecture {
            if !cpu_desc.is_empty() {
                cpu_desc.push_str(" ");
            }
            cpu_desc.push_str(&format!("v{}", arch));
        }
        
        if processor_count > 0 {
            if !cpu_desc.is_empty() {
                cpu_desc.push_str(" ");
            }
            cpu_desc.push_str(&format!("({} cores)", processor_count));
        }
        
        if cpu_desc.is_empty() {
            cpu_desc = format!("{} cores", processor_count);
        }
        
        cpu_desc
    }
    
    fn parse_memory_from_meminfo(&self, meminfo: &str) -> String {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    let kb: u64 = parts[1].parse().unwrap_or(0);
                    let mb = kb / 1024;
                    let gb = mb / 1024;
                    
                    if gb > 0 {
                        return format!("{} GB", gb);
                    } else {
                        return format!("{} MB", mb);
                    }
                }
            }
        }
        "Unknown".to_string()
    }
    
    fn parse_uptime_from_proc(&self, uptime: &str) -> String {
        let parts: Vec<&str> = uptime.split_whitespace().collect();
        
        if let Some(seconds_str) = parts.first() {
            if let Ok(seconds) = seconds_str.parse::<f64>() {
                let days = (seconds / 86400.0) as u32;
                let hours = ((seconds % 86400.0) / 3600.0) as u32;
                let minutes = ((seconds % 3600.0) / 60.0) as u32;
                
                if days > 0 {
                    return format!("{}d {}h {}m", days, hours, minutes);
                } else if hours > 0 {
                    return format!("{}h {}m", hours, minutes);
                } else {
                    return format!("{}m", minutes);
                }
            }
        }
        "Unknown".to_string()
    }
    
    fn parse_os_from_release(&self, os_release: &str) -> String {
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() > 1 {
                    let value = parts[1].trim_matches('"');
                    return value.to_string();
                }
            }
        }
        "Unknown".to_string()
    }
}
