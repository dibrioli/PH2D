//! ⭐ **A cena 40** (`PH2D_OBJECT_MODE_SMOKE=40`, F33 — achado do dono no smoke de 26/10: Bevel +
//! Flag = vincos). Seis Nuvens (a forma do app, pela ferramenta de forma) com o MESMO Bevel + Drop
//! Shadow da foto do dono: a 1.ª sem efeito
//! (o controlo), as outras com os efeitos que reconstroem a curva — Flag (os números da foto),
//! Wave, Twist, Zig Zag suave e Sketch. O relevo tem de ler liso em todas; os vales continuam
//! quinas. No fim a nuvem do Flag fica selecionada, com a pilha dela no painel.
//! Plano: `docs/UI_New_and_Simple/F33_plano.md`.

use crate::state::VecState;
use ph2d_ecs::{FxOp, Name, SimWorld, VecFilter};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId, VecScene};
use std::sync::atomic::{AtomicU8, Ordering};

/// O estágio — UM só, no buraco entre a cena 39 (`43..=44`) e a 12 (`50..`).
pub(crate) const FIRST: u8 = 45;
pub(crate) const LAST: u8 = FIRST;
const _: () = assert!(LAST < 50);

const VERDE: Rgba8 = Rgba8::new(120, 190, 100, 255);
/// O centro de cada nuvem (mundo), duas filas de três.
const CENTROS: [[f64; 2]; 6] = [
    [-2.4, 1.0],
    [0.0, 1.0],
    [2.4, 1.0],
    [-2.4, -1.2],
    [0.0, -1.2],
    [2.4, -1.2],
];
/// Os efeitos, pelo TIPO (o nome que a cena mostra é o do próprio efeito); `None` = o controlo.
fn efeitos() -> [Option<PathEffect>; 6] {
    use ph2d_vec_scene::fx_warp_presets::{WarpSpec, WarpStyle};
    [
        None,
        Some(PathEffect::Warp(WarpSpec::new(WarpStyle::Flag))),
        Some(PathEffect::Warp(WarpSpec::new(WarpStyle::Wave))),
        Some(PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec::new())),
        Some(PathEffect::ZigZag(
            ph2d_vec_scene::fx_zigzag::ZigZagSpec::default(),
        )),
        Some(PathEffect::Sketch(
            ph2d_vec_scene::fx_sketch::SketchSpec::default(),
        )),
    ]
}
/// A do Flag fica selecionada no fim, com a pilha dela no painel.
const SELECIONADA: usize = 1;

static PASSO: AtomicU8 = AtomicU8::new(0);

/// O efeito com os números da cena (no Flag, os da foto do dono).
fn ajustado(mut e: PathEffect) -> PathEffect {
    let nomes: Vec<&str> = e.params().iter().map(|p| p.name).collect();
    for (i, n) in nomes.iter().enumerate() {
        let v = match *n {
            "Bend" => 17.45,
            "Horizontal" => 12.21,
            "Vertical" => -11.66,
            "Angle" => 60.0,
            "Size" => 4.0,
            "Ridges" => 12.0,
            "Smooth" => 1.0,
            "Passes" => 1.0,
            "Roughness" => 2.0,
            _ => continue,
        };
        e.set(i, v);
    }
    e
}

/// O Bevel + Drop Shadow da foto do dono (Depth 0.04 · luz 0.01/0.10 · sombra 0.04, 0.02/−0.09, 60 %).
fn filtro() -> VecFilter {
    let bevel = FxOp {
        radius: 0.04,
        offset: [0.01, 0.10],
        ..FxOp::new(FxOp::BEVEL)
    };
    let sombra = FxOp {
        radius: 0.04,
        offset: [0.02, -0.09],
        opacity: 0.6,
        ..FxOp::new(FxOp::DROP_SHADOW)
    };
    VecFilter::from_ops(vec![bevel, sombra])
}

/// Um quadro da cena 40; `Some(9)` = acabou.
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    _hero: &mut HeroScreen,
) -> Option<u8> {
    let n = vec.entities.len();
    let passo = usize::from(PASSO.load(Ordering::Relaxed));
    if passo < CENTROS.len() {
        // Uma nuvem por quadro, cada uma depois de a anterior ter entidade.
        if n != passo {
            return None;
        }
        crate::vector_mode::draw_shape(vec, scene, ShapeKind::Cloud, CENTROS[passo], 0.9);
        PASSO.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    if n < CENTROS.len() {
        return None;
    }
    let ids: Vec<VecPathId> = vec.entities.keys().copied().collect();
    let mut rotulos = Vec::new();
    for (&id, efeito) in ids.iter().zip(efeitos()) {
        let p = scene.path_mut(id)?;
        p.fill = Some(Paint::solid(VERDE));
        p.stroke = None;
        let rotulo = efeito
            .as_ref()
            .map_or(FxOp::spec(FxOp::BEVEL).name, PathEffect::label);
        p.effects = efeito
            .map(|e| vec![FxEntry::new(ajustado(e))])
            .unwrap_or_default();
        sim.world_mut()
            .entity_mut(ph2d_ecs::Entity::from_bits(*vec.entities.get(&id)?))
            .insert(Name::new(rotulo));
        rotulos.push(rotulo);
    }
    let postos = ph2d_vec_entities::filter::set_filter(sim, &vec.entities, &ids, Some(filtro()));
    vec.pen.select_many(&[*ids.get(SELECIONADA)?]);
    eprintln!(
        "[cena 40] {} nuvens com Bevel + Drop Shadow ({postos} filtros): {}",
        ids.len(),
        rotulos.join(", ")
    );
    Some(9)
}
