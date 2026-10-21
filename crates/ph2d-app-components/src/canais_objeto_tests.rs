//! Os gates da [`super`] — a família OBJETO: ler/escrever pela fronteira do commit, e a curva que
//! chega ao componente sem entrar no documento.

use super::*;

const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Sprite::atlas(0, [1.0, 1.0], BRANCO)))
        .id();
    (sim, e)
}

/// Todo campo publicado volta igual depois de escrito com o que se leu; um valor novo dentro do
/// domínio lê-se de volta.
#[test]
fn cada_campo_le_o_que_escreveu() {
    let (mut sim, e) = cena();
    let todos = campos(sim.world(), e);
    assert_eq!(
        todos.len(),
        2 + 3 + 3 + 4 + 16,
        "skew, emissão/opacidade/silhueta, cores"
    );
    for kk in todos {
        let alvo = if kk.info().is_some_and(|i| i.discreto) {
            1.0
        } else if kk.campo == o::EMISSAO {
            4.0
        } else {
            0.25
        };
        escrever(sim.world_mut(), e, kk, alvo);
        assert_eq!(ler(sim.world(), e, kk), Some(alvo), "{kk:?}");
    }
    let s = sim.world().get::<Sprite>(e).unwrap();
    assert_eq!(
        s.tint,
        [0.25, 0.25, 0.25, 1.0],
        "o alfa do tint é do `Opacity`"
    );
    assert!(s.tint_fill);
    let t = sim.world().get::<Transform>(e).unwrap();
    assert_eq!((t.skew_x, t.skew_y), (0.25, 0.25));
}

/// O domínio é o do commit do Inspector: o skew preso ao `SKEW_LIMIT`, as cores a `0..=1`, a
/// emissão ao tecto, o bool arredonda.
#[test]
fn a_escrita_prende_ao_dominio() {
    let (mut sim, e) = cena();
    let w = sim.world_mut();
    escrever(w, e, k(o::SKEW_X, 0), 10.0);
    assert_eq!(ler(w, e, k(o::SKEW_X, 0)), Some(Transform::SKEW_LIMIT));
    escrever(w, e, k(o::TINTA, 0), 3.0);
    assert_eq!(ler(w, e, k(o::TINTA, 0)), Some(1.0));
    escrever(w, e, k(o::OPACIDADE, 0), -1.0);
    assert_eq!(ler(w, e, k(o::OPACIDADE, 0)), Some(0.0));
    escrever(w, e, k(o::EMISSAO, 0), 1.0e9);
    assert_eq!(ler(w, e, k(o::EMISSAO, 0)), Some(ph2d_ecs::EMISSIVE_MAX));
    escrever(w, e, k(o::SILHUETA, 0), 0.4);
    assert_eq!(ler(w, e, k(o::SILHUETA, 0)), Some(0.0));
    assert!(
        ler(w, e, k(o::CANTOS, o::N_CANTOS)).is_none(),
        "não há quinto canto"
    );
    assert!(ler(w, e, FieldKey::new(OBJETO, o::TINTA, 1, 0)).is_none());
}

/// ⭐ **A ausência é a identidade**: a emissão zero REMOVE o componente (o quadro volta
/// byte-idêntico), e um canto escrito branco numa sprite sem degradê não o cria.
#[test]
fn zero_remove_e_branco_nao_cria() {
    let (mut sim, e) = cena();
    let w = sim.world_mut();
    assert_eq!(ler(w, e, k(o::EMISSAO, 0)), Some(0.0), "ausente lê-se 0");
    escrever(w, e, k(o::EMISSAO, 0), 2.0);
    assert_eq!(w.get::<SpriteEmissive>(e).map(|x| x.0), Some(2.0));
    escrever(w, e, k(o::EMISSAO, 0), 0.0);
    assert!(
        w.get::<SpriteEmissive>(e).is_none(),
        "zero remove o componente"
    );
    escrever(w, e, k(o::CANTOS + 2, 3), 1.0);
    assert!(
        w.get::<SpriteCornerTint>(e).is_none(),
        "branco é a ausência"
    );
    escrever(w, e, k(o::CANTOS + 2, 3), 0.5);
    let ct = w.get::<SpriteCornerTint>(e).copied().unwrap();
    assert_eq!(ct.0[3], [1.0, 1.0, 0.5, 1.0], "o canto BR, o azul");
    assert_eq!(ct.0[0], BRANCO, "os outros cantos ficam brancos");
}

