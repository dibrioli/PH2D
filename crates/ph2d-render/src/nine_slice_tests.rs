//! Os testes da geometria do 9-slice — irmão do [`super::nine_slice`] por CAP de ficheiro (700).
//!
//! ⚠️ **O corte é dos TESTES, não da lei.** A alternativa era alargar a allowlist, e *as
//! tolerâncias só encolhem*. Vinculado por `#[path]`, portanto continua a ver tudo o que é
//! privado no módulo — nenhuma função teve de virar `pub` para caber aqui.

use super::*;

fn sliced(borders: [f32; 4]) -> SliceNine {
    SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders,
        ..SliceNine::INERT
    }
}

fn active(p: &[Option<SlicePatch>; PATCH_COUNT]) -> Vec<usize> {
    (0..PATCH_COUNT).filter(|i| p[*i].is_some()).collect()
}

/// ⚠️ **A prova de identidade.** Bordas a zero ⇒ um quad só, do tamanho do sprite, com o UV
/// inteiro. Se isto quebrar, todo sprite com o componente anexado muda de aspeto.
#[test]
fn zero_borders_collapse_to_the_plain_sprite() {
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &sliced([0.0; 4]),
        [2.0, 2.0],
        100.0,
        [1.0, 1.0],
    );
    assert_eq!(active(&p), vec![CENTRE_INDEX], "sobrou mais do que o miolo");
    let c = p[CENTRE_INDEX].unwrap();
    assert_eq!(c.uv, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(c.size, [2.0, 2.0]);
    assert_eq!(c.center_offset, [0.0, 0.0]);
    assert_eq!(c.uv_xform, [1.0, 1.0, 0.0, 0.0]);
    assert_eq!(c.repeat_tag, None);
}

/// O modo é a porta: `Simple` não produz grelha nenhuma.
#[test]
fn simple_mode_produces_no_patches() {
    let s = SliceNine {
        borders: [8.0; 4],
        ..SliceNine::INERT
    };
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [2.0, 2.0],
        100.0,
        [1.0, 1.0],
    );
    assert_eq!(active(&p), Vec::<usize>::new());
}

/// Os nove quads existem, **ladrilham o alvo sem sobrepor nem deixar buraco**, e a soma das
/// larguras/alturas é exatamente o alvo.
#[test]
fn the_nine_patches_tile_the_target_exactly() {
    let w = 5.0;
    let h = 3.0;
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &sliced([8.0, 16.0, 8.0, 4.0]),
        [w, h],
        100.0,
        [1.0, 1.0],
    );
    assert_eq!(active(&p).len(), 9);
    // Larguras da linha do meio somam W; alturas da coluna do meio somam H.
    let row_w: f32 = (0..3).map(|c| p[3 + c].unwrap().size[0]).sum();
    let col_h: f32 = (0..3).map(|r| p[r * 3 + 1].unwrap().size[1]).sum();
    assert!(
        (row_w - w).abs() < 1e-5,
        "as colunas somam {row_w}, nao {w}"
    );
    assert!((col_h - h).abs() < 1e-5, "as linhas somam {col_h}, nao {h}");
    // Bordas coladas: o direito de um quad é o esquerdo do seguinte.
    for r in 0..3 {
        for c in 0..2 {
            let a = p[r * 3 + c].unwrap();
            let b = p[r * 3 + c + 1].unwrap();
            let a_right = a.center_offset[0] + 0.5 * a.size[0];
            let b_left = b.center_offset[0] - 0.5 * b.size[0];
            assert!(
                (a_right - b_left).abs() < 1e-5,
                "buraco/sobreposicao em ({r},{c})"
            );
            assert!(
                (a.uv[2] - b.uv[0]).abs() < 1e-6,
                "UV descolado em ({r},{c})"
            );
        }
    }
    // O conjunto cobre exatamente [-W/2, W/2] × [-H/2, H/2].
    let left = p[0].unwrap().center_offset[0] - 0.5 * p[0].unwrap().size[0];
    let right = p[2].unwrap().center_offset[0] + 0.5 * p[2].unwrap().size[0];
    assert!((left + 0.5 * w).abs() < 1e-5 && (right - 0.5 * w).abs() < 1e-5);
}

