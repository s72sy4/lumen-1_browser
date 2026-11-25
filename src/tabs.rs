use crate::{adblock, settings::Settings, theme};
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use webkit2gtk::{HardwareAccelerationPolicy, Settings as WebSettings, WebViewExt};

struct Tab {
    webview: webkit2gtk::WebView,
    row: gtk::ListBoxRow,
    is_start: bool,
}

pub struct TabManager {
    tabs: RefCell<Vec<Tab>>,
    active: Cell<usize>,
    container: gtk::Box,
    list: gtk::ListBox,
    progress: gtk::ProgressBar,
    settings: Rc<RefCell<Settings>>,
}

impl TabManager {
    pub fn new(
        container: gtk::Box,
        panel: gtk::Box,
        progress: gtk::ProgressBar,
        settings: Rc<RefCell<Settings>>,
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
            settings,
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

        manager
    }

    pub fn new_tab(&self, url: &str) {
        let content_manager = adblock::user_content_manager(self.settings.borrow().adblock);
        let webview = webkit2gtk::WebView::with_user_content_manager(&content_manager);
        let web_settings = WebSettings::builder()
            .hardware_acceleration_policy(HardwareAccelerationPolicy::Never)
            .build();
        webview.set_settings(&web_settings);

        if self.settings.borrow().adblock {
            webview.connect_load_changed(|view, event| {
                if event == webkit2gtk::LoadEvent::Finished {
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
            });
        }

        let label = gtk::Label::new(Some("Startseite"));
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);

        let row = gtk::ListBoxRow::new();
        row.set_widget_name("lumen-1_browser-tab");
        row.add(&label);
        row.show_all();
        self.list.add(&row);

        let progress_for_load = self.progress.clone();
        webview.connect_estimated_load_progress_notify(move |view| {
            let fraction = view.estimated_load_progress();
            progress_for_load.set_fraction(if fraction >= 0.999 { 0.0 } else { fraction });
        });

        let title_label = label.clone();
        webview.connect_notify_local(Some("title"), move |view, _| {
            if let Some(title) = view.title() {
                title_label.set_label(&title);
            }
        });

        let is_start = url == "lumen-1-browser://start";
        if is_start {
            let current = self.settings.borrow();
            let html = theme::start_page_html(
                current.wallpaper.as_deref(),
                current.transparent,
                current.search_engine,
            );
            webview.load_html(&html, Some("lumen-1-browser://start"));
        } else {
            webview.load_uri(url);
        }
        self.tabs.borrow_mut().push(Tab {
            webview,
            row,
            is_start,
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
            self.active.set(index);
        }
    }

    pub fn close_active(&self) {
        let index = self.active.get();
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
            self.new_tab("lumen-1-browser://start");
        } else {
            self.set_active(index.min(count - 1));
        }
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

    pub fn refresh_start_page(&self) {
        let current = self.settings.borrow();
        let tabs = self.tabs.borrow();
        if let Some(tab) = tabs.get(self.active.get()) {
            if tab.is_start {
                let html = theme::start_page_html(
                    current.wallpaper.as_deref(),
                    current.transparent,
                    current.search_engine,
                );
                tab.webview
                    .load_html(&html, Some("lumen-1-browser://start"));
            }
        }
    }
}
