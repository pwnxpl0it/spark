// Embeds a VERSIONINFO resource (product name, version, author) into the
// Windows executable. Unsigned binaries that carry no metadata are a common
// false-positive trigger for Windows Defender and SmartScreen; this does not
// replace Authenticode signing but noticeably lowers the heuristic score.
// On every other platform this script is a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Spark")
            .set("FileDescription", "Spark - TOML-driven project initializer")
            .set("CompanyName", "Mohamed Tarek @pwnxpl0it")
            .set("LegalCopyright", "Copyright (c) Mohamed Tarek")
            .set("OriginalFilename", "spark.exe")
            .set("InternalName", "spark");
        if let Err(err) = res.compile() {
            println!("cargo:warning=failed to embed Windows version info: {err}");
        }
    }
}
