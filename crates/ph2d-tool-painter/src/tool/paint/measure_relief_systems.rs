//! **AUDITORIA DE CUSTO dos DOIS sistemas de relevo** — o dente do PAPEL e o relevo do DEPÓSITO
//! (Enio, 2026-08-10: *"há muita queda de FPS em traços rápidos; auditoria completa dos dois sistemas
//! buscando otimização"*).
//!
//! ⚠️ **Toda coluna sai de uma porta do PRODUTO** — `on_canvas_pointer` para o que um evento custa,
//! `apply_impasto_light` para o que um quadro custa. Uma sonda com laço próprio mede o que ela mesma
//! escreveu, e esta linha já pagou isso duas vezes (`§5.11`, `§5.46`).
//!
//! ⚠️ **A ablação é por KNOB, nunca por instrumentação** — o que se liga e desliga é exatamente o que o
//! artista liga e desliga, então uma linha da tabela não pode medir um caminho que o produto não toma.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_relief -- --ignored --nocapture`
//!
//! # O VEREDITO (medido 2026-08-10, e a leitura INVERTEU duas vezes pelo caminho)
//!
//! **O papel é GRÁTIS num traço; quem custa é o depósito.** Traço de 600 px FIXOS, o sétimo, raio 20:
//!
//! | config | tela | pen-down | move | pen-up |
//! |---|---|---|---|---|
//! | nada | 2048 | 0,63 | **0,37** | 0,44 |
//! | só PAPEL | 2048 | 0,60 | **0,38** | 0,46 |
//! | só DEPÓSITO | 2048 | 4,61 | **1,39** | 2,95 |
//! | nada | 4096 | 3,57 | **0,38** | 2,71 |
//! | só PAPEL | 4096 | 3,59 | **0,38** | 2,93 |
//! | só DEPÓSITO | 4096 | 4,98 | **1,42** | 4,62 |
//!
//! * **O PAPEL não aparece em coluna nenhuma** — ele entra na LUZ, não no depósito, e a pista de GPU o
//!   dobra em **0,26-1,33 ms/quadro** sobre uma janela CONFINADA (medido: *zero* quadros full-canvas
//!   num traço de 16 eventos). O sistema do papel está são.
//! * **O DEPÓSITO custa 3,7× o Digital no MOVE** (0,38 → 1,40 ms) e o número é **PLANO na tela**, que é
//!   a forma correta: ele é limitado pela PEGADA. É esta linha que o artista sente como queda de FPS,
//!   porque um move acontece dezenas de vezes por segundo.
//! * **E o pen-down cobra 0,63 → 4,61 ms a 2048²** — um *hitch* no começo de cada traço.
//!
//! ## O mecanismo dos 3,7×, lido no código
//!
//! `film_depth > 0` torna `impasto_batch_active()` verdadeiro, então um traço **Digital** passa a rodar
//! `accumulate_dab_height` por dab — e esse passe **percorre a pegada do dab uma SEGUNDA vez,
//! re-avaliando a MESMA silhueta** que o depósito de cor acabou de avaliar, mais uma sub-amostragem de
//! AA na borda do filme. Não é ineficiência do kernel: é trabalho **repetido**.
//!
//! ## ⛔ MEDIDO E REJEITADO — não refaça: otimizar o laço de luz da CPU
//!
//! O laço de `apply_impasto_light` custa **95-114 ns/texel com papel contra 23 sem** (4,4×), porque a
//! diferença central chama `height_at` **4,00 vezes por texel** (contado, não cronometrado) e cada
//! chamada re-amostra o dente. A cura óbvia é uma janela deslizante de 3 linhas (∇h é linear, e ∇papel
//! é função pura da posição) — **4 amostragens viram 1**.
//!
//! **E ela não compra nada visível**, porque o produto quase não toma essa pista: o caminho que shipa é
//! o da GPU, cujo fold chama `height_at` **UMA** vez por texel (o shader faz a diferença central) e já é
//! `par_chunks_mut`. Eu media a PEÇA em vez do PRODUTO — a mesma lição que o §5.48 registra.
//!
//! ## ⚠️ E DUAS falhas de fixture minhas, cada uma invertendo a leitura
//!
//! 1. **O traço escalava com a tela** (`[side/4, 3·side/4]`), então uma tela 4× maior levava 2× mais
//!    dabs e a tabela reportava *"cresce com a tela"* sobre uma fixture que crescia junto (§5.13).
//! 2. **Eu media o PRIMEIRO traço**, que paga o *first-touch* dos ~200 MB de planos canvas-shaped que o
//!    commit aloca uma vez por documento: o pen-up media **29,52 ms**, e o do sétimo traço é **4,62**.
//!
//! # A CURA (2026-08-10) — e a metade do mecanismo acima que a medição DERRUBOU
//!
//! ⛔ **A fusão que este doc prescreveu NÃO existe, e a frase *"re-avaliando a MESMA silhueta"* está
//! errada.** O passe de altura de facto percorre a pegada uma segunda vez, mas o `t` que ele avalia é o
//! da **CÁPSULA** (o corpo varrido até o centro do dab anterior — a lei anti-corrugação de 2026-07-15)
//! enquanto a cor avalia o do **DISCO**: são dois números diferentes de propósito, e nenhum passe pode
//! consumir o do outro. O único ingrediente comum é a amostra da **Shape**, que não depende de `t` — e
//! o depósito de cor não a computa por texel tampouco: ele **BLITA um carimbo em cache**
//! (`stamp_cache`, a mesma cura, dez meses antes).
//!
//! **É essa a assimetria, e ela é de 15× por texel:** a cor faz 3,75 ns/texel (blit de cache) e a
//! altura 26-58 (re-derivação). Medido pela decomposição (raio 20, ablação por peça):
//!
//! | shape | passe | silhueta | filme (AA) | miolo+cauda |
//! |---|---|---|---|---|
//! | Stripes | 1,047 | **0,776 (74%)** | −0,014 | 0,270 |
//! | só falloff | 0,453 | **0,184 (41%)** | −0,001 | 0,269 |
//!
//! ⚠️ **A coluna `filme` é o CONTROLE INTERNO e ela tem de dar ~0:** `film_aa_wanted` exige o master do
//! impasto, que no regime do filme está desligado — as duas rotas do `if` são o mesmo `film_of(w)`.
//! Foi ela que reprovou a primeira corrida inteira (dizia **4,54 ms**, maior que o passe que a contém)
//! antes que eu pudesse acreditar nos outros números dela. *Uma tabela cuja coluna impossível não dá
//! zero é uma tabela que se joga fora, não uma que se interpreta.*
//!
//! ## O que decidiu a cura foi o RAIO
//!
//! O passe é **linear nos texels** e a cor é **plana** (o cache blita; o espaçamento cresce com o raio):
//!
//! | raio | digital | passe de altura |
//! |---|---|---|
//! | 20 | 0,38 | 1,02 |
//! | 60 | 0,36 | 3,40 |
//! | 100 | 0,30 | **4,67** |
//!
//! A 200 px de pincel um único move do mouse custa **4,96 ms, 94% dele o passe de altura** — 30% de um
//! quadro de 60 fps, dezenas de vezes por segundo. É esse o *"delay em traços rápidos"*.
//!
//! ## A cura: BANDAS, pela porta que o depósito de cor já usa
//!
//! `accumulate_dab_height` percorria em UMA thread o que a cor percorre em N ([`ph2d_painter_brush::dab::band_count`]).
//! As linhas são disjuntas, um escritor por texel, nenhum texel lê o vizinho ⇒ **byte-idêntico por
//! construção**, com gate de identidade contra a rota serial (`ablate::SERIAL`) e o controle que prova
//! que a fixture CRUZA o piso. Medido costas-com-costas na MESMA corrida (2048², ms por move):
//!
//! | shape | raio | serial | banda | ganho |
//! |---|---|---|---|---|
//! | Stripes | 20 | 1,307 | 0,880 | 1,49× |
//! | Stripes | 60 | 3,573 | 0,862 | 4,14× |
//! | Stripes | 100 | 4,617 | **0,763** | **6,05×** |
//! | só falloff | 60 | 1,572 | 0,567 | 2,77× |
//! | só falloff | 100 | 2,122 | 0,579 | 3,66× |
//!
//! ⚠️ **O piso é o do kernel de COR e isso é conservador de propósito** (ver o doc do
//! `walk_dab_rows`); e **a mordida do bow wave fica SERIAL**, porque `displaced` é uma soma em `f32`
//! cuja ordem o Enio aprovou olhando.
//!
//! ## ⚠️ E a máquina
//!
//! A primeira tabela do breakdown do papel dizia *"Roughness DOBRA o custo"* (1339 contra 660 ms) sob
//! `load average 37` — sete `rustc` de outras linhas. A corrida seguinte deu **422 contra 426**: ruído.
//! *Nenhum relógio desta workstation vale com o load acima de ~5*, e é por isso que o número central
//! desta auditoria (as 4,00 amostragens por texel) é uma CONTAGEM.

