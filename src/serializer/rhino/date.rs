pub struct GregorianDate {
    year: u16,
    month: u8,
    day: u8,
}

impl GregorianDate {
    const FIRST_YEAR: u16 = 1582;

    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, &'static str> {
        let date = GregorianDate { year, month, day };
        if Self::FIRST_YEAR <= year {
            if (1..=12).contains(&month) {
                if (1..=date.days_of_month()).contains(&day) {
                    Ok(date)
                } else {
                    Err("Invalid day")
                }
            } else {
                Err("Invalid month")
            }
        } else {
            Err("Invalid year")
        }
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }

    pub fn is_leap_year(&self) -> bool {
        (1624 <= self.year)
            && (0 == (self.year % 4))
            && (0 == (self.year % 400) || 0 != (self.year % 100))
    }

    pub fn days_of_month(&self) -> u8 {
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

    pub fn day_of_the_year(&self) -> u16 {
        let extra_day = if self.is_leap_year() { 1u16 } else { 0u16 };
        match self.month {
            1 => self.day as u16,
            2 => 31 + self.day as u16,
            3 => 59 + self.day as u16 + extra_day,
            4 => 90 + self.day as u16 + extra_day,
            5 => 120 + self.day as u16 + extra_day,
            6 => 151 + self.day as u16 + extra_day,
            7 => 181 + self.day as u16 + extra_day,
            8 => 212 + self.day as u16 + extra_day,
            9 => 243 + self.day as u16 + extra_day,
            10 => 273 + self.day as u16 + extra_day,
            11 => 304 + self.day as u16 + extra_day,
            12 => 334 + self.day as u16 + extra_day,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_year() {
        match GregorianDate::new(1582, 1, 1) {
            Ok(date) => {
                assert_eq!(1582, date.year)
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn invalid_year() {
        assert!(GregorianDate::new(1581, 1, 1).is_err());
    }

    #[test]
    fn valid_month() {
        for month in 1..=12 {
            assert!(GregorianDate::new(1582, month, 1).is_ok());
            match GregorianDate::new(1582, month, 1) {
                Ok(date) => {
                    assert_eq!(month, date.month);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }
    }

    #[test]
    fn invalid_month() {
        assert!(GregorianDate::new(1582, 0, 1).is_err());
        assert!(GregorianDate::new(1582, 13, 1).is_err());
    }

    #[test]
    fn valid_day() {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            for day in 1..=31 {
                match GregorianDate::new(1582, month, day) {
                    Ok(date) => {
                        assert_eq!(day, date.day);
                    }
                    Err(_) => {
                        assert!(false);
                    }
                }
            }
        }

        for month in [4, 6, 9, 1] {
            for day in 1..=30 {
                match GregorianDate::new(1582, month, day) {
                    Ok(date) => {
                        assert_eq!(day, date.day);
                    }
                    Err(_) => {
                        assert!(false);
                    }
                }
            }
        }

        for day in 1..=28 {
            match GregorianDate::new(1582, 2, day) {
                Ok(date) => {
                    assert_eq!(day, date.day);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }

        for day in 1..=29 {
            match GregorianDate::new(1624, 2, day) {
                Ok(date) => {
                    assert_eq!(day, date.day);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }
    }

    #[test]
    fn invalid_day() {
        for month in 1..=12 {
            assert!(GregorianDate::new(1582, month, 0).is_err());
        }
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert!(GregorianDate::new(1582, month, 32).is_err());
        }

        for month in [4, 6, 9, 11] {
            assert!(GregorianDate::new(1582, month, 31).is_err());
        }

        assert!(GregorianDate::new(1582, 2, 29).is_err());
        assert!(GregorianDate::new(1624, 2, 30).is_err());
    }

    #[test]
    fn days_of_month() {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(
                GregorianDate::new(1582, month, 1).unwrap().days_of_month(),
                31
            );
            assert_eq!(
                GregorianDate::new(1624, month, 1).unwrap().days_of_month(),
                31
            );
        }
        for month in [4, 6, 9, 11] {
            assert_eq!(
                GregorianDate::new(1582, month, 1).unwrap().days_of_month(),
                30
            );
            assert_eq!(
                GregorianDate::new(1624, month, 1).unwrap().days_of_month(),
                30
            );
        }
        assert_eq!(GregorianDate::new(1582, 2, 1).unwrap().days_of_month(), 28);
        assert_eq!(GregorianDate::new(1624, 2, 1).unwrap().days_of_month(), 29);
    }

    #[test]
    fn day_of_the_year() {
        for month in 1..12 {
            let initial_date = GregorianDate::new(1582, month, 1).unwrap();
            let final_date = GregorianDate::new(1582, month + 1, 1).unwrap();
            assert_eq!(
                final_date.day_of_the_year() - initial_date.day_of_the_year(),
                initial_date.days_of_month() as u16
            );
        }
    }

    #[test]
    fn is_leap_year() {
        assert!(GregorianDate::new(1624, 1, 1).unwrap().is_leap_year());
        assert!(GregorianDate::new(1628, 1, 1).unwrap().is_leap_year());
        assert!(GregorianDate::new(2000, 1, 1).unwrap().is_leap_year());

        assert!(!GregorianDate::new(1620, 1, 1).unwrap().is_leap_year());
        assert!(!GregorianDate::new(1625, 1, 1).unwrap().is_leap_year());
        assert!(!GregorianDate::new(1700, 1, 1).unwrap().is_leap_year());
    }
}
