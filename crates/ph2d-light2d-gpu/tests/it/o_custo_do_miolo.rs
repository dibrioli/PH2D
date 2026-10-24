//! ⭐⭐ W5o — **o custo do miolo e do entorno** (`docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W5o_2026-10-13_CUSTO.md`):
//! o miolo é limitado pelo NÚMERO de despachos (medido: ~1,2 µs cada, 605 por quadro na W5n; o nível
//! de 5×3 levava 64 por ciclo), e o entorno resolvia uma grade fina que ninguém lê. Na grade da
//! fluência do Low a 1080p (2560×1536), um disco opaco, a região da janela menor que a grade.
//!
//! Medido (RTX 5060 Ti, `smoke`, a rodada `medida_luz2d_w5o`, mínimo de 7 intercaladas):
//!
//! | | despachos do miolo | `c3_miolo` vazia · cena 5 (Low) | entorno cena 5 High |
//! |---|---|---|---|
//! | W5n | 605 | 1,067 · 1,171 ms | 8,70 ms |
//! | W5o | 247 | 0,720 · 0,813 ms (×0,68 · ×0,69) | 2,82 ms |

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, HOLOGRAFICA_PADRAO, MIOLO_PADRAO, Miolo, Solver};

use crate::comum::{disco, gpu, oclusor, regular};

const LARGURA: u32 = 2560;
const ALTURA: u32 = 1536;
const TEXEL_M: f64 = 0.004;

/// Os despachos do miolo por quadro com a base no grupo, uma relaxação por despacho — MEDIDO 247 (sem
/// ela 605); o teto é o medido + 5 %.
const TETO_DESPACHOS: usize = 260;
/// ⭐ W5q — com o bloco também (`Miolo::bloco` = 4 nos níveis pequenos) — MEDIDO 157 (sem ele 247); o
/// teto é o medido + 5 %.
const TETO_DESPACHOS_DO_BLOCO: usize = 165;
/// O `c3_miolo` com a base ÷ sem ela, no mesmo processo — MEDIDO ×0,68–×0,69; o teto deixa folga para
/// o ruído do relógio (o controlo, a razão 1 do custo antigo, reprova).
const TETO_RAZAO: f64 = 0.8;
/// A grade fina + a visibilidade do entorno (`so_a_c0`) ÷ as mesmas sem a flag — os passes vazios
/// medem ~0 contra ~0,2 ms (a vista vazia) a ~5 ms (a cena 5 High).
const TETO_ENTORNO: f64 = 0.1;

