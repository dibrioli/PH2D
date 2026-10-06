//! ⭐⭐⭐ **O CONTACTO DAS PEÇAS DO MOTION PELO MOTOR DA CASA** (`rapier2d`, ADR-0131) — doc 121
//! §9.20, ordem do dono (06/10): *«usar o motor da casa»*.
//!
//! Medido no oráculo antes de construir: a mesma pilha custa `1,5` ms por tique a `4 096` peças
//! no rapier contra `48`–`93` na lei por colunas que aqui esteve (`ph2d-contact::separate` +
//! `impulsos`), e a diferença é de ALGORITMO: o rapier guarda os contactos entre tiques e aquece o
//! solver com os impulsos da vez anterior.
//!
//! ## O desenho (as três perguntas, medidas no oráculo — a tabela está no doc)
//!
//! - **O mundo é PERSISTENTE** — o [`Mundo`] é a memória do `sim.step` ([`ph2d_nodegraph::cook::Memo`]),
//!   e viaja no ponto de recuo: um recuo seguido de Play dá os bits da 1.ª passagem. Reconstruído a
//!   cada tique ele custaria `10×` e a pilha ficaria `3×` mais agitada.
//! - **O stream é a VERDADE do estado; o mundo, a dos contactos.** Por passo o [`passo`] sincroniza
//!   por `id` (nascimentos, mortes, quem mudou de forma), escreve no corpo só o que mudou desde a sua
//!   última saída, dá UM passo do rapier e devolve posição, velocidade, ângulo e giro.
//! - **Os obstáculos do `sim.collide` são colisores FIXOS** ([`ph2d_contact::obstaculo`]): fora do
//!   mundo o peso do monte não passa pelo solver (`8×` mais agitado, medido).

#![forbid(unsafe_code)]

mod fixo;
mod peca;
mod rolar;

use ph2d_contact::obstaculo::{self, FormaFixa};
use ph2d_nodegraph::attr::{Column, Stream};
use rapier2d::prelude::*;
use std::collections::BTreeMap;

/// **Iterações do solver por passo** — a omissão do rapier, a mesma do oráculo que mediu a pilha
/// (`4 096` peças `1,54` ms; `8` iterações `1,62` ms e a mesma pilha, doc 121 §9.20).
pub const ITERACOES: usize = 4;

/// **Passes de ESTABILIZAÇÃO por iteração** — os que tiram do resultado a velocidade que a correcção
/// da penetração usou. ⚠️ Com a omissão (`1`) duas CAIXAS que nascem sobrepostas saem a VOAR, mesmo
/// paradas — a lei que o dono aprovou (*«quem nasce sobreposto e parado não ganha velocidade»*,
/// doc 111) caía. Medido (duas caixas `1 × 4` sobrepostas `0,4`, `release`; a pilha da `=114`):
///
/// | estabilizações | velocidade relativa que sobra | pilha `1 024` med · `4 096` med |
/// |---:|---:|---|
/// | `1` (omissão) | `0,84` u/s | `0,42` · `1,79` ms |
/// | `2` | — | `0,54` · `1,85` ms |
/// | **`4`** | **`0,009`** | `0,57` · `1,83` ms |
///
/// O recurso é o relógio do tique, e `4` cabe; os discos não sofriam (`0` com qualquer número).
pub const ESTABILIZACOES: usize = 4;

const GRAUS: f32 = 180.0 / std::f32::consts::PI;
const RADIANOS: f32 = std::f32::consts::PI / 180.0;
/// A etiqueta de um obstáculo no `user_data` do colisor (uma peça leva o `id`, que é `u32`).
const FIXO: u128 = 1 << 64;

