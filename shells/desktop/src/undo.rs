//! Undo/redo GLOBAL do editor — uma fila só, muitos tipos de objeto.
//!
//! A unidade é o **estado do projeto** num instante: o mundo (ECS) mais a geometria
//! vetorial. Undo e save gravam a MESMA captura — o save só adiciona os bytes das
//! imagens por cima (`crate::project`, bloco seguinte).
//!
//! **Snapshot-based, não command-based.** Cada passo guarda o estado ANTES da ação
//! e o restaura; não há operação invertível por-tipo. É o que faz "mover, deletar,
//! reparentear, criar, editar nó" caberem numa fila só, sem um `match` gigante — e
//! subsume o `ph2d_vec_edit::History` (o `VecScene` já está na captura).
//!
//! **Registro por DIFF, não por gesto.** [`crate::App::post_frame_undo`] roda uma vez
//! por frame: se houve input e nenhum gesto está em curso, compara o estado atual com
//! o baseline; qualquer diferença vira um passo. Um só ponto torna toda ação
//! desfazível (gizmo, pen, tecla, botão) sem instrumentar cada site.
//!
//! **Escopo (Enio 2026-07-09):** objetos, hierarquia e canvas. NÃO toca painéis —
//! as configs de painel têm undo próprio, com botões no header (bloco à parte).

use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world};
use ph2d_ecs::{Entity, SimWorld, Transform, With};
use ph2d_flip::FlipDoc;
use ph2d_vec_scene::VecScene;

/// Profundidade máxima da pilha (interações comuns, não infinitas). Igual ao
/// `HISTORY_CAP` do vetor.
const UNDO_CAP: usize = 256;

/// O estado mutável do projeto num instante: o mundo + a geometria vetorial.
///
/// `WorldSnapshot` cobre toda entidade com componente registrado — pose, nome,
/// árvore, trava, e as referências que ligam um path (`VecPathRef`) ou um objeto
/// Flip (`FlipObjectRef`) à entidade (ADR-0110/0114). `VecScene` e `FlipDoc` são
/// as geometrias, que vivem fora do ECS. Juntos são o projeto inteiro exceto os
/// pixels dos sprites (estáveis, não mudam a cada ação — o save os anexa à parte).
///
/// `FlipDoc` é determinístico (Vec/BTreeMap/ids monotônicos), então — ao contrário
/// do `WorldSnapshot` — não precisa de `canonicalize`: capturar o mesmo estado
/// duas vezes dá `FlipDoc`s iguais e o diff de undo não registra passo espúrio.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectState {
    pub(crate) world: WorldSnapshot,
    /// ⭐⭐⭐ **A cena vetorial, PARTILHADA entre passos** (F8, 2026-09-02).
    ///
    /// ⛔⛔ **Medido antes de mudar** (`ph2d-vec-scene/tests/measure_scene_clone.rs`): um passo
    /// clonava a cena INTEIRA — `236 KB` a 1 000 formas, `1,18 MB` a 5 000 —, e a pilha guarda
    /// `UNDO_CAP` passos ⇒ **60 MB** e **303 MB** só de cópias da mesma cena.
    ///
    /// ⚠️ **É o argumento que o [`WorldSnapshot`] já tinha feito** (`Arc` por linha, F2): a
    /// esmagadora maioria dos passos não toca no documento vetorial — mover um objecto, renomear,
    /// pôr um componente —, e para esses a cena de dois passos consecutivos é **a mesma**.
    ///
    /// ⚠️ **E não move um byte do formato**: a serde com a feature `rc` escreve um `Arc<T>` como o
    /// próprio `T`, então o `PROJECT_SCHEMA` fica onde está e todo ficheiro gravado continua a ler
    /// igual. O `PartialEq` compara o CONTEÚDO (o `Arc` delega), logo o diff do undo não muda de
    /// significado.
    pub(crate) vec: std::sync::Arc<VecScene>,
    pub(crate) flip: FlipDoc,
    /// As guias do documento. Plain data — nenhuma ponte a reconstruir, ao contrário do
    /// vetor e do Flip, e é por isso que o `restore` não as devolve na tupla: quem aplica
    /// simplesmente copia.
    pub(crate) guides: ph2d_guides::GuideSet,
    /// Os ESTADOS de UI (plano UI/UX W7). Plain data, como as guias — e aqui pelo mesmo motivo
    /// que elas: **gravar um estado tem de desfazer**. Ele é uma edição do documento, não uma
    /// preferência de vista.
    pub(crate) ui_states: ph2d_ui_state::StateSets,
    /// ⭐⭐⭐ **A BIBLIOTECA** — a taxonomia e o que o artista mandou sair dela (Enio, 2026-08-30:
    /// *«deveria ter undo/redo no painel inclusive em del»*). Plain data, como as guias e os
    /// estados de UI, e aqui pelo mesmo motivo que eles: **criar uma gaveta é autoria**.
    ///
    /// ⚠️ Ela é BYTES com uma cache por revisão, e o porquê está medido em
    /// [`crate::project_library`]: codificá-la por quadro custava até 28 % de um quadro.
    pub(crate) library: crate::project_library::LibraryDoc,
}

