//! **O ARRASTO da janela do Input Map** — irmão exacto do `fill_drag::arm_fill_modal_drag_*`.
//!
//! ⚠️ **Máquina de estados do SHELL, e não um evento de widget**, pela mesma razão que a do Fill: um
//! `Click` na faixa do título é *"largou sem mover"*; o **arrasto** precisa de ver cada `CursorMoved`
//! entre o Down e o Up, e isso não é uma coisa que um widget observe. O handler de chrome consome o
//! clique nu (para nunca vazar), e este ficheiro faz o movimento.

use std::cell::Cell;

use ph2d_editor::ids;

thread_local! {
    /// O último ponto do cursor enquanto a janela está a ser arrastada. `None` = não há arrasto.
    static INPUT_MAP_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

impl crate::App {
    /// Um Primary Down sobre a faixa do título arma o arrasto. Devolve `true` (consome o Down) para
    /// que a janela **se mova em vez de o Down fazer outra coisa** — e para que ela nunca feche a
    /// meio do movimento.
    pub(crate) fn arm_input_map_drag_if_on_handle(&mut self, px: f32, py: f32) -> bool {
        let on_handle = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.hit_index.hit(px, py))
            == Some(ids::INPUT_MAP_HANDLE);
        if !on_handle {
            return false;
        }
        INPUT_MAP_DRAG.with(|c| c.set(Some((px, py))));
        true
    }

    /// `CursorMoved` durante o arrasto: desloca a janela pelo delta do cursor. Devolve `true`
    /// (consome o movimento) enquanto arrasta, para não fazer pan nem conduzir um gizmo por baixo.
    pub(crate) fn input_map_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some((lx, ly)) = INPUT_MAP_DRAG.with(Cell::get) else {
            return false;
        };
        INPUT_MAP_DRAG.with(|c| c.set(Some((px, py))));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.move_input_map(px - lx, py - ly);
        }
        true
    }

    /// Primary Up: termina o arrasto. No-op quando não há nenhum.
    pub(crate) fn input_map_drag_up(&mut self) {
        INPUT_MAP_DRAG.with(|c| c.set(None));
    }
}

impl crate::App {
    /// **A ESCUTA TAMBÉM APANHA UM BOTÃO DE COMANDO** (plano 30 §0.1: *"qualquer objecto do game"*).
    ///
    /// ⚠️ **Aqui e não no despacho de teclado**, porque um gamepad não tem despacho: o adaptador do
    /// `gilrs` bombeia eventos para o retrato de dispositivos e ninguém os "encaminha". A pergunta
    /// certa é feita **uma vez por quadro**, sobre a **BORDA** (`pressed`, não `held`) — com `held`,
    /// um botão que já estivesse em baixo quando o artista carregou em `Bind…` ligar-se-ia sozinho,
    /// sem ele ter feito nada.
    ///
    /// ⚠️ **Só corre com a escuta armada**, e sai cedo caso contrário: é o mesmo custo de um `if`
    /// num quadro normal.
    pub(crate) fn poll_input_map_pad_binding(&mut self) {
        let Some(armed) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.input_map_listening())
        else {
            return;
        };
        // ⚠️ **O botão primeiro, a haste depois.** Um comando em repouso já reporta um resíduo
        // nos eixos; se a haste ganhasse, um analógico ligeiramente descentrado ligaria-se sozinho
        // antes de o artista tocar num botão. O limiar abaixo é a segunda metade dessa defesa.
        let hit = ph2d_input::GamepadButton::ALL
            .iter()
            .copied()
            .find(|b| self.input.gamepad.pressed(*b))
            .map(ph2d_input::Binding::PadButton)
            .or_else(|| self.listening_axis_push());
        let Some(b) = hit else {
            return;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        hero.store.stop_listening();
        if let Some(a) = hero.input_map.get_mut(armed) {
            // ⚠️ Não duplica, pela razão do teclado: duas linhas iguais no painel seriam
            // indistinguíveis ao apagar.
            if !a.bindings.contains(&b) {
                a.bindings.push(b);
            }
        }
        let map = hero.input_map.clone();
        ph2d_editor::screens::hero::chrome::sync_input_map_rows(&mut hero.store, &map);
    }

