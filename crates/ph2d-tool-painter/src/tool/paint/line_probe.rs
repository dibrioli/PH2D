//! **O que uma forma SÓLIDA custaria** — a medição 2 da W0 (plano
//! `docs/Painter/38_plano_linha_procedural.md`), e o preço da borda que o Enio pediu *"o melhor
//! possível"*.
//!
//! O `Style: Solid` do Alchemy preenche o caminho. ⚠️ **A rota barata é parar antes do traçado:** o
//! `stroke_boolean_contours` já rasteriza a máscara a `SS = 3`, compõe as formas e **então** traça o
//! contorno (Moore) para devolver uma polilinha que vira dabs — a região preenchida (`crisp`) está
//! em mãos no meio do caminho e é jogada fora.
//!
//! ⚠️ **A W7 mudou o que este número SIGNIFICA, e as tabelas continuam válidas.** Elas medem o custo
//! da MANCHA, e a mancha continua sendo exactamente isto. O que mudou é que ela deixou de
//! SUBSTITUIR o contorno: desde 2026-08-15 uma figura sólida é o preenchimento **mais** o traço
//! (o modelo do Flip), então o `traca`/`pts` que aqui aparecem como *"o que o Solid pula"* são hoje
//! **o que ele paga ao lado** — a economia medida é sobre o composite booleano, não sobre o pincel.
//!
//! As duas irmãs de motor (a fórmula de velocidade e o orçamento do Sketchy) moram no
//! `ph2d-painter-brush::line_probe`. O corte é por ASSUNTO: lá *o que o traço É*, aqui *o que a
//! figura CUSTA*.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// **O QUE O SOLID PAGA E O QUE ELE PULA** — a decomposição do composite, relida na moldura do Solid.
///
/// As três fases já são instrumentadas pelo produto (`stroke_boolean::diag`). A leitura nova é a
/// atribuição: `converte + rasteriza` é o que o Solid **paga** (ele precisa da máscara), `traca` é o
/// que ele **pula**, e `pts` é o que o Line ainda gasta **depois** — cada ponto do contorno traçado
/// vira posição de dab no `fill_polyline_preview`.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_what_a_solid_would_skip() {
    println!("[line] o composite booleano relido como Solid — Ellipse 200 px, Digital, 4096");
    println!(
        "{:>8}  {:>9} {:>9} {:>9}  {:>9} {:>9}  {:>11} {:>8}",
        "formas", "converte", "rasteriza", "PAGA", "traca", "PULA%", "celulas", "pts"
    );
    for extra in [0usize, 1, 3] {
        let side = 4096u32;
        let mut t = tool(side, PaintMedia::Digital, 48.0);
        #[allow(clippy::cast_precision_loss)]
        let cx = (side / 2) as f32;
        let r = 200.0f32;
        t.set_stroke_op_mode(1);
        for k in 0..extra {
            #[allow(clippy::cast_precision_loss)]
            let dx = -r * 1.6 + (k as f32) * r * 0.8;
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx + dx, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Move));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Up));
            t.park_active_shape();
        }
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        let _ = super::stroke_boolean::diag::take();
        for k in 0..5 {
            let d = if k % 2 == 0 { r + 2.0 } else { r - 2.0 };
            t.on_canvas_pointer(cp([cx + d, cx], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([cx + r, cx], PointerPhase::Up));
        let g = super::stroke_boolean::diag::take();
        let n = f64::from(g.calls.max(1));
        let (c, ra, tr) = (
            g.convert_us as f64 / 1e3 / n,
            g.raster_us as f64 / 1e3 / n,
            g.trace_us as f64 / 1e3 / n,
        );
        #[allow(clippy::cast_precision_loss)]
        let cells = g.cells as f64 / n;
        #[allow(clippy::cast_precision_loss)]
        let pts = g.pts as f64 / n;
        let pays = c + ra;
        println!(
            "{:>8}  {c:>9.3} {ra:>9.3} {pays:>9.3}  {tr:>9.3} {:>9.1} {cells:>11.0} {pts:>8.0}",
            extra + 1,
            100.0 * tr / (pays + tr),
        );
    }
    println!(
        "[line] leitura: `PULA%` e' o traçado do composite BOOLEANO, que a mancha nao precisa — \
         desde a W7 o traço e' carimbado ao lado dela (o modelo do Flip), so' que pelo motor de \
         dabs e nao por este contorno."
    );
}

