#![no_std]

use core::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub struct FirmwareVersion<'a> {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub prerelease: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct ParseFirmwareError {}

impl From<ParseIntError> for ParseFirmwareError {
    fn from(_: ParseIntError) -> Self {
        Self {}
    }
}

impl<'a> FirmwareVersion<'a> {
    pub fn parse(text: &'a str) -> Result<Self, ParseFirmwareError> {
        let (version, prerelease) = match text.split_once('-') {
            None => (text, ""),
            Some((a, b)) => (a, b),
        };

        let mut iter = version.split('.');

        let major = iter.next().ok_or(ParseFirmwareError {})?.parse()?;
        let minor = iter.next().ok_or(ParseFirmwareError {})?.parse()?;
        let patch = iter.next().ok_or(ParseFirmwareError {})?.parse()?;

        if iter.next().is_some() {
            return Err(ParseFirmwareError {});
        }

        let prerelease = if prerelease.is_empty() {
            None
        } else {
            Some(prerelease)
        };

        Ok(FirmwareVersion {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_major_minor_patch() {
        let result = FirmwareVersion::parse("42.7.127").unwrap();
        assert_eq!(result.major, 42);
        assert_eq!(result.minor, 7);
        assert_eq!(result.patch, 127);
        assert_eq!(result.prerelease, None);
    }

    #[test]
    fn parse_zeros() {
        let result = FirmwareVersion::parse("0.0.0").unwrap();
        assert_eq!(result.major, 0);
        assert_eq!(result.minor, 0);
        assert_eq!(result.patch, 0);
        assert_eq!(result.prerelease, None);
    }

    #[test]
    fn parse_major_minor_patch_prerelease() {
        let result = FirmwareVersion::parse("42.7.127-extra-stuff").unwrap();
        assert_eq!(result.major, 42);
        assert_eq!(result.minor, 7);
        assert_eq!(result.patch, 127);
        assert_eq!(result.prerelease, Some("extra-stuff"));
    }

    #[test]
    fn parse_prerelease_with_dots() {
        let result = FirmwareVersion::parse("42.7.127-extra.stuff").unwrap();
        assert_eq!(result.major, 42);
        assert_eq!(result.minor, 7);
        assert_eq!(result.patch, 127);
        assert_eq!(result.prerelease, Some("extra.stuff"));
    }

    #[test]
    fn error_if_too_few_dots() {
        let result = FirmwareVersion::parse("1.2");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_too_many_dots() {
        let result = FirmwareVersion::parse("1.2.3.4");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_major_empty() {
        let result = FirmwareVersion::parse(".2.3");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_minor_empty() {
        let result = FirmwareVersion::parse("1..3");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_patch_empty() {
        let result = FirmwareVersion::parse("1.2.");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_major_not_numeric() {
        let result = FirmwareVersion::parse("a.2.3");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_minor_not_numeric() {
        let result = FirmwareVersion::parse("1.b.3");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }

    #[test]
    fn error_if_patch_not_numeric() {
        let result = FirmwareVersion::parse("1.2.c");
        assert_eq!(result, Err(ParseFirmwareError {}));
    }
}
