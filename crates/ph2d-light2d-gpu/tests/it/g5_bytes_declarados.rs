//! **G5** — bytes DECLARADOS = bytes ALOCADOS. A `PassList` declara cada textura e cada buffer
//! fixo do solver; o relatório do alocador do wgpu (`Device::generate_allocator_report`, que nomeia
//! cada alocação pelo rótulo) diz o que a placa de facto recebeu. Toda alocação nova depois de
//! `Solver::novo` tem de ser um recurso declarado, uma vez; uma textura criada fora da lista (com
//! ou sem rótulo) reprova.
//!
//! O alocador dá a alguns recursos uns bytes a mais que o pedido — o ENCHIMENTO do driver, que
//! não depende do tamanho (medido, RTX 5060 Ti/Vulkan, nas três grades: o uniforme de 16 B ocupa
//! 64, a `emissao` (alvo de render `Rgba16Float`) +736 B, o resto 0). Por isso a lei é: o
//! enchimento de cada recurso é o MESMO nas três grades e menor que 1 KiB. Um tamanho declarado
//! errado escala com a grade e quebra a igualdade.

use std::collections::BTreeMap;

use ph2d_light2d_gpu::{Config, PassList, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro};
use crate::comum::{gpu, grade};

type Alocacoes = BTreeMap<String, (usize, u64)>;

