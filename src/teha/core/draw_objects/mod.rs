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
                      Container + Event + Mode {}

