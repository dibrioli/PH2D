//! ⭐⭐⭐ **A secção CAMERA** (TOP-20 #7, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`crate::render_loop::inspector_audio`], pela mesma razão dele.
//!
//! # ⚠️ O snapshot só existe para quem TEM a câmera
//!
//! É o ADR-0166: *o Inspector mostra o que o objecto TEM, e um componente anexa-se pela paleta*. O
//! `CameraFollow` e o `CameraLimits` são corpos **dentro** dela — anexá-los sem a câmera não pinta
//! nada, e isso está certo: seguir sem enquadrar não é uma pergunta.
//!
//! # ⭐⭐ QUATRO coisas são DERIVADAS aqui, e nenhuma se podia derivar no painel
//!
//! - **`camera_count` / `is_active_camera`** — quem manda sai de uma varredura da cena, com
//!   desempate por identidade. Sem isso o artista afina uma câmera que ninguém usa e lê o silêncio
//!   como *«os números não fazem nada»*.
//! - **`target_found`** — resolução por NOME contra a cena. *«segue ninguém»* e *«segue alguém
//!   parado»* dão a mesma câmera imóvel, e só uma delas é defeito.
//! - ⭐⭐ **`smaller_than_view`** — geometria da JANELA (proporção do ecrã × altura da câmera), não
//!   dos quatro números da cerca. Quando ela é verdadeira a lei **fixa a câmera no centro da
//!   caixa** e a câmera deixa de seguir seja quem for. *Nenhum artista adivinha isso olhando para
//!   `min` e `max`.*
//! - **`preview_on`** — é estado da SHELL, e o painel não tem como o saber.
//!
//! # ⚠️ O `Preview` não passa pelo commit
//!
//! Ele não escreve no documento: liga e desliga a vista. Quem o serve é o dreno, que tem a `App`.
//! *O que esta porta faz é o commit de um CAMPO.*

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    CAMERA_MAX_HEIGHT_WORLD, CAMERA_MIN_HEIGHT_WORLD, CameraFollow, CameraLimits, Entity,
    GameCamera, SimWorld, World,
};
use ph2d_editor::{
    CameraFieldEdit, InspectorCameraFollow, InspectorCameraInfo, InspectorCameraLimits,
    InspectorGameCamera,
};

use crate::render_loop::inspector_ordering::queue_set;

const CAMERA: &str = "ph2d::ecs::GameCamera";
const FOLLOW: &str = "ph2d::ecs::CameraFollow";
const LIMITS: &str = "ph2d::ecs::CameraLimits";

/// **A cerca não cabe na janela, nalgum eixo?**
///
/// ⚠️ **`lo + meia > hi − meia` é a MESMA condição do ramo de pino da lei** ([`ph2d_ecs::
/// clamp_axis_to_limits`]) — escrita aqui e não importada porque o painel precisa dela como um
/// `bool` de snapshot, não como uma posição. *Se ela divergir, o aviso mente sobre o que a câmera
/// faz*, e é por isso que o gate da shell mede as duas juntas.
fn limits_smaller_than_view(l: &CameraLimits, half: [f32; 2]) -> bool {
    (0..2).any(|i| l.min[i] + half[i] > l.max[i] - half[i])
}

/// O snapshot da secção, ou `None` quando o objecto não tem câmera.
pub(super) fn build_camera_info(
    world: &mut World,
    entity_bits: u64,
    selected_count: usize,
    aspect: f32,
    preview_on: bool,
) -> Option<InspectorCameraInfo> {
    let entity = Entity::from_bits(entity_bits);
    let cam = world.get::<GameCamera>(entity).cloned()?;
    let half = ph2d_ecs::half_extent(cam.height_world, aspect);

    let camera_count = ph2d_ecs::camera_count(world);
    let is_active_camera = ph2d_ecs::active_camera_of(world) == Some(entity);

    let follow = world.get::<CameraFollow>(entity).cloned().map(|f| {
        // ⚠️ **Um nome vazio não é «por achar»** — ele é *«não segue ninguém»*, e o painel só avisa
        // quando há um nome escrito que não casa. Marcar o vazio como perdido gritaria sobre uma
        // câmera fixa perfeitamente correcta.
        let target_found = !f.target.trim().is_empty() && {
            let id = ph2d_ecs::stable_id_for_name(world, f.target.trim());
            ph2d_ecs::entity_of_stable_id(world, ph2d_ecs::StableId(id)).is_some()
        };
        InspectorCameraFollow {
            target: f.target,
            damping: f.damping,
            dead_zone: f.dead_zone,
            lookahead: f.lookahead,
            offset: f.offset,
            target_found,
        }
    });

    let limits = world
        .get::<CameraLimits>(entity)
        .cloned()
        .map(|l| InspectorCameraLimits {
            smaller_than_view: limits_smaller_than_view(&l, half),
            min: l.min,
            max: l.max,
        });

    Some(InspectorCameraInfo {
        entity_bits,
        camera: InspectorGameCamera {
            height_world: cam.height_world,
            offset: cam.offset,
            priority: cam.priority,
            active: cam.active,
            cull_mask: cam.cull_mask,
        },
        follow,
        limits,
        camera_count,
        is_active_camera,
        preview_on,
        selected_count,
    })
}

