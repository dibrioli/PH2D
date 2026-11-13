//! O miolo das fotos da janela do jogo (`camera_chave`, `camera_desfaz`, `sprite_*`, onda 18): o que a mão lê
//! do mundo e as ações do barramento que empurra. `shells/desktop/src/janela_do_jogo_smoke.rs` só as executa.

use crate::mao::Mao;
use crate::por_nome;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_timeline::TimelineState;

/// As chaves que a timeline guarda para a «Camera» da cena: `propriedade=n` por ligação (a foto `camera_chave`).
pub fn chaves_da_camera(sim: &mut SimWorld, tl: &TimelineState) -> String {
    let Some(cam) = por_nome(sim, "Camera") else {
        return "sem câmera".to_owned();
    };
    let clip = tl.doc.active_clip();
    let v: Vec<String> = tl
        .doc
        .bindings()
        .iter()
        .filter(|b| b.entity == cam.to_bits())
        .map(|b| {
            let n = clip.track(b.target).map_or(0, |t| t.len());
            format!("{:?}={n}", b.prop)
        })
        .collect();
    if v.is_empty() {
        "nenhuma ligação".to_owned()
    } else {
        v.join(" · ")
    }
}

/// As ações do barramento de um gesto da mão: sair do Draw, aplicar a imagem à Platform High, deslocá-la ou
/// mudar-lhe o pivô pelo clique do Inspector. Vazio quando falta o alvo ou o gesto não empurra nada.
#[must_use]
pub fn acoes_da_mao(sim: &mut SimWorld, m: Mao) -> Vec<EditorAction> {
    use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
    match m {
        Mao::SaiDoDraw => vec![EditorAction::ObjectMode(ModeRequest::Enter(
            ObjectMode::Object,
        ))],
        Mao::Aplica => por_nome(sim, "Platform High")
            .map(|e| EditorAction::OneShotImageOp {
                tool_id: "painter",
                entity_bits: e.to_bits(),
            })
            .into_iter()
            .collect(),
        Mao::Desloca | Mao::Pivo(_) => mexe_na_sprite(sim, m).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// O clique do Inspector sobre a Platform High: pela mesma ação do barramento.
fn mexe_na_sprite(sim: &mut SimWorld, m: Mao) -> Option<EditorAction> {
    let e = por_nome(sim, "Platform High")?;
    let t = sim.world().get::<ph2d_ecs::Transform>(e).copied()?;
    Some(if let Mao::Pivo(ponto) = m {
        EditorAction::ImageUseEdit {
            entity_bits: e.to_bits(),
            edit: ph2d_editor_core::ImageUseEdit::Pivot(ph2d_editor_core::SpritePivot::Ponto(
                ponto,
            )),
        }
    } else {
        EditorAction::InspectorTransformEdit(ph2d_editor_core::InspectorTransformInfo {
            entity_bits: e.to_bits(),
            translation: [t.translation.x + 1.0, t.translation.y],
            rotation_rad: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew_rad: [t.skew_x, t.skew_y],
        })
    })
}

/// `[sprite-pixel]`: a imagem de cada plataforma (o primeiro byte do id), uma linha por plataforma.
#[must_use]
pub fn imagens_das_plataformas(
    sim: &mut SimWorld,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Vec<String> {
    ["Platform High", "Platform Low"]
        .into_iter()
        .map(|nome| {
            let e = por_nome(sim, nome);
            let id = e.map(|e| {
                let w = sim.world();
                ph2d_render::texture_of(
                    w.get::<ph2d_ecs::SpritePixels>(e),
                    w.get::<ph2d_render::Sprite>(e),
                    atlas,
                )
                .map(|a| a.as_bytes()[0])
            });
            format!("[sprite-pixel] {nome}: imagem {id:?}")
        })
        .collect()
}

/// `[sprite-prova]`: a pose da Platform High, onde o desenho e a colisão caem no mundo, o filho, e as chaves.
#[must_use]
pub fn prova_da_sprite(
    sim: &mut SimWorld,
    tl: &TimelineState,
    t: f64,
    desfazer: usize,
) -> Option<String> {
    let e = por_nome(sim, "Platform High")?;
    let filho = por_nome(sim, "Coin 4");
    let chaves: Vec<String> = tl
        .doc
        .bindings()
        .iter()
        .filter(|b| b.entity == e.to_bits())
        .map(|b| {
            let n = tl.doc.active_clip().track(b.target).map_or(0, |k| k.len());
            match b.prop {
                ph2d_timeline::PropKind::Field(f) => format!("campo{}={n}", f.campo),
                p => format!("{p:?}={n}"),
            }
        })
        .collect();
    let w = sim.world();
    let p = w
        .get::<ph2d_ecs::Transform>(e)
        .map_or(ph2d_core::Vec2::ZERO, |t| t.translation);
    let s = w.get::<ph2d_render::Sprite>(e).copied();
    let no_mundo = |o: [f32; 2]| [p.x + o[0], p.y + o[1]];
    let desenho = s.map(|s| no_mundo(s.resolve_anchor(100.0)));
    let col = w
        .get::<ph2d_physics_ecs::Collider>(e)
        .map(|c| no_mundo(c.offset));
    let local = filho.and_then(|f| w.get::<ph2d_ecs::Transform>(f).map(|l| l.translation));
    let filho = local.map(|l| no_mundo([l.x, l.y]));
    Some(format!(
        "[sprite-prova] t={t:.2} desfazer={desfazer} posicao=[{},{}] anchor={:?} desenho={desenho:?} colisao={col:?} filho={filho:?} chaves=[{}]",
        p.x,
        p.y,
        s.map(|s| s.anchor),
        chaves.join(" "),
    ))
}

/// Um quadro da foto `camera_desfaz`, pela ordem em que a janela o executa: relatar o formato, contar o quadro
/// como entrada (a fila só fecha passos com entrada), trocar o formato a Game Boy, desfazer/refazer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PassoDoFormato {
    pub relata: Option<&'static str>,
    pub entrada: bool,
    pub troca: bool,
    /// `Some(refaz)`: Ctrl+Z (`false`) ou Ctrl+Y (`true`).
    pub desfaz: Option<bool>,
}

/// O roteiro da foto `camera_desfaz` (onda 18): no quadro 40 o formato vai a Game Boy pela porta do Inspector
/// (com a entrada que um clique dá), e depois Ctrl+Z e Ctrl+Y — `[camera-desfaz]` diz o formato a cada passo.
#[must_use]
pub fn passo_do_formato(quadro: u32) -> PassoDoFormato {
    let p = PassoDoFormato::default();
    match quadro {
        40 => PassoDoFormato {
            relata: Some("antes"),
            entrada: true,
            troca: true,
            ..p
        },
        41..=43 => PassoDoFormato { entrada: true, ..p },
        50 => PassoDoFormato {
            relata: Some("trocado"),
            desfaz: Some(false),
            ..p
        },
        60 => PassoDoFormato {
            relata: Some("depois do Ctrl+Z"),
            desfaz: Some(true),
            ..p
        },
        70 => PassoDoFormato {
            relata: Some("depois do Ctrl+Y"),
            ..p
        },
        _ => p,
    }
}

/// A edição do Inspector que põe a «Camera» da cena no formato Game Boy.
#[must_use]
pub fn troca_para_game_boy(sim: &mut SimWorld) -> Option<EditorAction> {
    use ph2d_editor_core::{CameraFieldEdit, GamePreset};
    let cam = por_nome(sim, "Camera")?;
    let i = GamePreset::ALL
        .iter()
        .position(|p| *p == GamePreset::GameBoy)
        .unwrap_or(0);
    Some(EditorAction::InspectorCameraEdit {
        entity_bits: cam.to_bits(),
        edit: CameraFieldEdit::Format(i as u8),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O roteiro do `camera_desfaz`: troca a 40, entrada até 43, Ctrl+Z a 50, Ctrl+Y a 60, relato a 70.
    #[test]
    fn o_formato_troca_desfaz_e_refaz_por_esta_ordem() {
        let q: Vec<(u32, PassoDoFormato)> = (0..100)
            .map(|q| (q, passo_do_formato(q)))
            .filter(|(_, p)| *p != PassoDoFormato::default())
            .collect();
        let quadros: Vec<u32> = q.iter().map(|(n, _)| *n).collect();
        assert_eq!(quadros, [40, 41, 42, 43, 50, 60, 70]);
        assert!(q[0].1.troca && q[0].1.entrada && q[0].1.relata == Some("antes"));
        assert_eq!(q[4].1.desfaz, Some(false));
        assert_eq!(q[5].1.desfaz, Some(true));
        assert_eq!(q.iter().filter(|(_, p)| p.troca).count(), 1);
    }
}
