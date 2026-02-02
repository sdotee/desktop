use crate::storage::TextEntry;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(string = r#"
        <interface>
          <template class="SeeTextRow" parent="AdwActionRow">
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
                    <property name="tooltip-text" translatable="yes">Copy Raw URL</property>
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
    pub struct TextRow {
        #[template_child]
        pub qr_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub copy_page_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub copy_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        pub entry: RefCell<Option<TextEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextRow {
        const NAME: &'static str = "SeeTextRow";
        type Type = super::TextRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TextRow {}
    impl WidgetImpl for TextRow {}
    impl ListBoxRowImpl for TextRow {}
    impl PreferencesRowImpl for TextRow {}
    impl ActionRowImpl for TextRow {}
}

glib::wrapper! {
    pub struct TextRow(ObjectSubclass<imp::TextRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl TextRow {
    pub fn new(entry: &TextEntry) -> Self {
        let row: Self = glib::Object::new();
        let title = entry.title.as_deref().unwrap_or("Untitled");
        row.set_title(title);

        // Show page URL and raw URL on separate lines
        let subtitle = if let Some(ref page_url) = entry.page_url {
            format!("Share: {}\nRaw: {}/raw", page_url, entry.url)
        } else {
            format!("Raw: {}/raw", entry.url)
        };
        row.set_subtitle(&subtitle);
        row.set_subtitle_lines(2);

        row.imp().entry.replace(Some(entry.clone()));
        row
    }

    pub fn entry(&self) -> Option<TextEntry> {
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
