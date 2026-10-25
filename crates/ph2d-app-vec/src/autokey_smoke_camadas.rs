//! **A cena do smoke «cada camada leva a sua animação»** (F31, [plano](../../../docs/UI_New_and_Simple/F31_plano.md))
//! — `PH2D_AUTOKEY_SMOKE=36`. Só monta o palco, já a tocar em laço; os gestos são do dono:
//!
//! - à esquerda, o retângulo **camadas**: base azul, duas tintas na pilha do Appearance (um Fill
//!   deslocado que vai de vermelho a amarelo · um Stroke verde que engrossa) e dois Filters (Glow
//!   que cresce · Drop Shadow que se afasta) — apagar uma e ver a outra manter a animação (§B);
//! - à direita, a forma **contornos**: um caminho composto de dois quadrados (o de cima sobe, o de
//!   baixo encolhe) — apagar os pontos de um e ver o outro continuar a mexer como mexia (§C).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{Entity, FieldKey, FxOp, SimWorld, VecFilter};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::paint_stack::PaintEntry;
use ph2d_vec_scene::{
    Contour, Paint, Rgba8, ShapeKind, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex,
};

use crate::autokey_smoke::avanca as go;
use crate::state::VecState;

/// Os três instantes das chaves; o laço é `0..FIM`.
const T: [f64; 3] = [0.0, 1.0, 2.0];
const FIM: f64 = 2.0;
const CAMADAS_X: f64 = -1.4;
const CONTORNOS_X: f64 = 1.4;
const LADO: f64 = 0.6;
/// Meio lado e centro (y) de cada quadrado da forma composta.
const QUADRADO: f64 = 0.3;
const CIMA_Y: f64 = 0.45;
const BAIXO_Y: f64 = -0.55;

/// Um passo da cena por quadro.
pub(crate) fn step(
    nivel: u8,
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
            None
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            crate::vector_mode::draw_shape(
                vec,
                scene,
                ShapeKind::Rectangle,
                [CAMADAS_X, 0.0],
                LADO,
            );
            scene.push_path(contornos());
            go(3);
            None
        }
        3 => {
            let mut ids = scene.paths().iter().map(|p| p.id);
            let (camadas, compostos) = (ids.next()?, ids.next()?);
            let a = *vec.entities.get(&camadas)?;
            let b = *vec.entities.get(&compostos)?;
            let log = montar(scene, sim, tl, (camadas, a), (compostos, b));
            vec.pen.select_many(&[camadas]);
            hero.gizmo.replace_selection(Some(a));
            hero.set_panel_visible("timeline", true);
            // O laço pela porta do documento: o «Loop» da Timeline acende com ele.
            for keys in [true, false] {
                tl.doc.set_active_loop_for(keys, Some((0.0, FIM)));
            }
            ph2d_timeline::sync_transport_loop(&tl.doc, ph, true);
            ph.seek(0.0);
            ph.play();
            go(4);
            Some(log)
        }
        _ => None,
    }
}

fn rgba(r: u8, g: u8, b: u8) -> Rgba8 {
    Rgba8::new(r, g, b, 255)
}

fn quadrado(cx: f64, cy: f64, h: f64) -> Vec<VecVertex> {
    [[-h, -h], [h, -h], [h, h], [-h, h]]
        .into_iter()
        .map(|[x, y]| VecVertex::corner([cx + x, cy + y]))
        .collect()
}

/// A forma composta: contorno 0 = o quadrado de cima, contorno 1 = o de baixo.
fn contornos() -> VecPath {
    VecPath {
        verts: quadrado(CONTORNOS_X, CIMA_Y, QUADRADO),
        closed: true,
        subpaths: vec![Contour::new_closed(quadrado(
            CONTORNOS_X,
            BAIXO_Y,
            QUADRADO,
        ))],
        fill: Some(Paint::Solid(rgba(240, 150, 30))),
        ..VecPath::default()
    }
}

