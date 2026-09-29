//! Os gates da GAVETA e do DESENHO FIEL — ver o cabeçalho de [`super`].
//!
//! ⚠️ A gaveta é `thread_local` e a chave é a entidade: dois testes na mesma thread podem pedir os
//! mesmos bits. É por isso que os gates que contam derivações medem **DIFERENÇAS** de contagem e
//! nunca números absolutos — e é também a prova de que a chave não mente: o conteúdo decide.

use super::*;
use crate::barra_da_cena_tests_support::barra_da_cena_com;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_vec_scene::{VecPath, VecScene};

fn dobra(sim: &mut SimWorld, ossos: &[Entity], graus: f32) {
    for o in ossos.iter().skip(1) {
        sim.world_mut()
            .get_mut::<Transform>(*o)
            .expect("Transform")
            .rotation = graus.to_radians();
    }
}

fn caminho(scene: &VecScene, id: VecPathId) -> VecPath {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("path")
        .clone()
}

const PRODUTO: Leis = Leis {
    curva: true,
    rigido: true,
    campo: true,
    c1: false,
    desenho: true,
};

/// ⭐⭐⭐ **GATE — A GAVETA NÃO MUDA A RESPOSTA: o caminho da cena é a lei de sempre, AO BIT.**
///
/// ⚠️ A régua é a porta antiga ([`ph2d_vec_skin::curva::aplica_pela_curva_com`], que deriva o
/// índice ela própria) sobre a fonte lida à mão — *uma segunda escrita da lei não serviria de
/// régua; a lei antiga, sim.* E corre DUAS vezes: a 2.ª é a que vem da gaveta.
#[test]
fn a_gaveta_devolve_a_lei_de_sempre_ao_bit() {
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(false);
    dobra(&mut sim, &ossos, 60.0);
    let e = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let pele = crate::skin_live::skin_of(&sim, e).expect("pele resolvida");
    let mut esperado = g.path.clone();
    let fecha: &[f64] = if g.valida() { &g.pesos } else { &[] };
    ph2d_vec_skin::curva::aplica_pela_curva_com(
        &pele,
        &mut esperado,
        skin.pesos_do_quadro(fecha),
        &skin.correcoes_resolvidas(),
        true,
        g.campo.as_ref(),
    );
    for vez in 0..2 {
        let _ = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
        assert_eq!(
            caminho(&scene, id).verts,
            esperado.verts,
            "na passagem {vez} o caminho da cena deixou de ser a lei de sempre — a gaveta mudou a \
             RESPOSTA, e ela só pode mudar o relógio"
        );
    }
}

/// ⭐⭐⭐ **GATE — O ÍNDICE NASCE UMA VEZ POR FONTE, e um quadro parado não recalcula nada.**
///
/// As três metades: mover um osso RECALCULA o quadro e NÃO re-indexa; um quadro igual ao anterior
/// não recalcula nada; e mudar o bind SEM mudar a fonte (uma correcção de peso) recalcula sem
/// re-indexar. ⚠️ Contagens por DIFERENÇA — ver o cabeçalho do ficheiro.
#[test]
fn o_indice_nasce_uma_vez_por_fonte_e_o_quadro_parado_e_de_graca() {
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(false);
    dobra(&mut sim, &ossos, 30.0);
    let _ = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    let (i0, q0) = derivados();
    dobra(&mut sim, &ossos, 55.0);
    let _ = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    let (i1, q1) = derivados();
    assert_eq!(
        i1, i0,
        "mover um osso RE-INDEXOU a malha — o índice é do bind, não da pose"
    );
    assert_eq!(
        q1,
        q0 + 1,
        "mover um osso não recalculou o quadro — a gaveta devolveu uma pose velha"
    );
    let _ = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    let (i2, q2) = derivados();
    assert_eq!(
        (i2, q2),
        (i1, q1),
        "um quadro IGUAL ao anterior foi recalculado"
    );
    // A 3.ª metade: o bind muda, a fonte não.
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().get_mut::<SkinBind>(e).expect("pele").law =
        ph2d_skeleton_ecs::SkinLaw::Envelope;
    let _ = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    let (i3, q3) = derivados();
    assert_eq!(i3, i2, "mudar a lei do peso RE-INDEXOU — a fonte é a mesma");
    assert_eq!(
        q3,
        q2 + 1,
        "mudar a lei do peso não recalculou o quadro — a gaveta mentiu"
    );
}

