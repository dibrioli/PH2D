//! Os gates da [`super`] — a família COMPONENTE de ponta a ponta sobre a câmera.

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_ecs::GameCamera;
use ph2d_ecs::canal_componente::chave;

/// Sem árvore de tags: os campos destes gates não guardam referências.
const SEM: RefNames<'static> = RefNames { tags: None };

const CAMERA: &str = "ph2d::ecs::GameCamera";

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(GameCamera::default()).id();
    (sim, e)
}

fn k(field_id: u16, faixa: u16) -> FieldKey {
    chave(CAMERA, field_id, faixa).expect("cabe")
}

/// A câmera tem os seus seis campos como canais: altura, offset x/y, prioridade, activa, máscara,
/// dolly — e nada de outro componente.
#[test]
fn a_camera_publica_os_seus_campos() {
    let reg = registo();
    let (sim, e) = cena();
    let todos = campos(&reg, sim.world(), e);
    let quer = [
        k(0, 0),
        k(1, 0),
        k(1, 1),
        k(2, 0),
        k(3, 0),
        k(4, 0),
        k(5, 0),
    ];
    for q in quer {
        assert!(todos.contains(&q), "falta {q:?} em {todos:?}");
    }
    assert_eq!(todos.len(), quer.len(), "{todos:?}");
}

/// ⭐ **Cada campo lê o que se escreveu**, pela forma do valor guardado (real, faixa, inteiro,
/// booleano), e a escrita devolve o antes/depois para a pré-visualização.
#[test]
fn cada_campo_le_o_que_escreveu() {
    let reg = registo();
    let (mut sim, e) = cena();
    let mut t = TextosDosCanais::default();
    for (kk, x) in [
        (k(0, 0), 12.5),
        (k(1, 1), -3.0),
        (k(2, 0), 4.0),
        (k(3, 0), 0.0),
        (k(5, 0), 0.25),
    ] {
        let esc = escrever(&reg, sim.world_mut(), e, kk, x, &t, SEM).expect("mudou");
        assert_ne!(esc.antes, esc.depois);
        assert_eq!(
            ler(&reg, sim.world(), e, kk, &mut t, SEM),
            Some(x),
            "{kk:?}"
        );
    }
    let c = sim.world().get::<GameCamera>(e).expect("câmera");
    assert_eq!(
        (c.height_world, c.offset[1], c.priority, c.active),
        (12.5, -3.0, 4, false)
    );
    assert!(
        escrever(&reg, sim.world_mut(), e, k(0, 0), 12.5, &t, SEM).is_none(),
        "o mesmo valor não é escrita"
    );
}

/// ⛔ **A lei do componente vale para a curva** (`sanitize_camera`): uma altura impossível não chega
/// ao componente como foi escrita, e o eco diz o que ficou.
#[test]
fn a_lei_da_camera_vale_para_a_curva() {
    let reg = registo();
    let (mut sim, e) = cena();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(0, 0), -5.0);
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &reg, &mut drive, &TagTree::default()));
    let h = sim
        .world()
        .get::<GameCamera>(e)
        .expect("câmera")
        .height_world;
    assert!(h > 0.0, "a altura negativa passou: {h}");
}

/// ⭐⭐ **A curva chega ao componente e NÃO ao documento**: a captura repõe o autorado.
#[test]
fn a_curva_chega_ao_componente_e_nao_ao_documento() {
    let reg = registo();
    let (mut sim, e) = cena();
    let antes = sim
        .world()
        .get::<GameCamera>(e)
        .expect("câmera")
        .height_world;
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(0, 0), antes + 7.0);
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &reg, &mut drive, &TagTree::default()));
    assert_eq!(
        sim.world()
            .get::<GameCamera>(e)
            .expect("câmera")
            .height_world,
        antes + 7.0
    );
    let vivo = drive.substitute(&mut sim);
    assert_eq!(
        sim.world()
            .get::<GameCamera>(e)
            .expect("câmera")
            .height_world,
        antes,
        "a captura não repôs o autorado"
    );
    PreviewDrive::restore(&mut sim, &vivo);
    publicar(&mut sim, &reg, &[e.to_bits()], &TagTree::default());
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(e.to_bits(), k(0, 0)), Some(antes + 7.0));
}

