//! ⭐⭐⭐ **A PORTA px/m dos números de MUNDO do Inspector** — report do dono, 27/10: com o projecto
//! em px, os cartões mostravam `m`, `m/s` e `m/s2` fixos sobre o valor em metros.
//!
//! O componente guarda metros (ou, na âncora, px de mundo); o ecrã mostra o que o seletor do
//! projecto escolheu, com a MESMA conta do Transform (`DisplayUnit::from_meters_f64` /
//! `to_meters_f64`). Três sítios atravessam a porta, sempre pelo id:
//!
//! | sítio | função |
//! |---|---|
//! | a semente (`sync_*`) | [`semeia`] — guardado → ecrã |
//! | a escrita (`event_*`) | [`le`] — ecrã → guardado |
//! | a pintura (`rows::fields_row` / `num_row_unit`) | [`no_campo`] — sufixo e passo |
//!
//! O que viaja com o id: a faixa e a taxa de arrasto que o `populate` registou em metros
//! ([`reescala_faixas`], por quadro), e a re-semente quando o seletor muda (`sync.rs`).
//!
//! ⛔ **Um campo de MUNDO fora deste registo não mente**: a pintura mostra-o com o sufixo que a
//! secção declarou (`m`), e o censo ([`colhe_censo`]) aponta-o — o gate
//! `censo_nenhum_numero_de_mundo_foge_da_porta_px_m` reprova.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::project::{DisplayUnit, ProjectSettings};
use ph2d_editor_core::widget::Unit;

use crate::ids as i;

/// A grandeza que o COMPONENTE guarda num campo de mundo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grandeza {
    /// Metros.
    Comprimento,
    /// Metros por segundo.
    Velocidade,
    /// Metros por segundo ao quadrado.
    Aceleracao,
    /// Pixels do mundo (a posição da âncora: a shell converte a `pixels_per_meter`).
    PixelsDoMundo,
}

impl Grandeza {
    /// A grandeza de uma unidade declarada por uma secção, se for de mundo.
    #[must_use]
    pub const fn de_unit(u: Unit) -> Option<Self> {
        match u {
            Unit::Meters => Some(Self::Comprimento),
            Unit::MetersPerSecond => Some(Self::Velocidade),
            Unit::MetersPerSecondSquared => Some(Self::Aceleracao),
            _ => None,
        }
    }

    /// O sufixo do campo no seletor `d`.
    #[must_use]
    pub const fn sufixo(self, d: DisplayUnit) -> Unit {
        let px = matches!(d, DisplayUnit::Pixels);
        match self {
            Self::Comprimento | Self::PixelsDoMundo if px => Unit::Px,
            Self::Comprimento | Self::PixelsDoMundo => Unit::Meters,
            Self::Velocidade if px => Unit::PxPerSecond,
            Self::Velocidade => Unit::MetersPerSecond,
            Self::Aceleracao if px => Unit::PxPerSecondSquared,
            Self::Aceleracao => Unit::MetersPerSecondSquared,
        }
    }

    /// Guardado → ecrã.
    #[must_use]
    pub fn no_ecra(self, v: f64, d: DisplayUnit, ppm: f32) -> f64 {
        let metros = match self {
            Self::PixelsDoMundo => DisplayUnit::Pixels.to_meters_f64(v, ppm),
            _ => v,
        };
        d.from_meters_f64(metros, ppm)
    }

    /// Ecrã → guardado (o inverso de [`Self::no_ecra`]).
    #[must_use]
    pub fn do_ecra(self, v: f64, d: DisplayUnit, ppm: f32) -> f64 {
        let metros = d.to_meters_f64(v, ppm);
        match self {
            Self::PixelsDoMundo => DisplayUnit::Pixels.from_meters_f64(metros, ppm),
            _ => metros,
        }
    }
}

