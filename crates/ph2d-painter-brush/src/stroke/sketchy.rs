//! **SKETCHY — o traço que se costura a si mesmo** (o *neighbour points* do Ze Frank → Harmony →
//! Krita *Sketch*; plano 38 W3). Guarda-se a memória do traço e, a cada dab novo, ligam-se os pontos
//! vizinhos com fios de opacidade baixa: sai o hachurado de lápis que ninguém imita à mão.
//!
//! ⚠️ **Um fio NÃO é uma corrente de dabs, e a W0.3 mediu por quê:** a densidade 1,0 põe **16 mil px
//! de fio** num traço de 312 px, e um fio fino carimbado à spacing de sub-pixel poria a conta em
//! 10⁴–10⁶ dabs. Ele é um **segmento**, publicado por um canal próprio ([`Stroke::take_threads`]) e
//! rasterizado por área exata — que é como o Harmony e o Krita o desenham.
//!
//! ⚠️ **A `Density` é o ORÇAMENTO, não um enfeite** (W0.3): a um diâmetro de alcance os fios somam
//! **~50× o arco do traço**; a quatro, **~700×**. O teto sai DERIVADO daí, não escolhido — ver
//! [`crate::line_kind::SKETCHY_DENSITY_MAX`].

use super::{Dab, Stroke};

/// Um fio: o segmento `a → b` em pixels de canvas. Sem largura nem cor — quem os carrega é o
/// [`crate::BrushSpec`], porque um fio é uma linha do MESMO traço, não um objeto próprio.
pub type Thread = [f32; 4];

impl Stroke {
    /// Registra o centro de um dab na memória do traço e costura os fios que ele fecha.
    ///
    /// ⚠️ **A vizinhança é do TRAÇO, nunca do canvas:** a memória nasce vazia em cada
    /// [`Stroke::begin`], então um traço novo jamais liga um fio ao anterior. Sem isso o Sketchy
    /// costuraria o desenho inteiro à medida que o artista trabalha.
    pub(super) fn note_sketchy_point(&mut self, dab: &Dab) {
        if !self.spec.sketchy_active() {
            return;
        }
        let p = dab.center;
        // ⚠️ Pela porta única — o depósito pesa o fio contra ESTE número (ver o doc dela).
        let reach = self.spec.sketchy_reach_px();
        let density = self.spec.sketchy_density.clamp(0.0, 1.0);
        if reach > 0.0 && density > 0.0 {
            let r2 = reach * reach;
            // ⚠️ **A varredura anda de TRÁS para a frente e PARA no primeiro ponto fora do alcance
            // pelo ARCO.** A memória é ordenada por percurso, e o arco entre dois pontos é um limite
            // inferior da distância entre eles — então, passado `reach` de arco, nenhum ponto MAIS
            // ANTIGO pode estar dentro do raio... exceto quando o traço volta sobre si mesmo, que é
            // justamente o caso em que o Sketchy tem de costurar. Daí o corte não ser o arco e sim um
            // TETO de quantos pontos a memória oferece por dab ([`SKETCHY_SCAN_CAP`]): ele é o que
            // mantém o custo linear no traço em vez de quadrático, e o número dele é MEDIDO.
            let n = self.sketchy_pts.len();
            let from = n.saturating_sub(crate::line_kind::SKETCHY_SCAN_CAP);
            for i in from..n {
                let q = self.sketchy_pts[i];
                let d2 = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2);
                if d2 > r2 {
                    continue;
                }
                // Sorteio determinístico (splitmix64 do próprio traço — HR-5, replays reproduzíveis).
                if crate::jitter::next_f32(&mut self.sketchy_rng) > density {
                    continue;
                }
                crate::symmetry::push_symmetric_segment(
                    &mut self.threads,
                    [p[0], p[1], q[0], q[1]],
                    &self.spec.symmetry,
                );
            }
        }
        self.sketchy_pts.push(p);
    }

    /// **Drena os fios costurados desde a última chamada.** Canal PRÓPRIO, e não um `Dab` a mais:
    /// um fio tem largura e opacidade suas e é rasterizado como segmento, não carimbado.
    ///
    /// ⚠️ Quem chama é o dono do canvas, no MESMO ponto em que consome os dabs — os dois saem do
    /// mesmo gesto e têm de pousar na mesma região suja.
    pub fn take_threads(&mut self, out: &mut Vec<Thread>) {
        out.clear();
        out.append(&mut self.threads);
    }
}
