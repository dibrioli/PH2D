//! ⭐⭐⭐ **O CENSO DOS CONTROLOS: cada controlo que o painel pinta, em cada meio, MUDA o traço.**
//!
//! Ordem do dono (2026-10-02): *«detectar cada parâmetro morto ou mal utilizado em cada seção do
//! painel Painter para cada um dos modos de pintura»*. A população é DERIVADA DA TELA: o painel é
//! pintado com o meio escolhido e todas as secções abertas, e cada id com rectângulo é conduzido pela
//! porta do ponteiro (clique, número digitado, opção de menu) → `apply_event` → o barramento →
//! `Tool::handle_panel_event`. Depois risca-se o MESMO traço que na base e comparam-se os pixels.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::interaction::state::InteractiveState;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeMap;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 6000.0)
}

/// Os ficheiros que declaram os ids que o painel do pincel pinta — para o NOME de cada id no relatório.
const FICHEIROS_DE_IDS: [&str; 18] = [
    include_str!("../../../ph2d-tool-painter/src/ids/painter.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_shape_cor.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_brush_sections.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_deform.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_gradient.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_impasto.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_line.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_sculpt.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_selection.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_shape.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_stroke_op.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_substrate.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_symmetry.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_texture.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_tiling.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_watercolor.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/painter_wetpaint.rs"),
    include_str!("../../../ph2d-tool-painter/src/ids/wet_tuning.rs"),
];

/// `NodeId → "CONST (slug)"` a partir dos literais `pub const X: NodeId = hash_node_id("slug")`.
pub(crate) fn nomes() -> BTreeMap<NodeId, String> {
    let mut m = BTreeMap::new();
    for fonte in FICHEIROS_DE_IDS {
        for linha in fonte.lines() {
            let Some(resto) = linha.trim().strip_prefix("pub const ") else {
                continue;
            };
            let Some((nome, depois)) = resto.split_once(':') else {
                continue;
            };
            let Some((_, lit)) = depois.split_once("hash_node_id(\"") else {
                continue;
            };
            let Some((slug, _)) = lit.split_once('"') else {
                continue;
            };
            m.insert(
                ph2d_tool_registry::hash_node_id_runtime(slug),
                nome.trim().to_owned(),
            );
        }
    }
    m
}

pub(crate) fn nome(nomes: &BTreeMap<NodeId, String>, id: NodeId) -> String {
    nomes
        .get(&id)
        .cloned()
        .unwrap_or_else(|| format!("{:#018x}", id.0))
}

/// O lado da tela: 128² chega para os dois traços e para a borda da aguada.
const LADO: u32 = 128;

/// A tela de partida: branca, com um BLOCO de cor no quadrante de baixo-direita — os controlos que
/// MISTURAM (Pickup, Smudge, Mix, Rewet…) só agem sobre tinta que já está lá.
fn tela() -> Vec<u8> {
    let mut px = vec![255u8; (LADO * LADO * 4) as usize];
    for y in 64..LADO {
        for x in 64..LADO {
            let i = ((y * LADO + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[200, 60, 40, 255]);
        }
    }
    px
}

fn ferramenta(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(tela(), LADO, LADO);
    t.set_paint_media(media);
    // Uma cor SATURADA: com o cinzento de fábrica o Hue e o Saturation do Randomize não têm onde agir.
    t.set_brush_color_srgb8([40, 90, 200]);
    // A água num relógio FIXO: ao relógio real, sob carga, duas corridas iguais divergiam.
    t.set_wet_relogio_fixo(true);
    t
}

/// Uma ponta de imagem de UMA camada, 16², opaca, com as cores a variar por texel (o vermelho e o verde
/// em rampa) — a cor da textura que o «Use Texture Colors» pinta onde a mídia a lê.
fn ponta_colorida(t: &mut PainterTool) {
    const L: u32 = 16;
    let mut px = vec![255u8; (L * L * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (i as u32 % L, i as u32 / L);
        p[0] = (x * 16) as u8;
        p[1] = (y * 16) as u8;
        p[2] = 60;
    }
    t.set_brush_shape_image_rgba(&px, L, L, Some(7));
}

/// Pinta o painel do pincel como a ponte o publica, com TODA secção aberta.
fn pinta(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    host.open_all_sections();
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, vivos(rects))
}

fn vivos(rects: Vec<(NodeId, Rect)>) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = Vec::new();
    for (id, r) in rects {
        if r.w > 0.0 && r.h > 0.0 && !v.iter().any(|(i, _)| *i == id) {
            v.push((id, r));
        }
    }
    v
}

