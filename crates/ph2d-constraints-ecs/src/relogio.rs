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
                ..Self::default()
            };
        }
        let f = f.max(0);
        let (mut k, mut s) = self.partida(f, entrada);
        #[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
        let (dt, d, b) = (
            (1.0 / fps) as f32,
            mola.duracao as f32,
            mola.ressalto as f32,
        );
        let (tension, friction) = physics_of(MODE_TIME, 0.0, 0.0, d, b);
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
    fn partida(&mut self, f: i64, entrada: &mut dyn FnMut(i64) -> [f64; 3]) -> (i64, Passo) {
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
        let e = entrada(0);
        #[expect(clippy::cast_possible_truncation, reason = "a lei da mola é f32")]
        let s = Passo {
            valor: e.map(|v| v as f32),
            vel: [0.0; 3],
            entrada: e,
        };
        self.pontos.insert(0, s);
        (0, s)
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
