use entry::{EntryState, Entry, ParseError};

use std::boxed::Box;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use chrono::{NaiveDate, Duration};
use cursive::Cursive;
use cursive::view::View;
use cursive::align::Align;
use cursive::traits::*;
use cursive::event::EventResult;
use cursive::event::Key;
use cursive::event::Event;
use cursive::view::{Offset, Position};
use cursive::theme::{Effect, Style};
use cursive::utils::span::{SpannedString};
use cursive::views::{
    ViewRef,
    Dialog,
    TextView,
    LinearLayout,
    EditView,
    SelectView,
    ListView,
    OnEventView};

type EntryView = SelectView<Entry>;

struct DateWrapper<T>(T);

impl<T> DateWrapper<T> {
    fn new(v: T) -> DateWrapper<T> {
        DateWrapper(v)
    }
}

impl Into<SpannedString<Style>> for DateWrapper<NaiveDate> {
    fn into(self) -> SpannedString<Style> {
        SpannedString::styled(self.0.format("%Y-%m-%d").to_string(), Effect::Bold)
    }
}

struct FormattedDate(String);

impl From<NaiveDate> for FormattedDate {
    fn from(date: NaiveDate) -> FormattedDate {
        FormattedDate(date.format("%Y-%m-%d").to_string())
    }
}

pub fn daily_view(date: NaiveDate) -> Box<View> {
    let mut select = EntryView::new().on_submit(edit_entry);
    match load_day(date) {
        Ok(entries) => {
            // TODO: Do something more graceful than panicking
            for entry in entries.into_iter() {
                select.add_item(entry.to_display(), entry);
            }
        }
        Err(e) => panic!(e)
    };
    let title = TextView::new(DateWrapper::new(date)).align(Align::center());
    // Override j, k for nav
    let ev = OnEventView::new(select)
        .on_pre_event(Key::Backspace, delete_entry)
        .on_pre_event('l', add_day)
        .on_pre_event('h', sub_day)
        .on_pre_event('.', add_week)
        .on_pre_event(',', sub_week)
        .on_pre_event('e', mark_event)
        .on_pre_event('t', mark_incomplete)
        .on_pre_event('r', mark_note)
        .on_pre_event('d', mark_done)
        .on_pre_event(' ', toggle_completion)
        .on_pre_event_inner('j', |s| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('k', |s| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        });

    let help_view = ListView::new()
        .child("n", TextView::new("Add a new entry"))
        .child("j", TextView::new("Move the entry selection cursor up"))
        .child("k", TextView::new("Move the entry selection cursor down"))
        .child("h", TextView::new("View the previous day's journal"))
        .child("l", TextView::new("View the next day's journal"))
        .child("h", TextView::new("View the journal from one week prior"))
        .child("l", TextView::new("View the journal one week into the future"))
        .child("e", TextView::new("Mark the selected entry as an event"))
        .child("t", TextView::new("Mark the selected entry as a task"))
        .child("d", TextView::new("Mark the selected entry as done"))
        .child("m", TextView::new("View your monthly log (TODO)"))
        .child("f", TextView::new("View your future log (TODO)"))
        .child("r", TextView::new("Mark the entry as a note (r for remember)"))
        .child(",", TextView::new("Schedule (<) the task in the future (TODO)"))
        .child(".", TextView::new("Migrate (>) the task to a collection (TODO)"))
        .child("C-c", TextView::new("Quit"))
        .with_id("help");
    let day_view = OnEventView::new(LinearLayout::vertical()
                                    .child(title.with_id("title"))
                                    .child(ev.with_id("entries")))
        // TODO: add the help popup here and have only a short message in the window
        .on_pre_event(Event::Char('n'), |s| {
            // TODO: pull this out into a proper function somewhere
            s.screen_mut().add_layer(
                Dialog::around(EditView::new()
                               .on_submit(|s2, entry| {
                                   add_item(s2, entry);
                                   s2.pop_layer();
                               }).with_id("new-entry").min_width(20))
                    .dismiss_button("Cancel")
                    .button("Add", |s2| {
                        let edit_view: ViewRef<EditView> = s2.find_id("new-entry").expect("unable to get new-entry view");
                        add_item(s2, edit_view.get_content().as_ref());
                        s2.pop_layer();
                    }));
        });
    Box::new(LinearLayout::horizontal()
             .child(day_view.min_width(40))
             .child(Dialog::around(help_view).title("Bullet Terminal")))
}

