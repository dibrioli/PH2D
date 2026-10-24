//! **As etapas da família «mundo»** (W19 §5): uma trilha feita à mão — um passo que põe `Collider` e
//! `RigidBody` no «Player» e um «Simular» de 120 tiques — dá as duas etapas com o texto do painel; e
//! apagar uma delas recusa-se pelo nome.

use ph2d_app_macros::Cena;
use ph2d_app_macros::mundo::formato::{Componente, Evento, Inicial, Linha, Mudanca, Objeto, Quem, Simula};
use ph2d_ecs::{Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::{CampoDaEtapa, TipoDaEtapaView};
use ph2d_editor_core::macro_etapas::EtapasDaFamilia;
use ph2d_editor_core::macro_etapas::erros;
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_timeline::TimelineState;

const DONO: u64 = 10;

fn nome(n: &str) -> Componente {
    Componente {
        nome: "ph2d::ecs::Name".into(),
        bytes: postcard::to_allocvec(&ph2d_ecs::Name(n.into())).expect("postcard"),
    }
}

fn doc() -> Vec<u8> {
    let inicial = Inicial {
        objetos: vec![
            Objeto { id: 0, original: DONO, quem: Quem::Macro },
            Objeto { id: 1, original: 11, quem: Quem::Mundo },
        ],
        linhas: vec![
            Linha { id: 0, pai: None, componentes: vec![] },
            Linha { id: 1, pai: None, componentes: vec![nome("Player")] },
        ],
        recursos: vec![],
    };
    let passo = Evento::Passo {
        novos: vec![],
        mudancas: vec![Mudanca::Muda {
            id: 1,
            pai: None,
            componentes: vec![
                ("ph2d::physics::Collider".into(), Some(vec![1])),
                ("ph2d::physics::RigidBody".into(), Some(vec![2])),
            ],
        }],
        recursos: vec![],
    };
    let simula = Evento::Simula(Simula {
        de: 0,
        ate: 120,
        fita: ph2d_physics_ecs::TapeWire::default(),
        conferencia: vec![],
        obstaculos: [0; 32],
    });
    let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let t = rec.open_track(ph2d_macro::codec::FAMILIA_MUNDO, DONO, inicial.escreve());
    rec.push(t, 1_000_000, passo.escreve()).expect("tempo");
    rec.push(t, 2_000_000, simula.escreve()).expect("tempo");
    ph2d_macro::codec::write(&rec.finish())
}

#[test]
fn as_etapas_do_mundo_dizem_o_que_mudou() {
    let v = ph2d_app_macros::etapas::vistas(&doc(), &[]);
    assert_eq!(v.erro, None);
    assert_eq!(v.etapas.len(), 2, "uma etapa por evento: {:?}", v.etapas);
    assert_eq!(
        v.etapas[0].tipo,
        TipoDaEtapaView::Componentes {
            objeto: Some("Player".into()),
            postos: vec!["Collider".into(), "RigidBody".into()],
            mudados: vec![],
            tirados: vec![],
            outros: 0,
        }
    );
    assert_eq!(v.etapas[1].tipo, TipoDaEtapaView::Simula { segundos: 2.0 });
    assert_eq!((v.etapas[1].t0_s, v.etapas[1].t1_s), (2.0, 4.0));
    let textos: Vec<String> = v.etapas.iter().map(ph2d_panel_macros::nomes::nome).collect();
    assert_eq!(
        textos,
        ["Collider, RigidBody added \u{b7} Player", "Simulate \u{b7} 2.00 s"]
    );
}

#[test]
fn apagar_uma_etapa_do_mundo_recusa_se_pelo_nome() {
    let bytes = doc();
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((
            StableId(DONO),
            Macro {
                doc: bytes.clone(),
                modo: ModoDaMacro::Fiel,
            },
        ))
        .id()
        .to_bits();
    let mut tl = TimelineState::default();
    let mut intents = Vec::new();
    let mut c = Cena {
        sim: &mut sim,
        timeline: &mut tl,
        intents: &mut intents,
        partidas: None,
    };
    let r = ph2d_app_macros::etapas::apaga(&mut c, &[], bits, 0, 0);
    assert_eq!(r.map_err(|e| e.chave), Err(erros::MUNDO_FIXO));
    // Trocar o conteúdo também; a espera serve.
    let r = ph2d_app_macros::etapas::edita(&mut c, &[], bits, 0, 0, &CampoDaEtapa::Valor(1.0));
    assert_eq!(r.map_err(|e| e.chave), Err(erros::MUNDO_FIXO));
    // Passar à frente do «Simular» (t0 = 2 s) recusa-se; ficar antes dele serve.
    let r = ph2d_app_macros::etapas::edita(&mut c, &[], bits, 0, 0, &CampoDaEtapa::Instante(3.0));
    assert_eq!(r.map_err(|e| e.chave), Err(erros::MUNDO_ORDEM));
    let doc_de = |c: &Cena<'_>| {
        c.sim
            .world()
            .get::<Macro>(ph2d_ecs::Entity::from_bits(bits))
            .expect("macro")
            .doc
            .clone()
    };
    assert_eq!(doc_de(&c), bytes, "uma recusa não escreve nada");
    ph2d_app_macros::etapas::edita(&mut c, &[], bits, 0, 0, &CampoDaEtapa::Instante(1.5))
        .expect("antes do Simular cabe");
    ph2d_app_macros::etapas::edita(&mut c, &[], bits, 0, 1, &CampoDaEtapa::Espera(0.5))
        .expect("a espera serve");
    let v = ph2d_app_macros::etapas::vistas(&doc_de(&c), &[]);
    assert_eq!(v.erro, None);
    assert_eq!(
        v.etapas.iter().map(|e| e.t0_s).collect::<Vec<_>>(),
        [1.5, 2.5]
    );
    let _ = ph2d_app_macros::etapas_mundo::EtapasDoMundo.familia();
}