/// ⭐ **O alvo da câmera é um canal de texto** (F27 B2): o nome entra na tabela da sessão, a curva
/// escreve outro nome pelo índice, e o eco lê-o — o corte para outro alvo num quadro.
#[test]
fn o_alvo_da_camera_anima_por_nome() {
    let reg = registo();
    let mut sim = SimWorld::new();
    let e = (sim.world_mut())
        .spawn((GameCamera::default(), ph2d_ecs::CameraFollow::default()))
        .id();
    let alvo = chave("ph2d::ecs::CameraFollow", 0, 0).expect("cabe");
    assert!(
        campos(&reg, sim.world(), e).contains(&alvo),
        "o alvo não é canal"
    );
    assert!(alvo.info().is_some_and(|i| i.texto() && i.discreto()));
    let mut t = TextosDosCanais::default();
    let heroi = t.indice(Some("Heroi"));
    sim.world_mut().insert_resource(t);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), alvo, heroi);
    assert!(aplicar(
        &mut sim,
        &reg,
        &mut PreviewDrive::default(),
        &TagTree::default()
    ));
    let segue = sim.world().get::<ph2d_ecs::CameraFollow>(e).expect("segue");
    assert_eq!(segue.target, "Heroi");
    publicar(&mut sim, &reg, &[e.to_bits()], &TagTree::default());
    assert_eq!(
        sim.world()
            .resource::<CanaisLidos>()
            .valor(e.to_bits(), alvo),
        Some(heroi)
    );
}

/// ⭐⭐ **Toda opção do catálogo diz uma palavra da tabela** — a chave que o `options!` declara existe
/// (uma chave do cartão renomeada cairia CRUA na coluna do valor, `panel.inspector…`).
#[test]
fn toda_opcao_do_catalogo_diz_uma_palavra_da_tabela() {
    let mut crus = Vec::new();
    let mut n = 0;
    // Todo campo, canal do catálogo ou cedido a uma família à mão (F27 F (d)): o cartão pinta-a.
    for f in ph2d_component_desc::all().flat_map(|d| d.fields) {
        if let ph2d_component_desc::View::Options(o) = f.view {
            for x in o {
                n += 1;
                if ph2d_i18n::tr(x.rotulo) == x.rotulo {
                    crus.push((f.label_key, x.nome, x.rotulo));
                }
            }
        }
    }
    assert!(n >= 133, "o censo do F27 F contou 133 opções; agora {n}");
    assert!(crus.is_empty(), "opções sem palavra: {crus:?}");
}

/// ⭐⭐ **O rebobinar dos scripts não devolve o que a CURVA conduz** (a família do smoke do dono
/// 25/10, F30 §A1): a curva escreve a altura da câmera, a mão mexe-a, a curva volta a escrevê-la — e
/// o rebobinar (parado no início, todo quadro) deixa o valor da curva.
#[test]
fn o_rebobinar_dos_scripts_nao_devolve_o_campo_que_a_curva_conduz() {
    let reg = registo();
    let (mut sim, e) = cena();
    let mut drive = PreviewDrive::default();
    let curva = |sim: &mut SimWorld, drive: &mut PreviewDrive, x: f32| {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(e.to_bits(), k(0, 0), x);
        aplicar(sim, &reg, drive, &TagTree::default());
        drive.fields.observe(sim);
    };
    curva(&mut sim, &mut drive, 12.0);
    sim.world_mut()
        .get_mut::<GameCamera>(e)
        .expect("câmera")
        .height_world = 20.0;
    drive.fields.observe(&sim);
    curva(&mut sim, &mut drive, 15.0);
    drive.fields.release(&mut sim);
    let h = sim
        .world()
        .get::<GameCamera>(e)
        .expect("câmera")
        .height_world;
    assert!((h - 15.0).abs() < 1e-4, "{h}");
}

