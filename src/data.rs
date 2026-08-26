//! Game database loading: `assets/data/gamedata.json` + `assets/data/anchors2.json`.
//! These are the tables extracted from the original `pack/serbi.as`.

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

// ---------- gamedata.json ----------

#[derive(Debug, Deserialize, Clone)]
pub struct BoothDef {
    pub nama: Option<String>,
    #[serde(rename = "ROWS")]
    pub rows: i32,
    #[serde(rename = "COLS")]
    pub cols: i32,
    pub fr: u32,
    pub wall: Option<u32>,
    pub wall_alpha: Option<u32>,
    pub blok: u32,
    pub jenis: Option<String>,
    #[serde(default)]
    pub price: Vec<f64>,
    /// In gamedata this is `0` for Lobby but a per-level list for most booths.
    #[serde(default, deserialize_with = "num_or_list")]
    pub booked_price: Vec<f64>,
    #[serde(default)]
    pub opened: f64,
    #[serde(default)]
    pub closed: f64,
    #[serde(default)]
    pub salary: Vec<f64>,
    #[serde(default)]
    pub pop: Vec<f64>,
}

/// Accept either a bare number or a list of numbers (gamedata mixes both).
fn num_or_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrList {
        Num(f64),
        List(Vec<f64>),
    }
    Ok(match NumOrList::deserialize(d)? {
        NumOrList::Num(n) => vec![n],
        NumOrList::List(v) => v,
    })
}

#[derive(Debug, Deserialize, Clone)]
pub struct TileDef {
    pub nama: Option<String>,
    pub fr: u32,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub walkable: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneryDef {
    pub nama: Option<String>,
    pub fr: u32,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub pop: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExpandDef {
    #[serde(rename = "ROWS")]
    pub rows: i32,
    #[serde(rename = "COLS")]
    pub cols: i32,
    pub lahan: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct GameData {
    #[serde(rename = "Booth")]
    pub booth: HashMap<String, serde_json::Value>,
    #[serde(rename = "Tile")]
    pub tile: HashMap<String, serde_json::Value>,
    #[serde(rename = "Scenery")]
    pub scenery: HashMap<String, serde_json::Value>,
    #[serde(rename = "Visitor")]
    pub visitor: HashMap<String, serde_json::Value>,
    #[serde(rename = "Expand")]
    pub expand: Vec<ExpandDef>,
    #[serde(rename = "ExpandPrice")]
    pub expand_price: Vec<f64>,
}

impl GameData {
    pub fn booth_def(&self, name: &str) -> Option<BoothDef> {
        serde_json::from_value(self.booth.get(name)?.clone()).ok()
    }
    pub fn tile_def(&self, jenis: u32) -> Option<TileDef> {
        serde_json::from_value(self.tile.get(&format!("TILE_{jenis}"))?.clone()).ok()
    }
    pub fn scenery_def(&self, jenis: u32) -> Option<SceneryDef> {
        serde_json::from_value(self.scenery.get(&format!("PLANT_{jenis}"))?.clone()).ok()
    }
    pub fn visitor_speed(&self, skin: u32) -> Option<f32> {
        self.visitor
            .get(&format!("Visitor{skin}"))?
            .get("speed")?
            .as_f64()
            .map(|v| v as f32)
    }
}

// ---------- anchors2.json ----------

#[derive(Debug, Deserialize, Clone)]
pub struct TileAnchor {
    pub anchor: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Deserialize, Clone)]
pub struct BoothAnchor {
    pub anchor: [f32; 2],
    pub rows: i32,
    pub cols: i32,
    pub fr: u32,
    pub blok: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SceneryAnchor {
    pub fr: u32,
    pub base: [f32; 2],
}

/// Wall/building sprite drawn on top of the booth floor. `s_vertex` is the
/// pixel (top-left origin) that must sit on the S vertex of the footprint.
#[derive(Debug, Deserialize, Clone)]
pub struct WallAnchor {
    pub booth: String,
    pub kind: String,
    pub s_vertex: [f32; 2],
    pub size: [f32; 2],
    /// Pixel of the wall-base N corner (intersection of the two wall base
    /// edges); must sit on the footprint's N vertex, same as the floor anchor.
    pub n_corner: [f32; 2],
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct Anchors {
    pub tile: TileAnchor,
    pub booth: HashMap<String, BoothAnchor>,
    pub wall: HashMap<String, WallAnchor>,
    pub scenery: HashMap<String, SceneryAnchor>,
}

// ---------- JSON asset loader ----------

pub struct JsonLoader<T>(std::marker::PhantomData<T>);

impl<T> Default for JsonLoader<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: Asset + for<'de> Deserialize<'de>> AssetLoader for JsonLoader<T> {
    type Asset = T;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<T, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

// ---------- plugin: load both files, expose handles ----------

#[derive(Resource)]
pub struct DataHandles {
    pub gamedata: Handle<GameData>,
    pub anchors: Handle<Anchors>,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Playing,
}

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_asset::<GameData>()
            .init_asset::<Anchors>()
            .register_asset_loader(JsonLoader::<GameData>::default())
            .register_asset_loader(JsonLoader::<Anchors>::default())
            .add_systems(Startup, load_data)
            .add_systems(Update, check_loaded.run_if(in_state(AppState::Loading)));
    }
}

fn load_data(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(DataHandles {
        gamedata: assets.load("data/gamedata.json"),
        anchors: assets.load("data/anchors2.json"),
    });
}

fn check_loaded(
    handles: Res<DataHandles>,
    gamedata: Res<Assets<GameData>>,
    anchors: Res<Assets<Anchors>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if gamedata.get(&handles.gamedata).is_some() && anchors.get(&handles.anchors).is_some() {
        next.set(AppState::Playing);
    }
}
