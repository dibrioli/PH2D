//! **W5 — os pontos do «Outline» no canvas** pelas portas reais do ponteiro (`ph2d_shape_handles::Porta`), em
//! ecrã, sobre a argola traçada (`followed_ring`: 4 m × 4 m, buraco de 2 m no meio).

use super::*;
use ph2d_editor_core::action_bus::ActionBus;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_shape_handles::pontos::{PontosNoCanvas, do_alvo, locais};
use ph2d_shape_handles::{Porta, Teclas, alterna};

const WIN: WindowSize = WindowSize {
    width: 1400,
    height: 900,
};
fn cam() -> Camera2d {
    Camera2d::new([0.0, 0.0], 10.0)
}

fn porta<'a>(sim: &'a mut SimWorld, bus: &'a mut ActionBus) -> Porta<'a> {
    Porta {
        world: sim.world_mut(),
        camera: cam(),
        window: WIN,
        bus,
    }
}

/// Onde o ponto de mundo `w` está em ecrã.
fn ecra(w: [f32; 2]) -> (f32, f32) {
    cam().world_to_screen(w, WIN)
}

/// O ponto do anel `anel` mais perto de `w` (mundo; o corpo está na origem, sem rodar).
fn perto(sim: &SimWorld, shape: ph2d_ecs::Entity, anel: usize, w: [f32; 2]) -> [f32; 2] {
    let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
    locais(&pose, &m, &o)[anel]
        .iter()
        .copied()
        .min_by(|a, b| {
            let d = |p: &[f32; 2]| (p[0] - w[0]).hypot(p[1] - w[1]);
            d(a).total_cmp(&d(b))
        })
        .expect("anel")
}

fn arrasta(sim: &mut SimWorld, de: [f32; 2], para: [f32; 2]) {
    let mut bus = ActionBus::default();
    let mut p = porta(sim, &mut bus);
    assert!(p.desce_com(ecra(de), Teclas::default()), "agarrou");
    assert!(p.move_(ecra(para), Teclas::default()));
    assert!(p.sobe());
}

fn outline(sim: &SimWorld, shape: ph2d_ecs::Entity) -> ColliderOutline {
    sim.world()
        .get::<ColliderOutline>(shape)
        .cloned()
        .expect("contorno")
}

/// Uma bola largada em `(0, 4)` pousa no topo (raio 0,3).
fn pousa(sim: &mut SimWorld) -> f32 {
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 4.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(sim, true, tick);
    }
    let y = bridge.body_pose(ball).expect("corpo").1;
    sim.world_mut().despawn(ball);
    y
}

/// ⭐ (1) **Arrastar um ponto move-o e o motor segue**: os dois cantos de cima sobem 1 m (dois gestos) e a bola
/// pousa 1 m acima; o `Esc` a meio devolve os anéis do `Down` (o gesto é um valor só).
/// *Mutação: o `move_` dos pontos não escrever ⇒ a bola pousa onde pousava.*
#[test]
fn arrastar_um_ponto_move_o_e_a_bola_pousa_na_nova_altura() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let antes = pousa(&mut sim);
    assert!((antes - 2.3).abs() < 0.05, "no topo da argola: {antes}");
    let inicio = outline(&sim, shape);
    // Esc a meio: nada fica.
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    {
        let mut bus = ActionBus::default();
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(de), Teclas::default()));
        assert!(p.move_(ecra([de[0], de[1] + 1.0]), Teclas::default()));
        assert!(p.esc());
    }
    assert_eq!(outline(&sim, shape), inicio, "o Esc devolve o Down");
    for x in [-2.0, 2.0] {
        let de = perto(&sim, shape, 0, [x, 2.0]);
        arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
        let agora = perto(&sim, shape, 0, [x, 3.0]);
        assert!(
            (agora[1] - (de[1] + 1.0)).abs() < 0.01,
            "o ponto foi ao ponteiro: {agora:?}"
        );
    }
    let depois = pousa(&mut sim);
    assert!(
        (depois - 3.3).abs() < 0.05,
        "a bola pousa 1 m acima: {depois}"
    );
}

/// ⭐ (2) **Um clique numa aresta acrescenta o ponto ONDE se clicou (dono 09/11); Alt+clique tira-o**; tirar um
/// canto de fora é recusado (a diagonal cruzaria o buraco).
/// *Mutações: `acrescentado` no índice errado ⇒ o anel cruza-se e nada entra; `tirado` sem `validos` ⇒ o canto
/// sai.*
#[test]
fn clique_na_aresta_acrescenta_e_alt_clique_tira() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let n = |sim: &SimWorld| outline(sim, shape).rings[0].len();
    let n0 = n(&sim);
    let clique = [0.5, 2.0];
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(clique), Teclas::default()));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0 + 1, "o clique na aresta acrescentou");
    let novo = perto(&sim, shape, 0, clique);
    assert!(
        (novo[0] - clique[0]).hypot(novo[1] - clique[1]) < 0.01,
        "o ponto nasce onde se clicou: {novo:?}"
    );
    assert!(outline(&sim, shape).custom, "editar pontos passa a Custom");
    let alt = Teclas {
        alt: true,
        ..Teclas::default()
    };
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(novo), alt));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0, "Alt+clique tirou-o");
    let canto = perto(&sim, shape, 0, [-2.0, 2.0]);
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(canto), alt));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0, "o canto não sai: a diagonal cruzaria o buraco");
}

