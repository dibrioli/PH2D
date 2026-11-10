//! O jogo dos cursores TINGIDOS: um por `(figura, cor)`, criado fora do `&self` (o winit só cria
//! cursores com o laço de eventos à mão) e com tecto — o conjunto de cores de alça é pequeno.

use std::cell::RefCell;

use crate::Figura;

/// A chave de um cursor tingido.
pub type Chave = (Figura, [u8; 4]);

/// Quantos cursores tingidos vivem ao mesmo tempo: o recurso é o handle de cursor do sistema
/// (um `wl_buffer` por cursor no Wayland). As alças da Timeline têm ~10 cores de repouso e usam 3
/// figuras; 32 cobre o conjunto inteiro com folga, e o mais velho sai primeiro além disso.
pub const TECTO_DE_TINGIDOS: usize = 32;

/// Os cursores já criados e os pedidos à espera do próximo `prepara`.
#[derive(Debug)]
pub struct Tingidos<T> {
    feitos: Vec<(Chave, T)>,
    pedidos: RefCell<Vec<Chave>>,
}

impl<T> Default for Tingidos<T> {
    fn default() -> Self {
        Self {
            feitos: Vec::new(),
            pedidos: RefCell::new(Vec::new()),
        }
    }
}

impl<T: Clone> Tingidos<T> {
    /// O cursor de `k`, se já existe; senão fica pedido (uma vez) e a resposta é `None`.
    #[must_use]
    pub fn pega(&self, k: Chave) -> Option<T> {
        if let Some((_, t)) = self.feitos.iter().find(|(c, _)| *c == k) {
            return Some(t.clone());
        }
        let mut p = self.pedidos.borrow_mut();
        if !p.contains(&k) && p.len() < TECTO_DE_TINGIDOS {
            p.push(k);
        }
        None
    }

    /// Cria os pedidos com `faz`; além do tecto sai o mais velho.
    pub fn cumpre(&mut self, mut faz: impl FnMut(Chave) -> Option<T>) {
        for k in self.pedidos.take() {
            if self.feitos.iter().any(|(c, _)| *c == k) {
                continue;
            }
            if let Some(t) = faz(k) {
                if self.feitos.len() >= TECTO_DE_TINGIDOS {
                    self.feitos.remove(0);
                }
                self.feitos.push((k, t));
            }
        }
    }

    /// Quantos estão criados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.feitos.len()
    }

    /// Nenhum criado.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.feitos.is_empty()
    }
}
