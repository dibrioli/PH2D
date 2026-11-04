//! ⭐ A barra e as migalhas do grafo em cartões: o clique real pede o mesmo que `PH2D_CARDS=0`, e o grafo
//! não perde um pixel. Mocks e gestos reais, nos rects pintados.

use crate::paint_chrome::*;
use crate::snapshot::{Crumb, GraphViewSnapshot, drain_intents, set_current_motion_graph};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

const ORDINAIS: [u16; 9] = [
    CHROME_SPLIT_H,
    CHROME_SPLIT_V,
    CHROME_FIT,
    CHROME_BACKDROP,
    CHROME_KNIFE,
    CHROME_PROBE,
    CHROME_GROUP,
    CHROME_ARRANGE,
    CHROME_NODE_HELP,
];

fn layout() -> HeroLayout {
    HeroLayout::for_viewport_split(
        JANELA,
        false,
        ph2d_editor_core::screens::layout::rail_w(),
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    )
}

fn documento(dentro: bool) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: dentro.then_some(7),
        breadcrumb: if dentro {
            vec![
                Crumb {
                    level: None,
                    title: "Root".to_owned(),
                },
                Crumb {
                    level: Some(7),
                    title: "Group".to_owned(),
                },
            ]
        } else {
            Vec::new()
        },
        nodes: Vec::new(),
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

struct Palco {
    host: MockPanelHost,
    st: MotionGraphPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Palco {
    fn novo(cartoes: bool, dentro: bool) -> Self {
        ferramenta::force_cards_on_this_thread(Some(cartoes));
        set_current_motion_graph(Some(documento(dentro)));
        let _ = drain_intents();
        let mut p = Self {
            host: MockPanelHost::with_panel::<crate::MotionGraphPanel>(),
            st: MotionGraphPanelState::default(),
            rects: Vec::new(),
        };
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        self.rects =
            self.host
                .paint_with_layout::<crate::MotionGraphPanel>(&mut self.st, layout(), JANELA);
    }

    fn rect(&self, id: NodeId) -> Rect {
        self.rects
            .iter()
            .rev()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} não pintado"))
    }

    /// O clique no centro, e o quadro seguinte, que é onde o painel drena o gesto.
    fn clica(&mut self, id: NodeId) -> String {
        let r = self.rect(id);
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(x, y), Some(id), "outro id no centro");
        let _ = self.host.click_at(x, y);
        self.pinta();
        format!(
            "{:?} knife={} probe={} fitted={}",
            drain_intents(),
            self.st.knife_armed,
            self.st.probe_armed,
            self.st.fitted
        )
    }
}

/// ⭐⭐ Cada um dos nove chips pede, ao clique real, o que pedia com `PH2D_CARDS=0`, e os que armam armam.
/// *Mutação: `hits.push` a sair de `cartoes::draw` ⇒ o clique não vira gesto.*
#[test]
fn cada_chip_pede_o_mesmo_que_o_painel_de_antes() {
    let mut mudaram = 0;
    for id in ORDINAIS {
        let hit = chrome_hit_id_for_tests(id);
        let antes = Palco::novo(false, false).clica(hit);
        let depois = Palco::novo(true, false).clica(hit);
        assert_eq!(antes, depois, "o chip {id} mudou de efeito");
        let quieto = "[] knife=false probe=false fitted=false";
        mudaram += usize::from(antes != quieto);
    }
    assert!(
        mudaram >= 5,
        "vácuo: só {mudaram} chips fizeram alguma coisa"
    );
    let mut p = Palco::novo(true, false);
    let _ = p.clica(chrome_hit_id_for_tests(CHROME_KNIFE));
    assert!(p.st.knife_armed, "a faca não armou");
}

/// ⭐ As migalhas continuam a levar para fora. *Mutação: não registar o hit ⇒ «não pintado».*
#[test]
fn a_migalha_leva_ao_nivel_de_cima() {
    let hit = crate::paint::fnv_id("motion_graph/crumb/0");
    let antes = Palco::novo(false, true).clica(hit);
    let depois = Palco::novo(true, true).clica(hit);
    assert_eq!(antes, depois);
    assert!(antes.contains("GoToLevel"), "{antes}");
}

/// ⭐⭐ O grafo não perde um pixel: a barra e as migalhas flutuam sobre ele, e cabem nele.
#[test]
fn o_grafo_nao_perde_altura_e_a_barra_cabe_nele() {
    let area = |p: &Palco| p.rect(crate::hits::bg_hit_id());
    let antes = area(&Palco::novo(false, false));
    let novo = Palco::novo(true, false);
    assert_eq!(antes, area(&novo), "o fundo do grafo mudou");
    for id in ORDINAIS {
        let r = novo.rect(chrome_hit_id_for_tests(id));
        assert_eq!(r.h, m_field_h(), "o chip {id} não tem a altura do campo");
        assert!(r.y + r.h <= antes.y + antes.h, "o chip {id} sai do grafo");
        assert!(r.x >= antes.x, "o chip {id} sai do grafo");
    }
}

fn m_field_h() -> f32 {
    ph2d_editor_core::panel::cards::banda::metrics::FIELD_H
}

/// ⭐ O nó muda de tinta só em cartões; a faixa de params idem. *Mutação: tinta fixa ⇒ vermelho.*
#[test]
fn a_tinta_do_no_segue_o_interruptor() {
    let _ = Palco::novo(false, false);
    assert_eq!(crate::paint::tinta_do_corpo(), ph2d_tokens::ColorToken::Bg2);
    assert_eq!(crate::paint::tinta_da_faixa(), ph2d_tokens::ColorToken::Bg0);
    let _ = Palco::novo(true, false);
    assert_eq!(
        crate::paint::tinta_do_corpo(),
        ph2d_tokens::ColorToken::KanbanCard
    );
    assert_eq!(
        crate::paint::tinta_da_faixa(),
        ph2d_tokens::ColorToken::CardField
    );
}
