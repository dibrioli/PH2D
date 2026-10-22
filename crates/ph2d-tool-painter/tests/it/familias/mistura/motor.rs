//! Os recursos de MOTOR da família `mistura` (J4, «Motor J4» de `docs/Painter/Pinceis/exames/mistura.md`), presos ao
//! nível do motor `ph2d-wet-paint`: cada um desligado por omissão (o neutro = a lei de antes, ao byte) e com a lei que
//! o liga medida numa cena mínima.

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 260;
const H: usize = 140;
/// A faixa vermelha horizontal (y 70, x 20..240) e o cruzamento da água em x 130.
const Y: usize = 70;
const X: usize = 130;

/// Uma faixa LISA de tinta (y 58..82, x 20..240), semeada direito nos planos: MOLHADA (pigmento suspenso sobre um
/// filme de água) ou, com `seca`, assentada sobre o papel seco — a lei da água medida sem o tapete das cerdas.
fn faixa(seca: bool) -> Engine {
    let mut e = Engine::new(W, H);
    let g = e.active_grid_mut();
    for y in Y - 12..=Y + 12 {
        for x in 20..=240 {
            let i = x + 1 + (y + 1) * g.s;
            if seca {
                g.sett[i] = 120.0;
                g.sett_rgb[i] = [191.0, 0.0, 18.0];
            } else {
                g.susp[i] = 120.0;
                g.susp_rgb[i] = [191.0, 0.0, 18.0];
                g.film[i] = 0.3;
            }
        }
    }
    e
}

/// A água (ferramenta Wet) em vertical por x = 130, com o `push` dado (`None` = o knob por omissão).
fn agua(e: &mut Engine, push: Option<f64>) {
    if let Some(p) = push {
        e.tuning.set(Knob::WaterPush, p);
    }
    let mut prev: Option<[f64; 2]> = None;
    let mut y = 20.0;
    while y <= 120.0 {
        e.dispatch_pressure_dab_tool(Tool::Wet, X as f64, y, 5.0, 10.0, prev);
        prev = Some([X as f64, y]);
        y += 2.0;
    }
}

/// O pigmento (suspenso + assentado) na linha da faixa, média de 5 linhas, por coluna.
fn linha(e: &Engine) -> Vec<f64> {
    let g = e.active_grid();
    (0..W)
        .map(|x| {
            (Y - 2..=Y + 2)
                .map(|y| {
                    let i = x + 1 + (y + 1) * g.s;
                    f64::from(g.susp[i]) + f64::from(g.sett[i])
                })
                .sum::<f64>()
                / 5.0
        })
        .collect()
}

fn media(v: &[f64], a: usize, b: usize) -> f64 {
    v[a..b].iter().sum::<f64>() / (b - a) as f64
}

/// O pigmento (suspenso + assentado) na coluna do cruzamento (x 127..133) ABAIXO da faixa (y 83..110): o escorrido.
fn escorrido(e: &Engine) -> f64 {
    let g = e.active_grid();
    (Y + 13..Y + 40)
        .flat_map(|y| (X - 3..=X + 3).map(move |x| (x, y)))
        .map(|(x, y)| {
            let i = x + 1 + (y + 1) * g.s;
            f64::from(g.susp[i]) + f64::from(g.sett[i])
        })
        .sum()
}

