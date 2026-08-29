//! ⭐⭐⭐ **A PALETA DE FORMAS** (W100) — o que o botão *Add shape…* e a tecla `A` abrem.
//!
//! # Ela NÃO é um modal novo
//!
//! É o [`ph2d_editor_core::widget::command_palette`], que já é **genérico por desenho**: conhece só
//! um `PaletteModel` (título + categorias coloridas + itens com um `NodeId` opaco), e quem abriu
//! mapeia o id de volta. Já tem scrim, cascata de entrada, **busca com um predicado servindo o
//! filtro pintado *e* o `Enter`**, sub-clusters, rolagem e promoção a 2 colunas.
//!
//! ⇒ **o que este módulo constrói é o MODELO**, copiando os três precedentes já shipados: a
//! biblioteca de nós do Motion (86 tipos), o `Ctrl+K` global, e o `+` do Inspector
//! ([`crate::component_palette`], ADR-0166/F3).
//!
//! # ⚠️ Por que a fileira de chips não podia ficar
//!
//! `paint_chips` corta em `MAX_MODES` = **8**, e o catálogo já tinha **8**. A forma nº 9 seria
//! **pintada e morta** — ou nem pintada, sem uma palavra. Com 47 formas do catálogo vetorial e 15
//! sólidas na fila ([doc 08](../../../docs/3DModeling/08_formas_por_formula.md), Enio 2026-08-28:
//! *«ao final quero todas»*), a fileira não é uma escolha de gosto: ela **não tem onde caber**.
//!
//! ⭐ E o que a paleta compra além do espaço é o que faz um catálogo grande ser *usável*: **a busca
//! é o único acesso que não fica mais lento quando a lista cresce.** `A` · três letras · `Enter`.
//!
//! # ⚠️ O que NÃO está disponível aparece, e diz porquê
//!
//! A lei da W34 (*o painel oferece exatamente o que o gesto faz*) tirava da fileira as três formas
//! que dependem do que está escolhido — e com isso o artista não podia sequer **saber que elas
//! existem**. Aqui elas ficam, num sub-grupo cujo título **nomeia a condição**, e um clique numa
//! delas responde pela [`crate::field3d_notice`]. *Uma affordance que mente é a que aceita o clique
//! e não faz nada; esta responde.*

use crate::field3d_shapes::{Family, Make, SHAPES, Shape};
use ph2d_editor::NodeId;
use ph2d_editor::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};

/// O id de item de uma forma — o hash da **chave i18n** dela.
///
/// ⚠️ A chave e não a posição: um item de paleta sobrevive a inserções no meio do catálogo, que é
/// exatamente o que uma lista de 60 formas vai sofrer.
#[must_use]
pub(crate) const fn item_id(key: &'static str) -> NodeId {
    ph2d_tool_registry::hash_node_id(key)
}

/// A razão pela qual esta forma não pode ser criada agora — `None` quando ela pode.
///
/// ⚠️ **Escrita para o artista, e diz o GESTO que a destranca**, não a condição interna: *"escolha
/// um contorno fechado"* é acionável; *"profile_pick is none"* não é.
fn why_not(shape: &Shape, live_sculpt: bool, profile: bool) -> Option<&'static str> {
    if crate::field3d_shapes::available(shape, live_sculpt, profile) {
        return None;
    }
    Some(match shape.make {
        Make::Extrude | Make::Revolve => "pick a closed outline in the vector editor first",
        Make::SculptScene => "there is no sculpture in the scene yet",
        // ⚠️ Inalcançável por construção (as duas são sempre possíveis), e escrito assim de
        // propósito: um `_ =>` engoliria em silêncio uma forma nova que passasse a ter condição.
        Make::Formula(_) | Make::Sculpt => "not available right now",
    })
}

/// ⭐ **O modelo da paleta**, agrupado por família.
///
/// ⚠️ **Um grupo vazio não é pintado** — é o que deixa a [`Family::Plates`] nascer vazia à espera do
/// lote dela sem um cabeçalho órfão na tela.
pub(crate) fn build(live_sculpt: bool, profile: bool) -> PaletteModel {
    let mut groups = Vec::new();
    for family in Family::ALL {
        let mut ready = Vec::new();
        let mut blocked = Vec::new();
        for shape in SHAPES.iter().filter(|s| s.family == family) {
            let label = ph2d_i18n::tr(shape.key).to_string();
            let id = item_id(shape.key);
            match why_not(shape, live_sculpt, profile) {
                None => ready.push(PaletteItem { label, id }),
                Some(reason) => blocked.push(PaletteItem {
                    // ⚠️ **A razão viaja no RÓTULO**, e não num campo novo do widget genérico: o
                    // `PaletteItem` serve quatro consumidores, e um `disabled_reason` faria os
                    // outros três carregar um campo que não usam. É a mesma escolha da paleta de
                    // componentes, e pelo mesmo motivo.
                    label: format!("{label}  \u{2014}  {reason}"),
                    id,
                }),
            }
        }
        if ready.is_empty() && blocked.is_empty() {
            continue;
        }
        // ⚠️ **O que se pode criar vem PRIMEIRO, sempre** — o bloqueado é contexto, não oferta.
        let mut subs = Vec::new();
        if !ready.is_empty() {
            subs.push(PaletteSub {
                title: None,
                items: ready,
            });
        }
        if !blocked.is_empty() {
            subs.push(PaletteSub {
                title: Some("Needs something selected".to_string()),
                items: blocked,
            });
        }
        groups.push(PaletteGroup {
            title: family.title().to_string(),
            color: family.color(),
            subs,
        });
    }
    PaletteModel {
        title: "Add Shape".to_string(),
        groups,
        // ⚠️ **Sem caixa *Show all***: a paleta de componentes tem-na porque esconde o inaplicável;
        // esta mostra tudo sempre, com a razão ao lado. Não há segunda metade para revelar.
        toggle: None,
    }
}

/// A posição no catálogo que este id de item nomeia, se alguma. É o inverso do [`item_id`], e a
/// única forma de um pick voltar a ser uma forma.
///
/// ⚠️ **Varre o CATÁLOGO, não uma lista à mão** — uma segunda lista aqui envelheceria na primeira
/// forma nova, e o sintoma seria *"o item aparece e não faz nada"*.
#[must_use]
pub(crate) fn slot_of_pick(id: NodeId) -> Option<usize> {
    SHAPES.iter().position(|s| item_id(s.key) == id)
}

#[cfg(test)]
#[path = "field3d_shape_palette_tests.rs"]
mod tests;
