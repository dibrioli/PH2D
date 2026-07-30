//! **A metade RODA da §12** (W-Pulley W1) — o que a seção da corda faz com as
//! roldanas dela.
//!
//! Irmão de [`super::inspector_joint`], e o corte é o assunto: lá mora *o que um
//! JOINT é* (os campos, o `clamped`, o gesto de criar); aqui, *o que a CORDA faz
//! com a lista de rodas dela*. Nasceu do cap de 600 LOC quando a roldana virou
//! entidade, e a próxima wave da polia (motor por roda, ruptura no centro) chega
//! aqui.

use super::inspector_ordering::queue_set;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_editor::{InspectorWheelInfo, WheelFieldEdit};
use ph2d_physics_ecs::PhysicsBridge;

/// O nome de tipo do componente, como o registry o conhece.
const WHEEL: &str = "ph2d::physics::PulleyWheel";

/// Quantas roldanas apontam para esta corda, no mundo AUTORADO.
pub(crate) fn rope_wheel_count(sim: &mut SimWorld, joint: Entity) -> u32 {
    let Some(rope) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| stable_name_id(n.as_str()))
    else {
        return 0;
    };
    let mut q = sim.world_mut().query::<&ph2d_physics_ecs::PulleyWheel>();
    u32::try_from(q.iter(sim.world()).filter(|w| w.rope == rope).count()).unwrap_or(u32::MAX)
}

/// **Acrescentar uma roldana a uma corda** — o pedido (4) do artista, *"escolher
/// o número de roldanas, em tempo real"*.
///
/// A roda nova entra ao FIM da rota, e **sobre a corda**: no meio do último
/// trecho que a rota desenha hoje. Duas razões, e as duas são sobre não
/// surpreender — ali o comprimento quase não muda (a corda já passava por aquele
/// ponto, e o que ela ganha é a diferença entre o arco e a corda do enlace, que é
/// pequena), e o artista vê a roda aparecer EM CIMA da corda que ele está
/// olhando, em vez de num canto que ele teria de ir procurar.
///
/// ⚠️ **Sem roldana nenhuma o "último trecho" é a corda inteira**, então a
/// primeira nasce no meio dela — que é onde uma roldana faria sentido.
///
/// O raio herda o da última roldana (ou o default), porque uma corda com rodas de
/// tamanhos aleatórios não é o que ninguém pede; e o `order` é o seguinte, então
/// a rota simplesmente cresce por onde ela já ia.
pub(crate) fn add_pulley_wheel(sim: &mut SimWorld, physics: &PhysicsBridge, joint_bits: u64) {
    let joint = Entity::from_bits(joint_bits);
    let Some(name) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| n.as_str().to_string())
    else {
        return;
    };
    let rope = stable_name_id(&name);
    let Some(v) = physics.joint_views().find(|v| v.entity == joint) else {
        return;
    };
    // A geometria VIVA, pela mesma porta que o desenho usa — uma segunda
    // derivação poria a roda onde a corda não está.
    let wheels: Vec<_> = physics.rope_wheels(joint).map(|(_, w)| w).collect();
    let last = wheels.last().map_or(v.anchor_a, |w| w.centre);
    let centre = [
        0.5 * (last[0] + v.anchor_b[0]),
        0.5 * (last[1] + v.anchor_b[1]),
    ];
    let radius = wheels
        .last()
        .map_or(ph2d_physics_ecs::PulleyWheel::DEFAULT_RADIUS, |w| w.radius);
    let order = u16::try_from(wheels.len()).unwrap_or(u16::MAX);
    let label = crate::name_unique::unique_name(sim, &format!("{name} Wheel {}", order + 1));
    sim.world_mut().spawn((
        Name::new(label),
        ph2d_physics_ecs::PulleyWheel {
            rope,
            order,
            radius,
            // Uma roldana nasce COMUM: o segundo diâmetro (o tambor diferencial
            // do W4) é um SEGUNDO gesto, pela mesma razão que a montagem é.
            radius_out: 0.0,
            wrap: ph2d_physics_ecs::WrapSide::Auto,
            motor_speed: 0.0,
            // Uma roldana nasce no CENÁRIO: montá-la num corpo (a cadernal
            // móvel do W3) é um SEGUNDO gesto, e não um default. Nomeado e não
            // `..default()` porque estes são sítios de PRODUTO — o campo da
            // próxima wave nasceria neutro aqui em silêncio.
            body: 0,
            local: [0.0, 0.0],
            mounted: false,
            break_enabled: false,
            break_force: ph2d_physics_ecs::PulleyWheel::DEFAULT_BREAK_FORCE,
        },
        Transform::from_translation(ph2d_core::Vec2::new(centre[0], centre[1])),
    ));
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
}

