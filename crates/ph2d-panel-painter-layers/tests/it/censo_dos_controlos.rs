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
const FICHEIROS_DE_IDS: [&str; 17] = [
    include_str!("../../../ph2d-tool-painter/src/ids/painter.rs"),
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
fn nomes() -> BTreeMap<NodeId, String> {
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

fn nome(nomes: &BTreeMap<NodeId, String>, id: NodeId) -> String {
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
enum Gesto {
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
    /// Arrastar a pega de uma curva até `(fx, fy)` da ÁREA da curva (`0..1`, `y` para baixo).
    Arrasto(NodeId, f32, f32),
    /// Abrir o menu e clicar na opção `k` de `n`.
    Opcao(NodeId, NodeId, usize, usize),
}

impl Gesto {
    fn id(&self) -> Option<NodeId> {
        match self {
            Gesto::Nenhum | Gesto::Traco => None,
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
struct Entrega {
    ao_tool: usize,
    /// O instantâneo do pincel (`BrushSettings`) mudou com o gesto.
    ajuste_mudou: bool,
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
        Gesto::Nenhum | Gesto::Traco => Vec::new(),
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
fn risca(t: &mut PainterTool) -> Vec<u8> {
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
fn ensaio(media: PaintMedia, armar: &[Gesto], gesto: &Gesto) -> (Vec<u8>, Entrega) {
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
struct Bancada {
    tool: PainterTool,
    host: MockPanelHost,
    st: PainterLayersPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Bancada {
    fn nova(media: PaintMedia, armar: &[Gesto]) -> Self {
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
            b.aplica(a, &mut lixo);
            assert!(
                lixo.ao_tool > 0,
                "{media:?}: a pré-condição {a:?} não chegou à ferramenta"
            );
            lixo.ao_tool = 0;
        }
        b
    }

    /// Um gesto pela porta do ponteiro, dois quadros do app, e o painel re-pintado com o resultado.
    fn aplica(&mut self, gesto: &Gesto, entrega: &mut Entrega) {
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
fn diferenca(a: &[u8], b: &[u8]) -> (usize, u64, u8) {
    let mut n = 0;
    let mut soma = 0u64;
    let mut max = 0u8;
    for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
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
fn gestos(media: PaintMedia, armar: &[Gesto]) -> Vec<Gesto> {
    let Bancada {
        mut host,
        mut st,
        rects,
        ..
    } = Bancada::nova(media, armar);
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
fn em_paralelo<I: Sync, T: Send>(gestos: &[I], f: impl Fn(&I) -> T + Sync) -> Vec<T> {
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
        Gesto::Numero(..) => "número",
        Gesto::Slider(..) => "slider",
        Gesto::Clique(..) => "clique",
        Gesto::Arrasto(..) => "arrasto",
        Gesto::Opcao(..) => "opção",
    }
}

#[test]
#[ignore = "sonda: a tabela meio × controlo (o entregável (a) da missão de 2026-10-02)"]
fn sonda_a_tabela_dos_controlos() {
    let nomes = nomes();
    let so: Option<String> = std::env::var("CENSO_MEIO").ok();
    let rotulo = |g: &Gesto| -> String {
        let n = nome(&nomes, g.id().unwrap_or(NodeId(0)));
        match g {
            Gesto::Opcao(m, _, k, t) => format!("{}[{k}/{t}]", nome(&nomes, *m)),
            _ => n,
        }
    };
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        if so.as_ref().is_some_and(|m| *m != format!("{media:?}")) {
            continue;
        }
        let mut coberto: Vec<NodeId> = Vec::new();
        let mut candidatos: Vec<(Vec<Gesto>, Gesto)> = Vec::new();
        let mut fila: std::collections::VecDeque<Vec<Gesto>> = [Vec::new()].into();
        while let Some(armar) = fila.pop_front() {
            let estado = if armar.is_empty() {
                "fábrica".to_owned()
            } else {
                armar.iter().map(&rotulo).collect::<Vec<_>>().join(" + ")
            };
            let t0 = std::time::Instant::now();
            let (base, _) = mede(media, &armar, &Gesto::Nenhum);
            // O controlo do ruído corre-se UMA vez, na fábrica: um ensaio do Wet Paint custa 2 s.
            let de_novo = if armar.is_empty() {
                mede(media, &armar, &Gesto::Nenhum).0
            } else {
                base.clone()
            };
            let ruido = diferenca(&base, &de_novo);
            eprintln!(
                "CENSO\t{media:?}\tCONTROLO\t-\t{}\t{}\t{}\t-\t-\t-\testado={estado}\t({:.2} s por ensaio)",
                ruido.0,
                ruido.1,
                ruido.2,
                t0.elapsed().as_secs_f32() / 2.0
            );
            let mut deste: Vec<Gesto> = Vec::new();
            for g in gestos(media, &armar) {
                let id = g.id().unwrap_or(NodeId(0));
                if !coberto.contains(&id) {
                    coberto.push(id);
                    deste.push(g);
                }
            }
            let medidas = em_paralelo(&deste, |g| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mede(media, &armar, g)))
                    .ok()
            });
            for (g, medida) in deste.iter().zip(medidas) {
                let Some((px, entrega)) = medida else {
                    eprintln!(
                        "CENSO\t{media:?}\t{}\tPÂNICO\t-\t-\t-\t-\t-\t-\testado={estado}\t{g:?}",
                        rotulo(g)
                    );
                    continue;
                };
                let (n, soma, max) = diferenca(&base, &px);
                eprintln!(
                    "CENSO\t{media:?}\t{}\t{}\t{n}\t{soma}\t{max}\t{}\t{}\t{}\testado={estado}\t{g:?}",
                    rotulo(g),
                    tipo_do_gesto(g),
                    entrega.ao_tool,
                    if entrega.ajuste_mudou {
                        "mudou"
                    } else {
                        "igual"
                    },
                    entrega.outras.join(",")
                );
                if n == 0 && entrega.ao_tool > 0 && entrega.ajuste_mudou {
                    candidatos.push((armar.clone(), g.clone()));
                }
                let novos = entrega.revelados.iter().any(|i| !coberto.contains(i));
                if novos && armar.len() < PROFUNDIDADE && entrega.ao_tool > 0 {
                    let mut mais = armar.clone();
                    mais.push(g.clone());
                    fila.push_back(mais);
                }
            }
        }
        eprintln!("CENSO-COBERTO {media:?}: {} gestos", coberto.len());
        if std::env::var("CENSO_ARMAR").is_ok() {
            procura_as_pre_condicoes(media, &candidatos, &rotulo);
        }
    }
}

/// Quão longe (px, na vertical) um vizinho pode estar para contar como «do mesmo cartão».
const VIZINHANCA_PX: f32 = 160.0;

/// ⭐ **A PRÉ-CONDIÇÃO PROCURA-SE, não se adivinha.** Para cada candidato a morto (o ajuste mudou e a
/// imagem não), tenta como pré-condição cada gesto VIZINHO do mesmo estado (os controlos do mesmo
/// cartão, do mais perto para o mais longe) e um traço já pintado na tela. O primeiro que o faz mudar
/// a imagem — contra a base armada com o MESMO vizinho — é a pré-condição; nenhum ⇒ `MORTO`.
fn procura_as_pre_condicoes(
    media: PaintMedia,
    candidatos: &[(Vec<Gesto>, Gesto)],
    rotulo: &dyn Fn(&Gesto) -> String,
) {
    let resultados = em_paralelo(candidatos, |(armar, g)| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bancada = Bancada::nova(media, armar);
            let y_de = |id: NodeId| {
                bancada
                    .rects
                    .iter()
                    .find(|(i, _)| *i == id)
                    .map(|(_, r)| r.y)
            };
            let ancora = match g {
                Gesto::Opcao(menu, ..) => y_de(*menu),
                _ => g.id().and_then(y_de),
            };
            let todos = std::env::var("CENSO_ARMAR").is_ok_and(|v| v == "2");
            let mut vizinhos: Vec<(f32, Gesto)> = gestos(media, armar)
                .into_iter()
                .filter(|v| v.id() != g.id())
                .filter(|v| !matches!((v, g), (Gesto::Opcao(a, ..), Gesto::Opcao(b, ..)) if a == b))
                .filter_map(|v| {
                    let y = match &v {
                        Gesto::Opcao(menu, ..) => y_de(*menu),
                        _ => v.id().and_then(y_de),
                    }?;
                    let d = (y - ancora?).abs();
                    // Longe do cartão, de cada menu só as duas primeiras opções que não são a 0: os
                    // menus de 29 entradas (Shape, Grain, Paper) eram a maior parte do custo.
                    let poucas = !matches!(v, Gesto::Opcao(_, _, k, _) if k > 2);
                    (d <= VIZINHANCA_PX || (todos && poucas)).then_some((d, v))
                })
                .collect();
            vizinhos.sort_by(|a, b| a.0.total_cmp(&b.0));
            // A tinta na tela entra em CADA posição da sequência: o relevo que um esculpir precisa
            // pinta-se com o pincel de depósito, ANTES de a ferramenta de esculpir ser escolhida.
            let mut tentativas: Vec<(Gesto, Vec<Gesto>)> = (0..=armar.len())
                .rev()
                .map(|k| {
                    let mut mais = armar.clone();
                    mais.insert(k, Gesto::Traco);
                    (Gesto::Traco, mais)
                })
                .collect();
            tentativas.extend(vizinhos.into_iter().map(|(_, v)| {
                let mut mais = armar.clone();
                mais.push(v.clone());
                (v, mais)
            }));
            for (t, mais) in tentativas {
                let Ok((base, _)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mede(media, &mais, &Gesto::Nenhum)
                })) else {
                    continue;
                };
                let Ok((px, _)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mede(media, &mais, g)
                })) else {
                    continue;
                };
                let (n, ..) = diferenca(&base, &px);
                if n > 0 {
                    return Some((t, n));
                }
            }
            None
        }))
        .unwrap_or(None)
    });
    for ((armar, g), r) in candidatos.iter().zip(resultados) {
        let estado = if armar.is_empty() {
            "fábrica".to_owned()
        } else {
            armar.iter().map(rotulo).collect::<Vec<_>>().join(" + ")
        };
        match r {
            Some((t, n)) => eprintln!(
                "ARMA\t{media:?}\t{}\tINERTE-ATÉ\t{}\tn={n}\testado={estado}",
                rotulo(g),
                if matches!(t, Gesto::Traco) {
                    "tinta na tela".to_owned()
                } else {
                    rotulo(&t)
                }
            ),
            None => eprintln!(
                "ARMA\t{media:?}\t{}\tMORTO\t-\t-\testado={estado}",
                rotulo(g)
            ),
        }
    }
}

