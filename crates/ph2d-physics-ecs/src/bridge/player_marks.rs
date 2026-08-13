//! **O RELATO dos sensores** (`W-Probes`) — o que cada um olhou e o que
//! respondeu, para quem DESENHA.
//!
//! ⚠️ **Corte por RESPONSABILIDADE:** o irmão [`super::probes`] responde *"onde
//! este sensor olha, e o que ele viu"* — a geometria e os casts — e este
//! responde *"como isso se conta a quem pinta"*. Os dois crescem por motivos
//! diferentes: um por sensor novo, o outro por estado novo a distinguir.
//!
//! ⚠️ **Nada aqui DERIVA geometria.** Toda origem vem das MESMAS portas que
//! castaram (`ground_rays` · `wall_rays` · `corner_geom` · `headroom_offset` ·
//! `ledge_ray`); um segundo cálculo deste lado seria uma segunda resposta a
//! *"onde este raio nasce?"*, e ela divergiria no primeiro dia em que alguém
//! mexesse numa das duas.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::probes::{
    corner_geom, ground_rays, headroom_offset, ledge_offset, ledge_origin_rise, ledge_rays,
    wall_rays,
};
use super::*;
use crate::ProbeShape;

/// **A LEITURA** (`W-Probes`) — o que cada sensor olhou e o que respondeu, para
/// quem desenha.
///
/// ⚠️ **Chama as MESMAS portas que castaram**, e é essa a razão de ele viver
/// aqui e não do lado do overlay: as origens não são recalculadas, são pedidas.
///
/// # ⚠️ Um sensor DESARMADO não tem marca nenhuma
///
/// *"Inerte neste tique"* e *"esta capacidade está desligada"* são coisas
/// diferentes, e só a primeira é [`ProbeState::Idle`]. Desenhar seis raios
/// apagados à volta de todo personagem de toda cena onde o pulo de parede nunca
/// foi ligado seria ruído permanente — e a linha do painel já diz que a
/// capacidade está off. O que o `Idle` responde é a pergunta que a UI **não**
/// responde: *está armado, e mesmo assim não olhou agora*.
///
/// # ⚠️ Sem direção, o flanco é desenhado dos DOIS lados
///
/// O sensor lateral olha para onde o jogador empurra. Parado não há lado, e
/// escolher um seria desenhar um alcance no sítio errado; os dois apagados
/// dizem a verdade — *é este o alcance, de qualquer lado que empurres*.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_marks(
    out: &mut Vec<ProbeMark>,
    world: &ph2d_physics::PhysicsWorld,
    entity: Entity,
    handle: rapier2d_handle::Handle,
    cfg: &PlayerConfig,
    origin: [f32; 2],
    drive: f32,
    // ⚠️ **DOIS níveis, e o de fora é *a lei perguntou?*** — o mesmo formato dos
    // três irmãos abaixo (`Option<&T>`), e não uma redundância. `ray()` deriva o
    // estado do `hit`, então um `None` simples só sabe dizer *"perguntou e não
    // achou"*: a perna era a única sempre castada e por isso nunca precisou do
    // terceiro estado. O preview de um quadro PARADO precisa, e um downgrade
    // depois do facto seria um SEGUNDO lugar a decidir estado.
    ground: Option<[Option<f32>; MAX_WALL_SAMPLES]>,
    // O perfil e **a subida que ele usou** — o segundo não é derivável do
    // primeiro, e é ele que dá altura ao leque desenhado.
    corner: Option<(&CeilingProbe, f32)>,
    wall: Option<&WallProbe>,
    headroom: Option<&Headroom>,
    // ⚠️ **DOIS níveis, e o de fora é *a lei perguntou?*** — o mesmo formato da
    // perna, e pela mesma razão: este sensor devolve `None` quando **não há
    // beirada**, então um `Option` simples colapsaria *"não perguntei"* com
    // *"perguntei e não achei"* — que é exactamente a distinção que o `Idle`
    // existe para dar (*a capacidade está desligada* contra *o alcance é curto
    // demais*), e os dois vereditos são opostos.
    ledge: Option<Option<&ph2d_platformer::LedgeProbe>>,
) {
    // A PERNA — castada em todo tique, então ela nunca fica `Idle`.
    let (legs, ln) = ground_rays(world, handle, cfg, origin);
    for (i, g) in legs.iter().enumerate().take(ln) {
        out.push(match ground {
            // ⚠️ **Cada raio mostra o que ELE achou** — as distâncias vêm dos
            // casts que a lei fez, nunca de uma segunda consulta, e é isso que
            // torna o desenho um relato do que aconteceu: ver o pé da borda
            // aceso sobre uma fenda com o do meio apagado É o diagnóstico que a
            // perna em leque existe para dar.
            Some(hits) => {
                ProbeMark::ray(ProbeKind::Ground, g.origin, g.dir, g.reach, hits[i], g.skin)
            }
            None => ProbeMark::idle_ray(ProbeKind::Ground, g.origin, g.dir, g.reach, g.skin),
        });
    }

    // O FLANCO.
    if cfg.wall.armed() {
        if let Some(w) = wall {
            if let Some((rays, n)) = wall_rays(world, handle, cfg, w.side) {
                for (r, hit) in rays.iter().zip(w.hits.iter()).take(n) {
                    out.push(ProbeMark::ray(
                        ProbeKind::Wall,
                        r.origin,
                        r.dir,
                        r.reach,
                        hit.map(|h| h.distance),
                        r.skin,
                    ));
                }
            }
        } else {
            let both = [-1.0_f32, 1.0];
            let sides: &[f32] = if drive != 0.0 {
                std::slice::from_ref(&drive)
            } else {
                &both
            };
            for side in sides {
                if let Some((rays, n)) = wall_rays(world, handle, cfg, *side) {
                    for r in rays.into_iter().take(n) {
                        out.push(ProbeMark::idle_ray(
                            ProbeKind::Wall,
                            r.origin,
                            r.dir,
                            r.reach,
                            r.skin,
                        ));
                    }
                }
            }
        }
    }

    // A QUINA — o leque e as duas folgas laterais.
    if cfg.jump.corner_reach.is_finite() && cfg.jump.corner_reach > 0.0 {
        let rise = corner.map_or(0.0, |(_, r)| r);
        if let Some(gm) = corner_geom(world, handle, cfg, rise) {
            out.push(ProbeMark::profile(
                gm.top,
                gm.half_width,
                gm.reach,
                gm.rise,
                corner.map(|(p, _)| p.blocked),
                corner.map_or_else(
                    || odd_samples(cfg.jump.corner_samples, MAX_CORNER_SAMPLES),
                    |(p, _)| p.samples,
                ),
            ));
            let span = gm.half_width + gm.reach;
            for (i, dir) in [-1.0_f32, 1.0].iter().enumerate() {
                let from = [gm.top[0], gm.mid_y];
                // ⚠️ `side_clear` conta a partir da BORDA e satura no alcance;
                // o que se desenha conta a partir do CENTRO, que é de onde o
                // raio saiu. Somar a meia-largura é desfazer exactamente o
                // desconto que `probe_ceiling` fez.
                match corner {
                    Some((p, _)) if p.side_clear[i] < gm.reach => out.push(ProbeMark::ray(
                        ProbeKind::Side,
                        from,
                        [*dir, 0.0],
                        span,
                        Some(gm.half_width + p.side_clear[i]),
                        gm.half_width,
                    )),
                    Some(_) => {
                        out.push(ProbeMark::ray(
                            ProbeKind::Side,
                            from,
                            [*dir, 0.0],
                            span,
                            None,
                            gm.half_width,
                        ));
                    }
                    None => out.push(ProbeMark::idle_ray(
                        ProbeKind::Side,
                        from,
                        [*dir, 0.0],
                        span,
                        gm.half_width,
                    )),
                }
            }
        }
    }

    // O AGACHAR — a forma varrida, e não um raio (`W-ShapeCast`).
    if let Some(up) = headroom_offset(cfg) {
        out.push(ProbeMark::sweep(entity, up, headroom.map(|h| h.blocked)));
    }

    // A BEIRADA (`W-Ledge`) — e ela segue a MESMA regra do flanco: sem direção
    // é desenhada dos DOIS lados, porque escolher um seria mostrar um alcance no
    // sítio errado.
    if cfg.ledge.armed() {
        let seen = ledge.flatten();
        let sides: [f32; 2] = [-1.0, 1.0];
        let both = ledge.is_none() && drive == 0.0;
        for side in sides {
            if !both && seen.map_or(drive.signum(), |l| l.side) != side {
                continue;
            }
            let Some((rays, n)) = ledge_rays(world, handle, cfg, side) else {
                continue;
            };
            // ⚠️ **QUAL amostra achou**, e não *"o leque achou"*: é o mesmo
            // diagnóstico que o doc da perna em leque defende (o pé da borda
            // aceso sobre uma fenda com o do meio apagado). O afastamento
            // vencedor é recuperado do `across` que a lei publicou — a inversa
            // exacta da linha que o construiu —, então nada aqui re-decide o
            // que ela decidiu.
            let won_off = seen.and_then(|l| {
                let (mins, maxs) = world.body_aabb(handle)?;
                Some(l.across - (maxs[0] - mins[0]))
            });
            let span = if cfg.ledge.span.is_finite() {
                cfg.ledge.span.max(0.0)
            } else {
                0.0
            };
            for (i, r) in rays.iter().enumerate().take(n) {
                let hit = seen.filter(|_| {
                    won_off.is_some_and(|w| {
                        (ledge_offset(cfg.ledge.grab, span, n, i) - w).abs() <= 1.0e-4
                    })
                });
                out.push(match (ledge, hit) {
                    // ⚠️ **A distância vem do que a lei mediu**, re-derivada da
                    // grandeza que ela consome: `reach_y − lip_rise` é o
                    // `distance` do cast, e desenhá-lo de outra forma seria uma
                    // segunda resposta.
                    (Some(_), Some(l)) => ProbeMark::ray(
                        ProbeKind::Ledge,
                        r.origin,
                        r.dir,
                        r.reach,
                        Some(ledge_origin_rise(cfg) - l.lip_rise),
                        r.skin,
                    ),
                    // Perguntou e não achou: `Clear`, e não `Idle`.
                    (Some(_), None) => {
                        ProbeMark::ray(ProbeKind::Ledge, r.origin, r.dir, r.reach, None, r.skin)
                    }
                    _ => ProbeMark::idle_ray(ProbeKind::Ledge, r.origin, r.dir, r.reach, r.skin),
                });
            }
        }
    }
}

