#![forbid(unsafe_code)]
//! **O DESENHO DO OSSO** (estudo 42 item 5) — a metade visível do módulo do esqueleto.
//!
//! ⚠️ **Ele nunca soube o que é um caminho vectorial**, e por isso a mudança de casa em 2026-09-06
//! (era `ph2d-vec-render::bone`) não custou uma linha de lógica: um esqueleto sobre um desenho,
//! sobre uma imagem ou sobre uma malha desenha-se exactamente igual. *Uma peça que não importa o
//! documento de uma mídia já é do módulo — só falta mudá-la de sítio.*
//!
//! Um osso desenha-se como o **losango afilado** que toda ferramenta de rig usa (Spine, Moho, Rive,
//! Blender): largo na raiz, agudo na ponta. Não é decoração — a forma **diz a direcção**, que é a
//! única coisa que um segmento não diz e de que o artista precisa para saber para que lado o filho
//! sai.
//!
//! ⚠️ **A LARGURA é em píxeis de tela e o COMPRIMENTO é em mundo.** Um osso curto num zoom afastado
//! ficaria uma linha invisível se a espessura escalasse; e um losango de largura fixa em mundo
//! engoliria o desenho ao aproximar. É a mesma gramática das bolinhas do `envelope.rs`: *o ponto
//! sobe pelo afim, a espessura não*.
//!
//! ⛔ **Um osso NÃO é um `VecPath`** — ele não tem tinta, não exporta para SVG e não entra na cena
//! vectorial. O que se vê é isto, e só enquanto a ferramenta está na mão.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene,
};

/// ⭐⭐⭐ **A LARGURA DE UM OSSO É UMA FRACÇÃO DO COMPRIMENTO DELE**, nunca um número de píxeis.
///
/// ⛔ **Isto era `const BONE_HALF_PX: f64 = 4.0` e foi um defeito de produto reportado** (Enio,
/// 2026-09-06, com foto: *"os ossos viraram círculos"*). Medido no smoke com a câmera real, os
/// ossos têm **107 a 192 px** de comprimento na tela ⇒ o corpo saía com **8 px de largura, uma
/// proporção de 24:1**, desenhado a traço de `1,25 px`. E a bolinha da junta tem **12 px de
/// diâmetro** ⇒ *o anel era MAIS LARGO que o corpo que ele decora*, e o que o olho lia era o anel.
///
/// ⚠️ **A lei é a do Blender, escrita na doc dele:** *«the bone "size" (its thickness being
/// **proportional to its length**)»* — e é a mesma no Spine, no Moho e no Rive. Um osso longo é um
/// osso GROSSO; é isso que faz uma silhueta de rig ler-se de relance.
///
/// ⛔ **O antigo `4.0` não nomeava recurso nenhum** (CLAUDE.md §0.0): ele era largura de caneta a
/// fazer de largura de corpo. Os dois números abaixo nomeiam o deles.
const BONE_WIDTH_RATIO: f64 = 0.10;

/// Piso da meia-largura, em píxeis — **o recurso é o próprio CONTORNO**: abaixo de `2 × LINE_PX` o
/// corpo fica mais fino que a linha que o desenha, e as duas bordas fundem-se numa risca só.
const BONE_HALF_MIN_PX: f64 = 2.5;

/// Tecto da meia-largura, em píxeis — **o recurso é o DESENHO por baixo**: o osso é overlay e não
/// pode tapar a arte que deforma. Medido no smoke, a forma presa mais fina (o tentáculo, `0,8`
/// unidades de mundo) mede **61 px** na tela ⇒ um corpo de `2 × 14 = 28 px` fica em **46 %** dela,
/// e o artista continua a ver o que está a deformar.
const BONE_HALF_MAX_PX: f64 = 14.0;

/// A meia-largura do corpo de um osso de `comp` píxeis de comprimento **na tela**.
///
/// ⚠️ **Porta única, e é por isso que ela é `pub`:** o gate mede exactamente esta função. Uma
/// segunda conta da largura ao lado do desenho seria a resposta que envelhece.
#[must_use]
pub fn bone_half_width_px(comp: f64) -> f64 {
    (comp * BONE_WIDTH_RATIO).clamp(BONE_HALF_MIN_PX, BONE_HALF_MAX_PX)
}

/// Raio da bolinha da JUNTA, em píxeis — **e o mesmo alcance que o hit-test do host usa**
/// (`× px_to_world`), como o `ENVELOPE_HANDLE_R_PX` do irmão.
///
/// ⚠️ **Dois números fariam o dedo pegar a junta e a bolinha acender noutro sítio** — e aqui seria
/// pior que uma bolinha errada: a junta e o corpo executam VERBOS diferentes (deslocar × girar), e o
/// artista veria o osso andar quando queria girá-lo.
pub const BONE_JOINT_R_PX: f64 = 6.0;

/// Espessura do contorno, em píxeis.
const LINE_PX: f64 = 1.25;

