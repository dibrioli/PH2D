//! Gate das lanes do transporte ([`super::lanes`]): a rota em lanes e a escalar dão os mesmos bits, linha a
//! linha, em grelhas aleatórias com bordas, valores zero / denormais / negativos, saídas acima de 1,
//! retro-traços que saem da folha, células inativas e troços de largura não múltipla de 8.
use super::*;
use crate::painter::Engine;

struct Rng(u64);
impl Rng {
    fn u(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f(&mut self) -> f32 {
        (self.u() >> 40) as f32 / (1u64 << 24) as f32
    }
    /// Um valor de massa: zero, denormal, negativo pequeno ou positivo.
    fn massa(&mut self) -> f32 {
        match self.u() % 8 {
            0 | 1 => 0.0,
            2 => f32::from_bits(1 + (self.u() % 0x7f_ffff) as u32),
            3 => -self.f() * 0.01,
            _ => self.f() * 2.0,
        }
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn o_transporte_em_lanes_da_o_escalar_ao_bit() {
    const W: usize = 77;
    const H: usize = 41;
    let mut e = Engine::with_flow_ratio(W, H, 1);
    let p = e.sim.gather_params(&e.tuning);
    assert!(
        !p.km_mixing,
        "o gate mede a rota sem K–M (a única que tem lanes)"
    );
    let mut celulas = 0usize;
    for semente in 1..=24u64 {
        let mut r = Rng(0x9e37_79b9_7f4a_7c15 ^ semente.wrapping_mul(0x2545_f491_4f6c_dd1d));
        let g = e.active_grid_mut();
        let n = g.cells;
        g.scratch.ensure(n);
        for i in 0..n {
            g.film[i] = r.massa();
            g.susp[i] = r.massa();
            g.susp_rgb[i] = [r.f() * 255.0, r.massa(), r.f() * 255.0];
            g.active[i] = u8::from(!r.u().is_multiple_of(5));
            g.scratch.uv[i] = [(r.f() - 0.5) * 9.0, (r.f() - 0.5) * 9.0];
            g.scratch.outflow[i] = match r.u() % 6 {
                0 => 0.0,
                1 => 1.0 + r.f() * 3.0,
                2 => f32::from_bits(1 + (r.u() % 0x7f_ffff) as u32),
                _ => r.f(),
            };
        }
        // Blocos inteiros de lanes ativas (onde a recolha corre), e blocos inteiros parados.
        let faixa = (r.u() % (W as u64 / 2)) as usize * g.s;
        for i in faixa..(faixa + 3 * g.s).min(n) {
            g.active[i] = if semente % 3 == 0 { 0 } else { 1 };
        }
        g.spans_enabled = false;
        g.tiles_enabled = false;
        g.bx0 = 1 + (r.u() % 5) as i32;
        g.bx1 = W as i32 - (r.u() % 5) as i32;
        g.by0 = 1;
        g.by1 = H as i32;
        let s = g.s;
        let (c, scratch) = split(g, &p);
        let (uv, outflow): (&[[f32; 2]], &[f32]) = (&scratch.uv, &scratch.outflow);
        for y in c.by0..=c.by1 {
            let mut a = (vec![7.0f32; s], vec![7.0f32; s], vec![[7.0f32; 3]; s]);
            let mut b = a.clone();
            transport_row_escalar(&c, y, &mut a.0, &mut a.1, &mut a.2, outflow, 0, uv, 0);
            lanes::transport_row_lanes(&c, y, &mut b.0, &mut b.1, &mut b.2, outflow, 0, uv, 0);
            assert_eq!(bits(&a.0), bits(&b.0), "film, semente {semente}, linha {y}");
            assert_eq!(bits(&a.1), bits(&b.1), "susp, semente {semente}, linha {y}");
            let (ca, cb): (Vec<f32>, Vec<f32>) = (a.2.concat(), b.2.concat());
            assert_eq!(bits(&ca), bits(&cb), "cor, semente {semente}, linha {y}");
            celulas += (c.bx1 - c.bx0 + 1) as usize;
        }
    }
    assert!(celulas > 24 * 40 * 60, "o gate varreu {celulas} células");
}