    /// **A haste empurrada a fundo** — a metade ANALÓGICA da escuta, e o que torna os dois números
    /// da zona alcançáveis (sem um eixo ligado, eles não têm o que medir).
    ///
    /// ⚠️ **O limiar é `0,5` e ele é de GESTO, não de produto:** ele responde *"o artista empurrou
    /// esta haste de propósito?"*, e não *"a partir de onde esta acção conta?"* — essa é a pergunta
    /// do `press_point`, que o artista afina **depois**, na própria janela. Confundir os dois faria
    /// a zona morta de uma acção depender de como ela foi ligada.
    ///
    /// ⚠️ E o SINAL decide a metade: empurrar para a esquerda liga a metade negativa. É o que faz
    /// `move_left` e `move_right` serem duas acções sobre o mesmo eixo físico.
    fn listening_axis_push(&self) -> Option<ph2d_input::Binding> {
        /// Meio curso: fundo do curso é `1,0`, e o resíduo de um comando parado fica muito abaixo.
        const PUSHED: f32 = 0.5;
        ph2d_input::GamepadAxis::ALL
            .iter()
            .copied()
            .find_map(|axis| {
                let v = self.input.gamepad.axis(axis);
                (v.abs() >= PUSHED).then_some(ph2d_input::Binding::PadAxis {
                    axis,
                    positive: v > 0.0,
                })
            })
    }
}

impl crate::App {
    /// **A RODA sobre a janela do Input Map rola a lista.** `true` ⇒ consome.
    ///
    /// ⛔ Report do Enio (2026-08-24): *"estreito e **sem scroll**"*. Um cartão que cresce com a
    /// lista sai do ecrã e a última acção fica inalcançável — e nada na tela diz porquê.
    ///
    /// ⚠️ **O TETO vem daqui, não do `WidgetStore`**: ele não vê o mapa, e só quem conta as linhas
    /// sabe onde a lista acaba. Sem teto, a roda leva a lista para longe e o artista vê um cartão
    /// vazio sem saber como voltar.
    ///
    /// ⚠️ **Consome sempre que o cursor está sobre a janela**, mesmo quando ela cabe inteira: a
    /// roda que atravessasse o cartão daria zoom no canvas por baixo dele, que é o gesto errado
    /// com a mão no sítio certo.
    /// ⭐ **A roda pertence à PALETA enquanto ela estiver aberta** — ver
    /// [`crate::component_attach::palette_wheel`], onde mora o porquê.
    pub(crate) fn command_palette_wheel(&mut self, dy: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let viewport = gfx.hero_screen.as_ref().map(|h| h.last_viewport);
        let (Some(hero), Some(viewport)) = (gfx.hero_screen.as_mut(), viewport) else {
            return false;
        };
        crate::component_attach::palette_wheel(hero, &mut gfx.text_system, viewport, dy)
    }

    pub(crate) fn input_map_wheel(&mut self, dy: f32) -> bool {
        let (px, py) = self.last_pointer;
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        let Some((wx, wy)) = hero.store.input_map_pos() else {
            return false;
        };
        // ⚠️ **A ALTURA DA VIEWPORT É PARTE DA PERGUNTA** — auditoria 2026-08-24. A janela é
        // clampada ao ecrã pelo pintor; perguntar o tamanho sem a viewport devolvia o tamanho
        // PEDIDO, e a roda passava a testar um rectângulo que **não está na tela** assim que a
        // lista transborda. O `last_viewport` é o mesmo que o pintor recebeu no quadro anterior.
        let vh = hero.last_viewport.h;
        let (ww, wh, max_scroll) =
            ph2d_editor::screens::hero::chrome::input_map_window_size(&hero.input_map, vh);
        // ⚠️ E a POSIÇÃO também é clampada, pelo mesmo motivo: o pintor prende o canto à viewport,
        // e um cartão encostado à borda de baixo desenha acima de onde o store diz que ele está.
        let vx = hero.last_viewport.x;
        let vy = hero.last_viewport.y;
        let wx = wx.clamp(vx, (vx + hero.last_viewport.w - ww).max(vx));
        let wy = wy.clamp(vy, (vy + vh - wh).max(vy));
        if px < wx || px > wx + ww || py < wy || py > wy + wh {
            return false;
        }
        hero.store.scroll_input_map(-dy, max_scroll);
        true
    }
}
