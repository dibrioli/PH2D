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
    Affine, BezPath, Brush, Cap, Circle, Color as VelloColor, Fill, Point, Rect, Stroke,
    VectorScene,
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
/// artista veria o osso andar quando queria girá-lo. ⭐ Com o **hover** (2026-09-06) essa lei ficou
/// mais forte, não mais fraca: o realce tem de acender exactamente o que o clique pega.
///
/// ⛔⛔ **Era `6.0` e foi um defeito reportado** (Enio, 2026-09-06: *«a bolinha e sua área sensível
/// ao mouse precisa ser maior pois está difícil selecioná-la»*). O número **não foi escolhido, foi
/// achado**: a casa declara UMA tolerância de dedo para toda alça — `HANDLE_HIT_PX = 12` no
/// `input_dispatch` — e o `BONE_HIT_PX` do CORPO já a pedia emprestada *«para o dedo do artista ter
/// sempre a mesma tolerância»*. ⇒ o osso dava ao corpo a tolerância da casa e à junta **metade**
/// dela, e o alvo menor estava por dentro do maior.
pub const BONE_JOINT_R_PX: f64 = 12.0;

/// ⭐ **O raio da junta DESTE osso** — a porta única do desenho e do dedo.
///
/// ⚠️ **Ele encolhe num osso curto, e o recurso tem nome: o próprio verbo de GIRAR.** Duas juntas
/// de raio `r` comem `2r` do comprimento, e a partir daí o osso é todo junta — *«impossível de
/// girar»*, que é a cerca que o `grabbed_the_joint` já declarava. Com o tecto em `comp/4` sobra
/// sempre **metade** do corpo para o giro.
///
/// Medido nos ossos do smoke (107,52 · 163,84 · 192,00 px): os três ficam no `12` cheio, e a
/// redução só entra abaixo de `48 px` — um osso que na tela já é um risco.
#[must_use]
pub fn joint_radius_px(comp: f64) -> f64 {
    BONE_JOINT_R_PX.min(comp * 0.25)
}

/// **A PARTE de um osso que o ponteiro aponta** — e cada uma executa um verbo diferente.
///
/// ⚠️ **Não é «que osso», é «que parte de que osso»**: as alças vivem umas dentro/em cima das
/// outras, e um realce que não as distinguisse deixaria por responder a pergunta que ele existe
/// para responder — *o que acontece se eu carregar aqui?*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonePart {
    /// O losango. **Gira** o osso.
    Body,
    /// A bolinha da raiz. **Desloca** o osso.
    Joint,
    /// A alça na borda da região de influência. Muda a **força** — quanto o osso alcança.
    Influence,
    /// ⭐ A bolinha na PONTA de um osso que fecha a corrente (o *end effector*). Arrastá-la faz
    /// **cinemática inversa**: a corrente inteira dobra para a ponta chegar onde a mão foi.
    Tip,
}

/// **O que está sob o ponteiro**, para o realce dizer qual VERBO o clique vai executar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoneHover {
    /// Os bits da entidade do osso apontado.
    pub bone: u64,
    /// Qual das três alças dele.
    pub part: BonePart,
}

/// ⭐⭐⭐ **ONDE FICA A ALÇA DA FORÇA** — a porta única do desenho e do dedo.
///
/// No **meio** do osso, na perpendicular, à distância do raio de influência. ⚠️ **O meio, e não a
/// raiz nem a ponta, é o único sítio onde ela não disputa com nada**: a raiz tem a bolinha e a
/// ponta tem a bolinha do osso seguinte, e as duas executam outros verbos.
///
/// `None` quando o osso é degenerado (origem e ponta no mesmo sítio) — sem eixo não há
/// perpendicular, e inventar uma faria a alça saltar de lado a cada quadro.
#[must_use]
pub fn influence_handle(a: [f64; 2], b: [f64; 2], radius: f64) -> Option<[f64; 2]> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let comp = dx.hypot(dy);
    if comp <= f64::EPSILON {
        return None;
    }
    // A perpendicular unitária, no MESMO sentido em que o osso aponta rodado de 90° — assim a alça
    // acompanha a rotação do osso em vez de saltar para o outro lado a meio de um giro.
    let (nx, ny) = (-dy / comp, dx / comp);
    let meio = [f64::midpoint(a[0], b[0]), f64::midpoint(a[1], b[1])];
    Some([meio[0] + nx * radius, meio[1] + ny * radius])
}

/// Espessura do contorno, em píxeis.
const LINE_PX: f64 = 1.25;

/// Como um osso se apresenta — as três chaves que a gramática usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Look {
    /// O corpo aceso (apontado ou seleccionado).
    corpo: bool,
    /// A bolinha da raiz acesa.
    junta: bool,
    /// O corpo PREENCHIDO — reservado à selecção.
    cheio: bool,
}

