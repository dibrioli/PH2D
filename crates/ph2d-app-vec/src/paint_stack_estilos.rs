//! ⭐ **Os estilos de cor (D28/D30) do lado da cena**: os pedidos da lista do cartão
//! (`ph2d_editor_core::panel::cards::estilos`) aplicam-se à `VecScene` — que os grava e os leva no
//! Desfazer — e a vista volta para a lista. Corre no quadro, antes do seletor de cor.

use ph2d_ecs::ItemId;
use ph2d_editor_core::panel::cards::estilos::{self as lista, Estilo, Pedido};
use ph2d_vec_scene::{PaintEntry, Rgba8, VecPathId, VecScene};

/// Os usos `(forma, entrada)` da camada `camada` nas formas escolhidas (entradas sem id ficam de fora).
fn usos(scene: &VecScene, sel: &[VecPathId], camada: usize) -> Vec<(VecPathId, ItemId)> {
    sel.iter()
        .filter_map(|id| {
            let e = scene.path(*id)?.paints.get(camada)?;
            (e.id != ItemId::NONE).then_some((*id, e.id))
        })
        .collect()
}

/// Aplica os pedidos da lista e publica a vista. Devolve se o DOCUMENTO mudou.
pub fn drena(
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    scene: &mut VecScene,
    sel: &[VecPathId],
) -> bool {
    let mut mudou = false;
    for p in lista::take_pedidos() {
        mudou |= match p {
            Pedido::Aplicar { estilo, camada } => usos(scene, sel, camada)
                .into_iter()
                .fold(false, |m, u| scene.link_estilo(u, estilo) | m),
            Pedido::Novo { camada, nome } => {
                let Some(cor) = sel
                    .first()
                    .and_then(|id| scene.path(*id))
                    .and_then(|p| p.paints.get(camada))
                    .map(PaintEntry::swatch_color)
                else {
                    continue;
                };
                let e = scene.add_estilo(&nome, cor);
                for u in usos(scene, sel, camada) {
                    scene.link_estilo(u, e);
                }
                true
            }
            Pedido::Renomear { estilo, nome } => scene.rename_estilo(estilo, &nome),
            Pedido::Duplicar { estilo, nome } => scene.duplicate_estilo(estilo, &nome).is_some(),
            Pedido::Apagar { estilo } => scene.remove_estilo(estilo),
            Pedido::Cor { estilo, rgba } => {
                scene.set_estilo_color(estilo, Rgba8::new(rgba[0], rgba[1], rgba[2], rgba[3]))
            }
            Pedido::Soltar { camada } => usos(scene, sel, camada)
                .into_iter()
                .fold(false, |m, u| scene.unlink_estilo(u) | m),
        };
    }
    // O seletor aberto sobre uma camada que o estilo acabou de pintar segue a cor nova — senão o
    // próximo quadro devolvia-lhe a antiga (e soltava-a).
    if mudou
        && let Some(i) = super::layer_of_picker_target(store)
        && let Some(c) = sel
            .first()
            .and_then(|id| scene.path(*id))
            .and_then(|p| p.paints.get(i))
            .map(PaintEntry::swatch_color)
    {
        store.set_blender_value(
            ph2d_editor_core::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(c.r, c.g, c.b, c.a),
        );
    }
    publica(scene, sel);
    mudou
}

fn publica(scene: &VecScene, sel: &[VecPathId]) {
    let estilos = scene
        .estilos()
        .iter()
        .map(|e| Estilo {
            id: e.id,
            name: e.name.clone(),
            rgba: [e.color.r, e.color.g, e.color.b, e.color.a],
            usos: scene.usos_vivos(e.id),
        })
        .collect();
    let ligados = sel
        .first()
        .and_then(|id| scene.path(*id).map(|p| (*id, p)))
        .map(|(id, p)| {
            p.paints
                .iter()
                .map(|e| scene.estilo_de((id, e.id)).map(|s| s.id))
                .collect()
        })
        .unwrap_or_default();
    lista::publica(estilos, ligados);
}

#[cfg(test)]
#[path = "paint_stack_estilos_tests.rs"]
mod tests;
