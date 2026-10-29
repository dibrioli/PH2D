//! **A selecção do lado da família** (P6, leis do Trial em `oracle_trial_selection.json`): o gesto
//! das ferramentas de selecção (Rectangular/Elliptical Marquee, Lasso, Polygonal Lasso, Magic
//! Wand), o modo pelos modificadores (Shift somar, Alt+Shift tirar, Ctrl+Shift cruzar), o menu
//! Select e as teclas do Aseprite dos menus Select/Edit/Sprite. Cada gesto que muda a selecção é UM
//! passo do desfazer do sprite (`ph2d_pixel_core::selection`).

use ph2d_editor_core::pixel_panel::{
    PixelDialogKind, PixelEditAction as E, PixelRequest, PixelSelectAction as S, PixelSelectMode,
    PixelSpriteAction as Sp,
};
use ph2d_i18n::tr;
use ph2d_pixel_core::fill::BucketSample;
use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_core::selection::{self, Mask, SelectMode, Wand};
use ph2d_pixel_io::msk;
use winit::keyboard::KeyCode;

use crate::gesture::Mods;
use crate::input::KeyAnswer;
use crate::{PixelApp, Tool};

/// O gesto de selecção em curso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laco {
    pub(crate) key: u32,
    pub(crate) tool: Tool,
    pub(crate) mode: SelectMode,
    /// Marquee: `[premir, cursor]`. Laço: as amostras. Polígono: os vértices fixos e, no fim, o
    /// que segue o cursor.
    pub(crate) pts: Vec<Cell>,
    /// O cursor saiu da célula do premir (um clique parado com a marquee DESSELECCIONA).
    pub(crate) moved: bool,
    pub(crate) held: bool,
}

/// O modo de juntar no núcleo.
#[must_use]
pub fn mode_in(m: PixelSelectMode) -> SelectMode {
    match m {
        PixelSelectMode::Replace => SelectMode::Replace,
        PixelSelectMode::Add => SelectMode::Add,
        PixelSelectMode::Subtract => SelectMode::Subtract,
        PixelSelectMode::Intersect => SelectMode::Intersect,
    }
}

/// O modo de um premir: os modificadores mandam sobre o botão da barra (Trial: Shift = somar,
/// Alt+Shift = tirar, Ctrl+Shift = cruzar).
#[must_use]
pub fn mode_of(m: Mods, base: PixelSelectMode) -> SelectMode {
    match (m.shift, m.alt, m.ctrl) {
        (true, true, _) => SelectMode::Subtract,
        (true, false, true) => SelectMode::Intersect,
        (true, false, false) => SelectMode::Add,
        _ => mode_in(base),
    }
}

/// A máscara que o gesto `l` desenha agora num sprite `w × h` (`None` = ainda nada).
#[must_use]
pub fn preview(l: &Laco, w: u32, h: u32) -> Option<Mask> {
    let m = match l.tool {
        Tool::RectMarquee => Mask::marquee(w, h, l.pts[0], *l.pts.last()?),
        Tool::EllipseMarquee => Mask::ellipse(w, h, l.pts[0], *l.pts.last()?),
        Tool::Lasso if l.pts.len() > 1 => Mask::lasso(w, h, &l.pts),
        Tool::PolygonLasso if l.pts.len() > 1 => Mask::polygon(w, h, &l.pts),
        _ => return None,
    };
    Some(m)
}

impl PixelApp {
    /// Aplica um passo de selecção ao documento `key` (nada a fazer = nenhum passo).
    pub(crate) fn passo(&mut self, key: u32, op: Option<ph2d_pixel_core::Op>) {
        if let Some(op) = op {
            let _ = self.commit(key, op);
        }
    }

    fn junta(&mut self, key: u32, m: &Mask, mode: SelectMode) {
        let op = self
            .docs
            .get(&key)
            .and_then(|d| selection::select(&d.sprite, m, mode));
        self.passo(key, op);
    }

    fn desselecciona(&mut self, key: u32) {
        let op = self
            .docs
            .get(&key)
            .and_then(|d| selection::deselect(&d.sprite));
        self.passo(key, op);
    }

