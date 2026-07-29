//! **O PASSO INTERROMPÍVEL** — o gate que sustenta a wave inteira.
//!
//! Um passo custa **38,7 ms** na escala do produto (medido, `tests/measure_pass_cost.rs`) contra
//! 16,6 de um quadro de 60 Hz, então enquanto ele era ATÔMICO o frame que o continha estourava **por
//! construção** e nenhum orçamento de tempo consertava isso. O maior ESTÁGIO sozinho custa 10,26 ms
//! e **cabe** na folga do quadro ⇒ o passo passou a ser retomável
//! ([`ph2d_wet_paint::sim::StepCursor`]).
//!
//! ⚠️ **A correção é byte-idêntica POR CONSTRUÇÃO** — os mesmos estágios, na mesma ordem, com os
//! MESMOS params (capturados no início do passo, não a cada estágio) —, e
//! [`ph2d_wet_paint::sim::sim_step`] virou *o laço sobre os estágios*, então não existe uma segunda
//! implementação para divergir. Este arquivo **afirma** isso em vez de confiar na prosa: só a
//! identidade sobre uma sessão inteira distingue *"escrevi o laço certo"* de *"escrevi um laço"*.

mod util;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::{Engine, Tool};
use util::drive_stroke;

/// Todo campo persistente do grid, byte a byte (o mesmo dígito de `tests/spans.rs`).
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

/// As sessões: cada uma põe uma CADÊNCIA diferente sob o cursor. As cadências do passo são `n % 2`
/// (rebuild), `n % dry_every` (secagem, e o `dry_every` MUDA no último estágio), `n % 4` (flow ×
/// smooth) e `n % 3` (projeção) — uma sessão curta não visita todas as combinações.
fn run_session(kind: usize, staged: bool) -> Engine {
    let mut e = Engine::new(200, 160);
    e.sliders.water = 1.0;
    let step = |e: &mut Engine, n: usize| {
        for _ in 0..n {
            if staged {
                // UM estágio por chamada — a granularidade que o produto usa quando o orçamento do
                // frame não cabe um passo inteiro.
                while e.step_stage().is_none() {}
            } else {
                e.step_simulation();
            }
        }
    };
    match kind {
        0 => {
            drive_stroke(&mut e, 20.0, 80.0, 180.0, 80.0, 4.0, 0);
            step(&mut e, 61); // primo com 2, 3 e 4 — atravessa as quatro cadências
        }
        1 => {
            // Drip sob gravidade: a água ANDA, então `vmax` cruza o limiar de 0,5 e o `dry_every`
            // troca no meio da sessão (é o estado do `Sim` que o cursor NÃO carrega, de propósito).
            e.sim.gravity_override = Some([0.0, 1.0]);
            drive_stroke(&mut e, 40.0, 30.0, 160.0, 30.0, 3.0, 0);
            step(&mut e, 97);
        }
        2 => {
            // Dois traços com sim ENTRE eles: um passo começa com tinta nova na folha.
            drive_stroke(&mut e, 30.0, 40.0, 90.0, 40.0, 4.0, 0);
            step(&mut e, 23);
            drive_stroke(&mut e, 30.0, 45.0, 170.0, 120.0, 4.0, 0);
            step(&mut e, 41);
        }
        3 => {
            // Wet + Blend sobre tinta seca (os predicados de `susp` sem `film`).
            drive_stroke(&mut e, 30.0, 60.0, 170.0, 70.0, 4.0, 0);
            step(&mut e, 13);
            e.tool = Tool::Wet;
            drive_stroke(&mut e, 30.0, 62.0, 170.0, 100.0, 5.0, 0);
            step(&mut e, 17);
            e.tool = Tool::Blend;
            drive_stroke(&mut e, 60.0, 60.0, 140.0, 110.0, 5.0, 0);
            step(&mut e, 29);
        }
        _ => {
            // Secar por completo e continuar simulando: o passo aborta no estágio 0 (`has_fluid`
            // caiu no rebuild) — o caminho que devolve `Some(false)` SEM cadência e SEM cursor.
            drive_stroke(&mut e, 40.0, 40.0, 60.0, 60.0, 3.0, 0);
            e.fast_dry_now();
            step(&mut e, 7);
        }
    }
    e
}

const SESSIONS: usize = 5;

#[test]
fn a_step_run_stage_by_stage_is_byte_identical_to_the_atomic_one() {
    for kind in 0..SESSIONS {
        let atomic = digest(run_session(kind, false).active_grid());
        let staged = digest(run_session(kind, true).active_grid());
        assert_eq!(
            atomic.len(),
            staged.len(),
            "sessao {kind}: digests de tamanhos diferentes"
        );
        let at = atomic.iter().zip(staged.iter()).position(|(a, b)| a != b);
        assert_eq!(
            at, None,
            "sessao {kind}: o passo por estagios divergiu do atomico no byte {at:?}"
        );
    }
}

/// **E o relógio do motor não anda diferente** — `sim.frame` é a cadência de TODOS os `n % k`, então
/// contar passos errado (um estágio que incrementasse o frame, um `Some` cedo demais) mudaria a
/// física sem mudar um byte no primeiro passo.
#[test]
fn the_staged_step_advances_the_clock_exactly_once_per_step() {
    let atomic = run_session(1, false);
    let staged = run_session(1, true);
    assert_eq!(
        atomic.sim.frame, staged.sim.frame,
        "o relogio do motor divergiu: atomico {} vs por estagios {}",
        atomic.sim.frame, staged.sim.frame
    );
}

/// **NENHUM estágio deixa um passo em voo depois de o passo completar** — e o par é o que torna o
/// primeiro gate honesto: sem isto, um cursor esquecido faria o próximo `step_simulation` continuar
/// um passo velho, e a divergência apareceria a uma sessão de distância.
#[test]
fn a_completed_step_leaves_no_cursor_behind() {
    let mut e = Engine::new(120, 100);
    e.sliders.water = 1.0;
    drive_stroke(&mut e, 20.0, 50.0, 100.0, 50.0, 4.0, 0);
    for _ in 0..37 {
        let mut stages = 0u32;
        loop {
            assert!(stages < 64, "um passo nao completou em 64 estagios");
            stages += 1;
            if e.step_stage().is_some() {
                break;
            }
            assert!(
                e.sim.step_pending(),
                "estagio devolveu None e NAO deixou cursor: o passo seguinte comecaria de novo"
            );
        }
        assert!(
            !e.sim.step_pending(),
            "o passo completou e o cursor ficou: o proximo step_simulation continuaria este"
        );
    }
}

/// **E uma ação de CANVAS DRENA o passo em voo** — o grid entre dois estágios é intermediário, e
/// `wet_canvas`/`dry_canvas`/`fast_dry`/`capture_history` agem sobre a folha inteira.
#[test]
fn a_canvas_action_drains_a_step_in_flight() {
    let mut e = Engine::new(120, 100);
    e.sliders.water = 1.0;
    drive_stroke(&mut e, 20.0, 50.0, 100.0, 50.0, 4.0, 0);
    // Um estágio só: deixa o passo em voo de propósito.
    assert!(
        e.step_stage().is_none(),
        "o primeiro estagio ja completou o passo — a fixture nao contem o fenomeno"
    );
    assert!(e.sim.step_pending(), "controle: ha um passo em voo");
    e.wet_canvas_now();
    assert!(
        !e.sim.step_pending(),
        "a acao de canvas rodou COM um passo pela metade: meia fisica misturada com a acao"
    );
}
