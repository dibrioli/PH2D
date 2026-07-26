//! **O que o IMPASTO custa** — a frente I do plano 26 §9, e o caminho de medições que a fechou.
//!
//! Irmão do `measure_input_cost`, e o corte é de responsabilidade: lá se mede **o que um EVENTO de
//! ponteiro custa** (pen-down, move, coalescência); aqui **o que o CORPO da tinta custa** (o teto por
//! texel, a decomposição por knob, o AA do filme).
//!
//! ⚠️ Nenhuma sonda aqui é gate. Todas imprimem, todas são `#[ignore]`, e as tabelas nos doc-comments
//! são os números medidos na RTX — re-meça antes de citar.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool com impasto ligado pela PORTA do produto — o caso do produto desde 2026-07-13.
fn tool(side: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.toggle_brush_impasto();
    t.set_brush_size_px(24.0);
    t
}

fn ms(f: &mut dyn FnMut()) -> f64 {
    let t0 = std::time::Instant::now();
    f();
    t0.elapsed().as_secs_f64() * 1e3
}

/// **O TETO: quanto custa um texel de dab — e o impasto custa 19x o digital.**
///
/// A pergunta que decide se há melhoria a colher, e ela não tinha número. Medido a 2048², traço de
/// 600 px:
///
/// | raio | impasto | ns/texel | ms/traço |
/// |---|---|---|---|
/// | 100 | **off** | **9,6** | 9,0 |
/// | 100 | **ON** | **182,8** | **172,2** |
/// | 20 | off | 55,5 | 10,5 |
/// | 20 | ON | 238,9 | 45,0 |
///
/// **Duas leituras, e as duas são sobre o MECANISMO** (a razão entre ligado e desligado usa a mesma
/// contagem de dabs, então ela não depende da derivação analítica de `dabs`):
///
/// 1. **O impasto custa 19x o digital por texel** a raio 100 (4,3x a raio 20). Ele escreve 12 bytes a
///    mais por texel (`heights` f32 + `covers` u8 + `mats` 7xu8) contra os 4 do RGBA, ou seja **4x o
///    tráfego de memória** — e a distância entre 4x e 19x é a **folga candidata**. Um traço de raio
///    100 com impasto custa **172 ms**; o mesmo traço digital custa **9**.
/// 2. **O pincel digital a raio 100 está em 9,6 ns/texel** — eficiente. A raio 20 sobe para 55,5,
///    porque o custo FIXO por dab é amortizado sobre 25x menos texels: pincel pequeno é dominado por
///    overhead de dab, pincel grande por trabalho de texel. **São dois regimes, e uma frente que ataca
///    um não ajuda o outro.**
///
/// ⚠️ **O que este número NÃO diz, e a coluna `x-piso` é a armadilha:** o piso medido é um
/// `d = s + 1` byte a byte, que o compilador vetoriza — é um piso de **largura de banda**, não de
/// **trabalho**. Um dab legitimamente faz falloff, silhueta, grain, jitter, blend, gate de proteção,
/// envelope de altura, cápsula, material e simetria por texel. Então `3154x o piso` **não é uma
/// afirmação de desperdício**; a única comparação honesta aqui é impasto contra digital, porque as duas
/// rotas fazem o mesmo trabalho de pigmento e diferem exactamente no relevo.
///
/// ⚠️ **E a frente que sai disto ainda NÃO está justificada** — é a lição da frente C revertida (§8 do
/// plano 26), tomada a sério: eu tenho a razão 19x, e **não** tenho a decomposição de para onde ela vai
/// dentro do dab de impasto. A próxima medição é essa, e ela é barata: cronometrar as passadas do
/// depósito de altura separadamente (o envelope, a cápsula, o `settle`, o banco do Push) sobre a MESMA
/// pegada. Sem ela, "otimizar o impasto" é a mesma frase que "coalescer os eventos" era ontem.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_per_texel -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_per_texel_cost_of_a_dab_against_the_hardware_floor() {
    const DIST: f32 = 600.0;
    println!("[texel] raio  impasto   ms/traco  dabs   ns/texel   x-piso");
    for radius in [20.0f32, 100.0] {
        for impasto in [false, true] {
            let mut t = tool(2048);
            if !impasto {
                t.toggle_brush_impasto(); // `tool()` o liga; aqui desligamos p/ o digital puro
            }
            t.set_brush_size_px(radius * 2.0);
            // Aquece (o 1o traco paga a alocacao dos planos por-traco).
            for _ in 0..2 {
                t.on_canvas_pointer(cp([200.0, 300.0], PointerPhase::Down));
                for i in 1..=24u8 {
                    let x = 200.0 + DIST / 24.0 * f32::from(i);
                    t.on_canvas_pointer(cp([x, 300.0], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([200.0 + DIST, 300.0], PointerPhase::Up));
            }
            let ms_stroke = ms(&mut || {
                t.on_canvas_pointer(cp([200.0, 700.0], PointerPhase::Down));
                for i in 1..=24u8 {
                    let x = 200.0 + DIST / 24.0 * f32::from(i);
                    t.on_canvas_pointer(cp([x, 700.0], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([200.0 + DIST, 700.0], PointerPhase::Up));
            });
            // Dabs: o espacamento default e fracao do diametro, entao derivamos do arco/spacing real.
            let spacing_px = f64::from(radius * 2.0) * f64::from(t.brush_settings().spacing);
            let dabs = (f64::from(DIST) / spacing_px.max(1.0)).max(1.0);
            let texels = dabs * std::f64::consts::PI * f64::from(radius) * f64::from(radius);
            let ns_per_texel = ms_stroke * 1e6 / texels;
            // PISO: uma passada de leitura+escrita sobre o mesmo numero de texels (4 B/px cada).
            let n = texels as usize;
            let src = vec![7u8; n * 4];
            let mut dst = vec![0u8; n * 4];
            let floor_ms = ms(&mut || {
                for (d, s) in dst.iter_mut().zip(&src) {
                    *d = s.wrapping_add(1);
                }
            });
            let floor_ns = floor_ms * 1e6 / texels;
            println!(
                "[texel] {radius:>4}  {:>7}  {ms_stroke:>8.2}  {dabs:>4.0}  {ns_per_texel:>8.1}  {:>5.1}x",
                if impasto { "ON" } else { "off" },
                ns_per_texel / floor_ns.max(1e-9)
            );
        }
    }
}

/// **A DECOMPOSIÇÃO do 19x: o alvo está LOCALIZADO, o mecanismo NÃO.**
///
/// Medido a 2048², raio 100, traço de 600 px (ms por traço):
///
/// | config | 1º (camada virgem) | 2º (sobre tinta) |
/// |---|---|---|
/// | impasto **OFF** (controle) | 11,6 | 11,6 |
/// | impasto ON, smoothing 1,0 (default) | **130,3** | **162,6** |
/// | impasto ON, smoothing 0,0 | 125,1 | 156,8 |
///
/// **Três leituras, e uma não-leitura:**
///
/// 1. **O `settle` vale 3%** (5 ms de 162). Não é o alvo, e a nota que o suspeitava está corrigida.
/// 2. **A família do bow wave vale ~20%** (162 − 130 = 32 ms) — e ela entra pelo `ground`, que é
///    `Some` sempre que a **CAMADA** tem relevo, não quando o knob Push está alto. O comentário do
///    `impasto.rs` afirma *"o custo cai exactamente onde a feature está, em tinta sobre tinta"*, e a
///    medição CONFIRMA: 130 na camada virgem, 162 a partir do 2º traço.
///    ⚠️ Isto invalida qualquer sonda que aqueça no MESMO lugar onde mede — a minha primeira fazia
///    isso e media o caso tinta-sobre-tinta enquanto o cabeçalho dizia "traço".
/// 3. **O resto — ~118 ms, 73% — é o depósito de altura base**, e é ele o alvo.
/// 4. ⚠️ **O que eu NÃO consegui estabelecer:** *por que* esses 118 ms. Ver o irmão
///    `the_impasto_draw_to_split`, cujo termo "superaditivo" desmanchou quando a sonda ganhou medianas.
///    Nomear um mecanismo aqui seria a terceira hipótese não-medida do dia (§8 do plano 26).
///
/// Levers, todos portas do PRODUTO (nenhuma instrumentação):
/// * `impasto` off -> o controle (pigmento puro, mesma lista de dabs);
/// * **1o traço numa camada virgem** -> `ground = None`, logo o banco/bite/onda **não rodam**;
/// * **2o traço na mesma camada** -> `ground = Some`, e a familia do bow wave entra;
/// * `impasto_smoothing = 0` -> mata o `settle` (o blur que assenta a tinta).
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_impasto_dab_decomposition -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_impasto_dab_decomposition() {
    const DIST: f32 = 600.0;
    fn stroke(t: &mut PainterTool, y: f32) -> f64 {
        ms(&mut || {
            t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
            for i in 1..=24u8 {
                let x = 200.0 + DIST / 24.0 * f32::from(i);
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([200.0 + DIST, y], PointerPhase::Up));
        })
    }
    println!("[decomp] raio 100, 2048², traco de 600 px — ms por traco");
    println!("[decomp] config                              1o(virgem)  2o(sobre tinta)  3o");
    for (name, impasto, smoothing) in [
        ("impasto OFF (controle)", false, 1.0f32),
        ("impasto ON, smoothing 1.0 (default)", true, 1.0),
        ("impasto ON, smoothing 0.0", true, 0.0),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
        if impasto {
            t.toggle_brush_impasto();
        }
        t.set_brush_size_px(200.0);
        t.paint.brush.impasto_smoothing = smoothing;
        for slot in &mut t.paint.brush_by_mode {
            slot.impasto_smoothing = smoothing;
        }
        // Os TRES na MESMA faixa: o 2o e o 3o encontram o relevo do anterior.
        let a = stroke(&mut t, 400.0);
        let b = stroke(&mut t, 400.0);
        let c = stroke(&mut t, 400.0);
        println!("[decomp] {name:<38} {a:>8.2}  {b:>13.2}  {c:>8.2}");
    }
}

/// **O 2º NÍVEL: dos 118 ms do depósito base, quanto é PIGMENTO e quanto é ALTURA?**
///
/// O `DrawTo` é a porta do produto que separa as duas metades sobre a MESMA pegada e a MESMA lista de
/// dabs: `Color` = pigmento sem corpo, `Depth` = corpo sem pigmento, `ColorAndDepth` = o default.
///
/// Medido a 2048², 1º traço em faixa virgem, **mediana de 5**:
///
/// | raio | Color | Depth | soma | ColorAndDepth | sobra |
/// |---|---|---|---|---|---|
/// | 10 | 6,8 | 24,5 | 31,3 | 32,0 | +2% |
/// | 25 | 10,5 | 33,6 | 44,1 | 47,4 | +8% |
/// | 50 | 20,1 | 45,9 | 66,0 | 72,1 | +9% |
/// | 100 | 9,0 | 111,6 | 120,7 | 160,8 | +33% |
///
/// **O que É robusto:** o depósito de ALTURA custa **2,3× a 12× o de pigmento** sobre a mesma lista de
/// dabs. Ele escreve 12 B/texel (`heights` f32 + `covers` u8 + `mats` 7×u8) contra os 4 do RGBA — 3× os
/// bytes — e faz a **própria varredura** da pegada, separada da varredura da cor.
///
/// ⚠️ **O que NÃO é robusto, e eu quase o publiquei como achado:** a primeira rodada desta sonda dava
/// UMA amostra por config e mostrava um termo **superaditivo de 40%** (as duas metades juntas custando
/// mais que a soma), que eu ia atribuir a **localidade de cache**. Com mediana de 5 e faixas
/// independentes ele **desmancha para 2-9%** nos raios 10-50; só o raio 100 mantém 33%, e ali o `Color`
/// sai **não-monotônico** (9,0 ms a raio 100 contra 20,1 a raio 50, com 2× os texels), o que denuncia a
/// própria medição. **Não há termo superaditivo estabelecido.**
///
/// ## A candidata, e por que ela NÃO é palpite de mecanismo
///
/// Fundir as duas varreduras numa só (silhueta avaliada UMA vez por texel, os 5 planos escritos juntos)
/// não é uma teoria sobre cache — é **remover uma varredura duplicada que se sabe existir**. E há
/// PRECEDENTE medido no próprio `impasto.rs`: o comentário do `PushBite` registra que fazer a mordida
/// *"num kernel de si mesma significava avaliar a silhueta DUAS vezes por texel, e isso sozinho punha o
/// custo do impasto em 5,0 ms/move, além do orçamento"*. O mesmo argumento, um nível acima.
///
/// ⚠️ É refatoração do kernel mais quente do módulo, com **byte-identidade** como gate. Wave própria,
/// e só com ordem do Enio.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_impasto_draw_to_split -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_impasto_draw_to_split() {
    use ph2d_painter_brush::height::DrawTo;
    const DIST: f32 = 600.0;
    println!("[split] 2048², 1o traco numa camada VIRGEM (sem bow wave) — ms");
    for radius in [10.0f32, 25.0, 50.0, 100.0] {
        println!("[split] --- raio {radius} ---");
        for (name, draw_to) in [
            ("Color (pigmento sem corpo)", DrawTo::Color),
            ("Depth (corpo sem pigmento)", DrawTo::Depth),
            ("ColorAndDepth (o default)", DrawTo::ColorAndDepth),
        ] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
            t.toggle_brush_impasto();
            t.set_brush_size_px(radius * 2.0);
            t.paint.brush.impasto_draw_to = draw_to;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_draw_to = draw_to;
            }
            // ⚠️ MEDIANA de 5, e cada traço numa faixa PRÓPRIA: a 1ª versão desta sonda dava UMA amostra
            // por config e o Color a raio 100 saiu ABAIXO do de raio 50 (não-monotônico), o que tornava o
            // termo "superaditivo" indistinguível de ruído. E as faixas têm de ser distintas, senão o 2º
            // traço encontra o relevo do 1º e o bow wave entra na conta (o `ground` é da CAMADA).
            let mut samples = Vec::new();
            for k in 0..5u8 {
                let y = 200.0 + f32::from(k) * 300.0;
                samples.push(ms(&mut || {
                    t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
                    for i in 1..=24u8 {
                        let x = 200.0 + DIST / 24.0 * f32::from(i);
                        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                    }
                    t.on_canvas_pointer(cp([200.0 + DIST, y], PointerPhase::Up));
                }));
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let ms_stroke = samples[samples.len() / 2];
            println!(
                "[split] {name:<30} {ms_stroke:>8.2}   (min {:.2} max {:.2})",
                samples[0],
                samples[samples.len() - 1]
            );
        }
    }
}

/// **O 3º NÍVEL — e o achado que fecha a frente I: o AA do filme é 54% do traço.**
///
/// Medido a 2048², traço de 600 px, camada virgem, **mediana de 5** (min/max apertados: 124-128 vs
/// 55-60, então isto não é ruído como o "superaditivo" da sonda irmã):
///
/// | | AA on | AA OFF | o AA custa | % do total |
/// |---|---|---|---|---|
/// | raio 5 | 21,7 | 12,3 | 9,4 | 43% |
/// | raio 15 | 29,2 | 20,4 | 8,8 | 30% |
/// | raio 40 | 55,6 | 37,7 | 17,9 | 32% |
/// | **raio 100** | **127,1** | **58,4** | **68,7** | **54%** |
///
/// E por rota, a raio 100: `Color`-only 9,2/8,8 · `Depth`-only 79,2/48,3 · **produto 130,6/54,7**. O
/// `film_aa_wanted` exige `deposits_height()`, então em `Color`-only o AA do pigmento está DESLIGADO —
/// é por isso que a linha dele parece grátis. No produto os dois pagam: ~31 ms o filme, **~38 ms o
/// pigmento**.
///
/// ## ⛔ E as DUAS curas óbvias morreram, cada uma por uma medição
///
/// **(1) Fundir as varreduras** (a ordem que abriu esta sessão) — **premissa FALSA.** O doc do
/// `height_film` diz que o AA dá *"a MESMA fração que o `dab.rs` dá ao pigmento"*, e lido o código as
/// duas closures supersampleiam **geometrias diferentes**: o pigmento amostra o **DISCO**
/// (`falloff_t(dx·inv_r, dy·inv_r)`, `dab/bands.rs:168`) e a altura amostra a **CÁPSULA VARRIDA**
/// (`sweep_residual(dx+ox, dy+oy, sweep)`, `height.rs`). Não são o mesmo número, então não há fração a
/// compartilhar. *"A caller's own swept-silhouette chain"* — a frase do próprio doc — já dizia que cada
/// chamador passa a SUA geometria, de propósito.
///
/// **(2) Gatear o AA por raio** — **não é grátis.** A hipótese era que a raio 100 o supersampling
/// calcula a média de uma rampa de 12,5 texels que já é suave (o argumento que o próprio
/// `height_film.rs` usou para estreitar a banda de 28 para 12,5 e ganhar 2 ms). Medido pelo irmão
/// `does_the_film_aa_change_a_pixel`: o AA muda **105.660 bytes com pior delta 62** a raio 100, sobre
/// 334.028 px pintados. Não é sub-quantum — é ~1/3 dos texels em até 62 níveis. Desligá-lo **muda a
/// arte**, e isso é decisão do Enio, não uma remoção de desperdício.
///
/// ## A avenida que SOBRA
///
/// Tornar o AA **mais barato pelo mesmo número**: a fração de área de um campo escalar suave tem
/// estimativa **analítica** pelo gradiente (`cobertura ≈ clamp(0,5 − f/|∇f|)`, o truque padrão de AA de
/// SDF), que custa ~2 amostras extras em vez das ~9 da grade sub-texel. ⚠️ É **aproximação**, então
/// entra com orçamento de épsilon declarado e gate de bytes-divergentes — o mesmo template da paridade
/// CPU×GPU da luz. Decisão de produto: **não construída sem ordem.**
///
/// ## A sonda (levers, todos knobs de produto)
///
/// Levers, todos knobs de produto sobre o MESMO traco Depth-only (sem pigmento, sem bow wave):
/// * `impasto_smooth_edges` -> o AA do filme (supersampling da banda do aro);
/// * `spacing` -> o comprimento da CAPSULA (cada dab varre de volta ao centro do anterior);
/// * `texture` (Grain) desligado -> um ingrediente a menos por texel.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_height_walk_layers -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_height_walk_layers() {
    use ph2d_painter_brush::height::DrawTo;
    const DIST: f32 = 600.0;
    println!("[layers] 2048², raio 100, Depth-only, camada virgem — mediana de 5 (ms)");
    for (name, aa, spacing, draw, radius) in [
        ("r=  5 AA on ", true, 0.10f32, DrawTo::ColorAndDepth, 5.0f32),
        ("r=  5 AA OFF", false, 0.10, DrawTo::ColorAndDepth, 5.0),
        ("r= 15 AA on ", true, 0.10, DrawTo::ColorAndDepth, 15.0),
        ("r= 15 AA OFF", false, 0.10, DrawTo::ColorAndDepth, 15.0),
        ("r= 40 AA on ", true, 0.10, DrawTo::ColorAndDepth, 40.0),
        ("r= 40 AA OFF", false, 0.10, DrawTo::ColorAndDepth, 40.0),
        ("r=100 AA on ", true, 0.10, DrawTo::ColorAndDepth, 100.0),
        ("r=100 AA OFF", false, 0.10, DrawTo::ColorAndDepth, 100.0),
    ] {
        let mut samples = Vec::new();
        for k in 0..5u8 {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
            t.toggle_brush_impasto();
            t.set_brush_size_px(radius * 2.0);
            t.paint.brush.impasto_draw_to = draw;
            t.paint.brush.impasto_smooth_edges = aa;
            t.paint.brush.spacing = spacing;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_draw_to = draw;
                slot.impasto_smooth_edges = aa;
                slot.spacing = spacing;
            }
            let y = 200.0 + f32::from(k) * 300.0;
            samples.push(ms(&mut || {
                t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
                for i in 1..=24u8 {
                    let x = 200.0 + DIST / 24.0 * f32::from(i);
                    t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([200.0 + DIST, y], PointerPhase::Up));
            }));
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        println!(
            "[layers] {name}  {:>8.2}   (min {:.2} max {:.2})",
            samples[2], samples[0], samples[4]
        );
    }
}