use super::measure_impasto_cost::{cp, ms};
use crate::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::TextureKind;

/// Os quatro pontos de operação que o artista alcança pelas duas seções.
#[derive(Clone, Copy)]
struct Cfg {
    name: &'static str,
    paper: bool,
    film: bool,
}

const CFGS: [Cfg; 4] = [
    Cfg {
        name: "nada",
        paper: false,
        film: false,
    },
    Cfg {
        name: "so PAPEL",
        paper: true,
        film: false,
    },
    Cfg {
        name: "so DEPOSITO",
        paper: false,
        film: true,
    },
    Cfg {
        name: "os DOIS",
        paper: true,
        film: true,
    },
];

fn tool(side: u32, c: Cfg) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([200, 30, 30]);
    t.set_brush_shape_kind(TextureKind::Stripes as u8);
    if c.paper {
        t.set_substrate_depth(1.0);
        t.set_substrate_roughness(0.5);
    }
    if c.film {
        t.set_shape_relief(1.0);
    }
    t
}

/// Um traço RÁPIDO — o que o Enio reportou: poucos eventos, muito caminho por evento.
fn fast_stroke(t: &mut PainterTool, side: u32) -> (f64, f64, f64) {
    let y = f32::from(u16::try_from(side / 2).unwrap_or(512));
    let x0 = y * 0.5;
    let x1 = y * 1.5;
    let down = ms(&mut || {
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    });
    let mut moves = Vec::new();
    // 16 eventos de 40 px — a assinatura de um traço rápido (um lento entrega ~1 px por evento).
    for i in 1..=16 {
        let x = x0 + (x1 - x0) * (f32::from(u8::try_from(i).unwrap_or(16)) / 16.0);
        moves.push(ms(&mut || {
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }));
    }
    let up = ms(&mut || {
        t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
    });
    moves.sort_by(f64::total_cmp);
    (down, moves[moves.len() / 2], up)
}

