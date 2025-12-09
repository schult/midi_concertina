use std::process::Command;
use version::FirmwareVersion;

fn main() {
    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    let version_text = match Command::new("git").args(["describe", "--tags", "--match", "firmware/v*"]).output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_owned(),
        Ok(o) => panic!("git-describe exited non-zero: {}", o.status),
        Err(err) => panic!("failed to execute git-describe: {err}"),
    };
    let version_text = version_text.strip_prefix("firmware/v").unwrap();
    let version = FirmwareVersion::parse(version_text).unwrap();

    println!("cargo:rustc-env=FIRMWARE_MAJOR_VERSION={}", version.major);
    println!("cargo:rustc-env=FIRMWARE_MINOR_VERSION={}", version.minor);
    println!("cargo:rustc-env=FIRMWARE_PATCH_VERSION={}", version.patch);
    if let Some(prerelease) = version.prerelease {
        println!("cargo:rustc-env=FIRMWARE_PRERELEASE_VERSION={prerelease}");
    }
}
