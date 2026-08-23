//! Gates da grelha desenhada — irmão de [`super`] pelo teto de LOC do shell.
//!
//! ⚠️ **O que se afirma são NÚMEROS, e não pixels.** O traço em si é traço; o que pode estar errado
//! é o sinal do `Y`, o pivô e o espelho — e esses cabem num `assert_eq`.

use super::*;

/// Uma sprite de células de `2×2` metros numa grelha `hf × vf`, parada no frame `live`.
///
/// ⚠️ **`Sprite::atlas` nasce CENTRADA**, então o `resolve_anchor` dela é a origem — e é por isso
/// que o gate do pivô abaixo o move de propósito: com o pivô na origem, uma implementação que
/// ignorasse o `resolve_anchor` ficaria verde.
fn spr(hf: u32, vf: u32, live: u32) -> Sprite {
    let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    s.hframes = hf;
    s.vframes = vf;
    s.frame = live;
    s
}

const PPM: f32 = 100.0;

/// **Sem grelha não há retículo** — e uma célula de área nula também não.
#[test]
fn there_is_no_lattice_without_a_grid() {
    assert!(lattice(&spr(1, 1, 0), PPM, false).is_none());
    let mut degenerate = spr(4, 2, 0);
    degenerate.size = [0.0, 2.0];
    assert!(
        lattice(&degenerate, PPM, false).is_none(),
        "uma celula de largura zero nao tem retículo"
    );
    assert!(lattice(&spr(4, 2, 0), PPM, false).is_some());
}

/// **A folha abre-se à volta da célula viva, e a linha seguinte fica ABAIXO.**
///
/// ⚠️ É a inversão `V cresce para baixo · Y do mundo cresce para cima`. Trocá-la desenharia a
/// grelha espelhada na vertical sobre células que estão do outro lado — e o desenho continuaria
/// «bonito», que é o que torna este gate necessário.
#[test]
fn the_lattice_opens_around_the_live_cell_and_downward() {
    // Viva = célula 0 (coluna 0, linha 0), grelha 4×2, células de 2 m.
    let l = lattice(&spr(4, 2, 0), PPM, false).unwrap();
    assert_eq!((l.live_cx, l.live_cy), (0.0, 0.0), "a viva esta' na origem");
    assert_eq!(l.x0, -1.0, "meia celula a' esquerda da viva");
    assert_eq!(
        l.y0, 1.0,
        "meia celula ACIMA da viva -- a linha 0 e' a de cima"
    );
    assert_eq!((l.w, l.h), (8.0, 4.0));

    // Viva = célula 5 (coluna 1, linha 1): a folha estende-se para trás e para cima.
    let l = lattice(&spr(4, 2, 5), PPM, false).unwrap();
    assert_eq!(l.x0, -3.0, "uma celula e meia a' esquerda");
    assert_eq!(
        l.y0, 3.0,
        "uma celula e meia acima -- a linha de cima existe"
    );
    // E o retângulo contém a célula viva, sempre.
    assert!(l.x0 <= l.live_cx - 1.0 && l.live_cx + 1.0 <= l.x0 + l.w);
    assert!(l.y0 - l.h <= l.live_cy - 1.0 && l.live_cy + 1.0 <= l.y0);
}

/// **O retículo segue o PIVÔ autorado**, e não a origem do objeto.
///
/// ⚠️ O shader desenha o quad em `anchor + quad_pos * size` — desenhar a grelha a partir da origem
/// deixaria as linhas ao lado da arte em toda sprite não centrada, que é o caso de toda sprite
/// importada com `Centered` desmarcado.
#[test]
fn the_lattice_follows_the_authored_pivot() {
    let centred = lattice(&spr(4, 2, 0), PPM, false).unwrap();
    let mut off = spr(4, 2, 0);
    off.centered = false;
    let moved = lattice(&off, PPM, false).unwrap();
    assert_ne!(
        (moved.live_cx, moved.live_cy),
        (centred.live_cx, centred.live_cy),
        "tirar o `centered` MOVE o quad -- se nao move, esta fixtura nao contem o fenomeno"
    );
    // E o retículo acompanha, mantendo a mesma folga da célula viva.
    assert_eq!(moved.x0 - moved.live_cx, centred.x0 - centred.live_cx);
    assert_eq!(moved.y0 - moved.live_cy, centred.y0 - centred.live_cy);
}

/// **O FLIP abre a folha para o outro lado** — e a célula viva não sai do lugar.
///
/// ⚠️ As duas metades juntas: o `ghost` nega o deslocamento, então as linhas têm de acompanhar,
/// **mas** o quad do sprite continua onde está. Uma cura que espelhasse o retículo inteiro (centro
/// incluído) descolaria as linhas da arte.
#[test]
fn a_flipped_sheet_opens_the_other_way_and_the_live_cell_stays() {
    let plain = lattice(&spr(4, 2, 0), PPM, false).unwrap();
    let mut fx = spr(4, 2, 0);
    fx.flip_x = true;
    let flipped = lattice(&fx, PPM, false).unwrap();
    assert_eq!(
        (flipped.live_cx, flipped.live_cy),
        (plain.live_cx, plain.live_cy),
        "a celula VIVA nao se move"
    );
    // Sem flip, a célula 0 é a mais à esquerda; com flip, é a mais à direita.
    assert_eq!(plain.x0, -1.0);
    assert_eq!(flipped.x0, -7.0, "a folha abre para a ESQUERDA da viva");
    assert_eq!(flipped.x0 + flipped.w, 1.0, "e acaba na borda direita dela");

    let mut fy = spr(4, 2, 0);
    fy.flip_y = true;
    let flipped = lattice(&fy, PPM, false).unwrap();
    assert_eq!(flipped.y0, 3.0, "a linha 0 passa a ser a de BAIXO");
    assert_eq!(flipped.y0 - flipped.h, -1.0);
}

