//! O gesto de **arrastar os cantos da gaiola** do Envelope (ADR-0129, Fatia 1) — o lado do HOST.
//!
//! A parte pura (que canto, e até onde) mora em `ph2d_vec_envelope::{nearest_corner,
//! move_corner_convex}`; aqui é o adaptador fino que lê a seleção, o componente ECS [`VecEnvelope`]
//! (no **container**, ADR-0129 Fatia 3) e o cursor, e ESCREVE `corners` de volta no componente. É o
//! padrão do [`crate::blend_live`] (um gesto de Node que toca o ECS), **não** o do `PenTool`.
//!
//! O alvo do gesto é uma **entidade** (os bits do container), não um `VecPathId`: o container não tem
//! path. O host passa a seleção do gizmo (`hero.gizmo.selection`), que — pela regra
//! *seleciona-só-o-container* de [`crate::vec_selection`] — é o container quando um envelope está
//! selecionado. Um alvo que não é envelope (sprite, path comum, grupo) é ignorado, e o pen segue dono
//! do clique.
//!
//! A alça é PRÓPRIA e vive no modo Node (ADR-0129 §3.3): o gizmo de sprite não a toca. Um gizmo sobre
//! a geometria de mundo que o [`crate::envelope_live::recook`] reescreve a cada frame dobraria — a
//! lição de 5 tentativas revertidas do Blend (ADR-0128).
//!
//! # Undo sai de graça
//!
//! O arrasto roda com o botão pressionado, e `App::post_frame_undo` suprime passos enquanto
//! `held_button` está `Some`. Os N frames do arrasto não viram N passos; ao soltar, o [`VecEnvelope`]
//! alterado (que viaja no `WorldSnapshot`) vira **um** passo no diff global. Nada a instrumentar aqui.

use ph2d_ecs::{Entity, EnvelopeKind, SimWorld, VecEnvelope};
use ph2d_vec_envelope::CageEdges;
use ph2d_vec_render::{ENVELOPE_HANDLE_R_PX, EnvelopeCageView};
use ph2d_vec_scene::{VecScene, Xform};

/// A gaiola do container `bits` em coordenadas **LOCAIS do container** (como vive no componente):
/// cantos, controles de lado e qual dos dois mapas ela aplica. `None` se `bits` não é um envelope (ou
/// sumiu). Quem a leva ao MUNDO é [`container_world_xform`].
#[must_use]
fn cage_of(sim: &SimWorld, bits: u64) -> Option<([[f64; 2]; 4], CageEdges, EnvelopeKind)> {
    sim.world()
        .get::<VecEnvelope>(Entity::from_bits(bits))
        .map(|env| (env.corners, env.edges, env.kind))
}

/// O gesto que a gaiola do container `bits` aplica — `Perspective` quando `bits` não é um envelope
/// (nada a desenhar aceso). É o que o painel pergunta para acender o chip certo, pela MESMA leitura
/// que o [`drag`] usa para escolher o guard.
#[must_use]
pub(crate) fn kind_of(sim: &SimWorld, bits: u64) -> EnvelopeKind {
    cage_of(sim, bits).map_or(EnvelopeKind::Perspective, |(_, _, k)| k)
}

/// O preset que gerou a gaiola do container `bits` (ou `None` se ela é manual) + a força dele.
/// `None` no todo se `bits` não é um envelope.
///
/// O painel pergunta para acender o botão certo e mostrar o Bend; o dispatch pergunta para saber o
/// que re-carimbar quando só o Bend mudou. **A mesma leitura para os dois** — se o painel dissesse
/// "Arc ativo" e o dispatch achasse outro, arrastar o slider trocaria o preset debaixo do dedo.
#[must_use]
pub(crate) fn warp_of(sim: &SimWorld, bits: u64) -> Option<(Option<ph2d_ecs::EnvelopeWarp>, f64)> {
    sim.world()
        .get::<VecEnvelope>(Entity::from_bits(bits))
        .map(|env| (env.warp, env.bend))
}

/// Os controles de lado a OFERECER — `Some` só no gesto Mesh.
///
/// **Porta única** de *"esta gaiola tem alça de lado?"*: o hit-test, o arrasto e o desenho perguntam
/// aqui. Em Perspective o mapa ignora os controles, então oferecê-los seria alça morta — e se cada
/// sítio decidisse por conta própria, um deles ofereceria e outro não.
#[must_use]
fn offered_edges(edges: CageEdges, kind: EnvelopeKind) -> Option<CageEdges> {
    (kind == EnvelopeKind::Mesh).then_some(edges)
}

