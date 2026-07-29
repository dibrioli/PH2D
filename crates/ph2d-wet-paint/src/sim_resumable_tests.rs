//! **O passo por ESTÁGIOS contra a rota atômica CONGELADA** — o oráculo da wave.
//!
//! Filho de [`super`] (`#[path]`) porque a referência é `#[cfg(test)]` e um teste de integração não a
//! alcança: `tests/` compila a crate **sem** `cfg(test)`.
//!
//! ⚠️ **Por que a referência é congelada e não `sim_step`:** [`super::sim_step`] **É** o laço sobre
//! [`super::sim_step_stage`], então comparar os dois passa pelo MESMO código e uma mutação dentro do
//! estágio move os dois lados — *razão entre dois doentes*. Medido: com o gate escrito daquele jeito,
//! **três mutações sobreviveram** (params re-colhidos por estágio · relógio andando por estágio · o
//! `apply_boundaries` pulado). Contra a referência, as três sangram.

use super::*;
use crate::grid::Grid;

/// Uma folha pequena com água semeada à mão — sem depender do harness de `tests/`.
fn seeded(w: usize, h: usize) -> Grid {
    let mut g = Grid::new(w, h);
    // Uma faixa diagonal de filme + pigmento: cobre `film`, `susp` e a bbox.
    for k in 0..40 {
        let x = 8 + k;
        let y = 6 + k / 2;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (2, 1)] {
            let (cx, cy) = (x + dx, y + dy);
            if cx >= w || cy >= h {
                continue;
            }
            let i = cy * g.s + cx;
            g.film[i] = 0.6;
            g.susp[i] = 0.3;
            g.susp_rgb[i] = [0.2, 0.4, 0.8];
            g.wet[i] = 200;
            g.expand_bbox(cx as i32, cy as i32, cx as i32, cy as i32);
        }
    }
    g.has_fluid = true;
    g
}

fn digest(g: &Grid) -> Vec<u8> {
    let mut v = Vec::with_capacity(g.cells * 40);
    let f = |v: &mut Vec<u8>, a: &[f32]| {
        for x in a {
            v.extend_from_slice(&x.to_bits().to_le_bytes());
        }
    };
    let f3 = |v: &mut Vec<u8>, a: &[[f32; 3]]| {
        for x in a {
            for c in x {
                v.extend_from_slice(&c.to_bits().to_le_bytes());
            }
        }
    };
    f(&mut v, &g.film);
    f(&mut v, &g.susp);
    f(&mut v, &g.sett);
    f3(&mut v, &g.susp_rgb);
    f3(&mut v, &g.sett_rgb);
    f(&mut v, &g.vel_x);
    f(&mut v, &g.vel_y);
    v.extend_from_slice(&g.wet);
    v.extend_from_slice(&g.active);
    v.extend_from_slice(&g.bloom);
    for n in [g.bx0, g.by0, g.bx1, g.by1] {
        v.extend_from_slice(&n.to_le_bytes());
    }
    v.push(u8::from(g.has_fluid));
    v
}

/// ⚠️ **61 passos, primo com 2, 3 e 4** — as cadências do passo são `n % 2` (rebuild), `n % 3`
/// (projeção), `n % 4` (flow × smooth) e `n % dry_every` (que MUDA no último estágio, conforme o
/// `vmax`): uma sessão curta não visita todas as combinações.
const STEPS: usize = 61;

fn run(staged: bool, gravity: Option<[f64; 2]>) -> (Grid, Sim) {
    let mut g = seeded(160, 120);
    let mut sim = Sim {
        gravity_override: gravity,
        ..Sim::default()
    };
    let tuning = Tuning::default();
    for _ in 0..STEPS {
        if staged {
            // UM estágio por chamada — a granularidade que o produto usa quando o orçamento do
            // frame não cabe um passo inteiro.
            while sim_step_stage(&mut sim, &mut g, &tuning).is_none() {}
        } else {
            sim_step_atomic_reference(&mut sim, &mut g, &tuning);
        }
    }
    (g, sim)
}

#[test]
fn the_staged_step_is_byte_identical_to_the_frozen_atomic_route() {
    // Sem gravidade e COM: o segundo faz `vmax` cruzar 0,5 e o `dry_every` trocar no meio da
    // sessão — o estado do `Sim` que o cursor NÃO carrega, de propósito.
    for grav in [None, Some([0.0, 1.0])] {
        let (ga, sa) = run(false, grav);
        let (gs, ss) = run(true, grav);
        let (da, ds) = (digest(&ga), digest(&gs));
        let at = da.iter().zip(ds.iter()).position(|(a, b)| a != b);
        assert_eq!(
            at, None,
            "grav {grav:?}: o passo por estagios divergiu da rota atomica congelada no byte {at:?}"
        );
        assert_eq!(
            (sa.frame, sa.dry_every),
            (ss.frame, ss.dry_every),
            "grav {grav:?}: o estado do Sim divergiu (relogio / cadencia de secagem)"
        );
    }
}

/// **E o passo tem SEIS pontos de INTERRUPÇÃO, não menos** — o número é o que torna a interrupção
/// útil (o maior estágio sozinho custa 10,26 ms contra 38,7 do passo inteiro). Um estágio que
/// fizesse dois trabalhos passaria no gate de identidade e devolveria a travada.
///
/// Seis `None` + o `Some` que fecha = os sete braços do `match`, sendo o último a cadência de
/// secagem (que não faz trabalho de grid).
#[test]
fn a_step_has_six_resumption_points() {
    let mut g = seeded(160, 120);
    let mut sim = Sim::default();
    let tuning = Tuning::default();
    let mut stages = 0;
    while sim_step_stage(&mut sim, &mut g, &tuning).is_none() {
        stages += 1;
        assert!(stages < 64, "um passo nao completou em 64 estagios");
    }
    assert_eq!(
        stages, 6,
        "o passo rodou em {stages} pontos de interrupcao, nao 6 — o maior estagio decide o pior \
         frame, e juntar trabalho num estagio devolve a travada"
    );
}

// ⚠️ **MUTAÇÃO SOBREVIVENTE, documentada em vez de escondida:** re-colher os params em CADA estágio
// (`c.p = sim.gather_params(tuning)` no topo do `sim_step_stage`) **não é pego por nenhum gate
// deste arquivo**. Tentei duas fixtures — mexer `Gravity` no meio de um passo (inválida: ela vira o
// `c.grav`, capturado à parte, então a mutação não a alcança) e mexer `ExtDiffusion` (o controle
// prova que o knob NÃO é inerte, e a mutação passou de todo modo) — e nenhuma discriminou.
//
// A lei continua escrita no [`super::StepCursor`] e é real (meia física com a lei nova e meia com a
// velha não é nem uma nem outra), mas **está sem oráculo**: um gate que não pode falhar pelo motivo
// que alega é pior que gate nenhum, então ele não foi shipado. Quem achar a fixture que separa as
// duas rotas fecha isto — e o caminho é um knob que o `Params` carrega, que um estágio ≥ 1 leia, e
// cuja mudança de valor mude BYTES naquele estágio.
