//! ⭐⭐⭐ **A PONTE DA CÂMERA DE JOGO** — onde a cena passa a mandar no enquadramento (TOP-20 #7, W2).
//!
//! # A fronteira, e porque ela é exactamente esta
//!
//! - A **lei** vive no [`ph2d_ecs::camera_2d`]: qual câmera manda, para onde ela olha, como amortece
//!   e onde a cerca a prende. Ela é pura, portada do Godot 4.7.2 (MIT) e gateada contra 45 corridas
//!   dele.
//! - A **vista** vive no [`ph2d_render::Camera2d`], que é o campo `App::camera` — o que o editor já
//!   usava para dar pan e zoom.
//! - Este ficheiro é a **costura**: lê o mundo, chama a lei, e devolve o que a vista devia ser.
//!
//! ⚠️ **Ele não ESCREVE a vista**, e a separação é o que torna esta função testável sem janela: ela
//! devolve uma [`CameraView`], e quem a aplica é o chamador. *Uma ponte que muta a shell só se mede
//! com uma shell.*
//!
//! # ⚠️ O que este passe NÃO faz
//!
//! ⛔ **Ele não escreve componente registado nenhum.** O único estado que ele muta é o
//! [`ph2d_ecs::CameraRuntime`], que **não** deriva `Serialize` e por isso o undo não o fotografa —
//! logo **não há passo espúrio a declarar** e o `preview_drive` não entra na assinatura. É a mesma
//! conta que o [`super::timer_tick`] faz, escrita pela mesma razão.
//!
//! # ⚠️ «Assentar» é uma ARESTA, e é o NASCIMENTO
//!
//! É a lição que o `Timer` e o som pagaram, com report do dono nas duas. Uma câmera que nasce em
//! `(0,0)` e amortece até ao alvo **viaja** pelo mundo durante o primeiro segundo de jogo. O
//! [`ph2d_ecs::CameraRuntime::settled`] responde *«esta câmera já foi posta alguma vez?»* — e no
//! quadro em que ela nasce a lei é saltada e o centro é o alvo. É o `reset_smoothing()` do oráculo.

use ph2d_ecs::{
    CameraFollow, CameraLimits, CameraRuntime, Entity, GameCamera, SimWorld, StableId, World,
};

/// O que a câmera de cena quer que a vista seja.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CameraView {
    pub(crate) center: [f32; 2],
    pub(crate) height_world: f32,
    pub(crate) cull_mask: u32,
}

/// O que o quadro fez com a câmera — o que o smoke imprime.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CameraSceneReport {
    /// Quantas câmeras a cena tem. ⚠️ **`0` é informação**: a vista fica sendo a do editor.
    pub(crate) cameras: usize,
    /// A câmera que manda tem `CameraFollow`?
    pub(crate) following: bool,
    /// ⭐ **O alvo do `follow` foi ENCONTRADO?** ⚠️ `false` com `following` verdadeiro é o report
    /// que o dono lê como *«a câmera não segue nada»* — um nome escrito com erro, ou um objecto que
    /// um `Ctrl+Z` levou. Sem esta coluna os dois casos leem-se como *«a câmera está parada»*.
    pub(crate) target_found: bool,
    /// O nome que o `follow` procura, quando há um.
    pub(crate) target_name: String,
    /// Onde a câmera activa ficou.
    pub(crate) center: [f32; 2],
    /// A cerca mordeu neste quadro? ⚠️ Só é `true` quando ela de facto MOVEU o centro.
    pub(crate) limited: bool,
}

/// **Toda câmera tem um estado vivo, e ele nasce onde a câmera está.**
///
/// ⚠️ **Antes de qualquer `ticks == 0`, e de propósito** — é a lei do `start_autostart_timers`: uma
/// câmera anexada pela paleta num quadro sem passo fixo ficaria por armar, e o sintoma seria *«às
/// vezes a câmera não pega»*.
fn ensure_runtime(world: &mut World) {
    let nascer: Vec<(Entity, [f32; 2])> = world
        .query_filtered::<(Entity, &GameCamera), bevy_ecs::prelude::Without<CameraRuntime>>()
        .iter(world)
        .map(|(e, _)| e)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|e| {
            let p = ph2d_ecs::world_transform(world, e)
                .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
            (e, p)
        })
        .collect();
    for (e, p) in nascer {
        if let Ok(mut ent) = world.get_entity_mut(e) {
            ent.insert(CameraRuntime {
                center: p,
                anchor: p,
                last_target: None,
                velocity: [0.0, 0.0],
                settled: false,
            });
        }
    }
}

