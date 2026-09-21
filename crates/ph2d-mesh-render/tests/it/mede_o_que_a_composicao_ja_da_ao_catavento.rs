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
//! - **O custo de RASTERIZAR é dos VÉRTICES e não dos pixels:** `8×` de lado (`64×` de área) não
//!   move o relógio (`0,131` → `0,124 ms`); `33×` de vértices move-o `1,6×`. ⇒ **uma fracção da
//!   resolução não compra relógio nenhum aqui.**
//! - ⛔⛔⛔ **E esta sonda mede METADE DA CORRENTE:** o produto é *rasterizar E ACENDER*, e o
//!   acender escala com a **ÁREA** (`0,505 ms` a `512²` contra `0,133` de rasterização). ⇒ dividir
//!   o orçamento pelo Bloco A dá **`126`** objectos e a corrente inteira dá **`26`** — *uma régua
//!   que mede o primeiro elo devolve uma contagem que o app nunca vai ver*. A outra metade é o
//!   **Bloco D**, em `ph2d-form-donation/src/mede_o_acender_por_quadro.rs`.
//! - **O readback é o preço inteiro da porta de hoje:** ela escala com a ÁREA (`~4×` por duplicação
//!   do lado) e vale **`31×`** a rasterização a `512²`. ⇒ chamá-la por quadro dá **`4`** objectos
//!   contra `126` — *a obra existe, e é abrir uma costura RESIDENTE entre duas leis que já vivem na
//!   placa.*
//! - **A resolução é uma conta de VRAM e não de relógio:** `1/2` erra `0,71 %` da silhueta e poupa
//!   `4×` de memória (`2,5` contra `10,0 MiB` por objecto).
//!
//! ## ⭐⭐⭐⭐ E o BLOCO E responde a pergunta que decide o que a rota B tem de SER
//!
//! O `02.2` promete *«rodar um sprite e ver a luz acompanhar — o efeito que nenhum sprite
//! normal-mapeado comum consegue»*. ⚠️ **Essa frase é verdadeira para METADE das rotações e FALSA
//! para a outra metade**, e a §5.0 manda medir qual — com o sucedâneo 2D aplicado a `90°`, onde ele
//! é uma permutação EXACTA de pixels e não tem reamostragem nenhuma:
//!
//! | malha | rotação de `90°` | mediana | p99 | silhueta |
//! |---|---|---|---|---|
//! | com relevo | **no plano, com o sucedâneo 2D** | **`0,00°`** | `0,03°` | `0,00 %` |
//! | com relevo | no plano, com o plano FIXO | `28,58°` | `130,93°` | `6,16 %` |
//! | com relevo | **fora do plano, com o plano FIXO** | **`31,69°`** | `125,14°` | `6,71 %` |
//! | esfera lisa (CONTROLO) | todas as linhas | `0,00°` | `0,03°` | `0,00 %` |
//!
//! ⇒ **A rota A JÁ DÁ a rotação NO PLANO, e dá-a exactamente:** uma rotação de `R` leva as normais
//! a `R·n` e a imagem a `R·imagem`, e as duas são operações 2D sobre o plano já assado. ⛔ **Fora do
//! plano não há sucedâneo nenhum** — aparecem faces que não estavam na imagem, e nenhuma operação
//! 2D as inventa.
//!
//! ⛔⛔⛔ **E isto decide o COMPONENTE, não só a cena:** o [`ph2d_ecs::Transform`] tem
//! `rotation: f32` e exprime **apenas** a rotação no plano — ou seja, exactamente a que a rota A já
//! dá. *A pose 3D tem de viajar no componente da malha, porque a hierarquia 2D não sabe exprimir a
//! rotação que justifica a obra.*
//!
//! ## ⛔⛔⛔ QUATRO defeitos da 1.ª redacção desta sonda, e cada um tem nome nesta casa
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
//! **(4) E o Bloco E mediu `4,43°` onde a resposta certa é `0,00°`**, por rodar a malha em torno da
//! ORIGEM DO MUNDO enquanto a câmera aponta ao CENTRO DA CAIXA da peça: fora do eixo, uma rotação
//! em torno de `z` **não é** uma rotação em torno do eixo da vista, e o sucedâneo — que roda em
//! torno do centro da IMAGEM — não a pode reproduzir. ⚠️ *O resíduo lia-se como sendo da lei*, e
//! era `6,5×` menor que o efeito, logo não gritava. ⇒ a rotação centra-se no `cam.target`.
//! ⭐ **E ele tinha ainda DUAS incógnitas tratadas como UMA** (a linha `0` do readback ser o topo ou
//! o fundo · o `y` das normais apontar para cima ou para baixo): viradas ao mesmo tempo, as duas
//! mãos liam **o mesmo número** — *um discriminador que muda os dois lados do que compara não
//! discrimina nada* —, e o CONTROLO admite **duas** das quatro combinações (elas diferem por um
//! `180°` global, que numa esfera é a identidade), logo quem desempata é a malha COM relevo.
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
        p.normal
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| c[3] > 0.5)
            .collect()
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

