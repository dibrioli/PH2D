//! **A conferência das edições de valor do Painter, fora do quadro** (W5e, `docs/Macros/02_plano_W5.md`
//! §7). Medido: conferir tocava a macro inteira DENTRO do quadro — 3,6 s no Splash a 2048², 26,6 s numa
//! macro de 61 s a 1024². Agora é DERIVADA da `Macro`: toda gravação com uma trilha do Painter com
//! edições de valor e ainda não conferida (pelos bytes) confere-se num fio próprio, no 1.º quadro em que
//! o componente já não mudou, a partir do instantâneo do palco em cena ([`Partidas`]). O resultado só se
//! escreve se a `Macro` ainda for a que se conferiu; uma mudança a meio pára o fio. A decisão fica guardada
//! pelos bytes: a mesma `Macro` que volta (um desfazer, um refazer) recebe-a já, sem tocar outra vez — a
//! decisão é função dos bytes, logo a versão normalizada é o mesmo estado.

use crate::{Cena, erros};
use ph2d_ecs::{Entity, Macro};
use ph2d_editor_core::macro_edits::ErroDaMacroView;
use ph2d_macro::{MacroDoc, Track};
use ph2d_tool_painter::macro_track::etapas::{self as pi, Conferida, Partida};
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// O instantâneo de onde a conferência da trilha do objecto `bits` recomeça (o palco em cena; a shell
/// compõe). `None` = do zero.
pub type Partidas<'a> = &'a dyn Fn(u64, &Track) -> Option<Partida>;

type Resultado = Result<Vec<(usize, Conferida)>, ErroDaMacro>;

/// O que se decidiu de uns bytes: as etapas `(trilha, âncora)` que saem da gravação, ou o erro.
type Decisao = Result<Vec<(usize, usize)>, ErroDaMacroView>;

enum Fio {
    Corre(JoinHandle<Resultado>),
    /// O fio não arrancou: conferiu-se aqui, do zero.
    Pronto(Resultado),
}

struct EmCurso {
    bits: u64,
    /// Os bytes da `Macro` que se conferem.
    hash: u64,
    parar: Arc<AtomicBool>,
    fio: Option<Fio>,
}

impl Drop for EmCurso {
    fn drop(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
    }
}

#[derive(Default)]
pub(crate) struct Conferencias {
    /// O tique de mudança da `Macro` visto, por objecto.
    vistos: BTreeMap<u64, u32>,
    /// Os objectos com uma conferência por fazer.
    por_conferir: BTreeSet<u64>,
    /// O que se decidiu, pelo hash do `Macro.doc`.
    decididas: BTreeMap<u64, Decisao>,
    em_curso: Option<EmCurso>,
}

fn hash(b: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

/// As trilhas do Painter com edições de VALOR (as que podem mudar o que a água gravada responde).
fn trilhas_a_conferir(doc: &MacroDoc) -> Vec<usize> {
    doc.tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            t.family == ph2d_tool_painter::macro_track::FAMILIA
                && t.edicoes
                    .iter()
                    .any(|e| !e.apagada && (!e.insercoes.is_empty() || !e.trocas.is_empty()))
        })
        .map(|(i, _)| i)
        .collect()
}

fn trabalho(trilhas: Vec<(usize, Track, Option<Partida>)>, parar: &AtomicBool) -> Resultado {
    trilhas
        .into_iter()
        .map(|(i, t, p)| pi::confere(&t, p, parar).map(|r| (i, r)))
        .collect()
}

impl Conferencias {
    /// Há uma conferência por fazer ou a correr (o undo espera por ela: a edição e a decisão dela são
    /// UM passo).
    pub(crate) fn a_conferir(&self) -> bool {
        self.em_curso.is_some() || !self.por_conferir.is_empty()
    }

    /// Os objectos (bits) com uma conferência por fazer ou a correr.
    pub(crate) fn a_conferir_em(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.por_conferir.iter().copied().collect();
        v.extend(self.em_curso.as_ref().map(|x| x.bits));
        v
    }

