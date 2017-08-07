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
use gdk;

use euclid::{self, Point2D};

use palette;

type Point = euclid::Point2D<f64>;
type Vector = euclid::Vector2D<f64>;
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

// TODO: not finished yet.
pub trait DocumentTrait: Draw + Name {

}

pub struct Document {

}