/// Os campos de mundo de tabela FIXA (uma unidade por id). As tabelas das secções que já são
/// indexadas por id (Platform Player, Particles) e a junta (unidade pelo tipo/eixo) respondem em
/// [`guardada`] a partir da PRÓPRIA tabela.
const FIXOS: [(NodeId, Grandeza); 52] = {
    use Grandeza::{Comprimento as L, PixelsDoMundo as P, Velocidade as V};
    [
        (i::INSP_CAMERA_HEIGHT, L),
        (i::INSP_CAMERA_OFFSET_X, L),
        (i::INSP_CAMERA_OFFSET_Y, L),
        (i::INSP_CAMERA_FOLLOW_OFF_X, L),
        (i::INSP_CAMERA_FOLLOW_OFF_Y, L),
        (i::INSP_CAMERA_MIN_X, L),
        (i::INSP_CAMERA_MIN_Y, L),
        (i::INSP_CAMERA_MAX_X, L),
        (i::INSP_CAMERA_MAX_Y, L),
        (i::INSP_PHYS_OFFSET_X, L),
        (i::INSP_PHYS_OFFSET_Y, L),
        (i::INSP_PHYS_RADIUS, L),
        (i::INSP_PHYS_HALF_X, L),
        (i::INSP_PHYS_HALF_Y, L),
        (i::INSP_PHYS_CAP_HALF_H, L),
        (i::INSP_PHYS_LINVEL_X, V),
        (i::INSP_PHYS_LINVEL_Y, V),
        (i::INSP_PHYS_WALK_BELT, V),
        (i::INSP_RAY_ORIGIN_X, L),
        (i::INSP_RAY_ORIGIN_Y, L),
        (i::INSP_RAY_REACH, L),
        (i::INSP_NAV_LINK_COST, L),
        (i::INSP_NAV_HALF_W, L),
        (i::INSP_NAV_HALF_H, L),
        (i::INSP_NAV_RADIUS, L),
        (i::INSP_NAV_ARRIVE, L),
        (i::INSP_NAV_REPATH, L),
        (i::INSP_NAV_TARGET_X, L),
        (i::INSP_NAV_TARGET_Y, L),
        (i::INSP_EMITTER_DENTRO, L),
        (i::INSP_EMITTER_FORA, L),
        (i::INSP_AUDIO_MAX_DIST, L),
        (i::INSP_AUDIO_RADIUS, L),
        (i::INSP_FACTORY_AREA_W, L),
        (i::INSP_FACTORY_AREA_H, L),
        (i::INSP_LIFE_OUTSIDE_MARGIN, L),
        (i::INSP_PJ_SPEED, V),
        (i::INSP_PJ_RANGE, L),
        (i::INSP_LUZ_TAMANHO, L),
        (i::INSP_LUZ_ALCANCE, L),
        (i::INSP_VIDA_NUMBERS_SIZE, L),
        (i::INSP_PARALLAX_TILE_X, L),
        (i::INSP_PARALLAX_TILE_Y, L),
        (i::INSP_PARALLAX_VEL_X, V),
        (i::INSP_PARALLAX_VEL_Y, V),
        (i::INSP_PARALLAX_MIN_X, L),
        (i::INSP_PARALLAX_MIN_Y, L),
        (i::INSP_PARALLAX_MAX_X, L),
        (i::INSP_PARALLAX_MAX_Y, L),
        (i::INSP_PF_LADO, L),
        (i::INSP_ANCHOR_POS[0], P),
        (i::INSP_ANCHOR_POS[1], P),
    ]
};

/// A segunda metade da tabela fixa (o tecto de um literal legível).
const FIXOS_2: [(NodeId, Grandeza); 13] = {
    use Grandeza::{Comprimento as L, Velocidade as V};
    [
        (i::INSP_SHAKE_AMPLITUDE, L),
        (i::INSP_WHEEL_RADIUS, L),
        (i::INSP_WHEEL_RADIUS_OUT, L),
        (i::INSP_BARRA_WIDTH, L),
        (i::INSP_BARRA_HEIGHT, L),
        (i::INSP_BARRA_OFFSET_X, L),
        (i::INSP_BARRA_OFFSET_Y, L),
        (i::INSP_DANO_KNOCKBACK, V),
        (i::INSP_DANO_KNOCKBACK_LIFT, V),
        (i::INSP_TD_SPEED, V),
        (i::INSP_SLICE_SIZE[0], L),
        (i::INSP_SLICE_SIZE[1], L),
        (i::INSP_JOINT_REST_LENGTH, L),
    ]
};

