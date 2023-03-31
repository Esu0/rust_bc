#![allow(unused)]
pub mod animation;
pub mod spawn;
use bevy::prelude::*;

#[derive(Component)]
pub struct Unit {
    id: i32,
}

pub struct BattleCatsDB {}

#[derive(Debug, Clone)]
pub struct Imgcut {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Imgcut {
    pub fn rect(&self) -> Rect {
        let (x2, y2) = (self.x + self.width, self.y + self.height);
        Rect::new(self.x as f32, self.y as f32, x2 as f32, y2 as f32)
    }

    pub fn mesh(&self, width: u32, height: u32) -> Mesh {
        let mut mesh = Mesh::from(shape::Quad::default());
        let (x, y) = (self.x as f32 / width as f32, self.y as f32 / height as f32);
        let x2 = (self.x + self.width) as f32 / width as f32;
        let y2 = (self.y + self.height) as f32 / height as f32;
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[x, y2], [x, y], [x2, y], [x2, y2]],
        );
        mesh
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlowType {
    #[default]
    None,
    Black,
    White,
    Inverse,
}

impl From<i32> for GlowType {
    fn from(value: i32) -> Self {
        match value {
            1 | 3 => Self::Black,
            2 => Self::White,
            -1 => Self::Inverse,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mamodel {
    parent: i32,
    imgind: i32,
    zorder: i32,
    posx: i32,
    posy: i32,
    pivotx: i32,
    pivoty: i32,
    scalex: i32,
    scaley: i32,
    angle: i32,
    opacity: i32,
    glow: GlowType,
}



use error::ErrorKind;


use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;
use std::str::{FromStr, Split};

use crate::material::Glow1Material;

/// io関連のエラーは引継ぎ、関数fの戻り値がfalseだったらFileFormatErrorとする
fn consume_buf(
    itr: &mut Lines<BufReader<File>>,
    f: impl FnOnce(&String) -> bool,
) -> Result<(), error::Error> {
    if let Some(buf) = itr.next() {
        let s = buf?;
        // println!("{}", s);
        // println!("{}", s.find("[imgcut]").is_some());
        if f(&s) {
            // println!("ok");
            Ok(())
        } else {
            // println!("error");
            Err(error::ErrorKind::FileFormatError.into())
        }
    } else {
        // println!("none");
        Err(error::ErrorKind::FileFormatError.into())
    }
}

fn get_string(itr: &mut Lines<BufReader<File>>) -> Result<String, error::Error> {
    match itr.next() {
        Some(buf) => buf.map_err(|err| err.into()),
        None => Err(error::ErrorKind::FileFormatError.into()),
    }
}

fn get_next_line<T: FromStr<Err = impl Into<Box<dyn std::error::Error + Send + Sync>>>>(
    itr: &mut Lines<BufReader<File>>,
) -> Result<T, error::Error> {
    match itr.next() {
        Some(buf) => {
            let s = buf?;
            s.parse()
                .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err))
        }
        None => Err(error::Error::new(
            ErrorKind::FileFormatError,
            "ファイルの終端に到達",
        )),
    }
}

const ASSET_PATH: &str = "assets";
const BC_ASSET_PATH: &str = "org";

impl Imgcut {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(String, Vec<Self>), error::Error> {
        let f = File::open(Path::new(ASSET_PATH).join(path))?;
        let reader = BufReader::new(f);
        let mut itr = reader.lines();

        // println!("get header");
        consume_buf(&mut itr, |s| s.starts_with("[imgcut]"))?;
        // println!("ignore");
        consume_buf(&mut itr, |_| true)?;

        // println!("get filename");
        let filename = get_string(&mut itr)?;
        // println!("get amount");
        let amount = get_string(&mut itr)?
            .parse::<usize>()
            .map_err(|_| error::Error::from(error::ErrorKind::FileFormatError))?;
        let mut v = Vec::with_capacity(amount);
        for _ in 0..amount {
            let s = get_string(&mut itr)?;
            let mut split = s.split(',');
            v.push(Imgcut {
                x: split
                    .next()
                    .ok_or(error::Error::from_kind(ErrorKind::FileFormatError))?
                    .parse()
                    .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err))?,
                y: split
                    .next()
                    .ok_or(error::Error::from_kind(ErrorKind::FileFormatError))?
                    .parse()
                    .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err))?,
                width: split
                    .next()
                    .ok_or(error::Error::from_kind(ErrorKind::FileFormatError))?
                    .parse()
                    .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err))?,
                height: split
                    .next()
                    .ok_or(error::Error::from_kind(ErrorKind::FileFormatError))?
                    .parse()
                    .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err))?,
            });
        }
        Ok((filename, v))
    }
}

fn get_next<T: FromStr<Err = impl Into<Box<dyn std::error::Error + Send + Sync>>>>(
    itr: &mut Split<char>,
) -> Result<T, error::Error> {
    match itr.next() {
        Some(s) => s
            .parse()
            .map_err(|err| error::Error::new(ErrorKind::FileFormatError, err)),
        None => Err(error::Error::new(
            ErrorKind::FileFormatError,
            "ファイルの終端に到達",
        )),
    }
}



#[derive(Clone, Debug)]
pub enum PartMaterialHandle {
    NormalMaterial(Handle<ColorMaterial>),
    GlowMaterial(Handle<Glow1Material>),
}

pub use PartMaterialHandle::*;

#[derive(Clone, Debug)]
pub struct Mamodels {
    models: Vec<Mamodel>,
    scale_ratio: u32,
    angle_ratio: u32,
    opacity_ratio: u32,
}

