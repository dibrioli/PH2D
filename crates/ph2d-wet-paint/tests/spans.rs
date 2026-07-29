//! **A FAIXA VIVA** — o gate diferencial e a propriedade que a justifica.
//!
//! O motor varria a BBOX, e a bbox é o casco da água: num traço diagonal ela
//! é a tela inteira enquanto a água é 2,4% dela (medido a 4096²,
//! `tests/measure_pass_cost.rs`). A faixa por-linha ([`ph2d_wet_paint::grid`])
//! troca o casco por um intervalo por linha.
//!
//! O oráculo é DIFERENCIAL, não um valor pinado: a mesma sessão roda com a
//! faixa LIGADA e DESLIGADA (`Grid::spans_enabled`, que faz a porta devolver a
//! bbox inteira — o intervalo que o motor varria antes) e os dois estados têm
//! de ser idênticos AO BIT. Não há segunda implementação a divergir: é o
//! mesmo laço com um intervalo mais largo.
//!
//! Isto é mais forte que o `fingerprint.rs` de duas maneiras — compara TODO
//! campo persistente (não um hash de sete) e roda várias FORMAS de sessão,
//! incluindo as que a faixa torna diferentes (diagonal, drip sob gravidade,
//! traço novo por cima do rastro seco de um antigo).

mod util;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::{Engine, Tool};
use util::drive_stroke;

/// Todo campo PERSISTENTE do grid, byte a byte.
///
/// `flow_x`/`flow_y` ficam de fora **de propósito**: são o rascunho transiente
/// (o `project` os reusa como divergência/pressão), reconstruído a cada frame
/// nas células ativas e lido só lá — o próprio `fingerprint.rs` não os inclui.
/// Estreitar a limpeza deles é o único ponto onde a faixa muda um byte que
/// ninguém lê, e omiti-los aqui é a afirmação de que ninguém lê.
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

fn first_difference(a: &[u8], b: &[u8]) -> Option<usize> {
    a.iter().zip(b.iter()).position(|(x, y)| x != y)
}

fn set_spans(e: &mut Engine, on: bool) {
    for l in &mut e.layers {
        l.grid.spans_enabled = on;
    }
}

/// As sessões: cada uma existe para pôr uma FORMA diferente sob a faixa.
fn run_session(kind: usize, spans: bool) -> Engine {
    let mut e = Engine::new(220, 180);
    set_spans(&mut e, spans);
    e.sliders.water = 1.0;
    match kind {
        // Horizontal: casco fino, o caso em que a faixa quase não muda nada.
        0 => drive_stroke(&mut e, 20.0, 90.0, 200.0, 90.0, 4.0, 40),
        // DIAGONAL: casco = tela, água esparsa — a forma que motivou a wave.
        1 => drive_stroke(&mut e, 20.0, 20.0, 200.0, 160.0, 4.0, 40),
        // Drip sob gravidade: a água ANDA, e o rastro que ela deixa é onde a
        // velocidade fóssil vive (o termo `vel != 0` da extensão viva).
        2 => {
            e.sim.gravity_override = Some([0.0, 1.0]);
            drive_stroke(&mut e, 40.0, 30.0, 180.0, 30.0, 3.0, 120);
        }
        // Dois traços SEPARADOS, o segundo por cima do rastro do primeiro,
        // com a sim indo a idle no meio: o caso base da indução (a faixa não
        // pode ser esquecida enquanto sobrar velocidade).
        3 => {
            drive_stroke(&mut e, 30.0, 40.0, 90.0, 40.0, 4.0, 200);
            drive_stroke(&mut e, 30.0, 45.0, 190.0, 130.0, 4.0, 60);
        }
        // Wet + Blend sobre tinta seca: os predicados de `susp` sem `film`.
        4 => {
            drive_stroke(&mut e, 30.0, 60.0, 190.0, 70.0, 4.0, 60);
            e.tool = Tool::Wet;
            drive_stroke(&mut e, 30.0, 62.0, 190.0, 100.0, 5.0, 30);
            e.tool = Tool::Blend;
            drive_stroke(&mut e, 60.0, 60.0, 160.0, 120.0, 5.0, 30);
        }
        // Traço em L: duas caixas cujo casco é enorme e a união é fina.
        _ => {
            drive_stroke(&mut e, 20.0, 20.0, 20.0, 160.0, 4.0, 10);
            drive_stroke(&mut e, 20.0, 160.0, 200.0, 160.0, 4.0, 40);
        }
    }
    e
}

