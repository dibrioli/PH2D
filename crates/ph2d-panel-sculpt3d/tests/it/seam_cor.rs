//! ⭐⭐⭐ **A COSTURA DA CAIXA DE COR** — o gesto REAL, de ponta a ponta.
//!
//! Ordem do dono (2026-09-20): *«troque os sliders de cor pelo seletor de Cor
//! (caixa de cor)»*.
//!
//! # Porque ela precisa de um ficheiro próprio
//!
//! O sweep do [`super::seam`] arma o **Crease**, e o `Crease` não deposita cor
//! — com ele na mão a amostra **nem é desenhada**. *Uma fixtura que não contém
//! o fenómeno não afirma nada sobre ele*, e é a sétima vez que esta crate o
//! escreve.
//!
//! # ⚠️ E ela não pode pedir um `WidgetEvent::Click`
//!
//! A amostra é uma espécie que o sweep irmão não sabe medir: o braço do
//! `pointer_down` que a serve **devolve antes** de o foco ser calculado, logo
//! ela nunca produz `Click` e nunca é focável no store. O que prova que ela está
//! viva é **o selector abrir**, e é isso que estes gates afirmam.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_sculpt3d::{
    Sculpt3dIntent, Sculpt3dPanel, Sculpt3dPanelState, Sculpt3dSnapshot, Sculpt3dUi, drain_intents,
    ids, set_current_sculpt3d,
};
use ph2d_sculpt3d::Verb;
use ph2d_ui_testkit::MockPanelHost;

/// Alto, porque a amostra vive no topo da cauda da secção do pincel e um paint
/// sem espaço não registaria nada — e passaria calado.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// ⚠️ **O retrato mínimo, e o `Paint` é a premissa declarada em vez de
/// herdada:** um `Verb::default()` que passasse a depositar cor deixaria este
/// ficheiro verde por acidente, e um que deixasse de o fazer deixá-lo-ia verde
/// por vácuo. Os anti-vácuos abaixo são o que separa os dois casos.
fn arrange(ui: Sculpt3dUi) -> (MockPanelHost, Sculpt3dPanelState) {
    set_current_sculpt3d(Some(retrato(ui)));
    let _ = drain_intents();
    (
        MockPanelHost::with_panel::<Sculpt3dPanel>(),
        Sculpt3dPanelState,
    )
}

/// ⭐ **O retrato — uma fixtura, DOIS consumidores.** O gate do fecho troca o
/// verbo no MESMO host, logo ele precisa de montar um segundo retrato; montá-lo
/// à mão seria a segunda resposta à pergunta *«o que a cena entrega a este
/// painel?»*, e ela divergiria no dia do décimo quarto campo.
fn retrato(ui: Sculpt3dUi) -> Sculpt3dSnapshot {
    Sculpt3dSnapshot {
        alpha_image_name: None,
        transform: None,
        filter_armed: false,
        ao_stale: false,
        ui,
        dyntopo: false,
        level: 0,
        level_count: 1,
        pieces: 1,
        isolated: false,
        matcap_keys: &["Clay"],
        verts: 6050,
        alpha_seed: 0.0375,
        model_span: 1.0,
        has_bake_target: true,
    }
}

/// O pincel que deposita cor, pela porta que o pintor pergunta.
fn pintando() -> Sculpt3dUi {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Paint);
    assert!(
        ui.brush.verb.deposita_a_cor_do_pincel(),
        "ANTI-VACUO: a fixtura tem de conter o fenomeno — este verbo nao \
         deposita cor, logo a amostra nem e' desenhada"
    );
    ui
}

