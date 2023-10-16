mod halley;
mod steam;

//use halley::pack::write_halley_pk;
use crate::halley::{
    assets::unpack::unpack_halley_pk,
    versions::{v2020::hpk::halley_pack_v2020_parse, v2023::hpk::halley_pack_v2023_parse},
};
use clap::{Parser, Subcommand};
use cookie_factory::WriteContext;
use halley::{
    assets::unpack::pack_halley_pk,
    versions::{common::hpk::HalleyPack, v2020::hpk::HpkSectionV2020, v2023::hpk::HpkSectionV2023},
};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    vec,
};
use steam::find_wargroove_assets_folder;

//static SECRET: &str = "+Ohzep4z06NuKguNbFRz3w==";
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
}

fn main() {
    let args = Args::parse();
    //println!("{:?}", args);

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
    };

    return;
    //println!("Wargroove folder: {:?}", find_wargroove_assets_folder(None));
}

fn unpack_wg(filename: String, game: Option<String>) -> Box<dyn HalleyPack> {
    let path = Path::new(&filename);

    let data = fs::read(&path).unwrap();

    let pack = if filename.contains("Wargroove 2/") || game == Some("wg2".to_string()) {
        halley_pack_v2023_parse(&data, SECRET).unwrap().1
    } else if filename.contains("Wargroove/") || game == Some("wg".to_string()) {
        halley_pack_v2020_parse(&data, SECRET).unwrap().1
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
