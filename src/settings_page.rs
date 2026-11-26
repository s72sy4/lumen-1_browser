use crate::dialogs;
use crate::history::BrowserData;
use crate::passwords::Vault;
use crate::settings::{wallpaper_dirs, Language, SearchEngine, Settings};
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

fn labeled(grid: &gtk::Grid, text: &str, row: i32) {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.style_context().add_class("settings-label");
    grid.attach(&label, 0, row, 1, 1);
}

fn shortcut_row(grid: &gtk::Grid, text: &str, value: &str, row: i32) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(value);
    labeled(grid, text, row);
    grid.attach(&entry, 1, row, 1, 1);
    entry
}

fn section(title: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    label.style_context().add_class("settings-section");
    label
}

pub fn build<F, G, H>(
    window: &gtk::ApplicationWindow,
    settings: Rc<RefCell<Settings>>,
    vault: Rc<Vault>,
    data: Rc<RefCell<BrowserData>>,
    on_apply: F,
    on_close: G,
    on_open_url: H,
) -> gtk::EventBox
where
    F: Fn() + 'static,
    G: Fn() + 'static,
    H: Fn(String) + 'static,
{
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.set_margin(28);

    let shell = gtk::EventBox::new();
    shell.set_widget_name("lumen-1_browser-settings");

    let scroller = gtk::ScrolledWindow::builder().build();
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.add(&root);
    shell.add(&scroller);

    let apply = Rc::new(on_apply);
    let close_page = Rc::new(on_close);
    let open_url = Rc::new(on_open_url);

    let title = gtk::Label::new(Some("Einstellungen"));
    title.set_widget_name("settings-title");
    title.set_xalign(0.0);

    let close = gtk::Button::with_label("×");
    close.set_widget_name("lumen-1_browser-button");
    close.set_tooltip_text(Some("Einstellungen schließen"));
    let close_for_button = Rc::clone(&close_page);
    close.connect_clicked(move |_| close_for_button());

    let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    title_row.pack_start(&title, true, true, 0);
    title_row.pack_start(&close, false, false, 0);
    root.pack_start(&title_row, false, false, 0);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    root.pack_start(&body, true, true, 0);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeft);
    stack.set_transition_duration(220);

    let sidebar = gtk::StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_widget_name("lumen-1_browser-settings-sidebar");
    sidebar.set_size_request(170, -1);
    body.pack_start(&sidebar, false, false, 0);
    body.pack_start(&stack, true, true, 0);

    let wallpaper_entry = gtk::Entry::new();
    wallpaper_entry.set_hexpand(true);
    wallpaper_entry.set_no_show_all(true);
    wallpaper_entry.hide();
    if let Some(path) = settings.borrow().wallpaper.as_ref() {
        wallpaper_entry.set_text(path);
    }

    let appearance = gtk::Box::new(gtk::Orientation::Vertical, 12);
    appearance.pack_start(&section("Hintergrundbild"), false, false, 0);

    let choose = gtk::Button::with_label("Eigenes Bild …");
    let file_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    file_row.pack_start(&choose, false, false, 0);
    appearance.pack_start(&file_row, false, false, 0);

    let mut files: Vec<PathBuf> = Vec::new();
    let mut seen = HashSet::new();
    for dir in wallpaper_dirs() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let supported = matches!(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_ascii_lowercase())
                    .as_deref(),
                Some("png") | Some("jpg") | Some("jpeg") | Some("webp")
            );
            if supported && seen.insert(path.clone()) {
                files.push(path);
            }
        }
    }
    files.sort();

    let flow = gtk::FlowBox::new();
    flow.set_widget_name("lumen-wallpaper-grid");
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_min_children_per_line(2);
    flow.set_max_children_per_line(4);
    flow.set_column_spacing(12);
    flow.set_row_spacing(12);

    let cards: Rc<RefCell<Vec<gtk::ToggleButton>>> = Rc::new(RefCell::new(Vec::new()));
    let current = settings.borrow().wallpaper.clone().map(PathBuf::from);

    for path in files {
        let button = gtk::ToggleButton::new();
        button.style_context().add_class("wallpaper-card");
        button.set_tooltip_text(Some(&path.to_string_lossy()));

        if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file_at_scale(&path, 220, 124, true) {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
            image.set_size_request(220, 124);
            button.add(&image);
        }

        if current.as_ref() == Some(&path) {
            button.set_active(true);
        }

        cards.borrow_mut().push(button.clone());

        let cards_for_card = Rc::clone(&cards);
        let settings_for_card = Rc::clone(&settings);
        let entry_for_card = wallpaper_entry.clone();
        let apply_for_card = Rc::clone(&apply);
        let value = path.to_string_lossy().to_string();
        button.connect_toggled(move |button| {
            if !button.is_active() {
                return;
            }
            for other in cards_for_card.borrow().iter() {
                if other != button {
                    other.set_active(false);
                }
            }
            entry_for_card.set_text(&value);
            settings_for_card.borrow_mut().wallpaper = Some(value.clone());
            let _ = settings_for_card.borrow().save();
            apply_for_card();
        });

        flow.add(&button);
    }
    appearance.pack_start(&flow, true, true, 0);

    let transparent = gtk::CheckButton::with_label("Transparenz-Modus (nur Desktop sichtbar)");
    transparent.set_active(settings.borrow().transparent);
    appearance.pack_start(&transparent, false, false, 0);

    let settings_for_transparent = Rc::clone(&settings);
    let apply_for_transparent = Rc::clone(&apply);
    transparent.connect_toggled(move |button| {
        settings_for_transparent.borrow_mut().transparent = button.is_active();
        let _ = settings_for_transparent.borrow().save();
        apply_for_transparent();
    });

    let opacity = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.3, 1.0, 0.05);
    opacity.set_hexpand(true);
    opacity.set_value(settings.borrow().opacity);
    opacity.set_digits(2);
    let opacity_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let opacity_label = gtk::Label::new(Some("Deckkraft"));
    opacity_label.set_xalign(0.0);
    opacity_label.style_context().add_class("settings-label");
    opacity_row.pack_start(&opacity_label, false, false, 0);
    opacity_row.pack_start(&opacity, true, true, 0);
    appearance.pack_start(&opacity_row, false, false, 0);

    let hardware = gtk::CheckButton::with_label("Hardwarebeschleunigung (weniger Lag)");
    hardware.set_active(settings.borrow().hardware_acceleration);
    appearance.pack_start(&hardware, false, false, 0);

    let settings_for_hardware = Rc::clone(&settings);
    hardware.connect_toggled(move |button| {
        settings_for_hardware.borrow_mut().hardware_acceleration = button.is_active();
        let _ = settings_for_hardware.borrow().save();
    });

    stack.add_titled(&appearance, "appearance", "Darstellung");

    let search_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    search_page.pack_start(&section("Suchen"), false, false, 0);
    let search_grid = gtk::Grid::new();
    search_grid.set_row_spacing(12);
    search_grid.set_column_spacing(16);

    let engine_combo = gtk::ComboBoxText::new();
    engine_combo.append_text(SearchEngine::DuckDuckGo.label());
    engine_combo.append_text(SearchEngine::Google.label());
    engine_combo.append_text(SearchEngine::Startpage.label());
    engine_combo.set_active(Some(settings.borrow().search_engine as u32));
    labeled(&search_grid, "Suchmaschine", 0);
    search_grid.attach(&engine_combo, 1, 0, 1, 1);

    let language_combo = gtk::ComboBoxText::new();
    language_combo.append_text(Language::Deutsch.label());
    language_combo.append_text(Language::English.label());
    language_combo.set_active(Some(settings.borrow().language as u32));
    labeled(&search_grid, "Sprache", 1);
    search_grid.attach(&language_combo, 1, 1, 1, 1);
    search_page.pack_start(&search_grid, false, false, 0);
    stack.add_titled(&search_page, "search", "Suchen");

    let keys_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    keys_page.pack_start(&section("Tastenkürzel"), false, false, 0);
    let keys_grid = gtk::Grid::new();
    keys_grid.set_row_spacing(12);
    keys_grid.set_column_spacing(16);

    let keys = {
        let current = settings.borrow();
        (
            current.tab_shortcut.clone(),
            current.new_tab_shortcut.clone(),
            current.close_tab_shortcut.clone(),
            current.url_shortcut.clone(),
            current.bookmark_shortcut.clone(),
            current.settings_shortcut.clone(),
            current.private_shortcut.clone(),
        )
    };
    let tab_entry = shortcut_row(&keys_grid, "Tab-Liste", &keys.0, 0);
    let new_entry = shortcut_row(&keys_grid, "Neuer Tab", &keys.1, 1);
    let close_entry = shortcut_row(&keys_grid, "Tab schließen", &keys.2, 2);
    let url_entry = shortcut_row(&keys_grid, "Adressleiste", &keys.3, 3);
    let bookmark_entry = shortcut_row(&keys_grid, "Lesezeichen", &keys.4, 4);
    let settings_entry = shortcut_row(&keys_grid, "Einstellungen", &keys.5, 5);
    let private_entry = shortcut_row(&keys_grid, "Privater Tab", &keys.6, 6);

    keys_page.pack_start(&keys_grid, false, false, 0);
    stack.add_titled(&keys_page, "keys", "Tastenkürzel");

    let profile_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    profile_page.pack_start(&section("Profil & Passwörter"), false, false, 0);
    let profile_grid = gtk::Grid::new();
    profile_grid.set_row_spacing(12);
    profile_grid.set_column_spacing(16);

    let profile_combo = gtk::ComboBoxText::new();
    {
        let current = settings.borrow();
        for name in &current.profiles {
            profile_combo.append_text(name);
        }
        if let Some(index) = current
            .profiles
            .iter()
            .position(|name| name == &current.profile)
        {
            profile_combo.set_active(Some(index as u32));
        }
    }
    labeled(&profile_grid, "Profil", 0);
    profile_grid.attach(&profile_combo, 1, 0, 1, 1);

    let profile_name = gtk::Entry::new();
    profile_name.set_placeholder_text(Some("Profilname"));
    labeled(&profile_grid, "Neuer Name", 1);
    profile_grid.attach(&profile_name, 1, 1, 1, 1);

    let add_profile = gtk::Button::with_label("Hinzufügen");
    let rename_profile = gtk::Button::with_label("Umbenennen");
    let delete_profile = gtk::Button::with_label("Löschen");
    let profile_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    profile_actions.pack_start(&add_profile, false, false, 0);
    profile_actions.pack_start(&rename_profile, false, false, 0);
    profile_actions.pack_start(&delete_profile, false, false, 0);
    profile_grid.attach(&profile_actions, 1, 2, 1, 1);

    let vault_button = gtk::Button::with_label("Passwörter verwalten …");
    profile_grid.attach(&vault_button, 1, 3, 1, 1);

    let settings_for_profiles = Rc::clone(&settings);
    profile_combo.connect_changed(move |combo| {
        if let Some(index) = combo.active() {
            let name = settings_for_profiles
                .borrow()
                .profiles
                .get(index as usize)
                .cloned();
            if let Some(name) = name {
                settings_for_profiles.borrow_mut().profile = name;
                let _ = settings_for_profiles.borrow().save();
            }
        }
    });

    let settings_for_add = Rc::clone(&settings);
    let combo_for_add = profile_combo.clone();
    let name_for_add = profile_name.clone();
    add_profile.connect_clicked(move |_| {
        let name = name_for_add.text().to_string();
        if name.is_empty() {
            return;
        }
        let mut current = settings_for_add.borrow_mut();
        if current.profiles.contains(&name) {
            return;
        }
        current.profiles.push(name.clone());
        current.profile = name;
        let _ = current.save();
        drop(current);
        combo_for_add.remove_all();
        for item in settings_for_add.borrow().profiles.clone() {
            combo_for_add.append_text(&item);
        }
        let index = settings_for_add.borrow().profiles.len() - 1;
        combo_for_add.set_active(Some(index as u32));
    });

    let settings_for_rename = Rc::clone(&settings);
    let combo_for_rename = profile_combo.clone();
    let name_for_rename = profile_name.clone();
    rename_profile.connect_clicked(move |_| {
        let name = name_for_rename.text().to_string();
        if name.is_empty() {
            return;
        }
        let mut current = settings_for_rename.borrow_mut();
        if let Some(index) = combo_for_rename.active() {
            let index = index as usize;
            if current.profiles.contains(&name) && current.profiles.get(index) != Some(&name) {
                return;
            }
            if let Some(old) = current.profiles.get(index).cloned() {
                current.profiles[index] = name.clone();
                if current.profile == old {
                    current.profile = name;
                }
                let _ = current.save();
            }
        }
        drop(current);
        combo_for_rename.remove_all();
        for item in settings_for_rename.borrow().profiles.clone() {
            combo_for_rename.append_text(&item);
        }
    });

    let settings_for_delete = Rc::clone(&settings);
    let combo_for_delete = profile_combo.clone();
    delete_profile.connect_clicked(move |_| {
        let mut current = settings_for_delete.borrow_mut();
        if current.profiles.len() <= 1 {
            return;
        }
        if let Some(index) = combo_for_delete.active() {
            current.profiles.remove(index as usize);
            let next = current
                .profiles
                .first()
                .cloned()
                .unwrap_or_else(|| "Standard".to_string());
            current.profile = next;
            let _ = current.save();
        }
        drop(current);
        combo_for_delete.remove_all();
        for item in settings_for_delete.borrow().profiles.clone() {
            combo_for_delete.append_text(&item);
        }
        combo_for_delete.set_active(Some(0));
    });

    profile_page.pack_start(&profile_grid, false, false, 0);
    stack.add_titled(&profile_page, "profile", "Profil");

    let history_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    history_page.pack_start(&section("Verlauf"), false, false, 0);

    let clear_history = gtk::Button::with_label("Verlauf löschen");
    history_page.pack_start(&clear_history, false, false, 0);

    let history_list = gtk::ListBox::new();
    history_list.set_widget_name("lumen-history-list");
    for entry in data.borrow().history.iter() {
        let row = gtk::ListBoxRow::new();
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let label = gtk::Label::new(Some(&format!(
            "{}\n{}\n{}",
            entry.title, entry.url, entry.visited
        )));
        label.set_xalign(0.0);
        label.set_line_wrap(true);
        label.set_hexpand(true);
        label.style_context().add_class("history-entry");
        row_box.pack_start(&label, true, true, 0);

        let open = gtk::Button::with_label("Öffnen");
        let url = entry.url.clone();
        let open_for_row = Rc::clone(&open_url);
        open.connect_clicked(move |_| open_for_row(url.clone()));
        row_box.pack_start(&open, false, false, 0);
        row.add(&row_box);
        history_list.add(&row);
    }
    history_page.pack_start(&history_list, true, true, 0);

    let data_for_clear = Rc::clone(&data);
    let list_for_clear = history_list.clone();
    clear_history.connect_clicked(move |_| {
        data_for_clear.borrow_mut().clear_history();
        for child in list_for_clear.children() {
            child.hide();
        }
    });
    stack.add_titled(&history_page, "history", "Verlauf");

    let bookmarks_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    bookmarks_page.pack_start(&section("Lesezeichen"), false, false, 0);
    let hint = gtk::Label::new(Some("Mit Strg+D kannst du die aktuelle Seite speichern."));
    hint.set_xalign(0.0);
    hint.style_context().add_class("settings-hint");
    bookmarks_page.pack_start(&hint, false, false, 0);

    let bookmark_list = gtk::ListBox::new();
    bookmark_list.set_widget_name("lumen-bookmark-list");
    for entry in data.borrow().bookmarks.iter() {
        let row = gtk::ListBoxRow::new();
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let label = gtk::Label::new(Some(&format!("{}\n{}", entry.title, entry.url)));
        label.set_xalign(0.0);
        label.set_line_wrap(true);
        label.set_hexpand(true);
        label.style_context().add_class("history-entry");
        row_box.pack_start(&label, true, true, 0);

        let remove = gtk::Button::with_label("Entfernen");
        let url = entry.url.clone();
        let data_for_remove = Rc::clone(&data);
        let row_for_remove = row.clone();
        remove.connect_clicked(move |_| {
            data_for_remove.borrow_mut().remove_bookmark(&url);
            row_for_remove.hide();
        });
        row_box.pack_start(&remove, false, false, 0);

        let open = gtk::Button::with_label("Öffnen");
        let url = entry.url.clone();
        let open_for_row = Rc::clone(&open_url);
        open.connect_clicked(move |_| open_for_row(url.clone()));
        row_box.pack_start(&open, false, false, 0);

        row.add(&row_box);
        bookmark_list.add(&row);
    }
    bookmarks_page.pack_start(&bookmark_list, true, true, 0);
    stack.add_titled(&bookmarks_page, "bookmarks", "Lesezeichen");

    let credits_page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    credits_page.pack_start(&section("Credits"), false, false, 0);
    let made = gtk::Label::new(Some("Entwickelt von s72sy"));
    made.set_xalign(0.0);
    credits_page.pack_start(&made, false, false, 0);
    let stack_label = gtk::Label::new(Some("Rust · GTK · WebKitGTK · libsoup"));
    stack_label.set_xalign(0.0);
    stack_label.style_context().add_class("settings-hint");
    credits_page.pack_start(&stack_label, false, false, 0);
    let link = gtk::LinkButton::with_label("https://guns.lol/s72sy", "guns.lol/s72sy");
    link.set_halign(gtk::Align::Start);
    credits_page.pack_start(&link, false, false, 0);
    let repo =
        gtk::LinkButton::with_label("https://github.com/s72sy4/lumen-1_browser", "GitHub-Repo");
    repo.set_halign(gtk::Align::Start);
    credits_page.pack_start(&repo, false, false, 0);
    stack.add_titled(&credits_page, "credits", "Credits");

    let chooser_parent = window.clone();
    let entry_for_chooser = wallpaper_entry.clone();
    let settings_for_chooser = Rc::clone(&settings);
    let apply_for_chooser = Rc::clone(&apply);
    choose.connect_clicked(move |_| {
        let chooser = gtk::FileChooserDialog::new(
            Some("Wallpaper wählen"),
            Some(&chooser_parent),
            gtk::FileChooserAction::Open,
        );
        chooser.add_button("Abbrechen", gtk::ResponseType::Cancel);
        chooser.add_button("Öffnen", gtk::ResponseType::Ok);
        chooser.set_filename(entry_for_chooser.text());
        let target = entry_for_chooser.clone();
        let settings = Rc::clone(&settings_for_chooser);
        let apply = Rc::clone(&apply_for_chooser);
        chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Ok {
                if let Some(path) = chooser.filename() {
                    let value = path.to_string_lossy().to_string();
                    target.set_text(&value);
                    settings.borrow_mut().wallpaper = Some(value);
                    let _ = settings.borrow().save();
                    apply();
                }
            }
            chooser.close();
        });
        chooser.show_all();
    });

    let vault_for_button = Rc::clone(&vault);
    let vault_parent = window.clone();
    vault_button.connect_clicked(move |_| {
        dialogs::vault_dialog(&vault_parent, Rc::clone(&vault_for_button));
    });

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_halign(gtk::Align::End);
    let save = gtk::Button::with_label("Speichern");
    save.style_context().add_class("suggested-action");
    let back = gtk::Button::with_label("Zurück");
    actions.pack_start(&back, false, false, 0);
    actions.pack_start(&save, false, false, 0);
    root.pack_start(&actions, false, false, 8);

    let window_for_error = window.clone();
    let settings_for_save = Rc::clone(&settings);
    let close_for_save = Rc::clone(&close_page);
    save.connect_clicked(move |_| {
        let wallpaper = {
            let value = wallpaper_entry.text().to_string();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        {
            let mut current = settings_for_save.borrow_mut();
            current.search_engine = SearchEngine::from_index(engine_combo.active().unwrap_or(0));
            current.language = Language::from_index(language_combo.active().unwrap_or(0));
            current.wallpaper = wallpaper;
            current.tab_shortcut = tab_entry.text().to_string();
            current.new_tab_shortcut = new_entry.text().to_string();
            current.close_tab_shortcut = close_entry.text().to_string();
            current.url_shortcut = url_entry.text().to_string();
            current.bookmark_shortcut = bookmark_entry.text().to_string();
            current.settings_shortcut = settings_entry.text().to_string();
            current.private_shortcut = private_entry.text().to_string();
            current.transparent = transparent.is_active();
            current.opacity = opacity.value();
            current.hardware_acceleration = hardware.is_active();
            if let Some(index) = profile_combo.active() {
                if let Some(name) = current.profiles.get(index as usize) {
                    current.profile = name.clone();
                }
            }
            if let Err(error) = current.save() {
                dialogs::error_dialog(&window_for_error, &error.to_string());
                return;
            }
        }
        apply();
        close_for_save();
    });

    back.connect_clicked(move |_| close_page());

    shell
}
