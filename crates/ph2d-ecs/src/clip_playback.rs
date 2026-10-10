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
use bevy_ecs::resource::Resource;

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

/// ⭐ **Os nomes dos clipes de cada Animação, publicados no mundo** (onda 5, item c) — `(bits do
/// dono, nome)`, na ordem do documento, só donos VIVOS; o dono [`Self::SCENE`] é a Cena.
///
/// Quem escreve é o `upkeep` da timeline (`ph2d-timeline-persist`), no fim de cada quadro e só
/// quando o documento difere; quem lê é o retrato do Inspector, que oferece o ▾ do *Play Clip*
/// sem ver o `TimelineDoc` (norte ECS: um recurso, não uma chamada). ⚠️ Estado de EXECUÇÃO como o
/// [`ClipPlayback`]: não é registado, não grava, não entra no undo.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct ClipNames(pub Vec<(u64, String)>);

impl ClipNames {
    /// A chave da Cena — o `ph2d_timeline::SCENE` (o `0` nunca é uma entidade bevy; há gate).
    pub const SCENE: u64 = 0;

    /// Os nomes da Animação de `owner`, por ordem — a mesma pergunta do `TimelineDoc::clips_of`.
    pub fn of(&self, owner: u64) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(move |(dono, _)| *dono == owner)
            .map(|(_, nome)| nome.as_str())
    }
}
