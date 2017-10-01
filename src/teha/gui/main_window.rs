//
// main_window.rs
//
// Copyright (C) 2017 Muhannad Alrusayni <0x3UH4224D@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use std::rc::Rc;
use std::cell::RefCell;

use gtk::{self, WidgetExt, GtkApplicationExt, FlowBoxExt, StackExt, EntryExt};

use super::*;
use super::app::Application;
use super::header_bar::HeaderBar;
use super::drawing_area::DrawingArea;
use super::toolbar::Toolbar;
use core::Document;

// note: acronyms names used here refer to:
// stcrim: start_create_import
// edpr: editor_project
// stup: start_up
// crwo: create_work
pub struct MainWindow {
    parent: gtk::ApplicationWindow,
    header_bar: Rc<RefCell<HeaderBar>>,
    drawing_area: Rc<RefCell<DrawingArea>>,
    toolbar: Rc<RefCell<Toolbar>>,
    documents: Vec<Document>,
    active_document_index: usize,
    root_stack: gtk::Stack,
    stcrim_stack: gtk::Stack,
    edpr_stack: gtk::Stack,
    ed_options: gtk::Notebook,
    stup_flowbox: gtk::FlowBox,
    crwo_infobar: gtk::InfoBar,
    crwo_infobar_title: gtk::Label,
    crwo_infobar_description: gtk::Label,
    crwo_file_name: gtk::Entry,
    crwo_file_location: gtk::FileChooserButton,
    crwo_width: gtk::SpinButton,
    crwo_height: gtk::SpinButton,
    crwo_transparent_background: gtk::Switch,
}

impl MainWindow {
    pub fn new(builder: gtk::Builder, app: gtk::Application) -> Self {
        let main_window: gtk::ApplicationWindow =
            builder.get_object("main_window").unwrap();
        let root_stack: gtk::Stack =
            builder.get_object("root_stack").unwrap();
        let stcrim_stack: gtk::Stack =
            builder.get_object("stcrim_stack").unwrap();
        let edpr_stack: gtk::Stack =
            builder.get_object("edpr_stack").unwrap();
        let ed_options: gtk::Notebook =
            builder.get_object("ed_options").unwrap();
        let stup_flowbox: gtk::FlowBox =
            builder.get_object("stup_flowbox").unwrap();
        let crwo_infobar: gtk::InfoBar =
            builder.get_object("crwo_infobar").unwrap();
        let crwo_infobar_title: gtk::Label =
            builder.get_object("crwo_infobar_title").unwrap();
        let crwo_infobar_description: gtk::Label =
            builder.get_object("crwo_infobar_description").unwrap();
        let crwo_file_name: gtk::Entry =
            builder.get_object("crwo_file_name").unwrap();
        let crwo_file_location: gtk::FileChooserButton =
            builder.get_object("crwo_file_location").unwrap();
        let crwo_width: gtk::SpinButton =
            builder.get_object("crwo_width").unwrap();
        let crwo_height: gtk::SpinButton =
            builder.get_object("crwo_height").unwrap();
        let crwo_transparent_background: gtk::Switch =
            builder.get_object("crwo_transparent_background").unwrap();

        crwo_file_name.set_max_length(63);
        app.add_window(&main_window);

        let header_bar = HeaderBar::new(builder.clone());
        let drawing_area = DrawingArea::new(builder.clone());
        let toolbar = Toolbar::new(builder.clone());

        MainWindow {
            parent: main_window,
            header_bar: Rc::new(RefCell::new(header_bar)),
            drawing_area: Rc::new(RefCell::new(drawing_area)),
            toolbar: Rc::new(RefCell::new(toolbar)),
            documents: vec![],
            active_document_index: 0,
            root_stack: root_stack,
            stcrim_stack: stcrim_stack,
            edpr_stack: edpr_stack,
            ed_options: ed_options,
            stup_flowbox: stup_flowbox,
            crwo_infobar: crwo_infobar,
            crwo_infobar_title: crwo_infobar_title,
            crwo_infobar_description: crwo_infobar_description,
            crwo_file_name: crwo_file_name,
            crwo_file_location: crwo_file_location,
            crwo_width: crwo_width,
            crwo_height: crwo_height,
            crwo_transparent_background: crwo_transparent_background,
        }
    }

