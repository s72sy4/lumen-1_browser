use crate::passwords::{Credential, Vault};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn error_dialog<T: IsA<gtk::Window>>(parent: &T, message: &str) {
    let dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        message,
    );
    dialog.run();
    dialog.close();
}

pub(crate) fn vault_dialog<T: IsA<gtk::Window> + Clone + 'static>(window: &T, vault: Rc<Vault>) {
    if !vault.configured() {
        let dialog = gtk::Dialog::with_buttons(
            Some("Master-Passwort einrichten"),
            Some(window),
            gtk::DialogFlags::MODAL,
            &[
                ("Abbrechen", gtk::ResponseType::Cancel),
                ("Einrichten", gtk::ResponseType::Ok),
            ],
        );
        let area = dialog.content_area();
        let password = gtk::Entry::new();
        password.set_visibility(false);
        password.set_placeholder_text(Some("Mindestens 8 Zeichen"));
        let confirm = gtk::Entry::new();
        confirm.set_visibility(false);
        confirm.set_placeholder_text(Some("Wiederholen"));
        area.pack_start(&password, false, false, 4);
        area.pack_start(&confirm, false, false, 4);
        dialog.show_all();
        let vault_for_setup = Rc::clone(&vault);
        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Ok {
                if password.text() != confirm.text() {
                    error_dialog(dialog, "Die Passwörter stimmen nicht überein");
                    return;
                }
                if let Err(error) = vault_for_setup.setup(&password.text()) {
                    error_dialog(dialog, &error.to_string());
                    return;
                }
            }
            dialog.close();
        });
        return;
    }

    let dialog = gtk::Dialog::with_buttons(
        Some("Passwort-Tresor entsperren"),
        Some(window),
        gtk::DialogFlags::MODAL,
        &[
            ("Abbrechen", gtk::ResponseType::Cancel),
            ("Entsperren", gtk::ResponseType::Ok),
        ],
    );
    let area = dialog.content_area();
    let password = gtk::Entry::new();
    password.set_visibility(false);
    area.pack_start(&password, false, false, 8);
    dialog.show_all();
    let vault_for_unlock = Rc::clone(&vault);
    let parent = window.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Ok {
            match vault_for_unlock.unlock(&password.text()) {
                Ok(store) => {
                    dialog.close();
                    manage_vault(&parent, Rc::new(RefCell::new(store)));
                }
                Err(error) => error_dialog(dialog, &error.to_string()),
            }
        } else {
            dialog.close();
        }
    });
}

fn manage_vault<T: IsA<gtk::Window> + Clone + 'static>(
    window: &T,
    store: Rc<RefCell<crate::passwords::CredentialStore>>,
) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Passwörter"),
        Some(window),
        gtk::DialogFlags::MODAL,
        &[("Schließen", gtk::ResponseType::Close)],
    );
    let area = dialog.content_area();
    let grid = gtk::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_margin(12);
    area.pack_start(&grid, true, true, 0);

    let view = gtk::TextView::new();
    view.set_editable(false);
    view.set_size_request(360, 180);
    let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.add(&view);
    grid.attach(&scroll, 0, 0, 2, 1);

    let site = gtk::Entry::new();
    site.set_placeholder_text(Some("Seite"));
    let username = gtk::Entry::new();
    username.set_placeholder_text(Some("Benutzername"));
    let password = gtk::Entry::new();
    password.set_placeholder_text(Some("Passwort"));
    password.set_visibility(false);
    grid.attach(&site, 0, 1, 1, 1);
    grid.attach(&username, 1, 1, 1, 1);
    grid.attach(&password, 0, 2, 1, 1);

    let add = gtk::Button::with_label("Hinzufügen");
    grid.attach(&add, 1, 2, 1, 1);

    let combo = gtk::ComboBoxText::new();
    grid.attach(&combo, 0, 3, 1, 1);
    let delete = gtk::Button::with_label("Entfernen");
    grid.attach(&delete, 1, 3, 1, 1);

    fn refresh(
        view: &gtk::TextView,
        combo: &gtk::ComboBoxText,
        store: &crate::passwords::CredentialStore,
    ) {
        let mut text = String::new();
        combo.remove_all();
        for credential in store.credentials() {
            text.push_str(&format!(
                "{} — {} — {}\n",
                credential.site,
                credential.username,
                "•".repeat(credential.password.len())
            ));
            combo.append_text(&format!("{} — {}", credential.site, credential.username));
        }
        if !store.credentials().is_empty() {
            combo.set_active(Some(0));
        }
        if let Some(buffer) = view.buffer() {
            buffer.set_text(&text);
        }
    }

    refresh(&view, &combo, &store.borrow());
    let view_for_add = view.clone();
    let combo_for_add = combo.clone();
    let store_for_add = Rc::clone(&store);
    let dialog_for_add = dialog.clone();
    add.connect_clicked(move |_| {
        if site.text().is_empty() || password.text().is_empty() {
            return;
        }
        let credential = Credential {
            site: site.text().to_string(),
            username: username.text().to_string(),
            password: password.text().to_string(),
        };
        if let Err(error) = store_for_add.borrow_mut().add(credential) {
            error_dialog(&dialog_for_add, &error.to_string());
            return;
        }
        refresh(&view_for_add, &combo_for_add, &store_for_add.borrow());
    });

    let view_for_delete = view.clone();
    let combo_for_delete = combo.clone();
    let store_for_delete = Rc::clone(&store);
    let dialog_for_delete = dialog.clone();
    delete.connect_clicked(move |_| {
        let index = combo_for_delete.active().unwrap_or(0) as usize;
        if let Err(error) = store_for_delete.borrow_mut().remove(index) {
            error_dialog(&dialog_for_delete, &error.to_string());
            return;
        }
        refresh(
            &view_for_delete,
            &combo_for_delete,
            &store_for_delete.borrow(),
        );
    });

    dialog.show_all();
    dialog.connect_response(|dialog, _| dialog.close());
}
