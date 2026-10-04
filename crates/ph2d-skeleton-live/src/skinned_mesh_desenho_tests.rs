//! ⭐⭐⭐ **O DESENHO FIEL contra o padrão-ouro, pela porta do PRODUTO.**
//!
//! Ordem do dono (2026-09-29): *«Nossa deformação de imagens não vetoriais está muito boa. […]
//! Hoje nosso problema é o uso de osso com desenho vetorial. Siga como achar melhor, buscando o
//! padrão ouro.»* ⇒ a régua é o padrão-ouro — a lei aplicada PONTO A PONTO, que é o que a imagem
//! presa desenha — e a fixtura é a barra da cena com os `8` nós que o `Bind` deixa desde
//! 2026-09-20.

use super::ondulacao_regua_tests::{DENSO, ideal_denso};
use super::ouro_reguas_tests::*;

/// O caminho da cena e o desenho fiel dele, pela porta do produto.
///
/// ⚠️ **`contacto` separa as DUAS etapas do que se vê** (F41): a LEI do desenho, que o padrão-ouro
/// julga, e a SILHUETA, que rola a bola por fora do contorno e **arredonda o vinco de propósito**
/// (decisão do dono, 2026-09-29: *«Arredondado»*). Desde que a bola corre sempre, julgar as duas
/// juntas contra o ideal acusaria o arredondamento pedido de ser um defeito da lei.
fn o_que_se_ve(p: &BPalco, contacto: bool) -> (ph2d_vec_scene::VecPath, ph2d_vec_scene::VecPath) {
    let mut sc = p.scene.clone();
    let desenho = crate::skin_live::recook_leis(
        &p.sim,
        &mut sc,
        crate::skin_desenho::Leis {
            c1: false,
            contacto,
            ..crate::skin_desenho::Leis::do_ambiente()
        },
    );
    let cru = sc
        .paths()
        .iter()
        .find(|q| q.id == p.id)
        .expect("path")
        .clone();
    let visto = desenho
        .get(&p.id)
        .cloned()
        .expect("a barra tem desenho fiel");
    (cru, visto)
}

/// A diagonal da caixa das âncoras — a régua da bola ([`ph2d_vec_boolean::overlap::RAIO_DO_VINCO`]).
fn diagonal_de(p: &ph2d_vec_scene::VecPath) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for v in p.verts_all() {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (hi[0] - lo[0]).hypot(hi[1] - lo[1])
}

/// ⭐⭐⭐ **GATE — O QUE SE VÊ SEGUE O PADRÃO-OURO nos 8 nós do artista, em toda a dobra.**
///
/// # As três metades
///
/// 1. ⛔ **O CONTROLO: a lei nos nós está LONGE do ouro** — sem ela a fixtura não conteria o
///    fenómeno que o desenho fiel existe para curar, e as outras metades passariam por vácuo.
/// 2. **O desenho fiel está PERTO**, com a barra tirada do vale medido (tabela no cabeçalho de
///    [`crate::skin_desenho`]).
/// 3. **E é uma melhoria de ORDEM DE GRANDEZA**, não um retoque — a razão é a que decide se a
///    wave compra alguma coisa.
#[test]
fn o_que_se_ve_segue_o_padrao_ouro_nos_nos_do_artista() {
    for graus in [30.0_f32, 60.0, 90.0] {
        let mut p = b_palco(false);
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let rest = b_amostra_com(&p.fonte, DENSO);
        let ouro = ideal_denso(&p, &pele, &rest, false);
        let (cru, visto) = o_que_se_ve(&p, false);
        let (_, c90, cmax) = b_perfil(&b_amostra_com(&cru, DENSO), &ouro);
        let (_, v90, vmax) = b_perfil(&b_amostra_com(&visto, DENSO), &ouro);
        let (_, silhueta) = o_que_se_ve(&p, true);
        let (_, _, smax) = b_perfil(&b_amostra_com(&silhueta, DENSO), &ouro);
        let bola = ph2d_vec_boolean::overlap::RAIO_DO_VINCO * diagonal_de(&silhueta);
        println!("  {graus:>4}°: com a silhueta máx {smax:.5} · a bola {bola:.5}");
        assert!(
            smax <= vmax + bola,
            "a {graus}° a silhueta afasta o que se vê {smax} do padrão-ouro — mais que a lei \
             ({vmax}) e UMA bola ({bola}) juntas"
        );
        println!(
            "  {graus:>4}°: nós do artista p90 {c90:.5} máx {cmax:.5} · desenho fiel ({} nós) p90 \
             {v90:.5} máx {vmax:.5} · {:.0}×",
            visto.verts_all().count(),
            cmax / vmax.max(1e-12)
        );
        if graus >= 60.0 {
            assert!(
                cmax > 0.05,
                "a {graus}° a lei nos nós leu máx {cmax} — a fixtura deixou de conter o fenómeno"
            );
        }
        assert!(
            vmax < 0.01,
            "a {graus}° o desenho fiel afasta-se {vmax} do padrão-ouro (diagonal ~7,07)"
        );
        assert!(
            vmax * 10.0 < cmax.max(1e-3) || cmax < 0.01,
            "a {graus}° o desenho fiel ({vmax}) não é uma ordem de grandeza melhor que a lei nos nós \
             ({cmax})"
        );
    }
}

