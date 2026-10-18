//! ⭐ **Os DOCUMENTOS do projecto além da cena** — hoje, os quadros (MiroClone), e qual deles a
//! barra de cima tem activo.
//!
//! > *«Os quadros não devem aparecer na hierarquia mas devem aparecer como abas na barra superior
//! > do APP.»* — o dono, 2026-10-05 (`docs/MiroClone/02_plano.md` §1.1).
//!
//! ⚠️ **Aqui e não na shell**, pela razão do [`crate::screens::HeroScreen::input_map`]: as abas são
//! pintadas por `paint_hero_screen`, que só recebe o `HeroScreen`. É dado de PROJECTO: a shell lê-o
//! ao gravar ([`Documents::boards`]) e escreve-o ao carregar
//! ([`crate::screens::hero::document_tabs::load`]).

pub use ph2d_board_model::BoardSet;
use ph2d_board_model::{Board, BoardId, History};
use std::collections::BTreeMap;

#[path = "documents_fac.rs"]
pub(crate) mod fac;
#[path = "documents_live.rs"]
pub mod live;
#[path = "documents_nav.rs"]
pub(crate) mod nav;
#[path = "documents_surface.rs"]
mod surface;
#[path = "documents_team.rs"]
pub mod team;

/// Os quadros e a aba activa.
#[derive(Debug, Default)]
pub struct Documents {
    boards: BoardSet,
    /// `None` = a aba `Cena`.
    active: Option<BoardId>,
    /// Âncora do arrasto da vista do quadro (px de ecrã), enquanto ele dura.
    pub(crate) pan_from: Option<[f64; 2]>,
    /// O quadro cuja aba é, agora, o campo de renomear.
    pub(crate) renaming: Option<BoardId>,
    /// Uma aba de quadro com o dedo em cima — ver `document_tabs_menu::pointer`.
    pub(crate) tab_drag: Option<TabDrag>,
    /// O que vive enquanto se edita um quadro (editor, desfazer por quadro, texto moldado).
    pub(crate) live: BoardLive,
    /// O quadro ao vivo (ADR-0181): o que o painel mostra e pede.
    pub(crate) live_view: live::LiveView,
    /// As famílias com documentos em aba (o editor de scripts) — `crate::doc_surface`.
    pub(crate) surfaces: crate::doc_surface::Surfaces,
    /// A aba de família activa; exclusiva com `active` (as duas `None` = a `Cena`).
    pub(crate) surface_active: Option<crate::doc_surface::SurfaceTab>,
}

