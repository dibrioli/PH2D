//! Gates do LAYOUT de texto ([`crate::vec_glyph`]) — módulo irmão pelo teto de 600 LOC
//! (HR-18). Fica FILHO de `vec_glyph` (via `#[path]`), então alcança o que é privado lá
//! dentro — a `glyph_frame`, que é a porta que estes gates existem para prender.

use super::*;
use ph2d_vec_scene::{Rgba8, VecVertex};

fn black() -> Paint {
    Paint::solid(Rgba8::new(0, 0, 0, 255))
}

/// Texto reto na origem — o assentamento de quase todo gate deste módulo.
fn at0() -> TextPlacement<'static> {
    TextPlacement::At([0.0, 0.0])
}

fn inter() -> VariableFont {
    VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida")
}

fn lay(size: f64) -> TextLayout {
    TextLayout {
        size,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
    }
}

fn run(font: &VariableFont, text: &str, l: &TextLayout, p: &TextPlacement<'_>) -> Vec<VecPath> {
    text_to_vec_paths(font, text, l, &[], p, &Some(black()), &None)
}

/// Uma reta de `(0,0)` a `(len,0)` como caminho de arco.
fn line_path(len: f64) -> ArcPath {
    ArcPath::from_contour(
        &[VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        false,
    )
    .expect("reta")
}

/// Um círculo de raio `r` centrado na origem, em quatro cúbicas (percurso ANTI-HORÁRIO).
fn circle_path(r: f64) -> ArcPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    let t = [[0.0, K * r], [-K * r, 0.0], [0.0, -K * r], [K * r, 0.0]];
    let verts: Vec<VecVertex> = (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - t[i][0], p[i][1] - t[i][1]],
            out_handle: [p[i][0] + t[i][0], p[i][1] + t[i][1]],
            kind: ph2d_vec_scene::VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect();
    ArcPath::from_contour(&verts, true).expect("círculo")
}

/// Um quarto de círculo de raio `r`, do eixo +X ao +Y — um caminho CURVO aberto, com
/// tangente bem definida nas duas pontas (ao contrário da reta, que o documento guarda na
/// forma degenerada e cuja derivada estaciona ali).
fn quarter_arc(r: f64) -> ArcPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let a = VecVertex {
        anchor: [r, 0.0],
        in_handle: [r, -K * r],
        out_handle: [r, K * r],
        kind: ph2d_vec_scene::VertexKind::Smooth,
        corner_radius: 0.0,
    };
    let b = VecVertex {
        anchor: [0.0, r],
        in_handle: [K * r, r],
        out_handle: [-K * r, r],
        kind: ph2d_vec_scene::VertexKind::Smooth,
        corner_radius: 0.0,
    };
    ArcPath::from_contour(&[a, b], false).expect("arco")
}

/// A menor e a maior distância de uma âncora ao centro — a "banda" que o texto ocupa.
fn radial_band(paths: &[VecPath]) -> (f64, f64) {
    paths
        .iter()
        .flat_map(|p| p.verts_all())
        .map(|v| v.anchor[0].hypot(v.anchor[1]))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), d| {
            (lo.min(d), hi.max(d))
        })
}

/// **Numa RETA, texto em caminho É texto normal.** O oráculo mais forte que existe para
/// esta feature, e o único que a compara com uma resposta que já era certa antes dela: se
/// a âncora ao meio, o recuo de meio avanço, a inversão de arco ou a normal estiverem
/// trocados, isto não fecha — e nenhum deles é visível num gate que só olhe para a curva.
///
/// A tolerância existe porque a posição vem de uma INVERSÃO de comprimento de arco
/// (bisseção); o resíduo medido aqui é ~1e-11.
#[test]
fn on_a_straight_path_the_text_lands_where_plain_layout_puts_it() {
    let font = inter();
    let l = lay(1.0);
    let straight = run(&font, "Path", &l, &at0());
    let path = line_path(100.0);
    let ridden = run(
        &font,
        "Path",
        &l,
        &TextPlacement::OnPath {
            path: &path,
            start_offset: 0.0,
            flip: false,
        },
    );
    assert_eq!(straight.len(), ridden.len(), "mesmo número de glyphs");
    for (a, b) in straight.iter().zip(&ridden) {
        for (va, vb) in a.verts_all().zip(b.verts_all()) {
            assert!(
                (va.anchor[0] - vb.anchor[0]).abs() < 1e-6
                    && (va.anchor[1] - vb.anchor[1]).abs() < 1e-6,
                "{:?} contra {:?}",
                va.anchor,
                vb.anchor
            );
        }
    }
}

