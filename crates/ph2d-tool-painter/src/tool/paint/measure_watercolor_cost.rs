//! **De que é feito um MOVE de aquarela** — a decomposição que o doc 28 §7 nomeou como o próximo passo.
//!
//! O censo dos quatro meios (`measure_the_four_media`) diz que a aquarela custa **3,1 ms/move** contra
//! **1,2 do Digital**, e que os dois são **planos no tamanho da tela** (3,07 → 3,12 e 1,17 → 1,21 de
//! 2048² para 4096²) — ou seja, limitados pela PEGADA, que é a forma correta. Ele não diz de que aquele
//! ~1,9 ms de diferença é feito, e sem isso qualquer otimização é palpite.
//!
//! ## Por que ABLAÇÃO POR ENTRADA, e não instrumentação
//!
//! A lição que resolveu o impasto: *uma sonda que re-implementa o laço fica CEGA à porta* — ela segue
//! imprimindo o custo antigo depois de o produto parar de pagá-lo. Então aqui nada é re-implementado:
//! cada linha dirige `on_canvas_pointer`, a porta de verdade, e o que muda entre linhas é um **knob que
//! o artista tem**.
//!
//! ⚠️ **E é por isso que a coluna se chama "o que esta FEATURE custa", não "o que dá para economizar".**
//! Desligar o Warp não é o mesmo trabalho sem uma etapa: é uma aquarela **diferente e mais barata**. O
//! número atribui custo a uma feature — que é a pergunta de PRODUTO ("vale o que cobra?") — e não
//! promete uma economia de graça, que seria vender o que a medição não mede.
//!
//! ## O que a tabela NÃO pode dizer
//!
//! As ablações **não somam** ao total: fases se compõem (o rewet lê campos que o spread constrói) e
//! desligar uma pode baratear outra. Ler a soma como decomposição exata seria o erro que a varredura de
//! raio deste repo já cometeu — lá o passo fixo confundia *raio* com *contagem de dabs*, e a coluna do
//! Digital chegou a FICAR MAIS BARATA com pincel maior. **Compare cada linha com o baseline, nunca as
//! linhas entre si.**

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas armado em aquarela com o pincel de aquarela **REAL**.
///
/// ⚠️ `..Default::default()` não é atalho aqui: `reset_brush_watercolor` restaura exatamente de
/// `BrushSpec::default()`, então os defaults do spec **são** o pincel que o botão Reset entrega. Uma
/// fixture que montasse os knobs à mão mediria uma aquarela que ninguém pinta.
fn wash(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// Mediana do custo de um MOVE — a porta `on_canvas_pointer`, que é o que o app reporta como
/// `INPUT (fora do frame)`.
///
/// ⚠️ **O `take_preview_arc` é DESCARTADO**, então o tool é dono único do canvas e o `Arc::make_mut`
/// dele é grátis. É o caminho otimista de propósito (mede a aquarela, não a costura de preview), e a
/// diferença contra o app está nomeada no doc 28 §4.8.3 — citar este número como *"o que o artista
/// sente"* seria vender o que a sonda não mede.
fn move_ms(t: &mut PainterTool, size: u32, radius: f32) -> f64 {
    let mid = f64::from(size / 2) as f32;
    let x0 = radius + 20.0;
    const STEP_PX: f32 = 40.0;
    let x1 = x0 + STEP_PX * 20.0;
    assert!(
        x1 < (size as f32) - radius,
        "o traço tem de caber no canvas"
    );
    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let _ = t.take_preview_arc();
    let mut moves = Vec::new();
    let mut x = x0 + STEP_PX;
    while x <= x1 {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        moves.push(t0.elapsed().as_secs_f64() * 1e3);
        let _ = t.take_preview_arc();
        x += STEP_PX;
    }
    t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
    moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    moves[moves.len() / 2]
}

/// **A tabela: o que cada feature da aquarela cobra por MOVE.**
///
/// A primeira linha é o baseline (o pincel de aquarela como ele sai do Reset); cada linha seguinte
/// desliga **um** knob e mede de novo. A última é o Digital, para dizer quanto do total é *carimbar
/// dabs* e quanto é *ser aquarela*.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_a_watercolor_move_is_made_of() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;

    // A configuração que estamos medindo, PERGUNTADA ao produto e impressa — ler os defaults do fonte e
    // torcer é como se documenta uma medição de outra coisa.
    {
        let t = wash(SIZE, RADIUS);
        let b = &t.paint.brush;
        println!(
            "\n[wash] pincel medido: edge_gain={:.2} edge_spread={:.2} granulation={:.2} warp={:.2} \
             wet_smudge={:.2} wet_rewet={:.2} wet_charge={:.2} wet_dilution={:.2} wet_pull={:.2} \
             pigment={} pigment_mix={:.2} fill={:.2} depth={:.2}",
            b.edge_gain,
            b.edge_spread,
            b.granulation,
            b.warp,
            b.wet_smudge,
            b.wet_rewet,
            b.wet_charge,
            b.wet_dilution,
            b.wet_pull,
            b.pigment,
            b.pigment_mix,
            b.fill,
            b.depth,
        );
    }

    let base = move_ms(&mut wash(SIZE, RADIUS), SIZE, RADIUS);
    println!(
        "\n{:<26} {:>10} {:>12}",
        "configuração", "move ms", "vs baseline"
    );
    println!("{:<26} {base:>10.3} {:>12}", "AQUARELA (baseline)", "—");

    // Cada ablação é um knob do PAINEL, aplicado pela porta pública que o painel usa.
    /// Um knob do painel e o nome dele — a unidade de ablação desta tabela.
    type Ablation = (&'static str, fn(&mut PainterTool));
    let ablations: [Ablation; 7] = [
        ("sem Warp", |t| t.set_brush_warp(0.0)),
        ("sem Granulation", |t| t.set_brush_granulation(0.0)),
        ("sem Edge (gain 0)", |t| t.set_brush_edge_gain(0.0)),
        ("sem Spread", |t| t.set_brush_edge_spread(0.0)),
        ("sem Rewet", |t| t.set_brush_wet_rewet(0.0)),
        ("sem Smudge", |t| t.set_brush_wet_smudge(0.0)),
        ("sem Pigment mixing", |t| t.set_brush_pigment_mixing(0.0)),
    ];
    for (name, off) in ablations {
        let mut t = wash(SIZE, RADIUS);
        off(&mut t);
        let ms = move_ms(&mut t, SIZE, RADIUS);
        println!("{name:<26} {ms:>10.3} {:>11.3}", ms - base);
    }

    // Tudo desligado: o que sobra é o depósito de cobertura + o recompose óptico mínimo.
    let mut t = wash(SIZE, RADIUS);
    for (_, off) in ablations {
        off(&mut t);
    }
    let bare = move_ms(&mut t, SIZE, RADIUS);
    println!(
        "{:<26} {bare:>10.3} {:>11.3}",
        "TUDO desligado",
        bare - base
    );

    // E o Digital, na MESMA fixture: a diferença é o preço de ser aquarela.
    let mut d = PainterTool::default();
    d.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let b = BrushSpec {
        radius_px: RADIUS,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    d.paint.brush = b;
    for slot in &mut d.paint.brush_by_mode {
        *slot = b;
    }
    d.set_paint_media(PaintMedia::Digital);
    let dig = move_ms(&mut d, SIZE, RADIUS);
    println!(
        "{:<26} {dig:>10.3} {:>11.3}",
        "DIGITAL (o carimbo)",
        dig - base
    );
    println!();
}
