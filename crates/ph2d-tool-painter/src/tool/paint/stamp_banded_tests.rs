//! Os gates do lote em bandas — a identidade primeiro, o relógio depois.
//!
//! ⚠️ **O oráculo é o PRODUTO, não uma segunda implementação:** com o piso em `usize::MAX` a própria
//! `stamp_plain_dabs_banded_with` cai no laço `for d in dabs` que o `stamp_dabs_per_pixel` rodava antes
//! desta wave, chamando o mesmo kernel com os mesmos argumentos. As duas rotas comparadas aqui são,
//! literalmente, *a de antes* e *a de agora*.

use super::stamp_banded::{BATCH_MIN_AREA, stamp_plain_dabs_banded_with, wants_bands};
use ph2d_painter_brush::{BrushSpec, Dab};

const W: u32 = 512;
const H: u32 = 512;

fn canvas() -> Vec<u8> {
    vec![255u8; (W as usize) * (H as usize) * 4]
}

fn brush() -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        color: [0.1, 0.2, 0.8],
        ..BrushSpec::default()
    }
}

/// Um arco de `n` dabs — a forma que um editor de figura re-carimba, com sobreposição de verdade
/// (espaçamento bem menor que o diâmetro) para que a ORDEM entre dabs seja observável.
fn arc(n: usize, radius: f32) -> Vec<Dab> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32) / (n as f32) * std::f32::consts::TAU;
            let (s, c) = (t.sin(), t.cos());
            Dab {
                center: [256.0 + c * radius, 256.0 + s * radius],
                radius_px: 12.0,
                coverage: 0.6,
                // A cor varia por dab: se a ordem de composição trocasse, o pixel de sobreposição
                // mudaria de cor — é o que torna a identidade um teste da ORDEM, não só da soma.
                #[allow(clippy::cast_precision_loss)]
                color: [(i % 7) as f32 / 7.0, 0.3, 0.9],
                rotation: [1.0, 0.0],
                dir: [c, s],
                arc_len: 0.0,
                stroke_radius_px: 12.0,
            }
        })
        .collect()
}

/// Todo pixel de `buf` que difere de `pristine` cabe em `r`? — e quantos mudaram.
fn covers_every_change(
    r: Option<ph2d_painter_brush::DirtyRect>,
    buf: &[u8],
    pristine: &[u8],
) -> (bool, usize) {
    let mut ok = true;
    let mut n = 0usize;
    for (i, (a, b)) in buf
        .chunks_exact(4)
        .zip(pristine.chunks_exact(4))
        .enumerate()
    {
        if a == b {
            continue;
        }
        n += 1;
        let (x, y) = ((i as u32) % W, (i as u32) / W);
        let inside = r.is_some_and(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h);
        ok &= inside;
    }
    (ok, n)
}

/// **A wave inteira se apoia nisto:** dividir as linhas entre os núcleos não pode mover um byte.
///
/// Bandas são linhas disjuntas e cada uma percorre TODOS os dabs na ordem da lista, então um pixel é
/// composto pelos mesmos dabs, na mesma ordem — muda quem AVALIA a linha, nunca o que ela diz.
///
/// ⚠️ **O retângulo NÃO é comparado por igualdade, e a primeira versão deste gate estava errada nisso.**
/// O laço serial devolve o *span* de todo dab que tocou alguma coisa; a rota em banda devolve só as
/// linhas em que de fato escreveu, então ela é **mais APERTADA** — as linhas do aro, onde o falloff já
/// zerou, entram no span e não são escritas. Apertado é melhor (menos upload, janela de undo menor) e
/// continua correto, então a propriedade honesta não é *"os dois retângulos são o mesmo"* e sim
/// **"o retângulo cobre tudo que mudou, e não inventa área"** — que é o que os dois consumidores
/// (`declare_wrote` e `mark_dirty`) de fato pedem.
#[test]
fn the_banded_batch_paints_exactly_what_the_serial_loop_painted() {
    for n in [2usize, 17, 200] {
        for radius in [40.0f32, 160.0] {
            let dabs = arc(n, radius);
            let pristine = canvas();
            let mut serial = canvas();
            let mut banded = canvas();
            let rs =
                stamp_plain_dabs_banded_with(&mut serial, W, H, &dabs, &brush(), false, usize::MAX);
            let rb = stamp_plain_dabs_banded_with(&mut banded, W, H, &dabs, &brush(), false, 0);
            // A metade que importa: os PIXELS.
            let diff = serial.iter().zip(&banded).filter(|(a, b)| a != b).count();
            assert_eq!(diff, 0, "{diff} bytes divergem (n={n}, r={radius})");
            // E o retângulo cobre tudo que mudou — nas DUAS rotas.
            let (ok_s, changed) = covers_every_change(rs, &serial, &pristine);
            let (ok_b, _) = covers_every_change(rb, &banded, &pristine);
            assert!(changed > 0, "a fixture não pintou nada (n={n}, r={radius})");
            assert!(
                ok_s,
                "o retângulo serial não cobre tudo (n={n}, r={radius})"
            );
            assert!(
                ok_b,
                "o retângulo da banda não cobre tudo (n={n}, r={radius})"
            );
            // …e não inventa área: o da banda cabe dentro do serial.
            if let (Some(a), Some(b)) = (rb, rs) {
                assert!(
                    a.x >= b.x && a.y >= b.y && a.x + a.w <= b.x + b.w && a.y + a.h <= b.y + b.h,
                    "o retângulo da banda escapa do serial (n={n}, r={radius}): {a:?} vs {b:?}"
                );
            }
        }
    }
}

