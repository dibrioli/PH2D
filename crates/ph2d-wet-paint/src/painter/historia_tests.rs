//! O histórico por tiles copy-on-write devolve o grid AO BYTE e retém só o que o traço mudou.

use super::*;
use crate::grid::snapshot_grid;
use std::hash::{Hash, Hasher};

fn h<T: std::fmt::Debug>(st: &mut Vec<u64>, v: &[T]) {
    // `Debug` de f32 distingue `-0.0`; o tamanho entra junto (um plano vazio ≠ um plano de zeros).
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    v.len().hash(&mut hs);
    for x in v {
        format!("{x:?}").hash(&mut hs);
    }
    st.push(hs.finish());
}

/// O hash de TODO o estado que o histórico promete devolver (o mesmo conjunto do `GridSnapshot`).
fn hash_grid(g: &Grid) -> Vec<u64> {
    let s = snapshot_grid(g);
    let mut st = Vec::new();
    h(&mut st, &s.film);
    h(&mut st, &s.susp);
    h(&mut st, &s.susp_rgb);
    h(&mut st, &s.sett);
    h(&mut st, &s.sett_rgb);
    h(&mut st, &s.relevo);
    h(&mut st, &s.altura);
    h(&mut st, &s.espessura);
    h(&mut st, &s.crista);
    h(&mut st, &s.nivelado);
    h(&mut st, &s.vela);
    h(&mut st, &s.material);
    h(&mut st, &s.aparencia);
    h(&mut st, &s.oleo);
    h(&mut st, &s.vel_x);
    h(&mut st, &s.vel_y);
    h(&mut st, &s.wet);
    h(&mut st, &s.active);
    h(&mut st, &s.bloom);
    h(&mut st, &s.row_lo);
    h(&mut st, &s.row_hi);
    h(
        &mut st,
        &[(s.bx0, s.by0, s.bx1, s.by1, s.has_fluid, s.paper_sheet)],
    );
    st
}

fn traco(e: &mut Engine, x: f64, y: f64, frames: usize) {
    e.pointer_down(x, y, None);
    for i in 0..frames {
        e.pointer_frame(x + 6.0 * i as f64, y + 2.0 * i as f64);
    }
    e.pointer_up();
    let last = (x + 6.0 * frames as f64, y + 2.0 * frames as f64);
    while e.release_frame(last.0, last.1) {}
}

fn corre(e: &mut Engine, passos: usize) {
    for _ in 0..passos {
        e.step_simulation();
    }
}

fn desfazer_e_refazer_devolvem_cada_estado_ao_byte(rf: usize) {
    let mut e = Engine::with_flow_ratio(300, 220, rf);
    e.sliders.water = 1.0;
    let mut antes = Vec::new();
    for (i, (x, y)) in [(40.0, 40.0), (200.0, 60.0), (60.0, 150.0), (180.0, 170.0)]
        .into_iter()
        .enumerate()
    {
        antes.push(hash_grid(e.active_grid()));
        traco(&mut e, x, y, 6 + i);
        let sem_sim = hash_grid(e.active_grid());
        corre(&mut e, 25);
        assert_ne!(
            sem_sim,
            hash_grid(e.active_grid()),
            "a água não correu depois do traço {i}"
        );
    }
    let fim = hash_grid(e.active_grid());
    for k in (0..4).rev() {
        assert!(e.undo());
        assert_eq!(
            hash_grid(e.active_grid()),
            antes[k],
            "undo até antes do traço {k} (rf {rf})"
        );
    }
    assert!(!e.undo());
    let depois: Vec<Vec<u64>> = antes[1..].iter().cloned().chain([fim.clone()]).collect();
    for (k, alvo) in depois.iter().enumerate() {
        assert!(e.redo());
        assert_eq!(hash_grid(e.active_grid()), *alvo, "redo {k} (rf {rf})");
    }
    // A água continua a correr depois de um desfazer: o refazer devolve o estado do traço e um novo
    // desfazer devolve o estado com a deriva.
    assert!(e.undo());
    assert!(e.undo());
    corre(&mut e, 20);
    let deriva = hash_grid(e.active_grid());
    assert_ne!(
        deriva, antes[2],
        "a água parou depois do desfazer — o caso da deriva não foi exercido"
    );
    assert!(e.redo());
    assert_eq!(
        hash_grid(e.active_grid()),
        antes[3],
        "redo depois da deriva (rf {rf})"
    );
    corre(&mut e, 5);
    assert!(e.redo());
    assert_eq!(
        hash_grid(e.active_grid()),
        fim,
        "2.º redo depois da deriva (rf {rf})"
    );
    assert!(e.undo());
    assert!(e.undo());
    assert_eq!(
        hash_grid(e.active_grid()),
        deriva,
        "undo volta à deriva (rf {rf})"
    );
}

