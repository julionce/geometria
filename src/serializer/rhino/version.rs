use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Version {
    V1,
    V2,
    V3,
    V4,
    V50,
    V60,
    V70,
}

#[derive(Debug, PartialEq)]
pub enum VersionError {
    InvalidVersion,
}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVersion => write!(f, "invalid version"),
        }
    }
}

impl TryFrom<u8> for Version {
    type Error = VersionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Version::V1),
            2 => Ok(Version::V2),
            3 => Ok(Version::V3),
            4 => Ok(Version::V4),
            50 => Ok(Version::V50),
            60 => Ok(Version::V60),
            70 => Ok(Version::V70),
            _ => Err(VersionError::InvalidVersion),
        }
    }
}

impl Into<u8> for Version {
    fn into(self) -> u8 {
        match self {
            Version::V1 => 1,
            Version::V2 => 2,
            Version::V3 => 3,
            Version::V4 => 4,
            Version::V50 => 50,
            Version::V60 => 60,
            Version::V70 => 70,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let mut version = Version::V1;
        assert_eq!(1u8, version.into());
        version = Version::V2;
        assert_eq!(2u8, version.into());
        version = Version::V3;
        assert_eq!(3u8, version.into());
        version = Version::V4;
        assert_eq!(4u8, version.into());
        version = Version::V50;
        assert_eq!(50u8, version.into());
        version = Version::V60;
        assert_eq!(60u8, version.into());
        version = Version::V70;
        assert_eq!(70u8, version.into());

        assert_eq!(Version::try_from(1u8).ok(), Some(Version::V1));
        assert_eq!(Version::try_from(2u8).ok(), Some(Version::V2));
        assert_eq!(Version::try_from(3u8).ok(), Some(Version::V3));
        assert_eq!(Version::try_from(4u8).ok(), Some(Version::V4));
        assert_eq!(Version::try_from(50u8).ok(), Some(Version::V50));
        assert_eq!(Version::try_from(60u8).ok(), Some(Version::V60));
        assert_eq!(Version::try_from(70u8).ok(), Some(Version::V70));
        assert_eq!(
            Version::try_from(0u8).err(),
            Some(VersionError::InvalidVersion)
        );
    }
}
