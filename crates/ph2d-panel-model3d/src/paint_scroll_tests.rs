//! **O corpo do painel de modelagem ROLA** — report do Enio, 2026-08-27: *«o painel 3d Model
//! precisa de scroll e barra de scroll»*.
//!
//! ⛔ **Ele já RECORTAVA e nunca rolava, que é a pior das três formas:** um painel sem recorte
//! desenha por cima do título e vê-se; um que recorta e rola funciona; **um que recorta e não rola
//! esconde os controles e não diz nada.** O rodapé e as fileiras de parâmetros de um documento com
//! vários nós ficavam inalcançáveis, sem sinal nenhum de que existiam.
//!
//! ⚠️ **Fazer um painel rolar são QUATRO edições e só três falham alto** (o arch-gate
//! `scrollable_panels_intercept_the_wheel` nomeia-as): o id do polegar, o braço no
//! `scrollbar_panel_for_id`, o **pintor** (que lê o `panel_scroll` e publica `content_h`/
//! `visible_h`) e o id em `cursor_over_hero_panel`. Este arquivo é o juiz da terceira.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::zones::Rect;
use ph2d_field::{Bound, Param};

/// O viewport das cenas — grande o bastante para o dock abrir na altura cheia.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

/// Um modelo com `n` fileiras de parâmetro, o molde das fixtures deste arquivo.
fn model_with_rows(n: usize) -> state::ModelSnapshot {
    state::ModelSnapshot {
        rows: (0..n)
            .map(|i| state::ParamRow {
                entity: i as u64,
                param: Param::Scale,
                key: "field.dim.scale",
                value: 0.5,
                lo: 0.0,
                live: true,
                integral: false,
                section: None,
                choices: &[],
                bound: Bound::Soft(1.0),
            })
            .collect(),
        ..Default::default()
    }
}

fn paint(host: &mut ph2d_ui_testkit::MockPanelHost) -> Vec<(NodeId, Rect)> {
    let mut st = state::Model3dPanelState;
    host.paint::<Model3dPanel>(&mut st, VIEWPORT)
}

/// ⭐⭐ **O painel PUBLICA a altura do CONTEÚDO, nunca a da moldura.**
///
/// ⚠️ **É a metade que não se vê e sem a qual a roda não faz nada:** o `dispatch_wheel` deriva o
/// `max_scroll` de `content_h`/`visible_h`, então um painel que recorta, desloca e até desenha o
/// polegar — mas não publica — **rola pelo polegar e fica inerte na roda**.
#[test]
fn the_panel_publishes_the_height_of_its_content_not_of_its_frame() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<Model3dPanel>();

    state::publish(model_with_rows(MAX_ROWS));
    paint(&mut host);
    // ⚠️ `expect` e não `unwrap_or(0.0)`: *não publicou nada* e *publicou pouco* são falhas
    // diferentes, e um default silencioso colapsá-las-ia na segunda.
    let cheio = host
        .store()
        .panel_content_h(ids::MODEL3D_PANEL)
        .expect("o painel publica a altura do CONTEÚDO");
    let visivel = host
        .store()
        .panel_visible_h(ids::MODEL3D_PANEL)
        .expect("o painel publica a altura VISÍVEL — sem ela o dispatch da roda não tem régua");
    assert!(visivel > 0.0, "a altura visível não pode ser zero");

    state::publish(model_with_rows(1));
    paint(&mut host);
    let curto = host
        .store()
        .panel_content_h(ids::MODEL3D_PANEL)
        .expect("o painel publica a altura do CONTEÚDO");
    assert!(
        curto < cheio,
        "um modelo de uma fileira publica a mesma altura de um de {MAX_ROWS} ({curto} contra \
         {cheio}) — o número publicado é o do conteúdo, não o do dock"
    );
}