/// Uma cópia da malha com as posições rodadas em torno de um eixo do MUNDO.
///
/// ⚠️ Roda-se a **MALHA** e não a câmera, e a razão é dura: o [`Camera3d`] é orbital
/// (`yaw`/`pitch`) e **não tem ROLL** — uma rotação NO PLANO do ecrã é inexprimível por ele.
/// *É a mesma ausência que obriga o componente da rota B a carregar a pose 3D dele.*
///
/// ⛔⛔ **O `rebuild()` no fim NÃO é higiene — sem ele esta sonda mede OUTRA COISA.** O doc do
/// [`Mesh::positions_mut`] escreve a dívida por extenso (*«quem escreve aqui fica devendo um
/// refresh: a normal, o octree e a caixa passam a descrever a malha de antes»*), e a 1.ª redacção
/// desta função não a pagou: as normais ficavam para trás, **presas a vértices que se mexeram**.
/// A esfera lisa de CONTROLO leu **exactamente `90,00°`** — *uma rotação de 90° medida como erro*,
/// que é a assinatura desse defeito e o motivo de o controlo existir.
fn malha_rodada(m: &Mesh, eixo: usize, ang: f32, centro: [f32; 3]) -> Mesh {
    let (s, c) = ang.sin_cos();
    let mut fora = m.clone();
    for p in fora.positions_mut() {
        // ⛔⛔ **O CENTRO é o ALVO DA CÂMERA e não a origem do mundo**, e a 1.ª redacção usou a
        // origem: uma peça cujo centro de caixa não está no zero é vista *fora do eixo*, logo uma
        // rotação em torno de `z` do MUNDO **não é** uma rotação em torno do eixo da VISTA — o
        // sucedâneo 2D (que roda em torno do centro da imagem) deixa de poder reproduzi-la, e o
        // resíduo que sobra lê-se como se fosse da lei. *Quando se compara uma rotação do objecto
        // com uma rotação da imagem, as duas têm de partilhar o mesmo centro.*
        let [x, y, z] = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
        let q = match eixo {
            0 => [x, c * y - s * z, s * y + c * z],
            1 => [c * x + s * z, y, -s * x + c * z],
            _ => [c * x - s * y, s * x + c * y, z],
        };
        *p = [q[0] + centro[0], q[1] + centro[1], q[2] + centro[2]];
    }
    fora.rebuild();
    fora
}

/// O plano de normais de uma malha, pela porta de assar (o readback não custa nada a uma régua
/// de CORRECÇÃO — só custaria a uma de relógio).
fn plano(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh: &Mesh,
    cam: &Camera3d,
    lado: u32,
) -> Vec<f32> {
    let mut r = MeshRenderer::new(device, MeshRenderer::GBUFFER_FORMAT);
    r.upload_at(device, queue, 0, mesh, &[]);
    r.form_plane(device, queue, cam, (lado, lado), rig_shade(), None)
        .expect("a porta devolveu None")
        .normal
}

