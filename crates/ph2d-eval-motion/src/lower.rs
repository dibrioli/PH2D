//! **Stream → instances** — the CPU lowering, and the one-shot cook around it.
//!
//! Split from `lib.rs` at the HR-18 LOC cap, along the seam that was already
//! there: this file answers *"what does a cooked stream look like on screen?"*
//! and knows nothing about ticks, memos or the `pre` feedback; `lib.rs` runs the
//! CLOCK (`MotionCookPump`) and calls in here to consume a result.
//!
//! The GPU path has the same split for the same reason — `ph2d-gpu-cook`'s
//! lowering is its own module and its own compute pass, so the two lowerings
//! stay comparable side by side (that comparison is the parity gate).

use crate::{Column, Graph, NodeId, OpResolver, PAR_THRESHOLD, RenderInstance, Stream};
use ph2d_nodegraph::cook::{Cook, CookError};
use ph2d_render::SinkStyle;
use rayon::prelude::*;

/// Lower a cooked instance stream **into `out`** (one instance per element),
/// reusing `out`'s capacity: `out` is cleared and refilled, so a steady stream
/// count frame-to-frame allocates nothing (M0.T11 — the per-frame bridge path;
/// zero-alloc gated by M0.T12). Pure + headless: no GPU.
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` for an
/// instance whose stream lacks the matching column (the M0 case — no framing
/// node yet). The shell passes a single opaque atlas tile plus a `size` below
/// the grid spacing so the raw default document renders as clean, distinct
/// quads; a headless caller passes the whole-atlas rect `[0,0,1,1]` and unit
/// size `[1,1]`.
/// O `flip_uv` de UMA linha: a coluna `blend` quando ela existe e diz alguma coisa, senão o
/// do sink (`fallback`).
///
/// ⚠️ **`0` na coluna quer dizer *"o do sink"*, não `Normal`** — ver a nota no chamador. E o
/// número é arredondado e limitado pelo mesmo teto que o `sink_blend_tag` usa (o array de
/// pipelines do renderer), porque um valor fora da faixa vindo de um `value.*` qualquer não
/// pode escolher um pipeline que não existe.
#[must_use]
fn blend_at(col: Option<&Column>, i: usize, fallback: u32) -> u32 {
    let v = scalar_at(col, i, 0.0);
    if !v.is_finite() || v < 0.5 {
        return fallback;
    }
    let top = ph2d_render::pipeline::BLEND_PIPELINE_COUNT as f32;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clampado a 1..=BLEND_PIPELINE_COUNT antes do cast"
    )]
    let tag = (v.round().clamp(1.0, top) as u8) - 1;
    RenderInstance::pack_blend_bits(tag)
}

pub fn lower_to_instances_into(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    style: SinkStyle,
    out: &mut Vec<RenderInstance>,
) {
    out.clear();
    lower_to_instances_onto(stream, default_uv_rect, default_size, style, out);
}

/// Like [`lower_to_instances_into`] but **appends** — `out` keeps whatever it
/// already holds. This is how several render sinks compose into one draw: the
/// pump clears once, then each `motion.output` node's stream lowers onto the
/// same buffer (still zero-alloc in steady state — capacity is retained).
pub fn lower_to_instances_onto(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    style: SinkStyle,
    out: &mut Vec<RenderInstance>,
) {
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    let uv_rect = stream.get("uv_rect");
    // doc 86 §2: the ONE substrate addition. A `texture_id` column names which
    // texture each instance samples — the sprite/vector/Flip a `source.*` node
    // brought in through the membrane. ABSENT → every id is `0` (the shared
    // atlas), which is exactly what this lowering did before ⇒ byte-identical
    // for any graph without an object source. A `texture_id` is a small `u32`
    // (an atlas sentinel `0`, or an `IndividualTextureStore` handle < 2²⁴,
    // exact in f32); the saturating `as u32` guards a non-finite/negative
    // column value to `0` rather than trusting the producer.
    let tex = stream.get("texture_id");
    // ⛔⛔ **A ALFA DA FONTE, e a ausência dela ESCURECIA as bordas** — report do Enio
    // (2026-08-30): *"o Alpha usado escurece as bordas da pintura (diferente da sprite)"*.
    //
    // Este campo era o literal `0.0`, ou seja *«a textura é alfa DIRETA»*, para toda
    // instância que o Motion emite. O caminho normal da sprite põe-no de
    // [`Sprite::premultiplied`], e um documento PINTADO sobe **premultiplicado** (há
    // assert a dizê-lo em `project_painter.rs`). ⇒ o fragmento pré-multiplicava outra vez,
    // dando `RGB·α²`: invisível no interior opaco e **escuro na borda anti-aliased**, que é
    // exactamente o que se vê.
    //
    // ⚠️ **Coluna, não um campo do `SinkStyle`:** a bandeira é da TEXTURA, e uma corrente
    // pode carregar sprites de várias texturas (mídia mista) — pô-la no sink daria uma
    // resposta por corrente a uma pergunta por linha.
    //
    // ⚠️ **Ausente ⇒ `0.0`**, que é o literal que aqui estava ⇒ toda corrente que não a
    // escreve fica **byte-idêntica**.
    let premul = stream.get("premultiplied");
    // doc 89, folha 17: the sink's blend mode, packed into `flip_uv` bits 5-7 —
    // the encoding a sprite's `BlendMode` already rides in, so the renderer keys
    // its draw runs on it with zero ABI cost. Hoisted OUT of `make`: it is one
    // number for the whole sink, not a per-element gather. Tag 0 (`Mix`) packs to
    // `0`, which is what this lowering hardcoded before the param existed ⇒ the
    // default is byte-identical.
    let flip_uv = style.flip_uv();
    // doc 89, folha 07 (o *Echo Operator* do AE): a coluna `blend` deixa cada LINHA
    // escolher como compõe, e não só o sink inteiro. É o que faz um rastro de LUZ — os
    // ecos somam-se em vez de se taparem.
    //
    // ⚠️ **A convenção é `0 = o modo do SINK`, `m + 1 = o modo `m`** — a mesma escada do
    // `texture_id`/`geometry_id`, e ela é o que mantém o default byte-idêntico: uma stream
    // sem a coluna, e uma linha que a junção preencheu com a identidade `0`, leem os dois o
    // número que o sink já dizia. Guardar o modo cru faria uma junção baixar toda linha
    // alheia para `Normal`.
    //
    // ⚠️ E o gather só existe quando a coluna existe: o `flip_uv` continua HOISTED no caso
    // comum (é um número para o sink inteiro), que é a razão por que ele foi tirado do
    // `make` quando o param do sink nasceu.
    let blend_col = stream.get("blend");
    // doc 89, folha 17 (o *SubImage* do Sprite Renderer do Niagara): a coluna `uv_cell`
    // escolhe QUE PEDAÇO da textura cada linha mostra — `[escala_u, escala_v, desloc_u,
    // desloc_v]`, que é exactamente o `uv_xform` que o shader já aplica DENTRO do sub-rect
    // da própria sprite.
    //
    // ⚠️ **Ela é RELATIVA, e é por isso que existe uma coluna nova em vez de se reusar a
    // `uv_rect`.** A `uv_rect` é o rectângulo ABSOLUTO no atlas, e quem o escreve
    // (`source.object`) é o único que sabe qual é o ladrilho do objecto; um nó de flipbook
    // a montante não sabe, e a shell fornece o ladrilho só no momento do lowering. Uma
    // fracção compõe com o ladrilho que a linha tiver — venha da coluna ou do default.
    //
    // Ausente ⇒ `IDENTITY_UV_XFORM`, que é o que este lowering cravava ⇒ byte-idêntico.
    let uv_cell = stream.get("uv_cell");
    // O `sampling` e o `sub_order` são do SINK inteiro (não há gather): içados aqui pela
    // mesma razão que o `flip_uv`.
    let sampling = style.sampling;
    out.reserve(n);
    // Each instance is a pure function of its own index (a five-column gather +
    // one `sin_cos`); no cross-element dependency. Above the threshold
    // `par_extend` spreads it across cores, order-preserving → byte-identical to
    // the serial extend, so the render is unchanged. GPU/M5 Fase 0.
    let make = |i: usize| -> RenderInstance {
        // ADR-0070-amendment-4: RenderInstance carries the 2×2 world
        // basis, not a rotation scalar. A Motion stream emits only a
        // rotation (no skew), so the basis is a pure rotation matrix
        // `[cos, sin, -sin, cos]`. RenderInstance is PresentWorld-only
        // (HR-5 exempt), so std `sin_cos` is fine here.
        //
        // The `rot` column is in **degrees** — the app's authored-angle unit
        // (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes).
        // Radians live nowhere in the Motion authoring surface; only this
        // conversion, at the very edge where the basis is built.
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).to_radians().sin_cos();
        RenderInstance {
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: vec2_at(size, i, default_size),
            atlas_uv: vec4_at(uv_rect, i, default_uv_rect),
            tint: vec4_at(tint, i, [1.0, 1.0, 1.0, 1.0]),
            basis: [cos_r, sin_r, -sin_r, cos_r],
            premultiplied: scalar_at(premul, i, 0.0),
            // doc 89, folha 17: o PIVÔ. A conversão fracção→metros vive numa função
            // só (`SinkStyle::anchor_for`) porque as duas rotas têm de a fazer igual,
            // e ela multiplica pelo tamanho DESTA linha — um stream tem um `size` por
            // elemento, e um pivô em metros deslocaria as peças pequenas de outra
            // maneira que as grandes.
            anchor: style.anchor_for(vec2_at(size, i, default_size)),
            // Sprite-Inspector-v2 v4 ABI fields: a Motion node stream has no
            // per-corner/opacity authoring surface, so those take their identity
            // values (white gradient, full opacity). `flip_uv` DOES have one now
            // — the sink's blend tag, hoisted above; its flip/repeat/tint_fill
            // bits stay zero, which is what `pack_blend_bits` writes.
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: blend_at(blend_col, i, flip_uv),
            texture_id: scalar_at(tex, i, 0.0) as u32,
            // Node-graph emit doesn't have a hierarchy slot — every
            // motion node's instances share `z_order = 0`. Renderer's
            // tiebreaker (`texture_id`) groups them into one run.
            z_order: 0,
            sampling,
            uv_xform: vec4_at(uv_cell, i, RenderInstance::IDENTITY_UV_XFORM),
            // Node-graph emit has no hierarchy → no clip silhouette.
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            // doc 89, folha 17: a SUB-ORDEM. `Texture` (o de sempre) deixa tudo a `0`
            // e o desempate volta a ser o `texture_id`; `Stream` diz que a ordem das
            // LINHAS é a ordem de desenho, e é o índice que a exprime.
            //
            // ⚠️ Ele é o índice na FILEIRA, não o índice no buffer: vários sinks
            // compõem no mesmo `out`, e um contador global faria o 2.º sink desenhar
            // sempre por cima do 1.º — que não é o que `Stream` quer dizer.
            sub_order: if style.stream_order { i as u32 } else { 0 },
        }
    };
    // doc 86 gave `texture_id`; ADR-0154 gives its sibling `geometry_id`. A row
    // whose `geometry_id` is a LIVE handle (`> 0`) is a crisp VECTOR shape drawn
    // by the vector pass (`lower_to_vector_instances_onto`), so it is skipped
    // here — a shape is never ALSO stamped as a shared-atlas quad. No such
    // column ⇒ the original `0..n` lowering, VERBATIM ⇒ byte-identical for every
    // pre-shape graph (the whole point of a convention column).
    match stream.get("geometry_id") {
        None => {
            if n >= PAR_THRESHOLD {
                out.par_extend((0..n).into_par_iter().map(make));
            } else {
                out.extend((0..n).map(make));
            }
        }
        Some(geo) => {
            // `filter` preserves order ⇒ the surviving sprite rows are
            // byte-identical to what the pre-shape lowering produced for them.
            let is_sprite = move |&i: &usize| scalar_at(Some(geo), i, 0.0) <= 0.5;
            if n >= PAR_THRESHOLD {
                out.par_extend((0..n).into_par_iter().filter(is_sprite).map(make));
            } else {
                out.extend((0..n).filter(is_sprite).map(make));
            }
        }
    }
}

