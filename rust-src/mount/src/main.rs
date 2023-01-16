use bcachefs_mount::Cli;
use bch_bindgen::{error, info};
use clap::Parser;
use colored::Colorize;

fn main() {
    let opt = Cli::parse();
    bch_bindgen::log::set_verbose_level(opt.verbose + bch_bindgen::log::ERROR);
    colored::control::set_override(opt.colorize);
    if let Err(e) = crate::main_inner(opt) {
        error!("Fatal error: {}", e);
    }
}

pub fn main_inner(opt: Cli) -> anyhow::Result<()> {
    use bcachefs_mount::{filesystem, key};
    unsafe {
        libc::setvbuf(filesystem::stdout, std::ptr::null_mut(), libc::_IONBF, 0);
        // libc::fflush(filesystem::stdout);
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

#[cfg(test)]
mod test {
    // use insta::assert_debug_snapshot;
    // #[test]
    // fn snapshot_testing() {
    //  insta::assert_debug_snapshot!();
    // }
}
