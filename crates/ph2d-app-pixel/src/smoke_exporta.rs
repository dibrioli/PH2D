//! **A cena 20 — Export e Import** (P9b). Os 16 quadros e as 4 tags da cena 8. Para as fotos,
//! `PH2D_PIXEL_SMOKE_GESTO`: `as` · `folha` · `import` abrem a janela (os eventos sintéticos não chegam
//! à tela virtual); o `import` lê a folha `PH2D_PIXEL_SMOKE_FOLHA`.

use ph2d_editor_core::pixel_panel::{PixelFileAction as A, PixelRequest as R};

use crate::PixelApp;

/// Uma vez, com o sprite em Draw.
pub(crate) fn conduz(app: &mut PixelApp) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if crate::smoke::armed_level() != Some(20)
        || app.chave.is_none()
        || FEITO.swap(true, Ordering::Relaxed)
    {
        return;
    }
    match std::env::var("PH2D_PIXEL_SMOKE_GESTO").as_deref() {
        Ok("as") => app.pede(R::File(A::ExportAs)),
        Ok("folha") => app.pede(R::File(A::ExportSheet)),
        Ok("import") => {
            if let Ok(c) = std::env::var("PH2D_PIXEL_SMOKE_FOLHA") {
                ph2d_app_host::modal::responde_o_proximo(c.into());
                app.pede(R::File(A::ImportSheet));
            }
        }
        _ => {}
    }
}