/// **O QUE UM EVENTO CUSTA** — pen-down, o move MEDIANO e o pen-up, por configuração e por tela.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_relief_stroke_cost() {
    println!("\n=== O QUE UM TRACO RAPIDO CUSTA (ms por evento, mediana dos 16 moves) ===\n");
    println!(
        "{:<14} {:>6} {:>9} {:>9} {:>9}",
        "config", "tela", "down", "move", "up"
    );
    for side in [2048u32, 4096] {
        for c in CFGS {
            let mut t = tool(side, c);
            let (down, mv, up) = fast_stroke(&mut t, side);
            println!("{:<14} {side:>6} {down:>9.2} {mv:>9.2} {up:>9.2}", c.name);
        }
    }
    println!();
}

/// **O QUE UM QUADRO CUSTA** — o passe de luz, sobre a janela que o traço suja e sobre a tela inteira.
///
/// ⚠️ **As duas colunas respondem perguntas diferentes:** a janela é o que um quadro de traço paga; a
/// tela inteira é o que uma mudança de knob paga (o dente cobre todo texel, então mexer no Relief do
/// papel derruba o confinamento — a lei do `573bc109b`).
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_relief_light_cost() {
    println!("\n=== O QUE A LUZ CUSTA (ms) ===\n");
    println!(
        "{:<14} {:>6} {:>11} {:>11} {:>11}",
        "config", "tela", "janela 256", "tela toda", "ns/texel"
    );
    for side in [2048u32, 4096] {
        for c in CFGS {
            let mut t = tool(side, c);
            fast_stroke(&mut t, side);
            let win = Region {
                x: side / 4,
                y: side / 2 - 128,
                w: 256,
                h: 256,
            };
            let mut buf = vec![200u8; (win.w * win.h * 4) as usize];
            // MEDIANA de sete: a 1ª amostra paga o first-touch do buffer e a resolução do slot, e a
            // tabela anterior mostrou o preço disso (23,5 contra 6,3 ms para a MESMA janela).
            let mut w7: Vec<f64> = (0..7)
                .map(|_| ms(&mut || t.apply_impasto_light(&mut buf, win)))
                .collect();
            w7.sort_by(f64::total_cmp);
            let a = w7[3];
            let full = Region {
                x: 0,
                y: 0,
                w: side,
                h: side,
            };
            let mut big = vec![200u8; (side as usize) * (side as usize) * 4];
            let b = ms(&mut || t.apply_impasto_light(&mut big, full));
            let ns = b * 1e6 / f64::from(side * side);
            let rect = t.preview_gpu_region();
            let shown = rect.map_or("TELA TODA".to_string(), |(_, _, w, h)| format!("{w}x{h}"));
            println!(
                "{:<14} {side:>6} {a:>11.3} {b:>11.2} {ns:>11.2}  dirty {shown}",
                c.name
            );
        }
    }
    println!();
}

