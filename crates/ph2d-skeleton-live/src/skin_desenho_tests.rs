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
    com_e_sem_contacto_em(graus, Forma::C)
}

/// A FORMA da dobra: as duas juntas para o mesmo lado (`C`, a de sempre destes gates), em Z (a da
/// cena `=4` do par) ou só a primeira junta (a da foto do dono de 2026-09-29).
#[derive(Clone, Copy, Debug)]
enum Forma {
    C,
    Z,
    Uma,
}

fn com_e_sem_contacto_em(graus: f32, forma: Forma) -> (VecPath, VecPath) {
    let (mut sim, mut scene, _map, id, ossos) = barra_da_cena_com(false);
    for (k, o) in ossos.iter().enumerate().skip(1) {
        let g = match (forma, k) {
            (Forma::C, _) | (Forma::Z | Forma::Uma, 1) => graus,
            (Forma::Z, _) => -graus,
            (Forma::Uma, _) => 0.0,
        };
        sim.world_mut()
            .get_mut::<Transform>(*o)
            .expect("Transform")
            .rotation = g.to_radians();
    }
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

/// 📏 **SONDA — a QUINA do contacto, dobra a dobra** (report do dono de 2026-09-29: *«a depender do
/// ângulo a quina fica inconsistente»*). Para cada vértice da silhueta onde o contorno VIRA, os dois
/// segmentos vizinhos (corda) e as duas alças — um segmento minúsculo junto da quina é o que faz a
/// junção do traço mudar de cara com o ângulo.
#[test]
#[ignore = "sonda — imprime a geometria da quina"]
fn diag_a_quina_do_contacto() {
    let dir = |a: [f64; 2], b: [f64; 2]| {
        let (x, y) = (b[0] - a[0], b[1] - a[1]);
        let l = x.hypot(y);
        (l > 1e-12).then(|| [x / l, y / l])
    };
    for forma in [Forma::C, Forma::Z, Forma::Uma] {
        let mut g = 100.0_f32;
        while g <= 150.0 {
            let (_, com) = com_e_sem_contacto_em(g, forma);
            let vs = &com.verts;
            let n = vs.len();
            for i in 0..n {
                let (p, c, q) = (&vs[(i + n - 1) % n], &vs[i], &vs[(i + 1) % n]);
                let ent = dir(c.in_handle, c.anchor)
                    .or_else(|| dir(p.out_handle, c.anchor))
                    .or_else(|| dir(p.anchor, c.anchor));
                let sai = dir(c.anchor, c.out_handle)
                    .or_else(|| dir(c.anchor, q.in_handle))
                    .or_else(|| dir(c.anchor, q.anchor));
                let (Some(e), Some(s)) = (ent, sai) else {
                    println!("  {g:>5.1}° v{i}: tangente DEGENERADA");
                    continue;
                };
                let vira = (e[0] * s[0] + e[1] * s[1])
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees();
                if vira > 15.0 {
                    let corda = |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
                    println!(
                        "  {forma:?} {g:>5.1}° v{i}/{n}: vira {vira:>6.1}° · corda antes {:.4} depois {:.4} · \
                     alça in {:.4} out {:.4} · kind {:?}",
                        corda(p.anchor, c.anchor),
                        corda(c.anchor, q.anchor),
                        corda(c.in_handle, c.anchor),
                        corda(c.anchor, c.out_handle),
                        c.kind
                    );
                }
            }
            g += 5.0;
        }
    }
}

/// 📏 **SONDA — o contorno DESENHADO, em SVG** (`PH2D_SONDA_DIR`), para se VER a quina: um traço
/// de `0,06` com junta em bico e os vértices marcados.
#[test]
#[ignore = "sonda — escreve SVG"]
fn diag_desenha_a_quina_do_contacto() {
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    for (forma, g) in [
        (Forma::C, 140.0_f32),
        (Forma::Uma, 100.0),
        (Forma::Uma, 120.0),
        (Forma::Z, 115.0),
        (Forma::Z, 130.0),
    ] {
        for (lado, p) in [("sem", 0), ("com", 1)] {
            let par = com_e_sem_contacto_em(g, forma);
            let path = if p == 0 { par.0 } else { par.1 };
            let mut d = String::new();
            let mut cont: Vec<&[ph2d_vec_scene::VecVertex]> = vec![&path.verts];
            cont.extend(path.subpaths.iter().map(|c| c.verts.as_slice()));
            let mut pontos = String::new();
            for vs in cont {
                let n = vs.len();
                for i in 0..=n {
                    let v = &vs[i % n];
                    if i == 0 {
                        d += &format!("M{} {} ", v.anchor[0], -v.anchor[1]);
                    } else {
                        let a = &vs[i - 1];
                        d += &format!(
                            "C{} {} {} {} {} {} ",
                            a.out_handle[0],
                            -a.out_handle[1],
                            v.in_handle[0],
                            -v.in_handle[1],
                            v.anchor[0],
                            -v.anchor[1]
                        );
                    }
                    if i < n {
                        pontos += &format!(
                            "<circle cx='{}' cy='{}' r='0.015' fill='red'/>",
                            v.anchor[0], -v.anchor[1]
                        );
                    }
                }
                d += "Z ";
            }
            let c = diagonal(&path);
            let _ = c;
            let svg = format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-10 -6 10 8' width='1200' \
                 height='960'><rect x='-10' y='-6' width='10' height='8' fill='#606060'/>\
                 <path d='{d}' fill='#e6aa5a' stroke='#783f15' stroke-width='0.06' \
                 stroke-linejoin='miter' fill-rule='nonzero'/>{pontos}</svg>"
            );
            std::fs::write(format!("{dir}/quina_{forma:?}_{g}_{lado}.svg"), svg).expect("svg");
        }
    }
}

