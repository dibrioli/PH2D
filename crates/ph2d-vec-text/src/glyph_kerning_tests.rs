//! Gates do KERNING (prompt 49, W3) — filho de `glyph` via `#[path]`, como os irmãos, para alcançar
//! a `glyph_frame` e a `line_advance`.

use super::*;
use ph2d_vec_scene::VecVertex;

/// A fixture do oráculo: o `vectorcraft-cli` desenhou cada palavra e cada prefixo dela
/// (`docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc_kerning.py`).
const ORACLE: &str = include_str!("../tests/fixtures/vectorcraft_kerning.tsv");

fn inter() -> VariableFont {
    VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida")
}

fn lay(size: f64, kerning: bool) -> TextLayout {
    TextLayout {
        size,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
        wrap_width: None,
        kerning,
    }
}

fn pens(font: &VariableFont, text: &str, l: &TextLayout, axes: &[(AxisTag, f32)]) -> Vec<f64> {
    let mut out = Vec::new();
    walk_glyphs(
        font,
        text,
        l,
        axes,
        &TextPlacement::At([0.0, 0.0]),
        |_, _, p, _| out.push(p[0]),
    );
    out
}

/// `(palavra, wght, pens)` de cada linha da fixture.
fn oracle_rows() -> Vec<(&'static str, f32, Vec<f64>)> {
    ORACLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut c = l.split('\t');
            let word = c.next().expect("palavra");
            let wght = c.next().expect("wght").parse().expect("wght");
            let pens = c
                .next()
                .expect("pens")
                .split(',')
                .map(|p| p.parse().expect("pen"))
                .collect();
            (word, wght, pens)
        })
        .collect()
}

/// ⭐ A lei do W3: com o kerning ligado, o pen de cada glifo cai onde o VectorCraft o pôs, sobre a
/// MESMA fonte. A `font-size = unitsPerEm` da fixture faz `scale = 1`: o pen é unidade de design.
#[test]
fn the_kerned_pens_match_the_vectorcraft_oracle() {
    let font = inter();
    let upem = f64::from(font.units_per_em());
    let rows = oracle_rows();
    assert!(
        rows.len() >= 10,
        "controlo: a fixture trouxe {} linhas",
        rows.len()
    );
    let mut nominal_misses = 0;
    for (word, wght, want) in &rows {
        let axes = [(AxisTag::WEIGHT, *wght)];
        let got = pens(&font, word, &lay(upem, true), &axes);
        assert_eq!(got.len(), want.len(), "{word}: um glifo por char");
        for (i, (g, w)) in got.iter().zip(want).enumerate() {
            // O VC escreve coordenadas inteiras a este tamanho; a grelha do moldador é 1/64.
            assert!(
                (g - w).abs() <= 0.5,
                "{word}[{i}] a {wght}: PH2D {g} · VectorCraft {w}"
            );
        }
        let plain = pens(&font, word, &lay(upem, false), &axes);
        if plain.iter().zip(want).any(|(g, w)| (g - w).abs() > 0.5) {
            nominal_misses += 1;
        }
    }
    // ⛔ Controlo: a fixture MEDE kerning — sem ele, a maioria das palavras discorda do oráculo.
    // Se este número cair a zero, a fixture deixou de ter pares e o gate acima passou a ser vácuo.
    assert!(
        nominal_misses >= rows.len() / 2,
        "só {nominal_misses} de {} palavras discordam do oráculo com o kerning DESLIGADO",
        rows.len()
    );
}

/// Um texto sem pares sai **byte a byte** igual com a chave ligada ou desligada — em toda
/// instância (a 650 os avanços são fracionários, e é aí que um resto de arredondamento vazaria), em
/// todo alinhamento, com caixa e sobre um caminho.
#[test]
fn text_without_pairs_is_byte_identical() {
    let font = inter();
    let text = "HIM NUH\nMINIM HUM";
    for wght in [400.0, 650.0] {
        let axes = [(AxisTag::WEIGHT, wght)];
        for line in text.split('\n') {
            let k = font.kerning(line, &axes).expect("latim");
            assert!(
                k.iter().all(|a| *a == Default::default()),
                "{line}: precondição, sem pares: {k:?}"
            );
        }
        let path = ArcPath::from_contour(
            &[
                VecVertex::corner([0.0, 0.0]),
                VecVertex::corner([30.0, 4.0]),
            ],
            false,
        )
        .expect("reta");
        let placements = [
            TextPlacement::At([3.25, -1.5]),
            TextPlacement::OnPath {
                path: &path,
                start_offset: 0.5,
                flip: false,
            },
        ];
        for align in [TextAlign::Left, TextAlign::Center, TextAlign::Right] {
            for wrap in [None, Some(4.0)] {
                for placement in &placements {
                    let l = |kerning| TextLayout {
                        align,
                        wrap_width: wrap,
                        tracking: 0.03,
                        ..lay(1.0, kerning)
                    };
                    let bits = |kerning| {
                        let mut out = Vec::new();
                        for p in text_to_vec_paths(
                            &font,
                            text,
                            &l(kerning),
                            &axes,
                            placement,
                            &None,
                            &None,
                        ) {
                            p.verts_all().for_each(|v| {
                                out.extend(
                                    v.anchor
                                        .iter()
                                        .chain(&v.in_handle)
                                        .chain(&v.out_handle)
                                        .map(|x| x.to_bits()),
                                );
                            });
                        }
                        out
                    };
                    let off = bits(false);
                    assert!(!off.is_empty(), "controlo: o texto desenhou glifos");
                    assert_eq!(
                        off,
                        bits(true),
                        "{wght} {align:?} {wrap:?}: a chave mudou um texto sem pares"
                    );
                }
            }
        }
    }
}

