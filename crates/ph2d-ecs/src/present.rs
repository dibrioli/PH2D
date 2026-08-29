//! `PresentWorld` — opaque newtype over `bevy_ecs::World` for
//! presentation state (render, animation, particles, editor).
//! ADR-0021.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::{QueryState, Without};
use bevy_ecs::resource::IsResource;
use bevy_ecs::world::World;

/// Marker trait for components that belong in [`PresentWorld`].
///
/// **Convention** (M4): a component implements `PresentComponent`
/// iff it's derived state for rendering / animation / editor and is
/// rebuilt each frame from `SimWorld` via the `extract!` macro.
/// `PresentComponent` types are NOT serialized to save (HR-14) and
/// are NOT part of replay determinism (HR-5) — they're free to use
/// non-deterministic data (random jitter, timestamps, GPU handles).
pub trait PresentComponent: Component {}

/// Opaque newtype over `bevy_ecs::World` for presentation state.
///
/// Mirror of [`crate::SimWorld`] — same API shape; semantically
/// distinct so type errors catch accidental cross-pollination.
///
/// # ⚠️ O descarte por quadro é [`PresentWorld::clear`], e ele NÃO é `World::clear_entities()`
///
/// No `bevy_ecs` 0.19 *recursos são entidades*: cada recurso vive numa entidade marcada com
/// `IsResource`, e um índice interno diz qual entidade guarda qual recurso.
/// `World::clear_entities()` **contorna os hooks** — destrói essas entidades e deixa o índice a
/// apontar para elas. O mecanismo e o preço de cada saída estão em [`PresentWorld::clear`].
pub struct PresentWorld {
    inner: World,
    /// O alvo do descarte, reutilizado entre quadros.
    ///
    /// ⚠️ Não se pode despachar enquanto se itera o mundo, então a lista tem de ser materializada
    /// primeiro. Reaproveitar o `Vec` é o que mantém o descarte **sem alocação** em regime
    /// (HR-3) — o mesmo padrão do `WorklistBuf` da propagação, e é o portão
    /// `tests/propagate_no_alloc.rs` que o mede.
    doomed: Vec<Entity>,
    /// A consulta do descarte, construída **uma vez**.
    ///
    /// ⚠️ **Medido, não profilático:** com `world.query_filtered(…)` chamado por quadro, o portão
    /// de zero alocação media **107 blocos / 10 quadros** contra um orçamento de 64 — cada
    /// construção de `QueryState` aloca as suas tabelas de archetypes. Guardá-la é o que põe o
    /// descarte de volta em regime sem alocação. O `iter` actualiza a cache de archetypes sozinho,
    /// então ela sobrevive ao despacho.
    sweep: Option<QueryState<Entity, Without<IsResource>>>,
}

impl PresentWorld {
    pub fn new() -> Self {
        Self {
            inner: World::new(),
            doomed: Vec::new(),
            sweep: None,
        }
    }

    pub fn world(&self) -> &World {
        &self.inner
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.inner
    }

    /// **A porta única do descarte** — o que o quadro faz antes de voltar a espelhar o `SimWorld`.
    ///
    /// Descarta as entidades **do jogo** e deixa os recursos de pé.
    ///
    /// Existe como porta, e não como uma chamada escrita em cinco sítios, porque a lei que ela
    /// carrega mudou de conteúdo sem mudar de nome. *Uma lei escrita em cinco sítios ainda não é
    /// uma lei.*
    ///
    /// # ⛔ Por que NÃO é `World::clear_entities()`, que é o que estava aqui
    ///
    /// No `bevy_ecs` 0.19 recursos são entidades, e `clear_entities()` **contorna os hooks**: ele
    /// destrói as entidades-recurso e deixa o índice interno a apontar para elas. O que acontece a
    /// seguir depende de uma coisa que ninguém escolhe — se já se criaram entidades entre o
    /// descarte e o acesso:
    ///
    /// | ordem | o que acontece (medido na 0.19.1) |
    /// |---|---|
    /// | descartar → **inserir** recurso | **PÂNICO** (`ResourceCache … ValidButNotSpawned`) |
    /// | descartar → **criar entidades** → inserir recurso | ⛔ **silêncio, e corrompe** |
    /// | descartar → ler recurso | devolve `None`, sem aviso |
    ///
    /// ⚠️ **A do meio é exactamente a ordem deste laço** (descarta, espelha milhares de entidades,
    /// e só então alguém tocaria num recurso). Medido: das 5 entidades criadas, uma recebe a marca
    /// `IsResource` soldada por cima e **desaparece de toda consulta filtrada** — a contagem passa
    /// a 4, as 5 continuam lá, e nada falha. *Um pânico vê-se; isto evapora um objecto do artista.*
    ///
    /// # Os caminhos medidos
    ///
    /// | caminho | veredito |
    /// |---|---|
    /// | `clear_entities()` | ⛔ corrompe em silêncio na ordem deste laço |
    /// | despachar **tudo**, recursos incluídos | ⛔ **PÂNICO** — a entidade-recurso morre em cascata e é revisitada |
    /// | despachar só o que **não** é recurso — **este** | ✅ 300 quadros, recursos intactos, inserção posterior funciona |
    /// | `*self = World::new()` | ✅ correcto, mas **aloca um mundo por quadro** (parte o HR-3) |
    ///
    /// O preço deste é **+100 % do passe de descarte** (o passe é dominado pelo `spawn`, e
    /// despachar custa aproximadamente o mesmo que criar). É o preço de o mundo ficar **correcto
    /// por construção** em vez de correcto enquanto ninguém lhe puser um recurso.
    pub fn clear(&mut self) {
        if self.sweep.is_none() {
            self.sweep = Some(self.inner.query_filtered::<Entity, Without<IsResource>>());
        }
        let q = self.sweep.as_mut().expect("acabou de ser construida");
        self.doomed.clear();
        self.doomed.extend(q.iter(&self.inner));
        for e in self.doomed.drain(..) {
            let _ = self.inner.despawn(e);
        }
    }

    /// Quantas entidades **do jogo** este mundo tem.
    ///
    /// ⚠️ `Without<IsResource>` não é cerimónia: no `bevy_ecs` 0.19 um mundo recém-construído já
    /// contém **uma** entidade — a do recurso interno do próprio bevy —, então a conta crua
    /// devolveria `1` para um mundo vazio. O filtro é o que faz esta função continuar a responder
    /// à pergunta que o nome dela faz.
    pub fn entity_count(&mut self) -> usize {
        let mut q = self.inner.query_filtered::<Entity, Without<IsResource>>();
        q.iter(&self.inner).count()
    }
}

impl Default for PresentWorld {
    fn default() -> Self {
        Self::new()
    }
}
