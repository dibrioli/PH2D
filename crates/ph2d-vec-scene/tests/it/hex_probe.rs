//! **SONDA (não é gate) — o Hex ALTERA o motivo?**
//!
//! Report do dono do produto (2026-08-30, com foto): no modo *Hex* de um Texture Pattern *"as
//! posições dos objectos no grupo mudaram e o z order também"*. A arte é um grupo de duas formas
//! (um triângulo, e um círculo desenhado POR CIMA dele, deslocado).
//!
//! Esta sonda mede a metade **Hex** da acusação. Ela não cura nada e não afirma nada sobre o
//! assado do grupo — só pergunta o que sai do ladrilho.
//!
//! ⚠️ **Ela vive aqui e não em `ph2d-vec-pattern` porque a porta real é
//! `PatternFill::law(art_px)`, que é desta crate** — e a `ph2d-vec-pattern` é a folha de que esta
//! depende. Pô-la lá exigiria uma dev-dependency cíclica (edição de `Cargo.toml` de produto).
//!
//! # ⭐ O QUE ELA ACHOU, e o que os números dizem HOJE
//!
//! A causa foi medida e **curada no mesmo dia**: a `PatternFill::period` derivava o passo VERTICAL
//! da colmeia do passo da **COLUNA**, ignorando a altura da célula. Numa célula alta (`size` a
//! seguir uma arte `48x96`) isso dava `gap_px[1] = −54`, ou seja **56,2 % da altura em sobreposição**
//! e **51 % dos texels do motivo reescritos pela cópia vizinha** — que, deslocada meio passo na
//! horizontal, aparece também *ao lado*. Daí *"as posições mudaram e o z também"*.
//!
//! ⇒ Hoje a lei aperta o **eixo dela** (`√3/2 × (altura + vão)`) e a sobreposição é `13,4 %` em
//! **todo** aspecto — que é o que a colmeia É: as linhas encaixam. As duas tabelas do Q2 (`size`
//! quadrado e `size` a seguir a arte) tinham colunas **diferentes** e hoje são a **mesma**; se
//! voltarem a divergir, o passo voltou a sair do eixo errado.
//!
//! ⚠️ Ela fica como INSTRUMENTO, não como gate: o gate da lei é
//! `the_hex_squeezes_its_own_axis_by_the_same_fraction_at_every_aspect`, e é ele que reprova.
//! *Uma regra sem instrumento é uma nota que envelhece — e uma sonda sem gate não reprova ninguém.*
//!
//! `cargo test -p ph2d-vec-scene --test hex_probe -- --ignored --nocapture`

use ph2d_vec_pattern::{TileKind, TileLaw, bake};
use ph2d_vec_scene::{PatternFill, PatternSource, Rgba8};

const RED: [u8; 4] = [220, 40, 40, 255];
const GREEN: [u8; 4] = [40, 200, 60, 255];
const BLUE: [u8; 4] = [40, 70, 220, 255];

/// A fixtura: RGBA **reto**, assimétrica nos DOIS eixos, com SOBREPOSIÇÃO interna que identifica z.
///
/// - **BLUE** no topo (o análogo do triângulo: conteúdo opaco onde a vizinha vai cair);
/// - **RED** nos dois terços de baixo, encostado à esquerda;
/// - **GREEN** por cima do RED, deslocado para baixo-direita, **escrito depois** (o "círculo").
fn build_art(w: u32, h: u32) -> Vec<u8> {
    let mut px = vec![0u8; (w as usize) * (h as usize) * 4];
    let mut rect = |x0: u32, y0: u32, x1: u32, y1: u32, c: [u8; 4]| {
        for y in y0..y1.min(h) {
            for x in x0..x1.min(w) {
                let o = ((y as usize) * (w as usize) + x as usize) * 4;
                px[o..o + 4].copy_from_slice(&c);
            }
        }
    };
    rect(w / 8, 0, w, h / 8, BLUE);
    rect(0, h / 3, w * 3 / 4, h, RED);
    rect(w / 4, h / 2, w, h * 5 / 6, GREEN);
    px
}

