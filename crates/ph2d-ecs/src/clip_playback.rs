//! ⭐ **O clipe que um objecto está a tocar AGORA**, quando alguém o pediu (doc ObjectsAndComponents 01
//! §14, item 1 — escolha 4.1 do dono: *«os outros tocam quando algo pedir — estado, sinal, script»*).
//!
//! Sem este componente o objecto toca o clipe de ENTRADA da Animação dele desde o início do relógio
//! (`TimelineDoc::entry_clip_of`). Um pedido (o verbo de sinal *Play Clip*) põe aqui o NOME do clipe —
//! único na animação do objecto — e o instante do relógio da cena em que ele começa.
//!
//! ⚠️ **Estado de EXECUÇÃO, não de documento:** não é registado (não grava, não entra no undo). Um
//! pedido do futuro (o relógio voltou para antes dele — um rewind, um scrub) não vale: o objecto
//! toca a entrada, e o scrub mostra a cena como ela começa.

use bevy_ecs::component::Component;

/// O pedido em curso: o clipe `clip` desde o instante `desde` (segundos do relógio da cena).
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ClipPlayback {
    pub clip: String,
    pub desde: f64,
}

impl ClipPlayback {
    /// O pedido, se ele já começou no instante `t`.
    #[must_use]
    pub fn em(&self, t: f64) -> Option<(&str, f64)> {
        (t >= self.desde).then_some((self.clip.as_str(), t - self.desde))
    }
}
