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
use gdk::{self, ModifierType, EventMotion, EventButton, EventType};
use gtk::prelude::*;

use gettextrs::*;

use ncollide;
use ncollide::transformation::ToPolyline;
use na;
use alga::linear::{Transformation, ProjectiveTransformation};

use palette::{self};

type Point          = na::Point2<f64>;
type Vector         = na::Vector2<f64>;
type Translation    = na::Translation2<f64>;
type Rotation       = na::Rotation2<f64>;
type Segment        = ncollide::shape::Segment2<f64>;
type Cone           = ncollide::shape::Cone<f64>;

type RgbColor       = palette::Rgb<f64>;
type RgbaColor      = palette::Alpha<RgbColor, f64>;
type Context        = cairo::Context;

pub trait Draw {
    fn draw(&self, cr: &Context);
}

pub trait Name {
    fn name(&self) -> &String;
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
    fn motion_notify(&mut self, event: &EventMotion) -> bool { false }
    fn button_press(&mut self, event: &EventButton) -> bool { false }
    fn button_release(&mut self, event: &EventButton) -> bool { false }
}

pub struct Document {
    pub pages: Vec<Page>,
    page_number: usize,
    pub name: String,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            pages: vec![Page::default()],
            page_number: 0,
            name: gettext("Unnamed Document"),
        }
    }
}

impl Draw for Document {
    fn draw(&self, cr: &Context) {
        self.pages[self.page_number].draw(cr);
    }
}

impl Name for Document {
    fn name(&self) -> &String {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Event for Document {
    fn motion_notify(&mut self, event: &EventMotion) -> bool {
        self.pages[self.page_number].motion_notify(event)
    }

    fn button_press(&mut self, event: &EventButton) -> bool {
        self.pages[self.page_number].button_press(event)
    }

    fn button_release(&mut self, event: &EventButton) -> bool {
        self.pages[self.page_number].button_release(event)
    }
}

pub struct Page {
    pub layers: Vec<Box<LayerTrait>>,
    pub color: Option<RgbaColor>,
    pub border: Option<RgbColor>,
    pub grid: Option<RgbColor>,
    pub name: String,
}

impl Default for Page {
    fn default() -> Self {
        Page {
            layers: vec![Box::new(Layer::default())],
            color: None,
            border: Some(RgbColor::new(0.28, 0.28, 0.28)), // #484848
            grid: None,
            name: gettext("Unnamed Page"),
        }
    }
}

impl Draw for Page {
    fn draw(&self, cr: &Context) {
        // TODO: draw page..
        for layer in self.layers.iter() {
            layer.draw(cr);
        }
    }
}

impl Name for Page {
    fn name(&self) -> &String {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Event for Page {
    fn motion_notify(&mut self, event: &EventMotion) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.motion_notify(event) {
                return true;
            }
        }
        false
    }

    fn button_press(&mut self, event: &EventButton) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.button_press(event) {
                return true;
            }
        }
        false
    }

    fn button_release(&mut self, event: &EventButton) -> bool {
        for layer in self.layers.iter_mut() {
            if layer.button_release(event) {
                return true;
            }
        }
        false
    }
}

pub trait LayerTrait: Draw + Name + Lock + Visible + Container + Event {}

pub struct Layer {
    pub children: Vec<Box<ShapeTrait>>,
    pub name: String,
    pub lock: bool,
    pub visible: bool,
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
}

impl Name for Layer {
    fn name(&self) -> &String {
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
    fn motion_notify(&mut self, event: &EventMotion) -> bool {
        for child in self.children.iter_mut() {
            if child.motion_notify(event) {
                return true;
            }
        }
        false
    }

    fn button_press(&mut self, event: &EventButton) -> bool {
        for child in self.children.iter_mut() {
            if child.button_press(event) {
                return true;
            }
        }
        false
    }

    fn button_release(&mut self, event: &EventButton) -> bool {
        for child in self.children.iter_mut() {
            if child.button_release(event) {
                return true;
            }
        }
        false
    }
}

pub trait ShapeTrait: Draw + Name + Move + Lock +
          Visible + Container + Event {}

pub enum LineArrowControllers {
    Body,
    StartPoint,
    EndPoint,
    GoDirection,
    ArriveDirection,
}

pub struct LineArrow {
    pub children: Vec<Box<ShapeTrait>>,
    // ID field
    pub name: String,

    // control fields
    pub lock: bool,
    pub selected: bool,
    // this field for Event trait, it contains the selected controller and click
    // position, it contains None if the user click outside the draw area.
    selected_controller: Option<(LineArrowControllers, Point)>,

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
            selected_controller: None,
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
            selected_controller: None,
            visible: true,
            color: RgbaColor::new(0.0, 0.0, 0.0, 1.0),
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

    fn radius(&self) -> f64 {
        if self.width < 10.0  {
            3.0
        } else if self.width > 20.0 {
            9.0
        } else {
            // radius will be in range 4.0-8.0
            self.width * 0.40
        }
    }

    fn fill_color(&self) -> RgbColor {
        RgbColor::new(0.97, 0.97, 1.0) // #F8F8FF
    }