/// **Num CÍRCULO o texto ocupa uma BANDA, não uma faixa.** O oráculo modela o que se vê:
/// reto, as âncoras espalham-se pela LARGURA da palavra; no círculo elas ficam todas a uma
/// distância do centro que só varia pela ALTURA da fonte. Um gate que comparasse com
/// `frame_at` seria um espelho da função sob teste.
#[test]
fn on_a_circle_the_word_wraps_into_a_narrow_radial_band() {
    let font = inter();
    let l = lay(1.0);
    let word = "PATH PATH";
    let r = 6.0;
    let path = circle_path(r);
    let ridden = run(
        &font,
        word,
        &l,
        &TextPlacement::OnPath {
            path: &path,
            start_offset: 0.0,
            flip: false,
        },
    );
    assert!(ridden.len() >= 6, "a palavra coube: {}", ridden.len());
    let (lo, hi) = radial_band(&ridden);
    // A banda é a altura da fonte (descida a subida), não a largura da palavra (~4.9).
    assert!(
        hi - lo < 1.2,
        "banda radial {:.3} (de {lo:.3} a {hi:.3}) — devia ser da ordem da altura da fonte",
        hi - lo
    );
    // E ela senta NO raio: a baseline está em R, as letras sobem para dentro (anti-horário
    // ⇒ a esquerda da marcha aponta para o centro).
    assert!(
        (hi - r).abs() < 0.15,
        "o topo da banda ({hi:.3}) devia encostar no raio {r}"
    );
    assert!(lo < r - 0.4, "as letras sobem para dentro: {lo:.3}");
}

/// **A entrelinha vira deslocamento pela NORMAL** — a 2ª linha corre paralela à 1ª, curva
/// junto, e não precisou de uma linha de código a dizê-lo.
///
/// ⚠️ Este gate nasceu a exigir a 2ª linha MAIS PERTO do centro e ficou vermelho — e o
/// código estava certo, como no gate irmão do [`ph2d_vec_scene::text_path`]. Num círculo
/// **anti-horário** a subida da letra aponta para o centro, então "abaixo da baseline" é
/// para FORA: as linhas empilham-se afastando-se do centro. É a terceira vez que a
/// intuição de dentro/fora falha e a de *"à esquerda do sentido de marcha"* acerta —
/// dentro e fora dependem do winding, esquerda não.
///
/// O oráculo verdadeiro é o PARALELISMO: as duas bandas deslocam-se pela MESMA quantidade,
/// que é exatamente a entrelinha do texto reto. Um deslocamento que variasse ao longo da
/// palavra (letras a divergir) passaria por qualquer teste de "está mais para lá".
#[test]
fn the_second_line_rides_parallel_to_the_first() {
    let font = inter();
    let l = lay(1.0);
    let path = circle_path(9.0);
    let on = TextPlacement::OnPath {
        path: &path,
        start_offset: 0.0,
        flip: false,
    };
    let one = run(&font, "HH", &l, &on);
    let two = run(&font, "HH\nHH", &l, &on);
    assert_eq!(two.len(), 2 * one.len(), "as duas linhas desenharam");
    let (lo1, hi1) = radial_band(&two[..one.len()]);
    let (lo2, hi2) = radial_band(&two[one.len()..]);
    let step = l.size * l.line_height;
    // As DUAS bordas da banda andam o mesmo tanto: é isso que "paralela" significa, e é o
    // que uma divergência ao longo da palavra quebraria.
    assert!(
        (lo2 - lo1 - step).abs() < 0.02 && (hi2 - hi1 - step).abs() < 0.02,
        "banda 1 ({lo1:.3}, {hi1:.3}) contra banda 2 ({lo2:.3}, {hi2:.3}), passo {step}"
    );
    assert!(
        lo2 > hi1,
        "e as linhas não se sobrepõem: {lo2:.3} contra {hi1:.3}"
    );
}

