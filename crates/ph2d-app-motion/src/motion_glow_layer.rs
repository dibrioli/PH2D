//! **A CAMADA MOTION que o glow bright-passa** — a lista de instâncias que o passe
//! de isolamento re-renderiza no RT `Rgba16Float` (`present.rs`, Pass 1c).
//!
//! ## O bug que este módulo existe para curar
//!
//! Enio, 2026-08-20: *"Glow não funciona com shape"* — e depois, decidindo a cura:
//! *"tudo deve brilhar, não só shape, mas os objetos Sprite, Vector, Flip e mais
//! tarde os objetos 3d"*.
//!
//! O Motion desenha em **duas metades**, e depois do cook cada elemento cai numa:
//!
//! | metade | lista | quem desenha | quando |
//! |---|---|---|---|
//! | sprites | `pump.instances` | `SpriteRenderer` → `game_rt` | **antes** do tonemap (HDR) |
//! | vetor vivo | `pump.vector_instances` | `motion_shape_gen::encode` → cena Vello | **depois** (LDR) |
//!
//! O passe do glow lia só a primeira. Um `source.shape` — e um `source.object` de
//! VETOR ou de FLIP abaixo do limiar de LOD — emite `geometry_id` e cai na segunda,
//! então o bright-pass lia um RT em que aquele elemento nunca foi desenhado.
//!
//! ⚠️ **E o defeito era INTERMITENTE POR CONTAGEM:** a partição de LOD
//! (`motion_bridge_objects::apply_object_lod`, `LOD_COUNT = 16_000`) MOVE
//! para `instances` toda geometria carimbada acima do limiar que tenha tile — e
//! essas brilhavam. *A mesma forma não brilhava com 16 000 cópias e brilhava com
//! 16 001.* Este módulo mata o degrau: o tile entra na lista do glow **em qualquer
//! contagem**.
//!
//! ## Por que o TILE serve, e não é um remendo
//!
//! A rota «óbvia» seria rasterizar a metade vetorial em alta fidelidade num alvo
//! HDR. ⛔ **Medido: o Vello 0.8 não pode.** O `render_to_texture` dele escreve
//! numa *storage texture* `Rgba8Unorm` (`vello_pass.rs`: *"Vello requires Rgba8Unorm
//! + STORAGE_BINDING"*), então o RT `Rgba16Float` do glow está fora do alcance
//! dele — e passar por um intermediário LDR **perderia o HDR**, que é precisamente
//! onde o bloom vive.
//!
//! ⚠️ **A observação que torna isto barato: o halo NÃO PRECISA DE NITIDEZ.** A
//! primeira coisa que o passe faz com este RT é um bright-pass em meia resolução,
//! seguido de **seis** reduções de mip. Um tile de DPI fixo é indistinguível de uma
//! curva perfeita depois de duas dessas. O caminho CRISPO fica intocado para o
//! quadro visível; o que muda é só de onde o *bloom* tira a silhueta.
//!
//! ⚠️ **E o HDR sobrevive**, que é o que uma ponte por Vello não conseguiria:
//! [`vector_instance_as_tile`]
//! mantém o `tint` verbatim, o tile é branco, e o shader de sprite multiplica os
//! dois — um `tint` de `40` chega ao bright-pass como `40`.
//!
//! ## O que ainda não brilha, e onde está escrito
//!
//! Uma geometria viva **sem tile assado** não pode contribuir — não há de onde tirar
//! a silhueta. Hoje **todas** têm um: os `source.object` (Sprite / Vector / Flip)
//! por [`crate::motion_object_bake`], e o `source.shape` paramétrico por
//! [`crate::motion_shape_bake`], o irmão que esta ordem trouxe. A sonda
//! [`unreachable_geometries`] conta o que sobrar — ela é o que fará um caminho
//! FUTURO nascer visível em vez de mudo.

use super::motion_bridge::vector_instance_as_tile;
use crate::motion_object_bake::ObjectBake;
use crate::motion_shape_bake::ShapeBake;
use crate::motion_state::MotionState;
use ph2d_eval_motion::VectorInstance;
use ph2d_render::RenderInstance;

