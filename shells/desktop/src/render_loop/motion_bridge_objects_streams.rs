//! **Os streams que descrevem um objeto** — a APARÊNCIA (o template) e a POSE.
//!
//! ⚠️ **Três canais, e a separação é load-bearing** (ver `external::position_of`): a
//! aparência mora na ORIGEM sem pose, porque ela diz *como a coisa é* e o grafo decide
//! onde as cópias vão; a POSIÇÃO tem canal próprio; e desde 2026-08-21 a POSE também.
//! Fundi-los é como o `motion.look_at` shipou partido — uma coluna com o nome certo a
//! guardar a resposta de outra pergunta.

// ⚠️ `pub(crate)` e não `pub(super)`: o pai reexporta-os para o `render_loop`, e um
// `pub(super)` daqui alcança só o pai — o reexport falharia a compilar.
use ph2d_nodegraph::attr::{Column, Stream};

/// The one-instance appearance stream, at the origin. ⚠️ **The columns here are
/// exactly the ones `lower_to_instances` reads** — `P` (world_pos), `size`,
/// `tint`, `uv_rect` (atlas_uv), `texture_id` — so what the membrane publishes
/// and what the sink lowers cannot diverge (the two-doors bug). Pure, so that
/// column contract is unit-tested without a GPU atlas.
pub(crate) fn appearance_tile(
    size: [f32; 2],
    tint: [f32; 4],
    uv_rect: [f32; 4],
    texture_id: u32,
    premultiplied: bool,
) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![size]))
        .with("tint", Column::Vec4(vec![tint]))
        .with("uv_rect", Column::Vec4(vec![uv_rect]))
        // A small integer id, exact in f32; the lowering reads it back.
        .with("texture_id", Column::Scalar(vec![texture_id as f32]))
        // ⛔⛔ **A BANDEIRA DA ALFA VIAJA** — report do Enio (2026-08-30): *"o Alpha usado
        // escurece as bordas da pintura (diferente da sprite)"*. O lowering pré-multiplicava
        // toda instância do Motion (`premultiplied: 0.0` literal), e um documento PINTADO sobe
        // **já premultiplicado** ⇒ `RGB·α²` ⇒ borda escura. O caminho normal da sprite lê
        // [`ph2d_render::Sprite::premultiplied`]; agora esta rota também.
        //
        // ⚠️ **PARÂMETRO, não um default:** ele é uma pergunta que só o produtor sabe
        // responder, e um default aqui seria a mesma mentira noutro sítio.
        .with(
            "premultiplied",
            Column::Scalar(vec![f32::from(u8::from(premultiplied))]),
        )
}

/// The one-instance appearance stream for a LIVE VECTOR (a `source.object` that
/// names a vector, ADR-0154 reused for objects): `(P, size, tint, geometry_id)` —
/// no `uv_rect`/`texture_id`, because a live vector is drawn crisp by the vector
/// pass, not sampled as an atlas quad. The lowering routes a `geometry_id > 0` row
/// there automatically (its `> 0.5` split), and the sprite lowering SKIPS it, so a
/// mixed group stream draws each part once. The tint is WHITE — the drawing's own
/// fill/stroke carry its colours ([`ph2d_vec_render::draw_shape_instance`]).
/// **A POSE de um objeto, como stream** — a rotação e a escala que ele carrega na cena.
///
/// ⚠️ **Função NOMEADA e não uma expressão inline, e a razão foi medida:** trocar
/// `t.rotation` por `0.0` dentro do `publish` não punha nenhum teste vermelho (mutação
/// sobrevivente, doc 89 folha 14). Uma expressão enterrada num laço que precisa de um
/// mundo ECS e de um atlas para correr é uma expressão que ninguém gateia; extraída, ela
/// tem um gate de três linhas.
///
/// ⚠️ **Canal próprio, pela mesma razão do `position_of`:** a APARÊNCIA mora na origem
/// sem pose de propósito — ela diz *como a coisa é*, e o grafo decide onde as cópias vão.
pub(crate) fn pose_stream(t: &ph2d_ecs::Transform) -> Stream {
    Stream::new(1)
        .with("rotation", Column::Scalar(vec![t.rotation]))
        .with("size", Column::Vec2(vec![[t.scale[0], t.scale[1]]]))
}

pub(crate) fn appearance_vector(size: [f32; 2], tint: [f32; 4], geometry_id: u32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![size]))
        .with("tint", Column::Vec4(vec![tint]))
        .with("geometry_id", Column::Scalar(vec![geometry_id as f32]))
}
