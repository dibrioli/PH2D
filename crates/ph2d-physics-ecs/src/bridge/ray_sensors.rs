//! **O RAIO persistente** (suplente #21) — um por [`RaySensor`], por TIQUE.
//!
//! # ⚠️ Por TIQUE, e não por dispatch — a mesma razão do canal de contactos
//!
//! Um quadro que deve três tiques anda o mundo três vezes, e uma parede que apareça e desapareça
//! dentro dele seria **invisível** a uma amostragem por quadro. É a frase que o
//! [`ContactEvent`](super::contacts::ContactEvent) já escreve (*«um toque mais curto que um tique
//! ainda grita»*), um nível ao lado.
//!
//! # ⚠️ A POSE sai do MUNDO quando há corpo, e do `Transform` quando não há
//!
//! Dentro do laço de tiques o `Transform` é do quadro ANTERIOR — o `readback` só corre no fim —,
//! logo um raio que o lesse nasceria onde o objecto estava, não onde ele está. *Uma sonda que mede
//! o estado anterior não fica calada: ela responde, e errado.*
//!
//! # ⛔ O que este módulo NÃO faz
//!
//! Ele **não** mantém uma lista própria de sinais. As duas listas que ele enche
//! ([`ray_enters`](PhysicsBridge::ray_enters) / [`ray_exits`](PhysicsBridge::ray_exits)) são
//! **canais**, como os do contacto e os do gatilho, e quem os transforma em sinal é o
//! [`signal_events`](PhysicsBridge::signal_events) — que é onde o `SignalTagFilter` já mora. *Uma
//! segunda lista seria uma segunda resposta a «o que aconteceu neste quadro».*
//!
//! # ⭐⭐⭐ E o DESENHO sai daqui, porque é aqui que o raio existe (W5)
//!
//! [`ray_marks`](PhysicsBridge::ray_marks) é publicado **dentro do laço que lança**, do mesmo
//! [`RaioEmMundo`] que alimentou o `cast_ray`. Derivá-lo do lado do desenho seria a segunda resposta
//! a *«onde este raio nasce?»*, e ela divergiria no primeiro dia em que alguém mexesse numa das duas
//! — com o sintoma a ser um overlay que **mente sobre o que o produto mede**, que é a única coisa
//! pior do que overlay nenhum. É a regra que o `scaled_shape` impôs ao contorno, a
//! `zone_force_world_at` à seta e o `player_view` aos sensores do personagem.
//!
//! ⚠️⚠️ **A marca leva a direcção NORMALIZADA, e é uma correcção e não arrumação.** O
//! [`ProbeShape::Ray`](super::player_view::ProbeShape::Ray) declara `dir` unitária porque quem desenha calcula `origem + dir·reach`; a
//! porta do motor normaliza **por dentro**, logo um `dir` autorado de `(1, 1)` com `reach = 3`
//! castaria 3 m e desenharia **4,24** — o overlay a mentir sobre o alcance exactamente para o
//! artista que está a afiná-lo.
//!
//! ⚠️ **Ele é publicado MESMO SEM ACERTO**, e é esse o argumento inteiro do overlay de sondas: *o
//! que se desenha é ONDE ELE OLHA, e a intensidade diz o que aconteceu*. Desenhar só o que acertou
//! deixaria invisível justamente o raio cujo alcance é curto demais.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, World};
use ph2d_physics::RigidBodyHandle;

use super::PhysicsBridge;
use super::player_view::{ProbeKind, ProbeMark};
use crate::{RayHit, RaySensor};

/// **A LINHA que um sensor olha, em MUNDO** — a porta única, com dois chamadores: o que **lança**
/// ([`PhysicsBridge::cast_ray_sensors`]) e o que só **desenha**
/// ([`PhysicsBridge::preview_ray_marks`]).
///
/// ⚠️ **Duas cópias desta resolução seriam duas respostas a «onde este raio está?»**, e a segunda
/// divergiria com o solver desarmado — que é precisamente o estado em que o artista afina um
/// alcance.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RaioEmMundo {
    /// Onde ele nasce, já com a pose do objecto aplicada.
    pub origem: [f32; 2],
    /// Para onde ele aponta — **unitária**, ver o cabeçalho do módulo.
    pub dir: [f32; 2],
    /// Até onde, em metros (o número autorado, coado pelas cercas abaixo).
    pub reach: f32,
    /// O corpo dono, a excluir do cast. `None` para um olho sem corpo.
    pub excluir: Option<RigidBodyHandle>,
}

