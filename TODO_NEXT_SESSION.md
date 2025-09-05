# TODO - Next Development Session

## 🎯 Current Status (September 5, 2025)
- ✅ **TUI Implementation**: Complete and tested
- ✅ **System Info Collection**: Working for both Linux SBC and Android
- ✅ **Chip Detection**: Rockchip RK3399 (Khadas) and Amlogic (OHM) tested
- ✅ **Log Collection**: logcat (Android) and journald/syslog (Linux) working
- ✅ **Documentation**: README and progress files updated

## 🚀 Next Session Priorities

### 1. **Enhanced System Monitoring** (High Priority)
- [ ] **Temperature Monitoring**
  - Linux: `/sys/class/thermal/thermal_zone*/temp`
  - Android: `cat /sys/class/thermal/thermal_zone*/temp`
  - Display in TUI with color coding (red for high temp)

- [ ] **CPU Usage Monitoring**
  - Linux: `/proc/stat` parsing for CPU utilization
  - Android: `top` command or `/proc/stat`
  - Real-time CPU usage percentage

- [ ] **Memory Usage Details**
  - Linux: `/proc/meminfo` detailed parsing
  - Android: `free` command with used/free/cached breakdown
  - Memory usage percentage and trends

- [ ] **Network Interface Monitoring**
  - Linux: `/proc/net/dev` parsing
  - Android: `cat /proc/net/dev`
  - Network traffic (bytes in/out, packets)

### 2. **TUI Enhancements** (Medium Priority)
- [ ] **Performance Metrics Panel**
  - Add third panel for real-time performance data
  - CPU usage graph (simple text-based)
  - Memory usage bar
  - Temperature indicators

- [ ] **Alert System**
  - High temperature warnings
  - High CPU usage alerts
  - Memory usage warnings
  - Network connectivity issues

- [ ] **Configuration System**
  - Config file support (`~/.sbctool/config.toml`)
  - Customizable refresh intervals
  - Alert thresholds configuration
  - Panel layout preferences

### 3. **Additional SBC Support** (Medium Priority)
- [ ] **More Chipset Detection**
  - Raspberry Pi (Broadcom BCM series)
  - Orange Pi (Allwinner H series)
  - Banana Pi (Allwinner A series)
  - Odroid (Samsung Exynos, Amlogic)
  - Pine64 (Allwinner A64, Rockchip)

- [ ] **Enhanced Device Tree Parsing**
  - More comprehensive compatible string parsing
  - Board-specific information extraction
  - Hardware revision detection

### 4. **Advanced Features** (Low Priority)
- [ ] **File System Monitoring**
  - Disk usage (`df` command)
  - Inode usage
  - Mount point information

- [ ] **Process Monitoring**
  - Top processes by CPU usage
  - Top processes by memory usage
  - Process count and system load

- [ ] **System Services Status**
  - Linux: `systemctl` status for key services
  - Android: Service status via `dumpsys`

- [ ] **Hardware Information**
  - USB devices connected
  - PCI devices (if applicable)
  - GPIO status (for embedded systems)

### 5. **User Experience Improvements** (Low Priority)
- [ ] **Data Export**
  - Export system info to JSON
  - Export logs to file
  - Historical data collection

- [ ] **Multiple Device Support**
  - Connect to multiple devices simultaneously
  - Tabbed interface for multiple connections
  - Device comparison view

- [ ] **Web Interface**
  - Optional web-based monitoring
  - Remote access via browser
  - REST API for system information

## 🧪 Testing Plan for Next Session

### Test Devices Available
- **Khadas Edge-V**: Rockchip RK3399, Ubuntu 22.04, 4GB RAM
- **OHM**: Amlogic, Android 14, 1.9GB RAM
- **Dahlia**: (if available) - Additional test device

### Test Scenarios
1. **Temperature Monitoring Test**
   ```bash
   # Test thermal zones
   ssh khadas "ls /sys/class/thermal/thermal_zone*/temp"
   adb -s 192.168.1.215 shell "ls /sys/class/thermal/thermal_zone*/temp"
   ```

2. **CPU Usage Test**
   ```bash
   # Test CPU stats
   ssh khadas "cat /proc/stat"
   adb -s 192.168.1.215 shell "cat /proc/stat"
   ```

3. **Network Monitoring Test**
   ```bash
   # Test network interfaces
   ssh khadas "cat /proc/net/dev"
   adb -s 192.168.1.215 shell "cat /proc/net/dev"
   ```

## 📁 Files to Focus On

### Primary Files
- `crates/sbctool/src/system_info.rs` - Add new monitoring functions
- `crates/sbctool/src/tui.rs` - Enhance TUI with new panels
- `crates/sbctool/src/main.rs` - Add new command options

### New Files to Create
- `crates/sbctool/src/config.rs` - Configuration management
- `crates/sbctool/src/alerts.rs` - Alert system
- `crates/sbctool/src/performance.rs` - Performance monitoring

## 🔧 Development Approach

### Phase 1: Temperature Monitoring
1. Add temperature collection functions
2. Update TUI to display temperature
3. Test with both devices
4. Add color coding for temperature levels

### Phase 2: CPU Usage Monitoring
1. Implement CPU usage calculation
2. Add real-time CPU usage display
3. Test performance impact
4. Optimize refresh intervals

### Phase 3: Enhanced TUI
1. Add third panel for performance metrics
2. Implement alert system
3. Add configuration support
4. Test user experience

## 📝 Notes for Next Session

### Current Working Directory
```bash
cd /home/stulluk/sbctool
```

### Quick Start Commands
```bash
# Check current status
git log --oneline -3
git status

# Build and test
cargo build
./target/debug/sbctool ssh khadas
./target/debug/sbctool adb -s 192.168.1.215

# Check progress
cat sbctool_progress_2025-09-05.txt
```

### Key Dependencies Already Added
- `ratatui = "0.28"` - TUI framework
- `crossterm = "0.28"` - Terminal control
- `tokio = { version = "1.0", features = ["full"] }` - Async runtime
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `chrono = { version = "0.4", features = ["serde"] }` - Date/time

### Potential New Dependencies
- `toml = "0.8"` - Configuration file support
- `sysinfo = "0.30"` - System information (if needed)
- `clap = { version = "4.0", features = ["derive"] }` - Enhanced CLI (if needed)

## 🎯 Success Criteria for Next Session

### Must Have
- [ ] Temperature monitoring working on both devices
- [ ] CPU usage monitoring implemented
- [ ] Enhanced TUI with performance panel
- [ ] All tests passing

### Nice to Have
- [ ] Alert system implemented
- [ ] Configuration file support
- [ ] Additional SBC chipset support
- [ ] Performance optimizations

### Stretch Goals
- [ ] Network monitoring
- [ ] File system monitoring
- [ ] Data export functionality
- [ ] Multiple device support

---
**Last Updated**: September 5, 2025
**Next Session Goal**: Enhanced system monitoring with temperature, CPU usage, and improved TUI
**Estimated Time**: 4-6 hours for core features