/// Os ids da junta cuja unidade muda com o tipo ou o eixo.
fn e_da_junta(id: NodeId) -> bool {
    matches!(
        id,
        i::INSP_JOINT_LIMIT_MIN | i::INSP_JOINT_LIMIT_MAX | i::INSP_JOINT_MOTOR_SPEED
    ) || id == i::INSP_JOINT_MOTOR_TARGET
        || i::INSP_JOINT_AXIS_MIN.contains(&id)
        || i::INSP_JOINT_AXIS_MAX.contains(&id)
}

/// ⭐ **A grandeza que o componente guarda em `id`**, se for de mundo — `None` deixa o número
/// passar intacto.
#[must_use]
pub fn guardada(id: NodeId) -> Option<Grandeza> {
    if let Some(&(_, g)) = FIXOS.iter().chain(&FIXOS_2).find(|(o, _)| *o == id) {
        return Some(g);
    }
    if let Some(k) = i::INSP_PART_NUM.iter().position(|&o| o == id) {
        return crate::sections::particles::unidade_guardada(k).and_then(Grandeza::de_unit);
    }
    if e_da_junta(id) {
        let info = crate::state::current_inspector_joint()?;
        return crate::sections::joint::unidade_guardada(&info, id).and_then(Grandeza::de_unit);
    }
    crate::sections::player::PLAYER_CARDS
        .iter()
        .flat_map(|(_, _, rows)| rows.iter())
        .find(|(_, o, _, _)| *o == id)
        .and_then(|(_, _, _, u)| u.and_then(Grandeza::de_unit))
}

/// **Todos os ids que PODEM ser de mundo** — a régua do gate de varredura e o laço das faixas.
#[must_use]
pub fn candidatos() -> Vec<NodeId> {
    let mut v: Vec<NodeId> = FIXOS.iter().chain(&FIXOS_2).map(|&(id, _)| id).collect();
    v.extend(
        i::INSP_PART_NUM
            .iter()
            .enumerate()
            .filter(|(k, _)| {
                crate::sections::particles::unidade_guardada(*k)
                    .and_then(Grandeza::de_unit)
                    .is_some()
            })
            .map(|(_, &id)| id),
    );
    v.extend([
        i::INSP_JOINT_LIMIT_MIN,
        i::INSP_JOINT_LIMIT_MAX,
        i::INSP_JOINT_MOTOR_SPEED,
        i::INSP_JOINT_MOTOR_TARGET,
    ]);
    // Só os eixos de translação (X, Y): o terceiro é a rotação, em graus.
    v.extend(&i::INSP_JOINT_AXIS_MIN[..2]);
    v.extend(&i::INSP_JOINT_AXIS_MAX[..2]);
    for (_, _, rows) in crate::sections::player::PLAYER_CARDS {
        v.extend(
            rows.iter()
                .filter(|(_, _, _, u)| u.and_then(Grandeza::de_unit).is_some())
                .map(|(_, id, _, _)| *id),
        );
    }
    v
}

/// O número guardado de `id` na unidade do ecrã.
pub(crate) fn no_ecra(id: NodeId, v: f64, p: &ProjectSettings) -> f64 {
    guardada(id).map_or(v, |g| g.no_ecra(v, p.display_unit, p.pixels_per_meter))
}

/// ⭐ **A semente**: escreve no store o valor GUARDADO de `id` já na unidade do ecrã.
pub(crate) fn semeia(host: &mut dyn PanelHostInternal, id: NodeId, v: f64) {
    let v = no_ecra(id, v, host.project());
    host.store_mut().set_number_value(id, v);
}

/// ⭐ **A escrita**: o valor do store de `id` de volta à unidade que o componente guarda.
#[must_use]
pub(crate) fn le(host: &dyn PanelHostInternal, id: NodeId) -> Option<f64> {
    let v = host.store().number_value(id)?;
    let p = host.project();
    Some(guardada(id).map_or(v, |g| g.do_ecra(v, p.display_unit, p.pixels_per_meter)))
}

