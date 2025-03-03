/// Split-key functionality for enhanced security.
///
/// This module provides Shamir's Secret Sharing implementation for splitting
/// encryption keys into multiple shares, allowing for more secure key management
/// and multi-party authorization for decryption.
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::error::Error;
use std::fmt;
use std::str;

use sharks::{Share, Sharks};
use keyring::Entry;
use qrcode::{QrCode, render::svg};
use image::{DynamicImage, ImageFormat};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{Rng, thread_rng};
use data_encoding::{BASE32, BASE32_NOPAD};

use crate::encryption::EncryptionKey;

/// Error type for split key operations
#[derive(Debug)]
pub enum SplitKeyError {
    /// Error related to Shamir's Secret Sharing
    Sharing(String),
    /// Error related to key storage
    Storage(String),
    /// Error related to QR code generation
    QrCode(String),
    /// Error related to file I/O
    Io(std::io::Error),
    /// Error related to key operations
    Key(String),
    /// Error related to share encoding/decoding
    Encoding(String),
    /// Error related to transfer operations
    Transfer(String),
}

impl fmt::Display for SplitKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitKeyError::Sharing(msg) => write!(f, "Sharing error: {}", msg),
            SplitKeyError::Storage(msg) => write!(f, "Storage error: {}", msg),
            SplitKeyError::QrCode(msg) => write!(f, "QR code error: {}", msg),
            SplitKeyError::Io(err) => write!(f, "I/O error: {}", err),
            SplitKeyError::Key(msg) => write!(f, "Key error: {}", msg),
            SplitKeyError::Encoding(msg) => write!(f, "Encoding error: {}", msg),
            SplitKeyError::Transfer(msg) => write!(f, "Transfer error: {}", msg),
        }
    }
}

impl Error for SplitKeyError {}

impl From<std::io::Error> for SplitKeyError {
    fn from(err: std::io::Error) -> Self {
        SplitKeyError::Io(err)
    }
}

/// Calculate CRC16 checksum
fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    
    crc
}

// A small subset of common words for mnemonic encoding
// In a real implementation, you would use a larger wordlist like BIP39
const WORDLIST: [&str; 256] = [
    "apple", "banana", "cherry", "dog", "elephant", "fox", "grape", "horse", "igloo", "jacket",
    "kite", "lemon", "mango", "nest", "orange", "pear", "queen", "rabbit", "sun", "tree",
    "umbrella", "violet", "water", "xylophone", "yellow", "zebra", "air", "book", "cat", "door",
    "earth", "fire", "gold", "hat", "ice", "jar", "key", "lamp", "moon", "nail",
    "ocean", "paper", "quilt", "river", "star", "table", "uncle", "vase", "wind", "box",
    "yard", "zoo", "ant", "bear", "cow", "duck", "egg", "fish", "goat", "hen",
    "ink", "jam", "king", "lion", "milk", "nut", "owl", "pig", "quail", "rat",
    "sheep", "tiger", "urn", "van", "wolf", "yak", "zebra", "arrow", "ball", "coin",
    "dice", "eye", "flag", "gift", "hand", "iron", "jewel", "knife", "leaf", "map",
    "needle", "oar", "pen", "quartz", "rope", "sail", "tea", "urn", "veil", "wheel",
    "yarn", "zest", "arch", "bell", "cake", "desk", "egg", "fork", "gate", "hill",
    "ink", "jug", "kite", "lock", "mask", "net", "oven", "pot", "quilt", "ring",
    "sock", "toy", "urn", "vase", "well", "box", "yarn", "zone", "atom", "boat",
    "card", "drum", "eel", "flute", "gear", "harp", "ink", "jade", "keel", "lens",
    "mast", "note", "opal", "pipe", "quill", "reed", "sail", "tube", "urn", "valve",
    "wire", "xray", "yarn", "zinc", "ace", "bat", "cap", "dart", "ear", "fan",
    "gem", "hat", "ice", "jet", "key", "lid", "mat", "net", "orb", "pin",
    "queen", "rod", "saw", "tag", "urn", "vat", "web", "box", "yam", "zip",
    "arc", "bin", "cup", "dot", "elf", "fin", "gun", "hut", "ink", "jar",
    "kit", "log", "mug", "nut", "oil", "pan", "quip", "rag", "sip", "tin",
    "urn", "van", "wig", "box", "yew", "zap", "arm", "bug", "cog", "den",
    "eel", "fog", "gum", "hog", "ink", "jaw", "kit", "leg", "map", "nap",
    "oak", "peg", "quiz", "rib", "sap", "toe", "urn", "vet", "wax", "box",
    "yak", "zip"
];

