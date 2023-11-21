use cookie_factory::{
    bytes::{le_i8 as w_le_i8, le_u32 as w_le_u32},
    combinator::slice as w_slice,
    multi::all as w_all,
    sequence::tuple as w_tuple,
    SerializeFn,
};
use nom::{
    bytes::complete::take,
    combinator::{map, peek},
    error::Error,
    multi::{length_count, length_data},
    number::complete::{le_f32, le_i32, le_i64, le_i8, le_u32, le_u64, u8},
    sequence::tuple,
    IResult,
};
use std::{cmp::min, collections::HashMap, io::Write};

pub fn h_hashmap(i: &[u8]) -> IResult<&[u8], HashMap<String, String>> {
    length_count(le_u32, tuple((h_string, h_string)))(i).map(|(i, entries)| {
        let mut map = HashMap::new();
        for (k, v) in entries {
            map.insert(k, v);
        }
        (i, map)
    })
}

pub fn wh_hashmap<'a, W: Write + 'a>(
    hashmap: &'a HashMap<String, String>,
) -> impl SerializeFn<W> + 'a {
    let entries = hashmap
        .iter()
        .map(|(k, v)| w_tuple((wh_string(k), wh_string(v))));

    w_tuple((w_le_u32(hashmap.len() as u32), w_all(entries)))
}

pub fn h_pos_size(i: &[u8]) -> IResult<&[u8], (usize, usize)> {
    map(h_string, |meta| {
        let (pos_str, size_str) = meta.split_once(':').unwrap();
        let pos = pos_str.parse::<usize>().unwrap();
        let size = size_str.parse::<usize>().unwrap();
        (pos, size)
    })(i)
}

pub fn wh_pos_size<W: Write>(pos: usize, size: usize) -> impl SerializeFn<W> {
    wh_string(&format!("{}:{}", pos, size))
}

pub fn h_string(i: &[u8]) -> IResult<&[u8], String> {
    length_data(le_u32)(i).map(|(i, s)| (i, String::from_utf8(s.to_vec()).unwrap()))
}

pub fn wh_string<W: Write>(str: &String) -> impl SerializeFn<W> {
    let len = str.len();
    w_tuple((w_le_u32(len as u32), w_slice(str.clone())))
}

pub fn h_var_string(i: &[u8]) -> IResult<&[u8], String> {
    length_data(h_var_u)(i).map(|(i, s)| (i, String::from_utf8(s.to_vec()).unwrap()))
}

pub fn wh_var_string(str: &String) -> impl SerializeFn<Vec<u8>> {
    let len = str.len();
    w_tuple((wh_var_u(len as u64), w_slice(str.clone())))
}

pub fn h_bool(i: &[u8]) -> IResult<&[u8], bool> {
    le_i8(i).map(|(i, b)| (i, b == 1))
}

pub fn wh_bool<W: Write>(b: bool) -> impl SerializeFn<W> {
    w_le_i8(if b { 1 } else { 0 })
}

pub fn h_i32(i: &[u8]) -> IResult<&[u8], i32> {
    le_i32(i)
}

pub fn h_f32(i: &[u8]) -> IResult<&[u8], f32> {
    le_f32(i)
}

pub fn h_i64(i: &[u8]) -> IResult<&[u8], i64> {
    le_i64(i)
}

pub fn h_u64(i: &[u8]) -> IResult<&[u8], u64> {
    le_u64(i)
}

pub fn h_u32(i: &[u8]) -> IResult<&[u8], u32> {
    le_u32(i)
}

pub fn h_var_i(i: &[u8]) -> IResult<&[u8], i64> {
    map(var_u64(true), |(v, sign)| {
        let v = v as i64;
        if sign {
            v
        } else {
            -v
        }
    })(i)
}

pub fn wh_var_i(v: i64) -> impl SerializeFn<Vec<u8>> {
    w_var_u64(Some(v >= 0), v.abs() as u64)
}

pub fn h_var_u(i: &[u8]) -> IResult<&[u8], u64> {
    map(var_u64(false), |(v, _)| v)(i)
}

pub fn wh_var_u(v: u64) -> impl SerializeFn<Vec<u8>> {
    w_var_u64(None, v)
}

pub fn var_u64(is_signed: bool) -> impl Fn(&[u8]) -> IResult<&[u8], (u64, bool)> {
    move |i: &[u8]| {
        map(read_var_n_bytes, |(vec, n_bytes)| {
            let header_bits = min(n_bytes, 8);

            let mut bits_available = 8 - header_bits;
            let mut bits_read: usize = 0;
            let mut v: u64 = 0;
            let mut sign = false;

            for i in 0..n_bytes {
                let byte_mask: u64 = (1 << bits_available) - 1;
                v |= ((u64::from(vec[i])) & byte_mask) << bits_read;
                bits_read += bits_available;
                bits_available = 8;
            }

            if is_signed {
                let sign_pos =
                    ((n_bytes as i32) * 7 + (if n_bytes == 9 { 0 } else { -1 })) as usize;
                let sign_mask = 1 << sign_pos;
                sign = (v & sign_mask) != 0;
                v &= !sign_mask;
            }

            (v, sign)
        })(i)
    }
}