impl crate::PhysicsBridge {
    /// **A leitura de um quadro em que o relógio NÃO anda** (`W-Probes2`).
    ///
    /// # ⚠️ O defeito que isto fecha, com o número
    ///
    /// `drive_players` só corre no ramo que DÁ PASSO, e é ele que enche a
    /// leitura. Pausado (`Equal` → `settle`) ou com o toggle **Physics**
    /// desmarcado (`hold`), o artista arrastava o corpo e **os sensores ficavam
    /// para trás** — medido: corpo movido de `x = 2.000` para `x = 5.000`, a
    /// perna publicada continuava em **`x = 2.000`**, descrevendo o último tique
    /// simulado. E é justamente aí que se afina um alcance: o gesto é encostar o
    /// corpo na parede **com o relógio parado** e olhar até onde a marca chega.
    ///
    /// ⚠️ **Era decisão minha, escrita num gate** (*"um quadro pausado mostra a
    /// leitura do último tique — a mesma propriedade das cruzes de contato"*), e
    /// ela estava errada: uma cruz de contato descreve um EVENTO que aconteceu, e
    /// o alcance de um sensor é uma propriedade do CORPO, que existe parado.
    ///
    /// # ⚠️ Ela re-deriva a GEOMETRIA e não CASTA, de propósito
    ///
    /// Todo estado sai `Idle`, porque a lei não correu — e é isso que mantém o
    /// canal com um significado só. Castar aqui seria **responder a uma pergunta
    /// que ninguém fez**, e faria `Hit` querer dizer *"a lei achou"* com o
    /// relógio a andar e *"o overlay achou"* com ele parado. O que o artista
    /// precisa de ver é **até onde o sensor alcança**, e é isso que a marca
    /// desenha contra a parede.
    /// **A marca é do CORPO: ela pousa onde o corpo ESTÁ, não onde ele estava**
    /// (`W-ProbeAnchor`).
    ///
    /// # ⚠️ O defeito que isto fecha, com o número
    ///
    /// A leitura é gravada **antes** do `step` — tem de ser, porque é ela que a
    /// lei consome para decidir o tique — e o `readback` publica a pose **depois**.
    /// Então o overlay desenhava a cápsula no fim do tique e o leque de sensores
    /// no começo dele: medido pela porta do produto, um personagem a andar deixa
    /// os sensores **0,1000 m atrás** em regime (a distância que ele percorre num
    /// tique a 6 m/s), constante e para sempre — o *"drift dos gizmos"* que o
    /// Enio reportou com foto. Num arranque a 18 m/s são 0,3 m; parado é zero, e
    /// é por isso que nenhuma cena de repouso o mostrava.
    ///
    /// # ⚠️ Traduzir, e não re-castar
    ///
    /// Re-perguntar ao mundo depois do passo seria uma **segunda resposta** a
    /// *"o que este sensor viu neste tique?"* — e ela discordaria da que a lei
    /// usou, que é a única que produziu movimento. O que muda aqui é só a
    /// ÂNCORA: o leque inteiro é rígido em relação ao corpo, então deslocá-lo
    /// pelo que o corpo andou mantém `hit`, `reach` e `skin` exatamente como
    /// foram medidos, e põe o desenho no corpo a que ele pertence.
    ///
    /// ⚠️ **O `Sweep` não é tocado, e a assimetria é a prova:** ele já viaja
    /// como `(corpo, deslocamento)` e o overlay o resolve contra a pose viva —
    /// por isso ele **nunca** driftou. Deslocá-lo aqui somaria o movimento do
    /// corpo a um número que já é relativo a ele. Os irmãos `Ray`/`Profile`
    /// viajam em mundo e são estes que precisam da âncora.
    ///
    /// ⚠️ **DRENA as faixas**, então é idempotente e um caminho que não gravou
    /// nenhuma (o `preview`, que já deriva da pose viva) é um no-op.
    pub(crate) fn reanchor_player_probes(&mut self) {
        for (handle, from, to, cast_at) in self.player_probe_anchors.drain(..) {
            let Some(pose) = self.world.body_pose(handle) else {
                continue;
            };
            let d = [
                pose.translation.x - cast_at[0],
                pose.translation.y - cast_at[1],
            ];
            if d[0] == 0.0 && d[1] == 0.0 {
                continue;
            }
            for m in &mut self.player_probes[from..to] {
                match &mut m.shape {
                    ProbeShape::Ray { origin, .. } => {
                        origin[0] += d[0];
                        origin[1] += d[1];
                    }
                    ProbeShape::Profile { top, .. } => {
                        top[0] += d[0];
                        top[1] += d[1];
                    }
                    // Já é relativo ao corpo — ver o aviso acima.
                    ProbeShape::Sweep { .. } => {}
                }
            }
        }
    }