/// Uma peça no mundo: os seus dois handles, o que o stream declarou dela, e os bits da última
/// SAÍDA — o que diz se alguém (uma força, um `drive`, a taça antiga) lhe mexeu entre dois passos.
#[derive(Clone)]
struct Corpo {
    corpo: RigidBodyHandle,
    colisor: ColliderHandle,
    spec: peca::Spec,
    saida: [u32; 6],
    /// O impulso angular que os contactos do último passo suportam contra o giro (o `Rolling`) —
    /// ver [`rolar`].
    travao: f32,
    /// A fase PARADA do rolamento: a rotação trancada no solver até o binário pedido passar do
    /// `travao` (ver [`rolar`]).
    presa: bool,
}

#[derive(Clone)]
struct Fixo {
    colisor: ColliderHandle,
    forma: FormaFixa,
    atrito: f32,
    /// O salto contra cada linha do stream (o `Randomness` do cartão) — ver [`Saltos`].
    saltos: Vec<f32>,
    /// O salto VARIA entre as peças — só então o gancho corre.
    varia: bool,
}

/// ⭐ **O salto POR PEÇA de um obstáculo** — o `Randomness` do `sim.collide` varia-o por peça, e um
/// colisor do rapier tem UM. Só os obstáculos cujo salto varia ligam o gancho
/// (`MODIFY_SOLVER_CONTACTS`), e nele o salto de cada contacto é `max(o da peça contra o
/// obstáculo, o da peça)` — a regra `Max` de sempre (`ph2d_contact::atrito::salto`).
struct Saltos<'a> {
    fixos: &'a BTreeMap<u32, Fixo>,
    indice: &'a BTreeMap<u32, usize>,
}

impl PhysicsHooks for Saltos<'_> {
    fn modify_solver_contacts(&self, ctx: &mut ContactModificationContext) {
        let (Some(c1), Some(c2)) = (
            ctx.colliders.get(ctx.collider1),
            ctx.colliders.get(ctx.collider2),
        ) else {
            return;
        };
        let (fixo, peca) = if c1.user_data & FIXO != 0 {
            (c1, c2)
        } else {
            (c2, c1)
        };
        #[expect(clippy::cast_possible_truncation, reason = "a etiqueta guarda um u32")]
        let (chave, id) = (fixo.user_data as u32, peca.user_data as u32);
        let Some(e) = self
            .fixos
            .get(&chave)
            .zip(self.indice.get(&id))
            .and_then(|(f, i)| f.saltos.get(*i))
        else {
            return;
        };
        *ctx.restitution = e.max(peca.restitution());
    }
}

/// **O mundo de contacto** — a memória do `sim.step` (ver o cabeçalho da crate).
pub struct Mundo {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    islands: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    juntas: ImpulseJointSet,
    multi: MultibodyJointSet,
    ccd: CCDSolver,
    /// Só memória de trabalho do rapier — fora da cópia (a `ph2d-physics` também a deixa fora do
    /// ponto de recuo dela, e o recuo exacto tem gate lá e aqui).
    pipeline: PhysicsPipeline,
    /// As peças no mundo, ALINHADAS às linhas do stream do último passo (`ordem` = os `id`s delas):
    /// no caso comum (ninguém nasceu nem morreu) a linha `i` é o corpo `i`, sem busca nenhuma.
    corpos: Vec<Option<Corpo>>,
    ordem: Vec<u32>,
    fixos: BTreeMap<u32, Fixo>,
    /// O instante (o `sim_t` do stream) em que os corpos estão.
    t: f32,
    passos: u64,
}

impl Clone for Mundo {
    fn clone(&self) -> Self {
        Self {
            bodies: self.bodies.clone(),
            colliders: self.colliders.clone(),
            islands: self.islands.clone(),
            broad: self.broad.clone(),
            narrow: self.narrow.clone(),
            juntas: self.juntas.clone(),
            multi: self.multi.clone(),
            ccd: self.ccd.clone(),
            pipeline: PhysicsPipeline::new(),
            corpos: self.corpos.clone(),
            ordem: self.ordem.clone(),
            fixos: self.fixos.clone(),
            t: self.t,
            passos: self.passos,
        }
    }
}

