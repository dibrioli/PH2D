//! **O ESTADO TRANSIENTE do Input Map** — o pedido da aba e a ESCUTA partilhada (a aba central e o
//! cartão do Trigger), irmão de [`super::chrome_ops`].
//!
//! ⚠️ **Só o que é VISTA mora aqui**: qual acção está à escuta de uma ligação.
//! O **mapa** é documento e vive no `HeroScreen` — metê-lo aqui faria o `WidgetStore`, que é estado
//! de UI, passar a carregar conteúdo autorado que tem de sobreviver a tudo.

use super::WidgetStore;

impl WidgetStore {
    /// Settings ▸ Input Map… pede a aba central (plano InputMap §3); quem a abre é a família.
    pub fn request_input_map_tab(&mut self) {
        self.input_map_tab_request = true;
    }

    /// Lê e limpa o pedido da aba.
    pub fn take_input_map_tab_request(&mut self) -> bool {
        std::mem::take(&mut self.input_map_tab_request)
    }

    /// **Arma a escuta** para a acção `id`: a próxima tecla vira uma ligação dela.
    ///
    /// ⚠️ Armar uma segunda vez **substitui** a primeira, em vez de empilhar. Duas acções à escuta
    /// ao mesmo tempo tornariam *"para quem é esta tecla?"* uma pergunta sem resposta.
    pub fn listen_for_binding(&mut self, id: ph2d_input::ActionId) {
        self.input_map_listening = Some(id);
        self.input_map_listen_replaces = false;
    }

    /// **Arma a escuta que TROCA** — a do cartão do Trigger: a tecla apanhada substitui as teclas de
    /// teclado da acção; as ligações de comando ficam (dono 09/11).
    pub fn listen_for_binding_replacing(&mut self, id: ph2d_input::ActionId) {
        self.input_map_listening = Some(id);
        self.input_map_listen_replaces = true;
    }

    /// ⭐ **Um clique em QUALQUER sítio desarma a escuta do cartão** (auditoria do fecho 09/11): com
    /// ela armada e esquecida, a tecla seguinte — escrita num campo, o F5 do Play, a primeira do
    /// jogo — trocaria em silêncio as teclas da acção, e o Input Map não é do `Ctrl+Z`. Chamado pela
    /// porta ÚNICA do clique da shell em cada `Down`. A escuta da aba do mapa não é tocada.
    pub fn desarma_escuta_no_clique(&mut self) {
        self.input_map_listen_cancelled_by_down = self
            .input_map_listening
            .filter(|_| self.input_map_listen_replaces);
        if self.input_map_listen_cancelled_by_down.is_some() {
            self.stop_listening();
        }
    }

    /// **De que acção o clique em curso começou por cancelar a escuta?** O botão Key e o «Bind…» da
    /// janela leem-no para CANCELAR em vez de re-armar (o `Down` deles já desarmou). Drena.
    pub fn take_listen_cancelled_by_down(&mut self) -> Option<ph2d_input::ActionId> {
        std::mem::take(&mut self.input_map_listen_cancelled_by_down)
    }

    /// A escuta armada troca (`true`) ou acrescenta (`false`).
    #[must_use]
    pub fn input_map_listen_replaces(&self) -> bool {
        self.input_map_listen_replaces
    }

    /// Desarma a escuta (o `Esc`, ou a ligação já capturada).
    pub fn stop_listening(&mut self) {
        self.input_map_listening = None;
        self.input_map_listen_replaces = false;
    }

    /// **Guarda a ligação apanhada** (tecla ou comando) — escrita pela escuta, drenada pelo chrome.
    pub fn capture_binding(&mut self, b: ph2d_input::Binding) {
        self.input_map_captured = Some(b);
    }

    /// **Drena** a ligação apanhada — `take`, nunca uma leitura que a deixe lá.
    ///
    /// ⚠️ Uma leitura sem drenar faria a mesma tecla ligar-se outra vez ao gesto seguinte: o
    /// artista arma `Bind…` numa segunda acção e ela nasce já ligada à tecla da primeira, sem
    /// ninguém ter carregado em nada.
    pub fn take_captured_binding(&mut self) -> Option<ph2d_input::Binding> {
        self.input_map_captured.take()
    }

    /// Qual acção está à escuta, se alguma.
    ///
    /// ⚠️ **É esta a pergunta que o despacho de teclado tem de fazer PRIMEIRO.** Enquanto ela
    /// responde `Some`, a tecla é conteúdo — não é atalho. Sem essa ordem, ligar `S` a uma acção
    /// **salva o projecto**.
    #[must_use]
    pub fn input_map_listening(&self) -> Option<ph2d_input::ActionId> {
        self.input_map_listening
    }
}

/// **A LEI DA ESCUTA, num sítio só** — chamada pela shell **e** pelo despacho de teclado.
///
/// ⛔⛔ **Report do Enio (2026-08-24): *"Os atalhos de editor estão em conflito com o Bind"*.**
///
/// A primeira versão pôs esta guarda como o primeiro ramo do `dispatch_key` — o primeiro **dentro
/// do editor-core**. Mas a shell tem ~20 `return` no `key_input` **antes** de chamar o
/// `dispatch_key`: o `P` do menu radial, o `W` do painel de mundo, o Espaço do transporte, o peek
/// do Flip. Nenhuma dessas teclas chegava aqui, então carregar nelas durante o `Bind…` executava o
/// atalho e não ligava nada.
///
/// ⇒ **a guarda sobe para o topo do `key_input` da shell**, e esta função é a lei que os dois
/// chamam. *A ordem é a feature, e ela tem de estar no topo da cadeia REAL, não no topo de um
/// pedaço dela.*
///
/// Devolve `Some(evento)` quando a tecla foi **consumida pela escuta** — quem chama tem de parar
/// ali. `None` quando não há escuta armada e a tecla segue o caminho normal.
///
/// ⚠️ O `Esc` **desarma sem ligar**, e devolve `Some` na mesma: ele foi consumido, e deixá-lo cair
/// para o resto fecharia também um painel ou cancelaria uma ferramenta.
#[must_use]
pub fn capture_if_listening(
    store: &mut WidgetStore,
    keycode: u32,
) -> Option<crate::interaction::WidgetEvent> {
    /// O `Esc` no espaço de keycode normalizado do app.
    const ESCAPE: u32 = 0x1B;
    store.input_map_listening()?;
    // ⛔ O que a aba nunca liga não é capturado: segue e a escuta continua (InputMap W1/W2b). Uma tecla do SISTEMA,
    // o F5 (no Play é o transporte: a ligação nunca chegaria ao jogo), Ctrl/Alt e o acorde com Ctrl ou Alt em baixo
    // (a shell empurra os dois em cada `ModifiersChanged`: `set_cmd_held`/`set_alt_held`).
    let held = ph2d_input::editor_keys::held_from_flags(store.cmd_held(), store.alt_held());
    if ph2d_input::editor_keys::never_binds(ph2d_input::Key(keycode), held) {
        return None;
    }
    if keycode == ESCAPE {
        store.stop_listening();
        return Some(crate::interaction::WidgetEvent::Click(
            crate::ids::INPUT_MAP_LISTEN_CANCELLED,
        ));
    }
    store.capture_binding(ph2d_input::Binding::Key(ph2d_input::Key(keycode)));
    Some(crate::interaction::WidgetEvent::Click(
        crate::ids::INPUT_MAP_BIND_CAPTURED,
    ))
}
