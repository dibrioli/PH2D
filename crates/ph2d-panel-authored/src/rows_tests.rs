//! Gates da **tabela viva** (plano UI/UX W8b.2).

use super::*;
use ph2d_editor_core::ids;

/// **A lista viva É a lista gerada, na ordem dela.**
///
/// ⚠️ O oráculo é `generated::ROWS` e não uma cópia à mão: uma segunda lista aqui divergiria no
/// dia em que o artista acrescentasse um filho, e o gate ficaria verde sobre o painel errado.
#[test]
fn the_rows_are_the_generated_table_in_order() {
    let live = rows();
    assert_eq!(live.len(), crate::generated::ROWS.len());
    for (r, (kind, label, key)) in live.iter().zip(crate::generated::ROWS) {
        assert_eq!(r.kind, *kind);
        assert_eq!(r.label, *label);
        assert_eq!(r.key, *key);
    }
}

/// **O id de cada row sai da CHAVE, e nenhum colide com o outro nem com o chrome.**
///
/// ⚠️ O gerador **não cunha ids** (ver o doc de `ids::authored_row_id`), então a unicidade da
/// família contra o chrome estático não é verificável na foundational — as chaves REAIS só
/// existem aqui. É o mesmo arranjo do `wet_tuning_ids_dont_collide`.
#[test]
fn the_row_ids_do_not_collide_with_each_other_nor_with_the_chrome() {
    let mut seen: Vec<(ph2d_a11y::NodeId, &str)> = vec![
        (ids::AUTHORED_PANEL, "panel"),
        (ids::AUTHORED_CLOSE, "close"),
        (ids::AUTHORED_DRAG_HANDLE, "drag"),
        (ids::AUTHORED_RESIZE_HANDLE, "resize"),
        (ids::AUTHORED_RESIZE_HANDLE_BL, "resize_bl"),
    ];
    for r in rows() {
        // ⚠️ Duas rows de mesmo RÓTULO têm a mesma chave de propósito (ver o doc de
        // `duplicate_keys`), então o que se afirma é: chaves distintas dão ids distintos.
        if seen.iter().any(|(_, k)| *k == r.key) {
            continue;
        }
        assert!(
            !seen.iter().any(|(id, _)| *id == r.id),
            "a row `{}` colide com um id ja' usado",
            r.key
        );
        seen.push((r.id, r.key));
    }
}

/// **Só quem RESPONDE é controle** — a segunda metade da lei do §4.
///
/// ⚠️ Afirmada por TIPO, e não pela lista da cena: a cena pode mudar de desenho, a lei não. Um
/// `Divider` registado acenderia sob o rato e não faria nada — o item-de-menu-morto com outro
/// nome; e um `Slider` NÃO registado ficaria desenhado e imóvel sob o dedo.
#[test]
fn only_the_kinds_that_respond_are_controls() {
    let is = |kind| {
        Row {
            kind,
            label: "x",
            key: "x",
            id: ph2d_a11y::NodeId(1),
        }
        .is_control()
    };
    for k in [
        WidgetKind::Button,
        WidgetKind::Toggle,
        WidgetKind::Checkbox,
        WidgetKind::Slider,
        WidgetKind::TextInput,
        WidgetKind::Tag,
        WidgetKind::ListItem,
    ] {
        assert!(is(k), "{k:?} responde a um gesto e nao e' controle");
    }
    for k in [
        WidgetKind::ProgressBar,
        WidgetKind::SectionHeader,
        WidgetKind::Card,
        WidgetKind::Spinner,
        WidgetKind::Divider,
    ] {
        assert!(!is(k), "{k:?} so' desenha e foi tratado como controle");
    }
}

/// **Rótulos repetidos são CONTADOS, nunca renomeados.**
///
/// ⚠️ Inventar um sufixo (`opacity_2`) daria uma chave que o artista não escreveu, não vê e que
/// mudaria sozinha ao reordenar os filhos. O painel diz que há repetidos; quem desempata é ele, na
/// Hierarquia, com o nome.
#[test]
fn duplicate_labels_are_counted_not_renamed() {
    // A cena que shipa não tem repetidos — este é o CONTROLE.
    assert_eq!(duplicate_keys(), 0);
}

/// **A pintura entrega o ESTADO DO STORE, e não `None`.**
///
/// ⚠️ Arch-gate sobre o próprio `paint.rs`, e ele existe porque uma mutação sobreviveu a tudo o
/// mais: trocar `store.get(row.id)` por `None` deixa **todos** os gates de comportamento verdes —
/// as rows continuam vivas, o arrasto continua a mover o valor no store — e cada slider passa a
/// pintar a meio para sempre. O defeito só aparece numa screenshot, que é o modo de falha mais
/// caro que este repo conhece.
///
/// A metade complementar mora na porta (`the_live_state_reaches_the_paint`, em `skin/tests.rs`):
/// lá se prova que o estado MUDA a cena; aqui, que este painel o entrega.
#[test]
fn the_paint_hands_the_store_state_to_the_skin() {
    let src = include_str!("paint.rs");
    // Controle positivo: uma varredura vazia tornaria este gate verde por vácuo.
    assert!(
        src.contains("paint_widget_skin_with("),
        "o paint nao chama a porta da pele"
    );
    let call = src
        .find("paint_widget_skin_with(\n")
        .expect("a chamada da pele sumiu do paint");
    let window = &src[call..(call + 400).min(src.len())];
    assert!(
        window.contains("store.get(row.id)"),
        "a pele recebe um estado que nao e' o do store — os controles pintariam o valor de \
         PREVIA para sempre, com toda a suite verde"
    );
}
