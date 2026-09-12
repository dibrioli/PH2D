//! ⭐⭐⭐ **A metade MEDIDA da wave 17: o que a hierarquia de facto REGISTOU.**
//!
//! O censo irmão (`ph2d-editor-core/tests/it/a_list_is_not_a_form.rs`) lê o *fonte* e prova que
//! ninguém escreve o vão à mão. Ele é cego a duas coisas que só o pintor sabe: **o passo que a
//! lista de facto anda** e **o tamanho do alvo de clique de cada companheiro**.
//!
//! ⚠️ **A régua chama o que o produto chama** — `paint::<HierarchyPanel>` — e lê o que ele
//! *emitiu*: os rectângulos do `HitIndex`. Foi a lição paga três vezes nesta jornada: *um gate que
//! interroga a porta testemunha sobre a porta, nunca sobre o pintor.*

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeMap;

/// Os ids de uma cena viva nascem em `BASE_NODE_ID = 100_000` (o `EntityNodeMap` do shell), e
/// **abaixo de `2^32`** é o que faz a deteção de companheiro disparar (`COMPANION_ROW_ID_MAX`).
const FIRST_ROW: u64 = 100_000;
const ROWS: usize = 8;

fn entry(name: &str) -> HierarchyEntity {
    HierarchyEntity {
        name: name.into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
    }
}

/// Pinta o painel e devolve os rectângulos registados.
fn painted() -> (Vec<(ph2d_a11y::NodeId, Rect)>, Vec<ph2d_a11y::NodeId>) {
    // ⚠️ **A cena tem OITO objectos, e isso é load-bearing.** A fixtura do painel tem UMA linha
    // («Scene Root»), então sem entregar uma cena viva toda lei sobre o que acontece *entre* duas
    // linhas seria verdadeira por vacuidade — e os dois pisos abaixo existem para o dizer alto.
    //
    // As duas metades são as que o `ph2d_panel_hierarchy::sync_from_hierarchy` faz a cada quadro
    // no shell: a ORDEM no store e as ENTRADAS no thread-local do painel. Aqui elas chegam por
    // duas chamadas porque a porta do produto pede um `&mut WidgetStore`, que este arnês não
    // empresta de propósito (ver `MockPanelHost::set_hierarchy_rows`).
    let ordered: Vec<ph2d_a11y::NodeId> = (0..ROWS)
        .map(|i| ph2d_a11y::NodeId(FIRST_ROW + i as u64))
        .collect();
    let entries: BTreeMap<_, _> = ordered
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, entry(&format!("Object {i}"))))
        .collect();
    ph2d_panel_hierarchy::state::set_live_entries(Some(entries));
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    host.set_hierarchy_rows(&ordered);
    let mut state = HierarchyState::default();
    let out = host.paint::<HierarchyPanel>(&mut state, Rect::new(0.0, 0.0, 1366.0, 1024.0));
    ph2d_panel_hierarchy::clear_live_hierarchy();
    (out, ordered)
}

/// Os rectângulos das LINHAS, de cima para baixo. ⚠️ Um id de linha é o id CRU da cena; os
/// companheiros são o mesmo id com um bit alto ligado, então filtrar pela lista entregue é o que
/// os separa sem adivinhar.
fn rows(painted: &[(ph2d_a11y::NodeId, Rect)], ordered: &[ph2d_a11y::NodeId]) -> Vec<Rect> {
    let mut out: Vec<Rect> = painted
        .iter()
        .filter(|(id, _)| ordered.contains(id))
        .map(|(_, r)| *r)
        .collect();
    out.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    out
}

