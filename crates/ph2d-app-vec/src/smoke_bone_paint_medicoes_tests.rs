//! ⏱️ **AS MEDIÇÕES DA F9 (`--ignored`)** — filho da [`super`] (os gates da assadura) para herdar as
//! fixturas e as portas dela ([`super::posa_a_assada`], [`super::Campo`]).
//!
//! ⚠️ **Nada aqui reprova**: são sondas que imprimem a tabela de que os números do cabeçalho do
//! [`crate::smoke_bone_paint_assada_tests`] e do [`ph2d_skeleton_live::skin_bake`] saíram. *Uma
//! sonda com barra é um gate; um gate sem barra é ruído — e o que os separa é este ficheiro.*

use super::*;

fn maior_canto(m: &ph2d_poly2d::Mesh2d, posed: &[[f64; 2]], zoom: f64) -> f64 {
    silhueta(m, posed, zoom)
        .iter()
        .map(|(_, p)| vai_e_volta(p).2)
        .fold(0.0_f64, f64::max)
}

/// ⏱️ **SONDA (`--ignored`) — a tabela da W0.**
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-vec --lib -- --ignored --nocapture sonda_a_malha_assada
/// ```
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_a_malha_assada_contra_o_refinamento_por_quadro() {
    const DOBRAS: [f32; 5] = [25.0, 60.0, 90.0, 120.0, 150.0];
    const ZOOMS: [f64; 4] = [1.0, 4.0, 8.0, 16.0];
    // A pose e o zoom em que a malha é ASSADA: o pior caso desta varredura.
    const DOBRA_DA_ASSADURA: f32 = 150.0;
    const ZOOM_DA_ASSADURA: f64 = 16.0;

    let cena_de = |graus: f32| {
        cena_dobrada(
            super::super::super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        )
    };

    // ─── O CONTROLO: o bind é o MESMO em todas as dobras ──────────────────────────────────────
    let (sim0, e0) = cena_de(DOBRAS[0]);
    let (sm0, _, _) = campo_da_cena(&sim0, e0);
    let (rest0, pesos0, tris0) = (
        sm0.mesh.rest.clone(),
        sm0.pesos.clone(),
        sm0.mesh.tris.clone(),
    );
    for &g in &DOBRAS[1..] {
        let (sim, e) = cena_de(g);
        let (sm, _, _) = campo_da_cena(&sim, e);
        assert_eq!(
            (sm.mesh.rest.len(), sm.pesos.len(), sm.mesh.tris.len()),
            (rest0.len(), pesos0.len(), tris0.len()),
            "a {g} graus o BIND tem outro tamanho — a sonda compararia duas artes diferentes"
        );
        let pior_r = sm
            .mesh
            .rest
            .iter()
            .zip(&rest0)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0_f64, f64::max);
        let pior_w = sm
            .pesos
            .iter()
            .zip(&pesos0)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        assert!(
            pior_r < 1e-12 && pior_w < 1e-12,
            "a {g} graus o BIND mudou (repouso {pior_r:.3e}, pesos {pior_w:.3e}) — dobrar a cena \
             tem de mexer so' nas POSES dos ossos, senao esta sonda nao compara a mesma arte"
        );
    }
    println!(
        "CONTROLO: o bind e' o mesmo nas {} dobras ({} vertices, {} pecas)\n",
        DOBRAS.len(),
        rest0.len(),
        tris0.len()
    );

    // ─── A ASSADURA: uma vez, no pior caso ────────────────────────────────────────────────────
    let (sim_a, e_a) = cena_de(DOBRA_DA_ASSADURA);
    let (sm_a, p2l_a, pele_a) = campo_da_cena(&sim_a, e_a);
    let ossos = sm_a.ossos();
    let mut ws_a = pele_a.scratch();
    let mut campo_a =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele_a, p2l_a.apply(q), pesos, &mut ws_a);
    let assada = ph2d_skeleton_live::skin_refine::refine_skinned(
        &sm_a.mesh,
        &sm_a.pesos,
        ossos,
        &mut campo_a,
        opcoes(true, ZOOM_DA_ASSADURA),
    );
    let passo = assada
        .attrs
        .len()
        .checked_div(assada.mesh.rest.len())
        .unwrap_or(0);
    println!(
        "ASSADA a {DOBRA_DA_ASSADURA} graus, zoom {ZOOM_DA_ASSADURA}x: {} pecas, {} vertices \
         (bind: {} pecas) | tecto do quadro: {} pecas",
        assada.mesh.tris.len(),
        assada.mesh.rest.len(),
        tris0.len(),
        ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES
    );
    println!(
        "\n{:>5} {:>5} | {:>8} {:>9} {:>8} | {:>8} {:>9} {:>8}",
        "dobra", "zoom", "pf.pecas", "pf.faceta", "pf.canto", "as.pecas", "as.faceta", "as.canto"
    );

    for &graus in &DOBRAS {
        for &zoom in &ZOOMS {
            let (sim, e) = cena_de(graus);
            let (sm, p2l, pele) = campo_da_cena(&sim, e);
            let mut ws = pele.scratch();
            let mut campo =
                |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

            // O produto de hoje: refina a malha do bind NESTA pose, neste zoom.
            let pf = ph2d_skeleton_live::skin_refine::refine_skinned(
                &sm.mesh,
                &sm.pesos,
                ossos,
                &mut campo,
                opcoes(true, zoom),
            );
            let pf_faceta = ph2d_skeleton_live::skin_refine::skinned_deviation(
                &pf.mesh, &pf.posed, &pf.attrs, pf.law, &mut campo,
            ) * PX_POR_METRO
                * zoom;
            let pf_canto = maior_canto(&pf.mesh, &pf.posed, zoom);

            // A assada: a MESMA topologia, posada do repouso e dos pesos.
            let posed = posa_a_assada(&assada.mesh.rest, &assada.attrs, passo, ossos, &mut campo);
            let as_faceta = ph2d_skeleton_live::skin_refine::skinned_deviation(
                &assada.mesh,
                &posed,
                &assada.attrs,
                assada.law,
                &mut campo,
            ) * PX_POR_METRO
                * zoom;
            let as_canto = maior_canto(&assada.mesh, &posed, zoom);

            println!(
                "{graus:>5} {zoom:>5} | {:>8} {pf_faceta:>9.3} {pf_canto:>8.2} | {:>8} \
                 {as_faceta:>9.3} {as_canto:>8.2}",
                pf.mesh.tris.len(),
                assada.mesh.tris.len(),
            );
        }
    }
}

