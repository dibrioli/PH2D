//! **O ESTADO TRANSIENTE da janela do Input Map** — irmão de [`super::chrome_ops`], cortado por
//! assunto (plano 30 §0.2).
//!
//! ⚠️ **Só o que é VISTA mora aqui**: onde a janela está, e qual acção está à escuta de uma tecla.
//! O **mapa** é documento e vive no `HeroScreen` — metê-lo aqui faria o `WidgetStore`, que é estado
//! de UI, passar a carregar conteúdo autorado que tem de sobreviver a tudo.

use super::WidgetStore;

impl WidgetStore {
    /// **Abre a janela** em `(x, y)` — idempotente: reabrir só a reposiciona.
    pub fn open_input_map(&mut self, x: f32, y: f32) {
        self.input_map_window = Some((x, y));
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

    /// Move a janela (o arrasto pela faixa do título). No-op se ela estiver fechada — mover uma
    /// janela que não existe é a forma de a fazer reaparecer no sítio errado.
    pub fn move_input_map(&mut self, x: f32, y: f32) {
        if self.input_map_window.is_some() {
            self.input_map_window = Some((x, y));
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
