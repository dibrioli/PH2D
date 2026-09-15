//! ⭐⭐⭐ **O chrome do canvas é pintado ONDE A ARTE DESENHA** — e não onde o quad dela repousa.
//!
//! ⛔⛔ **A wave de 2026-09-14 curou o DEDO e deixou o OLHO** (medido 2026-09-15): a entrega do
//! ponteiro passou a resolver o clique pela malha posada e o editor de curva continuou a pintar os
//! pontos de controlo pelo afim do quad de repouso. ⇒ *um controlo desenhado por um mapa e agarrado
//! por outro*, que é a espécie de controlo morto que o `CLAUDE.md` §5.0 diz que nenhuma sonda deste
//! repo apanha.
//!
//! ⚠️ **Este gate mede a LEI, não o texto:** os gates de arquitectura irmãos (na `ph2d-editor-core`)
//! afirmam que os desenhadores *consultam* a porta; este afirma que a porta *responde certo*. As
//! duas metades são precisas — citar uma porta não é consultá-la, e consultá-la não é tê-la certa.

use ph2d_app_painter::canvas_map::CanvasMap;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, GlobalTransform, PresentWorld, SimRef, SimWorld};
use ph2d_render::{Camera2d, RenderInstance, SpriteMesh};
use ph2d_vector::{Affine, Point};

/// Uma sprite de `2 × 2` na origem, espelhada no mundo de apresentação.
fn sprite_de_2m(present: &mut PresentWorld, sim: &mut SimWorld) -> (Entity, u64) {
    let e = sim.world_mut().spawn(()).id();
    let gt =
        GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(0.0, 0.0)));
    let ri = RenderInstance {
        world_pos: [0.0, 0.0],
        size: [2.0, 2.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    present.world_mut().spawn((SimRef(e), gt, ri));
    (e, e.to_bits())
}

/// A malha POSADA para `x = 3..5` — o caso em que o quad e a malha discordam, e o único que diz
/// qual dos dois um consumidor está a ler (a mesma fixtura que o `ph2d-render` usa).
fn braco_posado(present: &mut PresentWorld, alvo: Entity) {
    let mut q = present.world_mut().query::<(Entity, &SimRef)>();
    let e = q
        .iter(present.world())
        .find(|(_, r)| r.0 == alvo)
        .map(|(e, _)| e)
        .expect("o espelho foi criado");
    present.world_mut().entity_mut(e).insert(SpriteMesh {
        local: vec![[3.0, 0.0], [5.0, 0.0], [3.0, 2.0]],
        uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2]],
    });
}

#[test]
fn a_handle_is_painted_where_the_bent_art_draws_that_texel() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();
    let (alvo, bits) = sprite_de_2m(&mut present, &mut sim);

    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let window = ph2d_host::WindowSize::new(800, 800);
    // Um afim de quad qualquer: o que importa é que ele NÃO é a resposta quando há malha.
    let afim = Affine::scale(1.0);
    let (iw, ih) = (100u32, 100u32);
    // A alça autorada a um quarto da largura e a três quartos da altura da imagem.
    let alca = [25.0f32, 75.0f32];

    // ── Sem malha: o mapa É o afim, ao bit. É este controlo que impede a cura de mudar o caminho
    // de sempre — uma sprite que se desenha como quad não passa a ser pintada por outra lei.
    let mapa = CanvasMap::new(present.world(), bits, iw, ih, afim, &camera, window);
    assert!(!mapa.is_mesh(), "sem `SpriteMesh` não há malha a consultar");
    assert_eq!(
        mapa.point(alca),
        afim * Point::new(f64::from(alca[0]), f64::from(alca[1])),
        "sobre um quad o mapa tem de ser o afim de sempre, sem um epsilon de diferença"
    );

    // ── Com malha: o mesmo texel é pintado onde a arte o DESENHA.
    braco_posado(&mut present, alvo);
    let mapa = CanvasMap::new(present.world(), bits, iw, ih, afim, &camera, window);
    assert!(
        mapa.is_mesh(),
        "com `SpriteMesh` a porta responde pela malha"
    );
    // A UV `(0,25 · 0,75)` deste triângulo cai no ponto local `(3,5 · 0,5)` — e o mundo é o local,
    // porque a base é a identidade e o pivô é a origem. No ecrã: `world_to_screen`.
    let esperado = {
        let (x, y) = camera.world_to_screen([3.5, 0.5], window);
        Point::new(f64::from(x), f64::from(y))
    };
    let p = mapa.point(alca);
    assert!(
        (p.x - esperado.x).abs() < 1e-3 && (p.y - esperado.y).abs() < 1e-3,
        "a alça tem de ser pintada em {esperado:?} (onde a arte desenha aquele texel) e foi {p:?}"
    );
    // ⛔ **O DISCRIMINADOR**: o afim do quad punha-a noutro sítio. Sem esta asserção, um mapa que
    // ignorasse a malha passaria a de cima se por acaso os dois coincidissem.
    let pelo_quad = afim * Point::new(f64::from(alca[0]), f64::from(alca[1]));
    assert!(
        (p.x - pelo_quad.x).abs() > 1.0,
        "a resposta é a da MALHA POSADA, nunca a do quad de repouso ({p:?} contra {pelo_quad:?})"
    );

    // ── Um ponto FORA da malha (a metade da imagem que este triângulo não cobre) cai na lei do
    // quad: é o que mantém uma alça arrastada para fora da silhueta visível em vez de desaparecer.
    let fora = [95.0f32, 5.0f32];
    assert_eq!(
        mapa.point(fora),
        afim * Point::new(f64::from(fora[0]), f64::from(fora[1])),
        "fora da arte desenhada o chamador fica com a lei dele"
    );

    // ── E o ladrilho do *Repeat Image* é um deslocamento de ECRÃ, nas DUAS metades.
    let deslocado = mapa.deslocado(40.0, -25.0);
    let q = deslocado.point(alca);
    assert!(
        (q.x - (p.x + 40.0)).abs() < 1e-6 && (q.y - (p.y - 25.0)).abs() < 1e-6,
        "o ladrilho desloca a resposta da MALHA também, senão ele desenha as alças por cima do centro"
    );
}