/// Um alvo da sonda das pré-condições: um id (o gesto é o que a tela armada oferece para ele) ou um
/// gesto escrito à mão (quando o sentido do movimento importa).
enum Alvo {
    Id(NodeId),
    Gesto(Gesto),
}

/// `(meio, a pré-condição em palavras, os gestos que a ARMAM, os alvos, medir DEPOIS do traço?)`.
type Linha = (PaintMedia, &'static str, Vec<Gesto>, Vec<Alvo>, bool);

fn opcao(menu: NodeId, opcao: NodeId) -> Gesto {
    Gesto::Opcao(menu, opcao, 0, 0)
}

/// As PRÉ-CONDIÇÕES que a 1.ª passagem do censo acusou e que a leitura do código nomeou. Cada linha
/// arma a condição pela porta do painel e mede os alvos contra a base JÁ armada.
fn pre_condicoes() -> Vec<Linha> {
    use PaintMedia::*;
    use ph2d_tool_painter::ids::*;
    let mut v: Vec<Linha> = Vec::new();
    for m in [Digital, Impasto, WetPaint] {
        v.push((
            m,
            "Taper com comprimento (a pega da curva arrastada)",
            vec![Gesto::Arrasto(painter_taper_handle_id(0), 0.6, 0.4)],
            vec![
                Alvo::Id(PAINTER_TAPER_TIP_START),
                Alvo::Id(PAINTER_TAPER_OPACITY),
            ],
            false,
        ));
        v.push((
            m,
            "Space Attenuation ligado",
            vec![Gesto::Clique(PAINTER_BRUSH_SPACE_ATTEN)],
            vec![Alvo::Id(PAINTER_BRUSH_ACCUMULATE)],
            false,
        ));
        v.push((
            m,
            "Accumulate ligado",
            vec![Gesto::Clique(PAINTER_BRUSH_ACCUMULATE)],
            vec![Alvo::Id(PAINTER_BRUSH_SPACE_ATTEN)],
            false,
        ));
        v.push((
            m,
            "Accumulate ligado e Strength 0,4",
            vec![
                Gesto::Clique(PAINTER_BRUSH_ACCUMULATE),
                Gesto::Slider(PAINTER_BRUSH_STRENGTH_SLIDER, 0.4),
            ],
            vec![Alvo::Id(PAINTER_BRUSH_SPACE_ATTEN)],
            false,
        ));
        v.push((
            m,
            "papel com relevo (Relief 0,6)",
            vec![Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6)],
            vec![
                Alvo::Gesto(opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(1),
                )),
                Alvo::Gesto(opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(2),
                )),
                Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS),
            ],
            false,
        ));
        v.push((
            m,
            "papel com relevo e um papel escolhido",
            vec![
                Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6),
                opcao(
                    PAINTER_WATERCOLOR_PAPER_KIND,
                    painter_paper_kind_option_id(1),
                ),
            ],
            vec![Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS)],
            false,
        ));
        v.push((
            m,
            "Shape não redonda",
            vec![opcao(PAINTER_SHAPE_KIND, painter_shape_kind_option_id(1))],
            vec![Alvo::Id(PAINTER_BRUSH_JITTER_ROTATE)],
            false,
        ));
    }
    for m in [Digital, Watercolor, Impasto, WetPaint] {
        v.push((
            m,
            "Jitter 0,8",
            vec![Gesto::Slider(PAINTER_BRUSH_JITTER, 0.8)],
            vec![Alvo::Gesto(opcao(
                PAINTER_BRUSH_JITTER_UNIT,
                painter_brush_jitter_unit_option_id(1),
            ))],
            false,
        ));
        v.push((
            m,
            "Dash ligado (Ratio 0,3)",
            vec![Gesto::Slider(PAINTER_BRUSH_DASH_RATIO, 0.3)],
            vec![
                Alvo::Id(PAINTER_BRUSH_DASH_LENGTH),
                Alvo::Id(PAINTER_BRUSH_DASH_LENGTH_CHIP),
            ],
            false,
        ));
    }
    v.push((
        Watercolor,
        "papel escolhido",
        vec![opcao(
            PAINTER_WATERCOLOR_PAPER_KIND,
            painter_paper_kind_option_id(1),
        )],
        vec![
            Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS),
            Alvo::Id(PAINTER_SUBSTRATE_RELIEF),
        ],
        false,
    ));
    v.push((
        Watercolor,
        "papel escolhido e Relief 0,6",
        vec![
            opcao(
                PAINTER_WATERCOLOR_PAPER_KIND,
                painter_paper_kind_option_id(1),
            ),
            Gesto::Numero(PAINTER_SUBSTRATE_RELIEF, 0.6),
        ],
        vec![Alvo::Id(PAINTER_SUBSTRATE_ROUGHNESS)],
        false,
    ));
    for (nome, arma) in [
        (
            "Wet 0,8, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_WET, 0.8),
        ),
        (
            "Smudge 0,8, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_SMUDGE, 0.8),
        ),
        (
            "Charge 0,4, Dry Time para o MÍNIMO",
            Gesto::Numero(PAINTER_WATERCOLOR_CHARGE, 0.4),
        ),
    ] {
        v.push((
            Watercolor,
            nome,
            vec![arma],
            vec![Alvo::Gesto(Gesto::Numero(PAINTER_WATERCOLOR_DRY_TIME, 2.0))],
            false,
        ));
    }
    v.push((
        Watercolor,
        "Same as Paper DESLIGADO",
        vec![Gesto::Clique(PAINTER_WATERCOLOR_GRAN_SAME)],
        (1..=4)
            .map(|k| {
                Alvo::Gesto(opcao(
                    PAINTER_BRUSH_TEXTURE_KIND,
                    painter_brush_texture_kind_option_id(k),
                ))
            })
            .collect(),
        false,
    ));
    v.push((
        Watercolor,
        "Charge 0,4 (o pincel volta a apanhar)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_CHARGE, 0.4)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_PULL)],
        false,
    ));
    v.push((
        Watercolor,
        "Wet 0,8 (a borda molhada)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_WET, 0.8)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_SPREAD)],
        false,
    ));
    v.push((
        Watercolor,
        "Ragged 24 (a borda manchada)",
        vec![Gesto::Numero(PAINTER_WATERCOLOR_WARP, 24.0)],
        vec![Alvo::Id(PAINTER_WATERCOLOR_SPREAD)],
        false,
    ));
    v.push((
        Watercolor,
        "fábrica, Dry Time para o MÍNIMO",
        Vec::new(),
        vec![Alvo::Gesto(Gesto::Numero(PAINTER_WATERCOLOR_DRY_TIME, 2.0))],
        false,
    ));
    v.push((
        Watercolor,
        "Automatic DESLIGADO e Shape não redonda",
        vec![
            Gesto::Clique(PAINTER_SHAPE_WATERCOLOR_AUTO),
            opcao(PAINTER_SHAPE_KIND, painter_shape_kind_option_id(1)),
        ],
        vec![
            Alvo::Id(PAINTER_BRUSH_JITTER_ROTATE),
            Alvo::Id(PAINTER_SHAPE_RAMP_ENABLE),
        ],
        false,
    ));
    v.push((
        Impasto,
        "um Grain escolhido",
        vec![opcao(
            PAINTER_BRUSH_TEXTURE_KIND,
            painter_brush_texture_kind_option_id(1),
        )],
        vec![Alvo::Id(PAINTER_IMPASTO_SOURCE_GRAIN)],
        false,
    ));
    v.push((
        Impasto,
        "a luz 2 seleccionada",
        vec![Gesto::Clique(PAINTER_IMPASTO_LIGHT_2)],
        vec![Alvo::Id(PAINTER_IMPASTO_LIGHT_ON)],
        true,
    ));
    v.push((
        Impasto,
        "Adjust Last Stroke DESLIGADO (fábrica)",
        Vec::new(),
        vec![
            Alvo::Id(PAINTER_IMPASTO_DEPTH),
            Alvo::Id(PAINTER_IMPASTO_BODY),
            Alvo::Id(PAINTER_IMPASTO_SHINE),
            Alvo::Id(PAINTER_IMPASTO_LIGHT_ANGLE),
        ],
        true,
    ));
    v.push((
        Impasto,
        "Adjust Last Stroke LIGADO",
        vec![Gesto::Clique(PAINTER_IMPASTO_LIVE_EDIT)],
        vec![
            Alvo::Id(PAINTER_IMPASTO_DEPTH),
            Alvo::Id(PAINTER_IMPASTO_BODY),
            Alvo::Id(PAINTER_IMPASTO_SHINE),
            Alvo::Id(PAINTER_IMPASTO_LIGHT_ANGLE),
        ],
        true,
    ));
    let ferramenta_wet = |k: usize| PAINTER_WETPAINT_TOOL_IDS[k];
    v.push((
        WetPaint,
        "tinta molhada na tela e a ferramenta Erase",
        vec![Gesto::Traco, Gesto::Clique(ferramenta_wet(1))],
        vec![Alvo::Id(PAINTER_WETPAINT_ERASE)],
        false,
    ));
    for (k, nome) in [(2, "a ferramenta Smear"), (3, "a ferramenta Blend")] {
        v.push((
            WetPaint,
            nome,
            vec![Gesto::Clique(ferramenta_wet(k))],
            vec![Alvo::Id(PAINTER_WETPAINT_PICKUP)],
            false,
        ));
    }
    v.push((
        WetPaint,
        "fábrica, Pickup medido DEPOIS do traço",
        Vec::new(),
        vec![
            Alvo::Id(PAINTER_WETPAINT_PICKUP),
            Alvo::Id(PAINTER_WETPAINT_ERASE),
        ],
        true,
    ));
    v
}

