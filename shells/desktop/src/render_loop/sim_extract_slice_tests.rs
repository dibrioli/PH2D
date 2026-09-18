//! Os gates da emissão do 9-slice — irmão do [`super`] pelo tecto de LOC da shell.
//!
//! ⚠️ Eles vivem aqui e não numa crate porque o sujeito é a COSTURA da shell: a geometria pura já
//! tem os gates dela na [`ph2d_render::nine_slice`], e o que falta provar é o que esta casa faz com
//! ela — o fan-out, a herança da instância, e (desde 2026-09-17) a **fracção** que a deformação por
//! ossos consome.

use super::*;
use ph2d_ecs::{SliceDrawMode, SliceNine};

/// A `basis` de uma sprite sem rotação nem escala: `[col0, col1]` = identidade.
const IDENTITY_BASIS: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

/// A fonte dos testes: uma imagem 64×64, sem região e sem grelha.
const SRC_64: SliceSource = SliceSource {
    region: None,
    grid: ph2d_ecs::SpriteGrid::SINGLE,
    src_dims: Some((64, 64)),
};

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
        sub_order: 0,
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
    let grid = ph2d_ecs::SpriteGrid {
        hframes: 4,
        vframes: 2,
        frame: 0,
    };
    assert_eq!(
        sub_rect_source_px(SliceSource {
            region: None,
            grid,
            src_dims: Some((256, 128))
        }),
        Some([64.0, 64.0])
    );
}

/// E é a REGIÃO, quando há região.
#[test]
fn the_source_px_follows_the_region_when_enabled() {
    let region = ph2d_ecs::SpriteRegion::for_atlas([10.0, 10.0, 32.0, 16.0]);
    assert_eq!(
        sub_rect_source_px(SliceSource {
            region: Some(region),
            grid: ph2d_ecs::SpriteGrid::SINGLE,
            src_dims: Some((256, 256))
        }),
        Some([32.0, 16.0])
    );
}

