//! **O depurador dos scripts** — a porta segura para os ganchos de depuração do Luau (plano 13,
//! Onda D: `docs/CodeEditor/13_plano_bancada.md`, decisão 11 do dono).
//!
//! - **Ponto de parada**: `lua_breakpoint` no TOPO do módulo (desce às funções filhas) reescreve a
//!   1.ª instrução da linha → custo zero nas outras (medido). Ao atingir, o gancho CEDE a corrotina
//!   do gancho da cena (o Luau retoma-a «yielded inside a hook»); a cena estaciona-a até Continue.
//! - **Next line**: o passo-a-passo SÓ nesse fio até a linha mudar à mesma profundidade ou acima.
//! - **Valores ao vivo**: o passo-a-passo SÓ nas corrotinas de UM objecto do script da aba aberta.
//! - **Pause on Error**: a pilha de uma corrotina que errou fica de pé; lê-se antes de a largar.
//!
//! A `interrupt` é da casa (o prazo de cada chamada, `ph2d-script`); estes são os outros campos de
//! `lua_Callbacks`. Medição do custo: `tests/sonda_gancho_de_linha.rs`.

mod ffi;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use mlua::{Function, Lua, Thread};

pub use ffi::{RawHook, breakpoint, set_hooks, single_step};
pub use mlua::ffi::{lua_Debug, lua_State};

/// Porque um objecto parou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopKind {
    /// Um ponto de parada.
    Breakpoint,
    /// O fim de um *Next line*.
    Step,
    /// Um erro (Pause on Error): a mensagem.
    Error(String),
}

/// Uma variável no sítio onde parou, já em texto (`412`, `"Bob"`, `table`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Var {
    pub name: String,
    pub value: String,
}

/// Onde parou e com que valores (a função Luau do topo da pilha).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StopFrame {
    /// A linha (1-based) no script.
    pub line: u32,
    /// O nome da função (`update`), vazio no topo do módulo.
    pub function: String,
    /// As variáveis locais visíveis, pela ordem do script.
    pub locals: Vec<Var>,
    /// Os campos do `self` (um nível), por nome.
    pub this: Vec<Var>,
    /// A pilha de chamadas, do topo para baixo: `(função, linha)`.
    pub stack: Vec<(String, u32)>,
}

/// O que uma linha fez no último quadro (do objecto vigiado).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineLive {
    /// Quantas vezes correu numa só corrida (o laço; nunca os passos fixos do quadro).
    pub hits: u32,
    /// As locais que esta linha mudou ou criou (o último valor).
    pub values: Vec<Var>,
    /// Quantos `print` saíram dela.
    pub prints: u32,
}

/// Um passo em curso (*Next line*).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Step {
    pub(crate) line: i32,
    pub(crate) depth: i32,
}

/// Um quadro da pilha do vigiado: a linha corrente e as locais quando ela começou.
pub(crate) struct LiveFrame {
    pub(crate) depth: i32,
    pub(crate) line: i32,
    pub(crate) snapshot: Vec<Var>,
}

