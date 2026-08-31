//! ⭐⭐⭐ **A QUEIXA É PINTADA** — a metade que um gate de dados não alcança.
//!
//! # Por que este ficheiro existe
//!
//! A shell mede que a `TextRow` **carrega** a queixa, e a crate do nó mede que a queixa **nasce**
//! do mesmo `return Err` que descarta a regra. ⚠️⚠️ **Nenhuma das duas prova que ela chega a
//! pixel** — e este repo tem a lição escrita: *um controlo nunca pintado e um morto sob o dedo
//! dão o MESMO report*. Um `match` que caísse no braço das irmãs (Angle/Seed/Text) deixaria os
//! dois gates verdes e o artista sem aviso nenhum.
//!
//! # A régua, e por que é a ALTURA
//!
//! Um aviso não se clica, então ele **não está no `HitIndex`** — de propósito: registá-lo poria
//! um alvo mudo por cima do campo. ⇒ a sonda de focalizabilidade não o vê, e a única marca que
//! ele deixa no estado publicado é **ocupar uma linha**. A régua é o `content_h` que o painel
//! publica: a MESMA row, com e sem queixa, tem de publicar alturas diferentes.
//!
//! ⚠️ **A comparação é entre duas pinturas do mesmo fixture**, e não contra um número escrito
//! aqui: um `assert_eq!(h, 68.0)` mediria o tema e a fonte, e ficaria vermelho no dia em que
//! alguém mexesse num token sem tocar nesta feature.

use super::*;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::ROW_H_PX;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

/// Uma row de texto, com ou sem queixa — tudo o resto igual ao bit.
fn node_with_text(problem: Option<&str>) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Fixture".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Text(TextRow {
            name: "rules",
            label: "Rules".into(),
            value: "A -> (40%) F".into(),
            problem: problem.map(str::to_string),
        })],
    }
}

fn paint_and_measure(problem: Option<&str>) -> f32 {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_text(problem)));
    let mut state = MotionParamsPanelState;
    host.paint::<MotionParamsPanel>(&mut state, VIEWPORT);
    host.store()
        .panel_content_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura do CONTEÚDO")
}

#[test]
fn a_text_row_with_a_problem_takes_more_room_than_one_without() {
    let mudo = paint_and_measure(None);
    let queixoso = paint_and_measure(Some("«A -> (40%) F»: o peso tem de ser um número"));
    assert!(
        queixoso > mudo,
        "a queixa não ocupa espaço nenhum ({queixoso} contra {mudo}) — ou ela não é pintada, \
         ou caiu no braço das rows-caixa e desapareceu"
    );
    // ⚠️ **UMA linha, e não uma altura qualquer** — se o braço passasse a desenhar a lista
    // inteira, o painel cresceria com o número de regras e empurraria o resto para fora do dock
    // enquanto o artista escreve, que é precisamente quando ele tem várias regras a meio.
    let uma_linha = queixoso - mudo;
    assert!(
        uma_linha <= ROW_H_PX * 2.0,
        "a queixa cresceu {uma_linha} px — é para ser UMA linha (~{ROW_H_PX})"
    );
}

#[test]
fn the_complaint_never_steals_a_click() {
    // ⛔ Um aviso não é um alvo. Registá-lo poria um rect mudo por cima do campo de texto, e o
    // artista clicaria na caixa sem a focar — o defeito «morto sob o ponteiro», ao contrário.
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_text(None)));
    let mut state = MotionParamsPanelState;
    let mudo = host.paint::<MotionParamsPanel>(&mut state, VIEWPORT).len();

    set_current_params(Some(node_with_text(Some("«A»: erro"))));
    let queixoso = host.paint::<MotionParamsPanel>(&mut state, VIEWPORT).len();
    assert_eq!(
        mudo,
        queixoso,
        "a queixa registou {} alvo(s) novo(s) no hit-index — ela não se clica",
        queixoso as i64 - mudo as i64
    );
    // ⚠️ O controlo do próprio filtro: com zero alvos os dois lados seriam `0` e o teste ficava
    // verde a não medir nada.
    assert!(
        mudo > 0,
        "a row de texto tem de registar o campo — senão este gate não mede nada"
    );
}
