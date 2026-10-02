//! ⭐⭐⭐ **O RELEVO DA TINTA ACENDE A PEÇA, e só onde há espessura** — a
//! metade de PIXEL da etapa 3b (`docs/3D/29`).
//!
//! A bancada de paridade prova que a placa lê a MESMA altura que a CPU; este
//! gate prova que essa altura chega à LUZ: um degrau de espessura inclina a
//! normal de sombreado (Mikkelsen 2010, derivadas de ecrã) e a peça muda de
//! brilho **na faixa do degrau e em mais sítio nenhum**.
//!
//! ⚠️ **As duas metades são a lei inteira:** *«o relevo muda a luz»* sozinho
//! passaria com um relevo que inclinasse a peça TODA (um sinal trocado no
//! gradiente, uma altura lida no endereço errado); *«só na faixa»* sozinho
//! passaria com um relevo inerte. E o CONTROLO é a fixtura sem relevo — com
//! ela as duas imagens têm de ser a MESMA, senão a diferença medida não é do
//! relevo.
//!
//! ⭐⭐ **E a terceira metade é o report do dono de 01/10** (*«o traço tem um
//! relevo indesejado na borda»*): a MESMA rampa com CORPO zero — a espessura
//! que o alisamento do impasto espalha para fora da tinta — **não acende
//! nada**, como no passe de luz 2D do Painter.

use ph2d_light::LightRig;
use ph2d_mesh::{Face, Mesh};
use ph2d_mesh_colors::Tinta;
use ph2d_mesh_render::MeshRenderer;

use super::device_de_teste::device;
use super::gpu_render::{FORMAT, H, W, camera_for, lum, render_using_rig_shade, rig_shade};

/// `N×N` quads no plano `xy`, de `-1` a `1`, virados para `+z` (a câmara).
const N: usize = 8;

fn grelha() -> Mesh {
    grelha_com(|_| 0.0)
}

/// A mesma grelha com cada vértice levantado `z(x)` — a rampa como GEOMETRIA.
fn grelha_com(z: impl Fn(f32) -> f32) -> Mesh {
    let lado = N + 1;
    let pos: Vec<[f32; 3]> = (0..lado)
        .flat_map(|j| (0..lado).map(move |i| (i, j)))
        .map(|(i, j)| {
            let x = -1.0 + 2.0 * i as f32 / N as f32;
            let y = -1.0 + 2.0 * j as f32 / N as f32;
            [x, y, z(x)]
        })
        .collect();
    let v = |i: usize, j: usize| (j * lado + i) as u32;
    let faces: Vec<Face> = (0..N)
        .flat_map(|j| (0..N).map(move |i| (i, j)))
        .map(|(i, j)| Face::quad(v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1)))
        .collect();
    Mesh::from_parts(pos, faces).expect("a grelha é válida")
}

/// A rampa: zero à esquerda de `x = -0,25`, `ALTURA` à direita de `0,25`,
/// linear entre as duas — logo SÓ as duas colunas do meio têm gradiente.
const ALTURA: f32 = 0.25;

fn altura_em(x: f32) -> f32 {
    ((x + 0.25) / 0.5).clamp(0.0, 1.0) * ALTURA
}

fn desenha(device: &wgpu::Device, queue: &wgpu::Queue, m: &Mesh, t: &Tinta) -> Vec<u8> {
    let mut r = MeshRenderer::new(device, FORMAT);
    r.upload_at(device, queue, 0, m, &[]);
    r.upload_tinta_at(device, queue, 0, m, Some(t));
    render_using_rig_shade(
        device,
        queue,
        &mut r,
        &camera_for(m),
        &LightRig::default(),
        rig_shade(),
    )
}

