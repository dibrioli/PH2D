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
//! - ⚠️ **Recarregar NÃO renasce.** Um ficheiro que muda troca as funções e mantém cada `self` (o
//!   *reset+restore* do HR-16): um objecto a meio de um salto continua o salto com a lei nova. E um
//!   script que tinha partido volta a correr — a correcção é a razão de o artista ter gravado.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::Path;
use std::time::SystemTime;

use mlua::{Lua, Table, Value};
use ph2d_ecs::{Entity, StableId, Transform, World};

use crate::component::LuauScript;
use crate::io::{EntityWrite, ReadSnapshot, WriteQueue, pose_field};
use crate::module::{ModuleError, ScriptModule, load_module, runtime_message};
use crate::props::{PropDecl, ScriptValue, resolve};

/// **O que se sabe de um ficheiro de script** — o que o painel mostra.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptInfo {
    /// Carregou; estas são as declarações dele.
    Ready(Vec<PropDecl>),
    /// O Luau recusou-o — a mensagem diz porquê.
    Broken(String),
    /// O ficheiro não se deixa ler (sumiu, sem permissão).
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

/// O carimbo barato de um ficheiro — mudar de tamanho ou de data manda reler.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
}

/// Um ficheiro de script, carregado ou não.
struct Slot {
    stamp: Option<Stamp>,
    hash: Option<[u8; 32]>,
    module: Option<ScriptModule>,
    info: ScriptInfo,
}

/// O vivo de UM objecto.
struct Instance {
    /// O ficheiro que ele corre — mudar de ficheiro é renascer.
    path: String,
    /// A tabela `self`.
    this: Table,
    /// Os valores que já foram escritos no `self` — só uma MUDANÇA volta a escrever (§3.5).
    applied: BTreeMap<String, ScriptValue>,
    /// Porque parou de correr, se parou.
    failed: Option<String>,
}