/// O desacordo entre dois planos de normais: o ângulo mediano e o p99 **onde os dois cobrem**, e a
/// fracção de pixels em que a SILHUETA discorda.
fn desacordo(a: &[f32], b: &[f32]) -> (f64, f64, f64) {
    let (mut angs, mut so_um, mut algum) = (Vec::new(), 0usize, 0usize);
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let (ca, cb) = (pa[3] > 0.5, pb[3] > 0.5);
        if ca || cb {
            algum += 1;
        }
        if ca != cb {
            so_um += 1;
            continue;
        }
        if !ca {
            continue;
        }
        let n = |p: &[f32]| {
            let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-12);
            [p[0] / l, p[1] / l, p[2] / l]
        };
        let (u, v) = (n(pa), n(pb));
        let d = f64::from(u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0);
        angs.push(d.acos().to_degrees());
    }
    if angs.is_empty() {
        return (f64::NAN, f64::NAN, 1.0);
    }
    angs.sort_by(f64::total_cmp);
    let p99 = angs[(angs.len() * 99 / 100).min(angs.len() - 1)];
    (
        angs[angs.len() / 2],
        p99,
        so_um as f64 / algum.max(1) as f64,
    )
}

/// O SUCEDÂNEO 2D de uma rotação de `90°` NO PLANO do ecrã, aplicado a um plano já assado: uma
/// permutação EXACTA dos pixels mais a troca `(x, y, z) → (∓y, ±x, z)` das normais.
///
/// ⭐⭐ **`90°` é escolhido para o sucedâneo não ter REAMOSTRAGEM** — a qualquer outro ângulo ele
/// interpolaria, e o erro lido seria o do filtro e não o da lei. *Assim, o que sobra é o fenómeno.*
///
/// ⛔⛔ **O `sentido` é a ÚNICA incógnita desta função, e ela mede-se em vez de se supor:** saber se
/// a linha `0` do readback é o TOPO ou o FUNDO da imagem decide a mão da permutação, e as duas
/// leituras diferem por `180°` no plano `xy`. ⇒ o Bloco E corre **as duas** e exige que **uma
/// delas** ponha o CONTROLO a zero; *uma sonda que escolhesse a mão que dá o resultado bonito
/// racionalizava qualquer coisa, e uma que a supusesse mediria a suposição.*
fn sucedaneo_no_plano(a: &[f32], n: usize, perm: i32, norm: i32) -> Vec<f32> {
    let mut fora = vec![0f32; a.len()];
    for r in 0..n {
        for c in 0..n {
            // ⛔⛔⛔ **São DUAS incógnitas INDEPENDENTES e a 1.ª redacção tratou-as como uma.**
            // Ela virava a permutação e a rotação das normais ao mesmo tempo: as duas mãos liam
            // **o mesmo número**, porque virar os dois lados do que se compara deixa a composição
            // onde estava — *um discriminador que muda os dois lados não discrimina nada*. E são
            // mesmo duas: a linha `0` do readback pode ser o TOPO ou o FUNDO (decide a `perm`) e o
            // `y` das normais pode apontar para cima ou para baixo (decide a `norm`). ⇒ varrem-se
            // as **quatro** combinações e o CONTROLO escolhe.
            let (lr, lc) = if perm > 0 {
                (c, n - 1 - r)
            } else {
                (n - 1 - c, r)
            };
            let orig = (lr * n + lc) * 4;
            let dest = (r * n + c) * 4;
            let p = &a[orig..orig + 4];
            let (px, py) = if norm > 0 {
                (-p[1], p[0])
            } else {
                (p[1], -p[0])
            };
            fora[dest] = px;
            fora[dest + 1] = py;
            fora[dest + 2] = p[2];
            fora[dest + 3] = p[3];
        }
    }
    fora
}

