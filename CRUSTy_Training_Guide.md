# CRUSTy Training Guide for Security Engineers

## Introduction

Welcome to CRUSTy, your new file security companion! This guide will walk you through the key features and functionality of our secure file encryption application. CRUSTy is designed to protect sensitive files using strong security methods while remaining user-friendly.

## What is CRUSTy?

CRUSTy is a desktop application that locks your files away from prying eyes. Think of it as a digital safe for your important documents. When you encrypt a file with CRUSTy, it transforms readable data into a scrambled format that can only be unscrambled with the correct key.

The application uses a security method called AES-256-GCM (Advanced Encryption Standard with 256-bit keys in Galois/Counter Mode). This is like having a lock that has 2^256 possible key combinations - a number so large that even the most powerful computers would need billions of years to try all possibilities.

## Core Features

### Basic File Protection

At its simplest, CRUSTy lets you:

1. Select files you want to protect
2. Choose or create a secret key
3. Lock (encrypt) your files so only someone with the key can access them
4. Unlock (decrypt) files when you need to use them again

Think of encryption like putting your document in a secure lockbox. Only people with the right key can open it and see what's inside.

### Key Management

Keys in CRUSTy are like physical keys to a safe - if you lose them, you can't access your protected files. The application helps you manage these keys by:

- Creating new keys with custom names
- Saving keys to files for backup
- Loading keys from files when needed
- Managing multiple keys for different purposes

Each key is a random string of characters that would be impossible to guess. Behind the scenes, CRUSTy stores these keys in a secure format (base64-encoded) and can save them to files that you should keep in a safe place.

### Recipient-Specific Encryption

Sometimes you want to send protected files to specific people. CRUSTy has a special feature for this:

1. You enter the recipient's email address
2. CRUSTy creates a unique key just for that recipient
3. The file is locked with this special key
4. The recipient's email is stored within the encrypted file

This works like a mailbox system - you're creating a special mailbox that only a specific person can open. When they try to decrypt the file, CRUSTy automatically detects it was meant for them.

Behind the scenes, CRUSTy uses the recipient's email to create a unique key through a process called key derivation (HKDF with SHA-256). This means both you and the recipient need to know the master key, but the file is additionally tied to that specific email address.

## Special Feature: Hardware Security Integration

### What is Hardware Security?

CRUSTy has a special ability to work with dedicated security hardware - specifically, small specialized computers called embedded systems. Think of these as security co-processors that can handle the encryption work separately from your main computer.

The specific hardware CRUSTy works with is called an STM32H5 device. This is like having a small, dedicated security guard for your encryption operations instead of handling everything on your main computer.

### Why Use Hardware Security?

Using dedicated hardware for security operations offers several benefits:

1. **Speed**: It's like having a math whiz do your calculations - hardware designed specifically for encryption can do it much faster than your computer's general-purpose processor.

2. **Security Isolation**: Imagine keeping your valuables in a separate safe rather than in your desk drawer. Hardware security keeps encryption operations physically separate from your main computer, making it harder for attackers to steal keys or intercept data.

3. **Efficiency**: Your computer can focus on other tasks while the security hardware handles the encryption work.

### How It Works

The process works like this:

1. You connect a special security device (STM32H573I-DK) to your computer
2. You tell CRUSTy to use this device for encryption operations
3. When you encrypt or decrypt files, CRUSTy sends the data to this device
4. The device performs the encryption and sends back the results
5. Your computer never sees the raw encryption keys

This is similar to how modern payment terminals work - your credit card details are processed in a separate secure element rather than on the main system.

### Setting Up Hardware Security

To use this feature:

1. Connect your security device to your computer
2. In CRUSTy, check "Use embedded system for cryptographic operations"
3. Select how you're connecting to the device (USB, Serial, or Network)
4. Enter the device's address or ID
5. Click "Apply Configuration"

Now all your encryption operations will use the hardware device instead of your computer's processor.

### Connection Types Explained

CRUSTy supports three ways to connect to security hardware:

- **USB**: Like plugging in a secure USB drive. This is the simplest method - just plug in and go.

- **Serial**: This is like having a private phone line directly to the security device. It's a bit more technical but can be more reliable in some environments.

- **Network**: This lets you connect to a security device over your local network, like accessing a network printer. Useful when the device can't be directly connected to your computer.

## Security Concepts

### How CRUSTy Protects Your Data

CRUSTy uses several layers of protection:

1. **Strong Encryption**: The AES-256-GCM method is like having an extremely complex combination lock that changes the combination for every file.

2. **Authentication**: CRUSTy doesn't just encrypt your data; it also adds a security seal. If anyone tampers with the encrypted file, the seal breaks, and CRUSTy will warn you.

3. **Unique Identifiers**: Each encryption operation uses a random number called a nonce (number used once). This is like adding a unique serial number to each locked box, ensuring that even if you encrypt the same file twice, the results will look completely different.

4. **Email-Based Keys**: When using recipient-specific encryption, CRUSTy creates a unique key based on the email address. This is like having a lock that checks both the key and the person's ID before opening.

### What CRUSTy Doesn't Protect Against

It's important to understand the limitations:

1. **Lost Keys**: If you lose your encryption key, there is no way to recover your data. Always keep backups of your keys in a secure location.

2. **Compromised Computers**: If your computer has malware or spyware, it might be able to capture your keys or see your data before it's encrypted.

3. **Physical Access**: If someone has physical access to your computer while you're using CRUSTy with an unlocked key, they might be able to access your files.

## Troubleshooting Common Issues

### "Destination file already exists"

CRUSTy won't overwrite existing files for safety. Delete the existing file or choose a different output directory.

### "Authentication failed" or "Wrong encryption key"

This means either:

- You're using the wrong key
- The file has been tampered with
- The file is corrupted

Try a different key if you have one, or check if the file was transferred correctly.

### "Embedded backend not implemented"

This means you're trying to use the hardware security feature, but:

- You haven't completed the configuration
- The device isn't properly connected
- The device isn't powered on

Make sure you've clicked "Apply Configuration" after entering all the device details.

### "Failed to connect to embedded device"

Check that:

- The device ID/address is correct
- All physical connections are secure
- The device is powered on
- The device has the correct firmware installed

### "Communication error with embedded device"

The connection was established but was interrupted. Check for:

- Loose connections
- Power issues with the device
- Try with smaller files if you're processing very large ones

## Practical Exercises

### Basic Encryption

1. Launch CRUSTy
2. Click "Create New Key" and name it "Training Key"
3. Select a test document
4. Choose an output directory
5. Click "Encrypt"
6. Verify that the encrypted file appears in your output directory

### Decryption

1. Select the encrypted file you just created
2. Choose an output directory
3. Make sure "Training Key" is selected
4. Click "Decrypt"
5. Verify that the decrypted file matches the original

### Recipient-Specific Encryption

1. Check "Use recipient-specific encryption"
2. Enter an email address (e.g., "training@example.com")
3. Select a test document
4. Choose an output directory
5. Click "Encrypt"
6. Try decrypting the file - notice that CRUSTy shows the recipient email

## Conclusion

CRUSTy provides strong, user-friendly file protection with the option of hardware-accelerated security. By understanding both the software and hardware components, you can make the most of its features while maintaining a high level of security.

Remember that security is only as strong as its weakest link - keep your keys safe, be mindful of your environment, and follow good security practices in addition to using encryption tools like CRUSTy.
