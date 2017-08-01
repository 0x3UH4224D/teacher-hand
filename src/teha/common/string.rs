//
// path.rs
//
// Copyright (C) 2017 Muhannad Alrusayni <Muhannad.Alrusayni@gmail.com>
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

use error::Error;

use gettextrs::*;

pub fn is_valid_filename(filename: &str) -> Result<(), String> {
    if filename.len() == 0 {
        return Err(gettext("File name cannot be empty"));
    }

    if filename.len() > 63 {
        return Err(gettext("File name is longer than 63 letter"));
    }

    if filename.contains("/") {
        return Err(gettext("File name cannot contain “/” letter."));
    }

    Ok(())
}

pub fn is_valid_size() -> Result<(), String> {
    Ok(())
}
