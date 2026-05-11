#![forbid(unsafe_code)]
//! `ph2d-asset-cooker` — JSON5 → postcard cooker for Prefab / Scene
//! assets (ADR-0025 M14.3a).
//!
//! Source files are JSON5 with a typed schema (`SourcePrefab` /
//! `SourceScene`). The cooker:
//!
//! 1. Parses the JSON5 with `serde_json5` into the typed source
//!    shape.
//! 2. For each component, looks up its canonical name in a built-in
//!    dispatch table (`cook_component`), deserializes the embedded
//!    JSON value into the typed `Component`, then postcard-encodes
//!    the bytes.
//! 3. Computes the cooked `type_id` via
//!    [`ph2d_ecs::scene::stable_type_id`] — blake3(name)[..8].
//! 4. Encodes the final [`PrefabDoc`] / [`SceneDoc`] with postcard.
//!
//! The output is byte-deterministic given identical input: postcard
//! is a deterministic format, `BTreeMap` iteration is alphabetical,
//! and no time/random/hash-seed enters the pipeline (HR-5 + HR-6).

use ph2d_asset::{
    ChildOfPair, ComponentBlob, PrefabDoc, PrefabInstance, PrefabRef, SceneDoc,
};
use ph2d_ecs::scene::stable_type_id;
use serde::{Deserialize, Serialize};

// Re-export AssetId for the binary so it can hex-decode prefab
// references in scene files.
pub use ph2d_asset::AssetId;

#[derive(Debug)]
pub enum CookError {
    Json(String),
    UnknownComponent(String),
    Postcard(postcard::Error),
    InvalidAssetId(String),
}

impl std::fmt::Display for CookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(s) => write!(f, "cooker: JSON5 parse error: {s}"),
            Self::UnknownComponent(s) => write!(
                f,
                "cooker: unknown component canonical name '{s}'. \
                 Register it in `tools/asset-cooker/src/lib.rs::cook_component` \
                 and `ph2d_*::register_*_components` (HR-10)."
            ),
            Self::Postcard(e) => write!(f, "cooker: postcard encode error: {e}"),
            Self::InvalidAssetId(s) => {
                write!(f, "cooker: AssetId must be 64-char hex, got: {s}")
            }
        }
    }
}

impl std::error::Error for CookError {}

/// JSON5 wire shape for a single component:
/// ```json5
/// { type: "ph2d::ecs::Transform", data: { translation: [0,0], rotation: 0, scale: [1,1] } }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceComponent {
    #[serde(rename = "type")]
    pub type_name: String,
    pub data: serde_json::Value,
}

/// JSON5 wire shape for a child reference:
/// ```json5
/// { prefab: "<64-char-hex>", overrides: [ ...components... ] }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourcePrefabRef {
    pub prefab: String,
    #[serde(default)]
    pub overrides: Vec<SourceComponent>,
}

/// JSON5 wire shape for a prefab source file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourcePrefab {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub components: Vec<SourceComponent>,
    #[serde(default)]
    pub children: Vec<SourcePrefabRef>,
}

fn default_version() -> u32 {
    1
}

/// JSON5 wire shape for one scene instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourcePrefabInstance {
    pub prefab: String,
    #[serde(default)]
    pub overrides: Vec<SourceComponent>,
}

/// JSON5 wire shape for one parent/child relation.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SourceChildOfPair {
    pub parent_index: u32,
    pub child_index: u32,
}

/// JSON5 wire shape for a scene source file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceScene {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub instances: Vec<SourcePrefabInstance>,
    #[serde(default)]
    pub relations: Vec<SourceChildOfPair>,
}

/// Cook one component spec: dispatch by `type_name`, decode the JSON
/// into the typed component via that crate's `Deserialize` impl,
/// postcard-encode the result.
pub fn cook_component(spec: &SourceComponent) -> Result<ComponentBlob, CookError> {
    // The dispatch table is intentionally explicit — see ADR-0025
    // M14.3a "ComponentRegistry strategy = manual": each known
    // component is enumerated here so a `grep` for the canonical
    // name resolves to the cook path.
    match spec.type_name.as_str() {
        "ph2d::ecs::Transform" => {
            cook_typed::<ph2d_ecs::Transform>(&spec.type_name, &spec.data)
        }
        "ph2d::ecs::Name" => cook_typed::<ph2d_ecs::Name>(&spec.type_name, &spec.data),
        "ph2d::render::Sprite" => {
            cook_typed::<ph2d_render::Sprite>(&spec.type_name, &spec.data)
        }
        "ph2d::script::LuauScript" => {
            cook_typed::<ph2d_script::LuauScript>(&spec.type_name, &spec.data)
        }
        other => Err(CookError::UnknownComponent(other.to_owned())),
    }
}

fn cook_typed<T: serde::de::DeserializeOwned + serde::Serialize>(
    name: &str,
    data: &serde_json::Value,
) -> Result<ComponentBlob, CookError> {
    let value: T = serde_json::from_value(data.clone())
        .map_err(|e| CookError::Json(format!("{name}: {e}")))?;
    let bytes = postcard::to_allocvec(&value).map_err(CookError::Postcard)?;
    Ok(ComponentBlob {
        type_id: stable_type_id(name),
        data: bytes,
    })
}

