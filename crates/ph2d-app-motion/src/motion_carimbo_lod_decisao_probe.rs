//! ⭐⭐⭐ **A DECISÃO DO LOD — se ele arma, e onde** (report do Enio, 2026-09-21). Irmã do
//! [`super`] por responsabilidade: ali mede-se quanto custa; aqui mede-se a ESCOLHA.
//!
//! A tabela que estas sondas produziram está no cabeçalho do irmão, com as duas conclusões: o LOD
//! que já existia movia **ZERO** cópias numa cena `5,6×` acima do joelho dele, e o que shipa arma
//! a partir de `zoom 0,5` (`3 px`) — sem armar a `1,0` (`6 px`), que é onde o artista olha de
//! perto.

use super::super::{carga, fatia, melhor_quente};
use super::{JANELA_PX, cena_do_report};

/// Sonda de diagnóstico: o que o `size` do `source.shape` de facto faz à geometria.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_que_o_size_faz() {
    let _fatia = fatia();
    eprintln!("\n   size pedido | caixa da forma | size da instância");
    eprintln!("  -------------|----------------|-------------------");
    for tam in [1.0f32, 0.5, 0.108, 0.054] {
        let (mut m, saida) = crate::motion_carimbo_probe::monta("grade + carimbo");
        let forma = m
            .doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "source.shape")
            .map(|n| n.id)
            .unwrap();
        m.doc
            .graph
            .set_param(forma, ph2d_node_motion_shape::param::SIZE, tam);
        crate::motion_shape_gen::publish(&mut m, 0.0);
        m.pump.mark_dirty();
        assert!(
            m.pump.pump(
                &m.doc.graph,
                &m.registry,
                &[saida],
                1,
                0.0,
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0]
            ),
            "cozinha"
        );
        let vi = m.pump.vector_instances.first().copied();
        let caixa = vi
            .and_then(|v| m.shape_store.get(v.geometry_id))
            .and_then(|p| {
                ph2d_vec_render::standalone_path_screen_bounds(p, ph2d_vector::Affine::IDENTITY)
            });
        let lado = caixa.map_or(0.0, |c| (c.2 - c.0).max(c.3 - c.1));
        eprintln!("   {tam:>11.3} | {lado:>14.5} | {:?}", vi.map(|v| v.size));
    }
}

/// ⭐⭐⭐ **PERGUNTA 1 — o LOD que JÁ EXISTE arma nesta cena?** A que pode dissolver a wave, e a
/// mais barata de todas. [`apply_object_lod`](crate::motion_bridge::objects::apply_object_lod) move
/// para tiles de GPU toda geometria carimbada mais de `LOD_COUNT` vezes **que tenha tile assada**;
/// a cena tem `90 000` cópias de UMA geometria, ou seja `5,6×` o joelho.
///
/// ⚠️ Se ela imprimir `movidas = 0`, o motor está certo e a POPULAÇÃO dele é que não cobre a forma
/// do dono — que é uma frase sobre arquitectura, não sobre um limiar mal escolhido.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_does_the_existing_lod_ever_arm() {
    let _fatia = fatia();
    let (mut m, _saida) = cena_do_report(0.5);
    let antes = m.pump.vector_instances.len();
    let gid = m.pump.vector_instances.first().map_or(0, |i| i.geometry_id);
    let tile = m.object_bake.tile_texture_for_gid(gid);
    let esperadas = (crate::motion_state::carimbo_demo::LADO_N as usize).pow(2);
    assert_eq!(
        antes, esperadas,
        "controlo: a cena tem de trazer {esperadas}"
    );

    let mut instancias = std::mem::take(&mut m.pump.instances);
    let mut vectores = std::mem::take(&mut m.pump.vector_instances);
    let quads_antes = instancias.len();
    crate::motion_bridge::objects::apply_object_lod(
        &mut instancias,
        &mut vectores,
        &m.object_bake,
        crate::motion_bridge::objects::LOD_COUNT,
    );
    let movidas = quads_antes.abs_diff(instancias.len());

    eprintln!(
        "\n  ═══ O LOD QUE JÁ EXISTE, NA CENA DO REPORT (load {}) ═══\n",
        carga()
    );
    eprintln!("    cópias de UMA geometria ........ {antes}");
    eprintln!(
        "    o joelho (`LOD_COUNT`) ......... {}",
        crate::motion_bridge::objects::LOD_COUNT
    );
    eprintln!(
        "    acima do joelho? ............... {}",
        if antes > crate::motion_bridge::objects::LOD_COUNT {
            "SIM"
        } else {
            "não"
        }
    );
    eprintln!("    tem TILE assada? ............... {tile:?}");
    eprintln!("    movidas para tile .............. {movidas}");
    eprintln!("    ficaram CRISP .................. {}\n", vectores.len());
    eprintln!(
        "  ⇒ {}\n",
        if movidas == 0 {
            "o LOD NÃO arma: a cerca `sem tile fica crisp` manda as 90 000 ao caminho caro"
        } else {
            "o LOD ARMA — a wave é outra"
        }
    );
}