/// ⭐⭐⭐ **BLOCO E — a pergunta que decide o que a rota B tem de SER: de que rotação é ela?**
///
/// A promessa do `02.2` é *«rodar um sprite e ver a luz acompanhar — o efeito que nenhum sprite
/// normal-mapeado comum consegue»*. ⚠️ **Essa frase é verdadeira para metade das rotações e FALSA
/// para a outra metade**, e a §5.0 manda medir qual antes da 1.ª linha:
///
/// - **NO PLANO do ecrã** (o que o [`ph2d_ecs::Transform`] exprime, e o ÚNICO que ele exprime — ele
///   tem `rotation: f32`): a rota A **já o dá**, porque uma rotação de `R` leva as normais a `R·n`
///   e a imagem a `R·imagem` — *duas operações 2D sobre o plano já assado*.
/// - **FORA DO PLANO** (a pá do catavento a virar): aparecem faces que não estavam na imagem. **Não
///   há operação 2D nenhuma que as invente**, e é aqui que a rota B é a única resposta.
///
/// ⛔⛔ **E o CONTROLO é uma ESFERA LISA**, que não tem orientação nenhuma: o campo de normais dela
/// visto de uma câmera **não depende** de como ela está rodada. *Uma fixtura sem o fenómeno lê zero
/// nas duas colunas, e é isso que prova que a régua vê o fenómeno e não o ruído.*
#[test]
#[ignore = "precisa de adapter"]
fn catavento_bloco_e_de_que_rotacao_e_a_rota_b() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    const LADO: u32 = 512;
    let reta = core::f32::consts::FRAC_PI_2;

    eprintln!(
        "\n=== BLOCO E — a rota A já dá a rotação NO PLANO? (load {}) ===",
        carga()
    );
    eprintln!(
        "{:>22} | {:>22} | {:>9} | {:>9} | {:>12}",
        "malha", "rotação de 90°", "mediana", "p99", "silhueta"
    );

    for (nome, mesh) in [
        // ⚠️ A que TEM o fenómeno: bossas ⇒ ela tem orientação.
        ("com relevo", shapes::uv_sphere_noisy(48, 64, 1.0, 0.18)),
        // ⛔ O CONTROLO: uma esfera não tem orientação — tem de ler ~0 nas duas linhas.
        ("esfera lisa (controlo)", shapes::uv_sphere(48, 64, 1.0)),
    ] {
        // ⚠️ `yaw = pitch = 0` ⇒ o olho fica em `+z` e a vista é o eixo `z`: NO PLANO é rodar em
        // torno de `z`, FORA DO PLANO é rodar em torno de `y`. Ver o [`Camera3d::eye`].
        let mut cam = Camera3d {
            yaw: 0.0,
            pitch: 0.0,
            fov_y: core::f32::consts::FRAC_PI_4,
            ..Camera3d::default()
        };
        cam.frame(mesh.bounds(), 1.0);

        let repouso = plano(&device, &queue, &mesh, &cam, LADO);

        // (1) NO PLANO: a verdade contra o sucedâneo 2D sobre o plano já assado, nas DUAS mãos.
        let verdade_z = plano(
            &device,
            &queue,
            &malha_rodada(&mesh, 2, reta, cam.target.into()),
            &cam,
            LADO,
        );
        for (perm, norm) in [(1i32, 1i32), (1, -1), (-1, 1), (-1, -1)] {
            let (m, p, s) = desacordo(
                &verdade_z,
                &sucedaneo_no_plano(&repouso, LADO as usize, perm, norm),
            );
            let sinal = |v: i32| if v > 0 { '+' } else { '−' };
            let rotulo = format!("no plano perm{} norm{}", sinal(perm), sinal(norm));
            eprintln!(
                "{nome:>22} | {rotulo:>22} | {m:>8.2}° | {p:>8.2}° | {:>11.2}%",
                s * 100.0
            );
        }

        // ⛔⛔ **O CONTROLO que dá sentido ao número de cima:** a MESMA rotação no plano, medida
        // contra um plano FIXO (nenhuma operação 2D). Sem esta linha, o `4°` do sucedâneo não tem
        // contra o que ser pequeno — *um resíduo só é pequeno ao lado do que ele evitou*.
        let (m, p, sil) = desacordo(&verdade_z, &repouso);
        eprintln!(
            "{nome:>22} | {:>22} | {m:>8.2}° | {p:>8.2}° | {:>11.2}%",
            "no plano vs FIXO",
            sil * 100.0
        );

        // (2) FORA DO PLANO: a verdade contra o que um plano FIXO dá — que é não mudar nada.
        let verdade_y = plano(
            &device,
            &queue,
            &malha_rodada(&mesh, 1, reta, cam.target.into()),
            &cam,
            LADO,
        );
        let (m, p, s) = desacordo(&verdade_y, &repouso);
        eprintln!(
            "{nome:>22} | {:>22} | {m:>8.2}° | {p:>8.2}° | {:>11.2}%",
            "fora vs plano FIXO",
            s * 100.0
        );
    }
}