/// Save the current day's entries. Looks at the contents of the "title" view to figure it out
/// There's probably a better place to store it, but this works with Cursive's API for now...
fn save_day(siv: &mut Cursive) {
    // duh. save them to a vec of string rather than vec of entry to avoid the borrowing issue
    let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
    let entry_view: &mut EntryView = event_view.get_inner_mut();
    // TODO: add an error type for this...
    let date_view: ViewRef<TextView> = siv.find_id("title").expect("Unable to find title view");
    let day = date_view.get_content().source().to_string();
    let entries: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bullet-terminal").unwrap();
    let data = xdg_dirs.place_config_file(format!("{}.txt", day)).expect("cannot create configuration directory!");
    if let Ok(mut f) = File::create(data) {
        for entry in entries.iter() {
            f.write_all(entry.as_bytes()).expect("Unable to write data to disk!");
            f.write_all("\n".as_bytes()).expect("Unable to write newline!");
        }
    }
}

fn load_day(date: NaiveDate) -> Result<Vec<Entry>, ParseError> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bullet-terminal").unwrap();
    let day = date.format("%Y-%m-%d");
    let data = xdg_dirs.place_config_file(format!("{}.txt", day)).expect("cannot create configuration directory!");
    if let Ok(f) = File::open(data) {
        let mut reader = BufReader::new(f);
        let mut vec = Vec::new();
        for maybe_line in reader.lines() {
            vec.push(Entry::from_str(&maybe_line?)?);
        }
        Ok(vec)
    } else {
        Ok(Vec::new())
    }
}

fn replace_entry(idx: usize, replacement: Entry, entry_view: &mut EntryView) {
    let _ = entry_view.remove_item(idx);
    entry_view.insert_item(idx, replacement.to_display(), replacement);
    let _ = entry_view.set_selection(idx);
}

/// Change the currently-selected entry to an event (o)
fn mark_event(siv: &mut Cursive) {
    {
        let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        let idx = entry_view.selected_id().unwrap();
        let entry = entry_view.selection().unwrap();
        replace_entry(idx, Entry::new(&entry.content, EntryState::Event), entry_view);
    }
    save_day(siv);
}

/// Change the currently-selected entry to a note (-)
fn mark_note(siv: &mut Cursive) {
    {
        let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        let idx = entry_view.selected_id().unwrap();
        let entry = entry_view.selection().unwrap();
        replace_entry(idx, Entry::new(&entry.content, EntryState::Note), entry_view);
    }
    save_day(siv);
}

/// Change the currently-selected entry to a task (•)
fn mark_incomplete(siv: &mut Cursive) {
    {
        let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        let idx = entry_view.selected_id().unwrap();
        let entry = entry_view.selection().unwrap();
        replace_entry(idx, Entry::new(&entry.content, EntryState::Incomplete), entry_view);
    }
    save_day(siv);
}

fn mark_done(siv: &mut Cursive) {
    {
        let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        let idx = entry_view.selected_id().unwrap();
        let entry = entry_view.selection().unwrap();
        replace_entry(idx, Entry::new(&entry.content, EntryState::Completed), entry_view);
    }
    save_day(siv);
}

/// Toggle the completion state of a task (• or ×). If a non-task is selected,
/// nothing will happen.
fn toggle_completion(siv: &mut Cursive) {
    {
        let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        let idx = entry_view.selected_id().unwrap();
        let (label, entry) = entry_view.get_item_mut(idx).unwrap();
        entry.toggle_state();
        label.replace_range(0.., &entry.to_display());
    }
    save_day(siv);
}

