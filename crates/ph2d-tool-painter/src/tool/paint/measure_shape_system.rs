//! **O que a MÁQUINA DE SHAPE custa quando a tinta sai da conta** — a pergunta do Enio de 2026-08-06:
//! *"descubra o custo do sistema mesmo que a pintura não esteja envolvida"*.
//!
//! Todas as sondas anteriores (`measure_shape_cost`) medem a porta do EVENTO inteira, e o evento
//! contém o depósito. A separação é possível **sem uma linha de ablação nova**, porque o
//! `stamp_drag_preview` já cronometra as suas quatro fases **no código que shipa**
//! (`stamp_banded::diag::note_restamp`, o precedente do split do composite da água). Então:
//!
//! ```text
//! sistema = evento − carimbo
//! geometria = evento − (restore + relevo + save + carimbo)
//! ```
//!
//! A `geometria` sai por **subtração**, e isso é honesto para uma atribuição de 1º nível: ela é tudo o
//! que acontece antes de o `stamp_drag_preview` ser chamado — o `capture_shape`, o `clone` do conjunto
//! parqueado, o offset/flatten/trim da espinha e o `fill_*_preview` que constrói a lista de dabs.
//!
//! ⚠️ **O `stamp_us` NÃO é *"a tinta"* inteira; é o que o carimbo cobra dentro do evento.** O composite
//! da pilha é do QUADRO, e o `what_a_frame_of_a_live_shape_costs_not_just_the_event` (irmão em
//! `measure_shape_cost`) é quem o mede. Aqui a fronteira é a porta do evento, e ela está declarada.
//!
//! Rodar:
//! `cargo test -p ph2d-tool-painter --release measure_shape_system -- --ignored --nocapture --test-threads=1`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::StrokeMethod;

use super::media::PaintMedia;
use super::stamp_banded::diag;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// ⚠️ **`set_brush_size_px` escreve `brush.radius_px` — ele toma o RAIO, não o diâmetro.**
///
/// A sonda irmã (`measure_shape_cost`) chama `set_brush_size_px(radius * 2.0)` e rotula a coluna
/// `r=24`: o pincel que ela de fato roda tem raio **48**, e a varredura que o doc dela apresenta como
/// *"o ponto do log de 05/08, pincel r~185"* roda com raio **370** — o dobro do raio, **quatro vezes**
/// a área por dab. O veredito de RAZÃO daquela tabela (Impasto ÷ Digital) sobrevive, porque os dois
/// lados pagam a mesma duplicação; os ABSOLUTOS dela descrevem outro pincel. Aqui o número é passado
/// como é: `radius` é o raio.
fn tool(side: u32, media: PaintMedia, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_paint_media(media);
    t.set_brush_size_px(radius);
    t
}

/// Um move: o tempo TOTAL do evento e as quatro fases que o produto já publica, em ms.
#[derive(Clone, Copy, Default)]
struct Phases {
    total: f64,
    restore: f64,
    relief: f64,
    save: f64,
    stamp: f64,
    dabs: f64,
    /// Lotes que tomaram a rota em BANDA (paralela) e os que ficaram SERIAIS.
    banded: f64,
    serial: f64,
    /// Visitas de texel do lote — o TRABALHO, sem assumir raio nenhum.
    visits: f64,
    /// Quantas ENTREGAS o `stamp_drag_preview` reportou neste evento.
    ///
    /// ⚠️ **Zero não quer dizer *"de graça"*, quer dizer *"NÃO MEDIDO"*.** A Aquarela entra pela porta
    /// própria (`stamp_drag_preview_watercolor`, doc 13 #3), que **não chama** o `note_restamp` — então
    /// as quatro fases vêm todas em zero e a subtração joga o evento inteiro na coluna `geom`, que
    /// leria como *"73 ms de geometria"*. Um instrumento mudo lê-se como resultado; esta coluna é o
    /// que impede a tabela de afirmar o que ela não mediu.
    deliveries: f64,
}

impl Phases {
    /// Tudo o que não é o carimbo — a máquina de shape.
    fn system(self) -> f64 {
        (self.total - self.stamp).max(0.0)
    }
    /// O que roda ANTES do `stamp_drag_preview`: captura, clone, offset, flatten, trim, fill.
    fn geometry(self) -> f64 {
        (self.total - self.restore - self.relief - self.save - self.stamp).max(0.0)
    }
}