/// Uma malha **DOBRADA**: duas colunas de quadrados em que a da direita se levanta. Uma linha
/// horizontal da imagem atravessa a dobra — que é exactamente o que a grelha do Painter faz.
///
/// ⚠️ A convenção de `uv` é a da fixtura acima: `v = 0` é o TOPO (o `y` maior).
fn arte_dobrada(present: &mut PresentWorld, alvo: Entity) {
    let mut q = present.world_mut().query::<(Entity, &SimRef)>();
    let e = q
        .iter(present.world())
        .find(|(_, r)| r.0 == alvo)
        .map(|(e, _)| e)
        .expect("o espelho foi criado");
    present.world_mut().entity_mut(e).insert(SpriteMesh {
        local: vec![
            [0.0, 2.0],
            [1.0, 2.0],
            [2.0, 3.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [2.0, 1.0],
        ],
        uv: vec![
            [0.0, 0.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.5, 1.0],
            [1.0, 1.0],
        ],
        tris: vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]],
    });
}

/// ⭐⭐⭐ **SOBRE UM QUAD PLANO, UM SEGMENTO CONTINUA A SER UMA RECTA — e um ponto só, ao bit.**
///
/// ⚠️ **Não é um atalho escrito à mão:** sobre um quad o mapa é um **AFIM**, e um afim leva recta em
/// recta por definição. ⛔ Partir ali gastaria travessias de malha sem mover um pixel, e mudaria a
/// geometria emitida por toda a chrome que já existe — que é o que este gate impede.
#[test]
fn a_segment_over_a_flat_quad_is_still_one_straight_line() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();
    let (_, bits) = sprite_de_2m(&mut present, &mut sim);
    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let window = ph2d_host::WindowSize::new(800, 800);
    let afim = Affine::scale(1.0);
    let mapa = CanvasMap::new(present.world(), bits, 100, 100, afim, &camera, window);

    let (de, ate) = ([0.0f32, 0.0], [100.0f32, 0.0]);
    let mut pontos = Vec::new();
    mapa.segment(de, ate, |p| pontos.push(p));
    assert_eq!(pontos.len(), 1, "um quad plano não se parte: {pontos:?}");
    assert_eq!(
        pontos[0],
        mapa.point(ate),
        "e o ponto é o de sempre, ao bit"
    );
}

/// ⭐⭐⭐ **SOBRE ARTE DOBRADA, O SEGMENTO PARTE-SE — E OS PEDAÇOS ATERRAM NA ARTE.**
///
/// A régua tem as **duas** metades, e uma sozinha não afirma nada: (a) o segmento emite mais de um
/// ponto, e (b) cada ponto emitido está sobre a arte (é o que o mapa responde naquele parâmetro) e
/// **longe da corda** que a versão recta desenhava.
///
/// ⛔ Sem (b), um `segment` que emitisse `N` pontos igualmente espaçados **sobre a corda** passaria
/// a primeira metade — que é a forma mais barata de uma cura parecer feita.
#[test]
fn a_segment_over_bent_art_is_broken_where_the_art_bends() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();
    let (alvo, bits) = sprite_de_2m(&mut present, &mut sim);
    arte_dobrada(&mut present, alvo);
    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let window = ph2d_host::WindowSize::new(800, 800);
    let mapa = CanvasMap::new(
        present.world(),
        bits,
        100,
        100,
        Affine::scale(1.0),
        &camera,
        window,
    );
    assert!(mapa.is_mesh());

    let (de, ate) = ([0.0f32, 0.0], [100.0f32, 0.0]);
    let mut pontos = Vec::new();
    mapa.segment(de, ate, |p| pontos.push(p));
    assert!(
        pontos.len() > 1,
        "a linha atravessa a dobra e saiu inteira: {pontos:?}"
    );

    // (b) o pior desvio entre um pedaço e a arte tem de caber na tolerância…
    let (a, b) = (mapa.point(de), *pontos.last().expect("emitiu"));
    let mut anterior = a;
    let mut pior = 0.0_f64;
    for (k, p) in pontos.iter().enumerate() {
        let t0 = k as f64 / pontos.len() as f64;
        let t1 = (k + 1) as f64 / pontos.len() as f64;
        let meio = mapa.point([
            (f64::from(de[0]) * (1.0 - (t0 + t1) / 2.0) + f64::from(ate[0]) * (t0 + t1) / 2.0)
                as f32,
            0.0,
        ]);
        let corda = Point::new((anterior.x + p.x) / 2.0, (anterior.y + p.y) / 2.0);
        pior = pior.max((meio.x - corda.x).hypot(meio.y - corda.y));
        anterior = *p;
    }
    assert!(pior <= 0.5, "o pior desvio ficou em {pior} px");

    // …e a CORDA recta que a versão antiga desenhava está longe da arte, senão o gate de cima é
    // trivial (a fixtura não dobra).
    let meio_da_arte = mapa.point([50.0, 0.0]);
    let meio_da_corda = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    assert!(
        (meio_da_arte.x - meio_da_corda.x).hypot(meio_da_arte.y - meio_da_corda.y) > 10.0,
        "a fixtura não dobra o suficiente para separar as duas leis"
    );
}