/// ⭐⭐⭐ **GATE — a luz muda na faixa do degrau e em mais sítio nenhum.**
#[test]
#[ignore = "precisa de adaptador"]
fn o_relevo_inclina_a_luz_so_onde_ha_degrau() {
    let Some((device, queue)) = device() else {
        panic!("sem adaptador — este gate não é verde por skip");
    };
    let m = grelha();
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();

    // ⚠️ Nível `0`: a retícula é só os cantos, logo a amostra `i` É o vértice
    //   `i` e a altura escreve-se pela posição dele, sem lei nenhuma a meio.
    let mut plano = Tinta::nova(m.vert_count(), faces.iter().map(|f| &f[..]), 0);
    assert_eq!(
        plano.amostras().len(),
        m.vert_count(),
        "a fixtura supõe uma amostra por vértice no nível 0"
    );
    for a in plano.amostras_mut() {
        *a = [0.8, 0.8, 0.8];
    }

    // (1) CONTROLO: o mesmo plano, desenhado duas vezes sem relevo, é a mesma
    //     imagem — e cobre a tela (senão o resto mede o fundo).
    let liso = desenha(&device, &queue, &m, &plano);
    assert_eq!(
        liso,
        desenha(&device, &queue, &m, &plano),
        "o desenho não é determinístico"
    );

    // A faixa do ecrã onde a grelha cai, lida da própria imagem.
    let mut x0 = W;
    let mut x1 = 0;
    for y in 0..H {
        for x in 0..W {
            if lum(&liso, x, y) > 0.0 {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    assert!(x1 > x0 + W / 3, "a grelha não cobre a tela ({x0}..{x1})");

    // (2) O RELEVO: a rampa só nas duas colunas do meio.
    for (i, p) in m.positions().iter().enumerate() {
        plano.relevo_mut()[i] = [altura_em(p[0]), 1.0];
    }
    let com = desenha(&device, &queue, &m, &plano);

    // (3) ⭐⭐ A MESMA rampa sem CORPO não acende — o anel do report de 01/10.
    //     ⚠️ A folga é a mesma do «fora»: o arredondamento de renormalizar,
    //     nunca uma inclinação.
    let mut sem_corpo = plano.clone();
    for r in sem_corpo.relevo_mut() {
        r[1] = 0.0;
    }
    let nu = desenha(&device, &queue, &m, &sem_corpo);
    let pior_nu = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .map(|(x, y)| (lum(&nu, x, y) - lum(&liso, x, y)).abs())
        .fold(0.0f32, f32::max);
    assert!(
        pior_nu <= 1.5,
        "a espessura SEM tinta acendeu a peça: {pior_nu} — o anel do report de 01/10"
    );

    let largura = (x1 - x0) as f32;
    let meio = (x0 + x1) as f32 / 2.0;
    // As duas colunas ocupam `1/4` da largura (`1/8` cada). ⚠️ A inclinação
    // é POR AMOSTRA (`docs/3D/29` §9): a amostra da beira da rampa dá metade
    // do declive à coluna vizinha, como uma normal por vértice — logo a faixa
    // é a rampa e UMA coluna de cada lado, e além dela nada (`+2` px de folga
    // da rasterização).
    let dentro = |x: u32| ((x as f32) - meio).abs() <= largura / 4.0 + 2.0;
    let mut mudou_dentro = 0usize;
    let mut pior_fora = 0.0f32;
    let mut onde_fora = (0, 0);
    for y in 0..H {
        for x in 0..W {
            let d = (lum(&com, x, y) - lum(&liso, x, y)).abs();
            if dentro(x) {
                if d > 4.0 {
                    mudou_dentro += 1;
                }
            } else if d > pior_fora {
                pior_fora = d;
                onde_fora = (x, y);
            }
        }
    }
    assert!(
        mudou_dentro > 100,
        "o relevo não chegou à luz: só {mudou_dentro} píxeis mudaram na faixa do degrau"
    );
    // ⚠️ Fora da faixa o gradiente é ZERO e a normal é a mesma — a folga de
    //   `1,5` é o arredondamento de renormalizar `|det|·n` (um ULP pode virar
    //   um byte), nunca uma inclinação.
    assert!(
        pior_fora <= 1.5,
        "o relevo mudou a luz FORA do degrau: {pior_fora} em {onde_fora:?}"
    );
}

/// ⭐⭐⭐ **GATE — o relevo acende como a GEOMETRIA que ele finge** (report do
/// dono de 01/10, a vista inclinada). A normal do relevo deixou de vir de
/// diferenças de ecrã e passou a ser o gradiente EXACTO da altura; a régua que
/// prova que ela aponta para o lado certo é a própria malha levantada pela
/// mesma rampa. Na faixa do degrau as duas imagens mudam a luz para o MESMO
/// lado e com força da mesma ordem.
///
/// ⚠️ Sem esta régua um gradiente com o SINAL trocado passava no gate de cima,
/// que só pergunta ONDE a luz muda. O CONTROLO é a fixtura ter a mudança:
/// a geometria tem de acender a faixa por mais do que o arredondamento.
#[test]
#[ignore = "precisa de adaptador"]
fn o_relevo_acende_como_a_geometria_que_ele_finge() {
    let Some((device, queue)) = device() else {
        panic!("sem adaptador — este gate não é verde por skip");
    };
    let m = grelha();
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let mut plano = Tinta::nova(m.vert_count(), faces.iter().map(|f| &f[..]), 0);
    for a in plano.amostras_mut() {
        *a = [0.8, 0.8, 0.8];
    }
    let liso = desenha(&device, &queue, &m, &plano);
    let levantada = grelha_com(altura_em);
    let geometria = desenha(&device, &queue, &levantada, &plano);
    for (i, p) in m.positions().iter().enumerate() {
        plano.relevo_mut()[i] = [altura_em(p[0]), 1.0];
    }
    let relevo = desenha(&device, &queue, &m, &plano);

    // A faixa do degrau: as duas colunas do meio, sem as bordas delas, onde
    // a malha levantada interpola a normal com a coluna vizinha.
    let (mut x0, mut x1) = (W, 0);
    for y in 0..H {
        for x in 0..W {
            if lum(&liso, x, y) > 0.0 {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    let largura = (x1 - x0) as f32;
    let meio = (x0 + x1) as f32 / 2.0;
    let (mut soma_g, mut soma_r, mut n) = (0.0f32, 0.0f32, 0usize);
    for y in H / 4..3 * H / 4 {
        for x in 0..W {
            if ((x as f32) - meio).abs() <= largura / 16.0 {
                soma_g += lum(&geometria, x, y) - lum(&liso, x, y);
                soma_r += lum(&relevo, x, y) - lum(&liso, x, y);
                n += 1;
            }
        }
    }
    let (g, r) = (soma_g / n as f32, soma_r / n as f32);
    assert!(
        g.abs() > 4.0,
        "CONTROLO: a rampa levantada não mudou a luz ({g})"
    );
    assert!(
        g.signum() == r.signum(),
        "o relevo acende para o lado CONTRÁRIO da geometria: {r} contra {g}"
    );
    let razao = r / g;
    assert!(
        (0.5..=2.0).contains(&razao),
        "o relevo acende {razao}× a geometria que finge ({r} contra {g})"
    );
}

/// ⭐⭐⭐ **GATE — esculpir a peça com relevo e subir de novo desenha o MESMO
/// que uma subida do zero** (`docs/3D/29` §9). A subida inteira refaz só as
/// inclinações das faces cujas posições mudaram desde a foto dela; se a foto
/// mentir, a luz da peça esculpida fica com a inclinação da forma de ANTES.
#[test]
#[ignore = "precisa de adaptador"]
fn esculpir_com_relevo_desenha_o_que_uma_subida_do_zero_desenha() {
    let Some((device, queue)) = device() else {
        panic!("sem adaptador — este gate não é verde por skip");
    };
    let m = grelha();
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let mut plano = Tinta::nova(m.vert_count(), faces.iter().map(|f| &f[..]), 0);
    for a in plano.amostras_mut() {
        *a = [0.8, 0.8, 0.8];
    }
    for (i, p) in m.positions().iter().enumerate() {
        plano.relevo_mut()[i] = [altura_em(p[0]), 1.0];
    }
    // A peça esculpida: um calombo em `z` em cima da rampa, com a MESMA
    // topologia — o caminho que reaproveita a foto.
    let mut pos = m.positions().to_vec();
    for p in &mut pos {
        let d2 = p[0] * p[0] + (p[1] - 0.25) * (p[1] - 0.25);
        p[2] += 0.3 * (-d2 / 0.08).exp();
    }
    let esculpida = Mesh::from_parts(pos, m.faces().to_vec()).expect("a mesma topologia");

    let camera = camera_for(&m);
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &m, &[]);
    r.upload_tinta_at(&device, &queue, 0, &m, Some(&plano));
    let antes = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );
    r.upload_at(&device, &queue, 0, &esculpida, &[]);
    r.upload_tinta_at(&device, &queue, 0, &esculpida, Some(&plano));
    let reaproveitada = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );

    let mut zero = MeshRenderer::new(&device, FORMAT);
    zero.upload_at(&device, &queue, 0, &esculpida, &[]);
    zero.upload_tinta_at(&device, &queue, 0, &esculpida, Some(&plano));
    let do_zero = render_using_rig_shade(
        &device,
        &queue,
        &mut zero,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );

    let pior = |a: &[u8], b: &[u8]| {
        (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .map(|(x, y)| (lum(a, x, y) - lum(b, x, y)).abs())
            .fold(0.0f32, f32::max)
    };
    assert!(
        pior(&antes, &do_zero) > 10.0,
        "o CONTROLO: esculpir tinha de mudar a luz da peça"
    );
    // ⚠️ A folga é a ordem de acumulação das médias (refeitas por pedaços
    //   contra refeitas inteiras), que pode virar um byte — nunca a forma de
    //   antes, que muda dezenas.
    let d = pior(&reaproveitada, &do_zero);
    assert!(
        d <= 1.5,
        "a subida que reaproveita a foto desenha {d} longe da do zero"
    );
}

/// ⭐⭐⭐ **GATE — um traço de impasto subido POR PEDAÇOS desenha o MESMO que
/// uma subida do zero** (`docs/3D/29` §9). Durante o traço só as amostras
/// escritas sobem; as inclinações que mudam são MAIS do que elas (as das
/// células vizinhas), e se o incremental as esquecer a luz fica com a encosta
/// de antes.
#[test]
#[ignore = "precisa de adaptador"]
fn o_relevo_subido_por_pedacos_desenha_o_que_uma_subida_do_zero_desenha() {
    let Some((device, queue)) = device() else {
        panic!("sem adaptador — este gate não é verde por skip");
    };
    let m = grelha();
    let faces: Vec<Vec<u32>> = m.faces().iter().map(|f| f.verts().to_vec()).collect();
    let mut plano = Tinta::nova(m.vert_count(), faces.iter().map(|f| &f[..]), 2);
    for a in plano.amostras_mut() {
        *a = [0.8, 0.8, 0.8];
    }
    // O relevo nasce com corpo em toda parte e altura nenhuma: o device fica
    // com o bit e as inclinações nulas — o estado do pen-down.
    for r in plano.relevo_mut() {
        *r = [0.0, 1.0];
    }
    let camera = camera_for(&m);
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &m, &[]);
    r.upload_tinta_at(&device, &queue, 0, &m, Some(&plano));
    let liso = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );

    // O «traço»: um morro de altura nas amostras perto do centro, e só elas
    // vão como sujas.
    let mut sujas = Vec::new();
    for (f, c) in faces.iter().enumerate() {
        let p = |k: usize| m.positions()[c[k] as usize];
        let l = plano.lado_da_face(f);
        let mut pontos = Vec::new();
        plano.para_cada_amostra_quad(f, c, |i, ij| pontos.push((i, ij)));
        for (i, ij) in pontos {
            let q = ph2d_mesh_colors::amostragem::posicao_quad([p(0), p(1), p(2), p(3)], l, ij);
            let d2 = q[0] * q[0] + q[1] * q[1];
            if d2 < 0.36 {
                plano.relevo_mut()[i as usize][0] = 0.2 * (1.0 - d2 / 0.36).powi(2);
                sujas.push(i);
            }
        }
    }
    assert!(
        r.upload_tinta_amostras_at(&queue, 0, &m, &plano, &mut sujas),
        "o slot armado com relevo tinha de aceitar o incremental"
    );
    let por_pedacos = render_using_rig_shade(
        &device,
        &queue,
        &mut r,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );

    let mut zero = MeshRenderer::new(&device, FORMAT);
    zero.upload_at(&device, &queue, 0, &m, &[]);
    zero.upload_tinta_at(&device, &queue, 0, &m, Some(&plano));
    let do_zero = render_using_rig_shade(
        &device,
        &queue,
        &mut zero,
        &camera,
        &LightRig::default(),
        rig_shade(),
    );

    let pior = |a: &[u8], b: &[u8]| {
        (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .map(|(x, y)| (lum(a, x, y) - lum(b, x, y)).abs())
            .fold(0.0f32, f32::max)
    };
    assert!(
        pior(&liso, &do_zero) > 10.0,
        "o CONTROLO: o morro tinha de acender"
    );
    let d = pior(&por_pedacos, &do_zero);
    assert!(
        d <= 1.5,
        "o incremental desenha {d} longe da subida do zero"
    );
}