/// ⭐ (3) **Arrastar para um cruzamento é recusado**: o canto de cima à esquerda levado para dentro do buraco
/// fica na última posição válida (antes de cruzar).
/// *Mutação: `movido` sem `validos` ⇒ o canto entra no buraco.*
#[test]
fn o_arrasto_que_cruzaria_fica_na_ultima_posicao_valida() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    let mut bus = ActionBus::default();
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra(de), Teclas::default()));
    assert!(p.move_(ecra([-2.5, 2.5]), Teclas::default()));
    assert!(p.move_(ecra([0.0, 0.0]), Teclas::default()));
    assert!(p.sobe());
    let agora = perto(&sim, shape, 0, [-2.5, 2.5]);
    assert!(
        (agora[0] + 2.5).abs() < 0.01 && (agora[1] - 2.5).abs() < 0.01,
        "fica na última válida: {agora:?}"
    );
}

/// ⭐ (4) **Os pontos só com a FORMA em edição** (resposta 3 do dono): o corpo não os tem; a forma tem.
/// *Mutação: `do_alvo` pela 1.ª forma do corpo ⇒ o corpo agarra um ponto.*
#[test]
fn os_pontos_so_aparecem_com_a_forma_selecionada() {
    let (_reg, mut sim, body, shape, _db) = followed_ring();
    let canto = perto(&sim, shape, 0, [-2.0, 2.0]);
    alterna(sim.world_mut(), body.to_bits());
    assert!(
        do_alvo(sim.world(), body.to_bits()).is_none(),
        "o corpo não tem pontos"
    );
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        let _ = p.desce_com(ecra(canto), Teclas::default());
    }
    assert!(
        sim.world()
            .get_resource::<PontosNoCanvas>()
            .is_none_or(|p| p.arrasto.is_none()),
        "com o corpo, nenhum ponto se agarra"
    );
    alterna(sim.world_mut(), shape.to_bits());
    assert!(
        do_alvo(sim.world(), shape.to_bits()).is_some(),
        "a forma tem pontos"
    );
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra(canto), Teclas::default()));
    assert!(
        p.world
            .get_resource::<PontosNoCanvas>()
            .is_some_and(|p| p.arrasto.is_some()),
        "com a forma, o ponto agarra-se"
    );
}

/// ⭐ (5) **Editar pontos passa a «Custom»; a imagem nova não lhes toca; «Fit to picture» retraça** e volta
/// a seguir. *Mutações: `escreve` sem `custom` ⇒ segue; `follows` sem o `custom` ⇒ o cartão diz Follows.*
#[test]
fn editar_pontos_e_custom_e_o_fit_retraca() {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let (reg, mut sim, body, shape, db) = followed_ring();
    let traced = outline(&sim, shape);
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(true)
    );
    alterna(sim.world_mut(), shape.to_bits());
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(false),
        "Custom"
    );
    let editado = outline(&sim, shape);
    let q = EditorCommandQueue::new();
    let disc = db.insert_image_rgba8(32, 32, img(32, 32, |_, _| true));
    sim.world_mut().entity_mut(body).insert(SpritePixels(disc));
    assert_eq!(
        follow(&mut sim, &db, &q, &reg),
        0,
        "o Custom não segue a imagem"
    );
    assert_eq!(outline(&sim, shape), editado);
    sim.world_mut()
        .entity_mut(body)
        .insert(SpritePixels(db.insert_image_rgba8(32, 32, ring_px())));
    let none = std::collections::BTreeMap::new();
    let fit = [(shape.to_bits(), P::FitToPicture)];
    ph2d_physics_outline::retrace_edited(sim.world(), &fit, &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    assert_eq!(outline(&sim, shape), traced, "o Fit devolve o traço");
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(true)
    );
}

fn clica(sim: &mut SimWorld, w: [f32; 2], t: Teclas) -> bool {
    let mut bus = ActionBus::default();
    let mut p = porta(sim, &mut bus);
    let c = p.desce_com(ecra(w), t);
    let _ = p.sobe();
    c
}

fn selecionados(sim: &SimWorld) -> usize {
    sim.world()
        .get_resource::<PontosNoCanvas>()
        .map_or(0, |p| p.selecionados.len())
}

fn apaga(sim: &mut SimWorld) -> bool {
    let mut bus = ActionBus::default();
    porta(sim, &mut bus).apaga()
}