    fn stroke_color(&self) -> RgbColor {
        RgbColor::new(0.47, 0.53, 0.60) // #778899
    }

    fn select_controller(&self, pos: &Point) -> Option<LineArrowControllers> {
        // we create zero size image since we don't really need it, but it's
        // required to create cairo::Context
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0);
        let cr = &Context::new(&surface);

        cr.save();
        self.draw_start_point(cr, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::StartPoint);
        }
        cr.restore();

        cr.save();
        self.draw_end_point(cr, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::EndPoint);
        }
        cr.restore();

        cr.save();
        self.draw_go_direction(cr, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::GoDirection);
        }
        cr.restore();

        cr.save();
        self.draw_arrive_direction(cr, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::ArriveDirection);
        }
        cr.restore();

        cr.save();
        self.draw_segment(cr, false);
        if cr.in_stroke(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        cr.save();
        self.draw_head(cr, false);
        if cr.in_fill(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        cr.save();
        self.draw_tail(cr, false);
        if cr.in_fill(pos.x, pos.y) {
            return Some(LineArrowControllers::Body);
        }
        cr.restore();

        None
    }

    fn draw_segment(&self, cr: &Context, draw_it: bool) {
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

        if draw_it {
            cr.stroke();
        }
    }

    // TODO: Not finished.
    fn draw_head(&self, cr: &Context, draw_it: bool) {
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
        if draw_it {
            cr.fill();
        }
    }

    fn draw_tail(&self, cr: &Context, draw_it: bool) {
        // TODO
    }

    fn draw_body(&self, cr: &Context, draw_it: bool) {
        cr.save();
        self.draw_segment(cr, draw_it);
        cr.restore();
        cr.save();
        self.draw_head(cr, draw_it);
        cr.restore();
        cr.save();
        self.draw_tail(cr, draw_it);
        cr.restore();
    }

    fn draw_start_point(&self, cr: &Context, draw_it: bool) {
        let start = self.segment.a();
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();
        let radius = self.radius();

        // draw start and end circle
        cr.new_sub_path();
        cr.arc(start.x, start.y,
               radius,
               0.0, (360_f64).to_radians());

        if draw_it {
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
        }
    }

    fn draw_end_point(&self, cr: &Context, draw_it: bool) {
        let end = self.segment.b();
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();
        let radius = self.radius();

        cr.new_sub_path();
        cr.arc(end.x, end.y,
               radius,
               0.0, (360_f64).to_radians());

        if draw_it {
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
        }
    }

    fn draw_go_direction(&self, cr: &Context, draw_it: bool) {
        let go_dir = &self.go_dir;
        let start = self.segment.a();
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();
        let radius = self.radius();

        // draw @go_dir and @arrive_dir circle
        cr.new_sub_path();
        cr.arc(start.x + go_dir.x, start.y + go_dir.y,
               radius,
               0.0, (360_f64).to_radians());

        if draw_it {
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
        }
    }

    fn draw_arrive_direction(&self, cr: &Context, draw_it: bool) {
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let fill_color = self.fill_color();
        let stroke_color = self.stroke_color();
        let radius = self.radius();

        cr.new_sub_path();
        cr.arc(end.x + arrive_dir.x, end.y + arrive_dir.y,
               radius,
               0.0, (360_f64).to_radians());

        if draw_it {
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
        }
    }

    fn draw_helper_shapes(&self, cr: &Context) {
        cr.save();
        let start = self.segment.a();
        let go_dir = &self.go_dir;
        let arrive_dir = &self.arrive_dir;
        let end = self.segment.b();
        let stroke_color = self.color.color.clone();

        if self.curve_like {
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
        }
        cr.restore();
    }

    fn draw_controllers(&self, cr: &Context) {
        cr.save();
        self.draw_start_point(cr, true);
        cr.restore();
        cr.save();
        self.draw_end_point(cr, true);
        cr.restore();
        cr.save();
        self.draw_go_direction(cr, true);
        cr.restore();
        cr.save();
        self.draw_arrive_direction(cr, true);
        cr.restore();
    }
}

impl ShapeTrait for LineArrow {}

impl Draw for LineArrow {
    fn draw(&self, cr: &Context) {
        if !self.visible || self.color.alpha == 0.0 {
            return;
        }

        self.draw_body(cr, true);
        if self.selected {
            self.draw_helper_shapes(cr);
            self.draw_controllers(cr);
        }

        // draw children if there are any.
        for child in self.children.iter() {
            child.draw(cr);
        }
    }
}

impl Name for LineArrow {
    fn name(&self) -> &String {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
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

impl Visible for LineArrow {
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

impl Container for LineArrow {
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

// TODO: override default methods
impl Event for LineArrow {
    fn button_press(&mut self, event: &EventButton) -> bool {
        if self.lock || !self.visible {
            return false;
        }

        if event.get_event_type() == EventType::ButtonPress {
            let position = event.get_position();
            let position = Point::new(position.0, position.1);
            let controller = self.select_controller(&position);
            match controller {
                None => {
                    self.selected_controller = None;
                    self.selected = false;
                    return false;
                },
                Some(val) => {
                    self.selected_controller = Some((val, position));
                    self.selected = true;
                    return true;
                },
            }
        }
        false
    }
}
