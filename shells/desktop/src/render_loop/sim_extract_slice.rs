//! **A emissão do 9-slice** — irmão do [`super::sim_extract`], que está no tecto de LOC.
//!
//! O extract emite UMA `RenderInstance` por sprite. Um sprite com 9-slice emite até **nove**, e
//! este módulo é a costura entre a geometria pura ([`ph2d_render::nine_slice`]) e a instância
//! que a GPU consome.
//!
//! # Os nove partilham o `SimRef`, e isso é a decisão
//!
//! Cada quad é uma entidade de PRESENTE própria, mas todas carregam o **mesmo** `SimRef`. É isso
//! que faz o passe pós-caminhada — o que carimba `z_order` e o agrupamento de clip — servir os
//! nove de uma vez, sem saber que existe 9-slice: ele já resolve *por entidade de simulação*.
//! Dar-lhes `SimRef` diferentes obrigaria esse passe a aprender uma segunda regra.
//!
//! ⚠️ Os oito extra levam [`SlicePatchMirror`] para que o HUD não os conte como entidades. A
//! contagem de INSTÂNCIAS sobe de propósito — nove quads são nove quads, e essa é a verdade que
//! um contador de desenho deve dizer.

use ph2d_render::nine_slice::{PATCH_COUNT, SlicePatch, nine_slice_patches};
use ph2d_render::{RenderInstance, Sprite};

/// Dimensões, em pixels da fonte, do rect que o `atlas_uv` **já recortado** cobre.
///
/// ⚠️ Tem de ser o rect FINAL (região aplicada, célula da folha aplicada), não a imagem inteira:
/// as bordas do 9-slice são medidas na imagem que o artista vê no Inspector. Medir na textura
/// completa faria a borda de um sprite numa folha 4×2 sair **oito vezes** maior do que se pediu.
pub(super) fn sub_rect_source_px(spr: &Sprite, src_dims: Option<(u32, u32)>) -> Option<[f32; 2]> {
    let (sw, sh) = src_dims?;
    let (mut w, mut h) = (sw as f32, sh as f32);
    if spr.region_enabled && spr.region_rect[2] > 0.0 && spr.region_rect[3] > 0.0 {
        w = spr.region_rect[2];
        h = spr.region_rect[3];
    }
    let hf = spr.hframes.max(1) as f32;
    let vf = spr.vframes.max(1) as f32;
    let (w, h) = (w / hf, h / vf);
    (w > 0.0 && h > 0.0).then_some([w, h])
}

/// Os quads deste sprite, ou `None` quando ele não é um 9-slice desenhável — e aí o chamador
/// segue o caminho de sempre, byte-idêntico.
pub(super) fn patches_for(
    slice: Option<&ph2d_ecs::SliceNine>,
    spr: &Sprite,
    atlas_uv: [f32; 4],
    src_dims: Option<(u32, u32)>,
    pixels_per_meter: f32,
    basis: [f32; 4],
) -> Option<[Option<SlicePatch>; PATCH_COUNT]> {
    let slice = slice?;
    if !slice.draw_mode.is_nine() {
        return None;
    }
    let src_px = sub_rect_source_px(spr, src_dims)?;
    let patches = nine_slice_patches(
        atlas_uv,
        src_px,
        slice,
        spr.size,
        pixels_per_meter,
        scale_of(basis),
    );
    // Um 9-slice que não produziu quad nenhum (bordas maiores que o alvo nos dois eixos, tudo
    // Blank) NÃO pode devolver `Some([None; 9])`: isso faria o sprite desaparecer em silêncio.
    // Devolver `None` fá-lo cair no caminho normal — visível, e o artista vê o que fez.
    patches.iter().any(Option::is_some).then_some(patches)
}

