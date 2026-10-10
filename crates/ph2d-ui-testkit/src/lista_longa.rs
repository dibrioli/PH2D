//! ⭐⭐ **A PROVA de que uma lista de dropdown LONGA rola** — a roda, o polegar e o clique REAIS,
//! pelo despacho do produto, sobre o que um painel pintou pela porta
//! `ph2d_editor_core::panel::paint_dropdown_list`.
//!
//! ⚠️ Uma prova só, e não uma por painel: a lei é UMA porta, e o que cada painel pode errar é o que
//! lhe entrega (o chip, a região, os ids). Cada painel corre esta prova sobre os seus popovers.

use super::{MockPanelHost, NANOS_PER_SECOND};
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, dispatch_pointer};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::widget::{DROPDOWN_SCROLLBAR_ID, DropdownState, ScrollbarState};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Meio pixel de tolerância entre duas medidas do mesmo rect.
const MEIO_PX: f32 = 0.5; // LITERAL-PX-OK: tolerância de medida, não desenho

impl MockPanelHost {
    /// ⭐ **Pinta o que `pinta` desenhar** sobre um [`PaintCtx`] deste arnês com o `layout` dado,
    /// sem um `Panel` inteiro — para um popover DIFERIDO cuja função o painel chama no fim do seu
    /// `paint`. Devolve o que ficou registado. (Uma banda de painéis BAIXA — `layout.inspector` —
    /// é como uma lista curta transborda.)
    pub fn paint_fn(
        &mut self,
        layout: &HeroLayout,
        pinta: impl FnOnce(&mut PaintCtx),
    ) -> Vec<(NodeId, Rect)> {
        self.hit_index.clear_for_frame();
        let viewport = layout.viewport;
        let mut scene = VectorScene::new();
        let mut text_system = TextSystem::without_system_fonts();
        {
            let mut ctx = PaintCtx {
                host: self,
                layout,
                slot: layout.inspector,
                viewport,
                scene: &mut scene,
                text_system: &mut text_system,
            };
            pinta(&mut ctx);
        }
        self.hit_index.iter_registrations().collect()
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.clock_ns += NANOS_PER_SECOND;
        let arena = Bump::new();
        let event = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: self.clock_ns,
        };
        dispatch_pointer(&mut self.store, &self.hit_index, event, &arena).to_vec()
    }

    /// Marca o chip `dd` como ABERTO no store — a roda só rola a lista de um dropdown aberto.
    fn abre_dropdown(&mut self, dd: NodeId) {
        if let Some(InteractiveState::Dropdown { open, .. }) = self.store.get_mut(dd) {
            *open = true;
        } else {
            self.store.register(
                dd,
                InteractiveState::Dropdown {
                    state: DropdownState::Normal,
                    open: true,
                    selected_index: None,
                },
            );
        }
    }
}

fn rect_de(pintado: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    pintado.iter().find(|(i, _)| *i == id).map(|(_, r)| *r)
}

