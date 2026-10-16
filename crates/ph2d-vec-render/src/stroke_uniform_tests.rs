//! Testes de `stroke_uniform.rs` — arquivo irmão pelo tecto de LOC (HR-18).
//!
//! Ligado por `#[path]` no módulo pai, então `use super::*` continua valendo.

use super::*;
use std::f64::consts::FRAC_PI_4;

/// ⭐ O rectângulo de raio MINÚSCULO sob traço grosso (a foto do dono de 11/10) vai pelo contorno
/// EXACTO — a cache da geometria marca-o e o desenho preenche o contorno em vez de o deixar à
/// placa, que achata o arquinho do canto num chanfro. O de cantos vivos segue pela placa.
#[test]
fn a_tiny_round_corner_under_a_thick_pen_is_drawn_by_the_exact_outline() {
    use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, cook};
    let shape = |r: f64| {
        let mut v = ShapeKind::RoundRect.defaults();
        v[0] = r;
        let mut p = cook(ShapeKind::RoundRect, [0.0, 0.0], [3.0, 3.6], &v);
        p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.3));
        p
    };
    let pen = Stroke::new(0.3)
        .with_join(ph2d_vector::Join::Miter)
        .with_miter_limit(4.0);
    for (r, exact) in [(1e-4, true), (0.0, false), (0.5, false)] {
        let p = shape(r);
        let bp = crate::build_bezpath(&p.cooked());
        let cached = crate::agudo_de(&p, Some(&bp));
        assert_eq!(cached, exact, "raio {r}: a cache");
        assert_eq!(
            cut_outline(&pen, Affine::IDENTITY, &bp, cached).is_some(),
            exact,
            "raio {r}"
        );
    }
}

/// ⭐ doc 121 §9.9 — **sob escala não uniforme o tracejado FECHA no contorno do ecrã.** Um círculo
/// esticado `(2, 0,7)` com o padrão ajustado no LOCAL (a `dash_fit`) e escalado por `√|det|` deixa
/// a emenda a meio de um período (o CONTROLO); ajustado no ecrã, o contorno é um número inteiro
/// de períodos, alongados pela folga — o fim cai DENTRO do último vão.
#[test]
fn o_tracejado_esticado_fecha_no_contorno_do_ecra() {
    use ph2d_vector::{Circle, Shape as _};
    let bp = Circle::new((0.0, 0.0), 1.0).to_path(1e-4);
    let local = ph2d_vec_scene::dash_fit::fit([0.3, 0.2].into(), 2.0 * std::f64::consts::PI, true);
    let m = Affine::scale_non_uniform(2.0, 0.7);
    let (pen, xf) = pen_for(&Stroke::new(0.1).with_dashes(0.0, local.as_slice().iter().copied()), m);
    let mut pen = pen.expect("nao conforme");
    assert_eq!(xf, Affine::IDENTITY);
    let ecra = m * bp;
    let total = ecra.perimeter(1e-6);
    let periodo = |p: &Stroke| p.dash_pattern.iter().sum::<f64>();
    let antes = total / periodo(&pen);
    assert!(
        (antes - antes.round()).abs() > 0.2,
        "CONTROLO: o ajuste local tem de deixar a emenda a meio ({antes})"
    );
    ajusta_no_ecra(&mut pen, &ecra);
    let n = total / periodo(&pen) * (1.0 + FOLGA_DO_AJUSTE);
    assert!((n - n.round()).abs() < 1e-6, "{n} periodos — nao fecha");
    assert!(
        total / periodo(&pen) < n.round(),
        "o fim tem de cair DENTRO do ultimo vao"
    );
    let razao = pen.dash_pattern[0] / pen.dash_pattern[1];
    assert!(
        (razao - 1.5).abs() < 1e-9,
        "o traço e o vao andam juntos ({razao})"
    );
}

/// Num ABERTO a lei é `n` períodos mais um traço — o caminho começa e acaba com traço inteiro.
#[test]
fn o_tracejado_esticado_aberto_acaba_com_traco_inteiro() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    let m = Affine::scale_non_uniform(4.0, 0.7);
    let (pen, _) = pen_for(&Stroke::new(0.05).with_dashes(0.0, [0.11, 0.07]), m);
    let mut pen = pen.expect("nao conforme");
    let ecra = m * bp;
    ajusta_no_ecra(&mut pen, &ecra);
    let [t, v] = [pen.dash_pattern[0], pen.dash_pattern[1]];
    let n = (ecra.perimeter(1e-9) - t) / (t + v);
    assert!((n - n.round()).abs() < 1e-6, "{n}");
}

