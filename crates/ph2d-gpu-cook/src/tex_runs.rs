//! The **texture-run partition** — the CPU-side half of drawing a
//! `source.object` graph on the device (this wave).
//!
//! The GPU lowering writes each instance's real `texture_id` into word 41
//! (metade A), but the sprite shader never reads that word: texture selection
//! is a per-draw CPU bind. So the cook hands the renderer a partition of the
//! instance range into contiguous same-`texture_id` runs, and the renderer
//! binds the object's texture per run. This module computes that partition from
//! the **boundary** stream's `texture_id` column — the CPU stream the cook was
//! handed — so it NEVER reads the device buffer back. Kept beside the sequencer
//! rather than in it because it answers a render question, not a cook one.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;
use ph2d_render::GpuTexRun;

/// Partition the instance range `[0, count)` into contiguous runs of equal
/// `texture_id`, read from the principal object boundary — the boundary
/// [`Stream`] whose `texture_id` `Scalar` column has length `count`.
///
/// A per-element GPU suffix (deformer, force, oscillator, `value.*`) preserves
/// count and position, so that length uniquely names the object's instance
/// stream, and the column is the same content the device's word 41 holds
/// (metade A) ⇒ a run boundary lands exactly where the device texture changes,
/// WITHOUT reading the device back. The count-changing cerca
/// (`GpuPlan::suffix_changes_count`) is what keeps that alignment true — a
/// reordering suffix recuses to the CPU render before it ever reaches here.
///
/// Leaves `out` empty — the renderer's legacy single atlas draw, byte-identical
/// — in two cases: no boundary carries `texture_id` (every non-object graph),
/// or every id is `0` (an object graph whose tiles all live in the shared
/// atlas, which the atlas draw already renders correctly). `v as u32` mirrors
/// the CPU lowering's `scalar_at(..) as u32` and the WGSL `u32(read_texture_id)`
/// so a run boundary lands where word 41 changes.
pub(crate) fn texture_runs_from_boundary(
    boundary_streams: &[(NodeId, &Stream)],
    count: u32,
    // ⭐⭐⭐ **A MISTURA DESTE SINK** — o tag que a `sink_style` leu da porta única. Ela
    // entra aqui e não no shader porque a escolha de pipeline é da CPU, por chamada de
    // desenho, exactamente como a textura (ver [`GpuTexRun`]).
    //
    // ⛔⛔ **E ela decide quando a partição pode ficar VAZIA.** Vazia quer dizer, para o
    // desenho, *«o átlas, em `Mix`»* — o ramo `runs.is_empty()` do `renderer_draw`, que é
    // o caminho de sempre. Com um tag diferente de `0` esse ramo **perderia a mistura em
    // silêncio**, que é o defeito que esta wave mediu em pixel; logo aqui um sink que peça
    // outra mistura emite um run EXPLÍCITO mesmo quando toda a textura é o átlas.
    blend: u8,
    // ⭐⭐ **A AMOSTRAGEM deste sink** (doc 119 §3) — a mesma regra da mistura: é um BIND da CPU, e
    // com um valor ≠ de fábrica a partição não pode ficar vazia.
    sampling: u32,
    out: &mut Vec<GpuTexRun>,
) {
    if count == 0 {
        return;
    }
    // ⚠️ O run de corpo inteiro: a saída dos dois casos que, com `Mix`, deixam `out`
    // vazio. Escrito uma vez porque *os dois têm de concordar* — um deles a esquecer o
    // tag seria uma cena a desenhar na mistura errada conforme o grafo traz ou não a
    // coluna, que é a forma que ninguém liga a uma causa.
    let tudo_num_run = |out: &mut Vec<GpuTexRun>| {
        if blend != 0 || sampling != 0 {
            out.push(GpuTexRun {
                texture_id: 0,
                start: 0,
                end: count,
                blend,
                sampling,
            });
        }
    };
    let n = count as usize;
    let ids = boundary_streams
        .iter()
        .find_map(|(_, s)| match s.get("texture_id") {
            Some(Column::Scalar(v)) if v.len() == n => Some(v.as_slice()),
            _ => None,
        });
    let Some(ids) = ids else {
        tudo_num_run(out);
        return;
    };
    // All-atlas object graph → the legacy path already draws it (an atlas run in
    // the device buffer needs no per-run bind), so stay byte-identical.
    if ids.iter().all(|&v| v as u32 == 0) {
        tudo_num_run(out);
        return;
    }
    let mut start = 0u32;
    let mut cur = ids[0] as u32;
    for (i, &v) in ids.iter().enumerate().skip(1) {
        let tid = v as u32;
        if tid != cur {
            out.push(GpuTexRun {
                texture_id: cur,
                start,
                end: i as u32,
                blend,
                sampling,
            });
            start = i as u32;
            cur = tid;
        }
    }
    out.push(GpuTexRun {
        texture_id: cur,
        start,
        end: count,
        blend,
        sampling,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::graph::NodeId;

    /// A boundary with one node carrying a `texture_id` column of the given ids.
    fn boundary(ids: &[f32]) -> Stream {
        let n = ids.len();
        let mut s = Stream::new(n);
        // `P` alongside `texture_id` — a real object stream carries both; the
        // partition reads only `texture_id`, so `P`'s value is irrelevant.
        s.set("P", Column::Vec2(vec![[0.0, 0.0]; n]));
        s.set("texture_id", Column::Scalar(ids.to_vec()));
        s
    }

    fn runs(ids: &[f32]) -> Vec<GpuTexRun> {
        runs_com(ids, 0)
    }

    /// A mesma coisa com a MISTURA do sink escolhida — o que a `sink_style` leu.
    fn runs_com(ids: &[f32], blend: u8) -> Vec<GpuTexRun> {
        let s = boundary(ids);
        let node = NodeId(0);
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(node, &s)], ids.len() as u32, blend, 0, &mut out);
        out
    }

    /// **K>1 — the phenomenon.** Two objects, three instances each, adjacent →
    /// two runs. This is the fixture that proves the partition (a single run
    /// would prove nothing about grouping). Red on a lowering that never carried
    /// the id: the boundary would be all-`0` and this returns one atlas run —
    /// but here the ids are the ARTIST's, so the run split must land on them.
    #[test]
    fn two_objects_form_two_runs() {
        assert_eq!(
            runs(&[7.0, 7.0, 7.0, 9.0, 9.0, 9.0]),
            vec![
                GpuTexRun {
                    texture_id: 7,
                    start: 0,
                    end: 3,
                    blend: 0,
                    sampling: 0,
                },
                GpuTexRun {
                    texture_id: 9,
                    start: 3,
                    end: 6,
                    blend: 0,
                    sampling: 0,
                },
            ]
        );
    }

    /// The run split lands wherever the id changes — three objects interleaved
    /// give three runs at the right offsets, not a merge.
    #[test]
    fn a_run_boundary_lands_where_the_id_changes() {
        assert_eq!(
            runs(&[5.0, 5.0, 8.0, 8.0, 8.0, 3.0]),
            vec![
                GpuTexRun {
                    texture_id: 5,
                    start: 0,
                    end: 2,
                    blend: 0,
                    sampling: 0,
                },
                GpuTexRun {
                    texture_id: 8,
                    start: 2,
                    end: 5,
                    blend: 0,
                    sampling: 0,
                },
                GpuTexRun {
                    texture_id: 3,
                    start: 5,
                    end: 6,
                    blend: 0,
                    sampling: 0,
                },
            ]
        );
    }

    /// **K=1 — the report's common case.** One object → one run over the whole
    /// range, binding that object's texture.
    #[test]
    fn one_object_is_one_run() {
        assert_eq!(
            runs(&[4.0, 4.0, 4.0]),
            vec![GpuTexRun {
                texture_id: 4,
                start: 0,
                end: 3,
                blend: 0,
                sampling: 0,
            }]
        );
    }

    /// An all-atlas object stream (every id `0`) leaves the partition EMPTY —
    /// the renderer's legacy single atlas draw handles it, byte-identical. This
    /// is what keeps a non-object graph on the exact path it shipped on.
    #[test]
    fn all_atlas_is_empty_so_the_legacy_draw_runs() {
        assert!(runs(&[0.0, 0.0, 0.0]).is_empty());
    }

    /// No `texture_id` column at all (a plain point/value stream) → empty,
    /// the legacy atlas draw. The byte-identity guard for every graph that
    /// shipped before objects existed.
    #[test]
    fn no_texture_id_column_is_empty() {
        let mut s = Stream::new(4);
        s.set("P", Column::Vec2(vec![[0.0, 0.0]; 4]));
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &s)], 4, 0, 0, &mut out);
        assert!(out.is_empty());
    }

    /// A `texture_id` column whose length does NOT match the sink count is
    /// ignored (the count-changing cerca should have recused, but the partition
    /// stays honest if it ever slips through — empty, the safe atlas draw,
    /// never a mis-aligned run).
    #[test]
    fn a_length_mismatch_is_ignored() {
        let s = boundary(&[7.0, 9.0, 7.0]); // 3-long column
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &s)], 5, 0, 0, &mut out); // sink count 5
        assert!(out.is_empty());
    }

    /// ⭐⭐⭐ **A MISTURA VIAJA EM CADA RUN** — sem isto o desenho escolhe a pipeline
    /// `0` e o tag que o sink declarou morre entre o cozimento e o pixel (medido em
    /// `ph2d-render`, gate `a_mistura_do_device_chega_ao_pixel`: os cinco modos liam o
    /// byte do `Mix`).
    #[test]
    fn cada_run_carrega_a_mistura_do_sink() {
        let r = runs_com(&[7.0, 7.0, 9.0], 3);
        assert_eq!(r.len(), 2, "a particao por textura nao muda com a mistura");
        assert!(
            r.iter().all(|r| r.blend == 3),
            "todo run deste sink leva o tag dele — leu {:?}",
            r.iter().map(|r| r.blend).collect::<Vec<_>>()
        );
    }

    /// ⛔⛔ **A METADE SUBTIL: com uma mistura que não é `Mix`, a partição deixa de poder
    /// ficar VAZIA.** Vazia é, para o desenho, *«o átlas, em `Mix`»* — o ramo
    /// `runs.is_empty()` do `renderer_draw`, que não tem onde levar um tag. Uma cena de
    /// Motion comum é toda átlas (`texture_id == 0`), logo **este é o caso NORMAL** de um
    /// artista que escolhe `Add` no sink: sem o run explícito ele desenharia em `Mix`.
    #[test]
    fn com_outra_mistura_o_atlas_deixa_de_cair_no_ramo_vazio() {
        // O CONTROLO: em `Mix` os dois casos continuam vazios, byte a byte.
        assert!(runs_com(&[0.0, 0.0, 0.0], 0).is_empty());
        let mut so_p = Stream::new(3);
        so_p.set("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &so_p)], 3, 0, 0, &mut out);
        assert!(out.is_empty(), "sem coluna de textura e em Mix: vazio");

        // E com outra mistura os DOIS emitem um run de corpo inteiro sobre o átlas.
        let esperado = vec![GpuTexRun {
            texture_id: 0,
            start: 0,
            end: 3,
            blend: 1,
            sampling: 0,
        }];
        assert_eq!(
            runs_com(&[0.0, 0.0, 0.0], 1),
            esperado,
            "todo átlas + Add tem de emitir o run explícito"
        );
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &so_p)], 3, 1, 0, &mut out);
        assert_eq!(
            out, esperado,
            "sem coluna de textura + Add tem de emitir o run explícito"
        );
    }

    /// ⭐⭐ **A AMOSTRAGEM do sink viaja em cada run e TAMBÉM tira o átlas do ramo vazio**
    /// (doc 119 §3, W1) — a mesma metade subtil da mistura, com a outra chave: o ramo vazio do
    /// desenho é *«o átlas com o sampler do PROJECTO»*, logo um `Filter = Nearest` numa cena toda
    /// de átlas morria nele. ⚠️ O CONTROLO (`sampling = 0`) continua vazio, byte a byte.
    #[test]
    fn a_amostragem_do_sink_viaja_e_tira_o_atlas_do_ramo_vazio() {
        let s = boundary(&[0.0, 0.0, 0.0]);
        let perto = ph2d_render::RenderInstance::pack_sampling(1, 0);
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &s)], 3, 0, 0, &mut out);
        assert!(out.is_empty(), "CONTROLO: amostragem de fábrica, vazio");
        texture_runs_from_boundary(&[(NodeId(0), &s)], 3, 0, perto, &mut out);
        assert_eq!(
            out,
            vec![GpuTexRun {
                texture_id: 0,
                start: 0,
                end: 3,
                blend: 0,
                sampling: perto,
            }],
            "todo átlas + Nearest tem de emitir o run explícito com a chave"
        );
        let s = boundary(&[7.0, 9.0]);
        let mut out = Vec::new();
        texture_runs_from_boundary(&[(NodeId(0), &s)], 2, 0, perto, &mut out);
        assert!(
            out.iter().all(|r| r.sampling == perto) && out.len() == 2,
            "cada run leva a amostragem do sink: {out:?}"
        );
    }

    /// `v as u32` truncates toward zero, exactly like the CPU lowering and the
    /// WGSL `u32(...)`, so a fractional id (never authored, but honest) lands on
    /// the same integer both sides would.
    #[test]
    fn ids_truncate_like_the_lowerings() {
        assert_eq!(
            runs(&[7.9, 7.1]),
            vec![GpuTexRun {
                texture_id: 7,
                start: 0,
                end: 2,
                blend: 0,
                sampling: 0,
            }]
        );
    }
}
