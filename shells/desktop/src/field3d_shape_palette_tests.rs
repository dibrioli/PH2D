//! Os gates da PALETA de formas (W100) — ver [`super`].

use super::{build, item_id, slot_of_pick};
use crate::field3d_shapes::{Family, Make, SHAPES};

/// Todos os rótulos do modelo, num saco só.
fn labels(model: &ph2d_editor::widget::command_palette::PaletteModel) -> Vec<String> {
    model
        .groups
        .iter()
        .flat_map(|g| g.subs.iter())
        .flat_map(|s| s.items.iter())
        .map(|i| i.label.clone())
        .collect()
}

/// ⭐⭐⭐ **TODA forma do catálogo é alcançável pela paleta** — a lei da W34 (*o painel oferece
/// exatamente o que o gesto faz*), medida do lado da oferta.
///
/// ⚠️ **E ela é a razão de a paleta existir:** a fileira de chips cortava em `MAX_MODES` = 8 e o
/// catálogo já tinha 8. A forma nº 9 seria pintada e morta, ou nem pintada — e nenhum gate deste
/// módulo notava, porque todos contavam a lista de origem, nunca o que sobrevivia ao corte.
#[test]
fn every_shape_in_the_catalogue_is_offered() {
    let model = build(true, true);
    let items: Vec<_> = model
        .groups
        .iter()
        .flat_map(|g| g.subs.iter())
        .flat_map(|s| s.items.iter())
        .collect();
    assert_eq!(
        items.len(),
        SHAPES.len(),
        "a paleta ofereceu {} de {} formas",
        items.len(),
        SHAPES.len()
    );
    for shape in SHAPES {
        assert!(
            items.iter().any(|i| i.id == item_id(shape.key)),
            "{} não chegou à paleta",
            shape.key
        );
    }
}

/// ⭐⭐ **O pick volta a ser a MESMA forma** — a ida e a volta, medidas juntas.
///
/// ⚠️ Sem este gate, o `item_id` e o `slot_of_pick` podiam divergir (um a hashar a chave, o outro o
/// rótulo, por exemplo) e o sintoma seria *"o item aparece e não faz nada"* — que é o defeito que a
/// paleta de componentes já nomeia no `name_of_pick`.
#[test]
fn a_pick_names_the_shape_that_was_offered() {
    for (slot, shape) in SHAPES.iter().enumerate() {
        assert_eq!(
            slot_of_pick(item_id(shape.key)),
            Some(slot),
            "a volta de {} não deu o slot dela",
            shape.key
        );
    }
}

/// ⭐ **Um id estrangeiro não é uma forma** — é o que torna o dreno condicional honesto.
///
/// ⚠️ Este canal tem **quatro** consumidores (a biblioteca do Motion, o `Ctrl+K`, o `+` do
/// Inspector e esta paleta). Um `slot_of_pick` que devolvesse `Some` para qualquer id faria esta
/// paleta engolir o pick dos outros três — e o sintoma seria *"às vezes o Ctrl+K cria uma caixa"*.
#[test]
fn a_foreign_pick_is_not_a_shape() {
    assert!(slot_of_pick(ph2d_tool_registry::hash_node_id("ph2d::ecs::SliceNine")).is_none());
    assert!(slot_of_pick(ph2d_tool_registry::hash_node_id("motion.oscillator")).is_none());
}

/// ⭐⭐⭐ **O que precisa de seleção APARECE, e diz porquê** — e este é o ganho de produto da W100.
///
/// ⚠️ Antes, as três sumiam da fileira quando não estavam disponíveis (a lei da W34 aplicada como
/// *esconder*), e com isso o artista não podia **saber que elas existem**. Aqui elas ficam, num
/// sub-grupo com título, e o rótulo carrega o gesto que as destranca.
///
/// ⛔ E o CONTROLE é a metade que faz este gate valer: com as condições ligadas, nenhuma delas
/// carrega razão nenhuma. Sem ele, uma razão colada em todos os rótulos passaria.
#[test]
fn what_needs_a_selection_says_so_and_only_then() {
    let sem = build(false, false);
    let com = build(true, true);
    for shape in SHAPES {
        let rotulo = ph2d_i18n::tr(shape.key);
        let bloqueada = labels(&sem)
            .into_iter()
            .any(|l| l.starts_with(rotulo) && l.len() > rotulo.len());
        let livre = labels(&com).into_iter().any(|l| l == rotulo);
        assert!(livre, "{} devia estar limpa com tudo disponível", shape.key);
        match shape.make {
            Make::Formula(_) | Make::Sculpt => assert!(
                !bloqueada,
                "{} não depende de nada e trouxe uma razão",
                shape.key
            ),
            Make::Extrude | Make::Revolve | Make::SculptScene => assert!(
                bloqueada,
                "{} depende da seleção e não disse porquê",
                shape.key
            ),
        }
    }
}

