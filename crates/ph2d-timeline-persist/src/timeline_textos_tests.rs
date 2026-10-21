//! ⭐ A tabela de TEXTOS atravessa o ficheiro de projecto (v22): o [`serialize`] grava-a
//! compactada a partir do recurso da sessão, e o [`upkeep`] do 1.º frame depois do load INTERNA-a
//! no recurso da sessão nova — sem mudar o texto de nenhum índice que ela já tinha dado.

use super::*;
use ph2d_ecs::canal::familia;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::{FieldKey, SimWorld};
use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

const CONTEUDO: FieldKey = FieldKey::new(familia::VETOR, v::TEXTO_CONTEUDO, 0, 0);

fn sessao_com(textos: &[&str]) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("titulo")))
        .id()
        .to_bits();
    let mut t = TextosDosCanais::default();
    for s in textos {
        t.indice(Some(s));
    }
    sim.world_mut().insert_resource(t);
    (sim, e)
}

fn textos_das_chaves(tl: &TimelineState, world: &World) -> Vec<Option<String>> {
    let tabela = world.resource::<TextosDosCanais>();
    let b = tl
        .doc
        .bindings()
        .iter()
        .find(|b| b.prop == PropKind::Field(CONTEUDO));
    let track = tl.doc.active_clip().track(b.unwrap().target).unwrap();
    track
        .keys()
        .iter()
        .map(|k| match k.value {
            ph2d_anim::AnimValue::Float(x) => tabela.texto(x).map(str::to_owned),
            _ => None,
        })
        .collect()
}

#[test]
fn o_texto_de_cada_chave_atravessa_o_ficheiro_e_entra_na_tabela_da_sessao() {
    // Sessão 1: o conteúdo foi escrito tecla a tecla — a tabela tem o rasto todo.
    let (mut sim, e) = sessao_com(&["O", "Ol", "Olá", "Olá m", "lixo"]);
    let mut timeline = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    for (t, texto) in [(0.0, "Olá mundo"), (1.0, "Olá")] {
        let x = sim
            .world_mut()
            .resource_mut::<TextosDosCanais>()
            .indice(Some(texto));
        let i = I::AddKey {
            entity: e,
            prop: PropKind::Field(CONTEUDO),
            t: ph2d_anim::RationalTime::from_seconds(t),
            value: ph2d_anim::AnimValue::Float(x),
            interp: ph2d_anim::Interp::Hold,
        };
        apply_intent(&mut timeline, &mut ph, i);
    }
    upkeep(&mut timeline, sim.world_mut());
    let gravado = textos_das_chaves(&timeline, sim.world());
    let bytes = serialize(&mut timeline, sim.world_mut()).expect("serializa");
    assert!(
        !bytes.windows(4).any(|w| w == b"lixo"),
        "o rasto das teclas não vai para o ficheiro"
    );

    // Sessão 2: outra tabela, com um texto em comum.
    let (mut sim2, _) = sessao_com(&["outro", "Olá"]);
    let mut loaded = install_from_project(&bytes).expect("install");
    let mut no_ficheiro = loaded.doc.textos().to_vec();
    no_ficheiro.sort();
    assert_eq!(
        no_ficheiro,
        ["Olá", "Olá mundo"],
        "só os textos que as chaves usam"
    );
    assert!(!upkeep(&mut loaded, sim2.world_mut()));
    assert!(loaded.doc.textos().is_empty(), "a tabela passou à sessão");
    assert_eq!(textos_das_chaves(&loaded, sim2.world()), gravado);
    let tabela = sim2.world().resource::<TextosDosCanais>();
    assert_eq!(
        tabela.texto(1.0),
        Some("outro"),
        "nenhum índice da sessão muda de texto"
    );
    assert_eq!(tabela.texto(2.0), Some("Olá"));
    assert_eq!(
        tabela.textos().len(),
        3,
        "«Olá» foi reaproveitado, só «Olá mundo» entrou"
    );
}