/// ⏱️⏱️ **A W2 DA F9 — QUANTOS OSSOS UM VÉRTICE DE FACTO USA?**
///
/// O número que DIMENSIONA o formato de vértice do *vertex shader* de pele: se um vértice for
/// influenciado por `K` ossos, o vértice carrega `K` índices e `K` pesos, e o resto do buffer é
/// desperdício pago em TODA sprite do app (o [`ph2d_render::QuadVertex`] é partilhado com o quad
/// simples).
///
/// ⚠️ **A pergunta é sobre pesos SIGNIFICATIVOS, não sobre não-zeros:** os pesos BBW são a solução
/// de um problema variacional sobre a arte inteira, logo quase todo vértice tem um resíduo minúsculo
/// em quase todo osso. O que interessa é quantos são precisos para reproduzir a pose **dentro da
/// barra que o produto já promete**.
///
/// `cargo test -p ph2d-app-vec --lib -- --ignored --nocapture quantos_ossos_um_vertice_usa`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn quantos_ossos_um_vertice_usa() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    println!(
        "\n  bind: {} vertices x {ossos} ossos\n",
        sm.mesh.rest.len()
    );
    for corte in [0.0_f64, 1e-6, 1e-4, 1e-3, 1e-2] {
        let mut hist = vec![0usize; ossos + 1];
        let mut pior_resto = 0.0_f64;
        for v in 0..sm.mesh.rest.len() {
            let w = sm.pesos_de(v);
            let n = w.iter().filter(|x| x.abs() > corte).count();
            hist[n] += 1;
            let resto: f64 = w.iter().filter(|x| x.abs() <= corte).map(|x| x.abs()).sum();
            pior_resto = pior_resto.max(resto);
        }
        let dist: Vec<String> = hist
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(n, c)| format!("{n}:{c}"))
            .collect();
        println!(
            "  corte {corte:>8.0e} | ossos por vertice {:<28} | pior peso descartado {pior_resto:.2e}",
            dist.join(" ")
        );
    }
    println!();
}

