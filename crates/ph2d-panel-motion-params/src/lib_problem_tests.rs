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
            help: None,
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

/// ⭐⭐⭐ **A QUEIXA CHEGA A TINTA — e não só a espaço reservado.**
///
/// ⛔⛔ **Este gate nasceu de o irmão de baixo estar errado no ponto que decidia tudo** (achado
/// §4.2 da auditoria de seis lentes). A altura é escrita por `y += ROW_H_PX + row_gap`, que é
/// **outra linha** que não o `paint_text_elided`: apagar a pintura inteira deixava o
/// `a_text_row_with_a_problem_takes_more_room_than_one_without` **verde**, e o doc-comment deste
/// ficheiro prometia ser *«a metade que prova que ela chega a pixel»*.
///
/// ⇒ a régua passa a ser o número de GLIFOS que o painel emite para a cena Vello: espaço
/// reservado não emite nenhum.
///
/// ⚠️⚠️ **E a 1.ª redacção desta régua estava ERRADA** — ela contava `n_path_segments` e leu
/// **`42` contra `42`**: o Vello encaminha texto por `draw_glyphs`, cuja saída vive em
/// `resources.glyphs`, e **nenhum glifo entra na contagem de caminhos**. *Uma régua que devolve o
/// mesmo número dos dois lados não distingue «não pintou» de «não vejo o que ele pintou», e ler
/// aquele empate como acusação teria mandado alguém consertar código que estava certo.*
///
/// ⚠️⚠️ **E a METADE QUE ENCHE O BALDE é a segunda:** o arnês monta um `TextSystem` **sem fontes
/// de sistema**, então um gate que só comparasse «com queixa» contra «sem» ficaria verde no dia
/// em que nenhum glifo resolvesse — *os dois lados a zero, e um zero de «não medido» e um de
/// «igual» são o mesmo byte*. Medir que uma queixa **mais longa** emite mais segmentos só passa
/// se os glifos de facto saírem.
#[test]
fn the_complaint_reaches_ink_and_not_only_reserved_space() {
    let segments = |problem: Option<&str>| -> u32 {
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
        set_current_params(Some(node_with_text(problem)));
        let mut state = MotionParamsPanelState;
        host.paint_and_count_geometry::<MotionParamsPanel>(&mut state, VIEWPORT)
            .0
    };
    let mudo = segments(None);
    let curta = segments(Some("erro"));
    let longa = segments(Some(
        "«A -> (40%) F»: o peso tem de ser um numero entre zero e um",
    ));
    assert!(
        curta > mudo,
        "a queixa nao emitiu glifo nenhum ({curta} contra {mudo}) — ela ocupa a linha e nao pinta \
         nada"
    );
    assert!(
        longa > curta,
        "uma queixa MAIS LONGA emitiu {longa} glifos contra {curta} da curta — ou os glifos nao \
         saem, ou o que cresce e' a caixa e nao o texto"
    );
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

/// ⛔⛔⛔ **A AJUDA CHEGA AO STORE, SOB O ID QUE O RATO USA** — e não a uma tabela qualquer.
///
/// # A costura que este gate mede
///
/// O `paint_hover_tooltip` (que corre depois de TODOS os painéis, sobre o viewport inteiro) lê
/// `store.tooltip_for(store.hot_id())`, e o `hot_id` vem do **hit-index**. A row de texto
/// regista o campo dela sob `param_text_id(i)`.
///
/// ⚠️⚠️ **Registar a dica noutro id seria uma dica que existe e ninguém alcança** — é a forma de
/// gate vazio que a auditoria deste repo apanhou dezenas de vezes, e é por isso que a régua é o
/// id, e não a presença.
///
/// ⚠️ E a metade oposta: uma row **sem** ajuda não pode deixar uma dica para trás — o painel
/// re-semeia a cada quadro, e uma dica órfã apareceria sobre o campo do nó seguinte.
#[test]
fn the_help_reaches_the_store_under_the_id_the_hover_reads() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    const AJUDA: &str = "F G anda e desenha · [ ] abre / fecha um ramo";

    let com = ParamsSnapshot {
        node: 7,
        title: "Fixture".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Text(TextRow {
            name: "rules",
            label: "Rules".into(),
            value: "F -> FF".into(),
            problem: None,
            help: Some(AJUDA.to_string()),
        })],
    };
    set_current_params(Some(com));
    let mut state = MotionParamsPanelState;
    let rects = host.paint::<MotionParamsPanel>(&mut state, VIEWPORT);

    let id = crate::snapshot::param_text_id(0);
    assert!(
        rects.iter().any(|(r, _)| *r == id),
        "o campo da row 0 tem de estar no hit-index sob `param_text_id(0)` — senão o `hot_id` \
         nunca é esse e a dica é inalcançável"
    );
    assert_eq!(
        host.store().tooltip_for(id),
        Some(AJUDA),
        "a ajuda não chegou ao store sob o id que o hover lê"
    );

    // ⚠️ A metade oposta: sem ajuda, nada fica para trás.
    let sem = ParamsSnapshot {
        node: 8,
        title: "Fixture".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Text(TextRow {
            name: "expr",
            label: "Formula".into(),
            value: "sin(t)".into(),
            problem: None,
            help: None,
        })],
    };
    set_current_params(Some(sem));
    host.paint::<MotionParamsPanel>(&mut state, VIEWPORT);
    assert_eq!(
        host.store().tooltip_for(id),
        None,
        "a dica do nó ANTERIOR sobreviveu — ela apareceria sobre o campo deste"
    );
}
