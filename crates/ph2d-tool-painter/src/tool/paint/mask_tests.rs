//! Gates da **cobertura da máscara** (2026-07-25, doc 25 §13.10).
//!
//! ## A ordem que estes gates pinam
//!
//! *"A máscara deve pintar exatamente como o brush digital normal"* (Enio, depois do smoke). Ela não tem
//! lei própria: roda o MESMO pipeline de dabs, com o MESMO acúmulo per-dab, e a única diferença é o que a
//! cor significa (preto = proteger, branco = desproteger, e o destino é o scratch em vez da camada).
//!
//! ## Por que existiu uma lei própria por algumas horas, e por que ela morreu
//!
//! A borda da máscara endurece sob muitas passadas (o produto per-dab afia a cauda do falloff — medido:
//! band 3,53 px numa passada → 1,38 px em quinze). A cura tentada foi o **envelope do modo Wash do
//! Krita** (`max` por-traço em vez do produto), que de fato mata o endurecimento — **e foi REPROVADA na
//! tela**: sem a saturação do produto, a modulação por-dab do perfil fica visível e o traço sai em
//! **CONTAS** ao longo do ombro. Renderizado nas duas leis, com a mesma sonda
//! ([`super::mask_probe::probe_mask_beading_along_the_axis`]).
//!
//! ⚠️ **A lição de medição que isso deixou:** a modulação foi medida **no EIXO** do traço (6 níveis de
//! 255) e chamada de invisível. As contas não vivem no eixo — vivem no **OMBRO**, onde o perfil é íngreme,
//! e lá a mesma modulação é enorme na aparência. Um número no lugar errado disse o contrário do que a foto
//! dizia (`reference_topic_oracle_discipline`).
//!
//! Então: **as duas leis têm artefato**, e a cura do endurecimento — se voltar à mesa — não é a lei da
//! cobertura. Não reconstrua nenhuma das duas sem um render-and-look que mostre as contas ausentes.

use super::mask_probe::{band_px, coverage, cp, mask_tool, vstroke};
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

const S: u32 = 256;

/// **A ORDEM, pinada.** A cobertura que um traço de máscara deixa tem de ser, **byte a byte**, o ALFA que
/// o mesmo traço do brush digital depositaria — mesma geometria, mesmo pincel, mesmo acúmulo.
///
/// O oráculo é o produto de verdade: pinta-se o MESMO traço duas vezes, uma em modo Mask (lendo o
/// scratch) e uma em modo Paint com tinta preta sobre branco (lendo o canvas). Se a máscara ganhar
/// qualquer lei própria — um envelope, um teto, um cap por-modo — os dois campos divergem e isto fica
/// vermelho, que é exactamente o que se quer: foi assim que a lei do canal foi construída, e é assim que
/// ela (ou outra) não volta em silêncio.
///
/// **Mutação que deve sangrar** (medida): forçar o buffer por-traço na máscara + a lei `max` (o envelope
/// Wash) faz **3020 texels divergirem, pior delta 120 de 255**.
#[test]
fn the_mask_lays_exactly_what_the_digital_brush_lays() {
    let stroke = |t: &mut PainterTool| vstroke(t, 128.0, 60.0, 200.0, 25);

    // (a) modo Mask: a cobertura é `1 − luma` do scratch.
    let mut m = mask_tool(S);
    stroke(&mut m);
    let mask_cov = coverage(&m, S);

    // (b) modo Paint, tinta PRETA sobre branco: a cobertura equivalente é `1 − luma` do canvas. O pincel
    //     é o do slot de máscara, copiado para todos os slots, para que a única diferença seja o MODO.
    let mut p = PainterTool::default();
    p.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let mut mask_brush = m.paint.brush;
    mask_brush.color = [0.0, 0.0, 0.0];
    p.paint.brush = mask_brush;
    for slot in &mut p.paint.brush_by_mode {
        *slot = mask_brush;
    }
    stroke(&mut p); // (o `vstroke` já drena o preview; o campo pintado mora no canvas)
    let canvas = p.canvas_rgba.clone();

    let mut diff = 0usize;
    let mut worst = 0i32;
    for i in 0..(S as usize * S as usize) {
        let paint_cov = 255 - i32::from(canvas[i * 4]);
        let mask_val = (mask_cov[i] * 255.0).round() as i32;
        let d = (paint_cov - mask_val).abs();
        if d > 0 {
            diff += 1;
            worst = worst.max(d);
        }
    }
    assert_eq!(
        diff, 0,
        "a máscara tem de depositar o MESMO campo que o brush digital: {diff} texels diferem, pior \
         delta {worst} de 255"
    );
}

// ⚠️ **NÃO existe aqui um gate numérico das CONTAS**, e a razão é medida: sob o envelope reprovado o
// pico-a-pico da modulação por-dab é **5 níveis de 255**, contra **3** sob a lei do brush — os dois na
// mesma ordem, porque o que o olho vê não é a amplitude, é a ONDULAÇÃO PERIÓDICA sobre um campo
// quase-sólido (uma ripple de 2% de contraste é visível; um bar de pico-a-pico não a separa do ruído de
// quantização). Um gate com bar 4 seria um gate que não pode falhar pelo motivo que alega. O oráculo das
// contas é o RENDER (a sonda `probe_mask_beading_along_the_axis` + a foto no doc 25 §13.10), e o gate que
// de fato as impede é o de byte-identidade acima: o brush digital não faz contas, então a máscara não faz.