/// ⭐ **ROTAÇÃO E ESCALA UNIFORME PASSAM PELO CAMINHO RÁPIDO.**
///
/// ⚠️ É a metade que protege o produto: se um afim comum caísse no ramo lento, toda forma da
/// cena pagaria o clone por instância que o `draw_path_with` recusa por escrito — e a conta
/// só apareceria num profiler.
#[test]
fn rotation_and_uniform_scale_take_the_fast_path() {
    for m in [
        Affine::IDENTITY,
        Affine::scale(3.0),
        Affine::scale(0.017),
        Affine::rotate(FRAC_PI_4),
        Affine::translate((120.0, -40.0)) * Affine::rotate(1.1) * Affine::scale(2.5),
        // Reflexão: a caneta continua redonda.
        Affine::scale_non_uniform(-2.0, 2.0),
    ] {
        assert!(is_conformal(m), "{:?} devia ser conforme", m.as_coeffs());
    }
}

/// ⛔ **ESCALA NÃO-UNIFORME E CISALHAMENTO NÃO PASSAM.**
///
/// ⚠️ O cisalhamento está aqui porque só a metade *"as colunas medem o mesmo"* o deixaria
/// passar — e ele deforma a caneta exactamente como a escala desigual.
#[test]
fn non_uniform_scale_and_shear_do_not() {
    assert!(!is_conformal(Affine::scale_non_uniform(2.0, 0.5)));
    assert!(!is_conformal(Affine::scale_non_uniform(1.0, 1.4)));
    // ⚠️ **O caso DIFÍCIL: colunas de comprimentos IGUAIS que não são perpendiculares.** Um
    // cisalhamento qualquer (`[1,0,1,1]`) tem colunas de comprimentos 1 e √2 e cairia já na
    // primeira metade — ele não prova a segunda. Este tem as duas de comprimento 1, a 45°: só
    // a checagem do produto interno o apanha.
    let k = std::f64::consts::FRAC_1_SQRT_2;
    let shear = Affine::new([1.0, 0.0, k, k, 0.0, 0.0]);
    let [a, b, c, d, _, _] = shear.as_coeffs();
    assert!(
        ((a * a + b * b) - (c * c + d * d)).abs() < 1e-12,
        "a fixture não é o caso difícil: as colunas têm de medir o MESMO"
    );
    assert!(
        (a * c + b * d).abs() > 0.5,
        "a fixture não é o caso difícil: as colunas têm de NÃO ser perpendiculares"
    );
    assert!(!is_conformal(shear));
}

/// ⭐⭐ **A CANETA QUE CHEGA AO VELLO É REDONDA — e é este o gate da cura.**
///
/// ⚠️ Os gates acima medem o `is_conformal` e o fator; **nenhum deles prova o desenho**. O que
/// decide o pixel é o par *(caneta, afim)*, e é ele que se mede aqui.
#[test]
fn the_pen_that_reaches_vello_is_round() {
    let w = 4.0;
    let s = Stroke::new(w);

    // Caminho rápido: caneta INTOCADA, afim intocado — byte a byte o mundo pré-cura.
    for m in [Affine::scale(3.0), Affine::rotate(0.9) * Affine::scale(0.5)] {
        let (pen, xf) = pen_for(&s, m);
        assert!(pen.is_none(), "um afim conforme saiu do caminho rápido");
        assert_eq!(xf.as_coeffs(), m.as_coeffs());
    }

    // Escala desigual: a geometria leva o afim, a caneta fica REDONDA com o raio equivalente.
    let m = Affine::scale_non_uniform(4.0, 1.0);
    let (pen, xf) = pen_for(&s, m);
    let pen = pen.expect("uma escala desigual tem de pré-transformar");
    assert_eq!(
        xf.as_coeffs(),
        Affine::IDENTITY.as_coeffs(),
        "a caneta continuou a atravessar o afim — ela seria uma ELIPSE"
    );
    assert!(
        (pen.width - w * 2.0).abs() < 1e-9,
        "a largura devia ser w·√|det| = {}, e mediu {}",
        w * 2.0,
        pen.width
    );
    // ⛔ E ela NÃO é nenhum dos dois eixos: nem `w·4` (o maior) nem `w·1` (o menor).
    assert!((pen.width - w * 4.0).abs() > 1e-6 && (pen.width - w).abs() > 1e-6);
}

