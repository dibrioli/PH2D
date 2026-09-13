//! ⛔⛔⛔ **O QUADRO SEM RECURSO TARDIO APAGA O ATLAS DO VELLO** — e esta é a porta que o impede.
//!
//! # O defeito (vello 0.10, medido 2026-09-13 na GPU)
//!
//! Uma imagem desenhada por um id ESTÁVEL (`ph2d_vector::StableImage`) é enviada ao atlas UMA vez e
//! fica residente. Se um quadro chegar ao Vello **sem nenhum recurso tardio** — nenhuma imagem,
//! gradiente ou texto —, o `Resolver::resolve` sai por `resolve_solid_paths_only` e devolve um atlas
//! de largura `0`; o `render.rs` troca então a textura do atlas por uma de `1×1` (`Resized`) e, no
//! quadro seguinte, por uma NOVA em branco. O `ImageCache` da CPU continua a dar a imagem por
//! enviada, não a reenvia, e ela **nunca mais aparece**:
//!
//! | sequência (`VelloPass`, alfa no centro) | resultado |
//! |---|---|
//! | imagem → imagem | `255 → 255` |
//! | imagem → **vazio** → imagem → imagem | `255 → 0 → 0 → 0` |
//! | imagem → imagem CRUA → imagem | `255 → 255 → 255` |
//!
//! ⚠️ **Pela porta crua isto nunca se via:** um id novo por quadro é reenviado por quadro. O defeito
//! ficou alcançável quando a pele de imagem do esqueleto passou a desenhar pela porta estável (a
//! cura do atlas cheio), e ele vale para **todo** utilizador de `StableImage` da casa.
//!
//! # A cura, e porque é AQUI
//!
//! O `VelloPass` é a única porta do produto que entrega uma cena ao `vello::Renderer`. Quando a cena
//! chega sem patch nenhum, ela é composta com **um desenho de uma imagem de `1×1` fora do alvo** — um
//! id fixo, logo nenhum reenvio, e cobertura zero, logo nenhum pixel mexe.
//!
//! ⚠️ **O preço é uma cópia da cena, e SÓ nos quadros sem recurso tardio.** No editor não há
//! nenhum (todo quadro tem texto); um runtime sem texto nem imagens pagaria a cópia de uma cena só
//! de caminhos sólidos.

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

/// A imagem que mantém o atlas, e a cena onde a composição acontece.
pub(crate) struct KeepAlive {
    image: ImageData,
    scene: Scene,
}

impl KeepAlive {
    pub(crate) fn new() -> Self {
        Self {
            image: ImageData {
                data: Blob::new(std::sync::Arc::new(vec![0_u8; 4])),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: 1,
                height: 1,
            },
            scene: Scene::new(),
        }
    }

    /// A cena a entregar ao Vello: a do chamador **sem cópia** quando ela já tem um recurso tardio,
    /// e a composição com a imagem de `1×1` fora do alvo quando não tem.
    pub(crate) fn scene_for_vello<'a>(&'a mut self, scene: &'a Scene) -> &'a Scene {
        if !scene.encoding().resources.patches.is_empty() {
            return scene;
        }
        self.scene.reset();
        self.scene.append(scene, None);
        let brush = ImageBrush::new(self.image.clone());
        self.scene
            .draw_image(brush.as_ref(), Affine::translate((-1.0e6, -1.0e6)));
        &self.scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vello::kurbo::Rect;
    use vello::peniko::{Color, Fill};

    /// ⭐ **A decisão, sem GPU:** uma cena sem recurso tardio sai com UM patch (o da imagem de
    /// `1×1`); uma que já tem sai **a mesma** (o ponteiro), sem cópia.
    ///
    /// ⚠️ A metade que prova a cura na tela é o gate de GPU
    /// `a_stable_image_survives_a_frame_without_late_bound_resources` (`tests/it`).
    #[test]
    fn a_scene_without_late_bound_resources_leaves_with_one_and_one_that_has_them_is_not_copied() {
        let mut keep = KeepAlive::new();

        let mut solida = Scene::new();
        solida.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(255, 0, 0, 255),
            None,
            &Rect::new(0.0, 0.0, 10.0, 10.0),
        );
        assert!(solida.encoding().resources.patches.is_empty(), "fixtura: tinha de ser sem patch");
        let n_caminhos = solida.encoding().n_paths;
        let entregue = keep.scene_for_vello(&solida);
        assert_eq!(
            entregue.encoding().resources.patches.len(),
            1,
            "a cena sem recurso tardio chegou ao Vello sem o patch que mantem o atlas"
        );
        assert_eq!(
            entregue.encoding().n_paths,
            n_caminhos + 1,
            "a composicao perdeu ou duplicou os caminhos do chamador"
        );

        let mut com_imagem = Scene::new();
        let img = ImageData {
            data: Blob::new(std::sync::Arc::new(vec![255_u8; 4])),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: 1,
            height: 1,
        };
        com_imagem.draw_image(ImageBrush::new(img).as_ref(), Affine::IDENTITY);
        let original: *const Scene = &com_imagem;
        let entregue: *const Scene = keep.scene_for_vello(&com_imagem);
        assert_eq!(
            original, entregue,
            "uma cena que ja tem recurso tardio foi copiada — o custo tinha de ser zero nela"
        );
    }

    /// ⛔⛔ **TODA entrega ao Vello passa pela porta — contadas no fonte do `VelloPass`.**
    ///
    /// ⚠️ O gate de GPU mede a cura pela `render_and_readback` → `render_to_intermediate`; a
    /// entrega do ECRÃ (`render`) não tem leitura de volta, e o defeito voltaria por ela sem nenhum
    /// gate o ver. ⇒ o número de `.render_to_texture(` tem de ser o de `scene_for_vello(`, e as duas
    /// contagens não podem ser zero.
    ///
    /// ⚠️ É um gate de TEXTO e responde «está escrito», nunca «corre» — a metade «corre» é a de GPU.
    #[test]
    fn every_hand_off_to_vello_goes_through_the_keepalive() {
        let fonte: String = include_str!("vello_pass.rs")
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let entregas = fonte.matches(".render_to_texture(").count();
        let portas = fonte.matches("keepalive.scene_for_vello(").count();
        assert!(entregas >= 2, "o censo nao achou as entregas ao Vello ({entregas}) — mudou de forma?");
        assert_eq!(
            entregas, portas,
            "{entregas} entregas ao Vello e so' {portas} passam pela porta que mantem o atlas — a \
             que ficou de fora apaga toda imagem estavel depois de um quadro sem recurso tardio"
        );
    }
}
