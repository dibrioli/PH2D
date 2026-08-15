//! **DE QUE A FAIXA É FEITA, E DE ONDE VINHA A ESPÍCULA** — as sondas do trilho de fora e das
//! travessas, irmãs das do [`super::ribbon_probe`] (que mede o que a fita CUSTA e quanto ela ATRASA).
//!
//! ⚠️ **O corte é de ASSUNTO:** *quanto tempo a fita atrasa e o que ela cobra por evento* e *de que
//! é feita a faixa* são perguntas diferentes, e foi a segunda que resolveu o report de 2026-08-15.
//!
//! Rodar: `cargo test -p ph2d-painter-brush --release measure_ -- --ignored --nocapture`

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn spec(weight: f32, friction: f32, gravity: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: LineKind::Ribbon,
        ribbon_weight: weight,
        ribbon_friction: friction,
        ribbon_gravity: gravity,
        ..Default::default()
    }
}

fn plain() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// **Quantos fios uma fita costura, e de que tamanho?** — a pergunta que separa *"a faixa não abre"*
/// de *"a faixa não existe"*, e ela é do MOTOR (o depósito é do tool).
#[test]
#[ignore = "sonda"]
fn measure_the_ribbon_rungs() {
    for (nome, w, ru) in [
        ("rungs 0.0", 0.20f32, 0.0f32),
        ("rungs 0.5", 0.20, 0.5),
        ("rungs 1.0", 0.20, 1.0),
        ("peso 0.45", 0.45, 0.5),
    ] {
        let mut sp = spec(w, 0.30, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = ru;
        let mut s = Stroke::new(sp, plain(), 1);
        let mut out = Vec::new();
        let mut fios = Vec::new();
        let mut total = 0usize;
        let mut comp = 0.0f32;
        let p = |x: f32| StrokePoint {
            pos: [x, 300.0],
            pressure: 1.0,
        };
        s.begin(p(100.0), &mut out);
        for i in 1..=120 {
            #[allow(clippy::cast_precision_loss)]
            let x = 100.0 + i as f32 * 20.0;
            s.extend(p(x), &mut out);
            s.tick(1.0 / 60.0, &mut out);
            s.take_threads(&mut fios);
            for f in &fios {
                total += 1;
                comp += ((f[2] - f[0]).powi(2) + (f[3] - f[1]).powi(2)).sqrt();
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let medio = if total == 0 { 0.0 } else { comp / total as f32 };
        println!("[fios] {nome}: {total} fios, comprimento medio {medio:.1} px");
    }
}

/// **DE ONDE VÊM AS ESPÍCULAS?** — o report do Enio (2026-08-15, com a foto): a faixa ficou boa e
/// sobraram riscos retos e ESCUROS atravessando o desenho, finos demais para serem dabs.
///
/// A sonda classifica cada fio pela ordem de emissão (`rung0, rung1, rail01, rung2, rail12, …` ⇒ de
/// 1 em diante, ímpar = TRAVESSA, par = TRILHO) e reporta a distribuição de comprimentos das duas
/// famílias sobre o gesto da foto: uma onda RÁPIDA com inversões.
#[test]
#[ignore = "sonda"]
fn measure_where_the_band_spikes_come_from() {
    for (nome, w, fr) in [
        ("default 0,45/0,30", 0.45f32, 0.30f32),
        ("peso 1,0", 1.0, 0.30),
    ] {
        let mut sp = spec(w, fr, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = 0.5;
        let mut s = Stroke::new(sp, plain(), 1);
        let (mut out, mut buf, mut all) = (Vec::new(), Vec::new(), Vec::new());
        // A onda da foto: rápida, com inversões duras (é nelas que a fita PARA e reverte).
        let pt = |u: f32| StrokePoint {
            pos: [80.0 + u * 900.0, 400.0 + (u * 16.0).sin() * 220.0],
            pressure: 1.0,
        };
        s.begin(pt(0.0), &mut out);
        for i in 1..=140 {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f32 / 140.0;
            s.extend(pt(u), &mut out);
            s.tick(1.0 / 60.0, &mut out);
            s.take_threads(&mut buf);
            all.append(&mut buf);
        }
        let comp = |t: &[f32; 4]| ((t[2] - t[0]).powi(2) + (t[3] - t[1]).powi(2)).sqrt();
        let mut trav: Vec<f32> = Vec::new();
        let mut trilho: Vec<f32> = Vec::new();
        for (i, t) in all.iter().enumerate() {
            if i == 0 || i % 2 == 1 {
                trav.push(comp(t));
            } else {
                trilho.push(comp(t));
            }
        }
        let est = |v: &mut Vec<f32>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            #[allow(clippy::cast_precision_loss)]
            let med = v[v.len() / 2];
            (
                med,
                v[v.len() - 1],
                v.iter().filter(|&&x| x > 4.0 * med).count(),
            )
        };
        let (tm, tmax, tlongos) = est(&mut trav);
        let (rm, rmax, rlongos) = est(&mut trilho);
        println!(
            "[espicula] {nome}: travessas n={} mediana {tm:.1} pior {tmax:.1} px \
             ({tlongos} acima de 4x a mediana) | trilho n={} mediana {rm:.1} pior {rmax:.1} px \
             ({rlongos} acima de 4x)",
            trav.len(),
            trilho.len()
        );
    }
}

/// **QUE FASE DESENHA A ESPÍCULA?** — a espícula é de DABS (a sonda dos fios inocentou o feixe), e
/// as três candidatas são: o gesto contínuo · a mão PARADA com o botão preso · a CAUDA do pen-up.
///
/// O oráculo é a maior corrida COLINEAR de dabs de cada fase: uma fita a percorrer uma curva emite
/// dabs que viram; um trecho reto e longo é a assinatura da espícula.
#[test]
#[ignore = "sonda"]
fn measure_which_phase_draws_the_spike() {
    let reta = |dabs: &[crate::stroke::Dab]| -> f32 {
        // A maior corrida de dabs cujo desvio da corda fica sob meio pixel.
        let (mut melhor, mut i) = (0.0f32, 0usize);
        while i + 2 < dabs.len() {
            let a = dabs[i].center;
            let mut j = i + 2;
            let mut fim = i + 1;
            while j < dabs.len() {
                let b = dabs[j].center;
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let n = (dx * dx + dy * dy).sqrt();
                if n < 1e-6 {
                    j += 1;
                    continue;
                }
                let ok = (i + 1..j).all(|k| {
                    let p = dabs[k].center;
                    ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / n < 0.5
                });
                if !ok {
                    break;
                }
                fim = j;
                j += 1;
            }
            let e = dabs[fim].center;
            melhor = melhor.max(((e[0] - a[0]).powi(2) + (e[1] - a[1]).powi(2)).sqrt());
            i = fim.max(i + 1);
        }
        melhor
    };
    for (nome, w) in [("default 0,45", 0.45f32), ("peso 1,0", 1.0)] {
        let mut sp = spec(w, 0.30, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = 0.5;
        // ⚠️ **O estabilizador é o do PRODUTO (0,5).** O `spec()` deste arquivo crava `0.0`, que é
        // exactamente o valor que faz o `Stroke::settle` sair no segundo `if` — com ele, a coluna
        // PAUSA desta tabela mede zero por construção, e foi assim que ela inocentou a pausa numa
        // rodada em que o produto a desenhava.
        sp.stabilizer = 0.5;
        let pt = |u: f32| StrokePoint {
            pos: [80.0 + u * 900.0, 400.0 + (u * 16.0).sin() * 220.0],
            pressure: 1.0,
        };
        // Três fases, medidas SEPARADAS: o gesto, a pausa com o botão preso, a cauda do pen-up.
        let mut s = Stroke::new(sp, plain(), 1);
        let (mut out, mut gesto) = (Vec::new(), Vec::new());
        s.begin(pt(0.0), &mut out);
        for i in 1..=100 {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f32 / 140.0;
            s.extend(pt(u), &mut out);
            gesto.extend_from_slice(&out);
            s.tick(1.0 / 60.0, &mut out);
            gesto.extend_from_slice(&out);
        }
        let mut pausa = Vec::new();
        for _ in 0..30 {
            // ⚠️ **A pausa é a AUSÊNCIA de evento, e a versão anterior desta linha entregava um
            // `extend` na mesma posição.** Um rato parado não emite `CursorMoved`, e o produto lê
            // isso como `parked` — que é o que chama o `Stroke::settle`. Com o `extend` sintético a
            // sonda media a pausa pelo único caminho em que o segundo percorredor fica adormecido.
            s.tick(1.0 / 60.0, &mut out);
            pausa.extend_from_slice(&out);
            let mut parado = Vec::new();
            s.settle(&mut parado);
            pausa.append(&mut parado);
        }
        let mut cauda = Vec::new();
        s.finish(&mut out);
        cauda.extend_from_slice(&out);
        println!(
            "[fase] {nome}: gesto {} dabs / maior reta {:.0} px · PAUSA {} dabs / reta {:.0} px \
             · CAUDA {} dabs / reta {:.0} px",
            gesto.len(),
            reta(&gesto),
            pausa.len(),
            reta(&pausa),
            cauda.len(),
            reta(&cauda)
        );
    }
}

/// **O GESTO DA FOTO — a onda que DESACELERA nos picos** (report do Enio de 2026-08-15, 2ª rodada,
/// com screenshot: *"parece melhor mas com as espículas do algoritmo antigo"*).
///
/// ⚠️ **A fixture anterior não continha o fenômeno, e o screenshot diz por quê:** as duas espículas
/// apontam para perto dos **picos** da onda, e as sondas existentes movem a mão a passo CONSTANTE.
/// Uma mão real desacelera onde o traço vira — é ali que o quadro cobre pouco arco do dedo e muito
/// arco da fita, e é ali que uma reta longa pode nascer sem ninguém ter parado.
///
/// **O oráculo é a TINTA DE UM TIQUE:** um tique normal deposita alguns dabs sobre alguns pixels; um
/// tique que desenha uma espícula deposita uma FILEIRA sobre centenas. A régua não conhece constante
/// nenhuma do produto — ela mede o que o carimbo recebeu.
///
/// ⚠️ **A primeira linha é o CONTROLE** (`LineKind::None`): sem ela a tabela não distingue *"a fita
/// desenha uma reta"* de *"o gesto TEM uma reta"*.
#[test]
#[ignore = "sonda"]
fn measure_the_ink_one_tick_lays_on_a_hand_that_slows_at_the_peaks() {
    let dist2 = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    for (nome, kind, w) in [
        ("controle (sem fita)", LineKind::None, 0.0f32),
        ("fita w=0.08", LineKind::Ribbon, 0.08),
        ("fita w=0.20", LineKind::Ribbon, 0.20),
        ("fita w=0.45", LineKind::Ribbon, 0.45),
    ] {
        let mut sp = spec(w, 0.30, 0.0);
        sp.radius_px = 6.0;
        sp.ribbon_rungs = 0.5;
        sp.line_kind = kind;
        let mut s = Stroke::new(sp, plain(), 1);
        let mut out = Vec::new();
        // A MÃO: anda em x, oscila em y, e o passo COLAPSA onde a curva vira (|cos| = 0 no pico).
        let k = 0.0075f32;
        let mut x = 100.0f32;
        s.begin(
            StrokePoint {
                pos: [x, 400.0],
                pressure: 1.0,
            },
            &mut out,
        );
        let mut fios: Vec<[f32; 4]> = Vec::new();
        let (mut pior_fio, mut pior_fio_i) = (0.0f32, 0usize);
        let mut pior_fio_kind = "-";
        let (mut leque, mut leque_i, mut leque_n) = (0.0f32, 0usize, 0usize);
        let (mut n_trav, mut n_trilho) = (0usize, 0usize);
        let (mut longas_trav, mut longos_trilho) = (0usize, 0usize);
        let mut pior_fio_seg = [0.0f32; 4];
        let (mut pior, mut pior_i, mut pior_n) = (0.0f32, 0usize, 0usize);
        let (mut pior_de, mut pior_ate) = ([0.0f32; 2], [0.0f32; 2]);
        let (mut total, mut parados) = (0usize, 0usize);
        for i in 0..120usize {
            // ⚠️ **A FREADA é a fixture, e sem ela a sonda inocenta o produto:** entre os tiques 60 e
            // 72 a mão quase para (1,5 px por tique) sem PARAR — acima do piso de meio pixel, então
            // o tique não é recusado. É o que uma mão faz numa inversão dura e no fim de um traço.
            let freando = (60..72).contains(&i);
            let passo = if freando {
                1.5
            } else {
                45.0 * (0.06 + 0.94 * (x * k).cos().abs())
            };
            x += passo;
            let y = 400.0 + 180.0 * (x * k).sin();
            s.extend(
                StrokePoint {
                    pos: [x, y],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(1.0 / 60.0, &mut out);
            total += out.len();
            // ⚠️ **Os FIOS entram na mesma tabela que os dabs, e a razão é o screenshot:** a espícula
            // é ESCURA e as travessas são cinza-claras, o que sugere dab — mas *sugere* não é
            // medição, e um trilho de fora ranço desenha uma reta longa pelo canal dos fios.
            s.take_threads(&mut fios);
            // ⚠️ **A classificação é a ORDEM DE EMISSÃO DENTRO DO TIQUE, não uma paridade global:**
            // `sew_rungs` emite `travessa, trilho, travessa, trilho, …` por quadro, e a primeira
            // travessa de um traço não tem trilho antes dela. Contar sobre a lista inteira do traço
            // desalinha no primeiro quadro e chama metade de cada família pelo nome da outra.
            // ⚠️ **A CUNHA é POR TIQUE, e a 1ª versão desta régua media a CADÊNCIA.** Ela olhava
            // duas travessas consecutivas e reportava sempre 13,5 px — que é exactamente
            // `ribbon_rung_px()`, porque a cadência é medida ao longo do trilho da FITA, então as
            // pontas de perto estão SEMPRE a um passo. O que se vê no desenho é outra coisa: as
            // travessas de UM tique abrindo muito no trilho da fita e fechando num ponto no do dedo.
            let (mut n_do_tique, mut perto_ini, mut perto_fim) = (0usize, [0.0f32; 2], [0.0f32; 2]);
            let (mut longe_ini, mut longe_fim) = ([0.0f32; 2], [0.0f32; 2]);
            for (j, t) in fios.drain(..).enumerate() {
                if j % 2 == 0 {
                    if n_do_tique == 0 {
                        perto_ini = [t[0], t[1]];
                        longe_ini = [t[2], t[3]];
                    }
                    perto_fim = [t[0], t[1]];
                    longe_fim = [t[2], t[3]];
                    n_do_tique += 1;
                }
                let d = dist2([t[0], t[1]], [t[2], t[3]]);
                let travessa = j % 2 == 0;
                if travessa {
                    n_trav += 1;
                    if d > 150.0 {
                        longas_trav += 1;
                    }
                } else {
                    n_trilho += 1;
                    if d > 150.0 {
                        longos_trilho += 1;
                    }
                }
                if d > pior_fio {
                    pior_fio = d;
                    pior_fio_i = i;
                    pior_fio_seg = t;
                    pior_fio_kind = if travessa { "TRAVESSA" } else { "TRILHO  " };
                }
            }
            if n_do_tique >= 2 {
                let abre = dist2(perto_ini, perto_fim);
                let fecha = dist2(longe_ini, longe_fim);
                if fecha < 5.0 && abre > leque {
                    leque = abre;
                    leque_i = i;
                    leque_n = n_do_tique;
                }
            }
            if out.is_empty() {
                parados += 1;
                continue;
            }
            let span = dist2(out[0].center, out[out.len() - 1].center);
            if span > pior {
                pior = span;
                pior_i = i;
                pior_n = out.len();
                pior_de = out[0].center;
                pior_ate = out[out.len() - 1].center;
            }
        }
        s.finish(&mut out);
        let cauda = if out.len() >= 2 {
            dist2(out[0].center, out[out.len() - 1].center)
        } else {
            0.0
        };
        println!(
            "[tique] {nome:20}: {total:5} dabs · {parados:3} MUDOS · \
             pior tique #{pior_i:3} = {pior:6.1} px/{pior_n:4} dabs \
             ({:.0},{:.0}->{:.0},{:.0}) · PIOR FIO {pior_fio_kind} #{pior_fio_i:3} = {pior_fio:6.1} px \
             ({:.0},{:.0}->{:.0},{:.0}) · >150px: {longas_trav}/{n_trav} travessas, \
             {longos_trilho}/{n_trilho} trilhos · CUNHA {leque:6.1} px em {leque_n:3} travessas \
             (tique #{leque_i:3}) · cauda {} dabs / {cauda:6.1} px",
            pior_de[0],
            pior_de[1],
            pior_ate[0],
            pior_ate[1],
            pior_fio_seg[0],
            pior_fio_seg[1],
            pior_fio_seg[2],
            pior_fio_seg[3],
            out.len()
        );
    }
}
