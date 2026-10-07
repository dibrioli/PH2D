//! **O VIDRO da aguada** (doc 48, BUGS #46) — o que o núcleo da aguada precisa para escrever, a cada
//! quadro, a transparência POR CANAL de cada texel ([`crate::compositor::vidro`]): o plano da camada
//! activa, o vidro da BASE (congelado com a base da tela: o núcleo repõe a base fora da mancha, e a
//! base guarda o seu próprio vidro) e o chão PRETO (as camadas de baixo sobre o preto; `None` quando
//! não há nenhuma — o preto puro). O núcleo avalia a óptica sobre o chão de sempre e sobre o preto; a
//! diferença, por canal, é o que o chão atravessa.

use super::*;
use crate::compositor::vidro::Vidro;
use std::sync::Weak;

/// A base congelada e o vidro dela.
type BaseCongelada = (Weak<Vec<u8>>, Arc<Vec<Vidro>>);
/// O chão (o `Weak` dele) e o chão preto que lhe corresponde (`None` = o preto puro).
type ChaoCongelado = (Weak<Vec<u8>>, Option<Arc<Vec<u8>>>);

/// O que se congela uma vez por base: ligado a ela por `Weak` (o endereço não se reutiliza enquanto o
/// seguramos), então uma base nova congela de novo e a mesma base de uma sessão molhada reaproveita.
#[derive(Default)]
pub(crate) struct VidroCongelado {
    base: Option<BaseCongelada>,
    chao_preto: Option<ChaoCongelado>,
}

/// O vidro de um quadro do núcleo. O plano sai do mapa e volta com [`PainterTool::devolve_o_vidro`].
pub(super) struct VidroDoQuadro {
    pub camada: RtLayerId,
    pub plano: Arc<Vec<Vidro>>,
    pub base: Arc<Vec<Vidro>>,
    pub chao_preto: Option<Arc<Vec<u8>>>,
}

impl PainterTool {
    /// O vidro deste quadro, ou `None` (a pintar uma máscara, ou sem tela).
    pub(super) fn vidro_do_quadro(
        &mut self,
        base: &Arc<Vec<u8>>,
        backdrop: &Arc<Vec<u8>>,
    ) -> Option<VidroDoQuadro> {
        #[cfg(test)]
        if self.sem_vidro {
            return None;
        }
        let camada = self
            .layers
            .active()
            .filter(|&id| !self.layers.is_mask(id))?;
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 || base.len() != n * 4 {
            return None;
        }
        let mut plano = self
            .vidros
            .remove(&camada)
            .filter(|p| p.len() == n)
            .unwrap_or_else(|| Arc::new(vec![[0u8; 7]; n]));
        let fresca = |w: &Weak<Vec<u8>>, a: &Arc<Vec<u8>>| Weak::ptr_eq(w, &Arc::downgrade(a));
        let vbase = match &self.vidro_congelado.base {
            Some((w, v)) if fresca(w, base) && v.len() == n => Arc::clone(v),
            _ => {
                self.vidro_congelado.base = Some((Arc::downgrade(base), Arc::clone(&plano)));
                Arc::clone(&plano)
            }
        };
        let chao_preto = match &self.vidro_congelado.chao_preto {
            Some((w, c)) if fresca(w, backdrop) => c.clone(),
            _ => {
                let abaixo = self.layers.z_order_bottom_up().first() != Some(&camada);
                let c = abaixo.then(|| Arc::new(self.build_wet_backdrop_sobre([0, 0, 0])));
                self.vidro_congelado.chao_preto = Some((Arc::downgrade(backdrop), c.clone()));
                c
            }
        };
        // O plano é escrito no lugar: quem o partilha (o desfazer, a base congelada) fica com a cópia.
        if Arc::strong_count(&plano) > 1 {
            plano = Arc::new(crate::plane_copy::par_clone(&plano));
        }
        Some(VidroDoQuadro {
            camada,
            plano,
            base: vbase,
            chao_preto,
        })
    }

    /// Devolve o plano escrito ao mapa.
    pub(super) fn devolve_o_vidro(&mut self, camada: RtLayerId, plano: Arc<Vec<Vidro>>) {
        self.vidros.insert(camada, plano);
    }
}

/// **Os alfas por canal de um texel da aguada**, da óptica sobre os dois chãos: `branco`/`preto` são as
/// aparências (bytes) sobre o chão de sempre e sobre o preto, `chao`/`chao_preto` os dois chãos. Onde
/// os dois chãos coincidem (uma camada opaca por baixo) o chão não se vê, e o texel fica com o alfa
/// único `a`.
#[inline]
pub(super) fn alfas_do_vidro(
    branco: [u8; 3],
    preto: [u8; 3],
    chao: [u8; 3],
    chao_preto: [u8; 3],
    a: u8,
) -> [u8; 3] {
    core::array::from_fn(|c| {
        let d = f32::from(chao[c]) - f32::from(chao_preto[c]);
        if d < 1.0 {
            return a;
        }
        let t = ((f32::from(branco[c]) - f32::from(preto[c])) / d).clamp(0.0, 1.0);
        ((1.0 - t) * 255.0).round() as u8
    })
}