/// ⭐⭐ **Rolar move as fileiras pelo número EXACTO do rolamento.**
///
/// O oráculo é o rectângulo de HIT, e não um pixel desenhado: é ele que decide onde o artista
/// clica. *Uma rolagem que desenhasse deslocado sem mover o hit deixaria o controle a responder
/// onde ele **estava** — a forma mais cruel deste defeito, porque a tela parece certa.*
#[test]
fn scrolling_moves_the_rows_by_exactly_the_scroll() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<Model3dPanel>();
    state::publish(model_with_rows(MAX_ROWS));

    let parado = paint(&mut host);
    assert!(!parado.is_empty(), "o painel tem de registar alguma coisa");

    const PASSO: f32 = 37.0;
    host.set_panel_scroll(ids::MODEL3D_PANEL, PASSO);
    let rolado = paint(&mut host);

    // ⚠️ **O oráculo é o MESMO id antes e depois**, e não o topo da lista: o topo é o botão de
    // fechar, que é **cromo do título** e não rola. *Uma régua que apanha o cromo mede a moldura e
    // não o corpo.*
    //
    // ⭐ **Nada se move MAIS do que o rolamento**, e pelo menos um move-se exactamente ele.
    //
    // ⚠️ **A lei não é uma igualdade, e a razão é o recorte:** uma fileira que passa a ficar meio
    // fora do corpo tem o hit-rect **aparado** pela banda, e o topo dela sobe menos do que o
    // rolamento inteiro (medido: `32,0` de `37`). *O rect registado é a parte VISÍVEL, que é
    // exactamente o que o clique tem de encontrar* — apará-lo é a blindagem a funcionar, e não um
    // desvio.
    let mut exactos = 0usize;
    for (id, antes) in &parado {
        let Some((_, depois)) = rolado.iter().find(|(o, _)| o == id) else {
            continue;
        };
        let delta = antes.y - depois.y;
        assert!(
            delta <= PASSO + 0.5,
            "o widget {id:?} subiu {delta:.1} com um rolamento de {PASSO} — nada pode andar mais do \
             que o rolamento pedido"
        );
        if (delta - PASSO).abs() < 0.5 {
            exactos += 1;
        }
    }
    assert!(
        exactos > 0,
        "CONTROLO: nenhum widget se moveu os {PASSO} inteiros — ou o corpo não rola, ou tudo o que \
         sobrou está aparado pela banda e o gate não está a medir a rolagem"
    );
}

/// ⭐⭐⭐ **Nada rolado para cima continua clicável debaixo do TÍTULO** — *uma banda, dois
/// consumidores*.
///
/// ⚠️ O `push_clip` da cena recorta o **desenho**; sem o gémeo no `HitIndex`, uma fileira rolada
/// para cima continua **registada** onde ninguém a vê, e o hit-rect dela sobe para a faixa do
/// título. É o defeito que o painel do Motion já pagou, e ligar a rolagem é o dia em que ele passa
/// a morder aqui.
#[test]
fn nothing_scrolled_above_the_body_can_still_be_clicked_under_the_title() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<Model3dPanel>();
    state::publish(model_with_rows(MAX_ROWS));

    let parado = paint(&mut host);
    let topo = parado.iter().map(|(_, r)| r.y).fold(f32::MAX, f32::min);
    assert!(topo.is_finite(), "o painel tem de registar alguma coisa");

    let conteudo = host
        .store()
        .panel_content_h(ids::MODEL3D_PANEL)
        .expect("o painel publica a altura do conteúdo");
    let visivel = host
        .store()
        .panel_visible_h(ids::MODEL3D_PANEL)
        .expect("o painel publica a altura visível");
    // ⛔ **O CONTROLO.** Se o conteúdo coubesse no corpo nada rolaria e este gate mediria o nada —
    // *um zero de «não mediu» e um de «perfeito» são o mesmo byte*.
    assert!(
        conteudo > visivel,
        "CONTROLO: o conteúdo ({conteudo}) cabe no corpo ({visivel}), então nada rola e este gate \
         não está a prender nada"
    );

    host.set_panel_scroll(ids::MODEL3D_PANEL, 10_000.0);
    let rolado = paint(&mut host);
    assert!(
        !rolado.is_empty(),
        "CONTROLO: o corpo rolado continua a registar o que está visível"
    );
    let intruso = rolado.iter().find(|(_, r)| r.y < topo - 0.5);
    assert!(
        intruso.is_none(),
        "uma fileira rolada para cima continua registada em y={:.1}, acima do corpo ({topo:.1}) — o \
         hit-rect dela vive sob o TÍTULO",
        intruso.map_or(0.0, |(_, r)| r.y)
    );
    // E o rolamento assentou onde o clamp manda, nunca no valor pedido.
    let assentou = host.store().panel_scroll(ids::MODEL3D_PANEL);
    assert!(
        (assentou - (conteudo - visivel)).abs() < 1.0,
        "o clamp tinha de assentar em conteúdo−visível ({}); assentou em {assentou}",
        conteudo - visivel
    );
}

/// ⭐ **Um modelo que ENCOLHE puxa o rolamento de volta.**
///
/// ⚠️ Apagar um nó na Hierarquia tira fileiras. Sem o clamp, um painel rolado até ao fim abriria
/// **em branco** — e o artista veria o painel vazio de um documento que tem conteúdo.
#[test]
fn a_model_that_shrinks_pulls_the_scroll_back() {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<Model3dPanel>();
    state::publish(model_with_rows(MAX_ROWS));
    paint(&mut host);
    host.set_panel_scroll(ids::MODEL3D_PANEL, 10_000.0);
    paint(&mut host);
    let rolado = host.store().panel_scroll(ids::MODEL3D_PANEL);
    assert!(rolado > 0.0, "CONTROLO: o modelo cheio tem de rolar");

    state::publish(model_with_rows(1));
    paint(&mut host);
    assert_eq!(
        host.store().panel_scroll(ids::MODEL3D_PANEL),
        0.0,
        "uma fileira cabe no corpo e o rolamento tinha de voltar a zero"
    );
}