/// Sem 9-slice, ou em `Simple`, não há quads — o caminho de sempre.
#[test]
fn no_component_and_simple_mode_both_mean_the_ordinary_path() {
    assert!(
        patches_for(
            None,
            &spr(),
            SRC_64,
            [0.0, 0.0, 1.0, 1.0],
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
            SRC_64,
            [0.0, 0.0, 1.0, 1.0],
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
            SRC_64,
            [0.0, 0.0, 1.0, 1.0],
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
        flip: [false, false],
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
        SRC_64,
        [0.0, 0.0, 1.0, 1.0],
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
        SRC_64,
        [0.0, 0.0, 1.0, 1.0],
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
    b.flip_uv =
        RenderInstance::pack_flip_flags(true, false, false) | RenderInstance::pack_repeat_bits(1);
    let p = SlicePatch {
        uv: [0.0, 0.0, 1.0, 1.0],
        size: [1.0, 1.0],
        center_offset: [0.0, 0.0],
        uv_xform: [2.0, 1.0, 0.0, 0.0],
        repeat_tag: Some(3), // Mirror
        flip: [false, false],
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

/// ⚠️ **UM SPRITE ESPELHADO ESPELHA A GRELHA INTEIRA, não cada célula no seu lugar.**
///
/// Defeito encontrado na auditoria de fecho (2026-08-22) e **não** reportado por smoke: o
/// `flip_x` do sprite põe um bit na instância, e o shader inverte o `quv` de **cada** quad —
/// o que espelha o conteúdo de cada célula *dentro dela própria* e deixa cada uma no seu
/// sítio. Num sprite normal isso é o espelho certo, porque há um quad só. Com nove, o canto
/// de cima-esquerda fica em cima-à-esquerda com o arco virado ao contrário: a moldura sai
/// partida.
///
/// A cura é geométrica e cabe numa linha: **espelhar as POSIÇÕES**. O conteúdo já vem
/// invertido pelo bit do sprite; negar o deslocamento do centro põe cada célula do outro lado,
/// e o resultado é o sprite inteiro ao espelho — que é o que o utilizador pediu.
#[test]
fn a_flipped_sprite_mirrors_the_whole_grid_not_each_cell_in_place() {
    let s = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [8.0; 4],
        ..SliceNine::INERT
    };
    let patches = patches_for(
        Some(&s),
        &spr(),
        SRC_64,
        [0.0, 0.0, 1.0, 1.0],
        100.0,
        IDENTITY_BASIS,
    )
    .expect("nove quads");
    let plain = base();
    let mut flipped = base();
    flipped.flip_uv = RenderInstance::pack_flip_flags(true, false, false);

    // O canto de CIMA-ESQUERDA da grelha (índice 0).
    let tl = patches[0].expect("o canto TL existe");
    let a = apply_patch(&plain, &tl);
    let b = apply_patch(&flipped, &tl);
    // Sem espelho ele está à esquerda do pivô; com espelho, à direita — mesma distância.
    let da = a.anchor[0] - plain.anchor[0];
    let db = b.anchor[0] - flipped.anchor[0];
    assert!(da < 0.0, "o canto TL nao esta' a' esquerda: {da}");
    assert!(
        (db + da).abs() < 1e-6,
        "o canto nao trocou de lado num sprite espelhado ({da} -> {db}): a grelha ficou no              sitio e so' o conteudo de cada celula inverteu — a moldura sai partida"
    );
    // Em Y nada se mexe: o espelho era só em X.
    assert_eq!(a.anchor[1], b.anchor[1], "espelhou em Y sem ninguem pedir");
    // E o tamanho do canto NÃO muda: espelhar move, não redimensiona.
    assert_eq!(a.size, b.size);
}

/// O mesmo em Y, e os dois ao mesmo tempo — o canto vai para a diagonal oposta.
#[test]
fn flipping_both_axes_sends_the_corner_to_the_opposite_diagonal() {
    let s = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [8.0; 4],
        ..SliceNine::INERT
    };
    let patches = patches_for(
        Some(&s),
        &spr(),
        SRC_64,
        [0.0, 0.0, 1.0, 1.0],
        100.0,
        IDENTITY_BASIS,
    )
    .unwrap();
    let tl = patches[0].unwrap();
    let plain = base();
    let mut both = base();
    both.flip_uv = RenderInstance::pack_flip_flags(true, true, false);
    let a = apply_patch(&plain, &tl);
    let b = apply_patch(&both, &tl);
    let (dax, day) = (a.anchor[0] - plain.anchor[0], a.anchor[1] - plain.anchor[1]);
    let (dbx, dby) = (b.anchor[0] - both.anchor[0], b.anchor[1] - both.anchor[1]);
    assert!(
        (dbx + dax).abs() < 1e-6 && (dby + day).abs() < 1e-6,
        "{dax},{day} -> {dbx},{dby}"
    );
}

/// ⚠️ **O espelho do quad é um XOR, não um OR** — e é isto que a distingue de uma imposição.
///
/// A coluna da direita inverte-se para encostar num ladrilho que fechou invertido
/// (`ph2d_render::nine_slice`, a lei da paridade). Se o SPRITE já estiver espelhado, os dois
/// espelhos cancelam-se: a orientação certa é a original. Com um OR, a borda de um sprite
/// invertido apontaria para o lado errado — e o defeito só apareceria em sprites virados,
/// que é a classe de bug que ninguém encontra a olhar para o código.
#[test]
fn the_patch_flip_toggles_the_sprites_own_mirror_instead_of_forcing_it() {
    let flipped_patch = SlicePatch {
        uv: [0.0, 0.0, 0.25, 1.0],
        size: [1.0, 1.0],
        center_offset: [0.0, 0.0],
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        repeat_tag: None,
        flip: [true, false],
    };
    // Sprite direito: o quad liga o espelho.
    let plain = apply_patch(&base(), &flipped_patch);
    assert_eq!(plain.flip_uv & 1, 1, "o quad nao ligou o espelho em X");
    assert_eq!(plain.flip_uv & 2, 0, "ligou Y sem ninguem pedir");

    // Sprite JÁ espelhado em X: os dois cancelam-se.
    let mut b = base();
    b.flip_uv = RenderInstance::pack_flip_flags(true, false, false);
    let twice = apply_patch(&b, &flipped_patch);
    assert_eq!(
        twice.flip_uv & 1,
        0,
        "dois espelhos seguidos tem de dar a orientacao original — isto foi um OR"
    );

    // Um quad sem espelho não toca em nada, nem sequer nos bits do sprite.
    let inert = SlicePatch {
        flip: [false, false],
        ..flipped_patch
    };
    assert_eq!(apply_patch(&b, &inert).flip_uv, b.flip_uv);
}
/// ⭐⭐⭐ **A FRACÇÃO PUBLICADA RECOMPÕE O SUB-RECT DE CADA PEDAÇO** — a ida e a volta.
///
/// ⚠️ É a costura de que a deformação por ossos depende: ela converte a fracção em pixels da
/// MALHA dela própria, e se a fracção não for exactamente o pedaço, a arte sai do sítio.
///
/// (Mutação: dividir por `du`/`dv` trocados ⇒ RED nas peças não-quadradas.)
#[test]
fn a_fraccao_de_cada_pedaco_recompoe_o_sub_rect_dele() {
    let s = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [8.0, 4.0, 12.0, 6.0],
        ..SliceNine::INERT
    };
    // ⚠️ Uma célula que NÃO é `[0,0,1,1]`: com a célula inteira a fracção e o uv coincidem, e o
    // gate passaria sobre um código que devolvesse o `uv` cru.
    let celula = [0.25, 0.5, 0.5, 1.0];
    let patches = nine_slice_patches(celula, [64.0, 64.0], &s, [4.0, 4.0], 100.0, [1.0, 1.0]);
    let (fontes, n) = sources(celula, &patches);
    let (_, n_inst) = instances(&base(), &patches);
    assert_eq!(n, n_inst, "as fontes e as instancias tem de vir em par");
    assert!(n >= 4, "so' {n} pedacos: a fixtura mede pouco");
    let (du, dv) = (celula[2] - celula[0], celula[3] - celula[1]);
    for (k, q) in patches.iter().flatten().enumerate() {
        let f = fontes[k].frac;
        let volta = [
            celula[0] + f[0] * du,
            celula[1] + f[1] * dv,
            celula[0] + f[2] * du,
            celula[1] + f[3] * dv,
        ];
        for i in 0..4 {
            assert!(
                (volta[i] - q.uv[i]).abs() < 1e-6,
                "o pedaco {k} recompoe {volta:?} contra o uv {:?}",
                q.uv
            );
        }
    }
}

/// ⛔ **UMA CÉLULA DE LADO NULO devolve a fracção INTEIRA**, e não um `NaN`.
///
/// ⚠️ Um `NaN` aqui atravessaria a costura e sairia do outro lado como um rectângulo de corte
/// impossível — a imagem desapareceria sem ninguém saber porquê. A fracção inteira faz a
/// deformação cair no caminho de sempre.
#[test]
fn uma_celula_de_lado_nulo_devolve_a_fraccao_inteira() {
    let s = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [8.0; 4],
        ..SliceNine::INERT
    };
    let patches = nine_slice_patches([0.0; 4], [64.0, 64.0], &s, [4.0, 4.0], 100.0, [1.0, 1.0]);
    let (fontes, n) = sources([0.0, 0.0, 0.0, 0.0], &patches);
    for f in fontes.iter().take(n) {
        assert_eq!(f.frac, [0.0, 0.0, 1.0, 1.0]);
        assert!(f.frac.iter().all(|v| v.is_finite()));
    }
}

