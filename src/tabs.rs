use crate::{adblock, history::BrowserData, settings::Settings, theme};
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;
use webkit2gtk::{
    HardwareAccelerationPolicy, Settings as WebSettings, URIRequestExt, WebViewExt,
    WebViewExtManual,
};

struct Tab {
    webview: webkit2gtk::WebView,
    row: gtk::ListBoxRow,
    is_start: bool,
    private: bool,
}

pub struct TabManager {
    tabs: RefCell<Vec<Tab>>,
    active: Cell<usize>,
    container: gtk::Box,
    list: gtk::ListBox,
    progress: gtk::ProgressBar,
    spinner: gtk::Spinner,
    private_badge: gtk::Label,
    url_entry: gtk::Entry,
    settings: Rc<RefCell<Settings>>,
    data: Rc<RefCell<BrowserData>>,
    weak_self: RefCell<Weak<Self>>,
}

impl TabManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        container: gtk::Box,
        panel: gtk::Box,
        progress: gtk::ProgressBar,
        spinner: gtk::Spinner,
        private_badge: gtk::Label,
        url_entry: gtk::Entry,
        settings: Rc<RefCell<Settings>>,
        data: Rc<RefCell<BrowserData>>,
    ) -> Rc<Self> {
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.set_widget_name("lumen-1_browser-tab-list");
        panel.pack_start(&list, true, true, 0);

        let manager = Rc::new(Self {
            tabs: RefCell::new(Vec::new()),
            active: Cell::new(0),
            container,
            list,
            progress,
            spinner,
            private_badge,
            url_entry,
            settings,
            data,
            weak_self: RefCell::new(Weak::new()),
        });

        let weak: Weak<Self> = Rc::downgrade(&manager);
        manager.list.connect_row_activated(move |_, row| {
            if let Some(manager) = weak.upgrade() {
                let index = manager.tabs.borrow().iter().position(|tab| tab.row == *row);
                if let Some(index) = index {
                    manager.set_active(index);
                }
            }
        });

        *manager.weak_self.borrow_mut() = Rc::downgrade(&manager);
        manager
    }

    pub fn new_tab(&self, url: &str, private: bool) {
        let is_start = url == "lumen-1-browser://start";
        let content_manager = adblock::user_content_manager(self.settings.borrow().adblock);
        let webview = if private {
            let context = webkit2gtk::WebContext::new_ephemeral();
            webkit2gtk::WebView::new_with_context_and_user_content_manager(
                &context,
                &content_manager,
            )
        } else {
            webkit2gtk::WebView::with_user_content_manager(&content_manager)
        };
        let policy = if self.settings.borrow().hardware_acceleration {
            HardwareAccelerationPolicy::Always
        } else {
            HardwareAccelerationPolicy::Never
        };
        let web_settings = WebSettings::builder()
            .hardware_acceleration_policy(policy)
            .build();
        webview.set_settings(&web_settings);

        let weak_for_create = self.weak_self.borrow().clone();
        webview.connect_create(move |_, action| {
            let manager = weak_for_create.upgrade()?;
            let url = action
                .request()
                .and_then(|request| request.uri())
                .map(|uri| uri.to_string())
                .unwrap_or_else(|| "about:blank".to_string());
            manager.new_tab(&url, false);
            let view = {
                let tabs = manager.tabs.borrow();
                tabs.last()
                    .map(|tab| tab.webview.clone().upcast::<gtk::Widget>())
            };
            view
        });

        let data_for_history = Rc::clone(&self.data);
        let retried = Rc::new(Cell::new(false));
        let spinner_for_load = self.spinner.clone();
        let weak_for_url = self.weak_self.borrow().clone();
        let adblock_enabled = self.settings.borrow().adblock;
        webview.connect_load_changed(move |view, event| {
            if event == webkit2gtk::LoadEvent::Started {
                spinner_for_load.start();
            }
            if event == webkit2gtk::LoadEvent::Committed || event == webkit2gtk::LoadEvent::Finished
            {
                if let Some(uri) = view.uri() {
                    if let Some(manager) = weak_for_url.upgrade() {
                        let active_uri = manager
                            .tabs
                            .borrow()
                            .get(manager.active.get())
                            .and_then(|tab| tab.webview.uri());
                        if active_uri.as_deref() == Some(uri.as_ref()) {
                            if uri.starts_with("lumen-1-browser://") {
                                manager.url_entry.set_text("");
                            } else {
                                manager.url_entry.set_text(&uri);
                            }
                        }
                    }
                }
            }
            if event != webkit2gtk::LoadEvent::Finished {
                return;
            }
            spinner_for_load.stop();
            if adblock_enabled {
                let is_youtube = view
                    .uri()
                    .map(|uri| uri.contains("youtube.com"))
                    .unwrap_or(false);
                if is_youtube {
                    view.evaluate_javascript(
                        adblock::youtube_cleaner_js(),
                        None,
                        None,
                        None::<&gio::Cancellable>,
                        |_| {},
                    );
                }
            }
            if !private {
                if let Some(uri) = view.uri() {
                    let title = view
                        .title()
                        .map(|title| title.to_string())
                        .unwrap_or_else(|| uri.to_string());
                    data_for_history
                        .borrow_mut()
                        .add_history(uri.to_string(), title);
                }
            }
        });

        let label = gtk::Label::new(Some("Startseite"));
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);

        let close = gtk::Button::with_label("×");
        close.set_widget_name("lumen-1_browser-tab-close");
        close.set_tooltip_text(Some("Tab schließen"));

        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row_box.pack_start(&label, true, true, 0);
        row_box.pack_start(&close, false, false, 0);

        let row = gtk::ListBoxRow::new();
        row.set_widget_name("lumen-1_browser-tab");
        row.add(&row_box);
        row.show_all();
        self.list.add(&row);

        let weak_for_close = self.weak_self.borrow().clone();
        let row_for_close = row.clone();
        close.connect_clicked(move |_| {
            if let Some(manager) = weak_for_close.upgrade() {
                let index = manager
                    .tabs
                    .borrow()
                    .iter()
                    .position(|tab| tab.row == row_for_close);
                if let Some(index) = index {
                    manager.close_tab(index);
                }
            }
        });

        webview.connect_load_failed(move |view, _, uri, error| {
            let message = error.message();
            if !retried.get()
                && (message.contains("unterbrochen") || message.contains("interrupted"))
            {
                retried.set(true);
                let view = view.clone();
                let uri = uri.to_string();
                glib::timeout_add_local_once(Duration::from_millis(200), move || {
                    view.load_uri(&uri);
                });
            }
            false
        });

        let progress_for_load = self.progress.clone();
        webview.connect_estimated_load_progress_notify(move |view| {
            let fraction = view.estimated_load_progress();
            progress_for_load.set_fraction(if fraction >= 0.999 { 0.0 } else { fraction });
        });

        let title_label = label.clone();
        let private_title = private;
        let start_title = is_start;
        webview.connect_notify_local(Some("title"), move |view, _| {
            if start_title {
                title_label.set_label(if private_title {
                    "Privater Tab"
                } else {
                    "Startseite"
                });
                return;
            }
            if let Some(title) = view.title() {
                let text = if private_title {
                    format!("Privat · {}", title)
                } else {
                    title.to_string()
                };
                title_label.set_label(&text);
            }
        });
        if private {
            label.set_label("Privater Tab");
        }

        if is_start {
            let current = self.settings.borrow();
            let html = theme::start_page_html(
                current.wallpaper.as_deref(),
                current.transparent,
                current.language,
            );
            webview.load_html(&html, Some("lumen-1-browser://start"));
        } else {
            webview.load_uri(url);
        }
        self.tabs.borrow_mut().push(Tab {
            webview,
            row,
            is_start,
            private,
        });
        let index = self.tabs.borrow().len() - 1;
        self.set_active(index);
    }

    pub fn set_active(&self, index: usize) {
        let tabs = self.tabs.borrow();
        if let Some(tab) = tabs.get(index) {
            for child in self.container.children() {
                self.container.remove(&child);
            }
            self.container.pack_start(&tab.webview, true, true, 0);
            self.container.show_all();
            self.list.select_row(Some(&tab.row));
            if tab.private {
                self.private_badge.show();
            } else {
                self.private_badge.hide();
            }
            self.active.set(index);
        }
    }

    pub fn close_tab(&self, index: usize) {
        {
            let mut tabs = self.tabs.borrow_mut();
            if index >= tabs.len() {
                return;
            }
            let tab = tabs.remove(index);
            self.list.remove(&tab.row);
        }
        let count = self.tabs.borrow().len();
        if count == 0 {
            for child in self.container.children() {
                self.container.remove(&child);
            }
            self.new_tab("lumen-1-browser://start", false);
        } else {
            self.set_active(index.min(count - 1));
        }
    }

    pub fn close_active(&self) {
        self.close_tab(self.active.get());
    }

    pub fn load_active(&self, url: &str) {
        if let Some(tab) = self.tabs.borrow().get(self.active.get()) {
            tab.webview.load_uri(url);
        }
    }

    pub fn go_back(&self) {
        if let Some(tab) = self.tabs.borrow().get(self.active.get()) {
            tab.webview.go_back();
        }
    }

    pub fn go_forward(&self) {
        if let Some(tab) = self.tabs.borrow().get(self.active.get()) {
            tab.webview.go_forward();
        }
    }

    pub fn reload(&self) {
        if let Some(tab) = self.tabs.borrow().get(self.active.get()) {
            tab.webview.reload();
        }
    }

    pub fn bookmark_active(&self) {
        if let Some(tab) = self.tabs.borrow().get(self.active.get()) {
            if tab.private {
                return;
            }
            if let Some(uri) = tab.webview.uri() {
                let title = tab
                    .webview
                    .title()
                    .map(|title| title.to_string())
                    .unwrap_or_else(|| uri.to_string());
                self.data.borrow_mut().add_bookmark(uri.to_string(), title);
            }
        }
    }

    pub fn refresh_start_page(&self) {
        let current = self.settings.borrow();
        let tabs = self.tabs.borrow();
        if let Some(tab) = tabs.get(self.active.get()) {
            if tab.is_start {
                let html = theme::start_page_html(
                    current.wallpaper.as_deref(),
                    current.transparent,
                    current.language,
                );
                tab.webview
                    .load_html(&html, Some("lumen-1-browser://start"));
            }
        }
    }
}
