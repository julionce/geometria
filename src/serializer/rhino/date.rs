use std::fmt::Display;

pub type Year = u16;
pub type Month = u8;
pub type DayOfMonth = u8;
pub type DayOfYear = u16;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GregorianDate {
    year: Year,
    month: Month,
    day_of_month: DayOfMonth,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidYear,
    InvalidMonth,
    InvalidDayOfMonth,
    InvalidDayOfYear,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidYear => write!(f, "invalid year, it must be greater than 1582"),
            Self::InvalidMonth => write!(f, "invalid month, it must be in the 1..=12 range"),
            Self::InvalidDayOfMonth => write!(f, "invalid day of the month"),
            Self::InvalidDayOfYear => write!(f, "invalid day of the year"),
        }
    }
}

impl GregorianDate {
    const FIRST_YEAR: Year = 1582;

    pub const fn year(&self) -> Year {
        self.year
    }

    pub const fn month(&self) -> Month {
        self.month
    }

    pub const fn day_of_month(&self) -> DayOfMonth {
        self.day_of_month
    }

    pub const fn day_of_year(&self) -> DayOfYear {
        let extra_day = if self.is_leap_year() { 1u16 } else { 0u16 };
        match self.month {
            1 => self.day_of_month as DayOfYear,
            2 => 31 + self.day_of_month as DayOfYear,
            3 => 59 + self.day_of_month as DayOfYear + extra_day,
            4 => 90 + self.day_of_month as DayOfYear + extra_day,
            5 => 120 + self.day_of_month as DayOfYear + extra_day,
            6 => 151 + self.day_of_month as DayOfYear + extra_day,
            7 => 181 + self.day_of_month as DayOfYear + extra_day,
            8 => 212 + self.day_of_month as DayOfYear + extra_day,
            9 => 243 + self.day_of_month as DayOfYear + extra_day,
            10 => 273 + self.day_of_month as DayOfYear + extra_day,
            11 => 304 + self.day_of_month as DayOfYear + extra_day,
            12 => 334 + self.day_of_month as DayOfYear + extra_day,
            _ => 0,
        }
    }

    pub const fn is_leap_year(&self) -> bool {
        (1624 <= self.year)
            && (0 == (self.year % 4))
            && (0 == (self.year % 400) || 0 != (self.year % 100))
    }

    pub const fn month_days(&self) -> DayOfMonth {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    pub const fn year_days(&self) -> DayOfYear {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }

    pub const fn first_of_month(&self) -> GregorianDate {
        let mut date = *self;
        date.day_of_month = 1;
        date
    }

    pub const fn last_of_month(&self) -> GregorianDate {
        let mut date = *self;
        date.day_of_month = self.month_days();
        date
    }
}

enum InternalDate {
    MonthAndDay(Month, DayOfMonth),
    DayOfYear(DayOfYear),
}

pub struct GregorianDateBuilder {
    year: Year,
    date: InternalDate,
}

impl GregorianDateBuilder {
    pub const fn new() -> Self {
        GregorianDateBuilder {
            year: GregorianDate::FIRST_YEAR,
            date: InternalDate::MonthAndDay(1, 1),
        }
    }

    pub const fn year(mut self, year: Year) -> Self {
        self.year = year;
        self
    }

    pub const fn month_and_day(mut self, month: Month, day: DayOfMonth) -> Self {
        self.date = InternalDate::MonthAndDay(month, day);
        self
    }

    pub const fn day_of_year(mut self, day_of_year: DayOfYear) -> Self {
        self.date = InternalDate::DayOfYear(day_of_year);
        self
    }

    pub const fn build(&self) -> Result<GregorianDate, Error> {
        if GregorianDate::FIRST_YEAR > self.year {
            return Err(Error::InvalidYear);
        }
        let mut date = GregorianDate {
            year: self.year,
            month: 1,
            day_of_month: 1,
        };
        match self.date {
            InternalDate::MonthAndDay(month, day) => {
                date.month = month;
                date.day_of_month = day;
                if 1 > month || 12 < month {
                    return Err(Error::InvalidMonth);
                }
                if 1 > day || date.month_days() < day {
                    return Err(Error::InvalidDayOfMonth);
                }
            }
            InternalDate::DayOfYear(day_of_year) => {
                if 1 > day_of_year || date.year_days() < day_of_year {
                    return Err(Error::InvalidDayOfYear);
                }
                date.month = (day_of_year / 30) as Month;
                date = Self::to_month_and_day(day_of_year, date);
            }
        }
        Ok(date)
    }

