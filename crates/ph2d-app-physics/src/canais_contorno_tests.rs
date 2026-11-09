//! (W4) **A curva do Detail é pré-visualização também no TRAÇO**: os anéis são derivados do pedido,
//! e a fotografia do undo tem de ver o par AUTORADO (pedido + anéis + caixa), senão cada quadro a
//! tocar com input vira um passo. *Mutação: `substituir` sem repor o contorno ⇒ vermelho.*

use super::*;
use crate::physics_tests::registry;
use ph2d_ecs::{ChildOf, Entity, SpritePixels, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderOutline, ColliderShape, RigidBody};
use ph2d_render::Sprite;

const PPM: f32 = 8.0;

/// Um pente 32×32 (dentes de 4 px que o Detail de 2 px guarda e o de 6 px fecha) num corpo Static;
/// a forma FILHA em «Outline», já traçada e anotada.
fn pente() -> (ComponentRegistry, SimWorld, Entity, ph2d_asset::AssetDb) {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let reg = registry();
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let px: Vec<u8> = (0..32u32)
        .flat_map(|y| (0..32u32).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            let on = (4..28).contains(&x) && (4..28).contains(&y) && (y >= 8 || (x / 3) % 2 == 0);
            [255, 255, 255, if on { 255 } else { 0 }]
        })
        .collect();
    let img = db.insert_image_rgba8(32, 32, px);
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::default(),
            Sprite::atlas(0, [4.0, 4.0], [1.0; 4]),
            SpritePixels(img),
        ))
        .id();
    let c = Collider {
        shape: ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0,
        },
        ..Collider::default()
    };
    sim.world_mut()
        .spawn((c, Transform::default(), ChildOf(body)));
    let q = EditorCommandQueue::new();
    let none = std::collections::BTreeMap::new();
    let pick = [(body.to_bits(), P::Shape(3))];
    ph2d_physics_outline::retrace_edited(sim.world(), &pick, &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    segue(&mut sim, &db, &reg);
    (reg, sim, body, db)
}

/// A passagem do quadro (`inspector_commits`): retraça o que a curva pediu, pela fila.
fn segue(sim: &mut SimWorld, db: &ph2d_asset::AssetDb, reg: &ComponentRegistry) {
    let q = EditorCommandQueue::new();
    let none = std::collections::BTreeMap::new();
    ph2d_physics_outline::retrace_followed(sim.world_mut(), db, &none, PPM, &q, reg);
    apply_editor_commands(sim.world_mut(), &q, reg).expect("aplica");
}

fn par(sim: &SimWorld, forma: Entity) -> (Collider, ColliderOutline) {
    let w = sim.world();
    (
        *w.get::<Collider>(forma).expect("c"),
        w.get::<ColliderOutline>(forma).cloned().expect("o"),
    )
}

/// ⭐⭐ Chaves no Detail (2 px em t0, 6 px em t1), a tocar através delas pelo caminho do quadro
/// (`aplicar` → `retrace_followed` → fila → `settle` → fotografia): a fotografia é a de ANTES em
/// todo quadro (zero passos), e o vivo segue a chave. Parada, a curva vira documento (um passo).
#[test]
fn a_curva_do_detail_nao_e_passo_de_undo_a_tocar() {
    let (reg, mut sim, body, db) = pente();
    let dono = ph2d_physics_ecs::dono_do_campo(sim.world(), body.to_bits(), f::CONTORNO_DETALHE);
    let forma = Entity::from_bits(dono);
    let base = par(&sim, forma);
    assert!(!base.1.rings.is_empty(), "o pente traçou");
    let mut conducao = ConducaoFisica::default();
    conducao.a_tocar(true);
    for detalhe in [3.0, 4.5, 6.0, 6.0] {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(dono, k(f::CONTORNO_DETALHE), detalhe);
        aplicar(&mut sim, &reg, &mut conducao, &Default::default());
        segue(&mut sim, &db, &reg);
        conducao.settle();
        let vivo = conducao.substituir(&mut sim, &reg);
        assert_eq!(
            par(&sim, forma),
            base,
            "a tocar, Detail {detalhe}: a fotografia é a de antes"
        );
        conducao.repor(&mut sim, &reg, vivo);
        let o = par(&sim, forma).1;
        assert_eq!(o.detail_q4, (detalhe * 16.0) as u16, "o vivo segue a chave");
        assert_ne!(o.rings, base.1.rings, "e retraçou");
    }
    let vivo_final = par(&sim, forma);
    conducao.a_tocar(false);
    conducao.settle();
    conducao.settle();
    assert!(conducao.is_empty());
    let vivo = conducao.substituir(&mut sim, &reg);
    assert_eq!(par(&sim, forma), vivo_final, "parada, a curva é documento");
    conducao.repor(&mut sim, &reg, vivo);
}

