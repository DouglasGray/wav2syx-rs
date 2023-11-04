use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use log::{error, info};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory which contains WAVs to convert
    #[arg(short, long)]
    src_dir: String,
    /// Directory to place SYX files in. Will retain the same
    /// structure as that of `src_dir.
    #[arg(short, long)]
    dst_dir: String,
}

#[derive(Debug)]
struct WavToConvert<'a> {
    src_root: &'a Path,
    dst_root: &'a Path,
    path: PathBuf,
}

fn main() -> Result<(), eyre::Report> {
    env_logger::init();

    let args = Args::parse();

    let src_dir = Path::new(&args.src_dir);
    let dst_dir = Path::new(&args.dst_dir);

    let mut wavs_to_convert = Vec::new();

    for entry in WalkDir::new(src_dir) {
        match entry {
            Ok(e) => {
                let path = e.path();

                if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                    if ext == "wav" {
                        wavs_to_convert.push(WavToConvert {
                            src_root: src_dir,
                            dst_root: dst_dir,
                            path: path
                                .strip_prefix(src_dir)
                                .expect("src_dir is guaranteed to prefix path")
                                .into(),
                        });
                    }
                }
            }
            Err(e) => {
                error!("error retrieving path: {}", e);
            }
        }
    }

    for w in wavs_to_convert {
        let src_wav: PathBuf = vec![w.src_root, w.path.as_path()].iter().collect();
        let dst_sds: PathBuf = vec![w.dst_root, w.path.as_path().with_extension("sds").as_path()]
            .iter()
            .collect();

        if let Some(p) = dst_sds.parent() {
            if let Err(e) = fs::create_dir_all(p) {
                error!(
                    "failed to create directory at {:?}: {}, skipping converting {:?}",
                    dst_sds.parent(),
                    e,
                    src_wav
                );
                continue;
            }
        }

        if let Err(e) = Command::new("sox")
            .arg(&src_wav)
            .arg("-r 44100")
            .arg("-c 1")
            .arg(&dst_sds)
            .output()
        {
            error!(
                "failed sox conversion {:?} => {:?}: {}",
                src_wav, dst_sds, e
            );
        } else {
            info!("converted {:?} => {:?}", src_wav, dst_sds);

            let dst_syx = dst_sds.with_extension("syx");

            if let Err(e) = fs::rename(&dst_sds, &dst_syx) {
                error!(
                    "failed to create .syx file {:?} by renaming its .sds counterpart: {}",
                    dst_syx, e
                );
            }
        }
    }

    Ok(())
}
