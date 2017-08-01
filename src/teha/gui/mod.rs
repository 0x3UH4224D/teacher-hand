//
// mod.rs
//
// Copyright (C) 2017 Muhannad Alrusayni <Muhannad.Alrusayni@gmail.com>
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

pub mod app;
pub mod main_window;

pub enum Tool {
    Controller,
    StyleCloner,
    Cropper,
    Eraser,
}

pub enum Object {
    MessageBox,
    ArrowLine,
    Line,
    Sticker,
    Highlighter,
    TextBox,
    BlurBox
}

pub enum CurrentTool {
    Tool(Tool),
    Object(Object),
}

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