/// (e) **O Detail editado no cartão chega ao que o Auto Key lê**: a edição pelo funil do dreno
/// (`apply_physics_edit` + fila) e `publicar` dão `CONTORNO_DETALHE` = 6 na FORMA (a trilha da
/// chave, opção B) — o Auto Key grava a diferença de `CanaisLidos`. *Mutação: `ler` do Detail a
/// devolver o padrão ⇒ vermelho.*
#[test]
fn o_detail_do_cartao_publica_o_canal_que_o_auto_key_grava() {
    use ph2d_editor_core::{OutlineEdit, PhysicsFieldEdit as P};
    let (reg, mut sim, body, _db) = pente();
    let b = body.to_bits();
    let dono = dono_da_chave(
        sim.world(),
        b,
        ph2d_timeline::PropKind::Field(k(f::CONTORNO_DETALHE)),
    );
    assert_ne!(dono, b, "a trilha é a da forma");
    let lido = |sim: &SimWorld| {
        sim.world()
            .resource::<CanaisLidos>()
            .valor(dono, k(f::CONTORNO_DETALHE))
    };
    publicar(&mut sim, &[b]);
    assert_eq!(lido(&sim), Some(2.0));
    let q = EditorCommandQueue::new();
    crate::physics_apply::apply_physics_edit(
        &mut sim,
        b,
        P::Outline(OutlineEdit::Detail(6.0)),
        &q,
        &reg,
    );
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    publicar(&mut sim, &[b]);
    assert_eq!(lido(&sim), Some(6.0), "o Auto Key vê o Detail novo");
}

/// Uma curva do Detail sobre um Outline à mão (`custom`) NÃO retraça: os pontos têm a trilha própria
/// (51/52). *Mutação: `retrace_followed` retraçar o pedido mudado com `custom` ⇒ vermelho.*
#[test]
fn a_curva_do_detail_nao_apaga_os_pontos_a_mao() {
    let (reg, mut sim, body, db) = pente();
    let dono = ph2d_physics_ecs::dono_do_campo(sim.world(), body.to_bits(), f::CONTORNO_DETALHE);
    let mut mao = par(&sim, Entity::from_bits(dono)).1.rings.to_vec();
    mao[0][0][0] += 16;
    ph2d_shape_handles::pontos::escreve(sim.world_mut(), dono, mao.clone(), true);
    let mut conducao = ConducaoFisica::default();
    conducao.a_tocar(true);
    for detalhe in [3.0, 6.0] {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(dono, k(f::CONTORNO_DETALHE), detalhe);
        aplicar(&mut sim, &reg, &mut conducao, &Default::default());
        segue(&mut sim, &db, &reg);
        let o = par(&sim, Entity::from_bits(dono)).1;
        assert!(
            o.custom && o.rings[..] == mao[..],
            "Detail {detalhe}: os pontos ficam"
        );
    }
}

