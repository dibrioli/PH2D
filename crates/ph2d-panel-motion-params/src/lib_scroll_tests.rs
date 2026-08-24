//! **O corpo do painel de params ROLA** (doc 88 §B3).
//!
//! O painel desenhava uma lista de altura fixa e o teto de linhas o defendia — `MAX_PARAM_ROWS`
//! era, ao mesmo tempo, um teto de POOL de ids e um teto de ALTURA. Medido: uma linha escalar
//! ocupa **34 px** e o dock comporta **24** delas, contra um teto de 16 e um pior nó
//! (`motion.tint`) de **15 params**. São oito linhas de folga para uma varredura que promete a
//! TODO nó o conjunto PRO, e o gate `a_full_panel_of_rows_fits_the_inspector` já dizia o que
//! fazer no dia: *"o painel precisa ROLAR antes de o teto subir mais"*.
//!
//! Com a rolagem, as duas perguntas se separam: o teto de linhas volta a ser só sobre o POOL de
//! ids, e a altura deixa de ser um limite de produto.

use super::*;
use ph2d_editor_core::zones::Rect;

/// O viewport das cenas — grande o bastante para o inspector abrir na altura cheia.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

/// Um nó com `n` linhas escalares, o molde das fixtures deste arquivo.
fn node_with_rows(n: usize) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Fixture".into(),
        modified: Default::default(),
        sections: Vec::new(),
        rows: (0..n)
            .map(|i| {
                ParamRow::Scalar(ScalarRow {
                    name: "p",
                    label: format!("Param {i}"),
                    value: 0.5,
                    min: 0.0,
                    max: 1.0,
                    hard_min: 0.0,
                    hard_max: 1.0,
                    step: 0.01,
                    integer: false,
                    driven_by: None,
                    display: Default::default(),
                })
            })
            .collect(),
    }
}

fn paint(host: &mut ph2d_ui_testkit::MockPanelHost) -> Vec<(NodeId, Rect)> {
    let mut state = MotionParamsPanelState;
    host.paint::<MotionParamsPanel>(&mut state, VIEWPORT)
}

/// **O painel PUBLICA a altura do CONTEUDO, nunca a da moldura.**
///
/// ⚠️ Este gate chamava-se `a_tall_node_publishes_more_content_than_the_dock_can_show` e o nome
/// afirmava mais do que ele mede: no teto de hoje (`MAX_PARAM_ROWS`) o corpo mede ~544 px contra
/// um dock de 880, entao **nenhum no transborda** e a rolagem esta inerte. O que ele de fato
/// prova — e que e o que a roda consome — e que o numero publicado segue o CONTEUDO.
///
/// ⚠️ Esta é a metade que não se vê e sem a qual a roda não faz nada: o `dispatch_wheel` deriva
/// o `max_scroll` de `content_h`/`visible_h`, então um painel que recorta, desloca e até desenha
/// o thumb — mas não publica — **rola pelo thumb e fica inerte na roda**. O gate afirma os dois
/// lados, porque só a comparação distingue *transborda* de *cabe*.
#[test]
fn the_panel_publishes_the_height_of_its_content_not_of_its_frame() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();

    set_current_params(Some(node_with_rows(MAX_PARAM_ROWS)));
    paint(&mut host);
    // ⚠️ `expect` e não `unwrap_or(0.0)`: *não publicou nada* e *publicou pouco* são falhas
    // diferentes, e um default silencioso as colapsaria na segunda.
    let tall_content = host
        .store()
        .panel_content_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura do CONTEÚDO");
    let visible = host
        .store()
        .panel_visible_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura VISÍVEL — sem ela o dispatch da roda não tem régua");
    assert!(visible > 0.0, "a altura visível não pode ser zero");

    set_current_params(Some(node_with_rows(2)));
    paint(&mut host);
    let short_content = host
        .store()
        .panel_content_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura do CONTEÚDO");

    assert!(
        short_content < tall_content,
        "um nó de 2 params não pode publicar a mesma altura de conteúdo de um de \
         {MAX_PARAM_ROWS} ({short_content} contra {tall_content}) — o número publicado é o do \
         conteúdo, não o do dock"
    );
    assert!(
        short_content <= visible,
        "duas linhas cabem no dock ({short_content} de {visible}): oferecer barra aqui seria \
         um controle que não faz nada"
    );
}