/// ⚠️ **O DEFEITO QUE O SMOKE DO ENIO APANHOU (2026-08-22): a sprite ESCALADA.**
///
/// O gizmo de redimensionar escreve `Transform.scale`, não `Sprite::size`, e a `basis`
/// multiplica os nove quads. Sem esta lei os cantos esticavam junto — *exatamente o mesmo
/// que não ter 9-slice*. O canto tem de sair com o MESMO tamanho de mundo, escalado ou não.
#[test]
fn a_scaled_sprite_keeps_its_corners_at_the_intrinsic_size() {
    let s = sliced([16.0; 4]);
    // Sem escala: o canto mede 16 px / 100 ppm = 0,16 m de mundo.
    let plain = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [2.0, 2.0],
        100.0,
        [1.0, 1.0],
    );
    let corner_world = |p: &[Option<SlicePatch>; PATCH_COUNT], sx: f32, sy: f32| {
        let c = p[0].expect("o canto TL tem de existir");
        [c.size[0] * sx, c.size[1] * sy]
    };
    let a = corner_world(&plain, 1.0, 1.0);
    assert!((a[0] - 0.16).abs() < 1e-5, "canto sem escala deu {a:?}");

    // A MESMA sprite, esticada 4x em X e 1,5x em Y (o gesto do gizmo).
    let scaled = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [2.0, 2.0],
        100.0,
        [4.0, 1.5],
    );
    let b = corner_world(&scaled, 4.0, 1.5);
    assert!(
        (b[0] - a[0]).abs() < 1e-5 && (b[1] - a[1]).abs() < 1e-5,
        "o canto mudou de tamanho com a escala: {a:?} -> {b:?}. Os cantos esticaram junto, \
             que e' o mesmo que nao ter 9-slice"
    );
    // E o MIOLO, esse, cresce: e' ele que absorve a esticadela.
    let mid_plain = plain[CENTRE_INDEX].unwrap().size[0] * 1.0;
    let mid_scaled = scaled[CENTRE_INDEX].unwrap().size[0] * 4.0;
    assert!(
        mid_scaled > mid_plain * 3.0,
        "o miolo tinha de absorver a esticadela: {mid_plain} -> {mid_scaled}"
    );
}

/// Uma escala impossível (zero, NaN) não pode dividir por zero e envenenar o buffer.
#[test]
fn an_impossible_scale_falls_back_to_one_instead_of_dividing_by_zero() {
    let s = sliced([8.0; 4]);
    for scale in [[0.0, 1.0], [1.0, f32::NAN], [f32::INFINITY, 1.0]] {
        let p = nine_slice_patches(
            [0.0, 0.0, 1.0, 1.0],
            [64.0, 64.0],
            &s,
            [4.0, 4.0],
            100.0,
            scale,
        );
        for patch in p.iter().flatten() {
            for v in patch.size.iter().chain(patch.center_offset.iter()) {
                assert!(v.is_finite(), "escala {scale:?} produziu {patch:?}");
            }
        }
    }
}

/// ⚠️ **A linha 0 é o TOPO** — `v` mínimo em `+Y`. É a lei do `QUAD_STRIP`, e uma troca aqui
/// só se vê quando as bordas de cima e de baixo diferem, que é o caso deste teste.
#[test]
fn row_zero_is_the_top_both_in_uv_and_in_world_y() {
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &sliced([8.0, 32.0, 8.0, 4.0]),
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    let top = p[1].unwrap();
    let bottom = p[7].unwrap();
    assert!(top.uv[1] < bottom.uv[1], "a linha 0 tem de ter o v MENOR");
    assert!(
        top.center_offset[1] > bottom.center_offset[1],
        "a linha 0 tem de estar em +Y"
    );
    // A borda de topo tem 32 px e a de baixo 4 px: a assimetria prova que não há troca.
    assert!(
        top.size[1] > bottom.size[1],
        "a borda grossa (topo, 32px) saiu mais fina que a fina (baixo, 4px) — os eixos estao trocados"
    );
}