/// O pente com os pontos editados à mão: a forma e os anéis.
fn a_mao(sim: &mut SimWorld, body: Entity) -> (u64, Vec<Vec<[i32; 2]>>) {
    let forma = ph2d_physics_ecs::dono_do_campo(sim.world(), body.to_bits(), f::CONTORNO_PONTO_X);
    let aneis = par(sim, Entity::from_bits(forma)).1.rings.to_vec();
    ph2d_shape_handles::pontos::escreve(sim.world_mut(), forma, aneis.clone(), true);
    (forma, aneis)
}

/// ⭐ Arrastar um ponto: `publicar` → `CanaisLidos` → `autokey_campos` (o caminho do quadro) pede UMA
/// chave com TODOS os pontos. *Mutação: `publicar` sem os pontos ⇒ vermelho.*
#[test]
fn arrastar_um_ponto_grava_uma_chave_com_todos_os_pontos() {
    let (_reg, mut sim, body, _db) = pente();
    let (forma, mut aneis) = a_mao(&mut sim, body);
    let n: usize = aneis.iter().map(Vec::len).sum();
    publicar(&mut sim, &[body.to_bits()]);
    let base = ph2d_ecs::canal::campos_lidos(&sim, forma)
        .into_iter()
        .collect();
    aneis[0][1][1] += 16;
    ph2d_shape_handles::pontos::escreve(sim.world_mut(), forma, aneis, true);
    publicar(&mut sim, &[body.to_bits()]);
    let lidos = ph2d_ecs::canal::campos_lidos(&sim, forma);
    let doc = ph2d_timeline::TimelineDoc::new();
    let plano = ph2d_timeline::autokey_campos(&doc, forma, 0.0, &lidos, &base, true, true);
    let ponto = |p: &ph2d_timeline::PropKind| matches!(p, ph2d_timeline::PropKind::Field(kk) if f::e_do_ponto(kk.campo));
    assert_eq!(plano.keys.iter().filter(|(p, _)| ponto(p)).count(), 2 * n);
}

/// ⭐ A tocar entre chaves os pontos andam e o rapier segue; a fotografia do undo vê os autorados
/// (nenhum passo). *Mutação: `aplicar` sem os pontos ⇒ vermelho.*
#[test]
fn a_tocar_os_pontos_andam_e_o_rapier_segue_sem_passo() {
    let (reg, mut sim, body, _db) = pente();
    let (forma, a) = a_mao(&mut sim, body);
    let b: Vec<Vec<[i32; 2]>> = a
        .iter()
        .map(|r| r.iter().map(|p| [p[0], p[1] - 64]).collect())
        .collect();
    let mut ponte = ph2d_physics_ecs::PhysicsBridge::new();
    ponte.dispatch(&mut sim, true, 1);
    let caixa = ponte.body_extent(body);
    let mut conducao = ConducaoFisica::default();
    conducao.a_tocar(true);
    let alvo = ColliderOutline {
        rings: b.clone().into(),
        custom: true,
        ..ColliderOutline::default()
    };
    for (kk, v) in ph2d_physics_outline::canais_pontos::publicados(&alvo) {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        w.escrever(forma, kk, v);
    }
    aplicar(&mut sim, &reg, &mut conducao, &Default::default());
    assert_eq!(
        par(&sim, Entity::from_bits(forma)).1.rings[..],
        b[..],
        "a curva leva os pontos"
    );
    ponte.dispatch(&mut sim, true, 2);
    assert_ne!(ponte.body_extent(body), caixa, "o collider do rapier segue");
    conducao.settle();
    let vivo = conducao.substituir(&mut sim, &reg);
    assert_eq!(
        par(&sim, Entity::from_bits(forma)).1.rings[..],
        a[..],
        "a fotografia: os autorados"
    );
    conducao.repor(&mut sim, &reg, vivo);
}