/// ⭐ doc 121 §9.14 (c) — **O QUE O HALO DESTE QUADRO DESENHA**, pela rota que o quadro tomou.
///
/// ⛔ Num quadro do DISPOSITIVO a bomba da CPU não corre (`MotionState::gpu_live`): as listas dela são
/// de um quadro VELHO (as sprites) ou vazias (as formas). O halo lia-as — as sprites brilhavam onde
/// estiveram, e as formas não brilhavam, e era por isso que uma cena com `fx.glow` e formas recusava a
/// placa. ⇒ as sprites do dispositivo saem do buffer dele (o mesmo que o passe de sprites liga) e as
/// formas da CAMADA do passe de formas, redesenhada no RT do halo
/// ([`crate::motion_shape_placa::PlacaDeFormas::redesenha_em`]).
pub struct Halo<'a> {
    /// As cópias da CPU: as sprites e o TILE de cada forma que a placa NÃO desenhou.
    pub cpu: Vec<RenderInstance>,
    /// As sprites do DISPOSITIVO: o buffer do cozimento, quantas, e os runs de textura.
    pub placa: Option<(&'a wgpu::Buffer, u32, &'a [ph2d_render::GpuTexRun])>,
    /// As formas saem da camada do passe de formas deste quadro.
    pub formas_da_placa: bool,
}

impl Halo<'_> {
    /// Nada a brilhar.
    #[must_use]
    pub fn vazio(&self) -> bool {
        self.cpu.is_empty() && self.placa.is_none() && !self.formas_da_placa
    }
}

/// O halo deste quadro; `formas_da_placa` = o passe de formas desenhou-as neste quadro (TUDO-OU-NADA:
/// então todas as formas vivas estão na camada dele, e nenhuma pede o tile).
#[must_use]
pub fn halo_do_quadro(motion: &MotionState, formas_da_placa: bool) -> Halo<'_> {
    if motion.gpu_live {
        // A MESMA pergunta do passe de sprites (`present.rs`): a arte desenha-se neste quadro?
        let arte = crate::lei_da_aparencia::a_arte_desenha(
            motion,
            ph2d_eval_motion::so_com_forma_por_ordem(),
        );
        let placa = arte
            .then(|| motion.gpu_cook.instances())
            .flatten()
            .map(|gi| (gi.buffer(), gi.len(), motion.gpu_cook.texture_runs()));
        return Halo {
            cpu: Vec::new(),
            placa,
            formas_da_placa,
        };
    }
    let vetores: &[VectorInstance] = if formas_da_placa {
        &[]
    } else {
        &motion.pump.vector_instances
    };
    Halo {
        cpu: layer_instances(
            &motion.pump.instances,
            vetores,
            &motion.object_bake,
            &motion.shape_bake,
        ),
        placa: None,
        formas_da_placa,
    }
}

/// A lista que o passe de isolamento do glow desenha: os sprites, mais toda
/// geometria viva que TENHA um tile assado, convertida em quad.
///
/// ⚠️ **Não é um `apply_*` — não move nada.** A partição de LOD move (ela decide o
/// que o artista VÊ); esta função só **deriva** uma segunda vista para o
/// bright-pass. As duas listas de origem ficam intactas, e é isso que mantém o
/// caminho crispo do quadro visível byte-a-byte como estava.
///
/// ⚠️ **Ordem: sprites primeiro, tiles depois.** O passe de isolamento reordena por
/// z (`sort_render_order`), então a ordem daqui não decide a aparência — mas ela
/// decide a de um `RenderInstance` empatado, e um append determinístico é o que
/// mantém o RT reproduzível entre quadros.
#[must_use]
pub fn layer_instances(
    sprites: &[RenderInstance],
    vectors: &[VectorInstance],
    bake: &ObjectBake,
    shapes: &ShapeBake,
) -> Vec<RenderInstance> {
    let mut out = Vec::with_capacity(sprites.len() + vectors.len());
    out.extend_from_slice(sprites);
    for vi in vectors {
        // ⚠️ **O OBJETO primeiro, e a ordem é load-bearing.** As duas rotas
        // partilham o `shape_store`, então uma geometria de objeto pode existir nos
        // dois assadores; a do objeto é a que o publicador escreveu com o tamanho de
        // mundo no `size`, e é ela que o `vector_instance_as_tile` sabe converter.
        if let Some(texture_id) = bake.tile_texture_for_gid(vi.geometry_id) {
            out.push(vector_instance_as_tile(vi, texture_id));
        } else if let Some(tile) = shapes.tile_for_gid(vi.geometry_id) {
            // A forma PARAMÉTRICA: o tamanho vem do tile e a âncora do bbox — ver
            // [`crate::motion_shape_bake::tile_quad`], que é onde um halo torto
            // nasceria.
            out.push(crate::motion_shape_bake::tile_quad(vi, tile));
        }
    }
    out
}