/// **O snapshot da §13.** `None` para qualquer coisa que não seja uma roldana —
/// como a §12 e pelo mesmo motivo, esta seção não tem face vazia.
pub(crate) fn build_wheel_info(
    sim: &mut SimWorld,
    entity_bits: u64,
    mount_pick_armed: bool,
    rope_pick_armed: bool,
) -> Option<InspectorWheelInfo> {
    let entity = Entity::from_bits(entity_bits);
    let wheel = *sim.world().get::<ph2d_physics_ecs::PulleyWheel>(entity)?;
    // A presença do marcador É o booleano (W-Weston) — lida aqui, antes de as queries
    // abaixo tomarem o mundo emprestado.
    let weston = sim
        .world()
        .get::<ph2d_physics_ecs::WestonAxle>(entity)
        .is_some();
    // W3: o CORPO em que o eixo está montado, resolvido do hash para o nome.
    // ⚠️ Exige um `RigidBody`, não só um nome que bate — uma roldana montada num
    // sprite sem corpo não está montada em coisa alguma, e mostrar o nome dele
    // seria a mesma mentira que o `bound` da corda existe para não contar.
    let mut mq = sim
        .world_mut()
        .query::<(&Name, &ph2d_physics_ecs::RigidBody)>();
    let mount_name = if wheel.body == 0 {
        String::new()
    } else {
        mq.iter(sim.world())
            .find(|(n, _)| stable_name_id(n.as_str()) == wheel.body)
            .map(|(n, _)| n.as_str().to_string())
            .unwrap_or_default()
    };
    // A corda a que ela pertence, resolvida do HASH para o nome. ⚠️ Exige um
    // `PhysicsJoint`, não só um nome que bate: uma roldana só entra numa rota se
    // o nome for o de uma CORDA, e um sprite homônimo não a põe em lugar nenhum.
    let mut q = sim
        .world_mut()
        .query::<(&Name, &ph2d_physics_ecs::PhysicsJoint)>();
    let world = sim.world();
    let rope_name = q
        .iter(world)
        .find(|(n, _)| stable_name_id(n.as_str()) == wheel.rope)
        .map(|(n, _)| n.as_str().to_string())
        .unwrap_or_default();
    // **A talha de WESTON e o que ela COMPROU** (W-Weston).
    //
    // ⚠️ **O quociente sai da porta do MOTOR**, nunca de uma conta escrita aqui ou no
    // painel: um readout com aritmética própria mostraria um número e o solver usaria
    // outro — que é exatamente o defeito do `ratio` DIGITADO que o W4 aposentou. E as
    // duas leis vivem lado a lado ali (`R/r` no tambor, `R/(R−r)` na Weston), então
    // esta fronteira só escolhe QUAL perguntar.
    let gear = if wheel.radius_out <= 0.0 {
        1.0
    } else if weston {
        ph2d_physics_ecs::rope_route::weston_gear(wheel.radius, wheel.radius_out)
    } else {
        wheel.radius / wheel.radius_out
    };
    Some(InspectorWheelInfo {
        entity_bits,
        bound: !rope_name.is_empty(),
        rope_name,
        radius: wheel.radius,
        radius_out: wheel.radius_out,
        weston,
        gear,
        // O componente conta de zero e a pessoa conta de um. A conversão mora
        // aqui e no `wheel_with_edit`, uma vez de cada lado.
        order_ui: u32::from(wheel.order) + 1,
        wrap_tag: wheel.wrap.tag(),
        // Radianos no componente, GRAUS na row — a fronteira do motor do Pin,
        // e a conversão mora aqui e no `wheel_with_edit`, uma vez de cada lado.
        motor_deg_per_s: wheel.motor_speed.to_degrees(),
        break_enabled: wheel.break_enabled,
        break_force: wheel.break_force,
        mount_name,
        mount_pick_armed,
        rope_pick_armed,
    })
}

