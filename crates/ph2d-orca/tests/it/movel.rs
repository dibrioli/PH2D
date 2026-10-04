//! ⭐ (W14) **Um corpo que ANDA é o mesmo corpo PARADO noutro referencial** (plano 30 §22.5) — as linhas
//! de um [`Movel`] calculam-se com a velocidade RELATIVA e deslocam-se pela dele. ⇒ resolver o agente contra
//! o polígono a andar com `u` dá o mesmo que resolvê-lo no referencial do polígono (parado, o agente com
//! `v − u` e `pref − u`) e somar `u`. Para a igualdade ser exacta: o disco da velocidade máxima não morde
//! (máxima grande), o peso de lado está a zero (ele lê a DIRECÇÃO do pedido), e a folga do que o corpo
//! anda (`|u|·τ`) entra no referencial parado como raio a mais do agente.

use ph2d_orca::{Agent, Crowd, Movel, Params, V2, Walls};

struct Lcg(u64);
impl Lcg {
    fn f(&mut self, a: f64, b: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((self.0 >> 11) as f64 / (1u64 << 53) as f64)
    }
}

fn resolve(pos: V2, vel: V2, pref: V2, raio: f64, poli: &[V2], u: V2) -> V2 {
    let p = Params {
        side_bias: 0.0,
        ..Params::PRODUCT
    };
    let agente = Agent {
        pos,
        vel,
        pref,
        radius: raio,
        max_speed: 1.0e3,
        avoids: true,
        ignores: None,
    };
    let m = Movel {
        walls: Walls::from_polygons(&[poli.to_vec()]),
        vel: u,
        centro: [0.0, 0.0],
        omega: 0.0,
    };
    Crowd::new(vec![agente], p)
        .with_moving(vec![m])
        .solve_all(|_| None, 1.0 / 60.0)[0]
}

#[test]
fn um_corpo_que_anda_e_o_mesmo_parado_no_referencial_dele() {
    let mut r = Lcg(0x6a11_1e00);
    let poli: Vec<V2> = vec![[-0.2, -1.5], [0.2, -1.5], [0.2, 1.5], [-0.2, 1.5]];
    let (mut casos, mut cortados) = (0, 0);
    while casos < 2_000 {
        let pos = [r.f(-4.0, 4.0), r.f(-4.0, 4.0)];
        if pos[0].abs() < 1.0 && pos[1].abs() < 2.2 {
            continue; // dentro ou colado: fora da pergunta
        }
        let (vel, pref, u) = (
            [r.f(-3.0, 3.0), r.f(-3.0, 3.0)],
            [r.f(-3.0, 3.0), r.f(-3.0, 3.0)],
            [r.f(-2.0, 2.0), r.f(-2.0, 2.0)],
        );
        let raio = 0.3;
        let folga = (u[0] * u[0] + u[1] * u[1]).sqrt() * Params::PRODUCT.time_horizon_walls;
        let a = resolve(pos, vel, pref, raio, &poli, u);
        let b = resolve(
            pos,
            [vel[0] - u[0], vel[1] - u[1]],
            [pref[0] - u[0], pref[1] - u[1]],
            raio + folga,
            &poli,
            [0.0, 0.0],
        );
        let (x, y) = (b[0] + u[0], b[1] + u[1]);
        assert!(
            (a[0] - x).abs() <= 1e-9 && (a[1] - y).abs() <= 1e-9,
            "caso {casos}: {pos:?} v {vel:?} pref {pref:?} u {u:?}: a andar {a:?}, parado + u [{x}, {y}]"
        );
        casos += 1;
        cortados += usize::from((a[0] - pref[0]).abs() + (a[1] - pref[1]).abs() > 1e-9);
    }
    // A população (medida: `785` de `2 000` cortados pelo polígono — a lei é posta à prova).
    assert!(cortados > 700, "{cortados} de {casos} cortados");
}
