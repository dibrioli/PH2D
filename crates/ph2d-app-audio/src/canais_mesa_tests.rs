//! Os gates da [`super`] — a família MESA: a curva chega ao vivo (e ao painel, de onde o motor lê)
//! sem tocar no autorado, e a mão ganha-lhe no quadro em que muda.

use std::collections::BTreeSet;

use super::*;
use crate::mesa_doc;

fn cena_nova() -> SimWorld {
    let mut sim = SimWorld::new();
    ph2d_ecs::garante_cena(sim.world_mut());
    sim
}

fn curva(sim: &mut SimWorld, campo: u16, v: f32) {
    let c = cena(sim).expect("cena");
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(c, k(campo), v);
}

fn lido(sim: &mut SimWorld, campo: u16) -> f32 {
    publicar(sim);
    let c = cena(sim).expect("cena");
    sim.world()
        .resource::<CanaisLidos>()
        .valor(c, k(campo))
        .expect("publicado")
}

/// O painel muda o autorado (o que `mesa_doc::quadro` faz com a edição tomada).
fn mao(sim: &mut SimWorld, f: impl Fn(&mut Mesa)) {
    let mut d = mesa_doc::doc(sim);
    f(&mut d.autorado);
    sim.world_mut().insert_resource(d);
}

/// O vocabulário conta os barramentos do motor.
#[test]
fn o_vocabulario_conta_os_barramentos_do_motor() {
    assert_eq!(usize::from(m::BUSES), ph2d_audio::SUB_BUS_COUNT);
}

/// ⭐⭐ A curva conduz o ganho do SFX: o vivo muda, o painel (de onde o motor lê) recebe-o, o
/// autorado fica — e sem curva volta.
#[test]
fn a_curva_conduz_o_ganho_do_sub_bus() {
    let mut sim = cena_nova();
    let campo = m::bus(1, m::B_GANHO);
    curva(&mut sim, campo, 0.25);
    assert!(aplicar(&mut sim), "o vivo mudou");
    mesa_doc::quadro(&mut sim, None);
    assert!((ph2d_panel_audio_mixer::sub_gain_target()[1] - 0.25).abs() < 1e-6);
    assert!(
        (ph2d_panel_audio_mixer::sub_gain_target()[0] - 1.0).abs() < 1e-6,
        "só o SFX"
    );
    assert_eq!(mesa_doc::autorado(&sim), Mesa::default(), "o autorado");
    // Controlo: sem escrita da curva, a mesa corre o autorado.
    assert!(aplicar(&mut sim));
    mesa_doc::quadro(&mut sim, None);
    assert!((ph2d_panel_audio_mixer::sub_gain_target()[1] - 1.0).abs() < 1e-6);
}

/// ⭐ O que o save grava (`project_save.rs` lê `mesa_doc::autorado`) é o AUTORADO, com uma curva
/// activa a mudar o vivo; o eco diz ao Auto Key que o documento vale a curva.
#[test]
fn o_save_grava_a_mesa_autorada_com_uma_curva_activa() {
    let mut sim = cena_nova();
    mao(&mut sim, |s| s.reverb_tamanho = 0.9);
    aplicar(&mut sim);
    curva(&mut sim, m::REVERB_TAMANHO, 0.1);
    curva(&mut sim, m::DUCKING_CHAVE, 0.7);
    assert!(aplicar(&mut sim));
    let d = mesa_doc::doc(&sim);
    assert!((d.vivo.reverb_tamanho - 0.1).abs() < 1e-6);
    assert_eq!(d.vivo.ducking_chave, 1, "a chave arredonda ao barramento");
    let gravado = Mesa::from_bytes(&mesa_doc::autorado(&sim).to_bytes()).expect("lê");
    assert!(
        (gravado.reverb_tamanho - 0.9).abs() < 1e-6,
        "o save grava a mão"
    );
    assert_eq!(gravado.ducking_chave, Mesa::default().ducking_chave);
    assert!(
        (lido(&mut sim, m::REVERB_TAMANHO) - 0.1).abs() < 1e-6,
        "o eco"
    );
}

/// ⭐ A MÃO ganha à curva no quadro em que muda o autorado — no vivo e no que o Auto Key lê.
#[test]
fn a_mao_ganha_a_curva_na_mesa() {
    let mut sim = cena_nova();
    let pan = m::bus(3, m::B_PAN);
    curva(&mut sim, pan, -0.5);
    aplicar(&mut sim);
    assert!((lido(&mut sim, pan) + 0.5).abs() < 1e-6);
    mao(&mut sim, |s| s.bus[3].pan = 0.8);
    curva(&mut sim, pan, -0.5);
    aplicar(&mut sim);
    assert!(
        (mesa_doc::doc(&sim).vivo.bus[3].pan - 0.8).abs() < 1e-6,
        "o vivo é da mão"
    );
    assert!(
        (lido(&mut sim, pan) - 0.8).abs() < 1e-6,
        "o Auto Key vê a mão"
    );
    // A chave nasceu: a curva volta a conduzir.
    curva(&mut sim, pan, 0.3);
    aplicar(&mut sim);
    assert!((mesa_doc::doc(&sim).vivo.bus[3].pan - 0.3).abs() < 1e-6);
}

/// Todo campo lê o que se escreveu; cada um tem nome traduzido e único.
#[test]
fn cada_campo_da_mesa_le_o_que_escreveu() {
    let mut nomes = BTreeSet::new();
    for campo in m::campos() {
        let mut s = Mesa::default();
        let antes = ler(&s, campo).expect("leitura");
        let discreto = k(campo).info().expect("info").discreto();
        let alvo = match campo {
            m::DUCKING_CHAVE => 0.0,
            _ if discreto => 1.0 - antes,
            _ => antes + 0.125,
        };
        escrever(&mut s, campo, alvo);
        assert_eq!(ler(&s, campo), Some(alvo), "campo {campo}");
        let outros = m::campos().filter(|&o| o != campo);
        for o in outros {
            assert_eq!(
                ler(&s, o),
                ler(&Mesa::default(), o),
                "{campo} escreveu em {o}"
            );
        }
        let rotulo = k(campo).info().expect("info").rotulo;
        let nome = ph2d_i18n::tr(rotulo);
        assert_ne!(nome, rotulo, "{campo} sem tradução");
        assert!(nomes.insert(nome), "dois campos com o nome {nome}");
    }
}