/// **O alpha-lock lê o pixel de baixo**, então ele é a fixture que separa "cada banda escreve o seu"
/// de "cada banda LÊ o seu" — um kernel que espiasse o vizinho quebraria aqui e não no gate acima.
#[test]
fn the_banded_batch_is_identical_under_alpha_lock_too() {
    let dabs = arc(120, 130.0);
    let mut serial = canvas();
    let mut banded = canvas();
    // Alpha variado por linha: o `preserve_alpha` multiplica pelo alpha ANTERIOR do pixel.
    for (i, px) in serial.chunks_exact_mut(4).enumerate() {
        px[3] = u8::try_from((i / W as usize) % 256).unwrap_or(255);
    }
    banded.copy_from_slice(&serial);
    let rs = stamp_plain_dabs_banded_with(&mut serial, W, H, &dabs, &brush(), true, usize::MAX);
    let rb = stamp_plain_dabs_banded_with(&mut banded, W, H, &dabs, &brush(), true, 0);
    assert!(
        rs.is_some() && rb.is_some(),
        "as duas rotas têm de pintar sob alpha-lock"
    );
    assert_eq!(
        serial.iter().zip(&banded).filter(|(a, b)| a != b).count(),
        0,
        "bytes divergem sob alpha-lock"
    );
}

/// **O piso protege o traço à mão livre**, que é a razão de o piso existir.
///
/// Um move de mão livre emite poucos dabs; dividi-los entre 32 núcleos perde. O piso é medido sobre a
/// SOMA DAS PEGADAS (o trabalho real), não sobre a caixa da união — os dabs de um traço se sobrepõem
/// fortemente, e a caixa mentiria para baixo.
#[test]
fn a_freehand_sized_batch_stays_serial_and_a_figure_sized_one_does_not() {
    // ⚠️ Pergunta ao PRODUTO (`wants_bands`), não à aritmética do teste: a 1ª versão deste gate
    // recomputava a regra por conta própria e teria ficado verde com o produto decidindo outra coisa.
    assert!(
        !wants_bands(&arc(6, 20.0), W, H, BATCH_MIN_AREA),
        "um lote de mão livre tem de ficar SERIAL"
    );
    assert!(
        !wants_bands(&arc(1, 20.0), W, H, BATCH_MIN_AREA),
        "um dab só nunca vale uma divisão"
    );
    assert!(
        wants_bands(&arc(525, 200.0), W, H, BATCH_MIN_AREA),
        "a figura do report tem de DIVIDIR"
    );
    // E a régua é a soma das pegadas, não a caixa: esta figura tem caixa PEQUENA e trabalho GRANDE.
    let tight = arc(400, 30.0);
    let bbox = 62usize * 62 * 4; // ordem da caixa desta figura — muito abaixo do piso
    assert!(
        bbox < BATCH_MIN_AREA && wants_bands(&tight, W, H, BATCH_MIN_AREA),
        "uma figura de caixa pequena e muito trabalho tem de DIVIDIR"
    );
}