fn cena() -> CenaDeLuz {
    let c = [
        (f64::from(LARGURA) * 0.5 * TEXEL_M) as f32,
        (f64::from(ALTURA) * 0.5 * TEXEL_M) as f32,
    ];
    CenaDeLuz {
        emissores: vec![disco(
            [c[0] - 1.6, c[1] + 0.4],
            0.15,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![oclusor(regular(c, 0.9, 96, 0.1), 1.0)],
        luzes: 1,
        ..Default::default()
    }
}

/// `bloco` = o `Miolo::bloco` (1 = uma relaxação por despacho: a lei da base isolada).
fn solver(gpu: &ph2d_gpu::GpuContext, base: bool, so_a_c0: bool, bloco: u32) -> Solver {
    let g = Grade {
        origem_m: [0.0, 0.0],
        texel_m: TEXEL_M,
        largura: LARGURA,
        altura: ALTURA,
    };
    let cfg = Config {
        holografica: Some(HOLOGRAFICA_PADRAO),
        miolo: Some(Miolo {
            base_no_grupo: base,
            bloco,
            ..MIOLO_PADRAO
        }),
        so_a_c0,
        ..Config::padrao_para(&gpu.device, g)
    };
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.regiao_fina([LARGURA / 4, ALTURA / 4], [LARGURA * 3 / 4, ALTURA * 3 / 4]);
    s
}

/// Os despachos que o miolo grava por quadro.
fn despachos_do_miolo(gpu: &ph2d_gpu::GpuContext, c: &CenaDeLuz, base: bool, bloco: u32) -> usize {
    let mut s = solver(gpu, base, false, bloco);
    assert!(s.medir_fases_do_miolo(gpu));
    s.quadro(gpu, c);
    let f = s.fases_do_miolo(gpu).expect("fases");
    f.iter().map(|(_, _, n)| n).sum()
}

/// ⭐ bug 7 (W6i) — o que custam `n[k]` despachos DEPENDENTES (cada um lê e escreve o mesmo buffer, como as
/// relaxações do miolo) numa passagem, nesta placa: o MÍNIMO de 7 rodadas intercaladas, em ms.
fn custo_de_despachos_ms(gpu: &ph2d_gpu::GpuContext, n: [usize; 2]) -> [f64; 2] {
    let d = &gpu.device;
    let m = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custo de um despacho"),
        source: wgpu::ShaderSource::Wgsl(
            "@group(0) @binding(0) var<storage, read_write> b: array<u32>;
             @compute @workgroup_size(64) fn main() { b[0] += 1u; }"
                .into(),
        ),
    });
    let pipe = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &m,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let buf = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 256,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipe.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buf.as_entire_binding(),
        }],
    });
    let qs = d.create_query_set(&wgpu::QuerySetDescriptor {
        label: None,
        ty: wgpu::QueryType::Timestamp,
        count: 2,
    });
    let resolve = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let periodo = f64::from(gpu.queue.get_timestamp_period());
    let medir = |quantos: usize| {
        let mut enc = d.create_command_encoder(&Default::default());
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: &qs,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });
            p.set_pipeline(&pipe);
            p.set_bind_group(0, &bg, &[]);
            for _ in 0..quantos {
                p.dispatch_workgroups(1, 1, 1);
            }
        }
        enc.resolve_query_set(&qs, 0..2, &resolve, 0);
        enc.copy_buffer_to_buffer(&resolve, 0, &leitura, 0, 16);
        gpu.queue.submit([enc.finish()]);
        let f = leitura.slice(..);
        f.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        d.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        let t: [u64; 2] = bytemuck::pod_read_unaligned(&f.get_mapped_range());
        leitura.unmap();
        t[1].saturating_sub(t[0]) as f64 * periodo * 1e-6
    };
    let mut minimo = [f64::INFINITY; 2];
    for rodada in 0..7 {
        for o in 0..2 {
            let k = (o + rodada) % 2;
            minimo[k] = minimo[k].min(medir(n[k]));
        }
    }
    minimo
}

fn tem_relogio(gpu: &ph2d_gpu::GpuContext) -> bool {
    gpu.device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
}

/// ⭐⭐ **GATE — o miolo grava poucos despachos**: com a base no grupo (`c3_miolo_base.wgsl`) ≤
/// [`TETO_DESPACHOS`]; o controlo (um despacho por relaxação, a W5n) passa dele. ⭐ W5q: e com o bloco ≤
/// [`TETO_DESPACHOS_DO_BLOCO`], o controlo (sem ele) acima.
#[test]
fn a_base_do_miolo_corta_os_despachos() {
    let Some(gpu) = gpu() else { return };
    if !tem_relogio(&gpu) {
        eprintln!("sem TIMESTAMP_QUERY — o instrumento das fases não corre; gate saltado");
        return;
    }
    let c = cena();
    let despachos = |base: bool, bloco: u32| despachos_do_miolo(&gpu, &c, base, bloco);
    let (com, sem) = (despachos(true, 1), despachos(false, 1));
    let bloco = despachos(true, MIOLO_PADRAO.bloco);
    eprintln!(
        "despachos do miolo: com a base {com} · sem ela {sem} · com a base e o bloco {bloco}"
    );
    assert!(
        com > TETO_DESPACHOS_DO_BLOCO,
        "controlo: sem o bloco o miolo grava {com} despachos — o teto do bloco não o separa"
    );
    assert!(
        bloco <= TETO_DESPACHOS_DO_BLOCO,
        "com o bloco o miolo grava {bloco} despachos por quadro (teto {TETO_DESPACHOS_DO_BLOCO})"
    );
    assert!(
        sem > TETO_DESPACHOS,
        "controlo: sem a base o miolo grava {sem} despachos — o teto não o separa"
    );
    assert!(
        com <= TETO_DESPACHOS,
        "o miolo grava {com} despachos por quadro (teto {TETO_DESPACHOS})"
    );
}

