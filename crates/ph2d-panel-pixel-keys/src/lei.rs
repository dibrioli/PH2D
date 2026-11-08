//! **A lei da janela, sem pintura** (P12a, Aseprite O1–O8 com as leis do dono; dono 08/11: um passo,
//! o clique no campo captura e a tecla entra no key-down): a captura de uma
//! tecla, o conflito (tecla do app recusada; tecla de outra acção perguntada, nunca roubada) e o
//! Reset All com confirmação. Quem grava o ficheiro é o painel, depois de [`Janela::aplica`].

use ph2d_i18n::tr;
use ph2d_pixel_keys::{Chord, Conflict, Context, Keymap, PixelAction};

/// O campo de tecla em captura: a acção e a tecla que ele substitui (`None` = o campo vazio).
pub type Campo = (PixelAction, Option<(Chord, Context)>);

/// O que a janela está a fazer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    #[default]
    Lista,
    /// Um campo clicado: limpo, à espera do acorde, que entra JÁ no key-down (dono 08/11, §9.8).
    Captura { campo: Campo },
    /// A tecla já é de `dono`: «Replace?» na fileira do campo.
    Pergunta {
        campo: Campo,
        tecla: Chord,
        dono: PixelAction,
    },
    /// «Reset All» premido: confirmar.
    ResetAll,
}

/// Um gesto do utilizador na janela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesto {
    /// Clique num campo de tecla: limpa-o e captura.
    Campo(Campo),
    /// O acorde premido durante a captura (`Esc` cancela, `Backspace`/`Del` tiram a tecla).
    Tecla(Chord),
    Confirma,
    Replace,
    Cancela,
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

fn nua(t: Chord, k: winit::keyboard::KeyCode) -> bool {
    t == Chord::new(k)
}