/// **Desenha os ossos** `(bits, origem, ponta)` em MUNDO. `selected` acende um deles.
///
/// ⚠️ **A cor NÃO é o estado da selecção sozinha**: o seleccionado vem `Accent` **cheio** e os
/// outros `AccentSoft` **vazados**, que é a mesma gramática do `envelope.rs` (forma + preenchimento
/// carregam o estado) — assim lê-se qual está aceso sem depender de distinguir dois tons.
pub fn draw_bones(
    bones: &[(u64, [f64; 2], [f64; 2])],
    selected: Option<u64>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (aceso, apagado) = (vello(ColorToken::Accent), vello(ColorToken::AccentSoft));
    for &(bits, a, b) in bones {
        let (pa, pb) = (
            transform * Point::new(a[0], a[1]),
            transform * Point::new(b[0], b[1]),
        );
        let (dx, dy) = (pb.x - pa.x, pb.y - pa.y);
        let comp = dx.hypot(dy);
        if comp <= f64::EPSILON {
            continue;
        }
        // A perpendicular unitária, em TELA — é ela que dá a largura constante em píxeis.
        let (nx, ny) = (-dy / comp, dx / comp);
        // O ombro do losango fica a um quarto do caminho: é o que faz a silhueta ler como uma seta
        // e não como um triângulo, e é a proporção que as três referências usam.
        let ombro = Point::new(pa.x + dx * 0.25, pa.y + dy * 0.25);
        // ⚠️ O `min(comp * 0.25)` continua por cima da lei, e não é redundante: num osso curtíssimo
        // ele impede o ombro de ficar mais largo que o próprio comprimento (a silhueta deixaria de
        // ser uma seta e viraria um losango gordo).
        let w = bone_half_width_px(comp).min(comp * 0.25);
        let mut p = BezPath::new();
        p.move_to(pa);
        p.line_to(Point::new(ombro.x + nx * w, ombro.y + ny * w));
        p.line_to(pb);
        p.line_to(Point::new(ombro.x - nx * w, ombro.y - ny * w));
        p.close_path();
        let sel = Some(bits) == selected;
        if sel {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(aceso),
                None,
                &p,
            );
        }
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(if sel { aceso } else { apagado }),
            None,
            &p,
        );
        // A JUNTA: a bolinha na raiz é o que se agarra para posar, e é ela que mostra que dois
        // ossos partilham um ponto quando a cadeia é contínua.
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(if sel { aceso } else { apagado }),
            None,
            &Circle::new(pa, BONE_JOINT_R_PX),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os comprimentos que os ossos do smoke de facto têm **na tela**, medidos com a câmera real
    /// em 2026-09-06 (`PH2D_VEC_BONE_SMOKE=1`, 11 ossos): o tentáculo a `107,52`, o braço a
    /// `163,84`, o esqueleto solto a `192,00`.
    ///
    /// ⛔ **Não são números escolhidos** — são a população que o dono vê quando abre a cena, e é
    /// contra ela que a proporção tem de valer.
    const COMPRIMENTOS_MEDIDOS_PX: [f64; 3] = [107.52, 163.84, 192.00];

    /// ⭐⭐⭐ **O CORPO DO OSSO É MAIS LARGO QUE O ANEL DA JUNTA** — o defeito reportado, dito como
    /// número.
    ///
    /// Com a lei antiga (`4 px` fixos) o corpo media **8 px** de largura contra os **12 px** de
    /// diâmetro do anel: *o enfeite era maior que a coisa*, e o olho só via o círculo. Este gate é
    /// vermelho com aquela constante e verde com a lei do Blender.
    #[test]
    fn the_body_of_a_bone_reads_wider_than_the_joint_ring_that_decorates_it() {
        for comp in COMPRIMENTOS_MEDIDOS_PX {
            let largura = 2.0 * bone_half_width_px(comp);
            let anel = 2.0 * BONE_JOINT_R_PX;
            assert!(
                largura > anel,
                "um osso de {comp} px sai com {largura} px de corpo contra um anel de {anel} px - \
                 e' o defeito de 2026-09-06 ('os ossos viraram circulos') a voltar"
            );
        }
    }

    /// ⭐ **A GROSSURA SEGUE O COMPRIMENTO** — a lei que o Blender declara (*«its thickness being
    /// proportional to its length»*), e o que a separa de um número de píxeis: dois ossos de
    /// comprimentos diferentes **dentro da faixa** têm de sair com grossuras diferentes.
    ///
    /// ⚠️ Sem este gate, alguém "simplifica" a lei de volta para uma constante e os dois gates
    /// vizinhos continuam verdes — uma constante alta passaria no de cima.
    #[test]
    fn a_longer_bone_is_a_thicker_bone_and_not_a_longer_hairline() {
        let curto = bone_half_width_px(60.0);
        let longo = bone_half_width_px(130.0);
        assert!(
            longo > curto * 1.5,
            "a grossura parou de seguir o comprimento: {curto} contra {longo}"
        );
        // E a razão é a da referência, não um valor qualquer.
        assert!((longo / 130.0 - BONE_WIDTH_RATIO).abs() < 1e-12);
    }

    /// ⛔ **AS DUAS PONTAS DA FAIXA, cada uma pelo recurso que a nomeia.**
    ///
    /// Em baixo, o corpo nunca fica mais fino que o traço que o desenha (senão as duas bordas
    /// fundem-se numa risca). Em cima, ele nunca tapa a arte que deforma — a forma presa mais fina
    /// do smoke mede `61 px` na tela, e o corpo tem de caber nela.
    #[test]
    fn the_band_never_thins_below_its_own_outline_nor_swallows_the_art_it_deforms() {
        assert!(
            bone_half_width_px(1.0) >= 2.0 * LINE_PX,
            "num osso minusculo o corpo ficou mais fino que o proprio contorno"
        );
        const FORMA_MAIS_FINA_PX: f64 = 61.0;
        assert!(
            2.0 * bone_half_width_px(10_000.0) < FORMA_MAIS_FINA_PX * 0.5,
            "num zoom fechado o osso passa a tapar o desenho que ele deforma"
        );
    }
}