/// ⭐⭐⭐ **A lista longa do chip `dd` (opções `opcoes`, por ordem) ROLA de ponta a ponta.**
///
/// `pinta` repinta o popover (o painel a chamar a sua função diferida, ou o painel inteiro).
/// Afirma, dirigindo os eventos reais:
/// 1. publica o alcance (`content_h > visible_h` — a fixtura TEM de transbordar), regista a barra
///    e só uma FATIA das linhas, todas dentro da lista e todas FOCÁVEIS (hit ⟹ `Button`);
/// 2. a roda rola, e a última linha fica registada e o clique nela vira o `Click` dela;
/// 3. uma rolagem rançosa além do fim é clampada de volta NO STORE;
/// 4. o polegar arrasta, e agarrado lê-se `Dragging` por quem pergunta pela barra.
///
/// # Panics
/// Quando qualquer das quatro falha — com a frase do que o artista veria.
pub fn prova_que_a_lista_longa_rola(
    host: &mut MockPanelHost,
    dd: NodeId,
    opcoes: &[NodeId],
    pinta: &mut dyn FnMut(&mut MockPanelHost) -> Vec<(NodeId, Rect)>,
) {
    let n = opcoes.len();
    host.abre_dropdown(dd);
    host.store.set_panel_scroll(dd, 0.0);

    // 1 — o alcance, a barra e a fatia.
    let pintado = pinta(host);
    let (dono, panel) = host
        .store
        .dropdown_popover()
        .expect("o rect da lista não foi publicado — a roda não a acha");
    assert_eq!(dono, dd, "o popover publicado é de outro chip");
    let content_h = host
        .store
        .panel_content_h(dd)
        .expect("sem altura do conteúdo");
    let visible_h = host.store.panel_visible_h(dd).expect("sem altura visível");
    assert!(
        content_h > visible_h,
        "a fixtura não transborda: {n} linhas cabem ({content_h:.1} em {visible_h:.1})"
    );
    assert!(
        rect_de(&pintado, DROPDOWN_SCROLLBAR_ID).is_some_and(|r| r.h > 0.0),
        "a barra da lista longa não foi registada"
    );
    let mut vistas = 0;
    for (i, &id) in opcoes.iter().enumerate() {
        let Some(r) = rect_de(&pintado, id) else {
            continue;
        };
        vistas += 1;
        assert!(
            r.y >= panel.y - MEIO_PX && r.y + r.h <= panel.y + panel.h + MEIO_PX,
            "a linha {i} foi registada fora da lista"
        );
        assert!(
            matches!(host.store.get(id), Some(InteractiveState::Button { .. })),
            "a linha {i} tem hit mas não é focável — morta sob o rato"
        );
    }
    assert!(
        vistas > 0 && vistas < n,
        "esperava-se uma FATIA registada, viu {vistas} de {n}"
    );

    // 2 — a roda, e a última linha clicável.
    let ultima = opcoes[n - 1];
    assert!(
        rect_de(&pintado, ultima).is_none(),
        "pré-condição: a última linha começa abaixo da dobra"
    );
    host.wheel_at(
        panel.x + panel.w * 0.5,
        panel.y + panel.h * 0.5,
        -content_h * 4.0,
    );
    assert!(
        host.store.panel_scroll(dd) > 0.0,
        "a roda não rolou a lista"
    );
    let pintado = pinta(host);
    let r = rect_de(&pintado, ultima).expect("rolada até ao fim, a última linha não aparece");
    let eventos = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        eventos.contains(&WidgetEvent::Click(ultima)),
        "o clique na última linha não virou o Click dela: {eventos:?}"
    );

    // 3 — a rolagem rançosa volta clampada ao store.
    host.abre_dropdown(dd);
    host.store.set_panel_scroll(dd, content_h * 10.0);
    let _ = pinta(host);
    let max_scroll = content_h - visible_h;
    assert!(
        (host.store.panel_scroll(dd) - max_scroll).abs() < MEIO_PX,
        "a rolagem rançosa não foi clampada de volta no store: {} contra {max_scroll}",
        host.store.panel_scroll(dd)
    );

    // 4 — o polegar.
    host.store.set_panel_scroll(dd, 0.0);
    let pintado = pinta(host);
    let barra = rect_de(&pintado, DROPDOWN_SCROLLBAR_ID).expect("sem barra");
    let x = barra.x + barra.w * 0.5;
    host.ponteiro(PointerKind::Down, x, barra.y + 1.0); // LITERAL-PX-OK: um pixel abaixo do topo da trilha, dentro do polegar em rolagem 0
    host.ponteiro(PointerKind::Move, x, barra.y + barra.h * 0.5);
    assert!(
        host.store.panel_scroll(dd) > 0.0,
        "arrastar o polegar não rolou a lista"
    );
    assert_eq!(
        host.store.scrollbar_visual(DROPDOWN_SCROLLBAR_ID).0,
        ScrollbarState::Dragging,
        "o polegar agarrado pinta-se em repouso"
    );
    host.ponteiro(PointerKind::Up, x, barra.y + barra.h * 0.5);
}