fn median_of(mut v: Vec<Phases>) -> Phases {
    if v.is_empty() {
        return Phases::default();
    }
    // Mediana por CAMPO: cada coluna é uma grandeza própria, e a amostra que é mediana no total não
    // é necessariamente a mediana no save. Somar as medianas não reconstrói o total exato — a
    // diferença é o ruído, e é ela que a coluna `geometria` absorve.
    let mut out = Phases::default();
    for (get, put) in [
        (
            (|p: &Phases| p.total) as fn(&Phases) -> f64,
            (|o: &mut Phases, x: f64| o.total = x) as fn(&mut Phases, f64),
        ),
        (|p| p.restore, |o, x| o.restore = x),
        (|p| p.relief, |o, x| o.relief = x),
        (|p| p.save, |o, x| o.save = x),
        (|p| p.stamp, |o, x| o.stamp = x),
        (|p| p.dabs, |o, x| o.dabs = x),
        (|p| p.banded, |o, x| o.banded = x),
        (|p| p.serial, |o, x| o.serial = x),
        (|p| p.visits, |o, x| o.visits = x),
        (|p| p.deliveries, |o, x| o.deliveries = x),
    ] {
        let mut c: Vec<f64> = v.iter().map(&get).collect();
        c.sort_by(|a, b| a.partial_cmp(b).unwrap());
        put(&mut out, c[c.len() / 2]);
    }
    v.clear();
    out
}

/// Roda um gesto de shape e devolve a mediana das fases de um MOVE.
///
/// `grow`: a figura CRESCE a cada move (o gesto de criação). `false` oscila em torno de `size`, que é
/// o gesto de ajuste — a figura fica do mesmo tamanho e o que se mede é re-carimbá-la.
fn measure(
    side: u32,
    media: PaintMedia,
    radius: f32,
    method: StrokeMethod,
    size: f32,
    parked: usize,
) -> Phases {
    let mut t = tool(side, media, radius);
    t.paint.brush.stroke_method = method;
    #[allow(clippy::cast_precision_loss)]
    let cx = (side / 2) as f32;

    // Formas PARQUEADAS: cada uma é uma elipse fechada e estacionada, e o re-stamp as re-constrói
    // TODAS a cada move (`restamp_shapes_preview` clona o conjunto e refaz a geometria de cada uma).
    //
    // ⚠️ **Elas ficam LONGE da ativa, e é isso que a fixture está medindo.** Com elas sobrepondo a
    // figura ativa, o custo delas seria trabalho HONESTO (o restore da pegada ativa apaga a tinta
    // parqueada na interseção, então ela precisa mesmo ser re-carimbada). O desperdício só é
    // observável quando a forma parada não toca em nada do que se move — aí re-carimbá-la é refazer
    // trabalho cujo resultado é byte-idêntico ao que já está na tela.
    for k in 0..parked {
        #[allow(clippy::cast_precision_loss)]
        let px = 200.0 + (k as f32) * 140.0;
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([px, 200.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([px + 60.0, 200.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([px + 60.0, 200.0], PointerPhase::Up));
        t.park_active_shape();
    }
    t.paint.brush.stroke_method = method;

    // ⚠️ O editor de **Line** é uma POLILINHA: o 1º Down cria UM ponto e o agarra, e um ponto não
    // desenha nada. Sem o 2º ponto a linha mede zero e leria como *"Line é de graça"*.
    if method == StrokeMethod::Line {
        t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Up));
    }
    t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));

    let _ = diag::take(); // zera o que o Down deixou
    let mut samples = Vec::new();
    for k in 0..9 {
        let d = if k % 2 == 0 { size + 2.0 } else { size - 2.0 };
        let e = cp([cx + d, cx], PointerPhase::Move);
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(e);
        let total = t0.elapsed().as_secs_f64() * 1e3;
        let d = diag::take();
        if k > 0 {
            // O 1º move aloca (o `save_region` nasce com o buffer da figura) e é descartado.
            samples.push(Phases {
                total,
                restore: d.restore_us as f64 / 1e3,
                relief: d.relief_us as f64 / 1e3,
                save: d.save_us as f64 / 1e3,
                stamp: d.stamp_us as f64 / 1e3,
                dabs: f64::from(d.dabs + d.dev_dabs),
                banded: f64::from(d.banded),
                serial: f64::from(d.serial),
                visits: d.visits as f64 + d.dev_visits as f64,
                deliveries: f64::from(d.deliveries),
            });
        }
    }
    t.on_canvas_pointer(cp([cx + size, cx], PointerPhase::Up));
    median_of(samples)
}