/// **O AA custa 54% do traco a raio 100 — ele MUDA algum pixel?**
///
/// A metade numerica do render-and-look: o MESMO traco com `impasto_smooth_edges` on e off, contando
/// quantos bytes divergem e qual o pior delta. Se a raio grande a diferenca for sub-quantum, o
/// supersampling esta calculando a media de uma rampa que ja e suave -- que e exactamente o argumento
/// que o proprio `height_film.rs` usou para estreitar a banda de 28 para 12,5 texels.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release does_the_film_aa_change_a_pixel -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn does_the_film_aa_change_a_pixel() {
    const DIST: f32 = 600.0;
    println!("[aa] raio  bytes divergentes  pior delta  (de um traco de 600 px a 2048²)");
    for radius in [5.0f32, 15.0, 40.0, 100.0] {
        let mut out = Vec::new();
        for aa in [true, false] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
            t.toggle_brush_impasto();
            t.set_brush_size_px(radius * 2.0);
            t.paint.brush.impasto_smooth_edges = aa;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_smooth_edges = aa;
            }
            t.on_canvas_pointer(cp([200.0, 400.0], PointerPhase::Down));
            for i in 1..=24u8 {
                let x = 200.0 + DIST / 24.0 * f32::from(i);
                t.on_canvas_pointer(cp([x, 400.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([200.0 + DIST, 400.0], PointerPhase::Up));
            out.push(t.canvas_rgba.as_ref().clone());
        }
        let diff = out[0].iter().zip(&out[1]).filter(|(a, b)| a != b).count();
        let worst = out[0]
            .iter()
            .zip(&out[1])
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        let painted = out[0].iter().step_by(4).filter(|&&v| v != 255).count();
        println!("[aa] {radius:>4}  {diff:>17}  {worst:>10}   ({painted} px pintados)");
    }
}

