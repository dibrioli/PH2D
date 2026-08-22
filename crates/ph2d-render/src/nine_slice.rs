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

use bevy_ecs::component::Component;
use ph2d_ecs::{
    PresentComponent, SliceDrawMode, SliceNine, SliceRegion, SliceTileMode, TileRegionMode,
};

/// Marca uma instância que é um **quad EXTRA** de um 9-slice — os oito do anel, quando o
/// primeiro já foi para o espelho principal da entidade.
///
/// ⚠️ **Existe por uma razão só, e ela é de honestidade de contagem.** Os nove quads partilham o
/// mesmo `SimRef` (é isso que faz o `z_order` e o agrupamento de clip serem carimbados uma vez
/// para todos), mas o HUD conta espelhos de `SimRef` para dizer *«quantas entidades há»*. Sem
/// esta marca, ligar 9-slice num sprite fá-lo-ia contar como nove — um número que passa a mentir
/// exatamente quando a cena fica interessante.
///
/// Componente de PRESENTE: reconstruído a cada quadro, nunca serializado, e por isso **fora do
/// `ComponentRegistry`** (não mexe nos dois contadores).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SlicePatchMirror;

impl PresentComponent for SlicePatchMirror {}

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
    /// `[flip_u, flip_v]` — este quad desenha o seu sub-rect **espelhado**.
    ///
    /// ⚠️ Só a coluna da direita e a linha de baixo o usam, e só quando a faixa vizinha acaba
    /// numa cópia invertida (ver [`nine_slice_patches`]). É uma correção de **continuidade**,
    /// não uma opção: sem ela a borda encosta na metade errada do ladrilho.
    pub flip: [bool; 2],
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
    world_scale: [f32; 2],
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
    let [w_local, h_local] = s.effective_size(sprite_size);
    // ⚠️ **A ESCALA DO MUNDO entra na conta, e é o defeito que o smoke do Enio apanhou
    // (2026-08-22).** O gizmo de redimensionar escreve `Transform.scale`, **não**
    // `Sprite::size` — e o shader multiplica os nove quads pela `basis`. Montar a grelha só com
    // o tamanho local fazia os nove esticarem juntos: **exatamente o mesmo que não ter
    // 9-slice**, com os cantos deformados. O smoke não o via porque criava a sprite já no
    // tamanho final, com escala 1 — *o fixture não continha o fenómeno que dizia medir*.
    //
    // A cura: montar a grelha no tamanho de MUNDO e devolver os quads em LOCAL (a dividir pela
    // escala), para a `basis` os voltar a multiplicar e o canto cair no tamanho intrínseco.
    let axis = |v: f32| {
        if v.is_finite() && v.abs() > f32::EPSILON {
            v.abs()
        } else {
            1.0
        }
    };
    let (sx, sy) = (axis(world_scale[0]), axis(world_scale[1]));
    let (w, h) = (w_local * sx, h_local * sy);
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

    // ⚠️ **A PARIDADE da faixa vizinha decide qual borda combina** (smoke do Enio, 2026-08-22).
    //
    // O espelho é uma onda triangular sobre `quv * n`: o ladrilho 0 sai direito, o 1 invertido, o
    // 2 direito… A faixa **começa** sempre em `u_min`, e é por isso que a borda ESQUERDA e a de
    // CIMA nunca precisam de nada. Mas ela **acaba** em `u_min` quando `n` é PAR e em `u_max`
    // quando é ÍMPAR — e a borda direita mostra `u_max`. Com `n` par, as duas metades que se
    // encontram são as duas pontas OPOSTAS da fonte: emenda dura, a toda a altura do sprite.
    //
    // A cura é a que o Enio descreveu: a coluna da direita passa a desenhar a **coluna esquerda,
    // espelhada** — assim ela abre exatamente no `u_min` em que o último ladrilho fechou. O mesmo
    // em Y para a linha de baixo. Os cantos herdam a decisão das duas faixas que tocam.
    //
    // ⚠️ Isto é o par da regra do número inteiro (`tile_count`): sem inteiro não há paridade
    // definida — a faixa acaba a meio de um ladrilho e **nenhuma** borda a encontra.
    let band_x = |row: usize| -> bool {
        let Some(m) = region_mode_at(&s, 1, row) else {
            return false;
        };
        ends_flipped(
            m,
            tile_count(
                m,
                s.draw_mode,
                s.tile_mode,
                s.stretch_value,
                cols_m[1],
                cols_src_m[1],
                true,
            ),
        )
    };
    let band_y = |col: usize| -> bool {
        let Some(m) = region_mode_at(&s, col, 1) else {
            return false;
        };
        ends_flipped(
            m,
            tile_count(
                m,
                s.draw_mode,
                s.tile_mode,
                s.stretch_value,
                rows_m[1],
                rows_src_m[1],
                true,
            ),
        )
    };
    let flip_x_at_row = [band_x(0), band_x(1), band_x(2)];
    let flip_y_at_col = [band_y(0), band_y(1), band_y(2)];

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
            // A coluna da direita e a linha de baixo trocam de fonte quando a faixa que lhes
            // encosta acaba invertida — e só elas: o começo da faixa nunca inverte.
            let flip = [
                col == 2 && flip_x_at_row[row],
                row == 2 && flip_y_at_col[col],
            ];
            let (uc, vr) = (if flip[0] { 0 } else { col }, if flip[1] { 0 } else { row });
            out[idx] = Some(SlicePatch {
                uv: [u_edges[uc], v_edges[vr], u_edges[uc + 1], v_edges[vr + 1]],
                // De MUNDO para LOCAL: a `basis` multiplica outra vez lá à frente.
                size: [pw / sx, ph / sy],
                center_offset: [cx / sx, cy / sy],
                uv_xform: [tiles_x, tiles_y, 0.0, 0.0],
                repeat_tag: repeats.then_some(match mode {
                    TileRegionMode::Mirror => REPEAT_MIRROR,
                    _ => REPEAT_ENABLED,
                }),
                flip,
            });
        }
    }
    out
}

