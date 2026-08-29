//! **Os ids da secção Texture Pattern** — módulo irmão de [`super`] pelo teto de LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_patternpath`: estes são os controles da TINTA
//! de uma forma quando ela é um padrão de textura (plano 33) — qual arte, que reticulado, que
//! tamanho, onde.
//!
//! ⚠️⚠️ **NÃO confundir com o `vector_patternpath`.** Aquele é o *Pattern Along Path* (plano 23): um
//! MOTIVO copiado ao longo de uma guia, com alças e picker. Este é o preenchimento. Os dois têm a
//! palavra *pattern* no nome e são coisas diferentes — a linha já se enganou uma vez, ao chamar o
//! módulo novo de `pattern_live` e sobrescrever o que já existia.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os irmãos: um id é o hash de uma STRING, então reordenar não
//! quebra nada — mas renomear uma string quebra tudo o que a referencia por nome, e é assim que um
//! widget fica órfão em silêncio.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

/// Secção **PATTERN do PREENCHIMENTO** — a tinta do miolo quando ela é um padrão de textura.
pub const VECTOR_SECTION_TEXPAT: NodeId = hash_node_id("vector.section.texpat");

/// ⭐⭐ Secção **PATTERN do TRAÇO** — a irmã, e o pedido do Enio de 2026-08-28: *"cada seção deve
/// ter seus ajustes próprios"*.
///
/// ⛔ **Ela SUBSTITUI a fileira `Fill | Stroke`** que a wave D pôs no topo de uma secção
/// partilhada. O plano 35 §2.4 recusava duplicar a secção — *"onze fileiras a dobrar, e as duas
/// divergiriam no primeiro knob novo"* —, e a recusa estava certa sobre o **CÓDIGO** e errada sobre
/// a **UI**: um alvo escondido num chip faz o artista mexer num knob e ver o outro sujeito mudar,
/// que foi exactamente o report que a wave D colheu.
///
/// ⭐ A divergência que a recusa temia continua impossível, e por construção: os controlos nascem
/// de **uma** função de pintura e de **uma** família de ids ([`texpat_id`]), com o slot por
/// parâmetro. Um knob novo aparece nas duas secções sozinho.
pub const VECTOR_SECTION_TEXPAT_STROKE: NodeId = hash_node_id("vector.section.texpat.stroke");

/// Quantas TINTAS a secção *Pattern* endereça: `0` = preenchimento, `1` = traço.
///
/// ⚠️ É o **espelho** do `ph2d_vec_render::PatternSlot`, e vive aqui porque o editor-core não
/// depende da crate de desenho — a mesma razão pela qual o painel espelha o `FillKind`.
pub const TEXPAT_SLOTS: usize = 2;

/// **Que controlo da secção *Pattern***, sem dizer de qual das duas tintas — o slot é o outro
/// parâmetro de [`texpat_id`].
///
/// ⭐ **Um enum, e não vinte constantes por secção.** A lista é fechada e o compilador conhece-a:
/// um knob novo é uma variante, e ele nasce com id nas DUAS secções, registado e roteado, sem que
/// ninguém se lembre de o duplicar. *Era a divergência que o plano 35 §2.4 temia — e ela morre no
/// tipo, não na disciplina.*
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TexPatKnob {
    /// **Source…** — troca a ARTE (abre o diálogo de ficheiro).
    Source,
    /// **Use Shape…** — arma o gesto de duas mãos (a arte passa a ser uma forma do documento).
    PickShape,
    /// O chip de reticulado `i` (`0` Grid · `1` Brick · `2` Column · `3` Hex).
    Tile(u8),
    /// O chip de repetição `i` (`0` Tile · `1` Mirror · `2` Clamp).
    Mode(u8),
    /// **Offset** — o denominador do desfasamento (`1/n`).
    Offset,
    /// O campo numérico gémeo do [`Self::Offset`].
    OffsetNum,
    /// **Width** — a largura de uma cópia, em unidades de MUNDO.
    Width,
    /// O campo numérico gémeo da [`Self::Width`].
    WidthNum,
    /// **Height** — a altura de uma cópia, em unidades de MUNDO.
    Height,
    /// O campo numérico gémeo da [`Self::Height`].
    HeightNum,
    /// ⭐ **Lock Aspect** — mexer num eixo leva o outro. Preserva a razão ACTUAL, não a natural da
    /// arte, e por isso descreve o GESTO: ele vive na sessão e **não viaja no ficheiro**.
    Lock,
    /// **Gap** — o vão acrescentado (negativo = sobreposição).
    Gap,
    /// O campo numérico gémeo do [`Self::Gap`].
    GapNum,
    /// **Angle** — a rotação do padrão, em graus.
    Angle,
    /// O campo numérico gémeo do [`Self::Angle`].
    AngleNum,
    /// **Shift X** — a fase dentro de UMA repetição, em % do eixo X do padrão.
    ShiftX,
    /// O campo numérico gémeo do [`Self::ShiftX`].
    ShiftXNum,
    /// **Shift Y** — a fase ao longo do eixo Y do padrão.
    ShiftY,
    /// O campo numérico gémeo do [`Self::ShiftY`].
    ShiftYNum,
}

