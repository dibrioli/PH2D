//! ⭐ **A PONTE do balde** — o que ainda pergunta à `App`, e só isso.
//!
//! A **lei** (a rede de paredes, a chave de conteúdo do cache, as âncoras de cada face, o passeio
//! que apaga uma tinta, e os três verbos `upkeep`/`hover`/`deposit`) é pura sobre o
//! `SimWorld`/`VecScene` e mudou-se para [`ph2d_app_vec::bucket`] na Fase C.
//!
//! # ⭐ Porque o corte ficou aqui, e não seis linhas acima
//!
//! A 1.ª tentativa desta fatia partiu o ficheiro em *«tudo o que não é `impl App`»* — e o
//! compilador acusou o corte: a ponte continuava a chamar **seis ajudantes privados**
//! (`contornos_mundo`, `preenchimentos`, `chave`, `geometria_local`, `fora_da_rede`, `Forma`) e a
//! ler **três campos** do `BucketCache`. Abri-los todos teria publicado a cozinha inteira do
//! módulo para servir quatro métodos.
//!
//! ⇒ o que atravessa passou a ser **três funções**, com o que elas precisam escrito em TIPOS
//! (HOWTO §1.5). ⚠️ *Um corte que obriga a abrir os ajudantes está no sítio errado: a régua é
//! quantos nomes atravessam, não quantas linhas ficam de cada lado.*
//!
//! ⛔ **Não é um pedido de sexto método de host**: o que estes quatro métodos leem — `gfx`,
//! `vec_draw_config`, `vec_bucket_cache`, `vec_bucket_face`, `vec_pen` — são campos da `App`, e a
//! cura de um campo é tirá-lo da `App`, nunca alargar o trait (HOWTO §1.5).
//!
//! ⚠️ **A re-exportação em baixo é o que mantém os sítios de chamada byte a byte iguais**: dois
//! campos da `App` são TIPADOS por este módulo (`vec_bucket_cache: Option<crate::vec_bucket::BucketCache>`
//! e `vec_bucket_face: Option<crate::vec_bucket::BucketHit>`) — *uma fronteira nova não muda um
//! tipo de sítio, muda o nome pelo qual se chega a ele.*

pub(crate) use ph2d_app_vec::bucket::*;

impl crate::App {
    /// No laço do quadro, em qualquer ferramenta. Ver [`ph2d_app_vec::bucket::upkeep`].
    pub(crate) fn bucket_upkeep(&mut self) {
        let armado = self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Bucket;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        ph2d_app_vec::bucket::upkeep(
            &gfx.sim,
            &mut gfx.vec_scene,
            &self.vec_entities,
            armado,
            &mut self.vec_bucket_cache,
            &mut self.vec_bucket_face,
        );
    }

    /// **Recalcula a região sob o cursor** — só com o balde na mão.
    ///
    /// ⚠️ **Fora do modo ele é LIMPO**, e não apenas não-actualizado: uma região a arder depois de
    /// trocar de ferramenta prometeria um preenchimento que nenhum clique faria.
    pub(crate) fn refresh_bucket_hover(&mut self, pointer: (f32, f32)) {
        if self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Bucket {
            self.vec_bucket_face = None;
            return;
        }
        let Some(world) = self.vec_world_at(pointer) else {
            self.vec_bucket_face = None;
            return;
        };
        let Some(cache) = self.vec_bucket_cache.as_ref() else {
            self.vec_bucket_face = None;
            return;
        };
        self.vec_bucket_face = ph2d_app_vec::bucket::hover(cache, world);
    }

    /// **A tinta que o balde deposita** — a corrente da ferramenta.
    ///
    /// ⚠️ **`alpha == 0` significa SEM preenchimento** neste app (a convenção que a ferramenta de
    /// forma usa ao fechar), e um balde que a ignorasse depositaria formas invisíveis.
    pub(crate) fn bucket_paint(&self) -> Option<ph2d_vec_scene::Rgba8> {
        let f = self.vec_pen.style().fill;
        (f.a != 0).then_some(f)
    }

    /// ⭐⭐⭐ **DEPOSITA a região que está acesa.** `true` se algo nasceu.
    ///
    /// ⚠️ **A receita (a semente) é armada DEPOIS do `sync`** — no clique a entidade ainda não
    /// existe —, e é lá que a forma também vai para o fundo.
    pub(crate) fn apply_bucket(&mut self) -> bool {
        let Some(tinta) = self.bucket_paint() else {
            eprintln!(
                "[ph2d-vec] balde: o preenchimento corrente e' transparente — escolha uma cor"
            );
            return false;
        };
        let Some(hit) = self.vec_bucket_face.clone() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let id = ph2d_app_vec::bucket::deposit(
            &gfx.sim,
            &mut gfx.vec_scene,
            &self.vec_entities,
            &hit,
            tinta,
        );
        #[allow(clippy::cast_possible_truncation)]
        self.vec_bucket_new
            .push((id, [hit.seed[0] as f32, hit.seed[1] as f32], hit.ancoras));
        self.vec_pen.select(Some(id));
        // A rede não muda (um preenchimento não é parede), mas o `upkeep` tem de reconhecer o
        // caminho novo como fill — o cache cai e o próximo quadro reconstrói com ele de fora.
        self.vec_bucket_cache = None;
        self.vec_bucket_face = None;
        true
    }
}
