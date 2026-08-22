//! **A geometria do 9-slice** — a função pura que transforma UM sprite em até **nove quads**.
//!
//! Autoria vive em [`ph2d_ecs::SliceNine`]; emissão vive no extract da shell. Aqui só mora a
//! matemática, e ela é pura de propósito: é a única parte que se pode provar sem GPU, sem ECS e
//! sem janela — e é onde todo erro de 9-slice de facto acontece (canto que estica, borda que
//! espelha ao contrário, miolo deslocado meio pixel).
//!
//! # O modelo
//!
//! As quatro bordas (`left/top/right/bottom`, em **pixels da fonte**) recortam a imagem numa
//! grelha 3×3. Ao desenhar num alvo de `W × H` metros:
//!
//! - os **quatro cantos** mantêm o tamanho intrínseco — é isto que faz um canto arredondado
//!   continuar redondo quando a caixa cresce;
//! - as **quatro bordas** esticam (ou repetem) **num eixo só**;
//! - o **centro** nos dois.
//!
//! ⚠️ **A correspondência V↔Y é lei da casa e não se adivinha:** o quad unitário mapeia
//! `pos.y = +0.5` (cima, no mundo) a `uv.y = 0` (topo da textura) — está escrito e comentado em
//! [`crate::sprite::vertex::QuadVertex::QUAD_STRIP`]. Por isso a **linha 0** desta grelha (a de
//! `v` mínimo) é a que fica em **`+Y`**. Trocar isto compila, passa em quase tudo, e desenha a
//! caixa de cabeça para baixo só quando as bordas de cima e de baixo diferem.
//!
//! # Degenerescência é a prova
//!
//! Com bordas a zero, as colunas ficam `[0, W, 0]` e as linhas `[0, H, 0]`: os oito quads do
//! anel têm área nula e são descartados, e sobra **um** quad de `W × H` com o UV inteiro — o
//! sprite de sempre, byte-idêntico. *Não é coincidência: é o teste de identidade embutido no
//! modelo*, e está afirmado em [`tests::zero_borders_collapse_to_the_plain_sprite`].

use ph2d_ecs::{SliceDrawMode, SliceNine, SliceRegion, SliceTileMode, TileRegionMode};

/// Um dos até nove quads em que um sprite com 9-slice se desenha.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SlicePatch {
    /// Sub-rect da textura, `[u_min, v_min, u_max, v_max]`.
    pub uv: [f32; 4],
    /// Tamanho do quad em metros LOCAIS.
    pub size: [f32; 2],
    /// Deslocamento do CENTRO deste quad em relação ao centro do sprite, em metros locais.
    /// Soma-se ao `anchor` do sprite — que é exatamente o que o shader faz
    /// (`local = anchor + quad_pos * size`).
    pub center_offset: [f32; 2],
    /// `[scale.x, scale.y, offset.x, offset.y]` para o `uv_xform` da instância: `scale > 1`
    /// repete dentro do sub-rect deste quad.
    pub uv_xform: [f32; 4],
    /// Tag de `RepeatMode` que este quad exige (`2` = Enabled/wrap, `3` = Mirror), ou `None`
    /// para deixar o modo resolvido do nó em paz. Só é `Some` quando o quad de facto repete —
    /// um quad que estica não tem opinião sobre wrap.
    pub repeat_tag: Option<u8>,
}

/// Tag de `RepeatMode::Enabled` — o wrap.
const REPEAT_ENABLED: u8 = 2;
/// Tag de `RepeatMode::Mirror`.
const REPEAT_MIRROR: u8 = 3;

/// Quantos quads a grelha 3×3 tem.
pub const PATCH_COUNT: usize = 9;

/// O índice do miolo na grelha 3×3 (linha 1, coluna 1).
pub const CENTRE_INDEX: usize = 4;

