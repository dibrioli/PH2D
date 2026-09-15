//! ⭐⭐⭐ **A MALHA DESENHADA NUM SUB-RECTÂNGULO DO ALVO** — o substrato dos
//! quatro viewports da escultura (ordem do Enio, 2026-09-08).
//!
//! ⚠️ **Gates de DEVICE, e por isso `#[ignore]`**: eles medem pixels que só uma
//! GPU produz. *Um skip gracioso não é verde* — sem adaptador eles dizem-no e
//! saem.
//!
//! ```text
//! cargo test -p ph2d-mesh-render --test gpu_viewport -- --ignored --nocapture
//! ```

use ph2d_mesh::Mesh;
use ph2d_mesh::shapes::uv_sphere;
use ph2d_mesh_render::{Camera3d, MeshRenderer, ScreenRect, Shade};

/// ⚠️ **O alvo é DEITADO (2:1) de propósito**: é a única forma de a régua do
/// aspecto ter o que separar — num alvo quadrado, usar o aspecto do alvo e o da
/// vista dá a mesma imagem, e o gate ficaria verde sobre o defeito.
const W: u32 = 256;
const H: u32 = 128;
/// ⚠️ **`Rgba8Unorm` e não o `Rgba16Float` do produto**, a mesma escolha (e a
/// mesma razão) do `gpu_render` ao lado: o que se afirma é APARÊNCIA, e ler
/// `f16` de volta poria uma conversão entre a medição e o olho.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-mesh viewport test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

fn target(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("alvo"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    tex: &wgpu::Texture,
) -> Vec<u8> {
    let bpr = (W * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(bpr * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let mapped = slice.get_mapped_range();
    let row = (W * 4) as usize;
    let mut out = Vec::with_capacity(row * H as usize);
    for r in 0..H {
        let s = (r * bpr) as usize;
        out.extend_from_slice(&mapped[s..s + row]);
    }
    drop(mapped);
    buffer.unmap();
    out
}

/// Um pixel tem tinta? — o alvo nasce a zero e a peça é clara.
fn lit(px: &[u8], x: u32, y: u32) -> bool {
    let i = ((y * W + x) * 4) as usize;
    u16::from(px[i]) + u16::from(px[i + 1]) + u16::from(px[i + 2]) > 8
}

/// A caixa dos pixels com tinta, e quantos são.
fn bbox(px: &[u8]) -> (u32, u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1, mut n) = (W, H, 0u32, 0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            if lit(px, x, y) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
                n += 1;
            }
        }
    }
    (x0, y0, x1, y1, n)
}

fn esfera() -> Mesh {
    uv_sphere(24, 36, 1.0)
}

fn camera(mesh: &Mesh, aspect: f32) -> Camera3d {
    let mut c = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        ..Camera3d::default()
    };
    c.frame(mesh.bounds(), aspect);
    c
}

/// Desenha a malha na `area` do alvo e devolve os pixels do alvo inteiro.
fn desenha(area: Option<ScreenRect>) -> Option<Vec<u8>> {
    let (device, queue) = device()?;
    let mesh = esfera();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let shade = Shade {
        ssao: 0.0,
        ..Shade::default()
    };
    match area {
        None => {
            let cam = camera(&mesh, W as f32 / H as f32);
            r.render(&device, &queue, &mut enc, &view, &cam, None, shade, (W, H));
        }
        Some(a) => {
            let cam = camera(&mesh, a.aspect());
            r.render_in(
                &device,
                &queue,
                &mut enc,
                &view,
                &cam,
                None,
                shade,
                (W, H),
                a,
            );
        }
    }
    Some(readback(&device, &queue, enc, &tex))
}

/// ⭐⭐⭐ **O CAMINHO DE OMISSÃO NÃO SE MEXEU** — `render_in` no alvo inteiro é
/// **byte a byte** o que o `render` sempre desenhou.
///
/// ⚠️ **É o gate mais importante desta wave.** Tudo o que shipa hoje passa pelo
/// `render`, e o `render` passou a delegar: se a delegação tivesse mudado uma
/// linha da conta — o aspecto, o uniform de viewport, a ordem — o produto
/// inteiro teria mudado de aparência sem uma única linha do chamador se mexer.
#[test]
#[ignore = "precisa de GPU"]
fn desenhar_no_alvo_inteiro_e_byte_identico_ao_de_sempre() {
    let (Some(sempre), Some(inteiro)) = (desenha(None), desenha(Some(ScreenRect::full((W, H)))))
    else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let difs = sempre.iter().zip(&inteiro).filter(|(a, b)| a != b).count();
    println!("bytes diferentes: {difs} de {}", sempre.len());
    assert_eq!(
        difs, 0,
        "`render_in` com a area INTEIRA divergiu do `render` em {difs} bytes -- \
         o caminho de omissao do produto mudou de aparencia"
    );
}

