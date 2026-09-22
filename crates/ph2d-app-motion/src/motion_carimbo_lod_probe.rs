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
//! | `4,000` | `24,00` | `3 900` | `1,42 ms` | `1,9 MB` |
//! | `1,000` (nasce) | `6,00` | `45 036` | `2,88 ms` | `22,2 MB` |
//! | `0,500` | `3,00` | `90 000` | `4,58 ms` | `44,3 MB` |
//! | `0,250` | `1,50` | `90 000` | `4,55 ms` | `44,3 MB` |
//! | `0,125` | `0,75` | `90 000` | `4,58 ms` | `44,3 MB` |
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
//! `cargo test -p ph2d-app-motion --release -- --ignored --nocapture audit_`

use super::{carga, fatia, melhor_quente, segmentos};

/// Amostra densa de um contorno cozido — o polígono que aproxima a silhueta ao nível a que a
/// comparação deixa de depender da amostragem (o controlo está na sonda: dobrar `POR_SEGMENTO`
/// não move o desvio).
const POR_SEGMENTO: usize = 64;

/// A silhueta de um `VecPath` como uma polilinha densa em coordenadas LOCAIS.
///
/// ⚠️ **Amostro a cúbica à mão em vez de pedir um achatamento à biblioteca**, porque o que se mede
/// aqui é a distância entre duas silhuetas e um achatamento com tolerância é, ele próprio, um
/// desvio — *a régua não pode ter a ordem de grandeza do que ela mede*.
fn silhueta(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let c = p.cooked();
    let mut pontos = Vec::new();
    for i in 0..c.contour_count() {
        let Some((verts, fechado)) = c.contour(i) else {
            continue;
        };
        if verts.len() < 2 {
            continue;
        }
        let n = verts.len();
        let pares = if fechado { n } else { n - 1 };
        for k in 0..pares {
            let a = &verts[k];
            let b = &verts[(k + 1) % n];
            let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
            for s in 0..POR_SEGMENTO {
                #[expect(clippy::cast_precision_loss, reason = "índice de amostra, < 64")]
                let t = s as f64 / POR_SEGMENTO as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                pontos.push([
                    w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
                    w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
                ]);
            }
        }
    }
    pontos
}

/// Distância de um ponto ao SEGMENTO `[a, b]`.
fn ao_segmento(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = vx * vx + vy * vy;
    let t = if l2 <= 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * vx + (p[1] - a[1]) * vy) / l2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (p[0] - (a[0] + t * vx), p[1] - (a[1] + t * vy));
    (dx * dx + dy * dy).sqrt()
}

