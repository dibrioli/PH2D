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
        // ⚠️ Pela porta única — o alcance da vizinhança sai de UM lugar (ver o doc dela).
        let reach = self.spec.sketchy_reach_px();
        let density = self.spec.sketchy_density.clamp(0.0, 1.0);
        // **MAGNETIFY — ele escolhe QUE PARES viram fio, nunca com que força um fio desenha.**
        //
        // ⚠️ A lei é a do manual do Krita, verbatim: *"It's what causes curve lines to form between
        // two close line sections … With Magnetify off, the curve line just forms on either side of
        // the current active portion of your connection line."* Ligado, todo par dentro do alcance
        // costura — inclusive dois trechos que o traço aproximou depois de dar uma volta inteira.
        // Desligado, só a **porção ativa** do traço: os pontos que o dedo acabou de deixar.
        //
        // ⚠️ **E a régua de "porção ativa" é o ARCO, não a contagem** — a distinção já estava escrita
        // logo abaixo, como o motivo de a varredura não poder cortar por arco: *o arco entre dois
        // pontos limita por baixo a distância entre eles, então passado `reach` de arco nenhum ponto
        // mais antigo está dentro do raio EXCETO quando o traço volta sobre si mesmo*. Voltar sobre
        // si mesmo é exatamente o que o Magnetify liga e desliga.
        let magnetify = self.spec.sketchy_magnetify;
        if reach > 0.0 && density > 0.0 {
            let r2 = reach * reach;
            // ⚠️ **A varredura tem TETO de CONSULTAS, não de arco** — é ele que mantém o custo linear
            // no traço em vez de quadrático, e o número é MEDIDO ([`SKETCHY_SCAN_CAP`]). Cortar por
            // arco aqui seria escolher o Magnetify OFF para todo mundo.
            let n = self.sketchy_pts.len();
            let from = n.saturating_sub(crate::line_kind::SKETCHY_SCAN_CAP);
            for i in from..n {
                let q = self.sketchy_pts[i];
                // Sem Magnetify o candidato tem de estar na porção ATIVA do percurso. O corte é
                // estritamente mais apertado que o do raio (arco ≥ corda), então o teste de distância
                // abaixo segue governando o alcance nos dois modos.
                if !magnetify && (dab.arc_len - q[2]) > reach {
                    continue;
                }
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
        self.sketchy_pts.push([p[0], p[1], dab.arc_len]);
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
