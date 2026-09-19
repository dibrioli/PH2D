//! ⭐⭐⭐ **A ROW DE PARAM CHEGA A PIXEL** — o gate que faltava, e que o smoke do Enio cobrou.
//!
//! ⛔⛔ **Os seis gates de geometria estavam VERDES sobre um ecrã em branco.** Eles mediam
//! `card_h` e `param_row_rect` — a **faixa RESERVADA** —, e a faixa continua a ser reservada
//! quando ninguém a pinta. Foi exactamente o achado §4.2 da auditoria do `source.lsystem`, uma
//! wave depois: *«o gate que prometia medir a queixa chega a PIXEL media a linha reservada;
//! apagar a pintura deixava-o verde»*. Aqui o oráculo é a CENA do Vello:
//! **glifos** (o texto) e **segmentos de caminho** (a barra), contados depois de pintar.
//!
//! ⚠️ **E o gate mede os DOIS zooms**, porque a lei tem dois lados: a barra pinta-se sempre,
//! o texto só acima do limiar de legibilidade. Um gate a `zoom = 1` teria passado sobre o
//! ecrã em branco que o Enio fotografou (a cena abre a `zoom ≈ 0,5`).

use crate::MotionGraphPanel;
use crate::snapshot::{
    CardParam, GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, set_current_motion_graph,
};
use crate::state::{MotionGraphPanelState, ViewState};
use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint, ParamWidget};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const W: f32 = 1200.0; // LITERAL-PX-OK: tela de teste
const H: f32 = 800.0; // LITERAL-PX-OK: tela de teste

fn node(k: usize) -> GraphNodeView {
    GraphNodeView {
        kind: NodeViewKind::Node,
        id: 1,
        display_name: "Grid".into(),
        category: NodeUiCategory::Source,
        silhouette: NodeSilhouette::Rect,
        x: 60.0,
        y: 60.0,
        inputs: vec![PortView {
            name: "in",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        }],
        outputs: vec![PortView {
            name: "out",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        }],
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: (0..k)
            .map(|i| {
                CardParam::from_hint(
                    ParamUiHint {
                        param: "p",
                        label: ["Rows", "Columns", "Gap X", "Gap Y"][i % 4],
                        min: 0.0,
                        max: 10.0,
                        step: 0.1,
                        widget: ParamWidget::Slider,
                    },
                    4.0,
                )
            })
            .collect(),
        sections: Vec::new(),
    }
}

/// `(glifos, segmentos)` de uma pintura com `k` params ao zoom `z`.
fn painted(k: usize, z: f32) -> (u32, u32) {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![node(k)],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let viewport = Rect::new(0.0, 0.0, W, H);
    let mut layout = HeroLayout::for_viewport(viewport);
    // Sem isto o painel do split recebe área ZERO e o gate fica vácuo — ver o doc do arnês.
    layout.motion_graph = viewport;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState {
        view: ViewState {
            zoom: z,
            ..ViewState::default()
        },
        // ⚠️ Sem isto o painel ENQUADRA o grafo na primeira pintura e o zoom que o teste pediu
        // é deitado fora — o gate mediria o auto-fit, não o LOD.
        fitted: true,
        ..MotionGraphPanelState::default()
    };
    let out =
        host.paint_and_count_geometry_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);
    set_current_motion_graph(None);
    out
}

/// **A BARRA CHEGA A PIXEL EM QUALQUER ZOOM** — inclusive no `0,5` com que a cena de smoke
/// abre, que é onde o ecrã ficou em branco. FALSIFICADO por voltar a saltar a row inteira
/// abaixo do limiar: os segmentos a `0,5` empatam com os de um cartão sem params.
#[test]
fn the_bar_of_a_param_row_reaches_pixel_at_every_zoom() {
    for z in [0.5_f32, 1.0, 2.0] {
        let (_, sem) = painted(0, z);
        let (_, com) = painted(4, z);
        assert!(
            com > sem,
            "zoom {z}: 4 params tem de emitir MAIS geometria que nenhum ({com} contra {sem})"
        );
    }
}

/// **O TEXTO SEGUE O ZOOM** — glifos a mais quando se lê, nenhum quando não se lê.
/// FALSIFICADO por `param_text_is_drawn` virar constante: `true` paga texto ilegível num grafo
/// afastado, `false` deixa o cartão sem números em qualquer zoom.
#[test]
fn the_text_of_a_param_row_appears_only_when_it_is_legible() {
    let (g_sem_longe, _) = painted(0, 0.5);
    let (g_com_longe, _) = painted(4, 0.5);
    assert_eq!(
        g_com_longe, g_sem_longe,
        "a 0,5 o rotulo seria uma mancha — nenhum glifo a mais"
    );
    let (g_sem_perto, _) = painted(0, 1.0);
    let (g_com_perto, _) = painted(4, 1.0);
    assert!(
        g_com_perto > g_sem_perto,
        "a 1,0 as quatro rows escrevem rotulo e valor ({g_com_perto} contra {g_sem_perto})"
    );
}