/// O factor das alocações por trás de cada corpo e colisor (as formas partilhadas, as ilhas, a
/// fase larga) — o da `ph2d-physics` (`ACCEL_FACTOR`), e o gate
/// `os_bytes_declarados_cobrem_a_copia` mede-o com um alocador que conta.
const FACTOR: usize = 3;

impl ph2d_nodegraph::cook::Memo for Mundo {
    fn approx_bytes(&self) -> usize {
        use std::mem::size_of;
        let estruturas = self.bodies.len() * size_of::<RigidBody>()
            + self.colliders.len() * size_of::<Collider>();
        let contactos: usize = self
            .narrow
            .contact_pairs()
            .map(|p| {
                size_of::<ContactPair>()
                    + p.manifolds
                        .iter()
                        .map(|m| {
                            size_of::<ContactManifold>() + m.points.len() * size_of::<Contact>()
                        })
                        .sum::<usize>()
            })
            .sum();
        (estruturas + contactos) * FACTOR
            + self.corpos.len() * (size_of::<Option<Corpo>>() + size_of::<u32>())
            + size_of::<Self>()
    }
}

impl Mundo {
    fn novo(t: f32) -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            islands: IslandManager::new(),
            broad: DefaultBroadPhase::new(),
            narrow: NarrowPhase::new(),
            juntas: ImpulseJointSet::new(),
            multi: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            pipeline: PhysicsPipeline::new(),
            corpos: Vec::new(),
            ordem: Vec::new(),
            fixos: BTreeMap::new(),
            t,
            passos: 0,
        }
    }

    /// Quantos passos do rapier este mundo deu desde que nasceu — o instrumento dos gates (um
    /// recomeço é um mundo NOVO; uma reavaliação no mesmo instante não dá passo nenhum).
    #[must_use]
    pub fn passos(&self) -> u64 {
        self.passos
    }

    /// Quantas peças estão no mundo agora.
    #[must_use]
    pub fn pecas(&self) -> usize {
        self.corpos.iter().flatten().count()
    }
}

/// O que o `sim.step` entrega ao mundo: o stream de ENTRADA (as colunas do colisor, do material,
/// do `id` e dos obstáculos declarados) e o relógio de cada peça.
pub struct Pedido<'a> {
    pub state: &'a Stream,
    /// O `inv_mass` de cada peça.
    pub pesos: &'a [f32],
    /// O passo de cada peça neste tique (o `sim.step` dá `0` a quem nasceu agora).
    pub dt: &'a [f32],
    /// O `sim_t` de entrada — `None` num stream que nunca deu um passo (um recomeço).
    pub sim_t: Option<&'a [f32]>,
    pub playhead: f32,
}

/// O estado das peças. ENTRA: `antes`/`rot_antes` (onde a peça estava e o seu ângulo, em graus),
/// `vel` e `spin` já com a metade da VELOCIDADE do `sim.step` (a força, o amortecimento, o limite).
/// SAI, para quem está no mundo: `p`, `vel`, `rot` e `spin` depois do contacto e da integração do
/// rapier. ⚠️ Quem não está no mundo não é tocado (o `sim.step` já o integrou).
pub struct Estado<'a> {
    pub antes: &'a [[f32; 2]],
    pub rot_antes: &'a [f32],
    pub p: &'a mut [[f32; 2]],
    pub vel: &'a mut [[f32; 2]],
    pub rot: &'a mut [f32],
    pub spin: &'a mut [f32],
}

/// O que o passo fez, para o `sim.step` escrever as colunas.
pub struct Feito {
    /// Quem o mundo moveu neste passo.
    pub movidas: Vec<bool>,
    /// As chaves dos obstáculos que estão no mundo — o `sim.step` passa o recibo a cada uma.
    pub recibos: Vec<u32>,
}