/// Aplica uma [`CameraFieldEdit`] que toca no DOCUMENTO.
///
/// ⚠️ **`Preview` devolve sem escrever nada** — ele não é um campo, e quem o serve é o dreno. Ver o
/// doc do módulo.
///
/// ⚠️ **Os três componentes são escritos SEPARADAMENTE**, e a edição diz qual: mandar os três a
/// cada mexida faria um objecto sem `CameraLimits` ganhar uma cerca por escrever a altura.
pub(super) fn apply_camera_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &CameraFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    match edit {
        CameraFieldEdit::Preview(_) => {}

        CameraFieldEdit::Height(_)
        | CameraFieldEdit::Offset(_)
        | CameraFieldEdit::Priority(_)
        | CameraFieldEdit::Active(_)
        | CameraFieldEdit::CullBit(_, _) => {
            let Some(mut c) = sim.world().get::<GameCamera>(entity).cloned() else {
                return;
            };
            match edit {
                CameraFieldEdit::Height(v) => {
                    // ⚠️ **A saturação é do MOTOR**, e mora aqui porque é aqui que o valor entra na
                    // cena — a mesma conta do `AudioSource2D::max_distance`.
                    c.height_world = v.clamp(CAMERA_MIN_HEIGHT_WORLD, CAMERA_MAX_HEIGHT_WORLD); // CLAMP-OK: o teto é o do motor
                }
                CameraFieldEdit::Offset(v) => c.offset = *v,
                CameraFieldEdit::Priority(v) => c.priority = *v,
                CameraFieldEdit::Active(on) => c.active = *on,
                CameraFieldEdit::CullBit(bit, on) => {
                    let m = 1u32 << u32::from(*bit).min(31); // CLAMP-OK: 32 bits, e o painel só tem 32 caixas
                    if *on {
                        c.cull_mask |= m;
                    } else {
                        c.cull_mask &= !m;
                    }
                }
                _ => unreachable!("o braço externo já separou as variantes"),
            }
            queue_set(queue, registry, entity_bits, CAMERA, &c);
        }

        CameraFieldEdit::Target(_)
        | CameraFieldEdit::Damping(_)
        | CameraFieldEdit::DeadZone(_)
        | CameraFieldEdit::Lookahead(_)
        | CameraFieldEdit::FollowOffset(_) => {
            let Some(mut f) = sim.world().get::<CameraFollow>(entity).cloned() else {
                return;
            };
            match edit {
                CameraFieldEdit::Target(t) => f.target = t.trim().to_string(),
                // ⚠️ **`0` é INSTANTÂNEO e não «parado»** — ver o doc da lei; o teto é o do oráculo,
                // acima do qual a interpolação sai do domínio dela.
                CameraFieldEdit::Damping(v) => {
                    f.damping = [v[0].clamp(0.0, 60.0), v[1].clamp(0.0, 60.0)]; // CLAMP-OK: 1/s, a faixa do campo
                }
                CameraFieldEdit::DeadZone(v) => {
                    f.dead_zone = [v[0].clamp(0.0, 1.0), v[1].clamp(0.0, 1.0)]; // CLAMP-OK: fracção da meia-janela
                }
                CameraFieldEdit::Lookahead(v) => {
                    f.lookahead = [v[0].clamp(0.0, 5.0), v[1].clamp(0.0, 5.0)]; // CLAMP-OK: segundos
                }
                CameraFieldEdit::FollowOffset(v) => f.offset = *v,
                _ => unreachable!("o braço externo já separou as variantes"),
            }
            queue_set(queue, registry, entity_bits, FOLLOW, &f);
        }

        CameraFieldEdit::LimitMin(_) | CameraFieldEdit::LimitMax(_) => {
            let Some(mut l) = sim.world().get::<CameraLimits>(entity).cloned() else {
                return;
            };
            match edit {
                CameraFieldEdit::LimitMin(v) => l.min = *v,
                CameraFieldEdit::LimitMax(v) => l.max = *v,
                _ => unreachable!("o braço externo já separou as variantes"),
            }
            // ⚠️ **A cerca invertida NÃO é recusada, e é deliberado:** arrastar o `min` para lá do
            // `max` é um estado transitório de um arrasto, e a lei trata-o (ela fixa no centro da
            // caixa). Recusar aqui faria o número parar debaixo do dedo.
            queue_set(queue, registry, entity_bits, LIMITS, &l);
        }
    }
}

#[cfg(test)]
#[path = "inspector_camera_tests.rs"]
mod tests;