/// Um alvo mais estreito que as duas bordas encolhe-as em proporção, em vez de produzir
/// larguras negativas.
#[test]
fn borders_that_do_not_fit_shrink_instead_of_going_negative() {
    // 40 px + 40 px de borda = 0.8 m, num alvo de 0.4 m.
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &sliced([40.0, 0.0, 40.0, 0.0]),
        [0.4, 1.0],
        100.0,
        [1.0, 1.0],
    );
    let l = p[3].unwrap();
    let r = p[5].unwrap();
    assert!(
        (l.size[0] - 0.2).abs() < 1e-5,
        "a borda esquerda deu {}",
        l.size[0]
    );
    assert!((r.size[0] - 0.2).abs() < 1e-5);
    assert!(
        p[4].is_none(),
        "o miolo tem largura zero e nao devia existir"
    );
    for patch in p.iter().flatten() {
        assert!(
            patch.size[0] > 0.0 && patch.size[1] > 0.0,
            "quad de area nao-positiva emitido"
        );
    }
}

/// `fill_center = false` faz a moldura oca — e não mexe nos outros oito.
#[test]
fn an_empty_centre_is_a_hollow_frame() {
    let s = SliceNine {
        fill_center: false,
        ..sliced([8.0; 4])
    };
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [4.0, 4.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    assert!(p[CENTRE_INDEX].is_none());
    assert_eq!(active(&p).len(), 8);
}

/// `Blank` apaga a sua região, e só a sua.
#[test]
fn a_blank_region_removes_exactly_itself() {
    let mut s = sliced([8.0; 4]);
    s.tile_modes[SliceRegion::Top as usize] = TileRegionMode::Blank;
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    assert!(p[1].is_none(), "a regiao Top devia ter sumido");
    assert_eq!(active(&p).len(), 8);
}

/// Uma borda lateral repete no eixo em que cresce (Y) e **não** no eixo fixo (X).
#[test]
fn an_edge_repeats_only_on_the_axis_it_stretches() {
    let mut s = sliced([8.0, 8.0, 8.0, 8.0]);
    s.tile_modes[SliceRegion::Left as usize] = TileRegionMode::Repeat;
    // fonte 64px, bordas 8: a faixa central vertical tem 48px = 0.48 m. Alvo 4 m de altura.
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    let left = p[3].unwrap();
    assert_eq!(
        left.uv_xform[0], 1.0,
        "a lateral nao pode repetir em X — X e' o eixo fixo"
    );
    assert!(left.uv_xform[1] > 1.0, "a lateral tem de repetir em Y");
    assert_eq!(left.repeat_tag, Some(REPEAT_ENABLED));
}

/// `Mirror` pede a tag de espelho, não a de wrap.
#[test]
fn mirror_asks_for_the_mirror_wrap() {
    let mut s = sliced([8.0; 4]);
    s.tile_modes[SliceRegion::Left as usize] = TileRegionMode::Mirror;
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    assert_eq!(p[3].unwrap().repeat_tag, Some(REPEAT_MIRROR));
}

/// Um quad que ESTICA não tem opinião sobre wrap — deixa o modo do nó em paz.
#[test]
fn a_stretching_patch_does_not_touch_the_nodes_wrap_mode() {
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &sliced([8.0; 4]),
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    for patch in p.iter().flatten() {
        assert_eq!(patch.repeat_tag, None, "um Sliced puro nao repete nada");
        assert_eq!(patch.uv_xform, [1.0, 1.0, 0.0, 0.0]);
    }
}

/// Em `Tiled` o miolo repete nos dois eixos sem ninguém o marcar por-região.
#[test]
fn tiled_mode_repeats_the_centre_on_both_axes() {
    let s = SliceNine {
        draw_mode: SliceDrawMode::Tiled,
        borders: [8.0; 4],
        ..SliceNine::INERT
    };
    let p = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    let c = p[CENTRE_INDEX].unwrap();
    assert!(c.uv_xform[0] > 1.0 && c.uv_xform[1] > 1.0);
    assert_eq!(c.repeat_tag, Some(REPEAT_ENABLED));
}

/// `Adaptive` descarta o ladrilho parcial quando ele é menor que o limiar, e mantém-no
/// quando não é. É o limiar documentado do Unity, e o slider é uma **fração de ladrilho**.
#[test]
fn adaptive_drops_a_partial_tile_below_the_threshold() {
    // Faixa central da FONTE = 48 px = 0.48 m. O alvo tem de deixar um parcial de verdade:
    // com W = 4.16 m as bordas comem 0.16 e sobram 4.0 ⇒ raw = 8.333… (parcial de 0.333).
    // ⚠️ O primeiro fixture que escrevi usava W = 4.0, onde a divisão dá 8 EXATO — um
    // fixture que não continha o fenómeno que o teste dizia medir.
    let mk = |stretch: f32| {
        let s = SliceNine {
            draw_mode: SliceDrawMode::Tiled,
            borders: [8.0; 4],
            tile_mode: SliceTileMode::Adaptive,
            stretch_value: stretch,
            ..SliceNine::INERT
        };
        nine_slice_patches(
            [0.0, 0.0, 1.0, 1.0],
            [64.0, 64.0],
            &s,
            [4.16, 4.16],
            100.0,
            [1.0, 1.0],
        )[CENTRE_INDEX]
            .unwrap()
            .uv_xform[0]
    };
    // limiar 0.1 < parcial 0.333 ⇒ o parcial FICA (conta fracionária).
    assert!(
        (mk(0.1).fract() - 0.3333).abs() < 1e-2,
        "o parcial devia ter ficado"
    );
    // limiar 0.9 > parcial 0.333 ⇒ o parcial cai, sobram 8 ladrilhos inteiros.
    assert_eq!(mk(0.9), 8.0, "o parcial devia ter sido descartado");
}

/// Entradas impossíveis não produzem grelha — nunca um `NaN` que envenena o buffer da GPU.
#[test]
fn impossible_inputs_produce_nothing_rather_than_nan() {
    let s = sliced([8.0; 4]);
    for (px, size, ppm) in [
        ([0.0, 64.0], [2.0, 2.0], 100.0),
        ([64.0, 64.0], [0.0, 2.0], 100.0),
        ([64.0, 64.0], [2.0, 2.0], 0.0),
        ([64.0, 64.0], [2.0, 2.0], f32::NAN),
    ] {
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], px, &s, size, ppm, [1.0, 1.0]);
        assert_eq!(
            active(&p),
            Vec::<usize>::new(),
            "px={px:?} size={size:?} ppm={ppm}"
        );
    }
    // E o que SAI nunca tem NaN.
    let ok = nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        &s,
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    for patch in ok.iter().flatten() {
        for v in patch
            .uv
            .iter()
            .chain(patch.size.iter())
            .chain(patch.center_offset.iter())
        {
            assert!(v.is_finite(), "valor nao-finito emitido: {patch:?}");
        }
    }
}

/// O sub-rect de origem é respeitado: um sprite que já vem recortado (region/sheet) fatia
/// DENTRO do seu rect, sem nunca sair dele para o atlas do vizinho.
#[test]
fn the_grid_stays_inside_the_incoming_sub_rect() {
    let uv = [0.25, 0.5, 0.5, 0.75];
    let p = nine_slice_patches(
        uv,
        [64.0, 64.0],
        &sliced([8.0; 4]),
        [4.0, 4.0],
        100.0,
        [1.0, 1.0],
    );
    for patch in p.iter().flatten() {
        assert!(
            patch.uv[0] >= uv[0] - 1e-6 && patch.uv[2] <= uv[2] + 1e-6,
            "vazou em U: {patch:?}"
        );
        assert!(
            patch.uv[1] >= uv[1] - 1e-6 && patch.uv[3] <= uv[3] + 1e-6,
            "vazou em V: {patch:?}"
        );
    }
}
