pub mod state_gen;
use crate::material::Glow1Material;

use self::state_gen::{Maanim, StateDiff, StateDiffVal, StateDiffs, from_data::StateGenerator};

use super::{*, error::Error};
use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};

pub struct BcuAnim;

// #[derive(Resource)]
// pub struct AnimDB {
//     data: Vec<AnimDBElem>,
// }

/// 使用中の全ユニットの画像とimagecutのデータ
#[derive(Resource)]
pub struct UnitImages {
    images: Vec<Option<UnitImage>>,
}

/// 1ユニットの各パーツのスプライトのID
#[derive(Clone, Resource)]
pub struct UnitSpriteId {
    parts: Vec<PartsEntity>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PartsEntity {
    parent: Entity,
    child: Entity,
}
// #[derive(Clone, Debug, Resource)]
// struct UnitStateTemp(Vec<UnitState>);

#[derive(Clone, Debug, Resource)]
pub struct UnitState {
    states: Vec<State>,
}

use std::{collections::HashMap, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Size2d {
    width: u32,
    height: u32,
}

impl From<Imgcut> for Size2d {
    fn from(value: Imgcut) -> Self {
        Size2d {
            width: value.width,
            height: value.height,
        }
    }
}
/// 1キャラの画像データ
#[derive(Clone)]
pub struct UnitImage {
    materials: Vec<PartMaterialHandle>,
    // glow1_image: HashMap<i32, Handle<Image>>,
    // imgcuts: Vec<Imgcut>,
    size: Vec<Size2d>,
    meshes: Vec<Mesh2dHandle>,
    mamodels: Mamodels,
}

// pub struct AnimDBElem {
//     imgfile: String,
//     imgcuts: Vec<Imgcut>,
//     mamodels: Vec<Mamodel>,
//     maanims: Vec<Maanim>,
// }

#[derive(Clone, Debug, Default)]
pub struct State {
    parent: i32,
    img: i32,
    zorder: i32,
    x: i32,
    y: i32,
    pivotx: i32,
    pivoty: i32,
    scale: i32,
    scalex: i32,
    scaley: i32,
    angle: i32,
    opacity: i32,
    glow: GlowType,
    // 水平方向の反転
    horizontal_flip: bool,
    // 鉛直方向の反転
    vertical_flip: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitForm {
    Form1,
    Form2,
    Form3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSelector {
    Unit((u16, UnitForm)),
    Enemy(u16),
}

impl UnitForm {
    pub fn to_char(self) -> char {
        match self {
            Self::Form1 => 'f',
            Self::Form2 => 'c',
            Self::Form3 => 's',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'f' => Some(Self::Form1),
            'c' => Some(Self::Form2),
            's' => Some(Self::Form3),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimSelector {
    Walk,
    Idle,
    Attack,
    HitBack,
    BurrowDown,
    BurrowMove,
    BurrowUp,
}

impl UnitSelector {
    pub fn unit_type(&self) -> &'static str {
        match self {
            Self::Unit(_) => "unit",
            Self::Enemy(_) => "enemy",
        }
    }

    pub fn id(&self) -> u16 {
        match self {
            Self::Unit((n, _)) => *n,
            Self::Enemy(n) => *n,
        }
    }

    pub fn path_parts_unit((id, form): &(u16, UnitForm)) -> (&'static str, String, char) {
        ("unit", format!("{id:>03}"), form.to_char())
    }

    pub fn path_parts_enemy(id: u16) -> (&'static str, String, char) {
        ("enemy", format!("{id:>03}"), 'e')
    }

    pub fn path(&self) -> String {
        match self {
            Self::Unit(tup) => {
                let (s1, s2, c) = Self::path_parts_unit(tup);
                format!("{0}/{1}/{2}", s1, s2, c)
            }
            Self::Enemy(id) => {
                let (s1, s2, _) = Self::path_parts_enemy(*id);
                format!("{0}/{1}", s1, s2)
            }
        }
    }

    pub fn filename(&self) -> String {
        match self {
            Self::Unit(tup) => {
                let (s1, s2, c) = Self::path_parts_unit(tup);
                format!("{0}/{1}/{2}/{1}_{2}", s1, s2, c)
            }
            Self::Enemy(id) => {
                let (s1, s2, c) = Self::path_parts_enemy(*id);
                format!("{0}/{1}/{1}_{2}", s1, s2, c)
            }
        }
    }

    pub fn image(&self) -> String {
        self.filename() + ".png"
    }

    pub fn imgcuts(&self) -> String {
        self.filename() + ".imgcut"
    }

    pub fn mamodels(&self) -> String {
        self.filename() + ".mamodel"
    }

    pub fn image_size(&self) -> String {
        self.filename() + ".png.size"
    }

    pub fn maanim(&self, selector: AnimSelector) -> String {
        self.filename()
            + match selector {
                AnimSelector::Walk => "00.maanim",
                AnimSelector::Idle => "01.maanim",
                AnimSelector::Attack => "02.maanim",
                AnimSelector::HitBack => "03.maanim",
                AnimSelector::BurrowDown => "_zombie00.maanim",
                AnimSelector::BurrowMove => "_zombie01.maanim",
                AnimSelector::BurrowUp => "_zombie02.maanim",
            }
    }

    pub fn load_imgcut(&self) -> Result<Vec<Imgcut>, Error> {
        Imgcut::load(Path::new(BC_ASSET_PATH).join(self.imgcuts())).map(|(_, v)| v)
    }

    pub fn load_mamodel(&self) -> Result<Mamodels, Error> {
        Mamodels::load(Path::new(BC_ASSET_PATH).join(self.mamodels()))
    }

    pub fn load_maanim(&self, selector: AnimSelector) -> Result<Maanim, Error> {
        Maanim::load(Path::new(BC_ASSET_PATH).join(self.maanim(selector)))
    }
}

use super::{ASSET_PATH, BC_ASSET_PATH};
impl UnitImages {
    fn load(
        id_set: &[UnitSelector],
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
        color_materials: &mut ResMut<Assets<ColorMaterial>>,
        glow_materials: &mut ResMut<Assets<Glow1Material>>,
    ) -> Self {
        Self {
            images: id_set
                .iter()
                .map(|id| {
                    match UnitImage::load(
                        id.clone(),
                        asset_server,
                        meshes,
                        color_materials,
                        glow_materials,
                    ) {
                        Ok(x) => Some(x),
                        Err(err) => {
                            println!(
                                "loading image failed (unit id: {id:?})\nerror info: {err:#?}"
                            );
                            None
                        }
                    }
                })
                .collect(),
        }
    }
}

// fn load_glow_image(
//     selector: UnitSelector,
//     asset_server: &Res<AssetServer>,
//     models: &[Mamodel],
// ) -> HashMap<i32, Handle<Image>> {
//     models
//         .iter()
//         .filter_map(|m| {
//             if m.glow == GlowType::Black {
//                 Some((
//                     m.imgind,
//                     asset_server.load(
//                         Path::new(BC_ASSET_PATH)
//                             .join(selector.filename() + &format!("_{:>03}_glow1.png", m.imgind)),
//                     ),
//                 ))
//             } else {
//                 None
//             }
//         })
//         .collect()
// }

/// ファイルに書き込まれている数字: width + height << 32
fn load_image_size(selector: &UnitSelector) -> Result<(u32, u32), super::error::Error> {
    let s = std::fs::read_to_string(
        Path::new(ASSET_PATH)
            .join(BC_ASSET_PATH)
            .join(selector.image_size()),
    )?;
    let num: u64 = s
        .parse()
        .map_err(|e| super::error::Error::new(super::error::ErrorKind::FileFormatError, e))?;
    Ok((num as u32, (num >> 32) as u32))
}

impl UnitImage {
    fn load(
        selector: UnitSelector,
        asset_server: &Res<AssetServer>,
        meshes: &mut ResMut<Assets<Mesh>>,
        color_materials: &mut ResMut<Assets<ColorMaterial>>,
        glow_materials: &mut ResMut<Assets<Glow1Material>>,
    ) -> Result<Self, super::error::Error> {
        let models = Mamodels::load(Path::new(BC_ASSET_PATH).join(selector.mamodels()))?;
        let imgcuts = Imgcut::load(Path::new(BC_ASSET_PATH).join(selector.imgcuts()))?.1;
        let (w, h) = load_image_size(&selector)?;
        let meshes = imgcuts
            .iter()
            .map(|imgcut| meshes.add(imgcut.mesh(w, h)).into())
            .collect();
        let texture: Handle<Image> =
            asset_server.load(Path::new(BC_ASSET_PATH).join(selector.image()));
        Ok(Self {
            materials: models
                .models
                .iter()
                .map(|model| model.get_material(&texture, color_materials, glow_materials))
                .collect(),
            size: imgcuts.into_iter().map(Size2d::from).collect(),
            meshes,
            mamodels: models,
        })
    }
}

fn space_pressed(input: Res<Input<KeyCode>>) -> bool {
    input.just_pressed(KeyCode::Space)
}

impl State {
    pub fn from_model(model: &Mamodel) -> Self {
        Self {
            parent: model.parent,
            img: model.imgind,
            zorder: model.zorder,
            x: model.posx,
            y: model.posy,
            pivotx: model.pivotx,
            pivoty: model.pivoty,
            scalex: model.scalex,
            scaley: model.scaley,
            angle: model.angle,
            opacity: model.opacity,
            glow: model.glow,
            ..default()
        }
    }

    pub fn load_diff(&mut self, diff: StateDiffVal) {
        match diff {
            StateDiffVal::Parent(v) => self.parent = v,
            StateDiffVal::Sprite(v) => self.img = v,
            StateDiffVal::Zorder(v) => self.zorder = v,
            StateDiffVal::Posx(v) => self.x = v,
            StateDiffVal::Posy(v) => self.y = v,
            StateDiffVal::Pivotx(v) => self.pivotx = v,
            StateDiffVal::Pivoty(v) => self.pivoty = v,
            StateDiffVal::Scale(v) => self.scale = v,
            StateDiffVal::Scalex(v) => self.scalex = v,
            StateDiffVal::Scaley(v) => self.scaley = v,
            StateDiffVal::Angle(v) => self.angle = v,
            StateDiffVal::Opacity(v) => self.opacity = v,
            StateDiffVal::HorizontalFlip(v) => self.horizontal_flip = v,
            StateDiffVal::VerticalFlip(v) => self.vertical_flip = v,
            _ => (),
        };
    }
}

fn startup_sprite_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut glow_materials: ResMut<Assets<Glow1Material>>,
) {
    let unit_id = std::fs::read_to_string("num.txt").unwrap().parse().unwrap();
    let selector = UnitSelector::Unit((unit_id, UnitForm::Form1));
    // 画像関連のデータ
    let image_data = UnitImages::load(
        &[selector],
        &asset_server,
        &mut meshes,
        &mut color_materials,
        &mut glow_materials,
    );

    // println!("loaded: {mamodels:?}");

    // アニメーションロード
    let maanim =
        Maanim::load(Path::new(BC_ASSET_PATH).join(selector.maanim(AnimSelector::Attack))).ok();

    let UnitImage {
        materials: material_handles,
        meshes: mesh_handles,
        size: sizes,
        mamodels,
    } = image_data.images[0].as_ref().unwrap();

    // for (i, model) in mamodels.models.iter().enumerate() {
    //     println!("{i}: {model:?}");
    // }
    // アニメーション定義
    commands.insert_resource(
        maanim
            .map(|anim| StateGenerator::from_anim(anim ,&mamodels))
            .unwrap_or_else(|| StateGenerator::empty(&mamodels)),
    );

    // commands.insert_resource(StateGenerator::with_raw_model(&mamodels));

    commands.spawn(Camera2dBundle::default());
    let parent = commands
        .spawn((
            Unit,
            SpatialBundle {
                transform: Transform::from_xyz(0., -300., 0.),
                ..default()
            },
        ))
        .id();
    let ids = UnitSpriteId {
        parts: material_handles
            .iter()
            .zip(&mamodels.models)
            .enumerate()
            .map(|(i, (mate, model))| {
                let parent = commands
                    .spawn((UnitSpritePartParent, SpatialBundle::default()))
                    .set_parent(parent)
                    .id();

                let size = sizes.get(model.imgind as usize).copied().unwrap_or(Size2d {
                    width: 0,
                    height: 0,
                });
                // let size =
                // if [0, 2, 11, 62].contains(&i) {
                //     sizes.get(model.imgind as usize).copied().unwrap_or(Size2d {
                //         width: 0,
                //         height: 0,
                //     })
                // } else {
                //     Size2d::default()
                // };
                let mesh_temp = Mesh2dHandle::default();
                let mesh = mesh_handles
                    .get(model.imgind as usize)
                    .unwrap_or(&mesh_temp);
                let child = match mate {
                    NormalMaterial(m) => commands.spawn((
                        UnitSpritePartChild,
                        MaterialMesh2dBundle {
                            mesh: mesh.clone(),
                            material: m.clone(),
                            transform: Transform::from_scale(Vec3::new(
                                size.width as f32,
                                size.height as f32,
                                1.,
                            )),
                            ..default()
                        },
                    )),
                    GlowMaterial(m) => commands.spawn((
                        UnitSpritePartChild,
                        MaterialMesh2dBundle {
                            mesh: mesh.clone(),
                            material: m.clone(),
                            transform: Transform::from_scale(Vec3::new(
                                size.width as f32,
                                size.height as f32,
                                1.,
                            )),
                            ..default()
                        },
                    )),
                }
                .set_parent(parent)
                .id();
                PartsEntity { parent, child }
            })
            .collect(),
    };
    commands.insert_resource(ids);
    commands.insert_resource(image_data);
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_unit_sprite(
    mut commands: Commands,
    mut query_parent: Query<
        &mut Transform,
        (With<UnitSpritePartParent>, Without<UnitSpritePartChild>),
    >,
    mut query_child: Query<
        (
            &mut Transform,
            &mut Mesh2dHandle,
            Option<&Handle<ColorMaterial>>,
            Option<&Handle<Glow1Material>>,
        ),
        (With<UnitSpritePartChild>, Without<UnitSpritePartParent>),
    >,
    mut states: ResMut<StateGenerator>,
    image_data: Res<UnitImages>,
    ids: Res<UnitSpriteId>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut glow_materials: ResMut<Assets<Glow1Material>>,
    // input: Res<Input<KeyCode>>,
) {
    update_texture(
        &mut commands,
        &mut query_parent,
        &mut query_child,
        states.next_state(),
        image_data.images[0].as_ref().unwrap(),
        ids.as_ref(),
        &mut color_materials,
        &mut glow_materials,
    );
    // let sprites = state.gen_sprites(&images.images[0].as_ref().unwrap());
    // for (id, (sprite, trans, img, parent)) in ids.parts.iter().zip(sprites) {
    //     if (0..ids.parts.len() as i32).contains(&parent) {
    //         commands.entity(*id).set_parent(ids.parts[parent as usize]);
    //     }
    //     let (mut s, mut t, mut i) = query.get_mut(*id).unwrap();
    //     *s = sprite;
    //     *t = trans;
    //     *i = img;
    // }
}

fn debug_system(
    query: Query<(&Transform), With<UnitSpritePartParent>>,
    ids: Res<UnitSpriteId>,
    // meshes: Res<Assets<Mesh>>,
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Space) {
        // let (t11, t62) = (
        //     query.get(ids.parts[11].parent).unwrap(),
        //     query.get(ids.parts[62].parent).unwrap(),
        // );
        // println!(
        //     "11: {:?}, angle: {}",
        //     t11,
        //     t11.rotation.angle_between(Quat::default()).to_degrees()
        // );
        // println!(
        //     "62: {:?}, angle: {}",
        //     t62,
        //     t62.rotation.angle_between(Quat::default()).to_degrees()
        // );
    }
    // println!("test");
    // if input.just_pressed(KeyCode::Left) {
    //     for mut transform in &mut query {
    //         transform.scale -= Vec3::new(0.2, 0.2, 0.);
    //     }
    // }
    // if input.just_pressed(KeyCode::Right) {
    //     for mut transform in &mut query {
    //         transform.scale += Vec3::new(0.2, 0.2, 0.);
    //     }
    // }
}

fn debug_system2(
    mut query: Query<&mut Transform, With<UnitSpritePartParent>>,
    ids: Res<UnitSpriteId>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::P) {
        for (i, id) in ids.parts.iter().enumerate() {
            println!("{i}: {:#?}", query.get(id.parent));
        }
    }
    // if input.just_pressed(KeyCode::Left) {
    //     for mut transform in &mut query {
    //         transform.scale *= 1.2;
    //     }
    // }
    // if input.just_pressed(KeyCode::Right) {
    //     for mut transform in &mut query {
    //         transform.scale /= 1.2;
    //     }
    // }
}
fn debug_system3(
    mut query: Query<&mut GlobalTransform, With<UnitSpritePartChild>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::C) {
        for transform in &mut query {
            println!("{}", transform.translation().z);
        }
    }
}
fn debug_system4(mut query: Query<&mut Transform, With<Unit>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Left) {
        for mut transform in &mut query {
            transform.scale /= 1.2;
        }
    }
    if input.just_pressed(KeyCode::Right) {
        for mut transform in &mut query {
            transform.scale *= 1.2;
        }
    }
}
pub struct PluginTemp;

impl Plugin for PluginTemp {
    fn build(&self, app: &mut App) {
        let timer = on_timer(Duration::from_secs_f32(1. / 30.));
        app.add_startup_system(startup_sprite_images)
            .add_system(update_unit_sprite.run_if(timer))
            .add_system(debug_system);
    }
}
pub struct UnitSpriteIter<'a> {
    itr_state: std::slice::Iter<'a, State>,
    images: &'a UnitImage,
    opacities: Vec<f32>,
    zorders: Vec<i32>,
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct UnitSpritePartChild;

#[derive(Component)]
pub struct UnitSpritePartParent;

impl UnitState {
    pub fn gen_sprites<'a>(&'a self, images: &'a UnitImage) -> UnitSpriteIter<'a> {
        UnitSpriteIter {
            itr_state: self.states.iter(),
            images,
            opacities: Vec::with_capacity(self.states.len()),
            zorders: Vec::with_capacity(self.states.len()),
        }
    }

    pub fn load_diff(&mut self, diff: StateDiffs) {
        diff.0.iter().for_each(|diff| {
            let StateDiff { id, diff } = *diff;
            self.states[id as usize].load_diff(diff);
        });
    }

    pub fn from_model(models: &Mamodels) -> Self {
        Self {
            states: models
                .models
                .iter()
                .map(|model| State {
                    x: 0,
                    y: 0,
                    pivotx: 0,
                    pivoty: 0,
                    scale: models.scale_ratio as _,
                    scalex: models.scale_ratio as _,
                    scaley: models.scale_ratio as _,
                    angle: 0,
                    opacity: models.opacity_ratio as _,
                    ..State::from_model(model)
                })
                .collect(),
        }
    }

    pub fn apply_model(&mut self, models: &Mamodels) {
        self.states
            .iter_mut()
            .zip(&models.models)
            .for_each(|(state, model)| {
                state.x += model.posx;
                state.y += model.posy;
                state.pivotx += model.pivotx;
                state.pivoty += model.pivoty;
                state.scalex *= model.scalex;
                state.scaley *= model.scaley;
                state.angle += model.angle;
                state.opacity *= model.opacity;
            });
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_texture(
    commands: &mut Commands,
    query_parent: &mut Query<
        &mut Transform,
        (With<UnitSpritePartParent>, Without<UnitSpritePartChild>),
    >,
    query_child: &mut Query<
        (
            &mut Transform,
            &mut Mesh2dHandle,
            Option<&Handle<ColorMaterial>>,
            Option<&Handle<Glow1Material>>,
        ),
        (With<UnitSpritePartChild>, Without<UnitSpritePartParent>),
    >,
    mut states: UnitState,
    image_data: &UnitImage,
    ids: &UnitSpriteId,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    glow_materials: &mut ResMut<Assets<Glow1Material>>,
) {
    states.apply_model(&image_data.mamodels);
    let mut opacities: Vec<Option<(f32, bool)>> = vec![None; states.states.len()];
    for (state, id) in states.states.iter().zip(&ids.parts).skip(1) {
        commands.entity(id.parent).remove_parent();
    }
    println!("state39: {:?}", states.states[39]);
    for (i, (state, id)) in states.states.iter().zip(&ids.parts).enumerate() {
        let opacity;
        let zorder;
        let mesh;

        let opacity_ratio = (image_data.mamodels.opacity_ratio as f32).powi(2);
        let scale_ratio = (image_data.mamodels.scale_ratio as f32).powi(3);
        let angle_ratio = image_data.mamodels.angle_ratio as f32;
        let mut angle_direction = 1.;
        if i > 0 {
            commands
                .entity(id.parent)
                .set_parent(ids.parts[state.parent as usize].parent);
            mesh = match image_data.meshes.get(state.img as usize) {
                Some(m) => m.clone(),
                None => Mesh2dHandle::default(),
            };
            zorder = state.zorder - states.states[state.parent as usize].zorder;
        } else {
            mesh = Mesh2dHandle::default();
            zorder = state.zorder;
        }

        if state.parent < 0 {
            opacity = state.opacity as f32 / opacity_ratio;
            opacities[i] = Some((
                opacity,
                state.scalex.is_positive() ^ state.scaley.is_positive(),
            ));
        } else if let Some((opa, sig)) = opacities[state.parent as usize] {
            opacity = opa * state.opacity as f32 / opacity_ratio;
            if sig {
                angle_direction = -angle_direction;
            }
            opacities[i] = Some((
                opacity,
                state.scalex.is_positive()
                    ^ state.scaley.is_positive()
                    // ^ state.vertical_flip
                    // ^ state.horizontal_flip
                    ^ sig,
            ));
        } else {
            let mut parent_ind = state.parent as usize;
            let mut indice = vec![i];
            while let None = opacities[parent_ind] {
                indice.push(parent_ind);
                parent_ind = states.states[parent_ind].parent as usize;
            }
            let mut prev_ind = parent_ind;
            for i in indice.iter().rev().copied() {
                opacities[i] = opacities[prev_ind].map(|(opa, sig)| {
                    let state = &states.states[i];
                    (
                        opa * states.states[i].opacity as f32 / opacity_ratio,
                        state.scalex.is_positive()
                            ^ state.scaley.is_positive()
                            // ^ state.vertical_flip
                            // ^ state.horizontal_flip
                            ^ sig,
                    )
                });
                prev_ind = i;
            }
            opacity = opacities[i].unwrap().0;
            if opacities[state.parent as usize].unwrap().1 {
                angle_direction = -angle_direction;
            }
        }

        let size = image_data
            .size
            .get(state.img as usize)
            .copied()
            .unwrap_or_default();
        let child_translation = Vec3::new(
            size.width as f32 / 2. - state.pivotx as f32,
            state.pivoty as f32 - size.height as f32 / 2.,
            0.,
        );

        let scalex = if state.horizontal_flip {
            -state.scalex as f32
        } else {
            state.scalex as f32
        } * state.scale as f32
            / scale_ratio;
        let scaley = if state.vertical_flip {
            -state.scaley as f32
        } else {
            state.scaley as f32
        } * state.scale as f32
            / scale_ratio;
        
        let mut angle = -state.angle as f32 / angle_ratio * 2. * std::f32::consts::PI;
        if state.horizontal_flip {
            angle = -angle;
        }
        if state.vertical_flip {
            angle = -angle;
        }
        let parent_transform = Transform::from_xyz(state.x as f32, -state.y as f32, zorder as f32 + i as f32 / states.states.len() as f32)
            .with_rotation(Quat::from_rotation_z(angle * angle_direction))
            .with_scale(Vec3::new(scalex, scaley, 1.));

        *query_parent.get_mut(id.parent).unwrap() = parent_transform;

        let (mut transform, mut mesh_handle, mate1, mate2) = query_child.get_mut(id.child).unwrap();

        transform.translation = child_translation;
        *mesh_handle = mesh;

        if let Some(material) = mate1 {
            color_materials
                .get_mut(material)
                .unwrap()
                .color
                .set_a(opacity);
        } else if let Some(material) = mate2 {
            glow_materials
                .get_mut(material)
                .unwrap()
                .color
                .set_a(opacity);
        }
        // println!("update");
    }
    // println!("opacities: {opacities:#?}");
}

// impl<'a> Iterator for UnitSpriteIter<'a> {
//     type Item = (Sprite, Transform, Handle<Image>, i32);
//     fn next(&mut self) -> Option<Self::Item> {
//         let Self {
//             itr_state,
//             images,
//             opacities,
//             zorders,
//         } = self;
//         itr_state.next().map(|state| {
//             if (0..opacities.len() as i32).contains(&state.parent) {
//                 let opacity = opacities[state.parent as usize] * state.opacity as f32 / 1000.;
//                 let zorder = state.zorder - zorders[state.parent as usize];
//                 let imgcut = &images.imgcuts[state.img as usize];
//                 let (rect, img) = if state.glow == GlowType::Black {
//                     (None, images.glow1_image[&state.img].clone())
//                 } else {
//                     (Some(imgcut.into_rect()), images.image.clone())
//                 };
//                 opacities.push(opacity);
//                 zorders.push(state.zorder);
//                 (
//                     Sprite {
//                         color: Color::rgba(1., 1., 1., opacity),
//                         flip_x: state.horizontal_flip,
//                         flip_y: state.vertical_flip,
//                         rect,
//                         anchor: Anchor::Custom(Vec2::new(
//                             state.pivotx as f32 / imgcut.width as f32 - 0.5,
//                             0.5 - state.pivoty as f32 / imgcut.height as f32,
//                         )),
//                         ..default()
//                     },
//                     Transform::from_xyz(state.x as f32, -state.y as f32, zorder as f32)
//                         .with_rotation(Quat::from_rotation_z(-f32::to_radians(
//                             state.angle as f32 / 10.,
//                         )))
//                         .with_scale(Vec3::new(
//                             state.scalex as f32 / 1000.,
//                             state.scaley as f32 / 1000.,
//                             1.,
//                         )),
//                     img,
//                     state.parent,
//                 )
//             } else {
//                 let opacity = state.opacity as f32 / 1000.;
//                 let imgcut = &images.imgcuts[state.img as usize];
//                 opacities.push(opacity);
//                 zorders.push(state.zorder);
//                 (
//                     Sprite {
//                         color: Color::rgba(1., 1., 1., opacity),
//                         flip_x: state.horizontal_flip,
//                         flip_y: state.vertical_flip,
//                         rect: Some(imgcut.into_rect()),
//                         anchor: Anchor::Custom(Vec2::new(
//                             state.pivotx as f32 / imgcut.width as f32 - 0.5,
//                             0.5 - state.pivoty as f32 / imgcut.height as f32,
//                         )),
//                         ..default()
//                     },
//                     Transform::from_xyz(state.x as f32, -state.y as f32, state.zorder as f32)
//                         .with_rotation(Quat::from_rotation_z(-f32::to_radians(
//                             state.angle as f32 / 10.,
//                         )))
//                         .with_scale(Vec3::new(
//                             state.scalex as f32 / 1000.,
//                             state.scaley as f32 / 1000.,
//                             1.,
//                         )),
//                     images.image.clone(),
//                     state.parent,
//                 )
//             }
//         })
//     }
// }
#[derive(Component)]
pub struct AnimTemp {
    dyn_id: i32,
    meta: Vec<ModelMeta>,
}

pub struct ModelMeta {
    x: f32,
    y: f32,
}

impl Plugin for BcuAnim {
    fn build(&self, app: &mut App) {}
}
