//! Os gates do A17 — **a raiz com a POSIÇÃO na timeline**: a track muda para o esqueleto, e o
//! mundo, o repouso, o ficheiro e os dois undos continuam a dar a mesma pose.

use super::tests::{mundo, osso};
use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_timeline::{
    PLACE_PROPS, PropKind as P, TimelineIntent as I, apply_from_doc, apply_intent,
};
use std::collections::BTreeMap;

/// Os instantes amostrados: as três keys, entre elas e depois da última.
const TEMPOS: [f64; 6] = [0.0, 0.4, 1.0, 1.7, 2.5, 3.1];

struct Cena {
    sim: SimWorld,
    tl: TimelineState,
    /// Raiz, filho, neto.
    ossos: [Entity; 3],
    /// O grupo posado onde a raiz mora (`k` ímpar).
    pai: Option<Entity>,
}

/// A cena `k`, determinística: raiz (dentro de um grupo com escala e skew se `k` é ímpar) com dois
/// filhos; a posição em keys X/Y, ou numa trajectória (`Position`) se `k % 3 == 2`; a rotação da
/// raiz keyada (fica no osso).
fn cena(k: u64) -> Cena {
    let mut rng = 0x9E37_79B9_7F4A_7C15_u64 ^ k.wrapping_mul(0xA24B_AED4_963E_E407);
    let mut f = |lo: f32, hi: f32| {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        lo + (hi - lo) * ((rng >> 40) as f32 / (1u64 << 24) as f32)
    };
    let mut sim = SimWorld::default();
    let pai = (k % 2 == 1).then(|| {
        let t = Transform {
            translation: Vec2::new(f(-900.0, 900.0), f(-900.0, 900.0)),
            rotation: f(-3.2, 3.2),
            scale: Vec2::new(f(0.2, 3.0), f(0.2, 3.0)),
            skew_x: f(-0.5, 0.5),
            skew_y: f(-0.5, 0.5),
        };
        sim.world_mut().spawn(t).id()
    });
    let raiz = osso(&mut sim, [f(-2e3, 2e3), f(-2e3, 2e3)], f(-3.2, 3.2), pai);
    {
        let mut t = sim.world_mut().get_mut::<Transform>(raiz).expect("raiz");
        t.scale = Vec2::new(f(0.3, 2.5), f(0.3, 2.5));
        t.skew_x = if k % 4 == 3 { f(-0.4, 0.4) } else { 0.0 };
    }
    let o2 = osso(
        &mut sim,
        [f(1.0, 40.0), f(-5.0, 5.0)],
        f(-3.2, 3.2),
        Some(raiz),
    );
    let o3 = osso(
        &mut sim,
        [f(1.0, 40.0), f(-5.0, 5.0)],
        f(-3.2, 3.2),
        Some(o2),
    );
    let mut tl = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let entity = raiz.to_bits();
    for s in [0.0, 1.0, 2.5] {
        let t = RationalTime::from_seconds(s);
        let key = |tl: &mut TimelineState, ph: &mut ph2d_core::Playhead, prop, v| {
            let value = AnimValue::Float(v);
            let interp = Interp::Linear;
            apply_intent(
                tl,
                ph,
                I::AddKey {
                    entity,
                    prop,
                    t,
                    value,
                    interp,
                },
            );
        };
        if k % 3 == 2 {
            tl.doc.key_the_path(entity, t, [f(-2e3, 2e3), f(-2e3, 2e3)]);
        } else {
            key(&mut tl, &mut ph, P::TranslationX, f(-2e3, 2e3));
            key(&mut tl, &mut ph, P::TranslationY, f(-2e3, 2e3));
        }
        key(&mut tl, &mut ph, P::Rotation, f(-3.2, 3.2));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    Cena {
        sim,
        tl,
        ossos: [raiz, o2, o3],
        pai,
    }
}

/// Um quadro da shell, na ordem dela: o apply, o `upkeep` (cura, volta a casa, purga), a porta.
fn quadro(c: &mut Cena, t: f64) -> Vec<u64> {
    apply_from_doc(c.sim.world_mut(), &mut c.tl.doc, t);
    ph2d_timeline_persist::upkeep(&mut c.tl, c.sim.world_mut());
    adopt_loose_roots(&mut c.sim, &mut c.tl)
}

fn id(sim: &SimWorld, e: Entity) -> u64 {
    sim.world().get::<StableId>(e).expect("id").0
}

fn por_id(sim: &SimWorld, id: u64) -> Entity {
    let w = sim.world();
    w.iter_entities()
        .find(|er| er.get::<StableId>().is_some_and(|s| s.0 == id))
        .map(|er| er.id())
        .expect("a identidade não está no mundo")
}

/// ⭐ **A fotografia** — o que o undo global guarda e o ficheiro grava: o mundo por `StableId`.
#[derive(Clone)]
struct Linha {
    id: StableId,
    t: Option<Transform>,
    osso: Option<Bone>,
    repouso: Option<BoneRest>,
    ordem: Option<RootOrder>,
    nome: Option<Name>,
    esqueleto: bool,
    pai: Option<StableId>,
}

fn fotografa(sim: &SimWorld) -> Vec<Linha> {
    let w = sim.world();
    w.iter_entities()
        .filter_map(|er| {
            Some(Linha {
                id: *er.get::<StableId>()?,
                t: er.get::<Transform>().copied(),
                osso: er.get::<Bone>().copied(),
                repouso: er.get::<BoneRest>().copied(),
                ordem: er.get::<RootOrder>().copied(),
                nome: er.get::<Name>().cloned(),
                esqueleto: er.contains::<Skeleton>(),
                pai: er
                    .get::<ChildOf>()
                    .and_then(|c| w.get::<StableId>(c.parent()).copied()),
            })
        })
        .collect()
}

/// Repõe a fotografia como o restore do undo: despawna o que tem identidade e respawna com bits
/// NOVOS no MESMO mundo (os bits velhos ficam mortos, nunca de outro objecto).
fn repoe(sim: &mut SimWorld, foto: &[Linha]) {
    let vivos: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.contains::<StableId>())
        .map(|er| er.id())
        .collect();
    for e in vivos {
        if sim.world().get_entity(e).is_ok() {
            sim.world_mut().despawn(e);
        }
    }
    let mut mapa = BTreeMap::new();
    for l in foto {
        let mut e = sim.world_mut().spawn(l.id);
        if let Some(t) = l.t {
            e.insert(t);
        }
        if let Some(b) = l.osso {
            e.insert(b);
        }
        if let Some(r) = l.repouso {
            e.insert(r);
        }
        if let Some(o) = l.ordem {
            e.insert(o);
        }
        if let Some(n) = l.nome.clone() {
            e.insert(n);
        }
        if l.esqueleto {
            e.insert(Skeleton);
        }
        mapa.insert(l.id.0, e.id());
    }
    for l in foto {
        if let Some(p) = l.pai {
            let e = mapa[&l.id.0];
            sim.world_mut().entity_mut(e).insert(ChildOf(mapa[&p.0]));
        }
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
}

/// As poses de mundo dos três ossos, por identidade.
fn poses(c: &Cena, ids: &[u64; 3]) -> [[u32; 7]; 3] {
    ids.map(|i| mundo(&c.sim, por_id(&c.sim, i)))
}

fn lugar_em(tl: &TimelineState, e: u64) -> usize {
    tl.doc
        .bindings()
        .iter()
        .filter(|b| b.entity == e && PLACE_PROPS.contains(&b.prop))
        .count()
}

/// ⭐⭐⭐ GATE 1 (A17) — **a raiz keyada entrega a posição ao esqueleto e o mundo fica AO BIT** —
/// 600 cenas (keys X/Y e trajectória, de topo e dentro de grupos com escala e skew), seis
/// instantes cada. O esqueleto nasce em `L` (a pose do quadro), a raiz em `0`, as tracks da posição
/// nomeiam o esqueleto e a da rotação fica no osso. Vermelho contra a lei de antes (esqueleto na
/// identidade, tracks no osso): a 1.ª asserção.
#[test]
fn a_keyed_root_hands_its_place_to_the_skeleton_and_the_world_stays_to_the_bit() {
    for k in 0..600 {
        let mut antes = cena(k);
        let mut depois = cena(k);
        apply_from_doc(antes.sim.world_mut(), &mut antes.tl.doc, 0.4);
        apply_from_doc(depois.sim.world_mut(), &mut depois.tl.doc, 0.4);
        let raiz = depois.ossos[0];
        let l = depois
            .sim
            .world()
            .get::<Transform>(raiz)
            .expect("r")
            .translation;
        let esq = Entity::from_bits(adopt_loose_roots(&mut depois.sim, &mut depois.tl)[0]);
        let w = depois.sim.world();
        assert_eq!(
            w.get::<Transform>(esq).expect("esq").translation,
            l,
            "esqueleto fora da raiz ({k})"
        );
        assert_eq!(
            w.get::<Transform>(raiz).expect("r").translation,
            Vec2::new(0.0, 0.0),
            "({k})"
        );
        assert_eq!(
            lugar_em(&depois.tl, raiz.to_bits()),
            0,
            "a posição ficou no osso ({k})"
        );
        assert_eq!(
            lugar_em(&depois.tl, esq.to_bits()),
            if k % 3 == 2 { 1 } else { 2 },
            "({k})"
        );
        assert!(
            depois
                .tl
                .doc
                .binding_for(raiz.to_bits(), P::Rotation)
                .is_some(),
            "a rotação saiu do osso"
        );
        let mundos = |c: &Cena| c.ossos.map(|e| mundo(&c.sim, e));
        assert_eq!(
            mundos(&depois),
            mundos(&antes),
            "a adopção mexeu na pose ({k})"
        );
        for t in TEMPOS {
            apply_from_doc(antes.sim.world_mut(), &mut antes.tl.doc, t);
            apply_from_doc(depois.sim.world_mut(), &mut depois.tl.doc, t);
            assert_eq!(
                mundos(&depois),
                mundos(&antes),
                "a pose saiu do bit ({k}, t = {t})"
            );
        }
    }
}

/// ⭐⭐⭐ GATE 2 (A17) — **Repor depois da mudança deixa o osso onde a timeline o põe.** O repouso
/// guardado está NOUTRO sítio (`R ≠ L`, o caso comum de uma raiz animada). A semântica: o repouso da
/// posição foi com a track (a raiz repousa em `0` no esqueleto), então *Repor* devolve a direcção, a
/// escala e o skew do repouso — os mesmos bits de antes — e NÃO afasta o osso do lugar onde estava
/// (antes da adopção, Repor levava-o ao `R` guardado, e o apply seguinte trazia-o de volta à key).
/// E Repor seguido do apply dá, em cada instante, o mesmo mundo AO BIT que antes da adopção.
#[test]
fn reset_to_rest_after_the_move_leaves_the_bone_where_the_timeline_puts_it() {
    use ph2d_skeleton_live::pose_de_repouso::repor;
    for k in 0..300 {
        let mut c = [cena(k), cena(k)];
        for c in &mut c {
            let raiz = c.ossos[0];
            let mut r = BoneRest::de(&Transform::IDENTITY);
            r.translation = [k as f32 * 3.5 - 400.0, 17.25];
            r.rotation = 0.25 + k as f32 * 0.01;
            r.scale = [1.5, 0.75];
            r.skew = [0.125, 0.0];
            c.sim.world_mut().entity_mut(raiz).insert(r);
            apply_from_doc(c.sim.world_mut(), &mut c.tl.doc, 0.4);
        }
        let [mut antes, mut depois] = c;
        let raiz = depois.ossos[0];
        adopt_loose_roots(&mut depois.sim, &mut depois.tl);
        let onde = mundo(&depois.sim, raiz);
        repor(&mut antes.sim, raiz);
        repor(&mut depois.sim, raiz);
        let (a, d) = (mundo(&antes.sim, raiz), mundo(&depois.sim, raiz));
        assert_eq!(
            d[..2],
            onde[..2],
            "Repor afastou o osso do lugar dele ({k})"
        );
        assert_eq!(
            d[2..],
            a[2..],
            "Repor não devolveu a direcção, a escala e o skew ({k})"
        );
        for t in TEMPOS {
            apply_from_doc(antes.sim.world_mut(), &mut antes.tl.doc, t);
            apply_from_doc(depois.sim.world_mut(), &mut depois.tl.doc, t);
            let mundos = |c: &Cena| c.ossos.map(|e| mundo(&c.sim, e));
            assert_eq!(
                mundos(&depois),
                mundos(&antes),
                "Repor + apply saiu do bit ({k}, {t})"
            );
        }
    }
}

/// ⭐⭐ GATE 3 (A17) — **a track movida atravessa o ficheiro do projecto e reencontra o esqueleto**
/// (a binding grava o `StableId` dele; o load destaca tudo e o `upkeep` recola). O registo de
/// sessão (`moved`) não viaja. Controlo: a pose depois do load é a de antes, ao bit.
#[test]
fn the_moved_place_crosses_the_project_file_and_finds_the_skeleton() {
    for k in [3, 4, 5] {
        let mut c = cena(k);
        let ids = c.ossos.map(|e| id(&c.sim, e));
        let esq = Entity::from_bits(quadro(&mut c, 0.4)[0]);
        let esq_id = id(&c.sim, esq);
        assert!(
            lugar_em(&c.tl, esq.to_bits()) > 0,
            "controlo: a track mudou ({k})"
        );
        let bytes = ph2d_timeline_persist::serialize(&mut c.tl, c.sim.world_mut()).expect("grava");
        let tl = ph2d_timeline_persist::install_from_project(&bytes).expect("carrega");
        let mut sim = SimWorld::default();
        repoe(&mut sim, &fotografa(&c.sim));
        let mut aberto = Cena {
            sim,
            tl,
            ossos: c.ossos,
            pai: None,
        };
        assert!(
            quadro(&mut aberto, 0.4).is_empty(),
            "o load criou outro esqueleto ({k})"
        );
        let esq2 = por_id(&aberto.sim, esq_id).to_bits();
        assert_eq!(
            lugar_em(&aberto.tl, esq2),
            lugar_em(&c.tl, esq.to_bits()),
            "({k})"
        );
        assert!(
            aberto.tl.doc.bindings().iter().all(|b| b.moved.is_none()),
            "o registo gravou-se"
        );
        for t in TEMPOS {
            apply_from_doc(c.sim.world_mut(), &mut c.tl.doc, t);
            apply_from_doc(aberto.sim.world_mut(), &mut aberto.tl.doc, t);
            assert_eq!(
                poses(&aberto, &ids),
                poses(&c, &ids),
                "o load mudou a pose ({k}, {t})"
            );
        }
    }
}

/// A cena `k` com a raiz já dentro de um esqueleto `S1` na identidade e a posição keyada no OSSO
/// — o estado de antes de o artista a arrastar para fora.
fn cena_num_esqueleto(k: u64) -> Cena {
    let mut c = cena(k);
    let raiz = c.ossos[0];
    let mut s1 = c.sim.world_mut().spawn((Transform::IDENTITY, Skeleton));
    match c.pai {
        Some(p) => s1.insert(ChildOf(p)),
        None => s1.insert(RootOrder(3)),
    };
    let s1 = s1.id();
    let mut r = c.sim.world_mut().entity_mut(raiz);
    r.remove::<RootOrder>();
    r.insert(ChildOf(s1));
    ph2d_ecs::assign_missing_stable_ids(c.sim.world_mut());
    assert!(
        quadro(&mut c, 0.4).is_empty(),
        "controlo: a raiz tem esqueleto"
    );
    c
}

/// ⭐⭐⭐ GATE 4 (A17) — **o undo GLOBAL atravessa a mudança de dono** (ele repõe o MUNDO e não o
/// documento). O artista arrasta a raiz keyada para fora do esqueleto → a porta adopta e muda a
/// track → Ctrl+Z: o esqueleto novo desaparece, a track volta ao osso (sem a volta a casa a purga
/// apagava-a) → Ctrl+Shift+Z: o esqueleto volta e a track com ele. Em cada estado a pose de cada
/// instante é a da cena que nunca foi arrastada, ao bit.
#[test]
fn a_global_undo_across_the_move_gives_the_place_back_and_the_redo_takes_it_again() {
    for k in 0..6 {
        let mut c = cena_num_esqueleto(k);
        let mut referencia = cena_num_esqueleto(k);
        let ids = c.ossos.map(|e| id(&c.sim, e));
        let pre = fotografa(&c.sim);
        let raiz = c.ossos[0];
        let mut r = c.sim.world_mut().entity_mut(raiz);
        r.remove::<ChildOf>();
        match c.pai {
            Some(p) => r.insert(ChildOf(p)),
            None => r.insert(RootOrder(3)),
        };
        let esq = Entity::from_bits(quadro(&mut c, 0.4)[0]);
        let esq_id = id(&c.sim, esq);
        assert!(
            lugar_em(&c.tl, esq.to_bits()) > 0,
            "a track não mudou ({k})"
        );
        let n = c.tl.doc.bindings().len();
        let pos = fotografa(&c.sim);
        let confere = |c: &mut Cena, referencia: &mut Cena, quando: &str| {
            for t in TEMPOS {
                apply_from_doc(c.sim.world_mut(), &mut c.tl.doc, t);
                apply_from_doc(referencia.sim.world_mut(), &mut referencia.tl.doc, t);
                let r = poses(referencia, &ids);
                assert_eq!(poses(c, &ids), r, "{quando}: a pose saiu do bit ({k}, {t})");
            }
        };
        repoe(&mut c.sim, &pre);
        assert!(
            quadro(&mut c, 0.4).is_empty(),
            "o undo adoptou de novo ({k})"
        );
        assert_eq!(
            c.tl.doc.bindings().len(),
            n,
            "o undo purgou a track da posição ({k})"
        );
        let raiz2 = por_id(&c.sim, ids[0]).to_bits();
        assert!(
            lugar_em(&c.tl, raiz2) > 0,
            "a track não voltou ao osso ({k})"
        );
        confere(&mut c, &mut referencia, "undo");
        repoe(&mut c.sim, &pos);
        assert!(
            quadro(&mut c, 0.4).is_empty(),
            "o redo adoptou de novo ({k})"
        );
        let esq2 = por_id(&c.sim, esq_id).to_bits();
        assert!(
            lugar_em(&c.tl, esq2) > 0,
            "a track não voltou ao esqueleto ({k})"
        );
        confere(&mut c, &mut referencia, "redo");
    }
}

/// ⭐⭐ GATE 5 (A17) — **o Ctrl+Z da TIMELINE nunca devolve a posição ao osso**: a mudança de dono
/// é de identidade, então reescreve também cada fotografia do histórico (as keys foram postas
/// ANTES da adopção). Sem isso o 1.º Ctrl+Z punha a track no osso, e o osso em `L + key`.
#[test]
fn the_timeline_undo_never_hands_the_place_back_to_the_bone() {
    let mut c = cena(4);
    assert!(c.tl.history.can_undo(), "controlo: as keys deixaram passos");
    let raiz = c.ossos[0].to_bits();
    let esq = quadro(&mut c, 0.4)[0];
    let mut passos = 0;
    while c.tl.undo() {
        passos += 1;
        assert_eq!(
            lugar_em(&c.tl, raiz),
            0,
            "o passo {passos} devolveu a posição ao osso"
        );
    }
    while c.tl.redo() {
        assert_eq!(
            lugar_em(&c.tl, raiz),
            0,
            "o redo devolveu a posição ao osso"
        );
    }
    assert_eq!(
        lugar_em(&c.tl, esq),
        2,
        "controlo: no fim a track está no esqueleto"
    );
}

/// ⭐⭐ GATE 6 (A17) — **onde mudar o dono não é exacto, fica a lei de antes** (esqueleto na
/// identidade, raiz e track intocadas): o osso com relógio próprio (Time Remap), a trajectória com
/// auto-orient, e uma fórmula que lê a posição do osso PELO NOME. Controlo: uma fórmula na própria
/// track (`value + 1`) ou que lê OUTRO objecto não impede a mudança.
#[test]
fn where_the_move_is_not_exact_the_skeleton_stays_at_identity() {
    /// O que arma cada caso na cena.
    type Arma<'a> = &'a dyn Fn(&mut Cena);
    let caso = |k: u64, arma: Arma| {
        let mut c = cena(k);
        let raiz = c.ossos[0];
        c.sim.world_mut().entity_mut(raiz).insert(Name::new("Osso"));
        arma(&mut c);
        apply_from_doc(c.sim.world_mut(), &mut c.tl.doc, 0.4);
        let l = c.sim.world().get::<Transform>(raiz).expect("r").translation;
        let esq = Entity::from_bits(adopt_loose_roots(&mut c.sim, &mut c.tl)[0]);
        let t = c
            .sim
            .world()
            .get::<Transform>(esq)
            .expect("esq")
            .translation;
        let r = c.sim.world().get::<Transform>(raiz).expect("r").translation;
        (
            t == Vec2::new(0.0, 0.0) && r == l,
            lugar_em(&c.tl, raiz.to_bits()) > 0,
        )
    };
    let formula = |c: &mut Cena, src: &str| {
        let outro = c
            .sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Outro")))
            .id();
        let tgt = c.tl.doc.bind(outro.to_bits(), P::TranslationX);
        c.tl.doc
            .bindings_mut()
            .iter_mut()
            .find(|b| b.target == tgt)
            .expect("b")
            .expr = Some(src.into());
    };
    let recusas: [(&str, u64, Arma); 3] = [
        ("Time Remap", 4, &|c| {
            c.tl.doc.bind(c.ossos[0].to_bits(), P::TimeRemap);
        }),
        ("auto-orient", 5, &|c| {
            c.tl.doc.set_auto_orient(c.ossos[0].to_bits(), true);
        }),
        ("fórmula pelo nome", 4, &|c| formula(c, "Osso.x * 2")),
    ];
    for (nome, k, arma) in recusas {
        assert_eq!(
            caso(k, arma),
            (true, true),
            "{nome}: mudou o dono sem ser exacto"
        );
    }
    let controlos: [(&str, Arma); 2] = [
        ("fórmula na própria track", &|c| {
            let b = c.ossos[0].to_bits();
            let tgt = c.tl.doc.binding_for(b, P::TranslationX).expect("x").target;
            c.tl.doc
                .bindings_mut()
                .iter_mut()
                .find(|b| b.target == tgt)
                .expect("b")
                .expr = Some("value + 1".into());
        }),
        ("fórmula que lê outro objecto", &|c| {
            formula(c, "Outro.x + 1")
        }),
    ];
    for (nome, arma) in controlos {
        assert_eq!(
            caso(4, arma),
            (false, false),
            "controlo «{nome}»: a recusa é larga demais"
        );
    }
}