/// ⭐⭐ **SONDA — O PREÇO POR QUADRO**, com a pose a MUDAR a cada chamada (um quadro parado vem da
/// gaveta e não custa nada). Imprime; corra em `--release` e com a máquina calma.
#[test]
fn diag_o_preco_do_desenho_fiel_por_quadro() {
    use std::time::Instant;
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let mede = |p: &mut BPalco, leis: crate::skin_desenho::Leis| {
        let mut sc = p.scene.clone();
        let t = Instant::now();
        let mut n = 0_u32;
        while t.elapsed().as_millis() < 300 {
            p.dobra_em_s(if n.is_multiple_of(2) { 90.0 } else { 60.0 });
            let _ = crate::skin_live::recook_leis(&p.sim, &mut sc, leis);
            n += 1;
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(n)
    };
    let base = crate::skin_desenho::Leis {
        c1: false,
        ..crate::skin_desenho::Leis::do_ambiente()
    };
    let so_nos = mede(
        &mut p,
        crate::skin_desenho::Leis {
            desenho: false,
            ..base
        },
    );
    let com = mede(&mut p, base);
    let com_c1 = mede(&mut p, crate::skin_desenho::Leis { c1: true, ..base });
    println!(
        "  µs/forma/quadro a mexer: nós {so_nos:.1} · com o desenho fiel {com:.1} · fiel C¹ {com_c1:.1} · loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// A barra da cena com QUINAS VIVAS (a ferramenta de cantos) em vez de nós de tangente — o caso
/// que o bake percorre pela fonte JÁ arredondada ([`crate::skin_desenho`]).
fn palco_com_quinas_vivas() -> BPalco {
    use crate::barra_da_cena_tests_support::{forma, osso};
    use ph2d_skeleton_ecs::SkinBind;
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let mut rect = ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Rectangle,
        [-8.5, 2.0],
        [-1.5, 3.0],
        &[],
    );
    rect.for_each_vert_mut(|v| {
        v.kind = ph2d_vec_scene::VertexKind::Corner;
        v.corner_radius = 0.4;
    });
    let id = scene.push_path(rect);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let passo = (-1.8f64 - -8.2) / 3.0;
    let mut pai = None;
    let mut ossos = Vec::new();
    for k in 0..3 {
        let pos = if k == 0 {
            [-8.2, 2.5]
        } else {
            [passo as f32, 0.0]
        };
        let e = osso(&mut sim, &format!("Bone {}", k + 1), pos, passo, pai);
        pai = Some(e);
        ossos.push(e);
    }
    crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None);
    let alvo = forma(&map, id);
    sim.world_mut()
        .entity_mut(alvo)
        .insert(ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]));
    let skin = sim.world().get::<SkinBind>(alvo).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("a fonte lê-se");
    BPalco {
        sim,
        scene,
        id,
        alvo,
        ossos,
        campo: g.campo.clone().expect("o bind guarda o campo"),
        correcoes: skin.correcoes_resolvidas(),
        fonte: g.path.clone(),
    }
}

