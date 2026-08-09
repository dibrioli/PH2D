//! RENDER-AND-LOOK do **Composite Brush** — a sonda que existe porque o oráculo desta família é a
//! APARÊNCIA, e nenhum número que eu medi até aqui reproduziu o que o Enio fotografou (2026-08-09:
//! *"o serrilhado sumiu, mas por que esse resultado bizarro?"*).
//!
//! ⚠️ **Quatro sondas numéricas minhas falharam em CONTER o fenômeno** (borda livre sobre papel limpo,
//! rampa 90%..10% num traço reto) — todas mediram `7 texels` nas quatro combinações da pilha, e a foto
//! mostra outra coisa. O método prescrito nesta linha, quando isso acontece, é **renderizar e olhar**
//! (o `push_look_probe` do bow wave é o precedente).
//!
//! Escreve PNG em `PH2D_COMPOSITE_LOOK_DIR`. Zero dependência nova: o encoder abaixo emite deflate
//! **STORED** (sem compressão) — um PNG grande, e é irrelevante, porque ninguém shipa isto.
//!
//! ```text
//! PH2D_COMPOSITE_LOOK_DIR=/tmp/look cargo test -p ph2d-tool-painter --release \
//!     composite_look -- --ignored --nocapture
//! ```

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

// ── PNG mínimo (RGBA8, sem compressão) ────────────────────────────────────────────────────────────

fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c = table[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in bytes {
        a = (a + u32::from(x)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = kind.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

/// RGBA8 → bytes de um PNG válido. Deflate STORED, blocos de 65535 bytes.
fn png_rgba(px: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity((h * (1 + w * 4)) as usize);
    for y in 0..h {
        raw.push(0); // filtro None
        let o = (y * w * 4) as usize;
        raw.extend_from_slice(&px[o..o + (w * 4) as usize]);
    }
    let mut z = vec![0x78, 0x01];
    for (i, part) in raw.chunks(65_535).enumerate() {
        let last = u8::from((i + 1) * 65_535 >= raw.len());
        z.push(last);
        z.extend_from_slice(&(part.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(part.len() as u16)).to_le_bytes());
        z.extend_from_slice(part);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8 bits, RGBA, sem interlace
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

// ── a cena ────────────────────────────────────────────────────────────────────────────────────────

const SIZE: u32 = 640;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um arrasto reto, entregue em passos de ~4 px (o que um mouse de fato produz).
fn stroke(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let n = 60;
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for i in 1..=n {
        let u = i as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
}

/// A cena do smoke: três traços que se cruzam, pincel grande, vermelho — a foto do Enio.
///
/// ⚠️ **A Strength é ingrediente da FIXTURE, não decoração.** Com o pincel opaco os cruzamentos
/// SATURAM e as duas imagens (com e sem pilha) saem iguais — foi assim que a 1ª versão desta sonda
/// nasceu cega. Na foto do Enio os cruzamentos ESCURECEM, o que só acontece abaixo de 1.0, e é
/// justamente o regime onde o cap de Accumulate (`stroke_mask`) tem voto.
fn scene(composite: bool) -> PainterTool {
    let strength: f32 = std::env::var("PH2D_LOOK_STRENGTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.5);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = 34.0;
    t.paint.brush.color = [0.75, 0.25, 0.25];
    t.paint.brush.strength = strength;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = composite;
    // ⚠️ A pilha SUBSTITUI a Strength do pincel pela da camada (`stamp_dabs_composite` escreve
    // `brush.strength = layer.strength`), e o default da camada Brush é 1.0 — então uma comparação
    // que não iguale as duas mede OPACIDADES diferentes e o A/B não diz nada sobre a pilha.
    t.paint.composite[0].strength = strength;
    stroke(&mut t, [90.0, 560.0], [470.0, 60.0]);
    stroke(&mut t, [140.0, 240.0], [600.0, 250.0]);
    stroke(&mut t, [60.0, 400.0], [560.0, 380.0]);
    t
}

#[test]
#[ignore = "render-and-look: escreve PNG, roda sob demanda"]
fn composite_look() {
    let dir = std::env::var("PH2D_COMPOSITE_LOOK_DIR").unwrap_or_else(|_| "/tmp/look".into());
    std::fs::create_dir_all(&dir).unwrap();
    for (name, on) in [("controle_off", false), ("pilha_on", true)] {
        let t = scene(on);
        let png = png_rgba(&t.canvas_rgba, SIZE, SIZE);
        let path = format!("{dir}/{name}.png");
        std::fs::write(&path, png).unwrap();
        eprintln!("escrito {path}");
    }
}

/// RENDER-AND-LOOK do **Grid Stamp**: um arrasto em L numa grade retangular, com deslocamento.
/// O oráculo é o olho — os carimbos têm de encher as células e ficar alinhados à rede.
#[test]
#[ignore = "render-and-look: escreve PNG, roda sob demanda"]
fn grid_stamp_look() {
    let dir = std::env::var("PH2D_COMPOSITE_LOOK_DIR").unwrap_or_else(|_| "/tmp/look".into());
    std::fs::create_dir_all(&dir).unwrap();
    for (name, cell, off) in [
        ("grid_quadrada", [64.0f32, 64.0], [0.0f32, 0.0]),
        ("grid_retangular", [80.0, 32.0], [0.0, 0.0]),
        ("grid_deslocada", [64.0, 64.0], [32.0, 16.0]),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.color = [0.15, 0.35, 0.75];
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::GridStamp;
        t.paint.brush.grid_cell_px = cell;
        t.paint.brush.grid_offset_px = off;
        // Um L: uma perna horizontal e uma vertical, para ver as duas direções da caminhada.
        t.on_canvas_pointer(cp([70.0, 120.0], PointerPhase::Down));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([70.0 + i as f32 * 12.0, 120.0], PointerPhase::Move));
        }
        for i in 1..=30 {
            t.on_canvas_pointer(cp([550.0, 120.0 + i as f32 * 14.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([550.0, 540.0], PointerPhase::Up));
        let png = png_rgba(&t.canvas_rgba, SIZE, SIZE);
        let path = format!("{dir}/{name}.png");
        std::fs::write(&path, png).unwrap();
        eprintln!("escrito {path}");
    }
}

/// SONDA — **quanto** a pilha arrasta os cruzamentos, em px.
///
/// O cruzamento é o único lugar da cena com contraste sob o pincel (o build-up de dois traços), então
/// ele é o ÚNICO ponto onde o arrasto do Smear é visível — é isso que o olho lê como *"um pedaço
/// escorregou"*. O oráculo é o **centroide dos texels escuros**: sem pilha ele senta na interseção
/// geométrica; com pilha ele anda pelo campo de deslocamento.
#[test]
#[ignore = "sonda de diagnóstico"]
fn probe_composite_crossing_drift() {
    let dark = |t: &PainterTool| {
        let (mut n, mut sx, mut sy) = (0.0f64, 0.0f64, 0.0f64);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let i = ((y * SIZE + x) * 4) as usize;
                // Só o build-up: um traço sozinho a 50% não chega aqui.
                if t.canvas_rgba[i + 1] < 180 {
                    n += 1.0;
                    sx += f64::from(x);
                    sy += f64::from(y);
                }
            }
        }
        (n, sx / n.max(1.0), sy / n.max(1.0))
    };
    {
        let t = scene(false);
        let g: Vec<u8> = t.canvas_rgba.chunks(4).map(|p| p[1]).collect();
        eprintln!(
            "fixture: verde min={} max={}",
            g.iter().min().unwrap(),
            g.iter().max().unwrap()
        );
    }
    let (n0, x0, y0) = dark(&scene(false));
    let (n1, x1, y1) = dark(&scene(true));
    eprintln!("controle: {n0:.0} texels escuros, centroide ({x0:.2}, {y0:.2})");
    eprintln!("pilha   : {n1:.0} texels escuros, centroide ({x1:.2}, {y1:.2})");
    eprintln!(
        "deriva do cruzamento: {:.2} px",
        ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
    );
}

/// SONDA — o ciclo de vida da sessão de smear em composite, **a que achou o defeito de 2026-08-09**.
///
/// Ela imprime os números que o doc de `end_smear_session` cita; sem ela aqueles números deixam de ser
/// reproduzíveis e viram folclore. Com a guarda antiga (`paint_mode.smears()`) a saída era
/// `active=true` e `disp` ≠ 0 em 9.904 → 19.808 → 29.712 texels; hoje é `active=false`, `disp` zerado,
/// e a região re-renderizada constante.
#[test]
#[ignore = "sonda de diagnóstico"]
fn probe_composite_session_lifetime() {
    const S: u32 = 300;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    t.paint.brush.radius_px = 20.0;
    t.paint.brush.color = [0.6, 0.0, 0.0];
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for k in 0..3 {
        let y = 80.0 + k as f32 * 40.0;
        stroke(&mut t, [40.0, y], [260.0, y]);
        eprintln!(
            "apos traco {k}: warp.active={} disp_nonzero={} touched_all={:?}",
            t.paint.warp.active,
            t.paint
                .warp
                .disp
                .iter()
                .filter(|d| d[0] != 0.0 || d[1] != 0.0)
                .count(),
            t.paint.warp.touched_all,
        );
    }
}