/// O factor ecrã / guardado de `id` no seletor actual (`1` fora do registo).
fn factor(id: NodeId, d: DisplayUnit, ppm: f32) -> f64 {
    guardada(id).map_or(1.0, |g| g.no_ecra(1.0, d, ppm))
}

/// ⭐ **A pintura**: o sufixo e o passo de uma linha de campos de `ids` (um par `X`/`Y` partilha a
/// unidade), no seletor que a pintura publicou.
pub(crate) fn no_campo(ids: &[NodeId], unit: Option<Unit>, passo: f64) -> (Option<Unit>, f64) {
    let Some(u) = unit else {
        return (None, passo);
    };
    let d = crate::state::current_display_unit();
    let ppm = crate::state::current_pixels_per_meter();
    let declarada = Grandeza::de_unit(u);
    match ids.first().and_then(|&id| guardada(id)) {
        Some(g) if declarada.is_some() || g == Grandeza::PixelsDoMundo => {
            regista(ids, false);
            (Some(g.sufixo(d)), passo * g.no_ecra(1.0, d, ppm))
        }
        _ => {
            if declarada.is_some() && d == DisplayUnit::Pixels {
                regista(ids, true);
            }
            (Some(u), passo)
        }
    }
}

/// O que a porta guarda de cada id: a faixa e a taxa de arrasto do `populate` (em unidades
/// GUARDADAS), para as reescrever no seletor de cada quadro.
pub type FaixasOriginais = BTreeMap<NodeId, (Option<(f64, f64, f64)>, Option<f64>)>;

/// ⭐ **A faixa viaja com o valor**: sem isto o `0..100 m` do raio da roldana prendia a caixa em
/// `100 px` (o valor escrito era aparado na unidade errada). Por quadro, porque a unidade da junta
/// muda com o tipo sem o seletor mudar.
pub(crate) fn reescala_faixas(host: &mut dyn PanelHostInternal, originais: &mut FaixasOriginais) {
    let d = host.project().display_unit;
    let ppm = host.project().pixels_per_meter;
    for id in candidatos() {
        let (faixa, taxa) = *originais.entry(id).or_insert_with(|| {
            (
                host.store().number_range(id),
                host.store().number_drag_rate(id),
            )
        });
        let f = factor(id, d, ppm);
        let s = host.store_mut();
        if let Some((lo, hi, passo)) = faixa {
            s.set_number_range(id, lo * f, hi * f, passo * f);
        }
        if let Some(t) = taxa {
            s.set_number_drag_rate(id, t * f);
        }
    }
}

/// A assinatura do seletor — mudar px/m ou a escala re-semeia tudo (`sync.rs`).
#[must_use]
pub(crate) fn assinatura(p: &ProjectSettings) -> (bool, u32) {
    (
        p.display_unit == DisplayUnit::Pixels,
        p.pixels_per_meter.to_bits(),
    )
}

/// ⭐ **O censo** — o que a pintura mostrou pela porta e o que lhe FUGIU.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Censo {
    /// Ids de mundo pintados pela porta.
    pub pela_porta: Vec<NodeId>,
    /// Ids pintados com `m`/`m/s`/`m/s2` FIXO com o projecto em px — fora do registo.
    pub fugas: Vec<NodeId>,
}

thread_local! {
    static ARMADO: Cell<bool> = const { Cell::new(false) };
    static CENSO: RefCell<Censo> = RefCell::new(Censo::default());
}

fn regista(ids: &[NodeId], fuga: bool) {
    if !ARMADO.with(Cell::get) {
        return;
    }
    CENSO.with(|c| {
        let mut c = c.borrow_mut();
        let lista = if fuga {
            &mut c.fugas
        } else {
            &mut c.pela_porta
        };
        lista.extend_from_slice(ids);
    });
}

/// Arma o censo (por thread) e esvazia-o.
pub fn arma_censo() {
    ARMADO.with(|a| a.set(true));
    CENSO.with(|c| *c.borrow_mut() = Censo::default());
}

/// Desarma e devolve o que a pintura registou desde [`arma_censo`].
#[must_use]
pub fn colhe_censo() -> Censo {
    ARMADO.with(|a| a.set(false));
    CENSO.with(|c| std::mem::take(&mut *c.borrow_mut()))
}
