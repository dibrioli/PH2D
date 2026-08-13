//! **As duas medições de MOTOR da W0** (plano [`docs/Painter/38_plano_linha_procedural.md`]): a
//! fórmula de velocidade que sobrevive ao polling, e o orçamento de fios de um Sketchy.
//!
//! As duas perguntam ao **caminho que o motor de fato percorre** — não ao caminho que o teste
//! desenhou. A diferença não é cerimônia: o estabilizador filtra a entrada, então a lista de dabs
//! que sai daqui é mais curta e mais lisa que a sequência de eventos que entrou, e é ela que
//! qualquer tipo de linha procedural vai ler. Uma sonda que fizesse a aritmética sobre os pontos de
//! entrada mediria um caminho que ninguém pinta.
//!
//! ⚠️ A terceira medição da W0 — o custo do Solid — mora no `ph2d-tool-painter`, ao lado do
//! composite booleano que ela cronometra. O corte é por ASSUNTO: aqui *o que o traço É*, lá *o que
//! a figura CUSTA*.

use crate::dynamics::Dynamics;
use crate::spec::BrushSpec;
use crate::stroke::{Dab, Stroke, StrokePoint};

/// Um quarto de círculo de raio `r`, amostrado em `n` pontos (o primeiro é o pen-down).
///
/// ⚠️ **Arco, nunca reta.** Numa reta o heading não vira, o estabilizador não tem o que filtrar e as
/// duas fórmulas de velocidade dariam o mesmo número — a fixture não conteria o fenômeno.
fn arc_path(r: f32, n: usize) -> Vec<[f32; 2]> {
    let c = [1024.0f32, 1024.0];
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / (n - 1) as f32;
            let a = t * std::f32::consts::FRAC_PI_2;
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

/// Percorre `path` no motor REAL e devolve os dabs emitidos.
fn walk(spec: BrushSpec, path: &[[f32; 2]]) -> Vec<Dab> {
    let mut s = Stroke::new(spec, Dynamics::default(), 0x0051_eed0);
    let mut out = Vec::new();
    let mut all = Vec::new();
    s.begin(
        StrokePoint {
            pos: path[0],
            pressure: 1.0,
        },
        &mut out,
    );
    all.append(&mut out);
    for p in &path[1..] {
        s.extend(
            StrokePoint {
                pos: *p,
                pressure: 1.0,
            },
            &mut out,
        );
        all.append(&mut out);
    }
    s.finish(&mut out);
    all.append(&mut out);
    all
}

/// **QUAL FÓRMULA DE VELOCIDADE SOBREVIVE AO POLLING** — a medição 1 da W0.
///
/// O `Speed Shapes` do Alchemy joga o dab para `p + v·k`. A pergunta que decide o desenho é o que
/// `v` significa, e há três candidatas com consequências diferentes:
///
/// 1. **deslocamento por EVENTO** (`|p − p_prev|`) — zero relógio, e é o que o Alchemy faz. ⚠️ Ela
///    é uma velocidade em *pixels por período de polling*: o mesmo gesto num mouse de 1000 Hz e num
///    de 125 Hz dá números 8× diferentes, e o traço muda de máquina para máquina.
/// 2. **arco entre DABS** — **zero informação, por construção**: no método `Space` os dabs saem a
///    intervalos FIXOS de arco (`spacing × diâmetro`), então essa distância é a mesma devagar ou
///    depressa. Está na tabela para ninguém a propor de novo.
/// 3. **arco por TICK** (`Δarco / dt`) — o `Tool` já recebe `on_tick(dt_ms)` e o `Stroke` já tem
///    `tick(dt, out)`, então o relógio existe e é o do quadro. ⚠️ E é a única que **não toca
///    contrato congelado**: o `CanvasPointer` é da superfície `CanvasPaintTool` (ADR-0040 emenda 3,
///    §6), não carrega timestamp, e acrescentar um seria Coord-only + ADR.
///
/// A tabela entrega o MESMO caminho em densidades de evento diferentes. A coluna que ficar **plana**
/// é a que descreve o gesto; a que escalar com a densidade descreve o dispositivo.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_which_speed_survives_the_polling() {
    for stab_off in [false, true] {
        // ⚠️ A ablação é pelo KNOB, na fixture — não por instrumentação. É ela que ATRIBUI os 3%
        // residuais da linha de 8 eventos ao estabilizador em vez de os deixar como ruído.
        let spec = BrushSpec {
            radius_px: 24.0,
            stabilizer: if stab_off {
                0.0
            } else {
                BrushSpec::default().stabilizer
            },
            ..Default::default()
        };
        speed_table(spec, stab_off);
    }
}

