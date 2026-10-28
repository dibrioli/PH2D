//! O formato do jogo e a moldura da câmera escolhidos no cartão Camera do Inspector (plano GEF 17):
//! são definições do PROJECTO, e esta porta escreve-as no `ProjectSettings`.

use ph2d_editor_core::game_format::{
    CAMERA_ICON_MAX_PX, CAMERA_ICON_MIN_PX, CUSTOM_MAX_PX, CUSTOM_MIN_PX,
};
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::{CameraFieldEdit, CameraFrameShow, GamePreset};

/// Aplica `edit` a `p` se for uma definição do projecto. `true` = a edição era daqui (consumida),
/// mesmo quando não mudou nada.
pub fn aplica_formato(p: &mut ProjectSettings, edit: &CameraFieldEdit) -> bool {
    match edit {
        CameraFieldEdit::Format(tag) => p.game_format.preset = GamePreset::from_u8(*tag),
        CameraFieldEdit::CustomSize(px) => {
            p.game_format.custom = px.map(|v| v.clamp(CUSTOM_MIN_PX, CUSTOM_MAX_PX)); // CLAMP-OK: tecto = textura do wgpu
        }
        CameraFieldEdit::IconSize(px) if px.is_finite() => {
            p.camera_icon_px = px.clamp(CAMERA_ICON_MIN_PX, CAMERA_ICON_MAX_PX); // CLAMP-OK: a faixa do «Icon Size»
        }
        CameraFieldEdit::IconSize(_) => {}
        CameraFieldEdit::Frame(so_selecionada) => {
            p.camera_frame = if *so_selecionada {
                CameraFrameShow::Selected
            } else {
                CameraFrameShow::Always
            };
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_formato_e_a_moldura_escrevem_no_projecto() {
        let mut p = ProjectSettings::default();
        assert!(aplica_formato(&mut p, &CameraFieldEdit::Format(8)));
        assert_eq!(p.game_format.preset, GamePreset::GameBoy);
        assert!(aplica_formato(
            &mut p,
            &CameraFieldEdit::CustomSize([1, 99_999])
        ));
        assert_eq!(p.game_format.custom, [CUSTOM_MIN_PX, CUSTOM_MAX_PX]);
        assert!(aplica_formato(&mut p, &CameraFieldEdit::Frame(true)));
        assert_eq!(p.camera_frame, CameraFrameShow::Selected);
        assert!(aplica_formato(&mut p, &CameraFieldEdit::IconSize(1000.0)));
        assert_eq!(p.camera_icon_px, CAMERA_ICON_MAX_PX);
        assert!(aplica_formato(&mut p, &CameraFieldEdit::IconSize(f32::NAN)));
        assert_eq!(p.camera_icon_px, CAMERA_ICON_MAX_PX);
        assert!(!aplica_formato(&mut p, &CameraFieldEdit::Height(3.0)));
    }
}
