//! O `Transform` dos objetos Flip (ADR-0111 parity, espelho de [`crate::vec_transform`]).
//!
//! Desde o ADR-0114 cada objeto Flip é uma entidade ([`crate::flip_entities`]).
//! Aqui ela ganha **pose**: a geometria dos traços passa a ser LOCAL, e o afim que
//! a leva ao mundo é `parent_world_transform ∘ Transform` — a MESMA cadeia de um
//! sprite. Consequência: um objeto Flip é movido/girado/escalado pelo **gizmo de
//! sprite**, individualmente ou numa multi-seleção mista, sem gizmo próprio.
//!
//! Identidade ⇒ local é mundo. Todo objeto recém-desenhado nasce assim (a geometria
//! é escrita em coordenadas de mundo pela mão de desenho); o [`settle_origins`] põe
//! o pivô no centro dele assim que o gesto termina, e daí em diante o objeto tem a
//! pose que o gizmo lhe der.
//!
//! Reusa os helpers GENÉRICOS de `vec_transform` (`world_transform`/
//! `xform_of_transform`) — eles não tocam `VecScene`, só o `SimWorld`/`Transform`.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_flip::{FlipDoc, FlipObjectId, Pose};
use ph2d_vec_scene::Xform;

use crate::flip_entities::FlipEntityMap;
use crate::vec_transform::{world_transform, xform_of_transform};

/// O afim local→mundo do objeto de `entity` (a cadeia de pais inclusa). É o `model`
/// que o render pré-multiplica no `world_to_clip` para rasterizar a geometria LOCAL
/// na pose certa.
#[must_use]
pub(crate) fn object_xform(sim: &SimWorld, entity: Entity) -> Xform {
    xform_of_transform(world_transform(sim, entity))
}

/// **A POSE DA CHAVE** como afim (W7.2): o afim do quadro ([`ph2d_flip::FlipFrame::pose`]),
/// em espaço LOCAL do objeto. Translação até o gizmo de rotate/escala; afim daí em diante —
/// e como a `Pose` já carrega os 6 coeficientes no MESMO layout do `Xform`, isto é só uma
/// conversão de tipo.
///
/// A cadeia completa da arte de um quadro é `objeto ∘ pose_da_chave`: a geometria do
/// desenho é local, a pose a coloca dentro do objeto, e o `Transform` do objeto leva
/// tudo ao mundo. Um quadro na pose neutra devolve a identidade — o caminho comum não
/// paga nada.
#[must_use]
pub(crate) fn key_xform(pose: Pose) -> Xform {
    let c = pose.coeffs();
    Xform([
        f64::from(c[0]),
        f64::from(c[1]),
        f64::from(c[2]),
        f64::from(c[3]),
        f64::from(c[4]),
        f64::from(c[5]),
    ])
}

/// **A arte de um quadro → MUNDO**: `objeto ∘ pose_da_chave`. É o `model` que o render
/// dobra na câmera.
#[must_use]
pub(crate) fn art_to_world(object: &Xform, key_pose: Pose) -> Xform {
    key_xform(key_pose).then(object)
}

/// **A arte de um quadro → MUNDO, com o papel deslizado** (Shift & Trace): a arte entra
/// na pose autorada da chave, o `shift` de exibição desliza a folha POR CIMA, e o objeto
/// leva tudo ao mundo. Com shift identidade delega ao [`art_to_world`] — byte a byte o
/// caminho de sempre (é o pin de regressão do modo).
#[must_use]
pub(crate) fn art_to_world_traced(object: &Xform, key_pose: Pose, shift: Pose) -> Xform {
    if shift.is_identity() {
        return art_to_world(object, key_pose);
    }
    key_xform(key_pose).then(&key_xform(shift)).then(object)
}

