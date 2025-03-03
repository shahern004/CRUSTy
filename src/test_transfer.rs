use crate::encryption::EncryptionKey;
use crate::split_key::{SplitEncryptionKey, KeyShareManager, TransferPackage, ShareFormat, KeyPurpose, SplitKeyError};
use std::path::PathBuf;

/// Test the transfer functionality
pub fn test_transfer() -> Result<(), SplitKeyError> {
    // Generate a new encryption key
    let key = EncryptionKey::generate();
    
    // Create a key share manager
    let app_name = "CRUSTy_Test";
    let share_dir = PathBuf::from("test_shares");
    let key_share_manager = KeyShareManager::new(app_name, &share_dir)?;
    
    // Create a transfer package
    let package = key_share_manager.create_transfer_package(&key, 2, 3)?;
    
    // Get the shares as text
    let share1 = package.get_share_text(0)?;
    let share2 = package.get_share_text(1)?;
    
    println!("Share 1: {}", share1);
    println!("Share 2: {}", share2);
    
    // Get the shares as mnemonics
    let mnemonic1 = package.get_share_mnemonic(0)?;
    let mnemonic2 = package.get_share_mnemonic(1)?;
    
    println!("Mnemonic 1: {}", mnemonic1);
    println!("Mnemonic 2: {}", mnemonic2);
    
    // Reconstruct the key from the shares
    let shares = vec![share1.to_string(), share2.to_string()];
    let reconstructed_key = key_share_manager.reconstruct_key_from_text_shares(&shares)?;
    
    // Verify the reconstructed key matches the original
    assert_eq!(reconstructed_key.to_base64(), key.to_base64());
    println!("Key successfully reconstructed!");
    
    Ok(())
}