impl PhysicsBridge {
    /// **Onde este sensor olha, AGORA** — a pose do objecto aplicada aos dois vectores locais.
    ///
    /// `None` quando não há como fazer um raio disto, e as três recusas têm razões diferentes:
    ///
    /// | recusa | porquê |
    /// |---|---|
    /// | sem pose | o objecto não está no mundo nem tem `Transform` — não há de onde partir |
    /// | direcção nula ou `NaN` | ⛔ **não se faz um vector unitário de um vector nulo**, e o
    ///   [`ProbeShape::Ray`](super::player_view::ProbeShape::Ray) exige-o |
    /// | `reach` negativo ou `NaN` | um `NaN` atravessa o `origem + dir·reach` e chega ao caminho
    ///   do Vello como um ponto que não existe |
    ///
    /// ⚠️ **As duas últimas coincidem HOJE com as que a porta do motor recusa
    /// ([`cast_ray`](ph2d_physics::PhysicsWorld::cast_ray)), e isso é um FACTO e não uma lei:** lá
    /// a razão é *«o parry responderia sobre um raio que não existe»* e aqui é *«a marca tem de ser
    /// desenhável»*. Duas razões, o mesmo conjunto — e no dia em que uma delas mudar, a outra não
    /// tem de mudar com ela.
    pub(super) fn raio_em_mundo(
        &mut self,
        world: &World,
        e: Entity,
        r: &RaySensor,
    ) -> Option<RaioEmMundo> {
        // A pose: do MUNDO se houver corpo (é a de AGORA), do `Transform` se não houver.
        let corpo = if self.bodies.contains_key(&e) {
            Some(e)
        } else {
            crate::owner_body(world, e)
        };
        let em_mundo = ph2d_ecs::world_transform(world, e);
        let (px, py, ang, excluir, lado) = match corpo
            .and_then(|c| self.bodies.get(&c).map(|b| (c, b)))
        {
            // A pose VIVA do corpo dono, mais o que separa o raio dele na árvore (zero se é o próprio).
            Some((c, b)) => {
                let p = self.world.body_pose(b.handle)?;
                let (rel, d_ang) = match (c == e, ph2d_ecs::world_transform(world, c), em_mundo) {
                    (false, Some(tc), Some(te)) => {
                        let (s, co) = (libm::sinf(-tc.rotation), libm::cosf(-tc.rotation));
                        let d = te.translation - tc.translation;
                        (
                            [d.x * co - d.y * s, d.x * s + d.y * co],
                            te.rotation - tc.rotation,
                        )
                    }
                    _ => ([0.0, 0.0], 0.0),
                };
                let a = p.rotation.angle();
                let (s, co) = (libm::sinf(a), libm::cosf(a));
                // O lado do personagem dono (a mesma fonte que a física usa), só com a opção ligada.
                let lado = if r.turns_with_facing {
                    self.player_state
                        .get(&c)
                        .map_or(1.0, |st| st.facing.signum())
                } else {
                    1.0
                };
                let rel = [rel[0] * lado, rel[1]];
                (
                    p.translation.x + rel[0] * co - rel[1] * s,
                    p.translation.y + rel[0] * s + rel[1] * co,
                    a + d_ang * lado,
                    Some(b.handle),
                    lado,
                )
            }
            None => {
                let t = em_mundo?;
                (t.translation.x, t.translation.y, t.rotation, None, 1.0)
            }
        };
        let (s, c) = (libm::sinf(ang), libm::cosf(ang));
        let v = r.dir_local();
        let v = ph2d_core::Vec2::new(v.x * lado, v.y);
        let d = [v.x * c - v.y * s, v.x * s + v.y * c];
        let n = libm::sqrtf(d[0] * d[0] + d[1] * d[1]);
        if !n.is_finite() || n <= f32::EPSILON || !r.reach.is_finite() || r.reach < 0.0 {
            return None;
        }
        Some(RaioEmMundo {
            origem: [px, py],
            dir: [d[0] / n, d[1] / n],
            reach: r.reach,
            excluir,
        })
    }

