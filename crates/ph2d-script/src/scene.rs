//! ⭐⭐⭐ **OS SCRIPTS DA CENA** — um módulo por ficheiro, uma tabela `self` por objecto, e os três
//! ganchos a correr no passo fixo (TOP-20 #16, W2). O plano é o
//! `docs/Components/13_plano_script_properties.md` §3.
//!
//! # O que este módulo decide, e o que ele NÃO decide
//!
//! **Decide** que script cada objecto corre, com que números, em que ordem, e o que as escritas
//! dele fazem ao `Transform`. **Não decide** QUANDO a cena corre (é o relógio da shell), nem como
//! uma escrita entra no `Ctrl+Z` (é o ledger do `preview_drive`, na ponte): ele devolve os FACTOS
//! ([`SceneReport`]) e a ponte declara-os.
//!
//! # ⚠️ As leis que ele honra, e onde cada uma foi paga
//!
//! - **Rebobinar é renascer** ([`SceneScripts::rewind`]): as tabelas `self` são o VIVO, e o vivo
//!   volta ao que um objecto acabado de criar recebe — o `init` corre outra vez. É a porta da
//!   família `Logic` (`ph2d_ecs::rewind_runtime`), um nível ao lado: a VM não mora no mundo.
//! - **A ordem é a da IDENTIDADE** (HR-5): dois objectos que escrevem no mesmo tique fazem-no
//!   sempre pela ordem do `StableId`, nunca pela da query.
//! - **Um passo fixo, uma chamada.** Ao contrário da lei pura do relógio, um script **não tem laço
//!   de recuperação** — agrupar tiques num `dt` maior faria o replay depender da taxa de quadros.
//! - **Todos leem a mesma fotografia** (HR-8): as poses são tiradas no início do tique e as
//!   escritas aplicadas no fim. Um script que escreve e lê o próprio `x` no mesmo `update` lê o
//!   valor de antes — é o modelo com fila que a casa escolheu no M7, não um acidente.
//! - **Cada escrita tem DONO**: a fila é drenada depois de CADA gancho, então um campo desconhecido
//!   é culpa do script que o escreveu, e o painel diz qual.
//! - ⚠️ **Recarregar NÃO renasce.** Um texto que muda troca as funções e mantém cada `self` (o
//!   *reset+restore* do HR-16): um objecto a meio de um salto continua o salto com a lei nova. E um
//!   script que tinha partido volta a correr — a correcção é a razão de o artista ter gravado.
//! - ⭐ **Um texto que não compila é RECUSADO e a última versão boa continua** (CodeEditor E6,
//!   `scene_reload.rs`): apagar um `end` não pára a cena; o diagnóstico fica no [`ScriptInfo::Broken`].
//! - **O texto vem da BIBLIOTECA do projeto** (ADR-0188 §2), por [`ScriptId`]: recarrega quando o
//!   `blake3` do texto (ou o nome) muda — sem disco. Um `Unimported` nunca corre.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use mlua::{Lua, Table, Value};
use ph2d_ecs::{Entity, StableId, Transform, World};

use crate::component::{LuauScript, ScriptSource};
use crate::console::speaking;
use crate::diagnostic::ScriptDiagnostic;
use crate::io::{EntityWrite, ReadSnapshot, WriteQueue, pose_field};
use crate::module::ScriptModule;
use crate::props::{PropDecl, ScriptValue, resolve};
use ph2d_script_doc::{ScriptId, ScriptLibrary};

/// **O que se sabe de um script** — o que o painel mostra.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptInfo {
    /// Carregou; estas são as declarações dele.
    Ready(Vec<PropDecl>),
    /// O Luau recusou o texto ACTUAL — o diagnóstico diz porquê e onde. Os objectos continuam na
    /// versão anterior quando havia uma ([`SceneScripts::runs_previous`]).
    Broken(ScriptDiagnostic),
    /// A biblioteca não tem este id (um undo que voltou a apontar um script apagado).
    Missing(String),
}

impl ScriptInfo {
    /// As declarações, **se forem conhecidas** — o `Option` que a [`resolve`] exige (Q10).
    #[must_use]
    pub fn decls(&self) -> Option<&[PropDecl]> {
        match self {
            Self::Ready(d) => Some(d),
            Self::Broken(_) | Self::Missing(_) => None,
        }
    }
}

