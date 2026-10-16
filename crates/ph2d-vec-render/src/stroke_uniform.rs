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

use ph2d_vector::{
    Affine, BezPath, Brush, Fill, Shape, Stroke, StrokeOpts, VectorScene, expand_stroke,
};

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
    sharp: bool,
) {
    let (pen, pen_xf) = pen_for(stroke, transform);
    match pen {
        // O caminho de sempre, byte a byte — e é por aqui que passa a esmagadora maioria.
        None => match cut_outline(stroke, pen_xf, bp, sharp) {
            None => target.inner_mut().stroke(stroke, pen_xf, brush, None, bp),
            Some(out) => target
                .inner_mut()
                .fill(Fill::NonZero, pen_xf, brush, None, &out),
        },
        // ⚠️ **A geometria atravessa o afim; a caneta não.** (E o afim muda os ângulos: o corte
        // mede-se no ecrã, sem a cache.)
        Some(mut pen) => {
            let screen = transform * bp.clone();
            ajusta_no_ecra(&mut pen, &screen);
            match cut_outline(&pen, pen_xf, &screen, true) {
                None => target
                    .inner_mut()
                    .stroke(&pen, pen_xf, brush, None, &screen),
                Some(out) => target
                    .inner_mut()
                    .fill(Fill::NonZero, pen_xf, brush, None, &out),
            }
        }
    }
}

/// A tolerância do contorno expandido, em píxeis de ecrã.
const CUT_TOL_PX: f64 = 0.05;

/// ⭐ **O traço com os bicos CORTADOS** ([`crate::miter_clip`]) — `None` quando nenhuma junta passa
/// do limite (o Vello traça, como sempre); senão o contorno da `kurbo` com os cortes, que se
/// PREENCHE — e o mesmo para uma curva mais apertada que meia caneta ([`crate::miter_clip::tighter_than`]).
/// `sharp` = a cache da geometria diz que pode haver uma das duas.
fn cut_outline(pen: &Stroke, xf: Affine, geo: &BezPath, sharp: bool) -> Option<BezPath> {
    if !sharp {
        return None;
    }
    let cuts = crate::miter_clip::patches(geo, pen);
    // A curva mais apertada que meia caneta também vai pelo contorno exacto: a placa achata-a.
    if cuts.is_empty() && !crate::miter_clip::tighter_than(geo, pen.width * 0.5) {
        return None;
    }
    let tol = CUT_TOL_PX / uniform_scale(xf).max(1e-12);
    let mut out = expand_stroke(geo.iter(), pen, &StrokeOpts::default(), tol);
    crate::miter_clip::append_oriented(&mut out, &cuts);
    Some(out)
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
    let (pen, geo) = match pen {
        None => (stroke.clone(), std::borrow::Cow::Borrowed(bp)),
        Some(mut pen) => {
            let screen = transform * bp.clone();
            ajusta_no_ecra(&mut pen, &screen);
            (pen, std::borrow::Cow::Owned(screen))
        }
    };
    match cut_outline(&pen, pen_xf, &geo, true) {
        None => target.stroke_path_image(
            &geo, &pen, pen_xf, image, brush.0, x_extend, y_extend, quality, alpha,
        ),
        Some(out) => target.fill_path_image(
            &out,
            Fill::NonZero,
            pen_xf,
            image,
            brush.0,
            x_extend,
            y_extend,
            quality,
            alpha,
        ),
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

/// A folga RELATIVA do período no ajuste de um FECHADO no ecrã: o padrão ajustado acaba EXACTAMENTE no
/// início, e aí o `f64` do kurbo e o `f32` da placa (doc 121 §9.9) decidiriam por arredondamento se o
/// último traço emenda no primeiro — um pontinho na emenda, ou não, conforme a máquina. Alongado em
/// `1e-4`, o fim cai sempre DENTRO do último vão (`≤ 0,3 px` numa volta de `3 000 px`); o erro
/// relativo do arco somado em `f32` na placa é `~1e-5`.
pub const FOLGA_DO_AJUSTE: f64 = 1e-4;

/// ⭐ **O AJUSTE DO TRACEJADO NO ECRÃ** (doc 121 §9.9) — a [`ph2d_vec_scene::dash_fit`] (a cura da emenda
/// de 22/08) mediu o contorno no espaço LOCAL; sob escala não uniforme o comprimento no ecrã não é
/// `√|det|` vezes o local, e a emenda voltava. ⇒ o padrão da caneta (já `× √|det|`) ajusta-se outra
/// vez ao contorno mais longo da geometria JÁ transformada, pela mesma lei. Sob afim conforme o ecrã
/// é o local escalado e o ajuste não muda nada — e esse caminho nem passa aqui.
///
/// ⚠️ A placa (`ph2d-shape-gpu`, `ajuste_do_tracejado`) faz a MESMA conta para um `[traço, vão]`;
/// aqui vale para os até três pares (F16c) com fase `0` — o que o `kurbo_stroke` produz.
pub fn ajusta_no_ecra(pen: &mut Stroke, screen: &BezPath) {
    let n = pen.dash_pattern.len();
    if n == 0 || n % 2 != 0 || n > 2 * ph2d_vec_scene::MAX_DASH_PAIRS || pen.dash_offset != 0.0 {
        return;
    }
    let Some((total, fechado)) = maior_contorno(screen) else {
        return;
    };
    let pares: Vec<[f64; 2]> = pen.dash_pattern.chunks_exact(2).map(|c| [c[0], c[1]]).collect();
    let ajuste = ph2d_vec_scene::dash_fit::fit(
        ph2d_vec_scene::DashLengths::from_pairs(&pares),
        total,
        fechado,
    );
    let folga = if fechado { 1.0 + FOLGA_DO_AJUSTE } else { 1.0 };
    pen.dash_pattern = ajuste.scaled(folga).as_slice().iter().copied().collect();
}

/// O sub-caminho mais LONGO de `bp` e se é fechado — a [`ph2d_vec_scene::dash_fit::longest_contour`]
/// sobre a geometria do ecrã (o fecho conta, como lá).
fn maior_contorno(bp: &BezPath) -> Option<(f64, bool)> {
    let mut melhor: Option<(f64, bool)> = None;
    let mut sub = BezPath::new();
    let mut inicio = ph2d_vector::Point::ZERO;
    let mut fecha = |sub: &mut BezPath, fechado: bool| {
        if sub.elements().len() > 1 {
            let l = sub.perimeter(1e-6);
            if l > 0.0 && melhor.is_none_or(|(m, _)| l > m) {
                melhor = Some((l, fechado));
            }
        }
        *sub = BezPath::new();
    };
    for el in bp.elements() {
        match el {
            ph2d_vector::PathEl::MoveTo(p) => {
                fecha(&mut sub, false);
                inicio = *p;
                sub.push(*el);
            }
            ph2d_vector::PathEl::ClosePath => {
                sub.push(*el);
                fecha(&mut sub, true);
            }
            e => {
                // Depois de um fecho sem `MoveTo`, o sub-caminho seguinte começa no mesmo início.
                if sub.elements().is_empty() {
                    sub.move_to(inicio);
                }
                sub.push(*e);
            }
        }
    }
    fecha(&mut sub, false);
    melhor
}

#[cfg(test)]
#[path = "stroke_uniform_tests.rs"]
mod tests;
