//! Gates da **CANETA da seleção** ([`super::selection_pen`]).
//!
//! O invariante que os organiza: **a peça que a caneta entrega é a peça que o Convert to Curve já
//! entregava** — uma `Freehand` FECHADA e editável ponto-a-ponto. Tudo o mais (a rasterização, o
//! booleano, o Edit mode, o undo) é consequência disso, e é por isso que nenhum gate aqui re-testa
//! aqueles sistemas.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca em modo Selection com a caneta em mãos.
fn pen_tool(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(selection_pen::SELECTION_MODE_PEN);
    t
}

/// Um clique de caneta SEM arrasto (uma quina).
fn click(t: &mut PainterTool, p: [f32; 2]) {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
}

/// Um clique COM arrasto — o gesto que puxa as tangentes (`to` é onde a mão parou).
fn click_drag(t: &mut PainterTool, p: [f32; 2], to: [f32; 2]) {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(to, PointerPhase::Move));
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
}

fn selected(t: &PainterTool, x: u32, y: u32) -> bool {
    t.selection_coverage_at(x, y) >= 128
}

fn selected_count(t: &PainterTool, size: u32) -> usize {
    (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .filter(|&(x, y)| selected(t, x, y))
        .count()
}

/// **A caneta entrega uma CURVA, e é isso que a separa do laço.** Um laço nasce polilinha e só vira
/// editável ponto-a-ponto depois do Convert; a caneta nasce editável, porque o artista acabou de pôr
/// cada tangente com a mão.
///
/// **Mutação que sangra:** o commit empurrar `CurveModel::raw_lasso(points, true)` — a forma fica igual
/// e `is_curve()` cai, então o Edit mode abre a CAIXA de transformação em vez do editor de pontos.
#[test]
fn the_pen_delivers_an_editable_curve_not_a_raw_polyline() {
    let mut t = pen_tool(64);
    click(&mut t, [16.0, 16.0]);
    click(&mut t, [48.0, 16.0]);
    click(&mut t, [48.0, 48.0]);
    click(&mut t, [16.0, 48.0]);
    // Fechar: o clique volta ao PRIMEIRO ponto.
    click(&mut t, [16.0, 16.0]);
    assert!(!t.selection_pen_live(), "a sessao terminou ao fechar");
    let shapes = t.selection_shapes_snapshot();
    assert_eq!(shapes.len(), 1, "uma forma entregue");
    let selection_shapes::SelectionShape::Freehand { model, .. } = &shapes[0].shape else {
        panic!("a caneta entrega uma Freehand");
    };
    assert!(model.closed, "uma regiao de selecao e FECHADA");
    assert!(
        model.is_curve(),
        "e ela e uma CURVA (handles paralelos), logo editavel ponto-a-ponto"
    );
    assert_eq!(model.points.len(), 4, "quatro quinas, sem ancora duplicada");
    assert!(selected(&t, 32, 32), "o miolo do quadrado esta selecionado");
    assert!(!selected(&t, 4, 4), "e o lado de fora nao");
}

/// **Arrastar CURVA o caminho.** É a metade da caneta que um polígono não tem: as mesmas quatro
/// posições de clique, com tangentes puxadas para fora, delimitam MAIS área.
///
/// **Mutação que sangra:** o `selection_pen_drag` não escrever os handles (o arrasto vira clique) — as
/// duas áreas passam a ser iguais.
#[test]
fn dragging_an_anchor_pulls_a_real_tangent() {
    let corners = [[16.0, 16.0], [48.0, 16.0], [48.0, 48.0], [16.0, 48.0]];
    let mut sharp = pen_tool(64);
    for c in corners {
        click(&mut sharp, c);
    }
    click(&mut sharp, corners[0]);

    let mut curved = pen_tool(64);
    // Cada âncora sai com a tangente puxada no sentido do percurso, para as arestas arquearem PARA FORA.
    let pulls = [[16.0, 6.0], [58.0, 16.0], [48.0, 58.0], [6.0, 48.0]];
    for (c, to) in corners.iter().zip(pulls) {
        click_drag(&mut curved, *c, to);
    }
    click(&mut curved, corners[0]);

    let (a, b) = (selected_count(&sharp, 64), selected_count(&curved, 64));
    assert!(
        b > a + 64,
        "as tangentes puxadas tem de arquear as arestas: quinas {a} px, curva {b} px"
    );
}

/// **Menos de três âncoras não é região, e não vira passo de undo.** Um Enter sobre um caminho de dois
/// pontos descarta — commitá-lo gravaria um passo cujo efeito é zero.
#[test]
fn a_path_with_fewer_than_three_anchors_is_discarded_not_committed() {
    let mut t = pen_tool(64);
    click(&mut t, [16.0, 16.0]);
    click(&mut t, [48.0, 16.0]);
    assert!(t.selection_pen_commit(), "havia sessao a terminar");
    assert!(!t.selection_pen_live());
    assert!(
        t.selection_shapes_snapshot().is_empty(),
        "nada foi entregue"
    );
    assert!(!t.selection_active(), "e nenhuma selecao nasceu");
}

/// **Esc devolve a seleção ao que era, sem gastar undo.** A sessão nunca foi commitada, então um
/// "cancelar via undo" apontaria o Ctrl+Z do artista para o gesto errado.
///
/// **Mutação que sangra:** o `selection_pen_cancel` não restaurar o `stroke_undo` — o preview da caneta
/// fica na máscara depois do Esc.
#[test]
fn esc_gives_the_selection_back_and_spends_no_undo() {
    let mut t = pen_tool(64);
    // Uma seleção PRÉVIA, para o Esc ter ao que voltar.
    t.set_rect_selection(4, 4, 8, 8);
    let before = selected_count(&t, 64);
    assert!(before > 0);

    click(&mut t, [16.0, 16.0]);
    click(&mut t, [48.0, 16.0]);
    click(&mut t, [48.0, 48.0]);
    assert!(t.selection_pen_live());
    assert!(t.selection_pen_cancel(), "havia caneta a descartar");

    assert_eq!(
        selected_count(&t, 64),
        before,
        "Esc devolve exatamente a selecao anterior"
    );
    assert!(
        t.selection_shapes_snapshot().len() == 1,
        "e a forma previa continua sendo a unica da lista"
    );
}

/// **A base booleana é da SESSÃO, não do clique.** Com `Add`, cada âncora nova compõe contra a seleção
/// que existia ANTES da caneta — senão o preview do clique anterior viraria base do seguinte e o `Add`
/// acumularia em cima de si mesmo.
///
/// O oráculo é o que sobra FORA da união: um retângulo prévio disjunto tem de sobreviver inteiro, e a
/// região da caneta tem de ser a do caminho FINAL, não a união dos previews intermediários.
///
/// **Mutação que sangra:** re-capturar `selection_base` em cada Down — os previews intermediários (o
/// triângulo dos três primeiros cliques) ficam somados ao quadrado final.
#[test]
fn the_boolean_base_is_the_session_not_the_click() {
    let mut t = pen_tool(64);
    t.set_rect_selection(2, 2, 6, 6); // um selo distante, que o Add tem de preservar
    t.set_selection_bool_op(1); // Add

    // ⚠️ A fixture tem de conter o fenômeno: o preview INTERMEDIÁRIO precisa cobrir pixels que a forma
    // FINAL não cobre — senão acumular contra ele é indistinguível de compor contra a sessão. Três
    // cliques dão um triângulo grande; o quarto fecha em GRAVATA-BORBOLETA, cujo preenchimento são dois
    // triângulos que encostam no cruzamento e deixam de fora a faixa lateral.
    click(&mut t, [20.0, 20.0]);
    click(&mut t, [60.0, 20.0]);
    click(&mut t, [20.0, 60.0]); // preview: o triângulo, que cobre (25, 40)
    click(&mut t, [60.0, 60.0]); // a gravata: (25, 40) fica de fora
    click(&mut t, [20.0, 20.0]); // fecha

    assert!(selected(&t, 4, 4), "o selo previo sobrevive ao Add");
    assert!(
        !selected(&t, 25, 40),
        "e o triangulo INTERMEDIARIO nao ficou somado a mascara"
    );
}

/// **Sair do modo Pen com uma caneta em voo a descarta** — um caminho aberto num modo que não o desenha
/// nem o termina é trabalho perdido em silêncio.
#[test]
fn leaving_pen_mode_discards_the_live_path() {
    let mut t = pen_tool(64);
    click(&mut t, [16.0, 16.0]);
    click(&mut t, [48.0, 16.0]);
    assert!(t.selection_pen_live());
    t.set_selection_mode(1); // Freehand
    assert!(!t.selection_pen_live(), "a caneta morreu ao trocar de modo");
    assert!(t.selection_shapes_snapshot().is_empty());
}

/// **Uma porta só desenha a Bézier autorada.** Enquanto a caneta vive, é ela que o overlay publica — e o
/// caminho aberto aparece desde o PRIMEIRO clique (senão o artista clica e não vê nada).
///
/// **Mutação que sangra:** a shell voltar a ler `curve_overlay()` — a caneta fica invisível.
#[test]
fn the_authoring_overlay_shows_the_pen_from_the_first_click() {
    let mut t = pen_tool(64);
    assert!(
        t.authoring_curve_overlay().is_none(),
        "sem caneta e sem figura, nada a desenhar"
    );
    click(&mut t, [16.0, 16.0]);
    let o = t.authoring_curve_overlay().expect("a caneta e publicada");
    assert_eq!(o.points.len(), 1, "a ancora ja aparece");
    assert!(
        o.transform_gizmo.is_none(),
        "um caminho em construcao nao carrega caixa de transformacao"
    );
    click(&mut t, [48.0, 16.0]);
    let o = t.authoring_curve_overlay().expect("segue publicada");
    assert!(
        o.spine.len() >= 2,
        "e o caminho entre as ancoras e desenhado"
    );
}

/// **A linha-fantasma segue a mão com o botão solto** — é ela que mostra onde a próxima âncora cai.
///
/// **Mutação que sangra:** tirar o `selection_pen_hover` do `on_canvas_hover` — o spine para de crescer
/// entre cliques.
/// ⚠️ O oráculo é **onde a perna TERMINA**, não quantos pontos o spine tem: a sessão nasce com o hover
/// no ponto do clique, então a perna existe desde o começo — ela só é DEGENERADA (comprimento zero) até
/// a mão andar. Medir o comprimento da lista mediria a existência da perna, que nunca falha.
#[test]
fn the_rubber_band_follows_the_hovering_hand() {
    let mut t = pen_tool(64);
    click(&mut t, [16.0, 16.0]);
    assert_eq!(
        t.authoring_curve_overlay().expect("publicada").spine.last(),
        Some(&[16.0, 16.0]),
        "recem-clicada, a perna e degenerada: ela termina na propria ancora"
    );
    t.on_canvas_hover([48.0, 40.0]);
    assert_eq!(
        t.authoring_curve_overlay().expect("publicada").spine.last(),
        Some(&[48.0, 40.0]),
        "e depois do hover ela termina onde a mao esta"
    );
}

/// **Fechar torna os pontos EDITÁVEIS** (Enio, 2026-08-07) — e o oráculo é o que a shell DESENHA, não o
/// flag: depois do clique no primeiro ponto tem de existir um editor por-âncora, com uma âncora para
/// cada ponto autorado.
///
/// ⚠️ A metade que este gate NÃO afirma sozinho é que aquelas âncoras respondem ao mouse — pintado e
/// morto sob o mouse é a doença que este repo nomeia —, e é por isso que existe o gate seguinte.
///
/// **Mutação que sangra:** tirar o `selection_edit_mode = true` do commit — `selection_gizmos()` volta
/// vazia e a curva fica invisível.
#[test]
fn closing_the_path_lights_the_gizmos_so_every_point_is_editable() {
    let mut t = pen_tool(256);
    for p in [[30.0, 30.0], [100.0, 30.0], [100.0, 100.0], [30.0, 100.0]] {
        click(&mut t, p);
    }
    click(&mut t, [30.0, 30.0]); // fecha no primeiro ponto
    assert!(!t.selection_pen_live(), "a sessao fechou");
    assert!(
        t.selection_gizmos_visible(),
        "e fechar acendeu os gizmos — senao os pontos existem e ninguem os ve"
    );
    let views = t.selection_gizmos();
    assert_eq!(views.len(), 1, "uma forma, um gizmo");
    let edit = views[0]
        .edit_curve
        .as_ref()
        .expect("o editor POR-ANCORA, nao a caixa de transformacao");
    assert_eq!(
        edit.anchors.len(),
        4,
        "uma ancora para cada ponto que a mao colocou"
    );
}

/// **E as âncoras respondem ao mouse.** Com os gizmos acesos e a caneta AINDA em mãos, um Down sobre uma
/// âncora a arrasta — ele não abre um caminho novo. É a primeira recusa dos gizmos, e é o que separa
/// *"os pontos aparecem"* de *"os pontos são editáveis"*.
///
/// **Mutação que sangra:** a porta rotear todo Down para a caneta em modo Pen — a âncora não se move e
/// uma sessão nova nasce sobre a curva fechada.
#[test]
fn an_anchor_of_the_closed_curve_moves_under_the_hand() {
    let mut t = pen_tool(256);
    for p in [[30.0, 30.0], [100.0, 30.0], [100.0, 100.0], [30.0, 100.0]] {
        click(&mut t, p);
    }
    click(&mut t, [30.0, 30.0]);
    // Down EM CIMA da âncora (100, 30) e arrasta 40 px para a direita.
    t.on_canvas_pointer(cp([100.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([140.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([140.0, 30.0], PointerPhase::Up));
    assert!(
        !t.selection_pen_live(),
        "pegar uma ancora NAO abre um caminho novo"
    );
    let views = t.selection_gizmos();
    let edit = views[0].edit_curve.as_ref().expect("o editor de pontos");
    let moved = edit
        .anchors
        .iter()
        .any(|a| (a[0] - 140.0).abs() < 1.0 && (a[1] - 30.0).abs() < 1.0);
    assert!(moved, "a ancora seguiu a mao: {:?}", edit.anchors);
}

/// **E a caneta continua sendo uma caneta.** Um Down em espaço vazio — que os gizmos não quiseram —
/// começa o caminho seguinte; sem esta queda-livre ela seria uma ferramenta de UM só uso, porque o
/// próprio fechamento acende os gizmos que passariam a comer todo clique.
///
/// **Mutação que sangra:** tirar a queda-livre — `selection_pen_live()` fica falso e o clique some.
#[test]
fn a_click_in_empty_space_starts_the_next_path() {
    let mut t = pen_tool(256);
    for p in [[30.0, 30.0], [100.0, 30.0], [100.0, 100.0], [30.0, 100.0]] {
        click(&mut t, p);
    }
    click(&mut t, [30.0, 30.0]);
    assert!(t.selection_gizmos_visible(), "os gizmos estao acesos");
    click(&mut t, [200.0, 200.0]); // longe de toda ancora, alca e linha
    assert!(
        t.selection_pen_live(),
        "o clique em espaco vazio abriu o caminho seguinte"
    );
    assert_eq!(t.selection_pen_points(), 1, "com a primeira ancora nele");
}

// ── O Ctrl+Z de uma caneta em voo ────────────────────────────────────────────────────────────────────

/// Uma seleção JÁ PRONTA, feita com a caneta, e uma segunda em voo com três âncoras — a cena exata do
/// report. O que o gate mede é de qual das duas o Ctrl+Z fala.
fn one_committed_and_one_in_flight() -> PainterTool {
    let mut t = pen_tool(256);
    for p in [[30.0, 30.0], [100.0, 30.0], [100.0, 100.0], [30.0, 100.0]] {
        click(&mut t, p);
    }
    click(&mut t, [30.0, 30.0]); // fecha a PRIMEIRA
    assert!(!t.selection_pen_live(), "a primeira fechou");
    assert!(selected(&t, 60, 60), "a primeira selecao existe");
    for p in [[150.0, 150.0], [200.0, 150.0], [200.0, 200.0]] {
        click(&mut t, p);
    }
    assert_eq!(t.selection_pen_points(), 3, "a segunda esta em voo");
    t
}

/// **O Ctrl+Z fala do caminho EM VOO, não da seleção que já estava pronta** (Enio, 2026-08-07).
///
/// As âncoras em voo não estão no `ProjectState` — a sessão inteira é UM passo, gravado só no fechamento
/// —, então sem um dono o atalho caía no passo estrutural anterior e apagava trabalho terminado enquanto
/// o artista desenhava. É a mesma doença que o Colorize do Flip pagou com os rabiscos transientes.
///
/// **Mutação que sangra:** tirar o `selection_pen_undo` do topo do `undo_last` — a primeira seleção some
/// e o caminho em voo fica com as três âncoras.
#[test]
fn undo_takes_the_last_anchor_not_the_selection_already_finished() {
    let mut t = one_committed_and_one_in_flight();
    assert!(t.undo_last(), "o undo foi consumido");
    assert_eq!(
        t.selection_pen_points(),
        2,
        "o Ctrl+Z tinha de tirar a ULTIMA ANCORA do caminho em voo"
    );
    assert!(
        selected(&t, 60, 60),
        "e a selecao que ja estava PRONTA continua la"
    );
}

/// **Desfazer até a última âncora encerra a sessão, e a seleção volta ao que era** — sem gastar passo de
/// undo, porque nada foi commitado.
///
/// **Mutação que sangra:** não encerrar a sessão com o caminho vazio — `selection_pen_live` fica verdadeiro
/// sobre um caminho de zero pontos.
#[test]
fn undoing_every_anchor_ends_the_session_and_costs_no_undo_step() {
    let mut t = one_committed_and_one_in_flight();
    for _ in 0..3 {
        assert!(t.undo_last(), "a caneta consumiu");
    }
    assert!(
        !t.selection_pen_live(),
        "a sessao morreu com o ultimo ponto"
    );
    assert!(
        selected(&t, 60, 60),
        "e a primeira selecao sobreviveu inteira"
    );
    // Agora sim: o Ctrl+Z seguinte é o estrutural, e desfaz a PRIMEIRA.
    t.undo_last();
    assert!(
        !selected(&t, 60, 60),
        "com a caneta fora, o Ctrl+Z volta a ser o estrutural"
    );
}

/// **Desfazer abaixo de três âncoras tira a REGIÃO da tela.**
///
/// ⚠️ O preview retornava cedo com menos de três pontos, então desfazer da terceira para a segunda
/// deixava desenhada a região que as TRÊS delimitavam: a máscara descrevendo um caminho que não existe
/// mais. Agora ele aplica a região vazia — o preview é função pura do caminho, que é a propriedade de
/// que o Ctrl+Z depende.
///
/// **Mutação que sangra:** o `return` cedo de volta ao `selection_pen_preview`.
#[test]
fn undoing_below_three_anchors_takes_the_region_off_the_screen() {
    let mut t = pen_tool(256);
    for p in [[150.0, 150.0], [220.0, 150.0], [220.0, 220.0]] {
        click(&mut t, p);
    }
    assert!(selected(&t, 200, 190), "o triangulo em voo esta na tela");
    assert!(t.undo_last(), "a caneta consumiu");
    assert!(
        !selected(&t, 200, 190),
        "a regiao das TRES ancoras continua desenhada depois de tirar uma"
    );
}

/// **Ctrl+Shift+Z devolve a âncora** — e, com a pilha vazia, é ENGOLIDO em vez de refazer um passo
/// estrutural sob um caminho em voo (o irmão exato do float do Deform).
///
/// **Mutação que sangra:** `selection_pen_redo` devolvendo `false` com a pilha vazia — o redo escapa para
/// o estrutural e reinstala a seleção que o teste acabou de desfazer.
#[test]
fn redo_gives_the_anchor_back_and_is_swallowed_when_there_is_none() {
    let mut t = one_committed_and_one_in_flight();
    t.undo_last();
    assert_eq!(t.selection_pen_points(), 2);
    assert!(t.redo_last(), "o redo foi consumido");
    assert_eq!(t.selection_pen_points(), 3, "a ancora voltou");
    // A pilha está vazia agora: o redo é ENGOLIDO.
    assert!(t.redo_last(), "consumido mesmo sem nada a devolver");
    assert_eq!(t.selection_pen_points(), 3);
    // ⚠️ O oráculo NÃO é *"a primeira seleção está na tela"* — sob `New` um caminho de três âncoras
    // SUBSTITUI a máscara, então ela corretamente não está. O que se afirma é que a fila ESTRUTURAL não
    // foi mexida: Esc joga a sessão fora e a primeira volta inteira. Foi este assert que reprovou código
    // correto na 1ª versão do gate.
    assert!(t.selection_pen_cancel(), "havia sessao para descartar");
    assert!(
        selected(&t, 60, 60),
        "a primeira selecao nao voltou — o redo mexeu na fila estrutural"
    );
}

/// **Uma âncora nova encerra o futuro** — a regra universal de um redo, e sem ela o Ctrl+Shift+Z
/// ressuscitaria um ponto que já não pertence ao caminho.
///
/// **Mutação que sangra:** tirar o `popped.clear()` do ramo que empurra a âncora.
#[test]
fn placing_an_anchor_drops_what_the_undo_had_kept() {
    let mut t = one_committed_and_one_in_flight();
    t.undo_last();
    assert_eq!(t.selection_pen_points(), 2);
    click(&mut t, [140.0, 210.0]); // uma âncora NOVA
    assert_eq!(t.selection_pen_points(), 3);
    assert!(t.redo_last(), "consumido");
    assert_eq!(
        t.selection_pen_points(),
        3,
        "o redo ressuscitou a ancora que a nova substituiu"
    );
}