/// ⭐ **Uma família sem formas não vira cabeçalho órfão.**
///
/// ⚠️ É o que deixa a [`Family::Plates`] nascer vazia à espera do lote dela — e é a mesma lei que a
/// paleta de componentes aplica às categorias sem componente oferecível.
#[test]
fn an_empty_family_paints_no_header() {
    let model = build(true, true);
    let vazias: Vec<_> = Family::ALL
        .iter()
        .filter(|f| !SHAPES.iter().any(|s| s.family == **f))
        .collect();
    assert!(
        !vazias.is_empty(),
        "o gate perdeu o sujeito: nenhuma família está vazia hoje, então ele não mede nada"
    );
    for f in vazias {
        assert!(
            !model.groups.iter().any(|g| g.title == f.title()),
            "a família {f:?} está vazia e mesmo assim pintou um grupo"
        );
    }
}

/// ⭐⭐⭐ **COM A PALETA ABERTA, O ROTEADOR DE TECLAS DO 3D SE CALA** — e este gate lê a FONTE.
///
/// # ⚠️ Por que a fonte, e não o comportamento
///
/// A guarda pergunta ao `HeroScreen`, que segura uma *surface* de janela real: num teste o `gfx` é
/// `None`, a guarda devolve sempre `false`, e **um teste de comportamento passaria com ela apagada**
/// — a mutação sobreviveria por o sujeito não existir. É a mesma razão pela qual a rota do ponteiro
/// do sculpt3d não é alcançável de um teste, e a mesma resposta que o `interact_menu_tests` do
/// Motion dá: *quando o comportamento não é alcançável, meça a ESTRUTURA que o produz*.
///
/// # ⛔ O que ela defende
///
/// `field3d_keys` corre **antes** da captura modal da paleta, e a guarda de cada tecla é o ponteiro
/// sobre a janela 3D — que continua verdadeiro com o modal por cima. Sem esta linha, escrever
/// «capsule» na busca dispara o `S` (escalar) e o `A` (reabrir), e as letras nunca chegam ao campo.
/// ⚠️ E ela tem de vir **antes do primeiro tratador**: depois de um deles, a tecla dele já foi
/// comida.
#[test]
fn the_field3d_keys_stand_down_while_the_palette_is_open() {
    let src = include_str!("input_dispatch/keyboard_field3d.rs");
    let guarda = src
        .find("if self.command_palette_open() {")
        .expect("o roteador de teclas do 3D tem de se calar com a paleta aberta");
    let primeiro = src
        .find("if self.field3d_home_key(code)")
        .expect("o primeiro tratador de tecla");
    assert!(
        guarda < primeiro,
        "a guarda da paleta vem DEPOIS do primeiro tratador - a tecla dele já foi comida"
    );
}

/// ⭐ **O que se pode criar vem primeiro dentro do grupo** — o bloqueado é contexto, não oferta.
#[test]
fn the_ready_shapes_come_before_the_blocked_ones() {
    let model = build(false, false);
    for g in &model.groups {
        let primeiro_com_titulo = g.subs.iter().position(|s| s.title.is_some());
        let ultimo_sem = g.subs.iter().rposition(|s| s.title.is_none());
        if let (Some(a), Some(b)) = (primeiro_com_titulo, ultimo_sem) {
            assert!(b < a, "o sub-grupo bloqueado do {} veio antes", g.title);
        }
    }
}