/// Aplica um [`WheelFieldEdit`], pelo mesmo funil do irmão `apply_joint_edit`:
/// lê a roldana viva e a escreve de volta mudada, porque uma escrita parcial
/// derrubaria os campos que não estão sendo editados.
/// Devolve **`true` se a ROTA da corda mudou** — o chamador usa isso para re-abrir
/// o `L0`, que é derivado da rota e ficava congelado (ver
/// `ph2d_physics_ecs::reseat_wheel_geometry`). ⚠️ **Não é *"o componente mudou"***:
/// autorar o motor durante o PLAY não pode re-derivar comprimento de corda, e a
/// cena `=59` autora o motor tocando de propósito.
pub(crate) fn apply_wheel_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: WheelFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<ph2d_physics_ecs::PulleyWheel>(entity) else {
        return false;
    };
    // **A talha de WESTON é um MARCADOR, não um campo** (W-Weston): a presença do
    // componente É o booleano, então ela sai do funil de campo e vai pela porta de
    // anexar/desanexar — o idioma do `Ccd`/`LockRotation`, e o que a mantém fora de um
    // bump de `PROJECT_SCHEMA`.
    //
    // ⚠️ **Devolve `true` sempre**, e é o único toggle desta seção que faz isso: armar
    // a Weston ACRESCENTA um nó à rota e re-pesa a corda, então o `L0` derivado tem de
    // ser re-aberto — e o `route_differs`, que compara campos da `PulleyWheel`, é cego
    // a um componente ao lado dela.
    if let WheelFieldEdit::Weston(on) = edit {
        const WESTON: &str = "ph2d::physics::WestonAxle";
        if on {
            queue_set(
                queue,
                registry,
                entity_bits,
                WESTON,
                &ph2d_physics_ecs::WestonAxle,
            );
        } else {
            super::inspector_ordering::queue_remove(queue, registry, entity_bits, WESTON);
        }
        return true;
    }
    let Some(next) = wheel_with_edit(current, edit) else {
        return false;
    };
    if next != current {
        queue_set(queue, registry, entity_bits, WHEEL, &next);
    }
    next.route_differs(&current)
}

/// **Uma edição aplicada a uma roldana** — a metade pura, e o funil único.
///
/// `None` quando a edição não é uma escrita de componente: um tag de `Wrap` que
/// não nomeia variante nenhum é **recusado**, nunca dobrado em `Auto`. Dobrar o
/// desconhecido no primeiro variant é o defeito que o `BodyKind` do W4 pagou —
/// com dois variants é redundante, com o terceiro vira um chip que seleciona
/// outra coisa.
#[must_use]
pub(crate) fn wheel_with_edit(
    current: ph2d_physics_ecs::PulleyWheel,
    edit: WheelFieldEdit,
) -> Option<ph2d_physics_ecs::PulleyWheel> {
    let mut next = current;
    match edit {
        WheelFieldEdit::Radius(v) => next.radius = v,
        // `0` volta a roldana a ser comum — a mesma regra que a geometria e a
        // engrenagem seguem, para que as três não possam discordar.
        WheelFieldEdit::RadiusOut(v) => next.radius_out = v,
        // ⚠️ **Recusado aqui de propósito:** a Weston é um MARCADOR e o
        // `apply_wheel_edit` a intercepta antes deste funil. Cair neste braço seria
        // uma escrita de campo para um fato que não mora em campo nenhum, e devolver
        // `Some(next)` inalterado faria a edição parecer aplicada.
        WheelFieldEdit::Weston(_) => return None,
        // 1-based na row, 0-based no componente. `saturating_sub` e não `- 1`:
        // a fronteira do painel já põe o piso em 1, e um zero que escapasse por
        // outra rota viraria `u16::MAX` num wrap silencioso.
        WheelFieldEdit::Order(v) => {
            next.order = u16::try_from(v.saturating_sub(1)).unwrap_or(u16::MAX);
        }
        WheelFieldEdit::Wrap(tag) => next.wrap = ph2d_physics_ecs::WrapSide::from_tag(tag)?,
        // Graus na row, radianos no componente.
        WheelFieldEdit::MotorDegPerS(v) => next.motor_speed = v.to_radians(),
        WheelFieldEdit::BreakEnabled(on) => next.break_enabled = on,
        WheelFieldEdit::BreakForce(v) => next.break_force = v,
        // ⚠️ **ARMA e não escreve**: o alvo vem do próximo clique no canvas. O
        // `None` diz *"isto não é uma escrita de componente"*, exatamente como o
        // tag de Wrap desconhecido — e é por ele que o `apply_wheel_edit` não
        // enfileira nada.
        // As duas ARMAM e não escrevem: o alvo vem do próximo clique no canvas.
        WheelFieldEdit::PickMountBody | WheelFieldEdit::PickRope => return None,
        // Voltar ao CENÁRIO. ⚠️ `mounted` volta a `false` junto: o local guardado
        // descreve um frame que não é mais o de ninguém, e deixá-lo semeado faria
        // a próxima montagem herdar o eixo da anterior em silêncio.
        WheelFieldEdit::Unmount => {
            next.body = 0;
            next.local = [0.0, 0.0];
            next.mounted = false;
        }
    }
    // A MESMA porta de carga que o load usa: raio negativo inverteria a
    // tangente, `NaN` envenenaria a pose e o hash C9.
    Some(next.clamped())
}

