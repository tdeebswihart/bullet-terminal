use chrono::{NaiveDateTime, ParseError as DateParseError};
use std::fmt;
use std::io;
use std::string::ToString;
use std::result::Result;

#[derive(Debug)]
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

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> ParseError {
        InvalidEntry(format!("{}", error))
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

#[derive(Debug)]
pub enum EntryState {
    Incomplete,
    Note,
    Event,
    Scheduled(NaiveDateTime),
    Collected(NaiveDateTime),
    Completed
}
use EntryState::*;

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
        (match self {
            Incomplete=> "•",
            Note => "-",
            Event => "o",
            Scheduled(_) => "<",
            Collected(_) => ">",
            Completed => "×"
        }).to_string()
    }

    /// Mutable implementation of the same
    /// Return the nice unicode display symbol, which is a pain to type on the keyboard
    /// so isn't also used for storage.
    pub fn to_display_mut(&mut self) -> String {
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

impl fmt::Display for EntryState {
    /// Convert it to the simple string for saving. That way it can be edited in other programs
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use entry::EntryState::*;
        match self {
            Incomplete => write!(f, "."),
            Note => write!(f, "-"),
            Event => write!(f, "o"),
            Scheduled(date) => write!(f, "{}", date.format("<%Y-%m-%d")),
            Collected(date) => write!(f, "{}", date.format(">%Y-%m-%d")),
            Completed => write!(f, "x")
        }
    }
}


#[derive(Debug)]
pub struct Entry {
    pub state: EntryState,
    pub content: String
}

impl Entry {
    pub fn new(content: &str, state: EntryState) -> Entry {
        Entry {state: state,
              content: content.trim().to_string()}
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

    pub fn toggle_state(&mut self) {
        match self.state {
            EntryState::Incomplete => self.state = EntryState::Completed,
            EntryState::Completed => self.state = EntryState::Incomplete,
            _ => ()
        };
    }

    /// Return the nice unicode display symbol, which is a pain to type on the keyboard
    /// so isn't also used for storage.
    pub fn to_display(&self) -> String {
        format!("{} {}", self.state.to_display(), self.content)
    }

}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.state, self.content)
    }
}