/// ⭐ **O estado de SESSÃO dos quadros** — nada disto vai para o ficheiro.
#[derive(Default)]
pub(crate) struct BoardLive {
    /// O editor do quadro activo (nasce no 1.º gesto: o estilo de nascença vem do tema).
    pub(crate) editor: Option<ph2d_board_edit::Editor>,
    /// O desfazer de CADA quadro (plano §1.4: `Ctrl+Z` num quadro nunca desfaz outro nem a cena).
    pub(crate) histories: BTreeMap<BoardId, History>,
    pub(crate) render_cache: ph2d_board_render::RenderCache,
    /// O último carregar no quadro `(instante ns, x, y)` — o duplo-clique.
    pub(crate) last_down: Option<(u128, [f32; 2])>,
    /// `Espaço` em baixo: arrastar move a vista (o idioma do Figma/Miro/Excalidraw).
    pub(crate) space: bool,
    /// A grelha de todas as formas, aberta pelo «mais formas» da barra curta.
    pub(crate) shapes_open: bool,
    /// A grelha «Forma» da barra de contexto (trocar o tipo das formas), aberta para ESTA selecção
    /// e ferramenta.
    /// W23 — o comando que o menu do botão direito numa forma enfileirou (corre na pintura seguinte).
    pub(crate) menu_command: Option<ph2d_board_edit::Command>,
    /// W23 — a grelha de todas as pontas, aberta para ESTA selecção e para a ponta `0`/`1`.
    pub(crate) heads: Option<(
        std::collections::BTreeSet<ph2d_board_model::ElementId>,
        usize,
    )>,
    pub(crate) reshape: Option<(
        std::collections::BTreeSet<ph2d_board_model::ElementId>,
        ph2d_board_edit::Tool,
    )>,
    /// A galeria dos modelos, aberta pelo botão «Templates» da barra curta (W6).
    pub(crate) templates_open: bool,
    /// ⭐ O quadro TOMOU o carregar do gesto em curso: só então o largar e o mover são dele. Um gesto
    /// sem esta marca está PENDURADO (o largar perdeu-se) e nunca engole o clique de um botão
    /// (report do dono, 07/10: *«com a caneta não consigo clicar noutra ferramenta até apertar Esc»*).
    pub(crate) gesture: bool,
    /// A última recusa do editor (a chave i18n da frase) e quando chegou — o quadro pinta-a por
    /// cima dele enquanto é fresca (W5: nunca só o terminal).
    pub(crate) notice: Option<(&'static str, std::time::Instant)>,
    /// A navegação (W5): minimapa, painel de Frames, busca, etiquetas, links.
    pub(crate) nav: nav::Nav,
    /// A facilitação (W7): cronómetros, votações, painéis.
    pub(crate) fac: fac::Fac,
    /// A equipa e a saída (W8): comentar, versões, apresentar, exportar.
    pub(crate) team: team::Team,
    /// W18 — o carregar em curso é do tocador de um vídeo (o mover e o largar também). W20: só no ecrã
    /// inteiro; no quadro o carregar num vídeo espera pelo largar ([`Self::video_click`]).
    pub(crate) video_press: bool,
    /// W20 — um carregar num vídeo à espera do largar: sem arrasto é um clique (activar / o tocador), com
    /// arrasto foi do editor (moveu o vídeo).
    pub(crate) video_click: Option<VideoClick>,
    /// W20 — o Espaço foi carregado com um vídeo activo e nenhum carregar do rato veio depois: no largar
    /// é tocar/pausar (o atalho do YouTube); com um arrasto pelo meio foi o pan do quadro.
    pub(crate) space_tap: bool,
    /// W20 — o trilho de cada janela de vídeo que navegou pelo «Mais vídeos» (as setas e o «voltar ao
    /// original»). Só da sessão: o documento guarda o vídeo de agora.
    pub(crate) video_trails: std::collections::BTreeMap<
        (ph2d_board_model::BoardId, ph2d_board_model::ElementId),
        VideoTrail,
    >,
    /// W18 — o botão «Video» da barra pediu o link da área de transferência (o app lê-a).
    pub(crate) video_ask: bool,
}

/// ⭐ W20 — um carregar num vídeo à espera do largar (o dono: «segurar e arrastar move, apenas clicar dá
/// play/pause»). O carregar segue para o editor (seleccionar, e o arrasto move o vídeo); o largar sem
/// arrasto é o clique (`board_video::pointer_up`). O oráculo também decide no largar (`sonda_w20.mjs` A2).
#[derive(Clone, Copy, Debug)]
pub(crate) struct VideoClick {
    pub(crate) id: ph2d_board_model::ElementId,
    /// Onde carregou (ecrã).
    pub(crate) at: [f64; 2],
    /// Dentro do tocador vivo do vídeo activo: o ponto em píxeis do tocador (o clique vai para ele). `None`
    /// = o clique activa o vídeo.
    pub(crate) player: Option<[f64; 2]>,
    /// Passou do limiar de arrasto da casa: o gesto foi do editor.
    pub(crate) dragged: bool,
    /// W21 — o relógio do carregar (`board_view::Input::now_ns`): segurado parado além do clique longo, o
    /// largar não é clique.
    pub(crate) at_ns: u128,
    /// W21 — com outra ferramenta na mão: o editor ainda não viu o carregar (recebe-o se o gesto deixar de
    /// ser um clique).
    pub(crate) tool: bool,
}

/// ⭐ W20 — os vídeos por onde uma janela passou, por ordem (o 1.º é o ORIGINAL), e onde está. A verdade do
/// vídeo de agora é o DOCUMENTO: `at` re-acha-se nele a cada uso (o `Ctrl+Z` também anda no trilho), e um
/// vídeo que não está no trilho (mudado por outra via) apaga-o.
#[derive(Clone, Debug, Default)]
pub(crate) struct VideoTrail {
    pub(crate) items: Vec<ph2d_board_model::Video>,
    pub(crate) at: usize,
}

impl VideoTrail {
    /// Re-acha `at` no vídeo de agora; `false` = ele não está no trilho.
    pub(crate) fn sync(&mut self, now: &ph2d_board_model::Video) -> bool {
        if self.items.get(self.at) == Some(now) {
            return true;
        }
        match self.items.iter().position(|v| v == now) {
            Some(i) => {
                self.at = i;
                true
            }
            None => false,
        }
    }
}

impl std::fmt::Debug for BoardLive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("histories", &self.histories.len())
            .field("space", &self.space)
            .finish_non_exhaustive()
    }
}

