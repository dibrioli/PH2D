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
    h(&mut st, &s.traco);
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
