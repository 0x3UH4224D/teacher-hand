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
use std::fs::File;
use std::error::Error;

use gtk::{self, WidgetExt, ApplicationExt, FlowBoxExt, StackExt, ButtonExt,
          EntryExt, LabelExt, FileChooserExt, SpinButtonExt, SwitchExt,
          ScaleExt};
use gtk::prelude::*;

use gettextrs::*;

use super::*;
use super::app::Application;
use common;
use common::types::*;
use core::{Document};

// note: acronyms names used here refer to:
// stcrim: start_create_import
// edpr: editor_project
// stup: start_up
// crwo: create_work
pub struct MainWindow {
    pub parent: gtk::ApplicationWindow,
    pub header_bar: Option<Rc<RefCell<HeaderBar>>>,
    pub drawing_area: Option<Rc<RefCell<DrawingArea>>>,
    pub documents: Vec<Document>,
    current_document: usize,
    root_stack: gtk::Stack,
    stcrim_stack: gtk::Stack,
    edpr_stack: gtk::Stack,
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
    pub fn new(app: &Application) -> Self {
        let main_window: gtk::ApplicationWindow =
            app.builder.get_object("main_window").unwrap();
        let root_stack: gtk::Stack =
            app.builder.get_object("root_stack").unwrap();
        let stcrim_stack: gtk::Stack =
            app.builder.get_object("stcrim_stack").unwrap();
        let edpr_stack: gtk::Stack =
            app.builder.get_object("edpr_stack").unwrap();
        let stup_flowbox: gtk::FlowBox =
            app.builder.get_object("stup_flowbox").unwrap();
        let crwo_infobar: gtk::InfoBar =
            app.builder.get_object("crwo_infobar").unwrap();
        let crwo_infobar_title: gtk::Label =
            app.builder.get_object("crwo_infobar_title").unwrap();
        let crwo_infobar_description: gtk::Label =
            app.builder.get_object("crwo_infobar_description").unwrap();
        let crwo_file_name: gtk::Entry =
            app.builder.get_object("crwo_file_name").unwrap();
        let crwo_file_location: gtk::FileChooserButton =
            app.builder.get_object("crwo_file_location").unwrap();
        let crwo_width: gtk::SpinButton =
            app.builder.get_object("crwo_width").unwrap();
        let crwo_height: gtk::SpinButton =
            app.builder.get_object("crwo_height").unwrap();
        let crwo_transparent_background: gtk::Switch =
            app.builder.get_object("crwo_transparent_background").unwrap();

        crwo_file_name.set_max_length(63);
        app.parent.add_window(&main_window);

        let mut teha_window = MainWindow {
            parent: main_window,
            header_bar: None,
            drawing_area: None,
            documents: vec![],
            current_document: 0,
            root_stack: root_stack,
            stcrim_stack: stcrim_stack,
            edpr_stack: edpr_stack,
            stup_flowbox: stup_flowbox,
            crwo_infobar: crwo_infobar,
            crwo_infobar_title: crwo_infobar_title,
            crwo_infobar_description: crwo_infobar_description,
            crwo_file_name: crwo_file_name,
            crwo_file_location: crwo_file_location,
            crwo_width: crwo_width,
            crwo_height: crwo_height,
            crwo_transparent_background: crwo_transparent_background,
        };

        let header_bar = HeaderBar::new(app);
        teha_window.header_bar = Some(Rc::new(RefCell::new(header_bar)));

        let drawing_area = DrawingArea::new(app);
        teha_window.drawing_area = Some(Rc::new(RefCell::new(drawing_area)));

        teha_window
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
        let window = teha_app.main_window.as_ref().unwrap().borrow();
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
    }
}

pub struct HeaderBar {
    pub parent: gtk::HeaderBar,
    left_stack: gtk::Stack,
    mid_stack: gtk::Stack,
    right_stack: gtk::Stack,
    crwo_back: gtk::Button,         // create work back
    crwo_forward: gtk::Button,      // create work forward
    ed_zoom_level: gtk::Scale,      // editor zoom level
}

impl HeaderBar {
    fn new(app: &Application) -> Self {
        let header_bar: gtk::HeaderBar =
            app.builder.get_object("main_window_headerbar").unwrap();
        let left_stack: gtk::Stack =
            app.builder.get_object("left_stack").unwrap();
        let mid_stack: gtk::Stack =
            app.builder.get_object("mid_stack").unwrap();
        let right_stack: gtk::Stack =
            app.builder.get_object("right_stack").unwrap();
        let crwo_back: gtk::Button =
            app.builder.get_object("crwo_back").unwrap();
        let crwo_forward: gtk::Button =
            app.builder.get_object("crwo_forward").unwrap();
        let ed_zoom_level: gtk::Scale =
            app.builder.get_object("ed_zoom_level").unwrap();

        let teha_headerbar = HeaderBar {
            parent: header_bar,
            left_stack: left_stack,
            mid_stack: mid_stack,
            right_stack: right_stack,
            crwo_back: crwo_back,
            crwo_forward: crwo_forward,
            ed_zoom_level: ed_zoom_level,
        };

        teha_headerbar
    }

    fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.main_window.as_ref().unwrap().clone();
        let header_bar = window.borrow().header_bar.as_ref().unwrap().clone();

        {
            // clone crwo_back to connect it to its closure
            let crwo_back = header_bar.borrow().crwo_back.clone();

            // clone the needed objects
            let app = app.clone();

            // connect "clicked" signal to a closure
            crwo_back.connect_clicked(move |_me| {
                let view_mode = app.borrow().last_view_mode;
                app.borrow_mut().update_view(view_mode);
            });
        }

        {
            // clone crwo_forward to connect it to its closure
            let crwo_forward = header_bar.borrow().crwo_forward.clone();

            // clone the needed objects
            let app = app.clone();
            let crwo_infobar = window.borrow().crwo_infobar.clone();
            let crwo_infobar_title = window.borrow().crwo_infobar_title.clone();
            let crwo_infobar_description =
                window.borrow().crwo_infobar_description.clone();
            let crwo_file_name = window.borrow().crwo_file_name.clone();
            let crwo_file_location = window.borrow().crwo_file_location.clone();
            let crwo_width = window.borrow().crwo_width.clone();
            let crwo_height = window.borrow().crwo_height.clone();
            let crwo_transparent_background =
                window.borrow().crwo_transparent_background.clone();
            let window = window.clone();

            crwo_forward.connect_clicked(move |_me| {
                // check if the file name is valid, show message through infobar
                // if not. and return
                let file_name = crwo_file_name.get_text().unwrap();
                match common::string::is_valid_filename(&file_name) {
                    Err(msg) => {
                        crwo_infobar_title.set_label(
                            gettext("<b>Invailed File Name</b>").as_str()
                        );
                        crwo_infobar_description.set_label(msg.as_str());
                        if !crwo_infobar.get_visible() {
                            crwo_infobar.show();
                        }
                        return;
                    },
                    Ok(_) => {},
                };

                // check if user select folder or not, and warn him if not.
                let mut file_path = match crwo_file_location.get_filename() {
                    Some(val) => val,
                    None => {
                        crwo_infobar_title.set_label(
                            gettext("<b>Invailed File Location</b>").as_str()
                        );
                        crwo_infobar_description.set_label(
                            gettext("File location cannot be empty").as_str()
                        );
                        if !crwo_infobar.get_visible() {
                            crwo_infobar.show();
                        }
                        return;
                    },
                };

                // collect info
                file_path.push(file_name.clone());
                file_path.set_extension("teha");
                let width = crwo_width.get_value_as_int();
                let height = crwo_height.get_value_as_int();
                let transparent_background =
                    crwo_transparent_background.get_active();


                // create a file in the given path
                let _ = match File::create(&file_path) {
                    Ok(file) => file,
                    Err(why) => {
                        // setting the title
                        let title = format!("{} {} {}",
                            /* TRANSLATORS: don't remove or translate <b> it's needed. Note the full sentence is "Couldn't Create [FILE_NAME] File" */
                            gettext("<b>Couldn't Create"),
                            file_name,
                            /* TRANSLATORS: don't remove or translate </b> it's needed. Note the full sentence is "Couldn't Create [FILE_NAME] File" */
                            gettext("File</b>"));
                        crwo_infobar_title.set_label(title.as_str());

                        // setting the description
                        let description = format!("{} {}",
                            /* TRANSLATORS: this wrod will be in sentence like this "Error [ERROR_DESCRIPTION]" */
                            gettext("Error"),
                            why.description());
                        crwo_infobar_description.set_label(
                            description.as_str()
                        );
                        if !crwo_infobar.get_visible() {
                            crwo_infobar.show();
                        }
                        return;
                    },
                };

                // start a new work with the information we got.
                window.borrow_mut().new_documents(
                    Document::new(
                        1,
                        file_path,
                        Size::new(width, height),
                        transparent_background
                    )
                );

                // cleanup the widgets for reuse
                crwo_file_name.set_text("");
                crwo_file_location.unselect_all();
                crwo_width.set_value(800_f64);
                crwo_height.set_value(600_f64);
                crwo_transparent_background.set_active(true);
                if crwo_infobar.get_visible() {
                    crwo_infobar.hide();
                }

                app.borrow_mut().update_view(ViewMode::Editing);
            });
        }

        {
            let ed_zoom_level = header_bar.borrow().ed_zoom_level.clone();
            ed_zoom_level.connect_format_value(move |_me, value| {
                format!("{}%", value * 100.0)
            });
        }