/// O estado partilhado com os ganchos C (um por VM).
#[derive(Default)]
pub struct State {
    /// O fio de gancho da cena que corre agora — só ELE pode ceder.
    pub(crate) current: usize,
    /// O fio que o gancho acabou de ceder para uma paragem.
    pub(crate) stop_pending: Option<(usize, StopKind)>,
    /// Retomado numa paragem: o 1.º toque na mesma linha (o da própria paragem) não pára; gasta-se
    /// nele ou quando a linha muda.
    pub(crate) suppress: BTreeMap<usize, i32>,
    /// Os *Next line* em curso, por fio.
    pub(crate) step: BTreeMap<usize, Step>,
    /// Dentro de um `exit` (sem fila a quem voltar): os ganchos não param.
    pub(crate) quiet: u32,
    /// O fio vigiado (valores ao vivo) e o nome do chunk dele (`=nome`).
    pub(crate) watched: usize,
    pub(crate) watched_chunk: String,
    pub(crate) frames: Vec<LiveFrame>,
    pub(crate) live: BTreeMap<u32, LineLive>,
    /// As vezes que cada linha correu na corrida em curso (um `update`, um `on_signal`).
    pub(crate) ran: BTreeMap<u32, u32>,
    pub(crate) shown: BTreeMap<u32, LineLive>,
    /// Paragens que não puderam ceder (dentro de uma função C ou de uma corrotina do script).
    pub(crate) blocked: u32,
    pub(crate) pause_on_error: bool,
    /// O sítio de cada fio parado, fotografado PELO GANCHO: depois de ceder, o `savedpc` fica na
    /// instrução da paragem e o Luau desconta uma — de fora leria a linha de cima e não veria a
    /// local criada nela (medido, `scene_debug_tests.rs`).
    pub(crate) stopped: BTreeMap<usize, StopFrame>,
    pub(crate) target: Option<(u64, u64)>,
    pub(crate) error_frame: Option<StopFrame>,
}

/// ⭐ **O depurador de UMA VM.** Instala os ganchos; o estado sai quando a última cópia cai.
#[derive(Clone)]
pub struct Debugger(Rc<Inner>);

pub struct Inner {
    key: usize,
    state: Rc<RefCell<State>>,
}

impl std::ops::Deref for Debugger {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl Debugger {
    /// Instala `debugbreak` e `debugstep` na VM.
    ///
    /// # Errors
    /// O da VM.
    pub fn install(lua: &Lua) -> mlua::Result<Self> {
        let key = ffi::vm_key(lua)?;
        let state = Rc::new(RefCell::new(State::default()));
        ffi::register(key, &state);
        set_hooks(lua, Some(ffi::on_break), Some(ffi::on_step))?;
        Ok(Self(Rc::new(Inner { key, state })))
    }

    /// Arma os pontos de parada `lines` no módulo cujo topo é `main` e desarma `previous`.
    /// Devolve as linhas onde ficaram (a próxima com instruções), sem repetidas.
    ///
    /// # Errors
    /// O da VM.
    pub fn arm(
        &self,
        lua: &Lua,
        main: &Function,
        lines: &[u32],
        previous: &[u32],
    ) -> mlua::Result<Vec<u32>> {
        for l in previous {
            breakpoint(lua, main, line_i32(*l), false)?;
        }
        let mut got = Vec::new();
        for l in lines {
            if let Some(at) = breakpoint(lua, main, line_i32(*l), true)? {
                let at = u32::try_from(at).unwrap_or(0);
                if !got.contains(&at) {
                    got.push(at);
                }
            }
        }
        got.sort_unstable();
        Ok(got)
    }

    /// Corre `f` com `th` como o fio de gancho da cena (o único que um gancho pode ceder).
    pub fn running<R>(&self, th: &Thread, f: impl FnOnce() -> R) -> R {
        let prev = std::mem::replace(&mut self.state.borrow_mut().current, ptr(th));
        let _back = Restore(&self.state, move |s: &mut State| s.current = prev);
        f()
    }

    /// Corre `f` sem paragens: um ponto ou um passo atingido nele segue (o `exit`, que não tem
    /// fila a quem voltar).
    pub fn quiet<R>(&self, f: impl FnOnce() -> R) -> R {
        self.state.borrow_mut().quiet += 1;
        let _back = Restore(&self.state, |s: &mut State| s.quiet -= 1);
        f()
    }

    /// `th` acabou (ou foi largado): o estado dele sai — o endereço pode voltar noutro fio.
    pub fn forget(&self, th: &Thread) {
        let me = ptr(th);
        let mut s = self.state.borrow_mut();
        s.step.remove(&me);
        s.suppress.remove(&me);
        s.stopped.remove(&me);
        if s.stop_pending.as_ref().is_some_and(|(t, _)| *t == me) {
            s.stop_pending = None;
        }
        if s.watched == me {
            s.watched = 0;
            s.frames.clear();
        }
    }

