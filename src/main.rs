extern crate cursive;
extern crate chrono;
extern crate xdg;


// my modules
mod entry;
mod views;
use views::daily::daily_view;

// external modules
use chrono::Local;
use cursive::Cursive;
use cursive::event::Event;
use cursive::views::{Dialog, TextView};


fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();
    // Load today's data
    let today = Local::today();
    // TODO: move this into the DailyView
    siv.add_global_callback(Event::Char('q'), |s| {
            s.screen_mut().add_layer(
                Dialog::around(TextView::new("Are you sure you want to quit?"))
                    .dismiss_button("No")
                    .button("Yes", |s2| s2.quit()));
    });
    siv.add_layer(daily_view(today.naive_local()));

    // Starts the event loop.
    siv.run();
}
