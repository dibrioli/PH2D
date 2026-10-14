//! **Sonda (ignorada): o Soft Pastel NO TEMPO** — a bancada de comparação do prompt 35 contra o Rebelle
//! «Pastel Dry / Dry Thin». Os MESMOS gestos de `ferramentas/pastel_eventos.py` (tela 1000×640, o azul da
//! bancada, pressão 1, 4 px por evento, um quadro de 1/60 s por evento): um recto de 220 px e um zig-zague
//! fechado. Fotos a meio do gesto, no pen-up e 0,5 · 1 · 3 · 6 s depois, em `PH2D_SONDA_OUT` como
//! `pastel_<gesto>_<instante>.ppm`. A régua: `ferramentas/pastel_regua.py`.
//!
//! `PH2D_SONDA_PRESET=<nome>` troca o preset (por omissão «Soft Pastel»); `PH2D_SONDA_TAMANHO=<px>` a largura.
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

pub(super) const W: usize = 1000;
pub(super) const H: usize = 640;
const QUADRO_MS: f32 = 1000.0 / 60.0;
const PASSO_PX: f32 = 4.0;
/// O azul da bancada do Rebelle (`gera_eventos.py`, `AZUL`).
const AZUL: [u8; 3] = [23, 101, 179];

/// Os dois gestos, em polilinhas: o recto e o zig-zague fechado (o da foto do dono, 14/10).
pub(super) fn gestos() -> [(&'static str, Vec<[f32; 2]>); 2] {
    [
        ("recto", recto()),
        (
            "zigue",
            vec![
                [70.0, 190.0],
                [420.0, 150.0],
                [90.0, 240.0],
                [440.0, 200.0],
                [120.0, 290.0],
                [460.0, 250.0],
                [150.0, 320.0],
            ],
        ),
    ]
}

/// O recto: `PH2D_SONDA_RECTO=x0,y0,x1,y1` troca-o (a direcção isolada, Fase 1).
fn recto() -> Vec<[f32; 2]> {
    let v: Vec<f32> = std::env::var("PH2D_SONDA_RECTO")
        .unwrap_or_else(|_| "60,70,280,70".into())
        .split(',')
        .map(|x| x.parse().expect("recto"))
        .collect();
    vec![[v[0], v[1]], [v[2], v[3]]]
}

/// A polilinha a `PASSO_PX` por evento.
pub(super) fn amostra(pts: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut out = vec![pts[0]];
    for par in pts.windows(2) {
        let (a, b) = (par[0], par[1]);
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / PASSO_PX).ceil().max(1.0) as usize;
        for k in 1..=n {
            let f = k as f32 / n as f32;
            out.push([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]);
        }
    }
    out
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn grava(dir: &str, nome: &str, rgba: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for p in rgba.as_chunks::<4>().0 {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava o ppm");
}

/// Píxeis que mudaram (algum canal > 2) e a soma de `|Δ|` — a deriva entre duas fotos.
fn deriva(a: &[u8], b: &[u8]) -> (usize, u64) {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .fold((0, 0), |(n, s), (p, q)| {
            let d: u64 = (0..3).map(|c| u64::from(p[c].abs_diff(q[c]))).sum();
            let mudou = (0..3).any(|c| p[c].abs_diff(q[c]) > 2);
            (n + usize::from(mudou), s + d)
        })
}

/// Σ suspenso · Σ assente · Σ água livre · `has_fluid` da camada ativa do motor.
fn planos(t: &mut PainterTool) -> (f64, f64, f64, bool) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let e = &s.engine;
    let g = &e.layers[e.active_layer].grid;
    let soma = |v: &[f32]| v.iter().map(|x| f64::from(*x)).sum::<f64>();
    let n8 = |v: &[u8]| v.iter().filter(|x| **x != 0).count();
    let h = |v: &[f32]| v.iter().fold(0u64, |a, x| a.rotate_left(5) ^ u64::from(x.to_bits()));
    let h3 = |v: &[[f32; 3]]| v.iter().flatten().fold(0u64, |a, x| a.rotate_left(5) ^ u64::from(x.to_bits()));
    println!(
        "    hash susp {:x} susp_rgb {:x} sett {:x} sett_rgb {:x} relevo {:x} paper {:x} film {:x}",
        h(&g.susp), h3(&g.susp_rgb), h(&g.sett), h3(&g.sett_rgb), h(&g.relevo), h(&g.paper), h(&g.film)
    );
    println!(
        "    wet {} · active {} · bloom {} · frame {} · bbox {:?}",
        n8(&g.wet),
        n8(&g.active),
        n8(&g.bloom),
        e.sim.frame,
        (g.bx0, g.by0, g.bx1, g.by1)
    );
    (soma(&g.susp), soma(&g.sett), soma(&g.film), g.has_fluid)
}

#[test]
#[ignore = "sonda do prompt 35: corre-se à mão"]
fn sonda_pastel_no_tempo() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Soft Pastel".into());
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = Biblioteca::carregar(&raiz, None);
    let p = &bib
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o preset de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    // `PH2D_SONDA_TAMANHO=<px>`: a largura do painel por cima da do preset (comparar à MESMA largura vista).
    let mut volumes = p.parametros.volumes.clone();
    if let Ok(d) = std::env::var("PH2D_SONDA_TAMANHO") {
        volumes.size_px = Some(d.parse().expect("tamanho"));
    }
    t.wet_preset_aplicar_volumes(&volumes);
    t.paint.wetpaint.tilt_on = false;
    // `PH2D_SONDA_KNOBS=drag=0;pickup=0`: os botões do motor por cima do preset (as variantes da Fase 1).
    for par in std::env::var("PH2D_SONDA_KNOBS").unwrap_or_default().split(';') {
        if let Some((k, v)) = par.split_once('=') {
            let v: f64 = v.parse().expect("valor do botão");
            let d = ph2d_wet_paint::tuning::Tuning::def_of_key(k).expect("botão");
            t.paint.wetpaint.knobs.set(d.knob, v);
        }
    }
    // `PH2D_SONDA_MAPEAMENTO=<0..3>` (ViewPlane · Tiled · Random · Stencil) e `PH2D_SONDA_GRAO=<size>`: o Grain.
    if let Ok(m) = std::env::var("PH2D_SONDA_MAPEAMENTO") {
        t.paint.brush.texture.mapping =
            ph2d_painter_brush::TextureMapping::from_u8(m.parse().expect("mapeamento"));
    }
    if let Ok(nome) = std::env::var("PH2D_SONDA_GRAO_TIPO") {
        use ph2d_painter_brush::texture::TextureKind;
        t.paint.brush.texture.kind = (0..TextureKind::COUNT)
            .map(TextureKind::from_u8)
            .find(|k| format!("{k:?}") == nome)
            .expect("tipo do grão");
    }
    if let Ok(g) = std::env::var("PH2D_SONDA_GRAO") {
        let g: f32 = g.parse().expect("grão");
        t.paint.brush.texture.size = [g, g];
    }
    if let Ok(d) = std::env::var("PH2D_SONDA_DUREZA") {
        t.paint.brush.hardness = d.parse().expect("dureza");
    }
    // `PH2D_SONDA_SHAPE=Noise:8`: o Shape (tipo e Size) por cima do preset — o Noise do smoke do dono.
    if let Some((nome, lado)) = std::env::var("PH2D_SONDA_SHAPE").ok().and_then(|v| {
        v.split_once(':')
            .map(|(a, b)| (a.to_owned(), b.parse::<f32>().expect("Size")))
    }) {
        use ph2d_painter_brush::texture::TextureKind;
        let k = (0..TextureKind::COUNT)
            .map(TextureKind::from_u8)
            .find(|k| format!("{k:?}") == nome)
            .expect("tipo do Shape");
        t.set_brush_shape_kind(k.to_u8());
        t.paint.brush.shape.size = [lado, lado];
    }
    t.paint.brush.color = AZUL.map(|c| f32::from(c) / 255.0);
    println!("| gesto | instante | mudados desde o pen-up | Σ|Δ| |");
    println!("|---|---|---:|---:|");
    let mut ordem = gestos();
    if std::env::var("PH2D_SONDA_ORDEM").as_deref() == Ok("zigue") {
        ordem.reverse();
    }
    for (g, pts) in ordem {
        let s = amostra(&pts);
        t.on_canvas_pointer(cp(s[0], PointerPhase::Down));
        t.on_tick(QUADRO_MS);
        for (k, q) in s.iter().enumerate().skip(1) {
            t.on_canvas_pointer(cp(*q, PointerPhase::Move));
            t.on_tick(QUADRO_MS);
            if k == s.len() / 2 {
                grava(&dir, &format!("pastel_{g}_meio"), &t.canvas_rgba);
            }
        }
        let (su, se, fi, hf) = planos(&mut t);
        println!("  {g}: antes do Up · Σsusp {su:.1} · Σsett {se:.1} · Σfilm {fi:.2} · has_fluid {hf}");
        t.on_canvas_pointer(cp(*s.last().expect("pontos"), PointerPhase::Up));
        let (su, se, fi, hf) = planos(&mut t);
        println!("  {g}: no Up · Σsusp {su:.1} · Σsett {se:.1} · Σfilm {fi:.2} · has_fluid {hf}");
        let up = t.canvas_rgba.clone();
        grava(&dir, &format!("pastel_{g}_up"), &up);
        let mut quadros = 0;
        let mut antes = up.clone();
        for (seg, rotulo) in [(0.5f32, "0.5"), (1.0, "1"), (3.0, "3"), (6.0, "6")] {
            while (quadros as f32) < seg * 60.0 {
                t.on_tick(QUADRO_MS);
                quadros += 1;
                let (n, _) = deriva(&antes, &t.canvas_rgba);
                if n > 0 || quadros <= 8 {
                    let (su, se, fi, hf) = planos(&mut t);
                    println!(
                        "  {g}: quadro {quadros} muda {n} px · Σsusp {su:.1} · Σsett {se:.1} · Σfilm {fi:.2} · has_fluid {hf}"
                    );
                }
                antes.clone_from(&t.canvas_rgba);
            }
            grava(&dir, &format!("pastel_{g}_{rotulo}"), &t.canvas_rgba);
            let (n, soma) = deriva(&up, &t.canvas_rgba);
            println!("| {g} | {rotulo} s | {n} | {soma} |");
        }
    }
}
