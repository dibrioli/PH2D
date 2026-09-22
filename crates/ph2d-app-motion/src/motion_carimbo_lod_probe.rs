//! ⭐⭐⭐ **O LOD DA FORMA — a medição ANTES do limiar** (report do Enio, 2026-09-21: *«funciona,
//! contudo ao dar o zoom com as pontas das estrelas arredondadas logo que muitas estrelas
//! apareçam o app trava. Não seria interessante criar um LOD para shapes? De modo que haja uma
//! forte otimização da forma quando ela fica menor na tela?»*).
//!
//! O recorte da câmara (o commit anterior desta linha) corta o que está FORA do ecrã. Ao afastar,
//! tudo entra ⇒ ele deixa de cortar e a conta volta inteira. A pergunta do dono é a seguinte e é
//! outra: **quando a forma fica pequena, ela ainda vale os segmentos que tem?**
//!
//! ⛔⛔ **E há um LOD nesta casa desde 2026-08-05** ([`crate::motion_bridge::objects::apply_object_lod`],
//! `LOD_COUNT = 16 000`) — antes de construir um segundo, estas sondas medem se a composição já o
//! exprime (`CLAUDE.md` §5.0). A leitura do código diz que **não pode armar aqui**, e uma leitura
//! não é uma medição: o LOD troca a forma por uma TILE assada, e quem assa
//! ([`crate::motion_object_bake::ObjectBake::bake`]) varre `select_present(world, map)` — os
//! `VecPathId` do **documento vectorial**. Uma estrela de `source.shape` é um nó de GRAFO: ela não
//! tem `VecPathId` nenhum, logo `tile_texture_for_gid` devolve `None` e a cerca
//! *«sem tile fica crisp»* manda-a de volta ao caminho caro — **em silêncio**, e com toda a razão
//! (a cerca existe para uma tile em falta não apagar a forma).
//!
//! ⚠️ **As três sondas respondem a três perguntas diferentes e nenhuma responde às outras:**
//!   1. [`audit_does_the_existing_lod_ever_arm`] — o LOD que existe arma nesta cena? (a que pode
//!      dissolver a wave inteira, e é a mais barata)
//!   2. [`audit_the_zoom_and_the_screen_size`] — ao afastar, quantas cópias entram, que TAMANHO
//!      cada uma tem no ecrã, e quanto custa o quadro.
//!   3. [`audit_when_the_rounding_stops_showing`] — **a barra**: a que tamanho em píxeis o
//!      arredondamento deixa de ser visível. É daqui que sai o número, nunca de um palpite (§0.0).
//!
//! ## ⭐⭐⭐ O QUE ESTAS SONDAS MEDIRAM (2026-09-22, máquina calma) — leia ANTES de construir
//!
//! **1. O LOD que já existe NÃO ARMA, e a cena está `5,6×` acima do joelho dele:**
//!
//! | cópias de UMA geometria | joelho | tem tile? | movidas | ficaram crisp |
//! |---|---|---|---|---|
//! | `90 000` | `16 000` | **`None`** | **`0`** | `90 000` |
//!
//! **2. Ao AFASTAR, o custo SATURA enquanto a forma encolhe `4×`** (janela `1920×1080`):
//!
//! | zoom | px/estrela | cópias | CPU (desenho+resolver) | p/ a placa |
//! |---|---|---|---|---|
//! | `4,000` | `24,00` | `2 584` | `1,41 ms` | `1,3 MB` |
//! | `1,000` (nasce) | `6,00` | `40 736` | `2,62 ms` | `20,0 MB` |
//! | `0,500` | `3,00` | `90 000` | `4,84 ms` | `44,3 MB` |
//! | `0,250` | `1,50` | `90 000` | `4,99 ms` | `44,3 MB` |
//! | `0,125` | `0,75` | `90 000` | `4,91 ms` | `44,3 MB` |
//!
//! ⇒ *o custo não desce com o tamanho porque o número de SEGMENTOS por forma não depende de
//! quantos píxeis ela ocupa.*
//!
//! **3. ⛔⛔ SIMPLIFICAR A SILHUETA É A CURA ERRADA, e foi a minha 1.ª hipótese.** A régua
//! geométrica (Hausdorff) dizia *«invisível abaixo de `4,4 px`»*; a régua do **PIXEL** — que é a
//! que o olho usa — refuta-a: largar o arredondamento erra **`7` a `255` níveis de 255** e
//! **nunca** fica invisível (a `2 px` ainda são `3,0`; a `1 px`, `1,5`).
//!
//! **4. ⭐⭐⭐ A TILE é a cura certa, e ela é indistinguível abaixo de `4 px`:**
//!
//! | lado no ecrã | TILE reamostrada | silhueta simplificada |
//! |---|---|---|
//! | `32 px` | `64,0` | `255,0` |
//! | `16 px` | `24,3` | `230,8` |
//! | `8 px` | `9,9` | `93,4` |
//! | **`4 px`** | **`0,4`** | `34,9` |
//! | **`2 px`** | **`0,6`** | `10,5` |
//! | **`1 px`** | **`0,2`** | `7,0` |
//!
//! ⚠️ **A barra NÃO é o lado da tile** (`28 px`, que é o que o [`BAKE_DPI`] dá a esta forma): é
//! **`4 px`**, e o que a fixa é a REAMOSTRAGEM — acima dela a tile perde as pontas finas que a
//! cobertura analítica do rasterizador ainda resolve.
//!
//! **5. ⭐⭐ E o assador da tile paramétrica TAMBÉM já existe** ([`crate::motion_shape_bake`], com
//! `tile_quad` e a âncora resolvida). Ele só não corre: a guarda do
//! `fase_vector_fx_recook` é `glows`, com a razão escrita (*«só assa se HOUVER quem consuma o
//! tile»*) ⇒ **a cura é o LOD ser o SEGUNDO consumidor**, não um motor novo.
//!
//! ⚠️⚠️ **E a régua precisou do CONTROLO DELA PRÓPRIA:** com `8×8` amostras por pixel a linha de
//! `1 px` lia `23,9` níveis e com `32×32` lê `1,5` — era ruído de amostragem, e eu já lhe tinha
//! escrito uma explicação (*«é a ÁREA»*) que o controlo desmentiu (o factor de área é `0,99911`:
//! as duas formas têm a MESMA área). *A régua não pode ter a ordem de grandeza do que ela mede.*
//!
//! **6. ⭐⭐⭐ E A PROVA DE PRODUTO: o LOD que shipa arma exactamente onde o custo satura.**
//!
//! | zoom | px/estrela | o LOD quer? | quads | crisp | Vello | p/ Vello |
//! |---|---|---|---|---|---|---|
//! | `4,000` | `24,00` | não | `0` | `90 000` | `1,43 ms` | `1,3 MB` |
//! | `1,000` (nasce) | `6,00` | não | `0` | `90 000` | `2,22 ms` | `20,0 MB` |
//! | **`0,500`** | **`3,00`** | **SIM** | `90 000` | `0` | **`0,00 ms`** | **`0,0 MB`** |
//! | **`0,125`** | `0,75` | **SIM** | `90 000` | `0` | `0,00 ms` | `0,0 MB` |
//!
//! ⚠️ **O custo não DESAPARECE — ele muda de passe.** As `90 000` passam a quads do
//! `SpriteRenderer`, que é instanciado e que o irmão declara ter escalado *«a milhões»*; ⛔ **esse
//! passe não foi medido nesta wave**, e dizer o contrário seria atribuir a esta cura um número que
//! ninguém tirou.
//!
//! ## ⛔⛔⛔ E A FIXTURA MEDIU OUTRO PROGRAMA ATÉ AO FIM — a lição mais cara do dia
//!
//! O arnês [`monta`](crate::motion_carimbo_probe::monta) põe o `kind` e **não o `size`**, logo a
//! estrela nascia no tamanho de FÁBRICA — `14×` maior que a da cena — e o LOD lia `9,5 px` onde a
//! cena tem `0,75`, recusando em toda a coluna. ⚠️⚠️ **É a MESMA armadilha que a sonda irmã já
//! tinha pago** (foi por ela que o `VAO`/`LADO_N` passaram a ser escritos), e ninguém passou o
//! `SIZE`: por isso a cura traz o **CONTROLO** dentro da [`cena_do_report`] — a forma tem de medir
//! no mundo o que a cena pede.
//!
//! ⚠️ E a 1.ª redacção desse controlo **disparou sobre uma fixtura correcta**, porque comparava a
//! caixa LOCAL com um tamanho de MUNDO: medido (`diag_o_que_o_size_faz`), a geometria de um
//! `source.shape` é **normalizada** (caixa `2,00000` para todo `size`) e o tamanho viaja na
//! INSTÂNCIA — ⛔ o oposto do que a doc do assador de tiles afirma por escrito.
//!
//! `cargo test -p ph2d-app-motion --release -- --ignored --nocapture audit_`

