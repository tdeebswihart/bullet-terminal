extern crate cursive;

use cursive::Cursive;
use cursive::traits::*;
use cursive::event::EventResult;
use cursive::event::Key;
use cursive::event::Event;
use cursive::view::{Offset, Position};
use cursive::views::{ViewRef, Dialog, TextView, LinearLayout, EditView, SelectView, OnEventView};

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    siv.add_global_callback(Event::CtrlChar('n'), |s| {
        s.focus_id("new");
    });
    siv.add_global_callback(Event::CtrlChar('l'), |s| {
        s.focus_id("items");
    });
    // Creates a dialog with a single "Quit" button
    // TODO: implement editing: on SelectView submit, send
    //       contents to the edit buffer, then on submit there
    //       update the item in the select
    let select = SelectView::<String>::new()
        .item_str("• Build bullet terminal")
        .item_str("o Thanksgiving @ 1:00PM")
        .on_submit(edit_item);
    // Override j, k for nav
    let ev = OnEventView::new(select)
        .on_pre_event_inner('e', make_event)
        .on_pre_event_inner('t', make_task)
        .on_pre_event_inner('n', make_note)
        .on_pre_event_inner(' ', toggle_completion)
        .on_pre_event_inner(Key::Backspace, delete_item)
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
    let edit_view = EditView::new()
        .on_submit(add_item);
    siv.add_layer(LinearLayout::horizontal()
                  .child(LinearLayout::vertical()
                         .child(ev.with_id("items"))
                         .child(edit_view.with_id("new"))
                         .fixed_width(40)
                  ).child(Dialog::around(TextView::new("Press ? for help"))
                          .title("Bullet Terminal")
                          .button("Quit", |s| s.quit())
                          .fixed_width(20)));

    // Starts the event loop.
    siv.run();
}

fn delete_item(item_view: &mut SelectView) -> Option<EventResult> {
    Some(EventResult::Consumed(None))
}

fn change_tag(tag: &str, item_view: &mut SelectView) {
    let idx = item_view.selected_id().unwrap();
    let item = item_view.selection().unwrap();
    let split_idx = item.find(" ").unwrap() + 1;
    let (_, body) = item.split_at(split_idx);
    let new_item = format!("{} {}", tag, body);
    let _ = item_view.remove_item(idx);
    item_view.insert_item(idx, new_item.to_string(), new_item.to_string());
    let _ = item_view.set_selection(idx);
}

fn make_event(item_view: &mut SelectView) -> Option<EventResult> {
    change_tag("o", item_view);
    Some(EventResult::Consumed(None))
}

fn make_note(item_view: &mut SelectView) -> Option<EventResult> {
    change_tag("-", item_view);
    Some(EventResult::Consumed(None))
}

fn make_task(item_view: &mut SelectView) -> Option<EventResult> {
    change_tag("•", item_view);
    Some(EventResult::Consumed(None))
}

fn toggle_completion(item_view: &mut SelectView) -> Option<EventResult> {
    let idx = item_view.selected_id().unwrap();
    let item = item_view.selection().unwrap();
    let split_idx = item.find(" ").unwrap();
    let (tag, _) = item.split_at(split_idx);
    let (_, body) = item.split_at(split_idx + 1);
    match tag {
        "•" => {
            let new_item = format!("× {}", body);
            let _ = item_view.remove_item(idx);
            item_view.insert_item(idx, new_item.to_string(), new_item.to_string());
            let _ = item_view.set_selection(idx);
        },
        "×" => {
            let new_item = format!("• {}", body);
            let _ = item_view.remove_item(idx);
            item_view.insert_item(idx, new_item.to_string(), new_item.to_string());
            let _ = item_view.set_selection(idx);
        }
        _ => ()
    };
    Some(EventResult::Consumed(None))
}

fn edit_item(siv: &mut Cursive, item: &str) {

    siv.screen_mut().add_layer_at(
        Position::new(Offset::Center, Offset::Parent(5)),
        Dialog::around(EditView::new().content(item).with_id("update"))
            .button("Update", |s| {
                // Look for a view tagged "text".
                // We _know_ it's there, so unwrap it.
                let update_view: ViewRef<EditView> = s.find_id("update").expect("Unable to get update view");
                let new_item = update_view.get_content();
                let mut event_view: ViewRef<OnEventView<SelectView>> = s.find_id("items").expect("Unable to get item view");
                let mut item_view: &mut SelectView = event_view.get_inner_mut();
                let selected = item_view.selected_id().unwrap();
                let cb = item_view.remove_item(selected);
                cb(s);
                item_view.insert_item(selected, new_item.to_string(), new_item.to_string());
                s.pop_layer();
            }).dismiss_button("Cancel"),
    );
}

fn add_item(s: &mut Cursive, item: &str) {
    if item.len() > 0 {
        let mut event_view: ViewRef<OnEventView<SelectView>> = s.find_id("items").expect("Unable to get item view");
        let item_view: &mut SelectView = event_view.get_inner_mut();
        // TODO: focus the item
        item_view.add_item_str(format!("• {}", item));
        let idx = item_view.len();
        let cb = item_view.set_selection(idx - 1);
        cb(s);
        s.call_on_id("new", |view: &mut EditView| {
            view.set_content("")
        });
        s.focus_id("items");
    }
}
