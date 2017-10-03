//
// draw_objects.rs
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

pub mod page;
pub mod layer;
pub mod line_arrow;

pub use self::page::Page;
pub use self::layer::Layer;
pub use self::line_arrow::LineArrow;

use gdk::{EventMotion, EventButton, EventKey};
use gtk::{self};

use core::context::Context;
use common::types::*;

pub trait Draw {
    fn draw(&self, cr: &Context);
    fn in_draw(&self, pos: &Point, cr: &Context) -> bool;
    fn draw_extents(&self, cr: &Context) -> Option<Rectangle>;
}

pub trait Flip {
    fn flip_vertical(&mut self);
    fn flip_horizontal(&mut self);
}

pub trait Order: Container
    where ShapeTrait: Select {

    fn move_child_forward(&mut self, index: usize) -> bool {
        // exit if index is out of bounds or
        // exit if there is less than two shapes or
        // exit if index is the last shape in the list
        if self.get_children().len() - 1 < index ||
           self.get_children().len() < 2 ||
           self.get_children().len() - 1 == index {
            return false;
        }

        self.get_mut_children().swap(index, index + 1);
        true
    }

    fn move_selected_children_forward(&mut self) {
        // exit if there is less than two shapes or
        if self.get_children().len() < 2 {
            return;
        }

        let mut i = self.get_children().len() - 2;
        loop {
            if self.get_children()[i].is_selected() &&
              !self.get_children()[i + 1].is_selected() {
                self.move_child_forward(i);
            }

            if i == 0 {
                break;
            } else {
                i -= 1;
            }
        }
    }

    fn move_child_to_front(&mut self, index: usize) -> bool {
        // exit if index is out of bounds or
        // exit if there is less than two shapes or
        // exit if index is the last shape in the list
        if self.get_children().len() - 1 < index ||
           self.get_children().len() < 2 ||
           self.get_children().len() - 1 == index {
            return false;
        }

        let shape = self.get_mut_children().remove(index);
        self.get_mut_children().push(shape);
        true
    }

    fn move_selected_children_to_front(&mut self) {
        let mut shapes = vec![];
        let mut i = 0;
        while i < self.get_children().len() {
            if self.get_children()[i].is_selected() {
                shapes.push(self.get_mut_children().remove(i));
            } else {
                i += 1;
            }
        }

        if !shapes.is_empty() {
            self.get_mut_children().append(&mut shapes);
        }
    }

    fn move_child_backward(&mut self, index: usize) -> bool {
        // exit if index is out of bounds or
        // exit if there is less than two shapes or
        // exit if index is the first shape in the list
        if self.get_children().len() - 1 < index ||
           self.get_children().len() < 2 ||
           0 == index {
            return false;
        }

        self.get_mut_children().swap(index, index - 1);
        true
    }

    fn move_selected_children_backward(&mut self) {
        // exit if there is less than two shapes or
        if self.get_children().len() < 2 {
            return;
        }

        let mut i = 1;
        loop {
            if self.get_children()[i].is_selected() &&
              !self.get_children()[i - 1].is_selected() {
                self.move_child_backward(i);
            }

            if i == self.get_children().len() - 1 {
                break;
            } else {
                i += 1;
            }
        }
    }

    fn move_child_to_rear(&mut self, index: usize) -> bool {
        // exit if index is out of bounds or
        // exit if there is less than two shapes or
        // exit if index is the last shape in the list
        if self.get_children().len() - 1 < index ||
           self.get_children().len() < 2 ||
           0 == index {
            return false;
        }

        let shape = self.get_mut_children().remove(index);
        self.get_mut_children().insert(0, shape);
        true
    }

    fn move_selected_children_to_rear(&mut self) {
        let mut shapes = vec![];
        let mut i = 0;
        while i < self.get_children().len() {
            if self.get_children()[i].is_selected() {
                shapes.push(self.get_mut_children().remove(i));
            } else {
                i += 1;
            }
        }

        if !shapes.is_empty() {
            shapes.append(self.get_mut_children());
            *self.get_mut_children() = shapes;
        }
    }
}

pub trait Rotate {
    fn rotate_left(&mut self);
    fn rotate_right(&mut self);
}

pub trait Name {
    fn name(&self) -> String;
    fn set_name(&mut self, name: &String);
}

pub trait Color {
    fn get_color(&self) -> RgbaColor;
    fn set_color(&mut self, color: &RgbaColor);
}

pub trait Move {
    fn position(&self) -> Point;
    fn move_to(&mut self, &Point);
    fn translate_by(&mut self, &Translation);
    fn rotate_by(&mut self, &Rotation, &Vector);
}

pub trait Select {
    fn is_selected(&self) -> bool;
    fn select(&mut self);
    fn unselect(&mut self);
    fn toggle_select(&mut self) -> bool;
}

pub trait Lock {
    fn is_locked(&self) -> bool;
    fn lock(&mut self);
    fn unlock(&mut self);
    fn toggle_lock(&mut self) -> bool;
}

pub trait Visible {
    fn is_visible(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn toggle_visible(&mut self) -> bool;
}

pub trait Container {
    fn add(&mut self, child: Box<ShapeTrait>);
    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>>;
    fn get_children(&self) -> &Vec<Box<ShapeTrait>>;
    fn get_mut_children(&mut self) -> &mut Vec<Box<ShapeTrait>>;
    fn set_children(&mut self, children: Vec<Box<ShapeTrait>>);
}

pub trait Event {
    // like gtk::Widget events it return "TRUE to stop other handlers from
    // being invoked for the event. FALSE to propagate the event further."
    #[allow(unused_variables)]
    fn motion_notify(
        &mut self,
        event: &EventMotion,
        pos: &Point,
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context,
        options_widget: &gtk::Notebook
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_release(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn key_press(
        &mut self,
        event: &EventKey,
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn key_release(
        &mut self,
        event: &EventKey,
        cr: &Context
    ) -> bool { false }
}

pub trait Mode {
    fn in_creating_mode(&self) -> bool;
    fn in_editing_mode(&self) -> bool;
}

pub trait ShapeTrait: Draw + Name + Color + Move + Select + Lock + Visible +
                      Container + Event + Mode + Order + Flip {}

