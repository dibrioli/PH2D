//! ⭐⭐⭐ **§5.0 DO CATAVENTO — o que a composição JÁ DÁ, medido antes da 1.ª linha de produto.**
//!
//! A 2.ª obra do plano de superar (`docs/Render3d/15_as_metas.md` §5) é *«um objecto 3D ao vivo
//! dentro do canvas 2D: roda, e a luz acompanha»* — a **rota B** do
//! `docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`, onde a malha filha rasteriza para o
//! G-buffer **por quadro**, em vez de ser assada uma vez.
//!
//! ⚠️ **A lei desta casa manda medir se a composição já o exprime** (`CLAUDE.md` §5.0), e aquele
//! doc deixa **três** perguntas por responder *«com medição, não com opinião»*:
//!
//! 1. **Qual é o preço, por quadro, de rasterizar uma malha filha para o G-buffer?** ⇒ **Bloco A**.
//! 2. **Quantos objectos em rota B cabem no orçamento?** ⇒ **Bloco A**, a dividir `16,67 ms`.
//! 3. **A resolução do G-buffer é a do rectângulo do sprite, ou uma fracção dele?** ⇒ **Bloco C**.
//!
//! ⛔⛔ **E o Bloco B é o CONTROLO que decide se há wave:** a porta que hoje existe
//! ([`MeshRenderer::form_plane`]) rasteriza **e LÊ DE VOLTA** para a CPU (`Vec<f32>`), porque quem
//! a chama é o assado. *Se o preço da rasterização e o do readback fossem da mesma ordem, a rota B
//! seria a rota A chamada mais vezes e não haveria obra nenhuma.*
//!
//! ## ⭐⭐⭐ O que ela MEDIU (2026-09-21, `load 44` ⇒ os relógios são um PISO)
//!
//! *A tabela inteira, com o mecanismo e as recusas, vive em
//! `docs/Render3d/17_a_rota_b_o_catavento.md` §1.*
//!
//! - **O custo da rota B é dos VÉRTICES e não dos pixels:** `8×` de lado (`64×` de área) não move o
//!   relógio (`0,131` → `0,124 ms`); `33×` de vértices move-o `1,6×`. ⇒ **`126` objectos por quadro**
//!   na peça de fábrica, e **uma fracção da resolução não compra relógio nenhum**.
//! - **O readback é o preço inteiro da porta de hoje:** ela escala com a ÁREA (`~4×` por duplicação
//!   do lado) e vale **`31×`** a rasterização a `512²`. ⇒ chamá-la por quadro dá **`4`** objectos
//!   contra `126` — *a obra existe, e é abrir uma costura RESIDENTE entre duas leis que já vivem na
//!   placa.*
//! - **A resolução é uma conta de VRAM e não de relógio:** `1/2` erra `0,71 %` da silhueta e poupa
//!   `4×` de memória (`2,5` contra `10,0 MiB` por objecto).
//!
//! ## ⛔⛔⛔ Três defeitos da 1.ª redacção desta sonda, e cada um tem nome nesta casa
//!
//! **(1) Os três blocos correram em PARALELO sobre a MESMA placa** e as tabelas saíram
//! entrelaçadas: o Bloco A leu `0,243 ms` a `128²` e `0,053` a `1024²` — *o lado grande mais
//! barato que o pequeno*, que é a assinatura de estar a medir CONTENÇÃO e não trabalho. ⇒ esta
//! sonda corre **`--test-threads=1`**, e diz isso no comando.
//!
//! **(2) A malha tinha `3 010` vértices** e a peça com que o módulo abre tem **`98 306`**
//! ([`scenes::mesh::peca_de_fabrica`], `sculpt_sphere(1.0)`) — *uma fixtura `33×` mais leve que o
//! produto mede o overhead de submissão e chama-lhe o preço do objecto*. ⇒ a escada tem as duas.
//!
//! **(3) E a régua do Bloco C não podia ver o fenómeno:** ela contava texels com cobertura
//! PARCIAL, e **o pipeline desta crate é `sample_count: 1`** — sem MSAA a cobertura é BINÁRIA,
//! logo ela leu `0` nas quatro fracções, sobre um produto correcto. ⇒ a régua passou a ser o
//! **DESACORDO DE SILHUETA**: rasterizar no rectângulo cheio, rasterizar na fracção, ampliar por
//! vizinho e contar os pixels do ecrã que ficam do lado errado. *É o que o olho vê, e é a grandeza
//! que a fracção de facto degrada.*
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test it -- --ignored --nocapture \
//!     --test-threads=1 catavento
//! ```
//!
//! ⚠️ **`--release` e a máquina CALMA** (`CLAUDE.md` §5.0: nenhuma leitura de relógio desta
//! workstation vale acima de `load ~5`) — a sonda imprime o `loadavg` ao lado de cada tabela, que é
//! a lei que esta casa paga sempre que a esquece.