impl ProjectState {
    /// Captura o estado atual. `prop`/`worklist` são scratch reusado (o
    /// `world_to_snapshot` é zero-alloc além do crescimento do próprio snapshot).
    ///
    /// ⚠️ **O `drive` é o PRIMEIRO argumento de propósito: ele é a pergunta *«o que aqui está a
    /// ser escrito por um motor?»*, e ela tem de ser respondida para haver captura nenhuma.** Foi
    /// posto na assinatura — e não numa função-irmã «com ledger» — porque uma segunda porta é
    /// exactamente como o defeito voltaria: quem capturasse pela porta antiga fotografava o
    /// instante em vez do documento, e o Ctrl+Z voltava a gastar-se a desfazer relógio
    /// ([`crate::preview_drive`]). Sem condução nenhuma (`PreviewDrive::default()`, o caso normal)
    /// o custo é zero e o resultado é byte-a-byte o de antes.
    ///
    /// ⚠️ Dez argumentos, e eles são **dez fatos independentes** — o ledger, o mundo, as três
    /// geometrias, a biblioteca, o registro e o scratch. Agrupá-los num struct só para agradar ao lint criaria
    /// um tipo cuja única razão de existir é a contagem, e todo chamador passaria a montá-lo.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub(crate) fn capture(
        drive: &crate::preview_drive::PreviewDrive,
        sim: &mut SimWorld,
        vec: &VecScene,
        flip: &FlipDoc,
        guides: &ph2d_guides::GuideSet,
        ui_states: &ph2d_ui_state::StateSets,
        library: &crate::project_library::LibraryDoc,
        registry: &ComponentRegistry,
        cache: &mut ph2d_ecs::scene::incremental::CaptureCache,
        // ⭐⭐ **O passo ANTERIOR, para lhe reaproveitar a cena** — ver o campo [`Self::vec`].
        //
        // ⚠️ **A comparação de conteúdo que isto faz já era paga**: o `post_frame_undo` compara o
        // estado inteiro com o baseline logo a seguir. ⇒ a partilha troca *clonar e depois
        // comparar* por *comparar e clonar só se diferir* — estritamente mais barato, e não só em
        // memória.
        prev: Option<&Self>,
    ) -> Self {
        // O mundo passa ao estado AUTORADO só durante a fotografia, e volta ao vivo a seguir.
        let live = drive.substitute_authored(sim);
        let mut world = WorldSnapshot::new();
        // ⭐ **A captura é INCREMENTAL desde a F2** (ADR-0164 §2.7): ela reaproveita a linha de
        // quem não mudou, então um passo custa o tamanho da EDIÇÃO e não o do mundo.
        //
        // ⚠️ **O `substitute_authored`/`restore_live` à volta CARIMBAM ticks** nas entidades sob
        // condução — e isso está certo: elas ficam «sujas» no pré-filtro e a **comparação de
        // bytes** absorve-as, porque o valor AUTORADO não mudou. O preço é ler essas poucas
        // linhas; o `CaptureReport` mede-o (`dirty − reserialized`), que é precisamente o campo
        // que existe para tornar este custo visível em vez de suposto.
        //
        // O snapshot só falha se um componente registrado não (de)serializa — um bug de registro,
        // não estado do usuário. Um estado vazio é o degradado seguro.
        let _ = ph2d_ecs::scene::incremental::capture_incremental(
            sim.world_mut(),
            cache,
            registry,
            &mut world,
        );
        crate::preview_drive::PreviewDrive::restore_live(sim, &live);
        Self {
            world,
            vec: match prev {
                Some(p) if *p.vec == *vec => std::sync::Arc::clone(&p.vec),
                _ => std::sync::Arc::new(vec.clone()),
            },
            flip: flip.clone(),
            guides: guides.clone(),
            ui_states: ui_states.clone(),
            // ⚠️ **Um `clone` de bytes já codificados, e é isso que o torna barato** — quem
            // codifica é a `LibraryCache`, uma vez por mutação da árvore.
            library: library.clone(),
        }
    }

    /// Restaura este estado. Limpa as entidades editáveis do mundo, re-spawna do
    /// snapshot, e devolve as geometrias + as **pontes reconstruídas** (vetor e
    /// Flip; os mapas são runtime-only — sem o rebuild, o `sync` duplicaria as
    /// formas/objetos).
    ///
    /// O chamador atribui os quatro: `gfx.vec_scene = vec; gfx.flip = flip;
    /// self.vec_entities = vec_map; self.flip_entities = flip_map`.
    #[must_use]
    pub(crate) fn restore(
        &self,
        sim: &mut SimWorld,
        registry: &ComponentRegistry,
    ) -> (
        VecScene,
        crate::vec_entities::VecEntityMap,
        FlipDoc,
        crate::flip_entities::FlipEntityMap,
    ) {
        // 1. Limpa: toda entidade editável tem `Transform` (sprites, formas,
        //    objetos Flip, grupos). O despawn cascateia por `ChildOf`, então um
        //    filho já removido é benigno.
        let editable: Vec<Entity> = {
            let mut q = sim.world_mut().query_filtered::<Entity, With<Transform>>();
            q.iter(sim.world()).collect()
        };
        for e in editable {
            let _ = sim.world_mut().despawn(e);
        }
        // 2. Re-spawna do snapshot (ids do mundo são novos — o snapshot é portável).
        let _ = snapshot_to_world(sim.world_mut(), &self.world, registry);
        // 3. Reconstrói as pontes a partir dos `VecPathRef`/`FlipObjectRef`
        //    restaurados.
        let vec_map = crate::vec_entities::rebuild_map(sim);
        let flip_map = crate::flip_entities::rebuild_map(sim);
        ((*self.vec).clone(), vec_map, self.flip.clone(), flip_map)
    }
}