/// ⛔⛔ **A CALIBRAÇÃO do Bloco E — sobre a ESFERA LISA, onde a resposta certa é conhecida.**
///
/// Numa esfera o campo de normais visto de uma câmera **não depende** de como ela está rodada, logo
/// as quatro linhas abaixo têm resposta sabida de antemão: a verdade rodada é o repouso, e o
/// sucedâneo COMPLETO é a identidade. *Qualquer linha que não leia `~0` nomeia qual metade do
/// instrumento está partida* — e foi assim que a 1.ª redacção do Bloco E se descobriu a medir
/// `70,87°` sobre uma peça sem orientação nenhuma.
#[test]
#[ignore = "precisa de adapter"]
fn catavento_diag_a_convencao_do_sucedaneo() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter — skip");
        return;
    };
    const LADO: u32 = 512;
    let n = LADO as usize;
    let mesh = shapes::uv_sphere(48, 64, 1.0);
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        fov_y: core::f32::consts::FRAC_PI_4,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), 1.0);

    let repouso = plano(&device, &queue, &mesh, &cam, LADO);
    let verdade_z = plano(
        &device,
        &queue,
        &malha_rodada(&mesh, 2, core::f32::consts::FRAC_PI_2, cam.target.into()),
        &cam,
        LADO,
    );

    // Só a permutação dos pixels, sem tocar nas normais.
    let mut so_permuta = vec![0f32; repouso.len()];
    for r in 0..n {
        for c in 0..n {
            let o = (c * n + (n - 1 - r)) * 4;
            let d = (r * n + c) * 4;
            so_permuta[d..d + 4].copy_from_slice(&repouso[o..o + 4]);
        }
    }
    // Só a rotação das normais, no mesmo pixel.
    let mut so_normais = repouso.clone();
    for p in so_normais.as_chunks_mut::<4>().0 {
        let (x, y) = (p[0], p[1]);
        p[0] = -y;
        p[1] = x;
    }

    eprintln!(
        "\n=== CALIBRAÇÃO do sucedâneo, na esfera LISA (load {}) ===",
        carga()
    );
    let amostra = repouso
        .as_chunks::<4>()
        .0
        .iter()
        .find(|p| p[3] > 0.5)
        .unwrap();
    eprintln!(
        "uma normal do plano: [{:.4}, {:.4}, {:.4}] · cobertura {:.2}",
        amostra[0], amostra[1], amostra[2], amostra[3]
    );
    for (rotulo, outro) in [
        ("a verdade rodada vs repouso", &verdade_z),
        ("só a PERMUTA vs repouso", &so_permuta),
        ("só as NORMAIS vs repouso", &so_normais),
        ("suced. perm+ norm+", &sucedaneo_no_plano(&repouso, n, 1, 1)),
        (
            "suced. perm+ norm−",
            &sucedaneo_no_plano(&repouso, n, 1, -1),
        ),
        (
            "suced. perm− norm+",
            &sucedaneo_no_plano(&repouso, n, -1, 1),
        ),
        (
            "suced. perm− norm−",
            &sucedaneo_no_plano(&repouso, n, -1, -1),
        ),
    ] {
        let (m, p, s) = desacordo(&repouso, outro);
        eprintln!("{rotulo:>30} | {m:>8.2}° | {p:>8.2}° | {:>7.2}%", s * 100.0);
    }
}
