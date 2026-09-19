//! ⭐⭐⭐ **O VOCABULÁRIO DO SEGUIDOR DE CAMINHO** (suplente #23) — o instantâneo e a edição, num
//! módulo abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tween_edits`], do [`crate::factory_edits`] e do
//! [`crate::topdown_edits`], e com o mesmo número atrás: a catraca do DAG tolera a aresta
//! `action_bus → screens` num **tecto** e escreve a cura ao lado dela.
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `Pick a drawn shape` | o campo do caminho está vazio | escrever o nome da forma |
//! | `No shape in the scene has that name` | o nome não casa com forma nenhuma | corrigir o nome, ou desenhar a curva |
//! | `That object has no drawn shape` | o nome casa com um objecto **sem geometria** | apontar a uma forma |
//! | `There is no Timer at that slot` | o índice do relógio não existe na lista | baixar o índice, ou acrescentar um timer |
//! | `The clock is stopped` | a ponte corre no passo fixo | carregar no play |
//!
//! ⚠️ **As três primeiras são TRÊS e não uma**, e a razão é que as curas são diferentes: *«escreve
//! um nome»*, *«corrige o nome»* e *«esse objecto não é uma forma»* mandariam o artista a sítios
//! diferentes, e uma mensagem só mandaria dois terços deles ao sítio errado. É a lei que o gatilho
//! (suplente #24) pagou com o `NoMapa`.

/// Snapshot da secção PATH FOLLOW da entidade selecionada.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorPathFollowInfo {
    pub entity_bits: u64,
    /// O NOME da forma a percorrer — vazio = nenhum.
    pub caminho: String,
    /// ⭐ **Uma forma com aquele nome existe, e ela TEM geometria?** — ver o cabeçalho: um nome
    /// errado e um objecto sem curva são dois defeitos com duas curas.
    pub nome_existe: bool,
    pub nome_tem_forma: bool,
    /// O índice do relógio que o faz andar.
    pub relogio: u8,
    /// Quantos relógios a entidade tem — o aviso do índice sem timer sai daqui.
    pub relogios: usize,
    /// ⭐ **A DURAÇÃO do relógio desse índice, em µs** — `None` quando não há relógio nenhum.
    ///
    /// ⚠️ É a lição da W7 do tween, herdada: *o painel DIZ onde mora o tempo*, em vez de mandar o
    /// artista adivinhar noutra secção.
    pub duracao_us: Option<u64>,
    pub repeat: bool,
    pub autostart: bool,
    /// A tag do [`ph2d_tween::Ciclo`].
    pub ciclo: u8,
    /// A tag da família da curva (`ph2d_anim::EasingFamily`).
    pub familia: u8,
    /// A tag do modo (`ph2d_anim::EasingMode`).
    pub modo: u8,
    /// A tag do [`ph2d_tween::AoAcabar`].
    pub ao_acabar: u8,
    /// Onde no percurso ele entra, em fracção.
    pub deslocamento: f32,
    pub alinha: bool,
    /// Em GRAUS — a unidade autorada do app.
    pub angulo: f32,
    /// Em metros.
    pub lado: f32,
    /// O relógio da cena está a andar?
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção PATH FOLLOW.
#[derive(Clone, Debug, PartialEq)]
pub enum PathFollowFieldEdit {
    /// O NOME da forma. ⚠️ Cru: quem o resolve é a ponte, que tem o mundo.
    Caminho(String),
    /// O índice do relógio.
    Relogio(u8),
    /// A tag do [`ph2d_tween::Ciclo`].
    Ciclo(u8),
    /// A tag da família da curva.
    Familia(u8),
    /// A tag do modo.
    Modo(u8),
    /// A tag do [`ph2d_tween::AoAcabar`].
    AoAcabar(u8),
    /// A fracção de entrada na pista.
    Deslocamento(f32),
    /// Roda para a tangente?
    Alinha(bool),
    /// O ângulo somado, em GRAUS.
    Angulo(f32),
    /// O deslocamento perpendicular, em metros.
    Lado(f32),
}

/// ⭐⭐⭐ **A QUEIXA da secção** — o que impede este seguidor de andar, da mais ESPECÍFICA para a
/// mais geral.
///
/// ⚠️ **A ordem é a lei**: dizer *«não há relógio»* a quem também não escreveu o nome é mandá-lo
/// resolver a metade errada. É a mesma escada que a `recusa::Entradas` da escultura já escreve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathFollowQueixa {
    /// O campo do caminho está vazio.
    SemNome,
    /// O nome não casa com objecto nenhum da cena.
    NomeDesconhecido,
    /// O objecto existe e **não tem forma desenhada**.
    SemForma,
    /// O índice do relógio não existe na lista de timers.
    SemRelogio,
    /// O relógio existe e dura `0` — ele nunca avança.
    RelogioSemDuracao,
}

impl InspectorPathFollowInfo {
    /// A queixa desta secção, se houver. `None` = o seguidor tem tudo o que precisa.
    ///
    /// ⚠️ **O relógio parado NÃO é queixa** — ele é o estado normal de uma cena em edição, e um
    /// aviso que aparece sempre é ruído que o artista aprende a ignorar exactamente quando ele
    /// passar a ser verdade. Quem o diz é uma linha de estado, não esta escada.
    #[must_use]
    pub fn queixa(&self) -> Option<PathFollowQueixa> {
        if self.caminho.trim().is_empty() {
            return Some(PathFollowQueixa::SemNome);
        }
        if !self.nome_existe {
            return Some(PathFollowQueixa::NomeDesconhecido);
        }
        if !self.nome_tem_forma {
            return Some(PathFollowQueixa::SemForma);
        }
        let Some(duracao_us) = self.duracao_us else {
            return Some(PathFollowQueixa::SemRelogio);
        };
        if duracao_us == 0 {
            return Some(PathFollowQueixa::RelogioSemDuracao);
        }
        None
    }
}

#[cfg(test)]
#[path = "path_follow_edits_tests.rs"]
mod tests;
