//! B29.1 na ferramenta: o traço de cor constante sob o tecto escolhe o [`ph2d_painter_brush::TexelLargo`] e o
//! Flow baixo chega ao alvo; o `u8` estanca. Medição: `docs/Painter/missao_4k/BACKLOG.md` B29.1.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_painter_brush::BrushSpec;

const L: u32 = 160;

fn cpx(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn ferramenta(ajusta: impl Fn(&mut BrushSpec)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.set_brush_size_px(24.0);
    ajusta(&mut t.paint.brush);
    for s in &mut t.paint.brush_by_mode {
        ajusta(s);
    }
    t
}

/// Preto sobre branco, Strength 1, Flow 2 %, Accumulate OFF, a cápsula (Continuous): o carimbo por dab.
fn flow_baixo(b: &mut BrushSpec) {
    b.color = [0.0, 0.0, 0.0];
    b.strength = 1.0;
    b.flow = 0.02;
    b.hardness = 1.0;
    b.accumulate = false;
    b.continuous_spacing = true;
}

/// `n` passadas de ida e volta num traço só; devolve o R do texel do meio e a escolha do tecto.
fn passadas(t: &mut PainterTool, n: usize) -> (u8, Option<bool>) {
    let (a, b, y) = (40.0, 120.0, 80.0);
    t.on_canvas_pointer(cpx([a, y], PointerPhase::Down));
    for i in 0..n {
        let x = if i % 2 == 0 { b } else { a };
        t.on_canvas_pointer(cpx([x, y], PointerPhase::Move));
    }
    let escolha = t.paint.tecto_largo.escolha;
    t.on_canvas_pointer(cpx([a, y], PointerPhase::Up));
    let i = ((80 * L + 80) * 4) as usize;
    (t.canvas_rgba[i], escolha)
}

/// ⭐ **O Flow 2 % chega ao tecto com o tecto largo** — e, no MESMO teste, o `u8` (o veto de teste) estanca.
/// Controlo: com 2 passadas nada estancou ainda, e os dois tectos dão a mesma tinta a ≤ 1 nível (não é um
/// largo que pinta opaco de uma vez).
#[test]
fn com_o_tecto_largo_o_flow_baixo_chega_ao_alvo_na_ferramenta() {
    let corre = |n: usize, so_byte: bool| {
        let mut t = ferramenta(flow_baixo);
        t.paint.tecto_largo.so_byte = so_byte;
        passadas(&mut t, n)
    };
    let ((l2, _), (b2, _)) = (corre(2, false), corre(2, true));
    let ((v_largo, e_largo), (v_byte, e_byte)) = (corre(80, false), corre(80, true));
    // O alvo: Strength 1, preto ⇒ o tecto é 0; 80 passadas a 2 % chegam-lhe em aritmética exacta.
    let leitura = format!(
        "2 passadas: largo {l2} · u8 {b2} | 80: largo {v_largo} ({e_largo:?}) · u8 {v_byte} ({e_byte:?})"
    );
    eprintln!("B29.1 ferramenta: {leitura}");
    assert_eq!(
        e_largo,
        Some(true),
        "a fixture TEM de escolher o tecto largo: {leitura}"
    );
    assert_eq!(
        e_byte,
        Some(false),
        "o veto de teste TEM de forçar o u8: {leitura}"
    );
    assert!(
        l2 > 100 && l2.abs_diff(b2) <= 1,
        "controlo: antes de estancar os dois tectos coincidem: {leitura}"
    );
    assert!(
        v_largo <= 1,
        "o tecto largo devia chegar ao alvo (0) a ≤ 1 nível: {leitura}"
    );
    assert!(
        v_byte > 10,
        "o u8 não estancou: a fixture não contém o fenómeno: {leitura}"
    );
}

/// O pincel LISO (sem a cápsula: a rota em banda, `stamp_plain_dabs_banded`) também escolhe o largo e chega ao alvo;
/// no mesmo teste o `u8` (o veto) estanca.
#[test]
fn o_pincel_liso_da_banda_leva_o_tecto_largo() {
    let liso = |b: &mut BrushSpec| {
        flow_baixo(b);
        b.continuous_spacing = false;
    };
    let corre = |so_byte: bool| {
        let mut t = ferramenta(liso);
        t.paint.tecto_largo.so_byte = so_byte;
        passadas(&mut t, 80)
    };
    let ((v_largo, e_largo), (v_byte, e_byte)) = (corre(false), corre(true));
    let leitura = format!("largo {v_largo} ({e_largo:?}) · u8 {v_byte} ({e_byte:?})");
    eprintln!("B29.1 banda na ferramenta: {leitura}");
    assert_eq!((e_largo, e_byte), (Some(true), Some(false)), "{leitura}");
    assert!(v_largo <= 1, "o largo devia chegar ao alvo: {leitura}");
    assert!(v_byte > 10, "o u8 não estancou: {leitura}");
}

/// Um traço que NÃO pode recompor (Randomize Color: a cor muda por dab) fica no `u8`, ao bit com o veto.
#[test]
fn a_cor_que_muda_por_dab_fica_no_tecto_u8() {
    let jitter = |b: &mut BrushSpec| {
        flow_baixo(b);
        b.color = [0.2, 0.4, 0.8];
        b.color_jitter_enabled = true;
        b.color_jitter_hue = 0.3;
    };
    let mut t = ferramenta(jitter);
    let (_, e) = passadas(&mut t, 6);
    let mut v = ferramenta(jitter);
    v.paint.tecto_largo.so_byte = true;
    let _ = passadas(&mut v, 6);
    assert_eq!(
        e,
        Some(false),
        "a cor que muda por dab não recompõe: o tecto é o u8"
    );
    assert!(
        t.canvas_rgba == v.canvas_rgba,
        "o u8 escolhido devia ser o do veto, ao bit"
    );
}

/// Sonda de custo (B29.1): o MESMO traço a 4K com o tecto `u8` e o largo, intercalados, o mínimo de 7.
/// `PH2D_RELEASE` não: corre em `--profile smoke` (`cargo test --profile smoke … -- --ignored --nocapture`).
#[test]
#[ignore = "sonda de custo: corre à mão"]
fn sonda_b29_1_traco_4k_largo_x_u8() {
    let (w, h) = (3840u32, 2160u32);
    let mut melhor = [f64::MAX; 2];
    for volta in 0..7 {
        for k in [volta % 2, 1 - volta % 2] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
            t.set_brush_size_px(120.0);
            flow_baixo(&mut t.paint.brush);
            for s in &mut t.paint.brush_by_mode {
                flow_baixo(s);
            }
            t.paint.tecto_largo.so_byte = k == 0;
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cpx([200.0, 1080.0], PointerPhase::Down));
            for i in 0..40 {
                let x = if i % 2 == 0 { 3600.0 } else { 200.0 };
                t.on_canvas_pointer(cpx([x, 1080.0 + i as f32 * 4.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cpx([200.0, 1240.0], PointerPhase::Up));
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            assert_eq!(t.paint.tecto_largo.escolha, Some(k == 1));
            melhor[k] = melhor[k].min(ms);
        }
    }
    eprintln!(
        "B29.1 traço 4K (40 passadas, r60, Flow 2 %): u8 {:.1} ms · largo {:.1} ms (mínimo de 7)",
        melhor[0], melhor[1]
    );
}

thread_local! {
    /// Veto de TESTE para quem não alcança o `PainterTool` (a miniatura cria o seu): o `u8` em todo o traço.
    pub(super) static VETO_DA_SONDA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Sonda (B29.1): cada miniatura de fábrica com o tecto largo × o `u8` (o veto), no MESMO binário — a lista
/// exacta dos pincéis que mudam pelo largo, com a tinta (Σ 255 − média RGB) e o alfa de cada lado.
#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_b29_1_miniaturas_largo_x_u8() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let tinta = |px: &[u8]| -> u64 {
        px.chunks(4)
            .map(|c| 765 - (u64::from(c[0]) + u64::from(c[1]) + u64::from(c[2])))
            .sum()
    };
    let mut n = 0;
    for e in &bib.entradas {
        VETO_DA_SONDA.with(|v| v.set(false));
        let largo = crate::miniatura_do_preset(&e.preset).rgba;
        VETO_DA_SONDA.with(|v| v.set(true));
        let byte = crate::miniatura_do_preset(&e.preset).rgba;
        VETO_DA_SONDA.with(|v| v.set(false));
        if largo == byte {
            continue;
        }
        n += 1;
        let px = largo
            .chunks(4)
            .zip(byte.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        let dmax = largo
            .iter()
            .zip(&byte)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        let (tl, tb) = (tinta(&largo), tinta(&byte));
        eprintln!(
            "MUDA {}: {px} px · Δmax {dmax} · tinta largo {tl} × u8 {tb} ({:+.1} %)",
            e.relativo,
            (tl as f64 / tb.max(1) as f64 - 1.0) * 100.0
        );
    }
    eprintln!(
        "B29.1: {n} de {} miniaturas mudam pelo tecto largo",
        bib.entradas.len()
    );
}