/// ⭐ (6) **Selecionar e apagar** (dono 09/11): o clique seleciona e a seleção fica depois de soltar; Shift alterna;
/// o vazio limpa; o Delete tira TODOS os selecionados; um anel nunca desce de 3 (o excesso fica).
/// *Mutações: `apaga` tirar só o 1.º ⇒ sobra um; `tirados` sem o piso de 3 ⇒ o buraco vira segmento.*
#[test]
fn clicar_seleciona_shift_alterna_o_vazio_limpa_e_delete_tira_os_selecionados() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let n = |sim: &SimWorld, k: usize| outline(sim, shape).rings[k].len();
    let n0 = n(&sim, 0);
    let (a, b) = ([-0.5, 2.0], [0.5, 2.0]);
    assert!(clica(&mut sim, a, Teclas::default()));
    assert!(clica(&mut sim, b, Teclas::default()));
    assert_eq!(n(&sim, 0), n0 + 2, "dois pontos novos na aresta de cima");
    assert_eq!(selecionados(&sim), 1, "o ponto novo nasce selecionado");
    let shift = Teclas {
        shift: true,
        ..Teclas::default()
    };
    assert!(clica(&mut sim, a, shift));
    assert_eq!(selecionados(&sim), 2, "Shift acrescenta à seleção");
    assert!(clica(&mut sim, a, shift));
    assert_eq!(selecionados(&sim), 1, "Shift de novo tira");
    assert!(!clica(&mut sim, [0.0, 0.0], Teclas::default()) || selecionados(&sim) == 0);
    assert_eq!(selecionados(&sim), 0, "o vazio limpa");
    assert!(!apaga(&mut sim), "sem seleção o Delete não é dos pontos");
    assert!(clica(&mut sim, a, Teclas::default()));
    assert!(clica(&mut sim, b, shift));
    assert!(apaga(&mut sim));
    assert_eq!(n(&sim, 0), n0, "o Delete tirou os dois");
    assert_eq!(selecionados(&sim), 0);
    // O buraco: os quatro cantos selecionados, sai um só — o anel fica com 3.
    assert_eq!(n(&sim, 1), 4, "o buraco é um quadrado");
    for (k, w) in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        .into_iter()
        .enumerate()
    {
        let t = if k == 0 { Teclas::default() } else { shift };
        let w = perto(&sim, shape, 1, w);
        assert!(clica(&mut sim, w, t));
    }
    assert_eq!(selecionados(&sim), 4);
    assert!(apaga(&mut sim));
    assert_eq!(n(&sim, 1), 3, "um anel nunca desce de 3");
}

/// ⭐ (7) **Soltar um ponto sobre outro funde** (dono 09/11): o arrastado sai; soltar longe só move.
/// *Mutação: `funde` sem tirar ⇒ o anel fica com o ponto.*
#[test]
fn soltar_um_ponto_sobre_outro_tira_o_arrastado() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let n = |sim: &SimWorld| outline(sim, shape).rings[0].len();
    assert!(clica(&mut sim, [0.0, 2.0], Teclas::default()));
    let n1 = n(&sim);
    let canto = perto(&sim, shape, 0, [2.0, 2.0]);
    arrasta(&mut sim, [0.0, 2.0], [1.0, 2.0]);
    assert_eq!(n(&sim), n1, "soltar longe só move");
    arrasta(&mut sim, [1.0, 2.0], canto);
    assert_eq!(n(&sim), n1 - 1, "soltar sobre o canto tirou o arrastado");
    assert_eq!(perto(&sim, shape, 0, [2.0, 2.0]), canto, "o canto fica");
}

/// ⭐ (8) **Com os pontos, a caixa da forma não arrasta** (dono 09/11): o miolo e as alças consomem o clique e nada
/// muda. *Mutação: `desce_com` armar a alça no Outline ⇒ o miolo move o Offset.*
#[test]
fn com_os_pontos_a_caixa_da_forma_nao_arrasta() {
    use ph2d_editor_core::action_bus::EditorAction;
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(
            p.desce_com(ecra([0.0, 0.0]), Teclas::default()),
            "consumido"
        );
        let _ = p.move_(ecra([1.5, 0.5]), Teclas::default());
        let _ = p.move_(ecra([3.0, 1.0]), Teclas::default());
        let _ = p.sobe();
    }
    let acoes: Vec<_> = bus.drain().collect();
    assert!(
        !acoes
            .iter()
            .any(|a| matches!(a, EditorAction::InspectorPhysicsEdit { .. })),
        "a caixa não escreveu: {acoes:?}"
    );
}

/// ⭐ (9) **Os indicadores**: sob o ponto = esse ponto; sobre a aresta = o «+» no sítio projetado; a meio do
/// arrasto sobre outro ponto = o alvo da fusão. *Mutação: `estados` sem o `+` ⇒ nenhum fantasma.*
#[test]
fn os_indicadores_seguem_o_ponteiro() {
    use ph2d_shape_handles::pontos::no_ecra;
    use ph2d_shape_handles::tinta_pontos::estados;
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let ecra_de = |sim: &SimWorld| {
        let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
        no_ecra(&pose, &m, &o, &cam(), WIN)
    };
    let canto = perto(&sim, shape, 0, [2.0, 2.0]);
    let e = ecra_de(&sim);
    assert!(estados(sim.world(), &e, ecra(canto)).sob.is_some());
    let mais = estados(sim.world(), &e, ecra([0.3, 2.02])).mais.expect("+");
    let alvo = ecra([0.3, 2.0]);
    assert!((mais.0 - alvo.0).hypot(mais.1 - alvo.1) < 0.5, "{mais:?}");
    assert!(clica(&mut sim, [0.0, 2.0], Teclas::default()));
    let mut bus = ActionBus::default();
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra([0.0, 2.0]), Teclas::default()));
    assert!(p.move_(ecra([1.0, 2.0]), Teclas::default()));
    let e = ecra_de(&sim);
    let est = estados(sim.world(), &e, ecra(canto));
    assert!(est.fusao.is_some(), "o canto é alvo da fusão");
    assert!(estados(sim.world(), &e, ecra([1.0, 2.0])).fusao.is_none());
}

