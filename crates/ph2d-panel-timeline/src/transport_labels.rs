//! **Os RÓTULOS da família de toggles da barra de transporte, e a coluna que eles partilham.**
//!
//! Módulo irmão do [`super`] por RESPONSABILIDADE e não por tamanho: aquele ficheiro dispõe a
//! barra (que item, onde, com que largura) e este responde **uma** pergunta — *que palavra cada
//! toggle mostra, e quanto espaço ela pede*. As duas crescem por motivos diferentes: a de lá
//! quando a barra ganha um controlo, esta quando um rótulo muda de língua ou de grafia.

use super::Item;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_tokens::TypeToken;

/// ⭐⭐⭐ **A COLUNA DE RÓTULO DOS TOGGLES MEDE A LISTA, NUNCA O RÓTULO EM MÃOS.**
///
/// Ela era o literal `52,0 px`, escolhido pela palavra `AutoKey` (`48,44`). Medido em
/// 2026-09-19, a família inteira dá:
///
/// | rótulo | inglês | idioma de teste |
/// |---|---:|---:|
/// | `Loop` | 28,70 | 44,82 |
/// | **`Ping-Pong`** | **60,45** | **91,34** |
/// | `Physics` | 44,51 | 67,97 |
/// | `AutoKey` | 48,44 | 71,37 |
/// | `Record` | 40,66 | 63,70 |
/// | `Path` | 25,78 | 42,02 |
/// | `Snap` | 29,20 | 45,33 |
/// | `Speed` | 36,69 | 56,09 |
/// | `Onion` | 33,91 | 53,67 |
/// | `Keys` | 27,96 | 44,09 |
///
/// ⛔ Em inglês **um** rótulo estourava os `52` e saía cortado — a grafia de então (`PingPong`,
/// `54,90`) e a que o dono escolheu (`Ping-Pong`, `60,45`), as duas. ⛔⛔ E no idioma de teste
/// estouram **SEIS dos dez**: *um número escolhido pela palavra mais larga do dia é uma aposta na
/// tradução que ainda não existe.* Hoje a coluna cresce com a língua sozinha — `60,45` em inglês,
/// `91,34` no idioma de teste — e ela é **TIGHT** (igual à palavra mais larga, sem folga): uma
/// folga escondida é onde o próximo rótulo cabe por sorte e o seguinte não.
///
/// ⚠️ **Mede-se no PESO em que se pinta** (`FontWeight::MEDIUM`, o do [`super::label`]) — a porta
/// declara-o uma vez, no ficheiro do pintor: medir num peso e pintar noutro corta exactamente na
/// fronteira em que o corte existe.
///
/// ⭐ É a mesma lei que o chip de escolha já declara um nível acima
/// (`ph2d_editor_core::widget::dropdown_label_budget`): *quem dimensiona uma superfície
/// PARTILHADA por uma família mede a família, nunca o membro que calhou estar à mão.*
///
/// ⭐⭐ **E os CHIPS numéricos (`Time` · `Frame` · `Length`) entram na MESMA medição** (2026-09-23,
/// ordem do dono *«alinhamento em todos os lugares»*). Eles tinham uma coluna própria de `48 px`
/// escrita à mão — a doença desta mesma nota, um nível ao lado —, e numa barra estreita, que
/// empilha um item por fileira, o valor de um chip arrancava a `69` e o interruptor da fileira de
/// baixo a `86,5` (varredura `onde_comeca_o_valor`, à largura do dono). Hoje os dois são a mesma
/// célula: `[pad | nome | pad | valor]`, e o valor cai no mesmo `x` sempre que empilham.
pub(super) fn toggle_label_w(ctx: &mut PaintCtx) -> f32 {
    ph2d_editor_core::paint::label_column_width(
        ctx.text_system,
        TypeToken::Sm.px(),
        super::ITEMS
            .iter()
            .filter_map(|&i| toggle_key(i).or_else(|| chip_key(i)))
            .map(ph2d_i18n::tr),
    )
}

/// **O rótulo que um chip numérico PINTA** — a mesma lista que [`toggle_label_w`] mede.
pub(super) fn chip_rotulo(item: Item) -> &'static str {
    chip_key(item).map(ph2d_i18n::tr).unwrap_or_default()
}

/// A chave crua de cada chip numérico da barra.
fn chip_key(item: Item) -> Option<&'static str> {
    Some(match item {
        Item::TimeChip => "panel.timeline.time_seconds",
        Item::FrameChip => "panel.timeline.frame",
        Item::LengthChip => "panel.timeline.length",
        _ => return None,
    })
}

/// **O rótulo que um toggle PINTA — a mesma lista de que [`toggle_label_w`] se mede.**
///
/// ⛔⛔ Ela existe para que a régua e o PINTOR leiam a mesma lista. Os rótulos viviam inline, um
/// `tr(...)` por braço do `match`, e uma lista escrita ao lado de um pintor é a segunda resposta
/// à mesma pergunta: quem acrescentasse um toggle com um rótulo mais largo pintá-lo-ia cortado
/// **sem tocar em nada que meça**. ⇒ o `paint_item` e o irmão de vista pedem o rótulo **por
/// aqui**, e um `Item` novo esquecido da tabela pinta **sem rótulo nenhum** — que se vê — em vez
/// de pintar um rótulo cortado, que não se vê.
pub(super) fn rotulo(item: Item) -> &'static str {
    toggle_key(item).map(ph2d_i18n::tr).unwrap_or_default()
}

/// A chave crua de cada toggle — a tabela que [`rotulo`] pinta e [`toggle_label_w`] mede.
fn toggle_key(item: Item) -> Option<&'static str> {
    Some(match item {
        Item::Loop => "panel.timeline.loop",
        Item::PingPong => "panel.timeline.ping_pong",
        Item::Physics => "panel.timeline.physics",
        Item::AutoKey => "panel.timeline.autokey",
        Item::Record => "panel.timeline.record",
        Item::MotionPath => "panel.timeline.motion_path",
        Item::Snap => "panel.timeline.snap",
        Item::Speed => "panel.timeline.speed",
        Item::Onion => "panel.timeline.onion",
        Item::OnionMode => "panel.timeline.onion_keys",
        _ => return None,
    })
}