/// Um script da biblioteca, carregado ou não.
struct Slot {
    /// O nome na biblioteca quando o texto foi lido pela última vez.
    name: String,
    hash: Option<[u8; 32]>,
    /// A última versão BOA — fica quando um texto novo é recusado (`scene_reload.rs`).
    module: Option<ScriptModule>,
    /// O nome do chunk com que o `module` foi carregado (o que o diagnóstico procura).
    chunk: String,
    info: ScriptInfo,
    /// Quantos objectos o apontam (no último `sync`).
    users: usize,
    /// `request_reload`: recarrega mesmo com o texto igual.
    forced: bool,
}

impl Slot {
    fn empty() -> Self {
        Self {
            name: String::new(),
            hash: None,
            module: None,
            chunk: String::new(),
            info: ScriptInfo::Missing(String::new()),
            users: 0,
            forced: false,
        }
    }
}

/// O vivo de UM objecto.
struct Instance {
    /// O script que ele corre — mudar de script é renascer.
    script: ScriptId,
    /// O nome do chunk desse script — o que o diagnóstico procura nas mensagens.
    chunk: String,
    /// A tabela `self`.
    this: Table,
    /// Os valores que já foram escritos no `self` — só uma MUDANÇA volta a escrever (§3.5).
    applied: BTreeMap<String, ScriptValue>,
    /// Porque parou de correr, se parou.
    failed: Option<ScriptDiagnostic>,
}

/// **O que um tique fez** — os factos que a ponte declara.
#[derive(Default, Debug)]
pub struct SceneReport {
    /// Objectos cujo `Transform` o script mudou neste tique.
    pub wrote: Vec<Entity>,
    /// Sinais emitidos, com quem os emitiu, pela ordem da identidade.
    pub emitted: Vec<(Entity, String)>,
    /// Objectos que pararam de correr NESTE tique, com o diagnóstico.
    pub failed: Vec<(Entity, ScriptDiagnostic)>,
    /// Quantos ganchos correram.
    pub calls: usize,
    /// ⭐ Ciclo 19: o que os scripts escreveram na tabela `motion` (já aplicado ao recurso).
    pub motion: Vec<crate::motion_api::MotionWrite>,
}

/// **O tempo que UMA chamada de gancho pode levar: um quadro.**
///
/// ⚠️ **De que recurso ele é:** do QUADRO (60 Hz ⇒ 16,7 ms). Uma chamada que o passa já congelou o
/// app de forma visível, e a única explicação comum é um laço sem saída — que sem este prazo
/// **congelaria o editor para sempre**, com o trabalho do artista por gravar. ⛔ Não é um orçamento
/// da cena (a soma de todos os scripts): uma cena pesada e legítima perde quadros, não scripts.
pub const HOOK_BUDGET: std::time::Duration = std::time::Duration::from_micros(16_667);

/// **O prazo da chamada em curso**, partilhado com a interrupção da VM. `u64::MAX` = sem prazo.
#[derive(Clone)]
pub(crate) struct Deadline(pub(crate) std::sync::Arc<std::sync::atomic::AtomicU64>);

impl Default for Deadline {
    fn default() -> Self {
        Self(std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            u64::MAX,
        )))
    }
}

/// O relógio da interrupção: nanossegundos desde a primeira leitura do processo.
pub(crate) fn now_nanos() -> u64 {
    static BASE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let base = *BASE.get_or_init(std::time::Instant::now);
    u64::try_from(base.elapsed().as_nanos()).unwrap_or(u64::MAX - 1)
}