/// ⭐⭐ **O Angle e o Reach do Ray Sensor animam** (GEF 07): o objeto «Ray Sensor» publica os três campos
/// (Angle, Reach, Layer) e uma curva no Angle chega ao componente em GRAUS, sem passar por um vetor.
///
/// **Mutações que devem sangrar:** voltar o campo 0 do catálogo a `Vec2` (Direction) · trocar o
/// `K::Angle` por `K::Scalar` (mutação 10 de 15/11, que sobrevivia).
#[test]
fn o_angle_e_o_reach_do_raio_animam() {
    const RAIO: &str = "ph2d::physics::RaySensor";
    let reg = registo();
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn(ph2d_physics_ecs::RaySensor::default())
        .id();
    let desc = ph2d_component_desc::desc_for(RAIO).expect("no catálogo");
    assert_eq!(
        desc.fields[0].kind,
        ph2d_component_desc::FieldKind::Angle,
        "o campo 0 é o Angle em GRAUS (o chip deg e a curva em graus)"
    );
    let kr = |f| chave(RAIO, f, 0).expect("cabe");
    let todos = campos(&reg, sim.world(), e);
    for f in 0..4 {
        assert!(todos.contains(&kr(f)), "falta o campo {f} em {todos:?}");
    }
    let mut t = TextosDosCanais::default();
    assert_eq!(
        ler(&reg, sim.world(), e, kr(0), &mut t, SEM),
        Some(-90.0),
        "o Angle de fábrica"
    );
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), kr(0), 45.0);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), kr(1), 3.0);
    assert!(aplicar(
        &mut sim,
        &reg,
        &mut PreviewDrive::default(),
        &TagTree::default()
    ));
    let r = *sim
        .world()
        .get::<ph2d_physics_ecs::RaySensor>(e)
        .expect("raio");
    assert_eq!((r.angle, r.reach), (45.0, 3.0));
}

/// ⭐⭐ **Um canal de REFERÊNCIA lê e escreve pelo NOME** (auditoria C6): o campo guarda a identidade
/// (a `TagId`, o `stable_name_id`) e a curva o índice do nome na tabela — como o script. Lido cru, o
/// hash de 64 bits perdia bits no `f32`; escrito como texto, nada voltava ao componente.
///
/// **Mutações que devem sangrar:** o `escrever` voltar ao caminho do texto · o `ler` devolver o cru.
#[test]
fn um_canal_de_referencia_le_e_escreve_pelo_nome() {
    let reg = registo();
    let mut tags = TagTree::new();
    let enemy = tags.create("Enemy").expect("tag").0;
    let friend = tags.create("Friend").expect("tag").0;
    let names = RefNames { tags: Some(&tags) };
    let mut t = TextosDosCanais::default();
    let mut sim = SimWorld::new();
    // (a) O «Only see» do raio: uma tag.
    let mut raio = ph2d_physics_ecs::RaySensor::default();
    raio.see_tag = enemy;
    let e = sim.world_mut().spawn(raio).id();
    let k = chave("ph2d::physics::RaySensor", 7, 0).expect("cabe");
    let lido = ler(&reg, sim.world(), e, k, &mut t, names).expect("canal");
    assert_eq!(t.texto(lido), Some("Enemy"), "o canal lê o NOME da tag");
    let x = t.indice(Some("Friend"));
    assert!(escrever(&reg, sim.world_mut(), e, k, x, &t, names).is_some());
    let r = sim
        .world()
        .get::<ph2d_physics_ecs::RaySensor>(e)
        .expect("raio");
    assert_eq!(r.see_tag, friend, "a curva escreve a IDENTIDADE da tag");
    // (b) A saída do atalho: um objecto pelo hash do nome, inteiro (64 bits).
    let longe = "Far Away Exit Door";
    sim.world_mut().spawn(ph2d_ecs::Name::new(longe));
    sim.world_mut().spawn(ph2d_ecs::Name::new("Near Door"));
    let hash = ph2d_ecs::stable_name_id(longe);
    assert!(hash > 1 << 24, "o hash não cabe num f32");
    let link = ph2d_physics_ecs::NavLink {
        to: ph2d_ecs::stable_name_id("Near Door"),
        two_way: false,
        teleport: true,
        cost: 0.0,
        on_crossed: String::new(),
    };
    let e = sim.world_mut().spawn(link).id();
    let k = chave("ph2d::physics::NavLink", 0, 0).expect("cabe");
    let lido = ler(&reg, sim.world(), e, k, &mut t, names).expect("canal");
    assert_eq!(t.texto(lido), Some("Near Door"));
    let x = t.indice(Some(longe));
    assert!(escrever(&reg, sim.world_mut(), e, k, x, &t, names).is_some());
    let l = sim
        .world()
        .get::<ph2d_physics_ecs::NavLink>(e)
        .expect("atalho");
    assert_eq!(l.to, hash, "o u64 exacto, sem passar por f32");
    let lido = ler(&reg, sim.world(), e, k, &mut t, names).expect("canal");
    assert_eq!(t.texto(lido), Some(longe), "e lê-se de volta pelo nome");
}