/// O afim LOCAL→MUNDO do CONTAINER `bits` — a MESMA pose que `vec_transform::build` publica
/// (ADR-0111), calculada por-entidade aqui para não depender do `VecXforms` do frame. É por ele que a
/// gaiola (cantos LOCAIS) se desenha e se hit-testa no MUNDO, e é essa pose que o gizmo do Select
/// move (Fatia 2). `None` se a entidade sumiu.
fn container_world_xform(sim: &SimWorld, bits: u64) -> Option<Xform> {
    let entity = Entity::from_bits(bits);
    if sim.world().get_entity(entity).is_err() {
        return None;
    }
    Some(crate::vec_transform::xform_of_transform(
        crate::vec_transform::world_transform(sim, entity),
    ))
}

/// Os 4 cantos LOCAIS levados ao MUNDO pela pose.
#[must_use]
fn to_world(local: [[f64; 2]; 4], xf: &Xform) -> [[f64; 2]; 4] {
    std::array::from_fn(|i| xf.apply(local[i]))
}

/// Os 8 controles LOCAIS levados ao MUNDO pela mesma pose.
#[must_use]
fn edges_to_world(local: CageEdges, xf: &Xform) -> CageEdges {
    std::array::from_fn(|i| std::array::from_fn(|j| xf.apply(local[i][j])))
}

/// **Pressão no modo Node:** se a entidade selecionada é um envelope e uma **alça** da gaiola está
/// sob o cursor, arma o arrasto (`*drag = Some((bits, alça))`) e devolve `true` — o host então PULA o
/// `PenTool`. Fora disso devolve `false` e o pen segue como hoje (seleção / edição de âncora).
///
/// As alças são os 4 cantos e — só no gesto Mesh ([`offered_edges`]) — os 8 controles de lado, num
/// espaço de índices único. Hit-test no MUNDO: a gaiola LOCAL sobe pela pose do container
/// ([`container_world_xform`]) e o cursor (mundo) é comparado a ela. `px_to_world` converte o raio da
/// bolinha (px, do renderer) para o alcance em mundo — a MESMA constante que o desenho usa, para o
/// dedo e a tela concordarem.
#[must_use]
pub(crate) fn press(
    sim: &mut SimWorld,
    scene: &VecScene,
    selected: Option<u64>,
    world_pt: [f64; 2],
    px_to_world: f64,
    drag: &mut Option<(u64, usize)>,
) -> bool {
    let Some(bits) = selected else { return false };
    let Some((corners, edges, kind)) = cage_of(sim, bits) else {
        return false;
    };
    let Some(xf) = container_world_xform(sim, bits) else {
        return false;
    };
    let radius = ENVELOPE_HANDLE_R_PX * px_to_world;
    if kind == EnvelopeKind::Pins {
        return press_pin(sim, bits, &xf, world_pt, radius, drag);
    }
    let world = to_world(corners, &xf);
    let world_edges = offered_edges(edges, kind).map(|e| edges_to_world(e, &xf));
    if let Some(handle) =
        ph2d_vec_envelope::nearest_handle(&world, world_edges.as_ref(), world_pt, radius)
    {
        *drag = Some((bits, handle));
        return true;
    }
    // Errou a alça, mas acertou a ARTE do envelope: o clique é consumido **sem armar nada**.
    //
    // ⚠️ Não é zelo — é o que impede o *"os pontos travam ao arrastar"* (Enio, 2026-07-18). A
    // geometria dos filhos é COZIDA: o `recook` a reescreve a cada frame a partir das fontes e da
    // gaiola. Deixar o pen agarrar uma âncora dela dá ao artista um ponto que **anda e volta** no
    // frame seguinte — medido: a âncora arrastada 2 unidades reverte ao original ao bit.
    //
    // É a MESMA regra que já governa a alça de raio numa Live Shape (ADR-0121): geometria que uma
    // relação viva possui não é editável à mão. Quem quiser os pontos de volta tem **Expand**.
    // Clique fora da arte continua a cair no pen — desselecionar segue funcionando.
    let on_art = hits_child_art(sim, scene, bits, world_pt, px_to_world);
    if on_art {
        crate::vec_overlay_diag::refused(
            "ancora de filho",
            "a geometria e' COZIDA (o recook a reescreve todo frame) -- use Expand",
        );
    }
    on_art
}

/// O cursor está sobre a arte de algum filho do envelope `bits`?
fn hits_child_art(
    sim: &SimWorld,
    scene: &VecScene,
    bits: u64,
    world_pt: [f64; 2],
    px_to_world: f64,
) -> bool {
    let hit_r = crate::vec_gizmo_view::stroke_hit_r_from(px_to_world);
    let p = [world_pt[0] as f32, world_pt[1] as f32];
    let Some(kids) = sim
        .world()
        .get::<ph2d_ecs::Children>(Entity::from_bits(bits))
        .map(|c| c.iter().copied().collect::<Vec<_>>())
    else {
        return false;
    };
    kids.into_iter()
        .any(|e| crate::vec_gizmo_view::contains_world(sim, scene, e, p, hit_r))
}

