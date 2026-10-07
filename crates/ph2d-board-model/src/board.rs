//! **Os quadros de um projecto** — a ordem das abas, o nome e a vista de cada um, e os bytes que
//! viajam no `.ph2dproj` (campo próprio, versão própria: o idioma do blob da Timeline).

use serde::{Deserialize, Serialize};

use crate::BoardDoc;

/// Versão do formato de [`BoardSet::to_bytes`]. postcard é posicional: qualquer campo novo sobe-a,
/// e as anteriores continuam a ler-se (`legacy.rs`). 2 = formas com estilo, texto e rotação (W1);
/// 3 = o texto das formas com estilo por trecho (W3); 4 = o rascunho (`Style::sketch`,
/// `Board::sketch`) e os traços da caneta (W4).
pub const FORMAT_VERSION: u32 = 4;

/// Identidade de um quadro no projecto. Nunca reusada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BoardId(pub u64);

/// A vista de um quadro: o ponto do mundo no centro da área e o zoom (`1` = 1 unidade por px).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
        }
    }
}

/// Um quadro: uma aba.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Board {
    pub id: BoardId,
    /// Nome dado pelo artista (dado, não i18n).
    pub name: String,
    pub camera: Camera,
    pub doc: BoardDoc,
    /// O quadro está em RASCUNHO: o que nasce nele nasce à mão (W4).
    pub sketch: bool,
}

/// Todos os quadros do projecto, na ordem das abas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardSet {
    version: u32,
    boards: Vec<Board>,
    next_id: u64,
}

impl Default for BoardSet {
    fn default() -> Self {
        Self {
            version: FORMAT_VERSION,
            boards: Vec::new(),
            next_id: 0,
        }
    }
}

impl BoardSet {
    pub(crate) fn from_parts(boards: Vec<Board>, next_id: u64) -> Self {
        Self {
            version: FORMAT_VERSION,
            boards,
            next_id,
        }
    }

    /// Os quadros, na ordem das abas.
    #[must_use]
    pub fn boards(&self) -> &[Board] {
        &self.boards
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.boards.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: BoardId) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == id)
    }

    pub fn get_mut(&mut self, id: BoardId) -> Option<&mut Board> {
        self.boards.iter_mut().find(|b| b.id == id)
    }

    /// Acrescenta um quadro vazio no fim das abas.
    pub fn create(&mut self, name: String) -> BoardId {
        self.next_id += 1;
        let id = BoardId(self.next_id);
        self.boards.push(Board {
            id,
            name,
            camera: Camera::default(),
            doc: BoardDoc::default(),
            sketch: false,
        });
        id
    }

    /// Cópia inteira de `id`, logo à direita dele. `None` se `id` não existe.
    pub fn duplicate(&mut self, id: BoardId, name: String) -> Option<BoardId> {
        let at = self.boards.iter().position(|b| b.id == id)?;
        self.next_id += 1;
        let new_id = BoardId(self.next_id);
        let mut copy = self.boards[at].clone();
        copy.id = new_id;
        copy.name = name;
        self.boards.insert(at + 1, copy);
        Some(new_id)
    }

    /// Muda o nome. `false` se `id` não existe.
    pub fn rename(&mut self, id: BoardId, name: String) -> bool {
        self.get_mut(id).map(|b| b.name = name).is_some()
    }

    /// Leva a aba de `id` para a posição `to` (limitada ao fim). `false` se `id` não existe.
    pub fn move_tab(&mut self, id: BoardId, to: usize) -> bool {
        let Some(from) = self.boards.iter().position(|b| b.id == id) else {
            return false;
        };
        let b = self.boards.remove(from);
        self.boards.insert(to.min(self.boards.len()), b);
        true
    }

    /// Tira o quadro e devolve-o (quem chama decide se o guarda para desfazer).
    pub fn remove(&mut self, id: BoardId) -> Option<Board> {
        let at = self.boards.iter().position(|b| b.id == id)?;
        Some(self.boards.remove(at))
    }

    /// Os bytes do campo `boards` do projecto.
    ///
    /// # Errors
    /// Só se o postcard falhar a serializar (não acontece com estes tipos).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(self).map_err(|e| e.to_string())
    }

    /// Lê o que [`Self::to_bytes`] escreveu. Bytes vazios = projecto sem quadros.
    ///
    /// # Errors
    /// Bytes corrompidos, ou de uma versão de formato que este build não lê.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        // A versão é o 1.º campo: lê-se sozinha, e decide com que structs ler o resto.
        let (version, _) = postcard::take_from_bytes::<u32>(bytes).map_err(|e| e.to_string())?;
        match version {
            FORMAT_VERSION => postcard::from_bytes(bytes).map_err(|e| e.to_string()),
            1 => crate::legacy::read_v1(bytes),
            2 => crate::legacy::read_v2(bytes),
            3 => crate::legacy::read_v3(bytes),
            v => Err(format!("board format version {v} != {FORMAT_VERSION}")),
        }
    }
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod tests;