/// Convert text to a mnemonic phrase
fn text_to_mnemonic(text: &str) -> Result<String, String> {
    // Remove dashes and whitespace
    let clean_text = text.replace(['-', ' '], "");
    
    // Convert to bytes
    let bytes = clean_text.as_bytes();
    
    // Convert each byte to a word
    let mut words = Vec::with_capacity(bytes.len());
    for &byte in bytes {
        words.push(WORDLIST[byte as usize]);
    }
    
    // Join with spaces
    Ok(words.join(" "))
}

/// Convert a mnemonic phrase back to text
fn mnemonic_to_text(mnemonic: &str) -> Result<String, String> {
    // Split into words
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    
    // Convert each word to a byte
    let mut bytes = Vec::with_capacity(words.len());
    for word in words {
        let word_lower = word.to_lowercase();
        match WORDLIST.iter().position(|&w| w == word_lower) {
            Some(index) => bytes.push(index as u8),
            None => return Err(format!("Unknown word in mnemonic: {}", word)),
        }
    }
    
    // Convert bytes to string
    match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(_) => Err("Invalid UTF-8 sequence in mnemonic".to_string()),
    }
}
}

/// Share format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShareFormat {
    /// Binary format (raw bytes)
    Binary,
    /// Text format (Base32 encoded)
    Text,
    /// Mnemonic format (word-based)
    Mnemonic,
}

/// Purpose of the split key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyPurpose {
    /// Standard key for personal use
    Standard,
    /// Transfer key for sending to others
    Transfer,
}

/// Represents a split encryption key using Shamir's Secret Sharing
pub struct SplitEncryptionKey {
    /// The threshold number of shares needed to reconstruct the key
    threshold: u8,
    /// The total number of shares created
    shares_count: u8,
    /// The shares of the key (if available)
    shares: Vec<Share>,
    /// The reconstructed key (if available)
    key: Option<EncryptionKey>,
    /// The purpose of this split key
    purpose: KeyPurpose,
}

impl SplitEncryptionKey {
    /// Create a new split key from an existing encryption key
    pub fn new(key: &EncryptionKey, threshold: u8, shares_count: u8, purpose: KeyPurpose) -> Result<Self, SplitKeyError> {
        if threshold < 2 {
            return Err(SplitKeyError::Sharing("Threshold must be at least 2".to_string()));
        }
        
        if shares_count < threshold {
            return Err(SplitKeyError::Sharing("Shares count must be at least equal to threshold".to_string()));
        }
        
        // Get the key as bytes
        let key_bytes = key.to_base64().into_bytes();
        
        // Create the Shamir's Secret Sharing scheme
        let sharks = Sharks(threshold);
        
        // Split the key into shares
        let dealer = sharks.dealer(&key_bytes);
        let shares: Vec<Share> = dealer.take(shares_count as usize).collect();
        
        Ok(SplitEncryptionKey {
            threshold,
            shares_count,
            shares,
            key: Some(key.clone()),
            purpose,
        })
    }
    
    /// Create a new split key specifically for transfer
    pub fn new_for_transfer(key: &EncryptionKey, threshold: u8, shares_count: u8) -> Result<Self, SplitKeyError> {
        Self::new(key, threshold, shares_count, KeyPurpose::Transfer)
    }
    
    /// Reconstruct a key from shares
    pub fn from_shares(shares: Vec<Share>, threshold: u8) -> Result<Self, SplitKeyError> {
        if shares.len() < threshold as usize {
            return Err(SplitKeyError::Sharing(
                format!("Not enough shares: got {}, need at least {}", shares.len(), threshold)
            ));
        }
        
        // Create the Shamir's Secret Sharing scheme
        let sharks = Sharks(threshold);
        
        // Reconstruct the secret
        let key_bytes = sharks.recover(&shares)
            .map_err(|e| SplitKeyError::Sharing(format!("Failed to recover key: {}", e)))?;
        
        // Convert back to a string and then to an EncryptionKey
        let key_base64 = String::from_utf8(key_bytes)
            .map_err(|e| SplitKeyError::Key(format!("Invalid key data: {}", e)))?;
        
        let key = EncryptionKey::from_base64(&key_base64)
            .map_err(|e| SplitKeyError::Key(format!("Invalid key: {}", e)))?;
        
        Ok(SplitEncryptionKey {
            threshold,
            shares_count: shares.len() as u8,
            shares,
            key: Some(key),
            purpose: KeyPurpose::Standard, // Default to standard purpose for reconstructed keys
        })
    }
    