/// ⭐⭐ **GATE — o custo do miolo desce com a base, e o entorno não paga a grade fina**: no mesmo
/// processo, intercalado, o MÍNIMO de 7 rodadas (`docs/DevOps/MEDIR_VELOCIDADE.md`); a régua é a razão,
/// não o número. Controlos: o custo antigo contra si mesmo (razão 1) e a grade fina sem a flag.
#[test]
fn o_custo_do_miolo_desce_com_a_base() {
    let Some(gpu) = gpu() else { return };
    if !tem_relogio(&gpu) {
        eprintln!("sem TIMESTAMP_QUERY — gate saltado");
        return;
    }
    let c = cena();
    // [com a base, sem ela, sem ela (o controlo), entorno com a flag, entorno sem ela].
    let mut s = [
        solver(&gpu, true, false, 1),
        solver(&gpu, false, false, 1),
        solver(&gpu, false, false, 1),
        solver(&gpu, true, true, 1),
        solver(&gpu, true, false, 1),
    ];
    let medir = |t: &ph2d_light2d_gpu::TemposMs, k: usize| {
        if k >= 3 {
            t.c3_fina + t.c3_visibilidade + t.c3_miolo
        } else {
            t.c3_miolo
        }
    };
    let mut minimo = [f64::INFINITY; 5];
    for x in &mut s {
        for _ in 0..3 {
            x.quadro(&gpu, &c);
        }
    }
    for rodada in 0..7 {
        for o in 0..s.len() {
            let k = (o + rodada) % s.len();
            for _ in 0..3 {
                s[k].quadro(&gpu, &c);
            }
            let t = s[k].tempos(&gpu).expect("TIMESTAMP_QUERY");
            minimo[k] = minimo[k].min(medir(&t, k));
        }
    }
    let razao = minimo[0] / minimo[1];
    let controlo = minimo[2] / minimo[1];
    let entorno = minimo[3] / minimo[4];
    eprintln!(
        "c3_miolo: com a base {:.3} · sem ela {:.3} · {:.3} ms (razão {razao:.3}, controlo {controlo:.3}); \
         entorno fina+vis+miolo: {:.3} contra {:.3} ms (razão {entorno:.3})",
        minimo[0], minimo[1], minimo[2], minimo[3], minimo[4]
    );
    assert!(
        controlo > TETO_RAZAO,
        "controlo: o custo antigo contra si mesmo dá {controlo:.3} — a régua não separa"
    );
    // ⭐ bug 7 (W6i) — a lei da base é a de uma placa LIMITADA POR DESPACHOS (a RTX: ~1,2 µs cada); numa que
    // não é (a Radeon integrada: o miolo ~11 ms de trabalho), cortar despachos não desce o custo. A placa diz
    // qual é: o custo dos despachos que a base corta, medido aqui, contra o miolo sem ela.
    let (com_n, sem_n) = (
        despachos_do_miolo(&gpu, &c, true, 1),
        despachos_do_miolo(&gpu, &c, false, 1),
    );
    let [t_sem, t_com] = custo_de_despachos_ms(&gpu, [sem_n, com_n]);
    let poupanca = (t_sem - t_com) / minimo[1];
    eprintln!(
        "os despachos: {sem_n} custam {t_sem:.3} ms · {com_n} custam {t_com:.3} ms — a base poupa {:.0} % do miolo sem ela",
        100.0 * poupanca
    );
    if poupanca >= 1.0 - TETO_RAZAO {
        assert!(
            razao <= TETO_RAZAO,
            "o miolo com a base custa {razao:.3} do de sem ela (teto {TETO_RAZAO})"
        );
    } else {
        eprintln!(
            "a placa não é limitada por despachos: a lei da base (≤ {TETO_RAZAO}) não se aplica nela; os despachos \
             continuam no teto (`a_base_do_miolo_corta_os_despachos`)"
        );
    }
    assert!(
        entorno <= TETO_ENTORNO,
        "o entorno com `so_a_c0` ainda paga {entorno:.3} da grade fina e do miolo (teto {TETO_ENTORNO})"
    );
}

