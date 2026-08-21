//! **O TILE de uma forma paramétrica** — a metade que faltava para o `source.shape`
//! brilhar (bug do Enio, 2026-08-20: *"Glow não funciona com shape"* → *"tudo deve
//! brilhar"*).
//!
//! ## Por que um assador SEPARADO do [`crate::motion_object_bake`]
//!
//! O irmão assa o que está na **cena vetorial** (`VecScene` + `VecXforms` +
//! `LiveGeometry`): ele resolve um `VecPathId` do documento. Uma forma de
//! `source.shape` não está no documento — ela é **paramétrica**, construída pelo
//! shell a partir dos params do nó e internada no `shape_store` sob a sua chave de
//! conteúdo. As duas rotas partilham o STORE (é o que o `bake_objects` diz:
//! *"there is ONE store for `source.shape` AND `source.object` vectors"*) e agora
//! partilham a lei de limites e a porta de desenho da `ph2d-vec-render` — o que elas
//! não partilham é de onde vem o caminho.
//!
//! ## O tile é para o BLOOM, não para a tela
//!
//! O caminho crispo continua a desenhar o quadro visível. Este tile existe só para o
//! bright-pass do glow ter de onde tirar a silhueta — ver
//! [`crate::render_loop::motion_glow_layer`], que explica por que um DPI fixo basta
//! (seis reduções de mip depois, ninguém distingue) e por que o Vello não podia
//! escrever direto no RT HDR.
//!
//! ## ⚠️ A ÂNCORA é o que separa isto de um halo torto
//!
//! Um quad de sprite é centrado no `world_pos` da instância. O bbox de uma forma
//! paramétrica **não** é necessariamente centrado na origem local dela (uma seta,
//! uma fatia de pizza, um arco), e a instância que o `source.shape` emite tem
//! `size` = **unidade** — a dimensão real está na própria geometria, porque
//! `build_shape_path` já a construiu no tamanho do param.
//!
//! Então o quad não pode copiar `size`/`world_pos` da instância como o
//! `vector_instance_as_tile` faz para os objetos (lá o publicador escreve o tamanho
//! de mundo assado no `size`). Ele tem de:
//!
//! 1. **medir** a forma → `world_size` e o **centro do bbox** em unidades locais;
//! 2. **escalar** o quad por `world_size · vi.size`;
//! 3. **deslocar** o centro pelo bbox, passando pela BASE da instância (rotação e
//!    escala), e só então somar ao `world_pos`.
//!
//! O passo 3 é o que um `vector_instance_as_tile` ingénuo erraria, e o sintoma seria
//! **o halo ao lado da forma** — pior que halo nenhum, porque parece um bug de
//! desenho. É [`tile_quad`], e é pura: tem gate.

use ph2d_eval_motion::VectorInstance;
use ph2d_render::{RenderInstance, SpriteRenderer};
use ph2d_vector::{Affine, VectorScene};

use crate::motion_object_bake::{BAKE_DPI, MAX_TILE_SIDE};
use crate::render_loop::motion_shape_gen::VecPathStore;

/// O tile assado de uma forma: a textura e o que ela mede em MUNDO.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShapeTile {
    /// O handle da `IndividualTextureStore` (refcontado).
    pub(crate) texture_id: u32,
    /// O que o tile mede em unidades de mundo, na escala unitária da instância.
    pub(crate) world_size: [f32; 2],
    /// O centro do bbox em unidades LOCAIS — a âncora que o quad tem de honrar.
    pub(crate) local_center: [f32; 2],
}

/// Os tiles das formas paramétricas deste documento, por `geometry_id`.
///
/// ⚠️ **Cache por HANDLE, e o handle é por chave de CONTEÚDO** (`shape_store::intern`):
/// mudar um param do nó dá outra chave, outro handle e outro tile, e o antigo fica
/// órfão até o `release`. É a mesma disciplina do irmão.
#[derive(Default)]
pub(crate) struct ShapeBake {
    tiles: std::collections::BTreeMap<u32, ShapeTile>,
    scratch: Option<ph2d_render::VelloPass>,
}

impl ShapeBake {
    /// O tile de uma geometria, se ela já foi assada.
    pub(crate) fn tile_for_gid(&self, geometry_id: u32) -> Option<ShapeTile> {
        self.tiles.get(&geometry_id).copied()
    }

    /// Semeia um tile sem GPU — a porta que deixa os gates da CAMADA provarem que a
    /// forma paramétrica chega ao bright-pass, sem um adaptador de gráficos.
    #[cfg(test)]
    pub(crate) fn seed_for_test(&mut self, geometry_id: u32, tile: ShapeTile) {
        self.tiles.insert(geometry_id, tile);
    }