/// Quantas geometrias vivas do quadro NÃO têm tile — as que o glow não alcança.
///
/// ⚠️ **Uma sonda, não um portão.** Ela existe para o gate afirmar o buraco em vez
/// de o descrever em prosa: enquanto o `source.shape` não for assado este número é
/// maior que zero, e quando alguém o assar o gate que o fixa fica vermelho e obriga
/// a reconferir esta nota — que é exactamente o que impede a nota de envelhecer.
#[must_use]
#[cfg(test)]
pub fn unreachable_geometries(
    vectors: &[VectorInstance],
    bake: &ObjectBake,
    shapes: &ShapeBake,
) -> usize {
    vectors
        .iter()
        .filter(|vi| {
            bake.tile_texture_for_gid(vi.geometry_id).is_none()
                && shapes.tile_for_gid(vi.geometry_id).is_none()
        })
        .count()
}

/// **O DIAGNÓSTICO da camada** (`PH2D_GLOW_DIAG=1`) — imprime, quando os números
/// MUDAM, de que é feita a lista que o bright-pass desenha.
///
/// ⚠️ **Existe porque «o halo não aparece» tem cinco causas indistinguíveis a olho:**
/// o nó não foi encontrado · a intensidade é zero · a geometria não tem tile · o
/// tile não subiu (e o run é **pulado em silêncio** pelo `material_bg` do
/// renderer) · ou o tile está lá e o `threshold` do bright-pass não o alcança.
/// Cada uma tem uma cura diferente, e a diferença entre elas é UMA linha de
/// números.
///
/// ⚠️ **Só imprime na MUDANÇA.** Um diagnóstico por quadro afoga o terminal e o
/// artista deixa de o ler — e a linha que interessa é a primeira.
///
/// ⭐ doc 121 §9.14 — e a ROTA do halo: as sprites do dispositivo (`placa=`) e as formas pela camada
/// do passe de formas (`formas_da_placa`); num quadro do dispositivo as listas da CPU não contam.
pub fn diag(motion: &MotionState, halo: &Halo<'_>, glow: Option<f32>) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static LAST: AtomicU64 = AtomicU64::new(u64::MAX);
    if std::env::var_os("PH2D_GLOW_DIAG").is_none() {
        return;
    }
    let (bake, shapes) = (&motion.object_bake, &motion.shape_bake);
    let (sprites, vectors): (&[RenderInstance], &[VectorInstance]) =
        if halo.placa.is_some() || halo.formas_da_placa {
            (&[], &[])
        } else {
            (&motion.pump.instances, &motion.pump.vector_instances)
        };
    let with_object = vectors
        .iter()
        .filter(|vi| bake.tile_texture_for_gid(vi.geometry_id).is_some())
        .count();
    let with_shape = vectors
        .iter()
        .filter(|vi| {
            bake.tile_texture_for_gid(vi.geometry_id).is_none()
                && shapes.tile_for_gid(vi.geometry_id).is_some()
        })
        .count();
    let blind = vectors.len() - with_object - with_shape;
    let placa = halo.placa.map_or(0, |(_, n, _)| n);
    let key = (sprites.len() as u64) << 40
        | (vectors.len() as u64) << 24
        | (with_object as u64) << 12
        | with_shape as u64
        | u64::from(placa) << 52
        | u64::from(halo.formas_da_placa) << 63;
    if LAST.swap(key, Ordering::Relaxed) == key {
        return;
    }
    eprintln!(
        "[glow-diag] sprites={} vetor_vivo={} (tile_objeto={with_object} tile_forma={with_shape} \
         SEM_TILE={blind}) camada={} placa={placa} formas_da_placa={} glow={}",
        sprites.len(),
        vectors.len(),
        halo.cpu.len(),
        halo.formas_da_placa,
        glow.map_or_else(|| "ausente".to_string(), |i| format!("intensidade {i}")),
    );
    if blind > 0 {
        eprintln!(
            "  (!) {blind} geometria(s) viva(s) SEM TILE — elas não podem brilhar. \
             Se for um `source.shape`, o assador dele não correu."
        );
    }
}

#[cfg(test)]
#[path = "motion_glow_layer_tests.rs"]
mod tests;
