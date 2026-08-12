//! **A BORDA DURA DENTRO DA LAVAGEM** — a sonda do report do Enio de 2026-08-11: em WATERCOLOR, com
//! *Smooth Edges* marcado e `Dilution > 0`, pixels DUROS aparecem para além da borda do traço.
//!
//! ⚠️ **O interruptor do AA é a CAUSA, não a cura que falta** — e é isso que a tabela diz. Medido no
//! interior da lavagem (a foto do report não tem papel nenhum: é toda vermelha, com um contorno
//! serrilhado dentro dela):
//!
//! ```text
//!   config                  largura   cliff max   n cliffs
//!   Dilution 0,45 - AA on      30,6        19,7         67
//!   Dilution 0,45 - AA off     28,2         5,7          0
//! ```
//!
//! **São DOIS fatos, e separá-los é o diagnóstico:**
//!
//! 1. **A DILATAÇÃO, e ela independe da Dilution.** O [`super::watercolor_field::aa_coverage`]
//!    devolve `cw = mx`, o **MÁXIMO** dos 3×3 supersamples sobre ±0,667 texel — isto é uma
//!    **dilatação morfológica** da silhueta. A lavagem sai **~2 px mais larga em TODA a faixa de
//!    Dilution** (34,8 contra 32,8 · 30,6 contra 28,2 · 25,4 contra 23,8), e o contorno de uma
//!    dilatação **segue a grade discreta**, logo é escadinha por construção. É literalmente
//!    *"empurra pixels para além das bordas do traço"*.
//! 2. **A Dilution decide se ela é VISÍVEL.** A janela de endurecimento é
//!    `smoothstep(SS0 = 0,12 · SS1 = 0,60)`, e `flow = 1 − dilution`: a 0,45 o CORPO da lavagem
//!    pousa em ~0,55, **dentro** da janela em vez de saturado acima dela. Aí as duas estatísticas
//!    3×3 (`mx` e `ss/mx`) passam a variar no interior em vez de darem 1, e cada degrau delas vira
//!    um degrau de tinta.
//!
//! **A previsão que a mecânica faz, e a varredura confirma:** os degraus formam um ARCO em Dilution
//! — poucos em 0 (corpo saturado ACIMA da janela), pico em 0,45-0,60 (corpo DENTRO), zero em 0,90
//! (corpo ABAIXO de `SS0`, nada renderiza).
//!
//! **Atribuição por ablação** (Dilution 0,45, `n cliffs` · `cliff max`): AA completo **67 · 19,7** ·
//! trocando só `cw = mx` por `cw = single` **39 · 9,0** · sem AA nenhum **0 · 5,7**. As duas metades
//! contribuem, e nenhuma sozinha explica o defeito.
//!
//! ⚠️ **CORREÇÃO da 2ª rodada (2026-08-11): a frase *"empurra pixels PARA ALÉM das bordas"* é falsa
//! no sentido literal, e [`measure_how_far_the_second_stroke_reaches`] a mede.** Um 2º traço de raio
//! 26 não muda **um único byte** além de 30 px do próprio eixo, em nenhuma das quatro configurações
//! — a dilatação 3×3 alarga a lavagem em ~2 px, e é só isso. O que o Enio vê é o **contorno dentro
//! do campo**, e a cena que o produz é [`measure_the_edge_over_dry_pigment`], não esta.
//!
//! ⚠️ **E os gates de AA estavam VERDES por DUAS razões independentes** (`watercolor_aa_tests.rs`):
//! o `count_cliffs` deles conta **só adjacências papel↔sólido** (`≥ 230` tocando `≤ 60`), e um degrau
//! entre dois vermelhos de meio-tom não é nenhum dos dois — o oráculo é **estruturalmente cego ao
//! interior**; e **nenhum** daqueles gates arma `wet_dilution` (zero ocorrências no arquivo), então a
//! fixture nunca conteve o fenômeno. As duas falhas são as da família que este módulo já pagou.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-tool-painter --release measure_the_carried_waters_edge -- --ignored --nocapture --test-threads=1
//! ```