/// **A PREMISSA DA LUT: o caro dentro do AA e o `sil_at` ou o resto?**
///
/// Eu afirmei "o caro e o sil_at, nao o film_of" num doc-comment SEM medir. A LUT pre-convoluida da
/// secao 9.6 do plano 26 inteira depende dessa frase, entao ela vai ser medida antes.
///
/// O metodo: rodar `film_at_exact` com a closure REAL (o produto) e com uma closure que devolve um
/// valor pre-calculado -- as 9 chamadas de `film_of` e o laco ficam, so a cadeia de silhueta sai. Se os
/// dois custarem o mesmo, a cadeia nao e o custo e a LUT nao tem o que economizar.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release is_the_silhouette_chain_the_aa_cost -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn is_the_silhouette_chain_the_aa_cost() {
    use ph2d_painter_brush::height_film::FilmAa;
    use ph2d_painter_brush::{BrushSpec, Falloff};
    const N: usize = 3_000_000;
    println!("[lut] falloff     cadeia REAL   closure const   razao");
    for falloff in [Falloff::Smooth, Falloff::Sphere, Falloff::Constant] {
        let radius = 100.0f32;
        let s = BrushSpec {
            radius_px: radius,
            falloff,
            impasto: true,
            impasto_depth: 0.5,
            impasto_smooth_edges: true,
            ..Default::default()
        };
        let aa = FilmAa::for_dab(&s, false, radius).expect("banda a r=100");
        let fp = s.dab_footprint([1.0, 0.0]);
        let inv = 1.0 / radius;
        // Um texel DENTRO da banda -- achado por varredura, porque os limites da banda sao privados
        // (e um `t` fora dela mediria o early-out, nao a grade). O criterio e o proprio produto: na
        // banda o `film_at_exact` difere do single-sample.
        let probe = |t: f32| {
            let d = t * radius;
            let sil = s.falloff_weight(t);
            let nine = aa.film_at_exact(t, sil, |ox, oy| {
                s.falloff_weight(fp.falloff_t((d + ox) * inv, oy * inv))
            });
            (sil, nine)
        };
        let mut mid_t = 0.0f32;
        for k in 1..2000u32 {
            let t = f32::from(u16::try_from(k).unwrap_or(u16::MAX)) / 2000.0;
            let (sil, nine) = probe(t);
            if (nine - ph2d_painter_brush::height_film::film_of(sil)).abs() > 1e-6 {
                mid_t = t;
                break;
            }
        }
        assert!(
            mid_t > 0.0,
            "controle: nao achei texel na banda de {falloff:?}"
        );
        let d = mid_t * radius; // distancia radial do texel
        let sil = s.falloff_weight(mid_t);
        let mut acc = 0.0f32;
        let real = ms(&mut || {
            for k in 0..N {
                let jitter = (k % 7) as f32 * 0.01;
                acc += aa.film_at_exact(mid_t, sil, |ox, oy| {
                    s.falloff_weight(fp.falloff_t((d + ox + jitter) * inv, (oy) * inv))
                });
            }
        });
        let konst = ms(&mut || {
            for k in 0..N {
                let jitter = (k % 7) as f32 * 0.01;
                acc += aa.film_at_exact(mid_t, sil, |_ox, _oy| sil + jitter * 1e-6);
            }
        });
        println!(
            "[lut] {falloff:<10?}  {real:>10.1} ms  {konst:>12.1} ms  {:>5.2}x   (acc {acc:.0})",
            real / konst.max(1e-9)
        );
    }
}

