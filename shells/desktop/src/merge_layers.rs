//! **FUNDIR EM CAMADAS** — instala no Painter o documento que a fusão produziu. Plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md) W10.
//!
//! > Enio, 2026-08-21: *"No menu do botão direito do mouse: Merge Sprites em camadas (cria uma
//! > camada por sprite)."*
//!
//! # O que este verbo é, ao lado do vizinho
//!
//! | | **Merge Sprites** | **Merge to Layers** |
//! |---|---|---|
//! | pixels | uma imagem, achatada | uma imagem achatada **e** um documento com N camadas |
//! | as fontes | despawnadas | despawnadas |
//! | dá para separar depois | **não** | **sim** — abra o Painter |
//!
//! ⚠️ **A geometria é EXACTAMENTE a mesma**, e é de propósito: a mesma união, o mesmo warp, o mesmo
//! «over». Duplicar aquele laço para o modo novo seria pedir que duas cópias concordassem para
//! sempre — em vez disso o [`crate::hero_intents::sprite_merge`] ganhou um **modo** que, além de
//! compor, guarda o que cada fonte pôs em cada pixel. *O que o Painter mostra é o que o ecrã já
//! mostra, porque é a mesma conta.*
//!
//! # Por que a sprite continua a ter a textura achatada
//!
//! ⚠️ **A sprite tem de desenhar e de GRAVAR sem o Painter.** O documento vive na ferramenta; o
//! ficheiro do projeto guarda os pixels da sprite. Se a única cópia fosse o documento, fechar o
//! app antes de abrir o Painter perdia a fusão — o mesmo defeito das ilhas do BG-Removal
//! (2026-08-20), que abriam perfeitas e gravavam vazias.
//!
//! Então há duas coisas, e as duas são verdade ao mesmo tempo: a **textura** (achatada, durável) e
//! o **documento** (em camadas, na ferramenta). Abrir o Painter mostra o segundo; tudo o resto vê
//! o primeiro.
//!
//! # A ordem das camadas
//!
//! ⚠️ **De baixo para cima, na ordem em que o «over» as compôs.** A fusão itera as fontes e põe
//! cada uma *sobre* o acumulado, por isso a última fica em cima; o [`PainterTool::add_raster_layer_with_pixels`]
//! insere no topo, por isso adicioná-las por essa mesma ordem reproduz a pilha. Trocar a ordem aqui
//! daria um documento que compõe diferente do que a sprite mostra — e ninguém saberia qual dos dois
//! está certo.

use ph2d_editor::{Toast, ToastQueue, ToolRegistry};
use ph2d_tool_painter::PainterTool;

use crate::hero_intents::MergedLayers;

