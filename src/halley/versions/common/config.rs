use super::{
    hpk::{Parsable, Writable},
    primitives::{h_bool, h_f32, h_i32, h_i64, h_string, h_u32, wh_bool, wh_string},
};
use cookie_factory::{
    bytes::{le_f32 as w_le_f32, le_i32 as w_le_i32, le_i64 as w_le_i64, le_u32 as w_le_u32},
    combinator::{cond as wh_cond, slice as wh_slice},
    multi::all as wh_all,
    sequence::tuple as wh_tuple,
    SerializeFn,
};
use nom::{
    combinator::{cond, map},
    multi::{length_count, length_data},
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(FromPrimitive, Serialize, Deserialize, ToPrimitive)]
pub enum ConfigNodeType {
    Undefined = 0,
    String = 1,
    Sequence = 2,
    Map = 3,
    Int = 4,
    Float = 5,
    Int2 = 6,
    Float2,
    Bytes,
    DeltaSequence, // For delta coding
    DeltaMap,      // For delta coding
    Noop,          // For delta coding
    Idx,           // For delta coding
    Del,           // For delta coding
    Int64,
    EntityId,
    Bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "__node_type", content = "__node_value")]
pub enum ConfigNode {
    Undefined,

    Int2((i32, i32)),   // vector 2i
    Float2((f32, f32)), // vector 2f
    Bytes(Vec<u8>),
    DeltaSequence((Vec<ConfigNode>, i32)), // For delta coding
    DeltaMap((ConfigNodeMap, i32)),        // For delta coding
    Noop,                                  // For delta coding
    Idx((i32, i32)),                       // For delta coding // vector 2i
    Del,                                   // For delta coding
    Int64(i64),
    EntityId(i64),

    #[serde(untagged)]
    Int(i32),
    #[serde(untagged)]
    Float(f32),
    #[serde(untagged)]
    Sequence(Vec<ConfigNode>),
    #[serde(untagged)]
    Map(ConfigNodeMap),
    #[serde(untagged)]
    String(String),
    #[serde(untagged)]
    Bool(bool),
}

pub type ConfigNodeMap = HashMap<String, ConfigNode>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub v: i32,
    pub store_file_position: bool,
    pub root: ConfigNode,
}

impl Parsable for ConfigFile {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        h_config_file(i)
    }
}

impl Writable for ConfigFile {
    fn write<'a>(&'a self) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        wh_config_file(self)
    }
}

pub fn h_config_file(i: &[u8]) -> IResult<&[u8], ConfigFile> {
    le_i32(i).map(|(i, v)| {
        let (i, store_file_position) = (cond(v > 2, h_bool))(i).unwrap();

        let store_file_position = store_file_position.unwrap_or(v == 2);

        let (i, root) = (if store_file_position {
            h_confignode_with_position(i)
        } else {
            h_confignode(i)
        })
        .unwrap();

        (
            i,
            ConfigFile {
                v,
                store_file_position,
                root,
            },
        )
    })
}

pub fn h_confignode(i: &[u8]) -> IResult<&[u8], ConfigNode> {
    h_confignode_layer(h_confignode, i)
}

pub fn h_confignode_with_position(i: &[u8]) -> IResult<&[u8], ConfigNode> {
    let (i, confignode) = h_confignode_layer(h_confignode_with_position, i)?;
    tuple((le_u32, le_u32))(i).map(|(i, (_line, _column))| (i, confignode))
}

pub type ConfigNodeParser = fn(&[u8]) -> IResult<&[u8], ConfigNode>;
pub type ConfigNodeWriter<'a> = dyn Fn(&'a ConfigNode) -> Box<dyn SerializeFn<Vec<u8>> + 'a>;

fn h_confignode_layer(h_confignode_deep: ConfigNodeParser, i: &[u8]) -> IResult<&[u8], ConfigNode> {
    let h_cn_map_deep = map(
        length_count(le_u32, tuple((h_string, h_confignode_deep))),
        vec_to_map,
    );
    le_u32(i).map(|(i, confignode_type)| {
        let a = match num::FromPrimitive::from_u32(confignode_type) {
            Some(ConfigNodeType::Noop) => (i, ConfigNode::Noop),
            Some(ConfigNodeType::Undefined) => (i, ConfigNode::Undefined),
            Some(ConfigNodeType::Del) => (i, ConfigNode::Del),
            Some(ConfigNodeType::Bool) => map(h_bool, ConfigNode::Bool)(i).unwrap(),
            Some(ConfigNodeType::String) => map(h_string, ConfigNode::String)(i).unwrap(),
            Some(ConfigNodeType::Map) => map(h_cn_map_deep, ConfigNode::Map)(i).unwrap(),
            Some(ConfigNodeType::DeltaMap) => {
                map(tuple((h_cn_map_deep, le_i32)), ConfigNode::DeltaMap)(i).unwrap()
            }
            Some(ConfigNodeType::Int64) => map(h_i64, ConfigNode::Int64)(i).unwrap(),
            Some(ConfigNodeType::EntityId) => map(h_i64, ConfigNode::EntityId)(i).unwrap(),
            Some(ConfigNodeType::Int) => map(h_i32, ConfigNode::Int)(i).unwrap(),
            Some(ConfigNodeType::Float) => map(h_f32, ConfigNode::Float)(i).unwrap(),
            Some(ConfigNodeType::Sequence) => map(
                length_count(le_u32, h_confignode_deep),
                ConfigNode::Sequence,
            )(i)
            .unwrap(),
            Some(ConfigNodeType::DeltaSequence) => map(
                tuple((length_count(le_u32, h_confignode_deep), le_i32)),
                ConfigNode::DeltaSequence,
            )(i)
            .unwrap(),
            Some(ConfigNodeType::Int2) => map(tuple((h_i32, h_i32)), ConfigNode::Int2)(i).unwrap(),
            Some(ConfigNodeType::Idx) => map(tuple((h_i32, h_i32)), ConfigNode::Idx)(i).unwrap(),
            Some(ConfigNodeType::Float2) => {
                map(tuple((h_f32, h_f32)), ConfigNode::Float2)(i).unwrap()
            }
            Some(ConfigNodeType::Bytes) => {
                map(length_data(h_u32), |b: &[u8]| ConfigNode::Bytes(b.to_vec()))(i).unwrap()
            }
            //Some(_) => (i, ConfigNode::Undefined), // throw err
            None => (i, ConfigNode::Undefined), // throw err
        };
        //println!("{:?} -> {:?}", confignode_type, a.1);
        a
    })
}
fn vec_to_map<K: Eq + Hash, V>(v: Vec<(K, V)>) -> HashMap<K, V> {
    v.into_iter()
        .map(|(k, v)| (k, v))
        .collect::<HashMap<K, V>>()
}