const SESSIONS: usize = 6;

#[test]
fn the_live_span_reproduces_the_bounding_box_sweep_to_the_byte() {
    for kind in 0..SESSIONS {
        let wide = digest(run_session(kind, false).active_grid());
        let narrow = digest(run_session(kind, true).active_grid());
        assert_eq!(
            wide.len(),
            narrow.len(),
            "sessao {kind}: digests de tamanhos diferentes"
        );
        if let Some(at) = first_difference(&wide, &narrow) {
            panic!(
                "sessao {kind}: a faixa viva mudou o estado no byte {at} \
                 (bbox {:?} vs faixa {:?})",
                wide[at], narrow[at]
            );
        }
    }
}

/// O Fast Dry roda o seu próprio laço sobre o casco (`fast_dry` halva o filme
/// antes de cada passe forçado) — a faixa o estreita pelo mesmo argumento
/// (`0.5 * 0 == 0`), e este gate é quem afirma que estreitou sem mudar nada.
#[test]
fn fast_dry_survives_the_live_span_to_the_byte() {
    let mut wide = run_session(2, false);
    let mut narrow = run_session(2, true);
    wide.fast_dry_now();
    narrow.fast_dry_now();
    assert_eq!(
        first_difference(&digest(wide.active_grid()), &digest(narrow.active_grid())),
        None,
        "o fast dry divergiu sob a faixa viva"
    );
}

/// O undo do motor devolve um snapshot arbitrário; a faixa é DERIVADA, então
/// o restore a abre inteira e o próximo rebuild a reaperta. Este gate afirma
/// que a rota de histórico continua byte-idêntica.
#[test]
fn the_history_restore_survives_the_live_span_to_the_byte() {
    let run = |spans: bool| {
        let mut e = Engine::new(220, 180);
        set_spans(&mut e, spans);
        e.sliders.water = 1.0;
        drive_stroke(&mut e, 30.0, 40.0, 190.0, 130.0, 4.0, 30);
        e.capture_history();
        drive_stroke(&mut e, 40.0, 130.0, 180.0, 40.0, 4.0, 30);
        e.undo();
        for _ in 0..40 {
            e.step_simulation();
        }
        e
    };
    assert_eq!(
        first_difference(&digest(run(false).active_grid()), &digest(run(true).active_grid())),
        None,
        "a rota de undo divergiu sob a faixa viva"
    );
}

/// **A PROPRIEDADE que torna a wave valer a pena**, e a razão de ela ser um
/// gate e não só uma medição: uma mudança futura que devolva a bbox inteira
/// por linha continuaria CORRETA (todos os gates acima passam) e teria jogado
/// fora o ganho inteiro em silêncio.
#[test]
fn on_a_diagonal_stroke_the_span_is_a_small_fraction_of_the_bounding_box() {
    let e = run_session(1, true);
    let g = e.active_grid();
    let mut span_cells = 0u64;
    let mut bbox_cells = 0u64;
    for y in g.by0..=g.by1 {
        let (lo, hi) = g.span_x(y);
        bbox_cells += (g.bx1 - g.bx0 + 1).max(0) as u64;
        if hi >= lo {
            span_cells += (hi - lo + 1) as u64;
        }
    }
    assert!(bbox_cells > 0, "cena degenerada: bbox vazia");
    let frac = span_cells as f64 / bbox_cells as f64;
    println!("    faixa/bbox no traco diagonal: {:.1}%", 100.0 * frac);
    assert!(
        frac < 0.55,
        "a faixa cobre {:.1}% da bbox num traco diagonal -- o ganho da wave \
         evaporou (esperado bem abaixo disso)",
        100.0 * frac
    );
}