    /// O menu Select; `false` = não é dele.
    pub(crate) fn aplica_selecao(&mut self, r: &PixelRequest) -> bool {
        let PixelRequest::Select(a) = r else {
            return false;
        };
        let Some(key) = self.chave else {
            return true;
        };
        let Some(s) = self.docs.get(&key).map(|d| &d.sprite) else {
            return true;
        };
        let op = match a {
            S::All => selection::select_all(s),
            S::Deselect => selection::deselect(s),
            S::Reselect => selection::reselect(s),
            S::Inverse => selection::invert(s),
            S::ColorRange => {
                self.abre_dialogo = Some(PixelDialogKind::ColorRange);
                None
            }
            S::Modify(k) => {
                if s.mask().is_some() {
                    self.abre_dialogo = Some(PixelDialogKind::Modify(*k));
                } else {
                    self.recusa("pixel.select.refuse.none");
                }
                None
            }
            S::LoadMsk => {
                let d = rfd::FileDialog::new().add_filter(tr("pixel.select.msk_kind"), &["msk"]);
                if let Some(c) = ph2d_app_host::modal::pick_file(d) {
                    match std::fs::read(&c) {
                        Ok(b) => self.carrega_msk(key, &b),
                        Err(e) => self.falha_msk(&e.to_string()),
                    }
                }
                return true;
            }
            S::SaveMsk => {
                let bytes = match self.msk_de(key) {
                    Some(Ok(b)) => b,
                    Some(Err(e)) => {
                        self.falha_msk(&e);
                        return true;
                    }
                    None => {
                        self.recusa("pixel.select.refuse.none");
                        return true;
                    }
                };
                let d = rfd::FileDialog::new()
                    .add_filter(tr("pixel.select.msk_kind"), &["msk"])
                    .set_file_name("selection.msk");
                if let Some(c) = ph2d_app_host::modal::save_file(d) {
                    let path = c.display().to_string();
                    match std::fs::write(&c, bytes) {
                        Ok(()) => {
                            let t =
                                ph2d_i18n::tr_with("pixel.select.msk_saved", &[("path", &path)]);
                            self.avisos.push((false, t));
                        }
                        Err(e) => self.falha_msk(&e.to_string()),
                    }
                }
                return true;
            }
        };
        self.passo(key, op);
        true
    }

    /// *Load from MSK file* com os bytes do ficheiro: UM passo (a selecção substitui a que havia).
    pub fn carrega_msk(&mut self, key: u32, bytes: &[u8]) {
        let Some(s) = self.docs.get(&key).map(|d| &d.sprite) else {
            return;
        };
        let lida = msk::read_msk(bytes).map_err(|e| format!("{e:?}"));
        let m = match lida {
            Ok(f) => f.to_selection(s.width(), s.height()),
            Err(e) => return self.falha_msk(&e),
        };
        let op = if m.count() == 0 {
            selection::deselect(s)
        } else {
            selection::select(s, &m, SelectMode::Replace)
        };
        self.passo(key, op);
        self.avisos
            .push((false, tr("pixel.select.msk_loaded").to_owned()));
    }

    /// Os bytes do *Save to MSK file* da selecção do documento `key` (`None` = sem selecção; `Err`
    /// = a caixa não cabe no formato).
    #[must_use]
    pub fn msk_de(&self, key: u32) -> Option<Result<Vec<u8>, String>> {
        let m = self.docs.get(&key)?.sprite.mask()?;
        Some(msk::write_msk(&msk::MaskFile::from_selection(m)?).map_err(|e| format!("{e:?}")))
    }

    fn falha_msk(&mut self, e: &str) {
        let t = ph2d_i18n::tr_with("pixel.select.msk_failed", &[("e", &e)]);
        self.avisos.push((true, t));
    }

    pub(crate) fn recusa(&mut self, chave: &str) {
        self.avisos.push((true, ph2d_i18n::tr(chave).to_owned()));
    }
}

