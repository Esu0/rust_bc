#![allow(dead_code)]

use crate::database::{consume_buf, get_next, get_next_line, get_string, Mamodel, Mamodels, ASSET_PATH};
use bevy::prelude::*;

use super::super::error::{Error, ErrorKind};
use super::UnitState;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;
use std::str::{FromStr, Split};

#[derive(Debug, Clone, Default)]
pub struct Maanim {
    parts: Vec<MaanimPart>,
    period: u32,
}

#[derive(Debug, Clone)]
pub struct MaanimPart {
    id: u16,
    modification: Modification,
    loops: bool,
    eases: Vec<Ease>,
    frame_start: i32,
    frame_end: i32,
}

#[derive(Debug, Default, Clone)]
pub struct Ease {
    frame: i32,
    value: i32,
    easing: Easing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modification {
    Parent,
    Id,
    Sprite,
    Zorder,
    Xpos,
    Ypos,
    Pivotx,
    Pivoty,
    Scale,
    Scalex,
    Scaley,
    Angle,
    Opacity,
    HorizontalFlip,
    VerticalFlip,
    ExtendX,
    ExtendY,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    #[default]
    Zero,
    Positive,
    Negative,
}

use Sign::{Negative, Positive, Zero};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    Nothing,
    InOut(i32),
    Ease3,
    Sine(Sign),
}

impl TryFrom<i32> for Modification {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Modification::Parent),
            1 => Ok(Modification::Id),
            2 => Ok(Modification::Sprite),
            3 => Ok(Modification::Zorder),
            4 => Ok(Modification::Xpos),
            5 => Ok(Modification::Ypos),
            6 => Ok(Modification::Pivotx),
            7 => Ok(Modification::Pivoty),
            8 => Ok(Modification::Scale),
            9 => Ok(Modification::Scalex),
            10 => Ok(Modification::Scaley),
            11 => Ok(Modification::Angle),
            12 => Ok(Modification::Opacity),
            13 => Ok(Modification::HorizontalFlip),
            14 => Ok(Modification::VerticalFlip),
            50 => Ok(Modification::ExtendX),
            52 => Ok(Modification::ExtendY),
            _ => Err(Error::new(
                ErrorKind::InvalidNumber,
                "無効なModificationタイプ",
            )),
        }
    }
}

impl Sign {
    fn from_int(num: i32) -> Self {
        match num {
            ..=-1 => Negative,
            0 => Zero,
            1.. => Positive,
        }
    }
}