/// O mapa com a tecla do campo trocada por `t` (`rouba` = depois do «Replace»).
fn troca(keys: &Keymap, (acao, antiga): Campo, t: Chord, rouba: bool) -> Result<Keymap, Conflict> {
    let mut k = keys.clone();
    let ctx = antiga.map_or(Context::Normal, |(_, c)| c);
    if let Some((a, _)) = antiga {
        k.remove(acao, a);
    }
    if rouba {
        k.assign_replacing(acao, t, ctx)?;
    } else {
        k.assign(acao, t, ctx)?;
    }
    Ok(k)
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
        use winit::keyboard::KeyCode as K;
        let antes = self.keys.clone();
        match (g, self.modo) {
            (Gesto::Campo(campo), _) => {
                self.modo = Modo::Captura { campo };
                self.aviso = None;
            }
            (Gesto::Tecla(t), Modo::Captura { .. }) if nua(t, K::Escape) => {
                self.modo = Modo::Lista;
                self.aviso = None;
            }
            (Gesto::Tecla(t), Modo::Captura { campo })
                if nua(t, K::Backspace) || nua(t, K::Delete) =>
            {
                if let (acao, Some((a, _))) = campo {
                    self.keys.remove(acao, a);
                }
                self.modo = Modo::Lista;
                self.aviso = None;
            }
            (Gesto::Tecla(t), Modo::Captura { campo }) => {
                match troca(&self.keys, campo, t, false) {
                    Ok(k) => {
                        self.keys = k;
                        self.modo = Modo::Lista;
                        self.aviso = None;
                    }
                    Err(Conflict::Reserved(dono)) => self.aviso = Some(tr(dono).to_owned()),
                    Err(Conflict::Taken(dono)) => {
                        self.modo = Modo::Pergunta {
                            campo,
                            tecla: t,
                            dono,
                        };
                        self.aviso = None;
                    }
                }
            }
            (Gesto::Replace, Modo::Pergunta { campo, tecla, .. }) => {
                match troca(&self.keys, campo, tecla, true) {
                    Ok(k) => self.keys = k,
                    Err(Conflict::Reserved(dono)) => self.aviso = Some(tr(dono).to_owned()),
                    Err(Conflict::Taken(_)) => {}
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
            (Gesto::Reset(a), _) => {
                self.keys.reset(a);
                self.modo = Modo::Lista;
            }
            (Gesto::ResetAll, _) => self.modo = Modo::ResetAll,
            (Gesto::Importa(texto), _) => {
                let rep = self.keys.import_text(&texto);
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

    /// O campo em captura ou em pergunta, se é este.
    #[must_use]
    pub fn campo(&self) -> Option<Campo> {
        match self.modo {
            Modo::Captura { campo } | Modo::Pergunta { campo, .. } => Some(campo),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode as K;

    fn j() -> Janela {
        Janela::new(Keymap::factory())
    }

    fn campo_de(w: &Janela, a: PixelAction) -> Campo {
        (a, w.keys.keys_of(a).first().copied())
    }

    #[test]
    fn o_clique_no_campo_captura_e_a_tecla_troca_ja() {
        let mut w = j();
        let t = Chord::new(K::F12);
        let c = campo_de(&w, PixelAction::Outline);
        assert!(!w.aplica(Gesto::Campo(c)));
        assert!(w.a_capturar());
        assert!(w.aplica(Gesto::Tecla(t)), "sem Confirm: entra no key-down");
        assert_eq!(w.modo, Modo::Lista);
        assert_eq!(
            w.keys.keys_of(PixelAction::Outline),
            vec![(t, Context::Normal)]
        );
    }

    #[test]
    fn o_campo_vazio_junta_uma_tecla() {
        let mut w = j();
        let t = Chord::new(K::F12);
        let antes = w.keys.keys_of(PixelAction::Outline).len();
        w.aplica(Gesto::Campo((PixelAction::Outline, None)));
        assert!(w.aplica(Gesto::Tecla(t)));
        assert_eq!(w.keys.keys_of(PixelAction::Outline).len(), antes + 1);
    }

    #[test]
    fn esc_cancela_e_backspace_tira() {
        let mut w = j();
        let c = campo_de(&w, PixelAction::ToolPencil);
        w.aplica(Gesto::Campo(c));
        assert!(!w.aplica(Gesto::Tecla(Chord::new(K::Escape))));
        assert_eq!(w.modo, Modo::Lista);
        assert_eq!(w.keys, Keymap::factory(), "o Esc devolve a tecla antiga");
        w.aplica(Gesto::Campo(c));
        assert!(w.aplica(Gesto::Tecla(Chord::new(K::Backspace))));
        assert!(w.keys.keys_of(PixelAction::ToolPencil).is_empty());
    }

    #[test]
    fn o_enter_tambem_se_captura() {
        let mut w = j();
        w.aplica(Gesto::Campo((PixelAction::Outline, None)));
        assert!(w.aplica(Gesto::Tecla(Chord::new(K::Enter).alt())));
    }

    #[test]
    fn tecla_do_app_e_recusada_com_o_nome_do_dono() {
        let mut w = j();
        w.aplica(Gesto::Campo((PixelAction::Outline, None)));
        assert!(!w.aplica(Gesto::Tecla(Chord::new(K::KeyK).ctrl())));
        assert!(w.a_capturar(), "a captura continua");
        let aviso = w.aviso.clone().expect("diz de quem é");
        assert!(aviso.starts_with("Used by the app:"), "{aviso}");
    }

    #[test]
    fn tecla_tomada_pergunta_e_so_o_replace_a_tira() {
        let mut w = j();
        let b = Chord::new(K::KeyB);
        let c = campo_de(&w, PixelAction::Outline);
        w.aplica(Gesto::Campo(c));
        assert!(!w.aplica(Gesto::Tecla(b)), "nunca rouba em silêncio");
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
        assert_eq!(cancelada.keys, Keymap::factory());
        assert!(w.aplica(Gesto::Replace));
        assert_eq!(
            w.keys.resolve(b, Context::Normal),
            Some(PixelAction::Outline)
        );
        assert_eq!(
            w.keys.keys_of(PixelAction::Outline),
            vec![(b, Context::Normal)]
        );
    }

    #[test]
    fn reset_e_reset_all_com_confirmacao() {
        let mut w = j();
        let c = campo_de(&w, PixelAction::ToolPencil);
        w.aplica(Gesto::Campo(c));
        w.aplica(Gesto::Tecla(Chord::new(K::Delete)));
        assert!(w.aplica(Gesto::Reset(PixelAction::ToolPencil)));
        assert_eq!(w.keys.to_text(), "", "sem delta: é a fábrica");
        w.aplica(Gesto::Campo(c));
        w.aplica(Gesto::Tecla(Chord::new(K::Delete)));
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
                .keys_of(PixelAction::Outline)
                .contains(&(Chord::new(K::F12), Context::Normal))
        );
    }
}
