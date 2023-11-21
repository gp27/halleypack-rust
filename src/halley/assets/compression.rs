use crate::halley::versions::common::primitives::{h_i32, h_u64};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use nom::{
    bytes::complete::{tag, take},
    sequence::tuple,
};
use std::io::Read;

static LZ4_MAGIC: &[u8] = b"LZ4\0";

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
            let len = data.len() as u64;
            let mut encoded = len.to_le_bytes().to_vec();

            ZlibEncoder::new(data, flate2::Compression::default())
                .read_to_end(&mut encoded)
                .unwrap();
            encoded
        }
        "lz4" => {
            let bound = lz4::block::compress_bound(data.len()).unwrap();
            let prefix_len: usize = LZ4_MAGIC.len();
            let mut compressed = vec![0; prefix_len + bound];
            compressed.splice(0..prefix_len, LZ4_MAGIC.to_vec());
            let compressed_size =
                lz4::block::compress_to_buffer(data, None, true, &mut compressed[prefix_len..])
                    .expect("Could not compress LZ4 data!");
            compressed.truncate(prefix_len + compressed_size);
            compressed
        }
        _ => {
            println!("Unknown compression type: {}", compression);
            data.to_vec()
        }
    }
}