/// ⭐⭐⭐ **GATE — UMA FORMA COM QUINAS VIVAS também segue o padrão-ouro.**
///
/// ⛔ O CONTROLO: a fonte TEM quinas vivas (senão a fixtura seria a barra de sempre); e o
/// padrão-ouro é o contorno ARREDONDADO em repouso, deformado ponto a ponto — o que a imagem presa
/// faria.
#[test]
fn uma_forma_com_quinas_vivas_segue_o_padrao_ouro() {
    let mut p = palco_com_quinas_vivas();
    assert!(
        p.fonte.has_live_corner(),
        "a fixtura perdeu as quinas vivas"
    );
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let cozida = p.fonte.cooked().into_owned();
    let rest = b_amostra_com(&cozida, DENSO);
    let ouro = ideal_denso(&p, &pele, &rest, false);
    let (cru, visto) = o_que_se_ve(&p, false);
    let hoje = cru.cooked().into_owned();
    let (_, c90, cmax) = b_perfil(&b_amostra_com(&hoje, DENSO), &ouro);
    let (_, v90, vmax) = b_perfil(&b_amostra_com(&visto, DENSO), &ouro);
    println!(
        "  quinas vivas a 90°: hoje p90 {c90:.5} máx {cmax:.5} · desenho fiel ({} nós) p90 \
         {v90:.5} máx {vmax:.5}",
        visto.verts_all().count()
    );
    assert!(
        visto.verts_all().all(|v| v.corner_size() == 0.0),
        "o desenho fiel ficou com raio de quina — ele seria arredondado OUTRA vez ao desenhar"
    );
    assert!(
        cmax > 0.05,
        "a lei de hoje leu máx {cmax} — a fixtura não contém o fenómeno"
    );
    assert!(
        vmax < 0.01,
        "o desenho fiel afasta-se {vmax} do padrão-ouro"
    );
}

/// ⭐⭐⭐ **GATE — O DESENHO FIEL NÃO ESPETA, em dobra nenhuma.**
///
/// Report do dono (2026-09-29, com foto): *«uma linha anómala no stroke, atravessando a forma»*.
/// Reproduzido na barra da cena em **C a `60°`**: o pior afastamento ao padrão-ouro era **`1,41`**
/// (a peça tem `1` de espessura) contra `≤ 0,004` em toda a outra dobra — uma cúbica aceite pelo
/// `kurbo` com a alça a `9,5` de distância. A causa e a cura estão no `ajusta` da
/// `ph2d_vec_skin::curva_segundo_corpo`, e o gate unitário dela usa as amostras EXACTAS.
///
/// ⚠️ **Aqui é a varredura pela porta do PRODUTO**, e os dois sentidos da dobra, de `10` em `10`
/// graus: um espeto é um acidente de UMA pose, e a pose do report não é a da cena.
///
/// ⚠️ **A barra (`0,1`) fica entre as DUAS coisas que aqui se leem parecidas e não são:** o
/// espeto (`1,41`, UMA cúbica) e a DOBRA do mapa — de `140°` para cima o próprio padrão-ouro
/// cruza-se sobre si (`2` cruzamentos) e o desenho fiel, que é liso por construção, afasta-se dele
/// `0,02`–`0,06` em centenas de amostras junto do cotovelo (pior `0,060` a `150°`). *Esse é o
/// vinco de dentro do cotovelo, que tem outra cura (rotas C/D da pesquisa 04), e não este defeito.*
#[test]
fn o_desenho_fiel_nao_espeta_em_dobra_nenhuma() {
    let mut pior = (0.0_f64, "", 0.0_f32);
    for (lado, s) in [("S", true), ("C", false)] {
        for k in 3..=15 {
            let graus = 10.0 * k as f32;
            let mut p = b_palco(false);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            if s {
                p.dobra_em_s(graus);
            } else {
                p.dobra(graus);
            }
            let pele = p.pele();
            let rest = b_amostra_com(&p.fonte, DENSO);
            let ouro = ideal_denso(&p, &pele, &rest, false);
            let (_, visto) = o_que_se_ve(&p, true);
            let (_, _, vmax) = b_perfil(&b_amostra_com(&visto, DENSO), &ouro);
            if vmax > pior.0 {
                pior = (vmax, lado, graus);
            }
        }
    }
    println!("  pior: {} a {}° — {:.4}", pior.1, pior.2, pior.0);
    assert!(
        pior.0 < 0.1,
        "o desenho fiel afasta-se {:.4} do padrão-ouro em {} a {}° — é a linha que atravessa a forma",
        pior.0,
        pior.1,
        pior.2
    );
}
