//! **Quem RODA o depósito, e quanto essa escolha custa** — irmão do [`super::measure_impasto_cost`].
//!
//! O corte é de responsabilidade: lá se mede *o que o CORPO da tinta custa* (o teto por texel, a
//! decomposição por knob, o AA do filme); aqui *qual caminho o lote toma e o que o caminho cobra* — uma
//! pergunta sobre agendamento, não sobre o modelo.
//!
//! ⚠️ **Ele nasceu de uma atribuição minha que estava errada** (2026-08-04): a primeira sonda mediu que
//! sair do ramo `plain` custa 3,3-4,3× e eu li isso como *a rota*. A segunda — o controle que atravessa
//! o piso de `PARALLEL_MIN_AREA` — mostrou que o mecanismo estava uma camada abaixo. As duas ficam,
//! nesta ordem, porque a lição é a sequência.

use super::PainterTool;
use super::measure_impasto_cost::{cp, ms};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// **Qual ROTA o lote toma — e quanto a rota custa.**
///
/// O log do artista (2026-08-04) traz, em toda janela de impasto, `stamps ... 8,18 ms cada` ao lado de
/// `deposito DEVICE: 0 lotes` **e** `deposito CPU: 0 em BANDA + 0 serial(is)`. Os dois contadores em
/// zero não significam *"o depósito foi barato"*: significam que o lote **não passou pela porta que os
/// conta** — o carimbo mais caro do app é o único estruturalmente excluído das duas rotas rápidas.
///
/// ## A cadeia, e ela é mecânica
///
/// `impasto_smooth_edges` (ligado por padrão) + `deposits_height()` ⇒ `film_aa_wanted` ⇒
/// [`super::PainterTool::stroke_cover_wanted`] ⇒ `accumulate_cap` ⇒ e o `accumulate_cap` está **dentro
/// do predicado `plain`**, que é a porta da rota em BANDA (`thread::scope`, uma faixa de linhas por
/// núcleo) e do DISPOSITIVO. Um pincel de impasto cai, por construção, no laço `for d in dabs` de uma
/// thread só.
///
/// ⚠️ **E a exclusão é EFEITO COLATERAL de uma correção, não uma decisão sobre velocidade.** O cap
/// existe porque um texel de aro fracionário tem de parar na fração de área dele (BUGS #16, medido
/// 0,64 → 0,94); ninguém escolheu com isso tirar o impasto do paralelo — a cláusula entrou no `plain`
/// porque `stamp_plain_dabs_banded` não recebe a máscara, que é limitação de **assinatura**.
///
/// ## O par de CONTROLE é a espinha desta tabela
///
/// As linhas 1 e 2 são o MESMO pincel digital, o MESMO caminho, a MESMA lista de dabs, e diferem em
/// **`strength 1,00` contra `0,99`** — um número que não muda o trabalho por texel e que **vira o
/// `stroke_cover_wanted`**, logo a rota. A razão entre elas é o preço de perder a banda, isolado de
/// tudo o mais. As linhas 3 e 4 dão a magnitude do lado do impasto, pela outra porta de produto
/// (`Smooth Edges`) — ⚠️ e essa **muda a arte** (105.660 bytes, pior delta 62, medido pelo irmão
/// `does_the_film_aa_change_a_pixel`), então ela mede *quanto a rota vale*, nunca *uma economia grátis*.
///
/// A rota é OBSERVADA no contador do produto, não deduzida: `banda/serial` saem do
/// [`super::stamp_banded::diag`], então uma linha que eu tenha classificado errado se denuncia — e é
/// ele que mostra que os lotes desta fixture têm **menos de 2 dabs**, logo ficam seriais mesmo no
/// ramo `plain` (`dabs.len() < 2`). ⚠️ **O que esta tabela mede, portanto, não é paralelismo: é o
/// KERNEL.** O ganho de banda que o log do artista mostra (`107 em BANDA`) vem POR CIMA disto.
///
/// ## ⚠️ A leitura de 2026-08-04 desta tabela estava ERRADA, e o irmão a corrigiu
///
/// Medido ANTES da correção, a tabela dizia `9,8 / 42,2 / 91,5 / 51,6` a 1024² e
/// `14,7 / 48,1 / 124,5 / 77,2` a 4096² — um par de controle de **4,3× e 3,3×** —, e eu atribuí isso à
/// **ROTA**, porque a cláusula que muda é a que decide `plain`. O `x-tela` refutava corretamente a
/// explicação canvas-shaped, e aí eu parei cedo demais.
///
/// **Com um pincel de falloff puro as duas linhas caem no MESMO `stamp_dabs_per_pixel` e chamam o
/// MESMO kernel** — a rota genérica não tem laço próprio. O mecanismo estava uma camada abaixo, e quem
/// o achou foi o controle que atravessa o piso de `PARALLEL_MIN_AREA`
/// ([`does_the_cap_cost_arithmetic_or_parallelism`]): **abaixo dele o cap custa 0,99×; acima, 4,15×** —
/// a máscara desligava o paralelismo por-dab do kernel. *Uma ablação atribui a um BLOCO; atribuir a uma
/// LINHA dentro dele é inferência de segunda ordem* (doc 28 §5.44), e eu o fiz duas vezes no mesmo dia.
///
/// ## Medido na RTX DEPOIS da correção (⚠️ máquina em load 13-14; leia as RAZÕES, não os absolutos)
///
/// | linha | 1024² | 4096² | x-tela | rota |
/// |---|---|---|---|---|
/// | 1 digital, strength 1,00 (cap OFF) | 10,7 | 15,0 | 1,41 | `plain` |
/// | 2 digital, strength 0,99 (cap ON) | **10,8** | **17,6** | 1,63 | genérica |
/// | 3 impasto, Smooth Edges ON (cap ON) | **68,8** | **101,4** | 1,47 | genérica |
/// | 4 impasto, Smooth Edges OFF (cap OFF) | 55,0 | 84,5 | 1,54 | `plain` |
///
/// O par de controle fecha em **1,01× e 1,17×** (era 4,3× e 3,3×), e as linhas 1 e 4 — cujo caminho não
/// mudou — ficam dentro do ruído da máquina. **O impasto do produto sai de 124,5 para 101,4 ms**, no
/// mesmo pincel e com a mesma arte: a correção é **byte-idêntica**, gateada em
/// `crates/ph2d-painter-brush/src/dab/band_mask_tests.rs`.
///
/// ⚠️ **O que SOBRA entre a linha 3 e a 4 (~17 ms) é o TRABALHO do AA do filme, não rota** — 54% do
/// traço pelo irmão [`the_height_walk_layers`] —, e é decisão de PRODUTO (a única cura restante é
/// aproximação por gradiente). A tabela não o promete de graça.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release which_route -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "measurement, not a gate — run explicitly with --test-threads=1"]
fn which_route_does_a_batch_take_and_what_does_the_route_cost() {
    const RADIUS: f32 = 100.0;
    const DIST: f32 = 700.0;
    const MOVES: u32 = 20;
    /// ⚠️ **A mesma GEOMETRIA em duas telas.** Um kernel é limitado pela PEGADA, logo plano no
    /// tamanho do canvas; a máscara do cap é canvas-shaped, logo escala com ele. A coluna `x-tela`
    /// separa as duas explicações rivais sem precisar de um cronômetro confiável.
    const SIDES: [u32; 2] = [1024, 4096];

    fn stroke(t: &mut PainterTool, y: f32) -> f64 {
        let x0 = RADIUS + 20.0;
        ms(&mut || {
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
            for i in 1..=MOVES {
                let x = x0 + DIST / f64::from(MOVES) as f32 * f64::from(i) as f32;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x0 + DIST, y], PointerPhase::Up));
        })
    }

    println!(
        "\nraio {RADIUS:.0}, traco de {DIST:.0} px, MESMA geometria nas duas telas — mediana de 3\n"
    );
    println!(
        "{:<40} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "linha", "1024² ms", "4096² ms", "x-tela", "banda", "serial"
    );
    for (name, impasto, strength, smooth_edges) in [
        ("1 digital, strength 1.00 (cap OFF)", false, 1.0f32, true),
        ("2 digital, strength 0.99 (cap ON)", false, 0.99, true),
        ("3 impasto, Smooth Edges ON (cap ON)", true, 1.0, true),
        ("4 impasto, Smooth Edges OFF (cap OFF)", true, 1.0, false),
    ] {
        let mut cols = [0.0f64; 2];
        let mut diag = super::stamp_banded::diag::DepositDiag::default();
        for (ci, side) in SIDES.iter().copied().enumerate() {
            let mut runs = Vec::new();
            for _ in 0..3u32 {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
                if impasto {
                    t.toggle_brush_impasto();
                }
                t.set_brush_size_px(RADIUS * 2.0);
                t.paint.brush.strength = strength;
                t.paint.brush.impasto_smooth_edges = smooth_edges;
                for slot in &mut t.paint.brush_by_mode {
                    slot.strength = strength;
                    slot.impasto_smooth_edges = smooth_edges;
                }
                let _ = super::stamp_banded::diag::take(); // a janela descreve ESTE traço
                runs.push(stroke(&mut t, 400.0));
                diag = super::stamp_banded::diag::take();
            }
            runs.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            cols[ci] = runs[1];
        }
        println!(
            "{name:<40} {:>10.1} {:>10.1} {:>8.2} {:>8} {:>8}",
            cols[0],
            cols[1],
            cols[1] / cols[0].max(1e-9),
            diag.banded,
            diag.serial,
        );
    }
    println!(
        "\nx-tela ~1 = limitado pela PEGADA (kernel) · x-tela grande = canvas-shaped (a mascara do cap)\n"
    );
}