    /// Get the reconstructed key
    pub fn get_key(&self) -> Option<&EncryptionKey> {
        self.key.as_ref()
    }
    
    /// Get a specific share
    pub fn get_share(&self, index: usize) -> Option<&Share> {
        self.shares.get(index)
    }
    
    /// Get all shares
    pub fn get_shares(&self) -> &[Share] {
        &self.shares
    }
    
    /// Get the threshold
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }
    
    /// Get the total number of shares
    pub fn get_shares_count(&self) -> u8 {
        self.shares_count
    }
    
    /// Get the purpose of this split key
    pub fn get_purpose(&self) -> KeyPurpose {
        self.purpose
    }
    
    /// Convert a share to a text representation
    pub fn share_to_text(&self, index: usize) -> Result<String, SplitKeyError> {
        if index >= self.shares.len() {
            return Err(SplitKeyError::Encoding(format!("Share index {} out of bounds", index)));
        }
        
        let share = &self.shares[index];
        
        // Format: version-index-threshold-checksum-data
        // Version: 1 byte
        // Index: 1 byte
        // Threshold: 1 byte
        // Checksum: 2 bytes (CRC16)
        // Data: variable length
        
        let mut buffer = Vec::with_capacity(5 + share.as_ref().len());
        
        // Version (1)
        buffer.push(1);
        
        // Index
        buffer.push(index as u8);
        
        // Threshold
        buffer.push(self.threshold);
        
        // Placeholder for checksum (will be filled later)
        buffer.push(0);
        buffer.push(0);
        
        // Share data
        buffer.extend_from_slice(share.as_ref());
        
        // Calculate checksum (CRC16)
        let checksum = crc16(&buffer[0..3]) as u16;
        buffer[3] = (checksum >> 8) as u8;
        buffer[4] = (checksum & 0xFF) as u8;
        
        // Encode as Base32
        let encoded = BASE32.encode(&buffer);
        
        // Format with dashes every 5 characters for readability
        let mut formatted = String::with_capacity(encoded.len() + (encoded.len() / 5));
        for (i, c) in encoded.chars().enumerate() {
            if i > 0 && i % 5 == 0 {
                formatted.push('-');
            }
            formatted.push(c);
        }
        
        Ok(formatted)
    }
    
    /// Convert a text representation back to a share
    pub fn share_from_text(text: &str) -> Result<Share, SplitKeyError> {
        // Remove dashes and whitespace
        let clean_text = text.replace(['-', ' '], "");
        
        // Decode from Base32
        let buffer = BASE32.decode(clean_text.as_bytes())
            .map_err(|e| SplitKeyError::Encoding(format!("Invalid Base32 encoding: {}", e)))?;
        
        // Check minimum length
        if buffer.len() < 5 {
            return Err(SplitKeyError::Encoding("Share text too short".to_string()));
        }
        
        // Check version
        if buffer[0] != 1 {
            return Err(SplitKeyError::Encoding(format!("Unsupported share version: {}", buffer[0])));
        }
        
        // Verify checksum
        let stored_checksum = ((buffer[3] as u16) << 8) | (buffer[4] as u16);
        let calculated_checksum = crc16(&buffer[0..3]);
        
        if stored_checksum != calculated_checksum {
            return Err(SplitKeyError::Encoding("Invalid checksum, share may be corrupted".to_string()));
        }
        
        // Extract share data
        let share_data = buffer[5..].to_vec();
        
        Ok(Share::from(share_data))
    }
    
    /// Convert a share to a mnemonic phrase
    pub fn share_to_mnemonic(&self, index: usize) -> Result<String, SplitKeyError> {
        if index >= self.shares.len() {
            return Err(SplitKeyError::Encoding(format!("Share index {} out of bounds", index)));
        }
        
        let share = &self.shares[index];
        
        // First convert to text format
        let text = self.share_to_text(index)?;
        
        // Then convert to mnemonic using BIP39 wordlist
        let mnemonic = text_to_mnemonic(&text)
            .map_err(|e| SplitKeyError::Encoding(format!("Failed to create mnemonic: {}", e)))?;
        
        Ok(mnemonic)
    }
    
    /// Convert a mnemonic phrase back to a share
    pub fn share_from_mnemonic(mnemonic: &str) -> Result<Share, SplitKeyError> {
        // Convert mnemonic to text
        let text = mnemonic_to_text(mnemonic)
            .map_err(|e| SplitKeyError::Encoding(format!("Failed to parse mnemonic: {}", e)))?;
        
        // Convert text to share
        Self::share_from_text(&text)
    }
    
    /// Store a share in the OS credential store
    pub fn store_share_in_credential_store(&self, index: usize, service_name: &str) -> Result<(), SplitKeyError> {
        if index >= self.shares.len() {
            return Err(SplitKeyError::Storage(format!("Share index {} out of bounds", index)));
        }
        
        let share = &self.shares[index];
        let share_data = STANDARD.encode(share.as_ref());
        
        let entry = Entry::new(service_name, &format!("crusty-share-{}", index))
            .map_err(|e| SplitKeyError::Storage(format!("Failed to create keyring entry: {}", e)))?;
            
        entry.set_password(&share_data)
            .map_err(|e| SplitKeyError::Storage(format!("Failed to store share: {}", e)))?;
            
        Ok(())
    }
    
    /// Retrieve a share from the OS credential store
    pub fn retrieve_share_from_credential_store(service_name: &str, index: usize) -> Result<Share, SplitKeyError> {
        let entry = Entry::new(service_name, &format!("crusty-share-{}", index))
            .map_err(|e| SplitKeyError::Storage(format!("Failed to create keyring entry: {}", e)))?;
            
        let share_data = entry.get_password()
            .map_err(|e| SplitKeyError::Storage(format!("Failed to retrieve share: {}", e)))?;
            
        let share_bytes = STANDARD.decode(&share_data)
            .map_err(|e| SplitKeyError::Storage(format!("Invalid share data: {}", e)))?;
            
        Ok(Share::from(share_bytes))
    }
    
    /// Save a share to a file
    pub fn save_share_to_file(&self, index: usize, path: &Path, format: ShareFormat) -> Result<(), SplitKeyError> {
        if index >= self.shares.len() {
            return Err(SplitKeyError::Storage(format!("Share index {} out of bounds", index)));
        }
        
        let mut file = File::create(path)?;
        
        match format {
            ShareFormat::Binary => {
                let share = &self.shares[index];
                let share_data = STANDARD.encode(share.as_ref());
                file.write_all(share_data.as_bytes())?;
            },
            ShareFormat::Text => {
                let text = self.share_to_text(index)?;
                file.write_all(text.as_bytes())?;
            },
            ShareFormat::Mnemonic => {
                let mnemonic = self.share_to_mnemonic(index)?;
                file.write_all(mnemonic.as_bytes())?;
            }
        }
        
        Ok(())
    }
    
    /// Load a share from a file
    pub fn load_share_from_file(path: &Path) -> Result<Share, SplitKeyError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        // Try to determine the format and parse accordingly
        if content.contains('-') || content.chars().all(|c| c.is_ascii_alphanumeric() || c.is_whitespace()) {
            // Looks like text format
            Self::share_from_text(&content)
        } else if content.split_whitespace().count() > 1 {
            // Looks like mnemonic format
            Self::share_from_mnemonic(&content)
        } else {
            // Assume base64 binary format (legacy)
            let share_bytes = STANDARD.decode(&content)
                .map_err(|e| SplitKeyError::Storage(format!("Invalid share data: {}", e)))?;
                
            Ok(Share::from(share_bytes))
        }
    }
    
    /// Generate a QR code for a share
    pub fn generate_share_qr_code(&self, index: usize) -> Result<String, SplitKeyError> {
        if index >= self.shares.len() {
            return Err(SplitKeyError::QrCode(format!("Share index {} out of bounds", index)));
        }
        
        let share = &self.shares[index];
        let share_data = STANDARD.encode(share.as_ref());
        
        let code = QrCode::new(share_data.as_bytes())
            .map_err(|e| SplitKeyError::QrCode(format!("Failed to generate QR code: {}", e)))?;
            
        let svg = code.render()
            .min_dimensions(200, 200)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();
            
        Ok(svg)
    }
    
    /// Save a QR code for a share to a file
    pub fn save_share_qr_code_to_file(&self, index: usize, path: &Path) -> Result<(), SplitKeyError> {
        let svg = self.generate_share_qr_code(index)?;
        
        let mut file = File::create(path)?;
        file.write_all(svg.as_bytes())?;
        
        Ok(())
    }
}