/// **DE QUE O CUSTO DO PAPEL É FEITO** — ablação por knob, dentro do sistema do papel.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_paper_breakdown() {
    println!("\n=== O DENTE DO PAPEL, PECA A PECA (tela toda 2048, ms) ===\n");
    let side = 2048u32;
    let full = Region {
        x: 0,
        y: 0,
        w: side,
        h: side,
    };
    let run = |arm: &dyn Fn(&mut PainterTool)| -> f64 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
        arm(&mut t);
        let mut big = vec![200u8; (side as usize) * (side as usize) * 4];
        let mut best = f64::MAX;
        for _ in 0..3 {
            best = best.min(ms(&mut || t.apply_impasto_light(&mut big, full)));
        }
        best
    };
    let base = run(&|t| {
        t.set_substrate_depth(1.0);
        t.set_substrate_roughness(0.5);
    });
    println!("papel completo (Cold, Rough 0,5)          {base:>8.2}");
    for (label, arm) in [
        (
            "Roughness 0 (sem o ganho de ingremeza)",
            &(|t: &mut PainterTool| {
                t.set_substrate_depth(1.0);
                t.set_substrate_roughness(0.0);
            }) as &dyn Fn(&mut PainterTool),
        ),
        (
            "Paper Size 4 (a mesma tile, 4x maior)",
            &|t: &mut PainterTool| {
                t.set_substrate_depth(1.0);
                t.set_substrate_roughness(0.5);
                t.set_brush_paper_size(0, 4.0);
                t.set_brush_paper_size(1, 4.0);
            },
        ),
        (
            "sem papel (o CONTROLE: a luz nao corre)",
            &|_t: &mut PainterTool| {},
        ),
    ] {
        let v = run(arm);
        println!("{label:<41} {v:>8.2}");
    }
    println!();
}

/// **QUANTAS VEZES O DENTE É AMOSTRADO POR TEXEL** — o oráculo que a máquina não move.
///
/// ⚠️ Esta sonda existe porque o relógio desta workstation é COMPARTILHADO: a primeira tabela do
/// breakdown mediu *"Roughness dobra o custo do papel"* sob `load average 37` (sete `rustc` de outras
/// linhas), e a corrida seguinte devolveu 422 contra 426 ms — ruído. Uma CONTAGEM é um fato sobre o
/// código, e nenhuma carga a move.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_tooth_samples_per_texel() {
    use std::sync::atomic::Ordering;
    println!("\n=== QUANTAS AMOSTRAGENS DE PAPEL POR TEXEL ILUMINADO ===\n");
    for c in [CFGS[1], CFGS[3]] {
        let side = 512u32;
        let mut t = tool(side, c);
        fast_stroke(&mut t, side);
        let win = Region {
            x: 128,
            y: 128,
            w: 64,
            h: 64,
        };
        let mut buf = vec![200u8; (win.w * win.h * 4) as usize];
        super::substrate_relief::TOOTH_SAMPLES.store(0, Ordering::Relaxed);
        t.apply_impasto_light(&mut buf, win);
        let n = super::substrate_relief::TOOTH_SAMPLES.load(Ordering::Relaxed);
        let texels = u64::from(win.w) * u64::from(win.h);
        println!(
            "{:<14} {n} amostragens / {texels} texels = {:.2} por texel",
            c.name,
            n as f64 / texels as f64
        );
    }
    println!(
        "\n(a diferenca central le 4 vizinhos; o 5o seria o proprio texel, que a lei NAO amostra)"
    );
}