/// O gesto do artista sobre UM controlo.
#[derive(Clone, Debug)]
pub(crate) enum Gesto {
    /// Nada — a base, e o CONTROLO do ruído (a base corrida duas vezes).
    Nenhum,
    /// Digitar um número no chip, com Enter.
    Numero(NodeId, f64),
    /// Arrastar o slider até `valor` (o que o arrasto deixa no store antes do `ValueChanged`).
    Slider(NodeId, f32),
    /// Um clique de ponteiro no centro do rectângulo.
    Clique(NodeId),
    /// Um traço na tela (o [`traco_um`]) com a ferramenta como está — para ARMAR um controlo que só
    /// age sobre tinta que já lá está (a borracha da água).
    Traco,
    /// Instalar uma ponta de IMAGEM colorida de uma camada (a rota do «Use as Brush Shape» da Hierarchy,
    /// que não é deste painel) — para ARMAR o «Use Texture Colors».
    PontaColorida,
    /// Arrastar a pega de uma curva até `(fx, fy)` da ÁREA da curva (`0..1`, `y` para baixo).
    Arrasto(NodeId, f32, f32),
    /// Abrir o menu e clicar na opção `k` de `n`.
    Opcao(NodeId, NodeId, usize, usize),
}

impl Gesto {
    pub(crate) fn id(&self) -> Option<NodeId> {
        match self {
            Gesto::Nenhum | Gesto::Traco | Gesto::PontaColorida => None,
            Gesto::Numero(id, _)
            | Gesto::Slider(id, _)
            | Gesto::Clique(id)
            | Gesto::Arrasto(id, ..) => Some(*id),
            Gesto::Opcao(_, o, ..) => Some(*o),
        }
    }
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// O valor para onde se leva um número: 60 % do caminho até à ponta MAIS LONGE da faixa.
fn alvo(valor: f64, faixa: Option<(f64, f64, f64)>) -> f64 {
    let Some((lo, hi, passo)) = faixa else {
        return valor + valor.abs().max(1.0);
    };
    let longe = if (hi - valor).abs() >= (valor - lo).abs() {
        hi
    } else {
        lo
    };
    let mut v = valor + (longe - valor) * 0.6;
    if passo > 0.0 {
        v = lo + ((v - lo) / passo).round() * passo;
    }
    if (v - valor).abs() < 1e-9 { longe } else { v }
}

/// O que o barramento levou: os `PanelEvent` à ferramenta e o nome das OUTRAS ações (que são da shell).
#[derive(Default)]
pub(crate) struct Entrega {
    pub(crate) ao_tool: usize,
    /// O instantâneo do pincel (`BrushSettings`) mudou com o gesto.
    pub(crate) ajuste_mudou: bool,
    /// Os ids que o gesto fez APARECER no painel — um estado novo que o censo visita a seguir.
    revelados: Vec<NodeId>,
    outras: Vec<String>,
}

/// Entrega o barramento à ferramenta, publica o instantâneo e pinta de novo — um QUADRO do app.
fn quadro(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    entrega: &mut Entrega,
) {
    for a in host.drained_actions() {
        match a {
            EditorAction::ToolPanelEvent(pe) => {
                tool.handle_panel_event(pe);
                entrega.ao_tool += 1;
            }
            outra => {
                let nome = format!("{outra:?}");
                let curto = nome.split(['(', ' ', '{']).next().unwrap_or("").to_owned();
                if !entrega.outras.contains(&curto) {
                    entrega.outras.push(curto);
                }
            }
        }
    }
    set_current_brush(Some(tool.brush_settings()));
    let _ = host.paint::<PainterLayersPanel>(st, viewport());
}

fn conduz(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    rects: &[(NodeId, Rect)],
    gesto: &Gesto,
) -> Vec<WidgetEvent> {
    let rect = |id: NodeId| rects.iter().find(|(i, _)| *i == id).map(|(_, r)| *r);
    match gesto {
        Gesto::Nenhum | Gesto::Traco | Gesto::PontaColorida => Vec::new(),
        Gesto::Numero(id, v) => {
            // ⚠️ O alvo recalcula-se se o chip JÁ mostra esse valor: um commit igual não emite nada, e
            // o estado do painel que sobrevive entre ensaios no mesmo fio (o stop seleccionado da
            // rampa) muda o que o chip mostra — foi assim que o Stop Pos se leu «mudo».
            let agora = host.store().number_value(*id).unwrap_or(*v);
            let v = if (agora - v).abs() < 1e-9 {
                alvo(agora, host.store().number_range(*id))
            } else {
                *v
            };
            host.type_into_number(*id, &format!("{v}"))
        }
        Gesto::Slider(id, v) => {
            host.set_slider_value(*id, *v);
            vec![WidgetEvent::ValueChanged(*id)]
        }
        Gesto::Clique(id) => {
            let (x, y) = centro(rect(*id).expect("o controlo foi pintado"));
            host.click_at(x, y)
        }
        Gesto::Arrasto(id, fx, fy) => {
            let (x, y) = centro(rect(*id).expect("a pega foi pintada"));
            let Some(InteractiveState::CurvePoint { canvas, .. }) = host.store().get(*id) else {
                panic!("{id:?} não é a pega de uma curva");
            };
            let (x1, y1) = (canvas.x + canvas.w * fx, canvas.y + canvas.h * fy);
            host.drag_at(x, y, x1, y1)
        }
        Gesto::Opcao(menu, opcao, ..) => {
            host.set_dropdown_open(*menu, true);
            let novos = vivos(host.paint::<PainterLayersPanel>(st, viewport()));
            let r = novos
                .iter()
                .find(|(i, _)| i == opcao)
                .map(|(_, r)| *r)
                .expect("a opção foi pintada com o menu aberto");
            let (x, y) = centro(r);
            host.click_at(x, y)
        }
    }
}

fn ponteiro(pos: [f32; 2], pressure: f32, tilt: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt,
        phase,
    }
}

