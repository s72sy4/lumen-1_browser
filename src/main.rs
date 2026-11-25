mod adblock;
mod clock;
mod dialogs;
mod passwords;
mod settings;
mod settings_page;
mod tabs;
mod theme;

use gdk::prelude::*;
use gtk::prelude::*;
use settings::{SearchEngine, Settings};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let app = gtk::Application::builder()
        .application_id("de.lumen_1_browser.browser")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn button(label: &str) -> gtk::Button {
    let widget = gtk::Button::with_label(label);
    widget.set_widget_name("lumen-1_browser-button");
    widget
}

fn resolve_input(input: &str, engine: SearchEngine) -> String {
    let value = input.trim();
    if value.is_empty() {
        return String::new();
    }
    if value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("file://")
        || (value.contains('.') && !value.contains(' '))
    {
        if value.contains("://") {
            value.to_string()
        } else {
            format!("https://{}", value)
        }
    } else {
        engine.query_url(value)
    }
}

fn window_opacity(settings: &Settings) -> f64 {
    if settings.transparent {
        settings.opacity
    } else {
        1.0
    }
}

fn add_action<F: Fn() + 'static>(window: &gtk::ApplicationWindow, name: &str, callback: F) {
    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| callback());
    window.add_action(&action);
}

fn shortcut_matches(accelerator: &str, event: &gdk::EventKey) -> bool {
    let (key, modifiers) = gtk::accelerator_parse(accelerator);
    let relevant = gdk::ModifierType::CONTROL_MASK
        | gdk::ModifierType::SHIFT_MASK
        | gdk::ModifierType::MOD1_MASK
        | gdk::ModifierType::SUPER_MASK;
    let keyval: u32 = *event.keyval();
    key == keyval && modifiers == (event.state() & relevant)
}