    pub(crate) fn preview_player_probes(&mut self, sim: &SimWorld) {
        self.player_probes.clear();
        // Ela deriva da pose VIVA, então não há âncora a resolver — e faixas de
        // um dispatch anterior descreveriam índices desta lista nova.
        //
        // ⚠️ **Higiene, e não uma defesa que carrega peso — medido:** a mutação
        // que apaga esta linha **sobrevive à suíte inteira**, porque as faixas
        // já não podem estar cheias aqui (o `reanchor` as DRENA e o
        // `drive_players` as limpa antes de as reencher). Ela fica pelo dia em
        // que uma rota nova publique marcas sem passar por nenhum dos dois.
        self.player_probe_anchors.clear();
        let world = sim.world();
        for (&entity, b) in self.bodies.iter() {
            let Some(cfg) = world.get::<PlatformPlayer>(entity) else {
                continue;
            };
            let Some(pose) = self.world.body_pose(b.handle) else {
                continue;
            };
            record_marks(
                &mut self.player_probes,
                &self.world,
                entity,
                b.handle,
                &cfg.config(),
                [pose.translation.x, pose.translation.y],
                // Sem relógio não há dedo: o flanco é publicado dos DOIS lados,
                // que é o que `record_marks` já faz sem direção.
                0.0,
                None,
                None,
                None,
                None,
                None,
            );
        }
    }
}