/// ⭐⭐⭐ **UM OSSO, DESENHADO — a porta única.** Devolve o comprimento em píxeis de tela.
///
/// ⚠️ **Ela existe por causa da PRÉ-VISUALIZAÇÃO** (Enio, 2026-09-07: *«o osso deve aparecer logo
/// no mouse down e crescer conforme o usuário arrasta»*): o osso que está a nascer tem de ser
/// desenhado pelo MESMO código que desenha o que já existe, senão o artista vê uma coisa enquanto
/// arrasta e recebe outra ao soltar. *Duas pinturas do mesmo objecto divergem no primeiro ajuste.*
fn glyph(
    pa: Point,
    pb: Point,
    look: Look,
    (aceso, apagado): (VelloColor, VelloColor),
    target: &mut VectorScene,
) -> f64 {
    let (dx, dy) = (pb.x - pa.x, pb.y - pa.y);
    let comp = dx.hypot(dy);
    if comp > f64::EPSILON {
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
        if look.cheio {
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
            &Brush::Solid(if look.corpo { aceso } else { apagado }),
            None,
            &p,
        );
    }
    // A JUNTA: a bolinha na raiz é o que se agarra para posar, e é ela que mostra que dois ossos
    // partilham um ponto quando a cadeia é contínua.
    //
    // ⭐ **Apontada, ela ENCHE** — e aqui o preenchimento não colide com a selecção porque o corpo
    // já a diz: uma bolinha cheia sobre um corpo apagado lê-se *"o clique aqui desloca"*, que é a
    // única coisa que o artista precisa de saber antes de carregar.
    //
    // ⚠️ **Ela é desenhada mesmo com o corpo de comprimento ZERO**, e é isso que faz o osso
    // «aparecer no mouse down»: no instante do press ainda não há eixo nenhum, e o que o artista
    // tem de ver é que o gesto COMEÇOU.
    let raio = joint_radius_px(comp);
    let bolinha = Circle::new(pa, raio);
    if look.junta && !look.cheio {
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(aceso),
            None,
            &bolinha,
        );
    }
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(if look.junta { aceso } else { apagado }),
        None,
        &bolinha,
    );
    comp
}

/// ⭐⭐⭐ **O OSSO QUE ESTÁ A NASCER** — a pré-visualização do arrasto (Enio, 2026-09-07).
///
/// ⛔ **Sem ela o osso só aparecia no `Up`**, e o artista desenhava às cegas: ele escolhia
/// direcção e comprimento sem ver nenhum dos dois.
///
/// ⚠️⚠️ **`armed` diz se este arrasto CHEGA a fazer um osso**, e não é decoração: o `Up` recusa um
/// arrasto mais curto que o raio das alças (um osso de comprimento zero não pesa ponto nenhum e é
/// invisível). Sem esta distinção a pré-visualização **prometeria** um osso que o `Up` não faz —
/// e o `CLAUDE.md` §5.0 nomeia isso: *uma cena que ensina o contrário do que acontece é pior que
/// uma cena ausente*. Armado = aceso; ainda curto = apagado.
///
/// ⛔ A decisão do `armed` **não é tomada aqui** — ela vem da mesma porta que o `Up` consulta.
pub fn draw_bone_preview(
    origin: [f64; 2],
    tip: [f64; 2],
    armed: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let c = ColorToken::Accent.resolve(theme);
    let aceso = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);
    let d = ColorToken::AccentSoft.resolve(theme);
    let apagado = VelloColor::from_rgba8(d.r, d.g, d.b, d.a);
    glyph(
        transform * Point::new(origin[0], origin[1]),
        transform * Point::new(tip[0], tip[1]),
        Look {
            corpo: armed,
            junta: armed,
            cheio: false,
        },
        (aceso, apagado),
        target,
    );
}