/// O `id` estável de cada linha — a coluna `id` quando ela existe e não se repete, senão o índice.
/// `anterior` é a ordem do passo de antes: igual a ela, a unicidade já foi verificada.
fn ids(s: &Stream, anterior: &[u32]) -> Vec<u32> {
    let n = s.count();
    if let Some(Column::Scalar(v)) = s.get("id")
        && v.len() == n
    {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma identidade é um inteiro >= 0"
        )]
        let ids: Vec<u32> = v.iter().map(|x| x.max(0.0).round() as u32).collect();
        if ids == anterior {
            return ids;
        }
        let mut vistos = ids.clone();
        vistos.sort_unstable();
        vistos.dedup();
        if vistos.len() == n {
            return ids;
        }
    }
    #[expect(clippy::cast_possible_truncation, reason = "um índice de linha")]
    (0..n).map(|i| i as u32).collect()
}

fn bits(p: [f32; 2], v: [f32; 2], rot: f32, spin: f32) -> [u32; 6] {
    [
        p[0].to_bits(),
        p[1].to_bits(),
        v[0].to_bits(),
        v[1].to_bits(),
        rot.to_bits(),
        spin.to_bits(),
    ]
}

/// A etiqueta do colisor de uma peça: o `id` nos 32 bits de baixo e o `μr` (os bits do `f32`) nos
/// 32 de cima — o gancho do rolamento lê-o sem mapa nenhum.
fn etiqueta(id: u32, spec: &peca::Spec) -> u128 {
    let mu_r = spec.material.map_or(0.0, |m| m.rolar);
    u128::from(id) | (u128::from(mu_r.to_bits()) << 32)
}

fn rotacao(graus: f32) -> Rotation {
    Rotation::new(graus * RADIANOS)
}