/// **O que os 5,1 ms do RELEVO no pen-down SÃO** (doc 28 §4.4).
///
/// A pergunta que decide a próxima wave: aquele custo é **setup canvas-proporcional** (os cinco planos
/// do traço, que são do tamanho da TELA) ou é o **trabalho do primeiro dab** (que é do tamanho da
/// PEGADA)? Os dois são invisíveis um ao outro num número só.
///
/// A separação é o raio: varrendo o pincel de 10 a 100 sobre a MESMA tela, um custo canvas-proporcional
/// fica **plano** e um custo de pegada cresce com **r²** — 100× entre as pontas. É a mesma régua dos
/// gates de razão do módulo, e ela é imune à deriva da máquina porque compara duas medições da mesma
/// corrida.
#[test]
#[ignore]
fn is_the_relief_pen_down_the_planes_or_the_dab() {
    use ph2d_painter_brush::height::DrawTo;
    println!("[relief-pd] pen-down do RELEVO por raio — plano = os PLANOS, r² = o DAB");
    for size in [2048u32, 4096] {
        let mut row = Vec::new();
        for radius in [10.0f32, 25.0, 50.0, 100.0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            t.toggle_brush_impasto();
            t.paint.brush.impasto_draw_to = DrawTo::Depth;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_draw_to = DrawTo::Depth;
            }
            t.set_brush_size_px(radius * 2.0);
            let c = size as f32 * 0.5;
            // Aquece: o 1º traço da camada é o único que aloca; medimos os SEGUINTES, que é o regime.
            t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
            t.on_canvas_pointer(cp([c - 260.0, c], PointerPhase::Up));
            let mut best = f64::MAX;
            for k in 1..=4u8 {
                let y = c + 40.0 * f64::from(k) as f32;
                let v = ms(&mut || {
                    t.on_canvas_pointer(cp([c - 300.0, y], PointerPhase::Down));
                });
                t.on_canvas_pointer(cp([c - 260.0, y], PointerPhase::Up));
                best = best.min(v);
            }
            row.push((radius, best));
        }
        let txt = row
            .iter()
            .map(|(r, v)| format!("r{r:.0} {v:.2}"))
            .collect::<Vec<_>>()
            .join(" | ");
        let ratio = row[3].1 / row[0].1;
        println!("[relief-pd] {size}^2  {txt}  =>  r100/r10 = {ratio:.2}x (pegada previa 100x)");
    }
}

