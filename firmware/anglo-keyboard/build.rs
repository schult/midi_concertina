use regex::Regex;
use std::process::Command;

fn main() {
    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    let version = match Command::new("git").args(["describe", "--tags", "--match", "firmware/v*"]).output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_owned(),
        Ok(o) => panic!("git-describe exited non-zero: {}", o.status),
        Err(err) => panic!("failed to execute git-describe: {err}"),
    };

    let re = Regex::new(r"^firmware/v(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:-(?P<prerelease>.*))?$").unwrap();
    let caps = re.captures(&version).unwrap();
    let major = caps.name("major").unwrap().as_str();
    let minor = caps.name("minor").unwrap().as_str();
    let patch = caps.name("patch").unwrap().as_str();

    println!("cargo:rustc-env=FIRMWARE_MAJOR_VERSION={major}");
    println!("cargo:rustc-env=FIRMWARE_MINOR_VERSION={minor}");
    println!("cargo:rustc-env=FIRMWARE_PATCH_VERSION={patch}");

    if let Some(prerelease) = caps.name("prerelease") {
        println!("cargo:rustc-env=FIRMWARE_PRERELEASE_VERSION={}", prerelease.as_str());
    }
}
