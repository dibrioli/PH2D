//! **O AUTO LAYOUT da seleção** — a projeção que o painel lê, e a porta que o edita (plano UI/UX
//! W2, ADR-0153).
//!
//! Irmão do [`crate::vec_frame_edit`], e com a mesma divisão de donos: a verdade mora no ECS
//! (`VecLayout` no pai, `VecLayoutItem` no filho) e isto é o que a shell publica por frame. O
//! painel não alcança o mundo — se alcançasse, a resposta que DESENHA o chip aceso divergiria da
//! que HONRA o clique.
//!
//! # Uma tabela, duas direções
//!
//! ⚠️ Os três rádios (direção · alinhamento · distribuição) precisam da mesma correspondência
//! `enum ↔ chip` lida nos dois sentidos: **para publicar** o aceso e **para resolver** o clique.
//! Escrevê-la duas vezes — um `match` que pinta e outro que honra — é como uma variante nova entra
//! só numa delas, e o sintoma é um chip que acende e não faz nada. Aqui cada rádio é **um array**,
//! e as duas perguntas são varreduras dele.
//!
//! # Quem é o SUJEITO da seção
//!
//! Duas metades, resolvidas por portas diferentes porque são perguntas diferentes:
//!
//! - **a moldura** é a que CONTÉM a seleção — a MESMA porta do W0
//!   ([`crate::vec_frame_edit::frame_of_selection`]), que já sabe desfazer a expansão de seleção
//!   dos contêineres;
//! - **o filho** é a seleção quando ela é UMA forma cujo pai flui. Uma moldura aninhada responde
//!   às duas, e é isso que faz os dois blocos aparecerem juntos nela.

use ph2d_ecs::{
    ChildOf, Entity, LayoutAlign, LayoutDir, LayoutJustify, LayoutSize, SimWorld, VecFrame,
    VecLayout, VecLayoutAbsolute, VecLayoutItem, VecLayoutSize,
};
use ph2d_editor::ids;
use ph2d_panel_vector::state::{LayoutFlow, LayoutItem};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// A direção ⟷ o chip. `Off` não está aqui: ele é a AUSÊNCIA do componente, e por isso não tem
/// variante a que corresponder.
const DIRS: &[(ph2d_editor::NodeId, LayoutDir)] = &[
    (ids::VECTOR_LAYOUT_DIR_ROW, LayoutDir::Row),
    (ids::VECTOR_LAYOUT_DIR_COL, LayoutDir::Column),
    (ids::VECTOR_LAYOUT_DIR_WRAP, LayoutDir::RowWrap),
];

const ALIGNS: &[(ph2d_editor::NodeId, LayoutAlign)] = &[
    (ids::VECTOR_LAYOUT_ALIGN_START, LayoutAlign::Start),
    (ids::VECTOR_LAYOUT_ALIGN_CENTER, LayoutAlign::Center),
    (ids::VECTOR_LAYOUT_ALIGN_END, LayoutAlign::End),
    (ids::VECTOR_LAYOUT_ALIGN_STRETCH, LayoutAlign::Stretch),
];

const JUSTIFIES: &[(ph2d_editor::NodeId, LayoutJustify)] = &[
    (ids::VECTOR_LAYOUT_JUSTIFY_START, LayoutJustify::Start),
    (ids::VECTOR_LAYOUT_JUSTIFY_CENTER, LayoutJustify::Center),
    (ids::VECTOR_LAYOUT_JUSTIFY_END, LayoutJustify::End),
    (
        ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        LayoutJustify::SpaceBetween,
    ),
    (
        ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
        LayoutJustify::SpaceAround,
    ),
];

/// O que um clique num chip do layout PEDE. Nomeado para o roteador não carregar cinco
/// `Option<…>` paralelos que podem estar preenchidos ao mesmo tempo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LayoutEdit {
    /// Uma direção — `None` = **Off**, que REMOVE o componente.
    Dir(Option<LayoutDir>),
    Align(LayoutAlign),
    Justify(LayoutJustify),
    /// O tamanho de um EIXO da moldura: `(eixo, abraça?)`.
    Size(usize, bool),
    /// O filho selecionado sai (ou volta a entrar) no fluxo do pai.
    Absolute(bool),
}

