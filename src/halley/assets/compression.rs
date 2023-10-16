use crate::halley::versions::common::primitives::{h_i32, h_u64};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use nom::{
    bytes::complete::{tag, take},
    sequence::tuple,
};
use std::io::Read;

pub fn decompress(data: &[u8], compression: &str) -> Vec<u8> {
    match compression {
        "deflate" => {
            let (deflated_data, length) = h_u64(data).unwrap();

            let mut inflated_data = vec![0; length as usize];
            ZlibDecoder::new(deflated_data)
                .read_exact(&mut inflated_data)
                .expect("Uncompressed file does not match expected length!");
            inflated_data
        }
        "lz4" => {
            //println!("LZ4 data: {:x?}", &raw_data[0..min(200, raw_data.len())]);
            let (deflated_data, (_, size, _header)) =
                tuple((tag(b"LZ4\0"), h_i32, take(0 as usize)))(data).unwrap();

            lz4::block::decompress(deflated_data, Some(size))
                .expect("Could not decompress LZ4 data!")
        }
        _ => {
            println!("Unknown compression type: {}", compression);
            data.to_vec()
        }
    }
}

pub fn compress(data: &[u8], compression: &str) -> Vec<u8> {
    match compression {
        "deflate" => {
            let mut encoded = Vec::new();
            ZlibEncoder::new(data, flate2::Compression::default())
                .read_to_end(&mut encoded)
                .unwrap();
            encoded
        }
        "lz4" => lz4::block::compress(data, None, false).expect("Could not compress LZ4 data!"),
        _ => {
            println!("Unknown compression type: {}", compression);
            data.to_vec()
        }
    }
}
