//! Inspector panel `apply_event` — ADR-0029 Phase C.1 port.
//!
//! Migrated from `ph2d_editor_core::screens::hero::inspector::{mod,
//! apply_event_full}` to the panel crate. The signature changed from
//! `(hero: &mut HeroScreen, ev: WidgetEvent)` to
//! `(state: &mut InspectorState, host: &mut dyn PanelHostInternal,
//! ev: WidgetEvent)`. All `hero.<field>` accesses route through
//! [`PanelHostInternal`] trait methods.
//! ✅ **O braço do ponto de cor deixou de ENUMERAR os seus leitores** (2026-08-21) — a cura que
//! esta nota nomeava está feita: **uma** tabela [`ids::LIVE_SECTIONS`] de pares `(seção, cor)`, e
//! `pre_populate` (dobra + registo) e este braço (despacho) são projeções dela.
//!
//! ⚠️ **A nota anterior estava certa no mecanismo e errada no número** — dizia que ORDERING /
//! SAMPLING / BLEND estavam «em nenhum dos dois sítios»; a auditoria de 7 lentes mediu **sete**
//! pontos mortos (mais Pulley Wheel e Platform Player) e **três** cabeçalhos que pintavam o chevron
//! e não dobravam. *Uma nota de dívida também envelhece — foi por isso que a cura virou tabela e
//! não uma sexta entrada na lista.*
//!
//! A varredura que impede a recaída é `tests/every_painted_id_is_reachable.rs`: ela pinta o
//! Inspector inteiro e exige que **todo id registado no índice de acerto** passe na pergunta do
//! `is_focusable`. Ela não tem lista nenhuma dentro — a lista é o que o painel pinta
//! ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
//!

use crate::ids;
use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorTransformInfo, InspectorVisibilityInfo, SpriteFieldEdit,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