    /// **Lança um raio por sensor e diz o que cada um viu**, acumulando as arestas.
    ///
    /// ⚠️ **O corpo dono é EXCLUÍDO por omissão** (`ignore_body`), e o gate
    /// `the_caster_can_exclude_itself` do motor mede o que acontece sem isso: *«um personagem acha-se
    /// no chão para sempre»*. ⭐ Um sensor num objecto **sem corpo** não tem o que excluir — gate
    /// `um_raio_sem_corpo_nao_exclui_ninguem`.
    ///
    /// As opções da versão C (dono 15/11) correm DENTRO da consulta do motor
    /// ([`cast_ray_query`](ph2d_physics::PhysicsWorld::cast_ray_query)), por forma candidata:
    ///
    /// | opção | onde |
    /// |---|---|
    /// | `ignore_parts` | o predicado: o corpo da forma tem a mesma [`familia_raiz`](crate::familia_raiz) e não é o dono |
    /// | `only_see` sem `walls_block` | o predicado: quem não passa é transparente |
    /// | `only_see` com `walls_block` | DEPOIS: o primeiro acerto só conta se passa (a parede tapa) |
    /// | `hit_triggers` / `starts_inside` | o [`RayQuery`](ph2d_physics::RayQuery) |
    /// | `all_hits` | `cast_ray_query_all`; com `walls_block` a lista pára no primeiro que não passa |
    ///
    /// ⚠️ **O mapa `handle → entidade` vem de FORA**, como o do canal de contactos.
    pub(super) fn cast_ray_sensors(
        &mut self,
        sim: &mut SimWorld,
        by_handle: &BTreeMap<(u32, u32), Entity>,
    ) {
        let mut agora: BTreeMap<Entity, RayHit> = BTreeMap::new();
        // Os raios `all_hits` deste tique — vazio (sem alocar) no caminho comum.
        let mut todos: Vec<Entity> = Vec::new();
        let mut velho_todos = std::mem::take(&mut self.ray_hits_all);
        // ⚠️ **A lista descreve UM tique, logo ela é reconstruída e não acumulada** — a frase que o
        // `player_probes` já escreve. A capacidade do `Vec` fica.
        self.ray_marks.clear();
        {
            let world = sim.world_mut();
            let mut q = world.query::<(Entity, &RaySensor)>();
            // ⚠️ A lista sai da consulta ANTES do laço: o `cast_ray` empresta `self.world`, e a
            // consulta empresta o `World` do ECS.
            let sensores: Vec<(Entity, RaySensor)> = q.iter(world).map(|(e, r)| (e, *r)).collect();
            let world: &World = world;
            for (e, r) in sensores {
                let Some(raio) = self.raio_em_mundo(world, e, &r) else {
                    continue;
                };
                let dono = if self.bodies.contains_key(&e) {
                    Some(e)
                } else {
                    crate::owner_body(world, e)
                };
                let raiz = if r.ignore_parts {
                    crate::familia_raiz(world, e)
                } else {
                    None
                };
                let tree = &self.nav.arvore;
                let entidade = |c: Option<RigidBodyHandle>| {
                    c.and_then(|b| by_handle.get(&b.into_raw_parts()).copied())
                };
                let passa = |b: Option<Entity>| {
                    r.only_see.passes(
                        r.see_tag,
                        tree,
                        b.and_then(|b| world.get::<ph2d_ecs::tags::Tags>(b)),
                    )
                };
                let aceita = |c: Option<RigidBodyHandle>| {
                    let b = entidade(c);
                    if let (Some(raiz), Some(b)) = (raiz, b)
                        && Some(b) != dono
                        && crate::familia_raiz(world, b) == Some(raiz)
                    {
                        return false;
                    }
                    r.walls_block || passa(b)
                };
                let consulta = ph2d_physics::RayQuery {
                    exclude_body: if r.ignore_body { raio.excluir } else { None },
                    layer: r.layer,
                    hit_sensors: r.hit_triggers,
                    hit_from_inside: r.starts_inside,
                };
                let ent_de = |h: &ph2d_physics::CastHit| entidade(h.body);
                let para_ray_hit = |alvo: Entity, h: &ph2d_physics::CastHit| RayHit {
                    body: alvo,
                    distance: h.distance,
                    point: h.point,
                    normal: h.normal,
                };
                let parou: Option<f32>;
                // Com «See all hits» a linha vai até ao BLOQUEIO (ou ao alcance inteiro), com um tique em cada acerto.
                let mut fim = raio.reach;
                let mut mais = crate::MaisAcertos::default();
                if r.all_hits {
                    self.world.cast_ray_query_all(
                        raio.origem,
                        raio.dir,
                        raio.reach,
                        consulta,
                        &aceita,
                        &mut self.ray_scratch,
                    );
                    let mut lista = velho_todos.remove(&e).unwrap_or_default();
                    let anterior = std::mem::take(&mut lista);
                    let mut bloqueio = None;
                    for h in &self.ray_scratch {
                        let b = ent_de(h);
                        if !passa(b) {
                            // Só chega aqui com `walls_block` (sem ele o predicado já o tirou).
                            bloqueio = Some(h.distance);
                            break;
                        }
                        if let Some(b) = b
                            && !lista.iter().any(|x| x.body == b)
                        {
                            lista.push(para_ray_hit(b, h));
                        }
                    }
                    parou = lista.first().map(|h| h.distance);
                    fim = bloqueio.unwrap_or(raio.reach);
                    for (slot, h) in mais.d.iter_mut().zip(lista.iter().skip(1)) {
                        *slot = h.distance;
                        mais.n += 1;
                    }
                    // As ARESTAS por objecto: entra quem passa a estar na lista, sai quem deixa.
                    for h in &lista {
                        if !anterior.iter().any(|x| x.body == h.body) {
                            self.ray_enters.push((e, h.body));
                        }
                    }
                    for h in &anterior {
                        if !lista.iter().any(|x| x.body == h.body) {
                            self.ray_exits.push((e, h.body));
                        }
                    }
                    if let Some(h) = lista.first() {
                        agora.insert(e, *h);
                    }
                    self.ray_hits_all.insert(e, lista);
                    todos.push(e);
                } else {
                    let achou = self.world.cast_ray_query(
                        raio.origem,
                        raio.dir,
                        raio.reach,
                        consulta,
                        &aceita,
                    );
                    parou = achou.map(|h| h.distance);
                    if let Some(h) = achou {
                        // ⭐ Com `walls_block` a primeira matéria TAPA: só se vê se ela passa.
                        if let Some(alvo) = ent_de(&h).filter(|&b| passa(Some(b))) {
                            agora.insert(e, para_ray_hit(alvo, &h));
                        }
                    }
                }
                // ⭐ **A marca sai do CAST, acerte ele ou não** — ver o cabeçalho.
                self.ray_marks.push(
                    ProbeMark::ray(
                        ProbeKind::Sensor,
                        raio.origem,
                        raio.dir,
                        fim,
                        parou,
                        // ⛔ **Sem `skin`, e a ausência é a lei:** a origem é **autorada**.
                        0.0,
                    )
                    .com_mais_acertos(mais.como_lista())
                    .na_camada(r.layer),
                );
            }
        }
        // Um raio que deixou de ser `all_hits` (ou de existir) solta o que via.
        for (e, lista) in &velho_todos {
            for h in lista {
                self.ray_exits.push((*e, h.body));
            }
        }
        // As ARESTAS dos raios de um só alvo, contra o que cada um via no tique anterior.
        //
        // ⚠️ **Trocar de alvo é uma SAÍDA e uma ENTRADA**, não um silêncio.
        let fora = |e: &Entity| todos.contains(e) || velho_todos.contains_key(e);
        for (&e, h) in &agora {
            if fora(&e) {
                continue;
            }
            match self.ray_hits.get(&e) {
                Some(anterior) if anterior.body == h.body => {}
                Some(anterior) => {
                    self.ray_exits.push((e, anterior.body));
                    self.ray_enters.push((e, h.body));
                }
                None => self.ray_enters.push((e, h.body)),
            }
        }
        for (&e, anterior) in &self.ray_hits {
            if !agora.contains_key(&e) && !fora(&e) {
                self.ray_exits.push((e, anterior.body));
            }
        }
        self.ray_hits = agora;
    }

