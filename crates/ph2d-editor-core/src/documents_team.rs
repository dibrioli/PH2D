//! ⭐ **A equipa e a saída — o estado de SESSÃO** (MiroClone W8, `docs/MiroClone/06_equipa_e_saida.md`):
//! a ferramenta de comentar e o fio aberto, o painel dos comentários e o das versões, o diálogo de
//! exportar, a apresentação a correr, o nome de quem escreve, e os PEDIDOS que a família do quadro
//! drena (abrir/gravar um ficheiro, ler/gravar o nome). Os comentários e as versões em si são
//! DOCUMENTO (`Board::team`).

use std::collections::BTreeMap;
use std::time::Instant;

use ph2d_board_model::{BoardId, Camera, ElementId, MessageId, Pin, ThreadId, VersionId};

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

/// K2 — a bolha a ser arrastada (px de ecrã): o fio, o desvio do ponteiro à ponta, onde o
/// carregar foi, onde o ponteiro está, e se já andou mais que o limiar (senão o largar é um clique).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubbleDrag {
    pub thread: ThreadId,
    pub grab: [f32; 2],
    pub from: [f32; 2],
    pub at: [f32; 2],
    pub moved: bool,
}

impl BubbleDrag {
    /// A ponta da bolha agora (o ponteiro menos o desvio).
    #[must_use]
    pub fn tip(&self) -> [f32; 2] {
        [self.at[0] - self.grab[0], self.at[1] - self.grab[1]]
    }
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
    /// P6 — a ferramenta de comentar estava armada antes (volta armada ao sair).
    pub was_commenting: bool,
}

/// O apagar armado à espera do 2.º clique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    /// Uma mensagem do fio (pelo id: a posição muda quando outra se apaga).
    Message(ThreadId, MessageId),
    /// Uma versão do quadro (pelo id: ao vivo, uma versão alheia que chega muda as posições).
    Version(BoardId, VersionId),
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
    /// K4 — a mensagem que se está a editar: o fio E o id dela (um índice apontava para outra mensagem
    /// depois de um apagar, auditoria W8).
    pub editing: Option<(ThreadId, MessageId)>,
    /// R2 — o painel das versões do quadro activo está aberto.
    pub versions_open: bool,
    /// R2 — a versão que se está a renomear: o quadro E o id dela (um índice passava a apontar
    /// outra versão quando uma chegava da rede ou se apagava).
    pub renaming_version: Option<(BoardId, VersionId)>,
    /// K4/R2 — apagar pede um 2.º clique no MESMO alvo (o 1.º arma; qualquer outro clique desarma).
    pub confirm: Option<Confirm>,
    /// A 1.ª linha da lista ou das versões à vista (os painéis paginam quando não cabem).
    pub from: usize,
    /// Quantas filas couberam na última pintura (o passo de uma página).
    pub page: usize,
    /// E3 — o tamanho do PNG já medido e a chave dele (quadro, revisão, escolha, selecção).
    #[allow(clippy::type_complexity)]
    pub px: Option<(
        (BoardId, u64, ExportChoice, Vec<ElementId>),
        Option<(u32, u32)>,
    )>,
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
    /// K2 — a bolha a ser arrastada.
    pub drag: Option<BubbleDrag>,
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

    /// A última frase dita (a que o topo do quadro pinta).
    #[must_use]
    pub fn told(&self) -> Option<&str> {
        self.live.team.message.as_ref().map(|(t, _)| t.as_str())
    }

    /// K10 — o nome mudou na interface: devolve-o UMA vez para a família o gravar.
    pub fn take_author_change(&mut self) -> Option<String> {
        std::mem::take(&mut self.live.team.author_dirty).then(|| self.live.team.author.clone())
    }

    /// X3 — um quadro novo no fim das abas com o documento importado, já activo e enquadrado.
    pub fn import_board(
        &mut self,
        name: String,
        doc: ph2d_board_model::BoardDoc,
        old_votes: std::collections::BTreeMap<ph2d_board_model::ElementId, u32>,
        files: ph2d_board_model::Files,
    ) -> BoardId {
        let id = self.create(name);
        if let Some(b) = self.boards.get_mut(id) {
            b.doc = doc;
            // W14 — os bytes das imagens que o ficheiro trazia.
            b.files = files;
            // W13 (ADR-0183 §9): os votos de um ficheiro de antes do histórico viram UMA sessão.
            b.team.adopt_old_votes(old_votes);
        }
        self.live.nav.fit_pending = true;
        id
    }

    /// R3 — restaura a versão `ver` do quadro `id` como um quadro NOVO à direita dele, já activo.
    /// `None` = não há essa versão, ou ela não se lê (bytes corrompidos, ou de um formato que este
    /// build não lê) — quem chama avisa.
    pub fn restore_version(&mut self, id: BoardId, ver: VersionId) -> Option<BoardId> {
        let b = self.boards.get(id)?;
        let v = b.team.versions.iter().find(|v| v.id == ver)?;
        let saved = v.read().ok()?;
        let name = format!("{} — {}", b.name, v.name);
        let new = self
            .boards
            .insert_after(id, name, saved.doc, saved.sketch)?;
        if let Some(r) = self.boards.get_mut(new) {
            r.team.hidden_slides = saved.team.hidden_slides;
        }
        self.activate(Some(new));
        self.live.nav.fit_pending = true;
        Some(new)
    }
}
