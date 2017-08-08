//
// draw-objects.rs
//
// Copyright (C) 2017 Muhannad Alrusayni <0x3UH4224D@gmail.com>
//
// This file is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as
// published by the Free Software Foundation; either version 3 of the
// License, or (at your option) any later version.
//
// This file is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use std::ops::AddAssign;
use std::rc::Weak;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

use cairo;
use gdk::{self, ModifierType, EventMotion, EventButton};
use gtk::prelude::*;

use euclid::{self, Point2D};

use palette::{self, Rgb};

type Point = euclid::Point2D<f64>;
type Vector = euclid::Vector2D<f64>;
type Line = euclid::Rect<f64>;
type RgbaColor = palette::Alpha<Rgb<f64>, f64>;
type Context = cairo::Context;

pub trait Draw {
    fn draw(&self, cr: &Context);
}

pub trait Name {
    fn get_mut_name(&mut self) -> &mut String;
    fn get_name(&self) -> &String;

    fn set_name(&mut self, name: &str) {
        self.get_mut_name().clone_from(&name.to_string());
    }
}

pub trait Move {
    fn get_mut_position(&mut self) -> &mut Point;
    fn get_position(&self) -> &Point;

    fn move_to(&mut self, pos: Point) {
        let point = self.get_mut_position();
        point.x += pos.x;
        point.y += pos.y;
    }

    fn transform(&mut self, vec: Vector) {
        let point = self.get_mut_position();
        point.add_assign(vec);
    }
}

pub trait Lock {
    fn get_mut_lock(&mut self) -> &mut bool;
    fn get_lock(&self) -> &bool;

    fn lock(&mut self) {
        let lock = self.get_mut_lock();
        if *lock == false {
            lock.clone_from(&true);
        }
    }

    fn unlock(&mut self) {
        let lock = self.get_mut_lock();
        if *lock == true {
            lock.clone_from(&false);
        }
    }

    fn toggle_lock(&mut self) -> bool {
        let lock = self.get_mut_lock();
        let mut new_val = !lock.clone();
        lock.clone_from(&new_val);
        new_val
    }
}

pub trait Visible {
    fn get_mut_visible(&mut self) -> &mut bool;
    fn get_visible(&self) -> &bool;

    fn show(&mut self) {
        let visible = self.get_mut_visible();
        if *visible == false {
            visible.clone_from(&true);
        }
    }

    fn hide(&mut self) {
        let visible = self.get_mut_visible();
        if *visible == true {
            visible.clone_from(&false);
        }
    }

    fn toggle_visible(&mut self) -> bool {
        let visible = self.get_mut_visible();
        let mut new_val = !visible.clone();
        visible.clone_from(&new_val);
        new_val
    }
}

pub trait Container {
    fn get_mut_children(&mut self) -> &mut Vec<Box<ShapeTrait>>;
    fn get_children(&self) -> &Vec<Box<ShapeTrait>>;

    fn add(&mut self, child: Box<ShapeTrait>) {
        let children = self.get_mut_children();
        children.push(child);
    }

    fn remove(&mut self, index: usize) -> Option<Box<ShapeTrait>> {
        let children = self.get_mut_children();
        if index > children.len() {
            None
        } else {
            Some(children.remove(index))
        }
    }
}

pub trait Event {
    fn motion_notify(&self, event: &EventMotion) {}
    fn button_press(&self, event: &EventButton) {}
    fn button_release(&self, event: &EventButton) {}
}

pub struct Page {
    pub layers: Vec<Box<LayerTrait>>,
    pub color: Option<RgbaColor>,
    pub border: Option<RgbaColor>,
    pub grid: Option<RgbaColor>,
}

impl Draw for Page {
    fn draw(&self, cr: &Context) {
        // TODO: draw page..
        for layer in self.layers.iter() {
            layer.draw(cr);
        }
    }
}

pub trait LayerTrait: Draw + Name + Lock + Visible + Container {}