/// Monta a cadeia do report **na população da cena `=126`** (`LADO_N²`), com o `corner` pedido.
/// ⚠️ A população é a da CENA e não a do arnês: uma tabela na população errada compara-se com a do
/// dono como se fosse a mesma experiência (a sonda irmã já pagou este defeito).
fn cena_do_report(
    corner: f32,
) -> (
    crate::motion_state::MotionState,
    ph2d_nodegraph::graph::NodeId,
) {
    let (mut m, saida) = crate::motion_carimbo_probe::monta("grade + carimbo");
    let forma = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `source.shape`");
    let grade = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.grid")
        .map(|n| n.id)
        .expect("a cadeia do carimbo tem uma `motion.grid`");
    #[expect(clippy::cast_precision_loss, reason = "o lado da grelha da cena")]
    let lado = crate::motion_state::carimbo_demo::LADO_N as f32;
    m.doc.graph.set_param(grade, "rows", lado);
    m.doc.graph.set_param(grade, "cols", lado);
    m.doc
        .graph
        .set_param(grade, "gap_x", crate::motion_state::carimbo_demo::VAO);
    m.doc
        .graph
        .set_param(grade, "gap_y", crate::motion_state::carimbo_demo::VAO);
    m.doc
        .graph
        .set_param(forma, ph2d_node_motion_shape::param::CORNER, corner);
    // ⛔⛔ **E O TAMANHO, que é o que faz desta a cena `=126` e não outra.** O arnês
    // [`monta`](crate::motion_carimbo_probe::monta) só põe o `kind`, logo a forma nascia no
    // tamanho de FÁBRICA do nó — `12,7×` maior que a da cena — e a coluna `px/estrela` de uma
    // sonda descrevia outro programa. ⚠️ **É a MESMA armadilha que a sonda irmã já pagou** (ela
    // passou a escrever o `VAO`/`LADO_N` por isto) e ninguém passou o `SIZE`: é por isso que a
    // cura vem com o CONTROLO abaixo, e não só com a linha que faltava.
    m.doc.graph.set_param(
        forma,
        ph2d_node_motion_shape::param::SIZE,
        crate::motion_state::carimbo_demo::TAMANHO,
    );
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
            [1.0, 1.0],
        ),
        "o quadro tem de cozinhar"
    );
    // ⭐⭐⭐ **O CONTROLO da fixtura**: a forma que o store guarda tem de MEDIR o que a cena diz.
    // Sem ele, um param que o arnês esquece volta a medir outro programa — e a assinatura disso é
    // silenciosa, porque tudo o resto continua a correr.
    let gid = m.pump.vector_instances.first().map_or(0, |i| i.geometry_id);
    let caixa = m
        .shape_store
        .get(gid)
        .and_then(|p| {
            ph2d_vec_render::standalone_path_screen_bounds(p, ph2d_vector::Affine::IDENTITY)
        })
        .expect("a cena carimba uma forma mensurável");
    // ⭐⭐⭐ **A geometria é NORMALIZADA e o tamanho viaja na INSTÂNCIA** — medido
    // (`diag_o_que_o_size_faz`: a caixa lê `2,00000` para todo `size`, e o `size` da instância é
    // o que o param diz). ⚠️ **A doc do assador de tiles afirma o CONTRÁRIO** (*«a dimensão real
    // está na própria geometria»*), e é por isso que este controlo compara o lado no MUNDO — o
    // produto das duas metades — e não a caixa local contra um tamanho de mundo, que foi a 1.ª
    // redacção e disparou sobre uma fixtura correcta.
    let size = f64::from(m.pump.vector_instances.first().map_or(1.0, |i| i.size[0]));
    let lado = (caixa.2 - caixa.0).max(caixa.3 - caixa.1) * size;
    let pedido = f64::from(crate::motion_state::carimbo_demo::TAMANHO) * 2.0;
    assert!(
        lado > pedido * 0.5 && lado < pedido * 1.2,
        "a forma mede {lado:.5} no mundo e a cena pede {pedido:.5} — o arnês não é a cena"
    );
    (m, saida)
}