/// ⭐⭐⭐ **UMA VISTA NÃO ESCREVE NA VIZINHA.**
///
/// ⚠️ **É o `set_scissor_rect` que isto mede, e não o `set_viewport`.** O
/// viewport **transforma** e não corta: com ele sozinho, a geometria que sai
/// pelo lado do frustum continuaria a rasterizar sobre o quadrante vizinho. Uma
/// esfera enquadrada folgadamente talvez não o mostrasse — por isso a câmera
/// aqui é aproximada de propósito, até a peça transbordar.
#[test]
#[ignore = "precisa de GPU"]
fn uma_vista_nao_escreve_na_vizinha() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mesh = esfera();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let esquerda = ScreenRect {
        x: 0,
        y: 0,
        w: W / 2,
        h: H,
    };
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let mut cam = camera(&mesh, esquerda.aspect());
    // ⚠️ **Perto o bastante para a peça TRANSBORDAR a vista** — sem isto o gate
    // mede uma esfera que já cabia, e o scissor fica sem nada a cortar.
    cam.distance *= 0.45;
    r.render_in(
        &device,
        &queue,
        &mut enc,
        &view,
        &cam,
        None,
        Shade {
            ssao: 0.0,
            ..Shade::default()
        },
        (W, H),
        esquerda,
    );
    let px = readback(&device, &queue, enc, &tex);
    let (x0, _, x1, _, n) = bbox(&px);
    println!(
        "tinta na coluna [{x0}, {x1}] de [0, {}] -- {n} pixels",
        W - 1
    );
    assert!(n > 0, "a vista da esquerda nao desenhou nada");
    assert!(
        x1 < W / 2,
        "a vista [0, {}) pintou ate' a coluna {x1} -- ela transbordou para a vizinha, \
         e e' o `set_scissor_rect` que falta (o viewport transforma, nao corta)",
        W / 2
    );
}

/// ⭐⭐⭐ **O ASPECTO É O DA VISTA, NÃO O DO ALVO.**
///
/// ⚠️ **A metade que uma implementação apressada esquece**, e ela é invisível
/// numa vista só: com quatro quadrantes o alvo continua deitado e cada vista
/// fica quase quadrada, então usar o aspecto do alvo esticaria a peça
/// horizontalmente **nas quatro, por igual** — que é como um erro de escala
/// deixa de se parecer com um erro.
///
/// A régua é a **silhueta**: uma esfera enquadrada tem de sair tão larga quanto
/// alta. Com o aspecto do alvo (`2:1`) numa vista quadrada ela sairia **metade**
/// da largura.
#[test]
#[ignore = "precisa de GPU"]
fn o_aspecto_e_o_da_vista_e_nao_o_do_alvo() {
    let quadrada = ScreenRect {
        x: 0,
        y: 0,
        w: H,
        h: H,
    };
    let Some(px) = desenha(Some(quadrada)) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let (x0, y0, x1, y1, n) = bbox(&px);
    assert!(n > 0, "a vista quadrada nao desenhou nada");
    let (largura, altura) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
    let razao = largura / altura;
    println!("silhueta {largura} x {altura} -> razao {razao:.4} (esfera => ~1)");
    assert!(
        (razao - 1.0).abs() < 0.08,
        "a esfera saiu {largura} x {altura} (razao {razao:.4}) numa vista QUADRADA -- \
         se a razao for ~0,5 o aspecto usado foi o do ALVO ({W}x{H}) em vez do da vista"
    );
}

