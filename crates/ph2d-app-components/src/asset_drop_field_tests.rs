//! GEF 20b — as caixas dos campos que o arrasto marca (o idioma do Godot).

use super::*;
use ph2d_ecs::{CameraFollow, GameCamera, World};
use ph2d_panel_inspector::ids::INSP_CAMERA_TARGET;

/// ⭐ **A caixa do Target diz se aceita**: arrastando um objecto, o campo acende verde mesmo longe do
/// cursor; uma imagem não o acende, e por cima dele ele fica vermelho.
///
/// **Mutações que devem sangrar:** marcar os campos que recusam longe do cursor · esquecer o de debaixo
/// do cursor · trocar o veredito.
#[test]
fn a_caixa_do_alvo_diz_se_aceita_o_que_vai_na_mao() {
    let mut w = World::new();
    let cam = w
        .spawn((GameCamera::default(), CameraFollow::default()))
        .id()
        .to_bits();
    let info = crate::camera_inspector::build_camera_info(
        &mut w,
        cam,
        1,
        Default::default(),
        Default::default(),
        24.0,
        false,
    )
    .expect("info");
    ph2d_panel_inspector::set_current_inspector_camera(Some(info));
    let ids = [NodeId(7), INSP_CAMERA_TARGET];
    let objecto = DragPayload::Object { stable_id: 42 };
    let imagem = DragPayload::Image { asset: [0; 32] };
    let marca = |verdict, under| DropFieldMark {
        id: INSP_CAMERA_TARGET,
        verdict,
        under,
    };
    let medido = [
        field_marks(ids, objecto, None),
        field_marks(ids, objecto, Some(INSP_CAMERA_TARGET)),
        field_marks(ids, imagem, None),
        field_marks(ids, imagem, Some(INSP_CAMERA_TARGET)),
    ];
    ph2d_panel_inspector::set_current_inspector_camera(None);
    assert_eq!(medido[0], [marca(DragVerdict::Accept, false)]);
    assert_eq!(medido[1], [marca(DragVerdict::Accept, true)]);
    assert_eq!(medido[2], []);
    assert_eq!(medido[3], [marca(DragVerdict::Refuse, true)]);
}

/// Sem câmera que segue no Inspector, o Target não é campo — nada se marca.
#[test]
fn sem_camera_que_segue_nada_se_marca() {
    ph2d_panel_inspector::set_current_inspector_camera(None);
    let objecto = DragPayload::Object { stable_id: 42 };
    assert_eq!(
        field_marks([INSP_CAMERA_TARGET], objecto, Some(INSP_CAMERA_TARGET)),
        []
    );
}

/// ⭐ GEF 21 E2 — **o campo Sound é o 4.º campo que recebe, pela MESMA porta**: um som acende-o verde
/// e larga-se nele (a fonte passa a nomeá-lo); uma imagem, um prefab ou um objecto recusam lá; o som
/// recusa no canvas, na ranhura de textura e numa gaveta, e no texto de um script escreve o nome.
///
/// **Mutações que devem sangrar:** tirar o `audio_sound_slot_pick` do `field_target` · aceitar um som
/// fora do campo · aceitar outra carga no campo.
#[test]
fn o_campo_sound_recebe_um_som_e_so_um_som() {
    use ph2d_panel_inspector::ids::INSP_AUDIO_SOUND;
    let mut w = World::new();
    let fonte = w.spawn(ph2d_ecs::AudioSource2D::default()).id().to_bits();
    let info = crate::audio_inspector::build_audio_info(&mut w, fonte, 1).expect("info");
    ph2d_panel_inspector::set_current_inspector_audio(Some(info));
    let som = DragPayload::Sound { asset: [5; 32] };
    let campo = field_target(INSP_AUDIO_SOUND).expect("o campo Sound recebe");
    assert_eq!(
        resolve(som, campo),
        DropAction::SetSound {
            entity_bits: fonte,
            asset: [5; 32]
        }
    );
    let marcas = field_marks([NodeId(7), INSP_AUDIO_SOUND], som, None);
    assert_eq!(
        marcas.len(),
        1,
        "o campo acende verde longe do cursor: {marcas:?}"
    );
    for outra in [
        DragPayload::Image { asset: [0; 32] },
        DragPayload::Prefab { stable_id: 3 },
        DragPayload::Object { stable_id: 4 },
    ] {
        assert_eq!(resolve(outra, campo), DropAction::Refuse, "{outra:?}");
    }
    for alvo in [
        DropTarget::Canvas {
            world: [0.0; 2],
            over: None,
        },
        DropTarget::InspectorTexture { entity_bits: 1 },
        DropTarget::CatalogRow { catalog: None },
        DropTarget::Chrome,
    ] {
        assert_eq!(resolve(som, alvo), DropAction::Refuse, "{alvo:?}");
    }
    assert_eq!(
        resolve(som, DropTarget::Code { on_text: true }),
        DropAction::WriteCode
    );
    ph2d_panel_inspector::set_current_inspector_audio(None);
}