/// Divide `uv` / `target` na grelha 3×3 do 9-slice.
///
/// `source_px` são as dimensões em pixels do **rect que `uv` cobre** (já com region/sheet
/// aplicados), e `pixels_per_meter` converte as bordas autoradas em pixels para metros.
///
/// Devolve os quads por índice `linha * 3 + coluna`, com `None` onde o quad não existe — área
/// nula, ou [`TileRegionMode::Blank`], ou o miolo com `fill_center = false`.
///
/// Um `draw_mode` que não seja de nove quads devolve tudo `None`: **a porta é o modo**, e o
/// chamador que perguntar sem olhar o modo recebe «nada a fazer» em vez de uma grelha errada.
pub fn nine_slice_patches(
    uv: [f32; 4],
    source_px: [f32; 2],
    slice: &SliceNine,
    sprite_size: [f32; 2],
    pixels_per_meter: f32,
) -> [Option<SlicePatch>; PATCH_COUNT] {
    let none = [None; PATCH_COUNT];
    if !slice.draw_mode.is_nine() {
        return none;
    }
    let s = slice.sanitized();
    let ppm = if pixels_per_meter.is_finite() && pixels_per_meter > 0.0 {
        pixels_per_meter
    } else {
        return none;
    };
    let (sw, sh) = (source_px[0], source_px[1]);
    if sw <= 0.0 || sh <= 0.0 || !sw.is_finite() || !sh.is_finite() {
        return none;
    }
    let [w, h] = s.effective_size(sprite_size);
    if w <= 0.0 || h <= 0.0 {
        return none;
    }

    // Bordas em metros, encolhidas proporcionalmente quando não cabem no alvo (Godot
    // NinePatchRect faz o mesmo): sem isto, uma caixa mais estreita que as duas bordas
    // desenharia colunas de largura negativa e o miolo pelo avesso.
    let (bl, br) = fit_pair(s.borders[0] / ppm, s.borders[2] / ppm, w);
    let (bt, bb) = fit_pair(s.borders[1] / ppm, s.borders[3] / ppm, h);
    let cols_m = [bl, (w - bl - br).max(0.0), br];
    let rows_m = [bt, (h - bt - bb).max(0.0), bb];

    // Fatias em UV, na fração que as bordas ocupam da FONTE (não do alvo) — é o que mantém o
    // canto com os seus pixels próprios enquanto o alvo cresce.
    let (fl, fr) = fit_pair(s.borders[0] / sw, s.borders[2] / sw, 1.0);
    let (ft, fb) = fit_pair(s.borders[1] / sh, s.borders[3] / sh, 1.0);
    let [u0, v0, u1, v1] = uv;
    let (du, dv) = (u1 - u0, v1 - v0);
    let u_edges = [u0, u0 + du * fl, u1 - du * fr, u1];
    let v_edges = [v0, v0 + dv * ft, v1 - dv * fb, v1];
    // Comprimento intrínseco de cada faixa, em metros: quantos metros UM ladrilho ocupa.
    let cols_src_m = [
        s.borders[0] / ppm,
        (sw - s.borders[0] - s.borders[2]).max(0.0) / ppm,
        s.borders[2] / ppm,
    ];
    let rows_src_m = [
        s.borders[1] / ppm,
        (sh - s.borders[1] - s.borders[3]).max(0.0) / ppm,
        s.borders[3] / ppm,
    ];

    // Cantos em Y: a linha 0 é a de `v` mínimo = o TOPO da textura = `+H/2` no mundo.
    let x_left = -0.5 * w;
    let y_top = 0.5 * h;

    let mut out = none;
    for row in 0..3usize {
        for col in 0..3usize {
            let idx = row * 3 + col;
            let (pw, ph) = (cols_m[col], rows_m[row]);
            if pw <= 0.0 || ph <= 0.0 {
                continue; // faixa de área nula — é assim que o caso degenerado colapsa
            }
            let mode = region_mode_at(&s, col, row);
            let Some(mode) = mode else {
                continue; // Blank, ou o miolo com fill_center = false
            };
            let cx = x_left + cols_m[..col].iter().sum::<f32>() + 0.5 * pw;
            let cy = y_top - rows_m[..row].iter().sum::<f32>() - 0.5 * ph;
            // Repetição por eixo: só faz sentido no eixo em que a faixa de facto estica.
            // Um canto (col e row de borda) nunca repete — o seu tamanho é o intrínseco.
            let tiles_x = tile_count(
                mode,
                s.draw_mode,
                s.tile_mode,
                s.stretch_value,
                pw,
                cols_src_m[col],
                col == 1,
            );
            let tiles_y = tile_count(
                mode,
                s.draw_mode,
                s.tile_mode,
                s.stretch_value,
                ph,
                rows_src_m[row],
                row == 1,
            );
            let repeats = tiles_x > 1.0 || tiles_y > 1.0;
            out[idx] = Some(SlicePatch {
                uv: [
                    u_edges[col],
                    v_edges[row],
                    u_edges[col + 1],
                    v_edges[row + 1],
                ],
                size: [pw, ph],
                center_offset: [cx, cy],
                uv_xform: [tiles_x, tiles_y, 0.0, 0.0],
                repeat_tag: repeats.then_some(match mode {
                    TileRegionMode::Mirror => REPEAT_MIRROR,
                    _ => REPEAT_ENABLED,
                }),
            });
        }
    }
    out
}

/// Encolhe um par de bordas proporcionalmente se a soma não couber em `total`.
fn fit_pair(a: f32, b: f32, total: f32) -> (f32, f32) {
    let sum = a + b;
    if sum > total && sum > 0.0 {
        let k = total / sum;
        (a * k, b * k)
    } else {
        (a, b)
    }
}