/// **O TRACEJADO ESCALA COM A CANETA.**
///
/// ⚠️ É a metade que se esquece: os comprimentos do dash vivem nas unidades do CAMINHO, e a
/// geometria passou a chegar já esticada. Deixá-los para trás faria o padrão encolher
/// exactamente onde a forma esticou.
#[test]
fn the_dash_travels_with_the_pen() {
    let s = Stroke::new(2.0).with_dashes(3.0, [8.0, 4.0]);
    let (pen, _) = pen_for(&s, Affine::scale_non_uniform(9.0, 1.0));
    let pen = pen.expect("escala desigual");
    assert!(
        (pen.dash_offset - 9.0).abs() < 1e-9,
        "o offset do dash ficou para trás"
    );
    let d: Vec<f64> = pen.dash_pattern.iter().copied().collect();
    assert!(
        (d[0] - 24.0).abs() < 1e-9 && (d[1] - 12.0).abs() < 1e-9,
        "o padrão do dash ficou para trás: {d:?}"
    );
}

/// ⭐ **O FATOR É `√|det|`, e para uma escala uniforme ele é a própria escala.**
///
/// É o que faz o caminho comum não mudar um pixel — e, na escala desigual, é o que preserva a
/// área da tinta em vez de escolher um dos dois eixos.
#[test]
fn the_uniform_factor_is_the_geometric_mean() {
    for s in [0.25f64, 1.0, 3.0] {
        assert!((uniform_scale(Affine::scale(s)) - s).abs() < 1e-12, "s={s}");
    }
    // (2, ½): a área não mudou, então o traço não muda.
    assert!((uniform_scale(Affine::scale_non_uniform(2.0, 0.5)) - 1.0).abs() < 1e-12);
    // (4, 1): a média geométrica é 2 — nem o 4 de um eixo, nem o 1 do outro.
    assert!((uniform_scale(Affine::scale_non_uniform(4.0, 1.0)) - 2.0).abs() < 1e-12);
    // E é invariante à ROTAÇÃO.
    let m = Affine::rotate(0.7) * Affine::scale_non_uniform(4.0, 1.0);
    assert!((uniform_scale(m) - 2.0).abs() < 1e-12);
}
/// ⭐⭐⭐ **UM AFIM CONFORME NÃO TOCA NA MOLDURA — nem na colocação, nem na caixa.**
///
/// É a metade que garante que o caso comum (rotação, translação, escala uniforme, reflexão)
/// desenha **byte a byte** o que desenhava antes desta cura, e ela é forte por CONSTRUÇÃO e não
/// por aritmética: a moldura devolve os mesmos valores, então o `stroke_uniform_image` recebe
/// literalmente os mesmos argumentos.
///
/// ⚠️ **A régua é a igualdade de BITS**, e é de propósito: `IDENTITY * m` não devolve os mesmos
/// bits para todo `m` (um `-0.0` vira `+0.0`), e uma barra `< 1e-12` deixaria essa redacção
/// passar. *Uma promessa de byte-identidade só se mede em bytes.*
#[test]
fn a_conformal_pose_leaves_the_placement_and_the_box_untouched() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((20.0, 0.0));
    bp.line_to((20.0, 10.0));
    bp.close_path();
    let caixa_local = ([0.0, 0.0], [20.0, 10.0]);
    // Uma colocação com um `-0.0` e um irracional — os dois casos que uma multiplicação move.
    let place = Affine::new([4.0, -0.0, 0.0, -2.0, 1.0 / 3.0, 7.5]);
    for (rotulo, m) in [
        ("identidade", Affine::IDENTITY),
        ("translacao", Affine::translate((37.0, -11.0))),
        ("escala uniforme", Affine::scale(3.0)),
        ("rotacao", Affine::rotate(0.6)),
        (
            "rot + escala + translacao",
            Affine::translate((5.0, -2.0)) * Affine::rotate(1.1) * Affine::scale(2.5),
        ),
        ("reflexao", Affine::scale_non_uniform(-2.0, 2.0)),
    ] {
        assert!(is_conformal(m), "a fixtura `{rotulo}` nao e' conforme");
        let frame = PatternFrame::of(m, [1.0, 3.0]);
        assert_eq!(
            frame.brush_for(place).probe_affine().as_coeffs(),
            place.as_coeffs(),
            "`{rotulo}`: a colocacao foi mexida no caminho conforme - o caso comum mudou"
        );
        assert_eq!(
            frame.box_of(&bp),
            caixa_local,
            "`{rotulo}`: a caixa saiu do espaco LOCAL no caminho conforme"
        );
    }
    // ⚠️ **CONTROLO**: um afim NÃO-conforme mexe nas duas. Sem isto, uma moldura que fosse
    // sempre a identidade passaria neste gate — e não curaria nada.
    let partido = PatternFrame::of(Affine::scale_non_uniform(3.0, 1.0), [1.0, 3.0]);
    assert_ne!(
        partido.brush_for(place).probe_affine().as_coeffs(),
        place.as_coeffs(),
        "a moldura de um afim PARTIDO deixou a colocacao intacta - ela e' inerte"
    );
    assert_ne!(
        partido.box_of(&bp),
        caixa_local,
        "a moldura de um afim PARTIDO mediu a caixa no espaco LOCAL - o Clamp nao cobre"
    );
}

