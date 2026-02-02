use crate::storage::LinkEntry;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(string = r#"
        <interface>
          <template class="SeeLinkRow" parent="AdwActionRow">
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
                    <property name="tooltip-text" translatable="yes">Show QR Code</property>
                    <style>
                      <class name="flat"/>
                      <class name="qr-button"/>
                    </style>
                  </object>
                </child>
                <child>
                  <object class="GtkButton" id="copy_button">
                    <property name="icon-name">see-copy-link-symbolic</property>
                    <property name="valign">center</property>
                    <property name="tooltip-text" translatable="yes">Copy to Clipboard</property>
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
    pub struct LinkRow {
        #[template_child]
        pub qr_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub copy_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        pub entry: RefCell<Option<LinkEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LinkRow {
        const NAME: &'static str = "SeeLinkRow";
        type Type = super::LinkRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LinkRow {}
    impl WidgetImpl for LinkRow {}
    impl ListBoxRowImpl for LinkRow {}
    impl PreferencesRowImpl for LinkRow {}
    impl ActionRowImpl for LinkRow {}
}

glib::wrapper! {
    pub struct LinkRow(ObjectSubclass<imp::LinkRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl LinkRow {
    pub fn new(entry: &LinkEntry) -> Self {
        let row: Self = glib::Object::new();
        row.set_title(&entry.short_url);
        row.set_subtitle(&entry.original_url);
        row.imp().entry.replace(Some(entry.clone()));
        row
    }

    pub fn entry(&self) -> Option<LinkEntry> {
        self.imp().entry.borrow().clone()
    }

    pub fn qr_button(&self) -> &gtk::Button {
        &self.imp().qr_button
    }

    pub fn copy_button(&self) -> &gtk::Button {
        &self.imp().copy_button
    }

    pub fn delete_button(&self) -> &gtk::Button {
        &self.imp().delete_button
    }
}
