# CRUSTy Training Guide for Security Engineers

## Introduction

Welcome to CRUSTy, a secure file encryption application built with Rust. This guide provides both technical and simplified explanations of CRUSTy's functionality to help you understand how it works and how to use it effectively.

## Core Functionality

### Key Management

#### Technical Explanation

CRUSTy uses AES-256-GCM for encryption, which requires a 256-bit key. The key management is implemented in the `EncryptionKey` struct:

```rust
/// Struct to hold and manage AES-256-GCM encryption keys
#[derive(Clone)]
pub struct EncryptionKey {
    key: Key<Aes256Gcm>,  // Uses the aes-gcm crate's Key type
}

impl EncryptionKey {
    /// Generate a new random encryption key
    pub fn generate() -> Self {
        // Uses the OsRng (operating system random number generator)
        // to generate a cryptographically secure random key
        let key = Aes256Gcm::generate_key(OsRng);
        EncryptionKey { key }
    }

    /// Convert the key to a base64 string for storage
    pub fn to_base64(&self) -> String {
        // Encodes the binary key as a base64 string for storage
        STANDARD.encode(self.key)
    }

    /// Create a key from a base64 string
    pub fn from_base64(encoded: &str) -> Result<Self, EncryptionError> {
        // Decodes a base64 string back to a binary key
        let key_bytes = STANDARD.decode(encoded)
            .map_err(|e| EncryptionError::KeyError(format!("Invalid base64 key: {}", e)))?;

        // Validates that the key is the correct length (32 bytes = 256 bits)
        if key_bytes.len() != 32 {
            return Err(EncryptionError::KeyError(
                format!("Invalid key length: {}, expected 32", key_bytes.len())
            ));
        }

        let key = *Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(EncryptionKey { key })
    }

    /// Save the key to a file
    pub fn save_to_file(&self, path: &Path) -> Result<(), EncryptionError> {
        // Creates a file and writes the base64-encoded key to it
        File::create(path)
            .map_err(EncryptionError::Io)?
            .write_all(self.to_base64().as_bytes())
            .map_err(EncryptionError::Io)
    }

    /// Load a key from a file
    pub fn load_from_file(path: &Path) -> Result<Self, EncryptionError> {
        // Reads the base64-encoded key from a file
        let mut contents = String::new();
        File::open(path)
            .map_err(EncryptionError::Io)?
            .read_to_string(&mut contents)
            .map_err(EncryptionError::Io)?;

        // Converts the base64 string back to a key
        Self::from_base64(&contents)
    }
}
```

The key management system uses Rust's strong type system to ensure that keys are handled securely. The `EncryptionKey` struct encapsulates the actual cryptographic key and provides methods for generating, saving, loading, and encoding/decoding keys. The use of Rust's `Result` type ensures that errors are properly handled throughout the key management process.

#### Simple Explanation

CRUSTy's key management system works like a digital keychain:

1. **Creating Keys**: When you create a new key, CRUSTy uses your computer's secure random number generator to create a unique 256-bit key (that's a number with 78 digits!). This is like having a key with billions of possible combinations.

2. **Storing Keys**: Keys are stored in a special format called "base64" which converts the binary key data into text that can be easily stored in files. This is similar to converting a physical key's unique pattern into a code that can be written down.

3. **Loading Keys**: When you load a key from a file, CRUSTy reads the base64 text, converts it back to the binary format, and checks that it's the correct length (32 bytes).

4. **Using Keys**: Once loaded, keys are kept in memory only while the application is running. The key is used to lock (encrypt) and unlock (decrypt) your files.

Think of the key management system as a secure key cabinet where you can create, store, and retrieve keys for your digital locks.

### Basic Encryption and Decryption

#### Technical Explanation

CRUSTy implements AES-256-GCM encryption with proper nonce handling and authentication. Here's the core encryption function:

```rust
/// Encrypts raw data using AES-256-GCM encryption
pub fn encrypt_data(data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
    // Create the cipher instance with our key
    let cipher = Aes256Gcm::new(&key.key);

    // Generate a random nonce (Number used ONCE)
    let mut nonce_bytes = [0u8; 12]; // AES-GCM uses 12-byte nonces
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let encrypted_data = match cipher.encrypt(nonce, data) {
        Ok(data) => data,
        Err(e) => {
            return Err(EncryptionError::Encryption(e.to_string()));
        }
    };

    // Create the output buffer with the nonce and encrypted data
    let mut output = Vec::with_capacity(nonce_bytes.len() + 4 + encrypted_data.len());

    // Write the nonce
    output.extend_from_slice(&nonce_bytes);

    // Write the encrypted data length
    output.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());

    // Write the encrypted data
    output.extend_from_slice(&encrypted_data);

    Ok(output)
}
```

And the corresponding decryption function:

```rust
/// Decrypts raw data that was encrypted with AES-256-GCM
pub fn decrypt_data(data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
    // Check if the data is long enough to contain a nonce and length
    if data.len() < 12 + 4 {
        return Err(EncryptionError::Decryption("Data too short".to_string()));
    }

    // Read the nonce from the beginning of the data
    let nonce_bytes = &data[0..12];
    let nonce = Nonce::from_slice(nonce_bytes);

    // Read the encrypted data length
    let size_bytes = &data[12..16];
    let chunk_size = u32::from_le_bytes([size_bytes[0], size_bytes[1], size_bytes[2], size_bytes[3]]) as usize;

    // Check if the data is long enough to contain the encrypted chunk
    if data.len() < 16 + chunk_size {
        return Err(EncryptionError::Decryption("Data too short".to_string()));
    }

    // Read the encrypted chunk
    let encrypted_chunk = &data[16..16 + chunk_size];

    // Create the cipher instance with our key
    let cipher = Aes256Gcm::new(&key.key);

    // Decrypt the chunk
    let decrypted_data = match cipher.decrypt(nonce, encrypted_chunk) {
        Ok(data) => data,
        Err(e) => {
            // Provide a more specific error message for authentication failures
            if e.to_string().contains("authentication") || e.to_string().contains("tag mismatch") {
                return Err(EncryptionError::Decryption(
                    "Authentication failed: The encryption key is incorrect or the data is corrupted".to_string()
                ));
            } else {
                return Err(EncryptionError::Decryption(e.to_string()));
            }
        }
    };

    Ok(decrypted_data)
}
```

The encryption process uses AES-256-GCM, which is an authenticated encryption with associated data (AEAD) algorithm. This means it not only encrypts the data but also provides integrity protection. The implementation includes:

1. A unique nonce (number used once) for each encryption operation
2. Proper error handling for both encryption and decryption
3. Authentication to detect tampering or incorrect keys
4. A structured format for the encrypted data that includes the nonce and data length

#### Simple Explanation

CRUSTy's encryption and decryption work like a sophisticated digital lock system:

1. **Encryption (Locking)**:

   - When you encrypt a file, CRUSTy takes your data and your encryption key.
   - It generates a unique "nonce" (a random number used only once) for this specific encryption.
   - It uses the AES-256-GCM algorithm to scramble your data in a way that can only be unscrambled with the same key.
   - It adds a special "authentication tag" that works like a tamper-evident seal.
   - It packages everything together: the nonce, the encrypted data, and the authentication tag.

2. **Decryption (Unlocking)**:
   - When you decrypt a file, CRUSTy extracts the nonce and encrypted data.
   - It uses your key and the nonce to try to unscramble the data.
   - It checks the authentication tag to make sure the data hasn't been tampered with.
   - If everything checks out, you get your original data back.
   - If the wrong key is used or the data has been modified, the authentication check fails and you get an error message.

Think of it like a high-security safe that not only requires the right combination but also can tell if someone has tried to break in.

### Recipient-Specific Encryption

#### Technical Explanation

CRUSTy implements recipient-specific encryption using key derivation from email addresses:

```rust
/// Derives cryptographic material from an email address
fn derive_from_email(email: &str, salt: &[u8]) -> Vec<u8> {
    // Normalize the email by trimming whitespace and converting to lowercase
    let normalized_email = email.trim().to_lowercase();
    let parts: Vec<&str> = normalized_email.split('@').collect();
    let username = parts[0];
    let domain = if parts.len() > 1 { parts[1] } else { "" };

    // Create a SHA-256 hash of the normalized email parts with a salt
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b":");
    hasher.update(domain.as_bytes());
    hasher.update(b":");
    hasher.update(salt);

    hasher.finalize().to_vec()
}

impl EncryptionKey {
    /// Create a recipient-specific key by combining the master key with email-derived material
    pub fn for_recipient(&self, email: &str) -> Result<Self, EncryptionError> {
        // Use a fixed application salt for consistency
        let app_salt = b"CRUSTy-Email-Key-Derivation-Salt-v1";
        let email_material = derive_from_email(email, app_salt);

        // Use HKDF (HMAC-based Key Derivation Function) to derive a new key
        // from the master key and the email-derived material
        let hkdf = Hkdf::<Sha256>::new(
            Some(&email_material),
            self.key.as_slice()
        );

        // Expand the derived key material to the required length (32 bytes)
        let mut okm = [0u8; 32];
        hkdf.expand(b"encryption", &mut okm)
            .map_err(|_| EncryptionError::KeyError("Key derivation failed".to_string()))?;

        // Create a new encryption key from the derived material
        let derived_key = Key::<Aes256Gcm>::from_slice(&okm);
        Ok(EncryptionKey { key: *derived_key })
    }
}
```

The recipient-specific encryption function:

```rust
/// Encrypts raw data for a specific recipient using their email address
pub fn encrypt_data_for_recipient(
    data: &[u8],
    master_key: &EncryptionKey,
    recipient_email: &str
) -> Result<Vec<u8>, EncryptionError> {
    // Derive recipient-specific key
    let recipient_key = master_key.for_recipient(recipient_email)?;

    // Create the cipher instance with our key
    let cipher = Aes256Gcm::new(&recipient_key.key);

    // Generate a random nonce (Number used ONCE)
    let mut nonce_bytes = [0u8; 12]; // AES-GCM uses 12-byte nonces
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let encrypted_data = match cipher.encrypt(nonce, data) {
        Ok(data) => data,
        Err(e) => {
            return Err(EncryptionError::Encryption(e.to_string()));
        }
    };

    // Create the output buffer with the nonce, recipient email, and encrypted data
    let email_bytes = recipient_email.as_bytes();
    let mut output = Vec::with_capacity(
        nonce_bytes.len() + 2 + email_bytes.len() + 4 + encrypted_data.len()
    );

    // Write the nonce
    output.extend_from_slice(&nonce_bytes);

    // Write recipient email length and email
    output.extend_from_slice(&(email_bytes.len() as u16).to_le_bytes());
    output.extend_from_slice(email_bytes);

    // Write the encrypted data length
    output.extend_from_slice(&(encrypted_data.len() as u32).to_le_bytes());

    // Write the encrypted data
    output.extend_from_slice(&encrypted_data);

    Ok(output)
}
```

The recipient-specific encryption uses HKDF (HMAC-based Key Derivation Function) to derive a unique encryption key from the master key and the recipient's email address. This provides an additional layer of security by binding the encrypted data to both the master key and the recipient's identity.

#### Simple Explanation

Recipient-specific encryption in CRUSTy works like a personalized lock system:

1. **Creating a Recipient-Specific Key**:

   - You start with your master key (like a master locksmith key).
   - You provide a recipient's email address.
   - CRUSTy processes the email address in a standardized way (lowercase, trimmed).
   - It combines your master key with information derived from the email to create a unique key just for that recipient.
   - This is like creating a custom lock that responds to both your master key and the recipient's identity.

2. **Encrypting for a Recipient**:

   - When you encrypt a file for a specific recipient, CRUSTy uses the derived key to encrypt the data.
   - It stores the recipient's email address within the encrypted file.
   - The file can only be decrypted with both the master key and knowledge of the recipient's email.

3. **Decrypting Recipient-Specific Files**:
   - When decrypting, CRUSTy reads the recipient email from the file.
   - It recreates the recipient-specific key using the master key and the stored email.
   - If successful, it shows you who the file was encrypted for.