    pub fn get_parent(&self) -> gtk::ApplicationWindow {
        self.parent.clone()
    }

    pub fn get_header_bar(&self) -> Rc<RefCell<HeaderBar>> {
        self.header_bar.clone()
    }

    pub fn get_drawing_area(&self) -> Rc<RefCell<DrawingArea>> {
        self.drawing_area.clone()
    }

    pub fn get_toolbar(&self) -> Rc<RefCell<Toolbar>> {
        self.toolbar.clone()
    }

    pub fn get_documents(&self) -> &Vec<Document> {
        &self.documents
    }

    pub fn get_mut_documents(&mut self) -> &mut Vec<Document> {
        &mut self.documents
    }

    pub fn get_active_document(&self) -> &Document {
        &self.documents[self.active_document_index]
    }

    pub fn get_mut_active_document(&mut self) -> &mut Document {
        &mut self.documents[self.active_document_index]
    }

    pub fn get_active_document_index(&self) -> usize {
        self.active_document_index
    }

    pub fn set_active_document_index(&mut self, index: usize) {
        self.active_document_index = index;
    }

    pub fn get_ed_options(&self) -> gtk::Notebook {
        self.ed_options.clone()
    }

    pub fn get_crwo_infobar(&self) -> gtk::InfoBar {
        self.crwo_infobar.clone()
    }

    pub fn get_crwo_infobar_title(&self) -> gtk::Label {
        self.crwo_infobar_title.clone()
    }

    pub fn get_crwo_infobar_description(&self) -> gtk::Label {
        self.crwo_infobar_description.clone()
    }

    pub fn get_crwo_file_name(&self) -> gtk::Entry {
        self.crwo_file_name.clone()
    }

    pub fn get_crwo_file_location(&self) -> gtk::FileChooserButton {
        self.crwo_file_location.clone()
    }

    pub fn get_crwo_width(&self) -> gtk::SpinButton {
        self.crwo_width.clone()
    }

    pub fn get_crwo_height(&self) -> gtk::SpinButton {
        self.crwo_height.clone()
    }

    pub fn get_crwo_transparent_background(&self) -> gtk::Switch {
        self.crwo_transparent_background.clone()
    }

    pub fn show(&self) {
        self.parent.show();
    }

    pub fn update_view(&mut self, view_mode: ViewMode) {
        let view_name: &str = view_mode.into();
        match view_mode {
            ViewMode::StartUp => {
                self.root_stack.set_visible_child_name(&view_name);
                self.stcrim_stack.set_visible_child_name(&view_name);
            },
            ViewMode::CreatingWork | ViewMode::ImportingWork => {
                let root_view_name: &str = ViewMode::StartUp.into();
                self.root_stack.set_visible_child_name(&root_view_name);
                self.stcrim_stack.set_visible_child_name(&view_name);
            },
            ViewMode::Editing | ViewMode::Previewing => {
                let root_view_name: &str = ViewMode::Editing.into();
                self.root_stack.set_visible_child_name(&root_view_name);
                self.edpr_stack.set_visible_child_name(&view_name);
            }
            ViewMode::Preferences => {
                // TODO: not finished yet
            }
        }
    }

    pub fn new_documents(&mut self, document: Document) {
        self.documents.push(document);
    }

    pub fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.get_main_window();
        let window = window.borrow();

        {
            let app = app.clone();
            let stup_flowbox = window.stup_flowbox.clone();
            stup_flowbox.connect_child_activated(move |_me, child| {
                let button_name = child.get_name().unwrap();
                if button_name == "startup_new_work_button" {
                    app.borrow_mut().update_view(ViewMode::CreatingWork);
                } else if button_name == "startup_import_work_button" {
                    app.borrow_mut().update_view(ViewMode::ImportingWork);
                }
            });
        }

        HeaderBar::connect_ui(app.clone());
        DrawingArea::connect_ui(app.clone());
        Toolbar::connect_ui(app.clone());
    }
}