/// O desvio máximo da polilinha `de` à polilinha `para` (Hausdorff unilateral).
fn desvio(de: &[[f64; 2]], para: &[[f64; 2]]) -> f64 {
    de.iter()
        .map(|&p| {
            (0..para.len())
                .map(|i| ao_segmento(p, para[i], para[(i + 1) % para.len()]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max)
}

/// A maior extensão da silhueta — o denominador que torna o desvio ADIMENSIONAL (e portanto
/// comparável entre tamanhos, que é a lei que esta casa já paga noutras réguas).
fn diametro(s: &[[f64; 2]]) -> f64 {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in s {
        x0 = x0.min(p[0]);
        y0 = y0.min(p[1]);
        x1 = x1.max(p[0]);
        y1 = y1.max(p[1]);
    }
    (x1 - x0).max(y1 - y0)
}

/// Monta a cadeia do report **na população da cena `=126`** (`LADO_N²`), com o `corner` pedido.
/// ⚠️ A população é a da CENA e não a do arnês: uma tabela na população errada compara-se com a do
/// dono como se fosse a mesma experiência (a sonda irmã já pagou este defeito).
fn cena_do_report(corner: f32) -> (crate::motion_state::MotionState, ph2d_nodegraph::graph::NodeId) {
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
    (m, saida)
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
    let gid = m
        .pump
        .vector_instances
        .first()
        .map_or(0, |i| i.geometry_id);
    let tile = m.object_bake.tile_texture_for_gid(gid);
    let esperadas = (crate::motion_state::carimbo_demo::LADO_N as usize).pow(2);
    assert_eq!(antes, esperadas, "controlo: a cena tem de trazer {esperadas}");

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

    eprintln!("\n  ═══ O LOD QUE JÁ EXISTE, NA CENA DO REPORT (load {}) ═══\n", carga());
    eprintln!("    cópias de UMA geometria ........ {antes}");
    eprintln!("    o joelho (`LOD_COUNT`) ......... {}", crate::motion_bridge::objects::LOD_COUNT);
    eprintln!(
        "    acima do joelho? ............... {}",
        if antes > crate::motion_bridge::objects::LOD_COUNT { "SIM" } else { "não" }
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
    let gid = m
        .pump
        .vector_instances
        .first()
        .map_or(0, |i| i.geometry_id);
    m.shape_store
        .get(gid)
        .expect("a cena carimba uma forma do store")
        .clone()
}

/// ⭐⭐⭐ **PERGUNTA 3 — A BARRA: a que tamanho no ecrã o arredondamento deixa de ser VISÍVEL?**
/// É daqui que sai o limiar do LOD, e não de um palpite (`CLAUDE.md` §0.0: *meça, e depois escreva
/// o número que a medição deu, com a tabela ao lado*).
///
/// A régua é **geométrica e adimensional**: o desvio de Hausdorff entre a silhueta arredondada e a
/// afiada, dividido pelo diâmetro da forma. Multiplicado pelos píxeis que a forma ocupa no ecrã, dá
/// o desvio **em píxeis** — e abaixo de meio píxel ele vive inteiro dentro do anti-serrilhado, que
/// é o mesmo que dizer que nenhum ecrã o pode mostrar.
///
/// ⚠️ **O CONTROLO é o `corner = 0`**: ali as duas silhuetas são a mesma e o desvio tem de ler
/// `0,000` — sem ele, uma régua partida daria «invisível» a tudo e o limiar sairia de nada.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_when_the_rounding_stops_showing() {
    let _fatia = fatia();
    let afiada = forma_da_cena(0.0);
    let s_afiada = silhueta(&afiada);
    let d = diametro(&s_afiada);
    assert!(d > 0.0, "controlo: a forma afiada tem extensão");

    eprintln!(
        "\n  ═══ QUANDO O ARREDONDAMENTO DEIXA DE SE VER (load {}) ═══\n",
        carga()
    );
    eprintln!("   corner | segmentos | desvio (frac. do lado) | invisível abaixo de");
    eprintln!("  --------|-----------|------------------------|--------------------");
    for c in [0.0f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        let p = forma_da_cena(c);
        let s = silhueta(&p);
        let h = desvio(&s_afiada, &s).max(desvio(&s, &s_afiada)) / d;
        let limite = if h > 0.0 { 0.5 / h } else { f64::INFINITY };
        eprintln!(
            "   {c:>6.2} | {:>9} | {h:>22.5} | {}",
            segmentos(&p),
            if limite.is_finite() {
                format!("{limite:>10.1} px de lado")
            } else {
                "       (não há desvio)".to_string()
            }
        );
    }
    let px = crate::motion_state::carimbo_demo::ESTRELA_PX;
    eprintln!(
        "\n  ⇒ a cena `=126` nasce com a estrela a {px:.1} px de lado; ao AFASTAR ela só encolhe.\n"
    );
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
    assert_eq!(insts.len(), esperadas, "controlo: a cena tem de trazer {esperadas}");

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
        JANELA_PX.0, JANELA_PX.1, carga()
    );
    eprintln!("    zoom | px/estrela |  cópias | % das 90k |  desenho | resolver |   soma | p/ a placa");
    eprintln!("  -------|------------|---------|-----------|----------|----------|--------|-----------");

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
        #[expect(clippy::cast_precision_loss, reason = "um tamanho de buffer / uma contagem")]
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
    let arredondada = m.shape_store.get(gid).expect("a cena carimba do store").clone();
    let afiada = forma_da_cena(0.0);
    assert_eq!(segmentos(&arredondada), 30, "controlo: a arredondada tem 30 segmentos");
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
            .map(|i| (i.geometry_id, crate::motion_shape_gen::instance_pose(i, cam), i.tint))
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
        let (mb_h, mb_l) = (t_hoje.scene_bytes as f64 / 1e6, t_lod.scene_bytes as f64 / 1e6);
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

/// Amostras por eixo dentro de um pixel — a cobertura de referência. `8×8 = 64` põe o degrau de
/// quantização em `1/64` da cobertura, uma ordem de grandeza abaixo do que se compara.
const SUPER: usize = 8;

/// **A cobertura de um polígono num pixel**, por amostragem — a rasterização de REFERÊNCIA.
///
/// ⭐ Ela existe porque a régua que decide um LOD é **o que o olho vê**, e o olho vê píxeis, não
/// distâncias. A geométrica (Hausdorff) diz o desvio da SILHUETA; esta diz quanto disso sobrevive
/// à grelha do ecrã — e é a segunda que manda, porque é ela que descreve a imagem.
///
/// ⚠️ Sem GPU **de propósito**: uma régua de LOD que precise de placa não corre no portão, e a
/// placa desta máquina está tomada. `par` conta cruzamentos (regra ímpar-par, que é a do
/// preenchimento de uma estrela).
fn rasteriza(poli: &[[f64; 2]], lado_px: usize, caixa: ([f64; 2], f64), sup: usize) -> Vec<f64> {
    let (min, extensao) = caixa;
    let mut img = vec![0.0f64; lado_px * lado_px];
    #[expect(clippy::cast_precision_loss, reason = "lado da imagem, dezenas de píxeis")]
    let passo = extensao / lado_px as f64;
    for py in 0..lado_px {
        for px in 0..lado_px {
            let mut dentro = 0usize;
            for sy in 0..sup {
                for sx in 0..sup {
                    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
                    let p = [
                        min[0] + (px as f64 + (sx as f64 + 0.5) / sup as f64) * passo,
                        min[1] + (py as f64 + (sy as f64 + 0.5) / sup as f64) * passo,
                    ];
                    let mut cruza = false;
                    for i in 0..poli.len() {
                        let (a, b) = (poli[i], poli[(i + 1) % poli.len()]);
                        if (a[1] > p[1]) != (b[1] > p[1]) {
                            let x = (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0];
                            if p[0] < x {
                                cruza = !cruza;
                            }
                        }
                    }
                    if cruza {
                        dentro += 1;
                    }
                }
            }
            #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
            let c = dentro as f64 / (sup * sup) as f64;
            img[py * lado_px + px] = c;
        }
    }
    img
}

/// A caixa comum às duas silhuetas — o MESMO enquadramento para as duas imagens, senão o que se
/// mede é o enquadramento e não a forma.
fn caixa_comum(a: &[[f64; 2]], b: &[[f64; 2]]) -> ([f64; 2], f64) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in a.iter().chain(b) {
        x0 = x0.min(p[0]);
        y0 = y0.min(p[1]);
        x1 = x1.max(p[0]);
        y1 = y1.max(p[1]);
    }
    let e = (x1 - x0).max(y1 - y0);
    ([x0, y0], e)
}

/// ⭐⭐⭐ **PERGUNTA 5 — A BARRA QUE O OLHO USA: quantos NÍVEIS de 255 separam a forma exacta da
/// simplificada, a cada tamanho?** É esta que manda sobre a geométrica, porque descreve a IMAGEM.
///
/// ⚠️ **O CONTROLO é a primeira coluna** (a forma comparada consigo mesma): ela tem de ler `0`
/// níveis a todo tamanho. Sem ele, uma rasterização partida daria «idêntico» a tudo.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_how_many_levels_the_eye_could_tell() {
    let _fatia = fatia();
    let afiada = silhueta(&forma_da_cena(0.0));
    let arredondada = silhueta(&forma_da_cena(0.5));
    let caixa = caixa_comum(&afiada, &arredondada);

    eprintln!(
        "\n  ═══ QUANTOS NÍVEIS DE 255 SEPARAM AS DUAS FORMAS, POR TAMANHO (load {}) ═══\n",
        carga()
    );
    eprintln!("   lado (px) | CONTROLO (ela mesma) | arredondada × afiada | × +ÁREA (8×8) | RÉGUA FINA (32×32) | o olho vê?");
    eprintln!("  -----------|----------------------|----------------------|---------------|--------------------|------------");
    // ⭐⭐⭐ **A COMPENSAÇÃO DE ÁREA** — a cura que a própria tabela encomenda. A `1 px` a forma
    // inteira cabe num pixel e o que sobra da silhueta é só a ÁREA: se a simplificada não tiver a
    // mesma, ela lê-se como uma mancha de outro BRILHO, a todo tamanho. `k` é o factor linear que
    // iguala as duas áreas (a área vai com o quadrado do lado).
    let area = |p: &[[f64; 2]]| {
        (0..p.len())
            .map(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum::<f64>()
            .abs()
            * 0.5
    };
    let k = (area(&arredondada) / area(&afiada)).sqrt();
    let centro_de = |p: &[[f64; 2]]| {
        let n = f64::from(u32::try_from(p.len()).unwrap_or(1));
        let s = p.iter().fold([0.0, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
        [s[0] / n, s[1] / n]
    };
    let c = centro_de(&afiada);
    let compensada: Vec<[f64; 2]> = afiada
        .iter()
        .map(|p| [c[0] + (p[0] - c[0]) * k, c[1] + (p[1] - c[1]) * k])
        .collect();
    eprintln!("   (o factor de área: {k:.5} — a afiada é {:.2}% maior)\n", (1.0 / k - 1.0) * 100.0);
    for lado in [24usize, 12, 8, 6, 4, 3, 2, 1] {
        let a = rasteriza(&arredondada, lado, caixa, SUPER);
        let b = rasteriza(&afiada, lado, caixa, SUPER);
        let comp = rasteriza(&compensada, lado, caixa, SUPER);
        let ctl = rasteriza(&arredondada, lado, caixa, SUPER);
        // ⚠️⚠️ **O CONTROLO DA PRÓPRIA RÉGUA** — a MESMA pergunta com `4×` as amostras por eixo
        // (`16×` por pixel). Uma coluna que se mexa entre as duas é RUÍDO DE AMOSTRAGEM, não uma
        // diferença entre as formas: *a régua não pode ter a ordem de grandeza do que ela mede*.
        let a_f = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let b_f = rasteriza(&afiada, lado, caixa, SUPER * 4);
        let niveis = |x: &[f64], y: &[f64]| {
            x.iter()
                .zip(y)
                .map(|(p, q)| (p - q).abs() * 255.0)
                .fold(0.0f64, f64::max)
        };
        let (n_ctl, n, n_c) = (niveis(&a, &ctl), niveis(&a, &b), niveis(&a, &comp));
        let n_fino = niveis(&a_f, &b_f);
        eprintln!(
            "   {lado:>9} | {n_ctl:>20.1} | {n:>20.1} | {n_c:>13.1} | {n_fino:>13.1} | {}",
            if n_fino < 1.0 { "não" } else { "SIM" }
        );
    }
    eprintln!(
        "\n  ⚠️ `1 nível de 255` é o degrau do canal: abaixo dele as duas imagens são o MESMO\n  \
         byte em cada pixel, e nenhum ecrã as pode distinguir.\n"
    );
}

/// Reamostra uma imagem quadrada `de × de` para `para × para` pela MÉDIA das células que cada
/// pixel novo cobre — o filtro de caixa, que é o que uma placa faz ao minificar uma textura com
/// mipmap. ⚠️ `de` é múltiplo de `para` por construção nas chamadas, logo não há meia célula.
fn reamostra(img: &[f64], de: usize, para: usize) -> Vec<f64> {
    #[expect(clippy::cast_precision_loss, reason = "lados de imagem, ≤ 2048")]
    let (de_f, para_f) = (de as f64, para as f64);
    let passo = de_f / para_f;
    let mut fora = vec![0.0f64; para * para];
    for y in 0..para {
        for x in 0..para {
            // A célula do destino, em coordenadas da origem — com pesos FRACCIONÁRIOS, porque o
            // lado da tile não é múltiplo do lado no ecrã (e fingir que é mediria outro caso).
            #[expect(clippy::cast_precision_loss, reason = "índices de pixel")]
            let (u0, v0) = (x as f64 * passo, y as f64 * passo);
            let (u1, v1) = (u0 + passo, v0 + passo);
            let (mut soma, mut peso) = (0.0f64, 0.0f64);
            #[expect(clippy::cast_possible_truncation, reason = "coordenadas dentro da imagem")]
            #[expect(clippy::cast_sign_loss, reason = "u0/v0 ≥ 0")]
            for sy in (v0 as usize)..=((v1.ceil() as usize).min(de).saturating_sub(1)) {
                #[expect(clippy::cast_possible_truncation, reason = "coordenadas dentro da imagem")]
                #[expect(clippy::cast_sign_loss, reason = "u0 ≥ 0")]
                for sx in (u0 as usize)..=((u1.ceil() as usize).min(de).saturating_sub(1)) {
                    #[expect(clippy::cast_precision_loss, reason = "índices de pixel")]
                    let w = (u1.min(sx as f64 + 1.0) - u0.max(sx as f64)).max(0.0)
                        * (v1.min(sy as f64 + 1.0) - v0.max(sy as f64)).max(0.0);
                    soma += img[sy * de + sx] * w;
                    peso += w;
                }
            }
            fora[y * para + x] = if peso > 0.0 { soma / peso } else { 0.0 };
        }
    }
    fora
}

/// ⭐⭐⭐ **PERGUNTA 6 — A ROTA QUE A CASA JÁ TEM: quanto erra uma TILE?** A pergunta 5 refutou
/// *«largar o arredondamento»* (ele nunca fica invisível — a `2 px` ainda são `3` níveis). Sobra a
/// rota que o [`apply_object_lod`](crate::motion_bridge_objects::apply_object_lod) já implementa e
/// que esta cena não alcança: assar a forma UMA vez e instanciar um quad texturado.
///
/// A régua é a mesma e o veredito é outro: uma tile é a forma **rasterizada e reamostrada**, logo
/// o erro dela é o da reamostragem — que encolhe com o tamanho em vez de crescer.
///
/// ⚠️ **O CONTROLO continua a ser a coluna da forma comparada consigo mesma**, e a coluna da
/// silhueta simplificada fica ao lado: é a comparação entre as DUAS curas que decide, não o valor
/// absoluto de nenhuma.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_how_much_a_tile_would_err() {
    let _fatia = fatia();
    let arredondada = silhueta(&forma_da_cena(0.5));
    let afiada = silhueta(&forma_da_cena(0.0));
    let caixa = caixa_comum(&arredondada, &arredondada);
    // ⭐⭐⭐ **A TILE COM O LADO QUE O ASSADOR DA CASA DE FACTO PRODUZ** — e não um número
    // redondo. [`BAKE_DPI`](crate::motion_object_bake::BAKE_DPI) é `256 px` por unidade de mundo e
    // a estrela mede `2 × TAMANHO` unidades ⇒ a tile sai com **`28 px` de lado**. Medir com uma
    // tile de `256` mediria um produto que esta casa não assa, e daria à rota da tile uma margem
    // que ela não tem (`CLAUDE.md` §0.0).
    #[expect(clippy::cast_possible_truncation, reason = "o lado de uma tile, dezenas de px")]
    #[expect(clippy::cast_sign_loss, reason = "um lado é positivo")]
    let tile_px = (f64::from(crate::motion_state::carimbo_demo::TAMANHO)
        * 2.0
        * crate::motion_object_bake::BAKE_DPI)
        .round() as usize;
    let assada = rasteriza(&arredondada, tile_px, caixa, 4);
    eprintln!("   (o assador da casa dá a esta forma uma tile de {tile_px} px de lado)\n");

    eprintln!(
        "\n  ═══ O ERRO DE UMA TILE, CONTRA O DE SIMPLIFICAR A SILHUETA (load {}) ═══\n",
        carga()
    );
    eprintln!("   lado (px) | CONTROLO | TILE reamostrada | silhueta simplificada | quem erra menos");
    eprintln!("  -----------|----------|------------------|-----------------------|----------------");
    for lado in [128usize, 64, 32, 28, 16, 8, 4, 2, 1] {
        let exacta = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let ctl = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let tile = reamostra(&assada, tile_px, lado);
        let simples = rasteriza(&afiada, lado, caixa, SUPER * 4);
        let niveis = |x: &[f64], y: &[f64]| {
            x.iter()
                .zip(y)
                .map(|(p, q)| (p - q).abs() * 255.0)
                .fold(0.0f64, f64::max)
        };
        let (c, t, s) = (
            niveis(&exacta, &ctl),
            niveis(&exacta, &tile),
            niveis(&exacta, &simples),
        );
        eprintln!(
            "   {lado:>9} | {c:>8.1} | {t:>16.1} | {s:>21.1} | {}",
            if t < s { "a TILE" } else { "a silhueta" }
        );
    }
    eprintln!(
        "\n  ⚠️ A tile é assada UMA vez por geometria e depois é um quad por cópia — o erro dela é\n  \
         o da REAMOSTRAGEM, e ele encolhe com o tamanho. O da silhueta simplificada CRESCE com\n  \
         ele, porque a forma é outra.\n"
    );
}
