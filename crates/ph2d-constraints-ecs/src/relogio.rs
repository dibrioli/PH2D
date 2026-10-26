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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Passo {
    valor: [f32; 3],
    vel: [f32; 3],
    /// A entrada com que este quadro se calculou — um checkpoint cuja entrada mudou (a animação foi
    /// editada) já não vale.
    entrada: [f64; 3],
}

/// ⭐ **A mola de UM item: o estado é função do QUADRO.** Tocar para a frente dá um passo; um salto
/// (scrub, rebobinar) volta ao checkpoint válido ≤ quadro e re-simula com a entrada de cada quadro —
/// o mesmo bit que tocar.
#[derive(Clone, Debug, Default)]
pub struct Simulador {
    ultimo: Option<(i64, Passo)>,
    /// O quadro antes do [`Self::ultimo`] quando se chegou a ele por um passo — a outra ponta da
    /// mistura entre quadros ([`Self::atraso_em`]).
    previo: Option<(i64, Passo)>,
    pontos: BTreeMap<i64, Passo>,
    chave: Option<[u64; 3]>,
    /// Num tempo virtual de laço: o período — o quadro 0 parte da ÓRBITA, não do repouso.
    periodo: Option<i64>,
    /// O tempo virtual do laço de agora, por `(de, ate, vaivém)` em quadros.
    ciclico: Option<((i64, i64, bool), Box<Simulador>)>,
}

impl Simulador {
    /// O atraso `(x, y, rot)` = estado − entrada no quadro `f` (`f < 0` lê-se 0). `entrada(k)` é a
    /// pose ANIMADA do dono no quadro `k`.
    pub fn atraso(
        &mut self,
        f: i64,
        mola: &Spring,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; 3],
    ) -> [f64; 3] {
        let chave = [
            mola.duracao.to_bits(),
            mola.ressalto.to_bits(),
            fps.to_bits(),
        ];
        if self.chave != Some(chave) {
            *self = Self {
                chave: Some(chave),
                periodo: self.periodo,
                ..Self::default()
            };
        }
        let f = f.max(0);
        #[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
        let (dt, d, b) = (
            (1.0 / fps) as f32,
            mola.duracao as f32,
            mola.ressalto as f32,
        );
        let (tension, friction) = physics_of(MODE_TIME, 0.0, 0.0, d, b);
        let (mut k, mut s) = self.partida(f, entrada, (tension, friction, dt));
        while k < f {
            if k + 1 == f {
                self.previo = Some((k, s));
            }
            k += 1;
            s = um_passo(s, entrada(k), tension, friction, dt);
            if k % STRIDE == 0 {
                self.pontos.insert(k, s);
            }
        }
        self.ultimo = Some((f, s));
        desvio(s)
    }

    /// ⭐ O atraso no instante CONTÍNUO `q` (em quadros): o ecrã desenha entre os quadros da timeline e
    /// a curva lê o instante contínuo — a mola anda com ela, a mistura linear dos quadros `⌊q⌋` e
    /// `⌊q⌋ + 1`. Função de `q` só: tocar e arrastar dão o mesmo bit.
    pub fn atraso_em(
        &mut self,
        q: f64,
        mola: &Spring,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; 3],
    ) -> [f64; 3] {
        let q = if q.is_finite() { q.max(0.0) } else { 0.0 };
        #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
        let k = q.floor() as i64;
        #[expect(clippy::cast_precision_loss, reason = "um quadro")]
        let u = q - k as f64;
        if u == 0.0 {
            return self.atraso(k, mola, fps, entrada);
        }
        let b = self.atraso(k + 1, mola, fps, entrada);
        let a = match self.previo {
            Some((p, s)) if p == k && entrada(k) == s.entrada => desvio(s),
            _ => {
                let a = self.atraso(k, mola, fps, entrada);
                let _ = self.atraso(k + 1, mola, fps, entrada);
                a
            }
        };
        [
            a[0] + (b[0] - a[0]) * u,
            a[1] + (b[1] - a[1]) * u,
            a[2] + wrap_pi(b[2] - a[2]) * u,
        ]
    }

    /// De onde re-simular para chegar a `f`: o último quadro (se é `f` ou `f − 1`), senão o
    /// checkpoint válido mais recente ≤ `f`, senão o repouso no quadro 0.
    fn partida(
        &mut self,
        f: i64,
        entrada: &mut dyn FnMut(i64) -> [f64; 3],
        lei: (f32, f32, f32),
    ) -> (i64, Passo) {
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
            Some(p) => Self::orbita(p, entrada, lei),
            None => repouso(entrada(0)),
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

impl Simulador {
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
        mola: &Spring,
        fps: f64,
        entrada: &mut dyn FnMut(i64) -> [f64; 3],
    ) -> [f64; 3] {
        #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
        let quadros = laco
            .filter(|l| l.de.is_finite() && l.ate.is_finite() && q >= l.de && q <= l.ate)
            .map(|l| (l, l.de.round() as i64, l.ate.round() as i64))
            .filter(|&(_, a, b)| b > a);
        let Some((l, a, b)) = quadros else {
            return self.atraso_em(q, mola, fps, entrada);
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
        ciclico.atraso_em(v.rem_euclid(pf), mola, fps, &mut virtual_)
    }

    /// A órbita do laço de período `p`: a mola parte do repouso e corre voltas inteiras até o estado
    /// no fim de uma volta ser o do começo (1e-6), ou até ao [`MAX_PASSOS_DA_ORBITA`].
    fn orbita(
        p: i64,
        entrada: &mut dyn FnMut(i64) -> [f64; 3],
        (tension, friction, dt): (f32, f32, f32),
    ) -> Passo {
        let ins: Vec<[f64; 3]> = (0..=p).map(&mut *entrada).collect();
        let mut s = repouso(ins[0]);
        let mut feitos = 0;
        while feitos < MAX_PASSOS_DA_ORBITA {
            let inicio = s;
            for e in &ins[1..] {
                s = um_passo(s, *e, tension, friction, dt);
            }
            feitos += p;
            let perto =
                |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-6);
            if perto(inicio.valor, s.valor) && perto(inicio.vel, s.vel) {
                break;
            }
        }
        s
    }
}