/// Premir com uma ferramenta de selecção na célula `at` do documento `key`.
pub(crate) fn press(app: &mut PixelApp, key: u32, at: Cell, m: Mods) -> bool {
    let tool = app.tool;
    if let Some(l) = app
        .laco
        .as_mut()
        .filter(|l| l.key == key && l.tool == Tool::PolygonLasso)
    {
        // O polígono continua: um clique no último vértice fixo (ou no primeiro) fecha.
        let fixos = l.pts.len() - 1;
        let fecha = fixos >= 2 && (at == l.pts[fixos - 1] || (fixos >= 3 && at == l.pts[0]));
        if fecha {
            l.pts.pop();
            fecha_poligono(app);
        } else {
            l.pts[fixos] = at;
            l.pts.push(at);
            l.held = true;
        }
        return true;
    }
    let mode = mode_of(m, app.options.select_mode);
    let o = app.options;
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    if tool == Tool::MagicWand {
        if doc.sprite.bounds().contains(at.0, at.1) {
            let opt = Wand {
                tolerance: o.wand_tolerance,
                contiguous: o.wand_contiguous,
                eight: o.wand_eight,
                sample: if o.wand_sample_visible {
                    BucketSample::Visible
                } else {
                    BucketSample::Layer
                },
            };
            let mask = selection::wand(&doc.sprite, (doc.layer, doc.frame), at, opt);
            app.junta(key, &mask, mode);
        }
        return true;
    }
    let pts = if tool == Tool::Lasso {
        vec![at]
    } else {
        vec![at, at]
    };
    app.laco = Some(Laco {
        key,
        tool,
        mode,
        pts,
        moved: false,
        held: true,
    });
    true
}

/// Mover com um gesto de selecção em curso: `true` = consumido.
pub(crate) fn moved(app: &mut PixelApp, at: Option<Cell>) -> bool {
    let Some(l) = app.laco.as_mut() else {
        return false;
    };
    let Some(at) = at else {
        return true;
    };
    match l.tool {
        Tool::Lasso if l.held => {
            if l.pts.last() != Some(&at) {
                l.pts.push(at);
                l.moved = true;
            }
        }
        Tool::RectMarquee | Tool::EllipseMarquee | Tool::PolygonLasso => {
            if let Some(last) = l.pts.last_mut()
                && *last != at
            {
                *last = at;
                l.moved = true;
            }
        }
        _ => {}
    }
    true
}

/// Largar: a marquee e o laço viram UM passo; o polígono fixa o vértice e espera o seguinte.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    let Some(l) = app.laco.as_mut() else {
        return false;
    };
    l.held = false;
    if l.tool == Tool::PolygonLasso {
        return true;
    }
    let Some(l) = app.laco.take() else {
        return true;
    };
    let Some((w, h)) = app
        .docs
        .get(&l.key)
        .map(|d| (d.sprite.width(), d.sprite.height()))
    else {
        return true;
    };
    match preview(&l, w, h) {
        Some(m) if l.moved => app.junta(l.key, &m, l.mode),
        // Um clique sem arrastar: Deselect (Trial 10/10), só no modo de substituir.
        _ if l.mode == SelectMode::Replace => app.desselecciona(l.key),
        _ => {}
    }
    true
}

/// Fecha o polígono em curso (clique no último vértice, `Enter`): UM passo.
pub(crate) fn fecha_poligono(app: &mut PixelApp) {
    let Some(mut l) = app.laco.take() else {
        return;
    };
    let Some((w, h)) = app
        .docs
        .get(&l.key)
        .map(|d| (d.sprite.width(), d.sprite.height()))
    else {
        return;
    };
    if l.held && l.pts.len() > 1 {
        l.pts.pop();
    }
    l.pts.dedup();
    if l.pts.len() >= 2 {
        let m = Mask::polygon(w, h, &l.pts);
        app.junta(l.key, &m, l.mode);
    }
}

fn pede(app: &mut PixelApp, r: PixelRequest) -> KeyAnswer {
    app.pede(r);
    KeyAnswer::Quiet
}

