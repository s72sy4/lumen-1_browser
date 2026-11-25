use crate::dialogs;
use crate::passwords::Vault;
use crate::settings::{wallpaper_dirs, SearchEngine, Settings};
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

pub fn build<F, G>(
    window: &gtk::ApplicationWindow,
    settings: Rc<RefCell<Settings>>,
    vault: Rc<Vault>,
    on_apply: F,
    on_close: G,
) -> gtk::EventBox
where
    F: Fn() + 'static,
    G: Fn() + 'static,
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

    let title = gtk::Label::new(Some("Einstellungen"));
    title.set_widget_name("settings-title");
    title.set_xalign(0.0);
    root.pack_start(&title, false, false, 0);

    let grid = gtk::Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(16);
    root.pack_start(&grid, false, false, 0);

    let engine_combo = gtk::ComboBoxText::new();
    engine_combo.append_text(SearchEngine::DuckDuckGo.label());
    engine_combo.append_text(SearchEngine::Google.label());
    engine_combo.append_text(SearchEngine::Startpage.label());
    engine_combo.set_active(Some(settings.borrow().search_engine as u32));
    labeled(&grid, "Suchmaschine", 0);
    grid.attach(&engine_combo, 1, 0, 1, 1);

    let wallpaper_entry = gtk::Entry::new();
    wallpaper_entry.set_hexpand(true);
    if let Some(path) = settings.borrow().wallpaper.as_ref() {
        wallpaper_entry.set_text(path);
    }
    let choose = gtk::Button::with_label("Datei …");
    let file_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    file_row.pack_start(&wallpaper_entry, true, true, 0);
    file_row.pack_start(&choose, false, false, 0);
    labeled(&grid, "Wallpaper", 1);
    grid.attach(&file_row, 1, 1, 1, 1);

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
    let wallpaper_files = Rc::new(files);
    let wallpaper_combo = gtk::ComboBoxText::new();
    for path in wallpaper_files.iter() {
        wallpaper_combo.append_text(&path.file_name().unwrap_or_default().to_string_lossy());
    }
    if let Some(current) = settings.borrow().wallpaper.as_ref() {
        let current = PathBuf::from(current);
        if let Some(index) = wallpaper_files.iter().position(|path| *path == current) {
            wallpaper_combo.set_active(Some(index as u32));
        }
    }
    labeled(&grid, "Wallpaper wechseln", 2);
    grid.attach(&wallpaper_combo, 1, 2, 1, 1);

    let settings_for_wp = Rc::clone(&settings);
    let files_for_wp = Rc::clone(&wallpaper_files);
    let entry_for_wp = wallpaper_entry.clone();
    let apply_for_wp = Rc::clone(&apply);
    wallpaper_combo.connect_changed(move |combo| {
        if let Some(index) = combo.active() {
            if let Some(path) = files_for_wp.get(index as usize) {
                let value = path.to_string_lossy().to_string();
                entry_for_wp.set_text(&value);
                settings_for_wp.borrow_mut().wallpaper = Some(value);
                let _ = settings_for_wp.borrow().save();
                apply_for_wp();
            }
        }
    });

    let shortcut_entry = gtk::Entry::new();
    shortcut_entry.set_text(&settings.borrow().tab_shortcut);
    labeled(&grid, "Tab-Shortcut", 3);
    grid.attach(&shortcut_entry, 1, 3, 1, 1);

    let transparent = gtk::CheckButton::with_label("Transparenz-Modus (nur Desktop sichtbar)");
    transparent.set_active(settings.borrow().transparent);
    grid.attach(&transparent, 1, 4, 1, 1);

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
    labeled(&grid, "Deckkraft", 5);
    grid.attach(&opacity, 1, 5, 1, 1);

    let adblock = gtk::CheckButton::with_label("Werbeblocker aktivieren");
    adblock.set_active(settings.borrow().adblock);
    grid.attach(&adblock, 1, 6, 1, 1);

    let profile_entry = gtk::Entry::new();
    profile_entry.set_text(&settings.borrow().profile);
    labeled(&grid, "Profil", 7);
    grid.attach(&profile_entry, 1, 7, 1, 1);

    let vault_button = gtk::Button::with_label("Passwörter verwalten …");
    grid.attach(&vault_button, 1, 8, 1, 1);

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
        let current = entry_for_chooser.text();
        chooser.set_filename(&current);
        let target = entry_for_chooser.clone();
        let settings_for_chooser = Rc::clone(&settings_for_chooser);
        let apply_for_chooser = Rc::clone(&apply_for_chooser);
        chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Ok {
                if let Some(path) = chooser.filename() {
                    let value = path.to_string_lossy().to_string();
                    target.set_text(&value);
                    settings_for_chooser.borrow_mut().wallpaper = Some(value);
                    let _ = settings_for_chooser.borrow().save();
                    apply_for_chooser();
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
        let profile = profile_entry.text().to_string();
        {
            let mut current = settings_for_save.borrow_mut();
            current.search_engine = SearchEngine::from_index(engine_combo.active().unwrap_or(0));
            current.wallpaper = wallpaper;
            current.tab_shortcut = shortcut_entry.text().to_string();
            current.transparent = transparent.is_active();
            current.opacity = opacity.value();
            current.adblock = adblock.is_active();
            if !profile.is_empty() && !current.profiles.contains(&profile) {
                current.profiles.push(profile.clone());
            }
            if !profile.is_empty() {
                current.profile = profile;
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
