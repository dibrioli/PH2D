//! ⭐ **A CASCATA de menus** (line/BarraDeMenus, onda 2): os PAIS que ficam abertos ao lado do menu
//! da ponta. O `context_menu` continua a ser a PONTA — quem já o lia não muda —, e os pais valem
//! só enquanto a ponta for a que eles abriram: qualquer outra escrita no slot (um diálogo, outro
//! menu, o fecho) deixa-os órfãos sem ninguém ter de os limpar.

use super::WidgetStore;
use crate::interaction::ContextMenuRequest;
use ph2d_a11y::NodeId;

/// Os pais abertos, da raiz para a ponta: o menu e a LINHA dele que abriu o nível seguinte.
#[derive(Clone, Debug, Default)]
pub struct MenuChain {
    pais: Vec<(ContextMenuRequest, NodeId)>,
    /// A ponta que estes pais abriram.
    ponta: Option<ContextMenuRequest>,
    /// O `Alt` desceu sozinho: onde o ponteiro estava. Soltá-lo sem mais nada abre o menu.
    alt: Option<Option<[f32; 2]>>,
    /// A faixa pediu a busca: a shell reabre-a com os grupos das famílias.
    busca_pedida: bool,
    /// «Esta linha é de um menu aberto?» — a pergunta mora em `screens::hero::menu_cascade` (as
    /// tabelas de linhas); o hero instala-a aqui para o despacho não depender do ecrã.
    linha_de_menu: Option<fn(&WidgetStore, NodeId) -> bool>,
}

impl MenuChain {
    /// Outro menu abriu, ou o menu fechou: os pais saem.
    pub(in crate::interaction) fn solta(&mut self) {
        self.pais.clear();
        self.ponta = None;
    }
}

impl WidgetStore {
    /// Instala a pergunta [`Self::is_open_menu_row`] (o hero, ao nascer).
    pub fn set_menu_row_probe(&mut self, probe: fn(&WidgetStore, NodeId) -> bool) {
        self.menu_chain.linha_de_menu = Some(probe);
    }

    /// `id` é uma linha de um menu aberto (o mesmo id que um botão fora do menu)?
    #[must_use]
    pub fn is_open_menu_row(&self, id: NodeId) -> bool {
        self.menu_chain
            .linha_de_menu
            .is_some_and(|probe| probe(self, id))
    }

    /// Os pais ainda válidos (vazio se a ponta mudou por outra porta).
    fn menu_pais(&self) -> &[(ContextMenuRequest, NodeId)] {
        if self.context_menu.is_some() && self.context_menu == self.menu_chain.ponta {
            &self.menu_chain.pais
        } else {
            &[]
        }
    }

    /// Os menus abertos, da raiz à ponta (vazio sem menu).
    #[must_use]
    pub fn menu_chain(&self) -> Vec<ContextMenuRequest> {
        self.menu_pais()
            .iter()
            .map(|(req, _)| *req)
            .chain(self.context_menu)
            .collect()
    }

    /// A linha do nível `depth` que abriu o nível seguinte — fica acesa enquanto o filho está aberto.
    #[must_use]
    pub fn menu_open_row(&self, depth: usize) -> Option<NodeId> {
        self.menu_pais().get(depth).map(|(_, row)| *row)
    }

    /// Esta linha tem o submenu dela aberto?
    #[must_use]
    pub fn menu_row_has_open_child(&self, row: NodeId) -> bool {
        self.menu_pais().iter().any(|(_, r)| *r == row)
    }

    /// Abre `req` como filho da linha `row` do nível `depth`: os níveis abaixo dele saem, ele fica.
    /// Sem esse nível aberto, `req` abre sozinho.
    pub fn open_submenu_at(&mut self, depth: usize, row: NodeId, req: ContextMenuRequest) {
        let chain = self.menu_chain();
        let Some(pai) = chain.get(depth).copied() else {
            self.open_context_menu(req);
            return;
        };
        let mut pais: Vec<_> = self.menu_pais().iter().copied().take(depth).collect();
        pais.push((pai, row));
        self.context_menu = Some(req);
        self.menu_chain.pais = pais;
        self.menu_chain.ponta = Some(req);
    }

    /// Fecha os níveis abaixo de `depth`: ele passa a ser a ponta.
    pub fn close_submenus_below(&mut self, depth: usize) {
        let chain = self.menu_chain();
        if depth + 1 >= chain.len() {
            return;
        }
        let pais: Vec<_> = self.menu_pais().iter().copied().take(depth).collect();
        self.context_menu = Some(chain[depth]);
        self.menu_chain.pais = pais;
        self.menu_chain.ponta = Some(chain[depth]);
    }

    /// O `Alt` desceu sem outra tecla: fica armado, com o ponteiro onde está.
    pub fn menu_alt_arm(&mut self) {
        self.menu_chain.alt = Some(self.last_pointer());
    }

    /// Outra tecla ou um clique: o `Alt` era de um atalho ou de um gesto, não do menu.
    pub fn menu_alt_disarm(&mut self) {
        self.menu_chain.alt = None;
    }

    /// O `Alt` subiu: `true` se desceu sozinho e o ponteiro não se mexeu entretanto (um `Alt`
    /// segurado para um gesto da tela não abre o menu ao soltar).
    pub fn menu_alt_fire(&mut self) -> bool {
        self.menu_chain.alt.take() == Some(self.last_pointer())
    }

    /// A faixa abriu a busca (o campo ou *Help ▸ Commands…*): a shell completa-a.
    pub fn request_global_palette(&mut self) {
        self.menu_chain.busca_pedida = true;
    }

    /// Toma o pedido da busca (uma vez por quadro, na shell).
    pub fn take_global_palette_request(&mut self) -> bool {
        std::mem::take(&mut self.menu_chain.busca_pedida)
    }
}
