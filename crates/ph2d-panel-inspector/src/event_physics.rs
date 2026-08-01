//! §11 Physics Body — the Inspector event arms (ADR-0131 D8).
//!
//! Its own module rather than another arm in `event_ordering`: physics is not
//! ordering, and that dispatcher is at its LOC cap. The split is the honest
//! one — this file is the whole answer to "what happens when the artist
//! clicks a physics control".

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};

pub(crate) fn apply_physics_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // A metade do CLIQUE: um chip, um botao. Resolvida por uma fn pura
    // (`click_edit`) e nao inline, pelo cap de 200 LOC das fns de painel — o
    // corte e o que as duas metades JA eram, *que controle foi apertado* contra
    // *que numero foi digitado*, e ele deixa o push no barramento escrito UMA
    // vez em vez de duas identicas.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_physics()
        && let Some(edit) = click_edit(id, info.clone())
    {
        host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // ⚠️ **`has_collider`, não `has_body`** (W-PartFace): TODA row numérica desta
    // lista descreve o COLLIDER (forma, dims, offset, densidade) ou uma zona, e um
    // `Collider` sem `RigidBody` é uma PEÇA — mais uma forma do corpo ancestral,
    // que o solver de fato integra. Enquanto a guarda era `has_body`, a face de
    // peça podia PINTAR os campos e nenhum deles chegava ao ECS: o artista
    // digitava e nada acontecia. As rows de zona ficam protegidas por
    // construção — o painter só as pinta num corpo (peça não tem efetor).
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(info) = state::current_inspector_physics()
        && info.has_collider
    {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        let edit = match id {
            ids::INSP_PHYS_RADIUS => Some(PhysicsFieldEdit::Radius(v)),
            ids::INSP_PHYS_HALF_X => Some(PhysicsFieldEdit::HalfX(v)),
            ids::INSP_PHYS_HALF_Y => Some(PhysicsFieldEdit::HalfY(v)),
            ids::INSP_PHYS_CAP_HALF_H => Some(PhysicsFieldEdit::CapHalfHeight(v)),
            // Collider offset — a collider property, honoured for any body (not
            // Dynamic-gated like velocity/gravity).
            ids::INSP_PHYS_OFFSET_X => Some(PhysicsFieldEdit::OffsetX(v)),
            ids::INSP_PHYS_OFFSET_Y => Some(PhysicsFieldEdit::OffsetY(v)),
            ids::INSP_PHYS_DENSITY => Some(PhysicsFieldEdit::Density(v)),
            // The explicit Mass row (Manual mode) — Dynamic-only, the same gate the
            // painter offers it under. The row is only painted in Manual mode, so
            // this cannot fire otherwise, but a refusal in the paint loop is not one.
            ids::INSP_PHYS_MASS if info.kind_tag == 0 => Some(PhysicsFieldEdit::Mass(v)),
            ids::INSP_PHYS_RESTITUTION => Some(PhysicsFieldEdit::Restitution(v)),
            ids::INSP_PHYS_FRICTION => Some(PhysicsFieldEdit::Friction(v)),
            // Honoured only for a Dynamic body — the same gate the painter
            // offers it under (rapier applies gravity to Dynamic bodies only).
            // The row is not painted for other kinds, so this cannot fire for
            // them, but a refusal that lives in the paint loop is not a refusal.
            ids::INSP_PHYS_GRAVITY_SCALE if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::GravityScale(v))
            }
            // Initial velocity (W9), Dynamic-only like gravity. Angular is
            // displayed in deg/s and converted to the component's radians here.
            ids::INSP_PHYS_LINVEL_X if info.kind_tag == 0 => Some(PhysicsFieldEdit::LinvelX(v)),
            ids::INSP_PHYS_LINVEL_Y if info.kind_tag == 0 => Some(PhysicsFieldEdit::LinvelY(v)),
            ids::INSP_PHYS_ANGVEL if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::Angvel(v.to_radians()))
            }
            // Dominance (W-Dominance), Dynamic-only like gravity/velocity. The widget
            // is a float; dominance is an i8 priority, so round and clamp to i8 range
            // at the panel boundary (the shell's edit and the component stay i8).
            ids::INSP_PHYS_DOMINANCE if info.kind_tag == 0 => {
                // `safe_clamp` (NaN-aware) rather than `f32::clamp`: the bounds are
                // `i8::MIN/MAX` cast to f32 — dynamic values, not literal constants,
                // so `arch_safe_clamp_only` requires the safe variant.
                let clamped = ph2d_editor_core::math::safe_clamp(
                    v.round(),
                    f32::from(i8::MIN),
                    f32::from(i8::MAX),
                );
                Some(PhysicsFieldEdit::Dominance(clamped as i8))
            }
            // Per-body damping (W-Damping), Dynamic-only like gravity/velocity.
            ids::INSP_PHYS_LINEAR_DAMPING if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::LinearDamping(v))
            }
            ids::INSP_PHYS_ANGULAR_DAMPING if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::AngularDamping(v))
            }
            // As sete rows da ZONA — o que esta ÁREA faz a OUTROS corpos, extraídas para
            // um irmão pelo cap de 200 LOC desta fn (o mesmo corte que separou
            // `components/area.rs` e `inspector_physics_area.rs`).
            other => area_edit(other, v, info.clone()),
        };
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    false
}