    /// **Por quadro**, depois dos pedidos: vê que `Macro`s mudaram, colhe a conferência que acabou e
    /// arranca a seguinte. Devolve o erro de uma conferência, para o painel o dizer.
    pub(crate) fn quadro(&mut self, c: &mut Cena<'_>) -> Option<(u64, ErroDaMacroView)> {
        let world = c.sim.world_mut();
        let es: Vec<Entity> = world
            .query::<(Entity, &Macro)>()
            .iter(world)
            .map(|(e, _)| e)
            .collect();
        let objs: Vec<(u64, u32)> = es
            .iter()
            .map(|&e| {
                let tique = world
                    .entity(e)
                    .get_change_ticks::<Macro>()
                    .map_or(0, |t| t.changed.get());
                (e.to_bits(), tique)
            })
            .collect();
        // Uma mudança depois desta leitura leva um tique NOVO (a shell não corre um `Schedule`).
        world.increment_change_tick();
        let vivo = |b: &u64| objs.iter().any(|(o, _)| o == b);
        self.vistos.retain(|b, _| vivo(b));
        self.por_conferir.retain(vivo);
        if self.em_curso.as_ref().is_some_and(|x| !vivo(&x.bits)) {
            self.em_curso = None;
        }
        let mut mudaram = BTreeSet::new();
        let mut erro = None;
        for &(bits, tique) in &objs {
            if self.vistos.insert(bits, tique) == Some(tique) {
                continue;
            }
            mudaram.insert(bits);
            self.por_conferir.remove(&bits);
            if self.em_curso.as_ref().is_some_and(|x| x.bits == bits) {
                self.em_curso = None;
            }
            let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
                continue;
            };
            let h = hash(&m.doc);
            match self.decididas.get(&h).cloned() {
                Some(Ok(saidas)) => self.aplica(c, bits, &saidas),
                Some(Err(e)) => erro = Some((bits, e)),
                None if ph2d_macro::codec::read(&m.doc)
                    .is_ok_and(|d| !trilhas_a_conferir(&d).is_empty()) =>
                {
                    self.por_conferir.insert(bits);
                }
                None => {}
            }
        }
        let erro = self.colhe(c).or(erro);
        if self.em_curso.is_none()
            && let Some(bits) = self
                .por_conferir
                .iter()
                .copied()
                .find(|b| !mudaram.contains(b))
        {
            self.por_conferir.remove(&bits);
            self.arranca(c, bits);
        }
        erro
    }

    fn arranca(&mut self, c: &Cena<'_>, bits: u64) {
        let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
            return;
        };
        let Ok(doc) = ph2d_macro::codec::read(&m.doc) else {
            return;
        };
        let h = hash(&m.doc);
        let trilhas: Vec<_> = trilhas_a_conferir(&doc)
            .into_iter()
            .map(|i| {
                let t = doc.tracks[i].clone();
                let p = c.partidas.and_then(|f| f(bits, &t));
                (i, t, p)
            })
            .collect();
        let parar = Arc::new(AtomicBool::new(false));
        let no_fio = Arc::clone(&parar);
        let fio = std::thread::Builder::new()
            .name("macro-confere".into())
            .spawn(move || trabalho(trilhas, &no_fio));
        let fio = match fio {
            Ok(f) => Fio::Corre(f),
            // O instantâneo foi com a closure que não arrancou: do zero.
            Err(_) => Fio::Pronto(trabalho(
                trilhas_a_conferir(&doc)
                    .into_iter()
                    .map(|i| (i, doc.tracks[i].clone(), None))
                    .collect(),
                &parar,
            )),
        };
        self.em_curso = Some(EmCurso {
            bits,
            hash: h,
            parar,
            fio: Some(fio),
        });
    }

    fn colhe(&mut self, c: &mut Cena<'_>) -> Option<(u64, ErroDaMacroView)> {
        if !self.em_curso.as_ref().is_some_and(|x| match &x.fio {
            Some(Fio::Corre(f)) => f.is_finished(),
            _ => true,
        }) {
            return None;
        }
        let mut x = self.em_curso.take()?;
        let r = match x.fio.take()? {
            Fio::Corre(f) => f.join(),
            Fio::Pronto(r) => Ok(r),
        };
        let bits = x.bits;
        // A `Macro` é a conferida: uma mudança dela pára o fio na leitura dos tiques (`quadro`).
        let decisao: Decisao = match r {
            Ok(Ok(rs)) if rs.iter().any(|&(_, r)| r == Conferida::Parada) => return None,
            Ok(Ok(rs)) => Ok(rs
                .iter()
                .filter_map(|&(i, r)| match r {
                    Conferida::SaiEm(a) => Some((i, a)),
                    _ => None,
                })
                .collect()),
            Ok(Err(e)) => Err(erros::do_painter(&e)),
            Err(_) => Ok(Vec::new()),
        };
        self.decididas.insert(x.hash, decisao.clone());
        match decisao {
            Ok(saidas) => {
                self.aplica(c, bits, &saidas);
                None
            }
            Err(e) => Some((bits, e)),
        }
    }

    /// Tira da gravação as etapas `saidas` da `Macro` de `bits` (os bytes novos ficam decididos).
    fn aplica(&mut self, c: &mut Cena<'_>, bits: u64, saidas: &[(usize, usize)]) {
        if saidas.is_empty() {
            return;
        }
        let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
            return;
        };
        let Ok(mut doc) = ph2d_macro::codec::read(&m.doc) else {
            return;
        };
        for &(i, a) in saidas {
            if let Some(t) = doc.tracks.get_mut(i) {
                t.edicao_mut(a).sai_da_gravacao = true;
            }
        }
        let bytes = ph2d_macro::codec::write(&doc);
        self.decididas.insert(hash(&bytes), Ok(Vec::new()));
        if let Some(mut m) = c.sim.world_mut().get_mut::<Macro>(Entity::from_bits(bits))
            && m.doc != bytes
        {
            m.doc = bytes;
        }
    }
}
