//
// controlbar.rs
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

pub struct Controlbar {
    parent: gtk::Stack,

    move_forward: gtk::Button,
    move_to_front: gtk::Button,
    move_backward: gtk::Button,
    move_to_rear: gtk::Button,

    align_top_edges: gtk::Button,
    align_left_edges: gtk::Button,
    align_right_edges: gtk::Button,
    align_bottom_edges: gtk::Button,
    align_center: gtk::Button,
    align_more: gtk::MenuButton,

    flip_vertical: gtk::Button,
    flip_horizontal: gtk::Button,

    rotate_right: gtk::Button,
    rotate_left: gtk::Button,
}

impl Controlbar {
    pub fn new(builder: gtk::Builder) -> Self {
        let parent: gtk::Stack =
            builder.get_object("ed_controlbar").unwrap();

        let move_forward: gtk::Button =
            builder.get_object("ed_cb_move_forward").unwrap();
        let move_to_front: gtk::Button =
            builder.get_object("ed_cb_move_to_front").unwrap();
        let move_backward: gtk::Button =
            builder.get_object("ed_cb_move_backward").unwrap();
        let move_to_rear: gtk::Button =
            builder.get_object("ed_cb_move_to_rear").unwrap();

        let align_top_edges: gtk::Button =
            builder.get_object("ed_cb_align_top_edges").unwrap();
        let align_left_edges: gtk::Button =
            builder.get_object("ed_cb_align_left_edges").unwrap();
        let align_right_edges: gtk::Button =
            builder.get_object("ed_cb_align_right_edges").unwrap();
        let align_bottom_edges: gtk::Button =
            builder.get_object("ed_cb_align_bottom_edges").unwrap();
        let align_center: gtk::Button =
            builder.get_object("ed_cb_align_center").unwrap();
        let align_more: gtk::MenuButton =
            builder.get_object("ed_cb_align_more").unwrap();

        let flip_vertical: gtk::Button =
            builder.get_object("ed_cb_flip_vertical").unwrap();
        let flip_horizontal: gtk::Button =
            builder.get_object("ed_cb_flip_horizontal").unwrap();

        let rotate_right: gtk::Button =
            builder.get_object("ed_cb_rotate_right").unwrap();
        let rotate_left: gtk::Button =
            builder.get_object("ed_cb_rotate_left").unwrap();

        Controlbar {
            parent: parent,
            move_forward: move_forward,
            move_to_front: move_to_front,
            move_backward: move_backward,
            move_to_rear: move_to_rear,
            align_top_edges: align_top_edges,
            align_left_edges: align_left_edges,
            align_right_edges: align_right_edges,
            align_bottom_edges: align_bottom_edges,
            align_center: align_center,
            align_more: align_more,
            flip_vertical: flip_vertical,
            flip_horizontal: flip_horizontal,
            rotate_right: rotate_right,
            rotate_left: rotate_left,
        }
    }

    pub fn get_parent(&self) -> gtk::Stack {
        self.parent.clone()
    }

    pub fn connect_ui(app: Rc<RefCell<Application>>) {
        let teha_app = app.borrow();
        let window = teha_app.get_main_window();
        let controlbar = window.borrow().get_controlbar();

        {
            let window = window.clone();
            controlbar.borrow()
                      .move_forward
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .move_selected_children_forward();
            });
        }

        {
            let window = window.clone();
            controlbar.borrow()
                      .move_to_front
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .move_selected_children_to_front();
            });
        }

        {
            let window = window.clone();
            controlbar.borrow()
                      .move_backward
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .move_selected_children_backward();
            });
        }

        {
            let window = window.clone();
            controlbar.borrow()
                      .move_to_rear
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .move_selected_children_to_rear();
            });
        }

        {
            let window = window.clone();
            controlbar.borrow()
                      .flip_horizontal
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .flip_selected_children_horizontally();
            });
        }

        {
            let window = window.clone();
            controlbar.borrow()
                      .flip_vertical
                      .connect_clicked(move |me| {
                window.borrow_mut()
                      .get_mut_active_document()
                      .get_mut_active_page()
                      .get_mut_active_layer()
                      .flip_selected_children_vertically();
            });
        }
    }
}