/// 📏 SONDA — os vértices do desenho (com e sem contacto) numa dobra de UMA junta, com a viragem.
#[test]
#[ignore = "sonda — lista vértices"]
fn diag_lista_os_vertices_da_quina() {
    let g: f32 = std::env::var("PH2D_SONDA_G")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);
    let forma = match std::env::var("PH2D_SONDA_FORMA").as_deref() {
        Ok("C") => Forma::C,
        Ok("Z") => Forma::Z,
        _ => Forma::Uma,
    };
    let (sem, com) = com_e_sem_contacto_em(g, forma);
    if std::env::var("PH2D_SONDA_EXACTO").is_ok() {
        let directo = ph2d_vec_boolean::resolve_overlap(&sem);
        println!(
            "  DIRECTO: {:?} vértices, igual ao com: {}",
            directo.as_ref().map(|d| d.verts.len()),
            directo.as_ref() == Some(&com)
        );
        if let Some(d) = &directo {
            println!("  DIRECTO último: {:?}", d.verts.last());
        }
    }
    for (nome, p) in [("sem", &sem), ("com", &com)] {
        println!(
            "--- {nome} ({} vértices, {} sub)",
            p.verts.len(),
            p.subpaths.len()
        );
        let n = p.verts.len();
        for i in 0..n {
            let c = &p.verts[i];
            let q = &p.verts[(i + 1) % n];
            if std::env::var("PH2D_SONDA_EXACTO").is_ok() && i + 1 >= n.saturating_sub(1) {
                println!("  EXACTO v{i}: {c:?}\n  EXACTO próximo: {q:?}");
            }
            let vira = ph2d_vec_boolean::overlap::viragem_do_vertice(&p.verts, i).unwrap_or(-1.0);
            println!(
                "  v{i:>2} vira {vira:>6.1} ({:>8.4},{:>8.4}) in ({:>8.4},{:>8.4}) out ({:>8.4},{:>8.4}) → corda {:.4}",
                c.anchor[0],
                c.anchor[1],
                c.in_handle[0] - c.anchor[0],
                c.in_handle[1] - c.anchor[1],
                c.out_handle[0] - c.anchor[0],
                c.out_handle[1] - c.anchor[1],
                (q.anchor[0] - c.anchor[0]).hypot(q.anchor[1] - c.anchor[1])
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — TODA quina da silhueta respeita a junta do painel** (report do dono de
/// 2026-09-29: *«a depender do ângulo a quina fica inconsistente. Faça obedecer ao que foi
/// escolhido no painel»*).
///
/// Nas três formas de dobra (C · Z · uma junta), de `100°` a `150°` de `5` em `5`: nenhum vértice
/// vira mais do que a [`ph2d_vec_boolean::overlap::viragem_maxima`] — logo nenhuma junta `Miter`
/// passa do limite e vira chanfro sem ser pedido — e nenhum segmento cabe na solda (a junta seria
/// calculada sobre uma tangente arbitrária). ⚠️ Com piso de população: alguma dobra da régua TEM de
/// ter passado pela porta, senão o gate varreria desenhos sem contacto e ficaria verde a medir nada.
#[test]
fn toda_quina_da_silhueta_respeita_a_junta_do_painel() {
    let maxima = ph2d_vec_boolean::overlap::viragem_maxima();
    let mut resolvidas = 0;
    for forma in [Forma::C, Forma::Z, Forma::Uma] {
        let mut g = 100.0_f32;
        while g <= 150.0 {
            let (sem, com) = com_e_sem_contacto_em(g, forma);
            if com != sem {
                resolvidas += 1;
            }
            let tol = ph2d_vec_boolean::overlap::SOLDA_DA_QUINA * diagonal(&com);
            let vs = &com.verts;
            let n = vs.len();
            for i in 0..n {
                let (c, q) = (&vs[i], &vs[(i + 1) % n]);
                let longe = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) > tol;
                assert!(
                    longe(c.anchor, q.anchor)
                        || longe(c.anchor, c.out_handle)
                        || longe(c.anchor, q.in_handle),
                    "{forma:?} {g}°: o segmento v{i} cabe na solda — a junta seria calculada \
                     sobre uma tangente arbitrária"
                );
                if let Some(v) = ph2d_vec_boolean::overlap::viragem_do_vertice(vs, i) {
                    assert!(
                        v <= maxima + 1e-9,
                        "{forma:?} {g}°: o vértice v{i} vira {v:.1}° (máx {maxima:.1}°) — uma \
                         junta Miter passaria do limite e sairia em chanfro"
                    );
                }
            }
            g += 5.0;
        }
    }
    assert!(
        resolvidas >= 20,
        "só {resolvidas} dobras passaram pela porta — a régua deixou de conter o contacto"
    );
}
