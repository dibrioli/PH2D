//! Os gates do [`MaskTransform`].
//!
//! O oráculo central **não confere a fórmula que eu escrevi** — ele pergunta uma
//! propriedade que a geometria tem de ter: *girar não muda a distância ao eixo*.
//! É isso que o torna capaz de reprovar a lei da referência (o lerp), que passa
//! por qualquer teste que compare o resultado com a própria expressão.

use super::*;
use ph2d_mesh::shapes;

/// A distância ao eixo Y — a grandeza que um giro em torno de Y preserva.
fn radius_from_y(p: [f32; 3]) -> f32 {
    p[0].mul_add(p[0], p[2] * p[2]).sqrt()
}

/// Uma esfera com máscara SUAVE: o peso livre varre 0..1 com a latitude, então
/// a fixture **contém a banda de transição** — e é ela que separa as duas leis.
/// Uma máscara dura (0 ou 1) faria lerp e fração concordarem em todo vértice.
fn soft_masked_sphere() -> Mesh {
    let mut mesh = shapes::uv_sphere(32, 48, 1.0);
    let n = mesh.vert_count();
    let w: Vec<f32> = (0..n)
        .map(|i| (0.5 + mesh.positions()[i][1]).clamp(0.0, 1.0))
        .collect();
    let m = mesh.masks_mut();
    for i in 0..n {
        m[i] = 1.0 - w[i];
    }
    mesh
}

/// Metade protegida DURA: `y > 0` livre, `y <= 0` pregado.
fn half_masked_sphere() -> Mesh {
    let mut mesh = shapes::uv_sphere(32, 48, 1.0);
    let n = mesh.vert_count();
    let up: Vec<bool> = (0..n).map(|i| mesh.positions()[i][1] > 0.0).collect();
    let m = mesh.masks_mut();
    for i in 0..n {
        m[i] = if up[i] { 0.0 } else { 1.0 };
    }
    mesh
}

#[test]
fn a_weighted_rotation_preserves_the_distance_to_the_axis() {
    let mut mesh = soft_masked_sphere();
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    let before: Vec<[f32; 3]> = t.base_positions().to_vec();

    t.apply(
        &mut mesh,
        &Gesture::Rotate {
            axis: [0.0, 1.0, 0.0],
            radians: std::f32::consts::PI,
        },
    );

    // ⚠️ Meia volta é o ângulo em que a lei da referência **colapsa** o vértice
    // de meio peso sobre o eixo (medido: 0,00000). A fração exata não move a
    // distância de vértice nenhum.
    let mut worst = 0.0f32;
    for (k, &i) in t.moving().iter().enumerate() {
        let r0 = radius_from_y(before[k]);
        let r1 = radius_from_y(mesh.positions()[i as usize]);
        worst = worst.max((r1 - r0).abs());
    }
    assert!(
        worst < 1.0e-5,
        "girar mudou a distancia ao eixo em ate' {worst} -- a lei nao e' uma rotacao"
    );
}

#[test]
fn the_protected_part_is_not_even_listed_and_the_free_part_carries_its_weight() {
    let mesh = half_masked_sphere();
    let t = MaskTransform::begin(&mesh).expect("ha' o que mover");

    // ⚠️ A metade pregada não entra na lista: ela não é lida, não é escrita e
    // não aparece na janela do undo. Inverter a convenção da máscara troca as
    // duas metades e este gate sangra nas DUAS pontas.
    for &i in t.moving() {
        let y = mesh.positions()[i as usize][1];
        assert!(
            y > 0.0,
            "um vertice PREGADO (y={y}) entrou na lista de moveis"
        );
    }
    let free = (0..mesh.vert_count())
        .filter(|&i| mesh.positions()[i][1] > 0.0)
        .count();
    assert_eq!(
        t.moving_count(),
        free,
        "a lista de moveis nao e' a metade livre"
    );
}

#[test]
fn a_quarter_masked_vertex_moves_three_quarters_of_the_way() {
    // ⚠️ **A PRIMEIRA versão deste gate era um ESPELHO.** Ela computava o
    // esperado chamando `free_weight` — a própria porta sob teste —, então
    // inverter a convenção invertia os DOIS lados e ele ficava **VERDE** no meio
    // de uma rodada em que 57 outros sangravam. *Um oráculo que usa a função sob
    // teste para computar o que espera é sempre verde.*
    //
    // A convenção agora está escrita como NÚMERO: `mask = 0,25` é um vértice um
    // quarto protegido, logo ele anda **três quartos** do gesto.
    let mut mesh = shapes::uv_sphere(8, 12, 1.0);
    let n = mesh.vert_count();
    let quarters: Vec<f32> = (0..n).map(|i| (i % 5) as f32 * 0.25).collect();
    {
        let m = mesh.masks_mut();
        m[..n].copy_from_slice(&quarters);
    }
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    let mut moved = mesh.clone();
    t.apply(
        &mut moved,
        &Gesture::Move {
            delta: [1.0, 0.0, 0.0],
        },
    );
    for (k, &i) in t.moving().iter().enumerate() {
        let dx = moved.positions()[i as usize][0] - t.base_positions()[k][0];
        let want = 1.0 - quarters[i as usize];
        assert!(
            (dx - want).abs() < 1.0e-6,
            "vertice de mascara {}: andou {dx}, devia andar {want}",
            quarters[i as usize]
        );
    }
    // E o totalmente protegido (`1,00`) não pode estar na lista.
    let protegidos = quarters.iter().filter(|&&m| m >= 1.0).count();
    assert_eq!(t.moving_count(), n - protegidos);
}