/// Fecha um quadro do lado da tela: o tique e a drenagem da pré-visualização, na ordem da ponte.
///
/// ⚠️ **O Wet Paint corre a água numa thread própria, ritmada pelo RELÓGIO REAL** (`wetpaint/offthread.rs`):
/// sem dormir, a água nunca andava (a 1.ª corrida leu Water, Flow, Gravity… «mortos»); dormindo, sob
/// carga, duas corridas iguais divergiam em 1 148 texels (um «vivo» podia ser ruído). ⇒ a ferramenta do
/// censo corre a água no relógio FIXO ([`ferramenta`]): cada tique deve os passos de 16,7 ms.
fn tique(t: &mut PainterTool, ultima: &mut Option<Vec<u8>>) {
    t.on_tick(1000.0 / 60.0);
    if let Some((px, _, _)) = t.take_preview_arc() {
        *ultima = Some(px.as_ref().clone());
    }
}

/// Um traço de 40 amostras de `de` a `ate`, ondulado, com a pressão de `p0` a `p1`; cada amostra fecha
/// um quadro.
fn traco(
    t: &mut PainterTool,
    ultima: &mut Option<Vec<u8>>,
    de: [f32; 2],
    ate: [f32; 2],
    p: [f32; 2],
    tilt: [f32; 2],
) {
    let n = 40;
    t.on_canvas_pointer(ponteiro(de, p[0], tilt, PointerPhase::Down));
    let mut pos = de;
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let onda = (f * std::f32::consts::TAU * 1.5).sin() * 6.0;
        pos = [
            de[0] + (ate[0] - de[0]) * f,
            de[1] + (ate[1] - de[1]) * f + onda,
        ];
        t.on_canvas_pointer(ponteiro(
            pos,
            p[0] + (p[1] - p[0]) * f,
            tilt,
            PointerPhase::Move,
        ));
        tique(t, ultima);
    }
    t.on_canvas_pointer(ponteiro(pos, p[1], tilt, PointerPhase::Up));
    tique(t, ultima);
}