/// O cap de Accumulate segue armando exactamente onde armava, e **o MODO não entra na conta** — é isso
/// que "pinta como o brush digital" significa na porta.
#[test]
fn the_coverage_cap_arms_where_it_always_did() {
    let mut t = mask_tool(S);
    let base = ph2d_painter_brush::BrushSpec::default();
    assert!(
        !t.stroke_cover_wanted(&base),
        "Strength cheia + Accumulate OFF: o cap é inobservável, ninguém threada buffer"
    );
    assert!(
        t.stroke_cover_wanted(&ph2d_painter_brush::BrushSpec {
            strength: 0.5,
            accumulate: false,
            ..base
        }),
        "Strength < 1 + Accumulate OFF: o cap é observável e o buffer é threadado"
    );
    assert!(
        !t.stroke_cover_wanted(&ph2d_painter_brush::BrushSpec {
            strength: 0.5,
            accumulate: true,
            ..base
        }),
        "Accumulate ON não rastreia cobertura em modo nenhum"
    );
    let capped = ph2d_painter_brush::BrushSpec {
        strength: 0.5,
        ..base
    };
    t.paint.paint_mode = PaintMode::Paint;
    let in_paint = t.stroke_cover_wanted(&capped);
    t.paint.paint_mode = PaintMode::Mask;
    let in_mask = t.stroke_cover_wanted(&capped);
    assert_eq!(
        in_paint, in_mask,
        "a porta da cobertura não pode olhar o MODO: a máscara acumula como o brush digital"
    );
}

/// A máscara é um passo de undo, e o traço seguinte deposita normal — o ciclo de vida que qualquer lei de
/// cobertura tem de respeitar (o buffer por-traço é transiente, não estado de documento).
#[test]
fn a_mask_stroke_is_one_undo_step_and_the_next_stroke_starts_fresh() {
    let mut t = mask_tool(S);
    let blank = coverage(&t, S)[130 * S as usize + 128];
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let painted = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        painted > 0.9 && blank < 0.01,
        "fixture: o traço tem de proteger o miolo ({blank:.3} -> {painted:.3})"
    );
    assert!(t.undo_last(), "um traço de máscara é um passo de undo");
    let undone = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        undone < 0.01,
        "o undo tem de devolver a cobertura pré-traço, got {undone:.3}"
    );
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let again = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        (again - painted).abs() < 0.01,
        "re-pintar depois do undo tem de depositar o mesmo ({painted:.3} depois {again:.3})"
    );
}

/// **O custo é da PEGADA, não do canvas.** Quadruplicar a tela não pode mover o custo por movimento —
/// razão primeiro (imune à deriva da máquina), depois um kill de wall-clock generoso. Medido ~1,0× e
/// 0,9 ms médio / 2,5 ms pior nos dois perfis, contra um frame de 16,7 ms.
#[test]
fn the_mask_stroke_cost_does_not_follow_the_canvas() {
    let cost = |size: u32| -> f64 {
        let mut t = mask_tool(size);
        let c = size as f32 * 0.5;
        t.set_brush_size_px(120.0);
        t.on_canvas_pointer(cp([c - 100.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let n = 20;
        let t0 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 100.0 + i as f32 * 10.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let dt = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 100.0, c], PointerPhase::Up));
        dt
    };
    let small = cost(1024);
    let big = cost(2048);
    assert!(
        big < small * 1.6 + 0.15,
        "um move de máscara é limitado pela pegada: {small:.2} ms @1024² vs {big:.2} ms @2048² \
         (um passe que percorresse o plano daria 4×)"
    );
    assert!(
        big < 8.0,
        "um move de máscara cabe num frame com folga: {big:.2} ms @2048² (kill 8 ms)"
    );
}

/// A borda que a lei do brush deixa, **MEDIDA** — não uma asserção de que ela é boa. Existe para que o
/// número do endurecimento fique num teste executável e ninguém precise re-medir para saber do que se
/// fala: o traço nasce com ~3,5 px de rampa e ela aperta com as passadas. **É o defeito que segue ABERTO**
/// (doc 25 §13.10), e a cura não é a lei da cobertura — as duas leis foram tentadas.
#[test]
fn the_documented_hardening_is_still_there_and_this_is_its_number() {
    let mut t = mask_tool(S);
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let one = band_px(&coverage(&t, S), S, 130);
    for _ in 0..14 {
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    }
    let fifteen = band_px(&coverage(&t, S), S, 130);
    assert!(
        (one - 3.53).abs() < 0.5 && (fifteen - 1.38).abs() < 0.5,
        "o endurecimento documentado mudou de número (era 3.53 px numa passada e 1.38 em quinze): \
         got {one:.2} e {fifteen:.2}. Se foi de propósito, atualize o doc 25 §13.10 com a medição nova"
    );
}
