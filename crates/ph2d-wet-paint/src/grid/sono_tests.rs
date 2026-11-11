//! B13.3: com ε = 0 o sono é ao bit contra o caminho sem sono (`ligado = false`), e dorme de facto.
use crate::painter::Engine;

fn traco(e: &mut Engine, x: f64, y0: f64, y1: f64) {
    e.pointer_down(x, y0, None);
    let mut y = y0;
    while y < y1 {
        y = (y + 4.0).min(y1);
        e.pointer_frame(x + (y - y0) * 0.2, y);
        if e.sim_should_run() {
            e.step_simulation();
        }
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Duas manchas longe nas mesmas linhas, uma 3.ª pintada a meio (acorda por pincel), 400 passos.
fn sessao(rf: usize, jacobi: bool, ligado: bool, eps: f32) -> (Vec<u64>, u64) {
    let mut e = Engine::with_flow_ratio(900, 330, rf);
    e.sim.order_invariant = jacobi;
    e.sliders.water = 1.0;
    e.sim.gravity_override = Some([0.0, 0.0]);
    e.active_grid_mut().sono.ligado = ligado;
    e.active_grid_mut().sono.eps = eps;
    traco(&mut e, 40.0, 30.0, 130.0);
    traco(&mut e, 620.0, 30.0, 230.0);
    let mut d = Vec::new();
    for k in 0..400 {
        if k == 200 {
            traco(&mut e, 330.0, 120.0, 200.0);
        }
        e.step_simulation();
        let g = e.active_grid();
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut f = |b: u32| h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
        for a in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
            a.iter().for_each(|v| f(v.to_bits()));
        }
        g.susp_rgb.iter().flatten().for_each(|v| f(v.to_bits()));
        g.wet.iter().for_each(|&v| f(u32::from(v)));
        d.push(h);
    }
    let g = e.active_grid();
    (d, g.sono.saltados)
}

#[test]
fn o_sono_com_eps_zero_da_o_caminho_sem_sono_ao_bit() {
    for rf in [1, 2] {
        for jacobi in [false, true] {
            let (com, _) = sessao(rf, jacobi, true, 0.0);
            let (sem, _) = sessao(rf, jacobi, false, 0.0);
            let dif = com.iter().zip(&sem).filter(|(a, b)| a != b).count();
            assert_eq!(dif, 0, "rf {rf} · jacobi {jacobi}: {dif} passos diferem");
        }
    }
}

/// A ilha 3×3: só o tile cujo bairro inteiro está quieto sai da máscara, e o fim do passo repõe-na.
#[test]
fn so_a_ilha_quieta_sai_da_mascara_e_volta_no_fim_do_passo() {
    let mut e = Engine::with_flow_ratio(700, 260, 1);
    let g = e.active_grid_mut();
    g.row_tiles.fill(u64::MAX);
    g.sono.ligado = true;
    super::inicio(g, 1);
    let ntx = g.sono.ntx;
    for ty in 1..4 {
        for tx in 2..5 {
            g.sono.quietos[ty * ntx + tx] = super::PASSOS_PARA_DORMIR;
        }
    }
    let antes = g.row_tiles.clone();
    super::adormece(g);
    let tw = g.tile_words;
    for y in 0..g.rows {
        let w = g.row_tiles[y * tw];
        let dorme = (128..192).contains(&y);
        assert_eq!(w & (1 << 3) == 0, dorme, "linha {y}: tile 3");
        assert_eq!(w | (1 << 3), u64::MAX, "linha {y}: outro tile saiu");
    }
    super::conta(g);
    assert!(g.row_tiles == antes, "o fim do passo repõe a máscara");
    g.sono.quietos[2 * ntx + 4] = 0;
    super::adormece(g);
    assert!(
        g.row_tiles == antes,
        "um vizinho acordado mantém o tile acordado"
    );
    super::conta(g);
    g.note_live(200, 140, 210, 150);
    assert_eq!(g.sono.quietos[2 * ntx + 3], 0, "o pincel acorda");
}
