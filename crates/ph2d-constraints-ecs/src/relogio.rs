//! Os tipos que leem o TEMPO (W3, `docs/BlenderConstraints/03_W3_poderosas.md` §2): o Action toca um
//! clip do dono no instante que o alvo pede; a Spring segue a pose animada com atraso de mola. A
//! timeline depende desta crate: quem amostra os clips e a pose animada é a família, por estes recursos.

use std::collections::BTreeMap;
use std::sync::Arc;

use bevy_ecs::prelude::Resource;
use ph2d_constraints::{Spring, wrap_pi};
use ph2d_node_motion_spring::law::{MODE_TIME, passo, physics_of};

/// Os canais LOCAIS (x, y, rot, sx, sy) que um clip anima no instante `t`; `None` = não o anima.
pub type Amostra = Arc<dyn Fn(f64) -> [Option<f64>; 5] + Send + Sync>;

/// O clip de cada Action, por `(Entity::to_bits do dono, Action::clip)` — instalado pela família
/// antes da fase.
#[derive(Resource, Default)]
pub struct Accoes(pub BTreeMap<(u64, u32), Amostra>);

/// Um checkpoint a cada `STRIDE` quadros — o da física (`ph2d-physics/src/world/checkpoint.rs`).
pub const STRIDE: i64 = 10;

/// O estado de um quadro — opaco fora desta crate (só as [`Dinamica`] daqui o constroem).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passo<const N: usize> {
    pub(crate) valor: [f32; N],
    pub(crate) vel: [f32; N],
    /// A entrada com que este quadro se calculou — um checkpoint cuja entrada mudou (a animação foi
    /// editada) já não vale.
    pub(crate) entrada: [f64; N],
}

/// A lei de uma mola de `N` canais: os knobs do nó `motion.spring` (modo Time) e quais canais são
/// ÂNGULOS (perseguem pelo arco curto). A pose da Spring é `N = 3` com a rotação no índice 2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lei<const N: usize> {
    pub duracao: f64,
    pub ressalto: f64,
    pub angular: [bool; N],
}

/// ⭐ **A regra de UM passo** de um [`Simulador`]: a mola ([`Lei`]) ou o Atrair
/// ([`crate::comportamento::Atracao`]). O simulador é o mesmo — checkpoints, scrub, laço.
pub trait Dinamica<const N: usize>: Copy {
    /// Os knobs: mudados, a história deixa de valer.
    fn chave(&self) -> [u64; 3];
    /// Os canais que são ângulos (perseguem e misturam pelo arco curto).
    fn angular(&self) -> [bool; N];
    /// O estado do quadro 0 com a entrada `e`.
    fn repouso(&self, e: [f64; N]) -> Passo<N>;
    /// Um quadro de `dt` segundos com a entrada `e`.
    fn passo(&self, s: Passo<N>, e: [f64; N], dt: f32) -> Passo<N>;
    /// O que o simulador devolve de um estado.
    fn saida(&self, s: Passo<N>) -> [f64; N];
}

impl<const N: usize> Dinamica<N> for Lei<N> {
    fn chave(&self) -> [u64; 3] {
        [self.duracao.to_bits(), self.ressalto.to_bits(), 0]
    }

    fn angular(&self) -> [bool; N] {
        self.angular
    }

    fn repouso(&self, e: [f64; N]) -> Passo<N> {
        repouso(e)
    }

    #[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
    fn passo(&self, s: Passo<N>, e: [f64; N], dt: f32) -> Passo<N> {
        let (d, b) = (self.duracao as f32, self.ressalto as f32);
        let (tension, friction) = physics_of(MODE_TIME, 0.0, 0.0, d, b);
        um_passo(s, e, self.angular, (tension, friction, dt))
    }

    fn saida(&self, s: Passo<N>) -> [f64; N] {
        desvio(s, self.angular)
    }
}

impl From<&Spring> for Lei<3> {
    fn from(m: &Spring) -> Self {
        Self {
            duracao: m.duracao,
            ressalto: m.ressalto,
            angular: [false, false, true],
        }
    }
}

/// ⭐ **A mola de UM item: o estado é função do QUADRO.** Tocar para a frente dá um passo; um salto
/// (scrub, rebobinar) volta ao checkpoint válido ≤ quadro e re-simula com a entrada de cada quadro —
/// o mesmo bit que tocar.
#[derive(Clone, Debug, Default)]
pub struct Simulador<const N: usize = 3> {
    ultimo: Option<(i64, Passo<N>)>,
    /// O quadro antes do [`Self::ultimo`] quando se chegou a ele por um passo — a outra ponta da
    /// mistura entre quadros ([`Self::atraso_em`]).
    previo: Option<(i64, Passo<N>)>,
    pontos: BTreeMap<i64, Passo<N>>,
    chave: Option<[u64; 4]>,
    /// Num tempo virtual de laço: o período — o quadro 0 parte da ÓRBITA, não do repouso.
    periodo: Option<i64>,
    /// O tempo virtual do laço de agora, por `(de, ate, vaivém)` em quadros.
    ciclico: Option<((i64, i64, bool), Box<Simulador<N>>)>,
}