impl Deadline {
    /// Abre o prazo de uma chamada.
    pub(crate) fn arm(&self) {
        let budget = u64::try_from(HOOK_BUDGET.as_nanos()).unwrap_or(u64::MAX);
        self.0.store(
            now_nanos().saturating_add(budget),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    /// Fecha-o.
    pub(crate) fn disarm(&self) {
        self.0.store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
    }

    /// Chama `f` com o prazo aberto — e fecha-o com qualquer resultado.
    pub(crate) fn within<T>(&self, f: impl FnOnce() -> T) -> T {
        self.arm();
        let out = f();
        self.disarm();
        out
    }
}

/// **A fila dos sinais emitidos** — `ph2d.emit(nome)` empurra aqui; o executor drena por gancho.
#[derive(Clone, Default)]
pub(crate) struct EmitQueue(pub(crate) std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl EmitQueue {
    fn drain(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(|p| p.into_inner()))
    }
}

/// ⭐ **Os scripts da cena.** Vive dentro do `ScriptHost`, que é quem tem a VM.
#[derive(Default)]
pub struct SceneScripts {
    slots: BTreeMap<ScriptId, Slot>,
    instances: BTreeMap<u64, Instance>,
    /// As recargas por drenar — uma por script (`scene_reload.rs`).
    reloads: BTreeMap<ScriptId, ScriptReload>,
    /// A biblioteca que os `slots` seguem; outra = outro projecto (os ids recomeçam em 1).
    epoch: Option<crate::LibraryEpoch>,
}

/// Um objecto com script, no instante do tique.
struct Actor {
    entity: Entity,
    cfg: LuauScript,
}

/// Os objectos com script, **pela ordem da identidade**.
fn actors(world: &mut World) -> Vec<Actor> {
    let mut out: Vec<(u64, u64, Actor)> = world
        .query::<(Entity, &LuauScript, Option<&StableId>)>()
        .iter(world)
        .map(|(entity, cfg, sid)| {
            (
                // ⚠️ Sem identidade vai para o fim — `0` é o «nenhum» da casa, e o `u64::MAX`
                // impede que ele passe à frente de quem tem uma.
                sid.map_or(u64::MAX, |s| s.0),
                entity.to_bits(),
                Actor {
                    entity,
                    cfg: cfg.clone(),
                },
            )
        })
        .collect();
    out.sort_by_key(|(sid, bits, _)| (*sid, *bits));
    out.into_iter().map(|(_, _, a)| a).collect()
}

/// ⭐ **A FOTOGRAFIA**: todos leem as poses do início da passagem (HR-8).
fn photograph(reads: &ReadSnapshot, world: &World, actors: &[Actor]) {
    reads.clear();
    for a in actors {
        if let Some(t) = world.get::<Transform>(a.entity) {
            reads.set_pose(a.entity.to_bits(), pose_of(t));
        }
    }
}

/// A pose como o script a lê.
fn pose_of(t: &Transform) -> [f64; 5] {
    [
        f64::from(t.translation.x),
        f64::from(t.translation.y),
        f64::from(t.rotation),
        f64::from(t.scale.x),
        f64::from(t.scale.y),
    ]
}

/// O valor como o script o lê em `self`.
///
/// ⚠️⚠️ **A tabela de um `vec2`/`color` sai da MESMA porta que o construtor `ph2d.vec2` usa**
/// ([`crate::valores::tabela_de`]): o que o artista escreve na declaração e o que ele lê em `self`
/// têm de ser a mesma forma, senão copiar uma para a outra produz um default que o painel não sabe
/// pintar. Há gate de ida-e-volta.
fn to_lua(lua: &Lua, v: &ScriptValue) -> mlua::Result<Value> {
    if let Some(t) = crate::valores::tabela_de(lua, v)? {
        return Ok(t);
    }
    Ok(match v {
        ScriptValue::Number(n) => Value::Number(*n),
        ScriptValue::Bool(b) => Value::Boolean(*b),
        ScriptValue::Text(s) => Value::String(lua.create_string(s)?),
        // A [`crate::valores::tabela_de`] já os devolveu acima.
        ScriptValue::Vec2(_) | ScriptValue::Color(_) => unreachable!("a porta ja' os cobriu"),
    })
}

impl SceneScripts {
    /// ⚠️ **A VM mudou** (o `load_script` do spike reconstrói-a): toda tabela guardada aqui é da VM
    /// velha. Esquece tudo; o próximo `sync` recarrega da biblioteca.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.instances.clear();
    }

    /// ⛔ **Outra biblioteca = outro projecto**: os módulos, os vivos e as recargas por drenar são
    /// do projecto anterior e nomeiam ids que o novo reusa — esquecê-los (senão o `#1` partido do
    /// projecto novo «continuaria a correr» o `#1` do anterior, e o selo diria «Reloaded»).
    /// `true` = havia um projecto antes e foi esquecido.
    pub(crate) fn follow_epoch(&mut self, epoch: Option<crate::LibraryEpoch>) -> bool {
        if self.epoch == epoch {
            return false;
        }
        // A 1.ª biblioteca (o arranque) não troca nada: não havia projecto a esquecer.
        let swapped = self.epoch.is_some();
        self.epoch = epoch;
        if swapped {
            self.clear();
            self.reloads.clear();
        }
        swapped
    }

    /// ⭐ **REBOBINAR É RENASCER** — o `init` volta a correr no próximo tique.
    pub fn rewind(&mut self) -> usize {
        let n = self.instances.len();
        self.instances.clear();
        n
    }

    /// O que se sabe do script `id` — `None` se nenhum objecto o apontou ainda.
    #[must_use]
    pub fn info(&self, id: ScriptId) -> Option<&ScriptInfo> {
        self.slots.get(&id).map(|s| &s.info)
    }

    /// Porque o objecto parou de correr o script, se parou.
    #[must_use]
    pub fn failure(&self, entity: Entity) -> Option<&ScriptDiagnostic> {
        self.instances
            .get(&entity.to_bits())
            .and_then(|i| i.failed.as_ref())
    }

    /// Quantos objectos têm o vivo nascido — o número que um gate lê.
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.instances.len()
    }

