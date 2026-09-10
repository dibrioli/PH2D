//! Os gates da 2.ª mídia — a régua da imagem contra a régua do canvas.

use super::*;
use ph2d_ecs::Transform;

/// Uma sprite de `size` unidades de mundo, sem âncora.
fn sprite(w: f32, h: f32, ax: f32, ay: f32) -> Sprite {
    Sprite {
        anchor: [ax, ay],
        ..Sprite::atlas(0, [w, h], [1.0; 4])
    }
}

/// ⭐⭐⭐ **O CANTO DA IMAGEM É O CANTO DO QUAD, E O `y` VIRA.**
///
/// ⛔⛔ **É a única inversão de todo o módulo, e esquecê-la desenha o personagem de cabeça para
/// baixo** — um defeito que passa por TODO gate de geometria (a malha está certa, a deformação
/// está certa, e a imagem está ao contrário). Uma imagem tem o `y` a crescer para baixo; o mundo
/// tem-no a crescer para cima.
///
/// (Mutação: tirar o sinal de `-sy / h` ⇒ RED nos dois cantos de `y`.)
#[test]
fn the_image_corner_is_the_quad_corner_and_the_y_axis_flips() {
    // Uma imagem de 100×50 pixels ocupando 4×2 unidades de mundo, centrada no pivô.
    let s = sprite(4.0, 2.0, 0.0, 0.0);
    let m = pixel_to_local(&s, [100, 50]).expect("a imagem tem lado");
    let perto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    // (0,0) é o canto SUPERIOR esquerdo da textura ⇒ o canto de cima-esquerda do quad.
    assert!(
        perto(m.apply([0.0, 0.0]), [-2.0, 1.0]),
        "o pixel (0,0) saiu em {:?} — ele e' o canto de CIMA a' esquerda",
        m.apply([0.0, 0.0])
    );
    // (w,h) é o canto inferior direito.
    assert!(
        perto(m.apply([100.0, 50.0]), [2.0, -1.0]),
        "o pixel (w,h) saiu em {:?}",
        m.apply([100.0, 50.0])
    );
    // E o centro da imagem cai no pivô.
    assert!(perto(m.apply([50.0, 25.0]), [0.0, 0.0]));
}

/// ⭐⭐ **A ÂNCORA DESLOCA O QUAD INTEIRO, e não a régua da imagem.**
///
/// ⚠️ Ela é o vector do pivô ao **centro** do quad, então move os quatro cantos por igual — se
/// entrasse na escala, uma sprite com pivô fora do centro sairia esticada.
#[test]
fn the_anchor_moves_the_whole_quad_and_not_the_scale() {
    let sem = pixel_to_local(&sprite(4.0, 2.0, 0.0, 0.0), [100, 50]).expect("lado");
    let com = pixel_to_local(&sprite(4.0, 2.0, 1.5, -0.25), [100, 50]).expect("lado");
    for p in [[0.0, 0.0], [100.0, 50.0], [37.0, 11.0]] {
        let (a, b) = (sem.apply(p), com.apply(p));
        assert!(
            (b[0] - a[0] - 1.5).abs() < 1e-9 && (b[1] - a[1] + 0.25).abs() < 1e-9,
            "o ponto {p:?} deslocou {:?} em vez do vector da ancora",
            [b[0] - a[0], b[1] - a[1]]
        );
    }
    // ⛔ Uma imagem de lado zero não tem régua.
    assert_eq!(pixel_to_local(&sprite(4.0, 2.0, 0.0, 0.0), [0, 50]), None);
}

/// ⭐⭐ **SÓ O CANAL ALFA DECIDE A SILHUETA.**
///
/// ⚠️ Passar as três cores junto daria ao traçador três respostas para a mesma pergunta. Este gate
/// mede-o pelo caso que separa: um quadrado **preto** e um **branco** com o mesmo alfa dão a mesma
/// malha, e um transparente não dá nenhuma.
#[test]
fn only_the_alpha_channel_decides_the_silhouette() {
    let faz = |cor: [u8; 3], alfa: u8| -> Vec<u8> {
        let mut v = vec![0u8; 20 * 20 * 4];
        for y in 5..15 {
            for x in 5..15 {
                let i = (y * 20 + x) * 4;
                v[i..i + 3].copy_from_slice(&cor);
                v[i + 3] = alfa;
            }
        }
        v
    };
    let opts = ph2d_poly2d::MeshOptions::default();
    let preto = mesh_from_rgba(&faz([0, 0, 0], 255), 20, 20, opts).expect("ha' alfa");
    let branco = mesh_from_rgba(&faz([255, 255, 255], 255), 20, 20, opts).expect("ha' alfa");
    assert_eq!(
        preto, branco,
        "a cor mudou a malha — so' o ALFA pode decidir a silhueta"
    );
    assert_eq!(
        mesh_from_rgba(&faz([255, 255, 255], 0), 20, 20, opts),
        None,
        "tinta branca com alfa ZERO nao e' tinta"
    );
}

