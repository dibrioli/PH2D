//! ⭐⭐ **A MINIATURA de um cartão, e o memo de textura que ela obriga** (wave A6) — irmão por
//! ASSUNTO do [`super::paint`], que ficou a **uma linha** do tecto de 600 LOC dos painéis.
//!
//! ⛔⛔ **O memo não é uma optimização — sem ele o navegador reenvia CADA cartão ao atlas do
//! `vello`, TODO o quadro.** O `draw_image_rgba` faz `Blob::new(rgba.clone())` em cada chamada e o
//! `vello` indexa o cache por `data.id()`; com o tecto de 512 células a 96² isso é ~18 MB de
//! upload + repack por quadro. O `StableImage` guarda o id, e é o que faz o cache dele acertar.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;

thread_local! {
    /// `AssetRef → (os bytes que a construíram, a textura estável)`.
    ///
    /// ⚠️ **A chave da revalidação é a IDENTIDADE do `Arc`**, não os bytes: o `Thumb` compara em
    /// `O(1)` por `ptr_eq`, e quem produz uma miniatura nova produz um `Arc` novo (a junção
    /// garante-o guardando o `Arc` na memória por conteúdo). ⛔ Mutar um `Arc` existente em sítio
    /// manteria o id da `Blob` e o atlas serviria os pixels velhos, **sem erro nenhum**.
    static THUMB_TEX: std::cell::RefCell<
        std::collections::BTreeMap<ph2d_asset_index::AssetRef, (ph2d_asset_index::Thumb, ph2d_vector::StableImage)>,
    > = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// ⭐⭐ **O memo guarda o que o QUADRO pintou, e mais nada** (auditoria de 2026-08-30, achado nº 8).
///
/// ⚠️ **Sem isto ele cresce para sempre**: nada o purgava — nem *Remove from Library*, nem um
/// `Open Project`. A conta é `96·96·4 = 36 864 B` por asset alguma vez miniaturizado, e o `N` não
/// tinha tecto. Agora o tecto é o do próprio painel (`MAX_ASSET_CELLS = 512` ⇒ **≤ 18,9 MB**), e
/// ele é **derivado**, não escolhido.
///
/// ⚠️ Chamado no fim do `paint_grid`, com os endereços que a grade de facto desenhou.
pub(crate) fn retain_painted(keys: &[ph2d_asset_index::AssetRef]) {
    THUMB_TEX.with(|c| c.borrow_mut().retain(|k, _| keys.contains(k)));
}

/// Desenha a miniatura dentro do quadrado do cartão, **aspecto preservado e centrada**.
///
/// ⚠️ **Nunca esticada:** uma tira 8:1 esticada num quadrado lê-se como outra forma, e a miniatura
/// existe precisamente para se reconhecer a forma.
pub(crate) fn paint_thumb(
    ctx: &mut PaintCtx,
    key: ph2d_asset_index::AssetRef,
    thumb: &ph2d_asset_index::Thumb,
    square: Rect,
    inset: f32,
) {
    let img = THUMB_TEX.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((cached, img)) = c.get(&key)
            && cached == thumb
        {
            return Some(img.clone());
        }
        let img = ph2d_vector::StableImage::from_rgba(thumb.rgba.clone(), thumb.w, thumb.h)?;
        c.insert(key, (thumb.clone(), img.clone()));
        Some(img)
    });
    let Some(img) = img else { return };

    let (bw, bh) = (
        (square.w - inset * 2.0).max(0.0),
        (square.h - inset * 2.0).max(0.0),
    );
    let (tw, th) = (thumb.w.max(1) as f32, thumb.h.max(1) as f32);
    let s = (bw / tw).min(bh / th).max(0.0);
    let (dw, dh) = (tw * s, th * s);
    let x0 = f64::from(square.x + inset + (bw - dw) * 0.5);
    let y0 = f64::from(square.y + inset + (bh - dh) * 0.5);
    ctx.scene.draw_stable_image(
        &img,
        (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
        // Bilinear: uma miniatura é um render encolhido, e suavizar entre texels lê-se melhor que
        // o `Nearest` que uma MEDIÇÃO (um espectrograma) quereria.
        ph2d_vector::ImageQuality::Medium,
    );
}