    /// ⭐ **Uma vez por QUADRO, a correr ou não**: recarrega o que mudou na biblioteca e esquece o
    /// vivo de quem já não tem script (ou trocou de script).
    ///
    /// ⚠️ **Parado também**, e é o que faz o painel ver as declarações no instante em que o artista
    /// escolhe o script — sem isto a linha de cada número só apareceria depois do Play.
    pub(crate) fn sync(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        world: &mut World,
        lib: &ScriptLibrary,
    ) {
        let actors = actors(world);
        let mut wanted: BTreeMap<ScriptId, usize> = BTreeMap::new();
        for id in actors.iter().filter_map(|a| library_id(&a.cfg)) {
            *wanted.entry(id).or_default() += 1;
        }
        // Quem ninguém aponta sai — o vivo de um script esquecido não fica na VM.
        self.slots.retain(|id, _| wanted.contains_key(id));
        for (id, users) in wanted {
            self.refresh(lua, deadline, lib, id, users);
        }
        let now: BTreeMap<u64, Option<ScriptId>> = actors
            .iter()
            .map(|a| (a.entity.to_bits(), library_id(&a.cfg)))
            .collect();
        self.instances
            .retain(|bits, inst| now.get(bits).is_some_and(|id| *id == Some(inst.script)));
    }

    /// ⭐ **Os sinais deste quadro chegam a quem os ouve** (`on_signal(self, nome)`), uma chamada
    /// por par. Só em quem já nasceu — um objecto que ainda não correu um tique não tem `self`.
    ///
    /// ⚠️ **A mesma fotografia e a mesma fila do [`Self::tick`]**: as poses são tiradas antes do
    /// primeiro gancho, e as escritas aplicadas depois do último.
    pub(crate) fn hear(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        reads: &ReadSnapshot,
        world: &mut World,
        signals: &[&str],
    ) -> SceneReport {
        let mut report = SceneReport::default();
        if signals.is_empty() || self.instances.is_empty() {
            return report;
        }
        let actors = actors(world);
        photograph(reads, world, &actors);
        let mut writes: Vec<EntityWrite> = Vec::new();
        for a in &actors {
            let Some(hook) = self.module_of(&a.cfg).and_then(|m| m.hook("on_signal")) else {
                continue;
            };
            let Some(inst) = self.instances.get_mut(&a.entity.to_bits()) else {
                continue;
            };
            for name in signals {
                if inst.failed.is_some() {
                    break;
                }
                report.calls += 1;
                let ran = speaking(lua, inst.script, Some(a.entity), || {
                    deadline.within(|| hook.call::<()>((inst.this.clone(), *name)))
                });
                settle_call(lua, a.entity, ran, inst, &mut writes, &mut report);
            }
        }
        apply_writes(world, &writes, &mut report);
        crate::motion_api::apply(world, &report.motion);
        report.wrote.sort_by_key(|e| e.to_bits());
        report.wrote.dedup();
        report
    }