/// [(id0, diff0), (id1, diff1), (id2, diff2), ...]
/// idについて昇順ソート済み
#[derive(Clone, Debug)]
pub struct StateDiffs(pub Vec<StateDiff>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StateDiff {
    pub id: i32,
    pub diff: StateDiffVal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StateDiffVal {
    Parent(i32),
    Id(i32),
    Sprite(i32),
    Zorder(i32),
    Posx(i32),
    Posy(i32),
    Pivotx(i32),
    Pivoty(i32),
    Scale(i32),
    Scalex(i32),
    Scaley(i32),
    Angle(i32),
    Opacity(i32),
    HorizontalFlip(bool),
    VerticalFlip(bool),
}

impl StateDiffVal {
    pub fn new(modi: Modification, val: i32) -> Option<Self> {
        match modi {
            Modification::Parent => Some(Self::Parent(val)),
            Modification::Id => Some(Self::Id(val)),
            Modification::Sprite => Some(Self::Sprite(val)),
            Modification::Zorder => Some(Self::Zorder(val)),
            Modification::Xpos => Some(Self::Posx(val)),
            Modification::Ypos => Some(Self::Posy(val)),
            Modification::Pivotx => Some(Self::Pivotx(val)),
            Modification::Pivoty => Some(Self::Pivoty(val)),
            Modification::Scale => Some(Self::Scale(val)),
            Modification::Scalex => Some(Self::Scalex(val)),
            Modification::Scaley => Some(Self::Scaley(val)),
            Modification::Angle => Some(Self::Angle(val)),
            Modification::Opacity => Some(Self::Opacity(val)),
            Modification::HorizontalFlip => Some(Self::HorizontalFlip(val != 0)),
            Modification::VerticalFlip => Some(Self::VerticalFlip(val != 0)),
            _ => None,
        }
    }
}

impl Maanim {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let f = File::open(Path::new(ASSET_PATH).join(path))?;
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        consume_buf(&mut lines, |s| {
            s.starts_with("[modelanim:animation2]") || s.starts_with("[modelanim:animation]")
        })?;
        consume_buf(&mut lines, |_| true)?;
        let len = get_next_line(&mut lines)?;
        let mut parts = Vec::with_capacity(len);
        let mut period = 0;
        for _ in 0..len {
            let s = get_string(&mut lines)?;
            let mut split = s.split(',');

            let part_id = get_next(&mut split)?;
            let modification: i32 = get_next(&mut split)?;
            let loops: i32 = get_next(&mut split)?;

            let len = get_next_line(&mut lines)?;

            let mut eases = Vec::with_capacity(len);

            for _ in 0..len {
                let s = get_string(&mut lines)?;
                let mut split = s.split(',');
                eases.push(Ease {
                    frame: get_next(&mut split)?,
                    value: get_next(&mut split)?,
                    easing: {
                        match get_next::<i32>(&mut split)? {
                            0 => Easing::Linear,
                            1 => Easing::Nothing,
                            2 => Easing::InOut(get_next(&mut split)?),
                            3 => Easing::Ease3,
                            4 => Easing::Sine(Sign::from_int(get_next(&mut split)?)),
                            _ => {
                                return Err(Error::new(
                                    ErrorKind::InvalidNumber,
                                    "無効なEasingタイプ",
                                ));
                            }
                        }
                    },
                });
            }

            let (frame_start, frame_end) = eases
                .first()
                .zip(eases.last())
                .map(|(e1, e2)| (e1.frame, e2.frame))
                .unwrap_or_default();
            parts.push(MaanimPart {
                id: part_id,
                modification: modification.try_into()?,
                loops: loops != -1,
                eases,
                frame_start,
                frame_end,
            });
            let range = (frame_end - frame_start) as u32;
            if period < range {
                period = range;
            }
        }
        parts.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Maanim { parts, period })
    }

    pub fn into_state_generator(self, mamodels: &Mamodels) -> StateGenerator {
        let part_len = self.parts.len();
        let mut state = UnitState::from_model(mamodels);
        let part_indice = self
            .parts
            .iter()
            .map(|anim| {
                let ind = 
                anim.eases
                    .binary_search_by(|ease| ease.frame.cmp(&0))
                    .unwrap_or_default();
                if let Some(diff) = StateDiffVal::new(anim.modification, anim.eases[ind].value) {
                    let state_ref = &mut state.states[anim.id as usize];
                    state_ref.load_diff(diff);
                }
                ind as u16
            })
            .collect();

        StateGenerator {
            diff_generator: StateDiffGenerator {
                maanim: self,
                current_frame: 0,
                part_indice,
                buf_queue: vec![VecDeque::new(); part_len],
            },
            current_state: state,
        }
    }
}

use std::collections::VecDeque;
#[derive(Clone, Debug, Default)]
pub struct StateDiffGenerator {
    maanim: Maanim,
    current_frame: i32,
    part_indice: Vec<u16>,
    buf_queue: Vec<VecDeque<Option<StateDiffVal>>>,
}

#[derive(Clone, Resource, Debug)]
pub struct StateGenerator {
    diff_generator: StateDiffGenerator,
    current_state: UnitState,
}

impl StateGenerator {
    pub fn next_state(&mut self) -> UnitState {
        let StateGenerator {diff_generator, current_state} = self;
        let next = current_state.clone();
        current_state.load_diff(diff_generator.next_state_diff());
        next
    }
}

// impl Iterator for StateGenerator {
//     type Item = UnitState;
//     fn next(&mut self) -> Option<Self::Item> {
//         let StateGenerator {diff_generator, current_state} = self;
//         let next = current_state.clone();
//         current_state.load_diff(diff_generator.next_state_diff());
//         Some(next)
//     }
// }

impl StateGenerator {
    pub fn empty(models: &Mamodels) -> Self {
        Self {
            diff_generator: StateDiffGenerator::default(),
            current_state: UnitState {
                states: vec![super::State::default(); models.models.len()],
            }
        }
    }

