//! doc 121 §9.19 (5) — os pares dos impulsos pela GRELHA dão os bits de todos-os-pares.

use super::super::{Colisor, Deslize, Material, Pecas, Saida, separate};
use super::{Leis, Movimento, TODOS_OS_PARES, impulsos};

/// Um número em `[0, 1)` a partir de `(i, faixa)` — determinístico, sem estado.
fn acaso(i: u32, faixa: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ faixa.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    #[expect(clippy::cast_precision_loss, reason = "24 bits cabem num f32")]
    let v = (h >> 8) as f32 / 16_777_216.0;
    v
}

/// Uma pilha apertada de caixas rodadas e discos, com uma peça GRANDE (a que a grelha põe à parte) e um
/// obstáculo de peso zero.
fn pilha(n: u32) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    let mut p = Vec::new();
    let mut c = Vec::new();
    let mut w = Vec::new();
    for i in 0..n {
        #[expect(clippy::cast_precision_loss, reason = "coordenadas de fixture")]
        let (gx, gy) = ((i % 24) as f32, (i / 24) as f32);
        p.push([gx * 0.2 + acaso(i, 1) * 0.05, gy * 0.2 + acaso(i, 2) * 0.05]);
        let a = acaso(i, 3) * std::f32::consts::TAU;
        c.push(Some(if i % 3 == 0 {
            Colisor::disco(0.11 + acaso(i, 4) * 0.03)
        } else if i == 7 {
            Colisor::caixa([0.9, 0.4], [a.cos(), a.sin()])
        } else {
            Colisor::caixa([0.1, 0.07 + acaso(i, 5) * 0.04], [a.cos(), a.sin()])
        }));
        w.push(if i == 11 { 0.0 } else { 1.0 });
    }
    (p, c, w)
}

/// Uma corrida do `sim.step`: separar e depois os impulsos (as leis em vigor), com material e deslize.
fn corre(todos: bool) -> (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<f32>, Vec<f32>) {
    let (mut p, c, w) = pilha(300);
    let n = p.len();
    let inv: Vec<f32> = c
        .iter()
        .zip(&w)
        .map(|(c, w)| c.map_or(0.0, |c| c.inv_inercia(*w)))
        .collect();
    let material = vec![
        Material {
            atrito: 0.6,
            salto: 0.3,
            rolar: 0.2,
        };
        n
    ];
    let antes: Vec<[f32; 2]> = p.iter().map(|q| [q[0] - 0.01, q[1] + 0.02]).collect();
    let girou: Vec<f32> = (0..n)
        .map(|i| acaso(u32::try_from(i).unwrap_or(0), 6) * 4.0 - 2.0)
        .collect();
    let pecas = Pecas {
        colisores: &c,
        pesos: &w,
        inv_inercia: &inv,
        deslize: Some(Deslize {
            antes: &antes,
            girou_antes: &girou,
            material: &material,
        }),
    };
    let mut giro = vec![0.0; n];
    let antes_do_passo = p.clone();
    separate(&mut p, &mut Saida { giro: &mut giro }, &pecas, 8);
    // Todas a convergir para o meio da pilha: os contactos fecham, e os impulsos têm o que travar.
    let vel0: Vec<[f32; 2]> = p
        .iter()
        .map(|q| [(2.3 - q[0]) * 3.0, (1.3 - q[1]) * 3.0])
        .collect();
    let mut vel = vel0.clone();
    let mut spin = vec![0.0; n];
    TODOS_OS_PARES.with(|t| t.set(todos));
    impulsos(
        &antes_do_passo,
        &mut Movimento {
            vel: &mut vel,
            giro: &mut giro,
            spin: &mut spin,
        },
        &pecas,
        |_| 1.0 / 60.0,
        Leis::EM_VIGOR,
    );
    TODOS_OS_PARES.with(|t| t.set(false));
    (vel0, vel, giro, spin)
}

#[test]
fn os_impulsos_pela_grelha_dao_os_bits_de_todos_os_pares() {
    let (_, vg, gg, sg) = corre(false);
    let (v0, vt, gt, st) = corre(true);
    // Os controlos: o contacto respondeu (velocidades e spins mudaram), senão a igualdade era de zeros.
    let mexeram = (0..v0.len()).filter(|&i| v0[i] != vt[i]).count();
    assert!(
        mexeram > 100,
        "os impulsos tinham de travar o aperto: so' {mexeram} pecas"
    );
    assert!(
        st.iter().filter(|s| s.abs() > 1e-3).count() > 20,
        "com a rotacao destravada os contactos tinham de dar spin"
    );
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    let pares = |v: &[[f32; 2]]| v.iter().flatten().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(
        pares(&vg),
        pares(&vt),
        "a velocidade difere de todos-os-pares"
    );
    assert_eq!(bits(&gg), bits(&gt), "o giro difere de todos-os-pares");
    assert_eq!(bits(&sg), bits(&st), "o spin difere de todos-os-pares");
}
