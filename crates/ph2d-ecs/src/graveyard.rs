//! ⭐ **O CEMITÉRIO** — um Ctrl+Z devolve o objecto apagado COM o que ele tinha nos documentos que
//! não vivem no undo global (os clipes da timeline, os nós do Motion). Doc ObjectsAndComponents 01
//! §14, item 3.
//!
//! O undo global é por diff do mundo; a timeline e o Motion têm filas próprias (de propósito: um
//! Ctrl+Z na tela não rebobina a animação). Quando um dono morre, a purga empurra o `antes` na fila
//! própria — e ENTERRA-o aqui com os ids dos donos. Quando o undo global os devolve, a cova abre: se
//! o passo da purga ainda é o TOPO da fila (nada foi editado depois), a fila desfá-lo sozinha, no
//! mesmo quadro — o segundo Ctrl+Z que o artista tinha de dar. Se foi editado, a cova fecha-se e o
//! passo continua lá: nada se perde, são dois Ctrl+Z como antes.
//!
//! ⛔ Recusado: um «passo ligado» entre filas — parte quando uma delas é desfeita fora de ordem.

/// Quantas covas se guardam: a profundidade do undo global (`UNDO_CAP` da shell, gate
/// `the_graveyard_is_as_deep_as_the_global_undo`). Mais fundo, o undo global já não devolve o
/// objecto, e a cova nunca abriria.
pub const GRAVEYARD_DEPTH: usize = 256;

#[derive(Debug, Clone)]
struct Grave<D> {
    ids: Vec<u64>,
    before: D,
}

/// As covas de um documento `D`, da mais antiga para a mais recente.
#[derive(Debug, Clone)]
pub struct Graveyard<D> {
    graves: Vec<Grave<D>>,
}

impl<D> Default for Graveyard<D> {
    fn default() -> Self {
        Self { graves: Vec::new() }
    }
}

impl<D> Graveyard<D> {
    /// Enterra o passo `before` que a purga dos donos `ids` (os `StableId`) empurrou na fila. Sem id
    /// nenhum (um dono que nunca teve identidade) não há o que devolver.
    pub fn bury(&mut self, ids: Vec<u64>, before: D) {
        if ids.is_empty() {
            return;
        }
        if self.graves.len() >= GRAVEYARD_DEPTH {
            self.graves.remove(0);
        }
        self.graves.push(Grave { ids, before });
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.graves.is_empty()
    }

    /// Abre, da mais recente para trás, toda cova cujos donos vivem outra vez, e entrega o `antes`
    /// dela a `open` — que desfaz o passo se ele ainda for o topo da fila e diz se o fez. Devolve se
    /// alguma abriu.
    pub fn rise(&mut self, alive: impl Fn(u64) -> bool, mut open: impl FnMut(D) -> bool) -> bool {
        let mut any = false;
        while let Some(i) = self
            .graves
            .iter()
            .rposition(|g| g.ids.iter().all(|&id| alive(id)))
        {
            any |= open(self.graves.remove(i).before);
        }
        any
    }
}

#[cfg(test)]
#[path = "graveyard_tests.rs"]
mod tests;