/// ⭐⭐⭐ **UM PASSO DO MUNDO DE CONTACTO** — devolve `None` quando ninguém declara colisor (e o
/// mundo some: o passo é o de sempre).
///
/// `mundo` é a memória do `sim.step`. Ele CONTINUA quando o stream está no instante em que o mundo
/// parou (`sim_t`); senão — um recomeço (`Loop`), uma memória perdida — nasce um mundo NOVO do
/// stream, e é isso que faz a 2.ª volta de um `Loop` dar os bits da 1.ª.
pub fn passo(
    mundo: &mut Option<Mundo>,
    pedido: &Pedido<'_>,
    estado: &mut Estado<'_>,
) -> Option<Feito> {
    let mut t0 = std::time::Instant::now();
    let s = pedido.state;
    let n = s.count();
    let specs = peca::specs(s, pedido.pesos);
    marca(0, &mut t0);
    let Some(specs) = specs else {
        *mundo = None;
        return None;
    };
    // O passo do mundo é o das peças que já tinham relógio; quem nasceu agora (dt 0) só entra
    // DEPOIS do passo — não se move neste tique, como sem o mundo.
    let dt = (0..n)
        .filter(|i| specs[*i].is_some())
        .map(|i| pedido.dt.get(i).copied().unwrap_or(0.0))
        .fold(0.0_f32, f32::max);
    let continua = mundo.as_ref().is_some_and(|m| {
        pedido.sim_t.is_some_and(|t| {
            (0..n).any(|i| specs[i].is_some() && pedido.dt[i] == dt && t[i] == m.t)
        })
    });
    if !continua {
        *mundo = Some(Mundo::novo(pedido.playhead - dt));
    }
    let m = mundo.as_mut()?;
    let ids = ids(s, &m.ordem);
    marca(1, &mut t0);
    sincroniza_fixos(m, &obstaculo::declarados(s));
    marca(2, &mut t0);
    realinha(m, &ids, &specs);
    marca(3, &mut t0);
    let mut depois = Vec::new();
    let mut antes_do_passo: Vec<Option<Rotation>> = vec![None; n];
    for i in 0..n {
        let Some(spec) = specs[i] else { continue };
        if pedido.dt[i] < dt && m.corpos[i].is_none() {
            depois.push(i);
            continue;
        }
        escreve(m, i, ids[i], &spec, estado, dt);
        antes_do_passo[i] = m.corpos[i].as_ref().map(|c| *m.bodies[c.corpo].rotation());
    }
    // O índice das linhas por `id` — só o gancho do salto por peça o lê.
    let indice: BTreeMap<u32, usize> = if m.fixos.values().any(|f| f.varia) {
        ids.iter().enumerate().map(|(i, id)| (*id, i)).collect()
    } else {
        BTreeMap::new()
    };
    let mut feito = Feito {
        movidas: vec![false; n],
        recibos: m.fixos.keys().copied().collect(),
    };
    marca(4, &mut t0);
    if dt > 0.0 {
        let params = IntegrationParameters {
            dt,
            num_solver_iterations: ITERACOES,
            num_internal_stabilization_iterations: ESTABILIZACOES,
            ..IntegrationParameters::default()
        };
        m.pipeline.step(
            Vector::ZERO,
            &params,
            &mut m.islands,
            &mut m.broad,
            &mut m.narrow,
            &mut m.bodies,
            &mut m.colliders,
            &mut m.juntas,
            &mut m.multi,
            &mut m.ccd,
            &Saltos {
                fixos: &m.fixos,
                indice: &indice,
            },
            &(),
        );
        m.passos += 1;
        marca(5, &mut t0);
        let rolantes: BTreeMap<u32, rolar::Rolante> = m
            .ordem
            .iter()
            .zip(&m.corpos)
            .enumerate()
            .filter_map(|(linha, (id, c))| {
                let c = c.as_ref()?;
                let mu_r = c.spec.material.map_or(0.0, |x| x.rolar);
                (mu_r > 0.0).then_some((
                    *id,
                    rolar::Rolante {
                        mu_r,
                        inercia_inv: c.spec.inercia_inv,
                        linha,
                    },
                ))
            })
            .collect();
        let lido = rolar::le(&m.narrow, &m.colliders, &m.bodies, &rolantes, n);
        for (i, c) in m.corpos.iter_mut().enumerate() {
            let Some(c) = c else { continue };
            let cap = lido.capacidade[i];
            c.travao = cap;
            if c.presa {
                // Destrava quando os contactos pedem mais binário do que o rolamento segura.
                c.presa = lido.pedido[i] <= cap;
            } else if cap > 0.0 && c.spec.inercia_inv > 0.0 {
                // O rolamento pára-a neste passo ⇒ fase PARADA (o giro que sobra é zero).
                let b = &mut m.bodies[c.corpo];
                if (b.angvel() / c.spec.inercia_inv).abs() <= cap {
                    c.presa = true;
                    b.set_angvel(0.0, false);
                }
            }
        }
        marca(6, &mut t0);
        for i in 0..n {
            let (Some(r0), Some(c)) = (antes_do_passo[i], m.corpos[i].as_mut()) else {
                continue;
            };
            let b = &m.bodies[c.corpo];
            let (t, v) = (b.translation(), b.linvel());
            let delta = (r0.inverse() * *b.rotation()).angle();
            let (p, vel) = ([t.x, t.y], [v.x, v.y]);
            let rot = if delta == 0.0 {
                estado.rot_antes[i]
            } else {
                estado.rot_antes[i] + delta * GRAUS
            };
            let spin = b.angvel() * GRAUS;
            if p.iter().chain(&vel).all(|x| x.is_finite()) && rot.is_finite() && spin.is_finite() {
                estado.p[i] = p;
                estado.vel[i] = vel;
                estado.rot[i] = rot;
                estado.spin[i] = spin;
                feito.movidas[i] = true;
            }
            c.saida = bits(estado.p[i], estado.vel[i], estado.rot[i], estado.spin[i]);
        }
    }
    marca(7, &mut t0);
    for i in depois {
        if let Some(spec) = specs[i] {
            escreve(m, i, ids[i], &spec, estado, dt);
        }
    }
    m.t = pedido.playhead;
    Some(feito)
}

