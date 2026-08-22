//! **A lei das duas cláusulas** — o que faz a faixa ladrilhada ENCOSTAR na sua borda.
//!
//! Ficheiro irmão do [`super::nine_slice_tests`] por assunto, não por cap: estes testes provam
//! uma lei só, e ela nasceu de um smoke (2026-08-22) em que o miolo espelhado ficou perfeito e
//! **a borda direita ficou com uma emenda dura a toda a altura**.
//!
//! ```text
//! 1. Uma faixa só encontra a borda se contiver um número INTEIRO de ladrilhos.
//! 2. Espelhada e com número PAR, ela fecha invertida — e a borda que combina é a OPOSTA,
//!    espelhada.
//! ```
//!
//! ⚠️ **As duas são inseparáveis.** Sem (1) não há paridade definida: a faixa acaba a meio de um
//! ladrilho, numa coluna arbitrária da fonte, e *nenhuma* borda a encontra. É por isso que
//! `Mirror` força o inteiro em vez de o oferecer.
//!
//! # A grelha destes testes
//!
//! Fonte 64×64, bordas de 16 px. Com `pixels_per_meter = 100` cada borda mede 0,16 m e a faixa
//! central da fonte mede 0,32 m — logo um alvo de `0.32 + 0.32·k` metros cabe **exatamente** `k`
//! ladrilhos, e o `k` é o que estes testes variam. Em UV as arestas caem em `[0, .25, .75, 1]`,
//! números exatos em `f32`: as comparações abaixo podem ser exatas de propósito.

use super::*;

/// Bordas de 16 px em todos os lados, sobre uma fonte de 64×64.
const BORDER_PX: f32 = 16.0;
/// As arestas em UV que essas bordas produzem — exatas em `f32`.
const U: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
/// Um ladrilho da faixa central, em metros (32 px a 100 px/m).
const TILE_M: f32 = 0.32;
/// Uma borda, em metros.
const EDGE_M: f32 = 0.16;

fn tiled(centre: TileRegionMode) -> SliceNine {
    SliceNine {
        draw_mode: SliceDrawMode::Tiled,
        borders: [BORDER_PX; 4],
        centre_tile_mode: centre,
        ..SliceNine::INERT
    }
}

/// Cozinha a grelha para um alvo que cabe `tiles_x` × `tiles_y` ladrilhos no miolo.
fn grid(s: &SliceNine, tiles_x: f32, tiles_y: f32) -> [Option<SlicePatch>; PATCH_COUNT] {
    nine_slice_patches(
        [0.0, 0.0, 1.0, 1.0],
        [64.0, 64.0],
        s,
        [
            EDGE_M * 2.0 + TILE_M * tiles_x,
            EDGE_M * 2.0 + TILE_M * tiles_y,
        ],
        100.0,
        [1.0, 1.0],
    )
}

/// Índices da grelha, para os testes se lerem.
const TL: usize = 0;
const L: usize = 3;
const R: usize = 5;
const BL: usize = 6;
const BR: usize = 8;

// ─────────────────────────────── cláusula 1: o inteiro ───────────────────────────────

/// ⚠️ **`Mirror` impõe o número inteiro, mesmo em `Continuous`.**
///
/// `Continuous` existe para deixar o último ladrilho cortado — e um ladrilho ESPELHADO cortado
/// acaba numa coluna arbitrária da fonte, que borda nenhuma encontra. A cláusula 2 (paridade) só
/// tem significado sobre um inteiro; sem isto ela seria um sorteio.
#[test]
fn a_mirrored_band_is_always_a_whole_number_of_tiles() {
    // 2,34 ladrilhos: em `Repeat` ficaria cortado; espelhado, tem de arredondar para 2.
    let s = tiled(TileRegionMode::Mirror);
    let n = grid(&s, 2.34, 1.0)[CENTRE_INDEX].unwrap().uv_xform[0];
    assert_eq!(n, 2.0, "o miolo espelhado saiu com {n} ladrilhos, nao 2");

    // E o mesmo pedido em `Repeat` continua cortado — a regra é do espelho, nao do Tiled.
    let mut r = tiled(TileRegionMode::Repeat);
    r.tile_mode = SliceTileMode::Continuous;
    let rn = grid(&r, 2.34, 1.0)[CENTRE_INDEX].unwrap().uv_xform[0];
    assert!(
        (rn - 2.34).abs() < 1e-3,
        "Continuous+Repeat deixou de cortar ({rn}) — a regra vazou para fora do espelho"
    );
}

/// **`Whole` é a cura para quem NÃO quer espelhar** — o `TILE_FIT` do Godot. Sem ele, a única
/// forma de não ter o ladrilho cortado era espelhar, que é uma decisão de aspeto.
#[test]
fn whole_rounds_the_count_for_every_repeating_region() {
    let mut s = tiled(TileRegionMode::Repeat);
    s.tile_mode = SliceTileMode::Whole;
    let n = grid(&s, 2.34, 1.0)[CENTRE_INDEX].unwrap().uv_xform[0];
    assert_eq!(n, 2.0, "Whole deixou {n} ladrilhos — devia arredondar");
    // ⚠️ Nunca zero: uma faixa mais estreita que um ladrilho continua a desenhar UM.
    let tiny = grid(&s, 0.2, 1.0)[CENTRE_INDEX].unwrap().uv_xform[0];
    assert_eq!(tiny, 1.0, "arredondar para zero apagaria a regiao");
}

// ─────────────────────────────── cláusula 2: a paridade ───────────────────────────────