/// Transfer package for out-of-band file transfers
pub struct TransferPackage {
    /// The shares for the transfer
    shares: Vec<String>,
    /// The threshold required to reconstruct the key
    threshold: u8,
    /// The format of the shares
    format: ShareFormat,
}

impl TransferPackage {
    /// Create a new transfer package from a split key
    pub fn new(split_key: &SplitEncryptionKey) -> Result<Self, SplitKeyError> {
        if split_key.get_purpose() != KeyPurpose::Transfer {
            return Err(SplitKeyError::Transfer(
                "Cannot create transfer package from non-transfer key".to_string()
            ));
        }
        
        let mut shares = Vec::with_capacity(split_key.shares.len());
        
        // Convert all shares to text format
        for i in 0..split_key.shares.len() {
            let share_text = split_key.share_to_text(i)?;
            shares.push(share_text);
        }
        
        Ok(TransferPackage {
            shares,
            threshold: split_key.threshold,
            format: ShareFormat::Text,
        })
    }
    
    /// Get a specific share as text
    pub fn get_share_text(&self, index: usize) -> Result<&str, SplitKeyError> {
        self.shares.get(index)
            .ok_or_else(|| SplitKeyError::Transfer(format!("Share index {} out of bounds", index)))
            .map(|s| s.as_str())
    }
    