    pub fn with_raw_model(models: &Mamodels) -> Self {
        Self {
            diff_generator: StateDiffGenerator::default(),
            current_state: UnitState::from_model(models),
        }
    }
}
impl StateDiffGenerator {
    fn next_state_diff(&mut self) -> StateDiffs {
        let Self {
            maanim,
            current_frame,
            part_indice,
            buf_queue,
        } = self;
        let mut diff_set = Vec::new();
        for ((part, ind), queue) in maanim.parts.iter().zip(part_indice).zip(buf_queue) {
            if part.loops {
                if *current_frame >= part.frame_end {
                    continue;
                }
            }
            if let Some(diff) = queue.pop_back() {
                if let Some(diff) = diff {
                    diff_set.push(StateDiff {
                        id: part.id as _,
                        diff,
                    });
                }
            } else {
                if part.eases.len() <= 1 {
                    continue;
                }
                let mut ease1 = &part.eases[*ind as usize];
                *ind += 1;
                let fd;
                let vd;
                let easing = ease1.easing;
                if let Some(ease2) = part.eases.get(*ind as usize) {
                    fd = ease2.frame - ease1.frame;
                    vd = ease2.value - ease1.value;
                } else {
                    ease1 = &part.eases[0];
                    let ease2 = &part.eases[1];
                    *ind = 1;
                    fd = ease2.frame - ease1.frame;
                    vd = ease2.value - ease1.value;
                }

                match easing {
                    Easing::Linear => {
                        let l = vd as f64 / fd as f64;
                        if let Some(diff) =
                            StateDiffVal::new(part.modification, ease1.value + l as i32)
                        {
                            diff_set.push(StateDiff {
                                id: part.id as _,
                                diff,
                            });
                            for i in 2..=fd {
                                queue.push_front(StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (l * i as f64) as i32,
                                ));
                            }
                        }
                    }
                    Easing::Nothing => {
                        if fd == 1 {
                            if let Some(diff) =
                                StateDiffVal::new(part.modification, ease1.value + vd)
                            {
                                diff_set.push(StateDiff {
                                    id: part.id as _,
                                    diff,
                                });
                            }
                        } else {
                            for _ in 2..fd {
                                queue.push_front(None);
                            }
                            queue
                                .push_front(StateDiffVal::new(part.modification, ease1.value + vd));
                        }
                        
                    }
                    Easing::InOut(p) => {
                        if p >= 0 {
                            let func = |x: f64| 1. - (1. - x.powi(p)).sqrt();
                            if let Some(diff) = StateDiffVal::new(
                                part.modification,
                                ease1.value + (vd as f64 * func(1. / fd as f64)) as i32,
                            ) {
                                diff_set.push(StateDiff {
                                    id: part.id as _,
                                    diff,
                                });
                                for i in 2..=fd {
                                    queue.push_front(StateDiffVal::new(
                                        part.modification,
                                        ease1.value
                                            + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                                    ));
                                }
                            }
                            
                        } else {
                            let func = |x: f64| (1. - (1. - x).powi(-p)).sqrt();
                            if let Some(diff) = StateDiffVal::new(
                                part.modification,
                                ease1.value + (vd as f64 * func(1. / fd as f64)) as i32,
                            ) {
                                diff_set.push(StateDiff {
                                    id: part.id as _,
                                    diff,
                                });
                                for i in 2..=fd {
                                    queue.push_front(StateDiffVal::new(
                                        part.modification,
                                        ease1.value
                                            + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                                    ));
                                }
                            }
                            
                        };
                    }
                    Easing::Sine(sig) => {
                        fn sine_p(x: f64) -> f64 {
                            1. - (x * std::f64::consts::FRAC_PI_2).cos()
                        }
                        fn sine_n(x: f64) -> f64 {
                            (x * std::f64::consts::FRAC_PI_2).sin()
                        }
                        fn sine_z(x: f64) -> f64 {
                            (1. - (x * std::f64::consts::PI).cos()) / 2.
                        }
                        let func = match sig {
                            Sign::Positive => sine_p,
                            Sign::Negative => sine_n,
                            Sign::Zero => sine_z,
                        };
                        if let Some(diff) = StateDiffVal::new(
                            part.modification,
                            ease1.value + (vd as f64 * func(1. / fd as f64)) as i32,
                        ) {
                            diff_set.push(StateDiff {
                                id: part.id as _,
                                diff,
                            });
                            for i in 2..=fd {
                                queue.push_front(StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                                ));
                            }
                        }
                        // println!("(id: {}, frame: {current_frame}): {queue:?}", part.id);
                        // println!("func(): {}", func(1. / fd as f64));
                    }
                    Easing::Ease3 => {
                        let low = (*ind - 1) as usize;
                        while *ind < (part.eases.len() - 1) as u16 {
                            if part.eases[*ind as usize].easing != Easing::Ease3 {
                                break;
                            }
                            *ind += 1;
                        }
                        let high = *ind as usize;
                        let factors: Vec<f64> = part.eases[..=high]
                            .iter()
                            .enumerate()
                            .skip(low)
                            .map(|(i, e)| {
                                let mut factor = 4096.0;
                                for j in low..i {
                                    factor /= (e.frame - part.eases[j].frame) as f64;
                                }
                                for j in (i + 1)..=high {
                                    factor /= (e.frame - part.eases[j].frame) as f64;
                                }
                                factor
                            })
                            .collect();
                        // println!("eases: {:?}", &part.eases[low..=high]);
                        // println!("{factors:?}");
                        let frame = (*current_frame - part.frame_start) % (part.frame_end - part.frame_start) + part.frame_start;
                        *queue = ((frame + 1)..part.eases[high].frame)
                            .map(|f| {
                                let mut sum = 0.;
                                for (i, factor) in factors.iter().enumerate() {
                                    let ind_i = i + low;
                                    let mut val = *factor * part.eases[ind_i].value as f64;
                                    for j in low..ind_i {
                                        val *= (f - part.eases[j].frame) as f64;
                                    }
                                    for j in (ind_i + 1)..=high {
                                        val *= (f - part.eases[j].frame) as f64;
                                    }
                                    sum += val;
                                }
                                StateDiffVal::new(part.modification, (sum / 4096.) as i32)
                            })
                            .rev()
                            .collect();
                        if let Some(Some(diff)) = queue.pop_back() {
                            // println!("{queue:?}");
                            diff_set.push(StateDiff {
                                id: part.id as _,
                                diff,
                            });
                        }
                    }
                }
            }
        }
        *current_frame += 1;
        StateDiffs(diff_set)
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maanim() {
        println!(
            "{:#?}",
            Maanim::load("assets/org/unit/697/s/697_s00.maanim").unwrap()
        );
    }

    #[test]
    fn load_all_unit() {
        let unit_path = Path::new("assets/org/unit");
        // unit読み込み
        for i in 0..=697 {
            for c in ['f', 's', 'c'] {
                for j in 0..4 {
                    let path =
                        unit_path.join(format!("{0:>03}/{c}/{0:>03}_{c}{1:>02}.maanim", i, j));
                    if let Ok(anim) = Maanim::load(path) {
                        if let Some(part) = anim
                            .parts
                            .iter()
                            .find(|elem| elem.modification == Modification::Id)
                        {
                            println!("({i}, {c}, {j}): {:#?}", part);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn load_all_enemy() {
        let enemy_path = Path::new("assets/org/enemy");
        for i in 0..=634 {
            for j in 0..4 {
                let path = enemy_path.join(format!("{0:>03}/{0:>03}_e{1:>02}.maanim", i, j));
                if let Ok(anim) = Maanim::load(path) {
                    if let Some(part) = anim
                        .parts
                        .iter()
                        .find(|elem| elem.modification == Modification::Id)
                    {
                        println!("({i}, {j}): {:#?}", part);
                    }
                }
            }
            for j in 0..3 {
                let path = enemy_path.join(format!("{0:>03}/{0:>03}_e_zombie{1:>02}.maanim", i, j));
                if let Ok(anim) = Maanim::load(path) {
                    if let Some(part) = anim
                        .parts
                        .iter()
                        .find(|elem| elem.modification == Modification::Id)
                    {
                        println!("({i}, zombie{j}): {:#?}", part);
                    }
                }
            }
        }
    }
}