/// ⭐⭐ **Ponta-a-ponta**: a curva escreve em `CanaisEscritos` → `aplicar` muda o componente, o eco
/// fica em `CanaisLidos`, e o AUTORADO (o que o undo e o save fotografam) continua o do artista.
#[test]
fn a_curva_chega_ao_componente_e_nao_ao_documento() {
    let (mut sim, e) = cena();
    let b = e.to_bits();
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        w.escrever(b, k(o::SKEW_Y, 0), 0.5);
        w.escrever(b, k(o::TINTA + 1, 0), 0.25);
        w.escrever(b, k(o::EMISSAO, 0), 3.0);
        w.escrever(b, FieldKey::new(99, 1, 0, 0), 7.0);
    }
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &mut drive));
    let w = sim.world();
    assert_eq!(w.get::<Transform>(e).unwrap().skew_y, 0.5);
    assert_eq!(w.get::<Sprite>(e).unwrap().tint[1], 0.25);
    assert_eq!(w.get::<SpriteEmissive>(e).map(|x| x.0), Some(3.0));
    assert!(
        !w.resource::<CanaisEscritos>().is_empty(),
        "a escrita de outra família fica para quem a drena"
    );
    assert_eq!(
        drive.authored(b, Driver::SolverPose),
        Some(Driven::SolverPose(Transform::default())),
        "o skew autorado é o de antes da curva"
    );
    let Some(Driven::SpriteCor { tint, emissao, .. }) = drive.authored(b, Driver::SpriteCor) else {
        panic!("a cor não foi declarada à pré-visualização");
    };
    assert_eq!((tint, emissao), ([1.0, 1.0, 1.0], None));
    // O eco: o documento ficou o que a curva mandou; a publicação lê-o de volta.
    publicar(&mut sim, &[b]);
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(b, k(o::TINTA + 1, 0)), Some(0.25));
    assert_eq!(l.valor(b, k(o::SKEW_Y, 0)), Some(0.5));
}

/// A fotografia repõe o autorado AUSENTE removendo o componente que a curva criou.
#[test]
fn repor_o_autorado_tira_a_emissao_que_a_curva_criou() {
    let (mut sim, e) = cena();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(o::EMISSAO, 0), 5.0);
    let mut drive = PreviewDrive::default();
    aplicar(&mut sim, &mut drive);
    let vivo = drive.substitute_authored(&mut sim);
    assert!(
        sim.world().get::<SpriteEmissive>(e).is_none(),
        "o autorado não emitia"
    );
    PreviewDrive::restore_live(&mut sim, &vivo);
    assert_eq!(sim.world().get::<SpriteEmissive>(e).map(|x| x.0), Some(5.0));
}

/// Todo campo publicado tem rótulo de grupo e nome de membro traduzidos (o dope-sheet lê-os).
#[test]
fn todo_campo_tem_nome_no_dope_sheet() {
    let (sim, e) = cena();
    for kk in campos(sim.world(), e) {
        let info = kk.info().expect("campo publicado sem vocabulário");
        assert_ne!(ph2d_i18n::tr(info.rotulo), info.rotulo, "{kk:?}");
        let m = kk.membro().expect("membro");
        assert_ne!(ph2d_i18n::tr(m), m, "membro de {kk:?}");
    }
}

/// Tirar a sprite tira os campos dela do que se publicou — e só os desta família: o que outra
/// família publicou da mesma entidade fica.
#[test]
fn tirar_o_componente_esquece_so_os_campos_desta_familia() {
    let (mut sim, e) = cena();
    let b = e.to_bits();
    let alheio = FieldKey::new(ph2d_ecs::canal::familia::VETOR, 1, 0, 0);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisLidos::default)
        .publicar(b, alheio, 7.0);
    publicar(&mut sim, &[b]);
    let tem_tinta = |sim: &SimWorld| {
        sim.world()
            .resource::<CanaisLidos>()
            .valor(b, k(o::TINTA, 0))
            .is_some()
    };
    assert!(tem_tinta(&sim));
    sim.world_mut().entity_mut(e).remove::<Sprite>();
    publicar(&mut sim, &[b]);
    assert!(
        !tem_tinta(&sim),
        "a sprite tirada não deixa a cor publicada"
    );
    assert_eq!(
        sim.world().resource::<CanaisLidos>().valor(b, alheio),
        Some(7.0)
    );
}