/// **O QUADRO DO PRODUTO** — o que a pista de GPU dobra a cada evento de um traço rápido.
///
/// ⚠️ **Esta é a porta que decide o FPS**, e nenhuma das tabelas acima a atravessava: quem escolhe a
/// janela do fold é o `take_preview_dirty` (que chama o `reconcile_substrate`) e não o laço da luz.
/// Um traço que confina publica um retângulo; um que não confina manda dobrar a TELA.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_frame_the_gpu_lane_folds() {
    use std::sync::atomic::Ordering;
    println!("\n=== O QUADRO DA PISTA DE GPU (por evento de um traco rapido) ===\n");
    println!(
        "{:<14} {:>6} {:>10} {:>14} {:>12} {:>10}",
        "config", "tela", "quadros", "janela media", "amostras/qd", "ms/quadro"
    );
    for side in [2048u32, 4096] {
        for c in CFGS {
            let mut t = tool(side, c);
            let y = f32::from(u16::try_from(side / 2).unwrap_or(512));
            let (x0, x1) = (y * 0.5, y * 1.5);
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
            let (mut frames, mut area, mut samples, mut whole) = (0u64, 0f64, 0u64, 0u64);
            let mut total = 0.0f64;
            super::substrate_relief::TOOTH_SAMPLES.store(0, Ordering::Relaxed);
            for i in 1..=16u8 {
                let x = x0 + (x1 - x0) * (f64::from(i) / 16.0) as f32;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                // O QUADRO: drenar (que escolhe a janela) e dobrar (o que a shell faz).
                if t.take_preview_dirty() {
                    let win = t.preview_gpu_region().unwrap_or((0, 0, side, side));
                    if win == (0, 0, side, side) {
                        whole += 1;
                    }
                    area += f64::from(win.2) * f64::from(win.3);
                    frames += 1;
                    total += ms(&mut || {
                        let _ = t.impasto_gpu_planes_in(win);
                    });
                }
            }
            samples += super::substrate_relief::TOOTH_SAMPLES.load(Ordering::Relaxed);
            t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
            let f = frames.max(1) as f64;
            println!(
                "{:<14} {side:>6} {frames:>10} {:>14.0} {:>12.0} {:>10.2}   ({whole} TELA TODA)",
                c.name,
                area / f,
                samples as f64 / f,
                total / f
            );
        }
    }
    println!();
}

/// **DE QUE O CUSTO DO DEPÓSITO É FEITO** — o move e o pen-up, contra o RAIO e contra a TELA.
///
/// ⚠️ **As duas colunas separam as duas doenças possíveis:** um custo que cresce com o RAIO é limitado
/// pela pegada (a forma correta de um move); um que cresce com a TELA é varredura de plano, e é o que
/// esta casa já curou quatro vezes.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_what_the_deposit_costs() {
    println!("\n=== O DEPOSITO: pegada ou plano? (ms) ===\n");
    println!(
        "{:<10} {:>7} {:>6} {:>9} {:>9}",
        "config", "raio", "tela", "move", "up"
    );
    for c in [CFGS[0], CFGS[2]] {
        for side in [1024u32, 2048, 4096] {
            for r in [20.0f32, 80.0] {
                let mut t = tool(side, c);
                t.set_brush_size_px(r * 2.0);
                let (_d, mv, up) = fast_stroke(&mut t, side);
                println!("{:<10} {r:>7.0} {side:>6} {mv:>9.2} {up:>9.2}", c.name);
            }
        }
    }
    println!("\n--- e o PEN-UP sem o commit de undo (a ablacao da §5.14) ---");
    for side in [2048u32, 4096] {
        let mut t = tool(side, CFGS[2]);
        let y = f32::from(u16::try_from(side / 2).unwrap_or(512));
        let (x0, x1) = (y * 0.5, y * 1.5);
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        for i in 1..=16u8 {
            let x = x0 + (x1 - x0) * (f64::from(i) / 16.0) as f32;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.paint.stroke_undo = None; // a MESMA ablacao por ENTRADA que a §5.14 usou
        let up = ms(&mut || {
            t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
        });
        println!("tela {side}: pen-up SEM o commit estrutural  {up:>8.2} ms");
    }
    println!();
}