    /// Assa o que falta: para cada `geometry_id` pedido que ainda não tem tile,
    /// mede a forma, rasteriza-a no DPI fixo e envia a textura.
    ///
    /// ⚠️ **Só o que FALTA.** O readback é lento (é uma leitura de GPU), e uma cena
    /// parada não pode pagá-lo por quadro — a mesma razão do irmão.
    pub(crate) fn bake_missing(
        &mut self,
        store: &VecPathStore,
        wanted: impl IntoIterator<Item = u32>,
        gpu: &ph2d_gpu::GpuContext,
        renderer: &mut SpriteRenderer,
        surface_format: wgpu::TextureFormat,
    ) {
        for gid in wanted {
            if self.tiles.contains_key(&gid) {
                continue;
            }
            if let Some(tile) = self.bake_one(store, gid, gpu, renderer, surface_format) {
                self.tiles.insert(gid, tile);
            }
        }
    }

    fn bake_one(
        &mut self,
        store: &VecPathStore,
        gid: u32,
        gpu: &ph2d_gpu::GpuContext,
        renderer: &mut SpriteRenderer,
        surface_format: wgpu::TextureFormat,
    ) -> Option<ShapeTile> {
        let path = store.get(gid)?;
        // A MESMA câmera do irmão: DPI fixo, Y invertido (a linha 0 do readback é o
        // topo da tela). Uma segunda convenção aqui poria o tile de cabeça para baixo.
        let camera = Affine::scale_non_uniform(BAKE_DPI, -BAKE_DPI);
        let (x0, y0, x1, y1) = ph2d_vec_render::standalone_path_screen_bounds(path, camera)?;
        #[expect(clippy::cast_possible_truncation, reason = "clampado a MAX_TILE_SIDE")]
        #[expect(clippy::cast_sign_loss, reason = "x1 > x0 por construção do bbox")]
        let wpx = ((x1 - x0).ceil() as u32).clamp(1, MAX_TILE_SIDE);
        #[expect(clippy::cast_possible_truncation, reason = "clampado a MAX_TILE_SIDE")]
        #[expect(clippy::cast_sign_loss, reason = "y1 > y0 por construção do bbox")]
        let hpx = ((y1 - y0).ceil() as u32).clamp(1, MAX_TILE_SIDE);

        let mut scene = VectorScene::new();
        ph2d_vec_render::draw_path_standalone(
            path,
            Affine::translate((-x0, -y0)) * camera,
            &mut scene,
        );
        let pass = match self.scratch.as_mut() {
            Some(p) => p,
            None => self
                .scratch
                .insert(ph2d_render::VelloPass::new(gpu, surface_format, (wpx, hpx)).ok()?),
        };
        let mut rgba = pass
            .render_and_readback(gpu, scene.inner(), (wpx, hpx))
            .ok()?;
        let want = (wpx * hpx * 4) as usize;
        if rgba.len() < want {
            return None;
        }
        rgba.truncate(want);
        let texture_id = renderer.acquire_individual(wpx, hpx, &rgba).ok()?;

        // ⚠️ **O bbox voltou pela CÂMERA, então desfazê-la é o que dá as unidades
        // locais** — e o Y da câmera é NEGATIVO, por isso o centro em Y troca de sinal.
        #[expect(clippy::cast_possible_truncation, reason = "px de tile em f32")]
        let world_size = [((x1 - x0) / BAKE_DPI) as f32, ((y1 - y0) / BAKE_DPI) as f32];
        #[expect(clippy::cast_possible_truncation, reason = "px de tile em f32")]
        let local_center = [
            ((x0 + x1) * 0.5 / BAKE_DPI) as f32,
            ((y0 + y1) * 0.5 / -BAKE_DPI) as f32,
        ];
        Some(ShapeTile {
            texture_id,
            world_size,
            local_center,
        })
    }
}

/// **O quad que representa a forma no bright-pass** — a colocação, e é pura.
///
/// ⚠️ **Três correcções sobre a conversão dos OBJETOS**, cada uma com um sintoma se
/// faltar (ver o cabeçalho): o tamanho vem do tile e não da instância (senão o halo
/// nasce 1×1); ele multiplica o `size` da instância (senão um `motion.scale` a
/// jusante não o alcança); e o centro anda pelo bbox **através da base** (senão o
/// halo aparece ao lado da forma numa espécie não-centrada).
#[must_use]
pub(crate) fn tile_quad(vi: &VectorInstance, tile: ShapeTile) -> RenderInstance {
    let [b0, b1, b2, b3] = vi.basis;
    let [sx, sy] = vi.size;
    // O centro do bbox, na escala e na rotação da instância.
    let (cx, cy) = (tile.local_center[0] * sx, tile.local_center[1] * sy);
    let off = [b0 * cx + b2 * cy, b1 * cx + b3 * cy];
    let mut inst = crate::render_loop::motion_bridge::vector_instance_as_tile(vi, tile.texture_id);
    inst.world_pos = [vi.world_pos[0] + off[0], vi.world_pos[1] + off[1]];
    inst.size = [tile.world_size[0] * sx, tile.world_size[1] * sy];
    inst
}

#[cfg(test)]
#[path = "motion_shape_bake_tests.rs"]
mod tests;
