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

impl crate::App {
    /// Captura o estado do projeto AGORA. `None` se o gfx ainda não inicializou.
    ///
    /// ⚠️ **Ela fotografa o DOCUMENTO, não o instante.** Enquanto um motor conduz uma entidade —
    /// uma animação a tocar, o solver a simular —, o valor autorado é reposto no mundo durante a
    /// fotografia e o vivo é devolvido logo a seguir ([`crate::preview_drive`]). Sem isto, cada
    /// clique dado enquanto algo se move sozinho empilhava um passo de undo cujo conteúdo era só o
    /// relógio (ou só a pose), e o Ctrl+Z do artista gastava-se a desfazer nada.
    ///
    /// ⚠️ **E vale para o SAVE pela mesma porta**, que é a lei deste módulo (undo e save gravam a
    /// MESMA captura): gravar a meio de uma reprodução guarda a célula que o artista escolheu, não
    /// onde o ciclo calhou estar.
    #[must_use]
    pub(crate) fn capture_project(&mut self) -> Option<ProjectState> {
        // ⭐⭐⭐ **O MUNDO E O DOCUMENTO TÊM DE CONCORDAR ANTES DA FOTOGRAFIA** (report do Enio,
        // 2026-08-27: *«as peças apagadas voltaram sem pais e na posição (0,0) do mundo»*).
        //
        // ⛔ **O mecanismo, reproduzido por sonda:** a reconciliação `path ⟺ entidade` corre CEDO
        // no quadro (`render_loop/mod.rs`, antes do canvas) e o *Delete* da Hierarquia corre TARDE.
        // Logo o quadro em que uma peça vetorial é apagada **termina inconsistente**: a entidade já
        // morreu e o `VecPath` dela ainda está no documento. Esta captura fotografava esse
        // instante, e o Ctrl+Z repunha-o — aí a reconciliação do quadro seguinte via *«um path sem
        // entidade»* e **CUNHAVA** uma: `Transform::default()`, sem `ChildOf`, chamada `Path N`.
        // Isto é, exactamente, um objeto **sem pai na origem do mundo**.
        //
        // ```text
        // UNDO+1: Path 0<-None@Vec2(0.0, 0.0) | Path 1<-None@Vec2(0.0, 0.0)
        // ```
        //
        // ⚠️ **Aqui, e não no dreno do Delete:** todo `despawn` produz a mesma inconsistência, e o
        // Delete é só um dos produtores. E é aqui que ela deixa de existir para os DOIS
        // consumidores — *undo e save gravam a MESMA captura*, que é a lei deste módulo: um save
        // feito no mesmo quadro escrevia o fantasma no ficheiro.
        //
        // ⚠️ **Sim, esta chamada pode CRIAR entidades** (a 3.ª direcção do `sync` cunha a entidade
        // de um path novo). É o que o quadro seguinte faria de qualquer maneira, e fazê-lo antes da
        // fotografia é o que põe a entidade **dentro** do passo de undo em vez de fora dele.
        // Idempotente: com o mundo e o documento a par, é no-op.
        //
        // ⚠️ **Empréstimos DISJUNTOS de `self`** — `gfx` e `vec_entities` são campos diferentes.
        if let Some(gfx) = self.gfx.as_mut() {
            crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
        }
        // ⚠️ Empréstimos DISJUNTOS de `self` — o ledger e o `gfx` são campos diferentes, e é por
        // isso que os dois podem estar vivos ao mesmo tempo.
        let drive = &self.preview_drive;
        let baseline = self.undo_baseline.as_ref();
        let gfx = self.gfx.as_mut()?;
        // ⚠️ **A biblioteca é codificada AQUI, e a cache é que a torna barata** — ver
        // [`crate::project_library`]: sem ela isto custava até 28 % de um quadro, em todo quadro
        // com input.
        let library = gfx.library_cache.doc(&gfx.catalogs).clone();
        Some(ProjectState::capture(
            drive,
            &mut gfx.sim,
            &gfx.vec_scene,
            &gfx.flip,
            &gfx.guides,
            &gfx.ui_states,
            &library,
            &gfx.component_registry,
            &mut gfx.undo_capture_cache,
            // ⚠️ **O baseline é o passo anterior**, e é dele que a cena é reaproveitada quando o
            // documento vetorial não mudou. Empréstimos disjuntos: `undo_baseline` e `gfx` são
            // campos diferentes de `self`.
            baseline,
        ))
    }