/// O quebrador mede com o kerning: uma caixa onde a linha moldada cabe e a nominal não, parte só sem
/// a chave. E a linha que ele aceitou acaba onde o desenhador a acaba (o caret na ponta é o pen
/// depois do último glifo).
#[test]
fn the_wrapper_breaks_at_the_kerned_width() {
    let font = inter();
    let text = "AVAVAV AVAVAV";
    let kerned = unwrapped_block_width(&font, text, &lay(1.0, true), &[]);
    let plain = unwrapped_block_width(&font, text, &lay(1.0, false), &[]);
    assert!(
        plain - kerned > 0.5,
        "controlo: o kerning encurta a linha ({plain} → {kerned})"
    );
    let w = kerned + 0.01;
    let boxed = |kerning| TextLayout {
        wrap_width: Some(w),
        ..lay(1.0, kerning)
    };
    let at = TextPlacement::At([0.0, 0.0]);
    assert_eq!(
        wrapped_lines(&font, text, &boxed(true), &[], &at),
        vec![text]
    );
    assert_eq!(wrapped_lines(&font, text, &boxed(false), &[], &at).len(), 2);

    let mut end = 0.0;
    walk_glyphs(&font, text, &boxed(true), &[], &at, |_, _, p, _| end = p[0]);
    let last = font.glyph_for_char('V').expect("V");
    let v_adv = f64::from(font.advance(last, &[]).unwrap_or(0.0)) / f64::from(font.units_per_em());
    let caret = caret_x_offset(&font, text, &boxed(true), &[]);
    assert!(
        (end + v_adv - caret).abs() < 1e-12,
        "o caret ({caret}) não está onde o último V acaba ({})",
        end + v_adv
    );
}

/// Sobre um caminho reto o kerning entra igual: o referencial de cada glifo recua para o pen moldado.
#[test]
fn text_on_a_path_carries_the_kerning() {
    let font = inter();
    let path = ArcPath::from_contour(
        &[
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([10.0, 0.0]),
        ],
        false,
    )
    .expect("reta");
    let on = TextPlacement::OnPath {
        path: &path,
        start_offset: 0.0,
        flip: false,
    };
    for kerning in [false, true] {
        let l = lay(1.0, kerning);
        let straight = pens(&font, "AVATAR", &l, &[]);
        let mut along = Vec::new();
        walk_glyphs(&font, "AVATAR", &l, &[], &on, |_, _, p, adv| {
            along.push(glyph_frame(&on, p, adv).expect("dentro do caminho").origin[0]);
        });
        for (s, a) in straight.iter().zip(&along) {
            assert!(
                (s - a).abs() < 1e-9,
                "kerning={kerning}: em caminho {a}, reto {s}"
            );
        }
    }
    assert!(
        pens(&font, "AVATAR", &lay(1.0, true), &[])[5]
            < pens(&font, "AVATAR", &lay(1.0, false), &[])[5] - 0.1,
        "controlo: o kerning move o último glifo de AVATAR"
    );
}

/// O par lê a instância do texto: o kerning de `AV` muda com o peso.
#[test]
fn the_kerning_follows_the_weight_axis() {
    let font = inter();
    let at = |w: f32| font.kerning("AV", &[(AxisTag::WEIGHT, w)]).expect("latim")[0].advance;
    assert!(
        (at(400.0) - at(900.0)).abs() > 1.0,
        "AV a 400: {} · a 900: {}",
        at(400.0),
        at(900.0)
    );
}