/// **Desenha os ossos** `(bits, origem, ponta)` em MUNDO. `selected` acende um deles, `hover` diz
/// o que está sob o ponteiro.
///
/// ⚠️ **A cor NÃO é o estado da selecção sozinha**: o seleccionado vem `Accent` **cheio** e os
/// outros `AccentSoft` **vazados**, que é a mesma gramática do `envelope.rs` (forma + preenchimento
/// carregam o estado) — assim lê-se qual está aceso sem depender de distinguir dois tons.
///
/// ⭐⭐ **E o HOVER usa o canal que sobrava, sem colidir com a selecção** (Enio, 2026-09-06:
/// *«precisamos de um efeito hover na bolinha e no corpo do osso»*): a **COR** diz apontado
/// (`AccentSoft` → `Accent`), o **PREENCHIMENTO** diz seleccionado. A escada lê-se de uma vez:
/// *traço apagado → traço aceso → cheio*.
///
/// ⚠️⚠️ **E o realce é por METADE, não por osso** — só a metade apontada acende. É o que faz o
/// artista SABER, antes de carregar, se vai **girar** (corpo) ou **deslocar** (junta): as duas
/// alças estão uma dentro da outra, e sem isto a única forma de descobrir o verbo é executá-lo.
pub fn draw_bones(
    bones: &[(u64, [f64; 2], [f64; 2])],
    selected: Option<u64>,
    hover: Option<BoneHover>,
    tips: &[u64],
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
        let sel = Some(bits) == selected;
        // ⚠️ O hover é lido POR PARTE: ele escolhe qual das alças acende, e as outras ficam no tom
        // apagado mesmo com o ponteiro sobre o mesmo osso.
        let sob = hover.filter(|h| h.bone == bits);
        let parte = |q: BonePart| sob.is_some_and(|h| h.part == q);
        let raio = glyph(
            pa,
            pb,
            Look {
                corpo: sel || parte(BonePart::Body),
                junta: sel || parte(BonePart::Joint),
                cheio: sel,
            },
            (aceso, apagado),
            target,
        );
        // ⭐⭐⭐ **A PONTA de quem fecha a corrente** — o *end effector*. Ela só existe onde não há
        // osso filho: em toda outra junta, a ponta de um osso É a raiz do seguinte, e ali já há
        // uma bolinha com outro verbo.
        //
        // ⚠️ Ela é um ANEL DUPLO, e a forma diz o que ela faz: as outras alças movem UM osso, esta
        // dobra a CORRENTE inteira. *Uma alça que faz uma coisa maior tem de se ler como maior.*
        if tips.contains(&bits) {
            let ponta_acesa = parte(BonePart::Tip);
            let cor = if ponta_acesa { aceso } else { apagado };
            for k in [1.0, 0.55] {
                target.inner_mut().stroke(
                    &Stroke::new(LINE_PX),
                    Affine::IDENTITY,
                    &Brush::Solid(cor),
                    None,
                    &Circle::new(pb, raio * k),
                );
            }
            if ponta_acesa {
                target.inner_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(aceso),
                    None,
                    &Circle::new(pb, raio * 0.55),
                );
            }
        }
    }
}

/// Alfa da mancha de influência — **a região é um FUNDO, não um objecto**: ela tem de dizer *até
/// onde este osso alcança* sem competir com o desenho que está por baixo dela, que é o que o
/// artista está a julgar.
///
/// ⚠️ Número de PRODUTO, não medido — e declarado como tal. O recurso que ele nomeia é a leitura do
/// desenho: a `0,18` a arte lê-se através da mancha, e o smoke é quem o julga.
const INFLUENCE_ALPHA: f32 = 0.18;

/// ⭐⭐⭐ **A REGIÃO DE INFLUÊNCIA de UM osso** — a mancha semi-transparente do *Bone Strength* do
/// Moho, mais a alça que a arrasta.
///
/// > *«a semi-transparent region appears around each bone that indicates the strength of the bone»*
/// > — manual do Moho.
///
/// ⚠️ **A forma é uma CÁPSULA e sai de graça**: o conjunto dos pontos a menos de `radius` de um
/// SEGMENTO é exactamente o que um traço de largura `2·radius` com ponta redonda pinta. ⛔ Construir
/// dois arcos e duas rectas à mão seria a segunda resposta para a mesma pergunta — e a que diverge
/// da lei do peso, que também mede distância ao segmento (`dist2_to_segment`).
///
/// ⚠️ **A largura entra em MUNDO** (o afim é passado ao `stroke`), ao contrário de tudo o resto
/// deste ficheiro: a região é uma grandeza do documento — ela cresce com o zoom porque a área que
/// ela cobre no desenho é a coisa que o artista está a decidir. A alça, essa, é chrome e mede-se em
/// píxeis.
///
/// ⛔ **UMA região de cada vez**, a do osso em foco. Todas ao mesmo tempo seriam sopa: elas
/// sobrepõem-se por construção (é o que faz a mistura ser suave), e o que o artista precisa de ver
/// é *quanto ESTE osso alcança*.
pub fn draw_influence(
    region: Option<(f64, [f64; 2], [f64; 2])>,
    hovered: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let Some((radius, a, b)) = region else {
        return;
    };
    if !(radius.is_finite() && radius > 0.0) {
        return;
    }
    let c = ColorToken::Accent.resolve(theme);
    let cor = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);
    let mut eixo = BezPath::new();
    eixo.move_to(Point::new(a[0], a[1]));
    eixo.line_to(Point::new(b[0], b[1]));
    target.inner_mut().stroke(
        &Stroke::new(2.0 * radius).with_caps(Cap::Round),
        transform,
        &Brush::Solid(cor.multiply_alpha(INFLUENCE_ALPHA)),
        None,
        &eixo,
    );
    // A ALÇA — quadrada, e a forma é que a distingue: a bolinha da junta é REDONDA, e duas alças
    // redondas em sítios diferentes obrigariam o artista a decorar qual é qual.
    let Some(h) = influence_handle(a, b, radius) else {
        return;
    };
    let p = transform * Point::new(h[0], h[1]);
    let r = INFLUENCE_HANDLE_R_PX;
    let quad = Rect::new(p.x - r, p.y - r, p.x + r, p.y + r);
    if hovered {
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(cor),
            None,
            &quad,
        );
    }
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(cor),
        None,
        &quad,
    );
}

