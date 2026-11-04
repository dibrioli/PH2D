//! ⭐ **A mesa de um painel em cartões**: pintar até assentar e provar cada controlo pelo gesto REAL,
//! nos rects pintados (`docs/CartoesUI/11_RECEITA`). Sem isto, cada painel escreveria a sua cópia.

use crate::MockPanelHost;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

/// Uma coluna alta: o que se mede é o painel, não a rolagem.
const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 2600.0,
};
const QUADROS_ATE_ASSENTAR: usize = 60;
const TECLA_ESC: u32 = 0x1B;

pub struct Mesa<P: Panel> {
    pub host: MockPanelHost,
    pub st: P::State,
    pub rects: Vec<(NodeId, Rect)>,
}

impl<P: Panel> Mesa<P> {
    pub fn nova(st: P::State) -> Self {
        let mut mesa = Self {
            host: MockPanelHost::with_panel::<P>(),
            st,
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    /// Os quadros que a mola das alturas pede para os cartões assentarem.
    pub fn assenta(&mut self) {
        self.assenta_em(COLUNA);
    }

    /// O mesmo, numa janela à escolha (uma janela baixa, por exemplo).
    pub fn assenta_em(&mut self, janela: Rect) {
        for _ in 0..QUADROS_ATE_ASSENTAR {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<P>(st, janela),
            );
        }
    }

    #[must_use]
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    pub fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for e in eventos {
            self.host.apply_panel_event::<P>(&mut self.st, e);
        }
        self.host.drained_actions()
    }

    /// O clique no centro do que se pintou; quem responde no centro tem de ser `id`.
    pub fn clica(&mut self, id: NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        self.aplica(ev)
    }

    pub fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }

    /// O scrub real: premir na PEGA e arrastar `dx`, sem largar (o limiar pede dois `Move`).
    pub fn arrasta_a_pega(&mut self, chip: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self
            .rect(chip)
            .unwrap_or_else(|| panic!("{chip:?} não pintado"));
        let pega = handle_rect(r);
        let (x, y) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
        let mut ev = self.ponteiro(PointerKind::Down, x, y);
        ev.extend(self.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(self.ponteiro(PointerKind::Move, x + dx, y));
        ev
    }

    pub fn esc(&mut self) -> Vec<WidgetEvent> {
        let arena = bumpalo::Bump::new();
        let tecla = ph2d_host::KeyEvent {
            keycode: TECLA_ESC,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        dispatch_key(self.host.store_mut(), tecla, &arena).to_vec()
    }

    /// ⭐ O número pelo scrub e pelo Esc: o gesto pede `SetValue(alvo, _)` à ferramenta, o valor muda,
    /// e o Esc devolve-o. Devolve `(arrastado, depois do Esc)`.
    pub fn prova_o_scrub(&mut self, chip: NodeId, alvo: NodeId, dx: f32) -> (f64, f64) {
        let ev = self.arrasta_a_pega(chip, dx);
        let actions = self.aplica(ev);
        let arrastado = ultimo_valor(&actions, alvo)
            .unwrap_or_else(|| panic!("o scrub de {chip:?} não pediu nada: {actions:?}"));
        let ev = self.esc();
        let actions = self.aplica(ev);
        let voltou = ultimo_valor(&actions, alvo)
            .unwrap_or_else(|| panic!("o Esc de {chip:?} não avisou a ferramenta: {actions:?}"));
        assert!(
            (arrastado - voltou).abs() > 1e-4,
            "{chip:?}: o Esc não devolveu ({arrastado} e {voltou})"
        );
        (arrastado, voltou)
    }

    /// ⭐ Escrever o número: o Enter pede `SetValue(alvo, _)`. Devolve o valor pedido.
    pub fn prova_a_escrita(&mut self, chip: NodeId, alvo: NodeId, texto: &str) -> f64 {
        let ev = self.host.type_into_number(chip, texto);
        let actions = self.aplica(ev);
        ultimo_valor(&actions, alvo)
            .unwrap_or_else(|| panic!("escrever em {chip:?} não pediu nada: {actions:?}"))
    }

    /// ⭐ O clique que encaminha `Click(id)` à ferramenta.
    pub fn prova_o_clique(&mut self, id: NodeId) {
        let actions = self.clica(id);
        assert!(
            actions.iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id)
            ),
            "{id:?} não chegou à ferramenta: {actions:?}"
        );
    }

    /// ⭐ O Cancel da faixa e o «×» da lista cancelam a ferramenta.
    pub fn prova_o_cancelar(&mut self, cancel: NodeId) {
        let actions = self.clica(cancel);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, EditorAction::CancelActiveTool)),
            "o Cancel não cancelou: {actions:?}"
        );
    }

    /// ⭐ A faixa dos verbos: abaixo do último cartão e dentro do rect flutuante do painel, senão o
    /// clique nela iria ao canvas.
    pub fn prova_a_faixa(&self, panel: NodeId, secoes: &[NodeId], verbos: &[NodeId]) {
        let lista = self
            .host
            .store()
            .panel_rect(panel)
            .expect("a lista publicou o rect flutuante");
        let fundo = secoes
            .iter()
            .filter_map(|s| self.rect(*s))
            .map(|r| r.y)
            .fold(f32::MIN, f32::max);
        for v in verbos {
            let r = self.rect(*v).unwrap_or_else(|| panic!("{v:?} não pintado"));
            assert!(
                r.y > fundo,
                "{v:?} em y={} não está abaixo dos cartões",
                r.y
            );
            assert!(
                lista.contains(r.x + r.w * 0.5, r.y + r.h * 0.5),
                "{v:?} fora do rect flutuante {lista:?}"
            );
        }
    }

    /// ⭐ Clicar no cabeçalho dobra o cartão, não pede nada à ferramenta, e o corpo deixa de registar.
    pub fn prova_a_dobra(&mut self, secao: NodeId, de_dentro: NodeId) {
        assert!(self.rect(de_dentro).is_some(), "{de_dentro:?} não pintado");
        let r = self.rect(secao).expect("o cabeçalho pinta-se");
        let ev = self.host.click_at(r.x + r.w * 0.3, r.y + 13.0);
        let a = self.aplica(ev);
        assert!(a.is_empty(), "dobrar pediu {a:?}");
        assert!(
            self.host.store().is_collapsed(secao),
            "{secao:?} não dobrou"
        );
        self.assenta();
        assert!(
            self.rect(de_dentro).is_none(),
            "dobrado, o corpo não regista clique"
        );
    }

    /// Os ids que a última pintura registou.
    #[must_use]
    pub fn pintados(&self) -> Vec<NodeId> {
        self.rects.iter().map(|(id, _)| *id).collect()
    }
}

/// O último `SetValue` a `alvo` numa lista de ações.
#[must_use]
pub fn ultimo_valor(actions: &[EditorAction], alvo: NodeId) -> Option<f64> {
    actions.iter().rev().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)) if *id == alvo => Some(*v),
        _ => None,
    })
}