/// A proporção `largura/altura` da janela — a **única** conversão de pixels desta ponte.
///
/// ⚠️ **Ela existe para o `as f32` ter um sítio só**: a meia-janela da lei precisa da proporção, e
/// uma conversão espalhada por dois chamadores é a segunda resposta à mesma pergunta. A perda de
/// precisão é nula na faixa de um ecrã (`u32` até 2²⁴ é exacto em `f32`).
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub(crate) fn aspect_of(size: ph2d_host::WindowSize) -> f32 {
    size.width.max(1) as f32 / size.height.max(1) as f32
}

/// A posição de mundo de uma entidade.
fn position_of(world: &World, e: Entity) -> Option<[f32; 2]> {
    let t = ph2d_ecs::world_transform(world, e)?;
    Some([t.translation.x, t.translation.y])
}

/// **Quem o `follow` procura** — pelo NOME, como toda referência durável desta casa.
///
/// ⚠️ **Devolve `None` quando o nome não casa**, e o chamador transforma isso numa coluna do
/// relatório em vez de num silêncio: *«segue ninguém»* e *«segue alguém que está parado»* dão
/// exactamente a mesma câmera imóvel, e só uma delas é um defeito da cena.
fn follow_target(world: &mut World, name: &str) -> Option<Entity> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let id = ph2d_ecs::stable_id_for_name(world, name);
    ph2d_ecs::entity_of_stable_id(world, StableId(id))
}

