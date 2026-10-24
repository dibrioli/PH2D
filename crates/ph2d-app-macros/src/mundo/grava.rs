//! **A gravação do mundo** (`docs/Macros/17_W19_plano.md` §2.2–2.3): a sub-árvore do objecto Macro a cada
//! passo do undo, e as corridas da física. Guarda o CRU; quem é dono de cada objecto (o Painter, o Motion
//! ou esta família) só se sabe ao Parar, quando as trilhas das outras famílias estão todas abertas
//! ([`super::fecha`]).

use super::formato::Simula;
use ph2d_ecs::scene::{EntitySnapshotRow, LinhaMudada, WorldSnapshot, linhas_que_mudam};
use std::collections::BTreeMap;
use std::sync::Arc;

/// As linhas de uma sub-árvore, pelo `StableId`.
pub type Linhas = BTreeMap<u64, Arc<EntitySnapshotRow>>;

/// **A sub-árvore de `raiz`** (`StableId`) numa fotografia: ela e os descendentes, pelo `parent` das linhas.
#[must_use]
pub fn sub_arvore(s: &WorldSnapshot, raiz: u64) -> Linhas {
    let pai: BTreeMap<u64, Option<u64>> = s
        .entities
        .iter()
        .map(|r| (r.id.0, r.parent.map(|p| p.0)))
        .collect();
    let mut dentro: BTreeMap<u64, bool> = BTreeMap::new();
    let mut e_dentro = |id: u64| {
        let mut cadeia = Vec::new();
        let mut x = Some(id);
        let r = loop {
            let Some(i) = x else { break false };
            if i == raiz {
                break true;
            }
            if let Some(&d) = dentro.get(&i) {
                break d;
            }
            cadeia.push(i);
            x = pai.get(&i).copied().flatten();
        };
        for i in cadeia {
            dentro.insert(i, r);
        }
        r
    };
    s.entities
        .iter()
        .filter(|r| e_dentro(r.id.0))
        .map(|r| (r.id.0, Arc::clone(r)))
        .collect()
}

fn como_snapshot(l: &Linhas) -> WorldSnapshot {
    WorldSnapshot {
        version: WorldSnapshot::VERSION,
        entities: l.values().cloned().collect(),
    }
}

/// Uma corrida da física a decorrer (ou acabada) — o [`Simula`] antes das ids locais.
#[derive(Clone, Debug, PartialEq)]
pub struct Corrida {
    pub de: u64,
    pub ate: u64,
    pub fita: ph2d_physics_ecs::TapeWire,
    /// `(StableId, [x, y, rotação])` no último tique.
    pub poses: Vec<(u64, [f32; 3])>,
    /// A impressão dos obstáculos ([`super::palco_fisica::impressao`]).
    pub obstaculos: [u8; 32],
}

/// O que se gravou, cru.
#[derive(Clone, Debug, PartialEq)]
pub enum Cru {
    Passo(Vec<LinhaMudada>),
    Corrida(Corrida),
}

/// **A gravação do mundo** de um objecto Macro.
#[derive(Default)]
pub struct Gravacao {
    /// O `StableId` do objecto Macro.
    pub dono: u64,
    /// A sub-árvore ao Gravar (a 1.ª fotografia que chega).
    pub inicio: Option<Linhas>,
    visto: Linhas,
    /// Por ordem de chegada, com o instante do chamador (µs).
    pub passos: Vec<(u64, Cru)>,
    /// A corrida aberta: os quadros em que a ponte anda estendem-na.
    corrida: Option<(u64, Corrida)>,
    /// A ponte andou neste quadro.
    andou: bool,
}

impl Gravacao {
    #[must_use]
    pub fn new(dono: u64) -> Self {
        Self {
            dono,
            ..Self::default()
        }
    }

    /// **O mundo passou de `antes` a `depois`** (um passo do undo, ou um desfazer). A 1.ª chamada fixa o
    /// início em `antes`; as seguintes comparam com o que esta gravação viu por último — um desfazer a
    /// meio também é uma mudança que o artista fez.
    pub fn mudou(&mut self, antes: &WorldSnapshot, depois: &WorldSnapshot, agora_us: u64) {
        if self.inicio.is_none() {
            let i = sub_arvore(antes, self.dono);
            self.visto.clone_from(&i);
            self.inicio = Some(i);
        }
        let novo = sub_arvore(depois, self.dono);
        let d = linhas_que_mudam(&como_snapshot(&self.visto), &como_snapshot(&novo));
        self.visto = novo;
        if !d.is_empty() {
            self.fecha_a_corrida();
            self.passos.push((agora_us, Cru::Passo(d)));
        }
    }

    /// **A ponte correu de `de` a `ate`** neste quadro, com a sub-árvore em `poses` (os corpos dela), a
    /// `fita` do projecto e a impressão dos `obstaculos`. Sem corpos na sub-árvore não é uma corrida
    /// desta macro. `ate == de` = o relógio anda e este quadro de render não teve tique: a corrida
    /// aberta continua (acaba quando o relógio pára, não num quadro sem tique).
    pub fn correu(
        &mut self,
        (de, ate): (u64, u64),
        poses: Vec<(u64, [f32; 3])>,
        fita: &ph2d_physics_ecs::TapeWire,
        obstaculos: [u8; 32],
        agora_us: u64,
    ) {
        if poses.is_empty() {
            return;
        }
        if ate <= de {
            self.andou |= self.corrida.is_some();
            return;
        }
        self.andou = true;
        let mut fita = fita.clone();
        let fim = usize::try_from((ate + 1).saturating_sub(fita.first)).unwrap_or(usize::MAX);
        fita.frames.truncate(fim);
        match &mut self.corrida {
            Some((_, c)) if de == c.ate => {
                (c.ate, c.poses, c.fita, c.obstaculos) = (ate, poses, fita, obstaculos);
            }
            _ => {
                self.fecha_a_corrida();
                let c = Corrida {
                    de,
                    ate,
                    fita,
                    poses,
                    obstaculos,
                };
                self.corrida = Some((agora_us, c));
            }
        }
    }

    /// O fim do quadro: uma corrida que não andou acabou.
    pub fn quadro(&mut self) {
        if !std::mem::take(&mut self.andou) {
            self.fecha_a_corrida();
        }
    }

    fn fecha_a_corrida(&mut self) {
        if let Some((t, c)) = self.corrida.take() {
            self.passos.push((t, Cru::Corrida(c)));
        }
    }

    /// Acaba: a corrida aberta fecha-se.
    #[must_use]
    pub fn acaba(mut self) -> Self {
        self.fecha_a_corrida();
        self
    }
}

impl Corrida {
    /// O [`Simula`] com as ids locais de `local`.
    #[must_use]
    pub fn simula(&self, local: &dyn Fn(u64) -> Option<u32>) -> Simula {
        let de = u32::try_from(self.de).unwrap_or(u32::MAX);
        let ate = u32::try_from(self.ate).unwrap_or(u32::MAX);
        Simula {
            de,
            ate,
            fita: self.fita.clone(),
            conferencia: self
                .poses
                .iter()
                .filter_map(|(s, p)| local(*s).map(|i| (i, *p)))
                .collect(),
            obstaculos: self.obstaculos,
        }
    }
}

#[cfg(test)]
#[path = "grava_tests.rs"]
mod tests;