/// **Rolar move as linhas pelo número EXATO do rolamento.**
///
/// O oráculo é o retângulo de HIT da primeira linha, não um pixel desenhado: é ele que decide
/// onde o artista clica, e uma rolagem que desenhasse deslocado sem mover o hit deixaria o
/// slider respondendo onde ele *estava* — a forma mais cruel deste defeito, porque a tela
/// parece certa.
#[test]
fn scrolling_moves_the_rows_by_exactly_the_scroll() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_rows(MAX_PARAM_ROWS)));

    let at_rest = paint(&mut host);
    // ⚠️ **Não é a PRIMEIRA linha, e a razão é a blindagem** (ver
    // `nothing_scrolled_above_the_body_can_still_be_clicked_under_the_title`):
    // desde que o `HitIndex` recebe a mesma banda que a cena, uma linha rolada
    // para fora do corpo deixa de se registar — de propósito. Este gate mede que
    // o desenho e o hit ANDAM JUNTOS, e para isso precisa de uma linha que ainda
    // esteja lá depois de andar: a oitava (34 px cada) sobra com folga sobre os
    // 100 px de rolamento.
    let first = param_slider_id(8);
    let before = at_rest
        .iter()
        .find(|(id, _)| *id == first)
        .map(|(_, r)| r.y)
        .expect("a linha registra um hit rect");

    const SCROLL: f32 = 100.0;
    host.store_mut()
        .set_panel_scroll(ids::MOTION_PARAMS_PANEL, SCROLL);
    let scrolled = paint(&mut host);
    let after = scrolled
        .iter()
        .find(|(id, _)| *id == first)
        .map(|(_, r)| r.y)
        .expect("a linha continua registrando um hit rect");

    assert!(
        (before - after - SCROLL).abs() < 0.01,
        "a linha tinha de subir exatamente {SCROLL} px e subiu {} — o desenho e o hit têm de \
         andar juntos",
        before - after
    );
}

/// **Trocar para um nó mais curto puxa o rolamento de volta.**
///
/// O conteúdo ENCOLHE ao trocar de seleção, e um rolamento que sobrevivesse ao fim do corpo
/// novo abriria o painel **em branco** — com o nó certo selecionado, os params existindo, e
/// nada na tela. É o modo de falha que parece "o painel quebrou".
#[test]
fn switching_to_a_shorter_node_pulls_the_scroll_back() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_rows(MAX_PARAM_ROWS)));
    paint(&mut host);
    host.store_mut()
        .set_panel_scroll(ids::MOTION_PARAMS_PANEL, 300.0);
    paint(&mut host);

    set_current_params(Some(node_with_rows(2)));
    paint(&mut host);
    let scroll = host.store().panel_scroll(ids::MOTION_PARAMS_PANEL);
    assert_eq!(
        scroll, 0.0,
        "duas linhas cabem inteiras, então não sobra rolamento nenhum a honrar"
    );
}

/// **SONDA — uma linha rolada para fora do topo continua registrada onde?**
///
/// O `push_clip` recorta o DESENHO; o `HitIndex` recebe o retângulo no `y` já deslocado. Se esse
/// `y` sobe acima do corpo, o retângulo de hit passa a morar sobre o TÍTULO do painel (ou fora
/// dele) — e um clique ali resolveria uma linha que o artista não vê. A sonda mede em vez de
/// supor, e imprime o número; ela é `#[ignore]` porque é diagnóstico, não gate.
#[test]
#[ignore = "sonda de diagnostico: cargo test -- --ignored measure_the_scrolled_row"]
fn measure_the_scrolled_row_hit_rect() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_rows(MAX_PARAM_ROWS)));

    let at_rest = paint(&mut host);
    let first = param_slider_id(0);
    let rest_y = at_rest
        .iter()
        .find(|(id, _)| *id == first)
        .map(|(_, r)| r.y)
        .expect("hit rect");

    for scroll in [0.0_f32, 60.0, 200.0, 400.0] {
        host.store_mut()
            .set_panel_scroll(ids::MOTION_PARAMS_PANEL, scroll);
        let painted = paint(&mut host);
        let Some((_, r)) = painted.iter().find(|(id, _)| *id == first) else {
            println!("scroll {scroll:6.1}: a 1a linha NAO registra hit rect");
            continue;
        };
        let body_top = rest_y; // a 1a linha em repouso marca onde o corpo comeca
        println!(
            "scroll {scroll:6.1}: hit y={:7.1} (corpo comeca ~{body_top:.1}) -> {}",
            r.y,
            if r.y + r.h < body_top {
                "FORA do corpo, e ainda registrado"
            } else {
                "dentro"
            }
        );
    }
}

