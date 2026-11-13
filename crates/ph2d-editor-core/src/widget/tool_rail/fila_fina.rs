//! ⭐ **A fila de ferramentas FINA** (dono 12/11, maquete C «A faixa que responde», `.c15tl`): uma
//! linha de `TOOL_ROW_H_PX` debaixo da faixa, glifos de `TOOL_ROW_ICON_PX` sem legenda por cima, as
//! opções (Global, Selected, Object Mode) como texto. O nome de cada botão vai na dica.
//!
//! ⛔ Contraste (dono): glifo e borda do escolhido ≥ 3:1 contra o chão, texto ≥ 4,5:1, em TODO tema —
//! as cores saem de [`TINTA`] e o gate `a_fila_fina_tem_contraste_em_todo_tema` mede-as.

use super::*;
use crate::widget::cromo_barra::Barra;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{TOOL_ROW_GAP_PX, TOOL_ROW_GROUP_GAP_PX, TOOL_ROW_ICON_PX, TOOL_ROW_PAD_PX};

/// As cores da fila fina, por papel. O gate de contraste lê ESTA tabela.
pub struct TintaFina {
    /// O glifo em repouso.
    pub glifo: ColorToken,
    /// O glifo e o texto sob o ponteiro.
    pub aceso: ColorToken,
    /// O glifo do escolhido.
    pub escolhido: ColorToken,
    /// A borda do chip do escolhido.
    pub borda: ColorToken,
    /// O texto das opções.
    pub texto: ColorToken,
}

/// ⭐ A tinta da fila fina.
pub const TINTA: TintaFina = TintaFina {
    glifo: ColorToken::Text2,
    aceso: ColorToken::Text1,
    escolhido: ColorToken::KanbanBlueBold,
    borda: ColorToken::KanbanBlueBold,
    texto: ColorToken::Text2,
};

/// O lado do chip de um glifo: o glifo e o respiro, metade de cada lado.
#[must_use]
pub fn lado_do_glifo() -> f32 {
    TOOL_ROW_ICON_PX + Spacing::Sm.px()
}

/// O respiro do texto de uma opção dentro do chip dela.
fn folga_do_texto() -> f32 {
    Spacing::Sm.px()
}

/// O tamanho de letra das opções da fila fina.
#[must_use]
pub fn fonte_fina() -> f32 {
    TypeToken::Sm.px()
}

/// A face de uma opção sem a seta do produtor: a fila fina é texto, como na maquete.
fn face_limpa(face: &str) -> &str {
    face.strip_suffix('▾').unwrap_or(face).trim_end()
}

/// Um glifo (ou amostra) — o que avança `lado_do_glifo` e agrupa com o vizinho.
fn e_glifo(entry: &ToolRailEntry) -> bool {
    matches!(
        entry,
        ToolRailEntry::Icon { .. } | ToolRailEntry::Glyph { .. } | ToolRailEntry::Swatch { .. }
    )
}

/// A fila só se pinta fina quando todas as entradas são glifos, opções ou divisores: um campo
/// numérico sem a legenda dele não diz o que é (a barra de um módulo continua alta).
#[must_use]
pub fn cabe_na_fila_fina(rail: &ToolRail) -> bool {
    rail.entries.iter().all(entrada_fina)
}

/// Uma entrada que a fila fina sabe pintar: glifo, amostra, opção ou divisor.
#[must_use]
pub fn entrada_fina(e: &ToolRailEntry) -> bool {
    e_glifo(e) || matches!(e, ToolRailEntry::Compound { .. } | ToolRailEntry::Divider)
}

