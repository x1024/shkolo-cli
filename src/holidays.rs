//! Bulgarian public holidays, used to tell school days from days off.
//!
//! Covers the official non-working days from the Labour Code: the fixed-date
//! holidays, the Orthodox Easter days (Good Friday through Easter Monday), and
//! the rule that a fixed holiday falling on a weekend is observed on the next
//! working day(s). School vacations set yearly by the ministry are not included.

use time::{Date, Duration, Month, Weekday};

use crate::i18n::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holiday {
    pub name_bg: &'static str,
    pub name_en: &'static str,
    /// Working day off in lieu of a holiday that fell on a weekend
    pub observed: bool,
}

impl Holiday {
    pub fn name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Bg => self.name_bg,
            Lang::En => self.name_en,
        }
    }
}

const FIXED: &[(Month, u8, &str, &str)] = &[
    (Month::January, 1, "Нова година", "New Year's Day"),
    (Month::March, 3, "Ден на Освобождението", "Liberation Day"),
    (Month::May, 1, "Ден на труда", "Labour Day"),
    (Month::May, 6, "Гергьовден", "St. George's Day"),
    (Month::May, 24, "Ден на българската просвета и култура", "Day of Bulgarian Education and Culture"),
    (Month::September, 6, "Ден на Съединението", "Unification Day"),
    (Month::September, 22, "Ден на независимостта", "Independence Day"),
    (Month::December, 24, "Бъдни вечер", "Christmas Eve"),
    (Month::December, 25, "Рождество Христово", "Christmas Day"),
    (Month::December, 26, "Рождество Христово", "Second Day of Christmas"),
];

const EASTER: &[(i64, &str, &str)] = &[
    (-2, "Велики петък", "Good Friday"),
    (-1, "Велика събота", "Holy Saturday"),
    (0, "Великден", "Easter Sunday"),
    (1, "Светли понеделник", "Easter Monday"),
];

const DATE_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

/// Orthodox Easter Sunday in the Gregorian calendar (Meeus' Julian algorithm,
/// shifted by the 13-day Julian/Gregorian offset valid for 1900–2099).
pub fn orthodox_easter(year: i32) -> Date {
    let a = year.rem_euclid(4);
    let b = year.rem_euclid(7);
    let c = year.rem_euclid(19);
    let d = (19 * c + 15) % 30;
    let e = (2 * a + 4 * b - d + 34).rem_euclid(7);
    let month = ((d + e + 114) / 31) as u8;
    let day = ((d + e + 114) % 31 + 1) as u8;
    let julian = Date::from_calendar_date(year, Month::try_from(month).expect("Easter month"), day)
        .expect("Easter date");
    julian + Duration::days(13)
}

fn is_weekend(date: Date) -> bool {
    matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

/// All public holidays of a year, in chronological order
pub fn holidays_in_year(year: i32) -> Vec<(Date, Holiday)> {
    let mut days: Vec<(Date, Holiday)> = FIXED
        .iter()
        .filter_map(|&(month, day, bg, en)| {
            let date = Date::from_calendar_date(year, month, day).ok()?;
            Some((date, Holiday { name_bg: bg, name_en: en, observed: false }))
        })
        .collect();

    let easter = orthodox_easter(year);
    for &(offset, bg, en) in EASTER {
        days.push((easter + Duration::days(offset), Holiday { name_bg: bg, name_en: en, observed: false }));
    }

    // A fixed holiday on a weekend moves to the first following working day that
    // is not itself a holiday (Labour Code art. 154(2); Easter days are exempt).
    let fixed_on_weekend: Vec<(Date, Holiday)> = days
        .iter()
        .filter(|(date, h)| is_weekend(*date) && !EASTER.iter().any(|e| e.1 == h.name_bg))
        .cloned()
        .collect();
    for (date, holiday) in fixed_on_weekend {
        let mut observed = date + Duration::days(1);
        while is_weekend(observed) || days.iter().any(|(d, _)| *d == observed) {
            observed += Duration::days(1);
        }
        days.push((observed, Holiday { observed: true, ..holiday }));
    }

    days.sort_by_key(|(date, _)| *date);
    days
}

/// The public holiday on `date`, if any
pub fn public_holiday(date: Date) -> Option<Holiday> {
    holidays_in_year(date.year())
        .into_iter()
        .find(|(d, _)| *d == date)
        .map(|(_, h)| h)
}

/// The public holiday on a `YYYY-MM-DD` date, if any
pub fn public_holiday_on(date: &str) -> Option<Holiday> {
    Date::parse(date, DATE_FORMAT).ok().and_then(public_holiday)
}

/// A weekday that is not a public holiday
pub fn is_school_day(date: Date) -> bool {
    !is_weekend(date) && public_holiday(date).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> Date {
        Date::parse(s, DATE_FORMAT).unwrap()
    }

    #[test]
    fn orthodox_easter_dates() {
        assert_eq!(orthodox_easter(2024), date("2024-05-05"));
        assert_eq!(orthodox_easter(2025), date("2025-04-20"));
        assert_eq!(orthodox_easter(2026), date("2026-04-12"));
        assert_eq!(orthodox_easter(2027), date("2027-05-02"));
        assert_eq!(orthodox_easter(2028), date("2028-04-16"));
    }

    #[test]
    fn fixed_and_easter_holidays() {
        assert_eq!(public_holiday_on("2026-09-22").unwrap().name_en, "Independence Day");
        assert_eq!(public_holiday_on("2026-03-03").unwrap().name_bg, "Ден на Освобождението");
        assert_eq!(public_holiday_on("2026-04-10").unwrap().name_en, "Good Friday");
        assert_eq!(public_holiday_on("2026-04-13").unwrap().name_en, "Easter Monday");
        assert!(public_holiday_on("2026-04-14").is_none());
        assert!(public_holiday_on("2026-09-21").is_none());
        assert!(public_holiday_on("not a date").is_none());
    }

    #[test]
    fn weekend_holidays_are_observed_on_following_working_days() {
        // Unification Day 2026 is a Sunday -> Monday 7 September is off
        let observed = public_holiday_on("2026-09-07").unwrap();
        assert_eq!(observed.name_en, "Unification Day");
        assert!(observed.observed);

        // Christmas 2027: 25th Saturday, 26th Sunday -> Monday 27 and Tuesday 28 off
        assert!(public_holiday_on("2027-12-27").unwrap().observed);
        assert!(public_holiday_on("2027-12-28").unwrap().observed);
        assert!(public_holiday_on("2027-12-29").is_none());

        // Easter days never move
        assert!(holidays_in_year(2026).iter().all(|(_, h)| !(h.observed && h.name_en.contains("Easter"))));
    }

    #[test]
    fn school_days() {
        assert!(is_school_day(date("2026-09-21"))); // Monday
        assert!(!is_school_day(date("2026-09-22"))); // Independence Day
        assert!(!is_school_day(date("2026-09-19"))); // Saturday
        assert!(is_school_day(date("2026-09-23"))); // Wednesday
    }
}