    /// ⭐⭐⭐ **UM PASSO FIXO** — nasce quem ainda não nasceu (`init`), actualiza os números que o
    /// artista mexeu, corre `update(self, dt)` e aplica as escritas.
    pub(crate) fn tick(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        reads: &ReadSnapshot,
        world: &mut World,
        dt: f64,
    ) -> SceneReport {
        let mut report = SceneReport::default();
        let actors = actors(world);
        photograph(reads, world, &actors);
        let mut writes: Vec<EntityWrite> = Vec::new();
        for a in &actors {
            let Some(module) = self.module_of(&a.cfg) else {
                continue; // sem script, partido, por importar ou por carregar: o painel já o diz
            };
            let decls = module.decls.clone();
            let init = module.hook("init");
            let update = module.hook("update");
            let bits = a.entity.to_bits();
            let chunk = self.chunk_of(&a.cfg);
            if let Entry::Vacant(slot) = self.instances.entry(bits) {
                let inst = match born(lua, a, bits, chunk.clone()) {
                    Ok(inst) => slot.insert(inst),
                    Err(e) => {
                        let d = ScriptDiagnostic::from_error(&e, &chunk);
                        if let Some(c) = crate::console::of(lua) {
                            c.diagnostic(
                                library_id(&a.cfg).unwrap_or_default(),
                                Some(a.entity),
                                &d,
                                false,
                            );
                        }
                        report.failed.push((a.entity, d));
                        continue;
                    }
                };
                apply_props(lua, inst, &decls, &a.cfg);
                if let Some(init) = init {
                    report.calls += 1;
                    let ran = speaking(lua, inst.script, Some(a.entity), || {
                        deadline.within(|| init.call::<()>(inst.this.clone()))
                    });
                    if !settle_call(lua, a.entity, ran, inst, &mut writes, &mut report) {
                        continue;
                    }
                }
            }
            let inst = self.instances.get_mut(&bits).expect("nasceu acima");
            if inst.failed.is_some() {
                continue;
            }
            apply_props(lua, inst, &decls, &a.cfg);
            if let Some(update) = update {
                report.calls += 1;
                let ran = speaking(lua, inst.script, Some(a.entity), || {
                    deadline.within(|| update.call::<()>((inst.this.clone(), dt)))
                });
                settle_call(lua, a.entity, ran, inst, &mut writes, &mut report);
            }
        }
        apply_writes(world, &writes, &mut report);
        crate::motion_api::apply(world, &report.motion);
        report.wrote.sort_by_key(|e| e.to_bits());
        report.wrote.dedup();
        report
    }

    fn module_of(&self, cfg: &LuauScript) -> Option<&ScriptModule> {
        library_id(cfg)
            .and_then(|id| self.slots.get(&id))
            .and_then(|s| s.module.as_ref())
    }

    fn chunk_of(&self, cfg: &LuauScript) -> String {
        library_id(cfg)
            .and_then(|id| self.slots.get(&id))
            .map(|s| s.chunk.clone())
            .unwrap_or_default()
    }
}

/// O script da biblioteca que um objecto aponta — `None` sem script ou por importar.
fn library_id(cfg: &LuauScript) -> Option<ScriptId> {
    match cfg.source {
        ScriptSource::Library(id) => Some(id),
        ScriptSource::None | ScriptSource::Unimported(_) => None,
    }
}

/// O `self` de um objecto acabado de nascer: só o `id`. Os números entram pelo [`apply_props`].
fn born(lua: &Lua, a: &Actor, bits: u64, chunk: String) -> mlua::Result<Instance> {
    let this = lua.create_table()?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "os bits de uma entidade cabem em 2^53 enquanto a geração for < 2^21 — e \
                  valem só dentro da sessão"
    )]
    this.set("id", bits as f64)?;
    Ok(Instance {
        script: library_id(&a.cfg).unwrap_or_default(),
        chunk,
        this,
        applied: BTreeMap::new(),
        failed: None,
    })
}