/// **Um glyph cuja âncora cai FORA do caminho não é desenhado** (a regra normativa do
/// `<textPath>`): saturar nas pontas empilharia as letras que sobram num montinho, que é o
/// desenho que este `None` existe para evitar.
///
/// ⚠️ **A fixture é um ARCO, e a primeira era uma RETA — que não continha o fenômeno.** Numa
/// reta o documento guarda a cúbica degenerada `(P0,P0,P3,P3)`, cuja derivada é nula nas
/// duas pontas; a saturação caía em `t = 1`, a tangente vinha nula e o glyph era descartado
/// **por acidente**. O gate ficava verde com o guard inteiro apagado. Numa curva de verdade
/// a tangente em `t = 1` é perfeitamente boa e só o guard separa truncar de empilhar.
#[test]
fn a_glyph_whose_anchor_falls_off_the_path_is_dropped_not_piled_up() {
    let font = inter();
    let l = lay(1.0);
    let long = "PATHPATHPATH";
    let full = run(&font, long, &l, &at0());
    // Um QUARTO de círculo grande: curto para a palavra (arco ~2.36) e com tangente
    // definida na ponta — é essa segunda metade que a reta não tinha.
    let quarter = quarter_arc(1.5);
    let ridden = run(
        &font,
        long,
        &l,
        &TextPlacement::OnPath {
            path: &quarter,
            start_offset: 0.0,
            flip: false,
        },
    );
    assert!(
        !ridden.is_empty() && ridden.len() < full.len(),
        "truncou: {} de {}",
        ridden.len(),
        full.len()
    );
    // E nenhuma letra se empilhou na ponta: os centros dos glyphs desenhados são todos
    // distintos (empilhar poria vários no MESMO ponto, que é como o defeito se vê).
    let mut centers: Vec<[f64; 2]> = ridden.iter().map(path_center).collect();
    centers.sort_by(|a, b| a[0].total_cmp(&b[0]));
    for w in centers.windows(2) {
        let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        assert!(d > 0.05, "dois glyphs no mesmo sítio: {:?}", w);
    }
}

/// **O pen avança MESMO nos glyphs que não são desenhados** — e o caso que o prova é o
/// gesto mais comum da feature: texto **centrado** num caminho.
///
/// Centrar põe o pen a começar em `−largura/2`, ou seja a primeira metade da palavra tem
/// âncora em arco NEGATIVO e é descartada. Se o pen só avançasse nos glyphs desenhados, ele
/// ficaria preso naquele valor negativo para sempre e **nada** seria desenhado: a palavra
/// inteira desapareceria por o começo dela não caber.
///
/// ⚠️ Este gate nasceu de uma mutação SOBREVIVENTE. O comentário no laço já afirmava a
/// regra, e afirmar não é prender.
#[test]
fn the_pen_advances_through_glyphs_that_are_not_drawn() {
    let font = inter();
    let centred = TextLayout {
        align: TextAlign::Center,
        ..lay(1.0)
    };
    let path = line_path(100.0);
    let on = TextPlacement::OnPath {
        path: &path,
        start_offset: 0.0,
        flip: false,
    };
    let ridden = run(&font, "PATH", &centred, &on);
    let all = run(&font, "PATH", &centred, &at0());
    assert!(
        !ridden.is_empty(),
        "a metade de trás da palavra tem de ser desenhada — nenhuma foi"
    );
    assert!(
        ridden.len() < all.len(),
        "e a da frente cai antes do começo do caminho: {} de {}",
        ridden.len(),
        all.len()
    );
    // Os sobreviventes estão onde o layout reto os punha — o pen contou todos.
    for (a, b) in all.iter().skip(all.len() - ridden.len()).zip(&ridden) {
        let (ca, cb) = (path_center(a), path_center(b));
        assert!(
            (ca[0] - cb[0]).abs() < 1e-5,
            "o pen perdeu passo: {ca:?} contra {cb:?}"
        );
    }
}