impl<const N: usize> Simulador<N> {
    /// O atraso `(x, y, rot)` = estado − entrada no quadro `f` (`f < 0` lê-se 0). `entrada(k)` é a
    /// pose ANIMADA do dono no quadro `k`.
    pub fn atraso(
        &mut self,
        f: i64,
        mola: impl Into<Lei<N>>,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        self.corre(f, mola.into(), fps, entrada)
    }

    /// [`Self::atraso`] sob qualquer [`Dinamica`]: a saída dela no quadro `f`.
    pub fn corre<D: Dinamica<N>>(
        &mut self,
        f: i64,
        lei: D,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        let [a, b, c] = lei.chave();
        let chave = [a, b, c, fps.to_bits()];
        if self.chave != Some(chave) {
            *self = Self {
                chave: Some(chave),
                periodo: self.periodo,
                ..Self::default()
            };
        }
        let f = f.max(0);
        #[expect(clippy::cast_possible_truncation, reason = "a lei é f32")]
        let dt = (1.0 / fps) as f32;
        let (mut k, mut s) = self.partida(f, entrada, lei, dt);
        while k < f {
            if k + 1 == f {
                self.previo = Some((k, s));
            }
            k += 1;
            s = lei.passo(s, entrada(k), dt);
            if k % STRIDE == 0 {
                self.pontos.insert(k, s);
            }
        }
        self.ultimo = Some((f, s));
        lei.saida(s)
    }

    /// ⭐ O atraso no instante CONTÍNUO `q` (em quadros): o ecrã desenha entre os quadros da timeline e
    /// a curva lê o instante contínuo — a mola anda com ela, a mistura linear dos quadros `⌊q⌋` e
    /// `⌊q⌋ + 1`. Função de `q` só: tocar e arrastar dão o mesmo bit.
    pub fn atraso_em(
        &mut self,
        q: f64,
        mola: impl Into<Lei<N>>,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        self.corre_em(q, mola.into(), fps, entrada)
    }

    /// [`Self::atraso_em`] sob qualquer [`Dinamica`].
    pub fn corre_em<D: Dinamica<N>>(
        &mut self,
        q: f64,
        mola: D,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        let q = if q.is_finite() { q.max(0.0) } else { 0.0 };
        #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
        let k = q.floor() as i64;
        #[expect(clippy::cast_precision_loss, reason = "um quadro")]
        let u = q - k as f64;
        if u == 0.0 {
            return self.corre(k, mola, fps, entrada);
        }
        let b = self.corre(k + 1, mola, fps, entrada);
        let a = match self.previo {
            Some((p, s)) if p == k && entrada(k) == s.entrada => mola.saida(s),
            _ => {
                let a = self.corre(k, mola, fps, entrada);
                let _ = self.corre(k + 1, mola, fps, entrada);
                a
            }
        };
        let angular = mola.angular();
        std::array::from_fn(|c| {
            let d = b[c] - a[c];
            a[c] + if angular[c] { wrap_pi(d) } else { d } * u
        })
    }

    /// De onde re-simular para chegar a `f`: o último quadro (se é `f` ou `f − 1`), senão o
    /// checkpoint válido mais recente ≤ `f`, senão o repouso no quadro 0.
    fn partida<D: Dinamica<N>>(
        &mut self,
        f: i64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
        lei: D,
        dt: f32,
    ) -> (i64, Passo<N>) {
        if let Some((u, s)) = self.ultimo
            && (u == f || u + 1 == f)
            && entrada(u) == s.entrada
        {
            return (u, s);
        }
        while let Some((&k, &p)) = self.pontos.range(..=f).next_back() {
            if entrada(k) == p.entrada {
                return (k, p);
            }
            // Os de depois dependem deste: todos caem.
            let _ = self.pontos.split_off(&k);
        }
        let s = match self.periodo {
            Some(p) => Self::orbita(p, entrada, lei, dt),
            None => lei.repouso(entrada(0)),
        };
        self.pontos.insert(0, s);
        (0, s)
    }
}

/// O laço do transporte, em quadros: `de..ate`, e se vai e volta (com o sentido de agora).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Laco {
    pub de: f64,
    pub ate: f64,
    pub vaivem: bool,
    pub para_a_frente: bool,
}