use super::{carga, fatia, melhor_quente, segmentos};

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

/// O `VecPath` que a cena carimba, com o `corner` pedido — lido do STORE, que é de onde o desenho
/// o lê. ⚠️ Construir a estrela à mão aqui mediria a minha ideia da forma, não a do produto.
fn forma_da_cena(corner: f32) -> ph2d_vec_scene::VecPath {
    let (m, _saida) = cena_do_report(corner);
    let gid = m.pump.vector_instances.first().map_or(0, |i| i.geometry_id);
    m.shape_store
        .get(gid)
        .expect("a cena carimba uma forma do store")
        .clone()
}

/// A janela do report — o alvo de render, que é contra o que o recorte julga (a lei do commit
/// anterior: o chrome desenha na janela INTEIRA, não numa banda recortada).
const JANELA_PX: (f64, f64) = (1920.0, 1080.0);

/// ⭐⭐⭐ **PERGUNTA 2 — O ZOOM**: ao afastar, quantas cópias entram, **que tamanho cada uma tem no
/// ecrã**, e quanto custa o quadro. É a reprodução do gesto do report (*«ao dar o zoom … logo que
/// muitas estrelas apareçam»*), e a coluna que interessa é a do TAMANHO: ela diz que a partir de
/// certo ponto o quadro paga `30` segmentos por forma para pintar um punhado de píxeis.
///
/// ⚠️ **A escala da câmara da cena É o [`PX_POR_UNIDADE`](crate::motion_state::carimbo_demo)** —
/// a estrela nasce com [`ESTRELA_PX`](crate::motion_state::carimbo_demo) píxeis de lado. Afastar é
/// descer essa escala; a coluna `px/estrela` é `2 × TAMANHO × escala`, medida da mesma constante
/// de que a cena é feita, nunca de um número escrito aqui.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_zoom_and_the_screen_size() {
    let _fatia = fatia();
    let (m, _saida) = cena_do_report(0.5);
    let insts = &m.pump.vector_instances;
    let store = &m.shape_store;
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
    let campo = (x1 - x0).max(y1 - y0);
    let nasce = f64::from(crate::motion_state::carimbo_demo::PX_POR_UNIDADE);
    let lado_un = f64::from(crate::motion_state::carimbo_demo::TAMANHO) * 2.0;
    let janela = ph2d_vector::Rect::new(0.0, 0.0, JANELA_PX.0, JANELA_PX.1);

    eprintln!(
        "\n  ═══ O ZOOM: O QUE ENTRA NO ECRÃ E QUE TAMANHO TEM (campo {campo:.1} un., \
         janela {}×{}, load {}) ═══\n",
        JANELA_PX.0,
        JANELA_PX.1,
        carga()
    );
    eprintln!(
        "    zoom | px/estrela |  cópias | % das 90k |  desenho | resolver |   soma | p/ a placa"
    );
    eprintln!(
        "  -------|------------|---------|-----------|----------|----------|--------|-----------"
    );

    for mult in [4.0f64, 2.0, 1.0, 0.5, 0.25, 0.125] {
        let z = nasce * mult;
        let cam = ph2d_vector::Affine::translate((JANELA_PX.0 * 0.5, JANELA_PX.1 * 0.5))
            * ph2d_vector::Affine::scale(z)
            * ph2d_vector::Affine::translate((-centro.0, -centro.1));
        let encoda = |cena: &mut ph2d_vector::VectorScene| {
            let mut sem_arte = |_: u32, _: [f32; 4]| None;
            crate::motion_shape_gen::encode(insts, store, &mut sem_arte, cam, Some(janela), cena);
        };
        let desenho = melhor_quente(3, encoda);
        let mut cena = ph2d_vector::VectorScene::new();
        encoda(&mut cena);
        let mut r = ph2d_vector::SceneResolver::new();
        let mut tamanho = r.resolve(&cena);
        let mut resolver = f64::INFINITY;
        for _ in 0..3 {
            let inicio = std::time::Instant::now();
            tamanho = r.resolve(&cena);
            resolver = resolver.min(inicio.elapsed().as_secs_f64() * 1e3);
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "um tamanho de buffer / uma contagem"
        )]
        let (mb, pct) = (
            tamanho.scene_bytes as f64 / 1e6,
            tamanho.draw_objects as f64 / esperadas as f64 * 100.0,
        );
        eprintln!(
            "   {mult:>5.3} | {:>10.2} | {:>7} | {pct:>8.1}% | {desenho:>5.2} ms | {resolver:>5.2} ms \
             | {:>4.2} ms | {mb:>6.1} MB",
            lado_un * z,
            tamanho.draw_objects,
            desenho + resolver,
        );
    }
    eprintln!(
        "\n  ⚠️ `zoom 1,000` é a cena como ela NASCE. Descer a coluna é AFASTAR: as cópias sobem\n  \
         até às 90 000 e o tamanho de cada uma cai — e o custo NÃO cai com ele, porque o número\n  \
         de segmentos por forma não depende de quantos píxeis ela ocupa.\n"
    );
    eprintln!("  load no fim: {}\n", carga());
}