/// A escala do mundo, tirada da `basis` 2×2 (colunas `[col0, col1]`).
///
/// ⚠️ **É o COMPRIMENTO de cada coluna**, não o elemento da diagonal: sob rotação a diagonal
/// deixa de ser a escala (uma rotação de 90° põe zeros lá) enquanto o comprimento da coluna se
/// mantém. Foi por ignorar a escala inteira que os cantos esticavam (smoke do Enio, 2026-08-22).
pub(super) fn scale_of(basis: [f32; 4]) -> [f32; 2] {
    [basis[0].hypot(basis[1]), basis[2].hypot(basis[3])]
}

/// A instância de UM quad, derivada da instância base do sprite.
///
/// Herda tudo — tinta, opacidade, base, ordenação, amostragem — e substitui só o que o quad
/// muda: o sub-rect, o tamanho, o centro e a repetição.
pub(super) fn apply_patch(base: &RenderInstance, p: &SlicePatch) -> RenderInstance {
    let mut ri = *base;
    ri.atlas_uv = p.uv;
    ri.size = p.size;
    // O `anchor` É o centro do quad em metros locais (`local = anchor + quad_pos * size` no
    // shader), por isso deslocar o quad é somar — e somar ao anchor do sprite preserva o pivô
    // autorado (centered/offset) por baixo do 9-slice.
    ri.anchor = [
        base.anchor[0] + p.center_offset[0],
        base.anchor[1] + p.center_offset[1],
    ];
    ri.uv_xform = p.uv_xform;
    if let Some(tag) = p.repeat_tag {
        // Limpa os bits de repeat do nó antes de pôr os do quad: um OR simples deixaria o modo
        // herdado misturado com o pedido, e `Disabled | Mirror` não é nenhum dos dois.
        let mask = !(0b11u32 << RenderInstance::REPEAT_SHIFT);
        ri.flip_uv = (ri.flip_uv & mask) | RenderInstance::pack_repeat_bits(tag);
    }
    ri
}

