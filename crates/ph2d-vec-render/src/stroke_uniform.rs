//! **A CANETA É REDONDA, mesmo sob escala não-uniforme** — a cura do [`BUGS_vector.md` #27].
//!
//! # O defeito, e a lei que o causa
//!
//! O traço de um caminho é emitido sob `camera * afim_do_path`. ⚠️ **No Vello o transform de um
//! `stroke` multiplica a CANETA, não só a geometria** — então com `scale = (sx, sy)` e `sx ≠ sy` a
//! caneta redonda de raio `w/2` vira uma **ELIPSE** de semi-eixos `w·sx/2` e `w·sy/2`: a borda fica
//! grossa num eixo e fina no outro (Enio, 2026-08-23, com as duas fotos).
//!
//! ⚠️ **A mesma lei já estava escrita duas vezes nesta crate** — no [`super::marquee`] (que a nomeia
//! por ela ter transformado um realce do Flip num borrão) e no [`super::hover_outline`]. Ela mordeu
//! uma terceira vez porque o caminho do DOCUMENTO nunca a tinha encontrado.
//!
//! # A decisão, e ela é do Enio
//!
//! *"Quando engrossa, engrossa por igual nos dois eixos."* ⇒ o traço **escala com o objecto**, mas
//! por um fator **ÚNICO**. Illustrator e Affinity oferecem a opção (desligada por omissão) e o
//! Figma não escala de todo — mas **nenhuma das três produz uma caneta elíptica**, e é isso que
//! esta porta garante.
//!
//! O fator é `√|det|`, a **média geométrica** dos semi-eixos:
//!
//! - para uma escala uniforme `s` ela é `√(s²) = s` — o caminho comum não muda um pixel;
//! - para `(2, ½)` ela é `1`: a forma ficou larga e baixa, a área não mudou, o traço não muda;
//! - e ela é **invariante à rotação**, então vale para qualquer afim e não só para escala
//!   alinhada aos eixos.
//!
//! # ⚠️ E o CAMINHO RÁPIDO fica intacto, de propósito
//!
//! Pré-transformar a geometria por instância é exactamente o que o `draw_path_with` recusa por
//! escrito: *"clonar o `BezPath` por instância era um custo por-instância que o cache de tesselação
//! não remove — e a 160k estrelas era metade do que sobrava"*.
//!
//! ⇒ **só o caso partido paga.** Um afim CONFORME (rotação + escala uniforme, com ou sem reflexão)
//! desenha pela chamada de sempre, byte a byte. Há gate.

use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

/// Tolerância relativa para chamar um afim de conforme. Ela é **relativa** de propósito: um teste
/// absoluto chamaria de não-uniforme todo afim de um documento em unidades grandes.
const CONFORMAL_EPS: f64 = 1e-6;

/// **ESTE AFIM PRESERVA CÍRCULOS?** — rotação e escala uniforme (com ou sem reflexão) preservam;
/// escala não-uniforme e cisalhamento não.
///
/// A parte linear é `[[a, c], [b, d]]`: as duas colunas têm de ter o **mesmo comprimento** e ser
/// **perpendiculares**. As duas metades importam — só a primeira deixaria passar um cisalhamento,
/// que também transforma a caneta numa elipse.
#[must_use]
pub fn is_conformal(m: Affine) -> bool {
    let [a, b, c, d, _, _] = m.as_coeffs();
    let (l1, l2) = (a * a + b * b, c * c + d * d);
    let scale = l1.max(l2).max(f64::MIN_POSITIVE);
    (l1 - l2).abs() <= scale * CONFORMAL_EPS && (a * c + b * d).abs() <= scale * CONFORMAL_EPS
}

/// **O FATOR UNIFORME EQUIVALENTE** de um afim — a média geométrica dos semi-eixos, `√|det|`.
///
/// ⚠️ Para um afim conforme de escala `s` ela devolve exactamente `s`, e é isso que faz o caminho
/// comum não mudar um pixel.
#[must_use]
pub fn uniform_scale(m: Affine) -> f64 {
    m.determinant().abs().sqrt()
}

