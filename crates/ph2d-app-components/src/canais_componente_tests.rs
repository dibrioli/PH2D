//! Os gates da [`super`] — a família COMPONENTE de ponta a ponta sobre a câmera.

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_ecs::GameCamera;
use ph2d_ecs::canal_componente::chave;

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
        let esc = escrever(&reg, sim.world_mut(), e, kk, x, &t).expect("mudou");
        assert_ne!(esc.antes, esc.depois);
        assert_eq!(ler(&reg, sim.world(), e, kk, &mut t), Some(x), "{kk:?}");
    }
    let c = sim.world().get::<GameCamera>(e).expect("câmera");
    assert_eq!(
        (c.height_world, c.offset[1], c.priority, c.active),
        (12.5, -3.0, 4, false)
    );
    assert!(
        escrever(&reg, sim.world_mut(), e, k(0, 0), 12.5, &t).is_none(),
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
    assert!(aplicar(&mut sim, &reg, &mut drive));
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
    assert!(aplicar(&mut sim, &reg, &mut drive));
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
    publicar(&mut sim, &reg, &[e.to_bits()]);
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
    assert!(aplicar(&mut sim, &reg, &mut PreviewDrive::default()));
    let segue = sim.world().get::<ph2d_ecs::CameraFollow>(e).expect("segue");
    assert_eq!(segue.target, "Heroi");
    publicar(&mut sim, &reg, &[e.to_bits()]);
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
