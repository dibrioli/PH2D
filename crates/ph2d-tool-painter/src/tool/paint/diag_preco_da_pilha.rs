//! DIAGNÓSTICO — **o que a ordem por TRAÇO custa**, medido na porta do produto.
//!
//! A recomposição regional replaya os lotes cuja caixa toca a do lote novo. Esse número **não
//! cresce com o traço** — ele vale `~2/spacing`, a sobreposição —, mas é o preço inteiro da wave e
//! tem de estar escrito com o relógio ao lado.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_preco_da_pilha -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ **Em `--release`**: em debug esta crate lê `~20×` mais lento e o tecto sairia cinco vezes
//! menor. ⚠️ E `/proc/loadavg` vai impresso ao lado — *nenhuma leitura de relógio desta máquina
//! vale nada acima de `load ~5`*.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 1024;
const Y: f32 = 512.0;
const X0: f32 = 152.0;
const X1: f32 = 872.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela_de(lado: u32, raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
    t.paint.brush.radius_px = raio;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    t
}

/// O traço inteiro em passos de 2 px — o regime do rato real.
fn traco(t: &mut PainterTool) -> std::time::Duration {
    let ini = std::time::Instant::now();
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += 2.0;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
    ini.elapsed()
}

#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_preco_da_pilha() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  O PREÇO DA ORDEM POR TRAÇO   (load {})", carga.trim());
    println!("  canvas {SIZE}² · traço de {} px · passo 2 px", X1 - X0);
    // ⭐ **A coluna que DECIDE não é a do traço NEM a de um evento: é a do QUADRO.** Um traço de
    // 720 px em passos de 2 px são `362` eventos, e somar o traço inteiro faz um custo interactivo
    // parecer um congelamento. ⛔⛔ **Mas dividir um evento por `16,7 ms` é o erro OPOSTO, e ele é
    // `16×`:** o método de omissão é o `Space`, que **NÃO** está no `coalesces_canvas_motion`
    // (`Arc | Ellipse | Polygon | Line | Anchored | DragDot`), logo um rato de `1000 Hz` entrega
    // **~16 eventos por quadro** e a pilha paga-os TODOS. *A 1.ª redacção desta coluna assumia um
    // evento por quadro e lia `10,7 %` onde a conta honesta lê `171 %`* — quem a apanhou foi a
    // §21.10, que já tinha a aritmética escrita.
    let eventos = ((X1 - X0) / 2.0).ceil() + 2.0;
    /// Eventos de ponteiro por quadro com um rato de 1000 Hz a 60 fps (§21.10, medido).
    const EV_POR_QUADRO: f64 = 16.0;
    println!(
        "\n  pilha                          |  raio |    ms | ms/evento | % do quadro (16 ev)"
    );
    println!("  -------------------------------+-------+-------+-----------+---------------------");
    let casos: [(&str, &[(CompositeOp, f32)]); 5] = [
        ("1 Brush (sem recomposição)", &[(CompositeOp::Brush, 1.0)]),
        (
            "2 Brush",
            &[(CompositeOp::Brush, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "3 Brush",
            &[
                (CompositeOp::Brush, 1.0),
                (CompositeOp::Brush, 1.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
        (
            "Blur sobre Brush",
            &[(CompositeOp::Blur, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Smear sobre Brush",
            &[(CompositeOp::Smear, 1.0), (CompositeOp::Brush, 1.0)],
        ),
    ];
    for raio in [24.0f32, 96.0] {
        for (nome, ops) in casos {
            let mut melhor = f64::MAX;
            for _ in 0..3 {
                let mut t = tela_de(SIZE, raio);
                for (i, &(op, s)) in ops.iter().enumerate() {
                    t.paint.composite[i] = CompositeLayer {
                        op,
                        strength: s,
                        ..CompositeLayer::default()
                    };
                }
                melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
            }
            let por_ev = melhor / f64::from(eventos);
            println!(
                "  {nome:30} | {raio:5.0} | {melhor:6.2} | {por_ev:9.3} | {:14.1}",
                por_ev * EV_POR_QUADRO / 16.7 * 100.0
            );
        }
    }

    // ⭐⭐⭐ **A PILHA DO DONO, tirada da foto de 2026-09-22** — e ela existe porque a bancada
    //     acima NÃO CONTINHA o regime dele: ela varre `raio 24` e `96`, e o pincel dele está em
    //     `size 0.4` ⇒ `1 + 0,4² × (512 − 1) = 82,8 px` de raio, com a camada de Blur a `Size
    //     2.048` ⇒ **~170 px**. *Uma bancada cujo pior caso é metade do caso do dono não mede o
    //     produto dele.*
    //
    //     A atribuição é por CAMADA: a pilha inteira, e depois cada camada sozinha (as outras a
    //     `strength = 0`, que é como o motor as pula). ⚠️ A soma das partes NÃO tem de bater o
    //     total — a recomposição por traço é partilhada —, e é a diferença que diz quanto é dela.
    let dono = [
        (CompositeOp::Blur, 1.0f32, 2.048f32),
        (CompositeOp::Brush, 0.133, 0.574),
        (CompositeOp::Brush, 0.204, 1.002),
        (CompositeOp::Brush, 0.176, 1.221),
        (CompositeOp::Smear, 0.596, 1.0),
        (CompositeOp::Erase, 0.104, 1.0),
    ];
    // ⚠️ Pela PORTA do produto e não pela fórmula à mão (auditoria de 2026-09-23): uma sonda que
    //    recalcula a lei do slider mede outro programa no dia em que ela mudar.
    let raio_do_dono = super::brush_settings::size_norm_to_px(0.4);
    println!("\n  A PILHA DO DONO   (raio do pincel {raio_do_dono:.1} px · foto de 2026-09-22)");
    println!("  camada viva                    |    ms | ms/evento | % do quadro (16 ev)");
    println!("  -------------------------------+-------+-----------+---------------------");
    let monta = |so: Option<usize>| -> f64 {
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let mut t = tela_de(SIZE, raio_do_dono);
            t.paint.composite_len = dono.len();
            for (i, &(op, st, sz)) in dono.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    op,
                    strength: if so.is_none_or(|j| j == i) { st } else { 0.0 },
                    size: sz,
                    ..CompositeLayer::default()
                };
            }
            melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
        }
        melhor
    };
    let linha = |nome: &str, ms: f64| {
        let por_ev = ms / f64::from(eventos);
        println!(
            "  {nome:30} | {ms:5.1} | {por_ev:9.3} | {:14.1}",
            por_ev * EV_POR_QUADRO / 16.7 * 100.0
        );
    };
    let total = monta(None);
    linha("a pilha INTEIRA (6)", total);
    for (i, &(op, st, sz)) in dono.iter().enumerate() {
        let ms = monta(Some(i));
        linha(&format!("{} {op:?} str {st} size {sz}", i + 1), ms);
    }

    // ⭐⭐⭐ **E a soma das camadas SOZINHAS não bate o total, por uma ordem de grandeza** — logo o
    //     custo não está em camada nenhuma, está no que só corre com DUAS OU MAIS. Duas medições
    //     separam os dois mecanismos possíveis:
    //
    //       (a) o custo é LINEAR no número de camadas  ⇒ é a aplicação por camada;
    //       (b) o custo cresce com o COMPRIMENTO do traço mais depressa do que linearmente
    //           ⇒ a região recomposta acompanha o traço acumulado, e isso é QUADRÁTICO.
    println!("\n  (a) QUANTAS CAMADAS   ·   (b) QUE COMPRIMENTO DE TRAÇO");
    println!("  camadas | comprimento |     ms | ms por evento");
    println!("  --------+-------------+--------+--------------");
    for n in [1usize, 2, 3, 4, 6] {
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let mut t = tela_de(SIZE, raio_do_dono);
            t.paint.composite_len = n;
            for (i, &(op, st, sz)) in dono.iter().take(n).enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    op,
                    strength: st,
                    size: sz,
                    ..CompositeLayer::default()
                };
            }
            melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
        }
        println!(
            "  {n:7} | {:11} | {melhor:6.1} | {:12.3}",
            (X1 - X0) as i32,
            melhor / f64::from(eventos)
        );
    }
    for frac in [0.25f32, 0.5, 1.0] {
        let x1 = X0 + (X1 - X0) * frac;
        let ev = ((x1 - X0) / 2.0).ceil() as u32 + 2;
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let mut t = tela_de(SIZE, raio_do_dono);
            t.paint.composite_len = dono.len();
            for (i, &(op, st, sz)) in dono.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    op,
                    strength: st,
                    size: sz,
                    ..CompositeLayer::default()
                };
            }
            let ini = std::time::Instant::now();
            t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
            let mut x = X0;
            while x < x1 {
                x += 2.0;
                t.on_canvas_pointer(cp([x.min(x1), Y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x1, Y], PointerPhase::Up));
            melhor = melhor.min(ini.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "  {:7} | {:11} | {melhor:6.1} | {:12.3}",
            dono.len(),
            (x1 - X0) as i32,
            melhor / f64::from(ev)
        );
    }

    // ⭐⭐⭐ **QUAL PAR acende o custo fixo.** O salto de UMA para DUAS camadas é `12×`, e de duas
    //     para seis é quase linear — logo há um custo que a recomposição paga UMA vez por evento.
    //     Se ele for estrutural, dois `Brush` pequenos custam o mesmo que `Blur + Brush`; se for a
    //     REGIÃO, o par com o borrão de `2.048` custa muito mais, porque o avental dele é `~254 px`
    //     de cada lado e a recomposição tem de cobrir isso.
    println!("\n  QUAL PAR ACENDE O CUSTO FIXO   (raio {raio_do_dono:.1} px)");
    println!("  par                            |     ms | ms por evento");
    println!("  -------------------------------+--------+--------------");
    type Par = (&'static str, [(CompositeOp, f32, f32); 2]);
    let pares: &[Par] = &[
        (
            "Brush 1.0 + Brush 1.0",
            [
                (CompositeOp::Brush, 1.0, 1.0),
                (CompositeOp::Brush, 1.0, 1.0),
            ],
        ),
        (
            "Brush 2.048 + Brush 1.0",
            [
                (CompositeOp::Brush, 1.0, 2.048),
                (CompositeOp::Brush, 1.0, 1.0),
            ],
        ),
        (
            "Blur 1.0 + Brush 1.0",
            [
                (CompositeOp::Blur, 1.0, 1.0),
                (CompositeOp::Brush, 1.0, 1.0),
            ],
        ),
        (
            "Blur 2.048 + Brush 1.0  (o dono)",
            [
                (CompositeOp::Blur, 1.0, 2.048),
                (CompositeOp::Brush, 1.0, 1.0),
            ],
        ),
        (
            "Smear 1.0 + Brush 1.0",
            [
                (CompositeOp::Smear, 1.0, 1.0),
                (CompositeOp::Brush, 1.0, 1.0),
            ],
        ),
    ];
    for (nome, ops) in pares {
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let mut t = tela_de(SIZE, raio_do_dono);
            t.paint.composite_len = 2;
            for (i, &(op, st, sz)) in ops.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    op,
                    strength: st,
                    size: sz,
                    ..CompositeLayer::default()
                };
            }
            melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
        }
        println!(
            "  {nome:30} | {melhor:6.1} | {:12.3}",
            melhor / f64::from(eventos)
        );
    }

    // ⭐⭐⭐ **AS FASES da pilha do dono** — depois de o borrão deixar de escrever a orla, a
    //     pergunta seguinte é onde ficam os `425,8 ms` que sobram. As quatro fases são a fotografia
    //     do `pre`, a acumulação dos dabs novos, as CÓPIAS (guardar a orla e devolvê-la) e a
    //     COMPOSIÇÃO. ⚠️ *As cópias são duas passagens sobre `alvo` por evento, e `alvo` não
    //     encolheu — só a aplicação do borrão encolheu.*
    {
        use super::composite_acumulado::fases;
        let mut t = tela_de(SIZE, raio_do_dono);
        t.paint.composite_len = dono.len();
        for (i, &(op, st, sz)) in dono.iter().enumerate() {
            t.paint.composite[i] = CompositeLayer {
                op,
                strength: st,
                size: sz,
                ..CompositeLayer::default()
            };
        }
        let _ = fases::take();
        let ms = traco(&mut t).as_secs_f64() * 1e3;
        let (us, _ev, _area) = fases::take();
        let soma: u64 = us.iter().sum();
        println!("\n  AS FASES DA PILHA DO DONO   (traço de {ms:.1} ms)");
        for (i, nome) in ["pre", "acumular", "compor", "cópias"].iter().enumerate() {
            println!(
                "    {nome:10} {:7.1} ms   {:5.1}%",
                us[i] as f64 / 1e3,
                us[i] as f64 / soma.max(1) as f64 * 100.0
            );
        }
    }

    // ⭐⭐⭐ **A CONTA: quantos PÍXEIS o borrão atravessa, sozinho e dentro da pilha.** O borrão
    //     sozinho custa `0,067 ms/evento` e dentro da pilha `0,667` — `10×`. Nenhuma régua de VALOR
    //     pode ver a diferença (as duas rotas desenham o mesmo), logo mede-se a CONTA.
    {
        use ph2d_painter_brush::blur_caixa::conta::{
            self, BORROES, MAIOR_LADO, PIXEIS_BORRADOS, PIXEIS_UTEIS,
        };
        println!("\n  A CONTA DO BORRÃO   (raio {raio_do_dono:.1} px · traço de 720 px)");
        println!(
            "  pilha                          | chamadas | Mpx lidos | Mpx úteis | avental | px/chamada"
        );
        println!(
            "  -------------------------------+----------+-----------+-----------+---------+-----------"
        );
        for (nome, ops) in [
            ("só Blur 2.048", vec![(CompositeOp::Blur, 1.0f32, 2.048f32)]),
            (
                "Blur 2.048 + Brush",
                vec![
                    (CompositeOp::Blur, 1.0, 2.048),
                    (CompositeOp::Brush, 1.0, 1.0),
                ],
            ),
            ("a pilha do dono (6)", dono.to_vec()),
        ] {
            conta::zera();
            let mut t = tela_de(SIZE, raio_do_dono);
            t.paint.composite_len = ops.len();
            for (i, &(op, st, sz)) in ops.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    op,
                    strength: st,
                    size: sz,
                    ..CompositeLayer::default()
                };
            }
            let _ = traco(&mut t);
            let n = BORROES.get();
            let px = PIXEIS_BORRADOS.get();
            let uteis = PIXEIS_UTEIS.get();
            println!(
                "  {nome:30} | {n:8} | {:9.1} | {:9.1} | {:6.1}% | {:10.0}",
                px as f64 / 1e6,
                uteis as f64 / 1e6,
                (px - uteis) as f64 / px.max(1) as f64 * 100.0,
                px as f64 / n.max(1) as f64
            );
            // ⭐ O maior LADO separa duas hipóteses com curas OPOSTAS: uma FAIXA que acompanha o
            //   dab (lado ~= pegada) contra a CAIXA DO TRAÇO INTEIRO (lado ~= comprimento).
            println!(
                "     └─ maior lado de região: {} px   (a pegada da camada mede {:.0} px)",
                MAIOR_LADO.get(),
                2.0 * raio_do_dono * ops.first().map_or(1.0, |o| o.2)
            );
        }
    }

    // ⭐⭐ **O TOPO DA QUOTA, na tela que o dono nomeou.** A quota é `3 Brush · 2 Erase · 1 Blur ·
    // 1 Smear` = **7**, e ela nunca tinha sido medida no produto — a tabela de cima pára em três
    // camadas de depósito. ⚠️ E o `2048²` é a cena da decisão dele sobre a tela grande: sem esta
    // linha, aquela decisão é tomada sobre um número que ninguém tirou.
    println!("\n  o TOPO DA QUOTA (3 Brush + 2 Erase + 1 Blur + 1 Smear = 7 camadas)");
    println!("  pilha        |  lado |  raio |      ms | ms/evento | % do quadro (16 ev)");
    println!("  -------------+-------+-------+---------+-----------+---------------------");
    let quota: [(CompositeOp, f32); 7] = [
        (CompositeOp::Brush, 1.0),
        (CompositeOp::Brush, 1.0),
        (CompositeOp::Brush, 1.0),
        (CompositeOp::Erase, 1.0),
        (CompositeOp::Erase, 1.0),
        (CompositeOp::Blur, 1.0),
        (CompositeOp::Smear, 1.0),
    ];
    // ⚠️ **A ATRIBUIÇÃO vem ANTES de qualquer cura** (ordem do dono 2026-09-22: *«atacar o Blur»*):
    // a quota INTEIRA contra a mesma quota **sem** a camada de Blur. Sem esta linha, «o Blur governa
    // o preço» é uma frase; com ela é um número, e ele diz quanto uma cura ali pode comprar.
    let sem_blur: Vec<(CompositeOp, f32)> = quota
        .iter()
        .filter(|(op, _)| *op != CompositeOp::Blur)
        .copied()
        .collect();
    for (rotulo, pilha) in [("7 (quota)", &quota[..]), ("6 (sem Blur)", &sem_blur[..])] {
        for lado in [SIZE, 2048] {
            for raio in [24.0f32, 96.0] {
                let mut melhor = f64::MAX;
                for _ in 0..3 {
                    let mut t = tela_de(lado, raio);
                    for (i, &(op, s)) in pilha.iter().enumerate() {
                        t.paint.composite[i] = CompositeLayer {
                            op,
                            strength: s,
                            ..CompositeLayer::default()
                        };
                    }
                    melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
                }
                let por_ev = melhor / f64::from(eventos);
                println!(
                    "  {rotulo:12} | {lado:5} | {raio:5.0} | {melhor:7.2} | {por_ev:9.3} | {:14.1}",
                    por_ev * EV_POR_QUADRO / 16.7 * 100.0
                );
            }
        }
    }
}
