//! **A BORDA DURA DENTRO DA LAVAGEM** — as sondas dos dois reports do Enio de 2026-08-11.
//!
//! ⚠️ **LEIA ISTO ANTES DOS NÚMEROS: as quatro primeiras rodadas destas sondas usavam
//! `Falloff::Constant` — um disco duro que o modo watercolor NÃO consegue produzir.** O falloff
//! deste modo é FIXO e é o [`ph2d_painter_brush::Falloff::Watercolor`] (planalto `1,0 → 0,92` até
//! `t ≤ 0,62`, depois rampa), e o Enio teve de dizê-lo depois de dois smokes gastos. Sobre o pincel
//! REAL (raio 72 = o Size 0,14 do slider) os efeitos que aquelas rodadas mediam **somem ou
//! invertem**: sobre pigmento seco o AA sai byte-idêntico ligado e desligado, e os "degraus" do
//! interior viram centenas nos DOIS modos (é o dente do papel, por desenho).
//!
//! **Os números que sobrevivem, e que decidiram a lei, são as impressões digitais** — o pincel real,
//! canvas inteiro, `Dilution 0,00` e `0,45`:
//!
//! ```text
//!   lei              Dilution 0,00        Dilution 0,45
//!   contínua         a6e16b9928c32f50     b4aca42f7e766705
//!   DURA (1ª rodada) 67a90771ef5720b0     929937a4cb8c423e  <- IDÊNTICO ao AA desligado
//!   sem portão       edefd4842174c3bb     f73c383eacbdc733
//!   AA DESLIGADO     2561a882650ff260     929937a4cb8c423e
//! ```
//!
//! Um portão DURO sobre o vão fecha em TODO texel — inclusive no aro — assim que a Dilution baixa a
//! cobertura, e o checkbox *Smooth Edges* vira **controle morto** exatamente em *"smooth edges +
//! dilution > 0"*. O gate que pina isso é
//! `watercolor_aa_tests::the_smooth_edges_checkbox_is_not_dead_under_dilution`.
//!
//! ⚠️ **Duas hipóteses minhas foram REFUTADAS por ablação e não devem voltar:** *"empurra pixels
//! PARA ALÉM das bordas"* é falsa no sentido literal (um 2º traço não muda um byte além de 30 px do
//! eixo — [`measure_how_far_the_second_stroke_reaches`]), e a metade **ÁGUA/backrun** da `Dilution`
//! (`water = dilution`, EDGE-2) é **inerte** aqui: ablacioná-la deixa os números byte-idênticos.
//! `WET_RAGGED`/`WET_EDGE_BOOST` também são inertes — respondem ao knob **Wet**, não ao Dilution.
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
        radius_px: 72.0,
        hardness: 1.0,
        falloff: Falloff::Watercolor,
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
        radius_px: 72.0,
        hardness: 1.0,
        falloff: Falloff::Watercolor,
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
        radius_px: 72.0,
        hardness: 1.0,
        falloff: Falloff::Watercolor,
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

// ---------------------------------------------------------------------------------------------
// A FRANJA PALIDA ADIANTE DA PONTA (a foto do Enio: "pigmento empurrado formando bordas pixeladas")
// ---------------------------------------------------------------------------------------------

/// Uma lavagem larga e, sobre ela, um traco CURTO que TERMINA dentro dela — a ponta e onde
/// `inner > cw` e o unsharp ASSINADO produz o lobo NEGATIVO (a franja palida: o pigmento migrou do
/// interior para a borda). E ela que a foto do Enio mostra, com contorno em blocos.
fn wash_with_a_tip(dilution: f32, smooth_edges: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
    t.paint.brush = BrushSpec {
        radius_px: 72.0,
        hardness: 1.0,
        falloff: Falloff::Watercolor,
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
    // A lavagem de fundo: uma faixa larga e baixa, que seca.
    t.on_canvas_pointer(cp([30.0, 170.0], PointerPhase::Down));
    for i in 1..=14u8 {
        t.on_canvas_pointer(cp([30.0 + f32::from(i) * 14.0, 170.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([226.0, 170.0], PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(16.0);
    }
    // O traco CURTO que sobe e PARA dentro dela — a ponta fica em y=70.
    t.on_canvas_pointer(cp([128.0, 168.0], PointerPhase::Down));
    for i in 1..=8u8 {
        t.on_canvas_pointer(cp([128.0, 168.0 - f32::from(i) * 12.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([128.0, 72.0], PointerPhase::Up));
    t.on_tick(16.0);
    t.canvas_rgba.as_ref().clone()
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_pale_fringe_at_a_stroke_tip() {
    println!("\n=== A FRANJA PALIDA ADIANTE DA PONTA (as setas pretas da foto) ===\n");
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "config", "franja pico", "grad max", "n degraus"
    );
    // A janela ao redor da PONTA do traco curto (que para em y=72).
    let rows: Vec<usize> = (40..=110).collect();
    let (wx0, wx1) = (80usize, 176usize);
    for dil in [0.0f32, 0.45] {
        for aa in [true, false] {
            let px = wash_with_a_tip(dil, aa);
            // A franja e PALIDA: mais clara que o corpo. O "pico" e o quanto ela clareia acima do
            // nivel do corpo da lavagem naquela linha; os degraus sao a aspereza do contorno dela.
            let (mut peak, mut gmax, mut steps) = (0.0f32, 0.0f32, 0usize);
            for &y in &rows {
                let prof = profile(&px, y);
                // Nivel do corpo: a mediana da faixa entintada da linha (o vermelho de fundo).
                let mut body: Vec<f32> =
                    (wx0..wx1).map(|x| prof[x]).filter(|&v| v < 250.0).collect();
                if body.len() < 16 {
                    continue;
                }
                body.sort_by(f32::total_cmp);
                let med = body[body.len() / 2];
                for x in wx0..wx1 {
                    if prof[x] >= 250.0 {
                        continue;
                    }
                    peak = peak.max(prof[x] - med);
                    let g = (prof[x + 1] - prof[x]).abs();
                    gmax = gmax.max(g);
                    if g > 6.0 {
                        steps += 1;
                    }
                }
            }
            let label = format!("Dilution {dil:.2} - AA {}", if aa { "on " } else { "off" });
            println!("{label:<30} {peak:>12.1} {gmax:>12.1} {steps:>12}");
        }
    }
    println!(
        "\n    'franja pico' = quanto a franja CLAREIA acima do corpo (o lobo negativo do unsharp).\n    \
         'grad max' = quao DURA e a borda dela. A foto mostra pico ALTO com borda DURA.\n"
    );
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_law_fingerprint() {
    // FNV-1a do canvas inteiro: com o pincel REAL, a lei do vão é observável ou não?
    let h = |px: &[u8]| {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &b in px {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    };
    println!("\n=== IMPRESSAO DIGITAL DA LEI (pincel REAL: Falloff::Watercolor, r=72) ===");
    for dil in [0.0f32, 0.45] {
        println!(
            "  Dilution {dil:.2}  AA on {:016x}   AA off {:016x}",
            h(&wash(dil, true)),
            h(&wash(dil, false))
        );
    }
}
