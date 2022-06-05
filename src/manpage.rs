// Copyright 2022 Manos Pitsidianakis <epilys@nessuent.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Default, Debug, Clone)]
pub struct TakesValue {
    pub kind: Option<&'static str>,
    pub multiple: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Flag {
    long: Option<String>,
    short: Option<String>,
    args: Option<TakesValue>,
    doc: Option<String>,
}

impl Flag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn long(&mut self, val: String) -> &mut Self {
        self.long = Some(val.trim_matches('"').to_string());
        self
    }

    pub fn short(&mut self, val: String) -> &mut Self {
        self.short = Some(val.trim_matches('"').to_string());
        self
    }

    pub fn doc(&mut self, val: String) -> &mut Self {
        self.doc = Some(val.trim_matches('"').to_string());
        self
    }

    pub fn args(&mut self, val: TakesValue) -> &mut Self {
        self.args = Some(val);
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct Subcommand {
    name: String,
    args: Option<TakesValue>,
    flags: Vec<Flag>,
    doc: Option<String>,
}

impl Subcommand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    pub fn doc(&mut self, val: String) -> &mut Self {
        self.doc = Some(val.trim_matches('"').to_string());
        self
    }

    pub fn flags(&mut self, val: Vec<Flag>) -> &mut Self {
        self.flags = val;
        self
    }
}

#[derive(Default, Clone, Debug)]
pub struct Manpage {
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub header_path: Option<PathBuf>,
    pub footer_path: Option<PathBuf>,
    pub flags: Vec<Flag>,
    pub subcommands: Vec<Subcommand>,
    short_flags: HashMap<Option<String>, String>,
    long_flags: HashMap<Option<String>, String>,
}

impl Manpage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&mut self, val: String) -> &mut Self {
        self.name = val.trim_matches('"').to_string();
        self
    }

    pub fn path(&mut self, val: PathBuf) -> &mut Self {
        self.path = Some(val);
        self
    }

    pub fn header_path(&mut self, val: PathBuf) -> &mut Self {
        self.header_path = Some(val);
        self
    }

    pub fn footer_path(&mut self, val: PathBuf) -> &mut Self {
        self.footer_path = Some(val);
        self
    }

    pub fn description(&mut self, val: Option<String>) -> &mut Self {
        self.description = val.map(|v| v.trim_matches('"').to_string());
        self
    }

    pub fn author(&mut self, val: Option<String>) -> &mut Self {
        self.author = val.map(|v| v.trim_matches('"').to_string());
        self
    }

    pub fn version(&mut self, val: Option<String>) -> &mut Self {
        self.version = val.map(|v| v.trim_matches('"').to_string());
        self
    }

    pub fn long_description(&mut self, val: Option<String>) -> &mut Self {
        self.long_description = val.map(|v| v.trim_matches('"').to_string());
        self
    }

    pub fn push_short_flag(&mut self, owner: Option<String>, ident: String) -> &mut Self {
        self.short_flags.insert(owner, ident);
        self
    }

    pub fn push_long_flag(&mut self, owner: Option<String>, ident: String) -> &mut Self {
        self.long_flags.insert(owner, ident);
        self
    }

    pub fn push_subcommand(&mut self, mut cmd: Self) {
        cmd.path = None;
        let name = std::mem::replace(&mut cmd.name, String::new());
        let description = cmd.description.take();
        let long_description = cmd.long_description.take();
        let flags = std::mem::replace(&mut cmd.flags, vec![]);

        let mut val = Subcommand::new(name);
        if let Some(v) = description {
            val.doc(v);
        }
        if let Some(v) = long_description {
            val.doc(v);
        }
        val.flags(flags);
        self.subcommands.push(val);
    }
}

impl std::fmt::Display for Manpage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut synopsis = ".Nm\n".to_string();
        let mut flag_table = ".Bl -tag -width flag -offset indent\n".to_string();
        for Flag {
            long,
            short,
            args,
            doc,
        } in self.flags.iter()
        {
            let mut line = String::new();
            match (long, short) {
                (Some(l), Some(s)) if l == s => {
                    line.push_str(&format!(".Op Fl -{}", l));
                }
                (None, None) => continue,
                (Some(l), Some(s)) => {
                    line.push_str(&format!(".Op Fl -{} | -{}", l, s));
                }
                (None, Some(v)) | (Some(v), None) => {
                    line.push_str(&format!(".Op Fl -{}", v));
                }
            }
            match args {
                Some(TakesValue {
                    kind,
                    multiple: true,
                }) => {
                    line.push_str(&format!(
                        " Ar {} ...",
                        if let Some(v) = kind.as_ref() {
                            *v
                        } else {
                            long.as_ref()
                                .or(short.as_ref())
                                .map(String::as_str)
                                .unwrap_or("ARGUMENT")
                        }
                    ));
                }
                Some(TakesValue {
                    kind,
                    multiple: false,
                }) => {
                    line.push_str(&format!(
                        " Ar {}",
                        if let Some(v) = kind.as_ref() {
                            *v
                        } else {
                            long.as_ref()
                                .or(short.as_ref())
                                .map(String::as_str)
                                .unwrap_or("ARGUMENT")
                        }
                    ));
                }
                None => {}
            }
            line.push('\n');
            flag_table
                .extend(format!(".It {}\n", line.strip_prefix(".Op").unwrap().trim()).chars());
            if let Some(doc) = doc {
                let doc = doc.trim();
                let doc = doc.trim_matches('.');
                let doc = doc.trim_matches('"');
                let doc = doc.trim_matches('.');
                flag_table.extend(format!("{}.\n", doc.trim()).chars());
            }
            synopsis.extend(line.chars());
        }
        flag_table.push_str(".El\n");
        let mut subcommands = r#".Bl -tag -width Ds -compact -offset indent