#[test]
fn historia_cow_desfaz_e_refaz_ao_byte_com_a_agua_a_correr() {
    desfazer_e_refazer_devolvem_cada_estado_ao_byte(1);
}

#[test]
fn historia_cow_desfaz_e_refaz_ao_byte_na_grade_de_fluxo_grossa() {
    desfazer_e_refazer_devolvem_cada_estado_ao_byte(2);
}

#[test]
fn historia_cow_acoes_de_tela_inteira_desfazem_ao_byte() {
    let mut e = Engine::new(200, 150);
    traco(&mut e, 50.0, 50.0, 5);
    corre(&mut e, 10);
    let a = hash_grid(e.active_grid());
    e.action_dry_canvas();
    let b = hash_grid(e.active_grid());
    e.action_clear();
    assert!(e.undo());
    assert_eq!(hash_grid(e.active_grid()), b);
    assert!(e.undo());
    assert_eq!(hash_grid(e.active_grid()), a);
    assert!(e.redo());
    assert!(e.redo());
    assert!(
        e.active_grid().sett.iter().all(|v| *v == 0.0),
        "o redo do clear não limpou"
    );
}

/// Seis traços pequenos numa tela 4096×2560: o histórico retém ≤ 10 % das seis fotos inteiras.
#[test]
fn historia_cow_seis_tracos_pequenos_retem_pouco() {
    let (w, ht) = (4096, 2560);
    let mut e = Engine::new(w, ht);
    for i in 0..6 {
        traco(
            &mut e,
            300.0 + 500.0 * i as f64,
            400.0 + 250.0 * i as f64,
            3,
        );
    }
    let hoje = HISTORY_DEPTH * snapshot_grid(e.active_grid()).bytes_retidos();
    let agora = e.bytes_do_historico(0);
    let px = (w * ht) as f64;
    eprintln!(
        "histórico: hoje {hoje} B ({:.1} B/px) → agora {agora} B ({:.2} B/px)",
        hoje as f64 / px,
        agora as f64 / px
    );
    assert!(
        agora * 10 <= hoje,
        "o histórico reteve {agora} B, mais de 10 % de {hoje}"
    );
}

type Tiles = std::collections::BTreeSet<(usize, usize)>;

/// Os tiles que um plano tem de volta no valor da base (0) depois de tudo ir a 1: cada tile inteiro, ou nada.
fn tiles_devolvidos(v: &[f32], s: usize, rows: usize) -> Tiles {
    let t = crate::grid::TILE;
    let mut out = Tiles::new();
    for ty in 0..rows.div_ceil(t) {
        for tx in 0..s.div_ceil(t) {
            let cel = (ty * t..((ty + 1) * t).min(rows))
                .flat_map(|y| (tx * t..((tx + 1) * t).min(s)).map(move |x| v[y * s + x]));
            let zeros = cel.clone().filter(|c| *c == 0.0).count();
            assert!(
                zeros == 0 || zeros == cel.count(),
                "tile ({tx}, {ty}) meio devolvido"
            );
            if zeros > 0 {
                out.insert((tx, ty));
            }
        }
    }
    out
}

/// `marca` num rectângulo, tudo a 1, `restaura`: que tiles (finos, de fluxo) o registo guardou.
fn guardados(x0: i64, y0: i64, x1: i64, y1: i64) -> (Tiles, Tiles) {
    let mut g = crate::grid::Grid::new(190, 190);
    let mut r = crate::grid::Registo::abre(&g);
    r.marca(&g, x0, y0, x1, y1);
    g.film.fill(1.0);
    g.vel_x.fill(1.0);
    r.restaura(&mut g, None);
    let (fs, frows) = (g.flow.s, g.flow.rows);
    (
        tiles_devolvidos(&g.film, g.s, g.rows),
        tiles_devolvidos(&g.vel_x, fs, frows),
    )
}