/// **A pressão no gesto Pinos** — e ela é de outra natureza: aqui o clique no VAZIO *cria*.
///
/// Nos gestos de gaiola as alças são fixas (4 cantos, 8 controles) e um clique fora delas pertence
/// ao pen. Um puppet warp não tem alça fixa: **pregar é o gesto**, como no Puppet Warp do Photoshop.
/// Então, no modo Node com Pinos ativo, o envelope toma o clique inteiro — pega o pino sob o cursor
/// ou prega um novo ali.
///
/// O pino nasce **em repouso** (`estava == foi`): pregar não move nada, e é o arrasto seguinte que
/// deforma. Um pino que deformasse ao nascer faria o artista perder a arte só por pregá-lo.
fn press_pin(
    sim: &mut SimWorld,
    bits: u64,
    xf: &Xform,
    world_pt: [f64; 2],
    radius: f64,
    drag: &mut Option<(u64, usize)>,
) -> bool {
    let Some(inv) = xf.inverse() else {
        return false;
    };
    let local_pt = inv.apply(world_pt);
    let entity = Entity::from_bits(bits);
    let Some(mut env) = sim.world_mut().get_mut::<VecEnvelope>(entity) else {
        return false;
    };
    // O hit é contra a posição MOVIDA (é onde a bolinha está desenhada) — perguntar pela de repouso
    // faria o dedo pegar o pino no lugar onde ele já não está. E é feito em MUNDO, com o mesmo raio
    // em píxeis que a gaiola usa: sob pose escalada, comparar em local encolheria o alvo.
    let r2 = radius * radius;
    let hit = env.pins.iter().position(|p| {
        let w = xf.apply(p[1]);
        (w[0] - world_pt[0]).powi(2) + (w[1] - world_pt[1]).powi(2) <= r2
    });
    let index = hit.unwrap_or_else(|| {
        env.pins.push([local_pt, local_pt]);
        env.pins.len() - 1
    });
    *drag = Some((bits, index));
    true
}

/// **Move durante o arrasto:** leva a alça agarrada para `world_pt`, mas só se a gaiola sobreviver ao
/// guard do gesto ([`ph2d_vec_envelope::move_handle`]: convexidade no Perspective · não-dobra no
/// Mesh). Recusado ⇒ a alça **para na fronteira** (a gaiola não muda neste frame). Devolve `true`
/// enquanto há um arrasto vivo — o host consome o Move —, tenha a alça andado ou não.
///
/// O cursor está em MUNDO e a gaiola vive em LOCAL do container: o ponto desce pela pose INVERSA
/// antes do `move_handle`, então mover uma alça sob pose girada/escalada segue o dedo. Convexidade e
/// orientação são invariantes a afim de determinante positivo, logo checá-las em local basta.
#[must_use]
pub(crate) fn drag(sim: &mut SimWorld, active: Option<(u64, usize)>, world_pt: [f64; 2]) -> bool {
    let Some((bits, handle)) = active else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    let (Some(xf), Some((corners, edges, kind))) =
        (container_world_xform(sim, bits), cage_of(sim, bits))
    else {
        return true; // arrasto vivo, mas a entidade/componente sumiu: consome e espera o release
    };
    let Some(inv) = xf.inverse() else {
        return true; // pose degenerada: nada a mover com sentido
    };
    let local_pt = inv.apply(world_pt);
    if kind == EnvelopeKind::Pins {
        return drag_pin(sim, entity, bits, handle, local_pt);
    }
    let mesh = kind == EnvelopeKind::Mesh;
    let moved = ph2d_vec_envelope::move_handle(corners, edges, mesh, handle, local_pt);
    if moved.is_none() {
        crate::vec_overlay_diag::refused(
            "alca da gaiola",
            &format!("mover a alca {handle} para {local_pt:?} quebraria o guard do gesto"),
        );
    }
    if let Some(next) = moved
        && let Some(mut env) = sim.world_mut().get_mut::<VecEnvelope>(entity)
    {
        env.corners = next.corners;
        env.edges = next.edges;
        // **A mão promove a gaiola a MANUAL** (ADR-0129 §4, "o preset vira promovível). Sem isto o
        // próximo toque no slider Bend re-carimbaria o preset por cima do que o artista acabou de
        // fazer — o preset e a mão seriam dois donos da mesma gaiola.
        env.warp = None;
    }
    true
}

