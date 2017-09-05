//
// mod.rs
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

pub mod draw_objects;
pub mod context;

use std::path::PathBuf;

use gdk::{EventMotion, EventButton};
use cairo;

use self::draw_objects::{Name, Page};
use common::types::Size;

pub struct Document {
    pub pages: Vec<Page>,
    pub page_number: usize,
    pub path: PathBuf,
    pub transparent: bool,
}

impl Document {
    pub fn new(pages_number: usize, path: PathBuf, size: Size<i32>, transparent: bool) -> Self {
        let mut pages: Vec<Page> = vec![];
        for _ in 0..pages_number {
            pages.push(Page::default());
            pages.last_mut()
                .unwrap()
                .size.resize(size.width, size.height);
        }

        Document {
            pages: pages,
            page_number: 0,
            path: path,
            transparent: transparent,
        }
    }

    // pub fn save(&self) -> io::Result<()> {
        // TODO write save method
    //     Ok(())
    // }

    // pub fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        // TODO write save_as method
    //     Ok(())
    // }

    pub fn draw(&self, cr: &cairo::Context) {
        self.pages[self.page_number].draw(cr);
    }

    pub fn motion_notify(&mut self, event: &EventMotion) -> bool {
        self.pages[self.page_number].motion_notify(event)
    }

    pub fn button_press(&mut self, event: &EventButton) -> bool {
        self.pages[self.page_number].button_press(event)
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        self.pages[self.page_number].button_release(event)
    }
}

impl Default for Document {
    fn default() -> Self {
        Document {
            pages: vec![Page::default()],
            page_number: 0,
            path: PathBuf::new(),
            transparent: true,
        }
    }
}

impl Name for Document {
    fn name(&self) -> &str {
        if self.path.is_file() {
            let file_name = match self.path.file_name() {
                Some(val) => val,
                None => return "",
            };
            let file_name = match file_name.to_str() {
                Some(val) => val,
                None => return "",
            };
            file_name
        } else {
            return "";
        }
    }

    fn set_name(&mut self, name: &str) {
        self.path.set_file_name(name);
    }
}