/// ⭐ **O QUE A CENA ESCONDE, O RATO TAMBÉM NÃO ALCANÇA** — a blindagem que o
/// gate anterior deste nome pedia, agora construída.
///
/// ⚠️ **A premissa do gate antigo DISSOLVEU, e é isso que ele existia para
/// apanhar.** Ele afirmava que a rolagem estava *inerte* (`content <= visible`)
/// e explicava porquê: o `push_clip` recortava o DESENHO, mas o `HitIndex`
/// recebia o retângulo no `y` já deslocado, então uma linha rolada para cima
/// continuava **registada** dentro da faixa do TÍTULO. O que tornava isso
/// inofensivo era aritmética — o teto de linhas cabia no dock, `max_scroll` era
/// `0` — e o doc dele dizia, em texto: *"o dia em que o teto subir é o dia em que
/// a blindagem passa a ser necessária"*.
///
/// O teto subiu (20 → 24, a wave do `motion.bezier_warp`) e a blindagem foi
/// escrita: **uma banda, dois consumidores** — o `HitIndex::push_clip` recebe o
/// mesmo `body_rect` que a cena. ⚠️ **A ferramenta já existia** (é a mesma pilha
/// que o `section_header::body` usa desde que nasceu); o que faltava era este
/// painel chamá-la.
///
/// *Mutação: tirar o `hit_index.push_clip` do painel ⇒ uma linha rolada para cima
/// volta a registar-se sob o título.*
#[test]
fn nothing_scrolled_above_the_body_can_still_be_clicked_under_the_title() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    set_current_params(Some(node_with_rows(MAX_PARAM_ROWS)));

    // Onde o corpo COMEÇA: o topo do que se regista com o rolamento em zero.
    let at_rest = paint(&mut host);
    let body_top = at_rest.iter().map(|(_, r)| r.y).fold(f32::MAX, f32::min);
    assert!(
        body_top.is_finite(),
        "o painel tem de registar alguma coisa"
    );

    // O conteúdo agora EXCEDE o corpo — a rolagem deixou de ser inerte, que é a
    // razão de esta blindagem existir.
    let content = host
        .store()
        .panel_content_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura do conteudo");
    let visible = host
        .store()
        .panel_visible_h(ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura visivel");
    assert!(
        content > visible,
        "CONTROLE: se o conteudo ({content}) coubesse no corpo ({visible}) nada rolaria, \
         e este gate estaria a medir o nada"
    );

    // Rola até ao fim e confere que NADA subiu para a faixa do título.
    host.store_mut()
        .set_panel_scroll(ids::MOTION_PARAMS_PANEL, 10_000.0);
    let scrolled = paint(&mut host);
    assert!(
        !scrolled.is_empty(),
        "CONTROLE: o corpo rolado continua a registar o que esta' visivel"
    );
    let intruder = scrolled.iter().find(|(_, r)| r.y < body_top - 0.5);
    assert!(
        intruder.is_none(),
        "uma linha rolada para cima continua registada em y={:.1}, acima do corpo ({body_top:.1}) \
         — o hit-rect dela vive sob o TITULO",
        intruder.map_or(0.0, |(_, r)| r.y)
    );
    // E o rolamento assentou onde o clamp manda, nunca no valor pedido.
    let settled = host.store().panel_scroll(ids::MOTION_PARAMS_PANEL);
    assert!(
        (settled - (content - visible)).abs() < 1.0,
        "o clamp tinha de assentar em content-visible; assentou em {settled}"
    );
}