/// ⭐ (10) **O cursor dos pontos** é UMA tabela: vazio = seta directa · ponto = seta com o ponto (Alt = pena «−»,
/// Shift = «±») · aresta = pena «+» · arrasto = mão fechada · arrasto sobre outro ponto = fundir. E a tinta acende o
/// mesmo: a aresta do «+» e o ponto que o Alt tira. *Mutação: o `Alt` ignorado ⇒ `SetaPonto` e nenhum `tira`.*
#[test]
fn o_cursor_e_os_indicadores_dizem_o_que_o_clique_faz() {
    use ph2d_shape_handles::pontos::no_ecra;
    use ph2d_shape_handles::tinta_pontos::estados_com;
    use ph2d_shape_handles::{Figura as F, cursor_dos_pontos};
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    let cur = |sim: &SimWorld, w: [f32; 2], t: Teclas| {
        cursor_dos_pontos(sim.world(), &cam(), WIN, ecra(w), t)
    };
    assert_eq!(
        cur(&sim, [0.0, 0.0], Teclas::default()),
        None,
        "sem edição, o cursor é de outro"
    );
    alterna(sim.world_mut(), shape.to_bits());
    let canto = perto(&sim, shape, 0, [2.0, 2.0]);
    let (alt, shift) = (
        Teclas {
            alt: true,
            shift: false,
        },
        Teclas {
            alt: false,
            shift: true,
        },
    );
    let tabela = [
        ([30.0, 30.0], Teclas::default(), F::Seta),
        (canto, Teclas::default(), F::SetaPonto),
        (canto, alt, F::PenaMenos),
        (canto, shift, F::Alterna),
        ([0.3, 2.02], Teclas::default(), F::PenaMais),
    ];
    for (w, t, f) in tabela {
        assert_eq!(cur(&sim, w, t), Some(f), "{w:?} {t:?}");
    }
    let e = {
        let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
        no_ecra(&pose, &m, &o, &cam(), WIN)
    };
    let est = estados_com(sim.world(), &e, ecra(canto), alt);
    assert_eq!(est.tira, est.sob, "Alt sobre o ponto: o que sai");
    assert!(est.tira.is_some());
    assert!(
        estados_com(sim.world(), &e, ecra(canto), Teclas::default())
            .tira
            .is_none()
    );
    let est = estados_com(sim.world(), &e, ecra([0.3, 2.02]), Teclas::default());
    assert!(
        est.aresta.is_some() && est.mais.is_some(),
        "a aresta do «+» acende"
    );
    assert!(clica(&mut sim, [0.0, 2.0], Teclas::default()));
    let mut bus = ActionBus::default();
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra([0.0, 2.0]), Teclas::default()));
    assert_eq!(
        cur(&sim, [0.0, 2.0], Teclas::default()),
        Some(F::SetaPonto),
        "antes de armar não é mão"
    );
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.move_(ecra([0.6, 2.0]), Teclas::default()));
    assert_eq!(cur(&sim, [0.6, 2.0], Teclas::default()), Some(F::Agarrando));
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.move_(ecra([1.0, 2.0]), Teclas::default()));
    assert_eq!(cur(&sim, canto, Teclas::default()), Some(F::Funde));
}

/// **«Merge by distance»** (dono 09/11): a argola com um cacho (dois pontos a 0,5 e 1 px do ponto 0, na aresta). Com
/// a forma em edição e os dois selecionados, só eles se fundem (o ponto 0 fica) e o fundido continua selecionado;
/// sem seleção, o cacho inteiro vira um. Sempre «Custom», e o cartão diz quantos saíram. *Mutações: `outline_edited`
/// sem a seleção ⇒ 0 sai; sem `custom: true` ⇒ vermelho.*
#[test]
fn merge_funde_o_cacho_selecionado_ou_todos_e_marca_custom() {
    use ph2d_editor_core::OutlineEdit as E;
    use ph2d_shape_handles::pontos::Ponto;
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    let mut o = outline(&sim, shape);
    let mut rings: Vec<Vec<[i32; 2]>> = o.rings.to_vec();
    let (a, b) = (rings[0][0], rings[0][1]);
    let len = f64::from(b[0] - a[0]).hypot(f64::from(b[1] - a[1]));
    assert!(len > 48.0, "a aresta tem de caber o cacho: {len}");
    let em = |t: f64| {
        [
            a[0] + (f64::from(b[0] - a[0]) * t / len).round() as i32,
            a[1] + (f64::from(b[1] - a[1]) * t / len).round() as i32,
        ]
    };
    rings[0].insert(1, em(8.0));
    rings[0].insert(2, em(16.0));
    o.rings = rings.clone().into();
    o.custom = false;
    sim.world_mut().entity_mut(shape).insert(o.clone());
    let total = |o: &ColliderOutline| o.rings.iter().map(Vec::len).sum::<usize>();
    let n0 = total(&o);
    alterna(sim.world_mut(), shape.to_bits());
    let sel = vec![Ponto { anel: 0, i: 1 }, Ponto { anel: 0, i: 2 }];
    sim.world_mut().insert_resource(PontosNoCanvas {
        selecionados: sel,
        ..Default::default()
    });
    let um =
        ph2d_physics_outline::outline_edited(sim.world_mut(), shape, E::Merge(3.0)).expect("funde");
    assert_eq!(total(&um), n0 - 1, "só os dois selecionados");
    assert!(um.custom, "o Merge é Custom");
    assert!(
        um.rings[0].contains(&a),
        "o ponto 0 não estava selecionado e fica"
    );
    let s = sim
        .world()
        .resource::<PontosNoCanvas>()
        .selecionados
        .clone();
    assert_eq!(s.len(), 1, "o fundido continua selecionado: {s:?}");
    // Sem seleção: todos.
    sim.world_mut().entity_mut(shape).insert(o.clone());
    sim.world_mut().insert_resource(PontosNoCanvas::default());
    let todos =
        ph2d_physics_outline::outline_edited(sim.world_mut(), shape, E::Merge(3.0)).expect("funde");
    assert!(
        total(&todos) <= n0 - 2,
        "o cacho inteiro vira um: {n0} → {}",
        total(&todos)
    );
    assert!(!todos.rings[0].contains(&a), "o ponto 0 entrou na média");
    sim.world_mut().entity_mut(shape).insert(todos.clone());
    let card = ph2d_physics_outline::card_info(sim.world(), shape).expect("cartão");
    assert_eq!(
        card.merged,
        Some(((n0 - total(&todos)) as u32, 0)),
        "o cartão diz quantos saíram"
    );
    // Nada perto: nada a gravar, e o cartão diz 0.
    assert_eq!(
        ph2d_physics_outline::outline_edited(sim.world_mut(), shape, E::Merge(0.25)),
        None
    );
    let card = ph2d_physics_outline::card_info(sim.world(), shape).expect("cartão");
    assert_eq!(card.merged, Some((0, 0)));
}