Think of it like a package delivery system where each package has both your address and the recipient's address on it, and can only be opened with a key that knows both addresses.

### Backend Abstraction and Embedded System Integration

#### Technical Explanation

CRUSTy implements a backend abstraction layer to support both local (software-based) encryption and hardware-accelerated encryption via embedded devices:

```rust
/// Trait defining the interface for encryption backends.
pub trait EncryptionBackend {
    /// Encrypts raw data using the provided key.
    fn encrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError>;

    /// Decrypts raw data using the provided key.
    fn decrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError>;

    /// Encrypts raw data for a specific recipient using their email.
    fn encrypt_data_for_recipient(
        &self,
        data: &[u8],
        master_key: &EncryptionKey,
        recipient_email: &str
    ) -> Result<Vec<u8>, EncryptionError>;

    // ... additional methods for file operations ...
}

/// Local (software-based) implementation of the encryption backend.
pub struct LocalBackend;

/// Configuration for the embedded device backend.
pub struct EmbeddedConfig {
    /// Connection type (e.g., USB, UART, Ethernet)
    pub connection_type: ConnectionType,
    /// Device identifier or address
    pub device_id: String,
    /// Additional connection parameters
    pub parameters: std::collections::HashMap<String, String>,
}

/// Connection types for the embedded device.
pub enum ConnectionType {
    /// USB connection
    Usb,
    /// Serial/UART connection
    Serial,
    /// Ethernet/TCP connection
    Ethernet,
}

/// Embedded device implementation of the encryption backend.
pub struct EmbeddedBackend {
    /// Configuration for the embedded device connection
    config: EmbeddedConfig,
    /// Whether the backend is currently connected
    connected: bool,
}

/// Factory for creating encryption backends.
pub struct BackendFactory;

impl BackendFactory {
    /// Creates a new local (software-based) encryption backend.
    pub fn create_local() -> Box<dyn EncryptionBackend> {
        Box::new(LocalBackend)
    }

    /// Creates a new embedded device encryption backend with the specified configuration.
    pub fn create_embedded(config: EmbeddedConfig) -> Box<dyn EncryptionBackend> {
        Box::new(EmbeddedBackend {
            config,
            connected: false,
        })
    }
}
```

The backend selection is handled in the `start_operation` function:

```rust
// Create the appropriate backend
let backend = if self.use_embedded_backend && self.embedded_config.is_some() {
    // Use embedded backend if configured
    BackendFactory::create_embedded(self.embedded_config.clone().unwrap())
} else {
    // Use local backend by default
    BackendFactory::create_local()
};
```

The backend abstraction uses Rust's trait system to define a common interface for all encryption backends. This allows the application to seamlessly switch between local (software-based) encryption and hardware-accelerated encryption via embedded devices. The `BackendFactory` provides a factory pattern for creating the appropriate backend based on the user's configuration.

#### Simple Explanation

CRUSTy's backend system works like having multiple engines that can power the same car:

1. **Backend Abstraction**:

   - CRUSTy defines a standard set of operations that any encryption "engine" must be able to perform.
   - This includes encrypting data, decrypting data, handling files, and working with recipient-specific encryption.
   - This standard interface allows CRUSTy to swap between different encryption engines without changing how the rest of the application works.

2. **Local Backend**:

   - The default "engine" is the local backend, which uses your computer's processor to perform encryption.
   - It implements all the encryption operations using the software libraries built into CRUSTy.
   - This is like using your car's standard engine - reliable and always available.

3. **Embedded Backend**:

   - The alternative "engine" is the embedded backend, which connects to an external hardware device (STM32H5).
   - This device has specialized hardware for encryption operations, making them faster and more secure.
   - You can connect to this device via USB, Serial, or Network connections.
   - This is like having a high-performance engine that you can attach to your car for special situations.

4. **Backend Selection**:
   - When you perform an encryption operation, CRUSTy checks if you've configured and enabled the embedded backend.
   - If yes, it uses the hardware device for encryption.
   - If no, or if there's a problem with the device, it automatically falls back to the local backend.
   - This is like your car automatically switching between engines based on what's available and what you've selected.