#[test]
fn a_fully_protected_mesh_has_nothing_to_transform() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    let n = mesh.vert_count();
    mesh.masks_mut()[..n].fill(1.0);
    assert!(
        MaskTransform::begin(&mesh).is_none(),
        "uma malha inteiramente pregada devolveu uma sessao de transform"
    );
}

#[test]
fn the_pivot_is_the_weighted_centre_of_what_moves() {
    // ⚠️ **A fixture DURA não continha o fenômeno.** Com `mask` em 0 ou 1 todo
    // vértice móvel pesa exatamente 1, então o centroide ponderado e o simples
    // são **o mesmo ponto** — e a mutação *"não pondere"* sobreviveu a esta
    // suíte inteira. O peso só é observável numa máscara MACIA, que é também a
    // que o artista pinta.
    let mesh = soft_masked_sphere();
    let t = MaskTransform::begin(&mesh).expect("ha' o que mover");

    // O centroide SIMPLES do mesmo conjunto — outra fórmula, não a função sob
    // teste.
    let mut plain = [0.0f64; 3];
    for p in t.base_positions() {
        for k in 0..3 {
            plain[k] += f64::from(p[k]);
        }
    }
    let plain_y = (plain[1] / t.moving_count() as f64) as f32;

    // O peso cresce com a latitude, então ponderar PUXA o pivô para cima.
    assert!(
        t.pivot()[1] > plain_y + 0.05,
        "o pivo ({}) nao esta' acima do centroide simples ({plain_y}) -- ele nao foi ponderado",
        t.pivot()[1]
    );
    // E, com a metade de baixo pregada, ele também não é o centro da PEÇA.
    assert!(t.pivot()[1] > 0.1, "o pivo caiu no meio da peca inteira");
}

#[test]
fn the_same_gesture_applied_twice_lands_in_the_same_place() {
    let mut mesh = soft_masked_sphere();
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    let g = Gesture::Rotate {
        axis: [0.0, 1.0, 0.0],
        radians: 0.7,
    };
    t.apply(&mut mesh, &g);
    let once: Vec<[f32; 3]> = mesh.positions().to_vec();
    t.apply(&mut mesh, &g);
    // ⚠️ Idempotência é o que o `pre` congelado COMPRA: sem ele a segunda
    // aplicação giraria o resultado da primeira.
    assert_eq!(
        mesh.positions(),
        &once[..],
        "aplicar o mesmo gesto duas vezes moveu a malha de novo"
    );
}

#[test]
fn twenty_small_steps_land_where_one_big_step_lands() {
    let g_total = Gesture::Rotate {
        axis: [0.3, 1.0, -0.2],
        radians: 1.1,
    };

    let mut one = soft_masked_sphere();
    let mut t1 = MaskTransform::begin(&one).expect("ha' o que mover");
    t1.apply(&mut one, &g_total);

    let mut many = soft_masked_sphere();
    let mut t2 = MaskTransform::begin(&many).expect("ha' o que mover");
    // O gesto TOTAL cresce; a malha é sempre reescrita do `pre`. É assim que um
    // arrasto chega: vinte eventos de ponteiro, cada um com o total até ali.
    for step in 1..=20u8 {
        t2.apply(
            &mut many,
            &Gesture::Rotate {
                axis: [0.3, 1.0, -0.2],
                radians: 1.1 * f32::from(step) / 20.0,
            },
        );
    }

    // ⚠️ **A lei que este modulo ja' pagou:** um transform incremental faria o
    // resultado depender da taxa de polling do mouse.
    assert_eq!(
        one.positions(),
        many.positions(),
        "vinte passos nao chegaram onde um passo chega -- o gesto e' incremental"
    );
}