#[test]
#[ignore = "sonda: os candidatos a morto com a PRÉ-CONDIÇÃO armada"]
fn sonda_as_pre_condicoes() {
    let nomes = nomes();
    let so: Option<String> = std::env::var("CENSO_MEIO").ok();
    for (media, condicao, armar, alvos, depois) in pre_condicoes() {
        if so.as_ref().is_some_and(|m| *m != format!("{media:?}")) {
            continue;
        }
        let corre = |g: &Gesto| {
            if depois {
                ensaio_depois(media, &armar, g)
            } else {
                ensaio(media, &armar, g)
            }
        };
        let armada = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (corre(&Gesto::Nenhum).0, gestos(media, &armar))
        }));
        let Ok((base, oferta)) = armada else {
            eprintln!("PRE\t{media:?}\t{condicao}\t-\tPRÉ-CONDIÇÃO-FORA-DA-TELA");
            continue;
        };
        for alvo in alvos {
            let g = match alvo {
                Alvo::Gesto(g) => g,
                Alvo::Id(id) => match oferta.iter().find(|g| g.id() == Some(id)) {
                    Some(g) => g.clone(),
                    None => {
                        eprintln!(
                            "PRE\t{media:?}\t{condicao}\t{}\tNÃO-PINTADO",
                            nome(&nomes, id)
                        );
                        continue;
                    }
                },
            };
            let corrida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| corre(&g)));
            let Ok((px, entrega)) = corrida else {
                eprintln!("PRE\t{media:?}\t{condicao}\t{g:?}\tPÂNICO");
                continue;
            };
            let (n, soma, max) = diferenca(&base, &px);
            // Quanto cada imagem PINTOU (texels diferentes da tela de partida) — distingue «o
            // controlo muda o traço» de «o controlo apaga o traço».
            let (pintou_base, ..) = diferenca(&tela(), &base);
            let (pintou, ..) = diferenca(&tela(), &px);
            eprintln!(
                "PRE\t{media:?}\t{condicao}\t{}\t{}\tn={n}\tsoma={soma}\tmax={max}\tajuste={}\tdepois={depois}\tpintou={pintou_base}->{pintou}",
                nome(&nomes, g.id().unwrap_or(NodeId(0))),
                tipo_do_gesto(&g),
                if entrega.ajuste_mudou {
                    "mudou"
                } else {
                    "igual"
                },
            );
        }
    }
}

/// O RUÍDO do Wet Paint: a mesma base, oito vezes em paralelo, sem e com tinta já na tela. A água anda
/// ao relógio (`wetpaint/offthread.rs`), e um censo cujo «vivo» é ruído esconde um morto.
#[test]
#[ignore = "sonda: o piso de ruído do Wet Paint ao relógio real"]
fn sonda_o_ruido_do_wet_paint() {
    for armar in [Vec::new(), vec![Gesto::Traco]] {
        let corridas: Vec<Vec<u8>> = em_paralelo(&[0u8; 8], |_| {
            mede(PaintMedia::WetPaint, &armar, &Gesto::Nenhum).0
        });
        let piores: Vec<usize> = corridas[1..]
            .iter()
            .map(|c| diferenca(&corridas[0], c).0)
            .collect();
        eprintln!("RUIDO-WET armar={armar:?} texels diferentes da 1.ª: {piores:?}");
    }
}