/// A cooked **vector** shape instance (ADR-0154) — the other side of the
/// `geometry_id` convention. Unlike [`RenderInstance`] this is NOT a GPU `Pod`:
/// it is consumed on the CPU by the shell, which looks `geometry_id` up in its
/// `VecPathStore` and encodes the `VecPath` at `world_pos`/`basis`/`size` into
/// the Vello scene the `VelloPass` already draws. `basis` is the same 2×2
/// rotation matrix `[cos, sin, -sin, cos]` a Motion stream carries (no skew).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorInstance {
    pub geometry_id: u32,
    pub world_pos: [f32; 2],
    pub size: [f32; 2],
    pub basis: [f32; 4],
    pub tint: [f32; 4],
    /// **O PIVÔ, em metros locais** — o gémeo exacto do `RenderInstance::anchor`
    /// (veredito do Enio, 2026-08-25: *«o sistema deve ser compatível com todos os tipos
    /// de objetos»*).
    ///
    /// ⚠️ **Ele entra ANTES do `basis`, como na sprite**: o ponto local `q` da forma vai
    /// para `P + basis · (anchor + q · size)`. Aplicá-lo depois giraria a peça no centro e
    /// só a deslocaria — o mesmo desenho para todo ângulo, que é precisamente o que um
    /// pivô NÃO é. A conversão fracção→metros é a mesma função dos dois lados
    /// (`SinkStyle::anchor_for`), porque um pivô que diferisse entre as rotas partiria a
    /// composição de mídia mista sem que gate nenhum de uma rota o visse.
    pub anchor: [f32; 2],
}

