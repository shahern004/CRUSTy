# CRUSTy - A Secure File Encryption Application

CRUSTy is a robust, memory-safe file encryption application built with Rust that implements AES-256-GCM authenticated encryption with a focus on security, usability, and extensibility. It provides a comprehensive solution for protecting sensitive data through strong cryptographic primitives while maintaining a user-friendly interface.

![CRUSTy Application](https://github.com/shahern004/CRUSTy/raw/main/screenshots/crusty_main.png)

## For Security Engineers

CRUSTy demonstrates several critical security engineering principles:

- **Defense-in-Depth**: Implements multiple security layers including authenticated encryption, key derivation functions, and optional hardware isolation
- **Cryptographic Agility**: Modular backend architecture allows for algorithm substitution and hardware acceleration
- **Secure Key Management**: Implements Shamir's Secret Sharing for key splitting, secure key storage, and recipient-specific key derivation
- **Memory Safety**: Built with Rust to eliminate entire classes of memory-related vulnerabilities (buffer overflows, use-after-free, etc.)
- **Authentication**: Uses AES-GCM's authenticated encryption to provide integrity verification and tamper detection
- **Secure Defaults**: Generates cryptographically secure random keys and nonces by default
- **Fail-Secure Design**: Implements proper error handling that defaults to secure states and prevents partial file corruption

The application serves as a practical example of implementing modern cryptographic protocols with proper nonce management, authenticated encryption, and key derivation techniques that align with NIST recommendations and cryptographic best practices.

## Technical Systems Overview

### Core Cryptographic Implementation

CRUSTy implements AES-256-GCM (Galois/Counter Mode) encryption with the following technical specifications:

- **Key Size**: 256-bit encryption keys
- **Nonce Management**: Unique 96-bit (12-byte) nonce generated for each encryption operation
- **Authentication Tag**: 128-bit authentication tag for integrity verification
- **Key Derivation**: HKDF with SHA-256 for recipient-specific key derivation
- **Random Number Generation**: OS-provided cryptographically secure random number generator (OsRng)

The encrypted file format includes:

1. 12-byte nonce
2. 4-byte encrypted data length
3. Encrypted data with authentication tag

For recipient-specific encryption, the format extends to:

1. 12-byte nonce
2. 2-byte recipient email length
3. Recipient email (variable length)
4. 4-byte encrypted data length
5. Encrypted data with authentication tag

### Architecture

CRUSTy follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────┐
│                  User Interface                      │
│  (GUI implementation using egui Rust framework)      │
└───────────────────────────┬─────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────┐
│               Operation Coordinator                  │
│  (Manages file operations, progress tracking, etc.)  │
└───────────────────────────┬─────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────┐
│               Backend Abstraction                    │
│  (Trait-based interface for encryption backends)     │
└─────────┬─────────────────────────────────┬─────────┘
          │                                 │
┌─────────▼─────────┐             ┌─────────▼─────────┐
│   Local Backend   │             │ Embedded Backend  │
│  (Software-based) │             │ (Hardware-based)  │
└─────────┬─────────┘             └─────────┬─────────┘
          │                                 │
┌─────────▼─────────┐             ┌─────────▼─────────┐
│  Encryption Core  │             │   STM32H5 Device  │
│  (AES-GCM impl.)  │             │ (HW acceleration) │
└───────────────────┘             └───────────────────┘
```

### Key Management System

CRUSTy implements a comprehensive key management system:

1. **Key Generation**: Cryptographically secure random key generation
2. **Key Storage**: Base64 encoding for human-readable storage
3. **Key Splitting**: Shamir's Secret Sharing (t,n) threshold scheme
   - Allows splitting keys into n shares where t shares are required for reconstruction
   - Supports 2-of-3 configuration for standard use cases
4. **Key Recovery**: Multiple recovery methods including mnemonic phrases and QR codes
5. **Recipient-Specific Keys**: Email-based key derivation using HKDF

### Backend Abstraction Layer

The backend abstraction layer enables flexible encryption implementations:

1. **Trait-Based Interface**: Common interface for all encryption backends
2. **Local Backend**: Software-based implementation using Rust's cryptographic libraries
3. **Embedded Backend**: Hardware-accelerated implementation for STM32H5 devices
4. **Factory Pattern**: Dynamic backend selection based on configuration

### Embedded System Integration

CRUSTy features a modular backend architecture that supports offloading cryptographic operations to embedded hardware, specifically designed for STM32H5 series microcontrollers with hardware cryptographic acceleration.

#### Components

1. **Backend Abstraction Layer**

   - Defines a common interface for all encryption backends
   - Allows seamless switching between local and embedded processing
   - Handles progress tracking and error management

2. **Local Backend**

   - Implements encryption/decryption using Rust's software libraries
   - Provides fallback functionality when embedded hardware is unavailable
   - Optimized for desktop CPU performance

3. **Embedded Backend**

   - Communicates with STM32H5 devices over USB, Serial, or Ethernet
   - Offloads cryptographic operations to hardware accelerators
   - Provides enhanced security through physical isolation

4. **STM32H5 Firmware**
   - Implements the CRUSTy protocol for secure communication
   - Utilizes hardware cryptographic accelerators (AES, PKA, HASH)
   - Provides secure key storage in isolated memory

For detailed information on the embedded systems integration, see the [Embedded Systems Documentation](Documentation/EMBEDDED_SYSTEMS.md).

### Error Handling and Logging

CRUSTy implements robust error handling:

1. **Custom Error Types**: Domain-specific error types with detailed messages
2. **Error Propagation**: Proper error propagation through the Result type
3. **Secure Error Handling**: Prevents information leakage in error messages
4. **Operation Logging**: Detailed logs of all encryption and decryption operations
5. **File Cleanup**: Automatic removal of partially written files on error

## Quick Start

For detailed installation and usage instructions, please see the [Usage Guide](Documentation/USAGE.md).

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
- **Recipient-Specific Encryption**:
  - Encrypt files for specific recipients using their email address
  - Derive unique encryption keys from recipient emails
  - Automatically detect recipient information during decryption
- **Progress Tracking**: Real-time progress indicators for encryption/decryption operations
- **Operation Logging**: Detailed logs of all encryption and decryption operations
- **Error Handling**: Clear error messages and prevention of corrupted output files

## Security Considerations

- CRUSTy uses AES-256-GCM, a secure authenticated encryption algorithm
- Each file is encrypted with a unique nonce to prevent replay attacks
- Email-based key derivation uses HKDF with SHA-256 for secure key generation
- The application has not been formally audited for security vulnerabilities
- For highly sensitive data, consider using established encryption tools

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This application is for learning and research purposes. While it uses strong encryption algorithms, it has not been audited for security vulnerabilities. Use at your own risk for sensitive data.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- The Rust community for providing excellent libraries and tools
- The egui framework for making GUI development in Rust accessible