fn parse_asset_id(hex: &str) -> Result<AssetId, CookError> {
    if hex.len() != 64 {
        return Err(CookError::InvalidAssetId(hex.to_owned()));
    }
    let mut digest = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
        let s = std::str::from_utf8(chunk)
            .map_err(|_| CookError::InvalidAssetId(hex.to_owned()))?;
        digest[i] = u8::from_str_radix(s, 16)
            .map_err(|_| CookError::InvalidAssetId(hex.to_owned()))?;
    }
    Ok(AssetId::from_digest(digest))
}

/// Cook a JSON5 prefab source into a [`PrefabDoc`].
pub fn cook_prefab(src: &SourcePrefab) -> Result<PrefabDoc, CookError> {
    let mut components = Vec::with_capacity(src.components.len());
    for c in &src.components {
        components.push(cook_component(c)?);
    }
    let mut children = Vec::with_capacity(src.children.len());
    for r in &src.children {
        let prefab = parse_asset_id(&r.prefab)?;
        let mut overrides = Vec::with_capacity(r.overrides.len());
        for c in &r.overrides {
            overrides.push(cook_component(c)?);
        }
        children.push(PrefabRef { prefab, overrides });
    }
    Ok(PrefabDoc {
        version: src.version,
        components,
        children,
    })
}

/// Cook a JSON5 scene source into a [`SceneDoc`].
pub fn cook_scene(src: &SourceScene) -> Result<SceneDoc, CookError> {
    let mut instances = Vec::with_capacity(src.instances.len());
    for i in &src.instances {
        let prefab = parse_asset_id(&i.prefab)?;
        let mut overrides = Vec::with_capacity(i.overrides.len());
        for c in &i.overrides {
            overrides.push(cook_component(c)?);
        }
        instances.push(PrefabInstance { prefab, overrides });
    }
    let relations = src
        .relations
        .iter()
        .map(|r| ChildOfPair {
            parent_index: r.parent_index,
            child_index: r.child_index,
        })
        .collect();
    Ok(SceneDoc {
        version: src.version,
        instances,
        relations,
    })
}

/// Top-level entry: parse JSON5 source → cook → postcard-encode the
/// resulting `PrefabDoc`. Output bytes are content-addressable via
/// `blake3` (HR-6).
pub fn cook_prefab_json5(source: &str) -> Result<Vec<u8>, CookError> {
    let src: SourcePrefab = serde_json5::from_str(source)
        .map_err(|e| CookError::Json(format!("prefab: {e}")))?;
    let doc = cook_prefab(&src)?;
    postcard::to_allocvec(&doc).map_err(CookError::Postcard)
}

/// Top-level entry: parse JSON5 scene source → cook → postcard.
pub fn cook_scene_json5(source: &str) -> Result<Vec<u8>, CookError> {
    let src: SourceScene = serde_json5::from_str(source)
        .map_err(|e| CookError::Json(format!("scene: {e}")))?;
    let doc = cook_scene(&src)?;
    postcard::to_allocvec(&doc).map_err(CookError::Postcard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cook_empty_prefab_is_v1() {
        let src = "{ version: 1 }";
        let bytes = cook_prefab_json5(src).unwrap();
        let decoded: PrefabDoc = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.version, 1);
        assert!(decoded.components.is_empty());
        assert!(decoded.children.is_empty());
    }

    #[test]
    fn cook_transform_component_round_trips() {
        let src = r#"{
            version: 1,
            components: [
                { type: "ph2d::ecs::Transform", data: { translation: [3.0, 4.0], rotation: 1.5, scale: [2.0, 2.0] } }
            ]
        }"#;
        let bytes = cook_prefab_json5(src).unwrap();
        let decoded: PrefabDoc = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.components.len(), 1);
        let blob = &decoded.components[0];
        assert_eq!(blob.type_id, stable_type_id("ph2d::ecs::Transform"));
        // Decode the blob back to a Transform and verify values.
        let t: ph2d_ecs::Transform = postcard::from_bytes(&blob.data).unwrap();
        assert_eq!(t.translation, ph2d_core::Vec2::new(3.0, 4.0));
        assert_eq!(t.rotation, 1.5);
        assert_eq!(t.scale, ph2d_core::Vec2::new(2.0, 2.0));
    }

    #[test]
    fn cook_unknown_component_errors() {
        let src = r#"{
            version: 1,
            components: [
                { type: "ph2d::ecs::Unknown", data: {} }
            ]
        }"#;
        let err = cook_prefab_json5(src).unwrap_err();
        match err {
            CookError::UnknownComponent(n) => assert_eq!(n, "ph2d::ecs::Unknown"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn cooker_is_deterministic_across_runs() {
        let src = r#"{
            version: 1,
            components: [
                { type: "ph2d::ecs::Name", data: "Boss" },
                { type: "ph2d::ecs::Transform", data: { translation: [1.0, 2.0], rotation: 0.0, scale: [1.0, 1.0] } }
            ]
        }"#;
        let a = cook_prefab_json5(src).unwrap();
        let b = cook_prefab_json5(src).unwrap();
        assert_eq!(a, b, "cooker must produce byte-identical output for the same input");
    }

    #[test]
    fn parse_asset_id_valid_hex() {
        let hex = "deadbeef".repeat(8);
        let id = parse_asset_id(&hex).unwrap();
        // Round-trip via hex.
        assert_eq!(id.to_hex(), hex);
    }

    #[test]
    fn parse_asset_id_rejects_short_hex() {
        let err = parse_asset_id("short").unwrap_err();
        assert!(matches!(err, CookError::InvalidAssetId(_)));
    }
}
