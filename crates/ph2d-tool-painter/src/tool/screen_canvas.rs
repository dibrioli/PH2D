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

    /// ⭐⭐ **A tela começa com o RETRATO da peça** — o pen-down dos modos que
    /// lêem a cor debaixo do pincel ([`Self::screen_canvas_reads_the_piece`]).
    /// `rgba` tem de ter o tamanho da tela presa; devolve se a semeou.
    ///
    /// ⚠️ Pelo `set_source` e não pelo `bind_document`: o documento já é a
    /// tela da vista, e o que muda é só o conteúdo dela.
    pub fn seed_screen_canvas(
        &mut self,
        rgba: Vec<u8>, // COLOR-RAW-OK: the portrait's bytes forwarded verbatim to the `set_source` trait contract (also `Vec<u8>`), like `bind_document`
    ) -> bool {
        let (w, h) = self.source_size;
        if !self.on_screen_canvas() || rgba.len() != (w as usize) * (h as usize) * 4 {
            return false;
        }
        self.set_source(rgba, w, h);
        true
    }

    /// ⭐⭐ **A tinta molhada muda de VISTA** — o pen-down depois de rodar a
    /// peça (report do dono, 29/09: *«rotacionar e pintar em seguida está
    /// pausando a simulação»*). Semear a tela com o retrato da vista nova
    /// matava a sessão da água; isto refaz a sessão na vista nova e a água
    /// CONTINUA a correr.
    ///
    /// `semente` é o retrato na vista nova; `origem`, por píxel da tela nova,
    /// o ponto na tela de ANTES que vê o mesmo sítio da superfície (`None`
    /// onde a vista de antes não o via). Devolve `false` — e nada muda — sem
    /// sessão da água viva: quem chama semeia como antes.
    ///
    /// ⚠️ **Só a tinta molhada (`Wet Paint`).** A humidade da aquarela vive
    /// noutra sessão, e rodar a vista continua a secá-la (declarado na
    /// costura da escultura).
    pub fn reproject_screen_canvas(
        &mut self,
        semente: &[u8], // COLOR-RAW-OK: the new view's portrait bytes, same contract as `seed_screen_canvas`
        origem: &[Option<[f32; 2]>],
    ) -> bool {
        self.on_screen_canvas() && self.wetpaint_reproject(semente, origem)
    }

    /// ⭐ **O raio do anel do cursor sobre a peça**, em píxeis da tela — o que
    /// o gesto vai de facto mexer.
    ///
    /// ⚠️ O `Deform` (o Liquify) tem um tamanho PRÓPRIO (`deform_size_px`),
    /// independente do pincel de pintura; lido do [`Self::dab_footprint_px`] o
    /// anel ficava parado no tamanho do pincel enquanto o slider do Liquify
    /// mudava o que o gesto deforma (report do dono, 24/09). É a mesma escolha
    /// que o anel da vista 2D faz (`painter_bridge_brush_ring`).
    #[must_use]
    pub fn screen_canvas_ring_px(&self) -> f32 {
        if self.is_deform_mode() {
            self.brush_settings().deform_size_px
        } else {
            self.dab_footprint_px()
        }
    }

    /// ⭐⭐ **O papel da aquarela ainda está MOLHADO?** — o traço que começar
    /// agora continua a sessão molhada e funde com o anterior
    /// ([`Self::wet_session_continues`], a MESMA pergunta que o pen-down faz).
    /// O papel seca sozinho no batimento (`dry_canvas_wet`).
    ///
    /// ⚠️ Ele só sobrevive enquanto ninguém chamar o `set_source`: limpar ou
    /// semear a tela SECA o papel (`reset_transient_edit_state` →
    /// `dry_session_now`). É por isso que a escultura pergunta isto no fim de
    /// um traço antes de decidir se limpa a tela, e no pen-down antes de a
    /// semear.
    ///
    /// ⭐ **E a tinta molhada (`Wet Paint`) também:** com a sessão da água viva a
    /// tela FICA pela mesma razão — semeá-la mataria a sessão (o guarda dela vê
    /// o `canvas_rgba` trocado), e o traço seguinte deixaria de se misturar com o
    /// que ainda está molhado.
    #[must_use]
    pub fn screen_canvas_is_wet(&self) -> bool {
        self.on_screen_canvas() && (self.wet_session_continues() || self.wet_paint_session_alive())
    }

    /// ⭐⭐ **A tinta molhada ainda ESCORRE na tela?** — a pincelada acabou de
    /// mexer só quando isto fica falso (report do dono, etapa 3: *«a tinta
    /// molhada escorre depois de largar»*). Enquanto for verdade, a escultura
    /// mantém o traço aberto e continua a pousar o que a água muda.
    #[must_use]
    pub fn screen_canvas_is_flowing(&self) -> bool {
        self.on_screen_canvas() && self.wet_paint_flowing()
    }

    /// **Solta a tela** — a escultura saiu do ecrã, e a ponte volta a poder
    /// ligar a sprite escolhida no quadro seguinte.
    pub fn release_screen_canvas(&mut self) {
        if self.bound_doc == Some(SCREEN_CANVAS_DOC) {
            self.forget_piece_layers();
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
        // ⭐ **A espessura vai no MESMO quadro que a cor** (`docs/3D/29`) — uma
        //   drenagem, uma verdade: quem pousa lê as duas do mesmo instante.
        //   ⚠️ A janela é o rectângulo com `MARGEM_DO_RELEVO` à volta: a
        //   pousada aceita um píxel fora do rectângulo e amostra bilinear, logo
        //   lê até dois píxeis além dele.
        let window = rect.map_or((0, 0, w, h), |(x, y, rw, rh)| {
            let x0 = x.saturating_sub(MARGEM_DO_RELEVO);
            let y0 = y.saturating_sub(MARGEM_DO_RELEVO);
            let x1 = (x + rw + MARGEM_DO_RELEVO).min(w);
            let y1 = (y + rh + MARGEM_DO_RELEVO).min(h);
            (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
        });
        let relief = self.screen_canvas_relief_in(window);
        Some(ScreenCanvasFrame {
            rgba,
            w,
            h,
            rect,
            relief,
        })
    }

    /// ⭐⭐ **A ESPESSURA da tela numa janela, em píxeis** — o relevo do
    /// impasto que a escultura pousa na peça (`docs/3D/29`). A cor que a
    /// [`Self::take_screen_canvas`] entrega sai SEM a luz 2D (ver o
    /// `impasto_visible`); a espessura sai daqui, crua.
    ///
    /// `None` fora da tela da vista ou quando nada nela tem relevo.
    #[must_use]
    pub fn screen_canvas_heights_in(&self, janela: (u32, u32, u32, u32)) -> Option<Vec<f32>> {
        if !self.on_screen_canvas() {
            return None;
        }
        let id = self.layers.active()?;
        self.layer_height_px_in(id, janela)
    }

    /// ⭐⭐ **O RELEVO da tela numa janela — a espessura E o corpo** (`docs/3D/29`
    /// §6): a janela que a drenagem leva, e a que a peça lê de volta depois de
    /// semear a tela ([`Self::seed_screen_canvas_relief`]), para a semente dela
    /// ser os MESMOS números que a tela tem.
    ///
    /// ⚠️ O corpo sai a ZERO onde a camada tem espessura sem cobertura: é o que
    /// o passe de luz 2D vê ali (luz nenhuma), e é o que a peça tem de ver.
    #[must_use]
    pub fn screen_canvas_relief_in(
        &self,
        window: (u32, u32, u32, u32),
    ) -> Option<ScreenCanvasRelief> {
        let px = self.screen_canvas_heights_in(window)?;
        let id = self.layers.active()?;
        let cover = self
            .layer_cover_in(id, window)
            .unwrap_or_else(|| vec![0.0; px.len()]);
        Some(ScreenCanvasRelief { px, cover, window })
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
    /// ⭐ A espessura do impasto à volta do `rect` (`docs/3D/29`); `None` sem
    /// relevo nenhum na tela.
    pub relief: Option<ScreenCanvasRelief>,
}

/// A espessura da tela numa janela — em PÍXEIS, linha a linha.
pub struct ScreenCanvasRelief {
    /// `window.2 × window.3` alturas.
    pub px: Vec<f32>,
    /// `window.2 × window.3` coberturas, `0..1` — QUANTA tinta está em cada
    /// píxel (o corpo do passe de luz 2D).
    pub cover: Vec<f32>,
    /// `(x, y, largura, altura)` da janela na tela.
    pub window: (u32, u32, u32, u32),
}

/// Quantos píxeis a janela da espessura estende o rectângulo mudado — um da
/// folga da pousada, mais um do vizinho bilinear.
const MARGEM_DO_RELEVO: u32 = 2;

fn transparente(w: u32, h: u32) -> Vec<u8> {
    vec![0; (w as usize) * (h as usize) * 4]
}

#[cfg(test)]
#[path = "screen_canvas_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "screen_canvas_reproject_tests.rs"]
mod reproject_tests;