pub fn wh_config_file<'a>(file: &'a ConfigFile) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    let writer = wh_tuple((
        w_le_i32(file.v),
        wh_cond(file.v > 2, wh_bool(file.store_file_position)),
        wh_cond(
            file.store_file_position,
            wh_confignode_with_position(&file.root),
        ),
        wh_cond(!file.store_file_position, wh_confignode(&file.root)),
    ));
    Box::new(writer)
}

pub fn wh_confignode<'a>(node: &'a ConfigNode) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    wh_confignode_layer(&wh_confignode, node)
}

pub fn wh_confignode_with_position<'a>(node: &'a ConfigNode) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    Box::new(wh_tuple((
        wh_confignode_layer(&wh_confignode_with_position, node),
        w_le_u32(0),
        w_le_u32(0),
    )))
}

fn wh_confignode_layer<'a>(
    wh_confignode_deep: &'a ConfigNodeWriter<'a>,
    node: &'a ConfigNode,
) -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
    let slice0 = Box::new(wh_slice::<_, Vec<u8>>(vec![]));

    let wh_cn_map_deep = |map: &'a ConfigNodeMap| -> Box<dyn SerializeFn<Vec<u8>> + 'a> {
        let writer = wh_tuple((
            w_le_u32(map.len() as u32),
            wh_all(
                map.iter()
                    .map(|(k, v)| wh_tuple((wh_string(k), wh_confignode_deep(v)))),
            ),
        ));
        Box::new(writer)
    };

    let (t, f): (ConfigNodeType, Box<dyn SerializeFn<Vec<u8>> + 'a>) = match node {
        ConfigNode::Undefined => (ConfigNodeType::Undefined, slice0),
        ConfigNode::Noop => (ConfigNodeType::Noop, slice0),
        ConfigNode::Del => (ConfigNodeType::Del, slice0),
        ConfigNode::Int(i) => (ConfigNodeType::Int, Box::new(w_le_i32(*i))),
        ConfigNode::Bool(b) => (ConfigNodeType::Bool, Box::new(wh_bool(*b))),
        ConfigNode::String(s) => (ConfigNodeType::String, Box::new(wh_string(s))),
        ConfigNode::Map(map) => (ConfigNodeType::Map, Box::new(wh_cn_map_deep(map))),
        ConfigNode::Sequence(seq) => (
            ConfigNodeType::Sequence,
            Box::new(wh_tuple((
                w_le_u32(seq.len() as u32),
                wh_all(seq.iter().map(|n| wh_confignode_deep(n))),
            ))),
        ),
        ConfigNode::Float(f) => (ConfigNodeType::Int, Box::new(w_le_f32(*f))),
        ConfigNode::Int2((i1, i2)) => (
            ConfigNodeType::Int2,
            Box::new(wh_tuple((w_le_i32(*i1), w_le_i32(*i2)))),
        ),
        ConfigNode::Float2((f1, f2)) => (
            ConfigNodeType::Float2,
            Box::new(wh_tuple((w_le_f32(*f1), w_le_f32(*f2)))),
        ),
        ConfigNode::Bytes(b) => (
            ConfigNodeType::Bytes,
            Box::new(wh_tuple((w_le_u32(b.len() as u32), wh_slice(b)))),
        ),
        ConfigNode::DeltaSequence((seq, i)) => (
            ConfigNodeType::DeltaSequence,
            Box::new(wh_tuple((
                w_le_u32(seq.len() as u32),
                wh_all(seq.iter().map(|n| wh_confignode_deep(n))),
                w_le_i32(*i),
            ))),
        ),
        ConfigNode::DeltaMap((map, i)) => (
            ConfigNodeType::DeltaMap,
            Box::new(wh_tuple((wh_cn_map_deep(map), w_le_i32(*i)))),
        ),
        ConfigNode::Idx((i1, i2)) => (
            ConfigNodeType::Idx,
            Box::new(wh_tuple((w_le_i32(*i1), w_le_i32(*i2)))),
        ),
        ConfigNode::Int64(i) => (ConfigNodeType::Int64, Box::new(w_le_i64(*i))),
        ConfigNode::EntityId(id) => (ConfigNodeType::EntityId, Box::new(w_le_i64(*id))),
    };
    let node_type = num_traits::ToPrimitive::to_i32(&t).unwrap();
    Box::new(wh_tuple((w_le_u32(node_type as u32), f)))
}
