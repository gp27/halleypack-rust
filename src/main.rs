use clap::{Parser, Subcommand};
use cookie_factory::WriteContext;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    vec,
};
use steam::find_wargroove_assets_folder;
use wgpack::halley::{
    assets::unpack::{pack_halley_pk, unpack_halley_pk},
    versions::{
        common::{hpk::HalleyPack, hsave::load_save_data},
        v2020::hpk::{HalleyPackV2020, HpkSectionV2020},
        v2023::hpk::{HalleyPackV2023, HpkSectionV2023},
    },
};

static SECRET_X: &str = "+Ohzep4z06NuKguNbFRz3w==";
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
        #[arg(short = 'g', long)]
        game: Option<String>,

        #[arg(short = 'i', long)]
        asset: String,

        #[arg(short = 'o', long)]
        out_dir: String,
    },
    Repack {
        #[arg(short = 'g', long)]
        game: Option<String>,

        #[arg(short = 'i', long)]
        asset: String,

        #[arg(short = 'o', long)]
        out_file: String,
    },
    Pack {
        #[arg(short = 'g', long)]
        game: Option<String>,

        #[arg(short = 'i', long)]
        pack_dir: String,

        #[arg(short = 'o', long)]
        out_file: String,
    },
    ReadSave {
        #[arg(short = 'i', long)]
        save_file: String,

        #[arg(short = 'o', long)]
        out_file: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Unpack {
            asset,
            out_dir,
            game,
        } => {
            let pack = unpack_wg(asset, game);
            unpack_halley_pk(&*pack, Path::new(&out_dir)).unwrap();
        }
        Commands::Repack {
            asset,
            out_file,
            game,
        } => {
            let pack = unpack_wg(asset, game);
            write_pack(pack, out_file)
        }
        Commands::Pack {
            pack_dir,
            out_file,
            game,
        } => {
            let pack = pack_wg(pack_dir, game);
            write_pack(pack, out_file)
        }
        Commands::ReadSave {
            save_file,
            out_file,
        } => {
            let data = load_save_data(save_file.as_ref(), Some(SECRET));
            println!(
                "save data -> {:x?}",
                &data[0..std::cmp::min(100, data.len())]
            );
        }
    };
}

fn unpack_wg(filename: String, game: Option<String>) -> Box<dyn HalleyPack> {
    let path = Path::new(&filename);

    let data = fs::read(&path).unwrap();

    let pack = if filename.contains("Wargroove 2/") || game == Some("wg2".to_string()) {
        HalleyPackV2023::load(path, SECRET).unwrap()
    } else if filename.contains("Wargroove/") || game == Some("wg".to_string()) {
        HalleyPackV2020::load(path, SECRET).unwrap()
    } else {
        panic!("Unknown game");
    };

    pack
}

fn pack_wg(dirname: String, game: Option<String>) -> Box<dyn HalleyPack> {
    let path = Path::new(&dirname);

    let pack = if game == Some("wg2".to_string()) {
        pack_halley_pk::<HpkSectionV2023>(path).unwrap()
    } else if game == Some("wg".to_string()) {
        pack_halley_pk::<HpkSectionV2020>(path).unwrap()
    } else {
        panic!("Unknown game");
    };
    pack
}

fn write_pack(pack: Box<dyn HalleyPack>, filename: String) {
    let mut writer = BufWriter::new(fs::File::create(filename).unwrap());
    let buf = vec![];
    let res = pack.write()(WriteContext {
        write: buf,
        position: 0,
    })
    .unwrap();
    writer.write_all(&res.write).unwrap();
}

// find steam path (or ask for path)
// save path to config file
// find wargroove assets folder
// create mod_bckp folder under assets
// copy all files from assets to mod_bckp (check for each if file exists first)