/// ⭐ (11) **O crescer do ponto sob o ponteiro** é `ease-out` pelo relógio do quadro: 0 ao chegar, já mais de meio a
/// meio do tempo, 1 no fim; outro ponto = 0; o pulso da fusão vai de 0 a 1 e volta. E o `passo` real arranca o
/// relógio quando o ponteiro chega ao ponto e diz que ainda mexe. *Mutação: `suave` linear ⇒ o meio dá 0,5.*
#[test]
fn o_ponto_sob_o_ponteiro_cresce_suave_pelo_relogio() {
    use ph2d_shape_handles::anima_pontos::{AnimaPontos, CRESCE_MS, PULSO_MS, passo, pulso, suave};
    use ph2d_shape_handles::pontos::Ponto;
    assert_eq!(suave(0.0), 0.0);
    assert!((suave(0.5) - 0.875).abs() < 1e-6, "{}", suave(0.5));
    assert_eq!(suave(1.0), 1.0);
    assert_eq!(suave(2.0), 1.0, "depois do fim fica");
    let pt = Ponto { anel: 0, i: 3 };
    let a = |agora| AnimaPontos {
        sob: Some((pt, 1000.0)),
        agora_ms: agora,
    };
    assert_eq!(a(1000.0).crescimento(pt), 0.0);
    assert!(a(1000.0 + CRESCE_MS / 2.0).crescimento(pt) > 0.8);
    assert_eq!(a(1000.0 + CRESCE_MS).crescimento(pt), 1.0);
    assert_eq!(a(5000.0).crescimento(Ponto { anel: 0, i: 4 }), 0.0);
    assert!(pulso(0.0) < 1e-6 && (pulso(PULSO_MS / 2.0) - 1.0).abs() < 1e-6);
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let canto = ecra(perto(&sim, shape, 0, [2.0, 2.0]));
    assert!(
        passo(sim.world_mut(), &cam(), WIN, canto, 500.0),
        "chegou: mexe"
    );
    assert!(passo(
        sim.world_mut(),
        &cam(),
        WIN,
        canto,
        500.0 + CRESCE_MS / 2.0
    ));
    let a_mexer = ph2d_shape_handles::anima_pontos::a_mexer;
    assert!(
        a_mexer(sim.world()),
        "B23.2: a crescer, o laço a pedido lê «a mexer»"
    );
    assert!(
        !passo(sim.world_mut(), &cam(), WIN, canto, 500.0 + CRESCE_MS),
        "cresceu: pára"
    );
    assert!(!a_mexer(sim.world()), "B23.2: assentado, o laço dorme");
    let an = *sim.world().resource::<AnimaPontos>();
    assert_eq!(
        an.sob.map(|s| s.1),
        Some(500.0),
        "o relógio nasce na chegada, não a cada quadro"
    );
}

/// **O «Merge» pedido antes do traço** (a foto `PH2D_PHYSICS_SMOKE_OUTLINE_MERGE`): sem anéis espera; com anéis, o
/// 1.º `retrace_followed` funde todos pela fila e o pedido sai. *Mutação: `pending` sem a espera dos anéis ⇒ o pedido
/// gasta-se no vazio e a 2.ª passagem não funde, vermelho.*
#[test]
fn o_merge_pendente_espera_o_traco_e_funde_uma_vez() {
    let (reg, mut sim, _body, shape, db) = followed_ring();
    let mut cheio = outline(&sim, shape);
    // Um ponto a meio px do ponto 0, na aresta: o cacho que o Merge tira.
    let mut r: Vec<Vec<[i32; 2]>> = cheio.rings.to_vec();
    let (a, b) = (r[0][0], r[0][1]);
    let t = 8.0 / f64::from(b[0] - a[0]).hypot(f64::from(b[1] - a[1]));
    let meio = |i: usize| a[i] + (f64::from([b[0] - a[0], b[1] - a[1]][i]) * t).round() as i32;
    r[0].insert(1, [meio(0), meio(1)]);
    cheio.rings = r.into();
    let mut vazio = cheio.clone();
    vazio.rings = Vec::new().into();
    sim.world_mut().entity_mut(shape).insert(vazio);
    let q = EditorCommandQueue::new();
    let px = 3.0;
    sim.world_mut()
        .insert_resource(ph2d_physics_outline::PendingMerge { shape, px });
    let _ = follow(&mut sim, &db, &q, &reg);
    assert!(
        sim.world()
            .get_resource::<ph2d_physics_outline::PendingMerge>()
            .is_some(),
        "sem anéis, o pedido espera"
    );
    let _ = q.drain();
    sim.world_mut().entity_mut(shape).insert(cheio.clone());
    assert!(follow(&mut sim, &db, &q, &reg) >= 1, "com anéis, funde");
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let n = |o: &ColliderOutline| o.rings.iter().map(Vec::len).sum::<usize>();
    assert!(n(&outline(&sim, shape)) < n(&cheio), "fundiu");
    assert!(outline(&sim, shape).custom);
    assert!(
        sim.world()
            .get_resource::<ph2d_physics_outline::PendingMerge>()
            .is_none()
    );
}