/// O centro do rect que o `paint` registou para a amostra.
fn centro_da_amostra(painted: &[(ph2d_a11y::NodeId, Rect)]) -> (f32, f32) {
    let r = painted
        .iter()
        .rev()
        .find(|(id, _)| *id == ids::SCULPT3D_COLOR_SWATCH)
        .map(|(_, r)| *r)
        .expect("a amostra de cor nao foi pintada com o pincel de pintura em maos");
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐⭐ **GATE — o dedo do artista abre o selector, e a cor que ele escolhe
/// chega ao PINCEL.**
///
/// A corrente inteira, pela porta do produto: o `paint` desenha e hit-indexa, o
/// `Down` **real** reconhece a amostra e abre o selector, o espelho do `hero`
/// põe a cor escolhida na `widget_color`, e o `paint` do quadro seguinte lê-a e
/// publica o `SetUi`.
///
/// ⛔ **Nenhum elo pode ser saltado:** montar o `picker_target` à mão deixaria
/// o gate verde sobre uma amostra que o clique nunca alcança — que é
/// precisamente a família de defeitos que esta crate pagou sete vezes.
#[test]
fn o_dedo_abre_o_selector_e_a_cor_escolhida_chega_ao_pincel() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui.clone());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let (cx, cy) = centro_da_amostra(&painted);

    // ⚠️ **O dono dos pixels é ela**, senão o clique abaixo mede outro widget.
    assert_eq!(
        host.hit_at(cx, cy),
        Some(ids::SCULPT3D_COLOR_SWATCH),
        "outra coisa e' dona dos pixels no centro da amostra"
    );
    assert_eq!(
        host.store().picker_target(),
        None,
        "ANTI-VACUO: o selector ja' estava aberto antes do clique"
    );

    let _ = host.click_at(cx, cy);
    assert_eq!(
        host.store().picker_target(),
        Some(ids::SCULPT3D_COLOR_SWATCH),
        "carregar na amostra no centro pintado NAO abriu o selector — ela esta' \
         no indice de hit e o `Down` nao a reconhece como amostra"
    );

    // O espelho do `hero`, que é o que o arnês encena.
    host.pick_colour_in_the_open_picker([10, 200, 30, 255]);
    let _ = drain_intents();
    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let intents = drain_intents();
    let cor = intents
        .iter()
        .find_map(|i| match i {
            Sculpt3dIntent::SetUi(u) => Some(u.brush.color),
            _ => None,
        })
        .expect("a cor escolhida no selector nao produziu um `SetUi`");
    assert_eq!(
        [
            (cor[0] * 255.0 + 0.5) as u8,
            (cor[1] * 255.0 + 0.5) as u8,
            (cor[2] * 255.0 + 0.5) as u8
        ],
        [10, 200, 30],
        "o pincel recebeu {cor:?}, que nao e' a cor escolhida"
    );
}

/// ⭐⭐⭐ **GATE — com o selector FECHADO a amostra semeia, e não escreve.**
///
/// ⛔⛔ **É a metade sem a qual a cor de fábrica muda sozinha no primeiro
/// quadro:** ela não é representável em 8 bits, logo uma amostra que escrevesse
/// de volta todo quadro publicaria um `SetUi` — e um passo de `Ctrl+Z` — sem
/// ninguém tocar em nada.
///
/// ⚠️ **As duas afirmações, e nenhuma basta:** a `widget_color` tem de ficar com
/// a cor do pincel (senão o selector abriria semeado com cinzento), e a fila de
/// intenções tem de ficar **vazia**.
#[test]
fn com_o_selector_fechado_a_amostra_semeia_e_nao_escreve() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui.clone());
    let _ = drain_intents();
    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    let esperada = [
        (ui.brush.color[0] * 255.0 + 0.5) as u8,
        (ui.brush.color[1] * 255.0 + 0.5) as u8,
        (ui.brush.color[2] * 255.0 + 0.5) as u8,
    ];
    assert_eq!(
        host.store().widget_color(ids::SCULPT3D_COLOR_SWATCH),
        Some([esperada[0], esperada[1], esperada[2], 255]),
        "a amostra nao foi semeada com a cor do pincel — o selector abriria \
         noutra cor que nao a que a caixa mostra"
    );
    assert!(
        drain_intents().is_empty(),
        "pintar a amostra com o selector FECHADO publicou uma intencao — um \
         passo de undo por quadro sobre uma cor que ninguem mexeu"
    );
}

