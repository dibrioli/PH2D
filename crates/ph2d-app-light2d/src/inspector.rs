//! ⭐ **A secção LUZ do Inspector, lado da família** (plano §4): o instantâneo que o painel pinta e
//! o dreno das edições que ele publica. O painel (`ph2d-panel-inspector`) não conhece o ECS; os
//! tipos que atravessam são os do [`ph2d_editor_core::luz_edits`].

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::luz_edits::{InspectorLuzInfo, LuzFieldEdit};
use ph2d_light2d_ecs::{Luz, TipoDeLuz};

/// A tag de cada tipo (a posição em `luz_edits::TIPOS`).
#[must_use]
pub fn tag_do_tipo(t: TipoDeLuz) -> u8 {
    match t {
        TipoDeLuz::Ponto => 0,
        TipoDeLuz::Ambiente => 1,
    }
}

/// A inversa de [`tag_do_tipo`]; `None` fora da faixa.
#[must_use]
pub fn tipo_da_tag(tag: u8) -> Option<TipoDeLuz> {
    match tag {
        0 => Some(TipoDeLuz::Ponto),
        1 => Some(TipoDeLuz::Ambiente),
        _ => None,
    }
}

/// O instantâneo da secção para o objeto `escolhido`; `None` se ele não tem `Luz` (ADR-0166: a
/// secção só existe onde o componente está). `luz` é a do quadro — os avisos vêm dela.
#[must_use]
pub fn build_info(
    world: &World,
    escolhido: Option<u64>,
    quantos: usize,
    luz: &crate::LuzDoQuadro,
) -> Option<InspectorLuzInfo> {
    let bits = escolhido?;
    let e = Entity::from_bits(bits);
    let l = world.get_entity(e).ok()?.get::<Luz>()?;
    Some(InspectorLuzInfo {
        entity_bits: bits,
        cor_srgb: l.cor.map(ph2d_color::srgb::linear_to_srgb_byte),
        intensidade: l.intensidade,
        tamanho_m: l.tamanho_m,
        alcance_m: l.alcance_m,
        tipo: tag_do_tipo(l.tipo),
        luz_a_1m: l.intensidade * l.cor.iter().copied().fold(0.0, f32::max) / std::f32::consts::TAU,
        avisos: luz.avisos_de(e),
        #[allow(clippy::cast_possible_truncation)]
        qualidade: luz
            .qualidade()
            .map(|(n, a)| (n, a, luz.custo_ms().map(|ms| ms as f32))),
        selected_count: quantos,
    })
}

/// Aplica uma edição a uma `Luz`. `false` = valor recusado (não finito, negativo) ou nada mudou.
fn aplicar(l: &mut Luz, edit: &LuzFieldEdit) -> bool {
    let antes = *l;
    match *edit {
        LuzFieldEdit::Cor(c) => l.cor = c.map(ph2d_color::srgb::srgb_to_linear_byte),
        LuzFieldEdit::Intensidade(v) if v.is_finite() && v >= 0.0 => l.intensidade = v,
        LuzFieldEdit::Tamanho(v) if v.is_finite() && v >= 0.0 => l.tamanho_m = v,
        // `∞` é legal (física pura); zero ou negativo não é um alcance.
        LuzFieldEdit::Alcance(v) if v > 0.0 && !v.is_nan() => l.alcance_m = v,
        LuzFieldEdit::Tipo(t) => match tipo_da_tag(t) {
            Some(t) => l.tipo = t,
            None => return false,
        },
        _ => return false,
    }
    *l != antes
}

/// **O dreno de UMA edição.** `true` = tocou no mundo.
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &LuzFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    let w = sim.world_mut();
    if w.get_entity(e).is_err() {
        return false;
    }
    w.get_mut::<Luz>(e)
        .is_some_and(|mut l| aplicar(&mut l, edit))
}

/// **O dreno do quadro.** `true` = alguma tocou no mundo.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, LuzFieldEdit)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}