#[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
fn repouso(e: [f64; 3]) -> Passo {
    Passo {
        valor: e.map(|v| v as f32),
        vel: [0.0; 3],
        entrada: e,
    }
}

/// O atraso de um estado: estado − entrada.
fn desvio(s: Passo) -> [f64; 3] {
    let e = s.entrada;
    [
        f64::from(s.valor[0]) - e[0],
        f64::from(s.valor[1]) - e[1],
        wrap_pi(f64::from(s.valor[2]) - e[2]),
    ]
}

#[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
fn um_passo(s: Passo, e: [f64; 3], tension: f32, friction: f32, dt: f32) -> Passo {
    let mut n = Passo { entrada: e, ..s };
    for c in 0..3 {
        // A rotação persegue pelo arco curto: o alvo desembrulha-se perto do estado.
        let alvo = if c == 2 {
            f64::from(s.valor[2]) + wrap_pi(e[2] - f64::from(s.valor[2]))
        } else {
            e[c]
        };
        (n.valor[c], n.vel[c]) = passo(s.valor[c], s.vel[c], alvo as f32, tension, friction, dt);
    }
    n
}

/// As molas da cena, por `(Entity::to_bits, Item::id)`: o simulador de cada uma e o atraso deste
/// quadro, que a fase soma à pose na posição do item.
#[derive(Resource, Default, Debug)]
pub struct Molas {
    pub sims: BTreeMap<(u64, u32), Simulador>,
    pub atrasos: BTreeMap<(u64, u32), [f64; 3]>,
}