/// ⭐⭐ **UMA ROW DE TEXTO MOSTRA O QUE ESTÁ ESCOLHIDO** — e cai no selo quando não há nada.
///
/// ⚠️ **Antes disto o cartão era MUDO sobre estes controlos:** o clique já mudava o valor (a
/// forma publicada anda, o ficheiro abre o diálogo) e a row continuava a desenhar o mesmo selo.
/// *Mudar uma coisa que não se consegue ler é meio controlo*, e a `§0.6` da casa chama a isso
/// pior que não começado.
///
/// FALSIFICADO por o braço devolver `Shown::Editor` mesmo com texto — a row volta a ser muda.
#[test]
fn a_text_row_shows_what_is_chosen_and_falls_back_to_the_badge() {
    let com = |t: &str| {
        let mut p = crate::CardParam::from_hint(
            ParamUiHint {
                param: "path",
                label: "Shape",
                min: 0.0,
                max: 0.0,
                step: 0.0,
                widget: ParamWidget::Source,
            },
            0.0,
        );
        p.text = crate::RowText::new(t);
        p
    };
    assert!(
        !crate::paint::paint_card_params::shows_a_level(&com("Estrela")),
        "um nome nao e' um nivel: ele nao se arrasta"
    );
    assert_eq!(com("Estrela").text.as_str(), "Estrela");
    assert!(
        com("").text.is_empty(),
        "sem valor, a row nao tem texto e o pintor desenha o selo"
    );
}

/// ⚠️ **O TEXTO CORTA-SE POR CARÁCTER, NUNCA POR BYTE** — um nome de ficheiro acentuado é o
/// caso normal, e cortar um `&str` a meio de um carácter multibyte é um `panic` no melhor caso.
///
/// FALSIFICADO por truncar em `s.len().min(CAP)` sem recuar até à fronteira.
#[test]
fn the_row_text_truncates_on_a_character_never_inside_one() {
    // 30 caracteres de 2 bytes = 60 bytes: o corte cai bem dentro da cadeia.
    let acentuado = "ç".repeat(30);
    let t = crate::RowText::new(&acentuado);
    assert!(!t.as_str().is_empty(), "alguma coisa tem de sobrar");
    assert!(
        t.as_str().chars().all(|c| c == 'ç'),
        "o corte partiu um caracter: {:?}",
        t.as_str()
    );
    assert!(
        acentuado.starts_with(t.as_str()),
        "o que fica e' um PREFIXO do que veio"
    );
    // E o caso curto passa inteiro.
    assert_eq!(crate::RowText::new("dados.csv").as_str(), "dados.csv");
}

/// A cor de teste e o id que a shell atribuiria à amostra.
const SWATCH_RGBA: [u8; 4] = [200, 40, 60, 255];
const SWATCH_ID: u64 = 0xC010_1234;

fn swatch_node() -> GraphNodeView {
    let mut n = node(1);
    let mut p = CardParam::from_hint(
        ParamUiHint {
            param: "r",
            label: "Colour",
            min: 0.0,
            max: 1.0,
            step: 0.01,
            widget: ParamWidget::Color {
                channels: ["r", "g", "b", "a"],
            },
        },
        0.0,
    );
    p.swatch = Some(SWATCH_RGBA);
    p.swatch_id = Some(SWATCH_ID);
    n.params = vec![p];
    n
}