/// Uma passada da tabela de velocidade. `stab_off` só rotula — a ablação é pelo KNOB, na fixture.
fn speed_table(spec: BrushSpec, stab_off: bool) {
    let r = 200.0f32;
    println!(
        "[line] a MESMA curva (quarto de circulo, raio 200 = 314 px de arco), raio 24, spacing default{}",
        if stab_off {
            " — ESTABILIZADOR OFF"
        } else {
            " — estabilizador default"
        }
    );
    println!(
        "{:>7}  {:>12} {:>12}  {:>10} {:>12}  {:>12} {:>12}",
        "eventos", "desloc/ev", "razao", "dabs", "arco/dab", "arco/quadro", "razao"
    );
    // O caminho é percorrido em 8 "quadros" de eventos — é assim que a shell entrega (o handshake é
    // por QUADRO, doc 28 §5.49), então esta é a granularidade em que um tick mediria o arco.
    const FRAMES: usize = 8;
    let mut base_ev = 0.0f64;
    let mut base_fr = 0.0f64;
    for (k, n) in [8usize, 32, 128, 512].into_iter().enumerate() {
        let path = arc_path(r, n);
        let mut d_ev = 0.0f64;
        for w in path.windows(2) {
            d_ev += f64::from((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]));
        }
        #[allow(clippy::cast_precision_loss)]
        let mean_ev = d_ev / (n - 1) as f64;

        let dabs = walk(spec, &path);
        let arc_end = f64::from(dabs.last().map_or(0.0, |d| d.arc_len));
        #[allow(clippy::cast_precision_loss)]
        let arc_per_dab = if dabs.len() > 1 {
            arc_end / (dabs.len() - 1) as f64
        } else {
            0.0
        };
        // O arco percorrido dentro de cada quadro: a grandeza que `Δarco / dt` divide.
        #[allow(clippy::cast_precision_loss)]
        let arc_per_frame = arc_end / FRAMES as f64;

        if k == 0 {
            base_ev = mean_ev;
            base_fr = arc_per_frame;
        }
        println!(
            "{n:>7}  {mean_ev:>12.3} {:>12.2} {:>10} {arc_per_dab:>12.3}  {arc_per_frame:>12.3} {:>12.2}",
            mean_ev / base_ev,
            dabs.len(),
            arc_per_frame / base_fr,
        );
    }
    println!(
        "[line] leitura: a coluna que fica em 1,00 descreve o GESTO; a que escala descreve o DISPOSITIVO."
    );
}

/// **O ORÇAMENTO DE UM SKETCHY** — a medição 3 da W0.
///
/// O *neighbour points* (Ze Frank → Harmony → Krita `Sketch`) guarda todos os pontos do traço e,
/// a cada ponto novo, liga os vizinhos dentro de um raio. ⚠️ É **quadrático por natureza** — cada
/// ponto pode ver todos os anteriores —, e é por isso que o Krita ship um *Simple Mode* para pincel
/// grande. O teto do slider `Reach` tem de sair deste número, não de um palpite.
///
/// A tabela conta os **fios** que nasceriam e converte para o que custa de fato: cada fio é uma
/// linha de dabs, então o preço é `comprimento_total / (spacing × diâmetro do fio)`.
///
/// ⚠️ **O vizinho IMEDIATO não conta.** Dois dabs consecutivos estão a um espaçamento um do outro
/// por construção; um "fio" entre eles é o próprio traço, e contá-lo faria a tabela dizer que todo
/// traço já é um Sketchy.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_how_many_threads_a_sketchy_would_emit() {
    println!("[line] fios de um Sketchy — quarto de circulo, 128 eventos, densidade 1,0 (todo par)");
    println!(
        "{:>6} {:>7}  {:>6} {:>8} {:>10}  {:>9} {:>11} {:>9} {:>9}",
        "raio", "arco", "dabs", "reach", "fios", "fios/dab", "compr(px)", "compr/arco", "dens@2x"
    );
    for radius in [12.0f32, 24.0, 48.0, 96.0] {
        let spec = BrushSpec {
            radius_px: radius,
            ..Default::default()
        };
        for arc_r in [200.0f32, 600.0] {
            let path = arc_path(arc_r, 128);
            let dabs = walk(spec, &path);
            let c: Vec<[f32; 2]> = dabs.iter().map(|d| d.center).collect();
            let arc = dabs.last().map_or(0.0, |d| d.arc_len);
            for reach_mult in [1.0f32, 2.0, 4.0] {
                let reach = reach_mult * 2.0 * radius; // em DIÂMETROS, a unidade do pincel
                let mut threads = 0u64;
                let mut total_len = 0.0f64;
                for i in 0..c.len() {
                    // `j < i - 1` — o vizinho imediato é o próprio traço.
                    for j in 0..i.saturating_sub(1) {
                        let d = (c[i][0] - c[j][0]).hypot(c[i][1] - c[j][1]);
                        if d <= reach {
                            threads += 1;
                            total_len += f64::from(d);
                        }
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                let per_dab = threads as f64 / (c.len().max(1)) as f64;
                // ⚠️ **A grandeza que orça é o COMPRIMENTO de fio, não a contagem.** Um fio é tinta
                // depositada; `compr/arco` diz quantas vezes o traço o artista pagaria por passar o
                // pincel uma vez. E `dens@2x` é a densidade que manteria esse gasto em 2× — que é o
                // teto que o slider precisa, derivado em vez de escolhido.
                let ratio = total_len / f64::from(arc.max(1.0));
                let dens = (2.0 / ratio).min(1.0);
                println!(
                    "{radius:>6.0} {arc:>7.0}  {:>6} {reach:>8.0} {threads:>10}  {per_dab:>9.2} {total_len:>11.0} {ratio:>9.1} {dens:>9.3}",
                    c.len(),
                );
            }
        }
    }
    println!(
        "[line] leitura: quem orca e' `compr/arco` (quantas vezes o traco o artista pagaria). `dens@2x` e' o teto do slider, derivado."
    );
}