/// **Um DELTA do espaço do OBJETO → o espaço da ARTE** (a parte LINEAR inversa da pose).
///
/// O gesto de mover (traço ou ponto) mede o delta no funil pose-free do objeto (a regra
/// W7.2: o funil pose-aware realimenta quando o alvo é a POSE). Quando o alvo é a
/// **GEOMETRIA** (arte exclusiva), o delta ainda precisa descer ao espaço do desenho —
/// e desde o W7.5 a pose pode ter rotação/escala, então a parte linear dela não se
/// cancela mais no delta (numa pose de translação pura isto é a identidade, byte a
/// byte). A translação da pose NÃO entra: delta é vetor.
///
/// Pose degenerada (det≈0) devolve o delta intacto — mover algo é melhor que travar.
#[must_use]
pub(crate) fn object_delta_to_art(pose: Pose, delta: ph2d_core::Vec2) -> ph2d_core::Vec2 {
    let c = pose.coeffs(); // apply = [a·x + c·y + tx, b·x + d·y + ty]
    let det = f64::from(c[0]) * f64::from(c[3]) - f64::from(c[2]) * f64::from(c[1]);
    if det.abs() < 1e-12 {
        return delta;
    }
    let (dx, dy) = (f64::from(delta.x), f64::from(delta.y));
    ph2d_core::Vec2::new(
        ((f64::from(c[3]) * dx - f64::from(c[2]) * dy) / det) as f32,
        ((-f64::from(c[1]) * dx + f64::from(c[0]) * dy) / det) as f32,
    )
}

/// **MUNDO → a arte de um quadro**: o inverso exato de [`art_to_world`]. É o que o funil
/// de entrada usa para levar o cursor ao espaço em que a geometria do desenho vive.
///
/// As duas existem como UM par, e as duas pontas do sistema chamam este par — não é
/// zelo: o render e a autoria já divergiram três vezes nesta linha (o balde, BUGS
/// #11/#14/#16), e cada vez o sintoma foi o mesmo *"funciona no zoom 1, erra no resto"*.
/// Enquanto quem PINTA e quem ESCREVE derivam a transform da mesma função, a
/// possibilidade some (`feedback_derived_coordinate_seed_must_match_sample`). O gate
/// `the_render_and_the_input_are_exact_inverses` prende o par.
#[must_use]
pub(crate) fn world_to_art(object: &Xform, key_pose: Pose) -> Xform {
    art_to_world(object, key_pose)
        .inverse()
        .unwrap_or(Xform::IDENTITY)
}

/// A pose (afim) da chave que está NA TELA agora, para o 1º objeto e a camada ativa
/// (fallback: o topo) — a MESMA amostragem do render (`pose_at_cycled`, W7.2).
///
/// Função livre porque tem DOIS chamadores com borrows diferentes:
/// `App::flip_active_pose` (o funil da autoria, `&self` inteiro) e o overlay dos
/// helpers do Gap Closure, que roda com `self.gfx` destruturado e só tem os campos —
/// duas resoluções de pose divergiriam exatamente onde o helper tem de sentar em cima
/// do desenho posado.
#[must_use]
pub(crate) fn active_pose(
    flip: &FlipDoc,
    active_layer: Option<ph2d_flip::LayerId>,
    playhead: &ph2d_core::Playhead,
) -> Pose {
    let Some(obj) = flip.objects().first() else {
        return Pose::IDENTITY;
    };
    let frame = obj.frame_at(playhead);
    active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))
        .and_then(|lid| obj.layer(lid))
        .map_or(Pose::IDENTITY, |l| l.pose_at_cycled(frame))
}

/// O afim de cada objeto Flip, uma vez por frame. Um objeto na identidade **não
/// entra no mapa** — o render trata o ausente como identidade e não paga lookup.
#[must_use]
pub(crate) fn build(sim: &SimWorld, map: &FlipEntityMap) -> Vec<(FlipObjectId, Xform)> {
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let x = object_xform(sim, e);
        if !x.is_identity() {
            out.push((id, x));
        }
    }
    out
}