fn ts(v: &[(usize, usize)]) -> Tiles {
    v.iter().copied().collect()
}

/// **O registo guarda EXACTAMENTE os tiles que o rectângulo `[x0, x1) × [y0, y1)` toca** — nas fronteiras de 64
/// (fim exclusivo a 64 não toca o tile seguinte; começo a 63 toca o anterior), vazio não guarda nada, e o fluxo
/// leva a folga −2/+3 (grade de fluxo colada, `rf = 1`, 192 × 192 = 3 × 3 tiles).
#[test]
fn historia_cow_marca_guarda_exactamente_os_tiles_tocados() {
    let g = crate::grid::Grid::new(190, 190);
    assert_eq!((g.s, g.rows, g.flow.s, g.flow.rows), (192, 192, 192, 192));
    let todos = ts(&[
        (0, 0),
        (1, 0),
        (2, 0),
        (0, 1),
        (1, 1),
        (2, 1),
        (0, 2),
        (1, 2),
        (2, 2),
    ]);
    let quad = ts(&[(0, 0), (1, 0), (0, 1), (1, 1)]);
    let casos: [((i64, i64, i64, i64), Tiles, Tiles); 8] = [
        ((0, 0, 64, 64), ts(&[(0, 0)]), quad.clone()),
        ((63, 63, 65, 65), quad.clone(), quad.clone()),
        ((64, 64, 128, 128), ts(&[(1, 1)]), todos.clone()),
        ((0, 0, 65, 1), ts(&[(0, 0), (1, 0)]), ts(&[(0, 0), (1, 0)])),
        ((10, 10, 10, 40), Tiles::new(), Tiles::new()),
        ((10, 40, 30, 40), Tiles::new(), Tiles::new()),
        (
            (-5, 130, 500, 140),
            ts(&[(0, 2), (1, 2), (2, 2)]),
            ts(&[(0, 2), (1, 2), (2, 2)]),
        ),
        // Fluxo 64..128: só o tile do meio; um a mais de cada lado (63..129) já toca os nove.
        ((66, 66, 125, 125), ts(&[(1, 1)]), ts(&[(1, 1)])),
    ];
    for ((x0, y0, x1, y1), finos, fluxo) in casos {
        assert_eq!(
            guardados(x0, y0, x1, y1),
            (finos, fluxo),
            "marca({x0}, {y0}, {x1}, {y1}): (finos, fluxo)"
        );
    }
    assert_eq!(guardados(65, 65, 126, 126), (ts(&[(1, 1)]), todos));
}

/// **O refazer devolve também os tiles do `extra`** (escritos desde o último evento e que o registo não guarda),
/// nos planos finos E nos de fluxo; e o fluxo TRANSITÓRIO (`flow_x`/`flow_y`) volta a zero, como no
/// `restore_grid`.
#[test]
fn historia_cow_restaura_usa_o_extra_e_zera_o_fluxo_transitorio() {
    let mut g = crate::grid::Grid::new(190, 190);
    let mut reg = crate::grid::Registo::abre(&g);
    reg.marca(&g, 0, 0, 10, 10);
    let mut desde = crate::grid::Registo::abre(&g);
    desde.marca(&g, 140, 140, 150, 150);
    let i = 145 * g.s + 145;
    g.film[i] = 5.0;
    g.vel_x[i] = 5.0;
    g.vel_y[i] = 6.0;
    g.flow_x.fill(3.0);
    g.flow_y.fill(4.0);
    reg.restaura(&mut g, Some(&desde));
    assert_eq!(
        (g.film[i], g.vel_x[i], g.vel_y[i]),
        (0.0, 0.0, 0.0),
        "o tile do `extra` não voltou"
    );
    assert!(
        g.flow_x.iter().chain(&g.flow_y).all(|v| *v == 0.0),
        "o fluxo transitório ficou do estado desfeito"
    );
}
