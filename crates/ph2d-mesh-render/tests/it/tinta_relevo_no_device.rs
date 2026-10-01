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
    let lado = N + 1;
    let pos: Vec<[f32; 3]> = (0..lado)
        .flat_map(|j| {
            (0..lado).map(move |i| {
                let x = -1.0 + 2.0 * i as f32 / N as f32;
                let y = -1.0 + 2.0 * j as f32 / N as f32;
                [x, y, 0.0]
            })
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
    // As duas colunas ocupam `1/4` da largura; `+2` px de folga, porque a
    // derivada de ecrã é tirada em quads de `2×2`.
    let dentro = |x: u32| ((x as f32) - meio).abs() <= largura / 8.0 + 2.0;
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
