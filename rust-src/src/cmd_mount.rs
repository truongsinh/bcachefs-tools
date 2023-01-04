use bch_bindgen::{error, info};
use clap::Parser;
use colored::Colorize;
use atty::Stream;
use uuid::Uuid;
use crate::filesystem;
use crate::key;
use crate::key::KeyLoc;

fn parse_fstab_uuid(uuid_raw: &str) -> Result<Uuid, uuid::Error> {
    let mut uuid = String::from(uuid_raw);
    if uuid.starts_with("UUID=") {
        uuid = uuid.replacen("UUID=", "", 1);
    }
    return Uuid::parse_str(&uuid);
}

fn stdout_isatty() -> &'static str {
    if atty::is(Stream::Stdout) {
        "true"
    } else {
        "false"
    }
}

/// Mount a bcachefs filesystem by its UUID.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Where the password would be loaded from.
    ///
    /// Possible values are:
    /// "fail" - don't ask for password, fail if filesystem is encrypted;
    /// "wait" - wait for password to become available before mounting;
    /// "ask" -  prompt the user for password;
    #[arg(short, long, default_value = "", verbatim_doc_comment)]
    pub key_location: KeyLoc,

    /// External UUID of the bcachefs filesystem
    ///
    /// Accepts the UUID as is or as fstab style UUID=<UUID>
    #[arg(value_parser = parse_fstab_uuid)]
    pub uuid: uuid::Uuid,

    /// Where the filesystem should be mounted. If not set, then the filesystem
    /// won't actually be mounted. But all steps preceeding mounting the
    /// filesystem (e.g. asking for passphrase) will still be performed.
    pub mountpoint: Option<std::path::PathBuf>,

    /// Mount options
    #[arg(short, default_value = "")]
    pub options: String,

    /// Force color on/off. Default: autodetect tty
    #[arg(short, long, action = clap::ArgAction::Set, default_value=stdout_isatty())]
    pub colorize: bool,

    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub fn cmd_mount_inner(opt: Cli) -> anyhow::Result<()> {
    unsafe {
        libc::setvbuf(filesystem::stdout, std::ptr::null_mut(), libc::_IONBF, 0);
    }

    let fss = filesystem::probe_filesystems()?;
    let fs = fss
        .get(&opt.uuid)
        .ok_or_else(|| anyhow::anyhow!("filesystem was not found"))?;

    info!("found filesystem {}", fs);
    if fs.encrypted() {
        let key = opt
            .key_location
            .0
            .ok_or_else(|| anyhow::anyhow!("no keyoption specified for locked filesystem"))?;

        key::prepare_key(&fs, key)?;
    }

    let mountpoint = opt
        .mountpoint
        .ok_or_else(|| anyhow::anyhow!("mountpoint option was not specified"))?;

    fs.mount(&mountpoint, &opt.options)?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn cmd_mount() {
    let opt = Cli::parse();
    bch_bindgen::log::set_verbose_level(opt.verbose + bch_bindgen::log::ERROR);
    colored::control::set_override(opt.colorize);
    if let Err(e) = cmd_mount_inner(opt) {
        error!("Fatal error: {}", e);
    }
}
