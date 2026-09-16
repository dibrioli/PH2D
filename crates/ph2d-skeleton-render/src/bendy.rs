//! ⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA do osso que dobra** — irmão do [`super`] pelo tecto de LOC, e
//! o corte é o mesmo do [`super::limit`]: ali mora *até onde a junta pode ir*, aqui *que forma o
//! corpo tem*.
//!
//! # ⛔ Por que elas não existiam (e por que isso era um controlo pela metade)
//!
//! A curvatura nasceu na F8 (2026-09-15) editável **só pelo painel**, com dois pares de números.
//! Um osso arqueia-se por uma FORMA, e uma forma não se escreve em quatro caixas — o artista
//! desenha-a. *A fila do módulo tinha isto como limite declarado; ele fechou em 2026-09-16.*
//!
//! # ⭐ A forma diz o que a alça faz, e o vocabulário já existe
//!
//! Elas são **os pontos de controlo da cúbica** ([`ph2d_skeleton::bend::handles`]), então o desenho
//! é o da alça de Bézier que todo editor vectorial tem: um **círculo pequeno** ligado por um traço
//! fino à extremidade que ele comanda. ⚠️ **E é isso que as separa das outras três alças do mesmo
//! osso**, que já usam as outras formas: a junta é um círculo GRANDE, a força um quadrado, as
//! paredes do limite são triângulos. *Três alças, três formas* — e agora cinco.
//!
//! # ⛔⛔ Elas só se desenham onde TÊM EFEITO
//!
//! Com `segments = 1` a curvatura é inerte **por construção** (o `BoneSpec::is_rigid` colapsa o
//! osso num só, seja qual for a `Bend`), então pintar a alça ali prometeria uma coisa que o arrasto
//! não faz — que é exactamente a espécie de controlo morto que o botão `Smooth` deste módulo acabou
//! de pagar. Quem decide é quem produz a posição ([`ph2d_skeleton_live::skin_live::bend_handles`]),
//! e este ficheiro só sabe desenhar o que lhe derem.

use super::{BonePart, LINE_PX};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene,
};

/// O raio da bolinha da alça de curvatura, em píxeis de TELA.
///
/// ⚠️ **Menor que a bolinha da junta e do mesmo tamanho da alça da força**: ela é uma alça de
/// propriedade, não uma junta. O alvo do DEDO é maior que isto (`BONE_HIT_PX`), como em todas as
/// outras — *o que se vê é o que se agarra, com a folga de sempre*.
pub const BEND_HANDLE_R_PX: f64 = 4.0;

/// **Onde as duas alças de curvatura estão e a que extremidades elas se ligam**, em MUNDO.
///
/// ⚠️ Um `type` e não um tuplo cru na assinatura: quatro pontos em dois pares leem-se ao contrário
/// com a mesma facilidade, e o nome é o que diz qual é qual.
///
/// ⚠️⚠️ **Há um gémeo na `ph2d-skeleton-live`, e as duas crates não se conhecem — de propósito.**
/// Este ficheiro nunca importou a lei do esqueleto (ele é `ph2d-vector` + tokens, e é isso que o
/// deixa desenhar sobre qualquer mídia); criar a aresta só para partilhar um nome trocaria uma
/// duplicação de quatro palavras por uma dependência entre duas famílias.
pub type BendHandles = ([[f64; 2]; 2], [[f64; 2]; 2]);

/// ⭐⭐⭐ **DESENHA AS DUAS ALÇAS DE CURVATURA** do osso em foco, já em MUNDO.
///
/// `pontos` é `(as duas alças, as duas extremidades)` do osso — o traço vai de cada alça para a
/// extremidade que ela comanda, que é o que diz qual é qual sem uma legenda.
///
/// ⚠️ `None` ⇒ não desenha nada, e é assim que um osso sem segmentos não ganha alça nenhuma.
pub fn draw_bend(
    pontos: Option<BendHandles>,
    hovered: Option<BonePart>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let Some(([h_inn, h_out], [raiz, ponta])) = pontos else {
        return;
    };
    let c = ColorToken::Accent.resolve(theme);
    let cor = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);

    // OS TRAÇOS, de cada extremidade até à alça dela — o desenho de alça de Bézier.
    let mut hastes = BezPath::new();
    for (de, para) in [(raiz, h_inn), (ponta, h_out)] {
        hastes.move_to(Point::new(de[0], de[1]));
        hastes.line_to(Point::new(para[0], para[1]));
    }
    // ⚠️ A largura entra em TELA (o afim já foi aplicado aos pontos abaixo), então aqui divide-se
    // pela escala do afim — a mesma conta que o [`super::limit`] faz, e pela mesma razão.
    let coef = transform.as_coeffs();
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX * coef[0].hypot(coef[1]).recip()),
        transform,
        &Brush::Solid(cor.multiply_alpha(0.5)),
        None,
        &hastes,
    );

    for (p, qual) in [(h_inn, BonePart::BendIn), (h_out, BonePart::BendOut)] {
        let q = transform * Point::new(p[0], p[1]);
        let bola = Circle::new(q, BEND_HANDLE_R_PX);
        if hovered == Some(qual) {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(cor),
                None,
                &bola,
            );
        }
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            None,
            &bola,
        );
    }
}