/// Sonda (`--release -- --ignored --nocapture`): o custo do kerning num re-cook — o que uma tecla
/// paga num parágrafo com caixa (o quebrador mede cada linha candidata). Variantes no MESMO
/// processo, intercaladas, o MÍNIMO de cada uma (`docs/DevOps/MEDIR_VELOCIDADE.md`). Medido 11/10,
/// 216 chars, caixa de 12 em: 0,333 ms → 0,702 ms (2,1×) com as tabelas do moldador guardadas na
/// fonte; refazê-las por linha dava 0,964 ms (2,85×).
/// E o Motion, que publica o `source.text` a CADA quadro: 17 chars numa linha, 3,4 → 10,9 µs
/// (load 12) — sem cache por quadro, porque não há o que poupar contra 16 ms.
#[test]
#[ignore = "sonda de custo; corre em release"]
fn measure_the_kerning_against_advance_only() {
    let font = inter();
    let text = "Typography AVATAR WAVE To Wa fi LT Yo Kerning minimum ".repeat(4);
    let mut best = [f64::INFINITY; 2];
    for round in 0..40 {
        // A ordem roda a cada volta: nenhuma variante corre sempre com a cache quente da outra.
        for kerning in [round % 2 == 1, round % 2 == 0] {
            let l = TextLayout {
                wrap_width: Some(12.0),
                ..lay(1.0, kerning)
            };
            let t0 = std::time::Instant::now();
            let n = text_to_vec_paths(
                &font,
                &text,
                &l,
                &[],
                &TextPlacement::At([0.0, 0.0]),
                &None,
                &None,
            )
            .len();
            let slot = &mut best[usize::from(kerning)];
            *slot = slot.min(t0.elapsed().as_secs_f64() * 1e3);
            assert!(n > 100, "controlo: o parágrafo desenhou {n} glifos");
        }
    }
    // O caso do Motion: o `source.text` publica a cada QUADRO, uma linha curta sem caixa.
    let short = "Motion Text AV To";
    let mut frame = [f64::INFINITY; 2];
    for round in 0..200 {
        for kerning in [round % 2 == 1, round % 2 == 0] {
            let t0 = std::time::Instant::now();
            let mut n = 0;
            walk_glyphs(
                &font,
                short,
                &lay(1.0, kerning),
                &[],
                &TextPlacement::At([0.0, 0.0]),
                |_, _, _, _| n += 1,
            );
            let slot = &mut frame[usize::from(kerning)];
            *slot = slot.min(t0.elapsed().as_secs_f64() * 1e6);
            assert_eq!(n, short.chars().count());
        }
    }
    println!(
        "linha do Motion ({} chars, por quadro): {:.1} µs → {:.1} µs",
        short.len(),
        frame[0],
        frame[1]
    );
    println!(
        "re-cook de {} chars com caixa: advance-only {:.3} ms · com kerning {:.3} ms ({:.2}×)",
        text.len(),
        best[0],
        best[1],
        best[1] / best[0]
    );
}

/// Sonda (`-- --ignored`): o ANTES/DEPOIS para o dono, desenhado pelo MESMO cozedor — as mesmas
/// palavras sem e com kerning, uma por cima da outra, em `target/kerning_antes_depois.svg`.
#[test]
#[ignore = "gera a imagem do antes/depois"]
fn draw_the_before_and_after_for_the_owner() {
    use std::fmt::Write as _;
    let font = inter();
    let text = "AVATAR WAVE\nTo Wa Ty LT Yo";
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.4 -4.0 9 5.7" width="1800" height="1140"><rect x="-0.4" y="-4.0" width="9" height="5.7" fill="white"/>"#,
    );
    for (kerning, y) in [(false, 2.9), (true, 0.0)] {
        let paths = text_to_vec_paths(
            &font,
            text,
            &lay(1.0, kerning),
            &[],
            &TextPlacement::At([0.0, y]),
            &None,
            &None,
        );
        let mut d = String::new();
        for p in &paths {
            for c in std::iter::once((&p.verts, p.closed))
                .chain(p.subpaths.iter().map(|s| (&s.verts, s.closed)))
            {
                let (verts, closed) = c;
                let n = verts.len();
                for i in 0..n + usize::from(closed) {
                    let v = &verts[i % n];
                    if i == 0 {
                        let _ = write!(d, "M{} {}", v.anchor[0], -v.anchor[1]);
                    } else {
                        let u = &verts[i - 1];
                        let _ = write!(
                            d,
                            "C{} {} {} {} {} {}",
                            u.out_handle[0],
                            -u.out_handle[1],
                            v.in_handle[0],
                            -v.in_handle[1],
                            v.anchor[0],
                            -v.anchor[1]
                        );
                    }
                }
                d.push('Z');
            }
        }
        let fill = if kerning { "#1f6fd1" } else { "#555" };
        let _ = write!(svg, r#"<path d="{d}" fill="{fill}" fill-rule="nonzero"/>"#);
    }
    svg.push_str("</svg>");
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/kerning_antes_depois.svg");
    std::fs::write(&out, svg).expect("escreve");
    println!("{}", out.display());
}

/// O `XPlacement` do par desloca o glifo SEM mover o pen: na fonte mínima o `B` desenha-se em
/// `500 − 50 + 30`, e o glifo seguinte (o 2.º `A`) parte de `500 − 50 + 500`, sem os `30`.
#[test]
fn a_placement_moves_the_glyph_and_not_the_pen() {
    let font = VariableFont::new(
        include_bytes!("../../ph2d-vector-font/tests/fixtures/kern_xplacement.ttf").to_vec(),
    )
    .expect("fixture");
    assert_eq!(
        pens(&font, "ABA", &lay(1000.0, true), &[]),
        vec![0.0, 480.0, 950.0]
    );
    assert_eq!(
        pens(&font, "ABA", &lay(1000.0, false), &[]),
        vec![0.0, 500.0, 1000.0]
    );
}