    /// **A geometria SEM a resposta** — o que se publica com o solver DESARMADO (o toggle Physics
    /// do transporte desmarcado).
    ///
    /// ⚠️⚠️ **Ele não casta, e é essa a honestidade inteira.** O `hold` já escreve a frase para os
    /// sensores do personagem: *o alcance de um sensor é uma propriedade do CORPO, que existe com o
    /// solver desligado — e o gesto de o afinar é precisamente encostar o corpo na parede **sem
    /// relógio***. Então em vez de apagar, re-derivar — a linha segue o corpo que a mão arrastou e
    /// todo estado sai [`ProbeState::Idle`](super::player_view::ProbeState::Idle), *porque a lei não
    /// correu*.
    ///
    /// ⛔ **Castar aqui seria pior do que não desenhar nada:** o mundo rapier assentou no
    /// `Transform` mas nenhum `step` correu, e uma resposta publicada nesse estado passaria pelos
    /// canais de aresta — um sinal de *«vi o herói»* emitido com a física desarmada.
    pub(super) fn preview_ray_marks(&mut self, sim: &SimWorld) {
        self.ray_marks.clear();
        let world = sim.world();
        // ⚠️ `try_query` e não `query`: aqui o mundo é **lido**, e o `hold` não o empresta mutável.
        // ⭐ Ele devolve `None` quando o componente é **desconhecido do mundo** — o que, para esta
        // pergunta, é a resposta certa (*«não há sensor nenhum»*) e não uma armadilha. ⛔ A
        // armadilha que a wave das Tags pagou é a outra: um `Option<&T>` na consulta faz o `None`
        // significar *«nunca vi este outro componente»*, e aqui só há um.
        let Some(mut q) = world.try_query::<(Entity, &RaySensor)>() else {
            return;
        };
        let sensores: Vec<(Entity, RaySensor)> = q.iter(world).map(|(e, r)| (e, *r)).collect();
        for (e, r) in sensores {
            let Some(raio) = self.raio_em_mundo(world, e, &r) else {
                continue;
            };
            self.ray_marks.push(
                ProbeMark::idle_ray(ProbeKind::Sensor, raio.origem, raio.dir, raio.reach, 0.0)
                    .na_camada(r.layer),
            );
        }
    }