    /// Todos os fios saem (rebobinar, outro projecto).
    pub fn forget_all(&self) {
        let mut s = self.state.borrow_mut();
        s.step.clear();
        s.suppress.clear();
        s.stopped.clear();
        s.stop_pending = None;
        s.watched = 0;
        s.frames.clear();
        s.ran.clear();
        s.live.clear();
        s.shown.clear();
    }

    /// `th` acabou de ceder para uma paragem? Consome-a.
    #[must_use]
    pub fn took_stop(&self, th: &Thread) -> Option<StopKind> {
        let mut s = self.state.borrow_mut();
        match s.stop_pending.take() {
            Some((t, kind)) if t == ptr(th) => Some(kind),
            other => {
                s.stop_pending = other;
                None
            }
        }
    }

    /// Antes de retomar `th` parado: não volta a parar na mesma linha; `step` = *Next line*.
    ///
    /// # Errors
    /// O da VM.
    pub fn before_resume(&self, lua: &Lua, th: &Thread, step: bool) -> mlua::Result<()> {
        let (seen, depth) = ffi::where_is(lua, th)?;
        {
            let mut s = self.state.borrow_mut();
            let line = s
                .stopped
                .remove(&ptr(th))
                .map_or(seen, |f| i32::try_from(f.line).unwrap_or(seen));
            s.suppress.insert(ptr(th), line);
            if step {
                s.step.insert(ptr(th), Step { line, depth });
            } else {
                s.step.remove(&ptr(th));
            }
        }
        ffi::thread_single_step(lua, th, true)
    }

    /// Onde `th` (parado ou morto por um erro) está, e os valores.
    ///
    /// # Errors
    /// O da VM.
    pub fn frame(&self, lua: &Lua, th: &Thread) -> mlua::Result<Option<StopFrame>> {
        if let Some(f) = self.state.borrow().stopped.get(&ptr(th)) {
            return Ok(Some(f.clone()));
        }
        ffi::frame_of(lua, th)
    }

    /// Vigia `th` (um gancho do objecto vigiado, script `chunk` = o nome com que foi carregado).
    ///
    /// # Errors
    /// O da VM.
    pub fn watch(&self, lua: &Lua, th: &Thread, chunk: &str) -> mlua::Result<()> {
        {
            let mut s = self.state.borrow_mut();
            s.watched = ptr(th);
            // O `source` de um módulo carregado com `set_name("=nome")`.
            let chunk = format!("={chunk}");
            if s.watched_chunk != chunk {
                s.watched_chunk = chunk;
                s.shown.clear();
                s.live.clear();
                s.ran.clear();
            }
            s.frames.clear();
        }
        ffi::thread_single_step(lua, th, true)
    }

    /// Ninguém vigiado (a aba fechou, outro script, o Play parou).
    pub fn unwatch(&self) {
        let mut s = self.state.borrow_mut();
        s.watched = 0;
        s.watched_chunk.clear();
        s.frames.clear();
        s.live.clear();
        s.ran.clear();
        s.shown.clear();
    }

    /// O texto do vigiado mudou (recarregou): os valores por linha são de outro texto.
    pub fn forget_lines(&self) {
        let mut s = self.state.borrow_mut();
        s.ran.clear();
        s.live.clear();
        s.shown.clear();
    }

    /// Um `print` saiu da linha `line` do vigiado.
    pub fn printed(&self, line: u32) {
        self.state.borrow_mut().live.entry(line).or_default().prints += 1;
    }

    /// O fio vigiado é o que corre agora?
    #[must_use]
    pub fn watching_now(&self) -> bool {
        let s = self.state.borrow();
        s.watched != 0 && s.watched == s.current
    }

