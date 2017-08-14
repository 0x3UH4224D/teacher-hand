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

use std::ops::AddAssign;
use std::rc::Weak;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

use cairo;
use gdk::{self, ModifierType, EventMotion, EventButton};
use gtk::prelude::*;

use ncollide;
use ncollide::transformation::ToPolyline;
use na;
use alga::linear::{Transformation, ProjectiveTransformation};

use palette::{self, Rgb};

type Point          = na::Point2<f64>;
type Vector         = na::Vector2<f64>;
type Translation    = na::Translation2<f64>;
type Rotation       = na::Rotation2<f64>;
type Segment        = ncollide::shape::Segment2<f64>;
type Cone           = ncollide::shape::Cone<f64>;

type RgbaColor      = palette::Alpha<Rgb<f64>, f64>;
type Context        = cairo::Context;

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
    fn position(&self) -> Point;
    fn move_to(&mut self, &Point);
    fn translate_by(&mut self, &Translation);
    fn rotate_by(&mut self, &Rotation, &Vector);
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
    // ID field
    pub name: String,

    // control fields
    pub lock: bool,
    pub selected: bool,

    // draw fields
    pub visible: bool,
    pub color: RgbaColor,
    pub width: f64,
    pub cap: cairo::LineCap,
    pub join: cairo::LineJoin,
    pub dashes: Vec<f64>,
    pub offset: f64,

    // Segment field
    pub segment: Segment,

    // Curve fields
    pub curve_like: bool,
    // these vector needed if we want to convert this line to curve
    pub go_dir: Vector, // dir refer to direction
    pub arrive_dir: Vector,
}

impl LineArrow {
    fn new(color: RgbaColor, width: f64, cap: cairo::LineCap,
           join: cairo::LineJoin, dashes: Vec<f64>, offset: f64,
           curve_like: bool, segment: Segment) -> Self {
        LineArrow {
            children: vec![],
            name: String::new(),
            lock: false,
            selected: false,
            visible: true,
            color: color,
            width: width,
            cap: cap,
            join: join,
            dashes: dashes,
            offset: offset,
            segment: segment,
            curve_like: curve_like,
            go_dir: Vector::new(0.0, 0.0),
            arrive_dir: Vector::new(0.0, 0.0),
        }
    }

    fn new_from_segment(segment: Segment) -> Self {
        LineArrow {
            children: vec![],
            name: String::new(),
            lock: false,
            selected: false,
            visible: true,
            color: RgbaColor::new(1.0, 0.0, 0.0, 1.0),
            width: 10.0,
            cap: cairo::LineCap::Round,
            join: cairo::LineJoin::Round,
            dashes: vec![],
            offset: 0.0,
            segment: segment,
            curve_like: false,
            go_dir: Vector::new(0.0, 0.0),
            arrive_dir: Vector::new(0.0, 0.0),
        }
    }

    fn draw_segment(&self, cr: &Context) {
        cr.save();

        cr.set_source_rgba(
            self.color.color.red,
            self.color.color.green,
            self.color.color.blue,
            self.color.alpha
        );
        cr.set_line_width(self.width);
        cr.set_line_cap(self.cap);
        cr.set_line_join(self.join);
        cr.set_dash(self.dashes.as_slice(), self.offset);

        let start = self.segment.a();
        let go_dir = &self.go_dir;
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        // start point
        cr.move_to(start.x, start.y);
        if self.curve_like {
            cr.curve_to(
                // go direction
                go_dir.x + start.x,
                go_dir.y + start.y,
                // arrive direction
                arrive_dir.x + end.x,
                arrive_dir.y + end.y,
                // end point
                end.x,
                end.y);
        } else {
            // end point
            cr.line_to(end.x, end.y);
        }
        cr.stroke();
        cr.restore();
    }

    // TODO: Not finished.
    fn draw_head(&self, cr: &Context) {
        cr.save();
        let start = self.segment.a();
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        cr.set_source_rgba(
            self.color.color.red,
            self.color.color.green,
            self.color.color.blue,
            self.color.alpha
        );

        let mut triangle = Cone::new(self.width * 1.25, self.width * 1.25).to_polyline(());

        let mut rotate;
        if self.curve_like {
            rotate = Rotation::rotation_between(
                &arrive_dir,
                &Vector::new(0.0, -1.0),
            );
        } else {
            // Convert @start point to Vector with @end point as it's origin.
            let start_vec = Translation::new(-end.x, -end.y).transform_point(&start);
            // calcualte the angle between @start_vec and our triangle.
            rotate = Rotation::rotation_between(
                &Vector::new(start_vec.x, start_vec.y),
                // y = -1.0 becuse cairo/gtk invert the y-axis after drawing it.
                &Vector::new(0.0, -1.0),
            );
        }
        // we make sure that the angle is not negative value.
        let angle = ((360_f64).to_radians() - rotate.angle())
            .to_degrees()
            .abs()
            .to_radians();
        // create a Rotation object.
        rotate = Rotation::new(angle);
        // rotate @triangle
        triangle.rotate_by(&rotate);
        // translate @triangle to end point.
        triangle.translate_by(&Translation::new(end.x, end.y));

        cr.move_to(triangle.coords()[0].x, triangle.coords()[0].y);
        cr.line_to(triangle.coords()[1].x, triangle.coords()[1].y);
        cr.line_to(triangle.coords()[2].x, triangle.coords()[2].y);
        cr.close_path();
        // cr.stroke();
        cr.fill();
        cr.restore();
    }