/// **O retículo e os fantasmas concordam sobre onde cada célula cai.**
///
/// ⚠️ O gate que liga os dois módulos: o `sim_extract_sheet` põe a arte e este põe as linhas, e
/// eles derivam a posição por caminhos diferentes (um por deslocamento relativo, o outro por canto
/// + índice). Se discordarem, as linhas caem no meio dos desenhos — e cada módulo passa sozinho.
#[test]
fn the_lines_land_on_the_cells_the_ghosts_draw() {
    for (hf, vf, live) in [(4u32, 2u32, 0u32), (4, 2, 5), (3, 3, 4), (8, 1, 7)] {
        let s = spr(hf, vf, live);
        let l = lattice(&s, PPM, false).unwrap();
        let cells = hf * vf;
        for i in 0..cells {
            // Onde o retículo diz que a célula `i` está (canto + índice, espelho já dentro).
            let (col, row) = (i % hf, i / hf);
            let want_cx = l.x0 + (f64::from(col) + 0.5) * l.cell_w;
            let want_cy = l.y0 - (f64::from(row) + 0.5) * l.cell_h;
            // Onde o fantasma a põe (deslocamento relativo à viva, espelho aplicado no `ghost`).
            let got = match super::super::sim_extract_sheet::cell(&s, [0.0, 0.0, 1.0, 1.0], i) {
                Some((_, off)) => {
                    let (mut dx, mut dy) = (f64::from(off[0]), f64::from(off[1]));
                    if s.flip_x {
                        dx = -dx;
                    }
                    if s.flip_y {
                        dy = -dy;
                    }
                    (l.live_cx + dx, l.live_cy + dy)
                }
                // A célula viva não tem fantasma — ela está no centro dela própria.
                None => (l.live_cx, l.live_cy),
            };
            assert!(
                (got.0 - want_cx).abs() < 1.0e-9 && (got.1 - want_cy).abs() < 1.0e-9,
                "grelha {hf}x{vf} viva {live}: a celula {i} esta' em {got:?} e a linha diz \
                 ({want_cx}, {want_cy})"
            );
        }
    }
}

/// **DESDOBRADA, o retículo centra-se no PIVÔ — e é isso que alinha as linhas com a arte pintada.**
///
/// ⚠️ O defeito que este gate prende foi fotografado pelo Enio (2026-08-23): a folha pintada
/// centra-se no pivô e o retículo continuava a dispor-se à volta da célula viva, o que desloca as
/// linhas **meia célula**. As duas contas só coincidem quando `lcol = hf/2 − ½`, que não é inteiro.
///
/// **Mutação que deve sangrar:** passar `false` no braço desdobrado do overlay.
#[test]
fn the_unfolded_lattice_is_centred_on_the_pivot_and_matches_the_painted_quad() {
    for live in 0..8u32 {
        let s = spr(4, 2, live);
        let l = lattice(&s, PPM, true).unwrap();
        // ⭐ O retículo E o quad que a pintura desenha descrevem o MESMO rectângulo.
        let size = super::super::sim_extract_sheet::unfolded_quad(&s).unwrap();
        let pivot = s.resolve_anchor(PPM);
        assert_eq!((l.w, l.h), (f64::from(size[0]), f64::from(size[1])));
        assert_eq!(l.x0, f64::from(pivot[0]) - f64::from(size[0]) * 0.5);
        assert_eq!(l.y0, f64::from(pivot[1]) + f64::from(size[1]) * 0.5);
        // ⚠️ E NÃO depende do frame: só o realce se move.
        assert_eq!(l.x0, lattice(&spr(4, 2, 0), PPM, true).unwrap().x0);
        // A célula viva está no SLOT dela, dentro do retículo.
        let (col, row) = (f64::from(live % 4), f64::from(live / 4));
        assert_eq!(l.live_cx, l.x0 + (col + 0.5) * l.cell_w);
        assert_eq!(l.live_cy, l.y0 - (row + 0.5) * l.cell_h);
    }

    // ⚠️ E a metade que nomeia o defeito: DOBRADA, a disposição é OUTRA — e tem de ser.
    //
    // ⚠️ **O desvio é `(lcol + ½ − hf/2)·cw`, e NÃO «meia célula» sempre** — a primeira versão
    // deste gate afirmou o segundo e sangrou na hora. Meia célula é o valor no caso
    // **fotografado** (8 células, viva na 4), e a asserção genérica é a fórmula.
    for live in 0..8u32 {
        let s = spr(4, 2, live);
        let folded = lattice(&s, PPM, false).unwrap();
        let unfolded = lattice(&s, PPM, true).unwrap();
        let lcol = f64::from(live % 4);
        assert_eq!(
            unfolded.x0 - folded.x0,
            (lcol + 0.5 - 4.0 * 0.5) * folded.cell_w,
            "o desvio do frame {live}"
        );
    }
    // O caso da FOTO: 8 células numa tira, a viva na 4 ⇒ exactamente meia célula.
    let photo = spr(8, 1, 4);
    let folded = lattice(&photo, PPM, false).unwrap();
    let unfolded = lattice(&photo, PPM, true).unwrap();
    assert_eq!(
        (unfolded.x0 - folded.x0).abs(),
        folded.cell_w * 0.5,
        "e' o deslocamento de meia celula que a foto mostra"
    );
}