/// **O DELAY DO PRIMEIRO TRAÇO** (Enio, 2026-07-26: *"bem melhor e bastante aceitável com exceção do
/// delay do primeiro traço"*).
///
/// A decomposição do traço mede o custo POR MOVE, e por ela o 1º traço é o mais BARATO (110 contra 143 ms
/// sobre tinta) — ou seja o que o artista sente não está lá. O que ele sente é a **latência do
/// pen-down**: o intervalo entre clicar e o primeiro dab aparecer.
///
/// Esta sonda separa as três coisas que um pen-down pode pagar, e mede cada uma no tamanho em que ela
/// dói: o **1º** pen-down de uma camada virgem (que aloca o que for lazy), o **2º** (tudo já alocado) e o
/// **move** que vem depois. A diferença entre o 1º e o 2º é, por definição, o custo de *estrear*.
#[test]
#[ignore]
fn the_first_stroke_latency() {
    use ph2d_painter_brush::height::DrawTo;
    println!("[pendown] ms — 1o pen-down (camada virgem) | 2o pen-down | move seguinte");
    for size in [2048u32, 4096] {
        for (name, impasto, draw_to) in [
            ("impasto OFF (controle)", false, DrawTo::ColorAndDepth),
            ("impasto ON, so PIGMENTO", true, DrawTo::Color),
            ("impasto ON, so RELEVO", true, DrawTo::Depth),
            ("impasto ON (default)", true, DrawTo::ColorAndDepth),
        ] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            if impasto {
                t.toggle_brush_impasto();
                t.paint.brush.impasto_draw_to = draw_to;
                for slot in &mut t.paint.brush_by_mode {
                    slot.impasto_draw_to = draw_to;
                }
            }
            t.set_brush_size_px(200.0);
            let c = size as f32 * 0.5;
            // 1º pen-down: a camada nunca foi tocada.
            let first = ms(&mut || {
                t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
            });
            // ⚠️ O move TEM de andar mais que o espaçamento, senão nenhum dab nasce e a coluna mede
            // ZERO — que foi o que a 1ª versão desta sonda reportou, e um zero desses lê como
            // "o move é grátis" quando na verdade é "o move não aconteceu".
            let step = t.paint.brush.dab_spacing_px().max(4.0) * 2.0;
            let mut mid = f64::MAX;
            for k in 1..=4u8 {
                let x = c - 300.0 + step * f32::from(k);
                mid = mid.min(ms(&mut || {
                    t.on_canvas_pointer(cp([x, c], PointerPhase::Move));
                }));
            }
            let move_ms = mid;
            t.on_canvas_pointer(cp([c + 100.0, c], PointerPhase::Up));
            // Os pen-downs SEGUINTES: tudo o que era lazy já existe. Se o custo NÃO cair, ele é
            // por-gesto (uma cópia canvas-sized por traço) e não estreia de nada.
            let mut rest = Vec::new();
            for k in 1..=4u8 {
                let y = c + 40.0 * f32::from(k);
                rest.push(ms(&mut || {
                    t.on_canvas_pointer(cp([c - 300.0, y], PointerPhase::Down));
                }));
                t.on_canvas_pointer(cp([c - 260.0, y], PointerPhase::Up));
            }
            let tail = rest
                .iter()
                .map(|v| format!("{v:.2}"))
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "[pendown] {size}^2 {name:<24} 1o {first:>7.2} | seguintes {tail} | move {move_ms:.2}"
            );
        }
    }
}