/// ⭐ **Clicar na aresta e arrastar no mesmo movimento** (dono 09/11): o ponto nasce no `Down` e segue o ponteiro
/// até soltar (cursor de mão fechada só a mexer); o `Esc` a meio devolve o anel SEM o ponto; soltar o novo sobre o
/// vizinho funde-o (o anel volta ao tamanho de antes). *Mutação: o ponto acrescentado não ficar agarrado ⇒ o
/// `move_` não o leva, vermelho.*
#[test]
fn clicar_na_aresta_e_arrastar_leva_o_ponto_novo() {
    use ph2d_shape_handles::{Figura as F, cursor_dos_pontos};
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let n = |sim: &SimWorld| outline(sim, shape).rings[0].len();
    let (n0, inicio) = (n(&sim), outline(&sim, shape));
    let (clique, para) = ([0.5, 2.0], [0.5, 2.8]);
    let cur = |sim: &SimWorld, w: [f32; 2]| {
        cursor_dos_pontos(sim.world(), &cam(), WIN, ecra(w), Teclas::default())
    };
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(clique), Teclas::default()));
        assert!(
            p.move_(ecra(para), Teclas::default()),
            "o arrasto é dos pontos"
        );
        assert!(p.esc());
    }
    assert_eq!(
        outline(&sim, shape),
        inicio,
        "o Esc devolve o anel sem o ponto"
    );
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(clique), Teclas::default()));
    }
    assert_ne!(
        cur(&sim, clique),
        Some(F::Agarrando),
        "antes de mexer não é mão"
    );
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.move_(ecra(para), Teclas::default()));
    }
    assert_eq!(
        cur(&sim, para),
        Some(F::Agarrando),
        "a mexer é a mão fechada"
    );
    assert!(porta(&mut sim, &mut bus).sobe());
    assert_eq!(n(&sim), n0 + 1, "um ponto a mais");
    let novo = perto(&sim, shape, 0, para);
    assert!(
        (novo[0] - para[0]).hypot(novo[1] - para[1]) < 0.01,
        "o ponto novo foi ao ponteiro: {novo:?}"
    );
    assert!(outline(&sim, shape).custom);
    let canto = perto(&sim, shape, 0, [2.0, 2.0]);
    let clique = [-2.0, 0.5];
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(clique), Teclas::default()));
        assert!(p.move_(ecra([-2.0, 1.5]), Teclas::default()));
        let viz = perto(&sim, shape, 0, [-2.0, 2.0]);
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.move_(ecra(viz), Teclas::default()));
        assert!(p.sobe());
    }
    assert_eq!(n(&sim), n0 + 1, "soltar o novo sobre o vizinho fundiu-o");
    assert_eq!(perto(&sim, shape, 0, [2.0, 2.0]), canto);
}

/// Trava os pontos de `shape` (regra B: trilha 51/52 e Auto Key desligado), como a shell publica por quadro.
fn trava(sim: &mut SimWorld, shape: ph2d_ecs::Entity) {
    ph2d_shape_handles::travados::publica(
        sim.world_mut(),
        [shape.to_bits()].into_iter().collect(),
        false,
    );
}

fn pedido(sim: &mut SimWorld) -> Option<u64> {
    ph2d_shape_handles::travados::pedido(sim.world_mut())
}

/// ⭐ Regra B (dono 09/11): com os pontos travados, ARRASTAR não muda os anéis e pede o modal; sem trava, o
/// mesmo arrasto muda. *Mutação: o `move_ponto` sem `recusa` ⇒ os anéis mudam, vermelho.*
#[test]
fn pontos_animados_com_auto_key_off_o_arrasto_nao_muda_e_pede_o_modal() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let inicio = outline(&sim, shape);
    trava(&mut sim, shape);
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    {
        let mut bus = ActionBus::default();
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(de), Teclas::default()), "agarrou");
        assert!(p.move_(ecra([de[0], de[1] + 1.0]), Teclas::default()));
        assert!(p.move_(ecra([de[0], de[1] + 1.5]), Teclas::default()));
        assert!(p.sobe());
    }
    assert_eq!(outline(&sim, shape), inicio, "travado: nada muda");
    assert_eq!(pedido(&mut sim), Some(shape.to_bits()), "pede o modal");
    assert_eq!(pedido(&mut sim), None, "abre uma vez");
    ph2d_shape_handles::travados::publica(sim.world_mut(), Default::default(), false);
    arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
    assert_ne!(outline(&sim, shape), inicio, "sem trilha: o arrasto muda");
    assert_eq!(pedido(&mut sim), None);
}

