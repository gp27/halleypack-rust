use std::collections::HashMap;

use nom::{
    combinator::map,
    multi::length_count,
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use super::super::common::primitives::{h_bool, h_string};

#[derive(Serialize, Deserialize)]
pub struct AudioEvent {
    pub actions: Vec<AudioEventAction>,
}

#[derive(Serialize, Deserialize)]
pub enum AudioEventActionType {
    PlayLegacy = 0,
    Play = 1,
    Stop,
    Pause,
    Resume,
    StopBus,
    PauseBus,
    ResumeBus,
    SetVolume,
    SetSwitch,
    SetVariable,
}

#[derive(Serialize, Deserialize)]
pub enum AudioEventAction {
    PlayLegacy,
    Play,
    Stop,
    Pause,
    Resume,
    StopBus,
    PauseBus,
    ResumeBus,
    SetVolume,
    SetSwitch,
    SetVariable,
}

pub fn audio_event_parser(i: &[u8]) -> IResult<&[u8], AudioEvent> {
    map(length_count(le_u32, audio_event_action_parser), |actions| {
        AudioEvent { actions }
    })(i)
}

fn audio_event_action_parser(i: &[u8]) -> IResult<&[u8], AudioEventAction> {
    Ok((i, AudioEventAction::PlayLegacy))
}