/// Move a ORIGEM (o pivô) do objeto de `entity` para `target_world`, **sem mover a
/// arte**: a translação vai para lá e a geometria local recua o mesmo tanto. Igual
/// ao `vec_transform::move_origin_to`, mas o bake é sobre TODOS os desenhos do
/// objeto ([`ph2d_flip::FlipObject::bake_affine`]).
///
/// `false` se a entidade sumiu, se o objeto sumiu, ou se o afim é degenerado.
pub(crate) fn move_origin_to(
    sim: &mut SimWorld,
    doc: &mut FlipDoc,
    entity: Entity,
    object: FlipObjectId,
    target_world: [f32; 2],
) -> bool {
    if sim.world().get_entity(entity).is_err() {
        return false;
    }
    // A translação vive no espaço do PAI; a geometria, no espaço local (pós R·S).
    let parent = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let Some(parent_inv) = xform_of_transform(parent).inverse() else {
        return false;
    };
    let target_parent = parent_inv.apply([f64::from(target_world[0]), f64::from(target_world[1])]);
    let Some(local) = sim.world().get::<ph2d_ecs::Transform>(entity).copied() else {
        return false;
    };
    // Quanto a origem andou, no espaço do pai.
    let delta_parent = [
        target_parent[0] - f64::from(local.translation.x),
        target_parent[1] - f64::from(local.translation.y),
    ];
    // O mesmo deslocamento no espaço LOCAL da geometria: desfaz a rotação/escala
    // próprias (a translação não entra — delta é um vetor).
    let rs = xform_of_transform(ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(0.0, 0.0),
        ..local
    });
    let Some(rs_inv) = rs.inverse() else {
        return false;
    };
    let delta_local = rs_inv.apply_vec(delta_parent);
    let Some(obj) = doc.object_mut(object) else {
        return false;
    };
    // A geometria recua exatamente o que a origem avançou ⇒ a arte não se move.
    obj.bake_affine([1.0, 0.0, 0.0, 1.0, -delta_local[0], -delta_local[1]]);
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
        t.translation = ph2d_core::Vec2::new(target_parent[0] as f32, target_parent[1] as f32);
    }
    true
}

