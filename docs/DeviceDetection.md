# OBD-II Adapter Device Detection Guide

This guide helps you identify and configure your OBD-II adapter device path for use with faraday.

## Quick Detection Steps

### 1. Check Current Devices
Before connecting your adapter, list existing serial devices:
```bash
ls /dev/tty*
```

### 2. Connect Adapter
1. Plug OBD-II adapter into vehicle's diagnostic port
2. Connect USB cable from adapter to computer
3. Wait 2-3 seconds for device enumeration

### 3. Find New Device
List devices again to find the newly appeared device:
```bash
ls /dev/tty*
```

Common device paths:
- `/dev/ttyUSB0` - FTDI-based adapters (most common)
- `/dev/ttyACM0` - CDC ACM adapters
- `/dev/ttyS0` - Built-in serial ports (rare)

### 4. Verify with Kernel Messages
Check kernel logs for USB connection details:
```bash
dmesg | tail -10
```

Look for messages like:
```
usb 1-1: new full-speed USB device
ftdi_sio 1-1:1.0: FTDI USB Serial Device converter detected
usb 1-1: FTDI USB Serial Device converter now attached to ttyUSB0
```

## Using Non-Default Device Path

If your adapter appears as something other than `/dev/ttyUSB0`, specify it with the `--adapter` flag:

```bash
# For /dev/ttyACM0
./target/release/faraday --adapter /dev/ttyACM0 read-dtc

# For /dev/ttyUSB1 (if multiple USB serial devices)
./target/release/faraday --adapter /dev/ttyUSB1 read-dtc
```

You can also set the environment variable:
```bash
export FARADAY_ADAPTER=/dev/ttyACM0
./target/release/faraday read-dtc
```

## Common Issues and Solutions

### Permission Denied
If you get "Permission denied" error:

**Option 1: Add user to dialout group (recommended)**
```bash
sudo usermod -a -G dialout $USER
# Log out and log back in
```

**Option 2: Use sudo (temporary)**
```bash
sudo ./target/release/faraday read-dtc
```

### Device Not Found
If your adapter doesn't appear:

1. **Check USB connection**: Try different USB ports
2. **Verify adapter power**: LED indicators should be active
3. **Check dmesg for errors**: `dmesg | grep -i usb`
4. **Try different cable**: USB data cables vs power-only cables
5. **Check adapter compatibility**: Ensure it's ELM327-compatible

### Multiple USB Serial Devices
If multiple `/dev/ttyUSB*` devices exist:

1. **Unplug all unnecessary USB serial devices**
2. **Use udev rules** to create consistent device names
3. **Check device vendor/product ID**:
   ```bash
   lsusb
   udevadm info --name=/dev/ttyUSB0
   ```

### Bluetooth Adapters
For Bluetooth OBD-II adapters, the device path will be different:
```bash
# Pair device first, then find the rfcomm device
rfcomm bind /dev/rfcomm0 XX:XX:XX:XX:XX:XX
./target/release/faraday --adapter /dev/rfcomm0 read-dtc
```

## Testing Device Communication

Once you've identified the correct device path, test basic communication:

```bash
# Test with verbose output
./target/release/faraday -v --adapter /dev/ttyUSB0 vin

# If successful, proceed with DTC reading
./target/release/faraday --adapter /dev/ttyUSB0 read-dtc
```

## Adapter Configuration Requirements

Before using faraday, ensure your adapter is configured for:
- **HS-CAN mode**: 500 kbps
- **11-bit CAN IDs**: Standard OBD-II format
- **Auto protocol selection**: Or manually set to ISO 15765-4 (CAN 11/500)

Most FORScan-compatible adapters can be configured using FORScan software before switching to faraday.