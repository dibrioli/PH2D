//! ⭐⭐⭐ **O QUE UMA LINHA DE MENU É, E QUANDO ELA ESTÁ ACESA.**
//!
//! Irmão do [`super::context_menu_overlay`], que é quem a DESENHA — e o corte é por
//! responsabilidade, não por tamanho: aquele ficheiro responde *«onde cada coisa fica no cartão»* e
//! este responde *«esta linha é um comando ou um risco, e é ela o estado corrente?»*. O primeiro
//! mexe em rectângulos; este não sabe o que é um pixel.
//!
//! ⚠️ **Foi o tecto de LOC que forçou o corte** (`724` contra `700`, 2026-09-13), e ele saiu melhor
//! por isso: a marca passou a ter um ficheiro com o nome da pergunta que responde, ao lado do gate
//! que já a media (`context_menu_overlay_tests.rs`). ⛔ **Nunca uma entrada no `FILE_OVERAGE_OK`** —
//! a catraca deste repo só desce.

use crate::ids;
use crate::interaction::WidgetStore;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;

use super::context_menu_overlay::ROW_H;

/// **Uma linha da coluna do menu** — um COMANDO, ou o RISCO que separa dois grupos.
///
/// # ⛔⛔ Porque ela nasceu (report do dono, 2026-09-13: *«funciona mas não totalmente»*)
///
/// O pulldown *Shading* do modelador contribui **três fileiras** — o modo (`Matcap`/`Render`), a
/// vista da cena (`Standard`/`Neutral`) e as cinco exposições —, e elas chegavam ao ecrã como
/// **nove linhas numa coluna única**, sem nada a dizer que são três perguntas diferentes. Um
/// artista a ler `Render · Standard · Exposure 0` seguidos não tem como saber que os três são
/// rádios independentes.
///
/// ⭐ **E o divisor já existia na casa** — [`crate::widget::ToolRailEntry::Divider`], a linha de
/// `1 px` em `Border` que o trilho desenha desde sempre. ⛔ O que faltava era o CONSUMIDOR: o
/// merge deste menu fazia `filter_map(|e| Some((e.node_id()?, e.label()?, None)))`, e um `Divider`
/// não tem nem id nem rótulo ⇒ **todo divisor contribuído evaporava-se em silêncio**. *Um produtor
/// sem consumidor e um consumidor sem produtor dão o mesmo ecrã, e as curas são opostas.*
pub(super) enum MenuRow<'a> {
    Item(NodeId, &'a str, Option<[u8; 4]>),
    Divider,
}

impl MenuRow<'_> {
    /// A altura que esta linha ocupa na coluna.
    ///
    /// ⚠️ **É a MESMA aritmética do trilho** (`tool_rail::entry_advance`): a linha mede `1 px` e a
    /// banda dela é `1 + gap × 2`. Um segundo número aqui faria o mesmo separador ter duas
    /// espessuras conforme o sítio onde é desenhado.
    pub(super) fn h(&self) -> f32 {
        match self {
            Self::Item(..) => ROW_H,
            Self::Divider => 1.0 + ph2d_tokens::DIVIDER_GAP_PX * 2.0,
        }
    }
}