/// **Stream → VECTOR instances** (ADR-0154) — the sibling of
/// [`lower_to_instances_onto`] for the other side of the `geometry_id`
/// convention. Each row whose `geometry_id` is a LIVE handle (`> 0`) lowers to a
/// [`VectorInstance`] the shell draws through `ph2d-vec-render` into the Vello
/// scene (a crisp path, not a textured quad). Rows with id 0 (or no column) are
/// sprites ⇒ skipped. **Appends** (the pump clears once), so a graph with no
/// shapes leaves `out` empty ⇒ byte-identical. Serial: shapes are a handful, and
/// the cost is the per-shape Vello encode downstream, not this gather.
pub fn lower_to_vector_instances_onto(
    stream: &Stream,
    style: SinkStyle,
    out: &mut Vec<VectorInstance>,
) {
    let Some(geo) = stream.get("geometry_id") else {
        return; // no shapes in this stream
    };
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    for i in 0..n {
        let id = scalar_at(Some(geo), i, 0.0);
        if id <= 0.5 {
            continue; // a sprite row — lowered by `lower_to_instances_onto`
        }
        // Same degrees→basis edge conversion as the sprite lowering (the `rot`
        // column is the app's one authored-angle unit).
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).to_radians().sin_cos();
        let sz = vec2_at(size, i, [1.0, 1.0]);
        out.push(VectorInstance {
            geometry_id: id as u32,
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: sz,
            basis: [cos_r, sin_r, -sin_r, cos_r],
            tint: vec4_at(tint, i, [1.0, 1.0, 1.0, 1.0]),
            // ⚠️ A MESMA função que a sprite usa. `StyleReach::VECTOR` declara que esta
            // rota honra o pivô e a ordem, e NOMEIA por que não honra os outros dois.
            anchor: style.anchor_for(sz),
        });
    }
}