fn delete_entry(siv: &mut Cursive) {
    siv.screen_mut().add_layer_at(
        Position::new(Offset::Center, Offset::Parent(5)),
        Dialog::around(TextView::new("Are you sure you want to delete the selected entry?"))
            .button("Yes", |s| {
                {
                    let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                    let entry_view: &mut EntryView = event_view.get_inner_mut();
                    let selected = entry_view.selected_id().unwrap();
                    let cb = entry_view.remove_item(selected);
                    cb(s);
                }
                save_day(s);
                s.pop_layer();
            }).dismiss_button("No"));
}

fn change_day(siv: &mut Cursive, diff: Duration) -> Result<(), ParseError> {
    // TODO: add an error type for this...
    let mut event_view: ViewRef<OnEventView<EntryView>> = siv.find_id("entries").expect("Unable to get entry view");
    let entry_view: &mut EntryView = event_view.get_inner_mut();
    // TODO: add an error type for this...
    let mut date_view: ViewRef<TextView> = siv.find_id("title").expect("Unable to find title view");
    let date_str = date_view.get_content().source().to_string();
    let current_day = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(ParseError::from)?;
    // TODO: add an error type for this
    let add_day = current_day.checked_add_signed(diff).unwrap();
    date_view.set_content(DateWrapper::new(add_day));
    let entries = load_day(add_day)?;
    entry_view.clear();
    for entry in entries.into_iter() {
        entry_view.add_item(entry.to_display(), entry);
    }
    Ok(())
}

/// Alter the view to reflect the previous day
fn sub_day(siv: &mut Cursive) {
    match change_day(siv, Duration::days(-1)) {
        Ok(_) => (),
        Err(error) => panic!(error)
    }
}

/// Change the currently-selected entry to an event (o)
fn add_day(siv: &mut Cursive) {
    match change_day(siv, Duration::days(1)) {
        Ok(_) => (),
        Err(error) => panic!(error)
    }
}

/// Alter the view to reflect the previous week
fn sub_week(siv: &mut Cursive) {
    match change_day(siv, Duration::weeks(-1)) {
        Ok(_) => (),
        Err(error) => panic!(error)
    }
}

/// Change the currently-selected entry to an event (o)
fn add_week(siv: &mut Cursive) {
    match change_day(siv, Duration::weeks(1)) {
        Ok(_) => (),
        Err(error) => panic!(error)
    }
}

fn update_at_index(entry_view: &mut EntryView, idx: usize, new_content: &str) {
    let (label, entry) = entry_view.get_item_mut(idx).unwrap();
    entry.content.replace_range(0.., new_content);
    label.replace_range(0.., &entry.to_display());
}

/// Edit the contents of the currently-selected entry.
fn edit_entry(siv: &mut Cursive, orig_entry: &Entry) {
    let content = orig_entry.content.clone();
    siv.screen_mut().add_layer_at(
        Position::new(Offset::Center, Offset::Parent(5)),
        Dialog::around(EditView::new()
                       .content(content)
                       .on_submit(|s, text| {
                           {
                               let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                               let entry_view: &mut EntryView = event_view.get_inner_mut();
                               let idx = entry_view.selected_id().unwrap();
                               update_at_index(entry_view, idx, text);
                               // replace_entry(idx, Entry::new(&update_view.get_content(), state), entry_view);
                           }
                           save_day(s);
                           s.pop_layer();
                       })
                       .with_id("update"))
            .button("Update", |s| {
                {
                    let update_view: ViewRef<EditView> = s.find_id("update").expect("Unable to get update view");
                    let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                    let entry_view: &mut EntryView = event_view.get_inner_mut();
                    let idx = entry_view.selected_id().unwrap();
                    update_at_index(entry_view, idx, &update_view.get_content());
                }
                save_day(s);
                s.pop_layer();
            }).dismiss_button("Cancel"),
    );
}

/// Add an entry. By default entrys are added as tasks (•)
fn add_item(s: &mut Cursive, text: &str) {
    if text.len() > 0 {
        {
            let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
            let entry_view: &mut EntryView = event_view.get_inner_mut();
            let entry = Entry::new(text, EntryState::Incomplete);
            entry_view.add_item(entry.to_display(), entry);
            let idx = entry_view.len();
            let cb = entry_view.set_selection(idx - 1);
            cb(s);
        }
        s.focus_id("entries").unwrap();
        save_day(s);
    }
}
