extern crate chrono;
extern crate cursive;
extern crate xdg;

mod entry;

// stdlib
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::string::ToString;

// my modules
use entry::{EntryState, Entry};

// external modules
use chrono::{Local, Date};
use cursive::Cursive;
use cursive::align::Align;
use cursive::traits::*;
use cursive::event::EventResult;
use cursive::event::Key;
use cursive::event::Event;
use cursive::theme::Effect;
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

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    siv.add_global_callback(Key::Backspace, |s| {
        s.screen_mut().add_layer_at(
            Position::new(Offset::Center, Offset::Parent(5)),
            Dialog::around(TextView::new("Are you sure you want to delete the selected entry?"))
                .button("Yes", |s| {
                    let mut event_view: ViewRef<OnEventView<SelectView>> = s.find_id("entrys").expect("Unable to get entry view");
                    let entry_view: &mut SelectView = event_view.get_inner_mut();
                    let selected = entry_view.selected_id().unwrap();
                    let cb = entry_view.remove_item(selected);
                    cb(s);
                    let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
                    save_data(lines.as_slice(), Local::today());
                    s.pop_layer();
                }).dismiss_button("No"));
    });
    let mut select = SelectView::<String>::new().on_submit(edit_entry);
    // Load today's data
    let today = Local::today();
    match load_data(today) {
        Some(lines) => {
            for line in lines.into_iter() {
                select.add_item_str(line);
            }
        }
        None => ()
    };
    // Override j, k for nav
    let ev = OnEventView::new(select)
        .on_pre_event_inner('e', mark_event)
        .on_pre_event_inner('t', mark_task)
        .on_pre_event_inner('n', mark_note)
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

    // TODO: force the EditView to stay at the bottom
    // TODO: add a hotkey to focus it/
    let title = TextView::new(Local::today().format("%Y-%m-%d").to_string()).align(Align::center());
    let help_view = ListView::new()
        .child("n", TextView::new("Add a new entry"))
        .child("j", TextView::new("Move the entry selection cursor up"))
        .child("k", TextView::new("Move the entry selection cursor down"))
        .child("h", TextView::new("View the previous day's journal"))
        .child("l", TextView::new("View the next day's journal"))
        .child("e", TextView::new("Mark the selected entry as an event"))
        .child("t", TextView::new("Mark the selected entry as a task"))
        .child("d", TextView::new("Mark the selected entry as done"))
        .child("r", TextView::new("Mark the entry as a note (r for remember)"))
        .child(",", TextView::new("Schedule (<) the task in the future (TODO)"))
        .child(".", TextView::new("Migrate (>) the task to a collection (TODO)"))
        .child("C-c", TextView::new("Quit"))
        .with_id("help");
    siv.add_layer(LinearLayout::horizontal()
                  .child(LinearLayout::vertical()
                         .child(title.with_id("title"))
                         .child(ev.with_id("entrys"))
                         .fixed_width(40)
                  ).child(Dialog::around(help_view.fixed_width(30)).title("Bullet Terminal")));

    siv.add_global_callback(Event::CtrlChar('n'), |s| {
        s.screen_mut().add_layer_at(
            Position::new(Offset::Center, Offset::Parent(5)),
            Dialog::around(EditView::new()
                           .on_submit(|s2, entry| {
                               add_item(s2, entry);
                               s2.pop_layer();
                           }).with_id("new-entry"))
                .dismiss_button("Cancel")
                .button("Add", |s2| {
                    let edit_view: ViewRef<EditView> = s2.find_id("new-entry").expect("unable to get new-entry view");
                    add_item(s2, edit_view.get_content().as_ref());
                    s2.pop_layer();
                }));
    });
    siv.add_global_callback(Event::CtrlChar('l'), |s| {
        s.focus_id("entrys").unwrap();
    });
    // Starts the event loop.
    siv.run();
}

fn save_data(lines: &[String], date: Date<Local>) {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bullet-terminal").unwrap();
    let day = date.format("%Y-%m-%d");
    let data = xdg_dirs.place_config_file(format!("{}.txt", day)).expect("cannot create configuration directory!");
    if let Ok(mut f) = File::create(data) {
        for line in lines.iter() {
            f.write_all(line.as_bytes()).expect("Unable to write data to disk!");
            f.write_all("\n".as_bytes()).expect("Unable to write newline!");
        }
    }
}

fn load_data(date: Date<Local>) -> Option<Vec<String>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bullet-terminal").unwrap();
    let day = date.format("%Y-%m-%d");
    let data = xdg_dirs.place_config_file(format!("{}.txt", day)).expect("cannot create configuration directory!");
    if let Ok(f) = File::open(data) {
        let mut reader = BufReader::new(f);
        Some(reader.lines().map(|l| l.unwrap().to_string()).collect())
    } else {
        None
    }
}
fn change_tag(tag: &str, entry_view: &mut SelectView) {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    let split_idx = entry.find(" ").unwrap() + 1;
    let (_, body) = entry.split_at(split_idx);
    let new_entry = format!("{} {}", tag, body);
    let _ = entry_view.remove_item(idx);
    entry_view.insert_item(idx, new_entry.to_string(), new_entry.to_string());
    let _ = entry_view.set_selection(idx);
    let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
    save_data(lines.as_slice(), Local::today());
}