/// **O que um tique fez** — os factos que a ponte declara.
#[derive(Default, Debug)]
pub struct SceneReport {
    /// Objectos cujo `Transform` o script mudou neste tique.
    pub wrote: Vec<Entity>,
    /// Sinais emitidos, com quem os emitiu, pela ordem da identidade.
    pub emitted: Vec<(Entity, String)>,
    /// Objectos que pararam de correr NESTE tique, com a mensagem.
    pub failed: Vec<(Entity, String)>,
    /// Quantos ganchos correram.
    pub calls: usize,
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
    slots: BTreeMap<String, Slot>,
    instances: BTreeMap<u64, Instance>,
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
    /// velha. Esquece tudo; o próximo `sync` recarrega do disco.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.instances.clear();
    }

    /// ⭐ **REBOBINAR É RENASCER** — o `init` volta a correr no próximo tique.
    pub fn rewind(&mut self) -> usize {
        let n = self.instances.len();
        self.instances.clear();
        n
    }

    /// O que se sabe do ficheiro `path` — `None` se nenhum objecto o nomeou ainda.
    #[must_use]
    pub fn info(&self, path: &str) -> Option<&ScriptInfo> {
        self.slots.get(path).map(|s| &s.info)
    }

    /// Porque o objecto parou de correr o script, se parou.
    #[must_use]
    pub fn failure(&self, entity: Entity) -> Option<&str> {
        self.instances
            .get(&entity.to_bits())
            .and_then(|i| i.failed.as_deref())
    }

    /// Quantos objectos têm o vivo nascido — o número que um gate lê.
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.instances.len()
    }

    /// ⭐ **Uma vez por QUADRO, a correr ou não**: recarrega o que mudou no disco e esquece o vivo
    /// de quem já não tem script (ou trocou de ficheiro).
    ///
    /// ⚠️ **Parado também**, e é o que faz o painel ver as declarações no instante em que o artista
    /// escolhe o ficheiro — sem isto a linha de cada número só apareceria depois do Play.
    pub(crate) fn sync(&mut self, lua: &Lua, deadline: &Deadline, world: &mut World) {
        let actors = actors(world);
        let wanted: BTreeMap<&str, ()> = actors
            .iter()
            .filter(|a| !a.cfg.source.trim().is_empty())
            .map(|a| (a.cfg.source.as_str(), ()))
            .collect();
        // Quem ninguém nomeia sai — um ficheiro esquecido não é relido para sempre.
        self.slots.retain(|p, _| wanted.contains_key(p.as_str()));
        for path in wanted.keys() {
            self.refresh(lua, deadline, path);
        }
        let now: BTreeMap<u64, &str> = actors
            .iter()
            .map(|a| (a.entity.to_bits(), a.cfg.source.as_str()))
            .collect();
        self.instances
            .retain(|bits, inst| now.get(bits).is_some_and(|p| *p == inst.path));
    }

    /// Relê `path` se o carimbo mudou, e recarrega se o CONTEÚDO mudou (HR-16).
    fn refresh(&mut self, lua: &Lua, deadline: &Deadline, path: &str) {
        let meta = std::fs::metadata(Path::new(path));
        let stamp = meta.as_ref().ok().map(|m| Stamp {
            len: m.len(),
            modified: m.modified().ok(),
        });
        let slot = self.slots.entry(path.to_owned()).or_insert_with(|| Slot {
            stamp: None,
            hash: None,
            module: None,
            info: ScriptInfo::Missing(String::new()),
        });
        if stamp.is_some() && slot.stamp == stamp {
            return;
        }
        slot.stamp = stamp;
        let source = match meta.and_then(|_| std::fs::read_to_string(path)) {
            Ok(s) => s,
            Err(e) => {
                slot.hash = None;
                slot.module = None;
                slot.info = ScriptInfo::Missing(e.to_string());
                return;
            }
        };
        let hash = *blake3::hash(source.as_bytes()).as_bytes();
        if slot.hash == Some(hash) {
            return; // tocado, não mudado
        }
        slot.hash = Some(hash);
        let name = Path::new(path)
            .file_name()
            .map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned());
        // ⚠️ O TOPO também corre sob o prazo: um laço sem saída fora de qualquer função congelaria
        // o app no instante em que o artista escolhe o ficheiro.
        let loaded = deadline.within(|| load_module(lua, &name, &source));
        // ⚠️ O topo de um script pode ter escrito ou emitido — isso não é de objecto nenhum.
        discard_queues(lua);
        match loaded {
            Ok(m) => {
                slot.info = ScriptInfo::Ready(m.decls.clone());
                slot.module = Some(m);
                // Recarregar não renasce — mas um script partido volta a tentar.
                for inst in self.instances.values_mut().filter(|i| i.path == path) {
                    inst.failed = None;
                }
            }
            Err(e) => {
                slot.info = ScriptInfo::Broken(match &e {
                    ModuleError::Syntax(m) => format!("syntax error: {m}"),
                    other => other.to_string(),
                });
                slot.module = None;
            }
        }
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
            let Some(hook) = self
                .module_of(&a.cfg.source)
                .and_then(|m| m.hook("on_signal"))
            else {
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
                let ran = deadline.within(|| hook.call::<()>((inst.this.clone(), *name)));
                settle_call(lua, a.entity, ran, inst, &mut writes, &mut report);
            }
        }
        apply_writes(world, &writes, &mut report);
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
            let Some(module) = self.module_of(&a.cfg.source) else {
                continue; // sem ficheiro, partido ou por carregar: o painel já o diz
            };
            let decls = module.decls.clone();
            let init = module.hook("init");
            let update = module.hook("update");
            let bits = a.entity.to_bits();
            if let Entry::Vacant(slot) = self.instances.entry(bits) {
                let inst = match born(lua, a, bits) {
                    Ok(inst) => slot.insert(inst),
                    Err(e) => {
                        report.failed.push((a.entity, runtime_message(&e)));
                        continue;
                    }
                };
                apply_props(lua, inst, &decls, &a.cfg);
                if let Some(init) = init {
                    report.calls += 1;
                    let ran = deadline.within(|| init.call::<()>(inst.this.clone()));
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
                let ran = deadline.within(|| update.call::<()>((inst.this.clone(), dt)));
                settle_call(lua, a.entity, ran, inst, &mut writes, &mut report);
            }
        }
        apply_writes(world, &writes, &mut report);
        report.wrote.sort_by_key(|e| e.to_bits());
        report.wrote.dedup();
        report
    }

    fn module_of(&self, path: &str) -> Option<&ScriptModule> {
        self.slots.get(path).and_then(|s| s.module.as_ref())
    }
}

/// O `self` de um objecto acabado de nascer: só o `id`. Os números entram pelo [`apply_props`].
fn born(lua: &Lua, a: &Actor, bits: u64) -> mlua::Result<Instance> {
    let this = lua.create_table()?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "os bits de uma entidade cabem em 2^53 enquanto a geração for < 2^21 — e \
                  valem só dentro da sessão"
    )]
    this.set("id", bits as f64)?;
    Ok(Instance {
        path: a.cfg.source.clone(),
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
    let failure = match ran {
        Err(e) => Some(runtime_message(&e)),
        Ok(()) => mine
            .iter()
            .find(|w| pose_field(&w.field).is_none())
            .map(|w| {
                format!(
                    "ph2d.set: unknown field '{}' (x, y, rotation, scale_x, scale_y)",
                    w.field
                )
            })
            .or_else(|| {
                mine.iter()
                    .find(|w| !w.value.is_finite())
                    .map(|w| format!("ph2d.set: '{}' got {} — not a number", w.field, w.value))
            }),
    };
    if let Some(msg) = failure {
        inst.failed = Some(msg.clone());
        report.failed.push((entity, msg));
        return false;
    }
    writes.extend(mine);
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

fn discard_queues(lua: &Lua) {
    let _ = drain_queues(lua);
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

#[cfg(test)]
#[path = "scene_tests.rs"]
mod tests;
