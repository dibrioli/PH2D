//! ⭐ **A placa num quadro da shell** (P4, doc 132 §4) — as duas chamadas que a shell faz: a decisão
//! (antes de o documento ser codificado: com a placa ativa ele vai às faixas) e o desenho (antes do
//! halo, que redesenha as formas). Vivem aqui para a shell só COMPOR: a rota (o dispositivo também
//! com um grupo só de imagens), as corridas e a pousa, e as fontes das imagens.

use ph2d_gpu::GpuContext;
use ph2d_vector::Affine;

use super::{Corrida, GeometriasDaPlaca, PlacaDeFormas, VecPathStore, imagens, misturas_do_quadro};
use crate::motion_state::MotionState;

/// **Este quadro vai à placa?** — pela rota do DISPOSITIVO quando o cozimento está vivo e escreveu
/// formas ou imagens de um grupo (`ligada_dispositivo`), senão pela da CPU (`ligada_cpu`).
pub fn decide_o_quadro(
    placa: &mut PlacaDeFormas,
    gpu: &GpuContext,
    motion: &mut MotionState,
    (ligada_dispositivo, ligada_cpu): (bool, bool),
    cam: Affine,
) -> bool {
    let (misturas, corridas, pousa) = misturas_do_quadro(gpu, motion);
    let formas = motion.gpu_cook.formas().map_or(0, |f| f.len());
    let imagens = motion.gpu_cook.imagens_do_grupo().is_some();
    if motion.gpu_live && (formas > 0 || imagens) {
        placa.decide_do_dispositivo_com_pousa(
            ligada_dispositivo,
            formas,
            &motion.formas_no_dispositivo,
            &motion.shape_store,
            &mut motion.placa_geometrias,
            cam,
            (corridas, pousa),
        )
    } else {
        placa.decide_com_misturas(
            ligada_cpu,
            &motion.pump.vector_instances,
            &motion.shape_store,
            &mut motion.placa_geometrias,
            cam,
            misturas,
        )
    }
}

/// **Desenha a camada do quadro** — as formas (o buffer do DISPOSITIVO só com o cozimento vivo, o
/// mesmo `gpu_live` que o passe de sprites pergunta) e as imagens das saídas em grupo, cada textura
/// pela porta única do renderer. `projeto` = o filtro de imagem do projecto (a tag `0` herda-o).
pub fn desenha_o_quadro<'p>(
    placa: &'p mut PlacaDeFormas,
    gpu: &GpuContext,
    tamanho: (u32, u32),
    motion: &MotionState,
    renderer: &ph2d_render::SpriteRenderer,
    projeto: ph2d_render::ImageFilterMode,
) -> Option<&'p wgpu::TextureView> {
    placa.com_imagens(imagens::fontes_do_quadro(motion, renderer, projeto));
    let buffer = motion
        .gpu_cook
        .formas()
        .filter(|_| motion.gpu_live)
        .map(|f| f.buffer());
    placa.desenha(gpu, tamanho, &motion.placa_geometrias, buffer)
}

/// ⭐ P4 — a decisão do dispositivo com a pousa, a pousa e as imagens do quadro (cortadas do
/// `motion_shape_placa.rs` pelo tecto de LOC: são a metade do quadro que esta porta compõe).
impl PlacaDeFormas {
    /// ⭐⭐⭐ **As cópias vêm do DISPOSITIVO** (doc 121 W3) — o cozimento escreveu `n` cópias no
    /// buffer dele, e a ponte já perguntou handle a handle se a placa as desenha
    /// ([`GeometriasDaPlaca::veredito`], a MESMA porta). Aqui só se garante que cada geometria viva
    /// está preparada para o passe.
    ///
    /// ⚠️ **`ligada` NÃO pergunta pelo vidro**, e é de propósito: o dispositivo já CALOU as formas
    /// no buffer das sprites, logo com a placa desligada elas não se desenhariam em sítio nenhum.
    /// Com o vidro subido, as formas desenham-se por baixo dele, como o resto do mundo.
    ///
    /// ⭐ `corridas` = as misturas por FAIXA de sink ([`super::corridas_por_faixa`], B6) — vazias, o
    /// `draw` de sempre em `Normal`.
    #[allow(clippy::too_many_arguments)] // a decisão do quadro: o buffer, as formas, a vista e as misturas
    pub fn decide_do_dispositivo(
        &mut self,
        ligada: bool,
        n: u32,
        vivas: &[u32],
        store: &VecPathStore,
        geometrias: &mut GeometriasDaPlaca,
        cam: Affine,
        corridas: Vec<Corrida>,
    ) -> bool {
        self.decide_do_dispositivo_com_pousa(
            ligada,
            n,
            vivas,
            store,
            geometrias,
            cam,
            (corridas, ph2d_render::Pousa::Normal),
        )
    }