/// **Montar o eixo desta roldana no corpo `body`** (W-Pulley W3) — a porta que o
/// eyedropper da §13 termina.
///
/// Escreve IN PLACE, como o `set_joint_body` do W-JointAuthoring e pela mesma
/// razão: o pick resolve no meio do frame, dentro do handler de ponteiro, e o
/// undo global por-diff captura o resultado no fim dele.
///
/// ⚠️ **`mounted: false` de propósito.** O local do eixo NÃO é derivado aqui: ele
/// é semeado pela ponte, no próximo reconcile, contra a pose de REPOUSO do corpo
/// — a mesma conversão, no mesmo lugar, para toda rota de autoria. Derivá-lo aqui
/// seria a segunda porta, e ela discordaria da primeira em qualquer corpo que o
/// artista tivesse acabado de mover.
///
/// ⚠️ **Um corpo sem `Name` não pode ser apontado**, então ele ganha um — a
/// mesma cura que o `create_joint` aplica: a montagem viaja pelo NOME, e um corpo
/// anônimo seria uma montagem que o próximo reconcile esquece.
pub(crate) fn set_wheel_mount(sim: &mut SimWorld, wheel_bits: u64, body: Entity) {
    let name = match sim.world().get::<Name>(body) {
        Some(n) => n.as_str().to_string(),
        None => {
            let n = format!("Body {}", body.index());
            sim.world_mut().entity_mut(body).insert(Name::new(&n));
            n
        }
    };
    let entity = Entity::from_bits(wheel_bits);
    if let Some(mut w) = sim
        .world_mut()
        .get_mut::<ph2d_physics_ecs::PulleyWheel>(entity)
    {
        w.body = stable_name_id(&name);
        w.local = [0.0, 0.0];
        w.mounted = false;
    }
}

/// **Religar uma roldana a OUTRA corda** (W1) — o alvo do eyedropper da row Rope.
///
/// A corda é citada pelo **NOME** (`stable_name_id`), a mesma chave por que a
/// roldana já a aponta e por que o reconcile a resolve: bits de entidade são id de
/// ALOCAÇÃO e o undo os troca. Uma corda sem nome ganha um (uma corda que ninguém
/// pode citar não é apontável).
///
/// ⚠️ **Re-abre o `L0` pela porta compartilhada**, e não é higiene: a roldana entra
/// numa rota que ela não atravessava, então o comprimento daquela corda cresce — e
/// `L(rota) ≤ L0` com o `L0` parado nasce **violada**, que é o salto explosivo que a
/// wave do piso mediu em 13,97 m. A corda que ela DEIXOU fica com folga, que a
/// mesma medição mostrou ser inofensiva (a lei é um PISO, não uma re-derivação).
///
/// ⚠️ **Recusa quando o alvo não é uma polia** e devolve `false`, então o pick
/// segue armado em vez de deixar a roldana apontando um joint que não é corda — a
/// mesma escolha do `set_joint_body` sobre o self-joint.
pub(crate) fn set_wheel_rope(sim: &mut SimWorld, wheel_bits: u64, rope: Entity) -> bool {
    if sim
        .world()
        .get::<ph2d_physics_ecs::PhysicsJoint>(rope)
        .is_none_or(|j| j.kind != ph2d_physics_ecs::JointKind::Pulley)
    {
        return false;
    }
    let name = match sim.world().get::<Name>(rope) {
        Some(n) => n.as_str().to_string(),
        None => {
            let n = format!("Rope {}", rope.index());
            sim.world_mut().entity_mut(rope).insert(Name::new(&n));
            n
        }
    };
    let entity = Entity::from_bits(wheel_bits);
    {
        let Some(mut w) = sim
            .world_mut()
            .get_mut::<ph2d_physics_ecs::PulleyWheel>(entity)
        else {
            return false;
        };
        w.rope = stable_name_id(&name);
        // O empréstimo do componente sai de escopo aqui — a re-abertura do `L0` precisa
        // do `&mut World` de novo.
    }
    ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), entity);
    true
}