/// **Alinha os corpos às linhas deste stream** — no caso comum (os mesmos `id`s pela mesma ordem)
/// não faz nada; senão casa-os por `id` e tira do mundo quem morreu. E quem deixou de declarar
/// colisor também sai.
fn realinha(m: &mut Mundo, ids: &[u32], specs: &[Option<peca::Spec>]) {
    if m.ordem != ids {
        let antigos: BTreeMap<u32, Corpo> = m
            .ordem
            .drain(..)
            .zip(m.corpos.drain(..))
            .filter_map(|(id, c)| Some((id, c?)))
            .collect();
        let mut antigos = antigos;
        m.corpos = ids.iter().map(|id| antigos.remove(id)).collect();
        m.ordem = ids.to_vec();
        for (_, c) in antigos {
            m.bodies.remove(
                c.corpo,
                &mut m.islands,
                &mut m.colliders,
                &mut m.juntas,
                &mut m.multi,
                true,
            );
        }
    }
    for (i, spec) in specs.iter().enumerate() {
        if spec.is_none()
            && let Some(c) = m.corpos[i].take()
        {
            m.bodies.remove(
                c.corpo,
                &mut m.islands,
                &mut m.colliders,
                &mut m.juntas,
                &mut m.multi,
                true,
            );
        }
    }
}

/// Põe a peça da linha `i` no mundo (nova) ou escreve nela o que mudou desde a última saída — e o
/// binário do `Rolling` para o passo de `dt`.
fn escreve(m: &mut Mundo, i: usize, id: u32, spec: &peca::Spec, e: &Estado<'_>, dt: f32) {
    let (p, v, rot, spin) = (e.antes[i], e.vel[i], e.rot_antes[i], e.spin[i]);
    if let Some(c) = m.corpos[i].as_mut() {
        let cinematico = spec.massa_inv <= 0.0;
        if c.spec != *spec {
            c.presa = false;
            let (tipo, massa, travada) = peca::tipo_e_massa(spec);
            m.colliders
                .remove(c.colisor, &mut m.islands, &mut m.bodies, false);
            c.colisor = m.colliders.insert_with_parent(
                peca::colisor(spec, etiqueta(id, spec)),
                c.corpo,
                &mut m.bodies,
            );
            let b = &mut m.bodies[c.corpo];
            b.set_body_type(tipo, false);
            b.set_additional_mass_properties(massa, false);
            b.lock_rotations(travada, false);
            c.spec = *spec;
        }
        let b = &mut m.bodies[c.corpo];
        let antiga = c.saida;
        if (antiga[0], antiga[1]) != (p[0].to_bits(), p[1].to_bits()) {
            b.set_translation(Vector::new(p[0], p[1]), false);
        }
        if antiga[4] != rot.to_bits() {
            b.set_rotation(rotacao(rot), false);
        }
        if cinematico {
            b.set_next_kinematic_translation(Vector::new(e.p[i][0], e.p[i][1]));
        } else {
            if (antiga[2], antiga[3]) != (v[0].to_bits(), v[1].to_bits()) {
                b.set_linvel(Vector::new(v[0], v[1]), false);
            }
            if antiga[5] != spin.to_bits() {
                // Alguém mexeu no giro (um `drive`): a fase PARADA não o pode engolir.
                b.set_angvel(spin * RADIANOS, false);
                c.presa = false;
            }
            b.reset_torques(false);
            let travada = spec.inercia_inv <= 0.0;
            let trancar = travada || c.presa;
            // ⚠️ Só quando MUDA: trancar recalcula as propriedades de massa do corpo.
            if b.locked_axes().contains(LockedAxes::ROTATION_LOCKED) != trancar {
                b.lock_rotations(trancar, false);
            }
            if c.presa {
                b.set_angvel(0.0, false);
            } else {
                let tau = rolar::binario(b.angvel(), spec.inercia_inv, c.travao, dt);
                if tau != 0.0 {
                    b.add_torque(tau, false);
                }
            }
        }
        return;
    }
    let (tipo, massa, travada) = peca::tipo_e_massa(spec);
    let mut b = RigidBodyBuilder::new(tipo)
        .translation(Vector::new(p[0], p[1]))
        .can_sleep(false)
        .additional_mass_properties(massa)
        .user_data(u128::from(id))
        .build();
    b.set_rotation(rotacao(rot), false);
    if tipo == RigidBodyType::Dynamic {
        b.set_linvel(Vector::new(v[0], v[1]), false);
        b.set_angvel(spin * RADIANOS, false);
    }
    b.lock_rotations(travada, false);
    let corpo = m.bodies.insert(b);
    let colisor = m.colliders.insert_with_parent(
        peca::colisor(spec, etiqueta(id, spec)),
        corpo,
        &mut m.bodies,
    );
    m.corpos[i] = Some(Corpo {
        corpo,
        colisor,
        spec: *spec,
        saida: bits(p, v, rot, spin),
        travao: 0.0,
        presa: false,
    });
}

