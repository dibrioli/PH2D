//! ⭐⭐⭐ **A ÂNCORA DE IK, desenhada** — o losango do alvo e o tracejado que o liga à ponta.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (o teto de 700 LOC do HR-18 pediu-o a `723`): o [`super`]
//! desenha o **OSSO** (o corpo, a junta, a ponta, a mancha de influência); aqui desenha-se o que
//! **manda** nele. São dois assuntos, e o segundo cresceu por cima do primeiro.
//!
//! ⚠️ As leis de tamanho vêm do irmão numa porta só ([`super::joint_radius_px`],
//! [`super::BONE_JOINT_R_PX`]): o losango nasce do tamanho do anel que ele substitui, e não de um
//! número próprio que envelheceria ao lado daquele.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

use super::{BONE_JOINT_R_PX, BoneHover, BonePart, LINE_PX, joint_radius_px};

/// ⭐⭐⭐ **O RAIO DO LOSANGO** — a porta única do desenho e do dedo, e ela é **maior que a bolinha
/// de propósito**.
///
/// ⛔ **É um report do dono** (2026-09-07): *«quando colocamos um IK num bone no meio dos ossos, o
/// losango do IK e o círculo do outro osso ficam sobrepostos»*. Uma âncora criada no MEIO de uma
/// corrente nasce exactamente sobre a junta do osso seguinte — dois alvos concêntricos com verbos
/// diferentes, e o dedo não tem como escolher.
///
/// ⭐ **A cura é a que ele propôs, e é a única que funciona para alvos concêntricos: eles têm de
/// diferir em TAMANHO.** O losango fica por FORA, a bolinha por dentro, e o anel entre os dois é a
/// zona exclusiva da âncora.
///
/// ⚠️ **O PISO desse anel é DERIVADO, não escolhido:** ele é exactamente [`BONE_JOINT_R_PX`] — a
/// tolerância do dedo desta casa (o mesmo `HANDLE_HIT_PX` que toda alça do vector usa). ⇒ o artista
/// tem sempre **pelo menos um dedo inteiro** de anel para agarrar a âncora, seja qual for o zoom e
/// mesmo com a bolinha por dentro no tamanho máximo.
///
/// ⭐ **E por cima do piso vem o [`GOAL_BIGGER`], que é decisão do DONO** (2026-09-07: *«o losango
/// deve ser 25% maior»*), depois de ver a primeira versão na tela. ⛔ Ele **não** é um teto nem um
/// limite de recurso — é a leitura, e quem a julga é o smoke. Medido: num osso longo o losango passa
/// de `24` para **`30` px** de meia-diagonal, e o anel exclusivo de `12` para **`18`**.
#[must_use]
pub fn goal_radius_px(comp: f64) -> f64 {
    (joint_radius_px(comp) + BONE_JOINT_R_PX) * GOAL_BIGGER
}

/// Quanto o losango cresce por cima do piso derivado — **veredito do dono sobre a tela**, e por isso
/// um número de produto e não uma medição.
///
/// ⚠️ Ele multiplica o RAIO inteiro, e não só o anel: *«25% maior»* é sobre o losango que se vê. E
/// como o piso já garantia um dedo de anel, subir aqui só **alarga** a zona exclusiva da âncora —
/// nunca a aperta.
const GOAL_BIGGER: f64 = 1.25;

/// A espessura do traço do losango.
///
/// ⚠️ **Mais grossa que a das outras alças, e o dono pediu-o pelo nome** (*«que seu gizmo tenha
/// espessura maior»*): num par concêntrico o que está por FORA tem de se ler como o alvo maior,
/// senão o anel exclusivo existe e não se vê. ⛔ Número de PRODUTO, e declarado como tal — o recurso
/// que ele nomeia é a leitura da tela, e o smoke é quem o julga.
const GOAL_LINE_PX: f64 = 2.5;

/// **Uma âncora, como o desenho a lê** — `(osso, alvo, origem do osso, ponta do osso)` em MUNDO.
///
/// ⚠️ O segmento do osso vem junto porque o tamanho do losango sai da mesma porta da bolinha
/// ([`joint_radius_px`], sobre o comprimento **na tela**): sem ele o desenho teria de re-encontrar o
/// osso, que é a segunda resposta à mesma pergunta.
pub type Goal = (u64, [f64; 2], [f64; 2], [f64; 2]);

