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

use crate::util;

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
        first_difference(
            &digest(run(false).active_grid()),
            &digest(run(true).active_grid())
        ),
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

/// **B13.2 — a máscara de tiles da faixa reproduz a faixa de UM intervalo ao bit.** O oráculo é a mesma
/// sessão com `tiles_enabled = false` (o intervalo inteiro da linha, o `span_x_of` de antes); as seis
/// formas de cima e duas manchas nas MESMAS linhas, longe uma da outra, com a água a escorrer de lado
/// (gravidade em x: cruza as costuras dos tiles de 64).
fn run_tiles(kind: usize, tiles: bool) -> Engine {
    if kind < SESSIONS {
        run_session_tiles(kind, tiles)
    } else {
        run_far_blobs(tiles, false)
    }
}

/// As duas manchas longe; `outra_lei` liga a tinta Pigmento, a borda graduada e o pigmento que desce
/// (os estágios que a sessão de omissão não percorre: `drying/tinta`, `soft_plane`, `sedimenta`).
fn run_far_blobs(tiles: bool, outra_lei: bool) -> Engine {
    {
        let mut e = Engine::new(700, 160);
        for l in &mut e.layers {
            l.grid.tiles_enabled = tiles;
        }
        if outra_lei {
            e.sim.tinta = ph2d_wet_paint::sim::LeiDaTinta::Pigmento;
            e.sim.sedimenta = 0.5;
            e.tuning
                .set(ph2d_wet_paint::tuning::Knob::EdgeSoftMass, 0.5);
        }
        e.sliders.water = 1.0;
        e.sim.gravity_override = Some([1.0, 0.3]);
        drive_stroke(&mut e, 40.0, 30.0, 60.0, 130.0, 4.0, 10);
        drive_stroke(&mut e, 620.0, 30.0, 640.0, 130.0, 4.0, 120);
        e
    }
}

/// As duas manchas longe na grade de FLUXO `rf` (a máscara projetada, `frow_tiles`), com o fluxo serial
/// (`build_flow_field`) ou o Jacobi (`order_invariant`: núcleo, carimbo de umidade e backrun).
fn run_far_blobs_rf(tiles: bool, rf: usize, jacobi: bool) -> Engine {
    let mut e = Engine::with_flow_ratio(700, 160, rf);
    for l in &mut e.layers {
        l.grid.tiles_enabled = tiles;
    }
    e.sim.order_invariant = jacobi;
    e.sliders.water = 1.0;
    e.sim.gravity_override = Some([1.0, 0.3]);
    drive_stroke(&mut e, 40.0, 30.0, 60.0, 130.0, 4.0, 10);
    drive_stroke(&mut e, 620.0, 30.0, 640.0, 130.0, 4.0, 120);
    e
}

/// O meio de óleo da cena com óleo: listrado em `x` e `y`, para o gather do meio ter fontes diferentes.
fn meio_listrado(x: usize, y: usize) -> f32 {
    ((x * 7 + y * 3) % 11) as f32 * 0.09
}

/// As duas manchas longe com o campo `oleo.meio` listrado (o Jacobi: `meio_transporte` e a ladeira de Bingham).
/// Devolve `(digest com o meio, o grid)`.
fn run_far_blobs_oleo(tiles: bool, rf: usize) -> (Vec<u8>, Engine) {
    use ph2d_wet_paint::grid::oleo_celula::{OleoCelula, campo_de};
    let mut e = Engine::with_flow_ratio(700, 160, rf);
    for l in &mut e.layers {
        let g = &mut l.grid;
        g.tiles_enabled = tiles;
        let s = g.s;
        let v: Vec<OleoCelula> = (0..g.cells)
            .map(|i| OleoCelula {
                meio: meio_listrado(i % s, i / s),
                ..OleoCelula::NEUTRA
            })
            .collect();
        g.oleo = campo_de(&v);
    }
    e.sim.order_invariant = true;
    e.sliders.water = 1.0;
    e.sim.gravity_override = Some([1.0, 0.3]);
    drive_stroke(&mut e, 40.0, 30.0, 60.0, 130.0, 4.0, 10);
    drive_stroke(&mut e, 620.0, 30.0, 640.0, 130.0, 4.0, 120);
    let g = e.active_grid();
    let mut d = digest(g);
    for i in 0..g.cells {
        d.extend_from_slice(&g.oleo[i].meio.to_bits().to_le_bytes());
    }
    (d, e)
}