/// Meia-aresta do quadradinho da força, em píxeis de tela.
///
/// ⚠️ **Ele é MENOR que a bolinha da junta** (`12`), e isso é hierarquia, não descuido: deslocar um
/// osso é um gesto que se faz o tempo todo; mudar a força dele é um ajuste. ⛔ O ALVO do dedo,
/// esse, continua a ser a tolerância da casa — ver `bone_gesture::hover`.
pub const INFLUENCE_HANDLE_R_PX: f64 = 5.0;

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

    /// ⭐⭐⭐ **O CORPO DO OSSO NÃO É UM FIO DE CABELO** — o defeito reportado, dito como número.
    ///
    /// Com a lei antiga (`4 px` fixos) um osso de `192 px` saía com **8 px** de largura: uma
    /// proporção de **24:1**, desenhada a traço de `1,25 px`. A referência entrega ~`5:1` (o
    /// octaedro do Blender), e é essa a barra.
    ///
    /// ⚠️⚠️ **A 1.ª redacção deste gate comparava o corpo com o DIÂMETRO DO ANEL, e essa régua era
    /// um PROXY que se invalidou horas depois:** o anel cresceu de `6` para `12` px de raio por uma
    /// razão independente e medida (a tolerância de dedo da casa — report de 2026-09-06, *«está
    /// difícil selecioná-la»*), e a comparação passou a reprovar um corpo que **não** regrediu.
    /// ⛔ *Uma barra que se move quando o outro lado dela muda por outro motivo não estava a medir a
    /// propriedade que nomeia* — e afrouxá-la teria sido esconder isso. A propriedade real é a
    /// PROPORÇÃO, que é o que a lei do Blender declara e o que de facto mudou (24:1 → 5:1).
    ///
    /// Este gate continua **vermelho com a constante antiga** — era ele o ponto —, e agora sem
    /// depender de um número que não é dele.
    #[test]
    fn the_body_of_a_bone_is_not_a_hairline() {
        /// A proporção que a referência entrega, com folga: o octaedro do Blender fica perto de
        /// `5:1` e a nossa lei dá `5:1` até ao tecto (`14 px`), onde um osso muito longo chega a
        /// `192/28 ≈ 6,9:1`. ⛔ Acima de `8:1` volta a ler-se como uma linha.
        const PIOR_PROPORCAO: f64 = 8.0;
        for comp in COMPRIMENTOS_MEDIDOS_PX {
            let largura = 2.0 * bone_half_width_px(comp);
            let proporcao = comp / largura;
            assert!(
                proporcao <= PIOR_PROPORCAO,
                "um osso de {comp} px sai com {largura} px de corpo ({proporcao:.1}:1) - e' o \
                 defeito de 2026-09-06 ('os ossos viraram circulos') a voltar"
            );
        }
    }

    /// ⛔ **E o ANEL nunca engole o osso inteiro** — a metade da régua antiga que continua a valer,
    /// dita sobre a grandeza certa: o anel da junta vive na raiz, e um osso cujo comprimento não
    /// passa do diâmetro dele não tem corpo nenhum para agarrar.
    ///
    /// ⚠️ É o mesmo recurso que o tecto do [`joint_radius_px`] nomeia (o verbo de GIRAR), visto do
    /// outro lado — e é por isso que a barra é `comp > 2 × raio DESTE osso`, nunca da constante.
    #[test]
    fn the_joint_ring_never_swallows_the_bone_it_sits_on() {
        for comp in [8.0, 20.0, 48.0, 107.52, 192.0, 1000.0] {
            let anel = 2.0 * joint_radius_px(comp);
            assert!(
                anel <= comp,
                "num osso de {comp} px o anel mede {anel} px - ele cobre o osso inteiro"
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