#[test]
fn growing_and_shrinking_by_the_same_ratio_are_mirror_gestures() {
    // ⚠️ **A PRIMEIRA versão deste gate pedia o que nenhuma escala ponderada
    // entrega:** *"escalar por 2 duas vezes é escalar por 4"*, re-congelando
    // entre as duas. Ele reprovou por **0,0768**, e o código estava certo — o
    // pivô é o centroide PONDERADO, e uma escala de peso variável **move** esse
    // centroide (`s^w` correlaciona com `w`), então a segunda sessão escala em
    // torno de outro ponto. Composição entre SESSÕES não é uma propriedade que
    // um pivô re-derivado possa ter, e a de dentro da sessão já é o gate dos
    // vinte passos.
    //
    // A propriedade que **discrimina** as duas leis, e que o artista de fato
    // sente: crescer por `s` e encolher por `1/s` são gestos espelhados, então
    // os dois fatores efetivos multiplicam **exatamente 1** em todo peso.
    // `s^w · s^-w = 1` sempre; o lerp da referência dá
    // `(1+w(s−1))·(1+w(1/s−1))`, que em `s=2, w=½` vale **1,125**.
    let mesh = soft_masked_sphere();
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    let pivot = t.pivot();
    let radius = |p: [f32; 3]| {
        let d = [p[0] - pivot[0], p[1] - pivot[1], p[2] - pivot[2]];
        d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt()
    };

    let mut up = mesh.clone();
    t.apply(&mut up, &Gesture::Scale { factor: 2.0 });
    let mut down = mesh.clone();
    t.apply(&mut down, &Gesture::Scale { factor: 0.5 });

    let mut worst = 0.0f32;
    for (k, &i) in t.moving().iter().enumerate() {
        let r0 = radius(t.base_positions()[k]);
        if r0 < 1.0e-3 {
            continue; // um vértice NO pivô não tem razão a medir.
        }
        let product =
            (radius(up.positions()[i as usize]) / r0) * (radius(down.positions()[i as usize]) / r0);
        worst = worst.max((product - 1.0).abs());
    }
    assert!(
        worst < 1.0e-5,
        "crescer e encolher pela mesma razao nao se cancelam: erro ate' {worst}"
    );
}

#[test]
fn without_a_mask_the_whole_piece_moves_rigidly() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut t = MaskTransform::begin(&mesh).expect("sem mascara, tudo se move");
    assert_eq!(t.moving_count(), mesh.vert_count());
    let delta = [0.3, -0.2, 0.1];
    t.apply(&mut mesh, &Gesture::Move { delta });
    // ⚠️ Sem máscara o peso é 1 em toda parte, então o Move é uma translação
    // RÍGIDA — a forma não pode deformar.
    for (k, &i) in t.moving().iter().enumerate() {
        let p = mesh.positions()[i as usize];
        let b = t.base_positions()[k];
        for a in 0..3 {
            assert!(
                (p[a] - b[a] - delta[a]).abs() < 1.0e-6,
                "sem mascara o vertice {i} nao andou o delta inteiro"
            );
        }
    }
}

#[test]
fn the_neutral_gesture_puts_every_vertex_back() {
    for kind in TransformKind::ALL {
        let mut mesh = soft_masked_sphere();
        let before: Vec<[f32; 3]> = mesh.positions().to_vec();
        let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
        t.apply(
            &mut mesh,
            &Gesture::Rotate {
                axis: [0.0, 0.0, 1.0],
                radians: 0.9,
            },
        );
        t.apply(&mut mesh, &Gesture::neutral(kind));
        for (i, (now, was)) in mesh.positions().iter().zip(&before).enumerate() {
            for (a, (n, w)) in now.iter().zip(was).enumerate() {
                assert!(
                    (n - w).abs() < 1.0e-5,
                    "o gesto neutro de {} nao devolveu o eixo {a} do vertice {i}",
                    kind.label()
                );
            }
        }
    }
}

#[test]
fn a_degenerate_axis_does_not_poison_the_mesh() {
    let mut mesh = soft_masked_sphere();
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    // ⚠️ Um eixo nulo faria o Rodrigues devolver `NaN` para a malha INTEIRA, e
    // `NaN` numa posição envenena a normal, a octree e o passe de render.
    t.apply(
        &mut mesh,
        &Gesture::Rotate {
            axis: [0.0; 3],
            radians: 1.0,
        },
    );
    for i in 0..mesh.vert_count() {
        assert!(
            mesh.positions()[i].iter().all(|c| c.is_finite()),
            "o vertice {i} saiu nao-finito de um eixo degenerado"
        );
    }
}

#[test]
fn a_collapsing_scale_is_floored_instead_of_folding_the_piece() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut t = MaskTransform::begin(&mesh).expect("ha' o que mover");
    t.apply(&mut mesh, &Gesture::Scale { factor: -3.0 });
    // Um fator negativo não é uma escala: é um espelho. O piso o transforma
    // numa peça muito pequena — visível, e desfazível — em vez de uma peça
    // virada do avesso que o artista não consegue nomear.
    for i in 0..mesh.vert_count() {
        let p = mesh.positions()[i];
        assert!(p.iter().all(|c| c.is_finite()));
        let d = [
            p[0] - t.pivot()[0],
            p[1] - t.pivot()[1],
            p[2] - t.pivot()[2],
        ];
        let r = d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt();
        assert!(
            r <= MIN_SCALE_FACTOR * 2.0,
            "o fator negativo passou: r={r}"
        );
    }
}