fn ferramenta(app: &mut PixelApp, t: Tool) -> KeyAnswer {
    crate::input::set_tool(app, t);
    KeyAnswer::Toast(t.named())
}

/// **As teclas da P6** (os atalhos do Aseprite dos menus Select/Edit/Sprite e das ferramentas de
/// selecção); `None` = não é daqui. O `Esc` e o `Enter` só são daqui quando há moldura, laço ou
/// selecção (senão seguem para o tocar).
pub(crate) fn key(
    app: &mut PixelApp,
    key: u32,
    code: KeyCode,
    ctrl: bool,
    shift: bool,
) -> Option<KeyAnswer> {
    let solta = app.solta.as_ref().is_some_and(|s| s.key == key);
    let laco = app.laco.as_ref().is_some_and(|l| l.key == key);
    let mascara = app
        .docs
        .get(&key)
        .is_some_and(|d| d.sprite.mask().is_some());
    Some(match (code, ctrl, shift) {
        (KeyCode::KeyM, false, false) => ferramenta(app, Tool::RectMarquee),
        (KeyCode::KeyM, false, true) => ferramenta(app, Tool::EllipseMarquee),
        (KeyCode::KeyQ, false, false) => ferramenta(app, Tool::Lasso),
        (KeyCode::KeyQ, false, true) => ferramenta(app, Tool::PolygonLasso),
        (KeyCode::KeyW, false, false) => ferramenta(app, Tool::MagicWand),
        // P6c: o `V` do Move do Aseprite escolhe o Transform, como o botão dele (levanta já).
        (KeyCode::KeyV, false, false) => {
            app.pede(PixelRequest::Tool(Tool::Transform));
            KeyAnswer::Toast(Tool::Transform.named())
        }
        (KeyCode::KeyA, true, false) => pede(app, PixelRequest::Select(S::All)),
        (KeyCode::KeyD, true, false) => pede(app, PixelRequest::Select(S::Deselect)),
        (KeyCode::KeyD, true, true) => pede(app, PixelRequest::Select(S::Reselect)),
        (KeyCode::KeyI, true, true) => pede(app, PixelRequest::Select(S::Inverse)),
        (KeyCode::Escape, _, _) if solta => pede(app, PixelRequest::Edit(E::Cancel)),
        (KeyCode::Escape, _, _) if laco => {
            app.laco = None;
            KeyAnswer::Quiet
        }
        (KeyCode::Escape, _, _) if mascara => pede(app, PixelRequest::Select(S::Deselect)),
        (KeyCode::Enter | KeyCode::NumpadEnter, false, _) if solta => {
            pede(app, PixelRequest::Edit(E::Commit))
        }
        (KeyCode::Enter | KeyCode::NumpadEnter, false, _) if laco => {
            fecha_poligono(app);
            KeyAnswer::Quiet
        }
        // O Ctrl+Z com células levantadas larga-as (não desfaz o passo de antes).
        (KeyCode::KeyZ, true, _) if solta || laco => {
            app.solta = None;
            app.laco = None;
            app.toca(key);
            KeyAnswer::Quiet
        }
        (KeyCode::KeyT, true, false) => pede(app, PixelRequest::Edit(E::Transform)),
        (KeyCode::KeyH, false, true) => pede(app, PixelRequest::Edit(E::FlipH)),
        (KeyCode::KeyV, false, true) => pede(app, PixelRequest::Edit(E::FlipV)),
        (KeyCode::Delete | KeyCode::Backspace, false, false) => {
            pede(app, PixelRequest::Edit(E::Clear))
        }
        (KeyCode::KeyC, true, false) => pede(app, PixelRequest::Edit(E::Copy)),
        (KeyCode::KeyX, true, false) => pede(app, PixelRequest::Edit(E::Cut)),
        (KeyCode::KeyC, true, true) => pede(app, PixelRequest::Edit(E::CopyMerged)),
        (KeyCode::KeyV, true, false) => pede(app, PixelRequest::Edit(E::Paste)),
        (KeyCode::KeyC, false, false) => pede(app, PixelRequest::Sprite(Sp::CanvasSize)),
        _ => return None,
    })
}