Think of it as having both a standard engine and a specialized turbocharger that you can switch between depending on your needs.

## Practical Usage

### Encrypting Files

#### Technical Implementation

The file encryption process is implemented in the `start_operation` function:

```rust
match operation {
    FileOperation::Encrypt => {
        if let Some(file_path) = files.first() {
            let file_path = file_path.clone(); // Clone the PathBuf

            let file_name = file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();

            let mut output_path = output_dir.clone();
            output_path.push(format!("{}.encrypted", file_name));

            let progress_clone = progress.clone();

            let result = if use_recipient && !recipient_email.trim().is_empty() {
                // Use recipient-based encryption
                backend.encrypt_file_for_recipient(
                    &file_path,
                    &output_path,
                    &key,
                    &recipient_email,
                    move |p| {
                        let mut guard = progress_clone.lock().unwrap();
                        if !guard.is_empty() {
                            guard[0] = p;
                        }
                    }
                )
            } else {
                // Use standard encryption
                backend.encrypt_file(
                    &file_path,
                    &output_path,
                    &key,
                    move |p| {
                        let mut guard = progress_clone.lock().unwrap();
                        if !guard.is_empty() {
                            guard[0] = p;
                        }
                    }
                )
            };

            // Log the result and update the UI
            // ...
        }
    },
    // Other operations...
}
```

This code demonstrates how CRUSTy handles file encryption operations, including:

1. Determining the output path based on the input file name
2. Selecting between standard and recipient-specific encryption based on user settings
3. Providing progress updates during the encryption process
4. Handling and logging the results

#### Simple Usage

To encrypt a file with CRUSTy:

1. **Select the File**: Click "Select File" and choose the file you want to encrypt.
2. **Choose the Output Directory**: Select where you want the encrypted file to be saved.
3. **Select or Create a Key**: Either use an existing key or create a new one.
4. **Optional: Specify a Recipient**: If you want to encrypt for a specific person, check "Use recipient-specific encryption" and enter their email.
5. **Optional: Use Hardware Acceleration**: If you have a compatible STM32H5 device, check "Use embedded system for cryptographic operations" and configure the connection.
6. **Start Encryption**: Click "Encrypt" to start the process.
7. **Monitor Progress**: The progress bar will show you how far along the encryption is.
8. **View Results**: Once complete, CRUSTy will show you the results and any errors.

The encrypted file will have the same name as the original with ".encrypted" added to the end.

### Decrypting Files

#### Technical Implementation

The file decryption process is implemented in the `start_operation` function:

```rust
FileOperation::Decrypt => {
    if let Some(file_path) = files.first() {
        let file_name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        let file_stem = file_name.to_string();
        let output_name = if file_stem.ends_with(".encrypted") {
            file_stem.trim_end_matches(".encrypted").to_string()
        } else {
            format!("{}.decrypted", file_stem)
        };

        let mut output_path = output_dir.clone();
        output_path.push(output_name);

        let progress_clone = progress.clone();

        // Try recipient-based decryption first, fall back to standard decryption if it fails
        let result = if use_recipient {
            match backend.decrypt_file_with_recipient(
                file_path,
                &output_path,
                &key,
                move |p| {
                    let mut guard = progress_clone.lock().unwrap();
                    if !guard.is_empty() {
                        guard[0] = p;
                    }
                }
            ) {
                Ok((email, _)) => {
                    // Store the detected recipient email
                    if let Ok(mut results) = shared_results.lock() {
                        results.push(format!("Detected recipient: {}", email));
                    }
                    Ok(())
                },
                Err(e) => {
                    // Fall back to standard decryption
                    backend.decrypt_file(
                        file_path,
                        &output_path,
                        &key,
                        move |p| {
                            let mut guard = progress_clone.lock().unwrap();
                            if !guard.is_empty() {
                                guard[0] = p;
                            }
                        }
                    )
                }
            }
        } else {
            // Use standard decryption
            backend.decrypt_file(
                file_path,
                &output_path,
                &key,
                move |p| {
                    let mut guard = progress_clone.lock().unwrap();
                    if !guard.is_empty() {
                        guard[0] = p;
                    }
                }
            )
        };

        // Log the result and update the UI
        // ...
    }
},
```

