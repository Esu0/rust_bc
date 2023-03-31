#![allow(dead_code)]

use crate::database::{
    consume_buf, get_next, get_next_line, get_string, Mamodel, Mamodels, ASSET_PATH,
};
use bevy::prelude::*;

use super::super::error::{Error, ErrorKind};
use super::UnitState;
use std::cmp::Ordering;
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
    ExtendX(i32),
    ExtendY(i32),
}

impl StateDiffVal {
    pub fn new(modi: Modification, val: i32) -> Self {
        match modi {
            Modification::Parent => Self::Parent(val),
            Modification::Id => Self::Id(val),
            Modification::Sprite => Self::Sprite(val),
            Modification::Zorder => Self::Zorder(val),
            Modification::Xpos => Self::Posx(val),
            Modification::Ypos => Self::Posy(val),
            Modification::Pivotx => Self::Pivotx(val),
            Modification::Pivoty => Self::Pivoty(val),
            Modification::Scale => Self::Scale(val),
            Modification::Scalex => Self::Scalex(val),
            Modification::Scaley => Self::Scaley(val),
            Modification::Angle => Self::Angle(val),
            Modification::Opacity => Self::Opacity(val),
            Modification::HorizontalFlip => Self::HorizontalFlip(val != 0),
            Modification::VerticalFlip => Self::VerticalFlip(val != 0),
            Modification::ExtendX => Self::ExtendX(val),
            Modification::ExtendY => Self::ExtendY(val),
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

            let mut eases = Vec::with_capacity(len + 1usize);
            if len > 0 {
                let s = get_string(&mut lines)?;
                let mut split = s.split(',');
                let frame: i32 = get_next(&mut split)?;
                let ease = Ease {
                    frame,
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
                };
                if frame > 0 {
                    eases.push(Ease {
                        frame: 0,
                        easing: Easing::Nothing,
                        value: ease.value,
                    });
                }
                eases.push(ease);
            }
            for _ in 1..len {
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
        // println!("period: {period}");
        Ok(Maanim { parts, period })
    }

    pub fn into_state_generator(self, mamodels: &Mamodels) -> StateGenerator {
        let part_len = self.parts.len();
        let mut state = UnitState::from_model(mamodels);
        let part_indice = self
            .parts
            .iter()
            .map(|anim| {
                anim.eases
                    .binary_search_by(|ease| ease.frame.cmp(&0))
                    .unwrap_or_else(|e| e - 1) as u16
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
    buf_queue: Vec<VecDeque<DiffOrNothing>>,
}

#[derive(Clone, Resource, Debug)]
pub struct StateGenerator {
    diff_generator: StateDiffGenerator,
    current_state: UnitState,
}

impl StateGenerator {
    pub fn next_state(&mut self) -> UnitState {
        let StateGenerator {
            diff_generator,
            current_state,
        } = self;
        current_state.load_diff(diff_generator.next_state_diff());
        current_state.clone()
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
            },
        }
    }

    pub fn with_raw_model(models: &Mamodels) -> Self {
        Self {
            diff_generator: StateDiffGenerator::default(),
            current_state: UnitState::from_model(models),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffOrNothing {
    Diff(StateDiffVal),
    Nothing(u16),
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

        // println!("next_state_diff");
        // println!("{}, {}, {}", maanim.parts.len(), part_indice.len(), buf_queue.len());
        for ((part, ind), queue) in maanim.parts.iter().zip(part_indice).zip(buf_queue) {
            let period = if part.loops {
                maanim.period as i32
            } else {
                part.frame_end - part.frame_start
            };
            // println!("{}", part.loops);
            let part_frame =
                (*current_frame - part.frame_start).rem_euclid(period) + part.frame_start;
            
            // if part.id == 5 && part.modification == Modification::Opacity {
            //     println!("part_frame:{part_frame}, ind:{ind}, queue:{queue:?}");
            // }
            if let Some(diff) = queue.pop_back() {
                match diff {
                    DiffOrNothing::Diff(diff) => diff_set.push(StateDiff {
                        id: part.id as _,
                        diff,
                    }),
                    DiffOrNothing::Nothing(n) => {
                        let n = n - 1;
                        if n > 0 {
                            queue.push_back(DiffOrNothing::Nothing(n));
                        }
                    }
                }
            } else {
                
                match part.eases.len() {
                    0 => {
                        continue;
                    }
                    1 => {
                        diff_set.push(StateDiff {
                            id: part.id as _,
                            diff: StateDiffVal::new(part.modification, part.eases[0].value),
                        });
                        continue;
                    }
                    _ => {}
                }
                match part_frame.cmp(&part.frame_end) {
                    Ordering::Greater => {
                        // println!("greater");
                        continue;
                    }
                    Ordering::Equal => {
                        diff_set.push(StateDiff {
                            id: part.id as _,
                            diff: StateDiffVal::new(
                                part.modification,
                                part.eases.last().unwrap().value,
                            ),
                        });
                        continue;
                    }
                    _ => {}
                }
                
                let mut ease1 = &part.eases[*ind as usize];
                *ind += 1;
                let fd;
                let vd;
                let mut easing = ease1.easing;

                // println!("{f}");
                if let Some(ease2) = part.eases.get(*ind as usize) {
                    fd = ease2.frame - ease1.frame;
                    vd = ease2.value - ease1.value;
                } else {
                    ease1 = &part.eases[0];
                    easing = ease1.easing;
                    let ease2 = &part.eases[1];
                    *ind = 1;
                    fd = ease2.frame - ease1.frame;
                    vd = ease2.value - ease1.value;
                    // println!("else");
                }
                let f = part_frame - ease1.frame;

                if !(ease1.frame..(ease1.frame + fd)).contains(&part_frame) {
                    println!("warning: wrong animation");
                    println!(
                        "info: current_frame = {current_frame}, part_frame = {part_frame}, ease1.frame = {}, fd = {fd}, id = {}, frame_start = {}",
                        ease1.frame, part.id, part.frame_start,
                    );
                }
                match easing {
                    Easing::Linear => {
                        let l = vd as f64 / fd as f64;
                        diff_set.push(StateDiff {
                            id: part.id as _,
                            diff: StateDiffVal::new(
                                part.modification,
                                ease1.value + (l * f as f64) as i32,
                            ),
                        });
                        for i in (f + 1)..fd {
                            queue.push_front(DiffOrNothing::Diff(StateDiffVal::new(
                                part.modification,
                                ease1.value + (l * i as f64) as i32,
                            )));
                        }
                    }
                    Easing::Nothing => {
                        diff_set.push(StateDiff {
                            id: part.id as _,
                            diff: StateDiffVal::new(part.modification, ease1.value),
                        });

                        if fd > 1 {
                            queue.push_front(DiffOrNothing::Nothing((fd - 1) as u16));
                        }
                    }
                    Easing::InOut(p) => {
                        if p >= 0 {
                            let func = |x: f64| 1. - (1. - x.powi(p)).sqrt();
                            diff_set.push(StateDiff {
                                id: part.id as _,
                                diff: StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (vd as f64 * func(f as f64 / fd as f64)) as i32,
                                ),
                            });
                            for i in (f + 1)..fd {
                                queue.push_front(DiffOrNothing::Diff(StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                                )));
                            }
                        } else {
                            let func = |x: f64| (1. - (1. - x).powi(-p)).sqrt();
                            diff_set.push(StateDiff {
                                id: part.id as _,
                                diff: StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (vd as f64 * func(f as f64 / fd as f64)) as i32,
                                ),
                            });
                            for i in (f + 1)..fd {
                                queue.push_front(DiffOrNothing::Diff(StateDiffVal::new(
                                    part.modification,
                                    ease1.value + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                                )));
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

                        diff_set.push(StateDiff {
                            id: part.id as _,
                            diff: StateDiffVal::new(
                                part.modification,
                                ease1.value + (vd as f64 * func(f as f64 / fd as f64)) as i32,
                            ),
                        });
                        for i in (f + 1)..fd {
                            queue.push_front(DiffOrNothing::Diff(StateDiffVal::new(
                                part.modification,
                                ease1.value + (vd as f64 * func(i as f64 / fd as f64)) as i32,
                            )));
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
                        // let frame = (*current_frame - part.frame_start) % (part.frame_end - part.frame_start) + part.frame_start;
                        *queue = (part_frame..part.eases[high].frame)
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
                                DiffOrNothing::Diff(StateDiffVal::new(
                                    part.modification,
                                    (sum / 4096.) as i32,
                                ))
                            })
                            .rev()
                            .collect();
                        if let Some(DiffOrNothing::Diff(diff)) = queue.pop_back() {
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
        // println!("{diff_set:?}");
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