/// Duas poças FUNDAS (filme até 7: o carimbo de umidade escreve) separadas por quatro tiles secos, com
/// pigmento assentado no raso e meio orçamento de floração gasto (o backrun dispara nas DUAS). Três
/// passes do Jacobi com `ExtBackrun`; devolve o grid.
fn deep_far_pools_jacobi(tiles: bool, rf: usize, ate_a_borda: bool) -> Grid {
    deep_far_pools(tiles, rf, ate_a_borda, true)
}

/// [`deep_far_pools_jacobi`] com o fluxo serial (`jacobi = false`, `build_flow_field`) ou o Jacobi.
fn deep_far_pools(tiles: bool, rf: usize, ate_a_borda: bool, jacobi: bool) -> Grid {
    use ph2d_wet_paint::solver::{
        build_flow_field, build_flow_field_jacobi, rebuild_active_region,
    };
    use ph2d_wet_paint::tuning::{Knob, Tuning};
    let (w, h) = (600usize, 34usize);
    let mut g = Grid::with_flow_ratio(w, h, rf);
    g.tiles_enabled = tiles;
    let s = g.s;
    for cx in [48.5f64, 500.5] {
        for y in 5..=if ate_a_borda { h } else { h - 5 } {
            for x in 1..=w {
                let d = (x as f64 - cx).abs();
                if d >= 30.0 {
                    continue;
                }
                let i = x + y * s;
                let teeth = if (d.floor() as i32) % 7 < 3 { 1.3 } else { 0.7 };
                let bump = 1.0 - d / 30.0;
                g.film[i] = (7.0 * bump * teeth) as f32;
                g.susp[i] = (400.0 * bump) as f32;
                g.susp_rgb[i] = [200.0, 40.0, 90.0];
                g.paper[i] = (0.35 + 0.25 * teeth) as f32;
                if g.film[i] < 2.0 {
                    g.sett[i] = 500.0;
                    g.sett_rgb[i] = [30.0, 120.0, 200.0];
                } else if y % 2 == 0 {
                    g.bloom[i] = 6;
                }
            }
        }
    }
    g.expand_bbox(1, 1, w as i32, h as i32);
    rebuild_active_region(&mut g);
    let mut t = Tuning::default();
    t.set(Knob::ExtBackrun, 1.5);
    let p = ph2d_wet_paint::sim::Sim::default().gather_params(&t);
    for _ in 0..3 {
        if jacobi {
            build_flow_field_jacobi(&mut g, &p, 0.0, 0.6, false);
        } else {
            build_flow_field(&mut g, &p, 0.0, 0.6, false);
        }
    }
    g
}

fn run_session_tiles(kind: usize, tiles: bool) -> Engine {
    // As formas de `run_session`, com a máscara ligada/desligada desde o 1.º passo.
    let mut e = Engine::new(220, 180);
    for l in &mut e.layers {
        l.grid.tiles_enabled = tiles;
    }
    e.sliders.water = 1.0;
    match kind {
        0 => drive_stroke(&mut e, 20.0, 90.0, 200.0, 90.0, 4.0, 40),
        1 => drive_stroke(&mut e, 20.0, 20.0, 200.0, 160.0, 4.0, 40),
        2 => {
            e.sim.gravity_override = Some([0.0, 1.0]);
            drive_stroke(&mut e, 40.0, 30.0, 180.0, 30.0, 3.0, 120);
        }
        3 => {
            drive_stroke(&mut e, 30.0, 40.0, 90.0, 40.0, 4.0, 200);
            drive_stroke(&mut e, 30.0, 45.0, 190.0, 130.0, 4.0, 60);
        }
        4 => {
            drive_stroke(&mut e, 30.0, 60.0, 190.0, 70.0, 4.0, 60);
            e.tool = Tool::Wet;
            drive_stroke(&mut e, 30.0, 62.0, 190.0, 100.0, 5.0, 30);
            e.tool = Tool::Blend;
            drive_stroke(&mut e, 60.0, 60.0, 160.0, 120.0, 5.0, 30);
        }
        _ => {
            drive_stroke(&mut e, 20.0, 20.0, 20.0, 160.0, 4.0, 10);
            drive_stroke(&mut e, 20.0, 160.0, 200.0, 160.0, 4.0, 40);
        }
    }
    e
}

