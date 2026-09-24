//! ⭐⭐⭐ **A TELA DA VISTA 3D** — o Painter a pintar sobre uma peça esculpida
//! (ordem do dono, 2026-09-24: *«a integração total do módulo Painter já
//! existente para que consiga pintar com os mesmos features na malha 3d»*).
//!
//! # O desenho
//!
//! O Painter não sabe o que é uma malha, e não precisa: a peça é desenhada no
//! ecrã, e o Painter pinta uma **imagem transparente do tamanho da vista** — o
//! mesmo motor, os mesmos pincéis, a mesma pressão, sobre um documento como
//! outro qualquer. Quem pousa essa imagem na peça é a família da escultura
//! (`ph2d_sculpt3d::tela_na_malha`), que a drena por [`PainterTool::take_screen_canvas`].
//!
//! # ⚠️ Porque é um DOCUMENTO e não um `set_source`
//!
//! A tela entra por [`PainterTool::bind_document`] com um id reservado
//! ([`SCREEN_CANVAS_DOC`]), e isso é o que protege a sprite que estava ligada:
//! o `bind_document` GUARDA as camadas do documento que sai (um `set_source`
//! achatá-las-ia) e devolve-as quando a sprite volta.
//!
//! # ⚠️ Enquanto a tela está presa, a PONTE DA SPRITE não a toca
//!
//! A ponte do Painter corre a cada quadro e faz três coisas que, sobre esta
//! tela, seriam defeitos: drena a pré-visualização para a subir à textura da
//! sprite escolhida (a escultura perderia o rectângulo sujo e a textura de uma
//! peça receberia a tela inteira), liga a sprite escolhida (a tela seria
//! trocada a meio de um traço) e corre o `Apply` (a tela seria assada numa
//! sprite). ⇒ as quatro portas que ela usa perguntam [`PainterTool::on_screen_canvas`]
//! — a guarda vive na FERRAMENTA, que é quem sabe o que tem ligado.

use super::*;

/// O id do documento que é a tela da vista 3D. `u64::MAX` não é um id de
/// entidade que o `bevy_ecs` produza para uma sprite (é o marcador de «nenhuma»),
/// logo não colide com nenhum documento guardado.
pub const SCREEN_CANVAS_DOC: u64 = u64::MAX;

impl PainterTool {
    /// **A tela da vista 3D está presa AGORA?** — ligada e com píxeis.
    #[must_use]
    pub fn on_screen_canvas(&self) -> bool {
        self.bound_doc == Some(SCREEN_CANVAS_DOC) && !self.canvas_rgba.is_empty()
    }

    /// **Prende a tela da vista**, transparente, com `largura × altura`.
    ///
    /// Idempotente enquanto o tamanho não muda; com a vista redimensionada a
    /// tela nasce de novo (a tinta que ela ainda não pousou já está na peça —
    /// quem chama pousa ANTES de redimensionar). Devolve se a tela nasceu agora.
    pub fn bind_screen_canvas(&mut self, largura: u32, altura: u32) -> bool {
        if largura == 0 || altura == 0 {
            return false;
        }
        if self.on_screen_canvas() {
            if self.source_size == (largura, altura) {
                return false;
            }
            self.set_source(transparente(largura, altura), largura, altura);
            return true;
        }
        self.bind_document(
            SCREEN_CANVAS_DOC,
            transparente(largura, altura),
            largura,
            altura,
        );
        true
    }

    /// **A tela volta a transparente** — o fim de um traço, depois de ele ter
    /// sido pousado na peça. Sem isto o traço seguinte compunha o anterior
    /// outra vez por cima dele.
    pub fn clear_screen_canvas(&mut self) {
        if self.on_screen_canvas() {
            let (w, h) = self.source_size;
            self.set_source(transparente(w, h), w, h);
        }
    }

    /// **Solta a tela** — a escultura saiu do ecrã, e a ponte volta a poder
    /// ligar a sprite escolhida no quadro seguinte.
    pub fn release_screen_canvas(&mut self) {
        if self.bound_doc == Some(SCREEN_CANVAS_DOC) {
            self.reset_transient_edit_state();
            self.replace_canvas(Arc::new(Vec::new()));
            self.bound_doc = None;
            self.preview_dirty = false;
        }
    }

    /// ⭐ **A drenagem da tela, para quem a pousa na peça** — a imagem e o
    /// rectângulo que mudou desde a última drenagem (`None` = a imagem toda).
    /// `None` inteiro quando não há tela presa ou nada mudou.
    pub fn take_screen_canvas(&mut self) -> Option<ScreenCanvasFrame> {
        if !self.on_screen_canvas() {
            return None;
        }
        let (rgba, w, h) = self.drain_preview_arc()?;
        let rect = self.take_preview_upload_bbox();
        Some(ScreenCanvasFrame { rgba, w, h, rect })
    }
}

/// O que uma drenagem da tela da vista entrega.
pub struct ScreenCanvasFrame {
    /// RGBA8 não pré-multiplicado, bytes sRGB.
    pub rgba: Arc<Vec<u8>>,
    /// Largura.
    pub w: u32,
    /// Altura.
    pub h: u32,
    /// `(x, y, largura, altura)` do que mudou; `None` = tudo.
    pub rect: Option<(u32, u32, u32, u32)>,
}

fn transparente(w: u32, h: u32) -> Vec<u8> {
    vec![0; (w as usize) * (h as usize) * 4]
}

#[cfg(test)]
#[path = "screen_canvas_tests.rs"]
mod tests;