fn header() {
    println!(
        "{:>9} {:>7} {:>7}  {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}  {:>6} {:>6} {:>10} {:>8}",
        "método",
        "tela",
        "figura",
        "EVENTO",
        "geom",
        "restore",
        "relevo",
        "save",
        "carimbo",
        "sist%",
        "rota",
        "visitas",
        "ns/vis"
    );
}

fn row(name: &str, side: u32, size: f32, p: Phases) {
    // ⚠️ A ROTA é o que distingue *"o carimbo é caro"* de *"o carimbo caiu na estrada serial"* — duas
    // leituras com curas opostas, e o log de smoke já mostrou que sem esta coluna a atribuição erra.
    let road = if p.banded > 0.0 { "banda" } else { "serial" };
    // ⚠️ Sem entrega reportada as quatro fases são DESCONHECIDAS, não zero — a linha diz isso em vez
    // de deixar a subtração inventar uma coluna de geometria.
    if p.deliveries == 0.0 {
        println!(
            "{name:>9} {side:>7} {size:>7.0}  {:>8.3} {:>8} {:>8} {:>8} {:>8} {:>8}  {:>6} {:>6} {:>10} {:>8}",
            p.total, "?", "?", "?", "?", "?", "n/medido", road, "?", "?"
        );
        return;
    }
    println!(
        "{:>9} {:>7} {:>7.0}  {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3}  {:>5.1}% {:>6} {:>10.0} {:>8.2}",
        name,
        side,
        size,
        p.total,
        p.geometry(),
        p.restore,
        p.relief,
        p.save,
        p.stamp,
        100.0 * p.system() / p.total.max(1e-9),
        road,
        p.visits,
        p.stamp * 1e6 / p.visits.max(1.0)
    );
}

/// **A tabela-mãe: quanto de um move de shape NÃO é o depósito**, por método.
///
/// O oráculo é a coluna `sist%` — a fração do evento que a máquina de shape cobra por conta própria.
/// Se ela for pequena, otimizar o depósito é otimizar a figura viva; se for grande, o módulo tem uma
/// frente que nenhuma wave de tinta alcança.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_shape_system_by_method() {
    println!("[shape-sys] o que um MOVE cobra fora do depósito — Digital, RAIO 48, figura 400 px");
    header();
    for side in [2048u32, 4096] {
        for (name, m) in [
            ("Line", StrokeMethod::Line),
            ("Ellipse", StrokeMethod::Ellipse),
            ("Polygon", StrokeMethod::Polygon),
            ("Curve", StrokeMethod::Arc),
            ("FreeHand", StrokeMethod::FreeHand),
        ] {
            let p = measure(side, PaintMedia::Digital, 48.0, m, 400.0, 0);
            row(name, side, 400.0, p);
        }
    }
}

/// **Como a máquina escala com o TAMANHO da figura** — a assinatura de cada fase.
///
/// `restore`/`save` copiam a bbox da figura, então crescem com a ÁREA dela; a `geometria` cresce com
/// o PERÍMETRO (a espinha achatada e a lista de dabs). Separá-las diz qual das duas curas vale.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_shape_system_against_figure_size() {
    println!("[shape-sys] contra o TAMANHO da figura — Ellipse, Digital, 4096, raio 48");
    header();
    for size in [100.0f32, 200.0, 400.0, 800.0, 1600.0] {
        let p = measure(
            4096,
            PaintMedia::Digital,
            48.0,
            StrokeMethod::Ellipse,
            size,
            0,
        );
        row("Ellipse", 4096, size, p);
    }
}

/// **O custo de N formas simultâneas** — o `parked_shapes.clone()` + a re-construção de cada uma.
///
/// O `restamp_shapes_preview` clona o conjunto parqueado inteiro a cada move e refaz a geometria de
/// TODA forma, ativa ou não. Se a coluna `geom` crescer linearmente com N, é isso que ela mede.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_shape_system_with_parked_shapes() {
    println!(
        "[shape-sys] contra o NÚMERO de formas na tela — Ellipse ativa, Digital, 4096, raio 48"
    );
    header();
    for n in [0usize, 1, 2, 4, 8] {
        let p = measure(
            4096,
            PaintMedia::Digital,
            48.0,
            StrokeMethod::Ellipse,
            400.0,
            n,
        );
        row(&format!("+{n} park"), 4096, 400.0, p);
    }
}