/// Este id é um chip do layout? Porta única do roteador — a mesma varredura das três tabelas.
#[must_use]
pub(crate) fn layout_edit_for_id(id: ph2d_editor::NodeId) -> Option<LayoutEdit> {
    if id == ids::VECTOR_LAYOUT_DIR_OFF {
        return Some(LayoutEdit::Dir(None));
    }
    if let Some(&(_, d)) = DIRS.iter().find(|(i, _)| *i == id) {
        return Some(LayoutEdit::Dir(Some(d)));
    }
    if let Some(&(_, a)) = ALIGNS.iter().find(|(i, _)| *i == id) {
        return Some(LayoutEdit::Align(a));
    }
    if let Some(e) = size_edit_for_id(id) {
        return Some(e);
    }
    if id == ids::VECTOR_LAYOUT_ITEM_ABSOLUTE {
        // ⚠️ O toggle é um ALTERNADOR, e o valor a escrever depende do estado de agora — que só a
        // shell conhece. O `apply` resolve-o lendo o mundo; aqui vai o pedido, não o valor.
        return Some(LayoutEdit::Absolute(true));
    }
    JUSTIFIES
        .iter()
        .find(|(i, _)| *i == id)
        .map(|&(_, j)| LayoutEdit::Justify(j))
}

/// Os quatro chips de tamanho, numa tabela — a mesma forma das outras três.
fn size_edit_for_id(id: ph2d_editor::NodeId) -> Option<LayoutEdit> {
    const SIZES: &[(ph2d_editor::NodeId, usize, bool)] = &[
        (ids::VECTOR_LAYOUT_SIZE_W_FIXED, 0, false),
        (ids::VECTOR_LAYOUT_SIZE_W_HUG, 0, true),
        (ids::VECTOR_LAYOUT_SIZE_H_FIXED, 1, false),
        (ids::VECTOR_LAYOUT_SIZE_H_HUG, 1, true),
    ];
    SIZES
        .iter()
        .find(|(i, _, _)| *i == id)
        .map(|&(_, axis, hug)| LayoutEdit::Size(axis, hug))
}

/// Qual campo numérico do layout este id endereça, e o que ele escreve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LayoutField {
    /// Vão: `0` principal, `1` transversal.
    Gap(usize),
    /// Recuo de UM lado (índice na ordem do CSS: topo, direita, base, esquerda).
    Pad(usize),
    /// Recuo dos QUATRO lados de uma vez (o campo do modo *All*).
    PadAll,
    Grow,
    Shrink,
    /// Piso de um eixo (`0` = índice da largura).
    Min(usize),
    /// Teto de um eixo.
    Max(usize),
}

/// Porta única do roteador para os campos numéricos.
#[must_use]
pub(crate) fn layout_field_for_id(id: ph2d_editor::NodeId) -> Option<LayoutField> {
    let f = match id {
        _ if id == ids::VECTOR_LAYOUT_GAP_MAIN => LayoutField::Gap(0),
        _ if id == ids::VECTOR_LAYOUT_GAP_CROSS => LayoutField::Gap(1),
        _ if id == ids::VECTOR_LAYOUT_PAD_ALL => LayoutField::PadAll,
        _ if id == ids::VECTOR_LAYOUT_PAD_T => LayoutField::Pad(0),
        _ if id == ids::VECTOR_LAYOUT_PAD_R => LayoutField::Pad(1),
        _ if id == ids::VECTOR_LAYOUT_PAD_B => LayoutField::Pad(2),
        _ if id == ids::VECTOR_LAYOUT_PAD_L => LayoutField::Pad(3),
        _ if id == ids::VECTOR_LAYOUT_ITEM_GROW => LayoutField::Grow,
        _ if id == ids::VECTOR_LAYOUT_ITEM_SHRINK => LayoutField::Shrink,
        _ if id == ids::VECTOR_LAYOUT_MIN_W => LayoutField::Min(0),
        _ if id == ids::VECTOR_LAYOUT_MAX_W => LayoutField::Max(0),
        _ if id == ids::VECTOR_LAYOUT_MIN_H => LayoutField::Min(1),
        _ if id == ids::VECTOR_LAYOUT_MAX_H => LayoutField::Max(1),
        _ => return None,
    };
    Some(f)
}

/// O chip aceso para uma variante — a MESMA tabela, lida ao contrário.
fn chip_of<T: PartialEq + Copy>(table: &[(ph2d_editor::NodeId, T)], v: T) -> ph2d_editor::NodeId {
    // ⚠️ O `unwrap_or` cai no PRIMEIRO chip da tabela, e nunca dispara: as tabelas são
    // exaustivas sobre enums fechados, e um `match` no compilador não as cobre porque elas são
    // dados. É o gate `every_layout_variant_has_a_chip` que o prova — e é ele que sangra no dia
    // em que uma variante nova entrar no enum e não na tabela.
    table
        .iter()
        .find(|(_, t)| *t == v)
        .map_or(table[0].0, |(i, _)| *i)
}