    /// ⭐ P4 (b) — o [`Self::decide_do_dispositivo`] com a POUSA do quadro ([`super::corridas_das_faixas`]).
    #[allow(clippy::too_many_arguments)] // a decisão do quadro: o buffer, as formas, a vista e as misturas
    pub fn decide_do_dispositivo_com_pousa(
        &mut self,
        ligada: bool,
        n: u32,
        vivas: &[u32],
        store: &VecPathStore,
        geometrias: &mut GeometriasDaPlaca,
        cam: Affine,
        (corridas, pousa): (Vec<Corrida>, ph2d_render::Pousa),
    ) -> bool {
        self.cam = cam;
        self.copias.clear();
        self.corridas = corridas;
        self.pousa = pousa;
        self.fonte.clear();
        for &h in vivas {
            let _ = geometrias.veredito(h, store);
        }
        // ⭐ P4: uma corrida só de IMAGENS (um grupo sem formas) também vai à placa.
        self.ativa = ligada && (n > 0 || self.corridas.iter().any(|c| !c.imagens.is_empty()));
        if !self.ativa {
            self.corridas.clear();
            self.pousa = ph2d_render::Pousa::Normal;
        }
        self.do_dispositivo = self.ativa.then_some(n);
        self.diz(n as usize, geometrias.len());
        self.ativa
    }

    /// ⭐ P4 (b) — **como a camada deste quadro pousa no mundo**: `Normal`, ou o modo da mistura que se
    /// mistura com o mundo fora da camada isolada ([`super::PousaDoQuadro`]). O presente cola a camada com
    /// ela (`ph2d_render::BandBlit::blit_pousa`).
    #[must_use]
    pub fn pousa(&self) -> ph2d_render::Pousa {
        self.pousa
    }

    /// ⭐ P4 — **pinta as imagens da corrida `c` deste quadro** no passe dado: o desenhador que a volta
    /// das células do `Each Shape` (`ph2d_fx_cells::Imagens`) chama no grupo delas. Nada sem imagens.
    pub fn desenha_as_imagens(&self, pass: &mut wgpu::RenderPass<'static>, c: &Corrida) {
        if let (Some(p), Some(l)) = (&self.imagens, &self.ligacoes) {
            p.desenha(pass, l, c);
        }
    }

    /// O [`Self::desenha`] deste quadro tinha imagens de um grupo para pintar.
    #[must_use]
    pub fn tem_imagens(&self) -> bool {
        self.desenhou && self.ligacoes.is_some()
    }

    /// ⭐ P4 (doc 132 §4) — **as imagens das saídas em grupo deste quadro** ([`imagens::fontes_do_quadro`]),
    /// para o [`Self::desenha`] seguinte as pintar nas corridas delas. Só a rota do dispositivo as tem.
    pub fn com_imagens(&mut self, fontes: Option<imagens::FontesDasImagens>) {
        self.fontes = fontes;
    }
}

#[cfg(test)]
mod tests {
    /// ⭐ **O ELO do dispositivo** (o gate de texto que morava na shell, doc 121 W3, trazido com o
    /// código): as duas pontas SÓ com o cozimento vivo, e a rota do dispositivo também num quadro só de
    /// IMAGENS de um grupo (P4). *Mutações: apagar o `filter` do desenho ou o `gpu_live` da decisão (um
    /// quadro que caiu para a CPU desenharia as formas do ANTERIOR) · esquecer as imagens na pergunta
    /// da rota (um grupo só de imagens, calado nas sprites, não se desenharia em sítio nenhum).*
    #[test]
    fn as_duas_pontas_do_quadro_so_com_o_cozimento_vivo() {
        let src = include_str!("motion_shape_placa_quadro.rs");
        let corpo = src.split("#[cfg(test)]").next().expect("o corpo");
        for fio in [
            "if motion.gpu_live && (formas > 0 || imagens) {",
            "placa.decide_do_dispositivo_com_pousa(",
            ".filter(|_| motion.gpu_live)\n        .map(|f| f.buffer());",
            "placa.com_imagens(imagens::fontes_do_quadro(motion, renderer, projeto));",
        ] {
            assert_eq!(corpo.matches(fio).count(), 1, "{fio}");
        }
    }
}