/// **O MESMO lote, DOIS lugares** — a experiência que separa *depósito redundante* de *bandas vazias*.
///
/// Com N formas parqueadas o move fica 7,4× mais caro, e há duas causas possíveis com curas OPOSTAS:
/// **(a)** as formas paradas são re-carimbadas (trabalho a mais — a cura é não refazer) · **(b)** a
/// união dos retângulos vira uma caixa quase do tamanho da tela e o `stamp_banded` divide a ALTURA
/// dela em bandas iguais, deixando a maioria **vazia** (o mesmo trabalho, mal distribuído — a cura é
/// dividir por trabalho).
///
/// A fixture põe as MESMAS formas parqueadas em dois lugares: **coladas** na ativa (união apertada) e
/// no **canto oposto** (união quase da tela). Mesma contagem de dabs, mesmas visitas, mesma tinta —
/// só a esparsidade muda. Se o custo subir, a diferença é **(b)**, e o que sobrar é **(a)**.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_sparse_batch_penalty() {
    println!("[shape-sys] o MESMO lote, COLADO x ESPALHADO — Ellipse ativa r=400, Digital, 4096");
    println!(
        "{:>10} {:>7}  {:>9} {:>9} {:>9} {:>9}  {:>10} {:>8}",
        "posição", "parked", "EVENTO", "restore", "save", "carimbo", "visitas", "ns/vis"
    );
    for parked in [0usize, 2, 8] {
        for (name, far) in [("colado", false), ("espalhado", true)] {
            if parked == 0 && far {
                continue; // sem formas paradas as duas colunas são a MESMA cena
            }
            let side = 4096u32;
            let mut t = tool(side, PaintMedia::Digital, 48.0);
            let cx = 2048.0f32;
            for k in 0..parked {
                #[allow(clippy::cast_precision_loss)]
                let kk = k as f32;
                // COLADO: logo ao lado da ativa (que vai de cx-400 a cx+400).
                // ESPALHADO: no canto, longe de tudo. Formas IDÊNTICAS nos dois casos.
                let p = if far {
                    [200.0 + kk * 140.0, 200.0]
                } else {
                    [cx - 380.0 + kk * 140.0, cx + 300.0]
                };
                t.paint.brush.stroke_method = StrokeMethod::Ellipse;
                t.on_canvas_pointer(cp(p, PointerPhase::Down));
                t.on_canvas_pointer(cp([p[0] + 60.0, p[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp([p[0] + 60.0, p[1]], PointerPhase::Up));
                t.park_active_shape();
            }
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
            let _ = diag::take();
            let mut s = Vec::new();
            for k in 0..9 {
                let d = if k % 2 == 0 { 402.0 } else { 398.0 };
                let e = cp([cx + d, cx], PointerPhase::Move);
                let t0 = std::time::Instant::now();
                t.on_canvas_pointer(e);
                let total = t0.elapsed().as_secs_f64() * 1e3;
                let g = diag::take();
                if k > 0 {
                    s.push(Phases {
                        total,
                        restore: g.restore_us as f64 / 1e3,
                        relief: 0.0,
                        save: g.save_us as f64 / 1e3,
                        stamp: g.stamp_us as f64 / 1e3,
                        dabs: f64::from(g.dabs + g.dev_dabs),
                        banded: f64::from(g.banded),
                        serial: f64::from(g.serial),
                        visits: g.visits as f64 + g.dev_visits as f64,
                        deliveries: f64::from(g.deliveries),
                    });
                }
            }
            t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Up));
            let p = median_of(s);
            println!(
                "{name:>10} {parked:>7}  {:>9.3} {:>9.3} {:>9.3} {:>9.3}  {:>10.0} {:>8.2}",
                p.total,
                p.restore,
                p.save,
                p.stamp,
                p.visits,
                p.stamp * 1e6 / p.visits.max(1.0)
            );
        }
    }
}