/// O teto de passos da órbita de um laço, por mudança de laço ou de animação: ~70 min de animação a
/// 24 q/s. Uma mola de ressalto ≤ 0,9 assenta em poucas voltas; perto do ressalto máximo (0,99) e com
/// uma duração longa a órbita pode não fechar a 1e-6 dentro dele — fica a última volta (o mesmo bit
/// sempre), e a costura carrega o resto.
const MAX_PASSOS_DA_ORBITA: i64 = 100_000;

impl<const N: usize> Simulador<N> {
    /// ⭐ [`Self::atraso_em`] sob o LAÇO do transporte. Fora do laço (ou sem ele), a timeline linear.
    /// Dentro, a mola corre num tempo VIRTUAL que não volta atrás: em laço o período é `ate − de`; em
    /// vaivém, a ida e a volta (`2·(ate − de)`), a volta a ler a animação ao contrário — a mola reage,
    /// para a frente, ao corpo que volta. O quadro 0 do tempo virtual é a ÓRBITA que se repete (a mola
    /// de um laço tocado há muito tempo): o fim junta-se ao começo sem salto. Função de `(q, laço,
    /// sentido)` só: tocar e arrastar dão o mesmo bit.
    pub fn atraso_no_laco(
        &mut self,
        q: f64,
        laco: Option<Laco>,
        mola: impl Into<Lei<N>>,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        self.corre_no_laco(q, laco, mola.into(), fps, entrada)
    }

    /// [`Self::atraso_no_laco`] sob qualquer [`Dinamica`].
    pub fn corre_no_laco<D: Dinamica<N>>(
        &mut self,
        q: f64,
        laco: Option<Laco>,
        mola: D,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
    ) -> [f64; N] {
        #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
        let quadros = laco
            .filter(|l| l.de.is_finite() && l.ate.is_finite() && q >= l.de && q <= l.ate)
            .map(|l| (l, l.de.round() as i64, l.ate.round() as i64))
            .filter(|&(_, a, b)| b > a);
        let Some((l, a, b)) = quadros else {
            return self.corre_em(q, mola, fps, entrada);
        };
        let n = b - a;
        let p = if l.vaivem { 2 * n } else { n };
        #[expect(clippy::cast_precision_loss, reason = "quadros")]
        let (u, nf, pf) = (q - a as f64, n as f64, p as f64);
        let v = if l.vaivem && !l.para_a_frente {
            2.0 * nf - u
        } else {
            u
        };
        let chave = (a, b, l.vaivem);
        let ciclico = match &mut self.ciclico {
            Some((c, s)) if *c == chave => s,
            outro => {
                *outro = Some((
                    chave,
                    Box::new(Self {
                        periodo: Some(p),
                        ..Self::default()
                    }),
                ));
                &mut outro.as_mut().expect("acabou de nascer").1
            }
        };
        let mut virtual_ = |k: i64| {
            let m = k.rem_euclid(p);
            entrada(a + if m > n { p - m } else { m })
        };
        ciclico.corre_em(v.rem_euclid(pf), mola, fps, &mut virtual_)
    }

    /// A órbita do laço de período `p`: a mola parte do repouso e corre voltas inteiras até o estado
    /// no fim de uma volta ser o do começo (1e-6), ou até ao [`MAX_PASSOS_DA_ORBITA`].
    fn orbita<D: Dinamica<N>>(
        p: i64,
        entrada: &mut dyn FnMut(i64) -> [f64; N],
        lei: D,
        dt: f32,
    ) -> Passo<N> {
        let ins: Vec<[f64; N]> = (0..=p).map(&mut *entrada).collect();
        let mut s = lei.repouso(ins[0]);
        let mut feitos = 0;
        while feitos < MAX_PASSOS_DA_ORBITA {
            let inicio = s;
            for e in &ins[1..] {
                s = lei.passo(s, *e, dt);
            }
            feitos += p;
            let perto =
                |a: [f32; N], b: [f32; N]| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-6);
            if perto(inicio.valor, s.valor) && perto(inicio.vel, s.vel) {
                break;
            }
        }
        s
    }
}

#[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
fn repouso<const N: usize>(e: [f64; N]) -> Passo<N> {
    Passo {
        valor: e.map(|v| v as f32),
        vel: [0.0; N],
        entrada: e,
    }
}

/// O atraso de um estado: estado − entrada.
fn desvio<const N: usize>(s: Passo<N>, angular: [bool; N]) -> [f64; N] {
    std::array::from_fn(|c| {
        let d = f64::from(s.valor[c]) - s.entrada[c];
        if angular[c] { wrap_pi(d) } else { d }
    })
}

#[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
fn um_passo<const N: usize>(
    s: Passo<N>,
    e: [f64; N],
    angular: [bool; N],
    (tension, friction, dt): (f32, f32, f32),
) -> Passo<N> {
    let mut n = Passo { entrada: e, ..s };
    for c in 0..N {
        // Um ângulo persegue pelo arco curto: o alvo desembrulha-se perto do estado.
        let alvo = if angular[c] {
            f64::from(s.valor[c]) + wrap_pi(e[c] - f64::from(s.valor[c]))
        } else {
            e[c]
        };
        (n.valor[c], n.vel[c]) = passo(s.valor[c], s.vel[c], alvo as f32, tension, friction, dt);
    }
    n
}