/// ⭐⭐ **GATE (item 4, 24/10) — saltar o fora não muda um bit**: o miolo de fábrica (`Miolo::salta_o_fora`:
/// os passes dos níveis ≥ 1 não gravam os texels fora dos opacos) contra o de antes (gravam), a fluência
/// inteira depois de 4 quadros: 0 texels diferentes em TODA placa (a conta é a mesma; só a escrita morta
/// sai). Controlo: um V-ciclo a menos muda a fluência (a régua vê o miolo). O custo: `medida_luz2d_miolo_formas`
/// (crate `ph2d-app-light2d`; RTX ×0,92–0,93, Radeon 610M ×0,42–0,59).
#[test]
fn saltar_o_fora_do_miolo_nao_muda_um_bit() {
    let Some(gpu) = gpu() else { return };
    let c = cena();
    let fluencia = |salta: bool, menos_um: bool| {
        let cfg = *solver(&gpu, true, false, MIOLO_PADRAO.bloco).config();
        let m = cfg.miolo.expect("o miolo");
        let cfg = Config {
            miolo: Some(Miolo {
                salta_o_fora: salta,
                ciclos: m.ciclos - u32::from(menos_um),
                ..m
            }),
            ..cfg
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        s.regiao_fina([LARGURA / 4, ALTURA / 4], [LARGURA * 3 / 4, ALTURA * 3 / 4]);
        for _ in 0..4 {
            s.quadro(&gpu, &c);
        }
        crate::comum::ler_fluencia(&gpu, &s)
    };
    assert!(MIOLO_PADRAO.salta_o_fora, "o miolo de fábrica salta o fora");
    let (produto, antes, controlo) = (
        fluencia(true, false),
        fluencia(false, false),
        fluencia(false, true),
    );
    let dif = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .filter(|(x, y)| x.map(f32::to_bits) != y.map(f32::to_bits))
            .count()
    };
    let (d, dc) = (dif(&produto, &antes), dif(&controlo, &antes));
    eprintln!(
        "saltar o fora: {d} texels diferentes de {} · controlo (um ciclo a menos) {dc}",
        antes.len()
    );
    assert!(
        dc > 0,
        "controlo: um V-ciclo a menos não mudou a fluência — a régua não vê o miolo"
    );
    assert_eq!(d, 0, "saltar o fora do miolo mudou {d} texels da fluência");
}