    /// Aplica um estado restaurado ao App: restaura o mundo + a geometria, reconstrói
    /// a ponte, e limpa os **bits** de seleção (o restore dá ids de entidade NOVOS, então
    /// bits antigos ficam mortos). Re-arma o baseline para não re-detectar como mudança.
    ///
    /// ⚠️ **Os bits morrem; a SELEÇÃO não precisa morrer com eles.** Até 2026-07-18 esta função
    /// zerava também o `vec_pen`, e a consequência era um bug que o Enio reportou como *"o undo faz
    /// os pins sumirem, embora ainda funcionando"*: o envelope segue a deformar (o recook varre por
    /// QUERY) e o **overlay** — gaiola e pinos — é desenhado a partir da seleção, que já não existe.
    /// A ferramenta ficava funcionando e invisível.
    ///
    /// A seleção do pen é `VecPathId` e **viaja no snapshot** (a `VecScene` é parte do estado), então
    /// ela sobrevive: [`surviving_selection`] mantém os ids que ainda existem na cena restaurada, e o
    /// `sync_selection` do frame seguinte re-deriva os bits do mapa reconstruído. Nada de bits mortos
    /// atravessa — que era o perigo real que o zeramento defendia.
    pub(crate) fn apply_project(&mut self, state: &ProjectState) {
        // ⛔ **As lápides ANTES da guarda do `gfx`** — elas vivem num global, e deixá-las dentro
        // dela fazia um restauro sem GPU manter as da sessão anterior. Ver `project_library`.
        crate::project_library::apply_forgotten(&state.library);
        // ANTES do restore: o que o artista tinha selecionado, em ids ESTÁVEIS.
        let was_selected = self.vec_pen.selected_paths().to_vec();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐⭐⭐ **E a seleção 3D também, pela mesma razão e na mesma unidade** — ver
        // [`field_selection_ids`]. ⚠️ Tem de ser AQUI, antes do `restore`: depois dele os bits que a
        // seleção guarda já apontam para entidades que deixaram de existir.
        let was_field: Vec<ph2d_ecs::StableId> =
            gfx.hero_screen.as_ref().map_or_else(Vec::new, |h| {
                let bits: Vec<u64> = h
                    .gizmo
                    .selection
                    .iter()
                    .copied()
                    .chain(h.gizmo.extra_selection.iter().copied())
                    .collect();
                field_selection_ids(gfx.sim.world(), &bits)
            });
        let (vec, map, flip, flip_map) = state.restore(&mut gfx.sim, &gfx.component_registry);
        gfx.vec_scene = vec;
        gfx.flip = flip;
        gfx.guides = state.guides.clone();
        gfx.ui_states = state.ui_states.clone();
        // ⭐ A seleção 3D volta com os bits NOVOS, depois de o mundo ter sido reconstruído.
        let field_back = field_selection_back(gfx.sim.world_mut(), &was_field);
        // ⭐⭐⭐ **A biblioteca volta** — a taxonomia e as lápides (Enio, 2026-08-30).
        //
        // ⛔ **E a cache TEM de ser invalidada**, senão o quadro seguinte devolveria os bytes
        // antigos: a revisão é por-árvore e a restaurada nasce em `0`, então colidir com a que a
        // cache já viu é o caso NORMAL, não o raro.
        gfx.catalogs = crate::project_library::apply_catalogs(&state.library);
        gfx.library_cache.invalidate();
        if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.clear_all_selection();
        for bits in field_back {
            hero.gizmo.add_to_selection(bits);
        }
    }
        self.vec_entities = map;
        self.flip_entities = flip_map;
        self.vec_sel = crate::vec_selection::VecSelSync::default();
        self.vec_pen.clear();
        // E de volta, filtrada pelo que sobreviveu. O `vec_sel` ficou zerado de propósito: no frame
        // seguinte o `sync_selection` vê "o pen mudou" e republica os bits NOVOS no gizmo.
        let alive = surviving_selection(&was_selected, &gfx.vec_scene);
        if !alive.is_empty() {
            self.vec_pen.select_many(&alive);
        }
        // Live Shapes: uma SESSÃO de texto viva reescreveria o `VecShape` (e a pose) da
        // entidade com os params dela a cada frame — ou seja, desfaria o undo no frame
        // seguinte. O estado restaurado é a verdade: a sessão termina (o objeto de texto
        // continua lá, editável pela seleção).
        self.vec_text_edit = None;
        self.vec_text_last_target = None;
        // O cache de poses dos rótulos é a memória de "o que EU escrevi no frame passado". O
        // restore põe poses novas no mundo sem passar por ele — deixá-lo faria o passe ler a
        // pose restaurada como um arrasto do usuário. É inócuo (o estado restaurado é
        // auto-consistente, e re-absorver devolve o MESMO offset), mas zerar é o honesto: a
        // memória não é mais de nada. O arm de rótulo pendente morre com a sessão de texto.
        self.vec_label_poses.clear();
        self.vec_label_pending = None;
        // O memo do Offset vivo é chaveado por `VecPathId`, e o restore RECICLA os ids: um id
        // que volta descrevendo outra forma acertaria o memo e desenharia o offset da forma
        // ANTERIOR sobre a nova, sem erro nenhum. O espelho do painel morre pela mesma razão
        // (a forma que ele espelhava pode ter deixado de ter offset).
        self.offset_live.forget();
        // O Pattern vivo pela mesma razão do offset.
        self.pattern_live.forget();
        self.contour_live.forget();
        // A SIMETRIA viva pela MESMA razão (id reciclado; o memo é chaveado por `VecPathId`).
        self.symmetry_live.forget();
        self.profile_live.forget();
        // O FX raster vivo (plano 24) pela mesma razão: a cena inteira mudou debaixo do cozimento.
        self.fx_live.forget();
        self.fx_silhouette.forget();
        self.vec_offset_mirrored = None;
        // A mesma forma (mesmo id) pode voltar com OUTROS parâmetros — zerar o alvo
        // força a re-semente dos sliders, senão o painel seguiria mostrando o valor
        // que o undo acabou de desfazer.
        self.vec_shape_last_focus = None;
        // ⚠️ **E o latch de «armado» também**: o restore repõe outra selecção, e um latch de antes
        // do undo descreveria um gesto que já não aconteceu.
        self.vec_shape_armed = false;
        self.vec_shape_armed_target = None;
        // O Colorize ao vivo guarda a base congelada de um desenho que este restore acaba de
        // substituir — re-Aplicar sobre ela apagaria o estado restaurado. A sessão termina.
        self.flip_colorize.end_live();
        self.undo_baseline = Some(state.clone());
        self.title_dirty = true;
    }

    /// Desfaz (ou refaz) um passo da fila global: empurra o estado atual pro outro
    /// lado e aplica o restaurado.
    pub(crate) fn apply_undo(&mut self, redo: bool) {
        let Some(current) = self.capture_project() else {
            return;
        };
        let restored = if redo {
            self.undo.redo(current)
        } else {
            self.undo.undo(current)
        };
        if let Some(state) = restored {
            self.apply_project(&state);
            if Self::undo_log_on() {
                eprintln!("[undo] {} aplicado", if redo { "redo" } else { "undo" });
            }
        } else if Self::undo_log_on() {
            eprintln!("[undo] {} sem passo", if redo { "redo" } else { "undo" });
        }
    }

    /// **O log do undo está ligado?** Cacheado — os três sítios que o consultam correm por
    /// passo de undo, e ler o ambiente é caro o suficiente para não o fazer em cada um.
    pub(crate) fn undo_log_on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("PH2D_UNDO_LOG").is_some())
    }

    /// Chamado UMA vez por frame, depois de `render_frame` (com `self` livre e o
    /// estado já reconciliado pelo `sync`). Duas coisas, nesta ordem:
    ///
    /// 1. **Ctrl+Z/Y** pendente: aplica e re-arma o baseline (então o passo 2 vê o
    ///    estado restaurado == baseline e não registra um passo espúrio).
    /// 2. **Auto-commit**: se houve input neste frame e nenhum gesto está em curso,
    ///    compara o estado atual com o baseline; se mudou, o baseline antigo (o
    ///    estado-pré) vira um passo de undo e o novo vira o baseline. É o ponto único
    ///    que torna "tudo que o usuário fizer" desfazível, por diff — sem instrumentar
    ///    cada ação.
    pub(crate) fn post_frame_undo(&mut self) {
        // ⚠️ **PRIMEIRO, e antes de qualquer captura:** o ledger da condução esquece quem os
        // motores não declararam neste quadro. Quem parou volta a ser documento **já nesta
        // fotografia** — é isso que faz uma corrida inteira colapsar em UM passo, em vez de zero.
        self.preview_drive.settle();
        // ⭐⭐⭐ **E o que o módulo 3D autorou SEM evento** (W115) — a forma que a paleta escolheu,
        // a escultura que o diálogo carregou. Elas chegam por **pedido servido noutro quadro**, e
        // sem esta metade nasciam **sem passo próprio**, fundindo-se na acção seguinte do artista.
        //
        // ⚠️ **Tirada em TODO quadro, ao lado da outra** — deixá-la pousada faria a próxima
        // supressão legítima registar um passo que já foi registado.
        let had_input = std::mem::take(&mut self.any_input_this_frame)
            | crate::field3d_smoke::take_authored_change();
        // O clique no botão Undo/Redo da barra entra pela MESMA porta do Ctrl+Z (que pode
        // rotear para o Áudio, o Painter, o global ou o image-edit). Ele arma o
        // `undo_request` logo abaixo, e o passo é aplicado ainda neste frame.
        if let Some(redo) = self.undo_button.take() {
            self.undo_or_redo(redo);
        }
        if let Some(redo) = self.undo_request.take() {
            self.apply_undo(redo);
            return;
        }
        // Estabelece o baseline no PRIMEIRO frame (gfx pronto), antes de qualquer
        // ação. Sem isto a primeira ação não teria pré-estado e não seria desfazível.
        if self.undo_baseline.is_none() {
            self.undo_baseline = self.capture_project();
            return;
        }
        // Um gesto em andamento (botão pressionado) muta a cada Move; espera o fim
        // para não registrar um passo por frame.
        //
        // ⚠️ **Um recálculo assíncrono pendente conta como gesto em andamento** — é a MESMA
        // frase acima aplicada a um trabalho que continua depois do botão soltar. O ajuste
        // Trap/Bleed do Colorize roda fora da thread de UI (`09 §7.2`, 304 ms/tique), então
        // soltar o slider registraria um passo com o resultado ANTIGO e a chegada do worker
        // registraria um segundo: dois Ctrl+Z para um arrasto.
        // ⚠️ **Uma PREVIEW ao vivo tambem conta como gesto em andamento** — e a MESMA frase, num
        // trabalho que o produto faz sozinho: uma transição de ESTADO de UI (plano UI/UX W7) põe
        // a cena numa pose DIFERENTE a cada quadro, então sem a supressão um Show de 150 ms
        // viraria nove passos de undo. Quando ela chega, a cena está numa pose AUTORADA e o diff
        // registra um — o preço certo de *"eu mostrei o hover"*.
        // ⚠️ **O gizmo 3D de modelagem tem de dizer-se por conta própria** (ADR-0161 W6): o gancho
        // de ponteiro dele consome o `Down` e volta ANTES da linha que escreve o `held_button`, então
        // a condição acima — que está certa — não alcançava aquele gesto. Sem esta, arrastar uma
        // seta registava um passo de undo POR QUADRO.
        //
        // ⭐⭐⭐ **E o motivo tem NOME** (W114) — porque a pergunta que um report de *«o undo pula
        // etapas»* faz é exactamente *«qual destas cinco comeu o meu passo?»*.
        //
        // ⚠️ **Uma mudança suprimida NÃO se perde: ela funde-se no PRÓXIMO passo**, porque o
        // `undo_baseline` só é substituído quando um passo é registado. ⇒ duas acções viram um
        // `Ctrl+Z` só, que é o sintoma que o artista descreve. *Um passo suprimido e um passo
        // ausente leem-se iguais de fora, e as causas são opostas.*
        let motivo = if !had_input {
            Some("sem entrada neste quadro")
        } else if self.held_button.is_some() {
            Some("botao do rato em baixo")
        } else if crate::field3d_smoke::gesture_in_progress() {
            Some("arrasto do gizmo 3D em curso")
        } else if self.flip_colorize.live_busy(self.flip_style.as_ref()) {
            Some("colorize a recalcular")
        } else if self.ui_state_live {
            Some("transicao de estado de UI ao vivo")
        } else {
            None
        };
        if let Some(motivo) = motivo {
            // ⚠️ **A captura só acontece com o log LIGADO** — ela é o passo caro do quadro, e
            // pagá-la em toda supressão seria trocar um diagnóstico por uma regressão de relógio.
            if Self::undo_log_on()
                && let Some(atual) = self.capture_project()
                && self.undo_baseline.as_ref() != Some(&atual)
            {
                eprintln!(
                    "[undo] ⛔ o documento MUDOU e o passo foi SUPRIMIDO — motivo: {motivo}                      (ela vai FUNDIR-SE no proximo passo)"
                );
            }
            return;
        }
        let Some(current) = self.capture_project() else {
            return;
        };
        if self.undo_baseline.as_ref() == Some(&current) {
            return; // nada mudou desde o último passo
        }
        if Self::undo_log_on() {
            let base = self.undo_baseline.as_ref();
            // ⭐ O que a captura INCREMENTAL fez (F2): quantas linhas o pré-filtro acusou,
            // quantas de facto mudaram, e **quantos bytes** o passo custa. ⚠️ Um `sujas` muito
            // maior que `reserializadas` é o falso positivo do `DerefMut` — alguém reescreve o
            // que já lá estava, e a cura é `set_if_neq` em quem escreve.
            let cap = self
                .gfx
                .as_ref()
                .map(|g| g.undo_capture_cache.last_report())
                .unwrap_or_default();
            eprintln!(
                "[undo]   captura: sujas={} reserializadas={} nascidas={} mortas={} linhas={} delta={} B",
                cap.dirty, cap.reserialized, cap.spawned, cap.despawned, cap.rows, cap.delta_bytes
            );
            eprintln!(
                "[undo] passo registrado (fila undo={}, {} sob conducao) — diff: world={} vec={} flip={}",
                self.undo.depth() + 1,
                // ⚠️ Quantas entidades um motor está a escrever AGORA. Um passo registado com este
                // número **alto** é a pergunta certa a fazer: ou a condução não foi declarada, ou o
                // artista de facto editou (`crate::preview_drive`).
                self.preview_drive.len(),
                base.is_some_and(|b| b.world != current.world),
                base.is_some_and(|b| b.vec != current.vec),
                base.is_some_and(|b| b.flip != current.flip),
            );
            if let Some(b) = base
                && b.vec != current.vec
            {
                let ids = |v: &VecScene| v.paths().iter().map(|p| p.id).collect::<Vec<_>>();
                let (a, c) = (ids(&b.vec), ids(&current.vec));
                let mut sa = a.clone();
                let mut sc = c.clone();
                sa.sort_unstable();
                sc.sort_unstable();
                eprintln!(
                    "[undo]   vec: base={a:?} atual={c:?} · mesmos ids={} · só a ORDEM={}",
                    sa == sc,
                    sa == sc && a != c
                );
            }
        }
        if let Some(base) = self.undo_baseline.replace(current) {
            self.undo.push_undo(base);
        }
    }
}

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