/// Os «Valor» da cena, por `(Entity::to_bits, Item::id)` (06_W5 §3.1), preparados pela família
/// antes da pilha: a saída deste quadro já na faixa do destino (a pilha aplica-a na posição do item
/// quando o destino é a pose); sem saída, a fonte ou o destino não existe.
#[derive(Resource, Default, Debug)]
pub struct Valores {
    pub sims: BTreeMap<(u64, u32), Simulador<1>>,
    /// A fonte que nenhuma curva anima: o valor que se viu no quadro em que mudou — o scrub relê-o.
    pub historia: BTreeMap<(u64, u32), BTreeMap<i64, f64>>,
    pub saidas: BTreeMap<(u64, u32), f64>,
}

/// As molas da cena, por `(Entity::to_bits, Item::id)`: o simulador de cada uma e o atraso deste
/// quadro, que a fase soma à pose na posição do item.
#[derive(Resource, Default, Debug)]
pub struct Molas {
    pub sims: BTreeMap<(u64, u32), Simulador>,
    pub atrasos: BTreeMap<(u64, u32), [f64; 3]>,
    /// ⭐ A mola AO VIVO com o tempo parado (W6, o idioma do Spine 4.2): só as que ainda balançam.
    pub vivas: BTreeMap<(u64, u32), AoVivo>,
    /// O quadro anterior de cada mola com o tempo parado: `(entrada mostrada, posição mostrada)`.
    pub vistos: BTreeMap<(u64, u32), ([f64; 3], [f64; 3])>,
    /// O instante da vista e o relógio de PAREDE (s) do quadro anterior.
    pub parede: Option<(u64, f64)>,
}

/// O sub-passo fixo da mola ao vivo (s): o relógio de parede anda aos saltos do ecrã, a lei não.
pub const SUBPASSO_AO_VIVO: f64 = 1.0 / 240.0;

/// O maior salto de parede (s) que um quadro dá à mola ao vivo — tecto do ORÇAMENTO DO QUADRO:
/// `0,1 s / SUBPASSO_AO_VIVO` = 24 sub-passos por mola. Um quadro mais longo é uma paragem (janela
/// minimizada, suspensão), não tempo que o artista viu: a mola continua de onde estava. Uma mola que
/// nunca assenta (ressalto alto) continua a balançar como no Spine, a ≤ 24 sub-passos por quadro.
pub const MAX_DT_AO_VIVO: f64 = 0.1;

/// Abaixo disto (m, rad, e por segundo) a mola ao vivo assentou e sai: o caminho puro volta a mandar.
pub const REPOUSO_AO_VIVO: f32 = 1e-5;

/// ⭐ **A mola ao vivo de um item** — preview, nunca documento: persegue a pose MOSTRADA (o gizmo) mais
/// o atraso determinístico do quadro, à lei da [`Lei`], em sub-passos fixos de relógio de parede.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AoVivo {
    s: Passo<3>,
    resto: f64,
}

impl AoVivo {
    /// Nasce parada onde o ecrã a mostrava no quadro anterior.
    #[must_use]
    pub fn nasce(posicao: [f64; 3]) -> Self {
        Self {
            s: repouso(posicao),
            resto: 0.0,
        }
    }

    /// Anda `dt` segundos de parede atrás de `alvo`. `true` = assentou (larga-se; vale o `alvo`).
    pub fn anda(&mut self, lei: Lei<3>, alvo: [f64; 3], dt: f64) -> bool {
        #[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
        let h = SUBPASSO_AO_VIVO as f32;
        self.resto += dt.clamp(0.0, MAX_DT_AO_VIVO);
        while self.resto >= SUBPASSO_AO_VIVO {
            self.resto -= SUBPASSO_AO_VIVO;
            self.s = lei.passo(self.s, alvo, h);
            if self.assentou(lei, alvo) {
                return true;
            }
        }
        self.assentou(lei, alvo)
    }

    fn assentou(&self, lei: Lei<3>, alvo: [f64; 3]) -> bool {
        (0..3).all(|c| {
            let d = f64::from(self.s.valor[c]) - alvo[c];
            let d = if lei.angular[c] { wrap_pi(d) } else { d };
            d.abs() <= f64::from(REPOUSO_AO_VIVO) && self.s.vel[c].abs() <= REPOUSO_AO_VIVO
        })
    }

    /// A posição de agora.
    #[must_use]
    pub fn posicao(&self) -> [f64; 3] {
        self.s.valor.map(f64::from)
    }
}
