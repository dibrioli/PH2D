//! Os gates do assador (plano 33 §5), **red-first**.
//!
//! ⛔⛔ **A fixtura é declarada ANTES dos gates, e é de propósito.** A lição que esta linha pagou
//! seis vezes em 26/08: *uma fixtura que não contém o fenómeno aprova a cura errada.* A arte de
//! referência aqui é **5x3** — largura ÍMPAR e não quadrada — e o tijolo de referência tem
//! `offset_denom = 3`, o que dá desfasamentos de `0, 1, 3` pixels: **passos DESIGUAIS** (1, depois
//! 2, depois 2 para fechar). Uma arte quadrada com meio passo aprovaria qualquer aritmética.

use super::{BakeError, MAX_TILE_EDGE_PX, TileKind, TileLaw, bake};

/// Arte de referência: o texel `(x, y)` carrega as próprias coordenadas, então trocar dois texels
/// de sítio é visível. Alfa 255 em toda a parte (o caso opaco; o transparente tem gate próprio).
fn art(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            #[allow(clippy::cast_possible_truncation)]
            v.extend_from_slice(&[x as u8, y as u8, 200, 255]);
        }
    }
    v
}

fn texel(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
    let o = ((y * w + x) * 4) as usize;
    [buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]
}

/// **O ORÁCULO — e ele é independente do assador de propósito.**
///
/// O assador constrói UM ladrilho, dando a volta por módulo. Este oráculo faz o contrário: enumera
/// as instâncias do reticulado **sem limite e sem módulo nenhum**, e pergunta qual delas cobre o
/// ponto `(X, Y)`. Se a volta do assador estiver errada, os dois discordam.
///
/// A instância `(col, row)` tem a origem da arte em
/// `ox = col*cw + (row div n)*cw + floor((row mod n)*cw / n)` (e o transposto para `BrickCol`) — o
/// termo `(row div n)*cw` é o que faz a escada continuar para além de um período, que é exactamente
/// a propriedade que o ladrilho promete.
fn oracle(law: &TileLaw, aw: u32, ah: u32, cell: [u32; 2], x: i64, y: i64) -> Option<[u8; 4]> {
    let (cw, ch) = (i64::from(cell[0]), i64::from(cell[1]));
    let n = i64::from(law.period());
    let a = art(aw, ah);
    // ⚠️ **O ALCANCE tem de cobrir a DERIVA, e a 1.ª versão deste oráculo não cobria.** Ele varria
    // `-4..=4` e reprovava o ponto `(-5, 15)`, que é coberto pela instância `(col -3, row 5)`: a
    // deriva `(row div n) * cw` empurra as instâncias para longe à medida que a escada sobe, então o
    // alcance NÃO é o número de células — é ele mais a deriva. Conferido à mão antes de mexer no
    // produto: `ox = -15 + 5 + 3 = -7`, `dx = 2`, `dy = 0`, que é exactamente o que o ladrilho tinha
    // lá. ⛔ *Uma régua curta demais acusa o produto de um defeito que é dela.*
    let mut hit = None;
    for row in -12i64..=12 {
        for col in -12i64..=12 {
            let (ox, oy) = match law.kind {
                TileKind::Grid => (col * cw, row * ch),
                TileKind::BrickRow | TileKind::Hex => {
                    let (q, m) = (row.div_euclid(n), row.rem_euclid(n));
                    (col * cw + q * cw + (m * cw) / n, row * ch)
                }
                TileKind::BrickCol => {
                    let (q, m) = (col.div_euclid(n), col.rem_euclid(n));
                    (col * cw, row * ch + q * ch + (m * ch) / n)
                }
            };
            let (dx, dy) = (x - ox, y - oy);
            if dx >= 0 && dy >= 0 && dx < i64::from(aw) && dy < i64::from(ah) {
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let t = texel(&a, aw, dx as u32, dy as u32);
                assert!(
                    hit.is_none_or(|h| h == t),
                    "a fixtura tem sobreposicao: o oraculo nao e' univoco em ({x},{y})"
                );
                hit = Some(t);
            }
        }
    }
    hit
}