/// **O cap custa ARITMÉTICA ou PARALELISMO?** — e a resposta muda a cura inteira.
///
/// O irmão [`which_route_does_a_batch_take_and_what_does_the_route_cost`] mede que sair do ramo `plain`
/// custa 3,3-4,3×, e eu atribuí isso à ROTA. ⚠️ **Está errado, e as duas rotas provam-no:** com um
/// pincel de falloff puro as DUAS caem em `stamp_dabs_per_pixel` e chamam o MESMO kernel
/// (`stamp_dab_textured_masked`) com os mesmos argumentos — a única diferença viva é a máscara. O
/// mecanismo está uma camada abaixo, escrito no `dab.rs`:
///
/// ```text
/// Some(mask) => stamp_band(&ctx, region, Some(mask_region), y0),   // SERIAL, uma banda
/// None       => parallel_band_stamp(buf, y0, y1, x0, x1, ...),     // linhas entre os núcleos
/// ```
///
/// ⚠️ **E a premissa que o comentário dele declara é justamente a que o impasto falsificou:** *"small
/// soft-brush dabs anyway, where the cap is observable"*. Isso valia quando o cap só disparava em
/// `strength < 1`; o **AA do filme** o liga para TODO pincel de impasto, inclusive os maiores do app.
///
/// ## O oráculo é o PISO, não o relógio
///
/// `parallel_band_stamp` só divide acima de `PARALLEL_MIN_AREA = 131 072` px de bbox de dab. Então:
///
/// * **raio 200** (bbox 400² = 160 000, ACIMA) — a rota sem máscara paraleliza, a com máscara não;
/// * **raio 100** (bbox 200² = 40 000, ABAIXO) — as DUAS são seriais.
///
/// Se o vão é a aritmética do cap, ele sobrevive nos dois raios. Se é o paralelismo perdido, ele
/// **encolhe** abaixo do piso — e a razão dentro de cada raio é medida na mesma corrida, o que a torna
/// imune à carga da máquina.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release does_the_cap_cost -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "measurement, not a gate — run explicitly with --test-threads=1"]
fn does_the_cap_cost_arithmetic_or_parallelism() {
    const SIZE: u32 = 2048;
    const DIST: f32 = 700.0;
    const MOVES: u32 = 20;
    const FLOOR: usize = 1 << 17;

    fn stroke(t: &mut PainterTool, radius: f32) -> f64 {
        let x0 = radius + 20.0;
        ms(&mut || {
            t.on_canvas_pointer(cp([x0, 900.0], PointerPhase::Down));
            for i in 1..=MOVES {
                let x = x0 + DIST / f64::from(MOVES) as f32 * f64::from(i) as f32;
                t.on_canvas_pointer(cp([x, 900.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x0 + DIST, 900.0], PointerPhase::Up));
        })
    }

    println!(
        "\ncanvas {SIZE}², traco de {DIST:.0} px — mediana de 3, piso do kernel = {FLOOR} px\n"
    );
    println!(
        "{:<7} {:>10} {:>8} {:>12} {:>12} {:>9}",
        "raio", "bbox dab", "vs piso", "cap OFF ms", "cap ON ms", "razao"
    );
    for radius in [100.0f32, 200.0] {
        let side = (radius * 2.0) as usize;
        let bbox = side * side;
        let mut col = [0.0f64; 2];
        for (ci, strength) in [1.0f32, 0.99].iter().copied().enumerate() {
            let mut runs = Vec::new();
            for _ in 0..3u32 {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
                t.set_brush_size_px(radius);
                t.paint.brush.strength = strength;
                for slot in &mut t.paint.brush_by_mode {
                    slot.strength = strength;
                }
                runs.push(stroke(&mut t, radius));
            }
            runs.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            col[ci] = runs[1];
        }
        let over = if bbox >= FLOOR { "ACIMA" } else { "abaixo" };
        println!(
            "{radius:<7.0} {bbox:>10} {over:>8} {:>12.1} {:>12.1} {:>9.2}x",
            col[0],
            col[1],
            col[1] / col[0].max(1e-9)
        );
    }
    println!();
}
