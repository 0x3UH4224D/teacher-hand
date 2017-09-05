//
// layer.rs
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

use cairo;
use gdk::{EventMotion, EventButton};

use gettextrs::*;

use ncollide::bounding_volume::BoundingVolume;

use core::context::Context;
use common::types::*;
use super::{Draw, Name, Lock, Visible, Container, Event, ShapeTrait, LineArrow};

pub trait LayerTrait: Draw + Name + Lock + Visible + Container + Event {}

pub struct Layer {
    pub children: Vec<Box<ShapeTrait>>,
    pub name: String,
    pub lock: bool,
    pub visible: bool,
}

impl Layer {
    pub fn new() -> Self {
        // FIXME: this line for testing/debugging
        // and should be removeed when release

        let mut line_arrow = LineArrow::new(
            RgbaColor::new(0.5, 0.5, 1.0, 1.0),
            10.0,
            cairo::LineCap::Round,
            cairo::LineJoin::Round,
            vec![],
            0.0,
            true,
            Segment::new(Point::new(-50.0, -50.0),
                         Point::new(50.0, 50.0)),
        );
        line_arrow.go_dir = Vector::new(10.0, 80.0);
        line_arrow.arrive_dir = Vector::new(-10.0, -80.0);

        let mut children: Vec<Box<ShapeTrait>> = vec![];
        children.push(Box::new(line_arrow));

        Layer {
            children: children,
            name: gettext("Unnamed Layer"),
            lock: false,
            visible: true,
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            children: vec![],
            name: gettext("Unnamed Layer"),
            lock: false,
            visible: true,
        }
    }
}

impl LayerTrait for Layer {}

impl Draw for Layer {
    fn draw(&self, cr: &Context) {
        if !self.visible {
            return;
        }

        for child in self.children.iter() {
            child.draw(cr);
        }
    }

    fn in_draw( &self, pos: &Point, cr: &Context) -> bool {
        if !self.visible {
            return false;
        }

        for child in self.children.iter() {
            if child.in_draw(pos, cr) {
                return true;
            }
        }
        false
    }

    fn draw_extents(&self, cr: &Context) -> Option<Rectangle> {
        if self.children.len() == 0 {
            return None;
        }

        let mut iter =
            self.children
                .iter()
                .filter_map(|s| s.draw_extents(cr));
        let init = match iter.next() {
            None => return None,
            Some(val) => val,
        };
        let result = iter.fold(init, |acc, ref x| acc.merged(x));
        Some(result)
    }
}

impl Name for Layer {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Lock for Layer {
    fn is_locked(&self) -> bool {
        self.lock
    }

    fn lock(&mut self) {
        if !self.lock {
            self.lock = true;
        }
    }

    fn unlock(&mut self) {
        if self.lock {
            self.lock = false;
        }
    }

    fn toggle_lock(&mut self) -> bool {
        self.lock = !self.lock;
        self.lock
    }
}

impl Visible for Layer {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        if !self.visible {
            self.visible = true;
        }
    }

    fn hide(&mut self) {
        if self.visible {
            self.visible = false;
        }
    }

    fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }
}

impl Container for Layer {
    fn add(&mut self, child: Box<ShapeTrait>) {
        self.children.push(child);
    }

    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>> {
        if index > self.children.len() {
            None
        } else {
            Some(self.children.remove(index))
        }
    }
}

impl Event for Layer {
    fn motion_notify(
        &mut self,
        event: &EventMotion,
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.motion_notify(event, pos, cr) {
                return true;
            }
        }
        false
    }

    fn button_press(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_press(event, pos, cr) {
                return true;
            }
        }
        false
    }

    fn button_release(
        &mut self,
        event: &EventButton,
        pos: &Point,
        cr: &Context
    ) -> bool {
        for child in self.children.iter_mut() {
            if child.button_release(event, pos, cr) {
                return true;
            }
        }
        false
    }
}
