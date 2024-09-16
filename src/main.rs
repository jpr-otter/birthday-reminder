use chrono::{NaiveDate, Datelike, Local};
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use native_dialog::{MessageDialog, MessageType};

const CSV_FILE_PATH: &str = r"C:\Users\jrotter\Desktop\geburtstage.csv";
const DAYS_IN_ADVANCE: i64 = 30;
const DAYS_IN_PAST: i64 = 5;

#[derive(Debug)]
struct BirthdayInfo {
    name: String,
    birthday: NaiveDate,
}

fn main() -> Result<(), Box<dyn Error>> {
    let birthdays = load_birthdays_from_csv(CSV_FILE_PATH)?;
    check_for_birthdays(&birthdays);
    Ok(())
}

fn load_birthdays_from_csv(path: &str) -> Result<Vec<BirthdayInfo>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(file);

    let mut birthdays = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.len() >= 2 {
            let name = record[0].to_string();
            if let Ok(birthday) = NaiveDate::parse_from_str(&record[1], "%d.%m.%Y") {
                birthdays.push(BirthdayInfo { name, birthday });
            } else {
                eprintln!("Unable to parse date for {}: {}", name, &record[1]);
            }
        }
    }

    Ok(birthdays)
}

fn check_for_birthdays(birthdays: &[BirthdayInfo]) {
    let today = Local::now().date_naive();
    let mut relevant_birthdays: Vec<_> = birthdays
        .iter()
        .filter_map(|b| {
            let days_until = get_days_until_birthday(&b.birthday, &today);
            if -DAYS_IN_PAST <= days_until && days_until <= DAYS_IN_ADVANCE {
                Some((b, days_until))
            } else {
                None
            }
        })
        .collect();

    // Sort birthdays: past birthdays first (most recent to oldest), then upcoming (nearest to farthest)
    relevant_birthdays.sort_by_key(|&(_, days)| {
        if days <= 0 {
            (-2, days) // Past birthdays (including today) come first, sorted from recent to old
        } else {
            (-1, days) // Then upcoming birthdays, sorted from nearest to farthest
        }
    });

    if !relevant_birthdays.is_empty() {
        let mut message = String::from("Birthdays:\n\n");
        for (birthday, days_until) in relevant_birthdays {
            let birthday_date = get_this_year_birthday(&birthday.birthday, &today);
            
            if days_until > 0 {
                message.push_str(&format!("{}: {} (in {} days)\n\n", birthday.name, birthday_date.format("%d.%m.%Y"), days_until));
            } else if days_until == 0 {
                message.push_str(&format!("{}: {} (TODAY - HAPPY BIRTHDAY!!!!)\n\n", birthday.name, birthday_date.format("%d.%m.%Y")));
            } else {
                message.push_str(&format!("{}: {} ({} days ago - Birthday is in the past)\n\n", birthday.name, birthday_date.format("%d.%m.%Y"), -days_until));
            }
        }

        MessageDialog::new()
            .set_type(MessageType::Info)
            .set_title("Birthday Reminders")
            .set_text(&message)
            .show_alert()
            .unwrap();
    }
}

fn get_days_until_birthday(birthday: &NaiveDate, today: &NaiveDate) -> i64 {
    let this_year_birthday = get_this_year_birthday(birthday, today);
    (this_year_birthday - *today).num_days()
}

fn get_this_year_birthday(birthday: &NaiveDate, today: &NaiveDate) -> NaiveDate {
    let this_year_birthday = NaiveDate::from_ymd_opt(
        today.year(),
        birthday.month(),
        birthday.day(),
    ).unwrap();

    if &this_year_birthday < today {
        NaiveDate::from_ymd_opt(today.year() + 1, birthday.month(), birthday.day()).unwrap()
    } else {
        this_year_birthday
    }
}