/// Does `id` correspond to the currently-active choice for any of
/// the single-choice menus? The row paint draws an accent bullet
/// next to the row whose id matches, mirroring SceneList's "current
/// scene" indicator (2026-05-24 menu standardization).
///
/// One function covers every menu because IDs are unique across all
/// menu kinds — checking `id == active_theme_id || id == active_radius
/// _id || ...` is unambiguous and avoids threading `kind` through the
/// row loop.
pub(super) fn id_is_currently_selected(
    id: NodeId,
    theme: Theme,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    motion: &crate::motion::UiMotion,
) -> bool {
    use crate::project::{DisplayAngle, DisplayUnit, ImageFilterMode};
    use crate::widget::RailButtonSize;
    // ⛔⛔ **AS DEZASSEIS LINHAS DE ALTERNÂNCIA DA BARRA DE MENUS** — os treze módulos, os dois
    // painéis e a régua. Elas nasceram em 2026-08-30 **sem marca nenhuma**: o menu *Window* dizia
    // exactamente a mesma coisa com o Vector aberto e fechado.
    //
    // ⚠️ É a lei que este ficheiro já documenta, paga na unidade de ângulo: *«fiar o clique não é
    // fiar o ESTADO»* — e a barra repetiu-a dezasseis vezes de uma vez. Antes dela a indicação
    // existia: o laço de reconciliação da shell força `Pressed` no pill do tool activo, e o pill
    // lia-o. O pill saiu; a marca não foi com ele para lado nenhum.
    if super::menu_bar::row_is_marked_by_button_state(id) {
        return matches!(
            store.button_state(id),
            Some(crate::widget::ButtonState::Pressed)
        );
    }
    let theme_id = super::theme_menu::theme_menu_id(theme);
    if id == theme_id {
        return true;
    }
    const RADIUS_ROUND_THRESH: f32 = 1.3; // LITERAL-PX-OK: midpoint between Default(1.0) and Round(1.5) radius preset
    let radius_id = if store.radius_scale() < 0.5 {
        ids::CTX_MENU_RADIUS_SHARP
    } else if store.radius_scale() > RADIUS_ROUND_THRESH {
        ids::CTX_MENU_RADIUS_ROUND
    } else {
        ids::CTX_MENU_RADIUS_DEFAULT
    };
    if id == radius_id {
        return true;
    }
    let rail_id = match store.rail_button_size() {
        RailButtonSize::Small => ids::CTX_MENU_RAIL_SIZE_SMALL,
        RailButtonSize::Medium => ids::CTX_MENU_RAIL_SIZE_MEDIUM,
        RailButtonSize::Large => ids::CTX_MENU_RAIL_SIZE_LARGE,
    };
    if id == rail_id {
        return true;
    }
    let ppm_id = match project.pixels_per_meter as i32 {
        16 => Some(ids::CTX_MENU_PPM_16),
        32 => Some(ids::CTX_MENU_PPM_32),
        100 => Some(ids::CTX_MENU_PPM_100),
        256 => Some(ids::CTX_MENU_PPM_256),
        1024 => Some(ids::CTX_MENU_PPM_1024),
        _ => None,
    };
    if ppm_id == Some(id) {
        return true;
    }
    let unit_id = match project.display_unit {
        DisplayUnit::Meters => ids::CTX_MENU_UNIT_METERS,
        DisplayUnit::Pixels => ids::CTX_MENU_UNIT_PIXELS,
    };
    if id == unit_id {
        return true;
    }
    // Angle unit — a irmã do `display_unit` acima (Enio, 2026-08-30). ⚠️ **Esta linha faltou na
    // 1.ª entrega da feature**, e o defeito é da família que este ficheiro existe para curar: o
    // menu abria, o clique funcionava e o valor gravava — mas **nenhuma das duas opções aparecia
    // marcada**, então não havia como ver em que unidade se estava sem abrir o Inspector e
    // comparar. *Fiar o clique não é fiar o ESTADO.*
    let angle_id = match project.display_angle {
        DisplayAngle::Degrees => ids::CTX_MENU_ANGLE_DEGREES,
        DisplayAngle::Radians => ids::CTX_MENU_ANGLE_RADIANS,
    };
    if id == angle_id {
        return true;
    }
    let filter_id = match project.image_filter {
        ImageFilterMode::PixelArt => ids::CTX_MENU_FILTER_PIXELART,
        ImageFilterMode::Smooth => ids::CTX_MENU_FILTER_SMOOTH,
    };
    if id == filter_id {
        return true;
    }
    // Text rendering — value lives on the `paint::text_rendering`
    // thread-local (published per-frame from `HeroScreen.text_rendering`),
    // so we can read it here without threading another param through.
    let text_id = match crate::paint::text_rendering() {
        ph2d_tokens::TextRendering::Default => ids::CTX_MENU_TEXT_DEFAULT,
        ph2d_tokens::TextRendering::CrispHeavy => ids::CTX_MENU_TEXT_CRISP_HEAVY,
        ph2d_tokens::TextRendering::CrispHeavyPlus => ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS,
    };
    if id == text_id {
        return true;
    }
    // Display submenu (VSync / Immediate) — store mirrors the last
    // value `settings_present::apply` published; default `true` matches
    // the shell's `Fifo` baseline.
    let display_id = if store.present_vsync() {
        ids::CTX_MENU_DISPLAY_VSYNC
    } else {
        ids::CTX_MENU_DISPLAY_IMMEDIATE
    };
    if id == display_id {
        return true;
    }
    // Motion — o carácter é um RÁDIO (uma linha acesa das duas) e o reduced motion é um TOGGLE
    // (aceso quando ligado). ⚠️ São perguntas independentes, então são dois `if` e não um `match`:
    // *Expressivo + reduced* tem de conseguir acender as duas linhas ao mesmo tempo.
    let character_id = match motion.character() {
        crate::motion::UiCharacter::Discrete => ids::CTX_MENU_MOTION_DISCRETE,
        crate::motion::UiCharacter::Expressive => ids::CTX_MENU_MOTION_EXPRESSIVE,
    };
    if id == character_id {
        return true;
    }
    if id == ids::CTX_MENU_MOTION_REDUCED && motion.reduced_motion() {
        return true;
    }
    false
}

