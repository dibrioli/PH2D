//! **A lei da janela, sem pintura** (P12a, Aseprite O1–O8 com as leis do dono): a captura de uma
//! tecla, o conflito (tecla do app recusada; tecla de outra acção perguntada, nunca roubada) e o
//! Reset All com confirmação. Quem grava o ficheiro é o painel, depois de [`Janela::aplica`].

use ph2d_i18n::tr;
use ph2d_pixel_keys::{Chord, Conflict, Context, Keymap, PixelAction};

/// O que a janela está a fazer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    #[default]
    Lista,
    /// `Add` premido: à espera do acorde (o `Enter` também se captura; confirma-se no botão).
    Captura {
        acao: PixelAction,
        tecla: Option<Chord>,
    },
    /// A tecla já é de `dono`: «Replace?».
    Pergunta {
        acao: PixelAction,
        tecla: Chord,
        dono: PixelAction,
    },
    /// «Reset All» premido: confirmar.
    ResetAll,
}

/// Um gesto do utilizador na janela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesto {
    Add(PixelAction),
    Tecla(Chord),
    Confirma,
    Replace,
    Cancela,
    Remove(PixelAction, Chord),
    Reset(PixelAction),
    ResetAll,
    /// O texto de um ficheiro importado: SOMA ao mapa (Aseprite R6).
    Importa(String),
}

/// O estado da janela: o mapa vivo, o modo e o aviso da última recusa.
#[derive(Debug, Clone, Default)]
pub struct Janela {
    pub keys: Keymap,
    pub modo: Modo,
    pub aviso: Option<String>,
}

/// A frase do conflito `Taken`.
#[must_use]
pub fn frase_tomada(tecla: Chord, dono: PixelAction) -> String {
    tr("panel.pixel_keys.taken")
        .replace("{chord}", &tecla.to_string())
        .replace("{action}", dono.name())
}

impl Janela {
    #[must_use]
    pub fn new(keys: Keymap) -> Self {
        Self {
            keys,
            ..Self::default()
        }
    }

    /// Aplica o gesto; `true` = o mapa mudou (o painel grava e entrega-o à família).
    pub fn aplica(&mut self, g: Gesto) -> bool {
        let antes = self.keys.clone();
        match (g, self.modo) {
            (Gesto::Add(acao), _) => {
                self.modo = Modo::Captura { acao, tecla: None };
                self.aviso = None;
            }
            (Gesto::Tecla(t), Modo::Captura { acao, .. }) => {
                self.modo = Modo::Captura {
                    acao,
                    tecla: Some(t),
                };
                self.aviso = None;
            }
            (
                Gesto::Confirma,
                Modo::Captura {
                    acao,
                    tecla: Some(t),
                },
            ) => match self.keys.assign(acao, t, Context::Normal) {
                Ok(()) => self.modo = Modo::Lista,
                Err(Conflict::Reserved(dono)) => self.aviso = Some(tr(dono).to_owned()),
                Err(Conflict::Taken(dono)) => {
                    self.modo = Modo::Pergunta {
                        acao,
                        tecla: t,
                        dono,
                    };
                }
            },
            (Gesto::Replace, Modo::Pergunta { acao, tecla, .. }) => {
                if let Err(Conflict::Reserved(dono)) =
                    self.keys.assign_replacing(acao, tecla, Context::Normal)
                {
                    self.aviso = Some(tr(dono).to_owned());
                }
                self.modo = Modo::Lista;
            }
            (Gesto::Confirma, Modo::ResetAll) => {
                self.keys.reset_all();
                self.modo = Modo::Lista;
            }
            (Gesto::Cancela, _) => {
                self.modo = Modo::Lista;
                self.aviso = None;
            }
            (Gesto::Remove(a, t), Modo::Lista) => self.keys.remove(a, t),
            (Gesto::Reset(a), Modo::Lista) => self.keys.reset(a),
            (Gesto::ResetAll, Modo::Lista) => self.modo = Modo::ResetAll,
            (Gesto::Importa(texto), _) => {
                let rep = crate::shim::import_text(&mut self.keys, &texto);
                let n = rep.unknown_ids.len() + rep.rejected.len();
                self.aviso = (n > 0)
                    .then(|| tr("panel.pixel_keys.import_skipped").replace("{n}", &n.to_string()));
            }
            _ => {}
        }
        self.keys != antes
    }

