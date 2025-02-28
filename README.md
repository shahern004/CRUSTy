# CRUSTy - A Secure File Encryption Application

CRUSTy is a desktop application built with Rust that provides secure file encryption and decryption using AES-256-GCM encryption.

![CRUSTy Application](https://github.com/shahern004/CRUSTy/raw/main/screenshots/crusty_main.png)

## Features

- **Strong Encryption**: Uses AES-256-GCM for secure, authenticated encryption
- **User-Friendly Interface**: Simple and intuitive GUI built with egui
- **Flexible Operation Modes**:
  - Single file encryption/decryption
  - Batch processing for multiple files
- **Key Management**:
  - Generate new encryption keys
  - Save keys to files for later use
  - Load keys from files
  - Manage multiple keys
- **Progress Tracking**: Real-time progress indicators for encryption/decryption operations
- **Operation Logging**: Detailed logs of all encryption and decryption operations
- **Error Handling**: Clear error messages and prevention of corrupted output files

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

## Usage

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

1. Navigate to the "Keys" section
2. Create new keys with custom names
3. Save keys to files for backup
4. Load keys from files

## Security Considerations

- CRUSTy uses AES-256-GCM, a secure authenticated encryption algorithm
- Each file is encrypted with a unique nonce to prevent replay attacks
- The application has not been formally audited for security vulnerabilities
- For highly sensitive data, consider using established encryption tools

## Technical Details

- Built with Rust for memory safety and performance
- GUI implemented with egui framework
- Uses the aes-gcm crate for encryption operations
- Secure random key generation via the rand crate
- Native file dialogs provided by rfd

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This application is for learning and research purposes. While it uses strong encryption algorithms, it has not been audited for security vulnerabilities. Use at your own risk for sensitive data.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- The Rust community for providing excellent libraries and tools
- The egui framework for making GUI development in Rust accessible 