/// O 1.º traço: ondulado, da esquerda para a direita, com a pressão a SUBIR. Atravessa as duas bordas
/// verticais — o Tiling X só tem o que fazer com um carimbo que sai da tela.
fn traco_um(t: &mut PainterTool, ultima: &mut Option<Vec<u8>>) {
    traco(
        t,
        ultima,
        [-8.0, 40.0],
        [136.0, 40.0],
        [0.3, 1.0],
        [0.0, 0.0],
    );
}

/// Os DOIS traços de fábrica: o [`traco_um`] e uma diagonal noutra COR que o cruza, entra no bloco
/// vermelho e sai pelo topo e pelo fundo (o Tiling Y), com a pressão a DESCER e a caneta inclinada.
/// Depois, meio segundo de quadros parados (a secagem, a água a correr).
///
/// ⚠️ **A segunda cor é a régua dos controlos que MISTURAM.** Com os dois traços da mesma cor, o
/// Pickup do Wet Paint apanhava azul de cima de azul e lia-se «morto» (2026-10-03).
pub(crate) fn risca(t: &mut PainterTool) -> Vec<u8> {
    let mut ultima = None;
    traco_um(t, &mut ultima);
    for _ in 0..4 {
        tique(t, &mut ultima);
    }
    t.set_brush_color_srgb8([230, 190, 40]);
    traco(
        t,
        &mut ultima,
        [24.0, 140.0],
        [104.0, -12.0],
        [1.0, 0.4],
        [0.4, -0.3],
    );
    for _ in 0..30 {
        tique(t, &mut ultima);
    }
    ultima.expect("os traços sujaram a pré-visualização")
}

/// Um ENSAIO: ferramenta nova no meio, painel novo, o gesto pela porta do ponteiro, dois quadros, os
/// dois traços. Devolve os pixels e o que o barramento levou.
pub(crate) fn ensaio(media: PaintMedia, armar: &[Gesto], gesto: &Gesto) -> (Vec<u8>, Entrega) {
    let mut bancada = Bancada::nova(media, armar);
    let antes = format!("{:?}", bancada.tool.brush_settings());
    let pintados: Vec<NodeId> = bancada.rects.iter().map(|(i, _)| *i).collect();
    let mut entrega = Entrega::default();
    bancada.aplica(gesto, &mut entrega);
    entrega.ajuste_mudou = format!("{:?}", bancada.tool.brush_settings()) != antes;
    entrega.revelados = bancada
        .rects
        .iter()
        .map(|(i, _)| *i)
        .filter(|i| !pintados.contains(i))
        .collect();
    (risca(&mut bancada.tool), entrega)
}

/// O ensaio ao CONTRÁRIO: risca primeiro e só depois faz o gesto — mede os controlos que agem sobre a
/// tinta que JÁ está na tela (a luz e o material do impasto, o Adjust Last Stroke, a água do Wet Paint).
fn ensaio_depois(media: PaintMedia, armar: &[Gesto], gesto: &Gesto) -> (Vec<u8>, Entrega) {
    let mut bancada = Bancada::nova(media, armar);
    let mut ultima = Some(risca(&mut bancada.tool));
    let antes = format!("{:?}", bancada.tool.brush_settings());
    let mut entrega = Entrega::default();
    bancada.aplica(gesto, &mut entrega);
    entrega.ajuste_mudou = format!("{:?}", bancada.tool.brush_settings()) != antes;
    for _ in 0..30 {
        tique(&mut bancada.tool, &mut ultima);
    }
    (ultima.expect("a tela foi lida"), entrega)
}

