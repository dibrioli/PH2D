//! **Os quadros de um projecto** — a ordem das abas, o nome e a vista de cada um, e os bytes que
//! viajam no `.ph2dproj` (campo próprio, versão própria: o idioma do blob da Timeline).

use serde::{Deserialize, Serialize};

use crate::{BoardDoc, Files, Team};

/// Versão do formato de [`BoardSet::to_bytes`]. postcard é posicional: qualquer campo novo sobe-a,
/// e as anteriores continuam a ler-se (`legacy.rs`). 2 = formas com estilo, texto e rotação (W1);
/// 3 = o texto das formas com estilo por trecho (W3); 4 = o rascunho (`Style::sketch`,
/// `Board::sketch`) e os traços da caneta (W4); 5 = organizar — frames, grupos, travar, links e
/// etiquetas (W5); 6 = o nó de mapa mental (`Org::mind`, W6); 7 = os votos e os carimbos
/// (`Org::votes`, `Org::stamps`, W7); 8 = a equipa (`Board::team`: comentários, versões, frames fora da
/// apresentação, W8); 9 = o aspecto que vem do Excalidraw (`Style::look`: preenchimento, rugosidade 2,
/// alinhamento e letra do texto; `Org::seed`: a semente do tremor, W9); 10 = a identidade das
/// mensagens e das versões da equipa (`Message::id`, `Version::id`, W11 — ADR-0181); 11 = o histórico
/// das votações (`Team::polls`, `Team::shown`) e o elemento sem `Org::votes` (W13 — ADR-0183); 12 = as
/// imagens (`Board::files`, `Version::files`, `ElementKind::Image`, W14).
pub const FORMAT_VERSION: u32 = 12;

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
    /// W8 — comentários, versões e frames fora da apresentação ([`Team`]).
    pub team: Team,
    /// W14 — os bytes das imagens, uma vez por conteúdo (`image.rs`). Grava-se só o que está em uso.
    pub files: Files,
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
            team: Team::default(),
            files: Files::new(),
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
        // ADR-0183 §8 — o histórico das votações não vai na cópia (Miro: «If I duplicate my board… No»).
        (copy.team.polls, copy.team.shown) = (Vec::new(), None);
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

    /// Os quadros, por valor (o leitor de uma versão, W8).
    #[must_use]
    pub fn into_boards(self) -> Vec<Board> {
        self.boards
    }

    /// Restaura uma versão (R3): um quadro NOVO logo à direita de `after`, com o documento e o rascunho
    /// dela e nada da equipa — e os ficheiros de imagem que ela mostra, tirados de `after` (uma versão
    /// não guarda bytes: o quadro guarda-os enquanto ela existir). `None` se `after` não existe.
    pub fn insert_after(
        &mut self,
        after: BoardId,
        name: String,
        doc: BoardDoc,
        sketch: bool,
    ) -> Option<BoardId> {
        let at = self.boards.iter().position(|b| b.id == after)?;
        self.next_id += 1;
        let id = BoardId(self.next_id);
        let used = doc.image_files();
        let files = self.boards[at]
            .files
            .iter()
            .filter(|(k, _)| used.contains(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        self.boards.insert(
            at + 1,
            Board {
                id,
                name,
                camera: Camera::default(),
                doc,
                sketch,
                team: Team::default(),
                files,
            },
        );
        Some(id)
    }

    /// Tira o quadro e devolve-o (quem chama decide se o guarda para desfazer).
    pub fn remove(&mut self, id: BoardId) -> Option<Board> {
        let at = self.boards.iter().position(|b| b.id == id)?;
        Some(self.boards.remove(at))
    }

    /// O quadro `id` em bytes, para quem entra numa sessão ao vivo (o protocolo tem a versão dele).
    #[must_use]
    pub fn board_bytes(board: &Board) -> Vec<u8> {
        postcard::to_allocvec(board).expect("postcard escreve qualquer quadro")
    }

    /// [`Self::board_bytes`] SEM os bytes das imagens (W14): o retrato de quem entra numa sessão ao
    /// vivo — os ficheiros seguem um a um, cada um na sua mensagem (o tecto de uma é o do transporte).
    #[must_use]
    pub fn board_bytes_lean(board: &Board) -> Vec<u8> {
        /// O MESMO desenho de campos que [`Board`] (gate `the_lean_board_is_the_board_without_files`).
        #[derive(Serialize)]
        struct Lean<'a> {
            id: BoardId,
            name: &'a str,
            camera: Camera,
            doc: &'a BoardDoc,
            sketch: bool,
            team: &'a Team,
            files: &'a Files,
        }
        let lean = Lean {
            id: board.id,
            name: &board.name,
            camera: board.camera,
            doc: &board.doc,
            sketch: board.sketch,
            team: &board.team,
            files: &Files::new(),
        };
        postcard::to_allocvec(&lean).expect("postcard escreve qualquer quadro")
    }

    /// Lê o que [`Self::board_bytes`] escreveu, com os índices refeitos.
    ///
    /// # Errors
    /// Bytes corrompidos.
    pub fn board_from_bytes(bytes: &[u8]) -> Result<Board, String> {
        let mut b: Board = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
        b.doc.reindex();
        Ok(b)
    }

    /// Os bytes do campo `boards` do projecto. De cada quadro só vão os ficheiros de imagem em uso
    /// ([`Board::files_in_use`]): uma imagem apagada deixa de pesar no projecto ao gravar.
    ///
    /// # Errors
    /// Só se o postcard falhar a serializar (não acontece com estes tipos).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        /// O MESMO desenho de campos que [`Board`] (gate `the_saved_set_is_the_set_with_its_files_in_use`).
        #[derive(Serialize)]
        struct BoardOut<'a> {
            id: BoardId,
            name: &'a str,
            camera: Camera,
            doc: &'a BoardDoc,
            sketch: bool,
            team: &'a Team,
            files: crate::image::Kept<'a>,
        }
        #[derive(Serialize)]
        struct SetOut<'a> {
            version: u32,
            boards: Vec<BoardOut<'a>>,
            next_id: u64,
        }
        let boards = self
            .boards
            .iter()
            .map(|b| BoardOut {
                id: b.id,
                name: &b.name,
                camera: b.camera,
                doc: &b.doc,
                sketch: b.sketch,
                team: &b.team,
                files: crate::image::Kept::of(&b.files, &b.files_in_use()),
            })
            .collect();
        let set = SetOut {
            version: self.version,
            boards,
            next_id: self.next_id,
        };
        postcard::to_allocvec(&set).map_err(|e| e.to_string())
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
            FORMAT_VERSION => postcard::from_bytes(bytes)
                .map(|mut set: Self| {
                    for b in &mut set.boards {
                        b.doc.reindex();
                    }
                    set
                })
                .map_err(|e| e.to_string()),
            1 => crate::legacy::read_v1(bytes),
            2 => crate::legacy::read_v2(bytes),
            3 => crate::legacy::read_v3(bytes),
            4 => crate::legacy::read_v4(bytes),
            5 => crate::legacy::read_v5(bytes),
            6 => crate::legacy::read_v6(bytes),
            7 => crate::legacy::read_v7(bytes),
            8 => crate::legacy::read_v8(bytes),
            9 => crate::legacy::read_v9(bytes),
            10 => crate::legacy::read_v10(bytes),
            11 => crate::legacy::read_v11(bytes),
            v => Err(format!("board format version {v} != {FORMAT_VERSION}")),
        }
    }
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod tests;
