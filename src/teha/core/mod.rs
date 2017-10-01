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

use gtk;
use gdk::{EventMotion, EventButton, EventKey};
use cairo;

use self::draw_objects::Page;
use common::types::Size;

pub struct Document {
    pages: Vec<Page>,
    active_page_index: usize,
    path: PathBuf,
    transparent: bool,
}

impl Document {
    pub fn new(pages_number: usize, path: PathBuf, size: Size<i32>, transparent: bool) -> Self {
        let mut pages: Vec<Page> = vec![];
        for _ in 0..pages_number {
            let mut page = Page::new();
            page.set_size(size.clone());
            pages.push(page);
        }

        Document {
            pages: pages,
            active_page_index: 0,
            path: path,
            transparent: transparent,
        }
    }

    pub fn get_pages(&self) -> &Vec<Page> {
        &self.pages
    }

    pub fn get_mut_pages(&mut self) -> &mut Vec<Page> {
        &mut self.pages
    }

    pub fn set_pages(&mut self, pages: Vec<Page>) {
        self.pages = pages;
    }

    pub fn get_active_page(&self) -> &Page {
        &self.pages[self.active_page_index]
    }

    pub fn get_mut_active_page(&mut self) -> &mut Page {
        &mut self.pages[self.active_page_index]
    }

    pub fn get_active_page_index(&self) -> usize {
        self.active_page_index
    }

    pub fn set_active_page_index(&mut self, active_page_index: usize) {
        self.active_page_index = active_page_index;
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_mut_path(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn get_transparent(&self) -> bool {
        self.transparent
    }

    pub fn set_transparent(&mut self, transparent: bool) {
        self.transparent = transparent;
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
        self.pages[self.active_page_index].draw(cr);
    }

    pub fn motion_notify(&mut self, event: &EventMotion) -> bool {
        self.pages[self.active_page_index].motion_notify(event)
    }

    pub fn button_press(
        &mut self, event: &EventButton, options_widget: &gtk::Notebook
    ) -> bool {
        self.pages[self.active_page_index].button_press(event, options_widget)
    }

    pub fn button_release(&mut self, event: &EventButton) -> bool {
        self.pages[self.active_page_index].button_release(event)
    }

    pub fn key_press(&mut self, event: &EventKey) -> bool {
        self.pages[self.active_page_index].key_press(event)
    }

    pub fn key_release(&mut self, event: &EventKey) -> bool {
        self.pages[self.active_page_index].key_release(event)
    }

    pub fn name(&self) -> &str {
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

    pub fn set_name(&mut self, name: &str) {
        self.path.set_file_name(name);
    }
}

impl Default for Document {
    fn default() -> Self {
        Document {
            pages: vec![Page::new()],
            active_page_index: 0,
            path: PathBuf::new(),
            transparent: true,
        }
    }
}