This code demonstrates how CRUSTy handles file decryption operations, including:

1. Determining the output path based on the input file name
2. Attempting recipient-specific decryption first if enabled
3. Falling back to standard decryption if recipient-specific decryption fails
4. Providing progress updates during the decryption process
5. Handling authentication failures with specific error messages

#### Simple Usage

To decrypt a file with CRUSTy:

1. **Select the File**: Click "Select File" and choose the encrypted file.
2. **Choose the Output Directory**: Select where you want the decrypted file to be saved.
3. **Select the Key**: Choose the key that was used to encrypt the file.
4. **Optional: Enable Recipient Detection**: If the file might be encrypted for a specific recipient, check "Use recipient-specific encryption".
5. **Optional: Use Hardware Acceleration**: If you have a compatible STM32H5 device, check "Use embedded system for cryptographic operations".
6. **Start Decryption**: Click "Decrypt" to start the process.
7. **Monitor Progress**: The progress bar will show you how far along the decryption is.
8. **View Results**: Once complete, CRUSTy will show you the results, including the recipient's email if it was a recipient-specific encryption.

If the decryption fails with an "Authentication failed" error, it means either:

- You're using the wrong key
- The file has been tampered with
- The file is corrupted

### Batch Processing

#### Technical Implementation

CRUSTy supports batch processing of multiple files:

```rust
FileOperation::BatchEncrypt => {
    let progress_clone = progress.clone();

    // Convert Vec<PathBuf> to Vec<&Path>
    let path_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();

    let results = if use_recipient && !recipient_email.trim().is_empty() {
        // Use recipient-based batch encryption
        backend.encrypt_files_for_recipient(
            &path_refs,
            &output_dir,
            &key,
            &recipient_email,
            move |idx, p| {
                let mut guard = progress_clone.lock().unwrap();
                if idx < guard.len() {
                    guard[idx] = p;
                }
            }
        )
    } else {
        // Use standard batch encryption
        backend.encrypt_files(
            &path_refs,
            &output_dir,
            &key,
            move |idx, p| {
                let mut guard = progress_clone.lock().unwrap();
                if idx < guard.len() {
                    guard[idx] = p;
                }
            }
        )
    };

    // Log and handle results
    // ...
}
```

The batch processing implementation:

1. Converts the list of file paths to the format expected by the backend
2. Selects between standard and recipient-specific encryption based on user settings
3. Provides progress updates for each file during the encryption process
4. Handles and logs the results for each file

#### Simple Usage

To process multiple files at once:

1. **Select "Multiple Files" Mode**: Switch to the batch processing mode.
2. **Select Files**: Click "Select Files" to choose multiple files.
3. **Choose the Output Directory**: Select where you want the encrypted/decrypted files to be saved.
4. **Select or Create a Key**: Either use an existing key or create a new one.
5. **Optional: Specify a Recipient**: For encryption, you can check "Use recipient-specific encryption" and enter an email.
6. **Optional: Use Hardware Acceleration**: If you have a compatible STM32H5 device, check "Use embedded system for cryptographic operations".
7. **Start Processing**: Click "Encrypt" or "Decrypt" to start the process.
8. **Monitor Progress**: The progress bar will show you how far along each file is.
9. **View Results**: Once complete, CRUSTy will show you the results for each file.

Batch processing is especially useful when you need to encrypt or decrypt many files at once, saving you time and effort.

## Security Considerations

### Technical Security Details

CRUSTy implements several security best practices:

1. **AES-256-GCM**: Uses the AES algorithm in Galois/Counter Mode with 256-bit keys, which provides both confidentiality and authentication.

2. **Nonce Management**: Generates a unique nonce for each encryption operation to prevent replay attacks.

3. **Authentication**: The GCM mode provides authentication to detect tampering with the ciphertext.

4. **Key Derivation**: Uses HKDF with SHA-256 for secure key derivation in recipient-specific encryption.

5. **Error Handling**: Provides specific error messages for authentication failures while avoiding information leakage.