/// **O QUADRO OCIOSO: o que uma figura viva cobra quando o artista NÃO faz nada.**
///
/// Esta é a pergunta na sua forma mais pura — *"o custo do sistema mesmo que a pintura não esteja
/// envolvida"*. Nenhum evento de ponteiro acontece aqui: só o que o shell pergunta ao tool **toda vez
/// que desenha um quadro**, para pintar o chrome da figura (âncoras, alças, gizmo, guias, badges).
///
/// ⚠️ **O `stroke_op_badges` é o suspeito nomeado:** ele chama `shape_state_bbox`, que chama
/// `parked_shape_dabs` — ou seja, **constrói a lista de dabs INTEIRA de cada forma parqueada** (offset,
/// flatten, trim, `fill_*_preview`) e joga tudo fora menos um min/max de quatro floats.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_idle_frame_of_a_live_shape() {
    println!(
        "[shape-sys] o que um quadro OCIOSO pergunta ao tool (µs) — 4096, raio 48, figura 400 px"
    );
    println!(
        "{:>26}  {:>10} {:>10}  {:>9}",
        "cena", "overlay", "badges", "TOTAL"
    );

    // Uma medição de UMA chamada é ruído; cada célula roda `N` vezes e reporta a média.
    const N: u32 = 200;
    let run = |label: String, t: &PainterTool| {
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            std::hint::black_box(t.curve_overlay());
            std::hint::black_box(t.ellipse_overlay());
            std::hint::black_box(t.polygon_overlay());
            std::hint::black_box(t.line_overlay());
        }
        let ov = t0.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        let t1 = std::time::Instant::now();
        for _ in 0..N {
            std::hint::black_box(t.stroke_op_badges());
        }
        let bg = t1.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!("{label:>26}  {ov:>10.1} {bg:>10.1}  {:>9.1}", ov + bg);
    };

    // (a) Uma figura ATIVA de cada tipo, nada parqueado.
    for (name, m) in [
        ("Line", StrokeMethod::Line),
        ("Ellipse", StrokeMethod::Ellipse),
        ("Polygon", StrokeMethod::Polygon),
        ("Curve", StrokeMethod::Arc),
    ] {
        let mut t = tool(4096, PaintMedia::Digital, 48.0);
        t.paint.brush.stroke_method = m;
        let cx = 2048.0f32;
        if m == StrokeMethod::Line {
            t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Up));
        }
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Move));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Up));
        run(format!("{name} ativa"), &t);
    }

    // (b) N formas PARQUEADAS + uma ativa — o eixo que o `stroke_op_badges` percorre.
    for n in [1usize, 2, 4, 8, 16] {
        let mut t = tool(4096, PaintMedia::Digital, 48.0);
        let cx = 2048.0f32;
        for k in 0..n {
            #[allow(clippy::cast_precision_loss)]
            let dx = -900.0 + (k as f32) * 60.0;
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx + dx, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx + dx + 200.0, cx], PointerPhase::Move));
            t.on_canvas_pointer(cp([cx + dx + 200.0, cx], PointerPhase::Up));
            t.park_active_shape();
        }
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Move));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Up));
        run(format!("{n} parqueada(s) + ativa"), &t);
    }

    // (c) Uma CURVA densa — o caso do Free Hand, cujo ajuste deixa dezenas a centenas de âncoras, e
    // cujo overlay re-achata a espinha inteira a cada quadro.
    for pts in [8usize, 32, 128, 512] {
        let mut t = tool(4096, PaintMedia::Digital, 48.0);
        t.paint.brush.stroke_method = StrokeMethod::Arc;
        let cx = 2048.0f32;
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Move));
        t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Up));
        if let Some(ed) = t.paint.curve.as_mut() {
            ed.model.points.clear();
            ed.model.handles.clear();
            ed.model.kinds.clear();
            for i in 0..pts {
                #[allow(clippy::cast_precision_loss)]
                let a = (i as f32) * 0.11;
                let p = [cx + a * 30.0, cx + (a * 3.0).rem_euclid(2.0) * 120.0];
                ed.model.points.push(p);
                ed.model
                    .handles
                    .push([[p[0] - 12.0, p[1]], [p[0] + 12.0, p[1]]]);
                ed.model.kinds.push(super::curve_handle::HandleKind::Free);
            }
            ed.model.selected = Some(0);
        }
        run(format!("Curve com {pts} âncoras"), &t);
    }
}