/// A gaiola a desenhar neste frame, se a entidade selecionada é um envelope — já em MUNDO (a gaiola
/// LOCAL levada pela pose do container). A alça sob arrasto (se pertencer à seleção) sai marcada
/// `dragging` — a bolinha cheia. Os controles de lado só viajam no gesto Mesh, pela MESMA
/// [`offered_edges`] que o hit-test consulta: uma alça pintada é sempre uma alça viva.
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    selected: Option<u64>,
    active: Option<(u64, usize)>,
) -> Option<EnvelopeCageView> {
    let bits = selected?;
    let (corners, edges, kind) = cage_of(sim, bits)?;
    if kind == EnvelopeKind::Pins {
        // O puppet não tem gaiola. Devolver uma vazia desenharia uma moldura que o mapa ignora.
        return None;
    }
    let xf = container_world_xform(sim, bits)?;
    let dragging = active.filter(|(d, _)| *d == bits).map(|(_, c)| c);
    Some(EnvelopeCageView {
        corners: to_world(corners, &xf),
        edges: offered_edges(edges, kind).map(|e| edges_to_world(e, &xf)),
        dragging,
    })
}

/// **Arrasta um pino** para `local_pt`, mas só se a arte não dobrar ([`ph2d_vec_envelope::pins_fold`]).
///
/// Recusado ⇒ o pino **para na fronteira**, exatamente como o canto não-convexo e o controle que
/// dobraria o Coons. É esta recusa que mantém `break_cusp` em `None` honesto: o estado dobrado fica
/// inalcançável pela mão, então não há cúspide a partir — e um fold em vetor é um contorno
/// auto-interseccionado, não um bico que se aproxima melhor.
///
/// ⚠️ **O preço, registrado:** o artista não torce um pino além de ~90°. É limite do MÉTODO (o ADR-0129
/// mediu `det J` mudar de sinal aí), não do guard.
fn drag_pin(
    sim: &mut SimWorld,
    entity: Entity,
    bits: u64,
    index: usize,
    local_pt: [f64; 2],
) -> bool {
    let Some(domain) = crate::envelope_live::domain_of(sim, bits) else {
        return true;
    };
    let Some(mut env) = sim.world_mut().get_mut::<VecEnvelope>(entity) else {
        return true;
    };
    let Some(pin) = env.pins.get(index).copied() else {
        return true;
    };
    let mut next = env.pins.clone();
    next[index][1] = local_pt;
    if ph2d_vec_envelope::pins_fold(&next, domain.0, domain.1) {
        // O pino PARA na fronteira. É por construção — e é exatamente o que o artista lê como
        // "travou", então tem de ser dizível.
        crate::vec_overlay_diag::refused(
            "pino",
            &format!("mover o pino {index} para {local_pt:?} dobraria a arte"),
        );
        return true;
    }
    let _ = pin;
    env.pins = next;
    true
}

/// **Apaga todos os pinos** do envelope `bits`. `true` se havia algum.
///
/// É a única porta de remoção da Fatia E, e é assumido: apagar UM pino exige um gesto próprio
/// (Alt+clique) que compete com o "clicar no vazio prega", e essa disputa é decisão de UX, não
/// encanamento. Sem *nenhuma* porta, porém, um pino mal pregado seria permanente.
pub(crate) fn clear_pins(sim: &mut SimWorld, bits: u64) -> bool {
    let Some(mut env) = sim
        .world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
    else {
        return false;
    };
    if env.pins.is_empty() {
        return false;
    }
    env.pins.clear();
    true
}

/// Os pinos do container `bits` em MUNDO, para o desenho — `[repouso, movido]` por pino.
#[must_use]
pub(crate) fn pins_world(sim: &SimWorld, bits: u64) -> Vec<[[f64; 2]; 2]> {
    let (Some(env), Some(xf)) = (
        sim.world().get::<VecEnvelope>(Entity::from_bits(bits)),
        container_world_xform(sim, bits),
    ) else {
        return Vec::new();
    };
    env.pins
        .iter()
        .map(|p| [xf.apply(p[0]), xf.apply(p[1])])
        .collect()
}

/// **Troca o gesto** da gaiola do container `bits`. `true` se algo mudou.
///
/// Ir para **Perspective** re-emite os lados RETOS (`rest_edges`): em Perspective os lados *são*
/// retos por invariante, e deixar guardados os controles que o artista dobrou faria a troca de volta
/// para Mesh ressuscitar uma gaiola que o mapa nunca aplicou. Ir para **Mesh** não mexe em nada — os
/// controles guardados já descrevem os lados atuais (é para isso que o invariante existe).
pub(crate) fn set_kind(sim: &mut SimWorld, bits: u64, kind: EnvelopeKind) -> bool {
    let Some(mut env) = sim
        .world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
    else {
        return false;
    };
    if env.kind == kind {
        return false;
    }
    env.kind = kind;
    if kind == EnvelopeKind::Perspective {
        env.edges = ph2d_vec_envelope::rest_edges(&env.corners);
    }
    true
}

#[cfg(test)]
#[path = "envelope_gesture_tests.rs"]
mod tests;