/// O filho de fluxo da seleção: UMA forma cujo pai empilha.
fn item_of_selection(sim: &SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> Option<Entity> {
    // A moldura que contém tudo responde primeiro (uma moldura aninhada é ela própria um item);
    // fora disso, só uma seleção de UMA forma tem um "filho" de que falar.
    let subject = crate::vec_frame_edit::frame_of_selection(sim, map, selected).or_else(|| {
        let [only] = selected else { return None };
        let &bits = map.get(only)?;
        let e = Entity::from_bits(bits);
        sim.world().get_entity(e).ok().map(|_| e)
    })?;
    let parent = sim.world().get::<ChildOf>(subject)?.parent();
    // ⚠️ **A pergunta é *"o pai é uma MOLDURA?"*, e não *"ele flui?"*.** Ela era a segunda, e o
    // preço foi um report de smoke: um filho de moldura parada publicava `None`, a seção Layout
    // não era pintada de todo, e o artista que procurava o *Absolute position* não tinha como
    // saber que faltava ligar o fluxo no PAI. O fluxo passou a ser um CAMPO (`in_flow`), que o
    // painel usa para explicar em vez de esconder.
    // ⚠️ **A união, e não `VecFrame` sozinho.** Trocar um predicado pelo outro REMOVERIA um caso
    // que funcionava: o passe de layout recolhe os filhos de qualquer entidade com `VecLayout`,
    // tenha ela `VecFrame` ou não. Dois gates sangraram nesta linha exactamente por isso.
    let w = sim.world();
    (w.get::<VecFrame>(parent).is_some() || w.get::<VecLayout>(parent).is_some()).then_some(subject)
}

/// O fluxo da moldura da seleção — `None` = não há moldura, ou ela não empilha.
#[must_use]
pub(crate) fn selected_flow(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<LayoutFlow> {
    let e = crate::vec_frame_edit::frame_of_selection(sim, map, selected)?;
    let l = sim.world().get::<VecLayout>(e)?;
    Some(LayoutFlow {
        dir: chip_of(DIRS, l.dir),
        gap: l.gap,
        pad: l.pad,
        align: chip_of(ALIGNS, l.align),
        justify: chip_of(JUSTIFIES, l.justify),
        // ⚠️ Ausente = `Fixed` nos dois eixos e sem limites: o mundo de antes desta feature.
        size: [size_chip(sim, e, 0), size_chip(sim, e, 1)],
        min: bounds(sim, e, true),
        max: bounds(sim, e, false),
    })
}

/// Como o filho selecionado se comporta — `None` = a seleção não está num fluxo.
#[must_use]
pub(crate) fn selected_item(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<LayoutItem> {
    let e = item_of_selection(sim, map, selected)?;
    let it = sim
        .world()
        .get::<VecLayoutItem>(e)
        .copied()
        .unwrap_or_default();
    Some(LayoutItem {
        grow: f64::from(it.grow),
        shrink: f64::from(it.shrink),
        absolute: sim.world().get::<VecLayoutAbsolute>(e).is_some(),
        in_flow: sim
            .world()
            .get::<ChildOf>(e)
            .is_some_and(|c| sim.world().get::<VecLayout>(c.parent()).is_some()),
    })
}

/// O chip de tamanho ACESO num eixo — a mesma leitura que o `chip_of` faz para as outras três
/// tabelas, e a ausência do componente lê `Fixed`, que é o mundo de antes desta feature.
fn size_chip(sim: &SimWorld, e: Entity, axis: usize) -> ph2d_editor::NodeId {
    let hug = sim
        .world()
        .get::<VecLayoutSize>(e)
        .is_some_and(|s| s.size[axis] == LayoutSize::Hug);
    match (axis, hug) {
        (0, false) => ids::VECTOR_LAYOUT_SIZE_W_FIXED,
        (0, true) => ids::VECTOR_LAYOUT_SIZE_W_HUG,
        (_, false) => ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        (_, true) => ids::VECTOR_LAYOUT_SIZE_H_HUG,
    }
}

/// Os dois limites de um lado, com **zero a significar ausência** — ver `VECTOR_LAYOUT_MIN_W`.
fn bounds(sim: &SimWorld, e: Entity, floor: bool) -> [f64; 2] {
    let Some(s) = sim.world().get::<VecLayoutSize>(e) else {
        return [0.0; 2];
    };
    let b = if floor { s.min } else { s.max };
    [b[0].unwrap_or(0.0), b[1].unwrap_or(0.0)]
}

/// Aplica um clique de chip. Devolve `true` se o mundo mudou — o `post_frame_undo` regista por
/// diff, então um no-op não custa passo de undo.
pub(crate) fn apply_layout_edit(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
    edit: LayoutEdit,
) -> bool {
    // ⚠️ **O fora-do-fluxo é resolvido ANTES de resolver a moldura, e a ordem é a correção
    // inteira.** Todo edit desta porta é da MOLDURA — menos este, que é do FILHO —, e
    // `frame_of_selection` recusa *um filho sozinho* por desenho. Com o guard à frente, o único
    // edit que é pedido com o filho selecionado era o único que o guard matava: o checkbox
    // pintava, acendia sob o mouse e **não fazia nada** (report do Enio, smoke da cena `=66`).
    //
    // O guard FICA para os outros quatro — abri-lo para todos faria um `Dir` pedido sobre um
    // filho solto ligar fluxo na entidade errada (gate irmão
    // `a_frame_edit_asked_with_only_the_child_selected_does_nothing`).
    if let LayoutEdit::Absolute(_) = edit {
        return apply_absolute(sim, map, selected);
    }
    let Some(e) = crate::vec_frame_edit::frame_of_selection(sim, map, selected) else {
        return false;
    };
    let cur = sim.world().get::<VecLayout>(e).copied();
    let next = match edit {
        // ⚠️ **Off REMOVE o componente** — vão, recuo e alinhamento vão junto. É o que o Figma faz
        // ao tirar o auto layout, e é a única leitura honesta: um componente inerte guardado no
        // arquivo seria uma regra que ninguém honra, invisível e a viajar em todo save.
        LayoutEdit::Dir(None) => None,
        LayoutEdit::Dir(Some(dir)) => Some(VecLayout {
            dir,
            ..cur.unwrap_or_default()
        }),
        // ⚠️ Alinhar uma moldura que ainda não flui **liga o fluxo** com o `Default` — o chip não
        // é pintado nesse estado, então chegar aqui só acontece por um clique roteado errado; e
        // escrever o alinhamento num componente que não existe seria perder o clique em silêncio.
        LayoutEdit::Align(align) => Some(VecLayout {
            align,
            ..cur.unwrap_or_default()
        }),
        LayoutEdit::Justify(justify) => Some(VecLayout {
            justify,
            ..cur.unwrap_or_default()
        }),
        // O tamanho não escreve no `VecLayout`: ele tem componente próprio, e o `write_layout`
        // no fim desta função não é o caminho dele. (O `Absolute` saiu daqui para o topo — ver
        // a nota na entrada da função.)
        LayoutEdit::Size(axis, hug) => return apply_size(sim, e, axis, hug),
        LayoutEdit::Absolute(_) => unreachable!("resolvido na entrada, antes do guard de moldura"),
    };
    write_layout(sim, e, cur, next)
}

/// **O tamanho de um EIXO.** O componente DESTACA quando volta ao neutro — um componente que não
/// faz nada não viaja no arquivo, a mesma lei do `VecLayoutItem`.
fn apply_size(sim: &mut SimWorld, e: Entity, axis: usize, hug: bool) -> bool {
    let cur = sim.world().get::<VecLayoutSize>(e).copied();
    let mut next = cur.unwrap_or_default();
    next.size[axis] = if hug {
        LayoutSize::Hug
    } else {
        LayoutSize::Fixed
    };
    write_size(sim, e, cur, next)
}

/// **O filho entra ou sai do fluxo.** ⚠️ O chip é um ALTERNADOR: o valor a escrever é o contrário
/// do que está lá, e quem sabe o que está lá é o mundo — por isso a decisão mora aqui, e não no
/// roteador que só viu um id.
fn apply_absolute(sim: &mut SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> bool {
    let Some(e) = item_of_selection(sim, map, selected) else {
        return false;
    };
    let on = sim.world().get::<VecLayoutAbsolute>(e).is_some();
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    if on {
        em.remove::<VecLayoutAbsolute>();
    } else {
        em.insert(VecLayoutAbsolute);
    }
    true
}

/// A escrita do [`VecLayoutSize`], com o destacar do neutro — irmã do [`write_layout`].
fn write_size(
    sim: &mut SimWorld,
    e: Entity,
    cur: Option<VecLayoutSize>,
    next: VecLayoutSize,
) -> bool {
    if cur == Some(next) {
        return false;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    if next == VecLayoutSize::default() {
        if cur.is_none() {
            return false;
        }
        em.remove::<VecLayoutSize>();
        return true;
    }
    em.insert(next);
    true
}

/// Aplica um valor de campo numérico.
pub(crate) fn apply_layout_field(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
    field: LayoutField,
    v: f64,
) -> bool {
    match field {
        LayoutField::Grow | LayoutField::Shrink => {
            let Some(e) = item_of_selection(sim, map, selected) else {
                return false;
            };
            let cur = sim.world().get::<VecLayoutItem>(e).copied();
            let mut next = cur.unwrap_or_default();
            // ⚠️ O piso em zero é o DOMÍNIO do modelo (o flexbox não exprime crescer negativo),
            // não um teto de gosto — por isso ele mora aqui, na porta do documento, e não como
            // faixa do widget. Não há teto: `grow = 1000` e `grow = 1` fazem a mesma coisa quando
            // só um filho cresce.
            let v = v.max(0.0) as f32;
            match field {
                LayoutField::Grow => next.grow = v,
                _ => next.shrink = v,
            }
            if cur == Some(next) {
                return false;
            }
            // O neutro DESTACA: um componente que não faz nada não viaja no arquivo.
            if next == VecLayoutItem::default() {
                if cur.is_none() {
                    return false;
                }
                if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
                    em.remove::<VecLayoutItem>();
                    return true;
                }
                return false;
            }
            if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
                em.insert(next);
                return true;
            }
            false
        }
        LayoutField::Min(axis) | LayoutField::Max(axis) => {
            // Os limites são da MOLDURA, não do filho: eles descrevem o tamanho dela.
            let Some(e) = crate::vec_frame_edit::frame_of_selection(sim, map, selected) else {
                return false;
            };
            let cur = sim.world().get::<VecLayoutSize>(e).copied();
            let mut next = cur.unwrap_or_default();
            // ⚠️ **Zero é AUSÊNCIA** (ver `VECTOR_LAYOUT_MIN_W`), e o negativo é domínio: um
            // comprimento não é negativo, e deixá-lo entrar poria o motor a resolver uma caixa
            // impossível em vez de recusar um número que ninguém quis digitar.
            let v = (v > 0.0).then_some(v.max(0.0));
            match field {
                LayoutField::Min(_) => next.min[axis] = v,
                _ => next.max[axis] = v,
            }
            write_size(sim, e, cur, next)
        }
        LayoutField::Gap(_) | LayoutField::Pad(_) | LayoutField::PadAll => {
            let Some(e) = crate::vec_frame_edit::frame_of_selection(sim, map, selected) else {
                return false;
            };
            // Um campo de fluxo só é PINTADO sobre uma moldura que já flui, então não há aqui o
            // caso de "ligar por engano": sem componente, não há campo.
            let Some(cur) = sim.world().get::<VecLayout>(e).copied() else {
                return false;
            };
            let mut next = cur;
            let v = v.max(0.0);
            match field {
                LayoutField::Gap(i) => next.gap[i] = v,
                LayoutField::Pad(i) => next.pad[i] = v,
                _ => next.pad = [v; 4],
            }
            // **Digitar um vão SOLTA o token dele** — o *detach* do Figma (W4c.4), a mesma lei que
            // a cor já segue. Sem isto o artista escreveria um número, o token continuaria a
            // espaçar, e o campo mostraria um valor que a moldura não usa: o pior estado possível
            // para um controlo.
            //
            // ⚠️ Ele solta AQUI, e não por um canal one-shot como o da cor, porque este sítio já
            // tem o mundo e a seleção na mão — e o da cor só existe porque quem SABE que uma cor
            // foi autorada é o tool, que não tem nenhum dos dois.
            if let LayoutField::Gap(i) = field {
                let prop = if i == 0 {
                    ph2d_ecs::BoundProp::LayoutGapMain
                } else {
                    ph2d_ecs::BoundProp::LayoutGapCross
                };
                crate::vec_bindings::set_selected_binding(sim, map, selected, prop, None);
            }
            write_layout(sim, e, Some(cur), Some(next))
        }
    }
}

/// A ÚNICA escrita de `VecLayout` — inserir, mudar ou remover, com o no-op recusado antes de
/// tocar no mundo.
fn write_layout(
    sim: &mut SimWorld,
    e: Entity,
    cur: Option<VecLayout>,
    next: Option<VecLayout>,
) -> bool {
    if cur == next {
        return false;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    match next {
        Some(l) => em.insert(l),
        None => em.remove::<VecLayout>(),
    };
    true
}

#[cfg(test)]
#[path = "vec_layout_edit_tests.rs"]
mod tests;