/// O modo da célula `(col, row)`, ou `None` quando ela não se desenha.
///
/// O miolo não é uma das oito regiões — ele obedece a `fill_center`, e é isso que faz uma
/// moldura oca.
fn region_mode_at(s: &SliceNine, col: usize, row: usize) -> Option<TileRegionMode> {
    if col == 1 && row == 1 {
        return s.fill_center.then_some(centre_mode(s));
    }
    let region = SliceRegion::ALL
        .iter()
        .copied()
        .find(|r| r.cell() == (col, row))?;
    match s.region_mode(region) {
        TileRegionMode::Blank => None,
        m => Some(m),
    }
}

/// O miolo não tem entrada por-região: em `Tiled` ele repete, em `Sliced` ele estica.
fn centre_mode(s: &SliceNine) -> TileRegionMode {
    match s.draw_mode {
        SliceDrawMode::Tiled => TileRegionMode::Repeat,
        _ => TileRegionMode::Stretch,
    }
}

/// Quantas vezes o pedaço da fonte cabe na faixa alvo, neste eixo.
///
/// `1.0` = não repete (estica). `axis_stretches` diz se ESTE eixo é o que cresce: uma borda
/// lateral cresce em Y e não em X, e repetir no eixo fixo seria desenhar o ladrilho comprimido.
///
/// ⚠️ **`Adaptive` segue o limiar documentado do Unity** (`adaptiveModeThreshold`): quando o
/// ladrilho parcial que sobra é menor que `stretch_value`, ele é descartado e os ladrilhos
/// inteiros esticam para preencher; senão o parcial fica. É por isso que o slider tem unidade —
/// ele é uma **fração de ladrilho**, não um ganho solto.
fn tile_count(
    mode: TileRegionMode,
    draw: SliceDrawMode,
    tile_mode: SliceTileMode,
    stretch_value: f32,
    target_m: f32,
    intrinsic_m: f32,
    axis_stretches: bool,
) -> f32 {
    let repeats = matches!(mode, TileRegionMode::Repeat | TileRegionMode::Mirror)
        || (draw == SliceDrawMode::Tiled && mode == TileRegionMode::Stretch);
    if !repeats || !axis_stretches || intrinsic_m <= 0.0 || target_m <= 0.0 {
        return 1.0;
    }
    let raw = target_m / intrinsic_m;
    match tile_mode {
        SliceTileMode::Continuous => raw,
        SliceTileMode::Adaptive => {
            let whole = raw.floor();
            let frac = raw - whole;
            // Menos de um ladrilho inteiro nao tem "parcial a descartar" — descarta-lo deixaria
            // a faixa VAZIA. Por isso as duas condicoes que devolvem `raw` sao a mesma lei:
            // *o parcial fica*.
            if whole < 1.0 || frac >= stretch_value {
                raw
            } else {
                whole
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [2.0, 2.0], 100.0);
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
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [4.0, 4.0], &s, [4.0, 4.0], 100.0);
        assert!(p[CENTRE_INDEX].is_none());
        assert_eq!(active(&p).len(), 8);
    }

    /// `Blank` apaga a sua região, e só a sua.
    #[test]
    fn a_blank_region_removes_exactly_itself() {
        let mut s = sliced([8.0; 4]);
        s.tile_modes[SliceRegion::Top as usize] = TileRegionMode::Blank;
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.0, 4.0], 100.0);
        assert!(p[1].is_none(), "a regiao Top devia ter sumido");
        assert_eq!(active(&p).len(), 8);
    }

    /// Uma borda lateral repete no eixo em que cresce (Y) e **não** no eixo fixo (X).
    #[test]
    fn an_edge_repeats_only_on_the_axis_it_stretches() {
        let mut s = sliced([8.0, 8.0, 8.0, 8.0]);
        s.tile_modes[SliceRegion::Left as usize] = TileRegionMode::Repeat;
        // fonte 64px, bordas 8: a faixa central vertical tem 48px = 0.48 m. Alvo 4 m de altura.
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.0, 4.0], 100.0);
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
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.0, 4.0], 100.0);
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
        let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.0, 4.0], 100.0);
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
            nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.16, 4.16], 100.0)
                [CENTRE_INDEX]
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
            let p = nine_slice_patches([0.0, 0.0, 1.0, 1.0], px, &s, size, ppm);
            assert_eq!(
                active(&p),
                Vec::<usize>::new(),
                "px={px:?} size={size:?} ppm={ppm}"
            );
        }
        // E o que SAI nunca tem NaN.
        let ok = nine_slice_patches([0.0, 0.0, 1.0, 1.0], [64.0, 64.0], &s, [4.0, 4.0], 100.0);
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
        let p = nine_slice_patches(uv, [64.0, 64.0], &sliced([8.0; 4]), [4.0, 4.0], 100.0);
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
}
