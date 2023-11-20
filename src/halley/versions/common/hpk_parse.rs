use base64::{engine::general_purpose, Engine as _};
use cookie_factory::{
    bytes::{le_u32 as w_le_u32, le_u64 as w_le_u64},
    combinator::slice as w_slice,
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn, WriteContext,
};
use flate2::{read::ZlibDecoder, read::ZlibEncoder};
use libaes::Cipher;
use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::length_count,
    number::complete::{le_u32, le_u64},
    sequence::tuple,
    IResult,
};
use std::{io::Read, mem::size_of};

use super::hpk::{HalleyPack, HalleyPackData, HpkSection, Parsable, Writable};

static IDENTIFIER: &str = "HALLEYPK";

pub fn parse_hpk<'a, Section>(
    i_full: &'a [u8],
    secret: Option<&str>,
) -> IResult<&'a [u8], impl HalleyPack>
where
    Section: Parsable + HpkSection + 'static,
{
    parse_hpk_header(i_full).map(move |(i, header)| {
        let (_, iv, _asset_db_start_pos, data_start_pos, asset_db_size) = header;

        let mut asset_db_bytes = vec![0; asset_db_size as usize];

        ZlibDecoder::new(i)
            .read_exact(&mut asset_db_bytes)
            .expect("Decompressed header does not match expected length");

        // println!(
        //     "asset_db -> {:?}",
        //     &asset_db_bytes[0..min(1000, asset_db_bytes.len())]
        // );

        let data = get_decrypted_data(&i_full[data_start_pos as usize..], secret, Some(&iv));

        //println!("data -> {:?}", &data[0..min(1000, data.len())]);

        let (_, asset_db) = length_count(le_u32, Section::parse)(&asset_db_bytes).unwrap();
        let asset_db = asset_db
            .into_iter()
            .map(move |s| Box::new(s) as Box<dyn HpkSection>)
            .collect();
        (i, HalleyPackData::new(asset_db, data))
    })
}

fn parse_hpk_header(i: &[u8]) -> IResult<&[u8], (&[u8], [u8; 16], u64, u64, u64)> {
    tuple((
        tag(IDENTIFIER),
        map(take(16usize), |iv: &[u8]| iv.try_into().unwrap()),
        le_u64,
        le_u64,
        le_u64,
    ))(i)
}

pub fn get_encrypted_data(
    data: &[u8],
    secret: Option<&str>,
    iv: Option<&[u8; 16]>,
) -> (Vec<u8>, [u8; 16]) {
    // TODO - remove empty secret check
    if secret.is_none() {
        return (data.to_vec(), [0 as u8; 16]);
    }
    let secret = secret.unwrap();

    let mut iv = *iv.unwrap_or(&[0 as u8; 16]);
    if iv == [0 as u8; 16] {
        iv = rand::random::<[u8; 16]>();
    }

    let mut key = [0; 16];
    general_purpose::STANDARD
        .decode_slice_unchecked(secret, &mut key)
        .unwrap();

    let data = encrypt(data, &key, &iv);
    (data, iv)
}

pub fn get_decrypted_data(data: &[u8], secret: Option<&str>, iv: Option<&[u8; 16]>) -> Vec<u8> {
    // TODO - remove empty secret check
    let iv = iv.unwrap_or(&[0 as u8; 16]);
    let has_crypt = *iv != [0 as u8; 16] && secret.is_some();
    let secret = secret.unwrap_or("");

    if has_crypt {
        let mut key = [0; 16];
        general_purpose::STANDARD
            .decode_slice_unchecked(secret, &mut key)
            .unwrap();
        decrypt(data, &key, iv)
    } else {
        data.to_vec()
    }
}

fn decrypt(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Vec<u8> {
    let mut c = Cipher::new_128(key);
    c.set_auto_padding(false);
    c.cbc_decrypt(iv, data)
}

fn encrypt(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Vec<u8> {
    let mut c = Cipher::new_128(key);
    c.set_auto_padding(false);
    c.cbc_encrypt(iv, data)
}

impl Writable for HalleyPackData {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let w = WriteContext::from(Vec::new());
        let asset_db = wh_tuple((
            w_le_u32(self.sections().len() as u32),
            wh_all(self.sections().iter().map(|s| s.write())),
        ))(w)
        .unwrap()
        .write;

        let mut encoded = Vec::new();
        let decoded: &[u8] = asset_db.as_ref();

        ZlibEncoder::new(decoded, flate2::Compression::default())
            .read_to_end(&mut encoded)
            .unwrap();

        let asset_db_size = asset_db.len();
        let asset_db_start_pos = IDENTIFIER.len() + 16 + (size_of::<u64>() * 2);
        let data_start_pos = asset_db_start_pos + size_of::<u64>() + encoded.len();

        assert!(asset_db_start_pos == 40);

        let writer = wh_tuple((
            w_slice(IDENTIFIER),
            w_slice([0 as u8; 16]),
            w_le_u64(asset_db_start_pos as u64),
            w_le_u64(data_start_pos as u64),
            w_le_u64(asset_db_size as u64),
            w_slice(encoded),
            w_slice(self.data()),
        ));

        Box::new(writer)
    }
}
