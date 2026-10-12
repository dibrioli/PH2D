//! ⭐ **O traço do Flip na luz 2D** (W5e, `docs/Luz2D/01_plano.md` §2.4) — os objetos Flip cuja
//! entidade tem `BloqueiaLuz` desenham-se, para a luz, numa camada alinhada à grade dela: a MESMA
//! tesselação do passe (o [`super::FlipComposite`] guarda-a por desenho) e o rasterizador do preview
//! (premultiplicado, a cobertura analítica do traço = a área), num `FlipRenderer` próprio — a
//! profundidade do tamanho da grade não disputa a do ecrã a cada quadro.

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipObjectId};
use ph2d_flip_entities::entities::FlipEntityMap;
use ph2d_flip_render::camera::fold_model;
use ph2d_flip_render::{CameraRaw, FlipRenderer};
use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{CamadaDeOclusao, FORMATO_DA_OCLUSAO};
use ph2d_vec_scene::Xform;

use super::FlipComposite;

/// O Flip do quadro, para a luz: o documento, a sessão do passe e os afins dos objetos.
pub struct OclusaoDoFlip<'a> {
    pub doc: &'a FlipDoc,
    pub composite: &'a mut Option<FlipComposite>,
    pub models: &'a [(FlipObjectId, Xform)],
    pub playhead: &'a Playhead,
    pub entidades: &'a FlipEntityMap,
}

/// ⭐ W5j — os objetos Flip (`FlipObjectId.0`) cujas entidades estão em `sem_luz` (`NaoRecebeLuz`).
#[must_use]
pub fn nao_recebem(entidades: &FlipEntityMap, sem_luz: &[ph2d_ecs::Entity]) -> Vec<u64> {
    entidades
        .iter()
        .filter(|(_, bits)| sem_luz.iter().any(|e| e.to_bits() == **bits))
        .map(|(id, _)| id.0)
        .collect()
}

/// A câmera que põe a janela do mundo `[x0, y0, x1, y1]` no alvo inteiro (a linha 0 é `y1`).
#[must_use]
pub fn camera_do_mundo(mundo: [f64; 4], (w, h): (u32, u32)) -> CameraRaw {
    let [x0, y0, x1, y1] = mundo;
    let (sx, sy) = (2.0 / (x1 - x0), 2.0 / (y1 - y0));
    // Coluna-major: clip = M · (x, y, 0, 1).
    let m = [
        [sx as f32, 0.0, 0.0, 0.0],
        [0.0, sy as f32, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [(-1.0 - x0 * sx) as f32, (-1.0 - y0 * sy) as f32, 0.0, 1.0],
    ];
    CameraRaw::new(m, [w as f32, h as f32], (f64::from(w) / (x1 - x0)) as f32)
}

impl CamadaDeOclusao for OclusaoDoFlip<'_> {
    fn desenhar(
        &mut self,
        gpu: &GpuContext,
        alvo: &wgpu::TextureView,
        tamanho: (u32, u32),
        mundo: [f64; 4],
        quem: &[u64],
    ) -> bool {
        let objetos: Vec<FlipObjectId> = self
            .entidades
            .iter()
            .filter(|(_, bits)| quem.contains(bits))
            .map(|(&id, _)| id)
            .collect();
        if objetos.is_empty() {
            return false;
        }
        let comp = self
            .composite
            .get_or_insert_with(|| FlipComposite::new(gpu));
        let raster = comp
            .luz
            .get_or_insert_with(|| FlipRenderer::new(&gpu.device, FORMATO_DA_OCLUSAO));
        let base = camera_do_mundo(mundo, tamanho);
        let mut desenhou = false;
        for obj in self
            .doc
            .objects()
            .iter()
            .filter(|o| objetos.contains(&o.id))
        {
            let frame = obj.frame_at(self.playhead);
            let model = self
                .models
                .iter()
                .find(|(id, _)| *id == obj.id)
                .map_or(Xform::IDENTITY, |(_, x)| *x);
            let cam = if model.is_identity() {
                base
            } else {
                fold_model(&base, &model)
            };
            for layer in obj.layers().iter().filter(|l| l.visible) {
                let Some(did) = layer.drawing_at_cycled(frame) else {
                    continue;
                };
                let Some(d) = obj.drawing(did).filter(|d| !d.strokes.is_empty()) else {
                    continue;
                };
                let chave = (obj.id.0, did.0);
                comp.tess.ensure(chave, d);
                if let Some(data) = comp.tess.get(&chave) {
                    super::rasterizar_em(raster, data, &cam, alvo, tamanho, gpu);
                    desenhou = true;
                }
            }
        }
        desenhou
    }
}

#[cfg(test)]
#[path = "pass_luz_tests.rs"]
mod testes;
