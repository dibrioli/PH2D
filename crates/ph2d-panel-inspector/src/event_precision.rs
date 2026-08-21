//! **OS DOIS PARES DA SEÇÃO RENDER SOURCE** — `Strategy` (Atlas / Individual / Hand-packed) e
//! `Format` (RGBA8 / RGBA16).
//!
//! Plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! W5.
//!
//! # Por que num IRMÃO e não no `event.rs`
//!
//! ⚠️ Dois tetos de LOC, e eles medem grandezas diferentes. O `apply_event_impl` está numa
//! **catraca** declarada no gate — cada vez que ele cresce, um cluster de braços sai para uma função
//! irmã e a tolerância desce (477 → 452 → 442). Mas extrair *dentro do mesmo ficheiro* curaria o
//! teto de FUNÇÃO e estouraria o de FICHEIRO, que este `event.rs` já roçava a 600.
//!
//! *O corte que cura os dois é para o irmão* — e é o padrão que os `event_joint` / `event_ordering`
//! / `event_physics` / `event_player` / `event_wheel` já seguem.
//!
//! # A história dos dois botões, porque ela é o motivo do gate
//!
//! Eles **já existiram e foram apagados**: pintados, registados no `WidgetStore` e hit-indexados —
//! e o clique **caía no chão**, sem arm de dispatch em lado nenhum. Nem um toast. É esta função que
//! fecha esse buraco, e o gate `seam_precision` afirma as três metades juntas porque cada uma falha
//! em silêncio de maneira diferente.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorSpriteSource, RequestedSpriteStrategy};
use ph2d_editor_core::widget::ButtonState;

use crate::state;

/// Despacha um clique no par de precisão. `true` quando o evento era deste par.
pub(crate) fn precision_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(requested) = (match id {
        ids::INSP_RENDER_FORMAT_RGBA8 => Some(ph2d_editor_core::Precision::Rgba8),
        ids::INSP_RENDER_FORMAT_RGBA16 => Some(ph2d_editor_core::Precision::Rgba16),
        _ => None,
    }) else {
        return false;
    };
    let Some(info) = state::current_inspector_sprite() else {
        return false;
    };
    // ⚠️ Uma textura cozida não tem precisão autorável (é BC/ASTC/ETC2 e depende do tier), e pedir
    // a que já está **não é uma edição** — senão um clique distraído no botão aceso vira um passo
    // de undo sobre uma cena que não mudou, e o artista aprende que o Ctrl+Z do app mente.
    let cooked = matches!(info.source_kind, InspectorSpriteSource::CookedTexture);
    if !cooked && info.source_precision != Some(requested) {
        host.bus_mut()
            .push(EditorAction::InspectorSpritePrecisionChange {
                entity_bits: info.entity_bits,
                precision: requested,
            });
    }
    // O botão volta ao repouso mesmo quando não houve edição: ele foi mesmo carregado.
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
    true
}

/// **A porta dos dois pares.** Tenta a estratégia; se o evento não era dela, tenta a precisão.
///
/// ⚠️ A ordem não é arbitrária: os dois pares partilham a seção mas **não** os ids, então a
/// primeira que reconhecer o id ganha e a outra nunca vê o evento. Trocar a ordem não muda nada
/// hoje — e é exatamente por isso que está escrito, para que uma futura sobreposição de ids não
/// seja resolvida por acidente.
pub(crate) fn render_source_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    strategy_click(host, ev) || precision_click(host, ev)
}

/// **O par `Strategy`** — M14.C. Mudou de casa em 2026-08-20 pela catraca de LOC do
/// `apply_event_impl` (ver o cabeçalho); o corpo é verbatim.
fn strategy_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = ev
        && let Some(requested) = match id {
            ids::INSP_RENDER_STRATEGY_ATLAS => Some(RequestedSpriteStrategy::Atlas),
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL => Some(RequestedSpriteStrategy::Individual),
            ids::INSP_RENDER_STRATEGY_HANDPACKED => Some(RequestedSpriteStrategy::HandPacked),
            _ => None,
        }
        && let Some(info) = state::current_inspector_sprite()
    {
        let current = match info.source_kind {
            InspectorSpriteSource::Atlas { .. } => RequestedSpriteStrategy::Atlas,
            InspectorSpriteSource::Individual { .. } => RequestedSpriteStrategy::Individual,
            InspectorSpriteSource::HandPacked { .. } => RequestedSpriteStrategy::HandPacked,
            // W2.T2: no `CookedTexture` strategy exists (cooked sources
            // aren't authored from the inspector). Map to the read-only
            // `HandPacked` strategy purely for change-detection.
            InspectorSpriteSource::CookedTexture => RequestedSpriteStrategy::HandPacked,
        };
        // For a cooked source, route EVERY radio click (audit L2): mapping
        // `current` to `HandPacked` above means a Hand-packed click would
        // otherwise short-circuit `requested != current` and give no toast,
        // while Atlas/Individual clicks do — inconsistent feedback. Force the
        // route so `inspector_commits` rejects any strategy click uniformly.
        let cooked = matches!(info.source_kind, InspectorSpriteSource::CookedTexture);
        if requested != current || cooked {
            host.bus_mut()
                .push(EditorAction::InspectorSpriteSourceChange {
                    entity_bits: info.entity_bits,
                    strategy: requested,
                });
        }
        if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
            *state = ButtonState::Normal;
        }
        return true;
    }
    false
}
