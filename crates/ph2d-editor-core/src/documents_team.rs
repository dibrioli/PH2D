//! ⭐ **A equipa e a saída — o estado de SESSÃO** (MiroClone W8, `docs/MiroClone/06_equipa_e_saida.md`):
//! a ferramenta de comentar e o fio aberto, o painel dos comentários e o das versões, o diálogo de
//! exportar, a apresentação a correr, o nome de quem escreve, e os PEDIDOS que a família do quadro
//! drena (abrir/gravar um ficheiro, ler/gravar o nome). Os comentários e as versões em si são
//! DOCUMENTO (`Board::team`).

use std::collections::BTreeMap;
use std::time::Instant;

use ph2d_board_model::{BoardId, Camera, ElementId, Pin, ThreadId};

use super::Documents;

/// O formato do exportar (E1) — e o `.excalidraw` (X1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Png,
    Svg,
    Pdf,
    Excalidraw,
}

/// A área do exportar (E2/E4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Board,
    Selection,
    /// Um frame por página/imagem: os seleccionados, senão os diapositivos.
    Frames,
}

/// As escolhas do diálogo de exportar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportChoice {
    pub format: Format,
    pub scope: Scope,
    /// O índice de `ph2d_board_export::PNG_MEGAPIXELS` (Small · Medium · Large).
    pub size: usize,
    /// E7 — fundo transparente (PNG e SVG).
    pub transparent: bool,
}

impl Default for ExportChoice {
    fn default() -> Self {
        Self {
            format: Format::Png,
            scope: Scope::Board,
            size: 1,
            transparent: false,
        }
    }
}

/// Um pedido que a família do quadro drena (`ph2d_app_board::io`): o diálogo de ficheiro é dela.
#[derive(Clone, Debug, PartialEq)]
pub enum IoRequest {
    /// Exportar o quadro `board` com estas escolhas, e a selecção de agora.
    Export {
        board: BoardId,
        choice: ExportChoice,
        selection: Vec<ElementId>,
    },
    /// Abrir um `.excalidraw` (ou um `.png`/`.svg` com a cena) num quadro novo.
    Import,
}

/// O fio aberto no cartão: um novo (ainda sem mensagem, preso em `Pin`) ou um que existe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpenThread {
    New(Pin),
    Thread(ThreadId),
}

/// A apresentação a correr (P1–P6).
#[derive(Clone, Debug, PartialEq)]
pub struct Show {
    pub board: BoardId,
    pub slides: Vec<ElementId>,
    pub at: usize,
    /// A viagem da câmara: de onde, para onde, desde quando.
    pub from: Camera,
    pub to: Camera,
    pub since: Instant,
    /// P6 — a vista de antes, devolvida ao sair.
    pub before: Camera,
}

/// O estado de sessão da equipa.
#[derive(Debug, Default)]
pub struct Team {
    /// K1 — a ferramenta de comentar está armada (`C`).
    pub commenting: bool,
    /// K8 — `Shift+C` esconde todos (por omissão à vista).
    pub hide_comments: bool,
    /// K5 — os resolvidos à vista (com o visto).
    pub show_resolved: bool,
    /// K8 — o painel da lista.
    pub list_open: bool,
    /// O fio aberto no cartão, por quadro.
    pub open: BTreeMap<BoardId, OpenThread>,
    /// K4 — a mensagem do fio aberto que se está a editar (o índice).
    pub editing: Option<usize>,
    /// R2 — o painel das versões do quadro activo está aberto.
    pub versions_open: bool,
    /// R2 — a versão que se está a renomear (o índice).
    pub renaming_version: Option<usize>,
    /// O diálogo de exportar está aberto, com as escolhas.
    pub export: Option<ExportChoice>,
    /// A apresentação a correr.
    pub show: Option<Show>,
    /// K10 — o nome de quem escreve (`~/.ph2d/board_author.txt`; a 1.ª vez, a conta do sistema).
    pub author: String,
    /// O nome mudou e a família do quadro tem de o gravar.
    pub author_dirty: bool,
    /// O pedido por drenar.
    pub io: Option<IoRequest>,
    /// K10 — o campo do nome está aberto na lista.
    pub editing_author: bool,
    /// A frase de um pedido servido (o ficheiro gravado, o que a importação saltou) e quando chegou —
    /// pintada no topo do quadro como o aviso (W5: nunca só o terminal).
    pub message: Option<(String, Instant)>,
}

impl Team {
    /// O quadro `id` saiu (apagado): o fio aberto e a apresentação dele também.
    pub fn forget(&mut self, id: BoardId) {
        self.open.remove(&id);
        if self.show.as_ref().is_some_and(|s| s.board == id) {
            self.show = None;
        }
    }

    /// Outro projecto: nada da sessão do anterior cai num quadro novo com o mesmo id (os `BoardId`
    /// recomeçam). O nome de quem escreve fica (é da máquina).
    pub fn reset(&mut self) {
        let author = std::mem::take(&mut self.author);
        *self = Self {
            author,
            ..Self::default()
        };
    }
}

impl Documents {
    /// ⭐ W8 — o pedido de abrir/gravar um ficheiro, tirado UMA vez: a família do quadro serve-o.
    pub fn take_io(&mut self) -> Option<IoRequest> {
        self.live.team.io.take()
    }

    /// O estado de sessão da equipa (só para ler: quem o muda é a interface).
    #[must_use]
    pub fn team_session(&self) -> &Team {
        &self.live.team
    }

    /// K10 — o nome de quem escreve.
    #[must_use]
    pub fn author(&self) -> &str {
        &self.live.team.author
    }

    /// K10 — o nome lido da máquina (a família do quadro chama-o ao arrancar).
    pub fn set_author(&mut self, name: String) {
        self.live.team.author = name;
    }

    /// A frase de um pedido servido (`Exported to …`, `Imported …`), pintada no topo do quadro.
    pub fn tell(&mut self, text: String) {
        self.live.team.message = Some((text, Instant::now()));
    }

    /// K10 — o nome mudou na interface: devolve-o UMA vez para a família o gravar.
    pub fn take_author_change(&mut self) -> Option<String> {
        std::mem::take(&mut self.live.team.author_dirty).then(|| self.live.team.author.clone())
    }

    /// X3 — um quadro novo à direita do activo (ou no fim) com o documento importado, já activo e
    /// enquadrado.
    pub fn import_board(&mut self, name: String, doc: ph2d_board_model::BoardDoc) -> BoardId {
        let id = self.create(name);
        if let Some(b) = self.boards.get_mut(id) {
            b.doc = doc;
        }
        self.live.nav.fit_pending = true;
        id
    }

    /// R3 — restaura a versão `n` do quadro `id` como um quadro NOVO à direita dele, já activo.
    /// `Err` = a versão não se lê.
    ///
    /// # Errors
    /// Bytes corrompidos, ou de um formato que este build não lê.
    pub fn restore_version(&mut self, id: BoardId, n: usize) -> Result<BoardId, String> {
        let b = self.boards.get(id).ok_or("no board")?;
        let v = b.team.versions.get(n).ok_or("no version")?;
        let (doc, sketch) = v.read()?;
        let name = format!("{} — {}", b.name, v.name);
        let new = self
            .boards
            .insert_after(id, name, doc, sketch)
            .ok_or("no board")?;
        self.activate(Some(new));
        self.live.nav.fit_pending = true;
        Ok(new)
    }
}