/// ⭐⭐ **A PARTE CONFORME E O ESTICÃO RECOMPÕEM A POSE** — `W ∘ E = transform`.
///
/// É esta identidade que compra a metade da POSIÇÃO: como a colocação se calcula em `E · forma`
/// e volta por `W`, **todo ponto da forma aterra onde o `transform` o punha**. Sem ela, uma cura
/// que uniformizasse a escala deslocaria a estampa — que foi a construção `A`, medida e
/// rejeitada (âncora a saltar `(1,268, −2,196)` no `Clamp`).
///
/// ⚠️ E o esticão é **`det = ±1` por construção** — ele é exactamente a parte que a caneta
/// recusa, nem mais nem menos.
#[test]
fn the_stretch_and_the_conformal_part_recompose_into_the_pose() {
    for (rotulo, m) in [
        ("(3, 1)", Affine::scale_non_uniform(3.0, 1.0)),
        ("(1, 0.25)", Affine::scale_non_uniform(1.0, 0.25)),
        (
            "rot * (1,9 . 0,7) + translacao",
            Affine::translate((13.0, -4.0))
                * Affine::rotate(0.4)
                * Affine::scale_non_uniform(1.9, 0.7),
        ),
        ("cisalhamento", Affine::new([1.0, 0.0, 0.6, 1.0, 2.0, 3.0])),
    ] {
        assert!(!is_conformal(m), "a fixtura `{rotulo}` e' conforme");
        let frame = PatternFrame::of(m, [1.0, 3.0]);
        let (w, e) = (
            frame.brush.expect("um afim partido tem parte conforme"),
            frame.undo.expect("um afim partido tem esticao"),
        );
        assert!(
            is_conformal(w),
            "`{rotulo}`: a parte que leva a colocacao a' tela nao e' conforme - o ladrilho \
             continua a esticar"
        );
        assert!(
            (e.determinant().abs() - 1.0).abs() < 1e-12,
            "`{rotulo}`: o esticao tem |det| = {} e devia ser 1 - ele leva escala consigo",
            e.determinant().abs()
        );
        let c = (w * e).as_coeffs();
        let alvo = m.as_coeffs();
        assert!(
            c.iter().zip(alvo).all(|(a, b)| (a - b).abs() < 1e-9),
            "`{rotulo}`: W . E = {c:?} e a pose e' {alvo:?} - a estampa deixa de aterrar onde \
             a forma aterra"
        );
    }
}

/// ⛔ **UMA POSE COLAPSADA CAI NO COMPORTAMENTO DE SEMPRE.**
///
/// `√|det| = 0` ⇒ a parte conforme é singular e não há esticão para inverter. A caneta também
/// sai de largura zero, então não há desenho a proteger — e inventar aqui uma resposta nova
/// mudaria em silêncio um caso hoje invisível. ⚠️ O que **não** pode acontecer é a colocação
/// ficar em espaço LOCAL sobre geometria de TELA (o `pen_for` devolve `IDENTITY` aqui), e é isso
/// que a pré-composição pelo `transform` mantém.
#[test]
fn a_collapsed_pose_falls_back_to_the_pre_cure_composition() {
    let m = Affine::scale_non_uniform(3.0, 0.0);
    assert!(!is_conformal(m), "a fixtura nao contem o fenomeno");
    let place = Affine::new([4.0, 0.0, 0.0, 2.0, 5.0, 7.0]);
    let frame = PatternFrame::of(m, [1.0, 3.0]);
    assert_eq!(
        frame.brush_for(place).probe_affine().as_coeffs(),
        (m * place).as_coeffs(),
        "a pose colapsada deixou de pre-compor - a colocacao fica no espaco LOCAL sobre \
         geometria de TELA"
    );
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((20.0, 10.0));
    assert_eq!(
        frame.box_of(&bp),
        ([0.0, 0.0], [20.0, 10.0]),
        "a caixa da pose colapsada saiu do espaco local"
    );
}

/// O sinal AGUDO vem também das QUINAS: o triângulo fino (bico de ~5,7°, além do limite) sob um
/// traço fino — sem curva nenhuma que aperte — vai pelo contorno exacto (o bico cortado).
#[test]
fn a_sharp_polygon_under_a_thin_pen_is_flagged_by_its_corners() {
    use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex};
    let p = VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.01)),
        ..VecPath::default()
    };
    let bp = crate::build_bezpath(&p.cooked());
    assert!(
        !crate::miter_clip::tighter_than(&bp, 0.005),
        "CONTROLO: nenhuma curva aperta"
    );
    assert!(crate::agudo_de(&p, Some(&bp)));
}