/// ⭐⭐ **DUAS VISTAS NO MESMO QUADRO, CADA UMA NA SUA METADE.**
///
/// ⚠️ **A cor é `LoadOp::Load` e a profundidade é `Clear`**, e este gate é o que
/// afirma que essa assimetria está certa: a segunda vista limpa a profundidade
/// do alvo INTEIRO e mesmo assim a primeira continua pintada. *Se a cor também
/// limpasse, a última vista apagaria as outras três e o defeito só apareceria
/// com a divisão aberta.*
#[test]
#[ignore = "precisa de GPU"]
fn duas_vistas_no_mesmo_quadro_sobrevivem_uma_a_outra() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mesh = esfera();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    // ⚠️ **Pela PORTA das N vistas**, e não com dois `render_in` no mesmo
    // encoder: esse caminho desenha as duas com a câmera da última (ver o irmão
    // `duas_vistas_no_mesmo_quadro_mostram_duas_cameras`). Aqui as duas câmeras
    // são iguais de propósito — o que este gate mede é a assimetria
    // `LoadOp::Load` na cor contra `Clear` na profundidade.
    let vistas: Vec<_> = [0, W / 2]
        .into_iter()
        .map(|x| {
            let a = ScreenRect {
                x,
                y: 0,
                w: W / 2,
                h: H,
            };
            (a, camera(&mesh, a.aspect()))
        })
        .collect();
    r.render_views(
        &device,
        &queue,
        &view,
        &vistas,
        None,
        Shade {
            ssao: 0.0,
            ..Shade::default()
        },
        (W, H),
        None,
    );
    let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let px = readback(&device, &queue, enc, &tex);
    let esq = (0..H)
        .flat_map(|y| (0..W / 2).map(move |x| (x, y)))
        .filter(|(x, y)| lit(&px, *x, *y))
        .count();
    let dir = (0..H)
        .flat_map(|y| (W / 2..W).map(move |x| (x, y)))
        .filter(|(x, y)| lit(&px, *x, *y))
        .count();
    println!("esquerda {esq} px | direita {dir} px");
    assert!(
        esq > 0,
        "a PRIMEIRA vista desapareceu -- a segunda apagou-a (a cor tem de ser LoadOp::Load)"
    );
    assert!(dir > 0, "a SEGUNDA vista nao desenhou nada");
    // As duas vêem a mesma peça pelo mesmo ângulo: têm de cobrir o mesmo tanto.
    let d = (esq as f32 - dir as f32).abs() / esq.max(dir) as f32;
    assert!(
        d < 0.02,
        "as duas vistas identicas cobrem {esq} e {dir} pixels ({:.1}% de diferenca)",
        d * 100.0
    );
}