/// **O TRAÇO N-ÉSIMO, COM COMPRIMENTO FIXO** — as duas correções de fixture que invertem a leitura.
///
/// ⚠️ **(1) O comprimento é FIXO em px.** A primeira versão punha o traço em `[side/4, 3·side/4]`, então
/// uma tela 4× maior levava um traço 2× mais longo — *mais dabs* — e a tabela reportava "cresce com a
/// tela" sobre uma fixture que crescia junto. É a lição do §5.13 (*traço FIXO em px*).
///
/// ⚠️ **(2) O traço medido é o SÉTIMO.** O primeiro paga o *first-touch* dos planos canvas-shaped que o
/// commit aloca (a 4096² são ~200 MB entre `heights`/`covers`/`mats`), e isso acontece **uma vez por
/// documento** — medi-lo como se fosse o preço de todo traço é o erro que o §5.13 documenta.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn measure_the_nth_stroke_at_fixed_length() {
    println!("\n=== O TRACO N-ESIMO, 600 px FIXOS (ms; mediana de 6 apos descartar o 1o) ===\n");
    println!(
        "{:<14} {:>7} {:>6} {:>9} {:>9} {:>9}",
        "config", "raio", "tela", "down", "move", "up"
    );
    for c in CFGS {
        for side in [2048u32, 4096] {
            let mut t = tool(side, c);
            t.set_brush_size_px(40.0);
            let cy = f32::from(u16::try_from(side / 2).unwrap_or(512));
            let (mut downs, mut movs, mut ups) = (Vec::new(), Vec::new(), Vec::new());
            for k in 0..7u8 {
                // Faixas paralelas, todas de 600 px: o comprimento nao pode ser funcao da tela.
                let y = cy + f32::from(k) * 6.0;
                let x0 = cy - 300.0;
                downs.push(ms(&mut || {
                    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
                }));
                let mut mv = Vec::new();
                for i in 1..=15u8 {
                    let x = x0 + 40.0 * f32::from(i);
                    mv.push(ms(&mut || {
                        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                    }));
                }
                mv.sort_by(f64::total_cmp);
                movs.push(mv[mv.len() / 2]);
                ups.push(ms(&mut || {
                    t.on_canvas_pointer(cp([x0 + 600.0, y], PointerPhase::Up));
                }));
            }
            // O 1o traco paga o first-touch dos planos: descartado, mediana dos seis restantes.
            for v in [&mut downs, &mut movs, &mut ups] {
                v.remove(0);
                v.sort_by(f64::total_cmp);
            }
            let m = |v: &Vec<f64>| v[v.len() / 2];
            println!(
                "{:<14} {:>7} {side:>6} {:>9.2} {:>9.2} {:>9.2}",
                c.name,
                20,
                m(&downs),
                m(&movs),
                m(&ups)
            );
        }
    }
    println!();
}

