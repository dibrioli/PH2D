//! ⭐ **O Wet do Digital é o ARO DA AGUADA** (bug #58, 7.ª volta — o dono 24/10: *«talvez seja melhor que o digital
//! fique coerente com watercolor»*). A mesma lei do aro do Watercolor ([`super::watercolor_rim`]), a segunda porta:
//!
//! ```text
//!   cw    = endurece(COV_HALF · v/τ), 1 a mais de r  // a silhueta endurecida (meia-altura no contorno τ do traço)
//!   inner = min(box_blur(cw, r), P(sd, r))           // o borrão do aro e o teto do flanco recto (doc 36)
//!   edge  = densidade_do_aro(Wet · 8, cw, inner)     // o unsharp, preso a 1 — o Edge Darkening 0..8
//!   ΔD    = max(edge, 0) / CORPO_DA_AGUADA · D_L     // em unidades do corpo (o Body de fábrica) × D de UMA passagem
//! ```
//!
//! `r = raio_do_aro(Edge width, raio)` (o Spread), `sd` a distância exacta ao contorno ([`super::traco_orla`]). A
//! densidade SOMA-SE à do texel, `1 − (1 − v)·e^(−ΔD)` ([`ph2d_painter_brush::traco::orla_da_aguada`]) — a aguada
//! soma `edge` a `cw · fill` —: o miolo é o traço seco ao bit (a aguada também não mexe no corpo com o aro), a
//! silhueta não mexe. `D_L = −ln(1 − L)`, `L = 2τ` o nível de uma passagem.
//!
//! ⚠️ **`cw = 1` a mais de `r` px dentro do contorno**: a cobertura da aguada é uma UNIÃO lisa (sem grão, sem vales
//! entre carimbos) e só tem estrutura na silhueta; o plano do Digital acumula (uma SOMA com vales e grão) e o borrão
//! do aro desenharia linhas dentro do traço (medido sem isto: 8 857 texels do fundo do Glaze Round com aro, |Δ| 81 —
//! o F2 da 2.ª volta).
//!
//! ⚠️ **O aro entra no 1.º píxel dentro do contorno** (`× min(sd, 1)`): na aguada o passe à parte (`aa_coverage`)
//! suaviza a silhueta; aqui o aro saltava de 0 a ~40 % de um texel ao seguinte — escada nas curvas.
//!
//! ⚠️ **Sem o lobo pálido** (`k ≥ 0`): na aguada a franja negativa cai FORA da silhueta e o antialias dela é um passe
//! à parte (`aa_coverage`); no Digital o antialias É a franja — comê-la era a serrilha da 2.ª volta.

use super::Region;
use super::watercolor_rim::{COV_HALF, densidade_do_aro, endurece, raio_do_aro, straight_edge_cap};
use crate::tool::PainterTool;

impl PainterTool {
    /// A máscara visível `visivel` (por índice RGBA da tela) na janela `r` alargada `folga` px.
    pub(super) fn janela_visivel(
        &self,
        r: Region,
        folga: u32,
        visivel: impl Fn(usize) -> f32,
    ) -> (Region, Vec<f32>) {
        let w = self.source_size.0 as usize;
        let jan = self.alarga(r, folga);
        let (jw, jh) = (jan.w as usize, jan.h as usize);
        let mut v = vec![0.0f32; jw * jh];
        for y in 0..jh {
            for x in 0..jw {
                v[y * jw + x] = visivel(((jan.y as usize + y) * w + jan.x as usize + x) * 4);
            }
        }
        (jan, v)
    }

    /// ⭐ **O aro da aguada em `r`**: `ΔD ≥ 0` por texel (linha a linha), para a molhada `m` e o contorno `tau`.
    pub(super) fn aro_da_aguada(
        &self,
        r: Region,
        tau: f32,
        m: f32,
        visivel: impl Fn(usize) -> f32,
    ) -> Vec<f32> {
        let n = (r.w * r.h) as usize;
        if tau <= 0.0 || m <= 0.0 {
            return vec![0.0; n];
        }
        let t = self.paint.brush.traco;
        let raio = raio_do_aro(t.largura_px().round() as usize, self.raio_do_pincel());
        let alcance = raio as f32 + 1.0;
        // A folga: o borrão lê `cw` até `raio` px à volta de `r`, e `cw` lê a distância exacta até `alcance`.
        let (jan, v) = self.janela_visivel(
            r,
            super::traco_orla::alcance_da_janela(alcance + raio as f32),
            visivel,
        );
        let sd = super::traco_orla::distancia_assinada(&v, jan, jan, tau, alcance);
        let cheio = raio as f32;
        let cw: Vec<f32> = v
            .iter()
            .zip(&sd)
            .map(|(&x, &d)| {
                if d >= cheio {
                    1.0
                } else {
                    endurece(COV_HALF * x / tau)
                }
            })
            .collect();
        let inner = super::watercolor_field::box_blur(&cw, jan.w as usize, jan.h as usize, raio);
        let ganho = m.clamp(0.0, 1.0) * super::watercolor_settings::GANHO_DA_ORLA_MAX;
        let unidade = ph2d_painter_brush::traco::densidade_de_uma_passagem(2.0 * tau)
            / super::state_default::CORPO_DA_AGUADA;
        let jw = jan.w as usize;
        (0..n)
            .map(|j| {
                let (x, y) = (
                    r.x as usize + j % r.w as usize,
                    r.y as usize + j / r.w as usize,
                );
                let i = (y - jan.y as usize) * jw + (x - jan.x as usize);
                if v[i] <= 0.0 {
                    return 0.0;
                }
                // O antialias do aro: ele ENTRA no 1.º píxel dentro do contorno (`× min(sd, 1)`) — sem isto saltava
                // de 0 a ~40 % de um texel ao seguinte (o `cw` do Digital é convexo no contorno: o borrão fica abaixo
                // de ½ ali), escada nas curvas e pontilhado onde o contorno ondula entre os carimbos.
                let d = sd[i];
                if d <= 0.0 {
                    return 0.0;
                }
                let dentro = inner[i].min(1.0).min(straight_edge_cap(d, raio));
                densidade_do_aro(ganho, cw[i], dentro).max(0.0) * unidade * d.min(1.0)
            })
            .collect()
    }
}