    /// Get a specific share as a mnemonic phrase
    pub fn get_share_mnemonic(&self, index: usize) -> Result<String, SplitKeyError> {
        let text = self.get_share_text(index)?;
        text_to_mnemonic(text)
            .map_err(|e| SplitKeyError::Encoding(format!("Failed to create mnemonic: {}", e)))
    }
    
    /// Get the threshold
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }
    
    /// Get the number of shares
    pub fn get_shares_count(&self) -> usize {
        self.shares.len()
    }
    
    /// Save a share to a file
    pub fn save_share_to_file(&self, index: usize, path: &Path) -> Result<(), SplitKeyError> {
        let share_text = self.get_share_text(index)?;
        
        let mut file = File::create(path)?;
        file.write_all(share_text.as_bytes())?;
        
        Ok(())
    }
    
    /// Reconstruct the key from a set of shares
    pub fn reconstruct_key(&self, share_indices: &[usize]) -> Result<EncryptionKey, SplitKeyError> {
        if share_indices.len() < self.threshold as usize {
            return Err(SplitKeyError::Sharing(
                format!("Not enough shares: got {}, need at least {}", share_indices.len(), self.threshold)
            ));
        }
        
        let mut shares = Vec::with_capacity(share_indices.len());
        
        // Convert text shares back to Share objects
        for &index in share_indices {
            let share_text = self.get_share_text(index)?;
            let share = SplitEncryptionKey::share_from_text(share_text)?;
            shares.push(share);
        }
        
        // Reconstruct the key
        let split_key = SplitEncryptionKey::from_shares(shares, self.threshold)?;
        
        // Get the reconstructed key
        split_key.get_key()
            .cloned()
            .ok_or_else(|| SplitKeyError::Key("Failed to reconstruct key".to_string()))
    }
}

/// Key share storage manager
pub struct KeyShareManager {
    /// Application name for credential store
    app_name: String,
    /// Directory for storing share files
    share_dir: PathBuf,
}

impl KeyShareManager {
    /// Create a new key share manager
    pub fn new(app_name: &str, share_dir: &Path) -> Result<Self, SplitKeyError> {
        // Create the share directory if it doesn't exist
        if !share_dir.exists() {
            fs::create_dir_all(share_dir)?;
        }
        
        Ok(KeyShareManager {
            app_name: app_name.to_string(),
            share_dir: share_dir.to_path_buf(),
        })
    }
    
    /// Store the primary share in the OS credential store
    pub fn store_primary_share(&self, split_key: &SplitEncryptionKey) -> Result<(), SplitKeyError> {
        split_key.store_share_in_credential_store(0, &self.app_name)
    }
    
    /// Retrieve the primary share from the OS credential store
    pub fn retrieve_primary_share(&self) -> Result<Share, SplitKeyError> {
        SplitEncryptionKey::retrieve_share_from_credential_store(&self.app_name, 0)
    }
    