/// **O DAB DE RELEVO custa 8× o de pigmento** (3,30 contra 0,39 ms a raio 100, 4096²) — e isto é o
/// custo de TODO move, não só do pen-down. Esta sonda pergunta de onde ele vem.
///
/// ⚠️ **A 1ª versão desta sonda era RUÍDO e eu acreditei nela.** Ela construía um `PainterTool` por
/// configuração — canvas novo de 64 MB, planos novos, páginas novas — e tirava a mediana de 8 moves.
/// O que a desmascarou foi uma mutação de medição que **não podia** tocar a linha de controle (*"só o
/// depósito"* tem o AA desligado, então o caminho mutado nem é chamado ali) e mesmo assim a viu saltar
/// de **4,26 para 5,99 ms**: ±40% de deriva entre corridas, com um efeito medido de 38% em cima.
///
/// A cura é **PAREAR**: um tool, uma tela, um traço, e as configurações **alternadas entre os moves**.
/// Cada amostra encontra as mesmas páginas, o mesmo estado de alocador e a mesma vizinhança do canvas,
/// então a diferença entre dois grupos é a configuração e nada mais. É o mesmo raciocínio dos gates de
/// RAZÃO do módulo: comparar duas medições da MESMA corrida é imune à deriva que separa duas corridas.
#[test]
#[ignore]
fn where_the_relief_dab_spends_its_time() {
    use ph2d_painter_brush::height::DrawTo;
    const SIZE: u32 = 4096;
    const ROUNDS: usize = 24;
    // (rótulo, smooth edges, push) — o SETTLE saiu: ele roda no commit (pen-up), não por dab, então
    // uma sonda de MOVE não pode vê-lo e mediu ruído nas duas versões.
    let cfgs = [
        ("tudo ligado (o default)", true, 1.0f32),
        ("sem o AA do filme", false, 1.0),
        ("sem o PUSH", true, 0.0),
        ("sem nenhum dos dois", false, 0.0),
    ];
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.toggle_brush_impasto();
    t.paint.brush.impasto_draw_to = DrawTo::Depth;
    for slot in &mut t.paint.brush_by_mode {
        slot.impasto_draw_to = DrawTo::Depth;
    }
    t.set_brush_size_px(200.0);
    let c = SIZE as f32 * 0.5;
    let step = t.paint.brush.dab_spacing_px().max(4.0) * 2.0;
    // ⚠️ SOMA por grupo, não mediana por-move: nem todo move produz um dab (o espaçamento decide), e a
    // mediana de uma amostra em que a maioria é `0,00` mede **quantos moves ficaram vazios**, não o
    // custo de um dab — foi o que a 2ª versão desta sonda reportou. Cada configuração percorre a MESMA
    // distância, então a soma do grupo carrega o mesmo número de dabs e é a comparação honesta.
    let mut totals = vec![0.0f64; cfgs.len()];
    let mut dabs = vec![0usize; cfgs.len()];
    // Um traço só, longo, com as configurações ALTERNADAS move a move.
    t.on_canvas_pointer(cp([c - 1500.0, c], PointerPhase::Down));
    let mut x = c - 1500.0;
    for round in 0..ROUNDS {
        for (k, (_, aa, push)) in cfgs.iter().enumerate() {
            t.paint.brush.impasto_smooth_edges = *aa;
            t.paint.brush.impasto_push = *push;
            x += step;
            let v = ms(&mut || {
                t.on_canvas_pointer(cp([x, c], PointerPhase::Move));
            });
            // A 1ª rodada aquece (a 1ª escrita em cada página do traço); as outras contam.
            if round > 0 {
                totals[k] += v;
                if v > 0.05 {
                    dabs[k] += 1;
                }
            }
        }
    }
    t.on_canvas_pointer(cp([x, c], PointerPhase::Up));
    println!("[relief-dab] raio 100, {SIZE}² — ms por MOVE, PAREADO (alternado no mesmo traço)");
    let mut per_dab = Vec::new();
    for (k, (name, _, _)) in cfgs.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "contagem pequena")]
        let d = dabs[k].max(1) as f64;
        per_dab.push(totals[k] / d);
        println!(
            "[relief-dab] {name:<28} total {:.1} ms em {} dabs => {:.2} ms/dab",
            totals[k],
            dabs[k],
            totals[k] / d
        );
    }
    // ⚠️ CONTROLE: os grupos têm de ter carregado o MESMO número de dabs. Se não tiverem, eles não
    // percorreram o mesmo trabalho e a diferença entre eles não é a configuração.
    assert!(
        dabs.iter().max().unwrap() - dabs.iter().min().unwrap() <= 1,
        "controle: os grupos carregaram numeros de dabs diferentes ({dabs:?}) — a comparacao nao vale"
    );
    println!(
        "[relief-dab] => o AA custa {:.2} ms ({:.0}% do dab) | o PUSH custa {:.2} ms",
        per_dab[0] - per_dab[1],
        (per_dab[0] - per_dab[1]) / per_dab[0] * 100.0,
        per_dab[0] - per_dab[2]
    );
}
