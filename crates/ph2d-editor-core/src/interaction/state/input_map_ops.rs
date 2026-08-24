//! **O ESTADO TRANSIENTE da janela do Input Map** — irmão de [`super::chrome_ops`], cortado por
//! assunto (plano 30 §0.2).
//!
//! ⚠️ **Só o que é VISTA mora aqui**: onde a janela está, e qual acção está à escuta de uma tecla.
//! O **mapa** é documento e vive no `HeroScreen` — metê-lo aqui faria o `WidgetStore`, que é estado
//! de UI, passar a carregar conteúdo autorado que tem de sobreviver a tudo.

use super::{ButtonState, InteractiveState, TextInputState, WidgetStore};

impl WidgetStore {
    /// **Abre a janela** em `(x, y)` — idempotente: reabrir só a reposiciona.
    ///
    /// ⚠️ **E REGISTA os widgets fixos, sem o que a janela seria desenho.** Foi a condição 2 da
    /// costura (*pintado **E** registrado*) a morder: na primeira versão o campo de nome era
    /// pintado e nunca registrado, então `Add` lia sempre a string vazia e **nada nascia** — com
    /// todos os outros gates verdes. O gate da sequência apanhou-o.
    ///
    /// ⚠️ **Reabrir NÃO limpa o campo** (o `register` só semeia quem ainda não existe aqui, porque
    /// o texto é reposto a vazio só no `Add`): fechar por engano e reabrir tem de devolver o que
    /// estava escrito.
    pub fn open_input_map(&mut self, x: f32, y: f32) {
        self.input_map_window = Some((x, y));
        if self.get(crate::ids::INPUT_MAP_NEW_NAME).is_none() {
            self.register(
                crate::ids::INPUT_MAP_NEW_NAME,
                InteractiveState::TextInput {
                    state: TextInputState::Normal,
                    text: String::new(),
                    caret: 0,
                    selection_anchor: None,
                },
            );
        }
        for id in [
            crate::ids::INPUT_MAP_ADD,
            crate::ids::INPUT_MAP_CLOSE,
            crate::ids::INPUT_MAP_HANDLE,
        ] {
            self.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }

    /// Fecha a janela — e ⚠️ **desarma a escuta junto**.
    ///
    /// Uma escuta que sobrevivesse ao fecho comeria a próxima tecla que o artista carregasse **com
    /// a janela já fechada**, e nada na tela diria porquê. *Fechar é largar tudo.*
    pub fn close_input_map(&mut self) {
        self.input_map_window = None;
        self.input_map_listening = None;
    }

    /// Onde a janela está, se estiver aberta.
    #[must_use]
    pub fn input_map_pos(&self) -> Option<(f32, f32)> {
        self.input_map_window
    }

    /// **Desloca a janela** de `(dx, dy)` — o arrasto pela faixa do título.
    ///
    /// ⚠️ **Um DELTA e não uma posição**, como a irmã `move_fill_modal`: o arrasto conhece o
    /// movimento do cursor, não o canto da janela, e converter no chamador espalharia a conta.
    ///
    /// No-op se ela estiver fechada — mover uma janela que não existe é a forma de a fazer
    /// reaparecer no sítio errado quando alguém a reabrir.
    pub fn move_input_map(&mut self, dx: f32, dy: f32) {
        if let Some((x, y)) = self.input_map_window {
            self.input_map_window = Some((x + dx, y + dy));
        }
    }

    /// **Arma a escuta** para a acção `id`: a próxima tecla vira uma ligação dela.
    ///
    /// ⚠️ Armar uma segunda vez **substitui** a primeira, em vez de empilhar. Duas acções à escuta
    /// ao mesmo tempo tornariam *"para quem é esta tecla?"* uma pergunta sem resposta.
    pub fn listen_for_binding(&mut self, id: ph2d_input::ActionId) {
        self.input_map_listening = Some(id);
    }

    /// Desarma a escuta sem fechar a janela (o `Esc`, ou a tecla já capturada).
    pub fn stop_listening(&mut self) {
        self.input_map_listening = None;
    }

    /// **Guarda a tecla apanhada** — chamado pelo despacho de teclado, drenado pelo chrome.
    pub fn capture_bound_key(&mut self, k: ph2d_input::Key) {
        self.input_map_captured = Some(k);
    }

    /// **Drena** a tecla apanhada — `take`, nunca uma leitura que a deixe lá.
    ///
    /// ⚠️ Uma leitura sem drenar faria a mesma tecla ligar-se outra vez ao gesto seguinte: o
    /// artista arma `Bind…` numa segunda acção e ela nasce já ligada à tecla da primeira, sem
    /// ninguém ter carregado em nada.
    pub fn take_captured_key(&mut self) -> Option<ph2d_input::Key> {
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