pub(crate) fn apply_event(
    state: &mut state::InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // **§12 Sockets / Anchors** — a ÚNICA família que precisa do estado do painel (clicar numa
    // linha muda a ficha aberta, e isso não é uma edição da cena). Por isso corre aqui, antes
    // do `apply_event_impl`, que só vê o `host`.
    if crate::event_anchor::apply_anchor_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    // **§11 Animation** — irmã da acima, e pela mesma razão: clicar numa linha mexe no estado do
    // painel (qual ficha está aberta). ⚠️ Aqui o clique **também** vai ao barramento, porque a
    // linha aberta é a animação que toca — ver `sections::anim`.
    if crate::event_anim::apply_anim_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    // **TIMERS** — a terceira que precisa do estado do painel. ⚠️ Aqui o clique numa linha
    // **não** vai ao barramento (ver `event_timer`): um `Timers` não tem «o timer actual».
    if crate::event_timer::apply_timer_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    // **SIGNAL ACTIONS** — a quarta com estado de painel, e pela mesma razão das outras três.
    if crate::event_action::apply_action_event(state, host, ev) {
        return EventOutcome::Consumed;
    }

    if crate::event_topdown::apply_topdown_event(host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_projectile::apply_projectile_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ A ARMA — sem estado de painel: um objecto tem UMA.
    if crate::event_weapon::apply_weapon_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ A PARALAXE (plano 24) — sem estado de painel: um objecto tem UMA camada.
    if crate::event_parallax::apply_parallax_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ O RAIO (suplente #21) — sem estado de painel: um objecto tem UM.
    if crate::event_ray::apply_ray_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ O TWEEN (suplente #22) — ele PRECISA do estado do painel (a lista tem uma linha
    // aberta), como o gatilho, a vigia e os timers.
    if crate::event_tween::apply_tween_event(host, ev, &mut state.tween_selected) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ O SEGUIDOR DE CAMINHO (suplente #23) — sem estado de painel: o componente é ÚNICO por
    // entidade, logo não há linha aberta a lembrar.
    if crate::event_path_follow::apply_path_follow_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ O HUD (TOP-20 #20) — sem estado de painel: um objecto tem UM de cada.
    if crate::event_hud::apply_hud_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ A CUTSCENE (TOP-20 #19) — sem estado de painel: um objecto toca UMA sequência.
    if crate::event_sequence::apply_sequence_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ O GATILHO (suplente #24) — ele precisa do estado do painel (a lista tem uma linha
    // aberta), como a vigia, os timers e a tabela de acções.
    // ⭐⭐⭐ O ABANÃO (suplente #25) — a da CÂMERA não precisa de estado de painel (não tem lista);
    // a do EMISSOR precisa, como o gatilho.
    if crate::event_shake::apply_shake_event(host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_shake::apply_emitter_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_action_trigger::apply_action_trigger_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ A VIGIA DO CONTADOR — ela precisa do estado do painel (a lista tem uma regra aberta),
    // como os timers e a tabela de acções.
    if crate::event_counter_watch::apply_counter_watch_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐ O EMISSOR DE PARTÍCULAS (TOP-20 #18) — sem estado de painel: um objecto tem UM emissor.
    if crate::event_particles::apply_particles_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ **O CÉREBRO** (TOP-20 #15) — ele precisa do estado do painel (as duas listas têm uma
    // linha aberta cada), como a tabela de acções e os timers.
    if crate::event_statemachine::apply_statemachine_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_factory::apply_factory_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐ O SCRIPT (TOP-20 #16) — sem estado de painel: as linhas são todas visíveis.
    if crate::event_script::apply_script_event(host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_camera::apply_camera_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ **TAGS** (TOP-20 #9) — ela NÃO precisa do estado do painel: não há «a tag aberta», e o
    // que a caixa de escolha guarda (o `open`) vive no store como o de todas as outras.
    if crate::event_tags::apply_tags_event(host, ev) {
        return EventOutcome::Consumed;
    }
    if crate::event_audio::apply_audio_event(host, ev) {
        return EventOutcome::Consumed;
    }
    // ⭐⭐⭐ **O VALOR de uma propriedade** — a terceira família que precisa do estado do painel:
    // carregar no chip aceso abre um campo, e *qual* eixo está aberto é estado de painel, não uma
    // edição da cena. Ver `crate::event_value`.
    if crate::event_value::apply_value_event(state, host, ev) {
        return EventOutcome::Consumed;
    }
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

/// ⭐ **O `+` do cabeçalho** (ADR-0166 / F3): um PEDIDO, não uma edição. O painel não sabe que
/// componentes existem nem o que este objeto já tem — quem sabe é a shell, e ela é que abre a
/// paleta.
///
/// ⚠️ **Sem entidade selecionada o clique é RECUSADO**, e não aceite em silêncio (DIRETIVA §2). O
/// `entity_bits` sai do **`Transform`**, que é a base de todo objeto (ADR-0166: a seção-base é
/// `Transform` + `Name`) — sem ele não há objeto nenhum sob o Inspector.
///
/// ⚠️ **Função irmã, e não um braço da mãe:** as 14 linhas dela levaram o `apply_event_impl` de 292
/// para 306 contra um teto de 200 cuja tolerância **só desce** — o precedente é o `visibility_toggle`
/// (função irmã no mesmo ficheiro, que está com folga sob o teto de 600 do ARQUIVO).
/// ⭐⭐ **A RANHURA DA TEXTURA abre a biblioteca** — *«o que é que eu posso pôr aqui?»*.
///
/// ⚠️ **Ela recebe QUEDAS e responde a CLIQUES**, e as duas metades são obrigatórias: um id
/// hit-indexado que não despacha é um controlo morto, e dois censos deste repo o dizem. O clique é
/// o que torna a queda **descoberta** — o artista abre, vê as imagens, e arrasta uma.
fn texture_slot_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if ev != WidgetEvent::Click(ids::INSP_RENDER_TEXTURE_SLOT) {
        return false;
    }
    host.bus_mut().push(EditorAction::OpenAssetBrowser);
    true
}

fn add_component_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if ev != WidgetEvent::Click(ids::INSP_ADD_COMPONENT) {
        return false;
    }
    let Some(bits) = crate::state::current_inspector_transform().map(|t| t.entity_bits) else {
        return false;
    };
    host.bus_mut()
        .push(EditorAction::InspectorAddComponentRequested { entity_bits: bits });
    true
}

/// **A GRELHA da folha** — as três caixas `Columns` / `Rows` / `Frame` da §4.
///
/// ⚠️ **Uma lei só, três ids:** os três números descrevem o mesmo pool de células (desde o corte da
/// F1 eles vivem no `SpriteGrid`, não na `Sprite`), e um `n` negativo ou não-finito vira `0` na
/// porta — a caixa de texto aceita digitar o que quiser.
///
/// ⚠️ **Saiu do [`apply_event_impl`] em 2026-08-25 pela catraca**, quando o `+` do cabeçalho
/// (ADR-0166 / F3) o levou a 295 contra uma tolerância de 292 que **só desce**. Levar só o braço
/// novo devolveria o número a 292 exactos, e *ficar no mesmo sítio não é encolher* — a mesma lição
/// que o par de PRECISÃO, o par de sliders e o cluster da REGIÃO já pagaram nesta família.
fn sheet_grid_changed(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(
            id,
            ids::INSP_SPRITE_HFRAMES | ids::INSP_SPRITE_VFRAMES | ids::INSP_SPRITE_FRAME
        )
        && let Some(info) = state::current_inspector_sprite()
    {
        let raw = host.store().number_value(id).unwrap_or(0.0);
        let n = raw.round().max(0.0) as u32;
        let edit = if id == ids::INSP_SPRITE_HFRAMES {
            SpriteFieldEdit::Hframes(n)
        } else if id == ids::INSP_SPRITE_VFRAMES {
            SpriteFieldEdit::Vframes(n)
        } else {
            SpriteFieldEdit::Frame(n)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    false
}

/// ⭐ **Os cliques de UM id, em TABELA.** Cada um responde *«era eu?»* e devolve `true` se agiu.
///
/// ⚠️ **Uma tabela, e não uma escada de `if`**: a escada era três linhas por entrada e, com o
/// `clear_orphans_click` da F5, empurrou o `apply_event_impl` acima do tecto. A catraca do
/// `architecture_panel_loc_cap` **só desce**, e o que ela pede é exactamente isto — *quando N
/// blocos têm a mesma forma, a forma é que é o dado.* O próximo entra numa linha.
const SINGLE_ID_CLICKS: &[fn(&mut dyn PanelHostInternal, WidgetEvent) -> bool] = &[
    add_component_click,
    // ⭐⭐ Os SEIS do CARTÃO DE INSTÂNCIA vivem no irmão [`crate::event_instance`] — corte por
    // assunto, imposto pelo tecto de 600 LOC deste ficheiro quando o *Put back* entrou.
    crate::event_instance::open_prefab_click,
    crate::event_instance::clear_orphans_click,
    crate::event_instance::drop_orphan_click,
    crate::event_instance::restore_piece_click,
    crate::event_instance::apply_added_click,
    crate::event_instance::apply_level_click,
    section_color_click,
    texture_slot_click,
];

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if SINGLE_ID_CLICKS.iter().any(|f| f(host, ev)) {
        return true;
    }

    if crate::event_color_tint::color_tint_click(host, ev) {
        return true;
    }

    // Close (X) — hide the Inspector. Same effect as toggling the
    // left-rail Inspector pill (vide `chrome/rail_panels.rs`). UI canon
    // post-2026-05-24: every floating panel except Hierarchy has X.
    //
    // Sync the left-rail RAIL_SHOW_INSPECTOR button state so its
    // Pressed/Normal visual tracks the panel's actual visibility —
    // without this, hiding via X leaves the rail toggle stuck
    // Pressed (bug reported 2026-05-24).
    if let WidgetEvent::Click(id) = ev
        && id == core_ids::INSP_CLOSE
    {
        let next = !host.panel_visible("inspector");
        host.set_panel_visible("inspector", next);
        if let Some(InteractiveState::Button { state }) =
            host.store_mut().get_mut(core_ids::RAIL_SHOW_INSPECTOR)
        {
            *state = if next {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
        return true;
    }
    // M14.5 inspector phase (6.4) — Reimport button.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_RENDER_SOURCE_REIMPORT
        && let Some(info) = state::current_inspector_sprite()
        && info.can_reimport
    {
        host.bus_mut().push(EditorAction::Reimport {
            entity_bits: info.entity_bits,
        });
        return true;
    }
    // M14.A — Transform editor commits.
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(
            id,
            ids::INSP_TRANSFORM_POS_X
                | ids::INSP_TRANSFORM_POS_Y
                | ids::INSP_TRANSFORM_ROT
                | ids::INSP_TRANSFORM_SCALE_X
                | ids::INSP_TRANSFORM_SCALE_Y
                | ids::INSP_TRANSFORM_SKEW_X
                | ids::INSP_TRANSFORM_SKEW_Y,
        )
        && let Some(info) = state::current_inspector_transform()
    {
        crate::event_transform::commit_transform_edit(host, info);
        return true;
    }
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_TRANSFORM_RESET
        && let Some(info) = state::current_inspector_transform()
    {
        host.bus_mut().push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [0.0, 0.0],
                rotation_rad: 0.0,
                scale: [1.0, 1.0],
                skew_rad: [0.0, 0.0],
            },
        ));
        return true;
    }
    if visibility_toggle(host, ev) {
        return true;
    }
    // W2 Sprite Inspector v2 — logical Flip H / Flip V toggled.
    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_SPRITE_FLIP_X | ids::INSP_SPRITE_FLIP_Y)
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        let edit = if id == ids::INSP_SPRITE_FLIP_X {
            SpriteFieldEdit::FlipX(checked)
        } else {
            SpriteFieldEdit::FlipY(checked)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // W2 Color & Tint — Tint Fill (silhouette) toggled.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SPRITE_TINT_FILL
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::TintFill(checked),
        });
        return true;
    }
    // **Os dois sliders-com-chip da sprite** — Opacidade e Emissive, no irmão
    // [`crate::event_sprite_value`]. Saíram juntos em 2026-08-21 quando a linha `Emissive` (plano
    // `docs/Sprite_projeto/18` W8) empurrou este despachante para 433 contra uma tolerância de 410:
    // levar só o novo devolveria o número a 410 e não desceria nada, e *a catraca só desce*.
    if crate::event_sprite_value::apply_sprite_slider_event(host, ev) {
        return true;
    }
    // W3 §7 Ordering — all ordering widget events (sibling module, LOC).
    if crate::event_ordering::apply_ordering_event(host, ev) {
        return true;
    }
    // **§5 9-Slice** — irmão, pelo mesmo cap de função que pôs os sliders no
    // `event_sprite_value`. Ver [`crate::event_slice`].
    if crate::event_slice::apply_slice_event(host, ev) {
        return true;
    }
    // W2 Sprite Sheet — HFrames / VFrames / Frame committed. Integer
    // fields; rounded from the NumberInput's f64. Clamps (>=1, in-grid)
    // land at the commit boundary (apply_sprite_field).
    if sheet_grid_changed(host, ev) {
        return true;
    }
    // **A REGIÃO e a ORIGEM** — sub-rect, Centered e Offset — moram no irmão
    // [`crate::event_sprite_geometry`]. Saíram em 2026-08-21 quando a §5 9-Slice empurrou este
    // despachante para 389 contra uma catraca de 384: *a catraca só desce, e um cluster de cada
    // vez*. As três leis do cluster são a mesma — despacho POR EIXO, para que um fan-out de
    // seleção múltipla não atropele o eixo divergente do vizinho.
    if crate::event_sprite_geometry::apply_sprite_geometry_event(host, ev) {
        return true;
    }
    // **Os dois pares da seção Render Source** — estratégia e precisão — moram num irmão.
    // Ver [`crate::event_precision`], e a nota de LOC lá dentro.
    if crate::event_precision::render_source_click(host, ev) {
        return true;
    }
    if section_text_changed(host, ev) {
        return true;
    }
    // ADR-0029 Phase C.1: showcase-shared events
    // (`CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`, `SECTION_IDS`,
    // radio/tab/tree pinning) are now routed at host level via
    // `widget::showcase::apply_showcase_event`. The Inspector panel
    // returns `Ignored` for those — host picks them up after the
    // registry walk.
    false
}

/// **Os campos de TEXTO do Inspector.** Os dois nomeiam alguma coisa — como o
/// objeto se chama, e o que ele GRITA quando algo chega nele — e os dois
/// viajam pelo mesmo barramento por-entidade, com o `InspectorNameInfo` no
/// lugar de um campo de `PhysicsFieldEdit`: o valor é uma STRING, e todo o
/// resto da §11 fala em número ou em chip. Um braço de string no enum dos
/// campos numéricos seria um segundo formato de edição vivendo dentro do
/// primeiro.
///
/// ⚠️ **Função própria pelo MESMO motivo que a `section_color_click` abaixo**, e
/// pela mesma catraca: o `apply_event_impl` vive sob um teto que só pode
/// ENCOLHER, e a row de sinal da W-Signal o empurrou de 452 para 470. Movê-la
/// para cá é a correção certa — subir o número do allowlist seria usar como
/// licença de crescimento uma entrada cuja prosa diz o contrário.
///
/// ⚠️ E ela ficou latente por uma causa que esta linha já pagou antes: este gate
/// mora em `ph2d-editor-core/tests/`, então um fechamento por `cargo test -p`
/// nas crates da física **não o alcança**.
fn section_text_changed(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::TextChanged(id) = ev else {
        return false;
    };
    // M14.E — o nome da entidade.
    if id == core_ids::INSP_ENTITY_NAME
        && let Some(info) = state::current_inspector_name()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorNameEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    // W-Signal · W-SignalLeave — os nomes que este objeto grita quando algo
    // CHEGA nele e quando algo SAI. Duas rows, dois contratos, duas ações: um
    // `leave` enfiado na mesma ação com um bool tornaria impossível ler o
    // barramento sem perguntar duas coisas para saber uma.
    if id == ids::INSP_PHYS_SIGNAL
        && let Some(info) = state::current_inspector_physics()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorSignalEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    if id == ids::INSP_PHYS_SIGNAL_LEAVE
        && let Some(info) = state::current_inspector_physics()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorSignalLeaveEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    false
}

/// A click on a section's colour dot — seed the canonical `BlenderPicker` at
/// that section's colour id, the same flow the Widget Gallery uses for its
/// `SECTION_COLOR_IDS`. The picker writes the chosen rgba back via
/// `set_widget_color(<color_id>, rgba)` (drained in `hero.rs`), and the next
/// `paint_section_header` paints the dot in it. UI canon 2026-05-24: every
/// section can carry a per-user accent colour.
///
/// Its own function because `apply_event_impl` is under a ratcheting LOC cap
/// and the two physics dots (§11/§12) pushed it over. Returns whether the
/// event was consumed.
/// **A caixa «Visible» do topo** (M14.D) — extraída da mãe em 2026-08-21 pela catraca de LOC.
///
/// ⚠️ O que a fez crescer foi o **fan-out**: esta caixa editava só a primária enquanto a §8
/// Visibility logo abaixo editava toda a seleção (auditoria `docs/Sprite_projeto/20` §3). Ganhar o
/// espalhamento exigiu ganhar antes a afordância de divergência — *espalhar sem sinal troca um
/// sub-aplicar silencioso por um esmagamento silencioso*.
fn visibility_toggle(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_VISIBILITY_CHECK
        && let Some(info) = state::current_inspector_visibility()
    {
        let visible = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked),
        );
        host.bus_mut().push(EditorAction::InspectorVisibilityEdit(
            InspectorVisibilityInfo {
                entity_bits: info.entity_bits,
                visible,
                // ⚠️ **A ação carrega `false` porque ela é uma DECISÃO, não um estado.** O `mixed`
                // do snapshot descreve o que a seleção era *antes*; o que sobe aqui é o que o
                // artista acabou de escolher para todos. Ecoar a divergência de volta faria o
                // dreno ter de a interpretar — e ele já não a lê.
                mixed: false,
            },
        ));
        return true;
    }
    false
}

fn section_color_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // ⚠️ **A condição deixou de ENUMERAR os seus leitores** (2026-08-21). Ela listava seis dos
    // treze pontos, e a nota no topo deste ficheiro — que já denunciava a podridão — dizia
    // **três**: *uma nota de dívida também envelhece*. Agora a fonte é `ids::LIVE_SECTIONS`, a
    // mesma tabela que o `pre_populate` lê para registar o ponto e a dobra. Um ponto novo arma no
    // dia em que a seção entra na tabela.
    if let WidgetEvent::Click(id) = ev
        && core_ids::LIVE_SECTION_COLOR_IDS.contains(&id)
    {
        let seed = host
            .store()
            .widget_color(id)
            .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral seed
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            core_ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }
    false
}