/// ⭐⭐⭐ **A ÂNCORA DE IK** — o losango do alvo, mais o tracejado que o liga à ponta da corrente.
///
/// # Por que um LOSANGO e não mais um anel
///
/// Porque o app já tem duas alças redondas (a junta e a ponta) e um quadrado (a força), e uma
/// terceira redonda obrigaria o artista a **decorar** qual é qual. *A forma carrega o verbo*: o
/// losango é o alvo, e ele lê-se como alvo em toda a referência (o Blender desenha o alvo de IK
/// como um *empty*, o Spine como uma cruz).
///
/// ⭐⭐ **E ele SUBSTITUI o anel da ponta, não se soma a ele.** Num osso com âncora a ponta deixa de
/// ser agarrável — o que se arrasta é o alvo — e desenhar as duas coisas por cima uma da outra
/// prometeria dois verbos onde há um. Quem decide é o `draw_bones`, que já recebe a lista de
/// pontas: a shell tira dela quem tem âncora.
///
/// ⚠️ **O tamanho sai da MESMA porta da bolinha** ([`joint_radius_px`], sobre o comprimento do osso
/// na tela): o losango nasce exactamente do tamanho do anel que ele substitui, e não de um número
/// próprio que envelheceria ao lado daquele.
///
/// ⚠️ **O tracejado só se vê quando eles se separam** — o que acontece quando o alvo está FORA DE
/// ALCANCE, e é aí que ele é informação: ele diz *«a corrente esticou e não chegou»*, que é o único
/// estado em que o artista precisa de ver os dois pontos.
pub fn draw_goals(
    goals: &[Goal],
    selected: Option<u64>,
    hover: Option<BoneHover>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (aceso, apagado) = (vello(ColorToken::Accent), vello(ColorToken::AccentSoft));
    for &(bits, ancora, a, b) in goals {
        let (pa, pb) = (
            transform * Point::new(a[0], a[1]),
            transform * Point::new(b[0], b[1]),
        );
        let comp = (pb.x - pa.x).hypot(pb.y - pa.y);
        if comp <= f64::EPSILON {
            continue;
        }
        let p = transform * Point::new(ancora[0], ancora[1]);
        let sel = Some(bits) == selected;
        let sob = hover.is_some_and(|h| h.bone == bits && h.part == BonePart::Tip);
        let cor = if sel || sob { aceso } else { apagado };
        // O TRACEJADO, primeiro: ele é um fundo, e o losango desenha-se por cima dele.
        let mut fio = BezPath::new();
        fio.move_to(pb);
        fio.line_to(p);
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX).with_dashes(0.0, DASH),
            Affine::IDENTITY,
            &Brush::Solid(apagado),
            None,
            &fio,
        );
        let r = goal_radius_px(comp);
        let mut losango = BezPath::new();
        losango.move_to(Point::new(p.x, p.y - r));
        losango.line_to(Point::new(p.x + r, p.y));
        losango.line_to(Point::new(p.x, p.y + r));
        losango.line_to(Point::new(p.x - r, p.y));
        losango.close_path();
        // ⛔ **O losango NÃO se enche, ao contrário das outras alças.** Ele é o de FORA de um par
        // concêntrico: enchê-lo tapava a bolinha do osso que vive por dentro dele, e o artista
        // deixava de ver o alvo que o clique de dentro pega. *A gramática do preenchimento vale
        // para alças que estão sozinhas.* Aqui quem diz «escolhido» é a COR, que é o outro canal.
        let _ = sel;
        target.inner_mut().stroke(
            &Stroke::new(GOAL_LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            None,
            &losango,
        );
    }
}

/// O tracejado do fio âncora→ponta. Padrão de marching-ants da casa, o mesmo das guias e da
/// timeline.
const DASH: [f64; 2] = [4.0, 3.0]; // LITERAL-PX-OK: marching-ants dash pattern