impl TexPatKnob {
    /// **Todos os controlos**, na ordem em que a secção os pinta.
    ///
    /// ⚠️ É esta lista que o `populate` regista e que o roteamento percorre para resolver um id de
    /// volta em `(slot, knob)`. Uma variante fora daqui nasce **pintada e morta** — a lacuna que
    /// esta casa já pagou com 36 células de física e dez chips do Painter.
    pub const ALL: [Self; 24] = [
        Self::Source,
        Self::PickShape,
        Self::Tile(0),
        Self::Tile(1),
        Self::Tile(2),
        Self::Tile(3),
        Self::Offset,
        Self::OffsetNum,
        Self::Width,
        Self::WidthNum,
        Self::Height,
        Self::HeightNum,
        Self::Lock,
        Self::Gap,
        Self::GapNum,
        Self::ShiftX,
        Self::ShiftXNum,
        Self::ShiftY,
        Self::ShiftYNum,
        Self::Angle,
        Self::AngleNum,
        Self::Mode(0),
        Self::Mode(1),
        Self::Mode(2),
    ];
}

/// O [`NodeId`] do controlo `knob` da secção da tinta `slot`.
///
/// ⚠️ Runtime `format!` + gémeo FNV no mesmo espaço de ids — a mesma fábrica do
/// `vector_marker_option_id` e dos chips do catálogo de formas.
#[must_use]
pub fn texpat_id(slot: usize, knob: TexPatKnob) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("vector.texpat.{slot}.{knob:?}"))
}

// ── A secção BRUSH (plano 36, W4) ─────────────────────────────────────────────
//
// ⚠️ **Secção PRÓPRIA, e não mais um alvo da família do padrão.** Os knobs são OUTROS: um pincel
// tem avanço e escala relativa; um padrão tem reticulado, fase e modo de repetição. Metade dos de
// cada um ficaria morta na outra — que é exactamente o defeito que a wave F do plano 35 curou ao
// separar as duas secções do padrão.
/// Secção **BRUSH** — a arte que percorre o contorno.
pub const VECTOR_SECTION_BRUSH: NodeId = hash_node_id("vector.section.brush");
/// **Use Shape…** — arma o gesto de duas mãos: o clique seguinte no canvas escolhe a ARTE.
pub const VECTOR_BRUSH_PICK_SHAPE: NodeId = hash_node_id("vector.brush.pick");
/// **Spacing** — multiplica a largura do motivo para dar o avanço por cópia.
pub const VECTOR_BRUSH_SPACING: NodeId = hash_node_id("vector.brush.spacing");
/// O campo numérico gémeo do [`VECTOR_BRUSH_SPACING`].
pub const VECTOR_BRUSH_SPACING_NUM: NodeId = hash_node_id("vector.brush.spacing.num");
/// **Size** — multiplica a altura DERIVADA da largura do traço (`1` = a arte tem a altura da faixa).
pub const VECTOR_BRUSH_SCALE: NodeId = hash_node_id("vector.brush.scale");
/// O campo numérico gémeo do [`VECTOR_BRUSH_SCALE`].
pub const VECTOR_BRUSH_SCALE_NUM: NodeId = hash_node_id("vector.brush.scale.num");
/// **Offset** — desvio ao longo da NORMAL, positivo para a esquerda do sentido de marcha.
pub const VECTOR_BRUSH_OFFSET: NodeId = hash_node_id("vector.brush.offset");
/// O campo numérico gémeo do [`VECTOR_BRUSH_OFFSET`].
pub const VECTOR_BRUSH_OFFSET_NUM: NodeId = hash_node_id("vector.brush.offset.num");
/// **Rotation** — orientação do motivo sobre a curva, em GRAUS.
pub const VECTOR_BRUSH_ROTATION: NodeId = hash_node_id("vector.brush.rotation");
/// O campo numérico gémeo do [`VECTOR_BRUSH_ROTATION`].
pub const VECTOR_BRUSH_ROTATION_NUM: NodeId = hash_node_id("vector.brush.rotation.num");
/// **Flip** — a arte do outro lado da curva, a percorrê-la ao contrário.
pub const VECTOR_BRUSH_FLIP: NodeId = hash_node_id("vector.brush.flip");
