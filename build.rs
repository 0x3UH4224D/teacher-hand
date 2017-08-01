//
// build.rs
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

use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let package_name = "teacherhand";
    let package_version = "0.1.0";
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // generating pot file
    println!("[build.rs:POT] Start generating pot file from the porject");

    let po_dir = format!("{}/data/po", root_dir);
    let po_linguas_file = format!("{}/LINGUAS", po_dir);
    let po_potfiles_file = format!("{}/POTFILES.in", po_dir);
    let comments_tag = "TRANSLATORS:";
    let from_code = "UTF-8";
    let copyright = "'Copyright (C) 2017 Muhannad Alrusayni'";

    let output = Command::new("xgettext")
                         .arg(&format!("--output={}.pot", package_name))
                         .arg(&format!("--output-dir={}", po_dir))
                         .arg(&format!("--default-domain={}", package_name))
                         .arg(&format!("--add-comments={}", comments_tag))
                         .arg(&format!("--from-code={}", from_code))
                         .arg(&format!("--copyright-holder={}", copyright))
                         .arg(&format!("--files-from={}", po_potfiles_file))
                         .arg(&format!("--directory={}", root_dir))
                         .arg(&format!("--package-name={}", package_name))
                         .arg(&format!("--package-version={}", package_version))
                         .output()
                         .expect("failed to execute xgettext");

    if (output.status.success()) {
        println!("[build.rs:POT] pot file have been generated")
    } else {
        println!("[build.rs:POT] xgettext exit with: {}", output.status);
        println!("[build.rs:POT] xgettext stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("[build.rs:POT] xgettext stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