fn texel(px: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
    let o = ((y as usize) * (w as usize) + x as usize) * 4;
    [px[o], px[o + 1], px[o + 2], px[o + 3]]
}

/// ⚠️ **CONTROLO DA FIXTURA** — uma fixtura sem o fenómeno lê-se como cura.
///
/// Prova que (a) há texels VERDES por cima de VERMELHOS, (b) a arte não é simétrica em y,
/// (c) nem em x, e (d) há conteúdo OPACO no topo (senão a cobertura só se veria sobre vazio).
fn assert_fixture_has_the_phenomenon(art: &[u8], w: u32, h: u32) {
    // (a) verde sobre vermelho: reconstruo a arte SEM o verde e conto onde ele apagou vermelho.
    let mut sem_verde = vec![0u8; art.len()];
    for y in 0..h {
        for x in 0..w {
            let o = ((y as usize) * (w as usize) + x as usize) * 4;
            let c = if x >= w / 8 && y < h / 8 {
                BLUE
            } else if y >= h / 3 && x < w * 3 / 4 {
                RED
            } else {
                [0, 0, 0, 0]
            };
            sem_verde[o..o + 4].copy_from_slice(&c);
        }
    }
    let verde_sobre_vermelho = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| texel(art, w, x, y) == GREEN && texel(&sem_verde, w, x, y) == RED)
        .count();
    assert!(
        verde_sobre_vermelho > 0,
        "CONTROLO: a fixtura não tem verde POR CIMA de vermelho — a sonda mediria zero por não ter fenómeno"
    );

    // (b)(c) assimetria nos dois eixos.
    let assim_y = (0..h / 2)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| texel(art, w, x, y) != texel(art, w, x, h - 1 - y))
        .count();
    let assim_x = (0..h)
        .flat_map(|y| (0..w / 2).map(move |x| (x, y)))
        .filter(|&(x, y)| texel(art, w, x, y) != texel(art, w, w - 1 - x, y))
        .count();
    assert!(assim_y > 0, "CONTROLO: a arte é simétrica em y");
    assert!(assim_x > 0, "CONTROLO: a arte é simétrica em x");

    // (d) conteúdo opaco no topo.
    let opaco_no_topo = (0..h / 8)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| texel(art, w, x, y)[3] == 255)
        .count();
    assert!(
        opaco_no_topo > 0,
        "CONTROLO: o topo da arte é todo transparente"
    );

    println!(
        "  [controlo da fixtura] verde-sobre-vermelho={verde_sobre_vermelho} texels · assimetria_y={assim_y} · assimetria_x={assim_x} · opacos_no_topo={opaco_no_topo}"
    );
}

/// A lei pela PORTA REAL — nunca aritmética minha.
fn law_of(kind: TileKind, size: [f64; 2], art_px: [u32; 2]) -> TileLaw {
    let mut fill = PatternFill::new(PatternSource::Shape(1), size, Rgba8::new(0, 0, 0, 255));
    fill.kind = kind;
    fill.law(art_px)
}

