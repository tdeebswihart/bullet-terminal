extern crate chrono;
extern crate cursive;
extern crate xdg;


// stdlib
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::string::ToString;

// my modules
mod entry;
use entry::{EntryState, Entry, ParseError};

// external modules
use chrono::{Local, Date};
use cursive::Cursive;
use cursive::align::Align;
use cursive::traits::*;
use cursive::event::EventResult;
use cursive::event::Key;
use cursive::event::Event;
use cursive::view::{Offset, Position};
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

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    let mut select = EntryView::new().on_submit(edit_entry);
    // Load today's data
    let today = Local::today();
    // TODO: move this into the DailyView
    match load_day(today) {
        Ok(entries) => {
            // TODO: Do something more graceful than panicking
            for entry in entries.into_iter() {
                select.add_item(entry.to_display(), entry);
            }
        }
        Err(e) => panic!(e)
    };
    // Override j, k for nav
    let ev = OnEventView::new(select)
        .on_pre_event(Key::Backspace, delete_entry)
        .on_pre_event_inner('e', mark_event)
        .on_pre_event_inner('t', mark_incomplete)
        .on_pre_event_inner('r', mark_note)
        .on_pre_event_inner('d', mark_done)
        .on_pre_event_inner(' ', toggle_completion)
        .on_pre_event_inner('j', |s| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('k', |s| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        });

    let title = TextView::new(Local::today().format("%Y-%m-%d").to_string()).align(Align::center());
    let help_view = ListView::new()
        .child("n", TextView::new("Add a new entry"))
        .child("j", TextView::new("Move the entry selection cursor up"))
        .child("k", TextView::new("Move the entry selection cursor down"))
        .child("h", TextView::new("View the previous day's journal (TODO)"))
        .child("l", TextView::new("View the next day's journal (TODO)"))
        .child("e", TextView::new("Mark the selected entry as an event"))
        .child("t", TextView::new("Mark the selected entry as a task"))
        .child("d", TextView::new("Mark the selected entry as done"))
        .child("r", TextView::new("Mark the entry as a note (r for remember)"))
        .child(",", TextView::new("Schedule (<) the task in the future (TODO)"))
        .child(".", TextView::new("Migrate (>) the task to a collection (TODO)"))
        .child("C-c", TextView::new("Quit"))
        .with_id("help");
    let day_view = OnEventView::new(LinearLayout::vertical()
                               .child(title.with_id("title"))
                                    .child(ev.with_id("entries")))
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
    siv.add_global_callback(Event::Char('q'), |s| {
            s.screen_mut().add_layer(
                Dialog::around(TextView::new("Are you sure you want to quit?"))
                    .dismiss_button("No")
                    .button("Yes", |s2| {
                        let edit_view: ViewRef<EditView> = s2.find_id("new-entry").expect("unable to get new-entry view");
                        add_item(s2, edit_view.get_content().as_ref());
                        s2.pop_layer();
                        s2.quit();
                    }));
    });
    siv.add_layer(LinearLayout::horizontal()
                  .child(day_view.min_width(40))
                  .child(Dialog::around(help_view).title("Bullet Terminal")));

    // Starts the event loop.
    siv.run();
}

/// TODO: move data saving into the views so that they can manage their own storage...
/// how then do I handle "linked" entries? ones that have been scheduled into the future or moved
/// to a collection?
fn save_day(entries: &[&Entry], date: Date<Local>) {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bullet-terminal").unwrap();
    let day = date.format("%Y-%m-%d");
    let data = xdg_dirs.place_config_file(format!("{}.txt", day)).expect("cannot create configuration directory!");
    if let Ok(mut f) = File::create(data) {
        for entry in entries.iter() {
            f.write_all(entry.to_string().as_bytes()).expect("Unable to write data to disk!");
            f.write_all("\n".as_bytes()).expect("Unable to write newline!");
        }
    }
}