use super::measure_impasto_cost::cp;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const SIDE: u32 = 256;

/// Uma lavagem de aquarela sobre papel branco, com os knobs do report.
fn wash(dilution: f32, smooth_edges: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
    t.paint.brush = BrushSpec {
        radius_px: 26.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.90, 0.15, 0.18],
        space_attenuation: false,
        watercolor: true,
        smooth_edges,
        wet_dilution: dilution,
        fill: 0.45,
        depth: 2.0,
        edge_gain: 1.2,
        edge_spread: 6.0,
        opacity: 0.4,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Um traço VERTICAL: cada linha do canvas atravessa a mesma borda, que é o que torna a
    // serrilha (variação de linha para linha do contorno) mensurável.
    let cx = f32::from(u16::try_from(SIDE / 2).unwrap_or(128));
    t.on_canvas_pointer(cp([cx, 40.0], PointerPhase::Down));
    for i in 1..=16u8 {
        t.on_canvas_pointer(cp([cx, 40.0 + f32::from(i) * 11.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([cx, 216.0], PointerPhase::Up));
    t.on_tick(16.0);
    t.canvas_rgba.as_ref().clone()
}

/// Quantos pixels de uma faixa de linhas diferem do papel, e quantos CLIFFS (saltos de mais de
/// `STEP` níveis entre vizinhos horizontais) existem no lado direito do traço.
///
/// `CLIFF` é a unidade que o doc do [`super::watercolor_field::aa_coverage`] já usa para dizer se
/// uma borda foi reconstruída ou ficou binária.
fn profile(px: &[u8], y: usize) -> Vec<f32> {
    (0..SIDE as usize)
        .map(|x| {
            let i = (y * SIDE as usize + x) * 4;
            (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0
        })
        .collect()
}

/// A faixa INTERIOR da lavagem numa linha: entre o primeiro e o último pixel que não é papel,
/// recuada de `PAD` para não contar a silhueta (que tem tratamento próprio).
const PAD: usize = 6;

fn interior(prof: &[f32]) -> Option<(usize, usize)> {
    let lo = (0..prof.len()).find(|&x| prof[x] < 250.0)?;
    let hi = (0..prof.len()).rev().find(|&x| prof[x] < 250.0)?;
    (hi > lo + 2 * PAD + 2).then(|| (lo + PAD, hi - PAD))
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_carried_waters_edge() {
    println!("\n=== O CONTORNO INTERIOR DA LAVAGEM (Watercolor, traco vertical) ===\n");
    println!(
        "{:<28} {:>9} {:>11} {:>10} {:>10}",
        "config", "largura", "cliff max", "n cliffs", "grad med"
    );

    let rows: Vec<usize> = (70..200).collect();
    let mut band: Vec<(String, Vec<u8>)> = Vec::new();
    for dil in [0.0f32, 0.15, 0.30, 0.45, 0.60, 0.75, 0.90] {
        for aa in [true, false] {
            band.push((
                format!("Dilution {dil:.2} - AA {}", if aa { "on " } else { "off" }),
                wash(dil, aa),
            ));
        }
    }

    for (label, px) in &band {
        let (mut cliff_max, mut cliffs, mut gsum, mut gn, mut wsum, mut wn) =
            (0.0f32, 0usize, 0.0f32, 0usize, 0usize, 0usize);
        for &y in &rows {
            let prof = profile(px, y);
            let Some((lo, hi)) = interior(&prof) else {
                continue;
            };
            wsum += hi - lo;
            wn += 1;
            for x in lo..hi {
                let g = (prof[x + 1] - prof[x]).abs();
                cliff_max = cliff_max.max(g);
                gsum += g;
                gn += 1;
                if g > 6.0 {
                    cliffs += 1;
                }
            }
        }
        let width = wsum as f32 / wn.max(1) as f32;
        let gmean = gsum / gn.max(1) as f32;
        println!("{label:<28} {width:>9.1} {cliff_max:>11.1} {cliffs:>10} {gmean:>10.3}");
    }
    println!(
        "\n    O INTERIOR de uma lavagem e liso por construcao (o modelo optico e continuo).\n    \
         Um degrau ali e um contorno que nao passou por reconstrucao nenhuma — e a pergunta\n    \
         e se o interruptor de Smooth Edges o alcanca.\n"
    );
}

// ---------------------------------------------------------------------------------------------
// A CENA DO REPORT: um SEGUNDO traco sobre pigmento ja SECO.
// ---------------------------------------------------------------------------------------------

/// Duas pinceladas: uma faixa HORIZONTAL (que seca, virando dona daqueles texels) e depois uma
/// VERTICAL que a atravessa. E a cena que o Enio reporta, e a que a sonda acima **nao continha**:
/// ela pintava um traco so, sobre papel branco.
fn wash_over_dry(dilution: f32, smooth_edges: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
    t.paint.brush = BrushSpec {
        radius_px: 26.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.90, 0.15, 0.18],
        space_attenuation: false,
        watercolor: true,
        smooth_edges,
        wet_dilution: dilution,
        fill: 0.45,
        depth: 2.0,
        edge_gain: 1.2,
        edge_spread: 6.0,
        opacity: 0.4,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Traco 1 — a faixa HORIZONTAL em y=90. Ao soltar ela commita: os texels dela passam a ter
    // um `style_owner` proprio, e o composite do traco seguinte os re-renderiza como `settled`.
    t.on_canvas_pointer(cp([24.0, 90.0], PointerPhase::Down));
    for i in 1..=16u8 {
        t.on_canvas_pointer(cp([24.0 + f32::from(i) * 13.0, 90.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([232.0, 90.0], PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(16.0);
    }
    // Traco 2 — a VERTICAL que atravessa. Sobre a faixa, ela pinta em cima de pigmento seco.
    let cx = f32::from(u16::try_from(SIDE / 2).unwrap_or(128));
    t.on_canvas_pointer(cp([cx, 30.0], PointerPhase::Down));
    for i in 1..=16u8 {
        t.on_canvas_pointer(cp([cx, 30.0 + f32::from(i) * 12.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([cx, 222.0], PointerPhase::Up));
    t.on_tick(16.0);
    t.canvas_rgba.as_ref().clone()
}

/// Degraus numa JANELA de colunas (o traco vertical tem largura conhecida; sobre pigmento seco nao
/// ha papel para delimitar um "interior", entao a janela e dada em vez de derivada).
fn cliffs_in(px: &[u8], rows: &[usize], x0: usize, x1: usize) -> (f32, usize, f32) {
    let (mut mx, mut n, mut sum, mut cnt) = (0.0f32, 0usize, 0.0f32, 0usize);
    for &y in rows {
        let prof = profile(px, y);
        for x in x0..x1 {
            let g = (prof[x + 1] - prof[x]).abs();
            mx = mx.max(g);
            sum += g;
            cnt += 1;
            if g > 6.0 {
                n += 1;
            }
        }
    }
    (mx, n, sum / cnt.max(1) as f32)
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_edge_over_dry_pigment() {
    println!("\n=== O SEGUNDO TRACO, SOBRE PIGMENTO SECO (a cena do report) ===\n");
    println!(
        "{:<30} {:>10} {:>11} {:>10} {:>10}",
        "config", "regiao", "cliff max", "n cliffs", "grad med"
    );

    // Sobre a faixa seca (y perto de 90) e fora dela (y baixo, papel limpo).
    let over_dry: Vec<usize> = (72..=108).collect();
    let over_paper: Vec<usize> = (150..=200).collect();
    // A janela cobre o traco vertical inteiro mais margem dos dois lados.
    let (wx0, wx1) = (80usize, 176usize);

    for dil in [0.0f32, 0.45] {
        for aa in [true, false] {
            let px = wash_over_dry(dil, aa);
            let label = format!("Dilution {dil:.2} - AA {}", if aa { "on " } else { "off" });
            for (region, rows) in [("seco", &over_dry), ("papel", &over_paper)] {
                let (mx, n, gm) = cliffs_in(&px, rows, wx0, wx1);
                println!("{label:<30} {region:>10} {mx:>11.1} {n:>10} {gm:>10.3}");
            }
        }
    }
    println!(
        "\n    Se 'seco' tiver degraus que 'papel' nao tem, a causa e a COSTURA entre os dois\n    \
         donos, nao a reconstrucao da silhueta.\n"
    );
}

/// O canvas DEPOIS do 1o traco e o canvas depois do 2o, com a MESMA semente — para perguntar o que
/// o 2o traco de fato mudou, e a que distancia do proprio eixo.
fn wash_two_stages(dilution: f32, smooth_edges: bool) -> (Vec<u8>, Vec<u8>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
    t.paint.brush = BrushSpec {
        radius_px: 26.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.90, 0.15, 0.18],
        space_attenuation: false,
        watercolor: true,
        smooth_edges,
        wet_dilution: dilution,
        fill: 0.45,
        depth: 2.0,
        edge_gain: 1.2,
        edge_spread: 6.0,
        opacity: 0.4,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.on_canvas_pointer(cp([24.0, 90.0], PointerPhase::Down));
    for i in 1..=16u8 {
        t.on_canvas_pointer(cp([24.0 + f32::from(i) * 13.0, 90.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([232.0, 90.0], PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(16.0);
    }
    let after_one = t.canvas_rgba.as_ref().clone();

    let cx = f32::from(u16::try_from(SIDE / 2).unwrap_or(128));
    t.on_canvas_pointer(cp([cx, 30.0], PointerPhase::Down));
    for i in 1..=16u8 {
        t.on_canvas_pointer(cp([cx, 30.0 + f32::from(i) * 12.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([cx, 222.0], PointerPhase::Up));
    t.on_tick(16.0);
    (after_one, t.canvas_rgba.as_ref().clone())
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_how_far_the_second_stroke_reaches() {
    println!(
        "\n=== ATE ONDE O 2o TRACO ALCANCA (raio 26 => a tinta acaba por volta de 26 px) ===\n"
    );
    println!(
        "{:<30} {:>28}",
        "config", "|delta| max por faixa de distancia"
    );
    println!(
        "{:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "", "0-20", "20-30", "30-40", "40-60", "60-90", ">90"
    );

    let cx = (SIDE / 2) as i32;
    let rows: Vec<usize> = (72..=108).collect(); // sobre a faixa SECA
    for dil in [0.0f32, 0.45] {
        for aa in [true, false] {
            let (a, b) = wash_two_stages(dil, aa);
            let mut band = [0i32; 6];
            for &y in &rows {
                for x in 0..SIDE as usize {
                    let i = (y * SIDE as usize + x) * 4;
                    let d = (0..3)
                        .map(|c| (i32::from(a[i + c]) - i32::from(b[i + c])).abs())
                        .max()
                        .unwrap_or(0);
                    let dist = (x as i32 - cx).abs();
                    let k = match dist {
                        0..=19 => 0,
                        20..=29 => 1,
                        30..=39 => 2,
                        40..=59 => 3,
                        60..=89 => 4,
                        _ => 5,
                    };
                    band[k] = band[k].max(d);
                }
            }
            let label = format!("Dilution {dil:.2} - AA {}", if aa { "on " } else { "off" });
            println!(
                "{label:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
                band[0], band[1], band[2], band[3], band[4], band[5]
            );
        }
    }
    println!(
        "\n    Um traco de raio 26 tem o direito de mudar pixels ate ~30 px do eixo.\n    \
         Tudo alem disso e o 2o traco reescrevendo tinta que ele nao tocou.\n"
    );
}
