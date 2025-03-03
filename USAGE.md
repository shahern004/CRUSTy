# CRUSTy - Usage Guide

This document provides detailed instructions for installing and using the CRUSTy file encryption application.

![CRUSTy Application](https://github.com/shahern004/CRUSTy/raw/main/screenshots/crusty_main.png)

## Table of Contents

- [Installation](#installation)
  - [Pre-built Binaries](#pre-built-binaries)
  - [Building from Source](#building-from-source)
- [Basic Usage](#basic-usage)
  - [Encrypting Files](#encrypting-files)
  - [Decrypting Files](#decrypting-files)
  - [Managing Keys](#managing-keys)
- [Advanced Features](#advanced-features)
  - [Recipient-Specific Encryption](#recipient-specific-encryption)
  - [Embedded System Integration](#embedded-system-integration)
  - [Batch Processing](#batch-processing)
- [Troubleshooting](#troubleshooting)

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/shahern004/CRUSTy/releases) page.

### Building from Source

1. Ensure you have Rust and Cargo installed:

   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:

   ```
   git clone https://github.com/shahern004/CRUSTy.git
   cd CRUSTy
   ```

3. Build the application:

   ```
   cargo build --release
   ```

4. Run the application:
   ```
   cargo run --release
   ```

## Basic Usage

### Encrypting Files

1. Select "Single File" or "Multiple Files" mode
2. Click "Select File(s)" to choose the file(s) you want to encrypt
3. Select an output directory
4. Create a new encryption key or select an existing one
5. Click "Encrypt"

### Decrypting Files

1. Select "Single File" or "Multiple Files" mode
2. Click "Select File(s)" to choose the encrypted file(s)
3. Select an output directory
4. Select the encryption key that was used to encrypt the file(s)
5. Click "Decrypt"

### Managing Keys

1. Navigate to the "Keys" section by clicking the "🔑 Keys" button
2. Create new keys with custom names
3. Save keys to files for backup
4. Load keys from files

![Key Management](https://github.com/shahern004/CRUSTy/raw/main/screenshots/KeyMgmt.png)

## Advanced Features

### Recipient-Specific Encryption

CRUSTy allows you to encrypt files specifically for a recipient using their email address:

1. In the main screen, check the "Use recipient-specific encryption" box
2. Enter the recipient's email address
3. Select or create a master encryption key
4. Proceed with encryption as normal

When a file is encrypted for a specific recipient:

- The recipient's email is used to derive a unique encryption key
- The file can only be decrypted using both the master key and knowledge of the recipient's email
- The recipient information is stored within the encrypted file

To decrypt a recipient-specific file:

1. Load the master key that was used for encryption
2. CRUSTy will automatically detect if the file was encrypted for a specific recipient
3. The recipient's email will be displayed in the results after successful decryption

### Embedded System Integration

CRUSTy supports offloading cryptographic operations to an STM32H5 embedded device for enhanced performance and security:

1. Connect your STM32H573I-DK or compatible device to your computer
2. In the main screen, check the "Use embedded system for cryptographic operations" box
3. Select the appropriate connection type (USB, Serial, or Ethernet)
4. Enter the device ID or address
5. Add any required connection parameters in the "Advanced Connection Parameters" section
6. Click "Apply Configuration"

Once configured, all encryption and decryption operations will be performed on the embedded device instead of your computer.

#### Connection Types

- **USB**: Use for direct connection to the STM32H5 device. Enter the device ID (e.g., `VID:PID` format).
- **Serial/UART**: Use for serial connection. Enter the port name (e.g., `COM3` on Windows, `/dev/ttyUSB0` on Linux).
- **Ethernet**: Use for network connection. Enter the IP address and port (e.g., `192.168.1.100:8080`).

#### Advanced Parameters

Depending on your connection type, you may need to specify additional parameters:

- For Serial: `baud_rate`, `data_bits`, `parity`, `stop_bits`
- For USB: `interface`, `endpoint`
- For Ethernet: `timeout`, `keep_alive`

#### Benefits

- **Performance**: Hardware-accelerated encryption is faster for large files
- **Security**: Cryptographic operations are isolated from the main system
- **Power Efficiency**: Reduces CPU load on your computer

### Batch Processing

For encrypting or decrypting multiple files at once:

1. Select "Multiple Files" mode
2. Click "Select Files" to choose multiple files
3. Select an output directory
4. Select or create an encryption key
5. Click "Encrypt" or "Decrypt"

Progress for each file will be displayed during the operation.

## Troubleshooting

### Common Issues

**Error: "Destination file already exists"**

- CRUSTy will not overwrite existing files for safety
- Delete the existing file or choose a different output directory

**Error: "Authentication failed: The encryption key is incorrect or the file is corrupted"**

- Make sure you're using the same key that was used to encrypt the file
- If using recipient-specific encryption, ensure the correct master key is loaded

**Error: "Failed to decrypt: Wrong encryption key used"**

- Try a different encryption key
- If you've lost the key, the file cannot be recovered

**Error: "Embedded backend not implemented"**

- This error occurs when trying to use the embedded backend before it's properly configured
- Make sure you've clicked "Apply Configuration" after entering the device details
- Check that your device is properly connected and powered on

**Error: "Failed to connect to embedded device"**

- Verify that the device ID/address is correct
- Check physical connections (USB cable, network connection, etc.)
- Ensure the device is powered on and running the CRUSTy firmware
- Try a different connection type if available

**Error: "Communication error with embedded device"**

- The connection was established but was interrupted during operation
- Check for loose connections
- Ensure the device has stable power
- Try reducing the file size if the operation involves large files

### Getting Help

If you encounter issues not covered here, please:

1. Check the [GitHub Issues](https://github.com/shahern004/CRUSTy/issues) for similar problems
2. Open a new issue with details about your problem
