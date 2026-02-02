use crate::api::async_bridge::{spawn_api_call, ApiRequest, ApiResponse};
use crate::config::Config;
use crate::storage::{FileEntry, HistoryStorage};
use crate::views::QrDialog;
use crate::widgets::FileRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::{Cell, RefCell};
use std::io::Write;
use std::path::PathBuf;

const ITEMS_PER_PAGE: usize = 10;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FilesView {
        pub domain_combo: RefCell<Option<adw::ComboRow>>,
        pub upload_button: RefCell<Option<gtk::Button>>,
        pub files_list: RefCell<Option<gtk::ListBox>>,
        pub toast_overlay: RefCell<Option<adw::ToastOverlay>>,
        pub storage: RefCell<Option<HistoryStorage>>,
        pub drop_target: RefCell<Option<gtk::DropTarget>>,
        pub domains: RefCell<Vec<String>>,
        // Pagination
        pub current_page: Cell<usize>,
        pub page_label: RefCell<Option<gtk::Label>>,
        pub prev_button: RefCell<Option<gtk::Button>>,
        pub next_button: RefCell<Option<gtk::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilesView {
        const NAME: &'static str = "SeeFilesView";
        type Type = super::FilesView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for FilesView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for FilesView {}
    impl BoxImpl for FilesView {}
}

glib::wrapper! {
    pub struct FilesView(ObjectSubclass<imp::FilesView>)
        @extends gtk::Widget, gtk::Box;
}

impl FilesView {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("hexpand", true)
            .property("vexpand", true)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Load storage
        match HistoryStorage::load() {
            Ok(storage) => {
                imp.storage.replace(Some(storage));
            }
            Err(e) => {
                log::error!("Failed to load history: {}", e);
            }
        }

        // Toast overlay wraps everything
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_hexpand(true);
        toast_overlay.set_vexpand(true);

        // Scrolled window for content
        let scrolled = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let clamp = adw::Clamp::builder()
            .maximum_size(700)
            .margin_start(16)
            .margin_end(16)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(24)
            .build();

        // Upload area
        let upload_group = adw::PreferencesGroup::builder()
            .title("Upload File")
            .description("Share files with a shareable link")
            .build();
        upload_group.add_css_class("create-card");

        let domain_combo = adw::ComboRow::builder()
            .title("Domain")
            .subtitle("Loading domains...")
            .build();

        upload_group.add(&domain_combo);

        // Drop target area with improved styling
        let drop_area = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(16)
            .halign(gtk::Align::Fill)
            .build();
        drop_area.add_css_class("drop-zone");

        let drop_icon = gtk::Image::builder()
            .icon_name("see-file-symbolic")
            .pixel_size(48)
            .build();
        drop_icon.add_css_class("drop-zone-icon");

        let drop_label = gtk::Label::builder()
            .label("Drop files here, paste (Ctrl+V), or click to browse")
            .build();
        drop_label.add_css_class("drop-zone-text");

        let drop_hint = gtk::Label::builder()
            .label("Maximum file size: 100 MB")
            .build();
        drop_hint.add_css_class("drop-zone-hint");

        let upload_button = gtk::Button::builder()
            .label("Choose File")
            .css_classes(["see-primary"])
            .halign(gtk::Align::Center)
            .margin_top(12)
            .build();

        drop_area.append(&drop_icon);
        drop_area.append(&drop_label);
        drop_area.append(&drop_hint);
        drop_area.append(&upload_button);

        upload_group.add(&drop_area);

        // Recent Files group with header
        let recent_group = adw::PreferencesGroup::builder()
            .title("Recent Files")
            .build();
        recent_group.add_css_class("history-section");

        // Clear history button in header
        let clear_button = gtk::Button::builder()
            .label("Clear")
            .css_classes(["flat", "clear-history"])
            .build();
        clear_button.set_tooltip_text(Some("Clear local history"));
        recent_group.set_header_suffix(Some(&clear_button));

        let files_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        // Empty state placeholder
        let empty_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .spacing(8)
            .margin_top(32)
            .margin_bottom(32)
            .build();
        empty_box.add_css_class("empty-state");

        let empty_icon = gtk::Image::builder()
            .icon_name("see-file-symbolic")
            .pixel_size(64)
            .build();
        empty_icon.add_css_class("empty-state-icon");

        let empty_title = gtk::Label::builder()
            .label("No Files Yet")
            .build();
        empty_title.add_css_class("empty-state-title");

        let empty_desc = gtk::Label::builder()
            .label("Uploaded files will appear here")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build();
        empty_desc.add_css_class("empty-state-description");

        empty_box.append(&empty_icon);
        empty_box.append(&empty_title);
        empty_box.append(&empty_desc);
        files_list.set_placeholder(Some(&empty_box));

        recent_group.add(&files_list);

        // Pagination controls
        let pagination_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Center)
            .spacing(16)
            .margin_top(16)
            .build();
        pagination_box.add_css_class("pagination-box");

        let prev_button = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .css_classes(["flat", "circular"])
            .sensitive(false)
            .build();
        prev_button.set_tooltip_text(Some("Previous page"));

        let page_label = gtk::Label::builder()
            .label("Page 1 / 1")
            .build();
        page_label.add_css_class("pagination-label");

        let next_button = gtk::Button::builder()
            .icon_name("go-next-symbolic")
            .css_classes(["flat", "circular"])
            .sensitive(false)
            .build();
        next_button.set_tooltip_text(Some("Next page"));

        pagination_box.append(&prev_button);
        pagination_box.append(&page_label);
        pagination_box.append(&next_button);

        content_box.append(&upload_group);
        content_box.append(&recent_group);
        content_box.append(&pagination_box);

        clamp.set_child(Some(&content_box));
        scrolled.set_child(Some(&clamp));
        toast_overlay.set_child(Some(&scrolled));

        self.append(&toast_overlay);

        // Set up drag and drop
        let drop_target = gtk::DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY);
        drop_target.connect_drop(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[upgrade_or]
            false,
            move |_, value, _, _| {
                if let Ok(file) = value.get::<gio::File>() {
                    if let Some(path) = file.path() {
                        view.upload_file(path);
                        return true;
                    }
                }
                false
            }
        ));

        drop_area.add_controller(drop_target.clone());

        // Set up keyboard shortcut for paste (Ctrl+V)
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, keyval, _, modifier| {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK)
                    && keyval == gdk::Key::v
                {
                    view.handle_paste();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        ));
        self.add_controller(key_controller);

        // Store references
        imp.domain_combo.replace(Some(domain_combo.clone()));
        imp.upload_button.replace(Some(upload_button.clone()));
        imp.files_list.replace(Some(files_list.clone()));
        imp.toast_overlay.replace(Some(toast_overlay.clone()));
        imp.drop_target.replace(Some(drop_target));
        imp.page_label.replace(Some(page_label.clone()));
        imp.prev_button.replace(Some(prev_button.clone()));
        imp.next_button.replace(Some(next_button.clone()));

        // Connect upload button
        upload_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.show_file_chooser();
            }
        ));

        // Connect clear history button
        clear_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.show_clear_history_dialog();
            }
        ));

        // Connect pagination buttons
        prev_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                let current = view.imp().current_page.get();
                if current > 0 {
                    view.imp().current_page.set(current - 1);
                    view.refresh_list();
                }
            }
        ));

        next_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                let current = view.imp().current_page.get();
                view.imp().current_page.set(current + 1);
                view.refresh_list();
            }
        ));

        // Load existing files
        self.refresh_list();

        // Fetch domains from API
        self.fetch_domains();
    }

    fn show_clear_history_dialog(&self) {
        let dialog = adw::AlertDialog::builder()
            .heading("Clear Local History?")
            .body("This will only clear the local history on this device.\n\nTo delete your data from S.EE servers, please visit s.ee/user/dashboard")
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("clear", "Clear");
        dialog.set_response_appearance("clear", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.connect_response(
            None,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move |_, response| {
                    if response == "clear" {
                        view.clear_local_history();
                    }
                }
            ),
        );

        if let Some(window) = self.root().and_downcast_ref::<gtk::Window>() {
            dialog.present(Some(window));
        }
    }

    fn clear_local_history(&self) {
        let imp = self.imp();
        if let Some(ref mut storage) = *imp.storage.borrow_mut() {
            storage.clear_files();
            if let Err(e) = storage.save() {
                log::error!("Failed to save after clearing history: {}", e);
            }
        }
        imp.current_page.set(0);
        self.refresh_list();
        self.show_toast("Local history cleared");
    }

    fn fetch_domains(&self) {
        let config = Config::load().unwrap_or_default();
        let receiver = spawn_api_call(config, ApiRequest::GetFileDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetFileDomains(Ok(domains)) => {
                            view.update_domains(domains);
                        }
                        ApiResponse::GetFileDomains(Err(e)) => {
                            log::error!("Failed to fetch file domains: {}", e);
                            view.update_domains(vec!["i.s.ee".to_string()]);
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn update_domains(&self, domains: Vec<String>) {
        let imp = self.imp();
        imp.domains.replace(domains.clone());

        if let Some(combo) = imp.domain_combo.borrow().as_ref() {
            let string_list =
                gtk::StringList::new(&domains.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            combo.set_model(Some(&string_list));
            combo.set_subtitle("");

            // Select default domain from config
            let config = Config::load().unwrap_or_default();
            if let Some(default_domain) = config.default_file_domain() {
                for (i, domain) in domains.iter().enumerate() {
                    if domain == default_domain {
                        combo.set_selected(i as u32);
                        break;
                    }
                }
            }
        }
    }

    fn show_file_chooser(&self) {
        let dialog = gtk::FileDialog::builder()
            .title("Select File to Upload")
            .build();

        dialog.open(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            view.upload_file(path);
                        }
                    }
                }
            ),
        );
    }

    fn upload_file(&self, path: PathBuf) {
        let config = Config::load().unwrap_or_default();

        let request = ApiRequest::UploadFile { path: path.clone() };

        let receiver = spawn_api_call(config, request);

        self.show_toast("Uploading file...");

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::UploadFile(Ok(result)) => {
                            // Extract domain from url
                            let file_url = &result.data.url;
                            let domain = file_url
                                .split('/')
                                .nth(2)
                                .unwrap_or("i.s.ee")
                                .to_string();

                            // Get page URL from API response
                            let page_url = result.data.page.clone();

                            let entry = FileEntry::new(
                                file_url.clone(),
                                page_url,
                                domain,
                                result.data.hash.clone(),
                                result.data.filename.clone(),
                                result.data.size,
                                None,
                            );

                            // Save to storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.add_file(entry) {
                                    log::error!("Failed to save file: {}", e);
                                }
                            }

                            // Go to first page to see the new file
                            view.imp().current_page.set(0);
                            view.refresh_list();
                            view.show_toast("File uploaded successfully!");

                            // Copy to clipboard
                            if let Some(display) = gdk::Display::default() {
                                display.clipboard().set_text(file_url);
                            }
                        }
                        ApiResponse::UploadFile(Err(e)) => {
                            view.show_toast(&format!("Error: {}", e));
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn refresh_list(&self) {
        let imp = self.imp();

        let files_list = match imp.files_list.borrow().as_ref() {
            Some(list) => list.clone(),
            None => return,
        };

        // Clear existing rows
        while let Some(child) = files_list.first_child() {
            files_list.remove(&child);
        }

        // Get files from storage with pagination
        if let Some(ref storage) = *imp.storage.borrow() {
            let all_files = storage.files();
            let total_items = all_files.len();
            let total_pages = (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE.max(1);
            let current_page = imp.current_page.get();

            // Ensure current page is valid
            let current_page = if total_pages == 0 {
                0
            } else {
                current_page.min(total_pages - 1)
            };
            imp.current_page.set(current_page);

            // Calculate slice
            let start = current_page * ITEMS_PER_PAGE;
            let end = (start + ITEMS_PER_PAGE).min(total_items);

            // Add rows for current page
            for entry in all_files.iter().skip(start).take(end - start) {
                let row = FileRow::new(entry);
                self.setup_row_actions(&row);
                files_list.append(&row);
            }

            // Update pagination controls
            self.update_pagination(current_page, total_pages);
        }
    }

    fn update_pagination(&self, current_page: usize, total_pages: usize) {
        let imp = self.imp();

        if let Some(label) = imp.page_label.borrow().as_ref() {
            if total_pages == 0 {
                label.set_label("No items");
            } else {
                label.set_label(&format!("Page {} / {}", current_page + 1, total_pages));
            }
        }

        if let Some(prev) = imp.prev_button.borrow().as_ref() {
            prev.set_sensitive(current_page > 0);
        }

        if let Some(next) = imp.next_button.borrow().as_ref() {
            next.set_sensitive(total_pages > 0 && current_page < total_pages - 1);
        }
    }

    fn setup_row_actions(&self, row: &FileRow) {
        let entry = row.entry();

        // Click on row to open share page in browser
        let entry_for_activate = entry.clone();
        row.connect_activated(move |_| {
            if let Some(ref entry) = entry_for_activate {
                let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                let _ = gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>);
            }
        });

        // Copy Page URL button
        row.copy_page_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    if let Some(display) = gdk::Display::default() {
                        let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                        display.clipboard().set_text(url);
                        view.show_toast("Share page URL copied");
                    }
                }
            }
        ));

        // Copy Direct URL button
        row.copy_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().set_text(&entry.url);
                        view.show_toast("Direct URL copied");
                    }
                }
            }
        ));

        // QR button - use page URL
        row.qr_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                    let dialog = QrDialog::new(url);
                    if let Some(window) = view.root().and_downcast_ref::<gtk::Window>() {
                        dialog.present(Some(window));
                    }
                }
            }
        ));

        // Delete button
        row.delete_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    view.delete_file(&entry.domain, &entry.slug);
                }
            }
        ));
    }

    fn delete_file(&self, domain: &str, key: &str) {
        let config = Config::load().unwrap_or_default();
        let domain = domain.to_string();
        let key = key.to_string();

        let request = ApiRequest::DeleteFile { key: key.clone() };

        let receiver = spawn_api_call(config, request);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            domain,
            #[strong]
            key,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::DeleteFile(Ok(())) => {
                            // Remove from storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.remove_file(&domain, &key) {
                                    log::error!("Failed to remove file from storage: {}", e);
                                }
                            }

                            view.refresh_list();
                            view.show_toast("File deleted");
                        }
                        ApiResponse::DeleteFile(Err(e)) => {
                            view.show_toast(&format!("Error: {}", e));
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn show_toast(&self, message: &str) {
        if let Some(ref overlay) = *self.imp().toast_overlay.borrow() {
            let toast = adw::Toast::new(message);
            overlay.add_toast(toast);
        }
    }

    fn handle_paste(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();

            // Try to read files first
            clipboard.read_value_async(
                gio::File::static_type(),
                glib::Priority::DEFAULT,
                None::<&gio::Cancellable>,
                glib::clone!(
                    #[weak(rename_to = view)]
                    self,
                    move |result| {
                        if let Ok(value) = result {
                            if let Ok(file) = value.get::<gio::File>() {
                                if let Some(path) = file.path() {
                                    view.upload_file(path);
                                    return;
                                }
                            }
                        }
                        // If no file, try to read texture (image from clipboard)
                        view.try_paste_image();
                    }
                ),
            );
        }
    }

    fn try_paste_image(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();

            clipboard.read_texture_async(
                None::<&gio::Cancellable>,
                glib::clone!(
                    #[weak(rename_to = view)]
                    self,
                    move |result| {
                        if let Ok(Some(texture)) = result {
                            // Save texture to temp file and upload
                            view.upload_texture(texture);
                        } else {
                            view.show_toast("No file or image in clipboard");
                        }
                    }
                ),
            );
        }
    }

    fn upload_texture(&self, texture: gdk::Texture) {
        // Save texture to temporary file
        let bytes = texture.save_to_png_bytes();

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let temp_path = temp_dir.join(format!("clipboard_{}.png", timestamp));

        match std::fs::File::create(&temp_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&bytes) {
                    self.show_toast(&format!("Failed to save image: {}", e));
                    return;
                }
                self.upload_file(temp_path);
            }
            Err(e) => {
                self.show_toast(&format!("Failed to create temp file: {}", e));
            }
        }
    }
}

impl Default for FilesView {
    fn default() -> Self {
        Self::new()
    }
}