/// Põe a origem de cada objeto Flip recém-desenhado no **centro da bbox dele**.
///
/// Um objeto nasce com a geometria em coordenadas de mundo e a entidade na
/// identidade — o pivô cai no centro do mundo. Isto conserta isso assim que a arte
/// pára de crescer (`gesturing` = o objeto que a mão de desenho/borracha ainda está
/// escrevendo em MUNDO a cada frame; assentá-lo no meio somaria geometria +
/// `Transform` e deslocaria a arte do cursor).
///
/// Só toca quem está na **identidade e sem pai** — um objeto já movido tem a origem
/// que o usuário lhe deu. Idempotente: depois de centrado, o delta é zero.
pub(crate) fn settle_origins(
    sim: &mut SimWorld,
    doc: &mut FlipDoc,
    map: &FlipEntityMap,
    gesturing: Option<FlipObjectId>,
) {
    let pending: Vec<(FlipObjectId, Entity)> = map
        .iter()
        .filter(|(id, _)| Some(**id) != gesturing)
        .map(|(&id, &bits)| (id, Entity::from_bits(bits)))
        .filter(|&(_, e)| {
            sim.world().get_entity(e).is_ok()
                && sim.world().get::<ph2d_ecs::ChildOf>(e).is_none()
                && sim
                    .world()
                    .get::<ph2d_ecs::Transform>(e)
                    .is_some_and(|t| *t == ph2d_ecs::Transform::IDENTITY)
        })
        .collect();
    for (id, e) in pending {
        let Some((lo, hi)) = doc.object(id).and_then(|o| o.geometry_bbox()) else {
            continue;
        };
        let center = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        if center[0] == 0.0 && center[1] == 0.0 {
            continue; // já centrado — nada a fazer
        }
        move_origin_to(sim, doc, e, id, center);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{ChildOf, FlipObjectRef, Name, Transform};
    use ph2d_flip::{FlipStroke, Hold, KeyKind};

    /// Um doc com um objeto (1 camada, 1 desenho) cujo traço vai de `a` a `b`, e a
    /// entidade que o referencia (identidade). Devolve `(doc, sim, map, id, entity)`.
    fn doc_with_segment(
        a: [f32; 2],
        b: [f32; 2],
    ) -> (FlipDoc, SimWorld, FlipEntityMap, FlipObjectId, Entity) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("Obj");
        let obj = doc.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
        s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
        obj.drawing_mut(d).unwrap().strokes.push(s);

        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
            .id();
        let mut map = FlipEntityMap::new();
        map.insert(oid, e.to_bits());
        (doc, sim, map, oid, e)
    }

    /// O objeto na identidade não entra no mapa de afins (caminho comum: nada a pagar).
    #[test]
    fn an_untransformed_object_is_absent_from_the_map() {
        let (_doc, sim, map, _, _) = doc_with_segment([0.0, 0.0], [1.0, 1.0]);
        assert!(build(&sim, &map).is_empty());
    }

    /// Assentar põe o pivô no CENTRO da arte sem mover um pixel dela, e a bbox de
    /// MUNDO (afim ∘ local) não muda. Idempotente.
    #[test]
    fn settling_centers_the_pivot_without_moving_the_art() {
        // Traço de (10,20) a (30,40): centro (20,30).
        let (mut doc, mut sim, map, oid, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
        settle_origins(&mut sim, &mut doc, &map, None);

        let t = sim.world().get::<Transform>(e).copied().unwrap();
        assert!(
            (t.translation.x - 20.0).abs() < 1e-4 && (t.translation.y - 30.0).abs() < 1e-4,
            "pivô no centro: {:?}",
            t.translation
        );
        // A geometria recuou o mesmo tanto: bbox local centrada na origem.
        let (lo, hi) = doc.object(oid).unwrap().geometry_bbox().unwrap();
        assert!((lo[0] + hi[0]).abs() < 1e-4 && (lo[1] + hi[1]).abs() < 1e-4);
        // Afim ∘ local reconstrói o mundo: o canto local sobe para (30,40).
        let x = object_xform(&sim, e);
        let w = x.apply([f64::from(hi[0]), f64::from(hi[1])]);
        assert!(
            (w[0] - 30.0).abs() < 1e-4 && (w[1] - 40.0).abs() < 1e-4,
            "{w:?}"
        );

        // Idempotente.
        settle_origins(&mut sim, &mut doc, &map, None);
        let t2 = sim.world().get::<Transform>(e).copied().unwrap();
        assert_eq!(t2.translation, t.translation);
    }

    /// O objeto EM GESTO não é assentado — a origem pularia a cada amostra e a arte
    /// sairia deslocada do cursor (a mão escreve MUNDO a cada frame).
    #[test]
    fn a_gesturing_object_is_never_settled() {
        let (mut doc, mut sim, map, oid, e) = doc_with_segment([40.0, 40.0], [50.0, 50.0]);
        settle_origins(&mut sim, &mut doc, &map, Some(oid));
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "o gesto continua em mundo: identidade"
        );
        assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");
        // Terminado o gesto, assenta.
        settle_origins(&mut sim, &mut doc, &map, None);
        assert!((sim.world().get::<Transform>(e).unwrap().translation.x - 45.0).abs() < 1e-4);
    }

    /// Um objeto parentado NÃO é assentado (a origem é herança do pai, não nossa).
    #[test]
    fn a_parented_object_is_left_alone() {
        let (mut doc, mut sim, map, _oid, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
        let parent = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("G")))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(parent));
        settle_origins(&mut sim, &mut doc, &map, None);
        assert_eq!(
            sim.world().get::<Transform>(e).copied().unwrap(),
            Transform::IDENTITY,
            "parentado: a origem é do pai"
        );
    }

    /// 🔴 **O render e a autoria são inversos EXATOS** — o par que impede o bug que já
    /// matou o balde três vezes (BUGS #11/#14/#16, sempre o mesmo sintoma: *"funciona no
    /// zoom 1 e erra no resto"*).
    ///
    /// Quem PINTA a arte de um quadro usa `art_to_world`; quem ESCREVE nela (a caneta, o
    /// balde, a escultura, a seleção) usa `world_to_art`. Se um dos dois esquecer a POSE
    /// da chave, um ponto apontado na tela não volta ao mesmo lugar — e o traço cai
    /// deslocado pelo tanto do deslocamento, calado.
    ///
    /// Mutação que sangra: tirar o `key_xform` de qualquer uma das duas.
    #[test]
    fn the_render_and_the_input_are_exact_inverses() {
        // Um objeto REAL: girado, escalado e movido (o único jeito de um erro de unidade
        // aparecer — na identidade, tudo casa por acidente).
        let object = Xform([0.6, 0.8, -0.8, 0.6, 30.0, -12.0]); // rot ~53°, escala 1
        let pose = Pose::from_translation(ph2d_core::Vec2::new(100.0, -40.0));

        let to_world = art_to_world(&object, pose);
        let to_art = world_to_art(&object, pose);

        for p in [[0.0, 0.0], [10.0, 5.0], [-250.0, 33.0]] {
            let round = to_art.apply(to_world.apply(p));
            assert!(
                (round[0] - p[0]).abs() < 1e-9 && (round[1] - p[1]).abs() < 1e-9,
                "ida e volta nao fecham: {p:?} -> {round:?}"
            );
        }

        // E a pose TEM de estar lá: sem ela, a arte sai no lugar do objeto sem pose.
        let neutral = art_to_world(&object, Pose::IDENTITY);
        assert_ne!(
            to_world.apply([0.0, 0.0]),
            neutral.apply([0.0, 0.0]),
            "a pose da chave nao entrou no render — mover uma instancia nao moveria nada"
        );
    }

    /// 🔴 **O delta do gesto desce ao espaço da ARTE pela parte linear inversa da pose**
    /// (W8) — sob uma pose GIRADA, transladar geometria pelo delta cru do objeto moveria
    /// a arte na direção errada (o render aplica a pose por cima). Numa pose de
    /// translação pura a conversão é a identidade, byte a byte (o caminho de sempre).
    ///
    /// Mutação que sangra: `object_delta_to_art` devolver o delta intacto.
    #[test]
    fn a_move_delta_descends_through_the_pose_linear_inverse() {
        // Pose de translação pura: identidade no delta (o caso comum não paga nada).
        let t = Pose::from_translation(ph2d_core::Vec2::new(50.0, -3.0));
        let d = ph2d_core::Vec2::new(7.0, -2.0);
        assert_eq!(object_delta_to_art(t, d), d);

        // Pose girada 90° CCW (a=0, b=1, c=-1, d=0): um delta +x do OBJETO tem de virar
        // −y na ARTE — para que, aplicado pela pose, volte a ser +x na tela.
        let rot90 = Pose::from_coeffs([0.0, 1.0, -1.0, 0.0, 10.0, 20.0]);
        let da = object_delta_to_art(rot90, ph2d_core::Vec2::new(1.0, 0.0));
        assert!(
            (da.x - 0.0).abs() < 1e-6 && (da.y - (-1.0)).abs() < 1e-6,
            "delta nao desceu pela pose: {da:?}"
        );
        // A volta fecha: pose_linear · delta_art == delta_objeto.
        let back =
            rot90.apply(ph2d_core::Vec2::new(da.x, da.y)) - rot90.apply(ph2d_core::Vec2::ZERO);
        assert!((back.x - 1.0).abs() < 1e-6 && back.y.abs() < 1e-6);

        // Degenerada (escala 0): devolve o delta — mover é melhor que travar.
        let flat = Pose::from_coeffs([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(object_delta_to_art(flat, d), d);
    }

    /// **Pose neutra = o caminho de sempre, byte a byte.** Quem nunca moveu uma instância
    /// não pode pagar nada — nem um épsilon de afim a mais.
    #[test]
    fn a_neutral_pose_is_the_object_transform_untouched() {
        let object = Xform([2.0, 0.0, 0.0, 2.0, 5.0, 7.0]);
        assert_eq!(art_to_world(&object, Pose::IDENTITY), object);
    }

    /// 🔴 **O funil do MOVE tem de ser POSE-FREE, ou ele realimenta e o desenho treme**
    /// (smoke do Enio, 2026-07-14).
    ///
    /// O gesto de mover uma instância ESCREVE a pose da chave. Se o cursor for convertido
    /// pelo funil **pose-aware** (`world_to_art`, o que o desenho/escultura usam), cada
    /// amostra mede num referencial que a amostra anterior acabou de deslocar — e um
    /// arrasto de passo CONSTANTE na tela produz um delta que PULA a cada quadro. O funil
    /// **pose-free** (o inverso do objeto, sem a pose) quebra o laço: passo constante →
    /// delta constante.
    ///
    /// Este é um ESPELHO do laço de `flip_edit_canvas_move`, exercitando as funções REAIS
    /// de funil (`world_to_art` / `Xform::inverse`). **O que ele prova e o que não prova:**
    /// prova que a ESCOLHA do funil decide entre tremor e suavidade (mutação que sangra:
    /// fazer `world_to_art` ignorar a pose → o `run(true)` para de driftar e a asserção
    /// cai). **NÃO** prova que o gesto real usa o funil certo — esse wiring mora no
    /// `App`+GPU (`flip_active_world_to_object`), fora do alcance de um teste de unidade, e
    /// é o **smoke** que o cobre. É o mesmo limite dos outros seams de gesto deste módulo.
    #[test]
    fn the_move_funnel_must_be_pose_free_or_the_drag_trembles() {
        // Um objeto REAL (movido/escalado): na identidade o bug se esconde.
        let object = Xform([1.5, 0.0, 0.0, 1.5, 20.0, -8.0]);
        // Cursor andando em PASSO CONSTANTE no mundo (o dedo em velocidade uniforme).
        let world: [[f64; 2]; 4] = [[0.0, 0.0], [9.0, 0.0], [18.0, 0.0], [27.0, 0.0]];

        // Roda o laço de `flip_edit_canvas_move` (o move de INSTÂNCIA: escreve a pose).
        let run = |pose_aware: bool| -> Vec<f64> {
            let funnel = |pose: ph2d_core::Vec2| -> Xform {
                if pose_aware {
                    world_to_art(&object, Pose::from_translation(pose))
                } else {
                    object.inverse().unwrap()
                }
            };
            let mut pose = ph2d_core::Vec2::ZERO;
            let p0 = funnel(pose).apply(world[0]);
            let mut last = ph2d_core::Vec2::new(p0[0] as f32, p0[1] as f32);
            let mut deltas = Vec::new();
            for w in &world[1..] {
                let p = funnel(pose).apply(*w);
                let now = ph2d_core::Vec2::new(p[0] as f32, p[1] as f32);
                let d = now - last;
                deltas.push(f64::from(d.x));
                pose += d; // o move de instância escreve a pose (o feedback vive aqui)
                last = now;
            }
            deltas
        };

        let free = run(false);
        // Passo de mundo constante (9) sob o inverso do objeto (escala 1,5) = delta local
        // constante (6) a cada amostra. Nada de tremor.
        for d in &free {
            assert!(
                (d - 6.0).abs() < 1e-4,
                "pose-free deveria dar delta constante: {free:?}"
            );
        }
        // O pose-aware DRIFTA: a prova de que usar aquele funil aqui treme (e o motivo do fix).
        let aware = run(true);
        assert!(
            (aware[0] - aware[1]).abs() > 1.0 || (aware[1] - aware[2]).abs() > 1.0,
            "o funil pose-aware nao realimentou ({aware:?}) — entao nao havia tremor a corrigir?"
        );
    }
}