/// ⭐⭐⭐ **UMA IMAGEM PRESA A UM ESQUELETO PARADO NÃO SE MEXE UM PIXEL.**
///
/// ⚠️ É a mesma lei que o bind de uma forma já declara (*«a pose de repouso é a identidade por
/// construção, logo prender não move um pixel»*), e ela é o **controlo** de tudo o resto: sem
/// ela, um erro de sinal na régua da imagem passaria despercebido no meio da deformação.
///
/// (Mutação: qualquer troca de sinal no [`pixel_to_local`] ⇒ RED.)
#[test]
fn binding_an_image_to_a_still_skeleton_moves_nothing() {
    let mut sim = ph2d_ecs::SimWorld::default();
    // Um osso deitado sobre o eixo X, e uma imagem por cima dele.
    let osso = crate::bone_gesture::create(&mut sim, None, [0.0, 0.0], [4.0, 0.0]).expect("osso");
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
        .id();
    let mut rgba = vec![0u8; 40 * 20 * 4];
    for y in 4..16 {
        for x in 4..36 {
            rgba[(y * 40 + x) * 4 + 3] = 255;
        }
    }
    assert!(
        crate::skeleton_live::bind_image(
            &mut sim,
            e,
            &rgba,
            [40, 20],
            ph2d_poly2d::MeshOptions::default(),
            Some(ph2d_ecs::Entity::from_bits(osso)),
        ),
        "o bind tinha de acontecer: ha' osso, ha' tinta e a pose nao e' singular"
    );
    let malha = malha_de(&sim, e).expect("a malha esta' guardada nos bytes opacos");
    let posados = posed_local(&sim, e, &malha).expect("a pele resolve");
    let p2l =
        pixel_to_local(sim.world().get::<Sprite>(e).expect("sprite"), malha.size).expect("regua");
    for (i, &r) in malha.rest.iter().enumerate() {
        let repouso = p2l.apply(r);
        let agora = posados[i];
        assert!(
            (agora[0] - repouso[0]).abs() < 1e-6 && (agora[1] - repouso[1]).abs() < 1e-6,
            "o vertice {i} saiu de {repouso:?} para {agora:?} sem ninguem mexer no osso"
        );
    }
}

/// ⭐⭐⭐ **GIRAR O OSSO LEVA A IMAGEM COM ELE** — a prova de que a 2.ª mídia de facto deforma.
///
/// ⚠️ E o **controlo** está dentro do gate: um ponto longe do alcance do osso tem de ficar onde
/// estava. Sem ele, um bug que movesse TUDO por igual (uma translação global) passaria.
#[test]
fn turning_the_bone_carries_the_image() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let osso = crate::bone_gesture::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
        .id();
    let mut rgba = vec![0u8; 40 * 20 * 4];
    for y in 4..16 {
        for x in 4..36 {
            rgba[(y * 40 + x) * 4 + 3] = 255;
        }
    }
    let raiz = ph2d_ecs::Entity::from_bits(osso);
    assert!(crate::skeleton_live::bind_image(
        &mut sim,
        e,
        &rgba,
        [40, 20],
        ph2d_poly2d::MeshOptions::default(),
        Some(raiz),
    ));
    let malha = malha_de(&sim, e).expect("malha");
    let antes = posed_local(&sim, e, &malha).expect("pele");
    // Gira o osso um quarto de volta.
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(raiz)
            .expect("Transform");
        t.rotation = std::f32::consts::FRAC_PI_2;
    }
    let depois = posed_local(&sim, e, &malha).expect("pele");
    let mexeu = antes
        .iter()
        .zip(&depois)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f64, f64::max);
    assert!(
        mexeu > 0.5,
        "girar o osso nao moveu a malha (maior deslocamento {mexeu}) — a imagem nao obedece"
    );
}

/// **A malha guardada nos bytes opacos da pele.** Ela vive aqui, no arnês, porque no produto quem
/// a lê é o desenho — e um segundo leitor no caminho do quadro seria a segunda porta.
fn malha_de(sim: &ph2d_ecs::SimWorld, e: ph2d_ecs::Entity) -> Option<ph2d_poly2d::Mesh2d> {
    let skin = sim.world().get::<ph2d_skeleton_ecs::SkinBind>(e)?;
    postcard::from_bytes(&skin.source).ok()
}
