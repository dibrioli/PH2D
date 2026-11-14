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
    // A lei do largo medida sem a porta do meio (só o Wet Paint a adota, BUGS_painter #115): ver
    // `fora_do_wet_paint_o_traco_fica_no_tecto_u8` para a porta.
    t.paint.tecto_largo.sem_a_porta_do_meio = true;
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

/// ⛔ **A porta do meio** (BUGS_painter #115; dono 07/10 «só o Wet Paint evolui»): no Digital, no Impasto e na
/// Watercolor o traço que o largo levaria fica no `u8`, ao bit do veto — os bytes do main 4f46a3573c. Controlo: o
/// MESMO traço sem a porta escolhe o largo e pinta outros bytes (a fixture contém o fenómeno).
#[test]
fn fora_do_wet_paint_o_traco_fica_no_tecto_u8() {
    use crate::PaintMedia;
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::Watercolor,
    ] {
        let corre = |sem_porta: bool, so_byte: bool| {
            let mut t = ferramenta(flow_baixo);
            t.set_paint_media(meio);
            t.paint.tecto_largo.sem_a_porta_do_meio = sem_porta;
            t.paint.tecto_largo.so_byte = so_byte;
            let (_, e) = passadas(&mut t, 80);
            (e, t.canvas_rgba)
        };
        let (e, px) = corre(false, false);
        let (_, veto) = corre(false, true);
        assert_ne!(
            e,
            Some(true),
            "{meio:?}: fora do Wet o traço não leva o largo"
        );
        assert!(px == veto, "{meio:?}: fora do Wet o traço é o u8 ao bit");
        if meio == PaintMedia::Digital {
            let (e_sem, px_sem) = corre(true, false);
            assert_eq!(
                e_sem,
                Some(true),
                "controlo: sem a porta o Digital leva o largo"
            );
            assert!(px_sem != veto, "controlo: o largo muda bytes nesta fixture");
        }
    }
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

/// O traço no PLANO do Digital (a cauda e um modo de cor, `traco_plano`) também leva o largo e chega ao alvo; a
/// cauda limpa e repõe o largo como o `u8`. No mesmo teste o `u8` (o veto) estanca.
#[test]
fn no_plano_do_digital_o_flow_baixo_chega_ao_alvo() {
    use ph2d_painter_brush::BrushBlend;
    for (nome, blend, cauda) in [
        ("cauda", BrushBlend::Mix, 3.0),
        ("Multiply", BrushBlend::Multiply, 0.0),
    ] {
        let no_plano = |b: &mut BrushSpec| {
            flow_baixo(b);
            b.blend = blend;
            b.taper.end = cauda;
        };
        let corre = |so_byte: bool| {
            let mut t = ferramenta(no_plano);
            t.set_paint_media(crate::PaintMedia::Digital);
            t.paint.tecto_largo.so_byte = so_byte;
            assert!(
                t.traco_no_plano(),
                "{nome}: a fixture TEM de pintar no plano"
            );
            passadas(&mut t, 80)
        };
        let ((v_largo, e_largo), (v_byte, e_byte)) = (corre(false), corre(true));
        let leitura = format!("{nome}: largo {v_largo} ({e_largo:?}) · u8 {v_byte} ({e_byte:?})");
        eprintln!("B29.1 plano: {leitura}");
        assert_eq!((e_largo, e_byte), (Some(true), Some(false)), "{leitura}");
        assert!(v_largo <= 1, "o largo devia chegar ao alvo: {leitura}");
        assert!(v_byte > 10, "o u8 não estancou: {leitura}");
    }
}

/// A ponta de PÍXEL fica no `u8` sem estancar (recusa medida): deposita UMA vez por traço (`pixel::carimba`, o tecto
/// binário; o Flow é a opacidade do píxel) — 80 passadas = 2 passadas.
#[test]
fn a_ponta_de_pixel_nao_estanca_no_u8() {
    let pixel = |b: &mut BrushSpec| {
        flow_baixo(b);
        b.pixel = ph2d_painter_brush::PixelTip::Aligned;
    };
    let corre = |n: usize| {
        let mut t = ferramenta(pixel);
        t.set_paint_media(crate::PaintMedia::Digital);
        passadas(&mut t, n)
    };
    let (p2, p80) = (corre(2), corre(80));
    eprintln!("B29.1 píxel: 2 {p2:?} · 80 {p80:?}");
    assert!(
        p2.0 < 255 && p2 == p80,
        "a ponta de píxel deposita uma vez: 2 {p2:?} · 80 {p80:?}"
    );
}