/// **TRAÇA `bp` sob `transform` com a caneta REDONDA** — a porta única do traço de documento.
///
/// ⚠️ Ela existe para a lei nascer **num sítio só**: os três sítios que emitiam traço no
/// `draw_path_with` (a peça emprestada, a encurtada pelos marcadores, e o símbolo) teriam de a
/// repetir, e a quarta chamada nasceria sem ela.
pub fn stroke_uniform(
    target: &mut VectorScene,
    stroke: &Stroke,
    transform: Affine,
    brush: &Brush,
    bp: &BezPath,
) {
    let (pen, pen_xf) = pen_for(stroke, transform);
    match pen {
        // O caminho de sempre, byte a byte — e é por aqui que passa a esmagadora maioria.
        None => target.inner_mut().stroke(stroke, pen_xf, brush, None, bp),
        // ⚠️ **A geometria atravessa o afim; a caneta não.**
        Some(pen) => {
            let screen = transform * bp.clone();
            target
                .inner_mut()
                .stroke(&pen, pen_xf, brush, None, &screen)
        }
    }
}

/// ⭐⭐ **O MESMO traço, pintado com uma IMAGEM** (plano 35, wave B) — e ele existe por causa de uma
/// armadilha que o irmão sólido não tem.
///
/// ⚠️⚠️ **O Vello compõe `transform * brush_transform`, e este módulo tem DOIS caminhos que passam
/// afins diferentes ao Vello.** No caminho rápido a geometria é local e o afim é o `transform` ⇒ o
/// `brush_transform` local chega certo. No caminho não-conforme a geometria **já foi levada à tela**
/// e o afim é `IDENTITY` ⇒ o padrão ficaria no espaço errado, encolhido no canto do mundo. Aqui ele
/// é pré-composto com o `transform`.
///
/// ⇒ *o padrão tem de cair no MESMO sítio nos dois caminhos*, e há gate a medi-lo.
#[allow(clippy::too_many_arguments)] // um facto por argumento, como o irmao de preenchimento
pub fn stroke_uniform_image(
    target: &mut VectorScene,
    stroke: &Stroke,
    transform: Affine,
    image: &ph2d_vector::StableImage,
    brush_transform: Affine,
    x_extend: ph2d_vector::Extend,
    y_extend: ph2d_vector::Extend,
    quality: ph2d_vector::ImageQuality,
    alpha: f32,
    bp: &BezPath,
) {
    let (pen, pen_xf) = pen_for(stroke, transform);
    match pen {
        None => target.stroke_path_image(
            bp,
            stroke,
            pen_xf,
            image,
            brush_transform,
            x_extend,
            y_extend,
            quality,
            alpha,
        ),
        Some(pen) => {
            let screen = transform * bp.clone();
            target.stroke_path_image(
                &screen,
                &pen,
                pen_xf,
                image,
                // ⚠️ A geometria já atravessou o afim; o pincel tem de o atravessar também.
                transform * brush_transform,
                x_extend,
                y_extend,
                quality,
                alpha,
            );
        }
    }
}

/// **A CANETA QUE CHEGA AO VELLO, e sob que afim** — a decisão inteira desta porta, isolada para
/// poder ser MEDIDA.
///
/// - `(None, transform)` ⇒ o caminho rápido: a caneta de sempre, sob o afim de sempre. É a
///   afirmação de que o desenho é **byte-idêntico** ao de antes desta cura.
/// - `(Some(pen), IDENTITY)` ⇒ a geometria já foi levada à tela pelo afim, e a caneta é **redonda**
///   com o raio equivalente.
///
/// ⚠️ **Ela existe porque um gate sobre o `is_conformal` não prova o DESENHO.** O que decide o
/// pixel é o par *(caneta, afim)* que chega ao Vello, e é ele que este par devolve — a mesma
/// disciplina do `button_visual`: *o que se mede é o que se entrega*.
///
/// ⚠️ **O tracejado escala com a caneta**, e é a metade que se esquece: os comprimentos do dash
/// vivem nas unidades do CAMINHO, então deixá-los para trás faria o padrão encolher exactamente
/// onde a forma esticou.
#[must_use]
pub fn pen_for(stroke: &Stroke, transform: Affine) -> (Option<Stroke>, Affine) {
    if is_conformal(transform) {
        return (None, transform);
    }
    let k = uniform_scale(transform);
    let mut pen = stroke.clone();
    pen.width *= k;
    pen.dash_pattern = stroke.dash_pattern.iter().map(|d| d * k).collect();
    pen.dash_offset = stroke.dash_offset * k;
    (Some(pen), Affine::IDENTITY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_4;

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
}
