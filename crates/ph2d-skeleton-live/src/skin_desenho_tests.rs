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
    contacto: true,
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

/// Os quadros da barra da cena dobrada a `graus` — com e sem a silhueta do contacto.
fn com_e_sem_contacto(graus: f32) -> (VecPath, VecPath) {
    let (mut sim, mut scene, _map, id, ossos) = barra_da_cena_com(false);
    dobra(&mut sim, &ossos, graus);
    let sem = crate::skin_live::recook_leis(
        &sim,
        &mut scene,
        Leis {
            contacto: false,
            ..PRODUTO
        },
    );
    let com = crate::skin_live::recook_leis(&sim, &mut scene, PRODUTO);
    (
        sem.get(&id).expect("desenho sem contacto").clone(),
        com.get(&id).expect("desenho com contacto").clone(),
    )
}

/// As dobras da régua — de uma pose calma até ao contacto franco.
const DOBRAS: [f32; 8] = [30.0, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0, 150.0];

/// ⭐⭐⭐ **GATE — NUMA DOBRA FORTE O DESENHO NÃO SE CRUZA: sai a SILHUETA dos membros.**
///
/// ⚠️ **As três metades são obrigatórias:**
/// - o **CONTROLO** (`contacto: false`): alguma dobra da régua CRUZA o contorno — sem isto a
///   barra deixou de conter o fenómeno e o gate passaria sobre uma cena sem contacto;
/// - com a lei, **nenhuma** dobra cruza;
/// - e onde o controlo NÃO cruza, o desenho sai **ao bit** o de antes — é o que prova que a porta
///   só corre no contacto e deixa toda pose calma intacta.
#[test]
fn numa_dobra_forte_o_desenho_nao_se_cruza() {
    let mut cruzou = Vec::new();
    for &g in &DOBRAS {
        let (sem, com) = com_e_sem_contacto(g);
        let sem_cruza = ph2d_vec_boolean::resolve_overlap(&sem).is_some();
        let com_cruza = ph2d_vec_boolean::resolve_overlap(&com).is_some();
        println!("  dobra {g:>5.1}° · sem contacto cruza: {sem_cruza} · com: {com_cruza}");
        assert!(
            !com_cruza,
            "a {g}° o desenho COM a lei do contacto ainda se cruza"
        );
        if sem_cruza {
            cruzou.push(g);
        } else {
            assert_eq!(
                com, sem,
                "a {g}° não há contacto e o desenho mudou — a porta correu fora do contacto"
            );
        }
    }
    assert!(
        !cruzou.is_empty(),
        "o CONTROLO: nenhuma dobra da régua cruza o contorno — a barra deixou de conter o contacto"
    );
}

/// ⭐⭐ **GATE — a silhueta está SOBRE o contorno de antes: ela corta o «olho», não inventa forma.**
///
/// Cada ponto da silhueta tem de estar sobre o desenho sem contacto (à tolerância do achatamento):
/// a fronteira da união de uma região é feita de pedaços da fronteira dela. ⚠️ Sem esta metade, uma
/// porta que devolvesse o CASCO convexo também passaria no gate irmão — ela não se cruza.
#[test]
fn a_silhueta_esta_sobre_o_contorno_de_antes() {
    let (sem, com) = com_e_sem_contacto(*DOBRAS.last().expect("régua"));
    assert!(
        ph2d_vec_boolean::resolve_overlap(&sem).is_some(),
        "o CONTROLO: a dobra mais forte da régua não cruza"
    );
    let amostra = |p: &VecPath| -> Vec<[f64; 2]> {
        let mut out = Vec::new();
        let mut contornos: Vec<Vec<ph2d_vec_scene::VecVertex>> = vec![p.verts.clone()];
        contornos.extend(p.subpaths.iter().map(|c| c.verts.clone()));
        for vs in &contornos {
            let n = vs.len();
            for i in 0..n {
                let (a, b) = (&vs[i], &vs[(i + 1) % n]);
                let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
                for k in 0..32 {
                    let t = f64::from(k) / 32.0;
                    let u = 1.0 - t;
                    let c = |i: usize| {
                        u * u * u * p0[i]
                            + 3.0 * u * u * t * p1[i]
                            + 3.0 * u * t * t * p2[i]
                            + t * t * t * p3[i]
                    };
                    out.push([c(0), c(1)]);
                }
            }
        }
        out
    };
    let antes = amostra(&sem);
    let depois = amostra(&com);
    let diag = diagonal(&sem);
    let pior = depois
        .iter()
        .map(|q| {
            antes
                .windows(2)
                .map(|w| dist_ao_segmento(*q, w[0], w[1]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0_f64, f64::max);
    println!("  pior distância da silhueta ao contorno de antes: {pior:.3e} (diagonal {diag:.3})");
    assert!(
        pior < 2e-3 * diag,
        "a silhueta afasta-se {pior:.3e} do contorno de antes — ela inventou forma"
    );
}

fn dist_ao_segmento(q: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        (((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((q[0] - a[0] - t * dx).powi(2) + (q[1] - a[1] - t * dy).powi(2)).sqrt()
}

/// ⭐ **GATE — o contacto nasce LIGADO**: sem a variável (e com qualquer valor que não `"0"`) a
/// porta está aberta; `"0"` fecha-a. Sem esta régua, trocar o valor de fábrica passaria calado —
/// todo gate acima corre com o `PRODUTO` escrito à mão, nunca com o ambiente.
#[test]
fn o_contacto_nasce_ligado() {
    assert!(
        contacto_de(None),
        "sem a variável o contacto está desligado"
    );
    assert!(contacto_de(Some("1")));
    assert!(
        !contacto_de(Some("0")),
        "`PH2D_SKIN_CONTACTO=0` não o desliga"
    );
}

/// 📏 **SONDA — o preço da porta por quadro** (`--release`): a detecção corre em TODO quadro
/// recalculado, com ou sem contacto; a união só no contacto.
#[test]
#[ignore = "sonda de relógio — corre em --release e com a máquina calma"]
fn diag_o_preco_da_porta_do_contacto() {
    for &g in &[45.0_f32, 90.0, 130.0] {
        let (sem, _) = com_e_sem_contacto(g);
        let n = 200;
        let t = std::time::Instant::now();
        for _ in 0..n {
            std::hint::black_box(ph2d_vec_boolean::resolve_overlap(std::hint::black_box(
                &sem,
            )));
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / f64::from(n);
        println!(
            "  dobra {g:>5.1}° · {} nós · porta {us:>8.1} µs",
            sem.verts_all().count()
        );
    }
}
