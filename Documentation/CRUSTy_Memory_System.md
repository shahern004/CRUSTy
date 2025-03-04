# CRUSTy Memory System

This document describes the memory system for the CRUSTy (Cryptographic Rust Utility) project. The memory system uses the Model Context Protocol (MCP) to store and retrieve knowledge about the CRUSTy codebase, helping developers maintain a consistent understanding of the project structure and relationships.

## Table of Contents

- [Overview](#overview)
- [Memory System Structure](#memory-system-structure)
  - [Entity Types](#entity-types)
  - [Relation Types](#relation-types)
  - [Memory Categories](#memory-categories)
- [Using the Memory System](#using-the-memory-system)
  - [Memory Retrieval](#memory-retrieval)
  - [Memory Update](#memory-update)
  - [Daily Development Workflow](#daily-development-workflow)
  - [Common Patterns](#common-patterns)
- [Practical Examples](#practical-examples)
  - [Example 1: Adding a New Feature](#example-1-adding-a-new-feature)
  - [Example 2: Fixing a Bug](#example-2-fixing-a-bug)
  - [Example 3: Adding a New Module](#example-3-adding-a-new-module)
- [Best Practices](#best-practices)

## Overview

The CRUSTy Memory System is designed to:

1. Store knowledge about the CRUSTy codebase structure
2. Track relationships between different components
3. Maintain a consistent understanding of the project architecture
4. Assist in development by providing relevant context
5. Ensure architectural integrity during feature development and bug fixes

By using this system, you can ensure that your changes integrate well with the existing architecture and maintain the overall design integrity of the application.

## Memory System Structure

The CRUSTy memory system organizes knowledge into entities and relations:

### Entity Types

- **Module**: Major code components (encryption.rs, backend.rs, gui.rs, etc.)
- **Feature**: Functional capabilities (file encryption, key management, etc.)
- **DataStructure**: Key classes and structs (EncryptionKey, Backend, CrustyApp, etc.)
- **Algorithm**: Encryption algorithms and techniques (AES-256-GCM, HKDF, etc.)
- **Interface**: Trait definitions and APIs (EncryptionBackend, etc.)
- **Configuration**: System settings and parameters (EmbeddedConfig, etc.)

### Relation Types

- **contains**: Application contains modules or modules contain features
- **provides**: Application provides features
- **defines**: Module defines data structures or interfaces
- **implements**: Module or data structure implements an interface
- **depends_on**: Module requires another module
- **uses**: Module, feature, or data structure uses an algorithm or another component
- **manages**: Feature manages a data structure

### Memory Categories

While working with the CRUSTy codebase, be attentive to any new information that falls into these categories:

1. **File Structure**: Module organization, file relationships, and code architecture
2. **Core Components**: Encryption system, backend implementations, GUI elements, etc.
3. **Features**: Encryption/decryption capabilities, key management, batch processing, etc.
4. **Implementation Details**: Algorithms, data structures, and specific code patterns
5. **Relationships**: Dependencies between modules, inheritance patterns, and API connections

## Using the Memory System

### Memory Retrieval

When working with the CRUSTy codebase, always begin by retrieving relevant information from your memory:

```
Remembering...

From my memory about CRUSTy:

CRUSTy is a secure file encryption application using AES-256-GCM encryption. It's built with Rust and uses eframe for GUI. Key features include encryption/decryption of individual files, batch processing, key management, operation logging, and progress tracking.

Core modules:
- encryption.rs: Implements AES-256-GCM encryption and key management
- backend.rs: Provides abstraction for different encryption backends
- backend_local.rs: Implements local software-based encryption
- backend_embedded.rs: Implements embedded device encryption
- gui.rs: Implements the graphical user interface

Key data structures:
- EncryptionKey: Represents an AES-256-GCM encryption key
- EncryptionBackend: Trait defining the interface for encryption backends
- CrustyApp: Main application structure for the GUI

Main features:
- FileEncryption: Encrypts individual files using AES-256-GCM
- BatchProcessing: Processes multiple files in a batch operation
- KeyManagement: Generates, saves, and loads encryption keys
```

### Memory Update

When you discover new information about the codebase, update your memory:

```
I've learned about a new module in CRUSTy called logger.rs that handles operation logging. I should update my memory:

New entity:
- Name: logger.rs
- Type: Module
- Observations: 
  * Handles logging of encryption/decryption operations
  * Provides error and status message logging
  * Writes logs to a file in the user's data directory

New relations:
- CRUSTy contains logger.rs
- gui.rs depends_on logger.rs
- encryption.rs uses logger.rs for operation logging
```

### Daily Development Workflow

1. **Start with Memory Retrieval**
   - Before making any changes, retrieve relevant information from your memory
   - Understand the components and their relationships

2. **Analyze the Task**
   - What components will be affected?
   - What relationships need to be considered?
   - How does this fit into the existing architecture?

3. **Make Code Changes**
   - Implement your changes with a clear understanding of how they fit into the existing codebase

4. **Update Your Memory**
   - Add new entities, relations, or observations based on your changes
   - Ensure the memory stays consistent with the codebase

5. **Test and Document**
   - Test your changes thoroughly
   - Update documentation as needed

### Common Patterns

#### Adding a New Feature

1. Retrieve information about related components
2. Implement the feature
3. Create a new Feature entity
4. Create relations to existing components
5. Update affected components with new observations

#### Fixing a Bug

1. Retrieve information about the affected component
2. Fix the bug
3. Add observations to the affected components
4. Update relations if the fix changes dependencies

#### Adding a New Module

1. Retrieve information about related modules
2. Implement the new module
3. Create a new Module entity
4. Create relations to existing modules
5. Add any new features or data structures

## Practical Examples

### Example 1: Adding a New Feature

Let's say we want to add a new feature to CRUSTy for secure file deletion (wiping files securely after encryption).

#### Step 1: Memory Retrieval

```
Remembering...

From my memory about CRUSTy:

CRUSTy is a secure file encryption application using AES-256-GCM encryption. It's built with Rust and uses eframe for GUI. Key features include encryption/decryption of individual files, batch processing, key management, operation logging, and progress tracking.

Core modules:
- encryption.rs: Implements AES-256-GCM encryption and key management
- backend.rs: Provides abstraction for different encryption backends
- backend_local.rs: Implements local software-based encryption
- backend_embedded.rs: Implements embedded device encryption
- gui.rs: Implements the graphical user interface

I need to understand how file operations are handled to implement secure file deletion.
```

#### Step 2: Analyze Existing Code

After examining the codebase, we find that file operations are primarily handled in the encryption.rs module, and the GUI for file operations is in gui.rs.

#### Step 3: Implement the New Feature

We create a new function in encryption.rs:

```rust
/// Securely wipes a file by overwriting it with random data multiple times
/// before deletion to prevent recovery.
pub fn secure_delete_file(
    file_path: &Path,
    passes: usize,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), EncryptionError> {
    // Implementation details...
}
```

And we add a new option in the GUI in gui.rs:

```rust
// In the CrustyApp struct
pub secure_delete_enabled: bool,

// In the show_main_screen method
ui.checkbox(&mut self.secure_delete_enabled, "Securely delete original files after encryption");
```

#### Step 4: Update Memory

```
I've implemented a new feature in CRUSTy for secure file deletion. I should update my memory:

New entity:
- Name: SecureFileDeletion
- Type: Feature
- Observations: 
  * Securely wipes files by overwriting with random data multiple times
  * Prevents recovery of sensitive files after encryption
  * Configurable number of overwrite passes
  * Includes progress tracking

New relations:
- CRUSTy provides SecureFileDeletion
- encryption.rs defines secure_delete_file function
- SecureFileDeletion depends_on FileEncryption
- CrustyApp uses SecureFileDeletion
```

### Example 2: Fixing a Bug

Let's say we discover a bug in the recipient-specific encryption feature.

#### Step 1: Memory Retrieval

```
Remembering...

From my memory about CRUSTy:

The encryption.rs module supports recipient-specific encryption using HKDF. The EncryptionKey struct provides a method for_recipient that derives a recipient-specific key using HKDF with SHA-256.

The recipient-specific encryption is used in the encrypt_data_for_recipient and decrypt_data_with_recipient functions.
```

#### Step 2: Analyze the Bug

After investigation, we find that the bug is in the for_recipient method in the EncryptionKey struct. The email normalization doesn't handle international characters correctly.

#### Step 3: Fix the Bug

```rust
/// Derive a recipient-specific key using HKDF
pub fn for_recipient(&self, recipient_email: &str) -> EncryptionKey {
    // Normalize the email address - FIX: Use unicode normalization
    let normalized_email = recipient_email.trim().to_lowercase();
    let normalized_email = unicode_normalization::UnicodeNormalization::nfc(&normalized_email).collect::<String>();
    
    // Rest of the implementation...
}
```

#### Step 4: Update Memory

```
I've fixed a bug in the recipient-specific encryption feature. I should update my memory:

Update entity:
- Name: EncryptionKey
- Add observation: "Uses unicode normalization for handling international characters in recipient emails"

Update entity:
- Name: encryption.rs
- Add observation: "Depends on unicode-normalization crate for proper handling of international characters"
```

### Example 3: Adding a New Module

Let's say we want to add a new module for handling secure network transfers of encrypted files.

#### Step 1: Memory Retrieval

```
Remembering...

From my memory about CRUSTy:

CRUSTy currently handles local file encryption and decryption. It has modules for encryption (encryption.rs), backend abstraction (backend.rs), and GUI (gui.rs).

I need to understand how these modules interact to properly integrate a new network transfer module.
```

#### Step 2: Design the New Module

We decide to create a new module called network.rs that will handle secure file transfers over the network.

#### Step 3: Implement the New Module

```rust
// network.rs
/// Network module for secure transfer of encrypted files.
/// 
/// This module provides functionality for:
/// - Establishing secure connections
/// - Transferring encrypted files
/// - Verifying file integrity after transfer
use std::path::Path;
use crate::encryption::EncryptionError;

/// Error type for network operations
#[derive(Debug, Error)]
pub enum NetworkError {
    // Error types...
}

/// Securely transfers an encrypted file to a remote destination
pub fn transfer_file(
    file_path: &Path,
    destination: &str,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), NetworkError> {
    // Implementation details...
}

// More functions...
```

#### Step 4: Update Memory

```
I've added a new module to CRUSTy for secure network transfers. I should update my memory:

New entity:
- Name: network.rs
- Type: Module
- Observations: 
  * Handles secure transfer of encrypted files over the network
  * Provides functions for establishing secure connections
  * Includes verification of file integrity after transfer
  * Supports progress tracking during transfers

New entity:
- Name: SecureNetworkTransfer
- Type: Feature
- Observations:
  * Allows transferring encrypted files securely over the network
  * Verifies file integrity after transfer
  * Supports various network protocols

New relations:
- CRUSTy contains network.rs
- CRUSTy provides SecureNetworkTransfer
- network.rs depends_on encryption.rs
- network.rs defines NetworkError
- SecureNetworkTransfer uses FileEncryption
- gui.rs uses network.rs for transfer operations
```

## Best Practices

1. **Be Consistent**: Use the same entity and relation types consistently
2. **Be Specific**: Provide clear and specific observations
3. **Keep it Updated**: Update your memory whenever you make significant changes
4. **Think in Relationships**: Focus on how components relate to each other
5. **Use Before Changing**: Always retrieve and understand before making changes
6. **Always Begin with "Remembering..."**: When retrieving information, always start with this phrase
7. **Refer to the Knowledge Graph as "Memory"**: This maintains consistency in communication
