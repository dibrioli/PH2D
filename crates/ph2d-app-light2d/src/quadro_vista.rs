//! As portas 1 e 1b da [`LuzDoQuadro`] numa chamada, para a superfície do editor ou para o alvo de outra vista (a
//! miniatura da câmera, `docs/GameEngineFuncional/18_miniatura_da_camera_plano.md`).

use super::LuzDoQuadro;
use bevy_ecs::world::World;
use kurbo::Affine;
use ph2d_gpu::GpuContext;

impl LuzDoQuadro {
    /// ⭐ **As portas 1 e 1b numa chamada** — o que a codificação da shell faz por quadro: encena a
    /// cena de smoke armada (uma vez), prepara e desenha os vetores que bloqueiam. Devolve se a luz
    /// está acesa (a shell força então o documento às faixas).
    pub fn codificar(
        &mut self,
        world: &mut World,
        superficie: &ph2d_gpu::SurfaceContext,
        mundo_para_tela: Affine,
        (vetores, xforms, flip): (
            &mut ph2d_vec_scene::VecScene,
            &ph2d_vec_scene::VecXforms,
            &mut ph2d_flip::FlipDoc,
        ),
    ) -> bool {
        let vetores = (vetores, xforms);
        if !self.smoke_encenado {
            self.smoke_encenado = true;
            if let Some(n) = crate::smoke::nivel_armado() {
                crate::smoke::encenar(world, n);
                self.semeadas = crate::smoke::semear_documento(vetores.0, n);
                self.semeado_flip = crate::smoke::semear_flip(flip, n);
                self.motion_por_marcar = n == 8;
            }
        }
        if self.motion_por_marcar {
            self.motion_por_marcar = crate::smoke::marcar_motion(world) == 0;
        }
        if let Some(id) = self.semeado_flip {
            self.semeado_flip = crate::smoke::marcar_flip(world, flip, id);
        }
        if !self.semeadas.is_empty() {
            let pendentes = std::mem::take(&mut self.semeadas);
            self.semeadas = crate::smoke::marcar_semeadas(world, vetores.0, pendentes);
        }
        let alvo = superficie.size();
        self.codificar_no_alvo(
            world,
            superficie.gpu(),
            (alvo.width, alvo.height),
            mundo_para_tela,
            (vetores.0, vetores.1),
        )
    }

    /// A luz de uma 2.ª vista (a miniatura da câmera, GEF 18 A6): a mesma lei, sem encenar o smoke.
    #[must_use]
    pub fn da_vista() -> Self {
        let mut luz = Self::new().com_montagem_em_fundo();
        luz.smoke_encenado = true;
        luz
    }

    /// [`Self::codificar`] num alvo que não é a superfície (a vista da câmera, `docs/GameEngineFuncional/18`).
    pub fn codificar_no_alvo(
        &mut self,
        world: &World,
        gpu: &GpuContext,
        alvo: (u32, u32),
        mundo_para_tela: Affine,
        (vetores, xforms): (&ph2d_vec_scene::VecScene, &ph2d_vec_scene::VecXforms),
    ) -> bool {
        self.lado_max = gpu.device.limits().max_texture_dimension_2d;
        let acesa = self.preparar(world, mundo_para_tela, alvo);
        self.desenhar_vetores(gpu, vetores, xforms);
        acesa
    }
}