/// As alocações vivas, por nome: (quantas, bytes somados). Antes de ler, a fila esvazia: as
/// alocações de *staging* de um `write_buffer` só morrem depois do submit seguinte.
fn alocacoes(gpu: &ph2d_gpu::GpuContext) -> Option<Alocacoes> {
    gpu.queue.submit([]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let r = gpu.device.generate_allocator_report()?;
    let mut m = Alocacoes::new();
    for a in r.allocations {
        let e = m.entry(a.name).or_default();
        e.0 += 1;
        e.1 += a.size;
    }
    Some(m)
}

fn novas(antes: &Alocacoes, depois: &Alocacoes) -> Alocacoes {
    depois
        .iter()
        .filter_map(|(nome, &(n, b))| {
            let (n0, b0) = antes.get(nome).copied().unwrap_or_default();
            (n > n0).then(|| (nome.clone(), (n - n0, b - b0)))
        })
        .collect()
}

fn declarado(lista: &PassList) -> BTreeMap<String, u64> {
    lista
        .texturas
        .iter()
        .map(|t| (t.nome.to_owned(), t.bytes()))
        .chain(lista.buffers.iter().map(|b| (b.nome.to_owned(), b.bytes)))
        .collect()
}

#[test]
fn g5_o_que_a_placa_alocou_e_o_que_a_pass_list_declara() {
    let Some(gpu) = gpu() else { return };
    if alocacoes(&gpu).is_none() {
        eprintln!("G5: o backend não dá relatório do alocador — saltado");
        return;
    }
    let mut enchimento: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    // Cada degrau, e o Baixo com a grade FINA da luz direta (W4b: a fluência fina e a visibilidade).
    let configs = DEGRAUS
        .into_iter()
        .map(|(degrau, lado, texel)| (degrau, Config::padrao(grade(lado, texel))))
        .chain(std::iter::once((
            "Baixo com a grade fina ×4 e a captura fina ×4",
            Config {
                fina: 4,
                captura: 4,
                ..Config::padrao(grade(DEGRAUS[2].1, DEGRAUS[2].2))
            },
        )));
    for (degrau, cfg) in configs {
        let antes = alocacoes(&gpu).expect("relatório");
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let decl = declarado(s.pass_list());
        let criado = novas(&antes, &alocacoes(&gpu).expect("relatório"));
        eprintln!("G5 {degrau}: recurso | declarado | alocado | enchimento do driver");
        for (nome, &(n, bytes)) in &criado {
            let d = *decl.get(nome).unwrap_or_else(|| {
                panic!("G5: «{nome}» ({bytes} B) foi alocado e a PassList não o declara")
            });
            eprintln!("  {nome} | {d} | {bytes} | {:+}", bytes as i64 - d as i64);
            assert_eq!(n, 1, "G5: «{nome}» alocado {n} vezes");
            assert!(
                bytes >= d,
                "G5: «{nome}» declarado {d} B, alocado só {bytes} B"
            );
            enchimento
                .entry(nome.clone())
                .or_default()
                .push(bytes as i64 - d as i64);
        }
        for nome in decl.keys() {
            assert!(
                criado.contains_key(nome),
                "G5: «{nome}» declarado e nunca alocado"
            );
        }

        // A geometria da cena aparece no primeiro quadro, com a capacidade que o solver diz ter.
        let antes = alocacoes(&gpu).expect("relatório");
        s.quadro(&gpu, &ceu_e_vidro());
        let geo = novas(&antes, &alocacoes(&gpu).expect("relatório"));
        let nomes: Vec<&str> = geo.keys().map(String::as_str).collect();
        // ⭐ W5a: a captura fina dos opacos sobe a sua própria geometria (com rótulos próprios).
        let esperados: &[&str] = if cfg.usa_captura_fina() {
            &[
                "luz2d/buf/formas",
                "luz2d/buf/formas_fina",
                "luz2d/buf/vertices",
                "luz2d/buf/vertices_fina",
            ]
        } else {
            &["luz2d/buf/formas", "luz2d/buf/vertices"]
        };
        assert_eq!(nomes, esperados, "G5 geometria: {geo:?}");
        let alocado: u64 = geo.values().map(|v| v.1).sum();
        eprintln!(
            "  geometria | {} | {alocado}",
            s.bytes_da_geometria_residente()
        );
        assert!(
            alocado >= s.bytes_da_geometria_residente(),
            "G5 geometria: {geo:?}"
        );
    }
    for (nome, e) in &enchimento {
        assert!(
            e.iter().all(|&x| x == e[0]),
            "G5: o enchimento de «{nome}» muda com a grade: {e:?}"
        );
        assert!(
            e[0] < 1024,
            "G5: «{nome}» com {} B a mais que o declarado",
            e[0]
        );
    }
}

/// ⭐ W5b (F4): o mundo FORA da grade — os objetos distantes, as arestas e a atlas das máscaras
/// crescem com ele, e o que a placa aloca para eles cobre o que o solver diz ter residente.
#[test]
fn g5_o_mundo_fora_da_grade_e_contado() {
    use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
    use ph2d_light2d_gpu::{Amostragem, CapturaDeSprites, SpriteNaC0};
    let Some(gpu) = gpu() else { return };
    if alocacoes(&gpu).is_none() {
        eprintln!("G5: o backend não dá relatório do alocador — saltado");
        return;
    }
    // A grade [0, 4]²; a lâmpada, um polígono e um sprite à direita, fora dela.
    let mut s = Solver::novo(&gpu, Config::padrao(grade(64, 1.0 / 16.0))).expect("solver");
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Disco {
            centro: [6.0, 2.0],
            raio: 0.2,
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        oclusores: vec![Oclusor {
            vertices: vec![[4.5, 1.5], [4.8, 1.5], [4.8, 2.5], [4.5, 2.5]],
            opacidade: 1.0,
            albedo: [0.0; 3],
        }],
        luzes: 1,
        ..Default::default()
    };
    let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("g5 sprite"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let vista = textura.create_view(&Default::default());
    let sprites = CapturaDeSprites {
        texturas: vec![(&vista, Amostragem::Bilinear)],
        sprites: vec![SpriteNaC0 {
            origem: [5.0, 1.0],
            eixo_s: [0.3, 0.0],
            eixo_t: [0.0, -1.0],
            uv: [0.0, 0.0, 1.0, 1.0],
            emissao: [0.0; 3],
            opacidade: 1.0,
            premultiplicada: false,
            textura: 0,
        }],
        ..Default::default()
    };
    let antes = alocacoes(&gpu).expect("relatório");
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    let r = s.gravar_com_sprites(&gpu, &mut enc, &cena, &sprites);
    gpu.queue.submit([enc.finish()]);
    let depois = alocacoes(&gpu).expect("relatório");
    // Um recurso que cresce é TROCADO (a contagem fica): conta o que mudou de bytes ou nasceu.
    let mudou: Alocacoes = depois
        .iter()
        .filter(|(nome, v)| antes.get(*nome) != Some(*v))
        .map(|(n, v)| (n.clone(), *v))
        .collect();
    let alocado: u64 = mudou.values().map(|v| v.1).sum();
    eprintln!(
        "G5 distantes: {} objetos, {} máscaras · {mudou:?} · residente {}",
        r.distantes,
        r.mascaras,
        s.bytes_da_geometria_residente()
    );
    assert_eq!((r.distantes, r.mascaras), (2, 1), "G5 distantes: {r:?}");
    for nome in [
        "luz2d/buf/distantes",
        "luz2d/buf/arestas",
        "luz2d/tex/mascaras",
    ] {
        assert!(
            mudou.contains_key(nome),
            "G5 distantes: «{nome}» não cresceu: {mudou:?}"
        );
    }
    assert!(
        alocado >= s.bytes_da_geometria_residente(),
        "G5 distantes: alocado {alocado} < residente {}",
        s.bytes_da_geometria_residente()
    );
}