/// ⭐ **O ponto neutro: a grade encostada devolve a arte ao BYTE.** É a mesma invariante que a rack
/// de áudio impõe a todo efeito, e a razão é a mesma: um caminho comum que não é identidade é um
/// caminho comum que ninguém consegue verificar.
#[test]
fn a_grid_tile_is_byte_identical_to_its_source() {
    let a = art(5, 3);
    let t = bake(&a, 5, 3, &TileLaw::grid()).expect("a grade encostada assa");
    assert_eq!((t.width, t.height), (5, 3));
    assert_eq!(t.cells, [1, 1]);
    assert_eq!(
        t.rgba, a,
        "a grade encostada tem de devolver a arte ao byte"
    );
}

/// ⚠️ **Um desfasamento de `1/1` é NENHUM desfasamento** — e por isso um tijolo com `offset_denom`
/// de `0` ou `1` tem de assar exactamente como a grade. Sem isto, `0` seria uma divisão por zero e
/// `1` um ladrilho de uma linha que se diz tijolo.
#[test]
fn an_offset_of_one_is_the_grid() {
    let a = art(5, 3);
    let grid = bake(&a, 5, 3, &TileLaw::grid()).expect("grade");
    for denom in [0u8, 1] {
        let law = TileLaw {
            kind: TileKind::BrickRow,
            offset_denom: denom,
            gap_px: [0, 0],
        };
        let t = bake(&a, 5, 3, &law).expect("tijolo sem desfasamento");
        assert_eq!(t, grid, "offset_denom = {denom} tem de ser a grade");
    }
}

/// ⭐⭐ **O gate forte: o ladrilho REPRODUZ o reticulado infinito.**
///
/// Compara o assado (lido com módulo, como a GPU o lê) contra o [`oracle`] (que enumera instâncias
/// sem módulo) sobre uma região de **3x3 ladrilhos**. Um erro na volta, um desfasamento que não
/// fecha ao fim de `n`, ou um arredondamento que perde um pixel aparecem aqui e em mais lado nenhum.
#[test]
fn the_lattice_closes_on_itself() {
    let (aw, ah) = (5u32, 3u32);
    let a = art(aw, ah);
    for kind in [TileKind::Grid, TileKind::BrickRow, TileKind::BrickCol] {
        for gap in [[0i32, 0], [2, 1]] {
            let law = TileLaw {
                kind,
                offset_denom: 3,
                gap_px: gap,
            };
            let t = bake(&a, aw, ah, &law).expect("assa");
            #[allow(clippy::cast_sign_loss)]
            let cell = [
                (i64::from(aw) + i64::from(gap[0])).max(1) as u32,
                (i64::from(ah) + i64::from(gap[1])).max(1) as u32,
            ];
            for y in -i64::from(t.height)..2 * i64::from(t.height) {
                for x in -i64::from(t.width)..2 * i64::from(t.width) {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let got = texel(
                        &t.rgba,
                        t.width,
                        x.rem_euclid(i64::from(t.width)) as u32,
                        y.rem_euclid(i64::from(t.height)) as u32,
                    );
                    let want = oracle(&law, aw, ah, cell, x, y).unwrap_or([0, 0, 0, 0]);
                    assert_eq!(
                        got, want,
                        "{kind:?} gap {gap:?}: o ladrilho discorda do reticulado em ({x},{y})"
                    );
                }
            }
        }
    }
}