/// Change the currently-selected entry to an event (o)
fn mark_event(entry_view: &mut SelectView) -> Option<EventResult> {
    change_tag("o", entry_view);
    Some(EventResult::Consumed(None))
}

/// Change the currently-selected entry to a note (-)
fn mark_note(entry_view: &mut SelectView) -> Option<EventResult> {
    change_tag("-", entry_view);
    Some(EventResult::Consumed(None))
}

/// Change the currently-selected entry to a task (•)
fn mark_task(entry_view: &mut SelectView) -> Option<EventResult> {
    change_tag("•", entry_view);
    Some(EventResult::Consumed(None))
}

fn mark_done(entry_view: &mut SelectView) -> Option<EventResult> {
    change_tag("×", entry_view);
    Some(EventResult::Consumed(None))
}

/// Toggle the completion state of a task (• or ×). If a non-task is selected,
/// nothing will happen.
fn toggle_completion(entry_view: &mut SelectView) -> Option<EventResult> {
    let idx = entry_view.selected_id().unwrap();
    let entry = entry_view.selection().unwrap();
    let split_idx = entry.find(" ").unwrap();
    let (tag, _) = entry.split_at(split_idx);
    let (_, body) = entry.split_at(split_idx + 1);
    match tag {
        "•" => {
            let new_entry = format!("× {}", body);
            let _ = entry_view.remove_item(idx);
            entry_view.insert_item(idx, new_entry.to_string(), new_entry.to_string());
            let _ = entry_view.set_selection(idx);
            let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
            save_data(lines.as_slice(), Local::today());
        },
        "×" => {
            let new_entry = format!("• {}", body);
            let _ = entry_view.remove_item(idx);
            entry_view.insert_item(idx, new_entry.to_string(), new_entry.to_string());
            let _ = entry_view.set_selection(idx);
            let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
            save_data(lines.as_slice(), Local::today());
        }
        _ => ()
    };
    Some(EventResult::Consumed(None))
}

/// Edit the contents of the currently-selected entry.
fn edit_entry(siv: &mut Cursive, entry: &str) {
    let split_idx = entry.find(" ").unwrap();
    let (_, body) = entry.split_at(split_idx + 1);
    siv.screen_mut().add_layer_at(
        Position::new(Offset::Center, Offset::Parent(5)),
        Dialog::around(EditView::new().content(body).with_id("update"))
            .button("Update", |s| {
                // Look for a view tagged "text".
                // We _know_ it's there, so unwrap it.
                let update_view: ViewRef<EditView> = s.find_id("update").expect("Unable to get update view");
                let mut event_view: ViewRef<OnEventView<SelectView>> = s.find_id("entrys").expect("Unable to get entry view");
                let entry_view: &mut SelectView = event_view.get_inner_mut();
                let selected = entry_view.selected_id().unwrap();
                let inner_entry = entry_view.selection().unwrap();
                let split_idx = inner_entry.find(" ").unwrap();
                let (tag, _) = inner_entry.split_at(split_idx);
                let new_entry = format!("{} {}", tag, update_view.get_content());
                let cb = entry_view.remove_item(selected);
                cb(s);
                entry_view.insert_item(selected, new_entry.to_string(), new_entry.to_string());
                let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
                save_data(lines.as_slice(), Local::today());
                s.pop_layer();
            }).dismiss_button("Cancel"),
    );
}

/// Add an entry. By default entrys are added as tasks (•)
fn add_item(s: &mut Cursive, text: &str) {
    if entry.len() > 0 {
        let mut event_view: ViewRef<OnEventView<SelectView>> = s.find_id("entrys").expect("Unable to get entry view");
        let entry_view: &mut SelectView = event_view.get_inner_mut();
        // TODO: focus the entry
        match Entry::from_str(text) {
            Ok(entry) => {
                entry_view.add_item(entry.to_string(), entry.to_string());
                let idx = entry_view.len();
                let cb = entry_view.set_selection(idx - 1);
                cb(s);
                s.focus_id("entrys").unwrap();
                let lines: Vec<String> = entry_view.iter().map(|(_, entry)| entry.to_string()).collect();
                save_data(lines.as_slice(), Local::today());
            },
            Err(error) => {
                s.screen_mut().add_layer(
                    Dialog::around(TextView::new(error.to_string()).with_id("error"))
                        .dismiss_button("Damn it"));
            }
        }
    }
}
