//! **B20.10 — a imagem das 8 telhas do Repeat Image, guardada.** Antes cada telha era um
//! `draw_image_rgba_transformed` = `Blob::new` = um id novo no Vello, logo 8 subidas da composição
//! por quadro (8 × 33 MB a 4K, parado incluído). Agora as 8 telhas desenham UM [`StableImage`],
//! guardado pela identidade do `Arc` do espelho da shell: parado, o Vello acerta o cache e não sobe
//! nada; o conteúdo muda ⇒ outro `Arc` (ou [`larga`] antes do remendo no lugar) ⇒ uma subida.

use std::cell::RefCell;
use std::sync::Arc;

use ph2d_vector::StableImage;

struct Guardada {
    rgba: Arc<Vec<u8>>,
    largura: u32,
    altura: u32,
    imagem: StableImage,
}

thread_local! {
    static GUARDADA: RefCell<Option<Guardada>> = const { RefCell::new(None) };
}

/// A imagem estável de `rgba` (`largura × altura`, RGBA reta). O mesmo `Arc` dá o MESMO id; segurar
/// o `Arc` garante que o endereço não é reusado por outro conteúdo (o `make_mut` de um terceiro copia).
pub(crate) fn imagem(rgba: &Arc<Vec<u8>>, largura: u32, altura: u32) -> Option<StableImage> {
    GUARDADA.with(|g| {
        let mut g = g.borrow_mut();
        if let Some(x) = g.as_ref()
            && Arc::ptr_eq(&x.rgba, rgba)
            && (x.largura, x.altura) == (largura, altura)
        {
            return Some(x.imagem.clone());
        }
        let imagem = StableImage::from_rgba(Arc::clone(rgba), largura, altura)?;
        *g = Some(Guardada {
            rgba: Arc::clone(rgba),
            largura,
            altura,
            imagem: imagem.clone(),
        });
        Some(imagem)
    })
}

/// Solta a imagem guardada: chamado antes do remendo no lugar do espelho
/// (`own_preview_buffer`), para a shell voltar a ser a dona única e o `make_mut` não copiar 33 MB.
pub(crate) fn larga() {
    GUARDADA.with(|g| g.borrow_mut().take());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::{Affine, ImageQuality, VectorScene};

    fn ids(desenha: impl Fn(&mut VectorScene)) -> Vec<u64> {
        let mut s = VectorScene::new();
        desenha(&mut s);
        s.probe_image_ids()
    }

    #[test]
    fn as_telhas_do_repeat_sao_uma_imagem_estavel_e_o_remendo_fica_no_lugar() {
        larga();
        let (w, h) = (4u32, 3u32);
        let mut rgba = Arc::new(vec![7u8; (w * h * 4) as usize]);
        let oito = |s: &mut VectorScene, rgba: &Arc<Vec<u8>>| {
            let im = imagem(rgba, w, h).expect("dims");
            for k in 0..8 {
                s.draw_stable_image_transformed(
                    &im,
                    Affine::translate((f64::from(k) * 4.0, 0.0)),
                    ImageQuality::Low,
                );
            }
        };
        let q1 = ids(|s| oito(s, &rgba));
        let q2 = ids(|s| oito(s, &rgba));
        assert_eq!(q1.len(), 8);
        assert!(q1.iter().all(|&i| i == q1[0]), "8 telhas, um id: {q1:?}");
        assert_eq!(
            q1, q2,
            "parado: o mesmo id no quadro seguinte (o Vello não sobe)"
        );
        // Controlo: o caminho de antes dá um id novo por telha.
        let antes = ids(|s| {
            for _ in 0..8 {
                s.draw_image_rgba_transformed(&rgba, w, h, Affine::IDENTITY, ImageQuality::Low);
            }
        });
        let mut u = antes.clone();
        u.sort_unstable();
        u.dedup();
        assert_eq!(u.len(), 8, "controlo: 8 ids no caminho antigo");
        // O remendo no lugar: largar ⇒ dono único ⇒ `make_mut` escreve no mesmo endereço.
        larga();
        let p = Arc::as_ptr(&rgba);
        Arc::make_mut(&mut rgba)[0] = 9;
        assert_eq!(Arc::as_ptr(&rgba), p, "largada, o remendo não copia");
        let q3 = ids(|s| oito(s, &rgba));
        assert_ne!(q3[0], q1[0], "conteúdo novo ⇒ id novo");
        // Sem largar, o `make_mut` copia ⇒ outro endereço ⇒ outro id: nunca a imagem velha.
        let p = Arc::as_ptr(&rgba);
        Arc::make_mut(&mut rgba)[0] = 11;
        assert_ne!(Arc::as_ptr(&rgba), p);
        let q4 = ids(|s| oito(s, &rgba));
        assert_ne!(q4[0], q3[0]);
        larga();
    }
}
