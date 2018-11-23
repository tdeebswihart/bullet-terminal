use chrono::{NaiveDateTime, ParseError as DateParseError};
use std::option;
use std::string::ToString;
use std::result::Result;

pub enum EntryState {
    Incomplete,
    Note,
    Event,
    Scheduled(NaiveDateTime),
    Collected(NaiveDateTime),
    Completed
}

pub enum ParseError {
    InvalidTag(String),
    InvalidDateTime(String),
    InvalidEntry(String),
}

use self::ParseError::*;

impl From<DateParseError> for ParseError {
    fn from(error: DateParseError) -> ParseError {
        InvalidDateTime(format!("{}", error))
    }
}


impl ToString for ParseError {
    fn to_string(&self) -> String {
        use self::ParseError::*;
        match self {
            InvalidTag(tag) => format!("Invalid entry tag {}", tag),
            InvalidDateTime(msg) => format!("Invalid DateTime value '{}'", msg),
            InvalidEntry(entry) => format!("Invalid entry '{}'. Entries should have a valid tag, then a space, then any amount of text", entry),
        }
    }
}

impl EntryState {
    pub fn from_str(tag: &str) -> Result<EntryState, ParseError> {
        use self::EntryState::*;
        match tag {
            "." => Ok(Incomplete),
            "-" => Ok(Note),
            "o" => Ok(Event),
            "x" => Ok(Completed),
            _ => {
                // Collected and scheduled are harder to handle, as they've an embedded date
                let slice = tag.get(1..).ok_or_else(|| InvalidTag(tag.to_string()))?;
                let date = NaiveDateTime::parse_from_str(slice, "%Y-%m-%d")?;
                // If we survived the .get above, we won't panic now
                match tag.get(0..1).expect("If the get before worked how could we possibly fail here?") {
                    "<" => Ok(Scheduled(date)),
                    ">" => Ok(Collected(date)),
                    _ => Err(InvalidTag(tag.to_string()))
                }
            }
        }
    }

    /// Return the nice unicode display symbol, which is a pain to type on the keyboard
    /// so isn't also used for storage.
    pub fn to_display(&self) -> String {
        use EntryState::*;
        (match self {
            Incomplete=> "•",
            Note => "-",
            Event => "o",
            Scheduled(_) => "<",
            Collected(_) => ">",
            Completed => "×"
        }).to_string()
    }
}

impl ToString for EntryState {
    /// Convert it to the simple string for saving. That way it can be edited in other programs
    pub fn to_string(&self) -> String {
        use entry::EntryState::*;
        match self {
            Incomplete => ".".to_string(),
            Note => "-".to_string(),
            Event => "o".to_string(),
            Scheduled(date) => date.format("<%Y-%m-%d").to_string(),
            Collected(date) => date.format(">%Y-%m-%d").to_string(),
            Completed => "x".to_string()
        }
    }
}


pub struct Entry {
    state: EntryState,
    content: String
}

impl Entry {
    pub fn new(content: &str, state: EntryState) -> Entry {
        Entry {state: state,
              content: content.to_string()}
    }

    pub fn from_str(line: &str) -> Result<Entry, ParseError> {
        let split_idx = line.find(" ");
        match split_idx {
            Some(idx) => {
                let (tag, body) = line.split_at(idx);
                let state = self::EntryState::from_str(tag.trim())?;
                Ok(self::Entry {content: body.to_string(), state: state})
            },
            None => Err(InvalidEntry(line.to_string()))
        }
    }
}

impl ToString for Entry {
    pub fn to_string(&self) -> String {
        [self.state.to_string(), self.content.clone()].join(" ")
    }
}

pub struct DailyView {

}