/// ⭐⭐⭐ **PERGUNTA 7 — A PROVA DE PRODUTO: o LOD arma na cena do report, e em que zooms?**
///
/// As seis sondas acima mediram o que ele DEVIA fazer; esta percorre a lei que shipa
/// ([`crate::motion_shape_lod::geometrias_para_lod`]) sobre a cena `=126` inteira, com a câmara
/// nos mesmos zooms da sonda 2. ⚠️ **O CONTROLO é a primeira linha**: a `zoom 4` a estrela mede
/// `24 px` e o LOD **tem** de recusar — se ele armasse ali, a cura teria trocado fidelidade por
/// velocidade onde o artista está a olhar de perto.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_where_the_shipped_lod_arms() {
    let _fatia = fatia();
    let (m, _saida) = cena_do_report(0.5);
    let insts = &m.pump.vector_instances;
    let esperadas = (crate::motion_state::carimbo_demo::LADO_N as usize).pow(2);
    assert_eq!(
        insts.len(),
        esperadas,
        "controlo: a cena tem de trazer {esperadas}"
    );

    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for i in insts {
        let (x, y) = (f64::from(i.world_pos[0]), f64::from(i.world_pos[1]));
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    let centro = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let nasce = f64::from(crate::motion_state::carimbo_demo::PX_POR_UNIDADE);
    let lado_un = f64::from(crate::motion_state::carimbo_demo::TAMANHO) * 2.0;

    eprintln!(
        "\n  ═══ ONDE O LOD QUE SHIPA ARMA, NA CENA DO REPORT (load {}) ═══\n",
        carga()
    );
    eprintln!("    zoom | px/estrela | o LOD quer? |  quads  |  crisp  | Vello | p/ Vello");
    eprintln!("  -------|------------|-------------|---------|---------|-------|----------");
    // ⚠️ **As COLUNAS do diagnóstico, e não só o veredito** — um `não` sozinho manda procurar em
    // três sítios (a contagem · a caixa · a barra), e as colunas resolvem-no numa corrida.
    {
        let gid = insts.first().map_or(0, |i| i.geometry_id);
        let caixa = m.shape_store.get(gid).and_then(|p| {
            ph2d_vec_render::standalone_path_screen_bounds(p, ph2d_vector::Affine::IDENTITY)
        });
        eprintln!(
            "   (gid={gid} · cópias={} · joelho={} · caixa local={caixa:?} · size da 1.ª={:?})",
            insts.len(),
            crate::motion_bridge::objects::LOD_COUNT,
            insts.first().map(|i| i.size),
        );
    }
    for mult in [4.0f64, 2.0, 1.0, 0.5, 0.25, 0.125] {
        let z = nasce * mult;
        let cam = ph2d_vector::Affine::translate((JANELA_PX.0 * 0.5, JANELA_PX.1 * 0.5))
            * ph2d_vector::Affine::scale(z)
            * ph2d_vector::Affine::translate((-centro.0, -centro.1));
        let quer = crate::motion_shape_lod::geometrias_para_lod(
            insts,
            &m.shape_store,
            cam,
            crate::motion_bridge::objects::LOD_COUNT,
        );
        // ⭐ **E o que ele TIRA da cena vectorial** — a partição corrida de verdade, com a tile
        // semeada (o assado é GPU e não corre aqui; o que se mede é o efeito dele).
        let mut crisp = insts.clone();
        let mut quads: Vec<ph2d_render::RenderInstance> = Vec::new();
        let mut bake = crate::motion_shape_bake::ShapeBake::default();
        for gid in &quer {
            bake.seed_for_test(
                *gid,
                crate::motion_shape_bake::ShapeTile {
                    texture_id: 9,
                    world_size: [1.0, 1.0],
                    local_center: [0.0, 0.0],
                },
            );
        }
        crate::motion_shape_lod::aplica_lod_de_forma(&mut quads, &mut crisp, &bake, &quer);
        let encoda = |cena: &mut ph2d_vector::VectorScene| {
            let mut sem_arte = |_: u32, _: [f32; 4]| None;
            crate::motion_shape_gen::encode(
                &crisp,
                &m.shape_store,
                &mut sem_arte,
                cam,
                Some(ph2d_vector::Rect::new(0.0, 0.0, JANELA_PX.0, JANELA_PX.1)),
                ph2d_render::ImageFilterMode::Smooth,
                cena,
            );
        };
        let desenho = melhor_quente(3, encoda);
        let mut cena = ph2d_vector::VectorScene::new();
        encoda(&mut cena);
        let mut r = ph2d_vector::SceneResolver::new();
        let tam = r.resolve(&cena);
        #[expect(clippy::cast_precision_loss, reason = "um tamanho de buffer")]
        let mb = tam.scene_bytes as f64 / 1e6;
        eprintln!(
            "   {mult:>5.3} | {:>10.2} | {:>11} | {:>7} | {:>7} | {desenho:>5.2} ms | {mb:>5.1} MB",
            lado_un * z,
            if quer.is_empty() { "não" } else { "SIM" },
            quads.len(),
            crisp.len(),
        );
    }
    eprintln!(
        "\n  ⚠️ A cena NASCE em `zoom 1,000` ({:.1} px). Descer a coluna é AFASTAR, que é o\n  \
         gesto do report — e é exactamente aí que o LOD passa a armar.\n",
        lado_un * nasce
    );
}
