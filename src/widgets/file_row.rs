use crate::storage::FileEntry;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(string = r#"
        <interface>
          <template class="SeeFileRow" parent="AdwActionRow">
            <property name="activatable">True</property>
            <style>
              <class name="see-row"/>
            </style>
            <child type="suffix">
              <object class="GtkBox">
                <property name="spacing">4</property>
                <property name="margin-start">8</property>
                <child>
                  <object class="GtkButton" id="qr_button">
                    <property name="icon-name">see-qr-code-symbolic</property>
                    <property name="valign">center</property>
                    <property name="tooltip-text" translatable="yes">Show QR Code (Share Page)</property>
                    <style>
                      <class name="flat"/>
                      <class name="qr-button"/>
                    </style>
                  </object>
                </child>
                <child>
                  <object class="GtkButton" id="copy_page_button">
                    <property name="icon-name">see-copy-page-symbolic</property>
                    <property name="valign">center</property>
                    <property name="tooltip-text" translatable="yes">Copy Share Page URL</property>
                    <style>
                      <class name="flat"/>
                      <class name="copy-button"/>
                    </style>
                  </object>
                </child>
                <child>
                  <object class="GtkButton" id="copy_button">
                    <property name="icon-name">see-copy-link-symbolic</property>
                    <property name="valign">center</property>
                    <property name="tooltip-text" translatable="yes">Copy Direct URL</property>
                    <style>
                      <class name="flat"/>
                      <class name="copy-button"/>
                    </style>
                  </object>
                </child>
                <child>
                  <object class="GtkButton" id="delete_button">
                    <property name="icon-name">see-delete-symbolic</property>
                    <property name="valign">center</property>
                    <property name="tooltip-text" translatable="yes">Delete</property>
                    <style>
                      <class name="flat"/>
                      <class name="destructive-action"/>
                    </style>
                  </object>
                </child>
              </object>
            </child>
          </template>
        </interface>
    "#)]
    pub struct FileRow {
        #[template_child]
        pub qr_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub copy_page_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub copy_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        pub entry: RefCell<Option<FileEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileRow {
        const NAME: &'static str = "SeeFileRow";
        type Type = super::FileRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileRow {}
    impl WidgetImpl for FileRow {}
    impl ListBoxRowImpl for FileRow {}
    impl PreferencesRowImpl for FileRow {}
    impl ActionRowImpl for FileRow {}
}

glib::wrapper! {
    pub struct FileRow(ObjectSubclass<imp::FileRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl FileRow {
    pub fn new(entry: &FileEntry) -> Self {
        let row: Self = glib::Object::new();
        row.set_title(&entry.filename);

        // Show page URL and direct URL with size on separate lines
        let subtitle = if let Some(ref page_url) = entry.page_url {
            format!(
                "Share: {}\nDirect: {}\nSize: {}",
                page_url,
                entry.url,
                Self::format_size(entry.size)
            )
        } else {
            format!("Direct: {}\nSize: {}", entry.url, Self::format_size(entry.size))
        };
        row.set_subtitle(&subtitle);
        row.set_subtitle_lines(3);

        row.imp().entry.replace(Some(entry.clone()));
        row
    }

    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.1} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }

    pub fn entry(&self) -> Option<FileEntry> {
        self.imp().entry.borrow().clone()
    }

    pub fn qr_button(&self) -> &gtk::Button {
        &self.imp().qr_button
    }

    pub fn copy_page_button(&self) -> &gtk::Button {
        &self.imp().copy_page_button
    }

    pub fn copy_button(&self) -> &gtk::Button {
        &self.imp().copy_button
    }

    pub fn delete_button(&self) -> &gtk::Button {
        &self.imp().delete_button
    }
}