/// Os obstáculos declarados neste passo — os novos entram, os que mudaram trocam de colisor, os
/// que deixaram de ser declarados saem.
fn sincroniza_fixos(m: &mut Mundo, declarados: &[(u32, obstaculo::Obstaculo)]) {
    let vivos: Vec<u32> = declarados.iter().map(|(k, _)| *k).collect();
    let saem: Vec<u32> = m
        .fixos
        .keys()
        .filter(|k| !vivos.contains(k))
        .copied()
        .collect();
    for k in saem {
        if let Some(f) = m.fixos.remove(&k) {
            m.colliders
                .remove(f.colisor, &mut m.islands, &mut m.bodies, false);
        }
    }
    for (k, o) in declarados {
        let salto = o.salto.first().copied().unwrap_or(0.0);
        let mesmos = |a: &[f32], b: &[f32]| {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
        };
        let igual = m.fixos.get(k).is_some_and(|f| {
            f.forma == o.forma
                && f.atrito.to_bits() == o.atrito.to_bits()
                && mesmos(&f.saltos, &o.salto)
        });
        if igual {
            continue;
        }
        let varia = o.salto.iter().any(|x| x.to_bits() != salto.to_bits());
        if let Some(f) = m.fixos.remove(k) {
            m.colliders
                .remove(f.colisor, &mut m.islands, &mut m.bodies, false);
        }
        let mut c = fixo::colisor(o.forma, o.atrito, salto, FIXO | u128::from(*k));
        if varia {
            c.set_active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS);
        }
        let colisor = m.colliders.insert(c);
        m.fixos.insert(
            *k,
            Fixo {
                colisor,
                forma: o.forma,
                atrito: o.atrito,
                saltos: o.salto.clone(),
                varia,
            },
        );
    }
}

#[cfg(test)]
mod tests;

// ⛔ TEMPORÁRIO (a prova do §9.20): o relógio por etapa do passo — sai com a prova.
thread_local! {
    static RELOGIO: std::cell::RefCell<[f64; 12]> = const { std::cell::RefCell::new([0.0; 12]) };
}
fn marca(i: usize, t0: &mut std::time::Instant) {
    let agora = std::time::Instant::now();
    RELOGIO.with(|r| r.borrow_mut()[i] += (agora - *t0).as_secs_f64() * 1e3);
    *t0 = agora;
}
/// ⛔ TEMPORÁRIO: soma `ms` à etapa `i` (8..12: as de fora do mundo).
pub fn soma(i: usize, ms: f64) {
    RELOGIO.with(|r| r.borrow_mut()[i] += ms);
}
/// ⛔ TEMPORÁRIO: o relógio acumulado por etapa (specs · ids · fixos · mortas · escreve · step · rolar · lê), e zera.
/// ⛔ TEMPORÁRIO
pub fn relogio() -> [f64; 12] {
    RELOGIO.with(|r| std::mem::take(&mut *r.borrow_mut()))
}
