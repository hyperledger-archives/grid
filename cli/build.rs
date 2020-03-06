// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::process::Command;

const FORCE_PANDOC: &str = "SPLINTER_FORCE_PANDOC";
const PATH: &str = "PATH";

/// This build script will take the markdown files in the /man directory and convert them to
/// man pages stored in packaging/man. This build script will check if pandoc is installed locally
/// and skip generating the manpages if it is not. If the build should fail if man pages cannot be
/// generated set environment variable SPLINTER_FORCE_PANDOC=true
fn main() -> Result<(), BuildError> {
    let paths = env::var(PATH)
        .map_err(|_| BuildError("Unable to read PATH environment variable".into()))?;
    let mut pandoc_exist = false;
    for path in paths.split(':') {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                // skip a directory in the path that cannot be read.
                println!("Unable to read path entry {}: {}", path, err);
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    // skip an entry in the path that cannot be read.
                    println!("Unable to read entry in {}: {}", path, err);
                    continue;
                }
            };

            let path = entry.path();
            if path.ends_with("pandoc") {
                pandoc_exist = true;
                break;
            }
        }
    }

    if !pandoc_exist {
        if let Ok(var) = env::var(FORCE_PANDOC) {
            let map_to_build_err = move |_| {
                BuildError("Unable to read SPLINTER_FORCE_PANDOC environment variable".into())
            };
            if var.parse().map_err(map_to_build_err)? {
                return Err(BuildError(
                    "Cannot generate man pages, pandoc is not installed".into(),
                ));
            }
        } else {
            println!("Skip generating man pages");
            return Ok(());
        }
    }

    let entries = match fs::read_dir("man/") {
        Ok(entries) => {
            match entries
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()
            {
                Ok(entries) => entries,
                Err(err) => {
                    return Err(BuildError(format!(
                        "Unable to retrieve entries for man pages: {}",
                        err
                    )))
                }
            }
        }
        Err(err) => {
            return Err(BuildError(format!(
                "Unable to read man pages directory: {}",
                err
            )))
        }
    };

    println!("Markdown files found {:?}", entries);

    for entry in entries {
        // This conversion would only fail if the filename is not valid UTF-8
        let markdown = &entry
            .to_str()
            .ok_or_else(|| BuildError("Cannot get markdown file path".into()))?
            .to_string();

        let file = entry
            .file_stem()
            .ok_or_else(|| BuildError("Cannot get markdown file name".into()))?;
        let manpage = &format!(
            "packaging/man/{}",
            file.to_str()
                .ok_or_else(|| BuildError("Cannot get markdown file name".into()))?
        );

        if markdown.ends_with(".md") {
            match Command::new("pandoc")
                .args(&["--standalone", "--to", "man", &markdown, "-o", &manpage])
                .status()
            {
                Ok(status) => {
                    if status.success() {
                        println!("Generated man page: {}", manpage);
                    } else {
                        println!("Unable to generate man page {} status {}", manpage, status);
                    }
                }
                Err(err) => {
                    return Err(BuildError(format!(
                        "Unable to generate man page: {} {}",
                        manpage, err
                    )))
                }
            }
        }
    }
    Ok(())
}

pub struct BuildError(String);

impl Error for BuildError {}

// This is the output that will be used for print errors returned from main
impl fmt::Debug for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}
