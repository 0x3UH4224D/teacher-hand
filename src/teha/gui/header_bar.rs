//
// header_bar.rs
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

use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

use gtk::prelude::*;
use gtk;

use gettextrs::*;

use super::*;
use super::app::Application;
use common;
use core::Document;
use common::types::*;

pub struct HeaderBar {
    parent: gtk::HeaderBar,
    left_stack: gtk::Stack,
    mid_stack: gtk::Stack,
    right_stack: gtk::Stack,
    crwo_back: gtk::Button,         // create work back
    crwo_forward: gtk::Button,      // create work forward
    ed_zoom_level: gtk::Scale,      // editor zoom level
}

impl HeaderBar {
    pub fn new(builder: gtk::Builder) -> Self {
        let header_bar: gtk::HeaderBar =
            builder.get_object("main_window_headerbar").unwrap();
        let left_stack: gtk::Stack =
            builder.get_object("left_stack").unwrap();
        let mid_stack: gtk::Stack =
            builder.get_object("mid_stack").unwrap();
        let right_stack: gtk::Stack =
            builder.get_object("right_stack").unwrap();
        let crwo_back: gtk::Button =
            builder.get_object("crwo_back").unwrap();
        let crwo_forward: gtk::Button =
            builder.get_object("crwo_forward").unwrap();
        let ed_zoom_level: gtk::Scale =
            builder.get_object("ed_zoom_level").unwrap();

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

    pub fn get_parent(&self) -> gtk::HeaderBar {
        self.parent.clone()
    }

    pub fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.get_main_window();
        let header_bar = window.borrow().get_header_bar();

        {
            // clone crwo_back to connect it to its closure
            let crwo_back = header_bar.borrow().crwo_back.clone();

            // clone the needed objects
            let app = app.clone();

            // connect "clicked" signal to a closure
            crwo_back.connect_clicked(move |_me| {
                let view_mode = app.borrow().get_last_view_mode();
                app.borrow_mut().update_view(view_mode);
            });
        }

        {
            // clone crwo_forward to connect it to its closure
            let crwo_forward = header_bar.borrow().crwo_forward.clone();

            // clone the needed objects
            let app = app.clone();
            let crwo_infobar = window.borrow().get_crwo_infobar();
            let crwo_infobar_title = window.borrow().get_crwo_infobar_title();
            let crwo_infobar_description =
                window.borrow().get_crwo_infobar_description();
            let crwo_file_name = window.borrow().get_crwo_file_name();
            let crwo_file_location = window.borrow().get_crwo_file_location();
            let crwo_width = window.borrow().get_crwo_width();
            let crwo_height = window.borrow().get_crwo_height();
            let crwo_transparent_background =
                window.borrow().get_crwo_transparent_background();
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
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .set_zoom_level(me.get_value());
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
