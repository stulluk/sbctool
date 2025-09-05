use anyhow::{Context, Result};
use clap::Parser;
use ssh2::Session;
use ssh_config::SSHConfig;
use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr};
use std::path::Path;
use std::process::Command;
use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice, ADBTcpDevice, ADBUSBDevice, search_adb_devices};

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

fn main() -> Result<()> {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Ssh { target } => {
			// Support `sbctool ssh help` style help
			if target == "help" || target == "--help" || target == "-h" {
				println!("Usage: sbctool ssh <user@host|alias>\n\nExamples:\n  sbctool ssh user@192.168.1.4\n  sbctool ssh khadas\n\nNotes:\n  - Aliases are resolved using 'ssh -G' when available; falls back to ~/.ssh/config and /etc/ssh/ssh_config.\n  - If user is omitted, tries ssh config, then $USER/LOGNAME.\n");
				return Ok(())
			}
			println!("Connecting to {} via SSH...", target);

			let home_dir = env::var("HOME").context("Failed to get HOME directory")?;
			let (user_from_target, host_from_target) = 
				if let Some((u, h)) = target.split_once('@') {
					(Some(u), h)
				} else {
					(None, target.as_str())
				};

			// First, try to resolve via `ssh -G <host>` to match OpenSSH behavior (can be disabled)
			let mut user_from_ssh_g: Option<String> = None;
			let mut host_from_ssh_g: Option<String> = None;
			let mut port_from_ssh_g: Option<u16> = None;
			let mut identity_from_ssh_g: Option<String> = None;
			let disable_ssh_g = env::var("SBCTOOL_DISABLE_SSH_G").ok().as_deref() == Some("1");
			let ssh_g_available = Command::new("ssh").arg("-G").arg("localhost").output().map(|o| o.status.success()).unwrap_or(false);
			if !disable_ssh_g && ssh_g_available {
				if let Ok(output) = Command::new("ssh").arg("-G").arg(host_from_target).output() {
					if output.status.success() {
						let stdout = String::from_utf8_lossy(&output.stdout);
						for line in stdout.lines() {
							if let Some(rest) = line.strip_prefix("user ") {
								if user_from_ssh_g.is_none() { user_from_ssh_g = Some(rest.trim().to_string()); }
							} else if let Some(rest) = line.strip_prefix("hostname ") {
								if host_from_ssh_g.is_none() { host_from_ssh_g = Some(rest.trim().to_string()); }
							} else if let Some(rest) = line.strip_prefix("port ") {
								if port_from_ssh_g.is_none() { port_from_ssh_g = rest.trim().parse::<u16>().ok(); }
							} else if let Some(rest) = line.strip_prefix("identityfile ") {
								if identity_from_ssh_g.is_none() { identity_from_ssh_g = Some(rest.trim().to_string()); }
							}
						}
					}
				}
			}

			// Build combined SSH client config: system-wide first, then user-specific
			let mut combined_cfg = String::new();
			if let Ok(system_cfg) = std::fs::read_to_string("/etc/ssh/ssh_config") {
				combined_cfg.push_str(&system_cfg);
				combined_cfg.push('\n');
			}
			let user_cfg_path = Path::new(&home_dir).join(".ssh/config");
			if let Ok(user_cfg) = std::fs::read_to_string(&user_cfg_path) {
				combined_cfg.push_str(&user_cfg);
			}

			let (user_from_cfg, host_from_cfg, port_from_cfg, identity_from_cfg) = if !combined_cfg.is_empty() {
				if let Ok(cfg) = SSHConfig::parse_str(&combined_cfg) {
					let m = cfg.query(host_from_target);
					let u = m.get("user").map(|s| s.to_string());
					let h = m.get("hostname").map(|s| s.to_string());
					let p = m.get("port").and_then(|p| p.parse::<u16>().ok());
					let i = m.get("identityfile").map(|s| s.to_string());
					(u, h, p, i)
				} else {
					(None, None, None, None)
				}
			} else {
				(None, None, None, None)
			};

			let user: String = if let Some(u) = user_from_target {
				u.to_string()
			} else if let Some(u) = user_from_ssh_g {
				u
			} else if let Some(u) = user_from_cfg {
				u
			} else if let Ok(u) = env::var("USER") {
				u
			} else if let Ok(u) = env::var("LOGNAME") {
				u
			} else {
				return Err(anyhow::anyhow!(
					"User not specified in target '{}' or in SSH config for host '{}'",
					target,
					host_from_target
				));
			};

			let host = host_from_ssh_g.as_deref().or(host_from_cfg.as_deref()).unwrap_or(host_from_target);
			let port = port_from_ssh_g.or(port_from_cfg).unwrap_or(22);

			if (user_from_target.is_none() && host_from_ssh_g.is_none() && host_from_cfg.is_none()) && !ssh_g_available {
				return Err(anyhow::anyhow!(
					"SSH alias '{}' cannot be resolved because system 'ssh' is not available. Please specify an IP address like user@1.2.3.4.",
					host_from_target
				));
			}

			let tcp = TcpStream::connect(format!("{}:{}", host, port))
				.with_context(|| format!("Failed to connect to {}:{}", host, port))?;
			let mut sess = Session::new()?;
			sess.set_tcp_stream(tcp);
			sess.handshake()?;

			let mut authenticated = false;

			if let Some(key_path_str) = identity_from_ssh_g.as_deref().or(identity_from_cfg.as_deref()) {
				let expanded_key_path = shellexpand::tilde(key_path_str).into_owned();
				let key_path = Path::new(&expanded_key_path);
				if sess
					.userauth_pubkey_file(&user, None, key_path, None)
					.is_ok()
				{
					println!(
						"Authenticated with public key (from SSH config: {})",
						key_path.display()
					);
					authenticated = true;
				}
			}

			if !authenticated {
				let private_key_path = Path::new(&home_dir).join(".ssh/id_rsa");
				if private_key_path.exists() {
					if sess
						.userauth_pubkey_file(&user, None, &private_key_path, None)
						.is_ok()
					{
						println!("Authenticated with default public key (~/.ssh/id_rsa).");
						authenticated = true;
					}
				}
			}

			if !authenticated {
				println!("Public key authentication failed, falling back to password.");
				print!("Password for {}@{}: ", user, host);
				io::stdout().flush()?;
				let mut password = String::new();
				io::stdin().read_line(&mut password)?;
				sess.userauth_password(&user, password.trim())?;
			}

			let mut channel = sess.channel_session()?;
			channel.exec("uname -a")?;

			let mut s = String::new();
			channel.read_to_string(&mut s)?;
			println!("{}", s);

			channel.wait_close()?;
			println!("Exit status: {}", channel.exit_status()?);
		}
		Commands::Adb { serial, extra } => {
			// handle `sbctool adb help`
			if extra.iter().any(|a| a == "help" || a == "--help" || a == "-h") {
				println!("Usage: sbctool adb [-s SERIAL]\n\nExamples:\n  sbctool adb\n  sbctool adb -s <usb-serial>\n  sbctool adb -s <ip>\n  sbctool adb -s <ip:port>\n\nBehavior:\n  - No -s: if exactly one USB device -> use USB; else list devices (server).\n  - -s ip:port: connect TCP direct to adbd.\n  - -s ip: default port 5555.\n  - -s usb-serial: use adb server to talk to that device.");
				return Ok(())
			}
			// If exactly one USB device and no -s provided, we could try USB direct here later.
			// Preferred UX:
			// - If -s is ip or ip:port -> direct TCP (default 5555 if missing port)
			// - If -s is not provided -> try server listing; if exactly one USB device, you could prefer it.
			if let Some(s) = serial.clone() {
				if let Ok(addr) = s.parse::<std::net::SocketAddr>() {
					println!("ADB TCP (direct): shell uname on {}", addr);
					let mut dev = ADBTcpDevice::new(addr.into())?;
					let mut out = Vec::new();
					dev.shell_command(&["uname", "-a"], &mut out)?;
					println!("{}", String::from_utf8_lossy(&out));
					return Ok(())
				}
				// ip without port: add 5555
				if s.parse::<std::net::Ipv4Addr>().is_ok() || s.parse::<std::net::Ipv6Addr>().is_ok() {
					let addr = format!("{}:5555", s).parse::<std::net::SocketAddr>()
						.map_err(|_| anyhow::anyhow!("invalid ip: {}", s))?;
					println!("ADB TCP (direct): shell uname on {}", addr);
					let mut dev = ADBTcpDevice::new(addr.into())?;
					let mut out = Vec::new();
					dev.shell_command(&["uname", "-a"], &mut out)?;
					println!("{}", String::from_utf8_lossy(&out));
					return Ok(())
				}
			}
			if let Some(s) = serial {
				println!("ADB server: shell uname on {}", s);
				let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
				let mut dev = ADBServerDevice::new(s.clone(), Some(server_addr));
				let mut out = Vec::new();
				dev.shell_command(&["uname", "-a"], &mut out)?;
				println!("{}", String::from_utf8_lossy(&out));
			} else {
				// Try direct USB first, fallback to ADB server
				if let Ok(Some((vid, pid))) = search_adb_devices() {
					println!("ADB USB (direct): found device {:04x}:{:04x}", vid, pid);
					
					// Try to create USB device, with retry on resource busy
					let mut dev = match ADBUSBDevice::new(vid, pid) {
						Ok(dev) => dev,
						Err(e) if e.to_string().contains("Resource busy") => {
							println!("USB device busy, trying to kill ADB server and retry...");
							let _ = Command::new("adb").arg("kill-server").output();
							std::thread::sleep(std::time::Duration::from_millis(500));
							ADBUSBDevice::new(vid, pid)?
						},
						Err(e) => return Err(e.into()),
					};
					
					let mut out = Vec::new();
					dev.shell_command(&["uname", "-a"], &mut out)?;
					println!("{}", String::from_utf8_lossy(&out));
				} else {
					println!("ADB server: listing devices and running uname on each");
					let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
					let mut server = ADBServer::new(server_addr);
					let devices = server.devices_long()?;
					if devices.is_empty() { println!("No ADB devices found."); return Ok(()) }
					for d in devices {
						let serial = d.identifier.clone();
						println!("\n--- {} ({:?}) ---", serial, d.state);
						let mut dev = ADBServerDevice::new(serial.clone(), Some(server_addr));
						let mut out = Vec::new();
						dev.shell_command(&["uname", "-a"], &mut out)?;
						println!("{}", String::from_utf8_lossy(&out));
					}
				}
			}
		}
	}

	Ok(())
}