/// ⭐ **A água EMPURRA o pigmento** (`Knob::WaterPush`, L-6 Water Brush): sobre a faixa MOLHADA, o traço de água com o
/// knob ligado ABRE o miolo (o pigmento no eixo cai abaixo de 0,35 do que estava — o Water Soft do Rebelle deixa 0,13)
/// e deixa a ORLA onde a água pára (o máximo a 2..6,6 r do eixo passa 2,2× a faixa intacta; tudo pelo caminho dava
/// 1,9); com o knob a 0 a água de sempre não abre nada (≥ 0,8). Sobre a faixa SECA (a água dissolve um pouco, o
/// `wetLift`) não há frente de lado — nenhuma orla na borda da pegada (≤ 1,1×) — e o que dissolveu ESCORRE pelo traço
/// para baixo da faixa (o escorrido do Rebelle): 789 de pigmento abaixo dela, contra 250 se a pegada
/// empurrasse de lado (a frente sem a regra «passa da pegada»).
#[test]
fn a_agua_empurra_o_pigmento_para_a_orla() {
    let r = 10.0_f64;
    let antes = linha(&faixa(false));
    let intacta = media(&antes, 40, 80);
    let mede = |seca: bool, push: Option<f64>| {
        let mut e = faixa(seca);
        e.tuning.set(Knob::WetLift, 0.5);
        e.tuning.set(Knob::ExtStaining, 0.5);
        // A água do Water Brush (o pingo de omissão a 1 deixa o filme a 0,003 — abaixo da célula molhada).
        e.tuning.set(Knob::WaterPerDab, 4.0);
        e.sliders.water = 1.0;
        let base = linha(&e);
        agua(&mut e, push);
        let l = linha(&e);
        let miolo = media(&l, X - 3, X + 4) / media(&base, X - 3, X + 4).max(1e-9);
        let lo = X + (2.0 * r) as usize;
        let hi = X + (6.6 * r) as usize + 3;
        let orla = l[lo..hi].iter().copied().fold(0.0, f64::max) / intacta.max(1e-9);
        let borda = (X + r as usize - 2..=X + r as usize + 3)
            .map(|x| l[x] / base[x].max(1e-9))
            .fold(0.0, f64::max);
        let esc = escorrido(&e);
        println!(
            "[mistura] água seca={seca} push={push:?}: miolo {miolo:.3} · orla {orla:.3} · borda {borda:.3} · escorrido {esc:.1}"
        );
        (miolo, orla, borda, esc)
    };
    let (m0, _, _, _) = mede(false, None);
    let (m1, o1, _, _) = mede(false, Some(0.87));
    let (_, _, b2, e2) = mede(true, Some(0.87));
    let (_, _, _, e3) = mede(true, None);
    let mut falhas = Vec::new();
    if m0 < 0.8 {
        falhas.push(format!(
            "sem o knob a água abriu a faixa molhada: miolo {m0:.3} < 0,8"
        ));
    }
    if m1 > 0.35 {
        falhas.push(format!(
            "com waterPush 0,87 o miolo ficou {m1:.3} > 0,35 (não abriu)"
        ));
    }
    if o1 < 2.2 {
        falhas.push(format!("com waterPush 0,87 a orla ficou {o1:.3} < 2,2× a faixa (o pigmento não foi para a frente)"));
    }
    if b2 > 1.1 {
        falhas.push(format!(
            "sobre a faixa SECA o knob fez orla na borda da pegada: {b2:.3} > 1,1"
        ));
    }
    if e2 < 500.0 + e3 {
        falhas.push(format!("sobre a faixa SECA o dissolvido não escorreu pelo traço: {e2:.1} < 500 + {e3:.1} sem o knob"));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// O knob a 0 é a água de antes AO BYTE: os planos do motor iguais com e sem a escrita explícita do neutro.
#[test]
fn a_agua_sem_empurrao_e_a_de_antes() {
    let mut a = faixa(false);
    let mut b = faixa(false);
    agua(&mut a, None);
    agua(&mut b, Some(0.0));
    let (ga, gb) = (a.active_grid(), b.active_grid());
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&ga.susp), bits(&gb.susp));
    assert_eq!(bits(&ga.sett), bits(&gb.sett));
    assert_eq!(bits(&ga.film), bits(&gb.film));
}

/// A faixa vermelha MOLHADA misturada por um traço do Blend (ferramenta do motor) em vertical por x = 130, com o grão
/// `grao` na máscara do misturador; devolve o pigmento suspenso da área do cruzamento.
fn mistura_com_grao(grao: Option<&(dyn Fn(i32, i32) -> f64 + Sync)>) -> Vec<f32> {
    let mut e = faixa(false);
    e.tool = Tool::Blend;
    e.begin_direct_stroke(0, X as f64, 20.0);
    let mut y = 20.0;
    while y <= 120.0 {
        e.direct_segment(0, 2.0);
        e.dispatch_pressure_dab_lane_blend_grao(0, X as f64, y, 5.0, 12.0, grao);
        y += 2.0;
    }
    e.end_direct_stroke();
    let g = e.active_grid();
    (40..100)
        .flat_map(|y| (110..150).map(move |x| x + 1 + (y + 1) * g.s))
        .map(|i| g.susp[i])
        .collect()
}

/// ⭐ **O grão modula a máscara do misturador** (L-2 Blender Grainy): o Blend do Wet com o Grain armado mistura onde
/// o grão deixa — dois grãos diferentes dão misturas diferentes (o Rebelle «Textured» funde em manchas granuladas; o
/// nosso dava a MESMA imagem ao byte com seis texturas) — e sem grão é a porta de sempre, ao bit.
#[test]
fn o_grao_modula_o_misturador() {
    let sem = mistura_com_grao(None);
    let xadrez = |x: i32, y: i32| f64::from(u8::from((x / 4 + y / 4) % 2 == 0));
    let riscas = |x: i32, _y: i32| f64::from(u8::from((x / 3) % 2 == 0));
    let a = mistura_com_grao(Some(&xadrez));
    let b = mistura_com_grao(Some(&riscas));
    let difere = |u: &[f32], v: &[f32]| {
        u.iter()
            .zip(v)
            .filter(|(p, q)| (*p - *q).abs() > 0.5)
            .count()
    };
    let (sa, ab) = (difere(&sem, &a), difere(&a, &b));
    println!("[mistura] grão no Blend: células que mudam sem × xadrez {sa} · xadrez × riscas {ab}");
    assert!(
        sa > 100 && ab > 100,
        "o grão não mudou a mistura: {sa} · {ab} células"
    );
    // Sem grão = a porta antiga, ao bit.
    let mut e = faixa(false);
    e.tool = Tool::Blend;
    e.begin_direct_stroke(0, X as f64, 20.0);
    let mut y = 20.0;
    while y <= 120.0 {
        e.direct_segment(0, 2.0);
        e.dispatch_pressure_dab_lane_blend(0, X as f64, y, 5.0, 12.0);
        y += 2.0;
    }
    let g = e.active_grid();
    let antiga: Vec<f32> = (40..100)
        .flat_map(|y| (110..150).map(move |x| x + 1 + (y + 1) * g.s))
        .map(|i| g.susp[i])
        .collect();
    assert!(
        antiga
            .iter()
            .zip(&sem)
            .all(|(p, q)| p.to_bits() == q.to_bits()),
        "sem grão a porta mudou"
    );
}