impl Mamodels {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, error::Error> {
        let f = File::open(Path::new(ASSET_PATH).join(path))?;
        let reader = BufReader::new(f);
        let mut itr = reader.lines();

        consume_buf(&mut itr, |s| {
            s.starts_with("[modelanim:model]") || s.starts_with("[modelanim:model2]")
        })?;
        consume_buf(&mut itr, |_| true)?;

        let length = get_next_line::<usize>(&mut itr)?;
        let mut v = Vec::with_capacity(length);

        for _ in 0..length {
            let s = get_string(&mut itr)?;
            let mut split = s.split(',');
            v.push(Mamodel {
                parent: get_next(&mut split)?,
                imgind: {
                    split.next();
                    get_next(&mut split)?
                },
                zorder: get_next(&mut split)?,
                posx: get_next(&mut split)?,
                posy: get_next(&mut split)?,
                pivotx: get_next(&mut split)?,
                pivoty: get_next(&mut split)?,
                scalex: get_next(&mut split)?,
                scaley: get_next(&mut split)?,
                angle: get_next(&mut split)?,
                opacity: get_next(&mut split)?,
                glow: get_next::<i32>(&mut split)?.into(),
            });
        }
        let s = get_string(&mut itr)?;
        let mut split = s.split(',');
        let scale_ratio: u32 = get_next(&mut split)?;
        let angle_ratio: u32 = get_next(&mut split)?;
        let opacity_ratio: u32 = get_next(&mut split)?;
        Ok(Mamodels {
            models: v,
            scale_ratio,
            angle_ratio,
            opacity_ratio,
        })
    }
}

impl Mamodel {
    pub fn get_material(
        &self,
        image_handle: &Handle<Image>,
        materials1: &mut ResMut<Assets<ColorMaterial>>,
        materials2: &mut ResMut<Assets<Glow1Material>>,
    ) -> PartMaterialHandle {
        if self.glow == GlowType::Black {
            GlowMaterial(materials2.add(Glow1Material::from(image_handle.clone())))
        } else {
            NormalMaterial(materials1.add(ColorMaterial::from(image_handle.clone())))
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn imgcut() {
        println!(
            "{:?}",
            Imgcut::load("assets/org/unit/697/s/697_s.imgcut").expect("失敗")
        )
    }

    #[test]
    fn mamodel() {
        println!(
            "{:?}",
            Mamodels::load("assets/org/unit/697/s/697_s.mamodel").unwrap()
        );
    }

    

    use super::animation::{UnitForm, UnitSelector};
    use std::fs;
    use std::path::Path;
    #[test]
    fn load_maanim() {
        let unit_path = Path::new("assets/org/unit");
        let mut counter = 0;
        for i in 0..=697 {
            for c in ['f', 's', 'c'] {
                let path = unit_path.join(format!("{0:>03}/{c}/{0:>03}_{c}.mamodel", i));
                if let Ok(models) = Mamodels::load(path) {
                    if let Some(model) = models.models.iter().find(|elem| elem.glow == GlowType::Inverse) {
                        println!("({i}, {c}): glow: {:?}", model.glow);
                    }
                }
            }
        }
    }

    
}

mod error {
    #[derive(Debug, Clone, Copy)]
    pub enum ErrorKind {
        IOError,
        FileFormatError,
        InvalidNumber,
    }

    impl ErrorKind {
        pub fn msg(self) -> &'static str {
            match self {
                ErrorKind::IOError => "ファイルを開けなかった",
                ErrorKind::FileFormatError => "ファイルのフォーマットが正しくない",
                ErrorKind::InvalidNumber => "無効な数が指定された",
            }
        }
    }

    use std::error;
    #[derive(Debug)]
    enum _Error {
        Simple(ErrorKind),
        Custom((ErrorKind, Box<dyn error::Error + Send + Sync>)),
    }

    pub struct Error {
        _error: _Error,
    }

    impl Error {
        pub fn new<E>(kind: ErrorKind, error: E) -> Self
        where
            E: Into<Box<dyn error::Error + Send + Sync>>,
        {
            Error {
                _error: _Error::Custom((kind, error.into())),
            }
        }

        pub const fn from_kind(kind: ErrorKind) -> Self {
            Error {
                _error: _Error::Simple(kind),
            }
        }

        pub fn kind(&self) -> ErrorKind {
            match &self._error {
                _Error::Simple(k) => *k,
                _Error::Custom(k) => k.0,
            }
        }
    }

    use std::fmt;
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self._error {
                _Error::Simple(kind) => f.write_str(kind.msg()),
                _Error::Custom((kind, source)) => {
                    f.write_str(kind.msg())?;
                    write!(f, "\n{}", source)
                }
            }
        }
    }

    impl fmt::Debug for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self._error {
                _Error::Simple(kind) => f.write_str(kind.msg()),
                _Error::Custom((kind, source)) => {
                    f.write_str(kind.msg())?;
                    write!(f, "\n{:?}", source)
                }
            }
        }
    }

    impl error::Error for Error {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match &self._error {
                _Error::Simple(_) => None,
                _Error::Custom(c) => c.1.source(),
            }
        }
    }

    impl From<ErrorKind> for Error {
        fn from(kind: ErrorKind) -> Self {
            Error {
                _error: _Error::Simple(kind),
            }
        }
    }

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            Error {
                _error: _Error::Custom((ErrorKind::IOError, err.into())),
            }
        }
    }

    // impl From<Error> for Box<dyn std::error::Error + Send + Sync> {
    //     fn from(value: Error) -> Self {
    //         Box::new(value)
    //     }
    // }
}
