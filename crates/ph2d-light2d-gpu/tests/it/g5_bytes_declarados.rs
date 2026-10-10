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
            "Baixo com a grade fina ×4",
            Config {
                fina: 4,
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
        assert_eq!(
            nomes,
            ["luz2d/buf/formas", "luz2d/buf/vertices"],
            "G5 geometria: {geo:?}"
        );
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
