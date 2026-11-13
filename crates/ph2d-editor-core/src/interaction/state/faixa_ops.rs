//! ⭐ **O estado da BUSCA QUE RESPONDE** (line/BarraDeMenus, onda 3): o chip que ensina o atalho, as
//! linhas do *Open Recent*, os objetos da cena e os campos dos cartões que a busca oferece, o pedido
//! de abrir uma ação no Input Map e o caminho do «show me where it lives». Quem os escreve: o hero
//! (`screens::hero::menu_teach`, `open_recent`, `menu_reveal`), a Hierarquia e os cartões ao pintar.

use super::WidgetStore;
use crate::interaction::InteractiveState;
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use std::cell::RefCell;

/// O chip da faixa: «`key` is `name`», do comando `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeachChip {
    pub id: NodeId,
    pub key: String,
    pub name: String,
}

/// Uma linha do *Open Recent*: o id, o rótulo pintado e o caminho que abre (`None` = a linha cinzenta
/// «No Recent Projects» ou o *Clear Recent*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentRow {
    pub id: NodeId,
    pub label: String,
    pub path: Option<String>,
}

/// Um campo pintado num cartão: a secção (o cartão), o título dele, o rótulo e o id do controlo.
#[derive(Clone, Debug, PartialEq)]
pub struct CardFieldSeen {
    pub card: NodeId,
    pub title: String,
    pub label: String,
    pub id: NodeId,
    /// O topo da linha DENTRO da lista (0 = o topo do primeiro cartão): a rolagem que o mostra.
    pub y: f32,
}

/// O que os cartões pintaram no último quadro (o título de cada um e os campos com rótulo).
#[derive(Clone, Debug, Default)]
pub struct CardDirectory {
    /// O quadro (contador do hero) em que estas linhas foram pintadas.
    pub frame: u64,
    pub cards: Vec<(NodeId, String)>,
    pub fields: Vec<CardFieldSeen>,
    /// O cartão que se pinta agora (o título que os campos herdam).
    current: Option<(NodeId, String)>,
    /// Onde a lista começa no ecrã neste quadro (já rolada).
    origin: f32,
}

/// Ver o cabeçalho do módulo.
#[derive(Clone, Debug, Default)]
pub struct FaixaState {
    chip: Option<TeachChip>,
    recent: Vec<RecentRow>,
    /// A ação que a busca escolheu: a aba do Input Map abre-a.
    input_map_focus: Option<ph2d_input::ActionId>,
    /// Os objetos da cena pela ordem da Hierarquia: a linha e o nome.
    objects: Vec<(NodeId, String)>,
    cards: RefCell<CardDirectory>,
    /// O quadro que se pinta agora (o hero conta-o).
    frame: u64,
    /// O caminho até à linha a mostrar: as linhas-cascata por abrir e a linha final.
    reveal: Option<(Vec<NodeId>, NodeId)>,
    /// O que pisca sem pintor próprio (a linha da Hierarquia, o campo do cartão): o anel genérico.
    ring: Option<NodeId>,
    /// A janela é do app (faixa única): `Some(maximizada)`; `None` = barra do sistema.
    window_frame: Option<bool>,
    /// O pedido dos botões da janela, à espera da shell.
    window_request: Option<super::window_frame::WindowRequest>,
    /// O último Down no fundo da faixa (o duplo clique que maximiza).
    window_press: super::window_frame::WindowPress,
}

impl WidgetStore {
    /// O chip que ensina o atalho, se há um à vista.
    #[must_use]
    pub fn teach_chip(&self) -> Option<&TeachChip> {
        self.faixa.chip.as_ref()
    }

    /// Mostra `chip` (substitui o que houver: nunca dois).
    pub fn set_teach_chip(&mut self, chip: Option<TeachChip>) {
        self.faixa.chip = chip;
    }

    /// Publica as linhas do *Open Recent* e regista cada uma como botão.
    pub fn set_recent_rows(&mut self, rows: Vec<RecentRow>) {
        for r in &rows {
            if self.button_state(r.id).is_none() {
                let state = ButtonState::Normal;
                self.register(r.id, InteractiveState::Button { state });
            }
        }
        self.faixa.recent = rows;
    }

    /// As linhas do *Open Recent* publicadas.
    #[must_use]
    pub fn recent_rows(&self) -> &[RecentRow] {
        &self.faixa.recent
    }

    /// A busca escolheu a ação `id`: a aba do Input Map abre e mostra-a.
    pub fn request_input_map_action(&mut self, id: ph2d_input::ActionId) {
        self.input_map_tab_request = true;
        self.faixa.input_map_focus = Some(id);
    }

    /// Lê e limpa a ação pedida (a família do Input Map, no `frame`).
    pub fn take_input_map_action(&mut self) -> Option<ph2d_input::ActionId> {
        self.faixa.input_map_focus.take()
    }