/// ⭐⭐⭐ **CLICAR NA AMOSTRA DE UM CARTÃO ABRE O SELECTOR** — pelo despachante REAL.
///
/// ⛔⛔ **É o gate que faltava, e o smoke do Enio cobrou-o**: *«o color picker não abre ao
/// clicar na amostra de cor dentro do nó no grafo»*. Os dois gates que eu tinha mediam a **lei**
/// — que o id carrega o nó, e que a shell resolve o nó a partir do id — e **nenhum dos dois está
/// na estrada que o clique percorre**. A lei estava certa o tempo todo.
///
/// **O mecanismo:** tudo o que o `push_param_row_hits` produz é registado como
/// [`ph2d_editor_core::interaction::InteractiveState::GraphSurface`], e o `pointer_down` do
/// `editor-core` **captura toda superfície de grafo e RETORNA** — ~200 linhas antes do ramo que
/// abre o selector. A amostra estava marcada, com o id certo e a cor semeada, e o clique nunca
/// chegava ao sítio onde isso é lido.
///
/// ⚠️ **Por isso este gate tem de pintar e DESPACHAR**, nunca chamar o gesto do painel: um teste
/// que empurra o gesto já assumiu a resposta (é o que o doc do `dispatch_pointer_event` avisa).
///
/// FALSIFICADO por voltar a registar a amostra em `push_param_row_hits`: a captura do grafo
/// engole o Down e `picker_target()` fica `None`.
#[test]
fn clicking_a_card_swatch_opens_the_shared_colour_picker() {
    use ph2d_editor_core::NodeId;
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![swatch_node()],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let viewport = Rect::new(0.0, 0.0, W, H);
    let mut layout = HeroLayout::for_viewport(viewport);
    layout.motion_graph = viewport;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState {
        view: ViewState {
            zoom: 1.0,
            ..ViewState::default()
        },
        fitted: true,
        ..MotionGraphPanelState::default()
    };
    let _ =
        host.paint_and_count_geometry_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);

    // O centro da row da cor — a MESMA conta que a pintura usou.
    let view = crate::geom::View::new(viewport, state.view);
    let r = crate::geom::param_row_rect(&swatch_node(), &view, 0);
    host.dispatch_pointer_event(ph2d_host::PointerEvent {
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        pressure: 1.0,
        kind: ph2d_host::PointerKind::Down,
        source: ph2d_host::PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 1,
    });
    set_current_motion_graph(None);

    assert_eq!(
        host.store().picker_target(),
        Some(NodeId(SWATCH_ID)),
        "o Down na amostra tem de ABRIR o selector — se ficou `None`, a captura da superficie \
         de grafo engoliu o clique outra vez"
    );
    // ⚠️ E a SEMENTE: sem ela o selector abre no cinzento de omissao (`0x888888`), e o primeiro
    // toque escreve esse cinzento no no'.
    assert_eq!(
        host.store().widget_color(NodeId(SWATCH_ID)),
        Some(SWATCH_RGBA),
        "o selector tem de abrir NA COR da amostra"
    );
}

// ---------------------------------------------------------------------------------------------
// ⛔⛔ O RÓTULO CABE NA COLUNA EM QUE ELE É PINTADO — ou é cortado sem ninguém saber
// ---------------------------------------------------------------------------------------------

/// Um cartão com UMA row, o rótulo e o valor dados — a fixtura da pergunta *«isto cabe?»*.
fn one_row(label: &'static str, value: f32) -> GraphNodeView {
    let mut n = node(0);
    n.params = vec![CardParam::from_hint(
        ParamUiHint {
            param: "p",
            label,
            // A faixa dos dois warps: ±10 unidades de mundo, que é onde o valor fica mais
            // largo (`-10.00`) e portanto onde o rótulo tem menos coluna.
            min: -10.0,
            max: 10.0,
            step: 0.01,
            widget: ParamWidget::Slider,
        },
        value,
    )];
    n
}

/// Quantos glifos uma row com este rótulo pinta, a `zoom 1`.
fn glyphs_of(label: &'static str, value: f32) -> u32 {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![one_row(label, value)],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let viewport = Rect::new(0.0, 0.0, W, H);
    let mut layout = HeroLayout::for_viewport(viewport);
    layout.motion_graph = viewport;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState {
        view: ViewState {
            zoom: 1.0,
            ..ViewState::default()
        },
        fitted: true,
        ..MotionGraphPanelState::default()
    };
    let (glyphs, _) =
        host.paint_and_count_geometry_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);
    set_current_motion_graph(None);
    glyphs
}