use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_render::{Camera3d, MeshRenderer};

use super::device_de_teste::device;

/// O `Shade` do caminho do RIG — o mesmo que o `gpu_render` usa, e pela mesma
/// razão: um `Shade::default()` traz o matcap ARMADO, e o fragment escolhe esse
/// caminho antes de tudo o resto.
fn rig_shade() -> ph2d_mesh_render::Shade {
    ph2d_mesh_render::Shade {
        lighting: ph2d_mesh_render::Lighting::Rig,
        ..ph2d_mesh_render::Shade::default()
    }
}

fn camera_for(mesh: &Mesh) -> Camera3d {
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.3,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), 1.0);
    cam
}

/// ⚠️ **A carga ao lado de cada tabela** — um relógio desta workstation lido acima de `load ~5`
/// não vale nada, e esta casa já pagou isso meia dúzia de vezes.
fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

/// As duas malhas da escada: a leve (que é o que a 1.ª redacção media sem o
/// saber) e **a peça com que o módulo ABRE**.
fn escada_de_malhas() -> [(&'static str, Mesh); 2] {
    [
        ("leve", shapes::uv_sphere(48, 64, 1.0)),
        ("fábrica", shapes::sculpt_sphere(1.0)),
    ]
}

/// Os dois alvos do G-buffer, no tamanho pedido.
fn alvos(device: &wgpu::Device, w: u32, h: u32) -> (wgpu::TextureView, wgpu::TextureView) {
    let mk = |label, format| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    (
        mk("catavento normal", MeshRenderer::GBUFFER_FORMAT),
        mk("catavento occ", MeshRenderer::OCCLUSION_FORMAT),
    )
}

/// Um quadro: rasteriza para os dois alvos e ESPERA o device (senão o relógio
/// mede o enfileiramento e não o trabalho).
fn um_quadro(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &mut MeshRenderer,
    cam: &Camera3d,
    n: &wgpu::TextureView,
    o: &wgpu::TextureView,
    size: (u32, u32),
) {
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    r.render_gbuffer(device, queue, &mut enc, n, o, cam, rig_shade(), size);
    queue.submit([enc.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// ⭐⭐⭐ **BLOCO A — o preço POR QUADRO da rota B, e quantos objectos cabem.**
///
/// ⚠️ **Duas colunas de propósito:** *reaproveitada* é o que um dirty-flag entrega (a textura vive
/// com o objecto e só o conteúdo é reescrito); *alocada* é o que a porta de assar faz hoje, que
/// cria as duas texturas a cada chamada. A diferença entre elas é o preço de NÃO ter o slot.
#[test]
#[ignore = "precisa de adapter"]
fn catavento_bloco_a_o_preco_por_quadro_de_uma_malha_filha() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    eprintln!(
        "\n=== BLOCO A — rota B: rasterizar a malha filha para o G-buffer (load {}) ===",
        carga()
    );
    for (nome, mesh) in escada_de_malhas() {
        let cam = camera_for(&mesh);
        let mut r = MeshRenderer::new(&device, MeshRenderer::GBUFFER_FORMAT);
        r.upload_at(&device, &queue, 0, &mesh, &[]);
        eprintln!("\n-- malha {nome}: {} vértices --", mesh.positions().len());
        eprintln!(
            "{:>6} | {:>14} | {:>14} | {:>12}",
            "lado", "reaproveitada", "alocada/quadro", "objs/quadro"
        );
        for lado in [128u32, 256, 512, 1024] {
            let size = (lado, lado);
            let (n, o) = alvos(&device, lado, lado);
            // aquecimento: a 1.ª chamada constrói o pipeline.
            um_quadro(&device, &queue, &mut r, &cam, &n, &o, size);

            let reaproveitada = mediana(
                (0..9)
                    .map(|_| {
                        let t = std::time::Instant::now();
                        um_quadro(&device, &queue, &mut r, &cam, &n, &o, size);
                        t.elapsed().as_secs_f64() * 1e3
                    })
                    .collect(),
            );
            let alocada = mediana(
                (0..9)
                    .map(|_| {
                        let t = std::time::Instant::now();
                        let (n, o) = alvos(&device, lado, lado);
                        um_quadro(&device, &queue, &mut r, &cam, &n, &o, size);
                        t.elapsed().as_secs_f64() * 1e3
                    })
                    .collect(),
            );
            eprintln!(
                "{lado:>6} | {reaproveitada:>11.3} ms | {alocada:>11.3} ms | {:>12.0}",
                16.67 / reaproveitada
            );
        }
    }
}

/// ⛔⛔ **BLOCO B — o CONTROLO: o que a porta de HOJE custa, com o readback dentro.**
///
/// Ela é a rota A (o assado), e a pergunta que este bloco responde é *«a rota B é a rota A chamada
/// mais vezes?»*. ⚠️ Se a razão contra o Bloco A for perto de `1`, não há obra; se for grande, o
/// readback é o preço e a wave existe para o evitar.
#[test]
#[ignore = "precisa de adapter"]
fn catavento_bloco_b_o_que_a_porta_de_assar_custa_hoje() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    eprintln!(
        "\n=== BLOCO B — rota A: a porta `form_plane`, COM readback (load {}) ===",
        carga()
    );
    for (nome, mesh) in escada_de_malhas() {
        let cam = camera_for(&mesh);
        let mut r = MeshRenderer::new(&device, MeshRenderer::GBUFFER_FORMAT);
        r.upload_at(&device, &queue, 0, &mesh, &[]);
        eprintln!("\n-- malha {nome}: {} vértices --", mesh.positions().len());
        eprintln!("{:>6} | {:>14}", "lado", "form_plane");
        for lado in [128u32, 256, 512, 1024] {
            let _ = r.form_plane(&device, &queue, &cam, (lado, lado), rig_shade(), None);
            let t = mediana(
                (0..5)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let p =
                            r.form_plane(&device, &queue, &cam, (lado, lado), rig_shade(), None);
                        let d = t0.elapsed().as_secs_f64() * 1e3;
                        assert!(p.is_some(), "a porta devolveu None a {lado}");
                        d
                    })
                    .collect(),
            );
            eprintln!("{lado:>6} | {t:>11.3} ms");
        }
    }
}