/// ⏱️ **A ESCADA τ ↔ PEÇAS NA ARTE REAL** — o número que decide se a W1 tem sujeito.
///
/// `cargo test -p ph2d-app-vec --lib -- --ignored --nocapture escada_da_assadura_na_arte_real`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn escada_da_assadura_na_arte_real() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let attrs = ph2d_poly2d::hermite_attrs(&sm.mesh, &sm.pesos, ossos);
    let lei = ph2d_poly2d::AttrLaw::Hermite { values: ossos };
    println!(
        "\n  bind: {} vertices x {ossos} ossos, {} pecas, arte {}x{} px (diagonal {:.0})\n",
        sm.mesh.rest.len(),
        sm.mesh.tris.len(),
        sm.mesh.size[0],
        sm.mesh.size[1],
        f64::from(sm.mesh.size[0]).hypot(f64::from(sm.mesh.size[1]))
    );
    println!("       tau |    pecas | x entrada |   desvio");
    println!("  ---------+----------+-----------+---------");
    for tau in [0.05_f64, 0.02, 0.01, 0.005, 0.002, 0.001] {
        let (m, _, r) = ph2d_poly2d::refine_rest_by_attrs(
            &sm.mesh,
            &attrs,
            ossos * 3,
            lei,
            ph2d_poly2d::RefineOptions {
                tolerance_px: tau,
                max_pieces: sm.mesh.tris.len() * 8,
                adaptativo: true,
            },
        );
        println!(
            "  {tau:>8.4} | {:>8} | {:>8.2}x | {:>8.5}",
            m.tris.len(),
            m.tris.len() as f64 / sm.mesh.tris.len() as f64,
            r.desvio.unwrap_or(f64::NAN)
        );
    }
    println!();
}