/// As instâncias deste sprite, em ordem: a `[0]` vai para o espelho principal da entidade e as
/// restantes nascem como quads extra.
///
/// ⚠️ **Existe como função PURA de propósito.** A alternativa era fazer o fan-out inline no
/// extract, onde ele é inalcançável por teste nenhum: o `ph2d-host-desktop` é um binário, o
/// `run()` precisa de um renderer, e a costura ficaria provada só por inspeção — que é
/// exatamente a classe de erro que a DIRETIVA §2 descreve («faltar uma ponta = clique dropado em
/// silêncio, não erro de compilação»). Devolve um array fixo + contagem: zero alocação (HR-3).
pub(super) fn instances(
    base: &RenderInstance,
    patches: &[Option<SlicePatch>; PATCH_COUNT],
) -> ([RenderInstance; PATCH_COUNT], usize) {
    let mut out = [*base; PATCH_COUNT];
    let mut n = 0;
    for q in patches.iter().flatten() {
        out[n] = apply_patch(base, q);
        n += 1;
    }
    (out, n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{SliceDrawMode, SliceNine};

    /// A `basis` de uma sprite sem rotação nem escala: `[col0, col1]` = identidade.
    const IDENTITY_BASIS: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    fn spr() -> Sprite {
        Sprite::atlas(0, [2.0, 2.0], [1.0; 4])
    }

    fn base() -> RenderInstance {
        RenderInstance {
            world_pos: [0.0, 0.0],
            size: [2.0, 2.0],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0; 4],
            basis: [1.0, 0.0, 0.0, 1.0],
            premultiplied: 0.0,
            anchor: [0.5, -0.25],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            texture_id: 0,
            z_order: 7,
            sampling: RenderInstance::SAMPLING_DEFAULT,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
        }
    }

    /// ⚠️ **A escala sai do COMPRIMENTO da coluna, não da diagonal.**
    ///
    /// Uma rotação de 90° põe zeros na diagonal da `basis`; ler a diagonal daria escala 0 e o
    /// 9-slice dividiria por ela. O comprimento da coluna sobrevive à rotação — que é o que se
    /// quer: rodar uma caixa não muda o tamanho do canto.
    #[test]
    fn the_scale_comes_from_the_column_length_not_the_diagonal() {
        assert_eq!(scale_of(IDENTITY_BASIS), [1.0, 1.0]);
        // Escala pura 3x / 2x.
        assert_eq!(scale_of([3.0, 0.0, 0.0, 2.0]), [3.0, 2.0]);
        // Rotação pura de 90°: a diagonal é [0, 0], mas a escala continua unitária.
        let r = scale_of([0.0, 1.0, -1.0, 0.0]);
        assert!(
            (r[0] - 1.0).abs() < 1e-6 && (r[1] - 1.0).abs() < 1e-6,
            "rodar mudou a escala: {r:?} — a diagonal foi lida no lugar da coluna"
        );
        // Rotação de 90° COM escala 4x em X: o comprimento da coluna 0 tem de ser 4.
        let rs = scale_of([0.0, 4.0, -2.0, 0.0]);
        assert!(
            (rs[0] - 4.0).abs() < 1e-6 && (rs[1] - 2.0).abs() < 1e-6,
            "deu {rs:?}"
        );
    }

    /// A medida das bordas é tirada da CÉLULA, não da folha inteira.
    #[test]
    fn the_source_px_is_the_cell_not_the_whole_sheet() {
        let mut s = spr();
        s.hframes = 4;
        s.vframes = 2;
        assert_eq!(sub_rect_source_px(&s, Some((256, 128))), Some([64.0, 64.0]));
    }

    /// E é a REGIÃO, quando há região.
    #[test]
    fn the_source_px_follows_the_region_when_enabled() {
        let mut s = spr();
        s.region_enabled = true;
        s.region_rect = [10.0, 10.0, 32.0, 16.0];
        assert_eq!(sub_rect_source_px(&s, Some((256, 256))), Some([32.0, 16.0]));
    }

    /// Sem 9-slice, ou em `Simple`, não há quads — o caminho de sempre.
    #[test]
    fn no_component_and_simple_mode_both_mean_the_ordinary_path() {
        assert!(
            patches_for(
                None,
                &spr(),
                [0.0, 0.0, 1.0, 1.0],
                Some((64, 64)),
                100.0,
                IDENTITY_BASIS,
            )
            .is_none()
        );
        let inert = SliceNine::INERT;
        assert!(
            patches_for(
                Some(&inert),
                &spr(),
                [0.0, 0.0, 1.0, 1.0],
                Some((64, 64)),
                100.0,
                IDENTITY_BASIS,
            )
            .is_none()
        );
    }

    /// ⚠️ Um 9-slice que não produz quad nenhum cai no caminho normal em vez de fazer o sprite
    /// **desaparecer**. Um sprite invisível por causa de um número de borda é indistinguível de
    /// um bug de renderização.
    #[test]
    fn a_slice_that_yields_nothing_falls_back_instead_of_vanishing() {
        let all_blank = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [8.0; 4],
            fill_center: false,
            tile_modes: [ph2d_ecs::TileRegionMode::Blank; 8],
            ..SliceNine::INERT
        };
        assert!(
            patches_for(
                Some(&all_blank),
                &spr(),
                [0.0, 0.0, 1.0, 1.0],
                Some((64, 64)),
                100.0,
                IDENTITY_BASIS,
            )
            .is_none()
        );
    }

    /// O quad herda tudo do sprite e só muda o que lhe pertence — e o `anchor` SOMA, para não
    /// atropelar o pivô autorado.
    #[test]
    fn a_patch_inherits_everything_and_only_moves_what_is_its_own() {
        let b = base();
        let p = SlicePatch {
            uv: [0.1, 0.2, 0.3, 0.4],
            size: [0.5, 0.25],
            center_offset: [-1.0, 2.0],
            uv_xform: [3.0, 1.0, 0.0, 0.0],
            repeat_tag: None,
        };
        let ri = apply_patch(&b, &p);
        assert_eq!(ri.atlas_uv, p.uv);
        assert_eq!(ri.size, p.size);
        assert_eq!(
            ri.anchor,
            [-0.5, 1.75],
            "o anchor tem de SOMAR ao pivo autorado"
        );
        assert_eq!(ri.uv_xform, p.uv_xform);
        // Herdado, intacto.
        assert_eq!(ri.tint, b.tint);
        assert_eq!(ri.z_order, b.z_order);
        assert_eq!(ri.basis, b.basis);
        assert_eq!(ri.opacity, b.opacity);
        assert_eq!(
            ri.flip_uv, b.flip_uv,
            "sem repeat_tag os bits do no ficam em paz"
        );
    }

    /// **O FAN-OUT.** Um `Sliced` de bordas iguais produz NOVE instâncias, e todas partilham o
    /// que a entidade tem de partilhar (ordenação, base, tinta) enquanto ocupam sítios distintos.
    #[test]
    fn a_sliced_sprite_fans_out_into_nine_instances() {
        let s = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [8.0; 4],
            ..SliceNine::INERT
        };
        let patches = patches_for(
            Some(&s),
            &spr(),
            [0.0, 0.0, 1.0, 1.0],
            Some((64, 64)),
            100.0,
            IDENTITY_BASIS,
        )
        .expect("um Sliced com bordas tem de produzir quads");
        let b = base();
        let (insts, n) = instances(&b, &patches);
        assert_eq!(n, 9, "sairam {n} quads, nao nove");
        for ri in insts.iter().take(n) {
            assert_eq!(ri.z_order, b.z_order, "um quad saiu com outra ordenacao");
            assert_eq!(ri.basis, b.basis);
            assert_eq!(ri.tint, b.tint);
        }
        // Nove sítios distintos: nenhum quad em cima de outro.
        let mut seen: Vec<[u32; 2]> = insts
            .iter()
            .take(n)
            .map(|r| [r.anchor[0].to_bits(), r.anchor[1].to_bits()])
            .collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            9,
            "dois quads no MESMO sitio — o fan-out perdeu o deslocamento"
        );
    }

    /// A moldura oca emite oito, e nenhum deles é o miolo.
    #[test]
    fn a_hollow_frame_emits_eight() {
        let s = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [8.0; 4],
            fill_center: false,
            ..SliceNine::INERT
        };
        let patches = patches_for(
            Some(&s),
            &spr(),
            [0.0, 0.0, 1.0, 1.0],
            Some((64, 64)),
            100.0,
            IDENTITY_BASIS,
        )
        .unwrap();
        let (_, n) = instances(&base(), &patches);
        assert_eq!(n, 8);
    }

    /// ⚠️ Os bits de repeat do nó são **substituídos**, não OR-ados: `Disabled | Mirror` não é
    /// nenhum dos dois.
    #[test]
    fn the_patch_replaces_the_nodes_wrap_bits_instead_of_or_ing_them() {
        let mut b = base();
        // Nó em `Disabled` (tag 1) + flip_x ligado, para provar que os outros bits sobrevivem.
        b.flip_uv = RenderInstance::pack_flip_flags(true, false, false)
            | RenderInstance::pack_repeat_bits(1);
        let p = SlicePatch {
            uv: [0.0, 0.0, 1.0, 1.0],
            size: [1.0, 1.0],
            center_offset: [0.0, 0.0],
            uv_xform: [2.0, 1.0, 0.0, 0.0],
            repeat_tag: Some(3), // Mirror
        };
        let ri = apply_patch(&b, &p);
        let bits = (ri.flip_uv >> RenderInstance::REPEAT_SHIFT) & 0b11;
        assert_eq!(bits, 3, "o quad pediu Mirror e saiu {bits}");
        assert_eq!(
            ri.flip_uv & 1,
            1,
            "o flip_x do sprite foi apagado pelo quad"
        );
    }
}