/// Escreve no `self` os números que MUDARAM desde a última vez (§3.5): a mão do artista manda,
/// mas só onde mexeu.
fn apply_props(lua: &Lua, inst: &mut Instance, decls: &[PropDecl], cfg: &LuauScript) {
    let r = resolve(Some(decls), &cfg.own);
    for p in &r.values {
        if inst.applied.get(&p.name) == Some(&p.value) {
            continue;
        }
        if let Ok(v) = to_lua(lua, &p.value)
            && inst.this.raw_set(p.name.as_str(), v).is_ok()
        {
            inst.applied.insert(p.name.clone(), p.value.clone());
        }
    }
    // Uma declaração que SAIU do script leva o campo com ela.
    let gone: Vec<String> = inst
        .applied
        .keys()
        .filter(|k| !r.values.iter().any(|p| p.name == **k))
        .cloned()
        .collect();
    for k in gone {
        let _ = inst.this.raw_set(k.as_str(), Value::Nil);
        inst.applied.remove(&k);
    }
}

/// Fecha uma chamada de gancho: drena as filas (as escritas e os sinais são DESTE objecto) e
/// regista a falha. Devolve se o objecto continua a correr.
fn settle_call(
    lua: &Lua,
    entity: Entity,
    ran: mlua::Result<()>,
    inst: &mut Instance,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) -> bool {
    let (mine, emitted) = drain_queues(lua);
    let motion = crate::motion_api::drain(lua);
    let failure = match ran {
        Err(e) => Some(ScriptDiagnostic::from_error(&e, &inst.chunk)),
        Ok(()) => mine
            .iter()
            .find(|w| pose_field(&w.field).is_none())
            .map(|w| {
                ScriptDiagnostic::runtime(format!(
                    "ph2d.set: unknown field '{}' (x, y, rotation, scale_x, scale_y)",
                    w.field
                ))
            })
            .or_else(|| {
                mine.iter().find(|w| !w.value.is_finite()).map(|w| {
                    ScriptDiagnostic::runtime(format!(
                        "ph2d.set: '{}' got {} — not a number",
                        w.field, w.value
                    ))
                })
            }),
    };
    if let Some(d) = failure {
        if let Some(c) = crate::console::of(lua) {
            c.diagnostic(inst.script, Some(entity), &d, false);
        }
        inst.failed = Some(d.clone());
        report.failed.push((entity, d));
        return false;
    }
    writes.extend(mine);
    report.motion.extend(motion);
    report
        .emitted
        .extend(emitted.into_iter().map(|n| (entity, n)));
    true
}

fn drain_queues(lua: &Lua) -> (Vec<EntityWrite>, Vec<String>) {
    let writes = lua
        .app_data_ref::<WriteQueue>()
        .map(|q| q.drain())
        .unwrap_or_default();
    let emits = lua
        .app_data_ref::<EmitQueue>()
        .map(|q| q.drain())
        .unwrap_or_default();
    (writes, emits)
}

pub(crate) fn discard_queues(lua: &Lua) {
    let _ = drain_queues(lua);
    let _ = crate::motion_api::drain(lua);
}

/// Aplica as escritas validadas ao `Transform` — **só quando o valor muda** (o `bevy` marca a
/// alteração no `deref_mut`, e um componente tocado todo o quadro é ruído para o ledger).
fn apply_writes(world: &mut World, writes: &[EntityWrite], report: &mut SceneReport) {
    for w in writes {
        let Some(entity) = Entity::try_from_bits(w.entity) else {
            continue;
        };
        let Some(i) = pose_field(&w.field) else {
            continue; // validado no `settle_call`
        };
        let Some(mut t) = world.get_mut::<Transform>(entity) else {
            continue; // um id que não é de um objecto com pose
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o `Transform` da casa é f32; o número do Luau é f64"
        )]
        let v = w.value as f32;
        let cur = match i {
            0 => t.translation.x,
            1 => t.translation.y,
            2 => t.rotation,
            3 => t.scale.x,
            _ => t.scale.y,
        };
        if cur == v {
            continue;
        }
        match i {
            0 => t.translation.x = v,
            1 => t.translation.y = v,
            2 => t.rotation = v,
            3 => t.scale.x = v,
            _ => t.scale.y = v,
        }
        report.wrote.push(entity);
    }
}

#[path = "scene_reload.rs"]
mod reload;
pub use reload::{ReloadOutcome, ScriptReload};

#[cfg(test)]
#[path = "scene_tests.rs"]
mod tests;