fn load_day(date: Date<Local>) -> Result<Vec<Entry>, ParseError> {
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
    // TODO: find a better way to serialize these
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
}

/// Change the currently-selected entry to an event (o)
fn mark_event(entry_view: &mut EntryView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    replace_entry(idx, Entry::new(&entry.content, EntryState::Event), entry_view);
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
    Some(EventResult::Consumed(None))
}

/// Change the currently-selected entry to a note (-)
fn mark_note(entry_view: &mut EntryView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    replace_entry(idx, Entry::new(&entry.content, EntryState::Note), entry_view);
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
    Some(EventResult::Consumed(None))
}

/// Change the currently-selected entry to a task (•)
fn mark_incomplete(entry_view: &mut EntryView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    replace_entry(idx, Entry::new(&entry.content, EntryState::Incomplete), entry_view);
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
    Some(EventResult::Consumed(None))
}

fn mark_done(entry_view: &mut EntryView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    replace_entry(idx, Entry::new(&entry.content, EntryState::Completed), entry_view);
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
    Some(EventResult::Consumed(None))
}

/// Toggle the completion state of a task (• or ×). If a non-task is selected,
/// nothing will happen.
fn toggle_completion(entry_view: &mut EntryView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    {
        let (label, entry) = entry_view.get_item_mut(idx).unwrap();
        entry.toggle_state();
        label.replace_range(0.., &entry.to_display());
    }
    let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
    save_day(&entries, Local::today());
    Some(EventResult::Consumed(None))
}

fn delete_entry(siv: &mut Cursive) {
    siv.screen_mut().add_layer_at(
        Position::new(Offset::Center, Offset::Parent(5)),
        Dialog::around(TextView::new("Are you sure you want to delete the selected entry?"))
            .button("Yes", |s| {
                let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                let entry_view: &mut EntryView = event_view.get_inner_mut();
                let selected = entry_view.selected_id().unwrap();
                let cb = entry_view.remove_item(selected);
                cb(s);
                let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
                save_day(&entries, Local::today());
                s.pop_layer();
            }).dismiss_button("No"));
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
                           let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                           let entry_view: &mut EntryView = event_view.get_inner_mut();
                           let idx = entry_view.selected_id().unwrap();
                           update_at_index(entry_view, idx, text);
                           // replace_entry(idx, Entry::new(&update_view.get_content(), state), entry_view);
                           let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
                           save_day(&entries, Local::today());
                           s.pop_layer();
                       })
                       .with_id("update"))
            .button("Update", |s| {
                // Look for a view tagged "text".
                // We _know_ it's there, so unwrap it.
                let update_view: ViewRef<EditView> = s.find_id("update").expect("Unable to get update view");
                let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
                let entry_view: &mut EntryView = event_view.get_inner_mut();
                let idx = entry_view.selected_id().unwrap();
                update_at_index(entry_view, idx, &update_view.get_content());
                // replace_entry(idx, Entry::new(&update_view.get_content(), state), entry_view);
                let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
                save_day(&entries, Local::today());
                s.pop_layer();
            }).dismiss_button("Cancel"),
    );
}

/// Add an entry. By default entrys are added as tasks (•)
fn add_item(s: &mut Cursive, text: &str) {
    if text.len() > 0 {
        let mut event_view: ViewRef<OnEventView<EntryView>> = s.find_id("entries").expect("Unable to get entry view");
        let entry_view: &mut EntryView = event_view.get_inner_mut();
        // TODO: focus the entry
        let entry = Entry::new(text, EntryState::Incomplete);
        // This should add it...
        entry_view.add_item(entry.to_display(), entry);
        let idx = entry_view.len();
        let cb = entry_view.set_selection(idx - 1);
        cb(s);
        s.focus_id("entries").unwrap();
        let entries: Vec<&Entry> = entry_view.iter().map(|(_, entry)| entry).collect();
        save_day(&entries, Local::today());
    }
}