/// **O tijolo por LINHA é o transposto do tijolo por COLUNA.** Duas leis com o mesmo mecanismo em
/// eixos trocados — se uma delas tiver um eixo errado, elas deixam de ser espelho uma da outra.
#[test]
fn brick_by_row_is_the_transpose_of_brick_by_column() {
    let (aw, ah) = (5u32, 3u32);
    let a = art(aw, ah);
    // A arte transposta: o texel (y, x) do original.
    let mut at = Vec::with_capacity((aw * ah * 4) as usize);
    for y in 0..aw {
        for x in 0..ah {
            at.extend_from_slice(&texel(&a, aw, y, x));
        }
    }
    let law = |kind| TileLaw {
        kind,
        offset_denom: 3,
        gap_px: [1, 2],
    };
    let row = bake(&a, aw, ah, &law(TileKind::BrickRow)).expect("linha");
    // Transposto: arte transposta E vão transposto.
    let mut lc = law(TileKind::BrickCol);
    lc.gap_px = [2, 1];
    let col = bake(&at, ah, aw, &lc).expect("coluna");
    assert_eq!((row.width, row.height), (col.height, col.width));
    for y in 0..row.height {
        for x in 0..row.width {
            assert_eq!(
                texel(&row.rgba, row.width, x, y),
                texel(&col.rgba, col.width, y, x),
                "o tijolo por linha nao e' o transposto do por coluna em ({x},{y})"
            );
        }
    }
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"em column, a depender do valor dos parâmetros o pattern
/// some"*.**
///
/// A 1.ª versão **RECUSAVA** um ladrilho maior que o atlas, e a forma voltava à cor de recurso sem
/// explicação. E o tecto chegava com facilidade absurda porque o **VÃO era assado na resolução da
/// ARTE** — e vão é espaço VAZIO.
///
/// ⭐ **Medido**: arte de `256 px` a medir 1 unidade, com `Gap 2` e `Column 1/8`, pedia
/// `6144x768`; a MESMA lei em `Grid` pede `768x768`. Os reticulados desfasados chegam ao tecto
/// **`n` vezes mais cedo** — e é por isso que o report é sobre a Column.
///
/// ⇒ o ladrilho é **REDUZIDO até caber**, nunca recusado.
///
/// # O tecto continua a ser do ATLAS
///
/// `vello_encoding-0.8.0/src/image_cache.rs:9-10`: o atlas nasce em `1024`, dobra até
/// `MAX_ATLAS_SIZE = 8192`, e é **um só para todas as imagens do quadro**. Um recurso sem vaga é
/// descartado **em silêncio** (`resolve.rs:296`).
#[test]
fn a_tile_that_would_overflow_the_atlas_is_scaled_not_refused() {
    let (aw, ah) = (256u32, 256);
    let a = art(aw, ah);
    let law = |gap: i32, denom: u8| TileLaw {
        kind: TileKind::BrickCol,
        offset_denom: denom,
        gap_px: [gap, gap],
    };
    // O caso do report: 8 colunas de (256 + 512) = 6144 px de largura.
    let t = bake(&a, aw, ah, &law(512, 8)).expect("tem de ASSAR, nao recusar");
    assert!(
        t.width <= MAX_TILE_EDGE_PX && t.height <= MAX_TILE_EDGE_PX,
        "o ladrilho reduzido nao coube: {}x{}",
        t.width,
        t.height
    );
    assert_eq!(t.cells, [8, 1], "a LEI nao muda com a reducao");
    // ⭐ E a proporção arte:vão sobrevive — é ela que faz o passo no mundo ficar igual.
    let cell_w = t.width / 8;
    let art_w = cell_w - (cell_w * 512 / (256 + 512));
    assert!(
        (f64::from(art_w) / f64::from(cell_w) - 256.0 / 768.0).abs() < 0.02,
        "a proporcao arte:vao mudou na reducao"
    );
    // CONTROLO: o que já cabia continua **na resolução nativa** — reduzir sempre seria pagar
    // qualidade por nada.
    let ok = bake(&a, aw, ah, &law(0, 4)).expect("cabe");
    assert_eq!((ok.width, ok.height), (1024, 256));
}

/// ⭐ **Vão NEGATIVO é a sobreposição** — o *Overlap* do Illustrator — e ela sai de graça da mesma
/// máquina de dar-a-volta que o tijolo precisa. Sem cópias a sobrepor-se, o ladrilho ficaria com o
/// tamanho da célula e um buraco onde a arte não chega.
#[test]
fn a_negative_gap_overlaps_instead_of_leaving_a_hole() {
    let a = art(4, 4);
    let law = TileLaw {
        kind: TileKind::Grid,
        offset_denom: 1,
        gap_px: [-2, -2],
    };
    let t = bake(&a, 4, 4, &law).expect("sobreposicao assa");
    assert_eq!((t.width, t.height), (2, 2), "a celula encolhe com o vao");
    for y in 0..2 {
        for x in 0..2 {
            assert_eq!(
                texel(&t.rgba, 2, x, y)[3],
                255,
                "a sobreposicao nao pode deixar buraco em ({x},{y})"
            );
        }
    }
}

/// ⚠️ **A célula nunca desce abaixo de um pixel.** Um vão negativo maior que a arte pediria uma
/// célula de zero ou negativa — e um ladrilho de largura zero é uma divisão por zero na GPU.
#[test]
fn a_gap_bigger_than_the_art_still_leaves_one_pixel() {
    let a = art(4, 4);
    let law = TileLaw {
        kind: TileKind::Grid,
        offset_denom: 1,
        gap_px: [-99, -99],
    };
    let t = bake(&a, 4, 4, &law).expect("assa");
    assert_eq!((t.width, t.height), (1, 1));
}

/// ⚠️⚠️ **Um texel TRANSPARENTE conserva a cor dele** — e este gate é a família do
/// [Bug #4 do Motion](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md), que só se vê fora de
/// `alpha = 1`.
///
/// Um PNG comum guarda RGB debaixo de alfa zero. Se o assador compusesse *source-over* sobre um
/// destino transparente com a fórmula geral, `0/0` apagaria esse RGB — e o `Grid` deixaria de ser
/// byte-idêntico **sem que nenhum gate opaco desse por isso**.
#[test]
fn a_transparent_texel_keeps_its_colour() {
    let a = vec![10, 20, 30, 0, 40, 50, 60, 255];
    let t = bake(&a, 2, 1, &TileLaw::grid()).expect("assa");
    assert_eq!(t.rgba, a, "o RGB debaixo de alfa zero tem de sobreviver");
}

/// **Arte vazia recusa** em vez de assar um ladrilho de zero bytes que a GPU descartaria em silêncio.
#[test]
fn empty_art_is_refused() {
    assert_eq!(bake(&[], 0, 0, &TileLaw::grid()), Err(BakeError::Empty));
    assert_eq!(
        bake(&[1, 2, 3], 5, 3, &TileLaw::grid()),
        Err(BakeError::Empty),
        "buffer que nao bate com as dimensoes"
    );
}

/// ⏱️ **O KILL-CRITERION do assador** (plano 33 §6), `#[ignore]` + `--release`.
///
/// O orçamento é **8 ms** — o mesmo tecto que o [plano 23](../../../docs/Vector%20Module/23_plano_pattern_along_path.md)
/// usou para o *pattern along path*, e pela mesma razão: um assado acontece quando o artista mexe
/// num knob, e meio quadro é o limite do que uma tecla pode custar.
///
/// ⚠️ **Isto NÃO é um gate de razão** (`CLAUDE.md` §5.0): a barra é um tecto absoluto com folga de
/// ordem de grandeza, não o quociente de dois relógios. Um gate que divide dois tempos reprova sob
/// fan-out e este não o faz — mas ele ainda lê um relógio, e por isso é `#[ignore]`.
#[test]
#[ignore = "mede relogio; correr com --release e a maquina calma"]
fn the_bake_of_a_full_tile_stays_under_the_kill() {
    let (aw, ah) = (512u32, 512u32);
    let a = art(aw, ah);
    let law = TileLaw {
        kind: TileKind::Hex,
        offset_denom: 2,
        gap_px: [24, 24],
    };
    let t0 = std::time::Instant::now();
    let t = bake(&a, aw, ah, &law).expect("assa");
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "assado {}x{} ({} celulas) em {ms:.3} ms",
        t.width, t.height, t.cells[1]
    );
    assert!(ms < 8.0, "o assado custou {ms:.3} ms, o kill e' 8");
}

