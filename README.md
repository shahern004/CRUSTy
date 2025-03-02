# CRUSTy - A Secure File Encryption Application

CRUSTy is a desktop application built with Rust that provides secure file encryption and decryption using AES-256-GCM encryption.

![CRUSTy Application](https://github.com/shahern004/CRUSTy/raw/main/screenshots/crusty_main.png)

## Quick Start

For detailed installation and usage instructions, please see the [Usage Guide](USAGE.md).

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

## Technical Architecture

### Email-Based Key Derivation

CRUSTy implements a secure method for recipient-specific encryption:

1. The user provides a recipient's email address
2. A cryptographic hash function (SHA-256) is used to derive material from the normalized email
3. The HKDF (HMAC-based Key Derivation Function) combines this material with the master key
4. The resulting derived key is used for encryption/decryption
5. The recipient's email is stored in the encrypted file for reference during decryption

This approach ensures that files can only be decrypted with both the master key and knowledge of the recipient's email address, adding an additional layer of security.

### TODO: C/C++ Integration Architecture

CRUSTy is designed with a modular architecture that will allow for integration with C/C++ front-ends in the future. This section outlines the planned architecture for such integration.

#### Overview

The integration will follow a layered architecture:

```
┌─────────────────────┐
│  C/C++ Front-end    │
│  (GUI or CLI)       │
└─────────┬───────────┘
          │
          │ FFI Boundary
          ▼
┌─────────────────────┐
│  CRUSTy C API       │ ← Shared library (.dll, .so, .dylib)
│  (FFI Layer)        │
└─────────┬───────────┘
          │
          │ Internal Rust API
          ▼
┌─────────────────────┐
│  CRUSTy Core        │
│  (Rust Engine)      │
└─────────────────────┘
```

#### Components

1. **CRUSTy Core (Rust Engine)**

   - Contains all encryption/decryption logic
   - Manages keys and cryptographic operations
   - Handles file I/O and progress tracking
   - Implements error handling and logging

2. **CRUSTy C API (FFI Layer)**

   - Exposes a C-compatible API using Rust's FFI capabilities
   - Provides simple function calls for all core operations
   - Handles memory management between Rust and C/C++
   - Translates between Rust types and C-compatible types

3. **C/C++ Front-end**
   - Implements the user interface (GUI or CLI)
   - Calls the CRUSTy C API functions
   - Handles application-specific logic
   - Can be implemented in any C/C++ framework

#### Implementation Plan

##### 1. Refactoring the Core Engine

The current encryption/decryption functionality will be refactored into a standalone library with:

- Clear separation between core logic and UI
- Well-defined API boundaries
- Proper error handling across language boundaries

##### 2. Creating the FFI Layer

A new module will be created to expose C-compatible functions:

```rust
// Example of the FFI interface (crusty_ffi.rs)
#[no_mangle]
pub extern "C" fn crusty_encrypt_file(
    input_path: *const c_char,
    output_path: *const c_char,
    key_data: *const u8,
    key_len: size_t,
    progress_callback: extern "C" fn(f32) -> (),
) -> i32 {
    // Convert C types to Rust types
    // Call the core encryption function
    // Return status code
}
```

##### 3. Building as a Shared Library

The FFI layer will be compiled into a shared library:

- Windows: `crusty_core.dll`
- Linux: `libcrusty_core.so`
- macOS: `libcrusty_core.dylib`

##### 4. C/C++ Header File

A C header file will be provided for C/C++ applications:

```c
// crusty.h
#ifndef CRUSTY_H
#define CRUSTY_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Error codes
#define CRUSTY_SUCCESS 0
#define CRUSTY_ERROR_INVALID_ARGS -1
#define CRUSTY_ERROR_IO -2
#define CRUSTY_ERROR_CRYPTO -3

// Key management
int crusty_generate_key(uint8_t* key_buffer, size_t buffer_size);
int crusty_load_key_from_file(const char* path, uint8_t* key_buffer, size_t buffer_size);
int crusty_save_key_to_file(const char* path, const uint8_t* key_data, size_t key_len);

// File operations
typedef void (*progress_callback_t)(float progress);

int crusty_encrypt_file(const char* input_path, const char* output_path,
                        const uint8_t* key_data, size_t key_len,
                        progress_callback_t progress_callback);

int crusty_decrypt_file(const char* input_path, const char* output_path,
                        const uint8_t* key_data, size_t key_len,
                        progress_callback_t progress_callback);

// Batch operations
int crusty_encrypt_files(const char** input_paths, size_t num_files,
                         const char* output_dir, const uint8_t* key_data,
                         size_t key_len, progress_callback_t progress_callback);

int crusty_decrypt_files(const char** input_paths, size_t num_files,
                         const char* output_dir, const uint8_t* key_data,
                         size_t key_len, progress_callback_t progress_callback);

#ifdef __cplusplus
}
#endif

#endif // CRUSTY_H
```

#### Benefits of This Architecture

1. **Language Flexibility**: Allows for front-ends in C, C++, or any language with C FFI support
2. **Performance**: Maintains the performance benefits of Rust's core encryption engine
3. **Security**: Core cryptographic operations remain in memory-safe Rust code
4. **Modularity**: Clear separation of concerns between UI and encryption logic
5. **Reusability**: The core engine can be used in multiple applications

## Security Considerations

- CRUSTy uses AES-256-GCM, a secure authenticated encryption algorithm
- Each file is encrypted with a unique nonce to prevent replay attacks
- Email-based key derivation uses HKDF with SHA-256 for secure key generation
- The application has not been formally audited for security vulnerabilities
- For highly sensitive data, consider using established encryption tools

## Technical Details

- Built with Rust for memory safety and performance
- GUI implemented with egui framework
- Uses the aes-gcm crate for encryption operations
- Secure random key generation via the rand crate
- HKDF implementation for email-based key derivation
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