    const fn to_month_and_day(day_of_year: DayOfYear, seed: GregorianDate) -> GregorianDate {
        let mut date = seed;
        if day_of_year < date.first_of_month().day_of_year() {
            date.month = date.month - 1;
            Self::to_month_and_day(day_of_year, date)
        } else if day_of_year > date.last_of_month().day_of_year() {
            date.month = date.month + 1;
            Self::to_month_and_day(day_of_year, date)
        } else {
            date.day_of_month =
                date.day_of_month + (day_of_year - date.day_of_year()) as DayOfMonth;
            date
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_build() {
        assert_eq!(
            GregorianDateBuilder::new().build().ok(),
            Some(GregorianDate {
                year: 1582,
                month: 1,
                day_of_month: 1
            })
        );
    }

    #[test]
    fn valid_build() {
        assert_eq!(
            GregorianDateBuilder::new()
                .year(1989)
                .month_and_day(11, 11)
                .build()
                .ok(),
            Some(GregorianDate {
                year: 1989,
                month: 11,
                day_of_month: 11
            })
        );
    }

    #[test]
    fn valid_year() {
        assert!(GregorianDateBuilder::new().year(1582).build().is_ok());
    }

    #[test]
    fn invalid_year() {
        assert_eq!(
            GregorianDateBuilder::new().year(1581).build().err(),
            Some(Error::InvalidYear)
        );
    }

    #[test]
    fn valid_month() {
        for month in 1..=12 {
            assert!(GregorianDateBuilder::new()
                .month_and_day(month, 1)
                .build()
                .is_ok());
        }
    }

    #[test]
    fn invalid_month() {
        assert_eq!(
            GregorianDateBuilder::new()
                .month_and_day(0, 1)
                .build()
                .err(),
            Some(Error::InvalidMonth)
        );
        assert_eq!(
            GregorianDateBuilder::new()
                .month_and_day(13, 1)
                .build()
                .err(),
            Some(Error::InvalidMonth)
        );
    }

    #[test]
    fn valid_day_of_month() {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            for day in 1..=31 {
                assert!(GregorianDateBuilder::new()
                    .month_and_day(month, day)
                    .build()
                    .is_ok());
            }
        }

        for month in [4, 6, 9, 1] {
            for day in 1..=30 {
                assert!(GregorianDateBuilder::new()
                    .month_and_day(month, day)
                    .build()
                    .is_ok());
            }
        }

        for day in 1..=28 {
            assert!(GregorianDateBuilder::new()
                .month_and_day(2, day)
                .build()
                .is_ok());
        }

        for day in 1..=29 {
            assert!(GregorianDateBuilder::new()
                .year(1624)
                .month_and_day(2, day)
                .build()
                .is_ok());
        }
    }

    #[test]
    fn invalid_day_of_month() {
        for month in 1..=12 {
            assert_eq!(
                GregorianDateBuilder::new()
                    .month_and_day(month, 0)
                    .build()
                    .err(),
                Some(Error::InvalidDayOfMonth)
            );
        }
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(
                GregorianDateBuilder::new()
                    .month_and_day(month, 32)
                    .build()
                    .err(),
                Some(Error::InvalidDayOfMonth)
            );
        }

        for month in [4, 6, 9, 11] {
            assert_eq!(
                GregorianDateBuilder::new()
                    .month_and_day(month, 31)
                    .build()
                    .err(),
                Some(Error::InvalidDayOfMonth)
            );
        }

        assert_eq!(
            GregorianDateBuilder::new()
                .month_and_day(2, 29)
                .build()
                .err(),
            Some(Error::InvalidDayOfMonth)
        );
        assert_eq!(
            GregorianDateBuilder::new()
                .year(1624)
                .month_and_day(2, 30)
                .build()
                .err(),
            Some(Error::InvalidDayOfMonth)
        );
    }

