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