        {
            let ed_zoom_level = header_bar.borrow().ed_zoom_level.clone();
            let window = window.clone();
            ed_zoom_level.connect_value_changed(move |me| {
                let current_document;
                let page_number;
                {
                    let window = window.borrow();
                    if window.documents.is_empty() {
                        return;
                    } else {
                        current_document = window.current_document;
                    }

                    if window.documents[current_document].pages.is_empty() {
                        return;
                    } else {
                        page_number = window
                            .documents[current_document]
                            .page_number;
                    }
                }
                window
                    .borrow_mut()
                    .documents[current_document]
                    .pages[page_number]
                    .zoom_level = me.get_value();
            });
        }
    }

    pub fn update_view(&mut self, view_mode: ViewMode) {
        let view_name: &str = view_mode.into();
        self.left_stack.set_visible_child_name(&view_name);
        self.mid_stack.set_visible_child_name(&view_name);
        self.right_stack.set_visible_child_name(&view_name);
    }
}

pub struct DrawingArea {
    pub parent: gtk::DrawingArea,
    _current_tool: CurrentTool,
    _scrolled_drawing_area: gtk::ScrolledWindow,
}

impl DrawingArea {
    pub fn new(app: &Application) -> Self {
        let drawing_area: gtk::DrawingArea =
            app.builder.get_object("drawing_area").unwrap();
        let scrolled_drawing_area: gtk::ScrolledWindow =
            app.builder.get_object("scrolled_drawing_area").unwrap();

        DrawingArea {
            parent: drawing_area,
            _current_tool: CurrentTool::Tool(Tool::Controller),
            _scrolled_drawing_area: scrolled_drawing_area,
        }
    }

    fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app
            .main_window
            .as_ref()
            .unwrap()
            .clone();
        let drawing_area = window
            .borrow()
            .drawing_area
            .as_ref()
            .unwrap()
            .clone();
        let drawing_area = drawing_area
            .borrow()
            .parent
            .clone();

        // connect drawing_area::draw to document::draw
        {
            let window = window.clone();
            drawing_area.connect_draw(move |me, cr| {
                {
                    if window.borrow().documents.len() == 0 {
                        return Inhibit(false);
                    }
                }

                let current_doc;
                {
                    current_doc = window.borrow()
                                        .current_document;
                }

                let page_n;
                {
                    page_n = window.borrow()
                                   .documents[current_doc]
                                   .page_number;
                }

                let page_bound;
                {
                    page_bound = window.borrow()
                                       .documents[current_doc]
                                       .pages[page_n]
                                       .draw_extents();
                }

                if let Some(val) = page_bound {
                    let width = (val.maxs().x - val.mins().x).abs();
                    let height = (val.maxs().y - val.mins().y).abs();
                    me.set_size_request(
                        (width * 1.3) as i32,
                        (height * 1.3) as i32
                    );

                    let half_width = width / 2.0;
                    let half_height = height / 2.0;
                    let draw_area = me.get_allocated_size();
                    let da_height = draw_area.0.height as f64;
                    let da_width = draw_area.0.width as f64;

                    // TODO: translate the page to the center of drawingarea
                    {
                        window.borrow_mut()
                              .documents[current_doc]
                              .pages[page_n]
                              .translate = Vector::new(
                                (da_width / 2.0) - half_width,
                                (da_height / 2.0) - half_height
                              );
                    }
                }

                {
                    window.borrow()
                          .documents[current_doc]
                          .draw(cr);
                }

                // FIXME: I'm pretty sure this is not the right way!
                me.queue_draw();
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_motion_notify_event to document::motion_notify
        {
            let window = window.clone();
            drawing_area.connect_motion_notify_event(move |_me, event| {
                {
                    if window.borrow()
                             .documents
                             .len() == 0 {
                        return Inhibit(false);
                    }
                }

                let current_document;
                {
                    current_document = window.borrow()
                                             .current_document;
                }

                window.borrow_mut()
                      .documents[current_document]
                      .motion_notify(event);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_button_press_event to document::button_press
        {
            let window = window.clone();
            drawing_area.connect_button_press_event(move |_me, event| {
                {
                    if window.borrow()
                             .documents
                             .len() == 0 {
                        return Inhibit(false);
                    }
                }

                let current_document;
                {
                    current_document = window.borrow()
                                             .current_document;
                }

                window.borrow_mut()
                      .documents[current_document]
                      .button_press(event);
                Inhibit(true)
            });
        }

        // connect drawing_area::connect_button_release_event to document::button_release
        {
            let window = window.clone();
            drawing_area.connect_button_release_event(move |_me, event| {
                {
                    if window.borrow()
                             .documents
                             .len() == 0 {
                        return Inhibit(false);
                    }
                }

                let current_document;
                {
                    current_document = window.borrow()
                                             .current_document;
                }

                window.borrow_mut()
                      .documents[current_document]
                      .button_release(event);
                Inhibit(true)
            });
        }
    }
}