/// ⭐⭐ **O passo de uma linha para a seguinte é a altura MAIS o vão de lista — e nada mais.**
///
/// **Mutação que deve sangrar:** trocar o `list_row_gap_px()` do `paint.rs` por
/// `Spacing::Xxs.px()` (o estado em que esta wave encontrou o app) ⇒ o passo lê 24 e a barra
/// exige 23.
#[test]
fn one_row_follows_the_next_by_the_list_pitch() {
    let (painted, ordered) = painted();
    let rows = rows(&painted, &ordered);
    assert!(
        rows.len() >= 3,
        "a fixtura pintou {} linha(s) — sem pelo menos tres nao ha passo para medir, e este \
         gate ficaria vacuo",
        rows.len()
    );
    let expected = ph2d_tokens::ROW_H_PX + ph2d_tokens::list_row_gap_px();
    for pair in rows.windows(2) {
        let step = pair[1].y - pair[0].y;
        assert!(
            (step - expected).abs() < 0.01,
            "duas linhas da hierarquia distam {step} px; a lei da lista da' {expected} \
             (altura {} + vao {})",
            ph2d_tokens::ROW_H_PX,
            ph2d_tokens::list_row_gap_px()
        );
        assert!(
            (pair[0].h - ph2d_tokens::ROW_H_PX).abs() < 0.01,
            "a linha da hierarquia mede {} px de altura e a linha do app mede {} — o token \
             `chrome.hier-row-h` morreu na wave 17 e alguem o ressuscitou",
            pair[0].h,
            ph2d_tokens::ROW_H_PX
        );
    }
}

/// ⭐⭐⭐ **Nenhum alvo de clique de uma linha sai da linha dela.**
///
/// ⚠️ **É o defeito que a wave 17 CRIARIA se ficasse pela metade:** os quatro companheiros
/// inflavam o ícone com folga própria (`16 + 2·4 = 24`, `12 + 12 = 24`), o que cabia numa linha de
/// 32 px e transborda numa de 22. E no `HitIndex` **quem regista depois ganha** ⇒ a fatia de baixo
/// da linha `N−1` passaria a comandar o olho da linha `N`, com todo gate de registo verde: o
/// companheiro está vivo, registado e alcançável — só grande demais.
///
/// **Mutação que deve sangrar:** devolver ao `row.rs` o `Rect::new(x, eye_rect.y - hit_pad, w,
/// eye_rect.h + hit_pad*2.0)` de antes.
#[test]
fn no_companion_target_leaves_its_own_row() {
    let (painted, ordered) = painted();
    let rows = rows(&painted, &ordered);
    assert!(!rows.is_empty(), "a fixtura nao pintou linha nenhuma");
    let mut checked = 0usize;
    let mut escapes = Vec::new();
    for (id, rect) in &painted {
        let row = ids::hier_eye_companion_to_row(*id)
            .or_else(|| ids::hier_lock_companion_to_row(*id))
            .or_else(|| ids::hier_group_companion_to_row(*id))
            .or_else(|| ids::hier_expand_companion_to_row(*id))
            .or_else(|| ids::hier_icon_companion_to_row(*id));
        let Some(row) = row else { continue };
        let Some((_, row_rect)) = painted.iter().find(|(i, _)| *i == row) else {
            continue;
        };
        checked += 1;
        if rect.y < row_rect.y - 0.01 || rect.y + rect.h > row_rect.y + row_rect.h + 0.01 {
            escapes.push(format!(
                "{id:?}: [{}, {}] contra a linha [{}, {}]",
                rect.y,
                rect.y + rect.h,
                row_rect.y,
                row_rect.y + row_rect.h
            ));
        }
    }
    assert!(
        checked >= 8,
        "so' {checked} companheiro(s) medidos — a fixtura deixou de os produzir e o gate ficou \
         vacuo (a deteccao de companheiro por bit alto e' o que ele le)"
    );
    assert!(
        escapes.is_empty(),
        "{} alvo(s) de clique saem da propria linha e roubam o clique a' vizinha:\n  {}",
        escapes.len(),
        escapes.join("\n  ")
    );
}

/// Pinta `names` como a cena viva, com o filtro de busca em `query`, e devolve
/// `(glifos, segmentos)` — o que a cena de facto recebeu.
fn painted_geometry(names: &[&str], query: &str) -> (u32, u32) {
    let ordered: Vec<ph2d_a11y::NodeId> = (0..names.len())
        .map(|i| ph2d_a11y::NodeId(FIRST_ROW + i as u64))
        .collect();
    let entries: BTreeMap<_, _> = ordered
        .iter()
        .zip(names)
        .map(|(id, n)| (*id, entry(n)))
        .collect();
    ph2d_panel_hierarchy::state::set_live_entries(Some(entries));
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    host.set_hierarchy_rows(&ordered);
    host.set_text(ph2d_panel_hierarchy::ids::HIER_SEARCH, query);
    let mut state = HierarchyState::default();
    let out = host.paint_and_count_geometry::<HierarchyPanel>(
        &mut state,
        Rect::new(0.0, 0.0, 1366.0, 1024.0),
    );
    ph2d_panel_hierarchy::clear_live_hierarchy();
    out
}

