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

pub mod app;
pub mod main_window;
pub mod header_bar;
pub mod drawing_area;
pub mod toolbar;
pub mod controlbar;

#[derive(Copy, Clone)]
pub enum ViewMode {
    StartUp,
    CreatingWork,
    ImportingWork,
    Editing,
    Previewing,
    Preferences,
}

impl Into<&'static str> for ViewMode {
    fn into(self) -> &'static str {
        match self {
            ViewMode::StartUp => "StartUp",
            ViewMode::CreatingWork => "CreatingWork",
            ViewMode::ImportingWork => "ImportingWork",
            ViewMode::Editing => "Editing",
            ViewMode::Previewing => "Previewing",
            ViewMode::Preferences => "Preferences",
        }
    }
}
