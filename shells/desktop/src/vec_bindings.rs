//! **O RESOLVEDOR de tokens** — o `ph2d_ecs::VecBindings` de cada forma, resolvido contra o modo
//! vigente, na forma que o renderer consome.
//!
//! É a metade que só a shell pode fazer: o documento não conhece tema, e a crate de desenho não
//! conhece o ECS onde os bindings moram.
//!
//! ⚠️ **Ele NÃO entra no [`crate::vec_entities::view_state`]**, e a razão é custo: aquela porta é
//! chamada por todo caminho de HIT-TEST e gesto (o pick, o marquee, a linha de corte), e nenhum
//! deles pergunta de que cor a forma é. Resolver token ali seria trabalho de desenho pago por quem
//! só quer geometria. Quem publica é o passe de desenho, uma vez por frame.

use ph2d_ecs::{BoundProp, Entity, SimWorld, VecBindings};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::{BoundPaint, Rgba8};

use crate::vec_entities::VecEntityMap;

/// A cor concreta de um token neste modo.
///
/// ⚠️ **A porta ÚNICA de *"que cor tem este token?"***, e ela tem três consumidores: este
/// resolvedor (para DESENHAR), o painel (para a swatch MOSTRAR) e os gates. Uma segunda tabela em
/// qualquer um deles é a swatch que mostra uma cor e a arte que desenha outra — divergência que só
/// aparece num screenshot.
///
/// Chave desconhecida devolve `None` em vez de uma cor de emergência: um token que sumiu da tabela
/// tem de deixar o LITERAL valer (a arte volta ao que o artista escreveu), e nunca pintar de rosa
/// choque uma forma que estava certa.
#[must_use]
pub(crate) fn token_color(key: &str, theme: Theme) -> Option<Rgba8> {
    let c = ColorToken::from_key(key)?.resolve(theme);
    Some(Rgba8::new(c.r, c.g, c.b, c.a))
}

/// As tintas que os tokens produzem neste frame, para o `VecViewState`.
///
/// Vazio quando nada está bindado — que é todo documento que já existe, e é o que faz o desenho
/// deles ficar byte-idêntico ao mundo pré-token.
#[must_use]
pub(crate) fn resolve(sim: &SimWorld, map: &VecEntityMap, theme: Theme) -> Vec<BoundPaint> {
    let w = sim.world();
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        let Some(b) = w.get::<VecBindings>(e) else {
            continue;
        };
        let paint = BoundPaint {
            path: id,
            fill: b.get(BoundProp::Fill).and_then(|k| token_color(k, theme)),
            stroke: b
                .get(BoundProp::StrokeColor)
                .and_then(|k| token_color(k, theme)),
        };
        // Uma entrada que não resolveu nada não descreve desenho nenhum — publicá-la faria o
        // renderer perguntar por ela em toda forma da cena sem nunca ter o que responder.
        if !paint.is_noop() {
            out.push(paint);
        }
    }
    out
}

/// **Que escolha do picker este id é** — a propriedade, e o token (ou `None` = SOLTAR).
///
/// ⚠️ Ele ENUMERA os ids gerados em vez de os inverter, porque um `NodeId` é um hash e um hash não
/// se inverte. É o mesmo desenho do `frames::device_preset`, e o custo é 162 comparações num
/// clique — não num frame.
#[must_use]
pub(crate) fn token_choice(id: ph2d_editor::NodeId) -> Option<(BoundProp, Option<&'static str>)> {
    for (prop, code) in [(BoundProp::Fill, 0_u16), (BoundProp::StrokeColor, 1)] {
        if id == ph2d_editor::ids::vector_token_option_id(code, 0) {
            return Some((prop, None));
        }
        for (i, t) in ColorToken::ALL.iter().enumerate() {
            if id == ph2d_editor::ids::vector_token_option_id(code, i + 1) {
                return Some((prop, Some(t.key())));
            }
        }
    }
    None
}

/// Prende (ou solta) a propriedade nas formas SELECIONADAS.
///
/// ⚠️ **Desanexa o componente quando fica vazio.** Um `VecBindings` sem entradas viaja no save e
/// entra no diff do undo, e então duas cenas logicamente iguais comparam diferente — o passo
/// espúrio que o `canonicalize` do undo global existe para matar.
pub(crate) fn set_selected_binding(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
    prop: BoundProp,
    token: Option<&str>,
) {
    for id in selected {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let mut b = sim
            .world()
            .get::<VecBindings>(e)
            .cloned()
            .unwrap_or_default();
        match token {
            Some(t) => b.set(prop, t),
            None => b.clear(prop),
        }
        if b.is_empty() {
            sim.world_mut().entity_mut(e).remove::<VecBindings>();
        } else {
            sim.world_mut().entity_mut(e).insert(b);
        }
    }
}

/// O que o painel mostra nas rows de token: o que a seleção tem preso, e se ela tem traço.
///
/// `None` quando a seleção não é uma forma única — as rows não são pintadas, porque prender um
/// token a *"várias formas ao mesmo tempo"* precisa de uma resposta a *"e se elas discordarem?"*
/// que esta wave não dá.
#[must_use]
pub(crate) fn selected_bindings(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
) -> Option<ph2d_panel_vector::state::TokenBindings> {
    let [id] = selected else { return None };
    let &bits = map.get(id)?;
    let e = Entity::from_bits(bits);
    if sim.world().get_entity(e).is_err() {
        return None;
    }
    let b = sim.world().get::<VecBindings>(e);
    Some(ph2d_panel_vector::state::TokenBindings {
        fill: b.and_then(|b| b.get(BoundProp::Fill)).map(str::to_owned),
        stroke: b
            .and_then(|b| b.get(BoundProp::StrokeColor))
            .map(str::to_owned),
        stroke_exists: scene.path(*id).is_some_and(|p| p.stroke.is_some()),
    })
}

thread_local! {
    /// `(fill, stroke)` — uma cor foi AUTORADA neste frame, e a propriedade tem de soltar o token.
    static COLOUR_AUTHORED: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((false, false)) };
}

/// A ponte drena o one-shot do tool aqui; o passe de aplicação o consome.
///
/// ⚠️ Um canal interno da shell, e não um argumento a mais: quem SABE que uma cor foi autorada é o
/// tool (a ponte é quem fala com ele), e quem pode SOLTAR o token é o passe que tem o mundo e a
/// seleção na mão. Os dois correm no mesmo frame, em ordem — a ponte primeiro.
pub(crate) fn note_colour_authored(v: (bool, bool)) {
    COLOUR_AUTHORED.with(|c| c.set(v));
}

/// **Escolher uma cor SOLTA o token daquela propriedade** — o *detach* do Figma.
///
/// ⚠️ Sem isto o artista escolheria uma cor, o token continuaria a cobri-la, e a swatch mostraria
/// um valor que a arte não usa: o pior estado possível para um controlo (decisão do Enio,
/// 2026-08-02). E é por isso que o `colour_authored` do tool arma só quando o valor MUDA — o
/// read-back do picker corre em todo frame em que ele está aberto.
pub(crate) fn detach_on_authored_colour(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[ph2d_vec_scene::VecPathId],
) {
    let (fill, stroke) = COLOUR_AUTHORED.with(std::cell::Cell::take);
    if fill {
        set_selected_binding(sim, map, selected, BoundProp::Fill, None);
    }
    if stroke {
        set_selected_binding(sim, map, selected, BoundProp::StrokeColor, None);
    }
}

#[cfg(test)]
#[path = "vec_bindings_tests.rs"]
mod tests;