/// ⭐ **As leis da seleção que sobrevive ao undo** vivem no irmão — ver
/// [`undo_selection`](self::selection).
#[path = "undo_selection.rs"]
mod selection;
pub(crate) use selection::{field_selection_back, field_selection_ids, surviving_selection};

// ⭐ **`canonicalize` MORREU AQUI** (ADR-0164 F1, snapshot v2).
//
// Ela reordenava as linhas do snapshot por CONTEÚDO a cada captura, para que dois estados
// logicamente iguais dessem bytes iguais — porque a ordem vinha do `Entity::to_bits()`, o id
// de ALOCAÇÃO, que muda a cada respawn do undo. Sem ela, todo quadro com input registava um
// passo espúrio e o Ctrl+Z parecia "não fazer nada" (Enio, 2026-07-09).
//
// ⚠️ **A propriedade não foi retirada — ela mudou de dono.** O `world_to_snapshot` agora
// ordena por `StableId`, que sobrevive ao respawn **por construção**; a invariância vem da
// identidade em vez de vir de reler os bytes. E o preço muda de classe: a chave desta função
// era a serialização INTEIRA de cada linha (~230 B), construída **dentro do comparador** do
// `sort_by` — ~266 k alocações a 10 k entidades, **18,7 ms** medidos, contra **0,088 ms** de
// um sort por inteiro (doc 04 §1.1).
//
// ⛔ Não a reintroduza "para garantir": duas ordens canónicas é a divergência que a F2 vai
// pagar, porque o cache incremental dela é chaveado pela mesma identidade.

/// A pilha de undo/redo global. O registro de passos é dirigido por **diff de
/// estado** no [`crate::App::post_frame_undo`] (não por begin/commit de gesto), então
/// a API é só `push_undo` + `undo`/`redo`.
#[derive(Default)]
pub(crate) struct ProjectUndo {
    undo: Vec<ProjectState>,
    redo: Vec<ProjectState>,
}

impl ProjectUndo {
    /// Empurra um estado-pré (o baseline antes da ação detectada). Limpa o redo.
    pub(crate) fn push_undo(&mut self, pre: ProjectState) {
        if self.undo.len() >= UNDO_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Quantos passos há na pilha de undo (para o log de diagnóstico).
    pub(crate) fn depth(&self) -> usize {
        self.undo.len()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Desfaz: devolve o estado anterior; empurra o `current` pro redo.
    #[must_use]
    pub(crate) fn undo(&mut self, current: ProjectState) -> Option<ProjectState> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    /// Refaz: devolve o próximo estado; empurra o `current` de volta pro undo.
    #[must_use]
    pub(crate) fn redo(&mut self, current: ProjectState) -> Option<ProjectState> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }
}

/// ⭐⭐ **Quem OPERA a fila a partir da `App`** vive no irmão — ver [`app`].
///
/// ⚠️ Ele saiu daqui na integração de 2026-09-04, quando duas linhas somadas levaram este
/// arquivo a `620 / 600`. O cabeçalho de lá tem o mecanismo.
#[path = "undo_app.rs"]
mod app;

#[cfg(test)]
#[path = "undo_tests.rs"]
mod tests;

/// ⭐⭐ **O que um passo PARTILHA com o anterior** (F8) — irmão por assunto do [`tests`], ver o
/// cabeçalho de lá. A pergunta ali é de igualdade; aqui é de IDENTIDADE.
#[cfg(test)]
#[path = "undo_sharing_tests.rs"]
mod sharing_tests;

#[cfg(test)]
#[path = "undo_selection_tests.rs"]
mod selection_tests;