/// ⭐⭐⭐ **A listra segue a ordem que se VÊ, não o índice do dado.**
///
/// Report do dono (2026-09-06, foto do *Outliner* do Blender): *«linhas pares e ímpares têm
/// tonalidade discretamente diferente».*
///
/// ⚠️ **Este é o único gate que separa as duas leis, e a fixtura é construída para isso.** Oito
/// linhas com nomes alternados `keep`/`drop`: filtrar por `keep` deixa visíveis as de índice de
/// dado `0,2,4,6` — **todas pares** — e filtrar por `drop` deixa `1,3,5,7` — **todas ímpares**.
///
/// - pela lei **visual** (a certa) os dois casos pintam **duas** listras ⇒ a mesma geometria;
/// - pelo índice do **dado** um caso pinta **zero** e o outro **quatro**.
///
/// ⇒ a igualdade é a afirmação. **Mutação que deve sangrar:** trocar o `row_rects.len()` do
/// `paint.rs` pelo `i` do laço.
///
/// ⛔ E o defeito real que ela apanha não é o filtro: é **recolher um ramo**. Ali o artista vê
/// duas linhas do mesmo tom encostadas, de forma intermitente — o report seria *«às vezes as
/// listras somem»*, que é dos que não se reproduzem.
#[test]
fn the_stripe_follows_what_is_seen_not_the_index_of_the_data() {
    let names = [
        "keep", "drop", "keep", "drop", "keep", "drop", "keep", "drop",
    ];
    let (glyphs_even, seg_even) = painted_geometry(&names, "keep");
    let (glyphs_odd, seg_odd) = painted_geometry(&names, "drop");
    let (_, seg_all) = painted_geometry(&names, "");

    assert!(
        glyphs_even > 0 && glyphs_odd > 0,
        "nenhuma das duas corridas pintou texto — o filtro escondeu tudo e o gate ficou vacuo          ({glyphs_even} / {glyphs_odd} glifos)"
    );
    assert!(
        seg_all > seg_even,
        "as oito linhas ({seg_all} segmentos) nao emitem mais geometria que as quatro filtradas          ({seg_even}) — o arnes nao esta' a reagir ao numero de linhas, e a igualdade abaixo          seria verdadeira por vacuidade"
    );
    assert_eq!(
        seg_even, seg_odd,
        "as quatro linhas de indice PAR emitiram {seg_even} segmentos e as de indice IMPAR          {seg_odd}: a listra esta' a seguir o indice do DADO, entao recolher um ramo (ou filtrar)          encosta duas linhas do mesmo tom"
    );
}

/// ⭐⭐ **A listra EXISTE, e cai na linha ímpar** — a metade que o teste do índice não pode dar.
///
/// ⚠️ **O irmão acima é cego a esta:** se ninguém pintasse listra nenhuma, «par» e «ímpar»
/// emitiriam a mesma geometria e ele passaria. *Uma igualdade prova qual é a lei, nunca que a lei
/// corre.*
///
/// A régua é o **custo de acrescentar uma linha**: com linhas idênticas, cada uma acrescenta a
/// mesma geometria — **mais** a listra, quando ela calha ímpar. ⇒ os saltos têm de alternar entre
/// dois valores, e o maior é o que traz a listra.
///
/// **Mutação que deve sangrar:** apagar a chamada ao `paint_row_stripe` do `paint.rs`.
#[test]
fn adding_an_odd_row_costs_more_geometry_than_adding_an_even_one() {
    let same = ["row"; 7];
    let seg: Vec<u32> = (2..=6)
        .map(|n| painted_geometry(&same[..n], "").1)
        .collect();
    let steps: Vec<i64> = seg.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect();
    let big = *steps.iter().max().unwrap();
    let small = *steps.iter().min().unwrap();
    assert!(
        big > small,
        "acrescentar uma linha custa sempre {big} segmentos — nenhuma linha traz listra nenhuma,          logo a hierarquia nao chama a porta (saltos: {steps:?})"
    );
    for pair in steps.windows(2) {
        assert_ne!(
            pair[0], pair[1],
            "dois saltos seguidos custaram o mesmo ({:?}) — a listra deixou de ALTERNAR",
            steps
        );
    }
}