    /// **Esquece o que os raios viram** — as duas descontinuidades chamam-na.
    ///
    /// ⛔⛔ **As ARESTAS têm de morrer aqui, e isto é uma CORRECÇÃO da W2:** elas são limpas no topo
    /// do `dispatch`, e o `hold` **não passa por lá** — ele é chamado *em vez* dele. Desarmar o
    /// Physics no quadro em que um raio acabou de ver alguém deixava a entrada de pé, e o dreno do
    /// shell re-emitia o sinal **em todo quadro, para sempre**. É a terceira metade da frase que o
    /// `hold` já escreve para o contacto e para o gatilho.
    pub(super) fn discard_ray_history(&mut self) {
        self.ray_hits.clear();
        self.ray_hits_all.clear();
        self.ray_enters.clear();
        self.ray_exits.clear();
    }

    /// **O que cada raio vê AGORA** — a leitura que o painel consome.
    #[must_use]
    pub fn ray_sensor_hits(&self) -> &BTreeMap<Entity, RayHit> {
        &self.ray_hits
    }

    /// **TODOS os acertos de um raio `all_hits`**, por distância — vazio para os outros (o mais
    /// perto deles é o [`ray_sensor_hits`](Self::ray_sensor_hits)).
    #[must_use]
    pub fn ray_sensor_hits_all(&self, e: Entity) -> &[RayHit] {
        self.ray_hits_all.get(&e).map_or(&[], Vec::as_slice)
    }

    /// **A LINHA que cada raio olhou** — o que o canvas desenha (W5).
    ///
    /// Mesma forma que [`player_probe_marks`](PhysicsBridge::player_probe_marks), e de propósito: o
    /// pintor é **um só**, e quem compõe as duas listas é a shell.
    #[must_use]
    pub fn ray_marks(&self) -> &[ProbeMark] {
        &self.ray_marks
    }

    /// Os raios que PASSARAM A ver alguém neste dispatch — `(quem olha, quem foi visto)`.
    #[must_use]
    pub fn ray_enters(&self) -> &[(Entity, Entity)] {
        &self.ray_enters
    }

    /// Os que DEIXARAM de ver — `(quem olha, quem deixou de ser visto)`.
    #[must_use]
    pub fn ray_exits(&self) -> &[(Entity, Entity)] {
        &self.ray_exits
    }
}