/// ⭐ Dono 09/11: dentro de «Edit shape on canvas» o Detail de um Outline à mão RETRAÇA da imagem — os pontos
/// à mão dão lugar aos traçados, `custom` volta a falso, e o pedido e o traço vão na MESMA fila (um passo de
/// undo). *Mutação: `retrace_edited` a saltar o `custom` ⇒ os pontos à mão ficam, vermelho.*
#[test]
fn o_detail_dentro_da_edicao_retraca_um_outline_a_mao() {
    use ph2d_editor_core::{OutlineEdit, PhysicsFieldEdit as P};
    let (reg, mut sim, body, db) = pente();
    let (forma, mut mao) = a_mao(&mut sim, body);
    mao[0][0][0] += 16;
    ph2d_shape_handles::pontos::escreve(sim.world_mut(), forma, mao.clone(), true);
    assert!(par(&sim, Entity::from_bits(forma)).1.custom);
    let b = body.to_bits();
    let edit = P::Outline(OutlineEdit::Detail(6.0));
    let q = EditorCommandQueue::new();
    crate::physics_apply::apply_physics_edit(&mut sim, b, edit, &q, &reg);
    let none = std::collections::BTreeMap::new();
    ph2d_physics_outline::retrace_edited(sim.world(), &[(b, edit)], &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let o = par(&sim, Entity::from_bits(forma)).1;
    assert!(!o.custom, "retraçado: já não é à mão");
    assert_ne!(
        o.rings[..],
        mao[..],
        "os pontos à mão deram lugar aos traçados"
    );
    assert_eq!(o.detail_q4, 96, "com o Detail novo");
}

/// ⭐ Regra B no retraço: o mesmo Detail dentro da edição, mas com trilha de pontos (FISICA 51) e o Auto Key
/// desligado — o cartão (`apply_physics_edit` ⇒ `outline_edited` recusa) e a cola (`retrace_edited`) não mexem
/// nos anéis à mão, e o quadro seguinte (`regra_b`) abre a pergunta. *Mutação: `retrace_edited` sem a trava ⇒
/// os pontos à mão dão lugar aos traçados, vermelho.*
#[test]
fn com_pontos_animados_o_detail_dentro_da_edicao_nao_retraca_e_pede_o_modal() {
    use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};
    use ph2d_editor_core::{OutlineEdit, PhysicsFieldEdit as P};
    let (reg, mut sim, body, db) = pente();
    let (forma, mut mao) = a_mao(&mut sim, body);
    mao[0][0][0] += 16;
    ph2d_shape_handles::pontos::escreve(sim.world_mut(), forma, mao.clone(), true);
    let mut tl = ph2d_timeline::TimelineState::new();
    tl.flags.auto_key = false;
    let kk = ph2d_ecs::FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        f::CONTORNO_PONTO_X,
        0,
        0,
    );
    tl.doc.upsert_key(
        forma,
        ph2d_timeline::PropKind::Field(kk),
        ph2d_anim::RationalTime::ZERO,
        ph2d_anim::AnimValue::Float(0.0),
        ph2d_anim::Interp::Linear,
    );
    let mut loja = WidgetStore::default();
    ph2d_physics_outline::canais_pontos::regra_b(sim.world_mut(), &tl, &mut loja);
    let b = body.to_bits();
    let edit = P::Outline(OutlineEdit::Detail(6.0));
    let q = EditorCommandQueue::new();
    crate::physics_apply::apply_physics_edit(&mut sim, b, edit, &q, &reg);
    let none = std::collections::BTreeMap::new();
    let n =
        ph2d_physics_outline::retrace_edited(sim.world(), &[(b, edit)], &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let o = par(&sim, Entity::from_bits(forma)).1;
    assert_eq!(n, 0, "travado: nada retraçado");
    assert!(o.custom, "continua à mão");
    assert_eq!(o.rings[..], mao[..], "os anéis à mão intactos");
    ph2d_physics_outline::canais_pontos::regra_b(sim.world_mut(), &tl, &mut loja);
    assert_eq!(
        loja.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ConfirmDeletePointsAnimation { shape: forma }),
        "o quadro seguinte abre a pergunta"
    );
}
