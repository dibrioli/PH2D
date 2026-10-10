//! **O quadro ao vivo, do lado do ECRÃ** (ADR-0181): o que o painel «Ao vivo», os avatares e os
//! cursores mostram, e o que pedem. A rede e a sessão vivem na família (`ph2d-app-board::live`), que
//! serve os pedidos a cada imagem e escreve aqui o que o ecrã desenha — o mesmo desenho do `take_io`.

use ph2d_board_model::{Board, BoardId, ElementId};

use super::Documents;

/// O que o ecrã pede à família.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveRequest {
    /// Abrir o quadro activo à rede local.
    Host,
    /// Entrar no quadro de `addr` (`None` = o primeiro achado na rede) com o código.
    Join { addr: Option<String>, code: String },
    /// Procurar quadros abertos na rede.
    Discover,
    /// Sair (o convidado) ou fechar a sala (o anfitrião).
    Leave,
    /// «Trazer todos» para a minha vista.
    Summon,
}

/// Em que pé está esta pessoa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LiveRole {
    #[default]
    Off,
    /// A sala está aberta: o código e os endereços a dizer a quem entra.
    Hosting { code: String, addrs: Vec<String> },
    /// À espera da resposta do anfitrião.
    Joining,
    /// Dentro da sala de outra pessoa.
    Guest { host: String },
}

/// Uma pessoa na sala.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivePeer {
    pub peer: u64,
    pub name: String,
    /// O índice na paleta das pessoas.
    pub color: u8,
    pub me: bool,
}

/// Um quadro aberto achado na rede.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveFound {
    pub board: String,
    pub host: String,
    pub addr: String,
}

/// O que se desenha de outra pessoa: o cursor (já interpolado) e a selecção.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveOther {
    pub peer: u64,
    pub name: String,
    pub color: u8,
    pub cursor: Option<[f64; 2]>,
    pub selection: Vec<ElementId>,
}

/// O estado partilhado entre o ecrã e a família.
#[derive(Clone, Debug, Default)]
pub struct LiveView {
    pub role: LiveRole,
    /// O quadro da sala (o anfitrião abriu-o; o convidado recebeu-o).
    pub board: Option<BoardId>,
    pub peers: Vec<LivePeer>,
    pub found: Vec<LiveFound>,
    pub others: Vec<LiveOther>,
    /// Quem estou a seguir (o ecrã pára-o ao fazer pan, zoom ou clicar no quadro).
    pub following: Option<u64>,
    /// Onde está o MEU rato no mundo do quadro da sala (o ecrã escreve-o).
    pub my_cursor: Option<[f64; 2]>,
    requests: Vec<LiveRequest>,
    /// O que só o ecrã sabe (o painel aberto, o relógio da procura, o que se pintou onde).
    pub(crate) panel: LivePanel,
}

/// O estado do painel «Live» no ecrã — nunca da família.
#[derive(Clone, Debug, Default)]
pub(crate) struct LivePanel {
    pub open: bool,
    /// Segundos até à próxima procura na rede (só com o painel aberto e ninguém em sessão).
    pub discover_in: f64,
    /// Os pares de cada avatar pintado, pela ordem (o clique no avatar `i` segue `avatars[i]`).
    pub avatars: Vec<u64>,
    /// O endereço de cada linha de «achados» pintada, pela ordem.
    pub found: Vec<String>,
}

impl Documents {
    /// O ecrã pede uma coisa à família.
    pub fn ask_live(&mut self, req: LiveRequest) {
        self.live_view.requests.push(req);
    }

    /// A família tira os pedidos por servir.
    pub fn take_live(&mut self) -> Vec<LiveRequest> {
        std::mem::take(&mut self.live_view.requests)
    }

    #[must_use]
    pub fn live_view(&self) -> &LiveView {
        &self.live_view
    }

    pub fn live_view_mut(&mut self) -> &mut LiveView {
        &mut self.live_view
    }

    /// O quadro `id`, para a família lhe fundir o que chega da rede.
    pub fn board_mut(&mut self, id: BoardId) -> Option<&mut Board> {
        self.boards.get_mut(id)
    }

    /// Um quadro inteiro que chegou de outra pessoa entra numa ABA nova (com um id desta máquina) e
    /// fica activo.
    pub fn adopt_board(&mut self, mut board: Board) -> BoardId {
        let id = self.create(board.name.clone());
        board.id = id;
        if let Some(b) = self.boards.get_mut(id) {
            *b = board;
        }
        self.activate(Some(id));
        id
    }

    /// A selecção de agora no quadro activo.
    #[must_use]
    pub fn active_selection(&self) -> Vec<ElementId> {
        self.live
            .editor
            .as_ref()
            .map(|e| e.selection().iter().copied().collect())
            .unwrap_or_default()
    }
}