/// A faixa MOLHADA com SULCOS (uma linha em cada quatro sem pigmento: as riscas das cerdas da tinta fresca).
fn faixa_riscada() -> Engine {
    let mut e = faixa(false);
    let g = e.active_grid_mut();
    for y in (Y - 12..=Y + 12).filter(|y| y % 4 == 0) {
        for x in 20..=240 {
            g.susp[x + 1 + (y + 1) * g.s] = 0.0;
        }
    }
    e
}

/// O secador (ferramenta Dry) em vertical por x = 130, com o `edge` dado.
fn seca_com(e: &mut Engine, edge: Option<f64>) {
    if let Some(v) = edge {
        e.tuning.set(Knob::DryerEdge, v);
    }
    let mut prev: Option<[f64; 2]> = None;
    let mut y = 20.0;
    while y <= 120.0 {
        e.dispatch_pressure_dab_tool(Tool::Dry, X as f64, y, 5.0, 10.0, prev);
        prev = Some([X as f64, y]);
        y += 2.0;
    }
}

/// ⭐ **O secador assenta e deixa orla** (`Knob::DryerEdge`, L-7 Dry/Lift): com o knob, a mancha que o secador sela
/// tem a LINHA escura por dentro da borda (o pigmento a ±(r−3..r) passa 1,15× a faixa) e o CLARO por fora (±(r+1..r+2)
/// abaixo de 0,9×) — a mancha seca do Rebelle: 1,37 dentro, 0,84 fora — e os sulcos das cerdas dentro dela NIVELAM
/// (o desvio entre linhas no eixo cai abaixo de metade). Sem o knob, o secador de sempre ao bit.
#[test]
fn o_secador_assenta_e_deixa_orla() {
    let mede = |edge: Option<f64>| {
        let mut e = faixa_riscada();
        seca_com(&mut e, edge);
        let g = e.active_grid();
        let px = |x: usize, y: usize| {
            f64::from(g.susp[x + 1 + (y + 1) * g.s]) + f64::from(g.sett[x + 1 + (y + 1) * g.s])
        };
        let faixa_em = |x: usize| (Y - 8..=Y + 8).map(|y| px(x, y)).sum::<f64>() / 17.0;
        let intacta = (40..80).map(faixa_em).sum::<f64>() / 40.0;
        let dentro = [X - 9, X - 8, X - 7, X + 7, X + 8, X + 9]
            .map(faixa_em)
            .iter()
            .sum::<f64>()
            / 6.0
            / intacta;
        let fora = [X - 12, X - 11, X + 11, X + 12]
            .map(faixa_em)
            .iter()
            .sum::<f64>()
            / 4.0
            / intacta;
        let col: Vec<f64> = (Y - 8..=Y + 8)
            .map(|y| (X - 2..=X + 2).map(|x| px(x, y)).sum::<f64>() / 5.0)
            .collect();
        let med = col.iter().sum::<f64>() / col.len() as f64;
        let desvio = (col.iter().map(|v| (v - med).powi(2)).sum::<f64>() / col.len() as f64).sqrt()
            / med.max(1e-9);
        println!(
            "[mistura] secador edge={edge:?}: dentro {dentro:.3} · fora {fora:.3} · desvio no eixo {desvio:.3}"
        );
        (dentro, fora, desvio)
    };
    let (d0, f0, s0) = mede(None);
    let (d1, f1, s1) = mede(Some(0.5));
    let mut falhas = Vec::new();
    if d1 < 1.15 {
        falhas.push(format!(
            "sem linha por dentro da borda: {d1:.3} < 1,15 (sem o knob {d0:.3})"
        ));
    }
    if f1 > 0.9 {
        falhas.push(format!(
            "sem o claro por fora: {f1:.3} > 0,9 (sem o knob {f0:.3})"
        ));
    }
    if s1 > 0.5 * s0 {
        falhas.push(format!(
            "os sulcos não nivelaram: desvio {s1:.3} > ½ de {s0:.3}"
        ));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
    // Sem o knob = o secador de antes, ao bit.
    let (mut a, mut b) = (faixa_riscada(), faixa_riscada());
    seca_com(&mut a, None);
    seca_com(&mut b, Some(0.0));
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&a.active_grid().susp), bits(&b.active_grid().susp));
    assert_eq!(bits(&a.active_grid().film), bits(&b.active_grid().film));
}