/// ⭐⭐⭐ **GATE — OS NÓS DO ARTISTA FICAM, e o que se vê tem os que precisa.**
///
/// ⛔ A metade do documento é a que protege o modo Node e o ponto novo: o caminho da cena tem de
/// continuar com a contagem da FONTE. A metade do desenho é o que o bake existe para comprar.
#[test]
fn os_nos_do_artista_ficam_e_o_desenho_tem_os_que_precisa() {
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(false);
    dobra(&mut sim, &ossos, 60.0);
    let e = Entity::from_bits(map[&id]);
    let fonte = crate::skinned_mesh::le(&sim.world().get::<SkinBind>(e).expect("pele").source)
        .expect("fonte")
        .path;
    let d = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    let n_fonte = fonte.verts_all().count();
    assert_eq!(
        caminho(&scene, id).verts_all().count(),
        n_fonte,
        "o caminho da cena deixou de ter os nós da fonte — o bake escreveu no DOCUMENTO"
    );
    let visto = d.get(&id).expect("a forma presa tem desenho fiel");
    assert!(
        visto.verts_all().count() > n_fonte,
        "o desenho fiel tem {} nós contra {n_fonte} da fonte — o bake não acrescentou nada",
        visto.verts_all().count()
    );
    // ⚠️ E ele leva o ESTILO VIVO: o que o artista mudou no traço tem de chegar ao que se vê.
    assert_eq!(visto.stroke, caminho(&scene, id).stroke);
    assert_eq!(visto.fill, caminho(&scene, id).fill);
    // O CONTROLO: com a lei desligada, não há desenho.
    let sem = crate::skin_live::recook_leis(
        &sim,
        &mut scene,
        Leis {
            desenho: false,
            ..PRODUTO
        },
    );
    assert!(
        sem.is_empty(),
        "com `desenho: false` ainda saiu desenho — a porta de bissecção mente"
    );
}

/// ⭐⭐ **GATE — QUINAS VIVAS NÃO SE PERCORREM PELOS NÓS, e EFEITOS ficam de fora.**
///
/// Uma quina viva mora no NÓ e o bake re-escreve os nós ⇒ os nós crus não servem, e o bake
/// percorre a fonte JÁ arredondada (o gate de fidelidade é o
/// `skinned_mesh::desenho_tests::uma_forma_com_quinas_vivas_segue_o_padrao_ouro`). Um efeito mora
/// no caminho VIVO e corre sobre a contagem de nós ⇒ essa forma desenha-se como antes.
#[test]
fn quinas_vivas_nao_vao_pelos_nos_e_efeitos_ficam_de_fora() {
    let (_sim, scene, _map, id, _ossos) = barra_da_cena_com(false);
    let base = caminho(&scene, id);
    assert!(
        os_nos_servem(&base) && o_estilo_serve(&base),
        "o CONTROLO: a barra serve"
    );
    let mut quina = base.clone();
    quina.verts[0].corner_radius = 0.2;
    quina.verts[0].kind = ph2d_vec_scene::VertexKind::Corner;
    assert!(
        !quina.has_live_corner() || !os_nos_servem(&quina),
        "uma quina viva foi percorrida pelos NÓS crus — o bake re-escreve os nós e mataria o raio"
    );
    let mut fx = base.clone();
    fx.effects.push(ph2d_vec_scene::effect::FxEntry::new(
        ph2d_vec_scene::effect::PathEffect::Trim(ph2d_vec_scene::fx_trim::TrimSpec::default()),
    ));
    assert!(
        !o_estilo_serve(&fx),
        "um efeito passou pelo bake — ele corre sobre a contagem de nós"
    );
}

/// ⭐⭐ **GATE — A FUSÃO não escreve por cima de outro produtor, e põe o desenho no MUNDO.**
#[test]
fn a_fusao_cede_a_quem_ja_la_estava_e_assa_a_pose() {
    let id: VecPathId = 7;
    let mut p = VecPath::default();
    p.verts.push(ph2d_vec_scene::VecVertex::corner([1.0, 0.0]));
    let mut d = SkinDesenhado::new();
    d.insert(id, p.clone());
    let mut xf = VecXforms::default();
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 10.0, 0.0]));
    let mut vivo: BTreeMap<VecPathId, Vec<VecPath>> = BTreeMap::new();
    funde(&d, &xf, &mut vivo);
    assert_eq!(
        vivo[&id][0].verts[0].anchor,
        [11.0, 0.0],
        "o desenho não foi posto no MUNDO"
    );
    let mut ocupado: BTreeMap<VecPathId, Vec<VecPath>> = BTreeMap::new();
    ocupado.insert(id, Vec::new());
    funde(&d, &xf, &mut ocupado);
    assert!(
        ocupado[&id].is_empty(),
        "a fusão escreveu por cima de outro produtor"
    );
}

/// A repartição das amostras — os extremos medidos e o orçamento.
#[test]
fn a_reparticao_das_amostras_e_a_medida() {
    assert_eq!(
        amostras_por_segmento(8),
        64,
        "a barra de 8 nós é o ponto medido de 64"
    );
    assert_eq!(
        amostras_por_segmento(54),
        16,
        "a fonte de 54 nós é o ponto medido de 16"
    );
    assert_eq!(amostras_por_segmento(0), 64);
    assert_eq!(amostras_por_segmento(1), 64);
}
