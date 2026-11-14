//! B13.4 — a cadência da secagem é do TILE (64×64), não da grade inteira: um pingo a correr longe não muda o
//! ritmo de uma poça calma. Defeito e desenho: `docs/Painter/missao_4k/medicoes/N10_agua_wet.md` (B13.4).

use crate::util::drive_stroke;
use ph2d_wet_paint::painter::Engine;

const W: usize = 384;
const H: usize = 192;

/// Duas poças finas em tiles longe (`x < 64` e `x ≥ 320`), ainda molhadas; depois a gravidade forte
/// (`[0, 1]`, nas DUAS sessões: as poças finas ficam calmas nela) e, com `pingo`, um traço de Water 1 no
/// meio que corre ladeira abaixo. Devolve o estado das duas poças, a água viva nelas antes do pingo e os
/// passos «a correr».
fn sessao(pingo: bool) -> (Vec<u32>, usize, usize) {
    let mut e = Engine::new(W, H);
    e.color = [40.0, 60.0, 190.0];
    e.sliders.water = 1.0;
    drive_stroke(&mut e, 20.0, 40.0, 44.0, 40.0, 2.0, 0);
    drive_stroke(&mut e, 330.0, 150.0, 356.0, 150.0, 2.0, 0);
    for _ in 0..4 {
        e.step_simulation();
    }
    let poca = |x: usize| x < 64 || x >= 320;
    let molhadas = {
        let g = e.active_grid();
        (0..g.film.len())
            .filter(|&i| poca(i % g.s) && g.film[i] > 0.0)
            .count()
    };
    e.sim.gravity_override = Some([0.0, 1.0]);
    if pingo {
        drive_stroke(&mut e, 170.0, 60.0, 214.0, 60.0, 1.0, 0);
    }
    let mut correndo = 0;
    for _ in 0..160 {
        e.step_simulation();
        correndo += usize::from(e.sim.dry_every == 3);
    }
    let g = e.active_grid();
    let mut v = Vec::new();
    for y in 0..H {
        for x in (0..W).filter(|&x| poca(x)) {
            let i = x + y * g.s;
            v.extend([g.film[i], g.susp[i], g.sett[i]].map(f32::to_bits));
        }
    }
    (v, molhadas, correndo)
}

#[test]
fn duas_pocas_isoladas_secam_igual_com_e_sem_um_pingo() {
    let (sem, m0, c0) = sessao(false);
    let (com, _, c1) = sessao(true);
    let dif = sem.iter().zip(&com).filter(|(a, b)| a != b).count();
    println!(
        "passos a correr: sem pingo {c0}, com pingo {c1}; células molhadas nas poças {m0}; valores diferentes {dif}"
    );
    assert!(
        c1 > c0,
        "o pingo não chegou a correr ({c0} × {c1}): a cena não mede nada"
    );
    assert!(m0 > 0, "as poças já estavam secas: a cena não mede nada");
    assert_eq!(
        dif, 0,
        "o pingo longe mudou a secagem das poças ({dif} valores)"
    );
}
