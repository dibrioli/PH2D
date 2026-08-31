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

/// **O AFIM DE PINCEL QUE CHEGA AO VELLO**, e o único jeito de o construir é a
/// [`PatternFrame::brush_for`].
///
/// ⚠️ **Ele existe porque o argumento é AMBÍGUO e a ambiguidade era silenciosa**: o Vello compõe
/// `transform * brush_transform`, e este módulo entrega afins de geometria **diferentes** nos dois
/// caminhos (`transform` no rápido, `IDENTITY` no partido). Um `Affine` cru no parâmetro deixava o
/// chamador escolher o espaço errado e **desenhar** — o padrão encolhido no canto do mundo. Um tipo
/// que só a moldura fabrica torna isso inexprimível.
#[derive(Clone, Copy, Debug)]
pub struct BrushXf(Affine);

impl BrushXf {
    /// O afim lá dentro — **só para MEDIR**, e é por isso que é `#[cfg(test)]`: expor um getter no
    /// produto convidaria alguém a desmontar o par *(geometria, pincel)* que este tipo protege.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn probe_affine(self) -> Affine {
        self.0
    }
}

/// ⭐⭐⭐ **O ESPAÇO EM QUE A ESTAMPA DE UM TRAÇO VIVE** — a cura do ITEM B (a estampa esticava quando
/// a ENTIDADE tinha um `Transform` não-uniforme, alcançável por estados de UI / Smart Animate).
///
/// # A lei, e ela é a MESMA da rota do bake (`a14b6a0cb`)
///
/// *Numa faixa, a banda e o que está dentro dela escalam pelo MESMO fator, e esse fator é o da
/// CANETA (`√|det|`).* O preenchimento está **colado à forma** e estica com ela; um traço **não
/// estica** desde o bug #27 (*"quando engrossa, engrossa por igual nos dois eixos"*), e a estampa
/// dele tem de obedecer à mesma lei — senão o motivo estica dentro de uma banda que não esticou.
///
/// Medido antes da cura, com arte `4 × 2` e a forma sob `(3, 1)`: o ladrilho ia a um aspecto
/// **3,0000×** o autorado (e **2,7143×** sob `rot·(1,9 · 0,7)`).
///
/// # ⚠️ A BIFURCAÇÃO, e ela decidiu-se por MEDIÇÃO
///
/// Des-esticar o ladrilho **desloca-o**: a colocação é calculada no espaço das âncoras e mapeada
/// pelo afim, então trocar `transform` pela sua parte conforme muda **onde** o ladrilho cai. Duas
/// construções foram medidas lado a lado (sonda `measure_the_three_constructions_...`):
///
/// | | aspecto | âncora | `Clamp` cobre a forma |
/// |---|---|---|---|
/// | hoje (`transform · colocação`) | **3,0000×** | ✓ | ✓ |
/// | **A** — só compor a parte conforme | 1,0000× | ⛔ move `(1,268, −2,196)` | ⛔ **não** |
/// | **B** — *(esta)* + caixa no espaço do ladrilho | 1,0000× | ✓ `(0,000, 0,000)` | ✓ |
///
/// ⛔ **A passaria num gate que só olhasse o aspecto.** É por isso que o gate desta cura mede as
/// duas coisas, e é no `Clamp` que a segunda se vê: ali desenha-se **uma** cópia enquadrada, e uma
/// cópia deslocada deixa a forma por pintar.
///
/// # A construção
///
/// `transform` parte-se em `W ∘ E`, onde:
///
/// - **`E`** é o esticão puro (`det = ±1`) em torno da âncora do padrão — a parte que a caneta
///   recusa. A colocação calcula-se **nesta** cópia da forma: [`Self::box_of`] devolve a caixa de
///   `E · bp`, e é isso que faz o `Clamp` voltar a cobrir (o enquadramento passa a medir a forma
///   que o ladrilho de facto tem de tapar).
/// - **`W`** é conforme (a rotação de `transform` com a escala uniformizada por `√|det|`) e leva a
///   colocação à tela. Como `W ∘ E = transform` por construção, **todo ponto volta ao sítio onde
///   está hoje** — a âncora do `Clamp` é `transform(canto_local)` ao bit da aritmética.
///
/// ⭐ **A âncora do esticão é a `origin` do padrão, e a escolha é load-bearing:** o `placement`
/// não-`Clamp` ancora o reticulado em `self.origin`, que é um número em coordenadas **locais**.
/// Fixando o esticão nesse ponto, ele continua a valer sem tradução na cópia esticada, e
/// `W(origin) = transform(origin)` — o reticulado não anda. Com outra âncora, ele andaria.
///
/// # ⭐ E o caminho comum não paga nem multiplica
///
/// Sob um afim **conforme** a moldura é a identidade em ambas as metades: [`Self::box_of`] devolve a
/// caixa local sem clonar o `BezPath`, e [`Self::brush_for`] devolve a colocação **sem uma
/// multiplicação** — logo o desenho é byte-idêntico ao de antes desta cura, e não por promessa.
#[derive(Clone, Copy, Debug)]
pub struct PatternFrame {
    /// `None` ⇒ a colocação local é a resposta (o `transform` chega ao Vello por si).
    brush: Option<Affine>,
    /// `None` ⇒ a caixa mede-se na geometria local, sem clone.
    undo: Option<Affine>,
}

