//! ⭐ (W14) **O ORÁCULO CORRIDO do corpo LARGO que anda** (plano 30 §22.5): a fixtura
//! `tests/fixtures/godot/largo.txt` é a saída de `docs/Components/ferramentas/godot_nav_oraculo/largo.gd`
//! (Godot 4.7.2, MIT, malha fechada, uma linha de execução; duas corridas iguais byte a byte).
//!
//! A pergunta é de DESFECHO, não de passo: o Godot trata um obstáculo de vértices como PARADO em cada
//! quadro (a velocidade dele não entra), e aqui ele leva a velocidade — os passos diferem por desenho.
//! O que se compara é quem CHEGA: com a barreira em POLÍGONO o agente desliza e contorna; com a fileira
//! de DISCOS fica preso, nos dois. ⚠️ Divergência DECLARADA: no eixo exacto o Godot prende-se no
//! empate simétrico (não tem peso de lado) e aqui o [`ph2d_orca::SIDE_BIAS`] tira-o de lá (W5, Q7).

use ph2d_orca::{Agent, Crowd, Movel, Params, V2, Walls};

const FIXTURA: &str = include_str!("../fixtures/godot/largo.txt");
const VMAX: f64 = 100.0;
const R: f64 = 10.0;
const VB: f64 = -10.0;
const ALVO: V2 = [550.0, 150.0];

/// O desfecho do Godot para a cena `nome`.
fn godot(nome: &str) -> bool {
    FIXTURA
        .lines()
        .find_map(|l| {
            let mut t = l.split_whitespace();
            (t.next() == Some("FIM") && t.next() == Some(nome))
                .then(|| l.trim_end().ends_with("chegou true"))
        })
        .unwrap_or_else(|| panic!("a cena {nome} não está na fixtura"))
}

/// A cena do `largo.gd`, aqui: chega a `< 2 px` do alvo em 900 quadros? (o quadro em que chegou)
fn corre(poligono: bool, y0: f64) -> Option<u32> {
    let (mut pos, mut vel) = ([50.0, y0], [0.0, 0.0]);
    for q in 1..=900u32 {
        let bx = 300.0 + VB * f64::from(q) / 60.0;
        let d = [ALVO[0] - pos[0], ALVO[1] - pos[1]];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if l < 2.0 {
            return Some(q);
        }
        let pref = if l > VMAX / 60.0 {
            [d[0] / l * VMAX, d[1] / l * VMAX]
        } else {
            [0.0, 0.0]
        };
        let mut corpos = vec![Agent {
            pos,
            vel,
            pref,
            radius: R,
            max_speed: VMAX,
            avoids: true,
            ignores: None,
        }];
        let mut moveis = Vec::new();
        if poligono {
            let caixa = [[-5.0, -50.0], [5.0, -50.0], [5.0, 50.0], [-5.0, 50.0]];
            moveis.push(Movel {
                walls: Walls::from_polygons(&[caixa.map(|[x, y]| [bx + x, 150.0 + y]).to_vec()]),
                vel: [VB, 0.0],
                centro: [bx, 150.0],
                omega: 0.0,
            });
        } else {
            corpos.extend((0..10).map(|k| Agent {
                pos: [bx, 150.0 - 45.0 + 10.0 * f64::from(k)],
                vel: [VB, 0.0],
                pref: [VB, 0.0],
                radius: 7.1,
                max_speed: -VB,
                avoids: false,
                ignores: None,
            }));
        }
        let mut c = Crowd::new(corpos, Params::PRODUCT).with_moving(moveis);
        let v = c.solve_all(|_| None, 1.0 / 60.0)[0];
        vel = v;
        pos = [pos[0] + v[0] / 60.0, pos[1] + v[1] / 60.0];
    }
    None
}

#[test]
fn o_corpo_largo_em_poligono_contorna_e_em_discos_prende_como_no_godot() {
    let (pol_assim, pol, discos, discos_assim) = (
        corre(true, 153.0),
        corre(true, 150.0),
        corre(false, 150.0),
        corre(false, 153.0),
    );
    eprintln!(
        "polígono fora do eixo {pol_assim:?} (Godot 458) · no eixo {pol:?} · discos {discos:?} · discos fora do eixo {discos_assim:?}"
    );
    assert_eq!(
        pol_assim.is_some(),
        godot("poligono_assimetrico"),
        "o polígono fora do eixo"
    );
    assert_eq!(discos.is_some(), godot("discos"), "a fileira de discos");
    assert_eq!(
        discos_assim.is_some(),
        godot("discos_assimetrico"),
        "a fileira de discos fora do eixo"
    );
    // A divergência declarada: no eixo o peso de lado desfaz o empate que prende o Godot.
    assert!(!godot("poligono") && pol.is_some(), "no eixo: {pol:?}");
}