pub fn w_var_u64(is_signed: Option<bool>, v: u64) -> impl SerializeFn<Vec<u8>> {
    let n_bits = if v == u64::MAX {
        64
    } else {
        std::cmp::max(1, ((v + 1) as f64).log2().ceil() as usize)
            + (if is_signed.is_some() { 1 } else { 0 })
    };

    let n_bytes = std::cmp::min((n_bits - 1) / 7, 8) + 1;
    let mut bytes = vec![0 as u8; 9];

    let mut to_write = v;
    match is_signed {
        Some(sign) => {
            let sign_pos = ((n_bytes as i32) * 7 + (if n_bytes == 9 { 0 } else { -1 })) as usize;
            let sign = if sign { 1 } else { 0 };
            to_write |= sign << sign_pos;
        }
        None => {}
    }

    let header_bits = n_bytes;
    bytes[0] = 255 ^ ((1 << (9 - header_bits)) - 1) as u8;

    let mut bits_available = 8 - std::cmp::min(header_bits, 8);
    let mut bits_to_write = n_bits;
    let mut pos: usize = 0;

    while bits_to_write > 0 {
        let n_bits = std::cmp::min(bits_to_write, bits_available);
        let mask = ((1 as u64) << bits_available) - 1;
        bytes[pos] |= (to_write & mask) as u8;
        to_write >>= n_bits;
        bits_available = 8;
        pos += 1;
        bits_to_write -= n_bits;
    }

    bytes.truncate(pos);

    w_slice(bytes)
}

fn read_var_n_bytes(i: &[u8]) -> IResult<&[u8], (&[u8], usize)> {
    peek_var_n_bytes(i).map(|(i, n_bytes)| {
        let (i, vec) = take::<usize, &[u8], Error<_>>(n_bytes)(i).unwrap();
        (i, (vec, n_bytes))
    })
}

fn peek_var_n_bytes(i: &[u8]) -> IResult<&[u8], usize> {
    map(peek(u8), |header| {
        //let n_bytes: usize = (1..=8)
        //     .find(|&mask| (header & (0xFF << (8 - mask))) != (0xFF << (8 - mask)))
        //     .unwrap_or(9);

        let n_bytes: usize = if header & 0x80 != 0x80 {
            1
        } else if header & 0xC0 != 0xC0 {
            2
        } else if header & 0xE0 != 0xE0 {
            3
        } else if header & 0xF0 != 0xF0 {
            4
        } else if header & 0xF8 != 0xF8 {
            5
        } else if header & 0xFC != 0xFC {
            6
        } else if header & 0xFE != 0xFE {
            7
        } else if header & 0xFF != 0xFF {
            8
        } else {
            9
        };
        n_bytes
    })(i)
}

#[cfg(test)]
mod tests {
    use cookie_factory::WriteContext;

    use super::*;

    fn convert_back_and_forth_i(n: i64) -> i64 {
        let buf = vec![];
        let res = wh_var_i(n)(WriteContext {
            write: buf,
            position: 0,
        })
        .unwrap();
        let (_, vv) = h_var_i(&res.write).unwrap();
        vv
    }

    fn convert_back_and_forth_u(n: u64) -> u64 {
        let buf = vec![];
        let res = wh_var_u(n)(WriteContext {
            write: buf,
            position: 0,
        })
        .unwrap();
        let (_, vv) = h_var_u(&res.write).unwrap();
        vv
    }

    #[test]
    fn test_wh_var_i() {
        let tests = vec![
            0,
            1,
            128,
            14141,
            8457345,
            275602752,
            61956541,
            9223372036854775807,
            -1,
            -114115,
            -128,
        ];

        for v in tests {
            assert_eq!(v, convert_back_and_forth_i(v));
        }
    }

    #[test]
    fn test_wh_var_u() {
        let tests = vec![
            0,
            1,
            100,
            128,
            14141,
            12800,
            1638400,
            8457345,
            61956541,
            209715200,
            275602752,
            26843545600,
            3435973836800,
            439804651110400,
            56294995342131200,
            7205759403792793600,
            9223372036854775807,
            18446744073709551615,
        ];

        for v in tests {
            assert_eq!(v, convert_back_and_forth_u(v));
        }
    }
}