/// **O DESPERDÍCIO da pegada: a bbox contra o que os dabs de fato tocam.**
///
/// O `stamp_drag_preview` salva e restaura **UM retângulo** — a bbox da união dos dabs. Mas a tinta de
/// uma figura fechada é um **ANEL**: a área pintada cresce com o PERÍMETRO × a largura do pincel,
/// enquanto a bbox cresce com o QUADRADO do raio. O miolo vazio é copiado duas vezes por move, para
/// nada.
///
/// Esta sonda não cronometra — ela conta pixels, que é uma propriedade dos DADOS e não do código
/// (um laço próprio aqui não fica cego a porta nenhuma: os dabs saem do produto, via `t.paint.dabs`).
/// A razão é o **TETO** de qualquer cura de pegada; ela diz se vale construir uma.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_shape_footprint_waste() {
    println!("[shape-sys] a BBOX que se copia contra o que a tinta TOCA — 4096, raio 48");
    println!(
        "{:>9} {:>7}  {:>12} {:>12} {:>9}  {:>7}",
        "método", "figura", "bbox px", "tocado px", "desperd.", "dabs"
    );
    for (name, m) in [
        ("Line", StrokeMethod::Line),
        ("Ellipse", StrokeMethod::Ellipse),
        ("Polygon", StrokeMethod::Polygon),
        ("Curve", StrokeMethod::Arc),
    ] {
        for size in [200.0f32, 400.0, 800.0, 1600.0] {
            let side = 4096u32;
            let radius = 48.0f32;
            let mut t = tool(side, PaintMedia::Digital, radius);
            t.paint.brush.stroke_method = m;
            #[allow(clippy::cast_precision_loss)]
            let cx = (side / 2) as f32;
            if m == StrokeMethod::Line {
                t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Down));
                t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Up));
            }
            t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx + size, cx], PointerPhase::Move));

            let dabs = t.paint.dabs.clone();
            if dabs.is_empty() {
                println!(
                    "{name:>9} {size:>7.0}  {:>12} {:>12} {:>9} {:>7}",
                    "-", "-", "-", 0
                );
                continue;
            }
            // A bbox que o produto salva: a união dos retângulos de dab.
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for d in &dabs {
                x0 = x0.min(d.center[0] - d.radius_px);
                y0 = y0.min(d.center[1] - d.radius_px);
                x1 = x1.max(d.center[0] + d.radius_px);
                y1 = y1.max(d.center[1] + d.radius_px);
            }
            let bbox = f64::from((x1 - x0).max(0.0)) * f64::from((y1 - y0).max(0.0));

            // O que a tinta TOCA: marcação por TILE de 64 px, que é a granularidade em que uma cura de
            // pegada trabalharia (copiar por tile, não por texel — copiar por texel seria um teto que
            // nenhuma implementação alcança, e um teto inalcançável não decide nada).
            const TILE: f32 = 64.0;
            let mut tiles = std::collections::BTreeSet::new();
            for d in &dabs {
                let (tx0, ty0) = (
                    ((d.center[0] - d.radius_px) / TILE).floor() as i32,
                    ((d.center[1] - d.radius_px) / TILE).floor() as i32,
                );
                let (tx1, ty1) = (
                    ((d.center[0] + d.radius_px) / TILE).ceil() as i32,
                    ((d.center[1] + d.radius_px) / TILE).ceil() as i32,
                );
                for ty in ty0..ty1 {
                    for tx in tx0..tx1 {
                        tiles.insert((tx, ty));
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let touched = tiles.len() as f64 * f64::from(TILE) * f64::from(TILE);
            t.on_canvas_pointer(cp([cx + size, cx], PointerPhase::Up));
            println!(
                "{name:>9} {size:>7.0}  {bbox:>12.0} {touched:>12.0} {:>8.1}x  {:>7}",
                bbox / touched.max(1.0),
                dabs.len()
            );
        }
    }
}

/// **A máquina de shape contra o MEIO** — ela é a mesma nos quatro?
///
/// A decomposição por meio (`what_a_shape_move_is_made_of`) mostrou o Impasto custando 19× o Digital,
/// e a wave de 06/08 atribuiu isso ao depósito de ALTURA. Se a coluna `sist%` for parecida nos quatro,
/// a máquina é neutra ao meio e a diferença é toda do carimbo — o que fecha a atribuição.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_shape_system_by_medium() {
    println!("[shape-sys] a máquina contra o MEIO — Ellipse 400 px, 4096, raio 96");
    header();
    for (name, media) in [
        ("Digital", PaintMedia::Digital),
        ("Impasto", PaintMedia::Impasto),
        ("Aquarela", PaintMedia::Watercolor),
    ] {
        let p = measure(4096, media, 96.0, StrokeMethod::Ellipse, 400.0, 0);
        row(name, 4096, 400.0, p);
    }
}