#[test]
fn a_mascara_de_tiles_da_faixa_reproduz_a_faixa_de_um_intervalo_ao_bit() {
    for kind in 0..=SESSIONS {
        let um = digest(run_tiles(kind, false).active_grid());
        let tiles = digest(run_tiles(kind, true).active_grid());
        assert_eq!(um.len(), tiles.len());
        if let Some(at) = first_difference(&um, &tiles) {
            panic!("sessao {kind}: a mascara de tiles mudou o estado no byte {at}");
        }
    }
    // As duas manchas longe com cada rota do commit forçada (1 = cópia dos troços, 2 = troca + complemento).
    use std::sync::atomic::Ordering::Relaxed;
    for rota in [1u8, 2] {
        ph2d_wet_paint::flow::COMMIT_ROTA.store(rota, Relaxed);
        let um = digest(run_tiles(SESSIONS, false).active_grid());
        let tiles = digest(run_tiles(SESSIONS, true).active_grid());
        ph2d_wet_paint::flow::COMMIT_ROTA.store(0, Relaxed);
        if let Some(at) = first_difference(&um, &tiles) {
            panic!("rota do commit {rota}: a mascara de tiles mudou o estado no byte {at}");
        }
    }
    let um = digest(run_far_blobs(false, true).active_grid());
    let tiles = digest(run_far_blobs(true, true).active_grid());
    if let Some(at) = first_difference(&um, &tiles) {
        panic!(
            "Pigmento + borda graduada + sedimenta: a mascara de tiles mudou o estado no byte {at}"
        );
    }
    for rf in [1usize, 2, 3] {
        for jacobi in [false, true] {
            let um = digest(run_far_blobs_rf(false, rf, jacobi).active_grid());
            let tiles = digest(run_far_blobs_rf(true, rf, jacobi).active_grid());
            if let Some(at) = first_difference(&um, &tiles) {
                panic!(
                    "grade de fluxo rf {rf} (jacobi {jacobi}): frow_tiles mudou o estado no byte {at}"
                );
            }
        }
    }
    // Óleo: o meio anda com o líquido nas duas manchas (o gather do meio e a ladeira de Bingham).
    for rf in [1usize, 2] {
        let (um, _) = run_far_blobs_oleo(false, rf);
        let (tiles, e) = run_far_blobs_oleo(true, rf);
        if let Some(at) = first_difference(&um, &tiles) {
            panic!("oleo rf {rf}: a mascara de tiles mudou o estado no byte {at}");
        }
        let g = e.active_grid();
        let s = g.s;
        let andou = (0..g.cells)
            .filter(|i| i % s >= 560)
            .any(|i| g.oleo[i].meio.to_bits() != meio_listrado(i % s, i / s).to_bits());
        assert!(
            andou,
            "oleo rf {rf}: o meio nao andou na 2.a mancha (cena vazia)"
        );
    }
    // Poças fundas + backrun: o carimbo de umidade e o orçamento de floração na 2.ª poça.
    for rf in [1usize, 2] {
        let um = deep_far_pools_jacobi(false, rf, false);
        let tiles = deep_far_pools_jacobi(true, rf, false);
        let (a, b) = (digest(&um), digest(&tiles));
        if let Some(at) = first_difference(&a, &b) {
            panic!("pocas fundas rf {rf}: frow_tiles mudou o estado no byte {at}");
        }
        let s = um.s;
        let far_wet =
            (1..=um.h).filter(|y| um.wet[460 + y * s..540 + y * s].iter().any(|v| *v != 0));
        assert!(
            far_wet.count() > 0,
            "rf {rf}: o carimbo nao rodou na 2.a poca (cena vazia)"
        );
    }
    // A máscara não é vazia de efeito: nas duas manchas longe ela salta o seco do meio.
    let e = run_tiles(SESSIONS, true);
    let g = e.active_grid();
    let (mut faixa, mut troços) = (0i64, 0i64);
    for y in g.by0..=g.by1 {
        let (lo, hi) = g.span_x(y);
        faixa += (i64::from(hi) - i64::from(lo) + 1).max(0);
        for (a, b) in ph2d_wet_paint::grid::faixa_tiles::span_runs(g.tiles_row(y), lo, hi) {
            troços += i64::from(b - a + 1);
        }
    }
    assert!(
        troços * 2 < faixa,
        "a mascara nao saltou o vao: {troços} de {faixa}"
    );
}

/// A água na ÚLTIMA linha da grade com o backrun ligado: o empurrão olha o vizinho de baixo (`fi + rf·fine_s`), que em
/// `rf > 1` cai fora da grade fina. Fora da grade não há vizinho, não há empurrão (a borda é seca).
#[test]
fn o_backrun_na_ultima_linha_nao_sai_da_grade() {
    for rf in [1usize, 2, 3] {
        for jacobi in [false, true] {
            let g = deep_far_pools(true, rf, true, jacobi);
            assert!(
                g.film.iter().all(|v| v.is_finite()),
                "rf {rf}: filme nao finito"
            );
        }
    }
}
