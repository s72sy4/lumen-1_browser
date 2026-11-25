use chrono::Local;
use gtk::prelude::*;

pub fn label() -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_widget_name("lumen-1_browser-clock");
    update(&label);

    let timer_label = label.clone();
    glib::timeout_add_seconds_local(15, move || {
        update(&timer_label);
        glib::ControlFlow::Continue
    });

    label
}

fn update(label: &gtk::Label) {
    let now = Local::now();
    label.set_label(&now.format("%H:%M").to_string());
    label.set_tooltip_text(Some(&now.format("%A, %d. %B %Y").to_string()));
}
