//! O portão do B14.1: a caixa que a óptica fora recebe é a das células que MUDARAM (com a folga de uma), não a do
//! fluido — e cobre toda célula que mudou (o espelho fica igual à grade ao bit depois da cópia).

use super::super::espelho::Espelho;
use super::super::{Dirty, Engine};

const N: usize = 160;

fn risca(e: &mut Engine, y: f64, agua: f64) {
    e.sliders.water = agua;
    let mut x = 20.0;
    e.pointer_down(x, y, None);
    while x < 140.0 {
        x += 6.0;
        e.dispatch_pressure_dab(x, y, 1.0, 12.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// As células da camada activa cuja óptica difere ao bit entre a grade e o espelho DENTRO de `regiao` (com a folga),
/// e se alguma FORA dela, que não seja do anel que `solver::apply_boundaries` reescreve a cada passo, difere.
fn mudou(e: &Engine, esp: &Espelho, regiao: Dirty) -> (Dirty, bool) {
    let (g, c) = (
        &e.layers[e.active_layer].grid,
        &esp.camadas[e.active_layer].grid,
    );
    let (mut d, mut fora) = (Dirty::Clean, false);
    for y in 1..=g.h {
        for x in 1..=g.w {
            if super::difere_em(g, c, y * g.s + x) {
                let (x, y) = (x as i32, y as i32);
                let p = Dirty::Rect {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                };
                if contem(regiao, p) {
                    d.une((x, y, x, y), e.width, e.height);
                } else {
                    fora |= !(x == 1 || y == 1 || x == g.w as i32 || y == g.h as i32);
                }
            }
        }
    }
    (d, fora)
}

/// Os planos da óptica que diferem (o diagnóstico do vermelho).
fn quais(e: &Engine, esp: &Espelho) -> Vec<&'static str> {
    let (g, c) = (
        &e.layers[e.active_layer].grid,
        &esp.camadas[e.active_layer].grid,
    );
    let bits = |a: &[f32], b: &[f32]| a.iter().zip(b).any(|(x, y)| x.to_bits() != y.to_bits());
    [
        ("film", bits(&g.film, &c.film)),
        ("susp", bits(&g.susp, &c.susp)),
        ("sett", bits(&g.sett, &c.sett)),
        ("susp_rgb", g.susp_rgb != c.susp_rgb),
        ("sett_rgb", g.sett_rgb != c.sett_rgb),
        ("relevo", bits(&g.relevo, &c.relevo)),
        ("altura", bits(&g.altura, &c.altura)),
        ("espessura", bits(&g.espessura, &c.espessura)),
        ("crista", bits(&g.crista, &c.crista)),
        ("nivelado", bits(&g.nivelado, &c.nivelado)),
        ("vela", bits(&g.vela, &c.vela)),
        ("paper", bits(&g.paper, &c.paper)),
        ("wet", g.wet != c.wet),
        ("material", g.material != c.material),
        ("aparencia", g.aparencia != c.aparencia),
        ("oleo", g.oleo != c.oleo),
        ("tinta", g.tinta != c.tinta),
        ("pigmento", g.pigmento != c.pigmento),
    ]
    .into_iter()
    .filter_map(|(n, d)| d.then_some(n))
    .collect()
}

fn contem(a: Dirty, b: Dirty) -> bool {
    match (a, b) {
        (_, Dirty::Clean) | (Dirty::Full, _) => true,
        (
            Dirty::Rect { x0, y0, x1, y1 },
            Dirty::Rect {
                x0: a0,
                y0: b0,
                x1: a1,
                y1: b1,
            },
        ) => x0 <= a0 && y0 <= b0 && x1 >= a1 && y1 >= b1,
        _ => false,
    }
}

fn area(d: Dirty) -> usize {
    match d {
        Dirty::Clean => 0,
        Dirty::Full => N * N,
        Dirty::Rect { x0, y0, x1, y1 } => ((x1 - x0 + 1) * (y1 - y0 + 1)) as usize,
    }
}

/// **B14.1: a água herdada no pen-down volta como sujo do PASSO** — a casa tira o sujo partido
/// ([`Engine::take_dirty_partido`]) e devolve-o ao worker: pela porta alheia (o caminho antigo, a `divida`) a óptica
/// compunha a caixa do fluido inteira; por [`Engine::devolve_sujo_do_passo`] o espelho redu-la ao que mudou.
#[test]
fn a_agua_herdada_volta_como_sujo_do_passo_e_o_espelho_a_reduz() {
    let mut e = Engine::new(N, N);
    let mut esp = Espelho::default();
    e.sim.oleo = true;
    risca(&mut e, 40.0, 0.2);
    e.sim.oleo = false;
    risca(&mut e, 44.0, 1.0);
    let _ = e.take_dirty();
    e.copia_para(&mut esp);
    let (mut velha, mut nova) = (0, 0);
    for k in 0..40 {
        e.step_simulation();
        let uniao = e.dirty;
        let (t, _) = mudou(&e, &esp, uniao);
        let (passo, alheio) = e.take_dirty_partido();
        assert_eq!(e.dirty, Dirty::Clean, "passo {k}: o sujo fica em casa");
        let junta = match (passo, alheio) {
            (d, Dirty::Clean) | (Dirty::Clean, d) => d,
            (
                Dirty::Rect { x0, y0, x1, y1 },
                Dirty::Rect {
                    x0: a,
                    y0: b,
                    x1: c,
                    y1: d,
                },
            ) => Dirty::Rect {
                x0: x0.min(a),
                y0: y0.min(b),
                x1: x1.max(c),
                y1: y1.max(d),
            },
            _ => Dirty::Full,
        };
        assert_eq!(junta, uniao, "passo {k}: partido, a uniao e a de sempre");
        let Dirty::Rect { x0, y0, x1, y1 } = passo else {
            continue;
        };
        e.merge_dirty_otica(x0, y0, x1, y1);
        let pela_divida = e.take_dirty_pela_otica(&esp);
        e.devolve_sujo_do_passo(x0, y0, x1, y1);
        let pelo_passo = e.take_dirty_pela_otica(&esp);
        assert!(
            contem(pela_divida, uniao),
            "passo {k}: a porta alheia nao se reduz"
        );
        assert_eq!(pelo_passo, t, "passo {k}: devolvido, e o que mudou");
        (velha, nova) = (velha + area(pela_divida), nova + area(pelo_passo));
        e.copia_para(&mut esp);
    }
    eprintln!("agua herdada: pela divida {velha} cel. -> pelo passo {nova} cel.");
    assert!(
        nova < velha,
        "a fixture nao contem o fenomeno ({nova} >= {velha})"
    );
}

/// Óleo e um traço molhado por cima, a água a correr; um traço no meio (a porta alheia). Por passo: a caixa da óptica
/// é EXACTAMENTE a das células mudadas (passo sem traço) ou contém-nas (com traço); depois da cópia, o espelho é a grade.
#[test]
fn a_optica_custa_pela_caixa_suja_e_nao_pela_folha() {
    let mut e = Engine::new(N, N);
    let mut esp = Espelho::default();
    e.sim.oleo = true;
    risca(&mut e, 40.0, 0.2);
    e.sim.oleo = false;
    risca(&mut e, 44.0, 1.0);
    let uniao = e.dirty;
    assert_eq!(e.take_dirty_pela_otica(&esp), uniao, "sem espelho, a uniao");
    e.copia_para(&mut esp);
    let (mut velha, mut nova, mut com_sobra) = (0, 0, 0);
    for k in 0..40 {
        let traco = k == 12;
        if traco {
            risca(&mut e, 110.0, 1.0);
        }
        e.step_simulation();
        let uniao = e.dirty;
        let d = e.take_dirty_pela_otica(&esp);
        let (t, fora) = mudou(&e, &esp, uniao);
        assert!(
            !fora,
            "passo {k}: mudou fora da uniao (alem do anel) ({:?})",
            quais(&e, &esp)
        );
        assert!(
            contem(d, t),
            "passo {k}: a caixa {d:?} nao cobre o que mudou {t:?} ({:?})",
            quais(&e, &esp)
        );
        assert!(
            contem(uniao, d),
            "passo {k}: a caixa {d:?} sai da uniao {uniao:?}"
        );
        if !traco {
            assert_eq!(
                d, t,
                "passo {k}: a caixa da optica nao e a do que mudou (uniao {uniao:?})"
            );
        }
        (velha, nova) = (velha + area(uniao), nova + area(d));
        com_sobra += usize::from(area(uniao) > area(d));
        e.copia_para(&mut esp);
        assert_eq!(
            mudou(&e, &esp, Dirty::Full),
            (Dirty::Clean, false),
            "passo {k}: o espelho nao e a grade"
        );
    }
    eprintln!(
        "caixa da optica: uniao {velha} cel. -> mudadas {nova} cel. ({com_sobra} passos com sobra)"
    );
    assert!(
        com_sobra > 0,
        "a fixture nao contem o fenomeno (nenhum passo sujou sem mudar)"
    );
}