/// ⭐⭐⭐ **A MARCA de uma linha CONTRIBUÍDA é o `ButtonState` que o MÓDULO DONO publica.**
///
/// # ⛔⛔ O defeito que isto cura (report do dono, 2026-09-13)
///
/// > *«coloque a marca de seleção no render selecionado (ponto como nos menus do topo)»*
///
/// O [`id_is_currently_selected`] responde por uma tabela de ids que o `ph2d-editor-core`
/// **conhece** — e as linhas de um pulldown de área são de **outro módulo por construção**: a
/// **D2** existe precisamente para esta crate não conhecer os ids de cada editor
/// ([`crate::interaction::AreaMenu`]). Resultado medido: as **nove** linhas do pulldown *Shading*
/// do modelador (o modo · as vistas · as exposições) abriam **todas iguais**, e não havia como ler
/// em que exposição a cena estava sem a mexer e comparar.
///
/// ⭐ **A verdade já estava publicada**, e há semanas: o `publish` de cada painel de área escreve
/// `Pressed` no chip aceso de cada fileira — a mesma lei do
/// [`super::menu_bar::publish_toggle_state`]. O que faltava era quem a **lesse na hora de pintar**.
/// *Fiar o clique não é fiar o ESTADO — e publicar o estado ainda não é PINTÁ-LO.*
///
/// # ⚠️ A cerca «é contribuída» é obrigatória, e não é arrumação
///
/// O `dispatch::pointer_down` também escreve `Pressed` num botão **enquanto o dedo está em baixo**
/// ([`crate::interaction`], `dispatch/hover.rs`). Sem esta cerca, uma linha **estática** acenderia
/// o ponto por baixo do dedo e a marca passaria a dizer *«é aqui que estou a carregar»* em vez de
/// *«é este o estado»* — duas perguntas diferentes com o mesmo glifo.
///
/// ⚠️ **E as linhas de AÇÃO ficam de fora sem um `if` a dizê-lo:** um *Export Fine* nunca é o
/// estado de nada, então o dono dele nunca lhe escreve `Pressed`. *A lista que decide é a do
/// módulo, não uma segunda aqui.*
pub(super) fn contributed_row_is_current(
    id: NodeId,
    contrib: &[crate::widget::ToolRailEntry],
    store: &WidgetStore,
) -> bool {
    contrib.iter().any(|e| e.node_id() == Some(id))
        && matches!(
            store.button_state(id),
            Some(crate::widget::ButtonState::Pressed)
        )
}