    /// Publica os objetos da cena (a Hierarquia, a cada sincronização).
    pub fn set_scene_objects(&mut self, objects: Vec<(NodeId, String)>) {
        self.faixa.objects = objects;
    }

    /// Os objetos da cena: a linha da Hierarquia e o nome.
    #[must_use]
    pub fn scene_objects(&self) -> &[(NodeId, String)] {
        &self.faixa.objects
    }

    /// Começa um quadro de pintura (o hero): o que os cartões pintarem a seguir é deste quadro.
    pub fn begin_card_directory_frame(&mut self) {
        self.faixa.frame = self.faixa.frame.wrapping_add(1);
    }

    /// Um cartão `card` com o título `title` começa a pintar o corpo.
    pub fn card_painting(&self, card: NodeId, title: &str) {
        let mut d = self.fresh_directory();
        if !d.cards.iter().any(|(c, _)| *c == card) {
            d.cards.push((card, title.to_owned()));
        }
        d.current = Some((card, title.to_owned()));
    }

    /// A lista de cartões do Inspector começa em `origin` (ecrã, já rolada) neste quadro.
    pub fn card_list_origin(&self, origin: f32) {
        self.fresh_directory().origin = origin;
    }

    /// Um campo com rótulo foi pintado (linha em `y`, ecrã) no cartão que se pinta agora.
    pub fn card_field_painted(&self, label: &str, id: NodeId, y: f32) {
        let mut d = self.fresh_directory();
        let Some((card, title)) = d.current.clone() else {
            return;
        };
        if label.trim().is_empty() || d.fields.iter().any(|f| f.id == id) {
            return;
        }
        let origin = d.origin;
        d.fields.push(CardFieldSeen {
            card,
            title,
            label: label.to_owned(),
            id,
            y: y - origin,
        });
    }

    /// O corpo do cartão acabou: os campos seguintes não são dele.
    pub fn card_painted(&self) {
        self.fresh_directory().current = None;
    }

    /// O que os cartões pintaram no último quadro completo (ou neste, se já começou).
    #[must_use]
    pub fn card_directory(&self) -> CardDirectory {
        let d = self.faixa.cards.borrow();
        if d.frame + 1 >= self.faixa.frame {
            d.clone()
        } else {
            CardDirectory::default()
        }
    }

    /// A gravação deste quadro (a do anterior sai quando o primeiro cartão deste pinta).
    fn fresh_directory(&self) -> std::cell::RefMut<'_, CardDirectory> {
        let mut d = self.faixa.cards.borrow_mut();
        if d.frame != self.faixa.frame {
            *d = CardDirectory {
                frame: self.faixa.frame,
                ..CardDirectory::default()
            };
        }
        d
    }

    /// ⭐ Faz piscar `id` ONDE ESTIVER pintado — o anel sobre o rectângulo dele no quadro, para os
    /// alvos que não pintam o piscar por si (a linha de um objeto, um campo de cartão).
    pub fn flash_anywhere(&mut self, id: NodeId) {
        self.flash(id);
        self.faixa.ring = Some(id);
    }

    /// O anel genérico agora: o id e a intensidade, enquanto [`Self::flash_anywhere`] pisca.
    #[must_use]
    pub fn ring_anywhere(&self, steady: bool) -> Option<(NodeId, f32)> {
        let want = self.faixa.ring?;
        self.flashing(steady).filter(|(id, _)| *id == want)
    }

    /// Mostra onde `row` mora: abre `path` (as linhas-cascata, da raiz para baixo) e acende-a.
    pub fn plan_menu_reveal(&mut self, path: Vec<NodeId>, row: NodeId) {
        self.faixa.reveal = Some((path, row));
    }

    /// O caminho por abrir.
    pub fn menu_reveal_mut(&mut self) -> &mut Option<(Vec<NodeId>, NodeId)> {
        &mut self.faixa.reveal
    }

    /// A shell, a cada quadro: a janela é do app (`Some(maximizada)`) ou do sistema (`None`).
    pub fn set_window_frame(&mut self, frame: Option<bool>) {
        self.faixa.window_frame = frame;
    }

    /// Ver [`Self::set_window_frame`].
    #[must_use]
    pub fn window_frame(&self) -> Option<bool> {
        self.faixa.window_frame
    }

    /// Um botão da janela foi clicado.
    pub fn request_window(&mut self, r: super::window_frame::WindowRequest) {
        self.faixa.window_request = Some(r);
    }

    /// Lê e limpa o pedido (a shell, depois de cada evento).
    pub fn take_window_request(&mut self) -> Option<super::window_frame::WindowRequest> {
        self.faixa.window_request.take()
    }

    /// A memória do Down da faixa ([`super::window_frame::press`]).
    pub fn window_press_mut(&mut self) -> &mut super::window_frame::WindowPress {
        &mut self.faixa.window_press
    }
}
