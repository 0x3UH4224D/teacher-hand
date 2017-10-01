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
    message_box: gtk::RadioButton,
    curve_arrow: gtk::RadioButton,
    line_arrow: gtk::RadioButton,
    text_box: gtk::RadioButton,
    highlighter: gtk::RadioButton,
    sticker: gtk::RadioButton,
    blur_box: gtk::RadioButton,
}

impl Toolbar {
    pub fn new(builder: gtk::Builder) -> Self {
        let parent: gtk::Stack =
            builder.get_object("ed_toolbar").unwrap();

        let message_box: gtk::RadioButton =
            builder.get_object("ed_tb_message_box").unwrap();
        let curve_arrow: gtk::RadioButton =
            builder.get_object("ed_tb_curve_arrow").unwrap();
        let line_arrow: gtk::RadioButton =
            builder.get_object("ed_tb_line_arrow").unwrap();
        let text_box: gtk::RadioButton =
            builder.get_object("ed_tb_text_box").unwrap();
        let highlighter: gtk::RadioButton =
            builder.get_object("ed_tb_highlighter").unwrap();
        let sticker: gtk::RadioButton =
            builder.get_object("ed_tb_sticker").unwrap();
        let blur_box: gtk::RadioButton =
            builder.get_object("ed_tb_blur_box").unwrap();

        Toolbar {
            parent: parent,
            message_box: message_box,
            curve_arrow: curve_arrow,
            line_arrow: line_arrow,
            text_box: text_box,
            highlighter: highlighter,
            sticker: sticker,
            blur_box: blur_box,
        }
    }

    pub fn get_parent(&self) -> gtk::Stack {
        self.parent.clone()
    }

    pub fn get_message_box(&self) -> gtk::RadioButton {
        self.message_box.clone()
    }

    pub fn get_curve_arrow(self) -> gtk::RadioButton {
        self.curve_arrow.clone()
    }

    pub fn get_line_arrow(&self) -> gtk::RadioButton {
        self.line_arrow.clone()
    }

    pub fn get_text_box(&self) -> gtk::RadioButton {
        self.text_box.clone()
    }

    pub fn get_highlighter(&self) -> gtk::RadioButton {
        self.highlighter.clone()
    }

    pub fn get_sticker(&self) -> gtk::RadioButton {
        self.sticker.clone()
    }

    pub fn get_blur_box(&self) -> gtk::RadioButton {
        self.blur_box.clone()
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

        // message_box

        // curve_arrow
        {
            let window = window.clone();
            toolbar.borrow()
                   .curve_arrow
                   .connect_property_active_notify(move |me| {
                line_arrow(me, line_arrow::Mode::CreatingCurveArrow, &window);
            });
        }

        // line_arrow
        {
            let window = window.clone();
            toolbar.borrow()
                   .line_arrow
                   .connect_property_active_notify(move |me| {
                line_arrow(me, line_arrow::Mode::CreatingLineArrow, &window);
            });
        }
    }
}