/// **O QUE O SOLID ACRESCENTA** — reduzir a máscara supersampleada a cobertura e compor no canvas.
///
/// ⚠️ **Esta é a única coluna ESTIMADA da W0, e está marcada como tal.** A peça não existe ainda;
/// o que se mede aqui é um laço equivalente sobre uma janela do MESMO tamanho que o composite de
/// fato produziu (a coluna `celulas` da tabela acima). Ele faz exatamente o que o Solid faria — a
/// caixa 3×3 do supersample (que É a cobertura por área, a menos da quantização) e um `over` sobre
/// o destino — então é um piso honesto, não um palpite.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_what_a_solid_would_add() {
    println!(
        "[line] o que o Solid ACRESCENTA: reduzir a mascara SS=3 a cobertura + compor (ESTIMATIVA)"
    );
    println!(
        "{:>12} {:>10}  {:>10} {:>10}  {:>10}",
        "janela SS", "px destino", "reduz(ms)", "compoe(ms)", "TOTAL(ms)"
    );
    for (sw, sh) in [(600usize, 600usize), (1200, 1200), (2400, 2400)] {
        let (dw, dh) = (sw / 3, sh / 3);
        // Uma máscara plausível: um disco, para o laço ter tanto miolo cheio quanto borda.
        let mut crisp = vec![0u8; sw * sh];
        #[allow(clippy::cast_precision_loss)]
        let (cx, cy, rr) = (sw as f32 * 0.5, sh as f32 * 0.5, sw as f32 * 0.45);
        for y in 0..sh {
            for x in 0..sw {
                #[allow(clippy::cast_precision_loss)]
                let d = (x as f32 - cx).hypot(y as f32 - cy);
                crisp[y * sw + x] = u8::from(d <= rr) * 255;
            }
        }
        let mut cov = vec![0u8; dw * dh];
        let t0 = std::time::Instant::now();
        for y in 0..dh {
            for x in 0..dw {
                let mut s = 0u32;
                for j in 0..3 {
                    let row = (y * 3 + j) * sw + x * 3;
                    s += u32::from(crisp[row])
                        + u32::from(crisp[row + 1])
                        + u32::from(crisp[row + 2]);
                }
                #[allow(clippy::cast_possible_truncation)]
                {
                    cov[y * dw + x] = (s / 9) as u8;
                }
            }
        }
        let reduce_ms = t0.elapsed().as_secs_f64() * 1e3;

        let mut dst = vec![255u8; dw * dh * 4];
        let col = [32u8, 64, 160];
        let t1 = std::time::Instant::now();
        for (i, &c) in cov.iter().enumerate() {
            let a = u32::from(c);
            if a == 0 {
                continue;
            }
            let p = i * 4;
            for ch in 0..3 {
                let d = u32::from(dst[p + ch]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    dst[p + ch] = ((u32::from(col[ch]) * a + d * (255 - a)) / 255) as u8;
                }
            }
        }
        let over_ms = t1.elapsed().as_secs_f64() * 1e3;
        println!(
            "{:>12} {:>10}  {reduce_ms:>10.3} {over_ms:>10.3}  {:>10.3}",
            sw * sh,
            dw * dh,
            reduce_ms + over_ms
        );
    }
    println!(
        "[line] leitura: some isto ao `PAGA` da tabela anterior e compare com `PAGA + traca + os dabs`."
    );
}