/// O modo do censo: `CENSO_MODO=depois` mede o gesto DEPOIS dos traços ([`ensaio_depois`]).
fn mede(media: PaintMedia, armar: &[Gesto], gesto: &Gesto) -> (Vec<u8>, Entrega) {
    if std::env::var("CENSO_MODO").is_ok_and(|m| m == "depois") {
        ensaio_depois(media, armar, gesto)
    } else {
        ensaio(media, armar, gesto)
    }
}

/// A ferramenta e o painel de UM ensaio, depois das PRÉ-CONDIÇÕES (`armar`) aplicadas pela mesma porta.
pub(crate) struct Bancada {
    pub(crate) tool: PainterTool,
    pub(crate) host: MockPanelHost,
    st: PainterLayersPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Bancada {
    pub(crate) fn nova(media: PaintMedia, armar: &[Gesto]) -> Self {
        let tool = ferramenta(media);
        let (host, st, rects) = pinta(&tool);
        let mut b = Bancada {
            tool,
            host,
            st,
            rects,
        };
        let mut lixo = Entrega::default();
        for a in armar {
            if matches!(a, Gesto::Traco) {
                let mut ultima = None;
                traco_um(&mut b.tool, &mut ultima);
                continue;
            }
            if matches!(a, Gesto::PontaColorida) {
                ponta_colorida(&mut b.tool);
                set_current_brush(Some(b.tool.brush_settings()));
                b.rects = vivos(b.host.paint::<PainterLayersPanel>(&mut b.st, viewport()));
                continue;
            }
            b.aplica(a, &mut lixo);
            assert!(
                lixo.ao_tool > 0,
                "{media:?}: a pré-condição {a:?} não chegou à ferramenta"
            );
            lixo.ao_tool = 0;
        }
        b
    }

    /// Os controlos pintados e vivos agora (id, rectângulo).
    pub(crate) fn rects(&self) -> &[(NodeId, Rect)] {
        &self.rects
    }

    /// A ferramenta do trilho `modo` pelo botão do trilho (a porta gravada do painel), e o painel
    /// re-pintado com as secções dela.
    pub(crate) fn no_modo(&mut self, modo: &str) {
        self.tool
            .handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
                ph2d_editor_core::ids::PAINTER_PAINT_MODE,
                modo.to_owned(),
            ));
        self.aplica(&Gesto::Nenhum, &mut Entrega::default());
    }

    /// Um gesto pela porta do ponteiro, dois quadros do app, e o painel re-pintado com o resultado.
    pub(crate) fn aplica(&mut self, gesto: &Gesto, entrega: &mut Entrega) {
        let eventos = conduz(&mut self.host, &mut self.st, &self.rects, gesto);
        for ev in eventos {
            let _ = self
                .host
                .apply_panel_event::<PainterLayersPanel>(&mut self.st, ev);
        }
        quadro(&mut self.host, &mut self.st, &mut self.tool, entrega);
        quadro(&mut self.host, &mut self.st, &mut self.tool, entrega);
        self.rects = vivos(
            self.host
                .paint::<PainterLayersPanel>(&mut self.st, viewport()),
        );
    }
}

/// `(texels diferentes, soma |Δ| sobre os canais, |Δ| máximo)`.
pub(crate) fn diferenca(a: &[u8], b: &[u8]) -> (usize, u64, u8) {
    let mut n = 0;
    let mut soma = 0u64;
    let mut max = 0u8;
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let mut mudou = false;
        for c in 0..4 {
            let d = pa[c].abs_diff(pb[c]);
            soma += u64::from(d);
            max = max.max(d);
            mudou |= d > 0;
        }
        n += usize::from(mudou);
    }
    (n, soma, max)
}

/// Os gestos que a tela oferece: um por número/slider, um clique por botão/cabeçalho, e uma opção de
/// cada menu (o menu aberto pinta as opções, que viram gestos próprios).
pub(crate) fn gestos(media: PaintMedia, armar: &[Gesto]) -> Vec<Gesto> {
    gestos_da(Bancada::nova(media, armar))
}