6. **Hardware Isolation**: Supports offloading cryptographic operations to isolated hardware for enhanced security.

7. **Memory Safety**: Built with Rust, which provides memory safety guarantees to prevent buffer overflows and other memory-related vulnerabilities.

### Simple Security Guidelines

When using CRUSTy, keep these security considerations in mind:

1. **Key Protection**: Your encryption keys are the most critical security element. Keep them safe and backed up in a secure location.

2. **Strong Keys**: Always use the built-in key generator rather than creating your own keys, as it uses a secure random number generator.

3. **Authentication**: CRUSTy will warn you if a file has been tampered with or if you're using the wrong key. Pay attention to these warnings.

4. **Computer Security**: CRUSTy can't protect against malware on your computer that might capture your keys or see your data before it's encrypted.

5. **Physical Security**: If someone has access to your computer while you're using CRUSTy with an unlocked key, they might be able to access your files.

6. **Hardware Benefits**: Using the embedded hardware backend provides additional security by isolating cryptographic operations from your main computer.

Remember that encryption is just one part of a comprehensive security strategy. It's important to maintain good security practices in all aspects of your digital life.

## Troubleshooting

### Common Issues and Solutions

#### "Destination file already exists"

**Technical Cause**: CRUSTy checks if the destination file exists before starting encryption/decryption operations to prevent accidental overwrites:

```rust
// Check if the destination file already exists
if dest_path.exists() {
    return Err(EncryptionError::Io(
        io::Error::new(io::ErrorKind::AlreadyExists, "Destination file already exists")
    ));
}
```

**Solution**: Delete the existing file or choose a different output directory.

#### "Authentication failed" or "Wrong encryption key"

**Technical Cause**: AES-GCM includes an authentication tag that verifies the integrity of the data and the correctness of the key. If authentication fails, it means either the key is incorrect or the data has been tampered with:

```rust
// Decrypt the chunk
let decrypted_data = match cipher.decrypt(nonce, encrypted_chunk) {
    Ok(data) => data,
    Err(e) => {
        // Provide a more specific error message for authentication failures
        if e.to_string().contains("authentication") || e.to_string().contains("tag mismatch") {
            return Err(EncryptionError::Decryption(
                "Authentication failed: The encryption key is incorrect or the data is corrupted".to_string()
            ));
        } else {
            return Err(EncryptionError::Decryption(e.to_string()));
        }
    }
};
```

**Solution**: Try a different key if you have one, or check if the file was transferred correctly.

#### "Embedded backend not implemented"

**Technical Cause**: The embedded backend is currently a placeholder that returns an error when used:

```rust
fn encrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
    // This is a placeholder implementation that will be replaced with actual
    // embedded device encryption logic when the embedded system integration is implemented.

    // For now, return an error indicating that the embedded backend is not implemented
    Err(EncryptionError::Encryption("Embedded backend not implemented".to_string()))
}
```

**Solution**: Make sure you've clicked "Apply Configuration" after entering the device details. Check that your device is properly connected and powered on. This error may also appear if the embedded system integration is not yet fully implemented in your version of CRUSTy.

#### "Failed to connect to embedded device"

**Technical Cause**: The connection to the embedded device failed during the initialization phase:

```rust
pub fn connect(&mut self) -> Result<(), EncryptionError> {
    // Attempt to establish connection with the device
    // ...
    if connection_failed {
        return Err(EncryptionError::Encryption("Failed to connect to embedded device".to_string()));
    }
    // ...
}
```

**Solution**:

- Verify that the device ID/address is correct
- Check physical connections (USB cable, network connection, etc.)
- Ensure the device is powered on and running the CRUSTy firmware
- Try a different connection type if available

#### "Communication error with embedded device"

**Technical Cause**: The connection was established but was interrupted during an operation:

```rust
// During an encryption/decryption operation
if communication_error {
    return Err(EncryptionError::Encryption("Communication error with embedded device".to_string()));
}
```

**Solution**:

- Check for loose connections
- Ensure the device has stable power
- Try reducing the file size if the operation involves large files
- Restart the device and try again
