//
// toolbar.rs
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

use gtk;
use gtk::prelude::*;

use super::app::Application;
use super::main_window::MainWindow;
use core::draw_objects::line_arrow::{self, LineArrow};
use common::types::*;

pub struct Toolbar {
    parent: gtk::Stack,
    // shapes
    shape_message_box: gtk::RadioButton,
    shape_curve_arrow: gtk::RadioButton,
    shape_line_arrow: gtk::RadioButton,
    shape_text_box: gtk::RadioButton,
    shape_highlighter: gtk::RadioButton,
    shape_sticker: gtk::RadioButton,
    shape_blur_box: gtk::RadioButton,
}

impl Toolbar {
    pub fn new(builder: gtk::Builder) -> Self {
        let parent: gtk::Stack =
            builder.get_object("ed_toolbar").unwrap();

        let shape_message_box: gtk::RadioButton =
            builder.get_object("ed_shape_message_box").unwrap();
        let shape_curve_arrow: gtk::RadioButton =
            builder.get_object("ed_shape_curve_arrow").unwrap();
        let shape_line_arrow: gtk::RadioButton =
            builder.get_object("ed_shape_line_arrow").unwrap();
        let shape_text_box: gtk::RadioButton =
            builder.get_object("ed_shape_text_box").unwrap();
        let shape_highlighter: gtk::RadioButton =
            builder.get_object("ed_shape_highlighter").unwrap();
        let shape_sticker: gtk::RadioButton =
            builder.get_object("ed_shape_sticker").unwrap();
        let shape_blur_box: gtk::RadioButton =
            builder.get_object("ed_shape_blur_box").unwrap();

        Toolbar {
            parent: parent,
            shape_message_box: shape_message_box,
            shape_curve_arrow: shape_curve_arrow,
            shape_line_arrow: shape_line_arrow,
            shape_text_box: shape_text_box,
            shape_highlighter: shape_highlighter,
            shape_sticker: shape_sticker,
            shape_blur_box: shape_blur_box,
        }
    }

    pub fn get_parent(&self) -> gtk::Stack {
        self.parent.clone()
    }

    pub fn get_shape_message_box(&self) -> gtk::RadioButton {
        self.shape_message_box.clone()
    }

    pub fn get_shape_curve_arrow(self) -> gtk::RadioButton {
        self.shape_curve_arrow.clone()
    }

    pub fn get_shape_line_arrow(&self) -> gtk::RadioButton {
        self.shape_line_arrow.clone()
    }

    pub fn get_shape_text_box(&self) -> gtk::RadioButton {
        self.shape_text_box.clone()
    }

    pub fn get_shape_highlighter(&self) -> gtk::RadioButton {
        self.shape_highlighter.clone()
    }

    pub fn get_shape_sticker(&self) -> gtk::RadioButton {
        self.shape_sticker.clone()
    }

    pub fn get_shape_blur_box(&self) -> gtk::RadioButton {
        self.shape_blur_box.clone()
    }

    pub fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.get_main_window();
        let toolbar = window.borrow().get_toolbar();

        fn line_arrow(
            radio: &gtk::RadioButton,
            mode: line_arrow::Mode,
            window: &Rc<RefCell<MainWindow>>
        ) {
            if radio.get_active() {
                let a = Point::new(10.0, 10.0);
                let b = Point::new(10.0, 10.0);
                let mut line_arrow = LineArrow::new(Segment::new(a, b));
                line_arrow.set_mode(mode);

                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .remove_shapes_in_creating_mode();

                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .add(Box::new(line_arrow));
            }
        }

        // shape_message_box

        // shape_curve_arrow
        {
            let window = window.clone();
            let shape_curve_arrow = toolbar.borrow().shape_curve_arrow.clone();
            shape_curve_arrow.connect_property_active_notify(move |me| {
                line_arrow(me, line_arrow::Mode::CreatingCurveArrow, &window);
            });
        }

        // shape_line_arrow
        {
            let window = window.clone();
            let shape_line_arrow = toolbar.borrow().shape_line_arrow.clone();
            shape_line_arrow.connect_property_active_notify(move |me| {
                line_arrow(me, line_arrow::Mode::CreatingLineArrow, &window);
            });
        }
    }
}
