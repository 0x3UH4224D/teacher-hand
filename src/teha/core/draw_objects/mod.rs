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

pub use self::page::*;
pub use self::layer::*;
pub use self::line_arrow::*;

use gdk::{EventMotion, EventButton};

use core::context::Context;
use common::types::*;

pub trait Draw {
    fn draw(&self, cr: &Context);
    fn in_draw(&self, pos: &Point, cr: &Context) -> bool;
    fn draw_extents(&self, cr: &Context) -> Option<Rectangle>;
}

pub trait Name {
    fn name(&self) -> &str;
    fn set_name(&mut self, name: &str);
}

pub trait Move {
    fn position(&self) -> Point;
    fn move_to(&mut self, &Point);
    fn translate_by(&mut self, &Translation);
    fn rotate_by(&mut self, &Rotation, &Vector);
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
        cr: &Context
    ) -> bool { false }
    #[allow(unused_variables)]
    fn button_release(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool { false }
}

pub trait ShapeTrait: Draw + Name + Move + Lock +
          Visible + Container + Event {}