/// ⭐⭐ **BLOCO C — a resolução: o rectângulo do sprite, ou uma fracção dele?**
///
/// ⛔⛔ **A régua é o DESACORDO DE SILHUETA, e não a cobertura parcial:** o pipeline desta crate é
/// `sample_count: 1`, logo **a cobertura é BINÁRIA** e uma régua que procure texels a meio caminho
/// lê `0` sobre um produto correcto — foi o que a 1.ª redacção desta sonda fez.
///
/// Aqui rasteriza-se no rectângulo CHEIO (o que o artista veria), rasteriza-se na fracção, amplia-se
/// por vizinho mais próximo até ao cheio, e contam-se os pixels do ECRÃ que ficam do lado errado.
/// ⚠️ **A coluna que se compara é a última** — os pixels errados em unidades do **PERÍMETRO** da
/// silhueta (`√área`), porque em valor absoluto ela cresce com a peça e não diz nada.
#[test]
#[ignore = "precisa de adapter"]
fn catavento_bloco_c_a_resolucao_do_gbuffer() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    // O rectângulo do sprite: 512 px de lado, que é uma sprite de personagem
    // grande num canvas normal.
    const SPRITE: usize = 512;

    let mesh = shapes::sculpt_sphere(1.0);
    let cam = camera_for(&mesh);
    let mut r = MeshRenderer::new(&device, MeshRenderer::GBUFFER_FORMAT);
    r.upload_at(&device, &queue, 0, &mesh, &[]);

    // A cobertura de uma rasterização, como máscara booleana.
    let mut cobertura = |lado: u32| -> Vec<bool> {
        let p = r
            .form_plane(&device, &queue, &cam, (lado, lado), rig_shade(), None)
            .expect("a porta devolveu None");
        p.normal.chunks_exact(4).map(|c| c[3] > 0.5).collect()
    };

    let cheia = cobertura(SPRITE as u32);
    let area = cheia.iter().filter(|c| **c).count();
    eprintln!(
        "\n=== BLOCO C — a silhueta por fracção do rectângulo do sprite (load {}) ===",
        carga()
    );
    eprintln!("rectângulo do sprite: {SPRITE}² · silhueta: {area} px");
    eprintln!(
        "{:>8} | {:>6} | {:>12} | {:>16}",
        "fracção", "lado", "px errados", "px/√área"
    );
    for (nome, div) in [("1/1", 1usize), ("1/2", 2), ("1/4", 4), ("1/8", 8)] {
        let lado = SPRITE / div;
        let pequena = cobertura(lado as u32);
        // Amplia por VIZINHO — que é o que um sprite pass faria ao amostrar um
        // G-buffer mais pequeno sem filtro; com filtro linear a fronteira ficaria
        // borrada em vez de errada, e isso é outra grandeza.
        let errados = (0..SPRITE * SPRITE)
            .filter(|i| {
                let (x, y) = (i % SPRITE, i / SPRITE);
                cheia[*i] != pequena[(y / div) * lado + (x / div)]
            })
            .count();
        eprintln!(
            "{nome:>8} | {lado:>6} | {errados:>12} | {:>16.2}",
            errados as f32 / (area as f32).max(1.0).sqrt()
        );
    }
}
