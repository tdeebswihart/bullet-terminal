extern crate cursive;

use cursive::Cursive;
use cursive::views::{Dialog, TextView, LinearLayout, EditView};

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = Cursive::default();

    // Creates a dialog with a single "Quit" button
    siv.add_layer(LinearLayout::vertical()
                  .child(Dialog::around(TextView::new("Hello Dialog!"))
                         .title("Cursive")
                         .button("Quit", |s| s.quit()))
                  .child(EditView::new()));

    // Starts the event loop.
    siv.run();
}