/// [`gestos`] de uma bancada já preparada (a ferramenta do trilho escolhida, a tela tocada).
pub(crate) fn gestos_da(b: Bancada) -> Vec<Gesto> {
    let Bancada {
        mut host,
        mut st,
        rects,
        ..
    } = b;
    let mut out = Vec::new();
    for (id, _) in &rects {
        match host.store().get(*id) {
            Some(InteractiveState::NumberInput { value, .. }) => {
                out.push(Gesto::Numero(
                    *id,
                    alvo(*value, host.store().number_range(*id)),
                ));
            }
            Some(InteractiveState::Slider { value, .. }) => {
                let v = if *value < 0.5 { 0.8 } else { 0.2 };
                out.push(Gesto::Slider(*id, v));
            }
            Some(InteractiveState::Dropdown { .. }) => {
                host.set_dropdown_open(*id, true);
                let abertos = vivos(host.paint::<PainterLayersPanel>(&mut st, viewport()));
                host.set_dropdown_open(*id, false);
                let opcoes: Vec<NodeId> = abertos
                    .iter()
                    .map(|(o, _)| *o)
                    .filter(|o| !rects.iter().any(|(i, _)| i == o))
                    .collect();
                for (k, o) in opcoes.iter().enumerate() {
                    out.push(Gesto::Opcao(*id, *o, k, opcoes.len()));
                }
            }
            Some(InteractiveState::CurvePoint { .. }) => {
                // A pega vai a 60 % × 40 % da área da curva: longe da ponta onde a fábrica a pôs.
                out.push(Gesto::Arrasto(*id, 0.6, 0.4));
            }
            _ => out.push(Gesto::Clique(*id)),
        }
    }
    out
}

/// Corre `f` sobre cada item em [`FIOS`] threads e devolve os resultados NA ORDEM dos gestos.
///
/// ⚠️ Os ensaios são independentes por construção — cada um faz a sua ferramenta e o seu painel, e o
/// instantâneo publicado ao painel (`set_current_brush`) é por THREAD. O paralelo existe pelo Wet
/// Paint: o ensaio dele DORME (a água anda ao relógio), e em série a exploração não cabia no prazo.
pub(crate) fn em_paralelo<I: Sync, T: Send>(gestos: &[I], f: impl Fn(&I) -> T + Sync) -> Vec<T> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let proximo = AtomicUsize::new(0);
    let saidas: Vec<std::sync::Mutex<Option<T>>> =
        gestos.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..FIOS {
            s.spawn(|| {
                loop {
                    let k = proximo.fetch_add(1, Ordering::Relaxed);
                    let Some(g) = gestos.get(k) else { break };
                    let r = f(g);
                    *saidas[k].lock().expect("sem envenenamento") = Some(r);
                }
            });
        }
    });
    saidas
        .into_iter()
        .map(|m| {
            m.into_inner()
                .expect("sem envenenamento")
                .expect("cada gesto foi medido")
        })
        .collect()
}

/// Quantos ensaios correm ao mesmo tempo.
const FIOS: usize = 8;

/// Quantos gestos de pré-condição o censo encadeia ao descer a estados revelados.
const PROFUNDIDADE: usize = 3;

fn tipo_do_gesto(g: &Gesto) -> &'static str {
    match g {
        Gesto::Nenhum => "nenhum",
        Gesto::Traco => "traço",
        Gesto::PontaColorida => "ponta colorida",
        Gesto::Numero(..) => "número",
        Gesto::Slider(..) => "slider",
        Gesto::Clique(..) => "clique",
        Gesto::Arrasto(..) => "arrasto",
        Gesto::Opcao(..) => "opção",
    }
}

// A tabela e as pré-condições, e os controlos esmaecidos: no MESMO módulo (os nomes dos testes ficam).
include!("censo_dos_controlos_tabela.rs");
include!("censo_dos_controlos_esmaecidos.rs");