/// **A consequência**: o lote dividido é materialmente mais rápido que o serial.
///
/// ⚠️ É uma RAZÃO entre as duas rotas medidas **costas-com-costas no mesmo processo e sobre o mesmo
/// estado**, e não um bar de relógio: a máquina desta linha é compartilhada, e ao longo de uma sessão
/// o MESMO trabalho já variou 2× sem uma linha mudar (doc 28 §5.46). Uma razão torna a carga um fator
/// comum. A barra é folgada de propósito — o que ela vigia é *a divisão acontecer*, e o número honesto
/// medido é ~10×.
///
/// ⚠️ **`#[ignore]`, e não por preguiça:** ele mede PARALELISMO, e a suíte roda os testes em paralelo —
/// as outras threads disputam os mesmos 32 núcleos e a razão desaba. Reprovou exatamente assim na 1ª
/// corrida da suíte cheia, verde isolado. Rodar:
/// `cargo test -p ph2d-tool-painter --release the_banded_batch_is_materially -- --ignored --test-threads=1`
#[test]
#[ignore = "mede PARALELISMO: precisa da máquina; --ignored --test-threads=1"]
fn the_banded_batch_is_materially_faster_than_the_serial_one() {
    // ⚠️ Raio 200 num canvas de 512: a figura do report tem raio 400, mas numa tela de 512 ela cairia
    // quase toda FORA — a fixture mediria o recorte, não o trabalho.
    let dabs = arc(525, 200.0);
    let b = brush();
    let mut buf = canvas();
    // Aquece as duas rotas antes de cronometrar (first-touch da tela).
    let _ = stamp_plain_dabs_banded_with(&mut buf, W, H, &dabs, &b, false, usize::MAX);
    let _ = stamp_plain_dabs_banded_with(&mut buf, W, H, &dabs, &b, false, 0);
    let mut ser = f64::MAX;
    let mut par = f64::MAX;
    for _ in 0..5 {
        let mut a = canvas();
        let t0 = std::time::Instant::now();
        let _ = stamp_plain_dabs_banded_with(&mut a, W, H, &dabs, &b, false, usize::MAX);
        ser = ser.min(t0.elapsed().as_secs_f64());
        let mut c = canvas();
        let t0 = std::time::Instant::now();
        let _ = stamp_plain_dabs_banded_with(&mut c, W, H, &dabs, &b, false, 0);
        par = par.min(t0.elapsed().as_secs_f64());
    }
    let ratio = ser / par.max(1e-12);
    assert!(
        ratio > 2.0,
        "o lote dividido tem de ser materialmente mais rápido: {ratio:.2}x (serial {:.3} ms, banda {:.3} ms)",
        ser * 1e3,
        par * 1e3
    );
}

fn cpx(
    pos: [f32; 2],
    phase: ph2d_editor_core::tool::PointerPhase,
) -> ph2d_editor_core::tool::CanvasPointer {
    ph2d_editor_core::tool::CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase, RasterEditTool as _};
/// **O modo de pintura SOBREVIVE a um stroke vivo?** — o report do Enio de 2026-08-03
/// (*"Wet Paint regride para digital ao usar os strokes vivos"*).
#[test]
fn the_paint_media_survives_every_live_shape_method() {
    use ph2d_painter_brush::StrokeMethod;
    for method in [
        StrokeMethod::Line,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::Arc,
        StrokeMethod::FreeHand,
    ] {
        let mut t = crate::tool::PainterTool::default();
        t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
        t.set_paint_media(crate::tool::paint::media::PaintMedia::WetPaint);
        assert_eq!(
            t.paint_media(),
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio nem chegou a armar"
        );
        t.paint.brush.stroke_method = method;
        t.on_canvas_pointer(cpx([80.0, 128.0], PointerPhase::Down));
        t.on_canvas_pointer(cpx([170.0, 128.0], PointerPhase::Move));
        let after_move = t.paint_media();
        t.on_canvas_pointer(cpx([170.0, 128.0], PointerPhase::Up));
        let after_up = t.paint_media();
        assert_eq!(
            after_move,
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio REGREDIU durante o arrasto -> {after_move:?}"
        );
        assert_eq!(
            after_up,
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio REGREDIU no pen-up -> {after_up:?}"
        );
    }
}
