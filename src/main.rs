use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use wgpack::halley::{
    assets::unpack::unpack_halley_pk, pack_asset, read_pack,
    versions::common::hsave::load_save_data, write_pack, PackVersion,
};

//static SECRET_X: &str = "+Ohzep4z06NuKguNbFRz3w==";
static SECRET: &str = "K09oemVwNHowNk51S2d1Tg==";

// #[derive(Clone, Debug)]
// enum Games {
//     Wargroove(String),
//     Wargroove2(String),
// }
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Unpack {
        #[arg(short = 'p', long)]
        pack_version: PackVersion,

        #[arg(short = 'i', long)]
        asset: PathBuf,

        #[arg(short = 'o', long)]
        out_dir: PathBuf,

        #[arg(short = 's', long)]
        secret: Option<String>,
    },
    Repack {
        #[arg(short = 'p', long)]
        pack_version: PackVersion,

        #[arg(short = 'i', long)]
        asset: PathBuf,

        #[arg(short = 'o', long)]
        out_file: PathBuf,

        #[arg(short = 's', long)]
        secret: Option<String>,
    },
    Pack {
        #[arg(short = 'p', long)]
        pack_version: PackVersion,

        #[arg(short = 'i', long)]
        pack_dir: PathBuf,

        #[arg(short = 'o', long)]
        out_file: PathBuf,

        #[arg(short = 's', long)]
        secret: Option<String>,
    },
    ReadSave {
        #[arg(short = 'i', long)]
        save_file: PathBuf,

        #[arg(short = 'o', long)]
        out_file: Option<PathBuf>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Unpack {
            asset,
            out_dir,
            pack_version,
            secret,
        } => {
            let pack = read_pack(&asset, pack_version, secret.as_deref());
            unpack_halley_pk(&*pack, Path::new(&out_dir)).unwrap();
        }
        Commands::Repack {
            asset,
            out_file,
            pack_version,
            secret,
        } => {
            let pack = read_pack(&asset, pack_version, secret.as_deref());
            write_pack(pack, &out_file)
        }
        Commands::Pack {
            pack_dir,
            out_file,
            pack_version,
            secret,
        } => {
            let pack = pack_asset(&pack_dir, pack_version);
            write_pack(pack, &out_file)
        }
        Commands::ReadSave {
            save_file,
            out_file,
        } => {
            let data = load_save_data(&save_file, Some(SECRET));
            println!(
                "save data -> {:x?}",
                &data[0..std::cmp::min(100, data.len())]
            );
        }
    };
}