/// ⭐ Mede a fila fina: cada opção fica com a largura da face mais larga que pode mostrar (o chip não
/// muda de tamanho quando o estado muda), cada glifo com o lado dele.
pub fn medir_fila_fina(rail: &mut ToolRail, text_system: &mut TextSystem) {
    let font = fonte_fina();
    for entry in &mut rail.entries {
        match entry {
            ToolRailEntry::Compound {
                faces, face, row_w, ..
            } => {
                let widest = faces
                    .iter()
                    .chain(std::iter::once(&*face))
                    .map(|f| {
                        text_system.prefix_width_weighted(face_limpa(f), font, FontWeight::MEDIUM)
                    })
                    .fold(0.0_f32, f32::max);
                *row_w = widest.ceil() + folga_do_texto() * 2.0;
            }
            ToolRailEntry::Icon { row_w, .. } | ToolRailEntry::Glyph { row_w, .. } => {
                *row_w = lado_do_glifo();
            }
            _ => {}
        }
    }
}

/// O respiro de uma entrada dentro do slot dela — o vão VISÍVEL entre itens desconta-o.
fn respiro(entry: &ToolRailEntry) -> f32 {
    if e_glifo(entry) {
        (lado_do_glifo() - TOOL_ROW_ICON_PX) * 0.5
    } else {
        folga_do_texto()
    }
}

fn avanco(entry: &ToolRailEntry) -> f32 {
    match entry {
        ToolRailEntry::Compound { row_w, .. } => *row_w,
        ToolRailEntry::Divider => 0.0,
        _ => lado_do_glifo(),
    }
}

/// ⭐⭐ **Onde cai cada entrada da fila fina** — a porta que o pintor, o hit e o transbordo perguntam.
/// O divisor não se pinta: ele só parte o grupo (o vão passa de `TOOL_ROW_GROUP_GAP_PX` a
/// `TOOL_ROW_GAP_PX`). `rect` é a faixa inteira; o respiro lateral sai daqui.
#[must_use]
pub fn fila_fina_slots(rail: &ToolRail, rect: Rect) -> Vec<EntrySlot> {
    let lado = lado_do_glifo();
    let y = rect.y + (rect.h - lado) * 0.5;
    let mut x = rect.x + TOOL_ROW_PAD_PX;
    let mut anterior: Option<&ToolRailEntry> = None;
    let mut partiu = false;
    let mut out = Vec::with_capacity(rail.entries.len());
    for (index, entry) in rail.entries.iter().enumerate() {
        if matches!(entry, ToolRailEntry::Divider) {
            partiu = true;
            out.push(EntrySlot {
                index,
                id: None,
                rect: Rect::new(x, y, 0.0, lado),
            });
            continue;
        }
        if let Some(a) = anterior {
            let mesmo_grupo = !partiu && e_glifo(a) && e_glifo(entry);
            let vao = if mesmo_grupo {
                TOOL_ROW_GROUP_GAP_PX
            } else {
                TOOL_ROW_GAP_PX
            };
            x += (vao - respiro(a) - respiro(entry)).max(0.0);
        } else {
            x -= respiro(entry);
        }
        let w = avanco(entry);
        out.push(EntrySlot {
            index,
            id: entry.node_id(),
            rect: Rect::new(x, y, w, lado),
        });
        x += w;
        anterior = Some(entry);
        partiu = false;
    }
    out
}

/// A fila inteira cabe em `width`?
#[must_use]
pub fn fila_fina_cabe(rail: &ToolRail, width: f32) -> bool {
    let fim = fila_fina_slots(rail, Rect::new(0.0, 0.0, width, lado_do_glifo()))
        .last()
        .map_or(0.0, |s| s.rect.x + s.rect.w);
    fim <= width
}

fn cor(barra: Barra, token: ColorToken) -> ph2d_vector::Color {
    barra.tinta().color(token)
}

