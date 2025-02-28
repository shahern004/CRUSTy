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

## TODO: C/C++ Integration Architecture

CRUSTy is designed with a modular architecture that will allow for integration with C/C++ front-ends in the future. This section outlines the planned architecture for such integration.

### Overview

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

### Components

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

### Implementation Plan

#### 1. Refactoring the Core Engine

The current encryption/decryption functionality will be refactored into a standalone library with:
- Clear separation between core logic and UI
- Well-defined API boundaries
- Proper error handling across language boundaries

#### 2. Creating the FFI Layer

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

#### 3. Building as a Shared Library

The FFI layer will be compiled into a shared library:
- Windows: `crusty_core.dll`
- Linux: `libcrusty_core.so`
- macOS: `libcrusty_core.dylib`

#### 4. C/C++ Header File

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

### Usage from C/C++

Example of using the library from C++:

```cpp
#include "crusty.h"
#include <iostream>
#include <vector>

// Progress callback function
void progress_update(float progress) {
    std::cout << "Progress: " << (progress * 100.0f) << "%" << std::endl;
}

int main() {
    // Generate a new key
    std::vector<uint8_t> key(32); // 256-bit key
    if (crusty_generate_key(key.data(), key.size()) != CRUSTY_SUCCESS) {
        std::cerr << "Failed to generate key" << std::endl;
        return 1;
    }
    
    // Encrypt a file
    int result = crusty_encrypt_file(
        "document.pdf",
        "document.pdf.encrypted",
        key.data(),
        key.size(),
        progress_update
    );
    
    if (result == CRUSTY_SUCCESS) {
        std::cout << "Encryption successful!" << std::endl;
    } else {
        std::cerr << "Encryption failed with error code: " << result << std::endl;
    }
    
    return 0;
}
```

### Benefits of This Architecture

1. **Language Flexibility**: Allows for front-ends in C, C++, or any language with C FFI support
2. **Performance**: Maintains the performance benefits of Rust's core encryption engine
3. **Security**: Core cryptographic operations remain in memory-safe Rust code
4. **Modularity**: Clear separation of concerns between UI and encryption logic
5. **Reusability**: The core engine can be used in multiple applications

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