/// **DE QUE O MOVE DO FILME É FEITO** — a decomposição que decide a wave, na regime do REPORT.
///
/// A auditoria acima nomeou o alvo (*o depósito custa 3,7× o Digital no MOVE*) e escreveu o mecanismo
/// lido no código (*a pegada é percorrida uma SEGUNDA vez*). ⚠️ **Ler o código diz o QUE acontece, não
/// quanto custa** — e o irmão [`super::measure_impasto_cost::what_the_height_walk_is_made_of`] mediu a
/// mesma decomposição **noutro regime** (`DrawTo::Depth`: impasto ligado, bow wave vivo, AA do filme
/// ligado), onde a silhueta valia 14%. **No filme nada disso está ligado**, então aquela tabela não
/// responde por esta: aqui `depth == 0` e `push == 0` (sem mordida) e o
/// [`ph2d_painter_brush::height_film::FilmAa`] nasce `None` (`film_aa_wanted` exige o master do
/// impasto), o que faz da coluna `filme` um controle que TEM de dar ~0.
///
/// ⚠️ **Duas fixturas, porque a Shape muda o kernel:** com uma Shape ativa o `silhouette_at` é
/// `compose(sample_shape(px,py), falloff(t))` — uma amostra de textura por texel; sem ela é só o
/// `falloff_weight(t)`. A razão entre as duas colunas diz se o alvo da fusão é a AMOSTRA DA SHAPE (que
/// **não depende de `t`**, logo é literalmente o mesmo número nos dois passes) ou a caminhada.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release what_a_film_move_is_made_of -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn what_a_film_move_is_made_of() {
    use ph2d_painter_brush::ablate;

    /// O move MEDIANO do sétimo traço de 600 px, com a máscara de ablação armada.
    fn move_at(shape: bool, film: bool, mask: u32, radius: f32) -> f64 {
        const SIDE: u32 = 2048;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
        t.set_brush_size_px(radius * 2.0);
        t.set_brush_color_srgb8([200, 30, 30]);
        if shape {
            t.set_brush_shape_kind(TextureKind::Stripes as u8);
        }
        if film {
            t.set_shape_relief(1.0);
        }
        let cy = 1024.0f32;
        let mut per_stroke = Vec::new();
        for k in 0..7u8 {
            let y = cy + f32::from(k) * 6.0;
            let x0 = cy - 300.0;
            let mut mv = Vec::new();
            ablate::with(mask, || {
                t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
                for i in 1..=15u8 {
                    let x = x0 + 40.0 * f32::from(i);
                    mv.push(ms(&mut || {
                        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                    }));
                }
                t.on_canvas_pointer(cp([x0 + 600.0, y], PointerPhase::Up));
            });
            mv.sort_by(f64::total_cmp);
            per_stroke.push(mv[mv.len() / 2]);
        }
        per_stroke.remove(0); // o 1o traco paga o first-touch dos planos
        per_stroke.sort_by(f64::total_cmp);
        per_stroke[per_stroke.len() / 2]
    }

    let move_ms = |shape: bool, film: bool, mask: u32| move_at(shape, film, mask, 20.0);

    println!("\n=== A BANDA: SERIAL x DIVIDIDO, na MESMA corrida (2048, ms por move) ===\n");
    println!(
        "{:<12} {:>6} {:>9} {:>9} {:>8} {:>9}",
        "shape", "raio", "serial", "banda", "ganho", "digital"
    );
    for shape in [true, false] {
        for r in [20.0f32, 60.0, 100.0] {
            // ⚠️ Costas-com-costas DENTRO da corrida: esta workstation ja mediu o mesmo passo em 14,5
            // e 30,2 ms sem uma linha mudar (doc 28 §5.46), e um A/B entre corridas atribuiria a carga
            // da maquina ao ganho.
            let ser = move_at(shape, true, ablate::SERIAL, r);
            let par = move_at(shape, true, 0, r);
            let dig = move_at(shape, false, 0, r);
            println!(
                "{:<12} {r:>6.0} {ser:>9.3} {par:>9.3} {:>7.2}x {dig:>9.3}",
                if shape { "Stripes" } else { "so falloff" },
                ser / par.max(1e-9),
            );
        }
    }

    println!("\n=== O PASSE DE ALTURA CONTRA O RAIO (2048, ms por move) ===\n");
    println!(
        "{:<12} {:>6} {:>9} {:>9} {:>8} {:>8}",
        "shape", "raio", "digital", "full", "passe", "razao"
    );
    for shape in [true, false] {
        for r in [20.0f32, 60.0, 100.0, 200.0] {
            let digital = move_at(shape, false, 0, r);
            let full = move_at(shape, true, 0, r);
            println!(
                "{:<12} {r:>6.0} {digital:>9.3} {full:>9.3} {:>8.3} {:>7.2}x",
                if shape { "Stripes" } else { "so falloff" },
                full - digital,
                full / digital.max(1e-9),
            );
        }
    }

    println!("\n=== DE QUE O MOVE DO FILME E FEITO (2048, raio 20, ms por move) ===\n");
    println!(
        "{:<12} {:>8} {:>8} {:>8} {:>9} {:>8} {:>8}",
        "shape", "digital", "full", "passe", "silhueta", "filme", "miolo+cauda"
    );
    for shape in [true, false] {
        let digital = move_ms(shape, false, 0);
        let full = move_ms(shape, true, 0);
        let no_tail = move_ms(shape, true, ablate::TAIL);
        let no_sil = move_ms(shape, true, ablate::TAIL | ablate::SILHOUETTE);
        let no_film = move_ms(shape, true, ablate::TAIL | ablate::FILM_AA);
        println!(
            "{:<12} {digital:>8.3} {full:>8.3} {:>8.3} {:>9.3} {:>8.3} {:>11.3}",
            if shape { "Stripes" } else { "so falloff" },
            full - digital,
            no_tail - no_sil,
            no_tail - no_film,
            full - digital - (no_tail - no_sil),
        );
    }
    println!();
}
