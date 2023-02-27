use atty::Stream;
use bch_bindgen::error;
use bch_bindgen::bcachefs;
use bch_bindgen::fs::Fs;
use bch_bindgen::btree::BtreeTrans;
use bch_bindgen::btree::BtreeIter;
use clap::Parser;
use colored::Colorize;
use std::ffi::{CStr, OsStr, c_int, c_char};
use std::os::unix::ffi::OsStrExt;

fn list_keys(fs: &Fs, opt: Cli) -> anyhow::Result<()> {
    let trans = BtreeTrans::new(fs);
    let mut iter = BtreeIter::new(&trans, opt.btree, opt.start, 1 << 11);

    while let Some(k) = iter.peek_and_restart()? {
        unsafe {
            if (*k.k).p > opt.end {
                break;
            }
        }

        println!("{}", k.to_text(fs));

        iter.advance();
    }

    Ok(())
}

fn list_btree_formats(fs: &Fs, opt: Cli) -> anyhow::Result<()> {
    let trans = BtreeTrans::new(fs);

    Ok(())
}

fn list_btree_nodes(fs: &Fs, opt: Cli) -> anyhow::Result<()> {
    let trans = BtreeTrans::new(fs);

    Ok(())
}

fn list_nodes_ondisk(fs: &Fs, opt: Cli) -> anyhow::Result<()> {
    let trans = BtreeTrans::new(fs);

    Ok(())
}

fn list_nodes_keys(fs: &Fs, opt: Cli) -> anyhow::Result<()> {
    let trans = BtreeTrans::new(fs);

    Ok(())
}

#[derive(Clone, clap::ValueEnum)]
enum Mode {
    Keys,
    Formats,
    Nodes,
    NodesOndisk,
    NodesKeys,
}

#[derive(Parser)]
struct Cli {
    /// Btree to list from
    #[arg(short, long, default_value_t=bcachefs::btree_id::BTREE_ID_extents)]
    btree:      bcachefs::btree_id,

    /// Btree depth to descend to (0 == leaves)
    #[arg(short, long, default_value_t=0)]
    level:      u8,

    /// Start position to list from
    #[arg(short, long, default_value="POS_MIN")]
    start:      bcachefs::bpos,

    /// End position
    #[arg(short, long, default_value="SPOS_MAX")]
    end:        bcachefs::bpos,

    #[arg(short, long, default_value="keys")]
    mode:       Mode,

    /// Check (fsck) the filesystem first
    #[arg(short, long, default_value_t=false)]
    fsck:       bool,

    /// Force color on/off. Default: autodetect tty
    #[arg(short, long, action = clap::ArgAction::Set, default_value_t=atty::is(Stream::Stdout))]
    colorize:   bool,
   
    /// Verbose mode
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose:    u8,

    #[arg(required(true))]
    devices:    Vec<std::path::PathBuf>,
}

fn cmd_list_inner(opt: Cli) -> anyhow::Result<()> {
    let fs_opts: bcachefs::bch_opts = Default::default();

    let fs = Fs::open(&opt.devices, fs_opts)?;

    match opt.mode {
        Mode::Keys          => list_keys(&fs, opt),
        Mode::Formats       => list_btree_formats(&fs, opt),
        Mode::Nodes         => list_btree_nodes(&fs, opt),
        Mode::NodesOndisk   => list_nodes_ondisk(&fs, opt),
        Mode::NodesKeys     => list_nodes_keys(&fs, opt),
    }
}

#[no_mangle]
pub extern "C" fn cmd_rust_list(argc: c_int, argv: *const *const c_char) {
    let argv: Vec<_> = (0..argc)
        .map(|i| unsafe { CStr::from_ptr(*argv.add(i as usize)) })
        .map(|i| OsStr::from_bytes(i.to_bytes()))
        .collect();

    let opt = Cli::parse_from(argv);
    colored::control::set_override(opt.colorize);
    if let Err(e) = cmd_list_inner(opt) {
        error!("Fatal error: {}", e);
    }
}