/// **Uma parametrização ESTACIONÁRIA não tem direção, e isso custa uma letra.**
///
/// No documento uma reta é a cúbica degenerada `(P0,P0,P3,P3)`, cuja derivada é **nula nas
/// duas pontas** — então `frame_at(0)` de um segmento reto devolve tangente nula, e o texto
/// (que não pode inventar um rumo) salta ali. Medido: sobre uma reta, `start_offset = 0`
/// não produz referencial nenhum; `start_offset = 1e-9` já produz.
///
/// ⚠️ **Não está corrigido de propósito, e o preço foi MEDIDO.** A cura é geometricamente
/// óbvia — a direção de uma reta é a reta, e basta amostrar a derivada um passo para dentro
/// —, mas ela mora no [`ArcPath::frame_at`], cujo **outro** consumidor é o Zig Zag: ele
/// amostra EXATAMENTE nas âncoras (`anchor_arcs`), que são precisamente os pontos
/// estacionários. Aplicar a cura faz sangrar o fingerprint
/// `the_zigzag_is_byte_identical_across_the_arc_walker_extraction` — ou seja, **muda o
/// desenho de um efeito que o Enio já aprovou em smoke**. Isso é decisão de produto, não
/// carona de uma wave de texto.
///
/// Este gate existe para o defeito não ser re-descoberto do zero, e para a cura não ser
/// aplicada sem que alguém veja o que ela custa.
#[test]
fn a_stationary_parameterisation_has_no_direction_and_the_text_skips_it() {
    let path = line_path(4.0);
    let on = |off: f64| TextPlacement::OnPath {
        path: &path,
        start_offset: off,
        flip: false,
    };
    assert!(
        caret_frame(&on(0.0), [0.0, 0.0]).is_none(),
        "a ponta de uma reta degenerada não tem tangente"
    );
    assert!(
        caret_frame(&on(1e-9), [0.0, 0.0]).is_some(),
        "um passo para dentro e a direção existe — a curva TEM rumo, a parametrização é que parou"
    );
    // E num ARCO a mesma ponta responde: o defeito é da representação da reta, não do zero.
    let arc = quarter_arc(3.0);
    let on_arc = TextPlacement::OnPath {
        path: &arc,
        start_offset: 0.0,
        flip: false,
    };
    assert!(caret_frame(&on_arc, [0.0, 0.0]).is_some());
}

/// **O caret e os glyphs concordam sobre onde a linha acaba** — porque perguntam à MESMA
/// função. É o gate que prende o desenho inteiro desta wave: enquanto a resposta for uma,
/// não existe o estado em que o cursor ficou no texto reto e as letras foram para a curva.
///
/// O oráculo: o caret na ponta de `"PAT"` tem de cair onde o `H` de `"PATH"` começa.
#[test]
fn the_caret_lands_where_the_next_glyph_would_start_on_the_curve() {
    let font = inter();
    let l = lay(1.0);
    let path = circle_path(7.0);
    let on = TextPlacement::OnPath {
        path: &path,
        start_offset: 0.0,
        flip: false,
    };
    let pen_x = caret_x_offset(&font, "PAT", &l, &[]);
    let caret = caret_frame(&on, [pen_x, 0.0]).expect("caret no caminho");
    // O referencial do próximo glyph, pela rota dos GLYPHS: o `H` seria carimbado aqui.
    let h = font.glyph_for_char('H').expect("H");
    let scale = l.size / f64::from(font.units_per_em().max(1));
    let advance = f64::from(font.advance(h, &[]).unwrap_or(0.0)) * scale;
    let glyph = glyph_frame(&on, [pen_x, 0.0], advance).expect("glyph no caminho");
    assert!(
        (caret.origin[0] - glyph.origin[0]).abs() < 0.02
            && (caret.origin[1] - glyph.origin[1]).abs() < 0.02,
        "caret {:?} contra pen origin do glyph {:?}",
        caret.origin,
        glyph.origin
    );
    // E o caret está INCLINADO — sobre uma curva ele não pode continuar vertical.
    assert!(
        caret.y_axis[0].abs() > 0.1,
        "o caret acompanha a normal da curva: {:?}",
        caret.y_axis
    );
}

/// FNV-1a sobre TODA coordenada produzida (âncoras **e** alças, todos os contornos).
/// Alças porque é lá que uma mudança de referencial se esconde: um erro que só as move
/// deixa as âncoras no lugar e muda a curva entre elas.
fn fnv(paths: &[VecPath]) -> u64 {
    fn eat(h: &mut u64, x: f64) {
        for b in x.to_bits().to_be_bytes() {
            *h ^= u64::from(b);
            *h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in paths {
        for v in p.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                eat(&mut h, q[0]);
                eat(&mut h, q[1]);
            }
        }
    }
    h
}

/// A fixture do fingerprint: exercita TODO ingrediente do layout reto — duas linhas
/// (entrelinha), alinhamento não-trivial, tracking, um eixo variável, glifo com furo
/// (`o`), glifo sem área (o espaço) e uma origem que não é a origem.
fn straight_fingerprint_paths() -> Vec<VecPath> {
    let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
    let lay = TextLayout {
        size: 1.3,
        line_height: 1.15,
        tracking: 0.07,
        align: TextAlign::Center,
    };
    text_to_vec_paths(
        &font,
        "on path\nAVo",
        &lay,
        &[(AxisTag::new(*b"wght"), 620.0)],
        &TextPlacement::At([0.75, -0.25]),
        &Some(black()),
        &None,
    )
}