/// **Que CONTROLE foi apertado** — a metade discreta da §11 (chips e botoes).
///
/// Extraida de [`apply_physics_event`] pelo cap de 200 LOC das fns de painel, na
/// linha de corte que as duas metades da funcao ja eram: aqui um chip ESCOLHE
/// entre alternativas, no `ValueChanged` um campo COMMITA um numero.
///
/// ⚠️ **Toda gate mora aqui, nao no laco de pintura**: o painel oferece um
/// controle so quando ele faz sentido, mas o id fica no store a sessao inteira e
/// *dim nao e uma recusa* ([[feedback_disabled_button_still_dispatches]]) — por
/// isso cada braco repete a condicao sob a qual foi pintado.
fn click_edit(
    id: ph2d_editor_core::NodeId,
    info: InspectorPhysicsInfo,
) -> Option<PhysicsFieldEdit> {
    if id == ids::INSP_PHYS_ADD && !info.has_body {
        Some(PhysicsFieldEdit::Add)
    } else if id == ids::INSP_PHYS_REMOVE && (info.has_body || info.has_collider) {
        // ⚠️ **`has_collider` também** (W-PartFace): numa PEÇA este botão se
        // chama *Remove Shape* e faz exatamente o mesmo — tira corpo e collider,
        // e `queue_remove` de um componente ausente é no-op. Sem a segunda
        // metade, uma peça era uma porta de mão ÚNICA: criada por um clique e
        // desfeita só apagando o objeto.
        Some(PhysicsFieldEdit::Remove)
    } else if let Some(i) = ids::INSP_PHYS_LAYER.iter().position(|&o| o == id) {
        // ⚠️ **`has_collider`** (W-PartFace): a camada é propriedade do
        // COLLIDER, e uma PEÇA tem uma. A guarda de `has_body` a tornava um chip
        // pintado e mudo na face de peça — dim não é recusa, e pintado-e-inerte
        // é pior ([[feedback_disabled_button_still_dispatches]]).
        info.has_collider
            .then_some(PhysicsFieldEdit::Layer(i as u8))
    } else if id == ids::INSP_PHYS_BAKE {
        // Gated on `has_body` AND on the body being the kind that actually
        // has simulated motion: a Static body never moves and a Kinematic
        // one is already driven by the scene, so a bake of either can only
        // report "nothing moved". The painter declines to offer the button
        // for those; this is the half that declines to honour it, because
        // the id lives in the store all session and dim is not a refusal.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::Bake)
    } else if id == ids::INSP_PHYS_JOIN_DRAW {
        // ⚠️ **NOT gated on the selection**, and that is the point: the
        // gesture names its two bodies by pointing at them, so requiring a
        // selection first would put it behind the very step it removes.
        Some(PhysicsFieldEdit::JoinDraw)
    } else if id == ids::INSP_PHYS_JOIN && info.join_count >= 2 {
        // Gated on the COUNT the SHELL computed — the painter only offers
        // the button for two or more selected bodies, and a refusal that
        // lives in the paint loop is not a refusal
        // ([[feedback_disabled_button_still_dispatches]]). ⚠️ The painter and
        // this handler read the SAME number; when they were a `bool` beside
        // a count they disagreed the day the chain arrived.
        Some(PhysicsFieldEdit::Join)
    } else if id == ids::INSP_PHYS_ADD_SHAPE && !info.part_owner.is_empty() && !info.has_collider {
        // A recusa vive AQUI e não no laço de pintura: uma forma sem corpo acima
        // não tem a quem pertencer, e um botão apagado que ainda despacha mente.
        //
        // ⚠️ **`!has_collider` é a metade nova** (W-PartFace): o apply escreve um
        // `Collider` DEFAULT, então clicar isto sobre algo que já tem forma
        // apaga a autorada em silêncio — medido, a barra `0,17 × 0,91` com
        // offset `[0,13, −0,07]`, densidade `3,5` e camada `2` virava a caixa do
        // sprite com tudo zerado. A face de peça não pinta mais o botão; esta é
        // a outra metade, porque o id fica no store a sessão inteira.
        Some(PhysicsFieldEdit::AddShape)
    } else if id == ids::INSP_PHYS_RIG && info.rig_parts > 0 {
        // W-Rig. Gateado no MESMO número que o painter usa para oferecer — o
        // zero é a resposta inteira, e uma recusa que mora no laço de pintura
        // não é recusa ([[feedback_disabled_button_still_dispatches]]).
        Some(PhysicsFieldEdit::Rig)
    } else if let Some(i) = ids::INSP_PHYS_JOIN_KIND.iter().position(|&o| o == id) {
        // ⚠️ **NOT gated on the selection any more** (W-J4): the kind
        // qualifies BOTH creation routes, and the canvas gesture needs no
        // selection at all — gating it would make the TYPE unchoosable for
        // exactly the route that removed the selection step.
        Some(PhysicsFieldEdit::JoinKind(i as u8))
    } else if let Some(i) = ids::INSP_PHYS_KIND.iter().position(|&o| o == id) {
        // Gated like every sibling. Kind and Shape were the only two §11
        // controls with NO `has_body` check, which made `Kind` a second
        // door to attaching an orphan `RigidBody` to a plain sprite — the
        // chips are painted only inside the body block, and a refusal that
        // lives in the paint loop is not a refusal
        // ([[feedback_disabled_button_still_dispatches]]).
        info.has_body.then_some(PhysicsFieldEdit::Kind(i as u8))
    } else if let Some(i) = ids::INSP_PHYS_SENSOR.iter().position(|&o| o == id) {
        // Two segments: `0` Solid, `1` Sensor. ⚠️ `has_collider` (W-PartFace):
        // *sólido ou atravessável* é pergunta do COLLIDER, e o solver a honra
        // numa peça (o `is_sensor` chega ao `build_collider` por ela). Dim não é
        // recusa, então a condição é a mesma que o painter usa para oferecer.
        info.has_collider
            .then_some(PhysicsFieldEdit::Sensor(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_CCD.iter().position(|&o| o == id) {
        // Two segments: `0` Discrete, `1` Continuous. Dynamic-only, the same
        // gate the painter offers it under (only a body the solver moves fast
        // can tunnel). Dim is not a refusal, so the check lives here too.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::Ccd(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_LOCKROT.iter().position(|&o| o == id) {
        // Two segments: `0` Free, `1` Locked. Dynamic-only, the same gate the
        // painter offers it under (only a body the solver rotates has a
        // rotation to freeze). Dim is not a refusal.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockRotation(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_LOCKX.iter().position(|&o| o == id) {
        // Freeze Position X — Free | Locked. Dynamic-only, the same gate the
        // painter offers it under (only a body the solver moves has a position
        // to freeze). Dim is not a refusal.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockPositionX(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_LOCKY.iter().position(|&o| o == id) {
        // Freeze Position Y — the vertical sibling, same gate.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockPositionY(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_MASSMODE.iter().position(|&o| o == id) {
        // Mass source: `0` Auto, `1` Manual. Dynamic-only, the same gate the
        // painter offers it under (a Static/Kinematic body has infinite mass).
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::MassMode(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_REST_COMBINE.iter().position(|&o| o == id) {
        // Restitution combine (W-Material): four segments Average/Min/Multiply/Max.
        // Gated on `has_body` like its siblings — but NOT Dynamic-only: it is a
        // collider material property, so a static floor's rule matters too. Dim is
        // not a refusal, so the check lives here.
        // ⚠️ `has_collider` (W-PartFace): material é do collider, peça tem um.
        info.has_collider
            .then_some(PhysicsFieldEdit::RestitutionCombine(i as u8))
    } else if let Some(i) = ids::INSP_PHYS_FRIC_COMBINE.iter().position(|&o| o == id) {
        // Friction combine — the sibling, same collider-only gate.
        info.has_collider
            .then_some(PhysicsFieldEdit::FrictionCombine(i as u8))
    } else if let Some(i) = ids::INSP_PHYS_ONEWAY.iter().position(|&o| o == id) {
        // Off | On (W-OneWay). NOT Dynamic-only — it is a collider property and a
        // platform is usually Static — but it IS solid-only: one-way works by
        // modifying solver CONTACTS, and a sensor generates none, so the painter
        // offers it for a solid collider alone. Dim is not a refusal, so the same
        // condition is asked here (W-Area made this row exclusive with Force).
        // ⚠️ `has_collider` (W-PartFace): a ponte lê `OneWayPlatform` da PEÇA.
        (info.has_collider && !info.is_sensor).then_some(PhysicsFieldEdit::OneWay(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_FORCE_AXES.iter().position(|&o| o == id) {
        // Zone | World (W-AreaFrame) — the mirror image of the One-Way gate above:
        // SENSOR-only, because it qualifies the force rows, and those only exist for
        // a sensor. Dim is not a refusal, so the condition is asked here too.
        (info.has_body && info.is_sensor).then_some(PhysicsFieldEdit::ForceWorldAxes(i == 1))
    } else if let Some(i) = ids::INSP_PHYS_DAMPMODE.iter().position(|&o| o == id) {
        // Damp mode: `0` Combine, `1` Replace (W-Damping). Dynamic-only, the same
        // gate the painter offers it under (damping decays a velocity only a
        // Dynamic body has). Dim is not a refusal, so the check lives here.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::DampMode(i as u8))
    } else if let Some(i) = ids::INSP_PHYS_BAKE_CH.iter().position(|&o| o == id) {
        // The bake channel selector — a GLOBAL option, but painted only for a
        // Dynamic body (the only kind that bakes), so honoured under the same
        // condition the painter offers it.
        (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::BakeChannels(i as u8))
    } else {
        ids::INSP_PHYS_SHAPE
            .iter()
            .position(|&o| o == id)
            // ⚠️ `has_collider` (W-PartFace): a FORMA é a pergunta central de
            // uma peça, e era a primeira que o artista tentaria mudar.
            .filter(|_| info.has_collider)
            .map(|i| PhysicsFieldEdit::Shape(i as u8))
    }
}

/// **O roteamento das rows de ZONA** — a força, o frame dela, o torque, o falloff e os
/// três knobs de meio (W-Area .. W-AreaFalloff).
///
/// ⚠️ **Todas gateadas em `is_sensor`, nunca no `kind_tag`**, e o motivo é o mesmo para as
/// sete: a narrow phase registra sobreposição só quando um dos lados é sensor, então numa
/// zona sólida qualquer destes números seria autorado e nunca lido. O painel oferece as
/// rows sob exatamente essa condição — e uma recusa que mora no laço de pintura não é uma
/// recusa, por isso ela é repetida aqui.
///
/// Extraída de [`apply_physics_event`] pelo cap de 200 LOC das fns de painel, na linha de
/// corte que a família das zonas já vinha desenhando sozinha nos dois outros arquivos que
/// ela obrigou a separar.
fn area_edit(
    id: ph2d_editor_core::NodeId,
    v: f32,
    info: InspectorPhysicsInfo,
) -> Option<PhysicsFieldEdit> {
    if !info.is_sensor {
        return None;
    }
    match id {
        ids::INSP_PHYS_FORCE_X => Some(PhysicsFieldEdit::ForceX(v)),
        ids::INSP_PHYS_FORCE_Y => Some(PhysicsFieldEdit::ForceY(v)),
        ids::INSP_PHYS_AREA_TORQUE => Some(PhysicsFieldEdit::AreaTorque(v)),
        // Cru: a fração é clampada UMA vez, no apply da shell, onde ela vira componente.
        ids::INSP_PHYS_AREA_FALLOFF => Some(PhysicsFieldEdit::AreaFalloff(v)),
        ids::INSP_PHYS_AREA_DRAG => Some(PhysicsFieldEdit::AreaDrag(v)),
        ids::INSP_PHYS_AREA_DENSITY => Some(PhysicsFieldEdit::AreaDensity(v)),
        ids::INSP_PHYS_AREA_FORM_DRAG => Some(PhysicsFieldEdit::AreaFormDrag(v)),
        _ => None,
    }
}