/// **A BORDA: quantos níveis de cobertura ela precisa** — o preço de *"o melhor possível"* (Enio,
/// 2026-08-12, decisão §5.2: a borda de uma forma sólida é **da FORMA**, não do pincel).
///
/// ⚠️ **`SS = 3` dá dez níveis** (0/9 … 9/9). A pergunta que decide não é quantos níveis existem, é
/// **quanto a borda erra** contra a cobertura por área — e a resposta tem de vir de uma medição,
/// porque um degrau de 28 níveis de 255 numa aresta longa e rasa é exatamente o que se lê como
/// *banding*.
///
/// A referência é `SS = 32` (1025 níveis, erro de quantização ≤ 0,25 nível): não é "exata" no papel,
/// mas está três ordens de grandeza abaixo do que se compara com ela.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_how_many_levels_a_solid_edge_needs() {
    println!(
        "[line] a borda de uma forma solida: erro contra a referencia SS=32, num disco de raio 300"
    );
    println!(
        "{:>5} {:>8}  {:>10} {:>10} {:>10}  {:>10}",
        "SS", "niveis", "erro medio", "erro max", "px>8/255", "custo(ms)"
    );
    const W: usize = 700;
    let cov = |ss: usize| -> (Vec<f32>, f64) {
        let t0 = std::time::Instant::now();
        let mut out = vec![0.0f32; W * W];
        #[allow(clippy::cast_precision_loss)]
        let (cx, cy, rr) = (W as f32 * 0.5, W as f32 * 0.5, 300.0f32);
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / (ss * ss) as f32;
        for y in 0..W {
            for x in 0..W {
                let mut hits = 0u32;
                for j in 0..ss {
                    for i in 0..ss {
                        #[allow(clippy::cast_precision_loss)]
                        let sx = x as f32 + (i as f32 + 0.5) / ss as f32;
                        #[allow(clippy::cast_precision_loss)]
                        let sy = y as f32 + (j as f32 + 0.5) / ss as f32;
                        hits += u32::from((sx - cx).hypot(sy - cy) <= rr);
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                {
                    out[y * W + x] = hits as f32 * inv;
                }
            }
        }
        (out, t0.elapsed().as_secs_f64() * 1e3)
    };
    let (reference, ref_ms) = cov(32);
    for ss in [3usize, 4, 8, 16] {
        let (c, ms) = cov(ss);
        let mut sum = 0.0f64;
        let mut worst = 0.0f64;
        let mut over = 0u32;
        let mut edge = 0u32;
        for i in 0..W * W {
            let r = f64::from(reference[i]);
            if r <= 0.0 || r >= 1.0 {
                continue; // só a BORDA: no miolo e fora todos concordam por construção
            }
            edge += 1;
            let e = (f64::from(c[i]) - r).abs() * 255.0;
            sum += e;
            worst = worst.max(e);
            over += u32::from(e > 8.0);
        }
        println!(
            "{ss:>5} {:>8}  {:>10.2} {worst:>10.2} {:>10.1} {ms:>10.1}",
            ss * ss + 1,
            sum / f64::from(edge.max(1)),
            100.0 * f64::from(over) / f64::from(edge.max(1)),
        );
    }
    // ── E a lei que SHIPOU, na MESMA tabela ────────────────────────────────────────────────────
    // ⚠️ A acumulação de área com sinal não amostra: ela integra a área EXATA do pixel coberto. Ela
    // entra aqui para o número dela ser comparável com os do supersample, e não citado à parte.
    {
        let disc: Vec<[f32; 2]> = (0..2048)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let a = i as f32 / 2048.0 * std::f32::consts::TAU;
                #[allow(clippy::cast_precision_loss)]
                let (cx, cy) = (W as f32 * 0.5, W as f32 * 0.5);
                [cx + 300.0 * a.cos(), cy + 300.0 * a.sin()]
            })
            .collect();
        let t0 = std::time::Instant::now();
        let cov = ph2d_painter_brush::solid::fill_coverage(&[disc], W, W, [0.0, 0.0]);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let (mut sum, mut worst, mut over, mut edge) = (0.0f64, 0.0f64, 0u32, 0u32);
        for i in 0..W * W {
            let r = f64::from(reference[i]);
            if r <= 0.0 || r >= 1.0 {
                continue;
            }
            edge += 1;
            let e = (f64::from(cov[i]) / 255.0 - r).abs() * 255.0;
            sum += e;
            worst = worst.max(e);
            over += u32::from(e > 8.0);
        }
        println!(
            "{:>5} {:>8}  {:>10.2} {worst:>10.2} {:>10.1} {ms:>10.1}",
            "area",
            256,
            sum / f64::from(edge.max(1)),
            100.0 * f64::from(over) / f64::from(edge.max(1)),
        );
    }
    println!(
        "[line] (referencia SS=32 custou {ref_ms:.1} ms; a coluna `px>8/255` e' a fracao da BORDA que erra mais que um degrau visivel)"
    );
}