/// O estilo, os filtros, os ids e as chaves. Devolve a linha do terminal.
pub(crate) fn montar(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    tl: &mut TimelineState,
    (camadas, a): (VecPathId, u64),
    (compostos, b): (VecPathId, u64),
) -> String {
    if let Some(p) = scene.path_mut(camadas) {
        p.fill = Some(Paint::Solid(rgba(40, 90, 220)));
        let sombra = PaintEntry {
            offset: [0.25, -0.25],
            ..PaintEntry::fill(Paint::Solid(rgba(220, 30, 30)))
        };
        let traco = PaintEntry::stroke(StrokeSpec::new(rgba(30, 200, 80), 0.04));
        p.paints = vec![sombra, traco];
    }
    let glow = FxOp::new(FxOp::GLOW);
    // Sombra dura e escura: o afastamento lê-se na foto.
    let sombra = FxOp {
        radius: 0.03,
        opacity: 0.95,
        ..FxOp::new(FxOp::DROP_SHADOW)
    };
    sim.world_mut()
        .entity_mut(Entity::from_bits(a))
        .insert(VecFilter::from_ops(vec![glow, sombra]));
    ph2d_ecs::item_id::assign_missing_item_ids(sim.world_mut());
    crate::ids_da_pilha::passe(sim, scene);

    let p = scene.path(camadas).expect("acabou de nascer");
    let (fill, stroke) = (p.paints[0].id, p.paints[1].id);
    let f = sim
        .world()
        .get::<VecFilter>(Entity::from_bits(a))
        .expect("acabou de entrar");
    let (fglow, fsombra) = (f.ids()[0], f.ids()[1]);

    // Fill: vermelho → amarelo (o verde sobe) · Stroke: engrossa.
    let verde = c::TINTA_PREENCH + c::OFF_SOLIDA + 1;
    chaves(
        tl,
        a,
        FieldKey::do_item(VETOR, verde, fill, 0),
        [30.0, 230.0, 30.0],
    );
    let largura = FieldKey::do_item(VETOR, c::TRACO_LARGURA, stroke, 0);
    chaves(tl, a, largura, [0.04, 0.2, 0.04]);
    // Glow: o raio cresce · Drop Shadow: afasta-se.
    let raio = FieldKey::do_item(VETOR, c::FILTRO_RAIO, fglow, 0);
    chaves(tl, a, raio, [0.05, 0.25, 0.05]);
    for campo in [c::FILTRO_DESLOCA_X, c::FILTRO_DESLOCA_Y] {
        let s = if campo == c::FILTRO_DESLOCA_X {
            1.0
        } else {
            -1.0
        };
        let kk = FieldKey::do_item(VETOR, campo, fsombra, 0);
        chaves(tl, a, kk, [0.06 * s, 0.4 * s, 0.06 * s]);
    }

    // Contorno 0 sobe e volta; contorno 1 encolhe para 40 % e volta. Sobre as âncoras de AGORA (a
    // ligação à entidade passa-as ao referencial dela); as alças são absolutas e vão com a âncora.
    let p = scene.path(compostos).expect("acabou de nascer");
    for i in 0..2_u16 {
        let Some((verts, _)) = p.contour(usize::from(i)) else {
            continue;
        };
        let n = f64::from(u32::try_from(verts.len()).unwrap_or(1).max(1));
        let meio = verts.iter().fold([0.0, 0.0], |m, v| {
            [m[0] + v.anchor[0] / n, m[1] + v.anchor[1] / n]
        });
        for (j, v) in verts.iter().enumerate() {
            let a = v.anchor;
            let meio_do_laco = if i == 0 {
                [a[0], a[1] + 0.6]
            } else {
                [
                    meio[0] + (a[0] - meio[0]) * 0.4,
                    meio[1] + (a[1] - meio[1]) * 0.4,
                ]
            };
            let pos = [a, meio_do_laco, a];
            let j = u16::try_from(j).unwrap_or(0);
            let eixos = [
                (0, [c::ANCORA_X, c::ENTRADA_X, c::SAIDA_X]),
                (1, [c::ANCORA_Y, c::ENTRADA_Y, c::SAIDA_Y]),
            ];
            for (eixo, campos) in eixos {
                for campo in campos {
                    let kk = FieldKey::new(VETOR, campo, i, j);
                    chaves(tl, b, kk, pos.map(|q| canal(q[eixo])));
                }
            }
        }
    }
    format!(
        "[autokey-smoke] =36 camadas {a:#x}: tintas Fill {fill:?} · Stroke {stroke:?} · filtros \
         Glow {fglow:?} · Drop Shadow {fsombra:?} · contornos {b:#x}: 2 · laço 0..{FIM} s a tocar"
    )
}

#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
fn canal(x: f64) -> f32 {
    x as f32
}

fn chaves(tl: &mut TimelineState, bits: u64, kk: FieldKey, v: [f32; 3]) {
    for (t, x) in T.into_iter().zip(v) {
        tl.doc.upsert_key(
            bits,
            PropKind::Field(kk),
            RationalTime::from_seconds(t),
            AnimValue::Float(x),
            Interp::Linear,
        );
    }
}

/// Que a montagem deixa uma tinta, um filtro e um contorno a responder pela MESMA chave depois de
/// o vizinho sair — ver o teste.
#[cfg(test)]
#[path = "autokey_smoke_camadas_tests.rs"]
mod tests;
