//! **Os pincéis guardados** (os espaços Alt+1, Alt+2… do Aseprite) — no COMPUTADOR, ao lado das
//! paletas do artista, valem em todo projecto (dono 02/11). Cada Ctrl+B cria um espaço
//! destrancado com *Brush Type* + *Image Color*; *Save Brush Here* cria um trancado com o pincel
//! actual inteiro; recuperar um espaço aplica só os parâmetros que ele guarda (Trial, §3 do handoff P7a).

use ph2d_editor_core::pixel_panel::{
    PixelBrushAction, PixelBrushShape, PixelBrushSlotInfo, PixelBrushSlotKind, PixelPatternTurn,
    PixelRequest,
};
use ph2d_pixel_core::pattern::PatternBrush;
use ph2d_pixel_io::brush_file::{self, BrushKind, BrushParams, BrushSlot};

use crate::PixelApp;
use ph2d_pixel_ponte::leis::{funde, miniatura};

pub use ph2d_pixel_ponte::pinceis::{FILE, pasta_do_utilizador};
use ph2d_pixel_ponte::pinceis::{le, shape_in, shape_out};

impl PixelApp {
    /// A lista (lida da pasta na 1.ª pergunta; um ficheiro estragado avisa e começa vazia, sem o
    /// apagar até à próxima gravação).
    pub fn espacos(&mut self) -> &[BrushSlot] {
        self.espacos_mut();
        self.espacos.as_deref().unwrap_or_default()
    }

    fn espacos_mut(&mut self) -> &mut Vec<BrushSlot> {
        if self.espacos.is_none() {
            let lida = self.pinceis.as_deref().map(le).unwrap_or(Ok(Vec::new()));
            let lista = lida.unwrap_or_else(|e| {
                self.avisos.push((
                    true,
                    ph2d_i18n::tr_with("pixel.brush.read_failed", &[("e", &e)]),
                ));
                Vec::new()
            });
            self.espacos = Some(lista);
        }
        self.espacos.get_or_insert_with(Vec::new)
    }

    fn grava_espacos(&mut self) {
        let Some(pasta) = self.pinceis.clone() else {
            return;
        };
        let texto = brush_file::write(self.espacos.as_deref().unwrap_or_default());
        let r =
            std::fs::create_dir_all(&pasta).and_then(|()| std::fs::write(pasta.join(FILE), texto));
        if let Err(e) = r {
            self.avisos.push((
                true,
                ph2d_i18n::tr_with("pixel.brush.save_failed", &[("e", &e.to_string())]),
            ));
        }
    }

    /// O espaço do pincel actual: as opções e o padrão (lido no sprite `key`).
    fn espaco_actual(&self, key: u32, params: BrushParams, locked: bool) -> Option<BrushSlot> {
        let o = &self.options;
        let kind = match &self.pincel {
            Some(p) => {
                let s = &self.docs.get(&key)?.sprite;
                BrushKind::Pattern {
                    size: p.size(),
                    cells: p.to_rgba(s),
                    origin: p.origin,
                }
            }
            None => BrushKind::Shape(shape_in(o.shape)),
        };
        Some(BrushSlot {
            locked,
            params,
            kind,
            size: o.size,
            angle: o.angle,
            image_color: self.pincel_imagem,
            ink: crate::gesture::ink(o.ink),
            fg: self.fg,
            bg: self.bg,
            pixel_perfect: o.pixel_perfect,
        })
    }

    /// O Ctrl+B acabou de pôr um padrão na mão: um espaço novo, destrancado.
    pub(crate) fn espaco_do_ctrl_b(&mut self, key: u32) {
        if let Some(s) = self.espaco_actual(key, BrushParams::NEW_BRUSH, false) {
            let l = self.espacos_mut();
            l.push(s);
            self.espaco_activo = Some(l.len() - 1);
            self.grava_espacos();
        }
    }

    /// **Save Brush Here**: o pincel actual inteiro num espaço TRANCADO.
    pub(crate) fn guarda_aqui(&mut self, key: u32) {
        if let Some(s) = self.espaco_actual(key, BrushParams::SAVE_HERE, true) {
            let n = {
                let l = self.espacos_mut();
                l.push(s);
                l.len()
            };
            self.espaco_activo = Some(n - 1);
            self.grava_espacos();
            self.avisos.push((
                false,
                ph2d_i18n::tr_with("pixel.brush.saved", &[("n", &n.to_string())]),
            ));
        }
    }

    /// **Alt+n**: recupera o espaço `n` (de 0) sobre o sprite `key` — só os parâmetros que ele
    /// guarda; o padrão volta com as cores da imagem.
    pub(crate) fn recupera(&mut self, key: u32, n: usize) -> bool {
        let Some(s) = self.espacos_mut().get(n).cloned() else {
            return false;
        };
        let p = s.params;
        self.espaco_activo = Some(n);
        if p.kind {
            match &s.kind {
                BrushKind::Shape(sh) => {
                    self.pincel = None;
                    self.options.shape = shape_out(*sh);
                }
                BrushKind::Pattern {
                    size,
                    cells,
                    origin,
                } => {
                    let Some(doc) = self.docs.get(&key) else {
                        return false;
                    };
                    let Some(b) = PatternBrush::from_rgba(&doc.sprite, *size, cells, *origin)
                    else {
                        return false;
                    };
                    self.pincel = Some(std::sync::Arc::new(b));
                }
            }
        }
        if p.size {
            self.options.size = s
                .size
                .clamp(1, ph2d_editor_core::pixel_panel::PIXEL_MAX_BRUSH);
        }
        if p.angle {
            self.options.angle = s.angle;
        }
        if p.image_color {
            self.pincel_imagem = s.image_color;
        }
        if p.ink {
            self.options.ink = crate::gesture::ink_out(s.ink);
        }
        if p.fg {
            self.fg = s.fg;
        }
        if p.bg {
            self.bg = s.bg;
        }
        if p.pixel_perfect {
            self.options.pixel_perfect = s.pixel_perfect;
        }
        true
    }