/// ⭐⭐⭐ **DUAS VISTAS NO MESMO QUADRO MOSTRAM DUAS CÂMERAS DIFERENTES.**
///
/// ⛔⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«com 4 views o mesh não está
/// correspondendo às views e ao tentar esculpir nas outras views o pincel tem
/// drift ou offset (esculpe no lugar errado)»* — **dois sintomas, uma causa.**
///
/// O renderizador tem **UM** buffer de uniform de câmera, e o
/// [`MeshRenderer::render_in`] escreve-o com `queue.write_buffer` antes de
/// gravar o passe. Mas `write_buffer` não é gravado no encoder: ele é agendado
/// na **fila**, e todas as escritas de antes de um `submit` acontecem **antes**
/// de qualquer comando desse submit correr. ⇒ com as quatro vistas num encoder
/// só, **as quatro desenham com a câmera da ÚLTIMA**.
///
/// E é isso que produz os dois sintomas de uma vez: a imagem de um quadrante é
/// a da última câmera, e o **pick** daquele quadrante usa a câmera **dele** —
/// logo o pincel cai onde a peça *estaria* e não onde ela *está desenhada*.
///
/// ⚠️⚠️ **O gate irmão `duas_vistas_no_mesmo_quadro_sobrevivem_uma_a_outra` não
/// o viu, e a razão é a de sempre: ele desenha as duas metades com a MESMA
/// câmera** (só o aspecto difere). *Uma fixtura em que as duas metades são
/// iguais não pode notar que uma delas ficou com a outra.*
#[test]
#[ignore = "precisa de GPU"]
fn duas_vistas_no_mesmo_quadro_mostram_duas_cameras() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    // ⚠️ **Uma peça ASSIMÉTRICA**: uma esfera dá a mesma silhueta de todo lado, e
    // o gate não teria como separar «a vista certa» de «a vista da vizinha».
    let mut mesh = esfera();
    for p in mesh.positions_mut() {
        p[0] *= 2.6;
    }
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let shade = Shade {
        ssao: 0.0,
        ..Shade::default()
    };
    // De FRENTE à esquerda (a peça é larga) e de LADO à direita (a peça é
    // estreita) — a mesma peça, duas câmeras, pela PORTA das N vistas.
    let vistas: Vec<_> = [(0, 0.0f32), (W / 2, std::f32::consts::FRAC_PI_2)]
        .into_iter()
        .map(|(x, yaw)| {
            let a = ScreenRect {
                x,
                y: 0,
                w: W / 2,
                h: H,
            };
            let mut cam = Camera3d {
                yaw,
                pitch: 0.0,
                ..Camera3d::default()
            };
            cam.frame(mesh.bounds(), a.aspect());
            (a, cam)
        })
        .collect();
    r.render_views(&device, &queue, &view, &vistas, None, shade, (W, H), None);
    let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let px = readback(&device, &queue, enc, &tex);
    let largura = |x0: u32, x1: u32| -> u32 {
        let (mut a, mut b) = (W, 0u32);
        for y in 0..H {
            for x in x0..x1 {
                if lit(&px, x, y) {
                    a = a.min(x);
                    b = b.max(x);
                }
            }
        }
        if b >= a { b - a + 1 } else { 0 }
    };
    let (esq, dir) = (largura(0, W / 2), largura(W / 2, W));
    println!("silhueta: de frente {esq} px | de lado {dir} px");
    assert!(esq > 0 && dir > 0, "uma das vistas nao desenhou nada");
    // Vista de frente a peça é `2,6×` mais larga que funda; de lado é o inverso.
    // Enquadradas, as duas ocupam a vista — o que muda é a ALTURA relativa, e o
    // discriminador honesto é a razão largura/altura da silhueta.
    let altura = |x0: u32, x1: u32| -> u32 {
        let (mut a, mut b) = (H, 0u32);
        for y in 0..H {
            for x in x0..x1 {
                if lit(&px, x, y) {
                    a = a.min(y);
                    b = b.max(y);
                }
            }
        }
        if b >= a { b - a + 1 } else { 0 }
    };
    let r_esq = esq as f32 / altura(0, W / 2).max(1) as f32;
    let r_dir = dir as f32 / altura(W / 2, W).max(1) as f32;
    println!("razao largura/altura: de frente {r_esq:.3} | de lado {r_dir:.3}");
    assert!(
        (r_esq - r_dir).abs() > 0.15,
        "as duas vistas desenharam a MESMA imagem (razao {r_esq:.3} contra {r_dir:.3}) -- o \
         uniform da camera e' UM so' e as escritas na fila acontecem todas antes do submit, \
         entao as duas passagens leem a camera da ULTIMA"
    );
}

/// ⛔⛔⛔ **UMA ÁREA MAIOR QUE O ALVO NÃO PODE ENTRAR EM PÂNICO** — o report do
/// dono (2026-09-14), ao **desacoplar a janela maximizada**:
///
/// ```text
/// In a CommandEncoder, label = 'ph2d-mesh view'
///   In a set_scissor_rect command
///     Scissor Rect { x: 244, y: 96, w: 1399, h: 926 }
///     is not contained in the render target (1024, 768, 1)
/// ```
///
/// ⚠️⚠️ **Não é um acidente de um quadro: é ESTRUTURAL.** O `size` chega do
/// quadro de AGORA (a superfície acabou de ser reconfigurada) e a `ScreenRect` é
/// a área que o painel **publicou no quadro ANTERIOR** — assim de propósito,
/// para que o desenho e o *pick* derivem do mesmo rectângulo. ⇒ **todo
/// redimensionamento tem um quadro em que as duas discordam.**
///
/// ⚠️ **Os NÚMEROS são os do report**, reduzidos à escala deste alvo pela mesma
/// razão: uma fixtura que não contém o fenómeno mede vácuo.
///
/// ⭐ **As duas metades:** ele não estoura **e** continua a desenhar (recortado).
/// *Sem a segunda, devolver cedo em toda a chamada passaria.*
#[test]
#[ignore = "precisa de GPU"]
fn uma_area_maior_que_o_alvo_recorta_em_vez_de_estourar() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mesh = esfera();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    // A forma do report: uma área da janela ANTIGA, sobre o alvo de AGORA.
    let velha = ScreenRect {
        x: W / 4,
        y: H / 8,
        w: W + W / 3,
        h: H + H / 5,
    };
    assert!(
        velha.x + velha.w > W && velha.y + velha.h > H,
        "a fixtura não transborda o alvo — ela não contém o fenómeno"
    );
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let cam = camera(&mesh, velha.aspect());
    r.render_in(
        &device,
        &queue,
        &mut enc,
        &view,
        &cam,
        None,
        Shade {
            ssao: 0.0,
            ..Shade::default()
        },
        (W, H),
        velha,
    );
    // ⭐ O `readback` submete e espera: se o `wgpu` recusasse o comando, é aqui
    // que este teste morreria — que é exactamente o que o dono viu.
    let px = readback(&device, &queue, enc, &tex);
    let (_, _, _, _, n) = bbox(&px);
    assert!(
        n > 0,
        "a área transbordante foi recortada até não sobrar nada — o recorte tem \
         de devolver a INTERSECÇÃO, não desistir"
    );
}

