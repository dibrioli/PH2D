//! Os gates da PALETA de formas (W100) — ver [`super`].

use super::{build, item_id, slot_of_pick};
use crate::field3d_shapes::{Family, Make, SHAPES};

/// O rótulo do item com este id, se ele estiver no modelo.
///
/// ⚠️ **Por ID e não por prefixo de rótulo**, e a diferença mordeu na W102: a primeira versão
/// perguntava `l.starts_with(rotulo)`, e o rótulo `"Torus"` é **prefixo** de `"Torus Arc"` — o gate
/// acusou o toro de trazer uma razão que era do arco. *Uma régua de prefixo sobre nomes de produto
/// parte no dia em que alguém acrescenta a variante.*
fn label_of(
    model: &ph2d_editor::widget::command_palette::PaletteModel,
    id: ph2d_editor::NodeId,
) -> Option<String> {
    model
        .groups
        .iter()
        .flat_map(|g| g.subs.iter())
        .flat_map(|s| s.items.iter())
        .find(|i| i.id == id)
        .map(|i| i.label.clone())
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
        let id = item_id(shape.key);
        let bloqueada = label_of(&sem, id).is_some_and(|l| l != rotulo);
        let livre = label_of(&com, id).is_some_and(|l| l == rotulo);
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

/// ⭐⭐⭐ **COM A PALETA ABERTA, AS QUATRO ENTRADAS DO 3D CALAM-SE** — teclado, clique, movimento e
/// roda. Este gate lê a FONTE.
///
/// # ⚠️ Por que a fonte, e não o comportamento
///
/// A guarda pergunta ao `HeroScreen`, que segura uma *surface* de janela real: num teste o `gfx` é
/// `None`, a guarda devolve sempre `false`, e **um teste de comportamento passaria com ela apagada**
/// — a mutação sobreviveria por o sujeito não existir. É a mesma razão pela qual a rota do ponteiro
/// do sculpt3d não é alcançável de um teste, e a mesma resposta que o `interact_menu_tests` do
/// Motion dá: *quando o comportamento não é alcançável, meça a ESTRUTURA que o produz*.
///
/// # ⛔ O defeito que o Enio viu, e a metade que este gate não tinha
///
/// A 1.ª versão cobria **só o teclado** — e o smoke devolveu o resto: *«o modal não funciona, não
/// fecha. Os modelos do modal não são criados.»* **Um mecanismo, dois sintomas**: o
/// `field3d_pointer_down` corre antes do despacho de chrome e reclama todo gesto que começa dentro
/// da área que a janela 3D desenhou — a paleta cobre-a. O clique nunca chegava ao handler dela, que
/// é quem regista o pick **e** quem a fecha.
///
/// ⚠️ *Eu gateei a entrada que estava a construir (a tecla `A`) e não a família dela.* O gate passa
/// a varrer as **quatro**, e a quinta que nascer sem a pergunta reprova aqui.
///
/// ⚠️ O **soltar** fica de fora de propósito: um gesto já em curso tem de poder acabar. Ver
/// `field3d_yields_to_modal`.
#[test]
fn the_field3d_keys_stand_down_while_the_palette_is_open() {
    /// O corpo de uma função, do `fn nome` até ao `fn ` seguinte.
    fn corpo<'a>(src: &'a str, nome: &str) -> &'a str {
        let ini = src
            .find(nome)
            .unwrap_or_else(|| panic!("a entrada `{nome}` tem de existir"));
        let resto = &src[ini + nome.len()..];
        &resto[..resto.find("\n    pub(crate) fn ").unwrap_or(resto.len())]
    }
    const PORTA: &str = "self.field3d_yields_to_modal()";
    let input = include_str!("field3d_input.rs");
    for entrada in [
        "fn field3d_pointer_down(",
        "fn field3d_pointer_move(",
        "fn field3d_wheel(",
    ] {
        assert!(
            corpo(input, entrada).contains(PORTA),
            "`{entrada}` não pergunta `{PORTA}` — com a paleta aberta ela rouba o gesto, e o \
             sintoma é «o modal não faz nada»"
        );
    }
    // ⚠️ E o teclado, cuja guarda tem de vir **antes do primeiro tratador**: depois de um deles, a
    // tecla dele já foi comida.
    let teclas = include_str!("input_dispatch/keyboard_field3d.rs");
    let guarda = teclas
        .find(PORTA)
        .expect("o roteador de teclas do 3D tem de se calar com a paleta aberta");
    let primeiro = teclas
        .find("if self.field3d_home_key(code)")
        .expect("o primeiro tratador de tecla");
    assert!(
        guarda < primeiro,
        "a guarda da paleta vem DEPOIS do primeiro tratador - a tecla dele já foi comida"
    );
    // ⛔ **O CONTROLE**: o SOLTAR NÃO a pergunta, e é deliberado. Sem esta metade, alguém
    // «uniformiza» as quatro para cinco e deixa o arrasto pousado para sempre.
    assert!(
        !corpo(input, "fn field3d_pointer_up(").contains(PORTA),
        "o soltar tem de ficar de fora — um gesto em curso tem de poder acabar"
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
