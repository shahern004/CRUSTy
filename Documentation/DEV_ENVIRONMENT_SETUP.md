# Development Environment Setup Guide

This guide provides instructions for setting up a development environment for the CRUSTy project on a new computer, mirroring the existing environment with all necessary tools and configurations.

## Table of Contents

- [Quick Setup (Automated)](#quick-setup-automated)
- [Manual Setup](#manual-setup)
- [MCP Server Configuration](#mcp-server-configuration)
- [Troubleshooting](#troubleshooting)

## Quick Setup (Automated)

For the fastest setup experience, you can use the provided PowerShell script that automates most of the installation and configuration process.

### Prerequisites

- Windows 10/11 with PowerShell
- Administrator privileges

### Setup Steps

1. Save the following script as `setup-dev-environment.ps1`:

```powershell
# setup-dev-environment.ps1

# 1. Install required software using winget (Windows Package Manager)
Write-Host "Installing required software..." -ForegroundColor Green
winget install Microsoft.VisualStudioCode
winget install Docker.DockerDesktop
winget install Git.Git

# 2. Create necessary directories
Write-Host "Creating directories..." -ForegroundColor Green
$mcp_dir = "$env:USERPROFILE\Documents\Cline\MCP"
New-Item -ItemType Directory -Force -Path $mcp_dir

# 3. Clone the repository
Write-Host "Cloning repository..." -ForegroundColor Green
$repo_dir = "$env:USERPROFILE\Shahern004 Github\CRUSTy"
New-Item -ItemType Directory -Force -Path (Split-Path $repo_dir)
git clone https://github.com/shahern004/CRUSTy.git $repo_dir

# 4. Pull Docker images
Write-Host "Pulling Docker images..." -ForegroundColor Green
docker pull mcp/git
docker pull mcp/filesystem
docker pull mcp/memory
docker pull mcp/sequentialthinking
docker pull mcp/puppeteer
docker pull mcp/sqlite
docker pull mcp/time

# 5. Install VSCode extensions
Write-Host "Installing VSCode extensions..." -ForegroundColor Green
code --install-extension saoudrizwan.claude-dev
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb

# 6. Configure MCP settings
Write-Host "Configuring MCP settings..." -ForegroundColor Green
$settings_dir = "$env:APPDATA\Code\User\globalStorage\saoudrizwan.claude-dev\settings"
New-Item -ItemType Directory -Force -Path $settings_dir

$mcp_settings = @"
{
  "mcpServers": {
    "git": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "--mount", "type=bind,src=$($repo_dir.Replace('\', '\\\\'))",dst=/projects/CRUSTy", "mcp/git"],
      "disabled": false,
      "autoApprove": []
    },
    "filesystem": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "--mount", "type=bind,src=$($mcp_dir.Replace('\', '\\\\'))\\\\filesystem,dst=/projects/filesystem", "--mount", "type=bind,src=$($repo_dir.Replace('\', '\\\\'))",dst=/projects/CRUSTy", "mcp/filesystem", "/projects"],
      "disabled": false,
      "autoApprove": []
    },
    "memory": {
      "command": "docker",
      "args": ["run", "-i", "-v", "claude-memory:/app/dist", "--rm", "mcp/memory"],
      "disabled": false,
      "autoApprove": []
    },
    "sequentialthinking": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "mcp/sequentialthinking"],
      "disabled": false,
      "autoApprove": []
    },
    "puppeteer": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "--init", "-e", "DOCKER_CONTAINER=true", "mcp/puppeteer"],
      "disabled": false,
      "autoApprove": []
    },
    "sqlite": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "--mount", "type=bind,src=$($mcp_dir.Replace('\', '\\\\'))\\\\sqlite,dst=/projects/sqlite", "mcp/sqlite"],
      "disabled": false,
      "autoApprove": []
    },
    "time": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "mcp/time"],
      "disabled": false,
      "autoApprove": []
    }
  }
}
"@

Set-Content -Path "$settings_dir\cline_mcp_settings.json" -Value $mcp_settings

# 7. Install Rust
Write-Host "Installing Rust..." -ForegroundColor Green
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile "$env:TEMP\rustup-init.exe"
Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait

Write-Host "Setup complete! Please restart your computer to ensure all changes take effect." -ForegroundColor Green
```

2. Open PowerShell as Administrator and run the script:

   ```
   .\setup-dev-environment.ps1
   ```

3. Restart your computer after the script completes.

4. Open VSCode and sign in to the Claude extension.

## Manual Setup

If you prefer to set up the environment manually or if the automated script doesn't work for your system, follow these steps:

### 1. Install Required Software

- [Visual Studio Code](https://code.visualstudio.com/download)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [Git](https://git-scm.com/downloads)
- [Rust](https://www.rust-lang.org/tools/install)

### 2. Install VSCode Extensions

Open VSCode and install the following extensions:

- Claude AI Assistant (saoudrizwan.claude-dev)
- Rust Analyzer (rust-lang.rust-analyzer)
- CodeLLDB (vadimcn.vscode-lldb)

### 3. Clone the Repository

```bash
mkdir -p "C:\Shahern004 Github"
cd "C:\Shahern004 Github"
git clone https://github.com/shahern004/CRUSTy.git
cd CRUSTy
```

### 4. Set Up MCP Servers

1. Create the necessary directories:

```bash
mkdir -p "C:\Users\Owner\Documents\Cline\MCP"
mkdir -p "C:\Users\Owner\Documents\Cline\MCP\filesystem"
mkdir -p "C:\Users\Owner\Documents\Cline\MCP\sqlite"
```

2. Pull the required Docker images:

```bash
docker pull mcp/git
docker pull mcp/filesystem
docker pull mcp/memory
docker pull mcp/sequentialthinking
docker pull mcp/puppeteer
docker pull mcp/sqlite
docker pull mcp/time
```

3. Configure the MCP settings file:
   - Create/edit this file: `C:\Users\Owner\AppData\Roaming\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json`
   - Add the configuration as shown in the automated script section.

## MCP Server Configuration

The MCP (Model Context Protocol) servers provide additional capabilities to Claude within VSCode. Here's a breakdown of each server's purpose:

- **git**: Provides Git operations directly from Claude
- **filesystem**: Allows Claude to interact with the file system
- **memory**: Provides persistent memory capabilities
- **sequentialthinking**: Enhances Claude's problem-solving abilities
- **puppeteer**: Enables browser automation
- **sqlite**: Provides database capabilities
- **time**: Offers time-related functions

## Troubleshooting

### Docker Issues

If you encounter issues with Docker:

1. Ensure Docker Desktop is running
2. Check that virtualization is enabled in your BIOS
3. For Windows Home users, ensure WSL2 is properly configured

### VSCode Extension Issues

If the Claude extension isn't working properly:

1. Check that all required MCP servers are properly configured
2. Restart VSCode
3. Check the extension logs for errors

### Path Issues

If you encounter path-related errors in the MCP configuration:

1. Ensure all paths use double backslashes (\\\\) in the JSON configuration
2. Verify that the paths exist on your system
3. Check for any special characters in paths that might need escaping

### Rust Setup Issues

If you encounter issues with Rust:

1. Run `rustup update` to ensure you have the latest version
2. Check that the Rust toolchain is properly installed with `rustc --version` and `cargo --version`
3. If needed, reinstall Rust using the installer from [rust-lang.org](https://www.rust-lang.org/tools/install)

## Additional Resources

- [VSCode Documentation](https://code.visualstudio.com/docs)
- [Docker Documentation](https://docs.docker.com/)
- [Rust Documentation](https://www.rust-lang.org/learn)
- [Claude Extension Documentation](https://claude.ai/docs)
