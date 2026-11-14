//! `PH2D_WETPAINT_SMOKE=3 PH2D_FPS_PRIMEIRO=<0..4>` — **o 1.º traço no app real** (Bug #72): escolhe o pincel, espera, dá
//! três traços iguais e imprime o período de cada quadro à volta de cada pen-down, com placa e apresentação dentro.
//! Tela: `PH2D_FPS_TELA` = `<lado>` (quadrada, a de sempre) ou `<w>x<h>` (B44.4: o 4K do produto é `3840x2160`),
//! lida por [`tela_do_fps`] — a mesma para esta cena e para a do `=3`.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::Instant;

const PINCEIS: [&str; 5] = [
    "Watercolor/Watercolor/Basic/Watercolor Basic.ph2d-brush.png",
    "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png",
    "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png",
    "WetPainter/Watercolor/Fluid/Wet Liner.ph2d-brush.png",
    "WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png",
];
/// Segundos do arranque ao pincel, e de cada traço: o tempo do artista, com o app ocioso entre um e outro.
const T_PINCEL: f64 = 4.0;
const T_TRACOS: [f64; 3] = [7.0, 10.0, 13.0];
const DUR: f64 = 1.0;
const HZ: f64 = 250.0;

#[derive(Default)]
struct Estado {
    t0: Option<Instant>,
    ultimo: Option<Instant>,
    com_pincel: bool,
    /// `(traço, eventos entregues)` do traço aberto.
    aberto: Option<(usize, usize)>,
    feitos: usize,
    /// Por quadro: `(período ms, ponteiro ms)`, à volta do pen-down.
    quadros: Vec<(f64, f64)>,
}

thread_local! {
    static ESTADO: std::cell::RefCell<Estado> = std::cell::RefCell::default();
}

/// O pincel pedido; `None` = a cena não está pedida.
pub fn pedido() -> Option<&'static str> {
    static I: std::sync::LazyLock<Option<usize>> = std::sync::LazyLock::new(|| {
        std::env::var("PH2D_FPS_PRIMEIRO")
            .ok()
            .and_then(|s| s.parse().ok())
    });
    PINCEIS.get((*I)?).copied()
}

/// A tela das cenas do FPS (`PH2D_FPS_TELA`): `[w, h]` em px; omissão `1024²`.
pub fn tela_do_fps() -> [u32; 2] {
    tela_de(std::env::var("PH2D_FPS_TELA").ok().as_deref())
}

/// `"<lado>"` → quadrada; `"<w>x<h>"` → retângulo; ausente ou ilegível → `1024²`.
fn tela_de(v: Option<&str>) -> [u32; 2] {
    let v = v.map(str::trim).filter(|s| !s.is_empty());
    let par = v.and_then(|s| match s.split_once('x') {
        Some((w, h)) => Some([w.parse().ok()?, h.parse().ok()?]),
        None => s.parse().ok().map(|l| [l, l]),
    });
    par.filter(|[w, h]: &[u32; 2]| *w > 0 && *h > 0)
        .unwrap_or([1024, 1024])
}

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um quadro da cena.
pub fn quadro(rel: &str, painter: &mut PainterTool) {
    ESTADO.with(|e| um_quadro(&mut e.borrow_mut(), rel, painter));
}

fn um_quadro(e: &mut Estado, rel: &str, painter: &mut PainterTool) {
    let agora = Instant::now();
    let periodo = e.ultimo.map_or(0.0, |u| (agora - u).as_secs_f64() * 1e3);
    e.ultimo = Some(agora);
    let t = (agora - *e.t0.get_or_insert(agora)).as_secs_f64();
    if !e.com_pincel && t >= T_PINCEL {
        e.com_pincel = true;
        let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/brushes")
            .join(rel);
        let preset = std::fs::read(f)
            .ok()
            .and_then(|b| ph2d_brush_presets::ler(&b).ok());
        match preset {
            Some(p) => {
                let _ = painter.preset_aplicar(&p);
                painter.set_brush_size_px(60.0);
                painter.set_brush_color_srgb8([40, 90, 220]);
            }
            None => eprintln!("[primeiro] preset {rel} não encontrado"),
        }
    }
    let [w, h] = tela_do_fps().map(|v| v as f32);
    let x = |k: usize| w * 0.1 + w * 0.8 * (k as f64 / (HZ * DUR)) as f32;
    let y = |n: usize| h * (0.25 + 0.25 * n as f32);
    let t_p = Instant::now();
    if e.aberto.is_none()
        && let Some(&inicio) = T_TRACOS.get(e.feitos)
        && t >= inicio
    {
        // Os quadros de antes do toque são a régua: o que o app faz ocioso.
        let n = e.quadros.len();
        e.quadros.drain(..n.saturating_sub(3));
        painter.on_canvas_pointer(ponto(x(0), y(e.feitos), PointerPhase::Down));
        e.aberto = Some((e.feitos, 0));
    } else if let Some((n, k)) = e.aberto {
        let inicio = T_TRACOS[n];
        let alvo = (((t - inicio) * HZ) as usize).min((DUR * HZ) as usize);
        for i in k + 1..=alvo {
            painter.on_canvas_pointer(ponto(x(i), y(n), PointerPhase::Move));
        }
        e.aberto = Some((n, alvo.max(k)));
        if t - inicio >= DUR {
            painter.on_canvas_pointer(ponto(x(alvo), y(n), PointerPhase::Up));
            e.aberto = None;
            e.feitos += 1;
            let mostra: Vec<String> = e
                .quadros
                .iter()
                .take(16)
                .map(|(p, q)| format!("{p:.1}/{q:.1}"))
                .collect();
            let mut todos: Vec<f64> = e.quadros.iter().map(|q| q.0).collect();
            todos.sort_by(f64::total_cmp);
            eprintln!(
                "[primeiro] traço {} | período/ponteiro ms (3 antes do toque, depois o toque e os seguintes): {} | máx {:.1} mediana {:.1}",
                n + 1,
                mostra.join(" "),
                todos.last().copied().unwrap_or(0.0),
                todos.get(todos.len() / 2).copied().unwrap_or(0.0),
            );
            e.quadros.clear();
        }
    }
    let ponteiro = t_p.elapsed().as_secs_f64() * 1e3;
    if e.feitos < T_TRACOS.len() {
        e.quadros.push((periodo, ponteiro));
    }
}

#[cfg(test)]
mod tests {
    /// B44.4: o `3840x2160` abre o retângulo do produto (antes caía no `1024²`), e o lado só continua quadrado.
    #[test]
    fn a_tela_do_fps_le_o_retangulo_e_o_lado_continua_quadrado() {
        assert_eq!(super::tela_de(Some("3840x2160")), [3840, 2160]);
        assert_eq!(super::tela_de(Some("2880")), [2880, 2880]);
        assert_eq!(super::tela_de(None), [1024, 1024]);
        assert_eq!(super::tela_de(Some("0x5")), [1024, 1024]);
        assert_eq!(super::tela_de(Some("lixo")), [1024, 1024]);
    }
}