    fn draw_tail(&self, cr: &Context) {
        // TODO
    }

    fn draw_selected(&self, cr: &Context) {
        cr.save();

        let start = self.segment.a();
        let go_dir = &self.go_dir;
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let stroke_color = self.color.color.clone();

        if self.curve_like {
            cr.save();

            cr.set_source_rgba(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
                0.3,
            );
            cr.set_line_width(2.0);
            cr.set_dash(&[10.0], 0.0);

            cr.move_to(start.x, start.y);
            cr.rel_line_to(go_dir.x, go_dir.y);
            cr.stroke();

            cr.move_to(end.x, end.y);
            cr.rel_line_to(arrive_dir.x, arrive_dir.y);
            cr.stroke();

            cr.restore();
        }

        cr.restore();
    }

    fn draw_controllers(&self, cr: &Context) {
        cr.save();

        let start = self.segment.a();
        let go_dir = &self.go_dir;
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let fill_color = Rgb::new(0.97, 0.97, 1.0); // #F8F8FF
        let stroke_color = Rgb::new(0.47, 0.53, 0.60); // #778899

        let radius;
        if self.width < 10.0  {
            radius = 3.0
        } else if self.width > 20.0 {
            radius = 9.0
        } else {
            // radius will be in range 4.0-8.0
            radius = self.width * 0.40;
        }

        // draw @go_dir and @arrive_dir circle
        cr.new_sub_path();
        cr.arc(start.x + go_dir.x, start.y + go_dir.y,
               radius,
               0.0, (360_f64).to_radians());

        cr.new_sub_path();
        cr.arc(end.x + arrive_dir.x, end.y + arrive_dir.y,
               radius,
               0.0, (360_f64).to_radians());

        // draw start and end circle
        cr.new_sub_path();
        cr.arc(start.x, start.y,
               radius,
               0.0, (360_f64).to_radians());

        cr.new_sub_path();
        cr.arc(end.x, end.y,
               radius,
               0.0, (360_f64).to_radians());

        cr.set_source_rgb(
            fill_color.red,
            fill_color.green,
            fill_color.blue,
        );
        cr.fill_preserve();
        cr.set_source_rgb(
            stroke_color.red,
            stroke_color.green,
            stroke_color.blue,
        );
        cr.stroke();

        cr.restore();
    }
}

impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context) {
        if !self.visible || self.color.alpha == 0.0 {
            return;
        }

        self.draw_segment(cr);
        self.draw_head(cr);
        self.draw_tail(cr);
        if self.selected {
            self.draw_selected(cr);
            self.draw_controllers(cr);
        }

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
    // get the position/center of this line.
    fn position(&self) -> Point {
        na::center(self.segment.a(), self.segment.b())
    }

    fn move_to(&mut self, pos: &Point) {
        let center = self.position();
        let mut trans = Translation::new(-center.x, -center.y);
        let mut a = trans.transform_point(self.segment.a());
        let mut b = trans.transform_point(self.segment.b());
        trans = Translation::new(pos.x, pos.y);
        a = trans.transform_point(&a);
        b = trans.transform_point(&b);
        self.segment = Segment::new(a, b);
    }

    fn translate_by(&mut self, trans: &Translation) {
        let a = trans.transform_point(self.segment.a());
        let b = trans.transform_point(self.segment.b());
        self.segment = Segment::new(a, b);
    }

    // TODO: test origin functionality.
    fn rotate_by(&mut self, rotate: &Rotation, origin: &Vector) {
        let mut center = self.position() + origin;
        let trans = Translation::new(-center.x, -center.y);
        let mut a = trans.transform_point(self.segment.a());
        let mut b = trans.transform_point(self.segment.b());

        a = rotate.transform_point(&a);
        b = rotate.transform_point(&b);

        a = trans.inverse_transform_point(&a);
        b = trans.inverse_transform_point(&b);

        self.segment = Segment::new(a, b);
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

impl Event for LineArrow {
    // TODO: override default methods
}