/// ⭐⭐⭐ **PERGUNTA 4 — o que um LOD DA FORMA compraria, nos zooms em que ele armaria.** As três
/// sondas acima dizem que (1) o LOD que existe não arma aqui, (2) abaixo de `zoom 0,5` o custo
/// **satura** enquanto a forma encolhe `4×`, e (3) abaixo de `4,4 px` de lado o arredondamento é
/// invisível. Esta mede o ganho: a MESMA cena desenhada com a silhueta afiada — que é, ao bit, o
/// que um LOD entregaria naqueles tamanhos.
///
/// ⚠️ **Mede-se [`ph2d_vec_render::draw_shared_instances`] directamente**, e é declarado: é a porta
/// a que o [`encode`](crate::motion_shape_gen::encode) delega o lote inteiro quando não há imagem
/// no meio (a cena `=126` não tem nenhuma), logo é o mesmo código — o que ela permite é dar a
/// silhueta por um fecho, sem inventar `geometry_id` nenhum.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_what_a_shape_lod_would_buy() {
    let _fatia = fatia();
    let (m, _saida) = cena_do_report(0.5);
    let insts = &m.pump.vector_instances;
    let gid = insts.first().map_or(0, |i| i.geometry_id);
    let arredondada = m
        .shape_store
        .get(gid)
        .expect("a cena carimba do store")
        .clone();
    let afiada = forma_da_cena(0.0);
    assert_eq!(
        segmentos(&arredondada),
        30,
        "controlo: a arredondada tem 30 segmentos"
    );
    assert_eq!(segmentos(&afiada), 12, "controlo: a afiada tem 12");

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
    let janela = ph2d_vector::Rect::new(0.0, 0.0, JANELA_PX.0, JANELA_PX.1);

    eprintln!(
        "\n  ═══ O QUE UM LOD DA FORMA COMPRARIA (load {}) ═══\n",
        carga()
    );
    eprintln!("    zoom | px/estrela |   HOJE (30 seg) |   LOD (12 seg) | ganho | p/ a placa");
    eprintln!("  -------|------------|-----------------|----------------|-------|------------");

    for mult in [4.0f64, 1.0, 0.5, 0.25, 0.125] {
        let z = nasce * mult;
        let cam = ph2d_vector::Affine::translate((JANELA_PX.0 * 0.5, JANELA_PX.1 * 0.5))
            * ph2d_vector::Affine::scale(z)
            * ph2d_vector::Affine::translate((-centro.0, -centro.1));
        let lote: Vec<(u32, ph2d_vector::Affine, [f32; 4])> = insts
            .iter()
            .map(|i| {
                (
                    i.geometry_id,
                    crate::motion_shape_gen::instance_pose(i, cam),
                    i.tint,
                )
            })
            .collect();
        let mede = |forma: &ph2d_vec_scene::VecPath| {
            let encoda = |cena: &mut ph2d_vector::VectorScene| {
                ph2d_vec_render::draw_shared_instances(
                    lote.iter().copied(),
                    |_| Some(forma),
                    Some(janela),
                    cena,
                );
            };
            let d = melhor_quente(3, encoda);
            let mut cena = ph2d_vector::VectorScene::new();
            encoda(&mut cena);
            let mut r = ph2d_vector::SceneResolver::new();
            let mut t = r.resolve(&cena);
            let mut res = f64::INFINITY;
            for _ in 0..3 {
                let i = std::time::Instant::now();
                t = r.resolve(&cena);
                res = res.min(i.elapsed().as_secs_f64() * 1e3);
            }
            (d + res, t)
        };
        let (hoje, t_hoje) = mede(&arredondada);
        let (lod, t_lod) = mede(&afiada);
        #[expect(clippy::cast_precision_loss, reason = "tamanhos de buffer")]
        let (mb_h, mb_l) = (
            t_hoje.scene_bytes as f64 / 1e6,
            t_lod.scene_bytes as f64 / 1e6,
        );
        eprintln!(
            "   {mult:>5.3} | {:>10.2} | {hoje:>10.2} ms | {lod:>9.2} ms | {:>4.2}× | {mb_h:>5.1} → {mb_l:.1} MB",
            lado_un * z,
            hoje / lod,
        );
    }
    eprintln!(
        "\n  ⚠️ A 1.ª linha (`zoom 4,000`, {:.1} px) é o CONTROLO: ali o arredondamento SE VÊ\n  \
         (a barra é 4,4 px de lado) e o LOD não deve armar — ela está aqui para mostrar o que\n  \
         se perderia se ele armasse, não o que se ganha.\n",
        lado_un * nasce * 4.0
    );
    eprintln!("  load no fim: {}\n", carga());
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

