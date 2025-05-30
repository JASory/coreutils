// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{Arg, ArgAction, Command};
use libc::mkfifo;
use std::ffi::CString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use uucore::display::Quotable;
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, help_about, help_usage, show};

static USAGE: &str = help_usage!("mkfifo.md");
static ABOUT: &str = help_about!("mkfifo.md");

mod options {
    pub static MODE: &str = "mode";
    pub static SE_LINUX_SECURITY_CONTEXT: &str = "Z";
    pub static CONTEXT: &str = "context";
    pub static FIFO: &str = "fifo";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    if matches.contains_id(options::CONTEXT) {
        return Err(USimpleError::new(1, "--context is not implemented"));
    }
    if matches.get_flag(options::SE_LINUX_SECURITY_CONTEXT) {
        return Err(USimpleError::new(1, "-Z is not implemented"));
    }

    let mode = match matches.get_one::<String>(options::MODE) {
        // if mode is passed, ignore umask
        Some(m) => match usize::from_str_radix(m, 8) {
            Ok(m) => m,
            Err(e) => return Err(USimpleError::new(1, format!("invalid mode: {e}"))),
        },
        // Default value + umask if present
        None => 0o666 & !(uucore::mode::get_umask() as usize),
    };

    let fifos: Vec<String> = match matches.get_many::<String>(options::FIFO) {
        Some(v) => v.cloned().collect(),
        None => return Err(USimpleError::new(1, "missing operand")),
    };

    for f in fifos {
        let err = unsafe {
            let name = CString::new(f.as_bytes()).unwrap();
            mkfifo(name.as_ptr(), 0o666)
        };
        if err == -1 {
            show!(USimpleError::new(
                1,
                format!("cannot create fifo {}: File exists", f.quote()),
            ));
        }

        // Explicitly set the permissions to ignore umask
        if let Err(e) = fs::set_permissions(&f, fs::Permissions::from_mode(mode as u32)) {
            return Err(USimpleError::new(
                1,
                format!("cannot set permissions on {}: {e}", f.quote()),
            ));
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(uucore::crate_version!())
        .override_usage(format_usage(USAGE))
        .about(ABOUT)
        .infer_long_args(true)
        .arg(
            Arg::new(options::MODE)
                .short('m')
                .long(options::MODE)
                .help("file permissions for the fifo")
                .value_name("MODE"),
        )
        .arg(
            Arg::new(options::SE_LINUX_SECURITY_CONTEXT)
                .short('Z')
                .help("set the SELinux security context to default type")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(options::CONTEXT)
                .long(options::CONTEXT)
                .value_name("CTX")
                .help(
                    "like -Z, or if CTX is specified then set the SELinux \
                    or SMACK security context to CTX",
                ),
        )
        .arg(
            Arg::new(options::FIFO)
                .hide(true)
                .action(ArgAction::Append)
                .value_hint(clap::ValueHint::AnyPath),
        )
}