/// ⚠️ **O defeito que o Enio marcou com as setas amarelas.**
///
/// Miolo espelhado, **2** ladrilhos: o segundo sai invertido e fecha no `u_min` da fonte — o
/// mesmo sítio em que a faixa abriu. A borda direita mostra o `u_max`, e as duas metades que se
/// encontram são as pontas OPOSTAS da imagem. A cura é a que ele descreveu: a coluna da direita
/// desenha a **coluna esquerda, espelhada**.
#[test]
fn an_even_mirrored_run_hands_the_right_border_the_left_source_mirrored() {
    let p = grid(&tiled(TileRegionMode::Mirror), 2.0, 1.0);
    let right = p[R].expect("a coluna da direita tem de existir");
    let left = p[L].expect("a coluna da esquerda tem de existir");
    assert!(
        right.flip[0],
        "a borda direita nao inverteu — a emenda dura continua la'"
    );
    assert!(!right.flip[1], "inverteu em Y sem ninguem pedir");
    assert_eq!(
        [right.uv[0], right.uv[2]],
        [left.uv[0], left.uv[2]],
        "inverteu mas continuou a ler o SEU sub-rect: espelhar o lado errado nao cura nada"
    );
    assert_eq!([right.uv[0], right.uv[2]], [U[0], U[1]]);
    // O sítio e o tamanho do quad NÃO mudam — trocou-se a fonte, não a grelha.
    assert_eq!(right.size[0], EDGE_M, "a coluna mudou de largura");
}

/// **Ímpar não precisa de nada.** O último ladrilho sai direito e fecha no `u_max`, que é onde a
/// borda direita começa — a costura já era exata.
#[test]
fn an_odd_mirrored_run_leaves_the_border_exactly_as_it_was() {
    let p = grid(&tiled(TileRegionMode::Mirror), 3.0, 1.0);
    let right = p[R].unwrap();
    assert_eq!(right.flip, [false, false]);
    assert_eq!(
        [right.uv[0], right.uv[2]],
        [U[2], U[3]],
        "trocou a fonte sem precisar"
    );
}

/// ⚠️ **A faixa COMEÇA sempre direita** — por isso a borda esquerda e a de cima nunca inventam
/// nada, com qualquer paridade. Um espelho que invertesse as duas pontas seria simetria a mais.
#[test]
fn the_left_and_top_borders_never_flip_whatever_the_parity() {
    for tiles in [1.0_f32, 2.0, 3.0, 4.0] {
        let p = grid(&tiled(TileRegionMode::Mirror), tiles, tiles);
        assert_eq!(
            p[L].unwrap().flip,
            [false, false],
            "L com {tiles} ladrilhos"
        );
        assert_eq!(p[TL].unwrap().flip, [false, false], "TL com {tiles}");
    }
}

/// **`Repeat` nunca inverte, nem em paridade par.** Um wrap simples fecha a aproximar-se do
/// `u_max`, que é exatamente onde a borda direita abre: trocar a fonte ali seria criar a emenda
/// que este código existe para tirar.
#[test]
fn repeat_is_untouched_by_the_parity_rule() {
    let mut s = tiled(TileRegionMode::Repeat);
    s.tile_mode = SliceTileMode::Whole; // inteiro e PAR, o gatilho da cláusula 2
    let p = grid(&s, 2.0, 1.0);
    let right = p[R].unwrap();
    assert_eq!(right.flip, [false, false], "Repeat inverteu a borda");
    assert_eq!([right.uv[0], right.uv[2]], [U[2], U[3]]);
}

/// O mesmo em **Y**: as faixas laterais espelhadas com contagem par mandam a linha de baixo
/// desenhar a linha de CIMA, invertida.
#[test]
fn an_even_mirrored_side_band_flips_the_bottom_row() {
    let mut s = tiled(TileRegionMode::Stretch);
    s.tile_modes[SliceRegion::Left as usize] = TileRegionMode::Mirror;
    let p = grid(&s, 1.0, 2.0);
    let bl = p[BL].expect("o canto de baixo-esquerda tem de existir");
    assert_eq!(
        bl.flip,
        [false, true],
        "a linha de baixo nao seguiu a faixa lateral"
    );
    assert_eq!([bl.uv[1], bl.uv[3]], [U[0], U[1]], "leu a linha errada");
}

/// ⚠️ **O canto obedece às DUAS faixas que lhe encostam** — e quando ambas fecham invertidas ele
/// desenha o canto oposto na diagonal, espelhado nos dois eixos. É o único sítio da grelha em que
/// as duas cláusulas se compõem, e por isso o único que prova que elas são independentes.
#[test]
fn the_corner_obeys_both_bands_that_touch_it() {
    let mut s = tiled(TileRegionMode::Stretch);
    s.tile_modes[SliceRegion::Bottom as usize] = TileRegionMode::Mirror; // manda em X na linha 2
    s.tile_modes[SliceRegion::Right as usize] = TileRegionMode::Mirror; // manda em Y na coluna 2
    let p = grid(&s, 2.0, 2.0);
    let br = p[BR].expect("o canto de baixo-direita tem de existir");
    assert_eq!(br.flip, [true, true]);
    // Ele passa a desenhar o canto de CIMA-ESQUERDA, espelhado nos dois eixos.
    assert_eq!(br.uv, [U[0], U[0], U[1], U[1]]);
    assert_eq!(p[TL].unwrap().uv, br.uv, "nao e' o mesmo pedaco da fonte");
}

/// **Um miolo vazio não manda em ninguém.** Sem faixa central não há corrida a fechar, e inverter
/// a borda seria uma decisão tirada de uma faixa que não se desenha.
#[test]
fn a_hollow_centre_does_not_decide_the_border() {
    let mut s = tiled(TileRegionMode::Mirror);
    s.fill_center = false;
    let p = grid(&s, 2.0, 1.0);
    assert_eq!(
        p[R].unwrap().flip,
        [false, false],
        "a borda seguiu um miolo que nao existe"
    );
}