/// O chip do escolhido / sob o ponteiro: o tom do campo e, no escolhido, a borda que se vê.
fn chip_de_estado(
    scene: &mut VectorScene,
    barra: Barra,
    theme: Theme,
    r: Rect,
    on: bool,
    sob: bool,
) {
    if on {
        fill_rounded_rect(
            scene,
            r,
            m::FIELD_RADIUS,
            cor(barra, ColorToken::KanbanCardRaised),
        );
        crate::paint::stroke_frame(
            scene,
            r,
            m::FIELD_RADIUS,
            theme,
            ph2d_tokens::visuals::Feel::Focused,
            StrokeToken::Hairline.px(),
            cor(barra, TINTA.borda),
        );
    } else if sob {
        fill_rounded_rect(
            scene,
            r,
            m::FIELD_RADIUS,
            cor(barra, ColorToken::CardFieldHover),
        );
    }
}

fn estado(store: &WidgetStore, id: NodeId) -> (ButtonState, bool) {
    let s = store.button_state(id).unwrap_or(ButtonState::Normal);
    let sob = matches!(s, ButtonState::Hovered | ButtonState::Focused);
    (s, sob)
}

/// ⭐ Pinta a fila fina. Os alvos regista-os quem chama, pelos mesmos [`fila_fina_slots`].
#[allow(clippy::too_many_arguments)]
pub fn paint_fila_fina(
    barra: Barra,
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
) {
    let traco = StrokeToken::Default.px();
    for slot in fila_fina_slots(rail, rect) {
        let r = slot.rect;
        let Some(entry) = rail.entries.get(slot.index) else {
            continue;
        };
        let glifo = Rect::new(
            r.x + (r.w - TOOL_ROW_ICON_PX) * 0.5,
            r.y + (r.h - TOOL_ROW_ICON_PX) * 0.5,
            TOOL_ROW_ICON_PX,
            TOOL_ROW_ICON_PX,
        );
        match entry {
            ToolRailEntry::Icon {
                id, icon, active, ..
            } => {
                let (s, sob) = estado(store, *id);
                let on = *active || s == ButtonState::Pressed;
                chip_de_estado(scene, barra, theme, r, on, sob);
                let fg = if on {
                    TINTA.escolhido
                } else if sob {
                    TINTA.aceso
                } else {
                    TINTA.glifo
                };
                paint_icon(scene, *icon, glifo, cor(barra, fg), traco);
            }
            ToolRailEntry::Glyph {
                id, path, active, ..
            } => {
                let (s, sob) = estado(store, *id);
                let on = *active || s == ButtonState::Pressed;
                chip_de_estado(scene, barra, theme, r, on, sob);
                crate::widget::paint_icon_button(
                    glifo,
                    crate::widget::IconGlyph::Path(path),
                    crate::widget::IconButtonStyle::Plain,
                    (s, crate::motion::SETTLED),
                    scene,
                    theme,
                );
            }
            ToolRailEntry::Swatch {
                id,
                color,
                active,
                mixed,
                ..
            } => {
                let (s, sob) = estado(store, *id);
                let on = *active || s == ButtonState::Pressed;
                chip_de_estado(scene, barra, theme, r, on, sob);
                if *mixed {
                    crate::widget::paint_mixed_swatch(glifo, scene, theme);
                } else {
                    let [cr, cg, cb, ca] = *color;
                    let fill = ph2d_vector::Color::from_rgba8(cr, cg, cb, ca); // LITERAL-COLOR-OK: user paint colour
                    fill_rounded_rect(scene, glifo, m::FIELD_RADIUS, fill);
                }
            }
            ToolRailEntry::Compound {
                id, face, active, ..
            } => {
                let (s, sob) = estado(store, *id);
                let on = *active || s == ButtonState::Pressed;
                chip_de_estado(scene, barra, theme, r, on, sob);
                let fg = if sob { TINTA.aceso } else { TINTA.texto };
                // O orçamento é o slot inteiro: o respiro já está na largura medida.
                crate::paint::paint_text_centered_com_orcamento(
                    text_system,
                    scene,
                    face_limpa(face),
                    r,
                    r.w,
                    fonte_fina(),
                    cor(barra, fg),
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "fila_fina_tests.rs"]
mod tests;