/// ⭐⭐⭐ **GATE — o selector SEGUE o sujeito: trocar de pincel fecha-o.**
///
/// ⛔ Sem isto, a janela ficaria a flutuar sobre o canvas a editar uma amostra
/// que ninguém desenha — e a cor que ela escrevesse iria para um pincel que o
/// artista já não tem na mão.
///
/// ⚠️ **O CONTROLO é a primeira metade:** sem ela, um gate que só verificasse
/// o fecho passaria sobre um selector que nunca chegou a abrir.
#[test]
fn trocar_de_pincel_fecha_o_selector() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui.clone());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let (cx, cy) = centro_da_amostra(&painted);
    let _ = host.click_at(cx, cy);
    assert_eq!(
        host.store().picker_target(),
        Some(ids::SCULPT3D_COLOR_SWATCH),
        "CONTROLO: o selector nem chegou a abrir, logo o fecho abaixo nao \
         afirma nada"
    );

    // ⚠️ **Pela porta do produto**, e não escrevendo `ui.brush.verb`: o
    // `switch_verb` é o que o painel corre, e é ele que carrega a memória
    // por-verbo.
    let mut outro = ui;
    ph2d_panel_sculpt3d::state::switch_verb(&mut outro, Verb::Draw);
    assert!(
        !outro.brush.verb.deposita_a_cor_do_pincel(),
        "ANTI-VACUO: este verbo tambem deposita cor, logo a amostra continua \
         a ser pintada e o fecho nao e' exercitado"
    );
    // ⭐⭐ **O MESMO host, e é isso que torna o gate honesto:** o que o artista
    // faz é trocar de pincel com o selector aberto, e o selector vive no store
    // — que é o mesmo objecto. Montar um segundo host obrigaria a repor o alvo
    // à mão, e um gate que semeia o que vai medir prova o que ele próprio pôs.
    set_current_sculpt3d(Some(retrato(outro)));
    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert_eq!(
        host.store().picker_target(),
        None,
        "trocar para um pincel que NAO deposita cor deixou o selector aberto \
         sobre uma amostra que ninguem pinta"
    );
}

/// ⭐⭐⭐ **GATE — com o selector ABERTO a amostra NÃO repõe a cor do pincel.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE** (o `else` do semeamento apagado,
/// ou seja *semear em todo quadro*): os três gates acima ficavam verdes, porque
/// o `SetUi` do quadro do clique é publicado **antes** de a reposição correr —
/// *o defeito só se vê no QUADRO SEGUINTE, e nenhum deles pintava dois*.
///
/// O que o artista veria: a roda do selector move-se e a cor volta atrás, todo
/// quadro. É o par de metades que o doc do pintor descreve e que nada media.
#[test]
fn com_o_selector_aberto_a_amostra_nao_repoe_a_cor_do_pincel() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui.clone());
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let (cx, cy) = centro_da_amostra(&painted);
    let _ = host.click_at(cx, cy);
    host.pick_colour_in_the_open_picker([10, 200, 30, 255]);

    // ⚠️ **O CONTROLO: a cor escolhida NÃO é a do pincel**, senão a asserção
    // abaixo é trivialmente verdadeira.
    let fabrica = [
        (ui.brush.color[0] * 255.0 + 0.5) as u8,
        (ui.brush.color[1] * 255.0 + 0.5) as u8,
        (ui.brush.color[2] * 255.0 + 0.5) as u8,
    ];
    assert_ne!(
        fabrica,
        [10, 200, 30],
        "ANTI-VACUO: a cor escolhida e' a do pincel, logo repor ou nao repor \
         dao o mesmo resultado"
    );

    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert_eq!(
        host.store().widget_color(ids::SCULPT3D_COLOR_SWATCH),
        Some([10, 200, 30, 255]),
        "a amostra REPOS a cor do pincel por cima da que o selector escreveu — \
         a roda mexe-se e a cor volta atras, todo quadro"
    );
}