/// **A INTERSECÇÃO, sem GPU** — a aritmética de [`ScreenRect::clip_to`].
///
/// ⚠️ **A linha que mais importa é a do REGIME NORMAL:** uma área que já cabe
/// tem de voltar **ela mesma**, senão esta cura mudava toda a imagem que já
/// shipava.
#[test]
fn o_recorte_devolve_a_interseccao_e_nao_toca_em_quem_ja_cabe() {
    let alvo = (1024u32, 768u32);
    // O caso do report.
    assert_eq!(
        ScreenRect {
            x: 244,
            y: 96,
            w: 1399,
            h: 926
        }
        .clip_to(alvo),
        Some(ScreenRect {
            x: 244,
            y: 96,
            w: 780,
            h: 672
        })
    );
    // ⭐ O REGIME NORMAL: quem cabe volta ao bit.
    let dentro = ScreenRect {
        x: 10,
        y: 20,
        w: 300,
        h: 400,
    };
    assert_eq!(dentro.clip_to(alvo), Some(dentro));
    assert_eq!(
        ScreenRect::full(alvo).clip_to(alvo),
        Some(ScreenRect::full(alvo))
    );
    // Fora por inteiro ⇒ não há vista.
    assert_eq!(
        ScreenRect {
            x: 2000,
            y: 20,
            w: 100,
            h: 100
        }
        .clip_to(alvo),
        None
    );
}

/// ⛔⛔⛔ **UMA VISTA INTEIRAMENTE FORA DO ALVO NÃO DESENHA NADA — e o que ela
/// NÃO pode fazer é pintar a JANELA TODA.**
///
/// ⚠️⚠️ **Este gate existe por uma mutação que SOBREVIVEU**, e o que ela expôs é
/// que os dois recortes desta crate não são redundantes: o
/// [`MeshRenderer::render_in`] recorta **à entrada** e o `set_area` recorta
/// **dentro do passe**, e matar só o primeiro não estoura — mas abre um defeito
/// **pior que o `panic`**.
///
/// ⭐ **O mecanismo:** com a área inteiramente fora, o `set_area` devolve cedo e
/// **não chega a chamar `set_scissor_rect`** ⇒ o passe fica com o scissor por
/// omissão, que é **o alvo inteiro**, e a malha é pintada por cima da janela
/// toda. *Um recorte que desiste em silêncio não recorta nada.*
///
/// ⇒ quem decide é a ENTRADA: sem vista, o passe não se abre.
#[test]
#[ignore = "precisa de GPU"]
fn uma_vista_inteiramente_fora_do_alvo_nao_pinta_a_janela_toda() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mesh = esfera();
    let mut r = MeshRenderer::new(&device, FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);
    let tex = target(&device);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let fora = ScreenRect {
        x: W * 2,
        y: 0,
        w: W,
        h: H,
    };
    assert!(fora.x >= W, "a fixtura não está fora do alvo");
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let cam = camera(&mesh, fora.aspect());
    r.render_in(
        &device,
        &queue,
        &mut enc,
        &view,
        &cam,
        None,
        Shade {
            ssao: 0.0,
            ..Shade::default()
        },
        (W, H),
        fora,
    );
    let px = readback(&device, &queue, enc, &tex);
    let (_, _, _, _, n) = bbox(&px);
    assert_eq!(
        n, 0,
        "uma vista fora do alvo deixou {n} pixels de tinta — sem scissor o \
         passe pinta o alvo INTEIRO, que é pior que o `panic` que esta wave \
         curou"
    );
}
