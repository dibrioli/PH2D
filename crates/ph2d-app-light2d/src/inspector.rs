//! ⭐ **A secção LUZ do Inspector, lado da família** (plano §4): o instantâneo que o painel pinta e
//! o dreno das edições que ele publica. O painel (`ph2d-panel-inspector`) não conhece o ECS; os
//! tipos que atravessam são os do [`ph2d_editor_core::luz_edits`].

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::luz_edits::{InspectorCorpoDaLuzInfo, InspectorLuzInfo, LuzFieldEdit};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz, TipoDeLuz};

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

/// O instantâneo dos cartões do CORPO («Light Occluder» · «Unlit») para o objeto `escolhido`;
/// `None` se ele não tem nenhum dos dois componentes (ADR-0166: o cartão só existe onde o
/// componente está).
#[must_use]
pub fn build_corpo_info(world: &World, escolhido: Option<u64>) -> Option<InspectorCorpoDaLuzInfo> {
    let bits = escolhido?;
    let ent = world.get_entity(Entity::from_bits(bits)).ok()?;
    let oclusor = ent.get::<BloqueiaLuz>().map(|b| b.opacidade);
    let sem_luz = ent.contains::<NaoRecebeLuz>();
    (oclusor.is_some() || sem_luz).then_some(InspectorCorpoDaLuzInfo {
        entity_bits: bits,
        oclusor,
        sem_luz,
    })
}

/// As edições do CORPO. `false` = componente ausente, valor recusado ou nada mudou.
fn aplicar_ao_corpo(w: &mut World, e: Entity, edit: &LuzFieldEdit) -> bool {
    match *edit {
        LuzFieldEdit::Opacidade(v) if v.is_finite() => {
            w.get_mut::<BloqueiaLuz>(e).is_some_and(|mut b| {
                let nova = v.clamp(0.0, 1.0);
                let mudou = b.opacidade != nova;
                if mudou {
                    b.opacidade = nova;
                }
                mudou
            })
        }
        LuzFieldEdit::RetirarOclusor => w.entity_mut(e).take::<BloqueiaLuz>().is_some(),
        LuzFieldEdit::RetirarSemLuz => w.entity_mut(e).take::<NaoRecebeLuz>().is_some(),
        _ => false,
    }
}

/// **O dreno de UMA edição.** `true` = tocou no mundo. As da lâmpada vão à `Luz`; as do corpo, à
/// `BloqueiaLuz` / `NaoRecebeLuz`.
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &LuzFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    let w = sim.world_mut();
    if w.get_entity(e).is_err() {
        return false;
    }
    match edit {
        LuzFieldEdit::Opacidade(_) | LuzFieldEdit::RetirarOclusor | LuzFieldEdit::RetirarSemLuz => {
            aplicar_ao_corpo(w, e, edit)
        }
        _ => w
            .get_mut::<Luz>(e)
            .is_some_and(|mut l| aplicar(&mut l, edit)),
    }
}

/// **O dreno do quadro.** `true` = alguma tocou no mundo.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, LuzFieldEdit)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

impl crate::LuzDoQuadro {
    /// ⭐ **O que a luz publica no fim do quadro** (W5c: saiu da shell, a função dela passava o teto):
    /// aplica a preferência do editor e devolve a ficha da secção LUZ do escolhido (`(entidade,
    /// quantos)`), a dos cartões do CORPO e a recusa NOVA do solver em palavras (recusa que só o
    /// terminal vê é botão mudo).
    pub fn publicar(
        &mut self,
        preferencia: ph2d_editor_core::luz_edits::PreferenciaDaLuz,
        world: &World,
        (escolhido, quantos): (Option<u64>, usize),
    ) -> (
        Option<InspectorLuzInfo>,
        Option<InspectorCorpoDaLuzInfo>,
        Option<String>,
    ) {
        self.preferir(preferencia);
        let info = build_info(world, escolhido, quantos, self);
        let corpo = build_corpo_info(world, escolhido);
        (info, corpo, self.recusa_nova())
    }
}
