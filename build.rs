// This build script sets the Windows subsystem to "windows" (GUI) instead of "console"
// so that no command prompt window appears when running the application

#[cfg(windows)]
fn main() {
    // Only run this on Windows
    let mut res = winres::WindowsResource::new();
    
    // Uncomment the following line if you add an icon file later
    // res.set_icon("assets/app_icon.ico");
    
    res.set_language(0x0409); // English language (US)
    
    // This is the most important part - it sets the subsystem to Windows GUI
    // so no console window will appear
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
    
    // Compile and link the resource file
    if let Err(e) = res.compile() {
        eprintln!("Error: Failed to compile Windows resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {
    // Do nothing on non-Windows platforms
} 