    /// Fecha uma corrida (um passo fixo, os sinais, o `process`): `hits` guarda o máximo de uma só.
    pub fn end_run(&self) {
        let mut s = self.state.borrow_mut();
        for (line, n) in std::mem::take(&mut s.ran) {
            let l = s.live.entry(line).or_default();
            l.hits = l.hits.max(n);
        }
    }

    /// Fecha o quadro: cada linha que o vigiado correu nele troca o mostrado DELA; as outras guardam o
    /// último valor (com dois passos fixos ou nenhum num quadro, uma linha não pisca).
    pub fn end_frame(&self) {
        self.end_run();
        let mut s = self.state.borrow_mut();
        for (line, mut l) in std::mem::take(&mut s.live) {
            if l.values.is_empty()
                && let Some(old) = s.shown.get_mut(&line)
            {
                l.values = std::mem::take(&mut old.values);
            }
            s.shown.insert(line, l);
        }
    }

    /// O que cada linha do vigiado fez no último quadro em que correu.
    #[must_use]
    pub fn live(&self) -> BTreeMap<u32, LineLive> {
        self.state.borrow().shown.clone()
    }

    /// O objecto vigiado: `(script, bits da entidade)` — os ganchos dele levam o passo-a-passo.
    pub fn set_target(&self, target: Option<(u64, u64)>) {
        self.state.borrow_mut().target = target;
    }

    /// `(script, entidade)` é o vigiado?
    #[must_use]
    pub fn is_target(&self, script: u64, entity: u64) -> bool {
        self.state.borrow().target == Some((script, entity))
    }

    /// Pause on Error ligado?
    #[must_use]
    pub fn pause_on_error(&self) -> bool {
        self.state.borrow().pause_on_error
    }

    pub fn set_pause_on_error(&self, on: bool) {
        self.state.borrow_mut().pause_on_error = on;
    }

    /// Guarda o sítio de um erro (lido antes de a corrotina cair) para quem regista a falha.
    pub fn stash_error_frame(&self, frame: StopFrame) {
        self.state.borrow_mut().error_frame = Some(frame);
    }

    /// O sítio do último erro guardado.
    #[must_use]
    pub fn take_error_frame(&self) -> Option<StopFrame> {
        self.state.borrow_mut().error_frame.take()
    }

    /// Quantas paragens não puderam ceder desde a última leitura.
    pub fn take_blocked(&self) -> u32 {
        std::mem::take(&mut self.state.borrow_mut().blocked)
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        ffi::unregister(self.key);
    }
}

/// Repõe um campo do estado ao sair do âmbito — também num pânico.
struct Restore<'a, F: FnMut(&mut State)>(&'a RefCell<State>, F);

impl<F: FnMut(&mut State)> Drop for Restore<'_, F> {
    fn drop(&mut self) {
        if let Ok(mut s) = self.0.try_borrow_mut() {
            (self.1)(&mut s);
        }
    }
}

fn ptr(th: &Thread) -> usize {
    th.to_pointer() as usize
}

fn line_i32(l: u32) -> i32 {
    i32::try_from(l).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn depurador_running_e_quiet_repoem_o_estado_num_panico() {
        let lua = Lua::new();
        let d = Debugger::install(&lua).expect("ganchos");
        let f = lua.create_function(|_, ()| Ok(())).expect("função");
        let th = lua.create_thread(f).expect("fio");
        d.watch(&lua, &th, "t").expect("vigia");
        let r = catch_unwind(AssertUnwindSafe(|| d.running(&th, || panic!("dentro"))));
        assert!(r.is_err());
        assert!(!d.watching_now(), "o fio corrente voltou ao de antes");
        let r = catch_unwind(AssertUnwindSafe(|| d.quiet(|| panic!("dentro"))));
        assert!(r.is_err());
        assert_eq!(d.state.borrow().quiet, 0, "as paragens voltaram");
        d.forget(&th);
        assert_eq!(d.state.borrow().watched, 0, "esquecido");
    }
}