/// ⚠️⚠️ **A REDUÇÃO não puxa a cor de debaixo do alfa zero.**
///
/// Somar RGBA **reto** de texels com alfas diferentes mistura a cor dos transparentes na dos
/// opacos — a borda de todo PNG com transparência muda de cor. A média é em alfa
/// **pré-multiplicado**: soma de `cor x alfa`, divisão pela soma dos alfas.
///
/// A fixtura contém o fenómeno: metade branca OPACA, metade **preta TRANSPARENTE**. Uma média reta
/// devolveria cinzento; a certa devolve branco com meia alfa.
#[test]
fn the_downscale_does_not_bleed_colour_from_under_zero_alpha() {
    // 2x1 -> 1x1: um texel branco opaco + um preto transparente.
    let src = vec![255u8, 255, 255, 255, 0, 0, 0, 0];
    // Um `gap_px` enorme força a redução; `Grid` mantém uma célula só.
    let law = TileLaw {
        kind: TileKind::Grid,
        offset_denom: 1,
        gap_px: [MAX_TILE_EDGE_PX as i32 * 2, 0],
    };
    let t = bake(&src, 2, 1, &law).expect("assa reduzido");
    // A arte reduziu para 1x1; o texel dela é o primeiro do ladrilho.
    let c = texel(&t.rgba, t.width, 0, 0);
    assert_eq!(
        [c[0], c[1], c[2]],
        [255, 255, 255],
        "a media puxou a cor de debaixo do alfa zero: {c:?}"
    );
    assert!(
        (i32::from(c[3]) - 128).abs() <= 1,
        "a alfa media devia ser ~128, deu {}",
        c[3]
    );
}