/// Lower a cooked instance stream to render instances (one per element).
/// Pure + headless. Allocates a fresh `Vec`; the per-frame path uses
/// [`lower_to_instances_into`] to reuse a buffer instead. Uses the whole-atlas
/// UV `[0,0,1,1]` for any instance without a `uv_rect` column (the shell path
/// supplies a real tile via [`lower_to_instances_into`]'s `default_uv_rect`).
pub fn lower_to_instances(stream: &Stream) -> Vec<RenderInstance> {
    let mut out = Vec::new();
    // No graph, no sink: this headless helper has nothing to ask, so it lowers
    // PLAIN — the style every caller of it got before the params existed.
    lower_to_instances_into(
        stream,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle::PLAIN,
        &mut out,
    );
    out
}

/// Cook `target` at `playhead` and lower its **output port 0** to render
/// instances. Reuse the same [`Cook`] across frames for incremental cheapness.
///
/// Lowering a single port is intentional: a Motion render target is one
/// instance stream. A target with several output ports has only port 0 lowered
/// here (a multi-port target would select the port at the call site — not
/// needed by any Motion node today, all of which have exactly one output). A
/// target that legitimately declares **zero** outputs yields an empty `Vec`;
/// note the cook itself already rejects a node that *declares* an output but
/// emits none ([`CookError::OutputCountMismatch`]), so an empty result here
/// means "no output port", never a dropped stream.
pub fn evaluate_motion(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
) -> Result<Vec<RenderInstance>, CookError> {
    let mut out = Vec::new();
    evaluate_motion_into(
        cook,
        graph,
        ops,
        target,
        playhead,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &mut out,
    )?;
    Ok(out)
}

/// Cook `target` at `playhead` and lower its output port 0 **into `out`**,
/// reusing the buffer's capacity (M0.T11 — the per-frame bridge entry). Same
/// single-port semantics as [`evaluate_motion`]; a target with no output port
/// leaves `out` empty. Reuse the same [`Cook`] AND the same `out` across frames
/// for the zero-alloc steady state (gated by M0.T12).
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` fallbacks for
/// a stream without the matching column (see [`lower_to_instances_into`]).
#[allow(clippy::too_many_arguments)] // cook + graph + resolver + target + playhead + 2 defaults + out
pub fn evaluate_motion_into(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) -> Result<(), CookError> {
    let outputs = cook.cook(graph, ops, target, playhead)?;
    // A cooked output port is a `CookValue`; a Motion target's port 0 is an
    // instance stream (ADR-0058-amendment-1). A non-stream value lowers to no
    // instances (its `as_stream()` is empty).
    match outputs.first() {
        Some(v) => lower_to_instances_into(
            v.as_stream(),
            default_uv_rect,
            default_size,
            // This helper COOKS a target, so the target IS the sink — it gets the
            // same answer the pump's loop gets, from the same door.
            crate::sink_style(graph, target),
            out,
        ),
        None => out.clear(),
    }
    Ok(())
}

fn scalar_at(c: Option<&Column>, i: usize, default: f32) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec2_at(c: Option<&Column>, i: usize, default: [f32; 2]) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec4_at(c: Option<&Column>, i: usize, default: [f32; 4]) -> [f32; 4] {
    match c {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