/// O inteiro de ladrilhos mais próximo, nunca menos que um: uma faixa sem ladrilho nenhum é uma
/// faixa vazia, e arredondar 0,4 para zero apagaria a região.
fn whole(raw: f32) -> f32 {
    raw.round().max(1.0)
}

/// A faixa acaba numa cópia **invertida**?
///
/// O espelho é uma onda triangular: `wrap(n)` vale `0` em `n` par e `1` em `n` ímpar. Logo uma
/// faixa espelhada com um número par de ladrilhos fecha no `u_min` da fonte — o mesmo sítio em
/// que ABRIU — e quem combina com ela é a borda oposta, espelhada.
///
/// Só `Mirror` inverte: `Repeat` com número inteiro fecha a aproximar-se de `u_max`, que é
/// exatamente onde a borda direita começa.
#[allow(clippy::cast_possible_truncation)] // `n` já é inteiro aqui (Mirror força `whole`)
fn ends_flipped(mode: TileRegionMode, n: f32) -> bool {
    mode == TileRegionMode::Mirror && (n.round() as i64) % 2 == 0
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
    // ⚠️ **O miolo é AUTORADO como as outras oito** desde 2026-08-22. Antes o modo estava fixado
    // aqui, e espelhar era inalcançável **na maior área ladrilhada** — a que mais mostra a
    // emenda entre dois ladrilhos (smoke do Enio).
    //
    // A regra «Stretch, em Tiled, significa repetir» é a MESMA da moldura (ver `tile_count`), por
    // isso o default continua a repetir o miolo em Tiled: ninguém perde o que já tinha.
    s.centre_tile_mode
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
///
/// ⚠️ **`Mirror` IMPÕE o número inteiro, qualquer que seja o `tile_mode`.** Não é preferência: um
/// espelho cortado a meio acaba numa coluna arbitrária da fonte e **nenhuma** borda a encontra —
/// a paridade que escolhe a borda certa (ver [`nine_slice_patches`]) só existe sobre um inteiro.
/// Em regiões espelhadas o `stretch_value` fica, portanto, inerte.
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
    if mode == TileRegionMode::Mirror {
        return whole(raw);
    }
    match tile_mode {
        SliceTileMode::Continuous => raw,
        SliceTileMode::Whole => whole(raw),
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
#[path = "nine_slice_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "nine_slice_parity_tests.rs"]
mod parity_tests;