/// O quadro `k` da cena a mexer: o disco opaco anda (passo fracionário de texel) e some no quadro 3; um
/// quadrado translúcido (`opacidade` 0,45) e um corpo de opacidade 0,8 entram e saem; a lâmpada anda.
fn cena_a_mexer(k: usize) -> CenaDeLuz {
    let c = [
        (f64::from(LARGURA) * 0.5 * TEXEL_M) as f32,
        (f64::from(ALTURA) * 0.5 * TEXEL_M) as f32,
    ];
    let t = k as f32;
    let mut oclusores = Vec::new();
    if k != 3 {
        oclusores.push(oclusor(
            regular([c[0] - 0.6 + 0.2371 * t, c[1] + 0.05 * t], 0.7, 96, 0.1),
            1.0,
        ));
    }
    if k % 2 == 0 {
        oclusores.push(oclusor(
            regular([c[0] + 1.1, c[1] - 0.5], 0.45, 4, 0.3 + 0.1 * t),
            0.45,
        ));
    }
    if k >= 2 {
        oclusores.push(oclusor(
            regular([c[0] + 0.3 - 0.113 * t, c[1] + 0.9], 0.35, 7, 0.0),
            0.8,
        ));
    }
    CenaDeLuz {
        emissores: vec![disco(
            [c[0] - 1.6 + 0.031 * t, c[1] + 0.4],
            0.15,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores,
        luzes: 1,
        ..Default::default()
    }
}

/// A região fina do quadro `k`: muda a cada quadro (a da janela que anda e muda de tamanho).
fn regiao_a_mexer(k: usize) -> ([u32; 2], [u32; 2]) {
    let d = 37 * k as u32;
    match k % 3 {
        0 => ([LARGURA / 4, ALTURA / 4], [LARGURA * 3 / 4, ALTURA * 3 / 4]),
        1 => (
            [LARGURA / 5 + d, ALTURA / 6],
            [LARGURA * 3 / 5 + d, ALTURA * 2 / 3 + d / 2],
        ),
        _ => (
            [LARGURA / 3 - d / 2, ALTURA / 3],
            [LARGURA * 5 / 6, ALTURA * 5 / 6],
        ),
    }
}

/// ⭐⭐ **GATE (item 4) — saltar o fora não muda um bit com a cena a MEXER**: o texel saltado guarda o
/// valor antigo, e o caso que o salto torna diferente é o texel que passa de dentro a fora a dentro.
/// Oito quadros de [`cena_a_mexer`] e [`regiao_a_mexer`] (oclusor que anda, some e volta; translúcido;
/// três corpos; a região fina a mudar), dois Solvers lado a lado (`salta_o_fora` true × false), a
/// fluência bit a bit em CADA quadro, em quatro configs: a de fábrica, a base fora do grupo com e sem
/// bloco (os `miolo_inicia`/`miolo_erro_inicia` despachados) e a base no grupo sem bloco. Controlo: um
/// V-ciclo a menos, a mesma sequência, muda a fluência.
#[test]
fn saltar_o_fora_nao_muda_um_bit_com_a_cena_a_mexer() {
    let Some(gpu) = gpu() else { return };
    let info = gpu.adapter.get_info();
    eprintln!("adaptador: {} ({:?})", info.name, info.backend);
    const QUADROS: usize = 8;
    let novo = |base: bool, bloco: u32, salta: bool, menos_um: bool| {
        let cfg = *solver(&gpu, base, false, bloco).config();
        let m = cfg.miolo.expect("o miolo");
        Solver::novo(
            &gpu,
            Config {
                miolo: Some(Miolo {
                    salta_o_fora: salta,
                    ciclos: m.ciclos - u32::from(menos_um),
                    ..m
                }),
                ..cfg
            },
        )
        .expect("solver")
    };
    let dif = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .filter(|(x, y)| x.map(f32::to_bits) != y.map(f32::to_bits))
            .count()
    };
    // (rótulo, base_no_grupo, bloco)
    let configs = [
        ("fábrica", true, MIOLO_PADRAO.bloco),
        ("base fora, bloco de fábrica", false, MIOLO_PADRAO.bloco),
        ("base fora, bloco 1 (CORTES=0)", false, 1),
        ("base no grupo, bloco 1", true, 1),
    ];
    let mut controlo_total = 0;
    let mut reprovados = Vec::new();
    for (rotulo, base, bloco) in configs {
        let mut s = [
            novo(base, bloco, true, false),
            novo(base, bloco, false, false),
        ];
        let mut ctl = (rotulo == "fábrica").then(|| novo(base, bloco, false, true));
        let mut por_quadro = Vec::with_capacity(QUADROS);
        for k in 0..QUADROS {
            let (c, (i, f)) = (cena_a_mexer(k), regiao_a_mexer(k));
            for x in s.iter_mut().chain(ctl.as_mut()) {
                x.regiao_fina(i, f);
                x.quadro(&gpu, &c);
            }
            let (produto, antes) = (
                crate::comum::ler_fluencia(&gpu, &s[0]),
                crate::comum::ler_fluencia(&gpu, &s[1]),
            );
            if let Some(x) = &ctl {
                controlo_total += dif(&crate::comum::ler_fluencia(&gpu, x), &antes);
            }
            let d = dif(&produto, &antes);
            por_quadro.push(d);
        }
        eprintln!("[{rotulo}] texels diferentes por quadro: {por_quadro:?}");
        reprovados.extend(
            por_quadro
                .iter()
                .enumerate()
                .filter(|(_, d)| **d > 0)
                .map(|(k, d)| format!("[{rotulo}] quadro {k}: {d}")),
        );
    }
    assert!(
        reprovados.is_empty(),
        "saltar o fora mudou texels da fluência: {reprovados:?}"
    );
    eprintln!(
        "controlo (um V-ciclo a menos, fábrica, soma dos {QUADROS} quadros): {controlo_total}"
    );
    assert!(
        controlo_total > 0,
        "controlo: um V-ciclo a menos não mudou a fluência — a régua não vê o miolo"
    );
}