/// ⭐⭐⭐ **GATE — a cor já aplicada NÃO volta a ser publicada.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE** (a guarda do *«mudou?»* trocada
/// por `true`): com o selector aberto, a amostra publicaria um `SetUi` **por
/// quadro** — um passo de `Ctrl+Z` por quadro sobre uma cor que ninguém mexeu.
///
/// ⚠️ **O gate tem de encenar o CICLO do produto**, e é isso que o torna
/// honesto: publicar não muda o retrato — quem o actualiza é a shell, ao drenar
/// a intenção. Sem esse passo, o quadro seguinte compara a cor nova com a
/// **velha** e publica outra vez, *que é o comportamento certo*.
#[test]
fn a_cor_ja_aplicada_nao_volta_a_ser_publicada() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let (cx, cy) = centro_da_amostra(&painted);
    let _ = host.click_at(cx, cy);
    host.pick_colour_in_the_open_picker([10, 200, 30, 255]);
    let _ = drain_intents();
    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);

    // A shell drena e aplica — é o elo que fecha o ciclo.
    let aplicado = drain_intents()
        .into_iter()
        .find_map(|i| match i {
            Sculpt3dIntent::SetUi(u) => Some(u),
            _ => None,
        })
        .expect("ANTI-VACUO: o 1.o quadro nao publicou nada, logo nao ha ciclo");
    set_current_sculpt3d(Some(retrato(aplicado)));

    let _ = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        drain_intents().is_empty(),
        "com a cor JA' aplicada e o selector ainda aberto, a amostra publicou \
         outra intencao — um passo de undo por quadro sobre uma cor parada"
    );
}

/// ⭐⭐⭐ **GATE — um pincel que puxa a cor do ANEL não mostra a caixa.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE** (a recusa do pintor apagada): os
/// três primeiros gates armam sempre o pincel de pintura, e *uma fixtura que
/// arma um só verbo não pode ver um controlo oferecido a TODOS*.
///
/// ⚠️ **É a espécie que o dono reporta como «não vejo efeito»:** o `Blur` e o
/// `Smear Color` tiram a cor da vizinhança e não olham para este campo — uma
/// caixa ali seria um controlo que o artista mexe e que não muda nada.
#[test]
fn um_pincel_que_puxa_a_cor_do_anel_nao_mostra_a_caixa() {
    let mut ui = Sculpt3dUi::default();
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Blur);
    assert!(
        !ui.brush.verb.deposita_a_cor_do_pincel(),
        "ANTI-VACUO: este verbo deposita a cor do pincel, logo a caixa e' \
         oferecida com razao e o gate nao afirma nada"
    );
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    assert!(
        !painted
            .iter()
            .any(|(id, _)| *id == ids::SCULPT3D_COLOR_SWATCH),
        "a caixa de cor e' pintada com um pincel que NAO a le' — o artista \
         escolhe uma cor e o barro sai com outra"
    );
}

/// ⚠️ **SONDA — onde a caixa cai no encaixe, e o que a troca comprou.**
///
/// O encaixe mede `~880 px` e o §94 desta linha mediu que a secção do pincel
/// sozinha já ocupa `657` deles: *cada fileira aqui é paga por todos os
/// pincéis*. Esta sonda imprime a posição da caixa e o que a troca de três
/// pistas por uma amostra devolveu ao orçamento.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_onde_a_caixa_cai_no_encaixe() {
    let ui = pintando();
    let (mut host, mut state) = arrange(ui);
    let painted = host.paint::<Sculpt3dPanel>(&mut state, VIEWPORT);
    let r = painted
        .iter()
        .rev()
        .find(|(id, _)| *id == ids::SCULPT3D_COLOR_SWATCH)
        .map(|(_, r)| *r)
        .expect("a caixa nao foi pintada");
    let passo = ph2d_tokens::ROW_H_PX + ph2d_tokens::control_gap_px();
    eprintln!(
        "[cor] a caixa cai em y = {:.1}  (dobra do encaixe: 880)",
        r.y
    );
    eprintln!("[cor] altura de uma fileira: {passo:.1} px");
    eprintln!(
        "[cor] tres pistas viraram UMA amostra => a seccao do pincel encolheu \
         {:.1} px com o pincel de pintura em maos",
        2.0 * passo
    );
}
