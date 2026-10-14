//! **O quadro ao vivo, do lado do ECRÃ** (ADR-0181): o que o painel «Ao vivo», os avatares e os
//! cursores mostram, e o que pedem. A rede e a sessão vivem na família (`ph2d-app-board::live`), que
//! serve os pedidos a cada imagem e escreve aqui o que o ecrã desenha — o mesmo desenho do `take_io`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, BoardId, ElementId, VoteSetup};

use super::Documents;

/// O que o ecrã pede à família.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveRequest {
    /// Abrir o quadro activo à rede local.
    Host,
    /// Entrar no quadro de `addr` (`None` = o primeiro achado na rede) com o código.
    Join { addr: Option<String>, code: String },
    /// ADR-0187 — abrir a sala também à internet e copiar o convite (de novo, se já estiver aberta).
    InviteInternet,
    /// ADR-0187 — entrar pelo link de convite colado.
    JoinLink { link: String },
    /// Procurar quadros abertos na rede.
    Discover,
    /// Sair (o convidado) ou fechar a sala (o anfitrião).
    Leave,
    /// «Trazer todos» para a minha vista.
    Summon,
    /// Liberar as portas do quadro ao vivo na firewall, só para a rede de casa (pede a senha).
    FirewallAllow,
    /// Desfazer o que o PH2D liberou.
    FirewallRemove,
    /// ADR-0182 — ligar o modo privado para todos.
    PrivateOn {
        anonymous: bool,
        anyone_can_end: bool,
    },
    /// «Turn off private mode for all» (revela depois da contagem).
    PrivateOff,
    /// «Anyone can reveal their stickies» (só quem ligou).
    AnyoneCanReveal(bool),
    /// «Reveal all my sticky notes».
    RevealMine,
    /// Abrir uma votação entre pessoas sobre estes alvos.
    VoteStart {
        setup: VoteSetup,
        targets: BTreeSet<ElementId>,
    },
    /// «End for all».
    VoteEnd,
    /// O meu boletim inteiro (a cada mudança).
    Ballot {
        round: u32,
        votes: BTreeMap<ElementId, u32>,
        done: bool,
    },
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

/// ADR-0187 — a entrada pela internet do anfitrião, como o painel a mostra.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LiveInternet {
    #[default]
    Off,
    /// A preparar (o endereço de fora, o ponto de encontro).
    Preparing,
    /// Aberta: o link foi copiado.
    Ready { link: String },
    /// Não abriu (a frase do erro).
    Failed(String),
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
    /// W12 — «Make names anonymous»: sem nome (já vem vazio) e na tinta neutra.
    pub anonymous: bool,
}

/// A firewall desta máquina, como a família a leu (ADR-0181 §8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiveFirewall {
    /// Uma firewall que o PH2D sabe mudar está a bloquear esta rede: o botão «Permitir».
    pub can_allow: bool,
    /// O PH2D liberou-a antes: o botão «Remover».
    pub can_remove: bool,
    /// À espera da senha do sistema (ou do comando).
    pub busy: bool,
}

/// ADR-0182 — a facilitação da sala como o ECRÃ a desenha (a família escreve-a a cada imagem).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LiveFac {
    pub private: Option<LivePrivate>,
    /// Notas de OUTROS com o texto escondido de mim: desenham-se sem o texto, com o olho fechado.
    pub hidden_to_me: BTreeSet<ElementId>,
    /// As MINHAS notas escondidas dos outros: o texto e o olho fechado.
    pub mine_hidden: BTreeSet<ElementId>,
    pub vote: Option<LiveVote>,
}

/// O modo privado ligado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivePrivate {
    /// Quem o ligou (o nome) e se fui eu.
    pub by: String,
    pub by_me: bool,
    pub anonymous: bool,
    pub anyone_can_end: bool,
    pub anyone_can_reveal: bool,
    /// Posso desligá-lo (fui eu, sou o anfitrião, ou qualquer um pode).
    pub may_end: bool,
    /// A contagem para revelar (ms que faltam).
    pub reveal_in_ms: Option<u64>,
}

/// A votação entre pessoas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveVote {
    pub round: u32,
    /// Quem a iniciou (o nome) e se fui eu.
    pub by: String,
    pub by_me: bool,
    /// Posso acabá-la («End for all»).
    pub may_end: bool,
    pub setup: VoteSetup,
    pub targets: BTreeSet<ElementId>,
    pub left_ms: Option<u64>,
    /// Quantos disseram «I'm done», de quantos estão na sala.
    pub done: usize,
    pub people: usize,
    /// Só para quem iniciou: o nome, os votos usados e se acabou (Miro: «Participants»).
    pub used: Vec<(String, u32, bool)>,
    pub ended: bool,
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
    pub firewall: LiveFirewall,
    /// ADR-0187 — a entrada pela internet (só o anfitrião).
    pub internet: LiveInternet,
    /// ADR-0182 — o modo privado e a votação da sala, como os desenho.
    pub fac: LiveFac,
    requests: Vec<LiveRequest>,
    /// O quadro da sala que saiu das abas: a família fecha a sala com ele e larga-o.
    parting: Option<Board>,
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

impl LiveView {
    /// O quadro da sala sai das abas: fica para a família fechar a sala com ele (`Leave`).
    pub(crate) fn part(&mut self, board: Board) {
        self.parting = Some(board);
        self.requests.push(LiveRequest::Leave);
    }
}

impl Documents {
    /// A família tira o quadro da sala que saiu das abas (ver [`LiveView::part`]).
    pub fn take_parting(&mut self) -> Option<Board> {
        self.live_view.parting.take()
    }

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