/// ⭐ **A RÉGUA** — as sondas que dizem *o que o olho pode ver*, num ficheiro irmão por
/// RESPONSABILIDADE (e o tecto de LOC obrigou-o a acontecer no dia certo): aqui mede-se o CUSTO e
/// onde o LOD arma, ali mede-se a FIDELIDADE que autoriza a troca.
#[path = "motion_carimbo_lod_regua_probe.rs"]
mod regua;

/// ⛔⛔ **PERGUNTA 8 — QUANTO CUSTA A DECISÃO, no quadro em que ela NÃO arma?**
///
/// A lei desta casa, escrita no `standalone.rs` quando o recorte por câmara a pagou: *uma cura
/// cujo teste custa o que ela poupa não é uma cura*. O [`geometrias_para_lod`] corre em TODO
/// quadro e faz **duas** passagens sobre as instâncias — e no zoom de nascimento ele decide
/// `não`, logo essas passagens são trabalho puro para o lixo.
///
/// ⚠️ **O CONTROLO é a coluna desligada** (a porta devolve vazio antes das passagens): a
/// diferença entre as duas colunas É o preço da decisão, e nada mais.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_what_the_decision_itself_costs() {
    let _fatia = fatia();
    let (m, _saida) = cena_do_report(0.5);
    let insts = &m.pump.vector_instances;
    let nasce = f64::from(crate::motion_state::carimbo_demo::PX_POR_UNIDADE);
    let (mut x0, mut x1) = (f64::MAX, f64::MIN);
    for i in insts {
        x0 = x0.min(f64::from(i.world_pos[0]));
        x1 = x1.max(f64::from(i.world_pos[0]));
    }
    let centro = (x0 + x1) * 0.5;

    eprintln!("\n  ═══ O PREÇO DA DECISÃO (load {}) ═══\n", carga());
    eprintln!("    zoom | arma? | LIGADA | DESLIGADA (controlo) | o que a decisão custa");
    eprintln!("  -------|-------|--------|----------------------|----------------------");
    for mult in [1.0f64, 0.25] {
        let z = nasce * mult;
        let cam = ph2d_vector::Affine::translate((JANELA_PX.0 * 0.5, JANELA_PX.1 * 0.5))
            * ph2d_vector::Affine::scale(z)
            * ph2d_vector::Affine::translate((-centro, -centro));
        let mede = |ligado: bool| {
            let mut melhor = f64::INFINITY;
            for _ in 0..5 {
                let t = std::time::Instant::now();
                let q = crate::motion_shape_lod::geometrias_para_lod_com(
                    insts,
                    &m.shape_store,
                    cam,
                    crate::motion_bridge::objects::LOD_COUNT,
                    ligado,
                );
                melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
                std::hint::black_box(&q);
            }
            melhor
        };
        let (on, off) = (mede(true), mede(false));
        let arma = !crate::motion_shape_lod::geometrias_para_lod_com(
            insts,
            &m.shape_store,
            cam,
            crate::motion_bridge::objects::LOD_COUNT,
            true,
        )
        .is_empty();
        eprintln!(
            "   {mult:>5.3} | {:>5} | {on:>4.2} ms | {off:>17.2} ms | {:>5.2} ms ({:>4.1}% de um quadro)",
            if arma { "SIM" } else { "não" },
            on - off,
            (on - off) / 16.67 * 100.0,
        );
    }
    eprintln!("\n  load no fim: {}\n", carga());
}