/// **O layout reto não se move.** O texto em caminho entra como um SEGUNDO referencial
/// por glifo, e o modo de falha que importa é o silencioso: a rota de sempre passar a
/// pousar as letras a um ulp de onde pousava, invisível na tela e presente em cada
/// arquivo salvo. O número foi medido no commit anterior a este e é a definição.
#[test]
fn the_straight_layout_is_byte_identical_across_the_text_on_path_wiring() {
    assert_eq!(fnv(&straight_fingerprint_paths()), 0x511c_90e4_7b1f_01db);
}

/// Multi-linha: a 2ª linha desce (world y-up → y negativo abaixo da baseline).
#[test]
fn a_second_line_sits_below_the_first() {
    let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
    let font = &font;
    let min_y = |paths: &[VecPath]| {
        paths
            .iter()
            .flat_map(|p| p.verts.iter())
            .map(|v| v.anchor[1])
            .fold(f64::INFINITY, f64::min)
    };
    let lay = TextLayout {
        size: 1.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
    };
    let one = text_to_vec_paths(font, "A", &lay, &[], &at0(), &Some(black()), &None);
    let two = text_to_vec_paths(font, "A\nA", &lay, &[], &at0(), &Some(black()), &None);
    assert!(
        min_y(&one) >= -1e-6,
        "linha única: baseline em 0, sem descer"
    );
    assert!(
        min_y(&two) < -0.5,
        "a 2ª linha desce abaixo da baseline da 1ª"
    );
}

/// Centralizar uma linha a desloca para a ESQUERDA da origem (o bloco fica
/// centrado no ponto de clique); alinhar à direita a termina na origem.
#[test]
fn alignment_shifts_the_line_horizontally() {
    let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
    let min_x = |paths: &[VecPath]| {
        paths
            .iter()
            .flat_map(|p| p.verts.iter())
            .map(|v| v.anchor[0])
            .fold(f64::INFINITY, f64::min)
    };
    let lay = |align| TextLayout {
        size: 1.0,
        line_height: 1.2,
        tracking: 0.0,
        align,
    };
    let left = text_to_vec_paths(
        &font,
        "AA",
        &lay(TextAlign::Left),
        &[],
        &at0(),
        &Some(black()),
        &None,
    );
    let center = text_to_vec_paths(
        &font,
        "AA",
        &lay(TextAlign::Center),
        &[],
        &at0(),
        &Some(black()),
        &None,
    );
    let right = text_to_vec_paths(
        &font,
        "AA",
        &lay(TextAlign::Right),
        &[],
        &at0(),
        &Some(black()),
        &None,
    );
    assert!(
        min_x(&center) < min_x(&left),
        "centralizado começa à esquerda do alinhado à esquerda"
    );
    assert!(
        min_x(&right) < min_x(&center),
        "à direita começa ainda mais à esquerda (termina na origem)"
    );
}

/// O texto vivo é UM path compound com todos os contornos: "Hi" = H (1) + ponto e
/// haste do i (2) → 1 verts + ≥2 subpaths, um objeto só.
#[test]
fn compound_path_merges_all_glyph_contours() {
    let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
    let lay = TextLayout {
        size: 1.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
    };
    let one = text_to_compound_path(&font, "Hi", &lay, &[], &at0(), &Some(black()), &None).unwrap();
    assert!(
        !one.subpaths.is_empty(),
        "vários glyphs/furos viram subpaths do mesmo path"
    );
    assert!(
        text_to_compound_path(&font, "   ", &lay, &[], &at0(), &Some(black()), &None).is_none(),
        "só espaços = sem geometria"
    );
}

/// Tracking positivo abre o espaço entre glyphs, então a mesma string ocupa mais
/// largura (o cursor avança mais).
#[test]
fn positive_tracking_widens_the_line() {
    let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
    let base = TextLayout {
        size: 1.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
    };
    let wide = TextLayout {
        tracking: 0.3,
        ..base
    };
    let narrow = caret_x_offset(&font, "AAA", &base, &[]);
    let opened = caret_x_offset(&font, "AAA", &wide, &[]);
    assert!(
        opened > narrow + 0.5,
        "tracking abre a linha (0.3·size × 2 gaps)"
    );
}
