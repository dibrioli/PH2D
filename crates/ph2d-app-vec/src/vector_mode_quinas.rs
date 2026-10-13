//! ⭐ **A cena 21** (`PH2D_OBJECT_MODE_SMOKE=21`, F14 — 13/10): AS QUINAS ARREDONDADAS. No layout
//! *Vetor*, as 36 formas que têm o «Corner round», em quatro filas; o botão *Forma*; e, pelo MESMO
//! press e arrasto da shell (`shape_handles_live::on`), o raio de cada uma subido — o ⊙ puxado pelo
//! trilho dele, ou o ⇕ subido onde as quinas são entre curvas. A foto: todos os cantos
//! redondos, nenhum laço. `PH2D_CENA21_FICA=<i>` deixa seleccionada a forma `i` (omissão: o cubo).
//! Doc: `docs/UI_New_and_Simple/vetor_nova_ui/24_arredondar_as_quinas.md`.

use crate::shape_handles_live::{Gate, on, view};
use crate::state::VecState;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::shape_handles::{Mods, Spot, handles_of};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use ShapeKind as K;

/// As formas, pela ordem do catálogo (a elipse inteira não tem quina: a pizza mostra a família).
pub(crate) const KINDS: [ShapeKind; 36] = [
    K::Rectangle,
    K::Pie,
    K::Segment,
    K::ArrowRight,
    K::ArrowDouble,
    K::ArrowBent,
    K::Chevron,
    K::Diamond,
    K::Parallelogram,
    K::Trapezoid,
    K::TrapezoidFlip,
    K::HexagonFlat,
    K::Document,
    K::Delay,
    K::Display,
    K::PredefinedProcess,
    K::OffPage,
    K::NoteBracket,
    K::SpeechRect,
    K::SpeechOval,
    K::Thought,
    K::Burst,
    K::Heart,
    K::Bolt,
    K::Drop,
    K::Shield,
    K::Tag,
    K::Gear,
    K::Cross,
    K::Check,
    K::Banner,
    K::IsoCube,
    K::IsoCone,
    K::IsoPyramid,
    K::Bone,
    K::RopeSegment,
];

/// Nove por fila, quatro filas (m).
const COLS: usize = 9;
const PITCH: f64 = 1.55;
pub(crate) const MEIA: f64 = 0.55;

/// Metros por pixel da vista de omissão — o mesmo das outras cenas.
const PX: f64 = 0.01;

/// Quanto a mão anda: o ⊙ pelo trilho dele, o ⇕ para cima (~12–19 px de raio na caixa de 110 px).
const PARA_DENTRO: f64 = 0.12;
const SOBE: f64 = 0.04;

/// O primeiro estágio da cena (um por forma a nascer, o botão, um por forma a arredondar, o fim).
pub(crate) const FIRST: u8 = 140;
const BUTTON: u8 = FIRST + KINDS.len() as u8;
const PULLS: u8 = BUTTON + 1;
pub(crate) const LAST: u8 = PULLS + KINDS.len() as u8;

/// O centro da forma `i`.
pub(crate) fn centre(i: usize) -> [f64; 2] {
    let (col, row) = ((i % COLS) as f64, (i / COLS) as f64);
    let rows = KINDS.len().div_ceil(COLS) as f64;
    [
        (col - (COLS as f64 - 1.0) * 0.5) * PITCH,
        ((rows - 1.0) * 0.5 - row) * PITCH,
    ]
}

/// A alça do «Corner round» de `kind`.
pub(crate) fn round_handle(kind: ShapeKind) -> Option<usize> {
    let f = kind.corner_round_field()?;
    handles_of(kind).iter().position(|h| h.fields == [f])
}

fn fica() -> usize {
    std::env::var("PH2D_CENA21_FICA")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|k: &usize| *k < KINDS.len())
        .unwrap_or(31)
}

/// Agarra a alça do raio da forma `i` pelo press da shell e leva-a; solta.
fn round_up(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    id: VecPathId,
    i: usize,
) -> bool {
    let Some(handle) = round_handle(KINDS[i]) else {
        return false;
    };
    vec.pen.select_many(&[id]);
    let Some(at) = view(sim, &vec.entities, &[id], Gate::of(vec), PX, None, None)
        .and_then(|v| v.points.get(handle).copied())
    else {
        return false;
    };
    // O ⇕ sobe; o ⊙ anda pelo TRILHO que o cursor por cima dele mostra (o arco a nascer na aresta).
    let d = if handles_of(KINDS[i])[handle].spot == Spot::Rail {
        [0.0, SOBE]
    } else {
        let track = view(sim, &vec.entities, &[id], Gate::of(vec), PX, None, Some(at))
            .map(|v| v.track)
            .unwrap_or_default();
        let (Some(a), Some(b)) = (track.first(), track.last()) else {
            return false;
        };
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let n = dx.hypot(dy).max(1e-12);
        [dx / n * PARA_DENTRO, dy / n * PARA_DENTRO]
    };
    if !on::press(vec, sim, scene, at, PX, 0) {
        return false;
    }
    for k in 1..=6 {
        let t = f64::from(k) / 6.0;
        on::drag(
            vec,
            sim,
            scene,
            [at[0] + d[0] * t, at[1] + d[1] * t],
            Mods::default(),
            PX,
        );
    }
    vec.shape_handles.release()
}

/// Um passo da cena 21; devolve o próximo estágio, se mudou. `FIRST..BUTTON` = uma forma por
/// estágio · `BUTTON` = o modo Forma · `PULLS..LAST` = o raio de cada uma · `LAST` = a que fica.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let feitas = vec.entities.len();
    let nth = |vec: &VecState, k: usize| vec.entities.keys().nth(k).copied();
    match stage {
        s if (FIRST..BUTTON).contains(&s) && feitas == usize::from(s - FIRST) => {
            let i = usize::from(s - FIRST);
            crate::vector_mode::draw_shape(vec, scene, KINDS[i], centre(i), MEIA);
            Some(s + 1)
        }
        BUTTON if feitas == KINDS.len() => {
            hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
            Some(PULLS)
        }
        s if (PULLS..LAST).contains(&s)
            && vec.draw_config.mode == ph2d_tool_vector::DrawMode::Shape =>
        {
            let i = usize::from(s - PULLS);
            round_up(vec, scene, sim, nth(vec, i)?, i).then_some(s + 1)
        }
        LAST => {
            vec.pen.select_many(&[nth(vec, fica())?]);
            Some(9)
        }
        _ => None,
    }
}