/// Instala o documento em camadas na ferramenta de pintura.
///
/// ⚠️ **Não activa o Painter.** Fundir não é entrar em modo de pintura — o artista pediu uma fusão,
/// e trocar-lhe a ferramenta da mão seria decidir por ele. O documento fica **à espera**: o
/// `bind_document` guarda-o na cache por sprite, e abrir o Painter nessa sprite devolve-o.
pub(crate) fn install(tools: &mut ToolRegistry, doc: &MergedLayers, toasts: &mut ToastQueue) {
    let Some(painter) = tools
        .tool_by_id_mut(&ph2d_editor::ToolId::new("painter"))
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
    else {
        // ⚠️ Diz-se, e não se cala: a sprite fundida existe e desenha certo, mas a metade que o
        // artista pediu — as camadas — não aconteceu. Um verbo que faz metade em silêncio é pior
        // que um que falha.
        toasts.push(Toast::error(
            "Merged, but the layered document could not be created (painter unavailable)",
        ));
        return;
    };
    let Some(((_, bottom), rest)) = doc.layers.split_first() else {
        return;
    };
    // A base entra pelo `bind_document`, que é a porta por onde um documento nasce.
    painter.bind_document(doc.entity_bits, bottom.clone(), doc.width, doc.height);
    // ⚠️ O nome da camada de baixo tem de ser reposto: o `set_source` chama-lhe «Layer 1», e uma
    // pilha em que só a primeira não diz de que sprite veio é a pior das duas hipóteses.
    if let Some(active) = painter.layers().active() {
        painter.set_layer_name(active, doc.layers[0].0.clone());
    }
    let mut made = 1usize;
    for (name, rgba) in rest {
        if painter
            .add_raster_layer_with_pixels(name.clone(), rgba.clone())
            .is_some()
        {
            made += 1;
        }
    }
    if made < doc.layers.len() {
        // O tecto de camadas do Painter, ou um tamanho que não bate. As duas coisas são reais e as
        // duas têm de ser ditas com o NÚMERO — «algumas falharam» não deixa ninguém decidir nada.
        toasts.push(Toast::info(format!(
            "Merged into {made} of {} layers — the rest did not fit",
            doc.layers.len()
        )));
    } else {
        toasts.push(Toast::success(format!(
            "Merged into {made} layers — open the Painter on it to separate them again"
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(layers: Vec<(&str, Vec<u8>)>, w: u32, h: u32) -> MergedLayers {
        MergedLayers {
            entity_bits: 1,
            width: w,
            height: h,
            layers: layers
                .into_iter()
                .map(|(n, p)| (n.to_string(), p))
                .collect(),
        }
    }

    /// **A pilha reproduz o que a sprite mostra**, e a ordem é a que decide isso.
    ///
    /// ⚠️ Duas camadas opacas: a de CIMA tem de ganhar. Se a ordem se inverter, o documento compõe
    /// o oposto da textura achatada que a mesma fusão produziu — e aí há duas verdades na cena,
    /// sem nada a dizer qual delas está certa.
    #[test]
    fn the_stack_is_bottom_first_so_it_composites_like_the_flattened_texture() {
        let mut painter = PainterTool::default();
        let red = vec![255u8, 0, 0, 255];
        let green = vec![0u8, 255, 0, 255];
        let d = doc(vec![("baixo", red), ("cima", green)], 1, 1);
        let mut toasts = ToastQueue::default();
        // O registry real não existe num teste de unidade; exercita-se a instalação directa.
        painter.bind_document(d.entity_bits, d.layers[0].1.clone(), d.width, d.height);
        if let Some(active) = painter.layers().active() {
            painter.set_layer_name(active, d.layers[0].0.clone());
        }
        for (name, rgba) in &d.layers[1..] {
            painter
                .add_raster_layer_with_pixels(name.clone(), rgba.clone())
                .expect("a camada tem de nascer — o tamanho bate");
        }
        let _ = &mut toasts;

        let names: Vec<String> = painter
            .layers()
            .root()
            .iter()
            .filter_map(|id| painter.layers().get(*id).map(|l| l.name.clone()))
            .collect();
        // `root` é do topo para baixo.
        assert_eq!(
            names,
            vec!["cima".to_string(), "baixo".to_string()],
            "a pilha saiu na ordem errada — o documento comporia o oposto da textura"
        );
    }

    /// ⚠️ **Uma camada do tamanho errado é RECUSADA, não composta.** Um documento de 512² com uma
    /// camada de 64² não é um erro a corrigir adiante: é a chamada errada, e compor lixo esconde-a.
    #[test]
    fn a_layer_of_the_wrong_size_is_refused() {
        let mut painter = PainterTool::default();
        painter.bind_document(1, vec![0u8; 4], 1, 1);
        assert!(
            painter
                .add_raster_layer_with_pixels("errada", vec![0u8; 16])
                .is_none(),
            "uma camada de tamanho errado entrou na pilha"
        );
    }

    /// **O nome de cada camada é o da sprite de origem** — é a razão de este verbo existir.
    #[test]
    fn each_layer_carries_the_name_of_the_sprite_it_came_from() {
        let mut painter = PainterTool::default();
        painter.bind_document(1, vec![0u8; 4], 1, 1);
        if let Some(active) = painter.layers().active() {
            painter.set_layer_name(active, "Heroi");
        }
        painter
            .add_raster_layer_with_pixels("Escudo", vec![0u8; 4])
            .expect("nasce");
        let names: Vec<String> = painter
            .layers()
            .root()
            .iter()
            .filter_map(|id| painter.layers().get(*id).map(|l| l.name.clone()))
            .collect();
        assert!(
            names.contains(&"Heroi".to_string()) && names.contains(&"Escudo".to_string()),
            "as camadas saíram como {names:?} — uma pilha de «Layer N» nao diz qual sprite e' qual, \
             que e' precisamente o que este verbo existe para mostrar"
        );
    }
}
