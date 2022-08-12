use super::date::{DayOfMonth, GregorianDate, GregorianDateBuilder, Month, Year};

struct Mask {
    position: u8,
    size: u8,
}

impl Mask {
    const fn max_value(&self) -> u64 {
        (1 << self.size) - 1
    }

    const fn mask(&self) -> u64 {
        self.max_value() << self.position
    }

    const fn extract_value(&self, value: u64) -> u64 {
        (value & self.mask()) >> self.position
    }

    const fn insert_value(&self, value: u64) -> u64 {
        (value & self.max_value()) << self.position
    }
}

const PLATFORM_MASK: Mask = Mask {
    position: 0,
    size: 2,
};
const DATE_MASK: Mask = Mask {
    position: PLATFORM_MASK.position + PLATFORM_MASK.size,
    size: 16,
};
const MINOR_VERSION_MASK: Mask = Mask {
    position: DATE_MASK.position + DATE_MASK.size,
    size: 7,
};
const MAJOR_VERSION_MASK: Mask = Mask {
    position: MINOR_VERSION_MASK.position + MINOR_VERSION_MASK.size,
    size: 6,
};
const MIN_DATE: GregorianDate = match GregorianDateBuilder::new()
    .year(2000)
    .month_and_day(12, 21)
    .build()
{
    Ok(date) => date,
    Err(_) => panic!("Bad GregorianDate"),
};
const MAX_DATE: GregorianDate = match GregorianDateBuilder::new()
    .year(2099)
    .month_and_day(12, 31)
    .build()
{
    Ok(date) => date,
    Err(_) => panic!("Bad GregorianDate"),
};
const DATE_MOD: u16 = 367;
const DATE_REF_YEAR: Year = 2000;
const MAJOR_VERSION_DEBUG: MajorVersion = 9;
const MAJOR_VERSION_MAX: MajorVersion = 7;

type MajorVersion = u8;
type MinorVersion = u8;
type Platform = u8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Version {
    major_version: MajorVersion,
    minor_version: MinorVersion,
    date: GregorianDate,
    platform: Platform,
}

pub struct NormalFormatVersion(u64);

pub struct DateFormatVersion(u64);

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidMajorVersion,
    InvalidMinorVersion,
    InvalidDate,
    InvalidPlatform,
    VersionDateMismatch,
}

impl Version {
    pub fn new(
        major_version: MajorVersion,
        minor_version: MinorVersion,
        date: GregorianDate,
        platform: Platform,
    ) -> Result<Version, Error> {
        if MAJOR_VERSION_MAX < major_version && MAJOR_VERSION_DEBUG != major_version {
            return Err(Error::InvalidMajorVersion);
        }

        if MINOR_VERSION_MASK.max_value() < minor_version as u64 {
            return Err(Error::InvalidMinorVersion);
        }

        if MIN_DATE > date || MAX_DATE < date {
            return Err(Error::InvalidDate);
        }

        if PLATFORM_MASK.max_value() < platform as u64 {
            return Err(Error::InvalidPlatform);
        }

        if (4 >= major_version && 2011 < date.year())
            || (5 == major_version && 2006 > date.year())
            || (6 == major_version && 2012 > date.year())
            || (7 == major_version && 2018 > date.year())
        {
            return Err(Error::VersionDateMismatch);
        }

        Ok(Version {
            major_version,
            minor_version,
            date,
            platform,
        })
    }

    pub fn major_version(&self) -> u8 {
        self.major_version
    }

    pub fn minor_version(&self) -> u8 {
        self.minor_version
    }

    pub fn date(&self) -> &GregorianDate {
        &self.date
    }

    pub fn platform(&self) -> u8 {
        self.platform
    }
}

impl TryFrom<NormalFormatVersion> for Version {
    type Error = Error;

    fn try_from(NormalFormatVersion(value): NormalFormatVersion) -> Result<Self, Self::Error> {
        let major_version: MajorVersion =
            MAJOR_VERSION_MASK.extract_value(value).try_into().unwrap();
        let minor_version: MinorVersion =
            MINOR_VERSION_MASK.extract_value(value).try_into().unwrap();
        let platform: Platform = PLATFORM_MASK.extract_value(value).try_into().unwrap();
        let raw_date: u16 = DATE_MASK.extract_value(value).try_into().unwrap();
        let date = match GregorianDateBuilder::new()
            .year((raw_date / DATE_MOD) + DATE_REF_YEAR)
            .day_of_year(raw_date % DATE_MOD)
            .build()
        {
            Ok(date) => date,
            Err(_) => {
                return Err(Error::InvalidDate);
            }
        };
        Version::new(major_version, minor_version, date, platform)
    }
}