    /// A captura está à espera de uma tecla (o app manda-lhe o acorde antes dos atalhos).
    #[must_use]
    pub const fn a_capturar(&self) -> bool {
        matches!(self.modo, Modo::Captura { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode as K;

    fn j() -> Janela {
        Janela::new(Keymap::factory())
    }

    #[test]
    fn add_captura_e_confirma_junta_a_tecla() {
        let mut w = j();
        let t = Chord::new(K::F12);
        assert!(!w.aplica(Gesto::Add(PixelAction::Outline)));
        assert!(w.a_capturar());
        assert!(!w.aplica(Gesto::Tecla(t)));
        assert!(w.aplica(Gesto::Confirma));
        assert_eq!(w.modo, Modo::Lista);
        assert!(
            w.keys
                .keys_of(PixelAction::Outline)
                .contains(&(t, Context::Normal))
        );
    }

    #[test]
    fn o_enter_tambem_se_captura() {
        let mut w = j();
        w.aplica(Gesto::Add(PixelAction::Outline));
        w.aplica(Gesto::Tecla(Chord::new(K::Enter).alt()));
        assert!(w.aplica(Gesto::Confirma));
    }

    #[test]
    fn tecla_do_app_e_recusada_com_o_nome_do_dono() {
        let mut w = j();
        w.aplica(Gesto::Add(PixelAction::Outline));
        w.aplica(Gesto::Tecla(Chord::new(K::KeyK).ctrl()));
        assert!(!w.aplica(Gesto::Confirma));
        assert!(w.a_capturar(), "a captura continua");
        let aviso = w.aviso.clone().expect("diz de quem é");
        assert!(aviso.starts_with("Used by the app:"), "{aviso}");
    }

    #[test]
    fn tecla_tomada_pergunta_e_so_o_replace_a_tira() {
        let mut w = j();
        let b = Chord::new(K::KeyB);
        w.aplica(Gesto::Add(PixelAction::Outline));
        w.aplica(Gesto::Tecla(b));
        assert!(!w.aplica(Gesto::Confirma), "nunca rouba em silêncio");
        let Modo::Pergunta { dono, .. } = w.modo else {
            panic!("{:?}", w.modo)
        };
        assert_eq!(dono, PixelAction::ToolPencil);
        assert_eq!(
            frase_tomada(b, dono),
            format!("B is used by {}. Replace?", PixelAction::ToolPencil.name())
        );
        let mut cancelada = w.clone();
        assert!(!cancelada.aplica(Gesto::Cancela));
        assert_eq!(
            cancelada.keys.resolve(b, Context::Normal),
            Some(PixelAction::ToolPencil)
        );
        assert!(w.aplica(Gesto::Replace));
        assert_eq!(
            w.keys.resolve(b, Context::Normal),
            Some(PixelAction::Outline)
        );
    }

    #[test]
    fn remove_reset_e_reset_all_com_confirmacao() {
        let mut w = j();
        let b = Chord::new(K::KeyB);
        assert!(w.aplica(Gesto::Remove(PixelAction::ToolPencil, b)));
        assert!(w.keys.keys_of(PixelAction::ToolPencil).is_empty());
        assert!(w.aplica(Gesto::Reset(PixelAction::ToolPencil)));
        assert_eq!(w.keys.to_text(), "", "sem delta: é a fábrica");
        w.aplica(Gesto::Remove(PixelAction::ToolPencil, b));
        assert!(!w.aplica(Gesto::ResetAll), "só pergunta");
        assert_eq!(w.modo, Modo::ResetAll);
        assert!(w.aplica(Gesto::Confirma));
        assert_eq!(w.keys, Keymap::factory());
    }

    #[test]
    fn importar_soma_ao_mapa() {
        let mut w = j();
        assert!(w.aplica(Gesto::Importa("edit.outline = F12\n".to_owned())));
        assert!(
            w.keys
                .keys_of(PixelAction::Outline)
                .contains(&(Chord::new(K::KeyO).shift(), Context::Normal)),
            "a tecla de fábrica fica: o Import soma"
        );
        assert!(
            w.keys
                .keys_of(PixelAction::ToolPencil)
                .contains(&(Chord::new(K::KeyB), Context::Normal))
        );
        assert!(
            w.keys
                .keys_of(PixelAction::Outline)
                .contains(&(Chord::new(K::F12), Context::Normal))
        );
    }
}