    /// Create a transfer package for out-of-band file transfer
    pub fn create_transfer_package(
        &self,
        key: &EncryptionKey,
        threshold: u8,
        shares_count: u8
    ) -> Result<TransferPackage, SplitKeyError> {
        // Create a split key specifically for transfer
        let split_key = SplitEncryptionKey::new_for_transfer(key, threshold, shares_count)?;
        
        // Create a transfer package
        let package = TransferPackage::new(&split_key)?;
        
        Ok(package)
    }
    
    /// Save the secondary share to a file
    pub fn save_secondary_share(
        &self, 
        split_key: &SplitEncryptionKey, 
        filename: &str,
        format: ShareFormat
    ) -> Result<PathBuf, SplitKeyError> {
        let path = self.share_dir.join(filename);
        split_key.save_share_to_file(1, &path, format)?;
        Ok(path)
    }
    
    /// Load the secondary share from a file
    pub fn load_secondary_share(&self, path: &Path) -> Result<Share, SplitKeyError> {
        SplitEncryptionKey::load_share_from_file(path)
    }
    
    /// Generate and save a recovery share in the specified format
    pub fn save_recovery_share(
        &self, 
        split_key: &SplitEncryptionKey, 
        filename: &str,
        format: ShareFormat
    ) -> Result<PathBuf, SplitKeyError> {
        let path = self.share_dir.join(filename);
        
        match format {
            ShareFormat::Binary => {
                split_key.save_share_to_file(2, &path, ShareFormat::Binary)?;
            },
            ShareFormat::Text => {
                split_key.save_share_to_file(2, &path, ShareFormat::Text)?;
            },
            ShareFormat::Mnemonic => {
                split_key.save_share_to_file(2, &path, ShareFormat::Mnemonic)?;
            }
        }
        
        Ok(path)
    }
    
    /// Generate and save a QR code for the recovery share (legacy method)
    pub fn save_recovery_share_qr_code(&self, split_key: &SplitEncryptionKey, filename: &str) -> Result<PathBuf, SplitKeyError> {
        let path = self.share_dir.join(filename);
        split_key.save_share_qr_code_to_file(2, &path)?;
        Ok(path)
    }
    
    /// Reconstruct a key from available shares
    pub fn reconstruct_key(&self, secondary_share_path: &Path) -> Result<EncryptionKey, SplitKeyError> {
        // Retrieve the primary share
        let primary_share = self.retrieve_primary_share()?;
        
        // Load the secondary share
        let secondary_share = self.load_secondary_share(secondary_share_path)?;
        
        // Reconstruct the key
        let shares = vec![primary_share, secondary_share];
        let split_key = SplitEncryptionKey::from_shares(shares, 2)?;
        
        // Get the reconstructed key
        split_key.get_key()
            .cloned()
            .ok_or_else(|| SplitKeyError::Key("Failed to reconstruct key".to_string()))
    }
    
    /// Reconstruct a key from text shares
    pub fn reconstruct_key_from_text_shares(&self, share_texts: &[String]) -> Result<EncryptionKey, SplitKeyError> {
        if share_texts.len() < 2 {
            return Err(SplitKeyError::Sharing(
                format!("Not enough shares: got {}, need at least 2", share_texts.len())
            ));
        }
        
        let mut shares = Vec::with_capacity(share_texts.len());
        
        // Convert text shares to Share objects
        for text in share_texts {
            let share = SplitEncryptionKey::share_from_text(text)?;
            shares.push(share);
        }
        
        // Reconstruct the key
        let split_key = SplitEncryptionKey::from_shares(shares, 2)?;
        
        // Get the reconstructed key
        split_key.get_key()
            .cloned()
            .ok_or_else(|| SplitKeyError::Key("Failed to reconstruct key".to_string()))
    }
    
    /// Reconstruct a key from primary share and recovery share
    pub fn reconstruct_key_with_recovery(&self, recovery_share: Share) -> Result<EncryptionKey, SplitKeyError> {
        // Retrieve the primary share
        let primary_share = self.retrieve_primary_share()?;
        
        // Reconstruct the key
        let shares = vec![primary_share, recovery_share];
        let split_key = SplitEncryptionKey::from_shares(shares, 2)?;
        
        // Get the reconstructed key
        split_key.get_key()
            .cloned()
            .ok_or_else(|| SplitKeyError::Key("Failed to reconstruct key".to_string()))
    }
}
