use crate::i18n::LocalText;
use crate::CONFIG;
use crate::{status, CONFIGURE_DATADIR};
use colored::Colorize;
use regex::Regex;
use std::io::prelude::*;
use std::{error, ffi::OsString, io, result};
use subprocess::{Exec, ExitStatus, Redirection};

type Result<T> = result::Result<T, Box<dyn error::Error>>;

// FTL: help-subcommand-make
/// Build specified target(s)
pub fn run(target: Vec<String>) -> Result<()> {
    crate::header("make-header");
    let mut makefiles: Vec<OsString> = Vec::new();
    makefiles.push(OsString::from("-f"));
    makefiles.push(OsString::from(format!(
        "{}{}",
        CONFIGURE_DATADIR, "rules/fontship.mk"
    )));
    let rules = status::get_rules()?;
    for rule in rules {
        makefiles.push(OsString::from("-f"));
        let p = rule.into_os_string();
        makefiles.push(p);
    }
    makefiles.push(OsString::from("-f"));
    makefiles.push(OsString::from(format!(
        "{}{}",
        CONFIGURE_DATADIR, "rules/rules.mk"
    )));
    let mut process = Exec::cmd("make").args(&makefiles).args(&target);
    // Start deprecating non-CLI usage
    process = process
        .env("FONTSHIP_CLI", "true")
        .env("FONTSHIPDIR", CONFIGURE_DATADIR)
        .env("GITNAME", status::get_gitname()?);
    if CONFIG.get_bool("debug")? {
        process = process.env("DEBUG", "true");
    };
    if CONFIG.get_bool("quiet")? {
        process = process.env("QUIET", "true");
    };
    if CONFIG.get_bool("verbose")? {
        process = process.env("VERBOSE", "true");
    };
    let repo = status::get_repo()?;
    let workdir = repo.workdir().unwrap();
    process = process.cwd(workdir);
    let process = process.stderr(Redirection::Merge).stdout(Redirection::Pipe);
    let mut popen = process.popen()?;
    let buf = io::BufReader::new(popen.stdout.as_mut().unwrap());
    let mut backlog: Vec<String> = Vec::new();
    let seps = Regex::new(r"").unwrap();
    let mut ret: u32 = 0;
    for line in buf.lines() {
        let text: &str = &line.unwrap();
        let fields: Vec<&str> = seps.splitn(text, 4).collect();
        match fields[0] {
            "FONTSHIP" => match fields[1] {
                "PRE" => report_start(fields[2]),
                "STDOUT" => {
                    if fields[2] == "_gha" {
                        println!("{}", fields[3]);
                    } else if CONFIG.get_bool("verbose")? {
                        report_line(fields[3]);
                    } else {
                        backlog.push(String::from(fields[3]));
                    }
                }
                "STDERR" => {
                    if CONFIG.get_bool("verbose")? {
                        report_line(fields[3]);
                    } else {
                        backlog.push(String::from(fields[3]));
                    }
                }
                "POST" => match fields[2] {
                    "0" => {
                        report_end(fields[3]);
                    }
                    val => {
                        report_fail(fields[3]);
                        ret = val.parse().unwrap_or(1);
                    }
                },
                _ => {
                    let errmsg = LocalText::new("make-error-unknown-code").fmt();
                    panic!(errmsg)
                }
            },
            _ => backlog.push(String::from(fields[0])),
        }
    }
    let status = popen.wait();
    match status {
        Ok(ExitStatus::Exited(int)) => {
            let foo = int + ret;
            match foo {
                0 => Ok(()),
                _ => {
                    if !CONFIG.get_bool("verbose")? {
                        dump_backlog(&backlog);
                    }
                    Err(Box::new(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        LocalText::new("make-error-failed").fmt(),
                    )))
                }
            }
        }
        _ => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            LocalText::new("make-error-process").fmt(),
        ))),
    }
}

fn dump_backlog(backlog: &Vec<String>) {
    let start = LocalText::new("make-backlog-start").fmt();
    eprintln!("{} {}", "┖┄".cyan(), start);
    for line in backlog.iter() {
        eprintln!("{}", line);
    }
    let end = LocalText::new("make-backlog-end").fmt();
    eprintln!("{} {}", "┎┄".cyan(), end);
}

fn report_line(line: &str) {
    eprintln!("{} {}", "┠╎".cyan(), line.dimmed());
}

fn report_start(target: &str) {
    let text = LocalText::new("make-report-start")
        .arg("target", target.white().bold())
        .fmt();
    eprintln!("{} {}", "┠┄".cyan(), text.yellow());
}

fn report_end(target: &str) {
    let text = LocalText::new("make-report-end")
        .arg("target", target.white().bold())
        .fmt();
    eprintln!("{} {}", "┠┄".cyan(), text.green());
}

fn report_fail(target: &str) {
    let text = LocalText::new("make-report-fail")
        .arg("target", target.white().bold())
        .fmt();
    eprintln!("{} {}", "┠┄".cyan(), text.red());
}