#[cfg(test)]
mod rope_pick_tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;

    fn rig() -> (SimWorld, u64, Entity, Entity) {
        let mut sim = SimWorld::new();
        let wheel = sim
            .world_mut()
            .spawn((
                Name::new("Wheel"),
                ph2d_physics_ecs::PulleyWheel {
                    rope: stable_name_id("Old Rope"),
                    radius: 0.3,
                    ..Default::default()
                },
                Transform::from_translation(Vec2::new(0.0, 4.0)),
            ))
            .id();
        let rope = sim
            .world_mut()
            .spawn((
                Name::new("New Rope"),
                ph2d_physics_ecs::PhysicsJoint {
                    kind: ph2d_physics_ecs::JointKind::Pulley,
                    anchored: true,
                    ..ph2d_physics_ecs::PhysicsJoint::of_kind(ph2d_physics_ecs::JointKind::Pulley)
                },
                Transform::default(),
            ))
            .id();
        // Um joint que NÃO é polia — o alvo que a porta tem de recusar.
        let pin = sim
            .world_mut()
            .spawn((
                Name::new("A Pin"),
                ph2d_physics_ecs::PhysicsJoint::default(),
                Transform::default(),
            ))
            .id();
        (sim, wheel.to_bits(), rope, pin)
    }

    /// **Religar a roldana escreve o NOME da corda nova e re-abre o `L0`.**
    ///
    /// ⚠️ **A re-abertura não é higiene:** a roldana entra numa rota que ela não
    /// atravessava, então o comprimento daquela corda CRESCE — e `L(rota) ≤ L0` com
    /// o `L0` parado nasce violada, o salto explosivo que a wave do piso mediu em
    /// 13,97 m.
    #[test]
    fn picking_a_rope_rebinds_the_wheel_and_reopens_the_length() {
        let (mut sim, wheel, rope, _) = rig();
        if let Some(mut j) = sim
            .world_mut()
            .get_mut::<ph2d_physics_ecs::PhysicsJoint>(rope)
        {
            j.anchored = true;
        }
        assert!(set_wheel_rope(&mut sim, wheel, rope), "a corda é uma polia");
        let w = *sim
            .world()
            .get::<ph2d_physics_ecs::PulleyWheel>(Entity::from_bits(wheel))
            .expect("a roldana vive");
        assert_eq!(
            w.rope,
            stable_name_id("New Rope"),
            "a roldana tinha de citar a corda NOVA pelo nome"
        );
        assert!(
            !sim.world()
                .get::<ph2d_physics_ecs::PhysicsJoint>(rope)
                .expect("a corda vive")
                .anchored,
            "o `L0` da corda nova tinha de ser RE-ABERTO — sem isso a rota cresce \
             sob um comprimento parado e o solver come a diferença num tique"
        );
    }

    /// **Um alvo que não é polia é RECUSADO**, e o `false` mantém o pick armado em
    /// vez de deixar a roldana apontando um joint que não é corda.
    #[test]
    fn picking_a_non_pulley_is_refused() {
        let (mut sim, wheel, _, pin) = rig();
        assert!(
            !set_wheel_rope(&mut sim, wheel, pin),
            "um Pin não é uma corda"
        );
        assert_eq!(
            sim.world()
                .get::<ph2d_physics_ecs::PulleyWheel>(Entity::from_bits(wheel))
                .expect("vive")
                .rope,
            stable_name_id("Old Rope"),
            "a recusa não pode ter mexido na corda que ela já tinha"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wheel() -> ph2d_physics_ecs::PulleyWheel {
        ph2d_physics_ecs::PulleyWheel {
            rope: 7,
            radius: 0.3,
            ..Default::default()
        }
    }

    /// **A pergunta é sobre a ROTA, não sobre a igualdade do componente.**
    ///
    /// É o predicado que decide se o `L0` da corda é re-aberto (2026-07-29). Ele
    /// tem de dizer SIM ao raio — o gesto que o Enio reportou — e **NÃO** ao
    /// motor: a cena `=59` autora o motor com o relógio ANDANDO de propósito, e
    /// re-derivar comprimento de corda ali prenderia a restrição na configuração
    /// do instante.
    ///
    /// Mutação: `route_differs` sempre `false` ⇒ as duas metades de raio caem;
    /// sempre `true` ⇒ a do motor cai.
    #[test]
    fn only_a_route_change_reopens_the_rope_length() {
        let base = wheel();
        for (name, edit, expected) in [
            ("Radius", WheelFieldEdit::Radius(0.9), true),
            ("RadiusOut", WheelFieldEdit::RadiusOut(0.2), true),
            ("Order", WheelFieldEdit::Order(3), true),
            ("Motor", WheelFieldEdit::MotorDegPerS(60.0), false),
            ("BreakForce", WheelFieldEdit::BreakForce(9.0), false),
        ] {
            let next = wheel_with_edit(base, edit).expect("edição de componente");
            assert_eq!(
                next.route_differs(&base),
                expected,
                "{name}: o predicado da rota respondeu {} e a geometria diz {expected}",
                next.route_differs(&base)
            );
        }
    }
}