/// Um passo da câmera de cena. Devolve o que a vista devia ser, ou `None` quando a cena não tem
/// câmera activa — e nesse caso o editor fica com o enquadramento dele, que é o de hoje.
///
/// `aspect` é `largura/altura` da janela; `ticks` e `fixed_dt` são os do passo fixo.
pub(crate) fn update(
    sim: &mut SimWorld,
    aspect: f32,
    ticks: u32,
    fixed_dt: f64,
) -> (Option<CameraView>, CameraSceneReport) {
    let world = sim.world_mut();
    ensure_runtime(world);

    let mut report = CameraSceneReport {
        cameras: ph2d_ecs::camera_count(world),
        ..CameraSceneReport::default()
    };
    let Some(cam_e) = ph2d_ecs::active_camera_of(world) else {
        return (None, report);
    };
    let Some(cam) = world.get::<GameCamera>(cam_e).cloned() else {
        return (None, report);
    };

    let half = ph2d_ecs::half_extent(cam.height_world, aspect);
    let limits = world.get::<CameraLimits>(cam_e).cloned();
    let follow = world.get::<CameraFollow>(cam_e).cloned();
    report.following = follow.is_some();

    // **Para onde ela olha.** Sem `follow`, é a pose autorada da própria câmera — que é o que faz
    // uma câmera fixa ser uma câmera fixa, sem componente nenhum a mais.
    let pose = position_of(world, cam_e).unwrap_or([0.0, 0.0]);
    let alvo = match follow.as_ref() {
        Some(f) => {
            report.target_name = f.target.trim().to_string();
            match follow_target(world, &f.target) {
                Some(t) => {
                    report.target_found = true;
                    position_of(world, t).unwrap_or(pose)
                }
                // ⚠️ **Alvo por achar ⇒ a câmera fica na pose dela**, e o relatório diz porquê.
                None => pose,
            }
        }
        None => pose,
    };

    let Some(mut rt) = world.get::<CameraRuntime>(cam_e).cloned() else {
        return (None, report);
    };

    let dt = fixed_dt as f32;
    // ⚠️⚠️ **O tempo entre as duas AMOSTRAS do alvo, que NÃO é o passo da lei.** A ponte vê o alvo
    // uma vez por quadro e um quadro leva `ticks` passos — dividir por um passo quando passaram
    // dois lê o DOBRO da velocidade, e foi isso que a auditoria de 2026-09-10 mediu (a mira saltava
    // de `+4,00 m` para `+8,00 m` em toda moldura de dois tiques). Ver [`ph2d_ecs::sample_velocity`].
    let sample_dt = dt * ticks as f32;
    if !rt.settled {
        // ⭐ **O nascimento não amortece E não antecipa** — sem duas amostras não há velocidade.
        let mira = ph2d_ecs::aim_at(alvo, [0.0, 0.0], follow.as_ref().unwrap_or(&SEM_FOLLOW));
        rt.center = mira;
        rt.anchor = mira;
        rt.velocity = [0.0, 0.0];
        rt.settled = true;
    } else if let Some(f) = follow.as_ref() {
        // ⚠️⚠️ **UM QUADRO SEM TEMPO NÃO TOCA EM NADA QUE SEJA POR-TEMPO.**
        //
        // ⛔ Foi o terceiro defeito do report de 2026-09-10, e o maior dos três: com `ticks == 0` o
        // `sample_dt` é zero, a amostra de velocidade devolve `[0, 0]` (não há tempo de onde a
        // tirar) e o `damp_axis` — cujo braço de `dt <= 0` significa **instantâneo** — adoptava-a.
        // ⇒ **cada quadro perdido ZERAVA a velocidade suavizada**, e com dois em dez ela nunca
        // subia: medida, ela lia `0,78 m/s` sobre um herói a `8,00`. *A antecipação estava dez
        // vezes fraca, e a causa não era a lei — era um quadro sem tempo a responder a uma pergunta
        // sobre tempo.*
        //
        // ⚠️ **A velocidade é ESTADO**, e estado não se re-deriva num instante em que nada passou.
        if ticks > 0 {
            // ⚠️ **Uma vez por quadro, fora do laço** — ela é propriedade da AMOSTRA, e o laço
            // percorre os passos que couberam entre duas amostras. Recalculá-la lá dentro leria a
            // mesma diferença `ticks` vezes.
            let cru = ph2d_ecs::sample_velocity(rt.last_target, alvo, sample_dt);
            rt.velocity = ph2d_ecs::smooth_velocity(rt.velocity, cru, f.lookahead, sample_dt);
        }
        let mira = ph2d_ecs::aim_at(alvo, rt.velocity, f);
        for _ in 0..ticks {
            let (a, c) = ph2d_ecs::follow_step(rt.anchor, rt.center, mira, half, f, None, dt);
            rt.anchor = a;
            rt.center = c;
        }
    } else {
        // Sem `follow` a câmera é a pose dela, sem lei nenhuma pelo meio.
        rt.center = alvo;
        rt.anchor = alvo;
    }
    // ⚠️⚠️ **A amostra só avança quando TEMPO PASSOU.** Num quadro sem tique nenhum a simulação não
    // andou, e guardar o alvo ali daria um par `(Δ, sample_dt)` cujo denominador conta um tempo que
    // o numerador não viu. *Uma amostra é um par, e guardar metade dele estraga o outro lado.*
    if ticks > 0 {
        rt.last_target = Some(alvo);
    }

    // ⭐⭐ **A cerca entra AQUI e não dentro do laço**, e a diferença é observável: aplicá-la a cada
    // tique deixaria o amortecimento a puxar para fora e a cerca a puxar de volta, que é o tremor
    // clássico na borda do nível. Ela é idempotente, então uma vez por quadro dá o mesmo sítio.
    if let Some(l) = limits.as_ref() {
        let antes = rt.center;
        for (eixo, (c, h)) in rt.center.iter_mut().zip(half.iter()).enumerate() {
            *c = ph2d_ecs::clamp_axis_to_limits(*c, *h, l.min[eixo], l.max[eixo]);
        }
        report.limited = rt.center != antes;
    }

    report.center = rt.center;
    let vista = CameraView {
        // ⚠️ **O `offset` entra na VISTA e não no estado vivo** — senão ele realimentaria a zona
        // morta e a câmera afastar-se-ia do alvo um pouco mais a cada quadro.
        center: [rt.center[0] + cam.offset[0], rt.center[1] + cam.offset[1]],
        height_world: cam.height_world.clamp(
            ph2d_ecs::CAMERA_MIN_HEIGHT_WORLD,
            ph2d_ecs::CAMERA_MAX_HEIGHT_WORLD,
        ), // CLAMP-OK: a faixa do motor
        cull_mask: cam.cull_mask,
    };
    if let Ok(mut ent) = world.get_entity_mut(cam_e) {
        ent.insert(rt);
    }
    (Some(vista), report)
}

/// O `follow` neutro, para a mira do nascimento quando não há nenhum.
const SEM_FOLLOW: CameraFollow = CameraFollow {
    target: String::new(),
    damping: [0.0, 0.0],
    dead_zone: [0.0, 0.0],
    lookahead: [0.0, 0.0],
    offset: [0.0, 0.0],
};

#[cfg(test)]
#[path = "camera_2d_tests.rs"]
mod tests;