impl Into<NormalFormatVersion> for Version {
    fn into(self) -> NormalFormatVersion {
        let mut ret = NormalFormatVersion(0);
        ret.0 = ret.0 ^ MAJOR_VERSION_MASK.insert_value(self.major_version() as u64);
        ret.0 = ret.0 ^ MINOR_VERSION_MASK.insert_value(self.minor_version() as u64);
        ret.0 = ret.0 ^ PLATFORM_MASK.insert_value(self.platform() as u64);
        ret.0 = ret.0
            ^ DATE_MASK.insert_value(
                ((self.date().year() - DATE_REF_YEAR) as u64 * DATE_MOD as u64)
                    + self.date().day_of_year() as u64,
            );
        ret
    }
}

impl TryFrom<DateFormatVersion> for Version {
    type Error = Error;

    fn try_from(DateFormatVersion(value): DateFormatVersion) -> Result<Self, Self::Error> {
        let major_version: MajorVersion = if 200612060 == value {
            5
        } else {
            (value % 10).try_into().unwrap()
        };
        let day: DayOfMonth = ((value / 10) % 100).try_into().unwrap();
        let month: Month = ((value / (10 * 100)) % 100).try_into().unwrap();
        let year: Year = (value / (10 * 100 * 100)).try_into().unwrap();
        let date = match GregorianDateBuilder::new()
            .year(year)
            .month_and_day(month, day)
            .build()
        {
            Ok(date) => date,
            Err(_) => {
                return Err(Error::InvalidDate);
            }
        };
        Version::new(major_version, 0, date, 0)
    }
}