fn instancia_da_sonda(s: &ph2d_render::Sprite) -> ph2d_render::RenderInstance {
    ph2d_render::RenderInstance {
        world_pos: [0.0, 0.0],
        size: s.size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: ph2d_render::RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: s.resolve_anchor(PPM),
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: ph2d_render::RenderInstance::pack_flip_flags(s.flip_x, s.flip_y, false),
        z_order: 0,
        sampling: 0,
        uv_xform: ph2d_render::RenderInstance::IDENTITY_UV_XFORM,
        clip_group: ph2d_render::RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// `(mínimo, mediana)` de amostras de relógio, em ms — o MÍNIMO é a leitura que sobrevive à carga
/// de fundo desta máquina (`CLAUDE.md` §5.0); a mediana ao lado diz quanto ela estava a roubar.
fn melhor_ms(mut ms: Vec<f64>) -> (f64, f64) {
    ms.sort_by(f64::total_cmp);
    (ms[0], ms[ms.len() / 2])
}

/// ⏱️⏱️⏱️ **O QUE UM QUADRO CUSTA COM A MALHA ASSADA — a medição que decide se a W2 (a placa) é
/// PRÉ-REQUISITO ou OPTIMIZAÇÃO.**
///
/// A W1 tirou o REFINAMENTO do quadro e pô-lo no bind. O que o quadro ainda paga, por imagem, é
/// **descodificar a malha guardada, deformar cada vértice e montar o `SpriteMesh`** — o `Fast`, a
/// `0,024 µs` por peça (F6-t). A assada é `5,76×` mais densa, logo esse custo multiplica por `5,76`
/// — e é essa multiplicação que esta sonda mede, na ARTE REAL e por número de imagens.
///
/// # ⚠️ A pergunta é de PRODUTO, e nunca foi medida
///
/// O cabeçalho do [`ph2d_skeleton_live::skin_bake`] afirma que a porta tem de nascer DESLIGADA
/// porque *«um interruptor que nasce ligado antes de a W2 pagar por ele poria mais vértices para a
/// CPU deformar por quadro»*. Isso é verdade sobre a DIRECÇÃO e não diz o NÚMERO: se o quadro
/// couber, o dono ganha *«o alisamento em qualquer cena»* HOJE e a W2 passa a ser o que tira esse
/// custo da CPU; se não couber, a W2 é pré-requisito e a porta fica fechada. *§0.0: meça antes de
/// limitar.*
///
/// ⚠️ **As `n` imagens são CÓPIAS do mesmo bind**, componente a componente — a mesma arte, os
/// mesmos ossos, a mesma tabela de pesos. É o modelo honesto de *«um personagem de muitas partes»*
/// para esta medição, porque o custo por imagem é função das peças dela e de mais nada.
///
/// `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture o_que_um_quadro_custa`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn o_que_um_quadro_custa_com_a_malha_assada() {
    use ph2d_ecs::{PresentWorld, SimRef, Transform};
    use ph2d_render::{Sprite, SpriteMesh};
    use ph2d_skeleton_live::skin_image::{SKIN_FRAME_PIECES, attach_skin_meshes};
    use std::time::Instant;

    const QUADRO_MS: f64 = 16.667;
    const RONDAS: usize = 30;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let porta = std::env::var("PH2D_SKIN_BAKE").unwrap_or_else(|_| "<fechada>".to_owned());
    println!(
        "\n  carga: {} · PH2D_SKIN_BAKE={porta}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let (sim0, e0) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim0, e0);
    let entrada_pecas = sm.mesh.tris.len();
    // ⏱️ **O que a ASSADURA custa, uma vez por bind** — ela não entra no quadro, e é por isso que
    // tem de ser medida à parte: um custo pago uma vez não se compara com um pago 60 vezes por
    // segundo, e confundi-los é o que faz um número parecer proibitivo ou barato ao contrário.
    let mut assaduras = Vec::with_capacity(5);
    for _ in 0..5 {
        let t = Instant::now();
        let _ = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, sm.ossos());
        assaduras.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let (assadura_min, assadura_med) = melhor_ms(assaduras);
    let assada_pecas = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, sm.ossos())
        .expect("a assadura parte alguma coisa nesta arte")
        .0
        .tris
        .len();
    println!(
        "  arte {}x{} px · bind {entrada_pecas} pecas · assada {assada_pecas} pecas ({:.2}x) · \
         orcamento do quadro {SKIN_FRAME_PIECES} pecas",
        sm.mesh.size[0],
        sm.mesh.size[1],
        assada_pecas as f64 / entrada_pecas as f64
    );
    println!(
        "  assar UMA vez (por bind, fora do quadro): {assadura_min:.3}/{assadura_med:.3} ms\n"
    );

    println!(
        "  {:>3} | {:>13} | {:>7} | {:>8}",
        "img", "ms (min/med)", "pecas", "% quadro"
    );
    println!("  ----+---------------+---------+---------");

    for n in [1_usize, 4, 8] {
        // ⚠️ **A fonte é a do BIND, sempre** — a assadura chega pelo caminho do PRODUTO (o memo),
        // e não escrita à mão aqui. *Uma sonda que planta o resultado na entrada mede a lei que ela
        // própria escreveu, nunca a que o quadro corre.*
        let (mut sim, e0) = cena(super::super::super::ALTURA_PX, None);
        let bind = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e0)
            .expect("pele do bind")
            .clone();
        let sprite = *sim.world().get::<Sprite>(e0).expect("sprite da arte");
        let mut artes = vec![e0];
        for _ in 1..n {
            let e = sim.world_mut().spawn((Transform::IDENTITY, sprite)).id();
            sim.world_mut().entity_mut(e).insert(bind.clone());
            artes.push(e);
        }
        let mut present = PresentWorld::new();
        let instancias: Vec<_> = artes
            .iter()
            .map(|&e| {
                present
                    .world_mut()
                    .spawn((SimRef(e), instancia_da_sonda(&sprite)))
                    .id()
            })
            .collect();

        // ⚠️ **Uma passagem a MORNO antes do relógio:** a assadura é paga uma vez por bind e já tem
        // coluna própria acima — deixá-la cair dentro da 1.ª ronda mediria as duas coisas somadas e
        // chamaria isso de custo por quadro.
        attach_skin_meshes(&sim, &mut present, PPM, &[]);

        let mut ms = Vec::with_capacity(RONDAS);
        for _ in 0..RONDAS {
            for p in &instancias {
                present.world_mut().entity_mut(*p).remove::<SpriteMesh>();
            }
            let t = Instant::now();
            attach_skin_meshes(&sim, &mut present, PPM, &[]);
            ms.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let saiu: usize = instancias
            .iter()
            .map(|p| {
                present
                    .world()
                    .get::<SpriteMesh>(*p)
                    .map_or(0, |m| m.tris.len())
            })
            .sum();
        let (min, med) = melhor_ms(ms);
        println!(
            "  {n:>3} | {min:>6.3}/{med:<6.3} | {saiu:>7} | {:>6.1} %",
            min / QUADRO_MS * 100.0
        );
    }
    println!();
}

/// ⏱️⏱️⏱️ **ONDE ESTÃO OS `8,16 ms` — a decomposição que decide se a placa é a ÚNICA cura.**
///
/// ⛔⛔ **A nota que PAROU a F9 em 2026-09-17 dizia `1,824 ms` (`10,9 %` de um quadro) a 8 imagens,
/// e a irmã acima lê `8,16 ms` (`49,0 %`) a `load 4,58`** — `4,5 ×`. *«Não se gasta uma wave a
/// comprar 11 % de um quadro que hoje sobra»* era uma afirmação sobre um número, e o número
/// mudou; §0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*.
///
/// ⚠️ **E antes de mover a deformação para a placa há uma pergunta mais barata que NINGUÉM fez:**
/// de que é feito aquele tempo. O [`attach_skin_meshes`] faz, por instância e por quadro, **duas
/// cópias inteiras** (a malha assada e a tabela de pesos) antes de deformar um único vértice — e
/// uma cópia não é lei nenhuma, é o preço de uma assinatura.
///
/// ⇒ esta sonda parte o relógio em três: **CÓPIA** · **a LEI por vértice** · **o resto**. *Um número
/// que não se sabe de que é feito não decide arquitectura nenhuma.*
///
/// ⛔⛔ **E desde que a placa posa (F9 W2, 2026-09-20) a terceira linha é um SUCEDÂNEO e pode sair
/// NEGATIVA — de propósito.** A linha da LEI chama a [`posed_sprite_mesh_corrigida`] **à mão**, que
/// é a REFERÊNCIA e já não o caminho do produto: com a porta aberta o `attach_skin_meshes` não posa
/// vértice nenhum, logo *«todo menos a lei»* subtrai um trabalho que o todo não fez.
/// ⚠️ **A linha que decide é a primeira**, e ela lê-se com a porta nos dois estados.
///
/// **Medido em 2026-09-20** (arte do dono, `--release`, 8 imagens, `load 5,24`):
///
/// | `PH2D_SKIN_GPU` | `attach_skin_meshes` | % de um quadro |
/// |---|---:|---:|
/// | `1` (omissão — a placa posa) | **`2,077 ms`** | **`12,5 %`** |
/// | `0` (a CPU posa) | `5,939 ms` | `35,6 %` |
///
/// ⇒ **`2,86 ×`**, e o que sobra dos `12,5 %` **não é deformação**: é a tabela de pesos a ser
/// derivada por vértice a cada construção de malha. ⏳ *Ela é uma grandeza do BIND* (a quota sai da
/// posição de REPOUSO), logo o memo do payload é o que a tira do quadro — dívida NOMEADA, e o
/// número acima é a medida dela.
///
/// `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture de_que_e_feito`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn de_que_e_feito_o_quadro_da_pele() {
    use ph2d_ecs::{PresentWorld, SimRef, Transform};
    use ph2d_render::{Sprite, SpriteMesh};
    use ph2d_skeleton_live::skin_image::{attach_skin_meshes, posed_sprite_mesh_corrigida};
    use std::time::Instant;

    const QUADRO_MS: f64 = 16.667;
    const RONDAS: usize = 30;
    const IMGS: usize = 8;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  carga: {}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let (mut sim, e0) = cena(super::super::super::ALTURA_PX, None);
    let bind = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e0)
        .expect("pele do bind")
        .clone();
    let sprite = *sim.world().get::<Sprite>(e0).expect("sprite da arte");
    let mut artes = vec![e0];
    for _ in 1..IMGS {
        let e = sim.world_mut().spawn((Transform::IDENTITY, sprite)).id();
        sim.world_mut().entity_mut(e).insert(bind.clone());
        artes.push(e);
    }
    let mut present = PresentWorld::new();
    let instancias: Vec<_> = artes
        .iter()
        .map(|&e| {
            present
                .world_mut()
                .spawn((SimRef(e), instancia_da_sonda(&sprite)))
                .id()
        })
        .collect();
    // Uma passagem a MORNO: a assadura é paga uma vez por bind e tem coluna própria na irmã.
    attach_skin_meshes(&sim, &mut present, PPM, &[]);

    // (1) O TODO — a porta do produto, exactamente como a irmã a mede.
    let mut todo = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        for p in &instancias {
            present.world_mut().entity_mut(*p).remove::<SpriteMesh>();
        }
        let t = Instant::now();
        attach_skin_meshes(&sim, &mut present, PPM, &[]);
        todo.push(t.elapsed().as_secs_f64() * 1e3);
    }

    // As peças de UMA imagem, pelas mesmas portas que o produto usa.
    let assada = ph2d_skeleton_live::skin_bake_cache::assada_da_arte(
        &sim,
        e0,
        &ph2d_skeleton_live::skin_image::skinned_mesh_of(&sim, e0).expect("malha"),
    )
    .unwrap_or_else(|| ph2d_skeleton_live::skin_image::skinned_mesh_of(&sim, e0).expect("malha"));
    let inst = *present
        .world()
        .get::<ph2d_render::RenderInstance>(instancias[0])
        .expect("instancia");
    let rect = [
        0.0,
        0.0,
        f64::from(assada.mesh.size[0]),
        f64::from(assada.mesh.size[1]),
    ];
    let p2l = ph2d_skeleton_live::skin_image::rect_to_quad(&sprite, rect, inst.anchor, inst.size)
        .expect("o mapa da arte");
    let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e0).expect("pele resolvida");
    let correcoes = bind.correcoes_resolvidas();

    // (2) SÓ A CÓPIA — as duas que a assinatura do `posed_sprite_mesh_corrigida` obriga.
    let mut copia = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        let t = Instant::now();
        for _ in 0..IMGS {
            let m = assada.mesh.clone();
            let w = assada.pesos.clone();
            std::hint::black_box((&m, &w));
        }
        copia.push(t.elapsed().as_secs_f64() * 1e3);
    }

    // (3) SÓ A DEFORMAÇÃO — a lei por vértice, sem a varredura e sem o memo.
    let mut deforma = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        let t = Instant::now();
        for _ in 0..IMGS {
            let saiu = posed_sprite_mesh_corrigida(
                assada.mesh.clone(),
                p2l,
                &pele,
                &assada.pesos,
                inst.anchor,
                inst.size,
                &correcoes,
            );
            std::hint::black_box(&saiu);
        }
        deforma.push(t.elapsed().as_secs_f64() * 1e3);
    }

    let (t_min, _) = melhor_ms(todo);
    let (c_min, _) = melhor_ms(copia);
    let (d_min, _) = melhor_ms(deforma);
    let vertices = assada.mesh.rest.len();
    println!(
        "  {IMGS} imagens · {vertices} vertices cada · {} pecas cada",
        assada.mesh.tris.len()
    );
    println!("  {:>26} | {:>8} | {:>8}", "", "ms", "% quadro");
    println!("  ---------------------------+----------+---------");
    for (nome, ms) in [
        ("TODO (attach_skin_meshes)", t_min),
        ("  so' a COPIA (2 clones)", c_min),
        ("  a LEI por vertice na CPU", d_min),
        ("  o TODO menos a LEI", t_min - d_min),
    ] {
        println!(
            "  {nome:>26} | {ms:>8.3} | {:>6.1} %",
            ms / QUADRO_MS * 100.0
        );
    }
    println!(
        "\n  por vertice: TODO {:.1} ns · DEFORMACAO {:.1} ns · COPIA {:.1} ns\n",
        t_min * 1e6 / (vertices * IMGS) as f64,
        d_min * 1e6 / (vertices * IMGS) as f64,
        c_min * 1e6 / (vertices * IMGS) as f64,
    );
}