/// ⭐ Regra B: acrescentar na aresta, Alt+clique e Delete não mudam nada e pedem o modal.
/// *Mutação: cada `recusa` tirada (aresta · Alt · Delete) ⇒ os anéis mudam, vermelho.*
#[test]
fn pontos_animados_acrescentar_tirar_e_delete_nao_mudam_e_pedem_o_modal() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let inicio = outline(&sim, shape);
    trava(&mut sim, shape);
    let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
    let r = &locais(&pose, &m, &o)[0];
    let (a, b) = (r[0], r[1]);
    let meio = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let alt = Teclas {
        alt: true,
        ..Teclas::default()
    };
    for (s, t) in [(meio, Teclas::default()), (a, alt)] {
        let mut bus = ActionBus::default();
        assert!(porta(&mut sim, &mut bus).desce_com(ecra(s), t), "consumido");
        assert_eq!(outline(&sim, shape), inicio, "travado: nada muda ({s:?})");
        assert_eq!(pedido(&mut sim), Some(shape.to_bits()), "pede o modal");
    }
    sim.world_mut().insert_resource(PontosNoCanvas {
        arrasto: None,
        selecionados: vec![ph2d_shape_handles::pontos::Ponto { anel: 0, i: 0 }],
    });
    let mut bus = ActionBus::default();
    assert!(porta(&mut sim, &mut bus).apaga(), "a tecla é do canvas");
    assert_eq!(outline(&sim, shape), inicio, "Delete travado: nada muda");
    assert_eq!(pedido(&mut sim), Some(shape.to_bits()));
}

/// ⭐ Regra B: «Merge» e o retraço (Detail dentro da edição) não mudam nada e pedem o modal.
/// *Mutação: `outline_edited` sem a trava ⇒ devolve o contorno novo, vermelho.*
#[test]
fn pontos_animados_merge_e_detail_nao_mudam_e_pedem_o_modal() {
    use ph2d_editor_core::OutlineEdit;
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    trava(&mut sim, shape);
    for e in [OutlineEdit::Merge(64.0), OutlineEdit::Detail(8.0)] {
        assert_eq!(
            ph2d_physics_outline::outline_edited(sim.world_mut(), shape, e),
            None,
            "travado: nada a gravar ({e:?})"
        );
        assert_eq!(pedido(&mut sim), Some(shape.to_bits()), "pede o modal");
    }
    ph2d_shape_handles::travados::publica(sim.world_mut(), Default::default(), false);
    assert!(
        ph2d_physics_outline::outline_edited(sim.world_mut(), shape, OutlineEdit::Detail(8.0))
            .is_some(),
        "sem trilha: o pedido grava-se"
    );
}

/// Uma trilha do ponto 0 da forma (FISICA 51), como a timeline a guarda.
fn trilha(tl: &mut ph2d_timeline::TimelineState, shape: ph2d_ecs::Entity) {
    let kk = ph2d_ecs::FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        ph2d_ecs::canal_fisica::CONTORNO_PONTO_X,
        0,
        0,
    );
    tl.doc.upsert_key(
        shape.to_bits(),
        ph2d_timeline::PropKind::Field(kk),
        ph2d_anim::RationalTime::ZERO,
        ph2d_anim::AnimValue::Float(0.0),
        ph2d_anim::Interp::Linear,
    );
}

/// ⭐ Regra B pelo caminho da shell (`regra_b` por quadro, o modal, `apagar_num_passo`): com trilha e Auto Key
/// off o arrasto não muda e o quadro seguinte abre a pergunta; «Delete animation» apaga a trilha num passo
/// ligado ao passo global (um Ctrl+Z devolve-a) e o arrasto seguinte muda; «Cancel» (nada) deixa tudo travado.
/// Com o Auto Key ligado nada trava. *Mutações: `regra_b` sem publicar ⇒ o arrasto muda, vermelho;
/// `apagar_num_passo` sem `apagar_pontos` ⇒ a trilha fica, vermelho.*
#[test]
fn regra_b_pelo_quadro_da_shell_trava_pergunta_e_delete_animation_desfaz_num_ctrl_z() {
    use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};
    use ph2d_physics_outline::canais_pontos::{animados, apagar_num_passo, regra_b};
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let inicio = outline(&sim, shape);
    let mut tl = ph2d_timeline::TimelineState::new();
    tl.flags.auto_key = false;
    trilha(&mut tl, shape);
    let mut loja = WidgetStore::default();
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    let pergunta = |l: &WidgetStore| l.context_menu().map(|r| r.kind);
    // Cancel = fechar sem nada: o quadro seguinte continua travado.
    for _ in 0..2 {
        regra_b(sim.world_mut(), &tl, &mut loja);
        arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
        assert_eq!(outline(&sim, shape), inicio, "travado: nada muda");
        regra_b(sim.world_mut(), &tl, &mut loja);
        assert_eq!(
            pergunta(&loja),
            Some(ContextMenuKind::ConfirmDeletePointsAnimation {
                shape: shape.to_bits()
            }),
            "o quadro seguinte abre a pergunta"
        );
        loja.close_context_menu();
    }
    assert!(animados(&tl.doc, shape.to_bits()), "Cancel deixa a trilha");
    // Auto Key ligado: mover destrava; o número continua travado.
    tl.flags.auto_key = true;
    regra_b(sim.world_mut(), &tl, &mut loja);
    assert!(!ph2d_shape_handles::travados::travado(
        sim.world(),
        shape.to_bits()
    ));
    assert!(ph2d_shape_handles::travados::numero_travado(
        sim.world(),
        shape.to_bits()
    ));
    tl.flags.auto_key = false;
    // Delete animation: um passo da timeline à espera do global.
    let antes = tl.doc.clone();
    assert!(apagar_num_passo(&mut tl, shape.to_bits()), "apagou");
    assert!(!animados(&tl.doc, shape.to_bits()));
    assert!(
        tl.history.liga_gesto_da_cena().is_some(),
        "o passo global da shell (`empurra`) liga-se a ele"
    );
    regra_b(sim.world_mut(), &tl, &mut loja);
    arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
    assert_ne!(outline(&sim, shape), inicio, "sem trilha: o arrasto muda");
    assert!(tl.undo(), "o Ctrl+Z leva a timeline junto");
    assert_eq!(tl.doc, antes, "a trilha volta");
    assert!(!apagar_num_passo(&mut tl, 999), "sem trilha: nenhum passo");
}