/// `over` em alfa RETO — **cópia verbatim da lei do assador**, e só existe para a REPRISE abaixo,
/// que é validada byte-a-byte contra o ladrilho do produto.
fn over(dst: &mut [u8], src: &[u8]) -> bool {
    if dst[3] == 0 {
        dst.copy_from_slice(src);
        return true;
    }
    if src[3] == 0 {
        return false;
    }
    let (sa, da) = (f32::from(src[3]) / 255.0, f32::from(dst[3]) / 255.0);
    let keep = da * (1.0 - sa);
    let out_a = sa + keep;
    for i in 0..3 {
        let c = (f32::from(src[i]) * sa + f32::from(dst[i]) * keep) / out_a;
        dst[i] = c.round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    true
}

/// A REPRISE do assado com PROVENIÊNCIA: quem escreveu por último em cada texel.
///
/// ⚠️ O valor dela é a **validação**: se os bytes não baterem com os do produto, a atribuição de
/// Q3 não vale nada e a sonda diz-o em voz alta.
fn replay(art: &[u8], aw: u32, ah: u32, law: &TileLaw, tw: u32, th: u32) -> (Vec<u8>, Vec<i64>) {
    let cells = law.cells();
    let cell = [tw / cells[0].max(1), th / cells[1].max(1)];
    let mut rgba = vec![0u8; (tw as usize) * (th as usize) * 4];
    let mut who = vec![-1i64; (tw as usize) * (th as usize)];
    for row in 0..cells[1] {
        for col in 0..cells[0] {
            let s = law.shift_px(cell, col, row);
            let origin = [col * cell[0] + s[0], row * cell[1] + s[1]];
            for y in 0..ah {
                let dy = (origin[1] + y) % th;
                for x in 0..aw {
                    let dx = (origin[0] + x) % tw;
                    let so = ((y as usize) * (aw as usize) + x as usize) * 4;
                    let di = (dy as usize) * (tw as usize) + dx as usize;
                    if over(&mut rgba[di * 4..di * 4 + 4], &art[so..so + 4]) {
                        who[di] = i64::from(row * cells[0] + col);
                    }
                }
            }
        }
    }
    (rgba, who)
}

struct Row0 {
    total: usize,
    changed: usize,
    covered_opaque: usize,
    filled_transparent: usize,
    by_neighbour: usize,
    mixture: usize,
}

/// Compara a **cópia da linha 0** (o motivo, na origem do ladrilho) com a arte original.
fn measure_row0(
    tile_rgba: &[u8],
    tw: u32,
    th: u32,
    who: &[i64],
    art: &[u8],
    aw: u32,
    ah: u32,
) -> Row0 {
    let mut r = Row0 {
        total: (aw as usize) * (ah as usize),
        changed: 0,
        covered_opaque: 0,
        filled_transparent: 0,
        by_neighbour: 0,
        mixture: 0,
    };
    for y in 0..ah.min(th) {
        for x in 0..aw.min(tw) {
            let a = texel(art, aw, x, y);
            let t = texel(tile_rgba, tw, x, y);
            if a == t {
                continue;
            }
            r.changed += 1;
            if a[3] == 255 {
                r.covered_opaque += 1;
            } else if a[3] == 0 {
                r.filled_transparent += 1;
            }
            // quem escreveu por último aqui? célula 0 = a própria cópia da linha 0.
            if who[(y as usize) * (tw as usize) + x as usize] > 0 {
                r.by_neighbour += 1;
            } else {
                r.mixture += 1;
            }
        }
    }
    r
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

/// **Q1** — o Hex altera o motivo? (com o Grid como CONTROLO byte-a-byte)
#[test]
#[ignore = "sonda de medição, não é gate"]
fn q1_does_hex_alter_the_motif() {
    let (aw, ah) = (96u32, 96u32);
    let art = build_art(aw, ah);
    println!("\n=== Q1 — arte {aw}x{ah} (1:1), PatternFill.size = [1,1], gap autorado = [0,0] ===");
    assert_fixture_has_the_phenomenon(&art, aw, ah);

    // CONTROLO: a grade devolve a arte AO BYTE.
    let grid = law_of(TileKind::Grid, [1.0, 1.0], [aw, ah]);
    let g = bake(&art, aw, ah, &grid).expect("grid bake");
    println!(
        "  GRID  law={:?} cells={:?} tile={}x{}",
        grid, g.cells, g.width, g.height
    );
    assert_eq!(
        (g.width, g.height),
        (aw, ah),
        "CONTROLO: o ladrilho da grade não tem o tamanho da arte"
    );
    assert_eq!(
        g.rgba, art,
        "CONTROLO: a grade NÃO é byte-idêntica à arte — a sonda está errada, não o produto"
    );
    println!("  GRID  => byte-idêntico à arte ✓ (o controlo passa)");

    // ⭐ CONTROLO DE ISOLAMENTO: o `BrickRow` tem o MESMO ladrilho de 2 células e o MESMO meio
    // passo horizontal — e NÃO deriva o período vertical. Se ele não altera o motivo, o que altera
    // é a lei do período (`hex_row_period`), não o reticulado desfasado.
    let brick = law_of(TileKind::BrickRow, [1.0, 1.0], [aw, ah]);
    let b = bake(&art, aw, ah, &brick).expect("brick bake");
    let (brep, bwho) = replay(&art, aw, ah, &brick, b.width, b.height);
    assert_eq!(brep, b.rgba, "REPRISE != produto (brick)");
    let br = measure_row0(&b.rgba, b.width, b.height, &bwho, &art, aw, ah);
    println!(
        "  BRICK law={:?} cells={:?} tile={}x{} · cell={}x{} · texels alterados na cópia da linha 0: {} de {} ({:.2}%)",
        brick,
        b.cells,
        b.width,
        b.height,
        b.width / brick.cells()[0],
        b.height / brick.cells()[1],
        br.changed,
        br.total,
        pct(br.changed, br.total)
    );

    // O HEX.
    let hex = law_of(TileKind::Hex, [1.0, 1.0], [aw, ah]);
    let t = bake(&art, aw, ah, &hex).expect("hex bake");
    let cells = hex.cells();
    let cell_h = t.height / cells[1];
    let overlap = ah as i64 - i64::from(cell_h);
    let (rep, who) = replay(&art, aw, ah, &hex, t.width, t.height);
    assert_eq!(
        rep, t.rgba,
        "a REPRISE não reproduz o ladrilho do produto — a atribuição de Q3 seria inválida"
    );
    let r = measure_row0(&t.rgba, t.width, t.height, &who, &art, aw, ah);
    println!(
        "  HEX   law={:?} cells={:?} tile={}x{} · gap_px={:?} · cell={}x{} · sobreposição vertical={overlap} px ({:.1}% da altura da arte)",
        hex,
        t.cells,
        t.width,
        t.height,
        hex.gap_px,
        t.width / cells[0],
        cell_h,
        100.0 * overlap as f64 / f64::from(ah)
    );
    println!(
        "  HEX   texels da cópia da linha 0 diferentes da arte: {} de {} ({:.2}%)",
        r.changed,
        r.total,
        pct(r.changed, r.total)
    );
    println!(
        "        destes: {} caíram sobre texel OPACO da arte · {} sobre texel TRANSPARENTE",
        r.covered_opaque, r.filled_transparent
    );
}

/// **Q2** — a sobreposição é função do ASPECTO da arte?
#[test]
#[ignore = "sonda de medição, não é gate"]
fn q2_is_the_overlap_a_function_of_the_aspect() {
    let cases: [(&str, u32, u32); 5] = [
        ("2:1", 96, 48),
        ("3:2", 96, 64),
        ("1:1", 96, 96),
        ("2:3", 64, 96),
        ("1:2", 48, 96),
    ];
    for (label, regime) in [
        ("size QUADRADO [1,1]", 0u8),
        ("size SEGUE o aspecto da arte", 1u8),
    ] {
        println!("\n=== Q2 — {label} ===");
        println!(
            "  {:<6} {:>9} {:>10} {:>12} {:>10} {:>14} {:>9} {:>12}",
            "aspec",
            "arte px",
            "gap_px[1]",
            "sobrepos px",
            "% altura",
            "ladrilho",
            "mudados",
            "% da arte"
        );
        for (name, aw, ah) in cases {
            let art = build_art(aw, ah);
            if regime == 0 {
                assert_fixture_has_the_phenomenon(&art, aw, ah);
            }
            let size = if regime == 0 {
                [1.0, 1.0]
            } else {
                [f64::from(aw) / f64::from(ah), 1.0]
            };
            let hex = law_of(TileKind::Hex, size, [aw, ah]);
            let t = bake(&art, aw, ah, &hex).expect("hex bake");
            let cells = hex.cells();
            let cell_h = t.height / cells[1];
            let overlap = i64::from(ah) - i64::from(cell_h);
            let (rep, who) = replay(&art, aw, ah, &hex, t.width, t.height);
            assert_eq!(rep, t.rgba, "REPRISE != produto em {name}");
            let r = measure_row0(&t.rgba, t.width, t.height, &who, &art, aw, ah);
            println!(
                "  {:<6} {:>9} {:>10} {:>12} {:>9.1}% {:>14} {:>9} {:>11.2}%",
                name,
                format!("{aw}x{ah}"),
                hex.gap_px[1],
                overlap,
                100.0 * overlap as f64 / f64::from(ah),
                format!("{}x{}", t.width, t.height),
                r.changed,
                pct(r.changed, r.total)
            );
        }
    }
}

/// **Q3** — a acusação de z tem sujeito? Quem cobre quem, nos texels alterados.
#[test]
#[ignore = "sonda de medição, não é gate"]
fn q3_who_covers_whom() {
    let (aw, ah) = (96u32, 96u32);
    let art = build_art(aw, ah);
    println!("\n=== Q3 — quem cobre quem (arte {aw}x{ah}, size [1,1]) ===");
    assert_fixture_has_the_phenomenon(&art, aw, ah);
    let hex = law_of(TileKind::Hex, [1.0, 1.0], [aw, ah]);
    let t = bake(&art, aw, ah, &hex).expect("hex bake");
    let (rep, who) = replay(&art, aw, ah, &hex, t.width, t.height);
    assert_eq!(rep, t.rgba, "REPRISE != produto");
    let r = measure_row0(&t.rgba, t.width, t.height, &who, &art, aw, ah);

    // Que COR ficou, nos texels alterados?
    let mut cores: std::collections::BTreeMap<[u8; 4], usize> = std::collections::BTreeMap::new();
    let mut linhas: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for y in 0..ah {
        for x in 0..aw {
            let a = texel(&art, aw, x, y);
            let tt = texel(&t.rgba, t.width, x, y);
            if a != tt {
                *cores.entry(tt).or_default() += 1;
                *linhas.entry(y).or_default() += 1;
            }
        }
    }
    println!(
        "  alterados={} de {} ({:.2}%) · escritos pela cópia VIZINHA (célula 1)={} · outra proveniência={}",
        r.changed,
        r.total,
        pct(r.changed, r.total),
        r.by_neighbour,
        r.mixture
    );
    println!("  cor RESULTANTE nos texels alterados (RED={RED:?} GREEN={GREEN:?} BLUE={BLUE:?}):");
    for (c, n) in &cores {
        // ⚠️ O NOME sai da PROVENIÊNCIA, nunca da cor: `by_neighbour` já disse quem escreveu.
        let nome = if *c == RED {
            "RED"
        } else if *c == GREEN {
            "GREEN"
        } else if *c == BLUE {
            "BLUE"
        } else {
            "MISTURA / outra"
        };
        println!("     {c:?} x{n:>6}  {nome}");
    }
    let ys: Vec<u32> = linhas.keys().copied().collect();
    println!(
        "  faixa de LINHAS alteradas: y de {:?} a {:?} (a arte tem {ah} linhas) — {} linhas tocadas",
        ys.first(),
        ys.last(),
        ys.len()
    );
    // As BANDAS: linhas contíguas alteradas.
    let mut bandas: Vec<(u32, u32, usize)> = Vec::new();
    for y in ys {
        let n = linhas[&y];
        match bandas.last_mut() {
            Some(b) if b.1 + 1 == y => {
                b.1 = y;
                b.2 += n;
            }
            _ => bandas.push((y, y, n)),
        }
    }
    for (y0, y1, n) in &bandas {
        // que cor domina a banda, e o que a arte tinha lá.
        let mut antes: std::collections::BTreeMap<[u8; 4], usize> =
            std::collections::BTreeMap::new();
        let mut depois: std::collections::BTreeMap<[u8; 4], usize> =
            std::collections::BTreeMap::new();
        for y in *y0..=*y1 {
            for x in 0..aw {
                let a = texel(&art, aw, x, y);
                let tt = texel(&t.rgba, t.width, x, y);
                if a != tt {
                    *antes.entry(a).or_default() += 1;
                    *depois.entry(tt).or_default() += 1;
                }
            }
        }
        println!("  banda y={y0}..={y1} ({n} texels): a arte tinha {antes:?} -> ficou {depois:?}");
    }
}