fn build_ui(app: &gtk::Application) {
    let settings = Rc::new(RefCell::new(Settings::load()));
    let vault = Rc::new(passwords::Vault::new());

    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(1280, 800);
    window.set_decorated(false);
    window.set_app_paintable(true);
    window.set_widget_name("lumen-1_browser-window");
    if let Some(screen) = gdk::Screen::default() {
        if let Some(visual) = screen.rgba_visual() {
            window.set_visual(Some(&visual));
        }
    }
    {
        let current = settings.borrow();
        theme::apply(current.wallpaper.as_deref());
        window.set_opacity(window_opacity(&current));
    }

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.add(&root);

    let header = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header_shell = gtk::EventBox::new();
    header_shell.set_widget_name("lumen-1_browser-header");
    header_shell.add(&header);
    root.pack_start(&header_shell, false, false, 0);

    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    bar.set_margin(8);
    header.pack_start(&bar, false, false, 0);

    let progress = gtk::ProgressBar::new();
    progress.set_widget_name("lumen-1_browser-progress");
    progress.set_show_text(false);
    progress.set_size_request(-1, 2);
    header.pack_start(&progress, false, false, 0);

    let back = button("←");
    let forward = button("→");
    let reload = button("↻");
    let new_tab_button = button("+");
    let tab_list_button = button("☰");
    let settings_button = button("⚙");

    let entry = gtk::Entry::new();
    entry.set_hexpand(true);
    entry.set_widget_name("lumen-1_browser-entry");
    entry.set_placeholder_text(Some("Suche oder URL eingeben …"));

    bar.pack_start(&back, false, false, 0);
    bar.pack_start(&forward, false, false, 0);
    bar.pack_start(&reload, false, false, 0);
    bar.pack_start(&entry, true, true, 0);
    bar.pack_start(&clock::label(), false, false, 0);
    bar.pack_start(&tab_list_button, false, false, 0);
    bar.pack_start(&new_tab_button, false, false, 0);
    bar.pack_start(&settings_button, false, false, 0);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.pack_start(&body, true, true, 0);

    let panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
    panel.set_size_request(230, -1);

    let panel_shell = gtk::EventBox::new();
    panel_shell.set_widget_name("lumen-1_browser-panel");
    panel_shell.add(&panel);

    let panel_revealer = gtk::Revealer::new();
    panel_revealer.set_widget_name("lumen-1_browser-panel-revealer");
    panel_revealer.set_transition_type(gtk::RevealerTransitionType::SlideRight);
    panel_revealer.set_transition_duration(260);
    panel_revealer.add(&panel_shell);
    panel_revealer.set_reveal_child(false);
    panel_revealer.connect_child_revealed_notify(|revealer| {
        if !revealer.is_child_revealed() {
            revealer.hide();
        }
    });
    body.pack_start(&panel_revealer, false, false, 0);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let stage = gtk::Overlay::new();
    stage.add(&content);

    let settings_host = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let settings_revealer = gtk::Revealer::new();
    settings_revealer.set_widget_name("lumen-1_browser-settings-revealer");
    settings_revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    settings_revealer.set_transition_duration(320);
    settings_revealer.set_halign(gtk::Align::Fill);
    settings_revealer.set_valign(gtk::Align::Fill);
    settings_revealer.set_hexpand(true);
    settings_revealer.set_vexpand(true);
    settings_revealer.add(&settings_host);
    settings_revealer.set_reveal_child(false);
    settings_revealer.connect_child_revealed_notify(|revealer| {
        if !revealer.is_child_revealed() {
            revealer.hide();
        }
    });
    stage.add_overlay(&settings_revealer);
    body.pack_start(&stage, true, true, 0);

    let tabs = tabs::TabManager::new(content, panel, progress, Rc::clone(&settings));
    tabs.new_tab("lumen-1-browser://start");

    let tabs_for_entry = Rc::clone(&tabs);
    let settings_for_entry = Rc::clone(&settings);
    entry.connect_activate(move |entry| {
        let input = entry.text().to_string();
        let url = resolve_input(&input, settings_for_entry.borrow().search_engine);
        if !url.is_empty() {
            tabs_for_entry.load_active(&url);
        }
    });

    let tabs_for_back = Rc::clone(&tabs);
    back.connect_clicked(move |_| tabs_for_back.go_back());
    let tabs_for_forward = Rc::clone(&tabs);
    forward.connect_clicked(move |_| tabs_for_forward.go_forward());
    let tabs_for_reload = Rc::clone(&tabs);
    reload.connect_clicked(move |_| tabs_for_reload.reload());
    let tabs_for_new = Rc::clone(&tabs);
    new_tab_button.connect_clicked(move |_| tabs_for_new.new_tab("lumen-1-browser://start"));
    let toggle_panel = {
        let revealer = panel_revealer.clone();
        Rc::new(move || {
            if revealer.reveals_child() {
                revealer.set_reveal_child(false);
            } else {
                revealer.show_all();
                revealer.set_reveal_child(true);
            }
        })
    };
    let toggle_for_button = Rc::clone(&toggle_panel);
    tab_list_button.connect_clicked(move |_| toggle_for_button());

    let revealer_for_open = settings_revealer.clone();
    let window_for_open = window.clone();
    let open_settings = Rc::new(move || {
        window_for_open.set_opacity(1.0);
        revealer_for_open.show_all();
        revealer_for_open.set_reveal_child(true);
    });
    let revealer_for_close = settings_revealer.clone();
    let window_for_close = window.clone();
    let settings_for_close = Rc::clone(&settings);
    let close_settings = Rc::new(move || {
        revealer_for_close.set_reveal_child(false);
        window_for_close.set_opacity(window_opacity(&settings_for_close.borrow()));
    });
    let settings_for_apply = Rc::clone(&settings);
    let window_for_apply = window.clone();
    let tabs_for_apply = Rc::clone(&tabs);
    let apply_settings = Rc::new(move || {
        {
            let current = settings_for_apply.borrow();
            theme::apply(current.wallpaper.as_deref());
            window_for_apply.set_opacity(window_opacity(&current));
        }
        tabs_for_apply.refresh_start_page();
    });
    let page = settings_page::build(
        &window,
        Rc::clone(&settings),
        Rc::clone(&vault),
        {
            let apply = Rc::clone(&apply_settings);
            move || apply()
        },
        {
            let close = Rc::clone(&close_settings);
            move || close()
        },
    );
    settings_host.pack_start(&page, true, true, 0);

    let open_for_button = Rc::clone(&open_settings);
    settings_button.connect_clicked(move |_| open_for_button());

    let toggle_for_action = Rc::clone(&toggle_panel);
    add_action(&window, "tab-list", move || toggle_for_action());
    let tabs_for_action = Rc::clone(&tabs);
    add_action(&window, "new-tab", move || {
        tabs_for_action.new_tab("lumen-1-browser://start")
    });
    let tabs_for_action = Rc::clone(&tabs);
    add_action(&window, "close-tab", move || tabs_for_action.close_active());
    let open_for_action = Rc::clone(&open_settings);
    add_action(&window, "settings", move || open_for_action());
    let window_for_focus = window.clone();
    let entry_for_focus = entry.clone();
    add_action(&window, "focus-url", move || {
        window_for_focus.set_focus(Some(&entry_for_focus));
        entry_for_focus.select_region(0, -1);
    });

    let settings_for_keys = Rc::clone(&settings);
    let window_for_keys = window.clone();
    let revealer_for_keys = settings_revealer.clone();
    let panel_for_keys = panel_revealer.clone();
    let close_for_keys = Rc::clone(&close_settings);
    window.connect_key_press_event(move |_, event| {
        if event.keyval() == gdk::keys::constants::Escape {
            if revealer_for_keys.reveals_child() {
                close_for_keys();
                return glib::Propagation::Stop;
            }
            if panel_for_keys.reveals_child() {
                panel_for_keys.set_reveal_child(false);
                return glib::Propagation::Stop;
            }
        }
        let configured = settings_for_keys.borrow().tab_shortcut.clone();
        let name = if shortcut_matches(&configured, event) {
            Some("tab-list")
        } else if shortcut_matches("<Control>T", event) {
            Some("new-tab")
        } else if shortcut_matches("<Control>W", event) {
            Some("close-tab")
        } else if shortcut_matches("<Control>K", event) {
            Some("focus-url")
        } else if shortcut_matches("<Control>comma", event) {
            Some("settings")
        } else {
            None
        };
        if let Some(name) = name {
            if let Some(action) = window_for_keys.lookup_action(name) {
                action.activate(None);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    window.show_all();
    settings_revealer.set_reveal_child(false);
    settings_revealer.hide();
    panel_revealer.set_reveal_child(false);
    panel_revealer.hide();
}