impl Into<DateFormatVersion> for Version {
    fn into(self) -> DateFormatVersion {
        let mut ret = DateFormatVersion(0);
        ret.0 = self.major_version() as u64;
        ret.0 = ret.0 + (self.date().day_of_month() as u64 * 10);
        ret.0 = ret.0 + (self.date().month() as u64 * 10 * 100);
        ret.0 = ret.0 + (self.date().year() as u64 * 10 * 100 * 100);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask() {
        let mut mask = Mask {
            position: 0,
            size: 0,
        };
        assert_eq!(0, mask.max_value());
        assert_eq!(0, mask.mask());
        assert_eq!(0, mask.extract_value(0b11));
        assert_eq!(0, mask.insert_value(0b11));

        mask = Mask {
            position: 1,
            size: 0,
        };
        assert_eq!(0, mask.max_value());
        assert_eq!(0, mask.mask());
        assert_eq!(0, mask.extract_value(0b11));
        assert_eq!(0, mask.insert_value(0b11));

        mask = Mask {
            position: 0,
            size: 1,
        };
        assert_eq!(1, mask.max_value());
        assert_eq!(0b01, mask.mask());
        assert_eq!(0b01, mask.extract_value(0b01));
        assert_eq!(0b00, mask.extract_value(0b10));
        assert_eq!(0b01, mask.insert_value(0b01));
        assert_eq!(0b00, mask.insert_value(0b10));

        mask = Mask {
            position: 1,
            size: 1,
        };
        assert_eq!(1, mask.max_value());
        assert_eq!(0b10, mask.mask());
        assert_eq!(0b00, mask.extract_value(0b01));
        assert_eq!(0b01, mask.extract_value(0b10));
        assert_eq!(0b10, mask.insert_value(0b01));
        assert_eq!(0b00, mask.insert_value(0b10));

        mask = Mask {
            position: 1,
            size: 2,
        };
        assert_eq!(3, mask.max_value());
        assert_eq!(0b110, mask.mask());
        assert_eq!(0b011, mask.extract_value(0b110));
        assert_eq!(0b000, mask.extract_value(0b001));
        assert_eq!(0b001, mask.extract_value(0b011));
        assert_eq!(0b110, mask.insert_value(0b011));
        assert_eq!(0b000, mask.insert_value(0b100));
        assert_eq!(0b100, mask.insert_value(0b010));
    }

    #[test]
    fn valid_version() {
        let mut date = MIN_DATE;
        for major_version in 0..=4 {
            assert!(Version::new(major_version, 0, date, 0).is_ok());
        }
        date = GregorianDateBuilder::new().year(2006).build().unwrap();
        assert!(Version::new(5, 0, date, 0).is_ok());

        date = GregorianDateBuilder::new().year(2012).build().unwrap();
        assert!(Version::new(6, 0, date, 0).is_ok());

        date = GregorianDateBuilder::new().year(2018).build().unwrap();
        assert!(Version::new(7, 127, date, 3).is_ok());
    }

    #[test]
    fn invalid_major_version() {
        assert_eq!(
            Version::new(8, 0, MIN_DATE, 0).err(),
            Some(Error::InvalidMajorVersion)
        );
    }

    #[test]
    fn invalid_minor_version() {
        assert_eq!(
            Version::new(0, 128, MIN_DATE, 0).err(),
            Some(Error::InvalidMinorVersion)
        );
    }

    #[test]
    fn invalid_date() {
        assert_eq!(
            Version::new(
                0,
                0,
                GregorianDateBuilder::new()
                    .year(2000)
                    .month_and_day(12, 20)
                    .build()
                    .unwrap(),
                0
            )
            .err(),
            Some(Error::InvalidDate)
        );

        assert_eq!(
            Version::new(
                0,
                0,
                GregorianDateBuilder::new()
                    .year(2100)
                    .month_and_day(1, 1)
                    .build()
                    .unwrap(),
                0
            )
            .err(),
            Some(Error::InvalidDate)
        );
    }

    #[test]
    fn invalid_platform() {
        assert_eq!(
            Version::new(0, 0, MIN_DATE, 4).err(),
            Some(Error::InvalidPlatform)
        );
    }

    #[test]
    fn mismatch_version_date() {
        let mut date = GregorianDateBuilder::new().year(2012).build().unwrap();
        for major_version in 0..=4 {
            assert_eq!(
                Version::new(major_version, 0, date, 0).err(),
                Some(Error::VersionDateMismatch)
            );
        }
        date = GregorianDateBuilder::new().year(2005).build().unwrap();
        assert_eq!(
            Version::new(5, 0, date, 0).err(),
            Some(Error::VersionDateMismatch)
        );

        date = GregorianDateBuilder::new().year(2011).build().unwrap();
        assert_eq!(
            Version::new(6, 0, date, 0).err(),
            Some(Error::VersionDateMismatch)
        );

        date = GregorianDateBuilder::new().year(2017).build().unwrap();
        assert_eq!(
            Version::new(7, 0, date, 0).err(),
            Some(Error::VersionDateMismatch)
        );
    }

    #[test]
    fn observers() {
        let version = Version::new(
            9,
            1,
            GregorianDateBuilder::new()
                .year(2002)
                .month_and_day(10, 27)
                .build()
                .unwrap(),
            2,
        )
        .unwrap();
        assert_eq!(9, version.major_version());
        assert_eq!(1, version.minor_version());
        assert_eq!(2002, version.date().year());
        assert_eq!(10, version.date().month());
        assert_eq!(27, version.date().day_of_month());
        assert_eq!(2, version.platform());
    }

    #[test]
    fn conversions() {
        let mut initial_version = Version::new(0, 0, MIN_DATE, 0).unwrap();
        let mut normal_format: NormalFormatVersion = initial_version.into();
        let mut date_format: DateFormatVersion = initial_version.into();

        let mut final_version = Version::try_from(normal_format).unwrap();
        assert_eq!(initial_version, final_version);

        final_version = Version::try_from(date_format).unwrap();
        assert_eq!(initial_version, final_version);

        initial_version = Version::new(9, 1, MAX_DATE, 1).unwrap();
        normal_format = initial_version.into();
        date_format = initial_version.into();

        final_version = Version::try_from(normal_format).unwrap();
        assert_eq!(initial_version, final_version);

        final_version = Version::try_from(date_format).unwrap();
        let initial_version_simplified = Version::new(9, 0, MAX_DATE, 0).unwrap();
        assert_eq!(initial_version_simplified, final_version);
    }
}
