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

/// **O que a divisão do LOTE compra com o cap de Accumulate ligado** — a medição que a wave de
/// 2026-08-04 deve, e a que decide se ela vale o código.
///
/// Os dois irmãos acima medem o KERNEL (um dab por vez). Este mede o degrau de cima: o lote inteiro
/// de dabs que um re-stamp entrega de uma vez. Até esta wave o cap **excluía** o lote da rota em
/// banda — a cláusula morava no predicado `plain`, com o racional *"estado compartilhado (a máscara
/// canvas-shaped)"*, que é sobre DABS e não sobre LINHAS.
///
/// ## A pergunta é *quando* isto paga, e a resposta é o número de dabs no lote
///
/// A rota abre threads, e abrir threads custa. O `BATCH_MIN_AREA` existe para que um lote pequeno
/// não pague esse custo — então a tabela varre o TAMANHO do lote e mostra os dois lados do piso.
///
/// ⚠️ **A régua é `visitas` (a soma das pegadas), nunca o raio** — a lei que o doc-comment do
/// [`super::stamp_banded::batch_work`] fixa: um log que traz `dabs` e omite o trabalho convida a uma
/// aritmética que assume um raio.
///
/// ## Como o A/B é feito, e por que assim
///
/// As duas rotas são cronometradas **costas-com-costas DENTRO da mesma corrida**, alternadas, sobre a
/// MESMA tela restaurada — a técnica do doc 28 §5.46. Esta máquina divide 32 núcleos com outras
/// linhas, e um A/B entre corridas atribuiria a carga ao ganho (a deriva medida lá foi de 2×, com o
/// MESMO binário).
///
/// ⚠️ E a metade (A) existe porque *o número que vira decisão de produto tem de sair da porta do
/// produto* (§5.40): ela dirige o `on_canvas_pointer` de um editor de figura com o cap ligado e lê o
/// contador do próprio depósito. As `visitas/lote` dela reconciliam com a coluna `visitas` de (B) —
/// é isso que prova que a fixture sintética está na escala certa.
///
/// ## Medido na RTX, máquina CALMA (`load average 0,65`), 2026-08-04
///
/// **(A) o produto TOMA a rota:** `9 lote(s) em BANDA · 0 serial(is)`, 1 693 853 visitas/lote,
/// **2,05 ns/visita** — e o escopo ao lado, sem o qual o número é inatribuível: um quadro de re-stamp
/// é `restore 3,31 · relevo 0,00 · save 6,41 · CARIMBO 39,36 ms` ⇒ **o carimbo é 80% do quadro**.
///
/// **(B) o A/B, `serial ÷ banda`** (raio 40, onde o kernel NÃO divide sozinho):
///
/// | dabs | visitas | vs piso | ganho (traço) | ganho (figura) | ganho (figura, cap off) |
/// |---|---|---|---|---|---|
/// | 2 | 13 778 | abaixo | 0,58× | 0,58× | 0,52× |
/// | 4 | 27 556 | abaixo | **1,10×** | **1,02×** | 0,93× |
/// | 8 | 55 112 | abaixo | 2,01× | 2,18× | 2,18× |
/// | 16 | 110 224 | abaixo | 3,77× | 3,09× | 3,52× |
/// | 32 | 220 448 | ACIMA | 6,22× | 6,28× | 6,57× |
/// | 128 | 874 322 | ACIMA | 11,29× | 10,79× | 10,78× |
/// | 512 | 3 612 420 | ACIMA | 11,50× | 10,20× | 11,06× |
///
/// A coluna `cap off` é o **CONTROLE**: ela acompanha a com cap linha a linha ⇒ a máscara não é o que
/// faz a rota pagar. E com raio **200** — onde a pegada de UM dab (400² = 160 000) cruza o piso do
/// kernel e a rota serial já é ela própria paralela — a banda **ainda** ganha `1,64× → 4,80×`: ela
/// paga **um** spawn onde a serial paga `n`.
///
/// No ponto de operação do produto (1,69 M visitas/lote) a tabela diz **~10×**.
///
/// ## ⚠️ E a metade (C) inverteu a conclusão que eu ia escrever
///
/// A (B) mostra o lote pagando **2,0× a 3,8× ABAIXO do piso**, e a leitura natural é *"o piso do LOTE
/// está alto; desacople-o do kernel"*. A (C) mede o piso do **kernel** — que nunca tinha sido medido,
/// ele nasceu escolhido — e ele tem **o mesmo break-even**: um dab de pegada 33 489 já paga `1,33×`
/// e um de 67 081 paga `2,64×`, os dois abaixo de 131 072.
///
/// ⇒ O doc-comment do [`BATCH_MIN_AREA`] está **CERTO** (*"é o mesmo número, e isso é deliberado: a
/// pergunta não muda por quem a faz"*). Quem está errado é **o número**, e para os dois. Desacoplar
/// teria consertado metade do defeito e enterrado a outra debaixo de uma justificativa.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release what_the_banded_batch -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "measurement, not a gate — run explicitly with --test-threads=1"]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn what_the_banded_batch_buys_when_the_cap_is_on() {
    use super::stamp_banded::{
        BATCH_MIN_AREA, batch_bounds, batch_work, diag, stamp_plain_dabs_banded_with, wants_bands,
    };
    use ph2d_painter_brush::{BrushSpec, Dab, StrokeMethod};
    use std::time::Instant;

    const SIZE: u32 = 2048;
    const SAMPLES: u32 = 9;
    const DAB_R: f32 = 40.0;

    fn median(v: &mut [f64]) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        v[v.len() / 2]
    }

    // ─── (A) a porta do PRODUTO ──────────────────────────────────────────────────────────────
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.set_brush_size_px(DAB_R * 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.paint.brush.strength = 0.5; // o cap, pela porta que o artista mexe
    for slot in &mut t.paint.brush_by_mode {
        slot.strength = 0.5;
        slot.stroke_method = StrokeMethod::Ellipse;
    }
    assert!(
        t.stroke_cover_wanted(&t.paint.brush),
        "a fixture TEM de ligar o cap, senão ela mede o mundo de antes"
    );

    let _ = diag::take();
    t.on_canvas_pointer(cp([524.0, 524.0], PointerPhase::Down));
    for i in 1..=8u32 {
        let g = 60.0 * f64::from(i) as f32;
        t.on_canvas_pointer(cp([1524.0 + g, 1524.0 + g], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([2004.0, 2004.0], PointerPhase::Up));
    let d = diag::take();
    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "a fixture não pintou um pixel — ela não contém o fenômeno"
    );
    let lotes = d.banded + d.serial;
    println!("\n(A) A PORTA DO PRODUTO — elipse re-carimbada, cap LIGADO, canvas {SIZE}²\n");
    println!(
        "    {} lote(s) em BANDA · {} serial(is) · {} dabs · {} visitas · {:.2} ms de CPU",
        d.banded,
        d.serial,
        d.dabs,
        d.visits,
        d.cpu_us as f64 / 1e3,
    );
    if lotes > 0 && d.visits > 0 {
        println!(
            "    {:.0} visitas/lote · {:.2} ns/visita",
            d.visits as f64 / f64::from(lotes),
            d.cpu_us as f64 * 1e3 / d.visits as f64,
        );
    }
    // ⚠️ **O ESCOPO do número, sem o qual ele é inatribuível.** Um quadro de re-stamp tem quatro
    // fases, e acelerar o carimbo 10x só move o quadro se o carimbo for a maior. A razão sai daqui.
    let frame_us = d.restore_us + d.relief_us + d.save_us + d.stamp_us;
    if d.deliveries > 0 && frame_us > 0 {
        println!(
            "    {} entrega(s) de re-stamp: restore {:.2} · relevo {:.2} · save {:.2} · CARIMBO \
             {:.2} ms  ⇒  o carimbo é {:.0}% do quadro",
            d.deliveries,
            d.restore_us as f64 / 1e3,
            d.relief_us as f64 / 1e3,
            d.save_us as f64 / 1e3,
            d.stamp_us as f64 / 1e3,
            d.stamp_us as f64 * 100.0 / frame_us as f64,
        );
    }

    // ─── (B) o A/B costas-com-costas ─────────────────────────────────────────────────────────
    // ⚠️ **A GEOMETRIA do lote é parte da fixture, e a primeira versão desta sonda errou nela.** Ela
    // punha `n` dabs num anel de raio FIXO, então um lote de 2 dabs saía com os dois em lados opostos
    // e um bbox de 1080 LINHAS — e bandas dividem LINHAS. Nenhum lote real tem essa forma: dois dabs
    // consecutivos distam um espaçamento. As duas formas abaixo são as que o produto de fato entrega,
    // as duas com o MESMO espaçamento (10% do diâmetro, o default do pincel):
    //   · TRAÇO  — dabs em fila, o lote de uma entrega de mão livre (bbox baixo e largo);
    //   · FIGURA — dabs fechando um anel, o re-stamp de um editor de figura (bbox alto).
    fn dab_at(p: [f32; 2], d: [f32; 2], r: f32) -> Dab {
        Dab {
            center: p,
            radius_px: r,
            coverage: 0.6,
            color: [0.1, 0.2, 0.8],
            rotation: [1.0, 0.0],
            dir: d,
            arc_len: 0.0,
            stroke_radius_px: r,
        }
    }
    // ⚠️ **O RAIO é o segundo eixo, e sem ele a tabela decide errado sobre o piso.** A rota serial
    // chama o kernel por dab, e o kernel tem o piso DELE (`PARALLEL_MIN_AREA = 131 072`, sobre a
    // pegada de UM dab): com raio 40 a pegada é 80² = 6400 e ele **nunca** divide (serial de
    // verdade), com raio 200 é 400² = 160 000 e ele **divide sozinho** — ali a rota em banda disputa
    // com um adversário JÁ paralelo, e o ganho tem de ser outro. Uma tabela de um raio só afirmaria
    // o primeiro regime como se fosse o único.
    //
    // ⚠️ E a primeira versão desta linha dizia `raio 150 ⇒ o kernel divide`: **falso**, 300² = 90 000
    // fica ABAIXO do piso. O segundo regime só começa em raio ~182, e eu teria publicado duas faixas
    // do mesmo regime afirmando que eram dois.
    fn batch(figure: bool, n: usize, r: f32) -> Vec<Dab> {
        let c = SIZE as f32 * 0.5;
        let step = r * 2.0 * 0.1; // o espaçamento do produto: 10% do diâmetro
        if figure {
            // O anel cujo PERÍMETRO comporta os `n` dabs nesse espaçamento.
            let ring = (n as f32) * step / std::f32::consts::TAU;
            (0..n)
                .map(|i| {
                    let a = (i as f32) / (n as f32) * std::f32::consts::TAU;
                    let (s, co) = (a.sin(), a.cos());
                    dab_at([c + co * ring, c + s * ring], [-s, co], r)
                })
                .collect()
        } else {
            (0..n)
                .map(|i| dab_at([c + (i as f32) * step, c], [1.0, 0.0], r))
                .collect()
        }
    }

    let pristine = vec![255u8; (SIZE * SIZE * 4) as usize];
    let mut buf = pristine.clone();
    let mut mask = vec![0u8; (SIZE * SIZE) as usize];
    let brush = BrushSpec {
        radius_px: DAB_R,
        color: [0.1, 0.2, 0.8],
        ..BrushSpec::default()
    };

    println!(
        "\n(B) A/B COSTAS-COM-COSTAS — mesma tela restaurada, alternado, mediana de {SAMPLES}\n    \
         piso do lote = {BATCH_MIN_AREA} visitas · piso do KERNEL = 131072 (pegada de UM dab)\n"
    );
    println!(
        "{:>5} {:>7} {:>5} {:>11} {:>7} {:>6} {:>11} {:>11} {:>8}",
        "raio", "forma", "dabs", "visitas", "bandas", "cap", "serial ms", "banda ms", "ganho"
    );
    // ⚠️ A lista de `n` é POR FAIXA porque o lote tem de caber na tela: com raio 200 o passo é 40 px,
    // e 128 dabs em fila medem 5120 px num canvas de 2048 — o `dab_write_bounds` clampa, as
    // `visitas` SATURAM e a linha deixa de conter o fenômeno que ela diz medir.
    for (r, figure, capped, ns) in [
        (DAB_R, false, true, &[2usize, 4, 8, 16, 32, 128, 512][..]),
        (DAB_R, true, true, &[2, 4, 8, 16, 32, 128, 512][..]),
        (DAB_R, true, false, &[2, 4, 8, 16, 32, 128, 512][..]),
        (200.0f32, false, true, &[2, 4, 8, 16, 32][..]),
    ] {
        let brush = BrushSpec {
            radius_px: r,
            ..brush
        };
        for n in ns.iter().copied() {
            let dabs = batch(figure, n, r);
            let work = batch_work(&dabs, SIZE, SIZE);
            assert!(
                wants_bands(&dabs, SIZE, SIZE, 0),
                "com piso 0 o lote de {n} dabs TEM de tomar a banda, senão as duas colunas são a \
                 mesma rota e a razão é 1,00x por construção"
            );
            assert!(
                !wants_bands(&dabs, SIZE, SIZE, usize::MAX),
                "com piso usize::MAX o lote TEM de ficar serial — é a rota de ablação"
            );

            let (mut band, mut ser) = (Vec::new(), Vec::new());
            for _ in 0..SAMPLES {
                for (dst, min_area) in [(&mut band, 0usize), (&mut ser, usize::MAX)] {
                    buf.copy_from_slice(&pristine);
                    mask.fill(0);
                    let m = if capped { Some(&mut mask[..]) } else { None };
                    let t0 = Instant::now();
                    let _ = stamp_plain_dabs_banded_with(
                        &mut buf, SIZE, SIZE, &dabs, &brush, false, m, min_area,
                    );
                    dst.push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            let (b, s) = (median(&mut band), median(&mut ser));
            // ⚠️ **A contagem de BANDAS é publicada, não assumida.** Desde que ela passou a sair do
            // TRABALHO, "o piso deixou passar" e "o lote de fato se dividiu" são frases diferentes —
            // e uma tabela que imprimisse só a primeira mostraria `1,00x` sem dizer que as duas
            // colunas rodaram a mesma rota.
            let nb = ph2d_painter_brush::band_count(
                work,
                batch_bounds(&dabs, SIZE, SIZE).map_or(0, |b| b.h as usize),
                BATCH_MIN_AREA,
            );
            println!(
                "{r:>5.0} {:>7} {n:>5} {work:>11} {nb:>7} {:>6} {s:>11.3} {b:>11.3} {:>7.2}x",
                if figure { "figura" } else { "traco" },
                if capped { "ON" } else { "off" },
                s / b.max(1e-9),
            );
        }
    }
    let _ = diag::take(); // o A/B carimbou pela porta instrumentada; não deixe o balde sujo
    println!(
        "\nganho > 1 = a banda paga · a linha `cap off` é o CONTROLE (a máscara não é o que faz pagar)\n"
    );

    // ─── (C) o piso do KERNEL, medido pela primeira vez ───────────────────────────────────────
    // ⚠️ **Sem esta metade a (B) decide errado.** Ela mostra que o lote tem break-even MUITO abaixo
    // do piso — e a conclusão natural (*"o piso do lote está alto"*) só se sustenta se o piso do
    // KERNEL estiver certo. Se ele também estiver alto, o número é UM e é o número que está errado,
    // não a partilha. O `PARALLEL_MIN_AREA` nunca foi medido: ele nasceu escolhido.
    println!("(C) O PISO DO KERNEL — UM dab, a mesma ablação, mediana de {SAMPLES}\n");
    println!(
        "{:>5} {:>11} {:>8} {:>11} {:>11} {:>8}",
        "raio", "pegada", "bandas", "serial ms", "banda ms", "ganho"
    );
    for r in [20.0f32, 40.0, 64.0, 90.0, 128.0, 181.0, 256.0, 362.0] {
        let one = vec![dab_at(
            [SIZE as f32 * 0.5, SIZE as f32 * 0.5],
            [1.0, 0.0],
            r,
        )];
        let spec = BrushSpec {
            radius_px: r,
            ..brush
        };
        let work = batch_work(&one, SIZE, SIZE);
        let (mut par, mut ser) = (Vec::new(), Vec::new());
        for _ in 0..SAMPLES {
            for (dst, min_area) in [(&mut par, 0usize), (&mut ser, usize::MAX)] {
                buf.copy_from_slice(&pristine);
                mask.fill(0);
                let t0 = Instant::now();
                // ⚠️ A porta do KERNEL, não a do lote: um dab, e o piso dele ablacionado. Chamá-la
                // pelo lote mediria o piso do LOTE outra vez.
                let _ = ph2d_painter_brush::stamp_dab_textured_masked_with(
                    &mut buf,
                    SIZE,
                    SIZE,
                    one[0].center,
                    &spec,
                    one[0].coverage,
                    false,
                    Some(&mut mask[..]),
                    one[0].rotation,
                    min_area,
                );
                dst.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        let (p, s) = (median(&mut par), median(&mut ser));
        let nb = ph2d_painter_brush::band_count(
            work,
            (r * 2.0) as usize + 2,
            ph2d_painter_brush::PARALLEL_MIN_AREA,
        );
        println!(
            "{r:>5.0} {work:>11} {nb:>8} {s:>11.3} {p:>11.3} {:>7.2}x",
            s / p.max(1e-9),
        );
    }
    println!(
        "\nse o break-even do KERNEL também estiver muito abaixo de 131072, o número errado é UM\n"
    );
}