    /// Apaga o espaço `n` (de 0).
    pub(crate) fn apaga_espaco(&mut self, n: usize) {
        let l = self.espacos_mut();
        if n < l.len() {
            l.remove(n);
            self.espaco_activo = match self.espaco_activo {
                Some(a) if a == n => None,
                Some(a) if a > n => Some(a - 1),
                a => a,
            };
            self.grava_espacos();
        }
    }

    /// Tranca ou destranca o espaço `n`.
    pub(crate) fn tranca_espaco(&mut self, n: usize) {
        if let Some(s) = self.espacos_mut().get_mut(n) {
            s.locked = !s.locked;
            self.grava_espacos();
        }
    }
}

impl PixelApp {
    /// **Um gesto do menu do pincel** — `false` se não é um destes.
    pub(crate) fn aplica_pincel(&mut self, r: &PixelRequest) -> bool {
        let PixelRequest::Brush(a) = r else {
            return false;
        };
        match *a {
            PixelBrushAction::Shape(s) => {
                self.pincel = None;
                let o = ph2d_editor_core::pixel_panel::PixelOptions {
                    shape: s,
                    ..self.options
                };
                crate::viva::opcoes(self, o);
            }
            PixelBrushAction::Recall(n) => {
                if let Some(key) = self.chave
                    && self.recupera(key, n)
                {
                    let n = (n + 1).to_string();
                    let texto = ph2d_i18n::tr_with("pixel.brush.recalled", &[("n", &n)]);
                    self.avisos.push((false, texto));
                }
            }
            PixelBrushAction::SaveHere => {
                if let Some(key) = self.chave {
                    self.guarda_aqui(key);
                }
            }
            PixelBrushAction::ToggleLock(n) => self.tranca_espaco(n),
            PixelBrushAction::Delete(n) => self.apaga_espaco(n),
            PixelBrushAction::Turn(t) => {
                use ph2d_pixel_core::transform as tf;
                let f = match t {
                    PixelPatternTurn::FlipH => tf::flip_h,
                    PixelPatternTurn::FlipV => tf::flip_v,
                    PixelPatternTurn::RotateRight => tf::rot90cw,
                    PixelPatternTurn::RotateLeft => tf::rot90ccw,
                };
                if let Some(p) = &self.pincel {
                    self.pincel = Some(std::sync::Arc::new(p.permuted(f)));
                }
            }
        }
        true
    }

    /// A lista como o painel a mostra (a miniatura amostrada por vizinho até
    /// [`PIXEL_BRUSH_THUMB_MAX`] de lado).
    pub(crate) fn espacos_info(&mut self) -> Vec<PixelBrushSlotInfo> {
        self.espacos()
            .iter()
            .map(|s| PixelBrushSlotInfo {
                locked: s.locked,
                kind: match &s.kind {
                    BrushKind::Shape(sh) => PixelBrushSlotKind::Shape {
                        shape: shape_out(*sh),
                        size: s.size,
                    },
                    BrushKind::Pattern { size, cells, .. } => {
                        PixelBrushSlotKind::Pattern(miniatura(*size, cells))
                    }
                },
            })
            .collect()
    }
}

/// Os PARÂMETROS do pincel na mão que um espaço destrancado acompanha, baratos de comparar a cada
/// quadro. As cores (frente, fundo, *Image Color*) NÃO: escolher uma cor depois do Ctrl+B e voltar
/// ao espaço devolve as cores da imagem no Trial (`rb2b`, teste
/// `each_ctrl_b_is_a_new_slot_and_alt_n_recalls_it_with_the_images_colours`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Visto {
    activo: usize,
    opcoes: (
        PixelBrushShape,
        u32,
        i32,
        ph2d_editor_core::pixel_panel::PixelInk,
        bool,
    ),
    padrao: Option<usize>,
}

impl PixelApp {
    /// **Uma vez por quadro**: o espaço activo DESTRANCADO acompanha o pincel na mão (o que ele
    /// guarda); trancado, nunca muda. Só relê o padrão quando algo mudou.
    pub(crate) fn acompanha_espaco(&mut self, key: u32) {
        let Some(n) = self.espaco_activo else {
            return;
        };
        let o = &self.options;
        let visto = Visto {
            activo: n,
            opcoes: (o.shape, o.size, o.angle, o.ink, o.pixel_perfect),
            padrao: self
                .pincel
                .as_ref()
                .map(|p| std::sync::Arc::as_ptr(p) as usize),
        };
        if self.espaco_visto.as_ref() == Some(&visto) {
            return;
        }
        self.espaco_visto = Some(visto);
        let Some(guardado) = self.espacos_mut().get(n).cloned() else {
            self.espaco_activo = None;
            return;
        };
        if guardado.locked {
            return;
        }
        let Some(agora) = self.espaco_actual(key, guardado.params, false) else {
            return;
        };
        let novo = funde(&guardado, &agora);
        if novo != guardado {
            self.espacos_mut()[n] = novo;
            self.grava_espacos();
        }
    }
}
