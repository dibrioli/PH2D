//! **A sprite com os seus píxeis** (W19): uma `Sprite` de um objecto `Quem::Mundo` que não traz um
//! `SpritePixels` (uma célula do atlas — a imagem importada de 8 bits) aponta uma célula que noutro
//! projecto é outra imagem. Ao Parar, os píxeis dela leem-se pela composição e entram como `Recurso`, com
//! um `SpritePixels` do mesmo id na linha: o aplicar fá-la nascer pela porta de produto, como qualquer
//! imagem embutida. Não se leram ⇒ a `Sprite` fica de fora, nomeada no aviso (nunca a célula crua).

use super::formato::{Componente, Quem, Recurso};
use super::{PIXEIS, Tradutor};
use crate::aplicar::Rgba;

const SPRITE: &str = "ph2d::render::Sprite";

/// O que só a composição lê, ao Parar.
pub struct Leitores<'a> {
    /// Os bytes de um recurso pelo id de conteúdo (o `AssetDb`).
    pub recurso: &'a dyn Fn(&[u8; 32]) -> Option<Vec<u8>>,
    /// A sprite que mostra esse recurso tem-no em alfa pré-multiplicado.
    pub premultiplicado: &'a dyn Fn(&[u8; 32]) -> bool,
    /// Os píxeis (alfa recto) da sprite do objecto `StableId`: `None` = o objecto já não existe;
    /// `Some(None)` = existe e não se leram.
    pub sprite: &'a mut dyn FnMut(u64) -> Option<Option<Rgba>>,
    /// A sprite (`StableId`) que o Painter pinta sem a ter assado, com a tinta
    /// (`PainterTool::imagem_por_assar`, #53): é ela que se embute, nunca o `SpritePixels` de antes.
    pub por_assar: Option<(u64, Rgba)>,
}

/// O id que o `AssetDb` cunha para estes píxeis (`insert_image_rgba8`: lados + bytes).
fn id_de(rgba: &[u8], w: u32, h: u32) -> [u8; 32] {
    let mut b = Vec::with_capacity(8 + rgba.len());
    b.extend_from_slice(&w.to_le_bytes());
    b.extend_from_slice(&h.to_le_bytes());
    b.extend_from_slice(rgba);
    *ph2d_asset_id::AssetId::from_bytes(&b).as_bytes()
}

impl Tradutor<'_> {
    fn do_mundo(&self, sid: u64) -> bool {
        self.donos.get(&sid).is_none_or(|d| d.0 == Quem::Mundo)
    }

    /// O id embutido dos píxeis da sprite de `sid` (lidos uma vez); `None` = não se leram.
    fn embutida(&mut self, sid: u64) -> Option<[u8; 32]> {
        if let Some(x) = self.sprites.get(&sid) {
            return *x;
        }
        let lida = match &self.l.por_assar {
            Some((s, px)) if *s == sid => Some(Some(px.clone())),
            _ => (self.l.sprite)(sid),
        };
        let png = match lida {
            // Um objecto que já não existe não se mostra no fim (o aviso nomeia-o como apagado).
            None => return None,
            Some(Some((rgba, w, h))) => {
                super::aplica::png(w, h, &rgba).map(|png| (id_de(&rgba, w, h), png))
            }
            Some(None) => None,
        };
        let id = match png {
            Some((id, bytes)) => {
                if self.recursos_vistos.insert(id) {
                    self.recursos.push(Recurso {
                        id,
                        bytes,
                        premultiplicado: false,
                    });
                }
                Some(id)
            }
            None => {
                self.recusados.insert(SPRITE.to_owned());
                None
            }
        };
        self.sprites.insert(sid, id);
        id
    }

    /// O `SpritePixels` de `sid` aponta a imagem de ANTES da tinta por assar: sai, e a sprite embute-se.
    pub(super) fn pixeis_velhos(&self, sid: u64, nome: &str) -> bool {
        nome == PIXEIS
            && self.do_mundo(sid)
            && self.l.por_assar.as_ref().is_some_and(|(s, _)| *s == sid)
    }

    fn componente(id: [u8; 32]) -> Componente {
        Componente {
            nome: PIXEIS.to_owned(),
            bytes: postcard::to_allocvec(&id).expect("32 bytes"),
        }
    }

    /// Uma linha inteira: a `Sprite` sem `SpritePixels` leva os píxeis, ou sai.
    pub(super) fn sprite_da_linha(&mut self, sid: u64, cs: &mut Vec<Componente>) {
        if cs.iter().any(|c| c.nome == PIXEIS) {
            self.com_pixeis.insert(sid);
            return;
        }
        self.com_pixeis.remove(&sid);
        if !self.do_mundo(sid) || !cs.iter().any(|c| c.nome == SPRITE) {
            return;
        }
        match self.embutida(sid) {
            Some(id) => cs.push(Self::componente(id)),
            None => cs.retain(|c| c.nome != SPRITE),
        }
    }

    /// Uma mudança: o mesmo, sobre o que mudou (o `SpritePixels` pode chegar ou sair nela).
    pub(super) fn sprite_da_mudanca(&mut self, sid: u64, cs: &mut Vec<(String, Option<Vec<u8>>)>) {
        if let Some((_, b)) = cs.iter().find(|c| c.0 == PIXEIS) {
            if b.is_some() {
                self.com_pixeis.insert(sid);
            } else {
                self.com_pixeis.remove(&sid);
            }
        }
        let sprite = cs.iter().any(|c| c.0 == SPRITE && c.1.is_some());
        if !sprite || self.com_pixeis.contains(&sid) || !self.do_mundo(sid) {
            return;
        }
        match self.embutida(sid) {
            Some(id) => {
                cs.retain(|c| c.0 != PIXEIS);
                cs.push((PIXEIS.to_owned(), Some(Self::componente(id).bytes)));
            }
            None => cs.retain(|c| c.0 != SPRITE),
        }
    }
}