/// ⭐⭐ **NENHUM RÓTULO DOS DOIS *WARPS* É CORTADO NO CARTÃO** — e o oráculo é a CENA, não a
/// aritmética da coluna.
///
/// ⚠️ **Por que este gate existe:** o painel lateral SAIU (ciclo 3), então o cartão é a única
/// superfície onde estes nomes aparecem — e ele tem `190 px` contra os `~35 caracteres` que a
/// row do painel tinha. Um rótulo que não cabe **não dá erro**: o elidor corta-o e o artista lê
/// `Bottom-Rig…`, que é indistinguível de `Bottom-Lef…` na coluna a seguir. *Foi assim que os
/// números dos cartões shiparam como `0....` em 2026-09-05* — medir num sítio e pintar noutro.
///
/// ⚠️ **A régua é a contagem de GLIFOS, e ela mede-se por DIFERENÇA.** ⛔ A primeira redacção
/// comparava a contagem ABSOLUTA com `rótulo + valor` e leu `23` contra `20`: um cartão pinta
/// também o **título** e os **rótulos dos pinos**, e um oráculo que os ignora acusa como
/// «cortado» um rótulo que está inteiro — *a régua errava para o lado que fabrica dívida*. A
/// pergunta certa é **quantos glifos o RÓTULO acrescenta**, e a resposta é a mesma cena pintada
/// duas vezes, com e sem ele: tudo o resto cancela-se. Se o rótulo passar pelo elidor, os
/// caracteres cortados desaparecem e entra **um** `…` no lugar de vários, logo a diferença CAI.
///
/// ⚠️ **O controle é o rótulo IMPOSSÍVEL**: sem ele, um pintor que deixasse de escrever o
/// rótulo devolveria menos glifos em todos os casos e a asserção `>=` ficaria... vermelha, sim
/// — mas um pintor que deixasse de ELIDIR passaria despercebido. O caso longo prova que a
/// elisão está viva e que este gate a vê.
#[test]
fn no_warp_label_is_cut_on_the_card() {
    // `-10.00` é o valor mais largo da faixa dos dois nós — o pior caso para o rótulo.
    const VALOR: f32 = -10.0;

    // Os rótulos mais LONGOS dos dois warps (o vocabulário que o ciclo 3 unificou) e dois
    // curtos das tangentes — o par que mostra que o gate não passa só por ser generoso.
    // O cartão sem rótulo nenhum — o título, os pinos e o valor, que é o que se cancela.
    let base = glyphs_of("", VALOR);
    for label in [
        "Bottom-Right X",
        "Bottom-Left Y",
        "Top-Right Y",
        "Top-Left X",
        "In X",
        "Out Y",
    ] {
        let esperado = u32::try_from(label.chars().count()).unwrap();
        let vistos = glyphs_of(label, VALOR).saturating_sub(base);
        assert_eq!(
            vistos, esperado,
            "o rótulo `{label}` é CORTADO no cartão: ele acrescenta {vistos} glifos onde os \
             seus {esperado} caracteres cabiam. Um rótulo elidido lê-se como o vizinho dele, \
             e no cartão não há painel lateral onde ver o nome inteiro"
        );
    }

    // ⚠️ **Controle: um rótulo que NÃO pode caber tem de ser cortado.** Sem isto o gate acima
    // passaria sobre um pintor que tivesse deixado de elidir — e aí a lei estaria a ser
    // afirmada por um elidor morto, não honrada por rótulos curtos.
    let impossivel = "Bottom-Right Tangent Out Horizontal";
    let cru = u32::try_from(impossivel.chars().count()).unwrap();
    let cortado = glyphs_of(impossivel, VALOR).saturating_sub(base);
    assert!(
        cortado < cru,
        "o elidor não cortou um rótulo de {} caracteres numa coluna de 190 px — \
         então o gate acima não está a medir elisão nenhuma",
        impossivel.chars().count()
    );
}

/// ⭐⭐⭐ **O ESTADO DE UM SELECTOR NO CARTÃO É UMA PALAVRA, NÃO UM IDENTIFICADOR.**
///
/// ⚠️ Esta é a **segunda** das três superfícies do mesmo array de opções, e não passa por
/// nenhuma das outras: o painel resolve no pintor dele, a LISTA resolve em
/// [`crate::snapshot::CardChoices::labels`], e o ESTADO — a palavra que o cartão mostra sem
/// abrir nada — resolve-se aqui. ⛔ Um `tr` em falta neste braço deixa os outros dois certos e
/// escreve `node.motion.wave.param.edges.0` na row fechada, que é o que o artista vê primeiro.
///
/// FALSIFICADO por devolver `(*s).to_string()` — a forma que este ficheiro tinha antes da 5.ª
/// fatia do HR-15, quando o array carregava texto.
#[test]
fn an_enum_row_shows_the_word_and_never_the_key() {
    const K: &[&str] = &[
        "node.motion.wave.param.edges.0",
        "node.motion.wave.param.edges.1",
    ];
    // ⛔ Controlo da fixtura: com a chave a não resolver, os dois lados seriam identificadores e
    // o gate mediria nada.
    assert_ne!(
        ph2d_i18n::tr(K[1]),
        K[1],
        "a chave {:?} não resolve — a fixtura deixou de conter o fenómeno",
        K[1]
    );
    let p = crate::CardParam::from_hint(
        ParamUiHint {
            param: "edges",
            label: "node.motion.wave.param.edges",
            min: 0.0,
            max: 1.0,
            step: 1.0,
            widget: ParamWidget::Enum { labels: K },
        },
        1.0,
    );
    let crate::paint::paint_card_params::Shown::State(s) =
        crate::paint::paint_card_params::shown(&p)
    else {
        panic!("um selector mostra um ESTADO");
    };
    assert_eq!(s, ph2d_i18n::tr(K[1]), "o cartão escreveu {s:?}");
    assert!(!s.starts_with("node."), "o cartão escreveu a chave: {s:?}");
}
