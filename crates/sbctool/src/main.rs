use anyhow::Result;
use clap::Parser;

mod tui;
mod system_info;
mod log_collector;

use tui::{TuiApp, setup_terminal, restore_terminal};
use system_info::SystemInfoCollector;

#[derive(Parser)]
#[command(name = "sbctool")]
#[command(author = "Your Name <you@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "A CLI tool to collect information from various Single Board Computers (SBCs).", long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Parser)]
enum Commands {
	/// Connect to an SBC using SSH
	Ssh {
		/// The user@host or ssh_config alias to connect to (e.g., root@192.168.1.4, my-sbc)
		#[arg(value_name = "TARGET")]
		target: String,
	},
	/// Connect to an SBC using ADB
	Adb {
		/// The device serial to connect to (e.g., 192.168.1.15:5555)
		#[arg(short, long)]
		serial: Option<String>,
		/// Extra args, e.g. allowing `sbctool adb help`
		#[arg(value_name = "ARGS", trailing_var_arg = true)]
		extra: Vec<String>,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Ssh { target } => {
			// Support `sbctool ssh help` style help
			if target == "help" || target == "--help" || target == "-h" {
				println!("Usage: sbctool ssh <user@host|alias>\n\nExamples:\n  sbctool ssh user@192.168.1.4\n  sbctool ssh khadas\n\nNotes:\n  - Aliases are resolved using 'ssh -G' when available; falls back to ~/.ssh/config and /etc/ssh/ssh_config.\n  - If user is omitted, tries ssh config, then $USER/LOGNAME.\n  - Launches TUI interface for real-time monitoring.\n");
				return Ok(())
			}
			
			// Launch TUI for SSH connection
			launch_ssh_tui(target).await?;
		}
		Commands::Adb { serial, extra } => {
			// handle `sbctool adb help`
			if extra.iter().any(|a| a == "help" || a == "--help" || a == "-h") {
				println!("Usage: sbctool adb [-s SERIAL]\n\nExamples:\n  sbctool adb\n  sbctool adb -s <usb-serial>\n  sbctool adb -s <ip>\n  sbctool adb -s <ip:port>\n\nBehavior:\n  - No -s: if exactly one USB device -> use USB; else list devices (server).\n  - -s ip:port: connect TCP direct to adbd.\n  - -s ip: default port 5555.\n  - -s usb-serial: use adb server to talk to that device.\n  - Launches TUI interface for real-time monitoring.");
				return Ok(())
			}
			
			// Launch TUI for ADB connection
			launch_adb_tui(serial.clone()).await?;
		}
	}

	Ok(())
}

async fn launch_ssh_tui(target: &str) -> Result<()> {
	println!("Connecting to {} via SSH...", target);

	// Setup terminal
	let mut terminal = setup_terminal()?;
	
	// Create TUI app
	let mut app = TuiApp::new();
	
	// Add initial log entry
	app.add_log(tui::LogEntry {
		timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
		level: "INFO".to_string(),
		message: format!("Connecting to {} via SSH", target),
	});
	
	// Create system info collector
	let collector = SystemInfoCollector::new("ssh", target);
	
	// Spawn async task to collect system info
	let app_clone = app.system_info.clone();
	let log_sender_clone = app.logs.clone();
	tokio::spawn(async move {
		// Add info log
		let info_log = tui::LogEntry {
			timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
			level: "INFO".to_string(),
			message: "Starting system info collection...".to_string(),
		};
		if let Ok(mut logs) = log_sender_clone.lock() {
			logs.push(info_log);
		}
		
		match collector.collect_system_info().await {
			Ok(info) => {
				if let Ok(mut system_info) = app_clone.lock() {
					*system_info = Some(info);
				}
				// Add success log
				let success_log = tui::LogEntry {
					timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
					level: "INFO".to_string(),
					message: "System info collected successfully".to_string(),
				};
				if let Ok(mut logs) = log_sender_clone.lock() {
					logs.push(success_log);
				}
			}
			Err(e) => {
				// Add error log
				let error_log = tui::LogEntry {
					timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
					level: "ERROR".to_string(),
					message: format!("Failed to collect system info: {}", e),
				};
				if let Ok(mut logs) = log_sender_clone.lock() {
					logs.push(error_log);
				}
			}
		}
	});
	
	// Spawn async task to collect logs
	let log_collector = log_collector::LogCollector::new("ssh", target, false);
	let log_sender = app.logs.clone();
	tokio::spawn(async move {
		log_collector.start_log_collection(log_sender).await;
	});
	
	// Run TUI
	app.run(&mut terminal)?;
	
	// Restore terminal
	restore_terminal(&mut terminal)?;
	
	Ok(())
}

async fn launch_adb_tui(serial: Option<String>) -> Result<()> {
	let target = if let Some(s) = &serial {
		s.clone()
	} else {
		"auto".to_string()
	};
	
	println!("Connecting to ADB device: {}", target);

	// Setup terminal
	let mut terminal = setup_terminal()?;
	
	// Create TUI app
	let mut app = TuiApp::new();
	
	// Add initial log entry
	app.add_log(tui::LogEntry {
		timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
		level: "INFO".to_string(),
		message: format!("Connecting to ADB device: {}", target),
	});
	
	// Create system info collector
	let collector = SystemInfoCollector::new("adb", &target);
	
	// Spawn async task to collect system info
	let app_clone = app.system_info.clone();
	let log_sender_clone = app.logs.clone();
	tokio::spawn(async move {
		match collector.collect_system_info().await {
			Ok(info) => {
				if let Ok(mut system_info) = app_clone.lock() {
					*system_info = Some(info);
				}
			}
			Err(e) => {
				// Add error log
				let error_log = tui::LogEntry {
					timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
					level: "ERROR".to_string(),
					message: format!("Failed to collect system info: {}", e),
				};
				if let Ok(mut logs) = log_sender_clone.lock() {
					logs.push(error_log);
				}
			}
		}
	});
	
	// Spawn async task to collect logs (Android logcat)
	let log_collector = log_collector::LogCollector::new("adb", &target, true);
	let log_sender = app.logs.clone();
	tokio::spawn(async move {
		log_collector.start_log_collection(log_sender).await;
	});
	
	// Run TUI
	app.run(&mut terminal)?;
	
	// Restore terminal
	restore_terminal(&mut terminal)?;
	
	Ok(())
}