pub struct Layer {
    pub children: Vec<Box<ShapeTrait>>,
    pub name: String,
    pub lock: bool,
    pub visible: bool,
}

impl LayerTrait for Layer {}

impl Draw for Layer {
    fn draw(&self, cr: &Context) {
        for child in self.children.iter() {
            child.draw(cr);
        }
    }
}

impl Name for Layer {
    fn get_mut_name(&mut self) -> &mut String {
        &mut self.name
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl Lock for Layer {
    fn get_mut_lock(&mut self) -> &mut bool {
        &mut self.lock
    }

    fn get_lock(&self) -> &bool {
        &self.lock
    }
}

impl Visible for Layer {
    fn get_mut_visible(&mut self) -> &mut bool {
        &mut self.visible
    }

    fn get_visible(&self) -> &bool {
        &self.visible
    }
}

impl Container for Layer {
    fn get_mut_children(&mut self) -> &mut Vec<Box<ShapeTrait>> {
        &mut self.children
    }

    fn get_children(&self) -> &Vec<Box<ShapeTrait>> {
        &self.children
    }
}

pub trait ShapeTrait: Draw + Name + Move + Lock +
          Visible + Container + Event {}

pub struct LineArrow {
    pub children: Vec<Box<ShapeTrait>>,
    pub name: String,
    pub lock: bool,
    pub visible: bool,
    pub color: RgbaColor,

    pub width: f64,
    pub line_cap: cairo::LineCap,
    pub line_join: cairo::LineJoin,
    pub dashes: Vec<f64>,
    pub offset: f64,

    // Line
    pub line: Line,

    // these vector needed if we want to convert this line to curve
    arrive_dir: Vector, // dir refer to direction
    go_dir: Vector,
    curve_like: bool,
}

impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context) {
        if !self.visible || self.color.alpha == 0.0 {
            return;
        }

        cr.save();

        cr.set_source_rgba(
            self.color.color.red,
            self.color.color.green,
            self.color.color.blue,
            self.color.alpha
        );
        cr.set_line_width(self.width);
        cr.set_line_cap(self.line_cap);
        cr.set_line_join(self.line_join);
        cr.set_dash(self.dashes.as_slice(), self.offset);

        // start point
        cr.move_to(self.line.origin.x,
                   self.line.origin.y);
        if self.curve_like {
            cr.rel_curve_to(
                // go direction
                self.go_dir.x, self.go_dir.y,
                // arrive direction
                self.line.size.width + self.arrive_dir.x,
                self.line.size.height + self.arrive_dir.y,
                // end point
                self.line.size.width, self.line.size.height
            );
        } else {
            cr.rel_line_to(self.line.size.width, self.line.size.height);
        }
        // TODO: draw arrow and tail..

        cr.stroke();
        cr.restore();

        // draw children if there are any.
        for child in self.children.iter() {
            child.draw(cr);
        }
    }
}

impl Name for LineArrow {
    fn get_mut_name(&mut self) -> &mut String {
        &mut self.name
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl Move for LineArrow {
    fn get_mut_position(&mut self) -> &mut Point {
        &mut self.line.origin
    }

    fn get_position(&self) -> &Point {
        &self.line.origin
    }
}

impl Lock for LineArrow {
    fn get_mut_lock(&mut self) -> &mut bool {
        &mut self.lock
    }

    fn get_lock(&self) -> &bool {
        &self.lock
    }
}

impl Visible for LineArrow {
    fn get_mut_visible(&mut self) -> &mut bool {
        &mut self.visible
    }

    fn get_visible(&self) -> &bool {
        &self.visible
    }
}

impl Container for LineArrow {
    fn get_mut_children(&mut self) -> &mut Vec<Box<ShapeTrait>> {
        &mut self.children
    }

    fn get_children(&self) -> &Vec<Box<ShapeTrait>> {
        &self.children
    }
}

// TODO: override default methods
impl Event for LineArrow {}