impl PatternFrame {
    /// A moldura de `transform` para um padrão ancorado em `anchor` (as coordenadas **locais** da
    /// `PatternFill::origin`).
    #[must_use]
    pub fn of(transform: Affine, anchor: [f64; 2]) -> Self {
        if is_conformal(transform) {
            return Self {
                brush: None,
                undo: None,
            };
        }
        let a = ph2d_vector::Vec2::new(anchor[0], anchor[1]);
        let alvo = transform * ph2d_vector::Point::new(anchor[0], anchor[1]);
        let [p, q, r, s, _, _] = transform.as_coeffs();
        let conforme = Affine::new(uniform_linear([p, q, r, s]));
        let w = Affine::translate((alvo.x, alvo.y)) * conforme * Affine::translate(-a);
        // ⛔ **O afim COLAPSADO cai no comportamento de sempre.** `√|det| = 0` ⇒ `W` é singular, o
        // `kurbo::Affine::inverse` devolve infinitos (não um `Option`), e não há `E`. A caneta
        // também sai com largura zero, então não há desenho a proteger — e inventar aqui uma
        // resposta nova mudaria em silêncio um caso que hoje é invisível.
        if w.determinant() == 0.0 || !w.as_coeffs().iter().all(|c| c.is_finite()) {
            return Self {
                brush: Some(transform),
                undo: None,
            };
        }
        Self {
            brush: Some(w),
            undo: Some(w.inverse() * transform),
        }
    }

    /// A caixa em que a colocação se calcula — a da forma **des-esticada**.
    #[must_use]
    pub fn box_of(&self, bp: &BezPath) -> ([f64; 2], [f64; 2]) {
        use ph2d_vector::Shape;
        let b = match self.undo {
            None => bp.bounding_box(),
            Some(e) => (e * bp.clone()).bounding_box(),
        };
        ([b.x0, b.y0], [b.x1, b.y1])
    }

    /// A colocação levada ao espaço em que a geometria vai ser desenhada.
    #[must_use]
    pub fn brush_for(&self, placement: Affine) -> BrushXf {
        // ⚠️ **Sem multiplicação no caminho comum, de propósito**: `IDENTITY * m` não é garantido
        // devolver os mesmos bits (um `-0.0` vira `+0.0`), e a promessa aqui é byte-identidade.
        BrushXf(match self.brush {
            None => placement,
            Some(w) => w * placement,
        })
    }
}

/// A parte linear CONFORME de `[a, b, c, d]` — a mesma decomposição polar da
/// [`ph2d_vec_scene::Xform::uniform_part`], que é a porta da rota do BAKE.
///
/// ⚠️ **A conta mora lá e é reutilizada aqui**, com a translação zerada: escrever a decomposição
/// uma segunda vez poria as duas rotas a divergir no primeiro ajuste, e elas têm de dar a MESMA
/// forma de ladrilho para a mesma pose.
fn uniform_linear(lin: [f64; 4]) -> [f64; 6] {
    ph2d_vec_scene::Xform([lin[0], lin[1], lin[2], lin[3], 0.0, 0.0])
        .uniform_part()
        .0
}

/// ⭐⭐ **O MESMO traço, pintado com uma IMAGEM** (plano 35, wave B) — e ele existe por causa de uma
/// armadilha que o irmão sólido não tem.
///
/// ⚠️⚠️ **O Vello compõe `transform * brush_transform`, e este módulo tem DOIS caminhos que passam
/// afins diferentes ao Vello.** No caminho rápido a geometria é local e o afim é o `transform` ⇒ a
/// colocação local chega certa. No caminho não-conforme a geometria **já foi levada à tela** e o
/// afim é `IDENTITY` ⇒ o padrão ficaria no espaço LOCAL sobre uma geometria de TELA: encolhido no
/// canto do mundo.
///
/// ⇒ quem responde por esse espaço é a [`PatternFrame`], e o [`BrushXf`] é a prova de que ela
/// respondeu — *não há como passar aqui um afim que não tenha saído dela*.
#[allow(clippy::too_many_arguments)] // um facto por argumento, como o irmao de preenchimento
pub fn stroke_uniform_image(
    target: &mut VectorScene,
    stroke: &Stroke,
    transform: Affine,
    image: &ph2d_vector::StableImage,
    brush: BrushXf,
    x_extend: ph2d_vector::Extend,
    y_extend: ph2d_vector::Extend,
    quality: ph2d_vector::ImageQuality,
    alpha: f32,
    bp: &BezPath,
) {
    let (pen, pen_xf) = pen_for(stroke, transform);
    match pen {
        None => target.stroke_path_image(
            bp, stroke, pen_xf, image, brush.0, x_extend, y_extend, quality, alpha,
        ),
        Some(pen) => {
            let screen = transform * bp.clone();
            target.stroke_path_image(
                &screen, &pen, pen_xf, image, brush.0, x_extend, y_extend, quality, alpha,
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
}
