//! **A força do pincel num passo** (smoke do dono, 19/10): por texel, quanto o pincel do passo o cobriu,
//! com os dabs compostos como o carimbo os compõe. Onde ela chega a [`MIOLO`] o texel é do artista mesmo
//! que o byte não mude (preto sobre o preto que a gravação deixou no fim). Só os ladrilhos tocados vivem.

use super::marca;
use std::collections::BTreeMap;

/// A força a partir da qual o texel é do artista. Uma marca é binária: num texel que o passo não mudou e
/// que o pincel cobriu com força `a`, mostrar o fim erra pelo que estava por baixo (`1 − a`) e mostrar a
/// gravação erra pela tinta do artista (`a`) — os dois erros cruzam-se em ½.
pub const MIOLO: f32 = 0.5;

const LADO: u32 = 64;

/// A transmitância (`1 − força`) de cada texel tocado, em ladrilhos de [`LADO`]².
#[derive(Clone, Debug, Default)]
pub(crate) struct ForcaDoPincel {
    ladrilhos: BTreeMap<(u32, u32), Box<[f32]>>,
}

impl ForcaDoPincel {
    pub(crate) fn limpa(&mut self) {
        self.ladrilhos.clear();
    }

    /// O pincel deposita `a(d)` (`d` = distância ao centro, px; `0..=1`) no disco `(cx, cy, r)` de uma tela
    /// `w`×`h`. `uniao`: os dabs compõem-se (`1 − Π(1 − aᵢ)`, Accumulate ligado); senão o traço fica na
    /// maior (o tecto por traço).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "a caixa do disco cortada à tela"
    )]
    pub(crate) fn deposita(
        &mut self,
        (w, h): (u32, u32),
        (cx, cy, r): (f32, f32, f32),
        uniao: bool,
        a: impl Fn(f32) -> f32,
    ) {
        if !(r > 0.0 && cx.is_finite() && cy.is_finite()) {
            return;
        }
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil().max(0.0) as u32).min(w);
        let y1 = ((cy + r).ceil().max(0.0) as u32).min(h);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let r2 = r * r;
        // Um ladrilho por vez: a busca no mapa é por ladrilho, nunca por texel.
        for ly in y0 / LADO..=(y1 - 1) / LADO {
            for lx in x0 / LADO..=(x1 - 1) / LADO {
                let l = self
                    .ladrilhos
                    .entry((lx, ly))
                    .or_insert_with(|| vec![1.0; (LADO * LADO) as usize].into_boxed_slice());
                for y in y0.max(ly * LADO)..y1.min((ly + 1) * LADO) {
                    let dy = y as f32 + 0.5 - cy;
                    let linha = &mut l[((y % LADO) * LADO) as usize..][..LADO as usize];
                    for x in x0.max(lx * LADO)..x1.min((lx + 1) * LADO) {
                        let dx = x as f32 + 0.5 - cx;
                        let d2 = dx * dx + dy * dy;
                        if d2 >= r2 {
                            continue;
                        }
                        let f = a(d2.sqrt()).clamp(0.0, 1.0);
                        let t = &mut linha[(x % LADO) as usize];
                        *t = if uniao {
                            *t * (1.0 - f)
                        } else {
                            t.min(1.0 - f)
                        };
                    }
                }
            }
        }
    }

    /// Liga em `out` (uma tela `w`×`h`) os texels com força ≥ [`MIOLO`].
    pub(crate) fn liga_o_miolo(&self, out: &mut [u64], (w, h): (u32, u32)) {
        for (&(lx, ly), l) in &self.ladrilhos {
            for (k, &t) in l.iter().enumerate() {
                let (x, y) = (lx * LADO + k as u32 % LADO, ly * LADO + k as u32 / LADO);
                if 1.0 - t >= MIOLO && x < w && y < h {
                    marca::liga(out, (y * w + x) as usize);
                }
            }
        }
    }
}