"#
        .to_string();
        for cmd in self.subcommands.iter() {
            subcommands.extend(format!(".It Ic {}", cmd.name).chars());
            match cmd.args {
                Some(TakesValue {
                    kind,
                    multiple: true,
                }) => {
                    subcommands.push_str(&format!(
                        " Ar {} ...",
                        if let Some(v) = kind.as_ref() {
                            *v
                        } else {
                            "ARGUMENT"
                        }
                    ));
                }
                Some(TakesValue {
                    kind,
                    multiple: false,
                }) => {
                    subcommands.push_str(&format!(
                        " Ar {}",
                        if let Some(v) = kind.as_ref() {
                            *v
                        } else {
                            "ARGUMENT"
                        }
                    ));
                }
                None => {}
            }
            for Flag {
                long,
                short,
                args,
                doc,
            } in cmd.flags.iter()
            {
                let mut line = "\n".to_string();
                match (long, short) {
                    (Some(l), Some(s)) if l == s => {
                        line.push_str(&format!(".Fl -{}", l));
                    }
                    (None, None) => continue,
                    (Some(l), Some(s)) => {
                        line.push_str(&format!(".Fl -{} | -{}", l, s));
                    }
                    (None, Some(v)) | (Some(v), None) => {
                        line.push_str(&format!(".Fl -{}", v));
                    }
                }
                match args {
                    Some(TakesValue {
                        kind,
                        multiple: true,
                    }) => {
                        line.push_str(&format!(
                            " Ar {} ...",
                            if let Some(v) = kind.as_ref() {
                                *v
                            } else {
                                long.as_ref()
                                    .or(short.as_ref())
                                    .map(String::as_str)
                                    .unwrap_or("ARGUMENT")
                            }
                        ));
                    }
                    Some(TakesValue {
                        kind,
                        multiple: false,
                    }) => {
                        line.push_str(&format!(
                            " Ar {}",
                            if let Some(v) = kind.as_ref() {
                                *v
                            } else {
                                long.as_ref()
                                    .or(short.as_ref())
                                    .map(String::as_str)
                                    .unwrap_or("ARGUMENT")
                            }
                        ));
                    }
                    None => {}
                }
                line.push('\n');
                if let Some(doc) = doc {
                    let doc = doc.trim();
                    let doc = doc.trim_matches('.');
                    let doc = doc.trim_matches('"');
                    let doc = doc.trim_matches('.');
                    line.extend(format!("{}.\n", doc).chars());
                }
                if !line.trim().is_empty() {
                    subcommands.extend(line.chars());
                }
            }
            subcommands.push('\n');
            if let Some(doc) = &cmd.doc {
                let doc = doc.trim();
                let doc = doc.trim_matches('.');
                let doc = doc.trim_matches('"');
                let doc = doc.trim_matches('.');
                subcommands.extend(format!("{}.\n", doc).chars());
            }
        }
        subcommands.push_str(".El\n.Pp\n");
        write!(
            fmt,
            r#"{synopsis}{flag_br}{flag_table}{subcmd_br}{subcommands}
"#,
            synopsis = if self.flags.is_empty() {
                ""
            } else {
                synopsis.trim()
            },
            flag_br = if self.flags.is_empty() { "" } else { "\n" },
            flag_table = if self.flags.is_empty() {
                ""
            } else {
                flag_table.trim()
            },
            subcmd_br = if self.subcommands.is_empty() {
                ""
            } else {
                "\n"
            },
            subcommands = if self.subcommands.is_empty() {
                ""
            } else {
                subcommands.trim()
            },
        )
    }
}

impl Drop for Manpage {
    fn drop(&mut self) {
        macro_rules! write_to_file {
            ($path:expr, $string:expr) => {{
                let mut file = match File::create(&$path) {
                    Err(err) => {
                        eprintln!("couldn't create {}: {}", $path.display(), err);
                        return;
                    }
                    Ok(file) => file,
                };

                match file.write_all($string.as_bytes()) {
                    Err(err) => {
                        eprintln!("couldn't write to {}: {}", $path.display(), err);
                        return;
                    }
                    Ok(_) => {}
                }
            }};
        }

        if let Some(path) = self.path.take() {
            write_to_file!(path, format!("{}", self));
        }

        if let Some(path) = self.header_path.take() {
            let header_string = format!(
                r#".Dd $Mdocdate$
.Dt {uppercase_name} 1
.Os
.Sh NAME
.Nm {name}
.Nd {description}."#,
                uppercase_name = self.name.to_uppercase().trim_matches('"'),
                name = self.name.as_str().trim_matches('"'),
                description = self
                    .description
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .trim_end_matches('.'),
            );
            write_to_file!(path, header_string);
        }

        if let Some(path) = self.footer_path.take() {
            let footer_string = format!(
                ".Sh AUTHORS\n{authors}",
                authors = self
                    .author
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or_default()
                    .trim_matches('"'),
            );
            write_to_file!(path, footer_string);
        }
    }
}
