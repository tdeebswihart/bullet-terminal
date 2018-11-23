use std::rc::Rc;

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

pub struct DailyView {
    // TODO: move the daily view presentation and event-handling logic to here
    date: Date<Local>,
    entries: Vec<Rc<Entry>>
}

pub struct WeeklyView {

}

pub struct MonthlyView {

}

pub struct SearchView {

}