/// `n` voltas a um quadrado (40..120) num traço só com Solid; devolve o R do contorno (40, 80), o do miolo
/// (80, 80) e a escolha do tecto.
fn voltas_solid(t: &mut PainterTool, n: usize) -> (u8, u8, Option<bool>) {
    let cantos = [[120.0, 40.0], [120.0, 120.0], [40.0, 120.0], [40.0, 40.0]];
    t.on_canvas_pointer(cpx([40.0, 40.0], PointerPhase::Down));
    for _ in 0..n {
        for c in cantos {
            t.on_canvas_pointer(cpx(c, PointerPhase::Move));
        }
    }
    let escolha = t.paint.tecto_largo.escolha;
    t.on_canvas_pointer(cpx([40.0, 40.0], PointerPhase::Up));
    let r = |x: u32, y: u32| t.canvas_rgba[((y * L + x) * 4) as usize];
    (r(40, 80), r(80, 80), escolha)
}

/// ⭐ O traço SOLID leva o tecto largo: a fita (Flow 2 %, 60 voltas) chega ao alvo da Strength 0,5 (127) e a
/// mancha recompõe o miolo ao nível de uma passagem lido do largo. No MESMO teste o `u8` (o veto) estanca a fita.
/// Controlo: 1 volta — os dois tectos dão a mesma fita e o mesmo miolo a ≤ 2 níveis (os cantos já sobrepõem
/// dabs: o `u8` começa a arredondar por dab).
#[test]
fn o_solid_leva_o_tecto_largo_e_chega_ao_alvo() {
    let solid = |b: &mut BrushSpec| {
        flow_baixo(b);
        b.strength = 0.5;
        b.style_solid = true;
    };
    let corre = |n: usize, so_byte: bool| {
        let mut t = ferramenta(solid);
        t.set_paint_media(crate::PaintMedia::Digital);
        t.paint.tecto_largo.so_byte = so_byte;
        voltas_solid(&mut t, n)
    };
    let ((cl1, ml1, _), (cb1, mb1, _)) = (corre(1, false), corre(1, true));
    let ((cl, ml, el), (cb, mb, eb)) = (corre(60, false), corre(60, true));
    let leitura = format!(
        "1 volta: largo fita {cl1} miolo {ml1} · u8 fita {cb1} miolo {mb1} | 60: largo fita {cl} miolo {ml} \
         ({el:?}) · u8 fita {cb} miolo {mb} ({eb:?})"
    );
    eprintln!("B29.1 Solid: {leitura}");
    assert_eq!((el, eb), (Some(true), Some(false)), "{leitura}");
    assert!(
        cl1.abs_diff(cb1) <= 2 && ml1.abs_diff(mb1) <= 2,
        "controlo: antes de estancar os dois tectos coincidem: {leitura}"
    );
    assert!(
        cl.abs_diff(127) <= 1,
        "a fita no largo devia chegar ao alvo 127: {leitura}"
    );
    assert!(
        cb.abs_diff(127) > 10,
        "o u8 não estancou: a fixture não contém o fenómeno: {leitura}"
    );
    assert!(
        ml.abs_diff(127) <= 1,
        "a mancha do largo devia recompor o miolo ao alvo 127: {leitura}"
    );
}

/// ⭐ O COMPOSITE de UMA camada Brush leva o tecto largo (trocado por camada como o `u8`) e chega ao alvo; no MESMO
/// teste o veto `u8` estanca. Com DUAS camadas fica no `u8`: a tela sob a 2.ª camada já não é o pré-traço (a 1.ª
/// pintou por baixo no mesmo lote), e `recompoe` apagaria a camada de baixo.
#[test]
fn o_composite_de_uma_camada_leva_o_tecto_largo() {
    let corre = |camadas: usize, so_byte: bool| {
        let mut t = ferramenta(flow_baixo);
        t.set_paint_media(crate::PaintMedia::Digital);
        t.paint.composite_enabled = true;
        for _ in 0..camadas {
            t.acrescenta_camada(super::super::super::composite::CompositeOp::Brush.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, 1.0);
        }
        assert!(t.composite_active(), "a fixture tem de armar o Composite");
        t.paint.tecto_largo.so_byte = so_byte;
        passadas(&mut t, 80)
    };
    let ((v_largo, e_largo), (v_byte, e_byte)) = (corre(1, false), corre(1, true));
    let (v_duas, e_duas) = corre(2, false);
    let leitura = format!(
        "1 camada: largo {v_largo} ({e_largo:?}) · u8 {v_byte} ({e_byte:?}) | 2 camadas: {v_duas} ({e_duas:?})"
    );
    eprintln!("B29.1 Composite: {leitura}");
    assert_eq!((e_largo, e_byte), (Some(true), Some(false)), "{leitura}");
    assert!(v_largo <= 1, "o largo devia chegar ao alvo (0): {leitura}");
    assert!(v_byte > 10, "o u8 não estancou: {leitura}");
    assert_eq!(
        e_duas,
        Some(false),
        "com 2 camadas o traço fica no u8: {leitura}"
    );
}
