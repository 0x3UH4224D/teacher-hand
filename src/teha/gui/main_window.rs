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
use std::path::Path;
use std::fmt;
use std::error::Error;

use gtk::{self, WidgetExt, ApplicationExt, FlowBoxExt, StackExt, ButtonExt,
          EntryExt, LabelExt, FileChooserExt, SpinButtonExt, SwitchExt,
          AdjustmentExt};
use gio::{self};

use gettextrs::*;

use super::*;
use super::app::TehaApplication;
use common;

pub struct TehaMainWindow {
    pub parent: gtk::ApplicationWindow,
    pub header_bar: Option<Rc<RefCell<TehaHeaderBar>>>,
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

impl TehaMainWindow {
    pub fn new(app: &TehaApplication) -> Self {
        let main_window: gtk::ApplicationWindow             = app.builder.get_object("main_window").unwrap();
        let root_stack: gtk::Stack                          = app.builder.get_object("root_stack").unwrap();
        let stcrim_stack: gtk::Stack                        = app.builder.get_object("stcrim_stack").unwrap();
        let edpr_stack: gtk::Stack                          = app.builder.get_object("edpr_stack").unwrap();
        let stup_flowbox: gtk::FlowBox                      = app.builder.get_object("stup_flowbox").unwrap();
        let crwo_infobar: gtk::InfoBar                      = app.builder.get_object("crwo_infobar").unwrap();
        let crwo_infobar_title: gtk::Label                  = app.builder.get_object("crwo_infobar_title").unwrap();
        let crwo_infobar_description: gtk::Label            = app.builder.get_object("crwo_infobar_description").unwrap();
        let crwo_file_name: gtk::Entry                      = app.builder.get_object("crwo_file_name").unwrap();
        let crwo_file_location: gtk::FileChooserButton      = app.builder.get_object("crwo_file_location").unwrap();
        let crwo_width: gtk::SpinButton                     = app.builder.get_object("crwo_width").unwrap();
        let crwo_height: gtk::SpinButton                    = app.builder.get_object("crwo_height").unwrap();
        let crwo_transparent_background: gtk::Switch        = app.builder.get_object("crwo_transparent_background").unwrap();

        crwo_file_name.set_max_length(63);
        app.parent.add_window(&main_window);

        let mut teha_window = TehaMainWindow {
            parent: main_window,
            header_bar: None,
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

        let header_bar = TehaHeaderBar::new(app, &teha_window);
        teha_window.header_bar = Some(Rc::new(RefCell::new(header_bar)));

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

    // pub fn new_work()

    pub fn connect_ui(app: Rc<RefCell<TehaApplication>>) {
        let teha_app = app.borrow();
        let window = teha_app.main_window.as_ref().unwrap().borrow();
        let header_bar = window.header_bar.as_ref().unwrap().clone();
        {
            let app = app.clone();
            let stup_flowbox = window.stup_flowbox.clone();
            stup_flowbox.connect_child_activated(move |me, child| {
                let button_name = child.get_name().unwrap();
                if button_name == "startup_new_work_button" {
                    app.borrow_mut().update_view(ViewMode::CreatingWork);
                } else if button_name == "startup_import_work_button" {
                    app.borrow_mut().update_view(ViewMode::ImportingWork);
                }
            });
        }

        TehaHeaderBar::connect_ui(app.clone());
    }
}

pub struct TehaHeaderBar {
    pub header_bar: gtk::HeaderBar,
    left_stack: gtk::Stack,
    mid_stack: gtk::Stack,
    right_stack: gtk::Stack,
    crwo_back: gtk::Button,
    crwo_forward: gtk::Button,
}

impl TehaHeaderBar {
    fn new(app: &TehaApplication, window: &TehaMainWindow) -> Self {
        let header_bar: gtk::HeaderBar = app.builder.get_object("main_window_headerbar").unwrap();
        let left_stack: gtk::Stack = app.builder.get_object("left_stack").unwrap();
        let mid_stack: gtk::Stack = app.builder.get_object("mid_stack").unwrap();
        let right_stack: gtk::Stack = app.builder.get_object("right_stack").unwrap();
        let crwo_back: gtk::Button = app.builder.get_object("crwo_back").unwrap();
        let crwo_forward: gtk::Button = app.builder.get_object("crwo_forward").unwrap();

        // window.parent.set_titlebar(&header_bar);

        let mut teha_headerbar = TehaHeaderBar {
            header_bar: header_bar,
            left_stack: left_stack,
            mid_stack: mid_stack,
            right_stack: right_stack,
            crwo_back: crwo_back,
            crwo_forward: crwo_forward,
        };

        teha_headerbar
    }

    fn connect_ui(app: Rc<RefCell<TehaApplication>>) {
        let teha_app = app.borrow();
        let window = teha_app.main_window.as_ref().unwrap().clone();
        let header_bar = window.borrow().header_bar.as_ref().unwrap().clone();

        {
            // clone crwo_back to connect it to a closure
            let crwo_back = header_bar.borrow().crwo_back.clone();

            // clone the needed objects
            let app = app.clone();
            let view_mode = app.borrow().last_view_mode;

            // connect "clicked" signal to a closure
            crwo_back.connect_clicked(move |me| {
                app.borrow_mut().update_view(view_mode);
            });
        }

        {
            // clone crwo_forward to connect it to a closure
            let crwo_forward = header_bar.borrow().crwo_forward.clone();

            // clone the needed objects
            let app = app.clone();
            let crwo_infobar = window.borrow().crwo_infobar.clone();
            let crwo_infobar_title = window.borrow().crwo_infobar_title.clone();
            let crwo_infobar_description = window.borrow().crwo_infobar_description.clone();
            let crwo_file_name = window.borrow().crwo_file_name.clone();
            let crwo_file_location = window.borrow().crwo_file_location.clone();
            let crwo_width = window.borrow().crwo_width.clone();
            let crwo_height = window.borrow().crwo_height.clone();
            let crwo_transparent_background = window.borrow().crwo_transparent_background.clone();

            crwo_forward.connect_clicked(move |me| {
                // check if the file name is valid, show message through infobar
                // if not. and return
                let file_name = crwo_file_name.get_text().unwrap();
                match common::string::is_valid_filename(&file_name) {
                    Err(msg) => {
                        crwo_infobar_title.set_label(gettext("<b>Invailed File Name</b>").as_str());
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
                        crwo_infobar_title.set_label(gettext("<b>Invailed File Location</b>").as_str());
                        crwo_infobar_description.set_label(gettext("File location cannot be empty").as_str());
                        if !crwo_infobar.get_visible() {
                            crwo_infobar.show();
                        }
                        return;
                    },
                };

                // collect and combine info
                file_path.push(file_name.clone());
                file_path.set_extension("teha");
                let width = crwo_width.get_value_as_int();
                let height = crwo_height.get_value_as_int();
                let transparent_background = crwo_transparent_background.get_active();


                // create a file in the given path
                let mut file = match File::create(&file_path) {
                    Ok(file) => file,
                    Err(why) => {
                        // setting the title
                        /* TRANSLATORS: don't remove or translate <b> it's needed. note the full sentence is "Couldn't Create File" */
                        let title1 = gettext("<b>Couldn't Create");
                        let title2 = format!(" {} ", file_name);
                        /* TRANSLATORS: don't remove or translate </b> it's needed. note the full sentence is "Couldn't Create File" */
                        let title3 = gettext("File</b>");
                        crwo_infobar_title.set_label(format!("{}{}{}", title1, title2, title3).as_str());

                        // setting the description
                        let description = gettext("Error");
                        let description = format!("{} {}", description, why.description());
                        crwo_infobar_description.set_label(description.as_str());
                        if !crwo_infobar.get_visible() {
                            crwo_infobar.show();
                        }
                        return;
                    },
                };

                // cleanup the widgets for reuse
                crwo_file_name.set_text("");
                crwo_file_location.unselect_all();
                crwo_width.set_value(1280_f64);
                crwo_height.set_value(800_f64);
                crwo_transparent_background.set_active(true);
                if crwo_infobar.get_visible() {
                    crwo_infobar.hide();
                }

                // start a new work with the information we got.

                app.borrow_mut().update_view(ViewMode::Editing);
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