/// O dedo desceu sobre a aba de `board` em `start` e está em `cursor` (px de ecrã).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TabDrag {
    pub board: BoardId,
    pub start: (f32, f32),
    pub cursor: (f32, f32),
}

impl Documents {
    /// W7 — a campainha de um cronómetro de quadro que chegou a zero (F6), tirada UMA vez: a shell
    /// toca-a. É um ALARME que o facilitador armou, não um som de UI.
    pub fn take_chime(&mut self) -> bool {
        std::mem::take(&mut self.live.fac.chime)
    }

    /// W17 — a chave do aviso que a faixa do quadro mostra (a última recusa do editor, a imagem que
    /// não se lê), enquanto não é trocada.
    #[must_use]
    pub fn notice(&self) -> Option<&'static str> {
        self.live.notice.map(|(k, _)| k)
    }

    /// W18 — os vídeos do desenho do quadro (o app dá-lhes vida: miniaturas, o tocador, os quadros).
    pub fn videos_mut(&mut self) -> &mut ph2d_board_render::Videos {
        &mut self.live.render_cache.videos
    }

    /// W18 — o botão «Video» foi carregado desde a última vez (o app lê o link da área de transferência).
    pub fn take_video_ask(&mut self) -> bool {
        std::mem::take(&mut self.live.video_ask)
    }

    /// W18 — um aviso na faixa do quadro, pela chave i18n (como as recusas do editor).
    pub fn warn(&mut self, key: &'static str) {
        self.live.notice = Some((key, std::time::Instant::now()));
    }

    /// W17 — há imagens do quadro a descodificar.
    #[must_use]
    pub fn images_pending(&self) -> bool {
        self.live.render_cache.images.pending()
    }

    /// Todos os quadros, na ordem das abas — o que a shell grava.
    #[must_use]
    pub fn boards(&self) -> &BoardSet {
        &self.boards
    }

    /// O quadro da aba activa, ou `None` se a aba activa é a `Cena`.
    #[must_use]
    pub fn active(&self) -> Option<BoardId> {
        self.active
    }

    #[must_use]
    pub fn active_board(&self) -> Option<&Board> {
        self.active.and_then(|id| self.boards.get(id))
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        self.active.and_then(|id| self.boards.get_mut(id))
    }

    /// Torna activa a aba de `id` (`None` = a `Cena`). Um id que não existe cai na `Cena`.
    /// Sair de um quadro termina o texto que se escrevia nele e esquece a selecção.
    pub fn activate(&mut self, id: Option<BoardId>) {
        let id = id.filter(|id| self.boards.get(*id).is_some());
        if id != self.active {
            self.leave_board();
            // W18 — os ids são por quadro: o vídeo activo de um não é o de outro.
            self.live.render_cache.videos.active = None;
            self.live.render_cache.videos.frame = None;
        }
        self.active = id;
        self.surface_active = None;
        self.pan_from = None;
    }

    fn leave_board(&mut self) {
        let Some(prev) = self.active else {
            return;
        };
        let live = &mut self.live;
        if let (Some(ed), Some(board)) = (live.editor.as_mut(), self.boards.get_mut(prev)) {
            let h = live.histories.entry(prev).or_default();
            ed.commit_text(&mut board.doc, h);
            ed.end_crop(&board.doc, h);
            ed.cancel_gesture(&mut board.doc);
            ed.select(&board.doc, []);
        }
    }

    /// O quadro activo, o histórico dele e o resto do estado de sessão — juntos, para um gesto.
    pub(crate) fn active_parts(&mut self) -> Option<(&mut Board, &mut BoardLive)> {
        let id = self.active?;
        let board = self.boards.get_mut(id)?;
        Some((board, &mut self.live))
    }

    /// Cria um quadro vazio no fim das abas e torna-o activo.
    pub fn create(&mut self, name: String) -> BoardId {
        let id = self.boards.create(name);
        self.activate(Some(id));
        id
    }

    /// Troca todos os quadros (ao carregar um projecto) e volta à `Cena`.
    pub fn replace(&mut self, boards: BoardSet) {
        self.leave_board();
        let old = std::mem::replace(&mut self.boards, boards);
        // A sala fecha com o quadro dela — os ids recomeçam, e o do projecto novo seria OUTRO quadro.
        if let Some(b) = self.live_view.board.and_then(|id| old.get(id)) {
            self.live_view.part(b.clone());
        }
        self.renaming = None;
        self.tab_drag = None;
        self.live.histories.clear();
        self.active = None;
        self.surface_active = None;
        self.pan_from = None;
        // W7 — os ids de quadro recomeçam em cada projecto: nada da facilitação do anterior (um
        // relógio, uma sessão de voto) pode cair sobre o quadro novo com o mesmo id (auditoria W7).
        self.live.fac = fac::Fac::default();
        self.live.team.reset();
        if let Some(ed) = self.live.editor.as_mut() {
            ed.put_vote(None);
        }
    }

    /// Muda o nome. `false` se `id` não existe.
    pub fn rename(&mut self, id: BoardId, name: String) -> bool {
        self.boards.rename(id, name)
    }

    /// Cópia de `id` logo à direita dele, e a cópia fica activa.
    pub fn duplicate(&mut self, id: BoardId, name: String) -> Option<BoardId> {
        let copy = self.boards.duplicate(id, name)?;
        self.activate(Some(copy));
        Some(copy)
    }

    /// Leva a aba de `id` para a posição `to`. `false` se `id` não existe.
    pub fn move_tab(&mut self, id: BoardId, to: usize) -> bool {
        self.boards.move_tab(id, to)
    }

    /// Apaga o quadro. Se era o activo, a aba que fica no lugar dele abre (a da direita, senão a
    /// da esquerda, senão a `Cena`) — o idioma das abas de um navegador.
    pub fn remove(&mut self, id: BoardId) -> Option<Board> {
        let at = self.boards.boards().iter().position(|b| b.id == id)?;
        if self.active == Some(id) {
            self.leave_board();
        }
        let gone = self.boards.remove(id)?;
        // A sala deste quadro fecha como «End session»: a família ainda precisa dele para revelar o
        // escondido e acabar a votação antes do adeus (ADR-0182 §5).
        if self.live_view.board == Some(id) {
            self.live_view.part(gone.clone());
        }
        self.live.histories.remove(&id);
        // W7 — o relógio, o painel e a sessão do quadro apagado vão com ele (a campainha não toca
        // por um quadro que já não existe — auditoria W7).
        if self.live.fac.forget(id)
            && let Some(ed) = self.live.editor.as_mut()
        {
            ed.put_vote(None);
        }
        self.live.team.forget(id);
        if self.renaming == Some(id) {
            self.renaming = None;
        }
        if self.active == Some(id) {
            let list = self.boards.boards();
            let next = list.get(at).or_else(|| list.last()).map(|b| b.id);
            self.activate(next);
        }
        Some(gone)
    }
}