/// Uma malha de `n × n` quadrados sobre a imagem inteira, **arqueada** — o tamanho e a forma que a
/// pele de uma imagem de facto entrega (`216` peças no `Fast`, mais no `Smooth`).
fn malha_de_pele(present: &mut PresentWorld, alvo: Entity, n: usize) {
    let mut q = present.world_mut().query::<(Entity, &SimRef)>();
    let e = q
        .iter(present.world())
        .find(|(_, r)| r.0 == alvo)
        .map(|(e, _)| e)
        .expect("o espelho foi criado");
    let (mut local, mut uv, mut tris) = (Vec::new(), Vec::new(), Vec::new());
    for j in 0..=n {
        for i in 0..=n {
            let (u, v) = (i as f32 / n as f32, j as f32 / n as f32);
            uv.push([u, v]);
            // O arco: a coluna sobe com um seno ao longo de `u`.
            let bojo = (u * std::f32::consts::PI).sin() * 0.8;
            local.push([u * 2.0, (1.0 - v) * 2.0 + bojo]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).expect("malha pequena");
    for j in 0..n {
        for i in 0..n {
            tris.push([idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)]);
            tris.push([idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)]);
        }
    }
    present
        .world_mut()
        .entity_mut(e)
        .insert(SpriteMesh { local, uv, tris });
}

/// ⭐ **O PREÇO DE UMA GRELHA SUBDIVIDIDA** — a medição que autoriza o tecto de pedaços.
///
/// ⛔ **`#[ignore]`, e não é um gate: ele IMPRIME.** Um tecto de relógio aqui seria mais um membro
/// da família de flakes de recurso do `CLAUDE.md` §5.0 — o que se quer é a ORDEM DE GRANDEZA contra
/// um quadro de `16,7 ms`, e ela decide-se uma vez.
///
/// O recurso é o **relógio do quadro**: a chrome do Painter redesenha-se toda a cada quadro, e cada
/// pedaço custa uma travessia da malha (`world_at_uv` varre triângulos).
#[test]
#[ignore = "sonda de relógio: imprime, não julga"]
fn measure_the_price_of_a_subdivided_grid() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();
    let (alvo, bits) = sprite_de_2m(&mut present, &mut sim);
    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let window = ph2d_host::WindowSize::new(800, 800);
    println!("[canvas-map] uma grelha de 40 linhas sobre a arte dobrada, mediana de 5 corridas:");
    for lado in [4usize, 8, 16, 24] {
        malha_de_pele(&mut present, alvo, lado);
        let mapa = CanvasMap::new(
            present.world(),
            bits,
            100,
            100,
            Affine::scale(1.0),
            &camera,
            window,
        );
        let mut amostras = Vec::new();
        let mut pedacos = 0usize;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            pedacos = 0;
            for k in 0..40 {
                let t = k as f32 * 2.5;
                // ⚠️ **HORIZONTAIS**, e a 1.ª redacção desta sonda usava verticais: nesta fixtura
                // o arco varia com `u`, logo uma linha de `u` constante **não dobra** e a sonda leu
                // `1` pedaço por linha — *uma medição sobre o eixo que não dobra mede o caso barato*.
                mapa.segment([0.0, t], [100.0, t], |p| {
                    pedacos += 1;
                    std::hint::black_box(p);
                });
            }
            amostras.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        amostras.sort_by(f64::total_cmp);
        let ms = amostras[2];
        println!(
            "  malha {:>5} tris  {pedacos:>5} pedacos  {ms:>7.3} ms  {:>5.1} % de um quadro",
            lado * lado * 2,
            ms / 16.7 * 100.0
        );
    }
}