    #[test]
    fn invalid_day_of_year() {
        assert_eq!(
            GregorianDateBuilder::new().day_of_year(0).build().err(),
            Some(Error::InvalidDayOfYear)
        );
        assert!(GregorianDateBuilder::new().day_of_year(365).build().is_ok());
        assert_eq!(
            GregorianDateBuilder::new().day_of_year(366).build().err(),
            Some(Error::InvalidDayOfYear)
        );
        assert!(GregorianDateBuilder::new()
            .year(2000)
            .day_of_year(366)
            .build()
            .is_ok());
        assert_eq!(
            GregorianDateBuilder::new()
                .year(2000)
                .day_of_year(367)
                .build()
                .err(),
            Some(Error::InvalidDayOfYear)
        );
    }

    #[test]
    fn valid_day_of_year() {
        for day_of_year in 1..=365 {
            let date: GregorianDate = GregorianDateBuilder::new()
                .day_of_year(day_of_year)
                .build()
                .unwrap();
            assert_eq!(date.day_of_year(), day_of_year);
        }

        for day_of_year in 1..=366 {
            let date: GregorianDate = GregorianDateBuilder::new()
                .year(2000)
                .day_of_year(day_of_year)
                .build()
                .unwrap();
            assert_eq!(date.day_of_year(), day_of_year);
        }
    }

    #[test]
    fn month_days() {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(
                GregorianDateBuilder::new()
                    .month_and_day(month, 1)
                    .build()
                    .unwrap()
                    .month_days(),
                31
            );
        }
        for month in [4, 6, 9, 11] {
            assert_eq!(
                GregorianDateBuilder::new()
                    .month_and_day(month, 1)
                    .build()
                    .unwrap()
                    .month_days(),
                30
            );
        }
        assert_eq!(
            GregorianDateBuilder::new()
                .month_and_day(2, 1)
                .build()
                .unwrap()
                .month_days(),
            28
        );

        assert_eq!(
            GregorianDateBuilder::new()
                .year(1624)
                .month_and_day(2, 1)
                .build()
                .unwrap()
                .month_days(),
            29
        );
    }

    #[test]
    fn year_days() {
        assert_eq!(
            GregorianDateBuilder::new()
                .year(1999)
                .build()
                .unwrap()
                .year_days(),
            365
        );
        assert_eq!(
            GregorianDateBuilder::new()
                .year(2000)
                .build()
                .unwrap()
                .year_days(),
            366
        );
    }

    #[test]
    fn day_of_year() {
        for month in 1..12 {
            let initial_date = GregorianDateBuilder::new()
                .month_and_day(month, 1)
                .build()
                .unwrap();
            let final_date = GregorianDateBuilder::new()
                .month_and_day(month + 1, 1)
                .build()
                .unwrap();
            assert_eq!(
                final_date.day_of_year() - initial_date.day_of_year(),
                initial_date.month_days() as DayOfYear
            );
        }
    }

    #[test]
    fn is_leap_year() {
        assert!(GregorianDateBuilder::new()
            .year(1624)
            .build()
            .unwrap()
            .is_leap_year());
        assert!(GregorianDateBuilder::new()
            .year(1628)
            .build()
            .unwrap()
            .is_leap_year());
        assert!(GregorianDateBuilder::new()
            .year(2000)
            .build()
            .unwrap()
            .is_leap_year());

        assert!(!GregorianDateBuilder::new()
            .year(1620)
            .build()
            .unwrap()
            .is_leap_year());
        assert!(!GregorianDateBuilder::new()
            .year(1625)
            .build()
            .unwrap()
            .is_leap_year());
        assert!(!GregorianDateBuilder::new()
            .year(1700)
            .build()
            .unwrap()
            .is_leap_year());
    }

    #[test]
    fn cmp_impl() {
        assert_eq!(
            GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 1
            },
            GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 1
            }
        );
        assert_ne!(
            GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 1
            },
            GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 2
            }
        );
        assert!(
            GregorianDate {
                year: 1625,
                month: 1,
                day_of_month: 1
            } > GregorianDate {
                year: 1624,
                month: 2,
                day_of_month: 2
            }
        );
        assert!(
            GregorianDate {
                year: 1624,
                month: 2,
                day_of_month: 1
            } > GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 2
            }
        );
        assert!(
            GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 2
            } > GregorianDate {
                year: 1624,
                month: 1,
                day_of_month: 1
            }
        );
    }
}