/// ⭐ Regra B estendida (dono 09/11): com trilha de pontos e o Auto Key LIGADO, todo gesto que muda o NÚMERO de
/// pontos (acrescentar na aresta, Alt+clique, Delete, fundir ao soltar, «Merge», o retraço do Detail) não muda os
/// anéis e o quadro seguinte (`regra_b`) abre a mesma pergunta; MOVER um ponto continua livre (o Auto Key grava a
/// chave) e não pergunta nada. Sem trilha, acrescentar muda. *Mutações (uma por porta): aresta · Alt · Delete ·
/// fusão · `outline_edited` com a trava do arrasto em vez da do número ⇒ os anéis mudam, vermelho.*
#[test]
fn pontos_animados_com_auto_key_on_mudar_o_numero_de_pontos_nao_muda_e_pede_o_modal() {
    use ph2d_editor_core::OutlineEdit;
    use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};
    use ph2d_physics_outline::canais_pontos::regra_b;
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let mut tl = ph2d_timeline::TimelineState::new();
    tl.flags.auto_key = true;
    trilha(&mut tl, shape);
    let mut loja = WidgetStore::default();
    let pede = ContextMenuKind::ConfirmDeletePointsAnimation {
        shape: shape.to_bits(),
    };
    let mut pergunta = |sim: &mut SimWorld, tl: &ph2d_timeline::TimelineState| {
        regra_b(sim.world_mut(), tl, &mut loja);
        let k = loja.context_menu().map(|r| r.kind);
        loja.close_context_menu();
        k
    };
    assert_eq!(pergunta(&mut sim, &tl), None);
    let n = |sim: &SimWorld| outline(sim, shape).rings[0].len();
    // Mover: livre, sem pergunta.
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    let inicio = outline(&sim, shape);
    arrasta(&mut sim, de, [de[0], de[1] + 0.5]);
    assert_ne!(outline(&sim, shape), inicio, "Auto Key ligado: mover muda");
    assert_eq!(pergunta(&mut sim, &tl), None, "mover não pergunta");
    let inicio = outline(&sim, shape);
    // Acrescentar na aresta e Alt+clique.
    let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
    let r = &locais(&pose, &m, &o)[0];
    let (a, b) = (r[0], r[1]);
    let meio = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let alt = Teclas {
        alt: true,
        ..Teclas::default()
    };
    for (s, t) in [(meio, Teclas::default()), (a, alt)] {
        let mut bus = ActionBus::default();
        assert!(porta(&mut sim, &mut bus).desce_com(ecra(s), t), "consumido");
        porta(&mut sim, &mut bus).sobe();
        assert_eq!(outline(&sim, shape), inicio, "o número travado ({s:?})");
        assert_eq!(pergunta(&mut sim, &tl), Some(pede), "pede o modal ({s:?})");
    }
    // Delete.
    sim.world_mut().insert_resource(PontosNoCanvas {
        arrasto: None,
        selecionados: vec![ph2d_shape_handles::pontos::Ponto { anel: 0, i: 0 }],
    });
    let mut bus = ActionBus::default();
    assert!(porta(&mut sim, &mut bus).apaga(), "a tecla é do canvas");
    assert_eq!(outline(&sim, shape), inicio, "Delete travado");
    assert_eq!(pergunta(&mut sim, &tl), Some(pede));
    sim.world_mut().insert_resource(PontosNoCanvas::default());
    // Fundir ao soltar: o arrasto move (Auto Key), mas soltar sobre o vizinho não tira o ponto.
    // (No buraco: tirar um canto dele deixa um triângulo válido.)
    let furo = |sim: &SimWorld| outline(sim, shape).rings[1].len();
    let n0 = furo(&sim);
    let canto = perto(&sim, shape, 1, [1.0, 1.0]);
    let viz = perto(&sim, shape, 1, [1.0, -1.0]);
    arrasta(&mut sim, viz, canto);
    assert_eq!(furo(&sim), n0, "soltar sobre o canto não fundiu");
    assert_eq!(pergunta(&mut sim, &tl), Some(pede), "a fusão pede o modal");
    // «Merge» e o retraço do Detail.
    for e in [OutlineEdit::Merge(64.0), OutlineEdit::Detail(8.0)] {
        assert_eq!(
            ph2d_physics_outline::outline_edited(sim.world_mut(), shape, e),
            None,
            "o número travado ({e:?})"
        );
        assert_eq!(pergunta(&mut sim, &tl), Some(pede), "pede o modal ({e:?})");
    }
    // Sem trilha: acrescentar muda.
    let livre = ph2d_timeline::TimelineState::new();
    assert_eq!(pergunta(&mut sim, &livre), None);
    let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
    let r = &locais(&pose, &m, &o)[0];
    let meio = [(r[0][0] + r[1][0]) * 0.5, (r[0][1] + r[1][1]) * 0.5];
    let antes = n(&sim);
    let mut bus = ActionBus::default();
    assert!(porta(&mut sim, &mut bus).desce_com(ecra(meio), Teclas::default()));
    porta(&mut sim, &mut bus).sobe();
    assert_eq!(n(&sim), antes + 1, "sem trilha: acrescenta");
}