/// ⭐⭐ **A EMISSÃO ATTACHA A FONTE NOS NOVE** — a base E os oito extra.
///
/// ⚠️⚠️ **Por `include_str!` e não por comportamento, e a razão é declarada:** o emit precisa de
/// um renderer e de uma janela, logo não é alcançável de um teste — a alternativa a esta régua
/// é *nenhuma*. ⛔ Faltar num dos dois ramos deixa metade do 9-slice sem deformar **em
/// silêncio**, que é a DIRETIVA §2 à letra («faltar uma ponta = clique dropado em silêncio»).
///
/// ⭐ E ela deixa de **compilar** se o irmão mudar de sítio, que é o que a torna honesta.
#[test]
fn a_emissao_attacha_a_fonte_nos_nove() {
    const EMIT: &str = include_str!("sim_extract_emit.rs");
    assert!(
        EMIT.contains("sim_extract_slice::sources(atlas_uv, &patches)"),
        "o emit deixou de pedir as fontes"
    );
    assert!(
        EMIT.contains("builder.insert(fontes[0]);"),
        "a instancia BASE do 9-slice ficou sem a fonte"
    );
    assert!(
        EMIT.contains("insts.iter().zip(&fontes)"),
        "os oito quads extra ficaram sem a fonte"
    );
}
