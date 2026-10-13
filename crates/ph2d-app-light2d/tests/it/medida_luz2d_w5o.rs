//! **A rodada da W5o — o custo da luz por passe** (método `docs/DevOps/MEDIR_VELOCIDADE.md`). À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5o -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A configuração do PRODUTO (a vista de fábrica a 1080p, sem o Automático) e o ENTORNO dele, por
//! variante do miolo como CONFIG EM EXECUÇÃO no mesmo processo; [`RODADAS`] intercaladas com a ordem
//! rodada, vale o MÍNIMO (a mediana do total ao lado). Mais uma instância com o instrumento das FASES
//! do miolo ([`ph2d_light2d_gpu::Solver::medir_fases_do_miolo`]).

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::regiao_da_janela;
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, MIOLO_PADRAO, Miolo, Solver, TemposMs};

use crate::medida_luz2d_w5e::{config_do_entorno, config_do_produto, fabrica};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 7;
const QUADROS: usize = 3;

/// Uma variante: (nome, ciclos do miolo — `None` = `PH2D_LIGHT2D_MIOLO=0` —, anel, a base no grupo
/// (W5o, corte 2), o entorno só com a c0 (W5o, corte 1)).
type Variante = (&'static str, Option<u32>, bool, bool, bool);

/// A W5n é o ANTES; a última é o padrão (W5o).
const VARIANTES: [Variante; 5] = [
    ("sem miolo", None, false, true, true),
    ("W5h (0 ciclos, sem anel)", Some(0), false, false, false),
    ("W5n (antes)", Some(3), true, false, false),
    ("corte 1 (entorno só a c0)", Some(3), true, false, true),
    ("W5o (cortes 1 + 2)", Some(3), true, true, true),
];

fn variante(cfg: Config, (_, ciclos, anel, base, so_c0): Variante) -> Config {
    Config {
        miolo: ciclos.map(|ciclos| Miolo {
            ciclos,
            base_no_grupo: base,
            ..MIOLO_PADRAO
        }),
        anel_fora_da_regiao: anel,
        // Só o entorno tem a flag no produto: a vista nunca a liga.
        so_a_c0: cfg.so_a_c0 && so_c0,
        ..cfg
    }
}

struct Par {
    vista: Solver,
    entorno: Option<Solver>,
}

impl Par {
    fn novo(gpu: &ph2d_gpu::GpuContext, vista: Config, entorno: Option<Config>) -> Self {
        let a = fabrica(FHD);
        let mut v = Solver::novo(gpu, vista).expect("solver da vista");
        let (i, f) = regiao_da_janela(v.grade_da_fluencia(), a, FHD);
        v.regiao_fina(i, f);
        let mut e = entorno.map(|c| Solver::novo(gpu, c).expect("solver do entorno"));
        if let Some(e) = e.as_mut() {
            let g = v.config().grade;
            e.mascarar_miolo(
                gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ]),
            );
        }
        v.somar_entorno(gpu, e.as_ref());
        Self {
            vista: v,
            entorno: e,
        }
    }

    fn quadro(&mut self, gpu: &ph2d_gpu::GpuContext, cena: &CenaDeLuz) {
        if let Some(e) = self.entorno.as_mut() {
            let sem_ceu = CenaDeLuz {
                ceu: [0.0; 3],
                ..cena.clone()
            };
            e.quadro(gpu, &sem_ceu);
        }
        self.vista.quadro(gpu, cena);
    }

    fn tempos(&self, gpu: &ph2d_gpu::GpuContext) -> (TemposMs, Option<TemposMs>) {
        (
            self.vista.tempos(gpu).expect("TIMESTAMP_QUERY"),
            self.entorno
                .as_ref()
                .map(|e| e.tempos(gpu).expect("TIMESTAMP_QUERY")),
        )
    }
}

fn minimo(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

fn mediana(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[s.len() / 2]
}

/// Uma coluna por passe: o nome e a leitura.
type Coluna = (&'static str, fn(&TemposMs) -> f64);
/// As fases do miolo de um quadro: (nome, ms, despachos).
type Fases = Vec<(&'static str, f64, usize)>;

/// As colunas por passe da vista.
const PASSES: [Coluna; 11] = [
    ("c0", |t| t.c0),
    ("c1", |t| t.c1),
    ("c2", TemposMs::c2_total),
    ("c3_vis", |t| t.c3_visibilidade),
    ("c3_fina(+anel)", |t| t.c3_fina),
    ("c0_fina", |t| t.c0_fina),
    ("c3_preenche", |t| t.c3_preenche),
    ("c3_mascaras", |t| t.c3_mascaras),
    ("c0_reb", |t| t.c0_rebote),
    ("c3_reb", |t| t.c3_rebote),
    ("c3_miolo", |t| t.c3_miolo),
];

fn cenas() -> Vec<(&'static str, CenaDeLuz)> {
    let mut vazia = World::new();
    vazia.spawn((
        ph2d_light2d_ecs::Luz::default(),
        ph2d_ecs::Transform::IDENTITY,
    ));
    let mut c5 = World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut c5, 5));
    vec![
        (
            "vazia (a do portão a_nitidez_da_luz)",
            ph2d_light2d_ecs::coleta::cena_de_luz(&vazia),
        ),
        (
            "cena 5 (os corpos)",
            ph2d_light2d_ecs::coleta::cena_de_luz(&c5),
        ),
    ]
}

#[test]
#[ignore = "rodada de medição W5o (placa): o custo da luz por passe"]
fn medida_luz2d_w5o() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
    {
        eprintln!("medida: sem TIMESTAMP_QUERY");
        return;
    }
    eprintln!(
        "# W5o · {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        crate::medida_luz2d_w4b::loadavg()
    );
    for (nome_cena, cena) in cenas() {
        for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
            let dg = degrau_do(nivel);
            let a = fabrica(FHD);
            let produto = config_do_produto(a, FHD, dg).expect("grade");
            // O produto troca a aritmética pela do dispositivo (`Config::padrao_para`).
            let aritmetica = Config::padrao_para(&gpu.device, produto.grade).aritmetica;
            let vista = Config {
                aritmetica,
                ..produto
            };
            let entorno = config_do_entorno(a, FHD, dg).map(|c| Config { aritmetica, ..c });
            let mut pares: Vec<Par> = VARIANTES
                .iter()
                .map(|&v| Par::novo(&gpu, variante(vista, v), entorno.map(|e| variante(e, v))))
                .collect();
            // O instrumento das fases: a W5n (antes) e a W5o.
            let mut fases: Vec<Par> = [VARIANTES[2], VARIANTES[4]]
                .iter()
                .map(|&v| {
                    let mut p =
                        Par::novo(&gpu, variante(vista, v), entorno.map(|e| variante(e, v)));
                    assert!(p.vista.medir_fases_do_miolo(&gpu));
                    p
                })
                .collect();
            // Aquecimento fora da régua.
            for p in pares.iter_mut().chain(fases.iter_mut()) {
                for _ in 0..QUADROS {
                    p.quadro(&gpu, &cena);
                }
                let _ = p.tempos(&gpu);
            }
            let np = pares.len();
            let n = np + fases.len();
            let mut amostras: Vec<Vec<(TemposMs, Option<TemposMs>)>> = vec![Vec::new(); np];
            let mut f_vista: Vec<Vec<Fases>> = vec![Vec::new(); fases.len()];
            for r in 0..RODADAS {
                for o in 0..n {
                    let k = (o + r) % n;
                    let p = if k < np {
                        &mut pares[k]
                    } else {
                        &mut fases[k - np]
                    };
                    for _ in 0..QUADROS {
                        p.quadro(&gpu, &cena);
                    }
                    if k < np {
                        amostras[k].push(p.tempos(&gpu));
                    } else {
                        let _ = p.tempos(&gpu);
                        f_vista[k - np].push(p.vista.fases_do_miolo(&gpu).expect("fases"));
                    }
                }
            }
            let gf = pares[4].vista.grade_da_fluencia();
            let ge = pares[4].entorno.as_ref().map(Solver::grade_da_fluencia);
            eprintln!(
                "\n## {nome_cena} · {nivel:?} · 1080p — fluência da vista {}×{} (níveis do miolo {:?}), \
                 do entorno {:?} (níveis {:?})",
                gf.largura,
                gf.altura,
                pares[4].vista.config().niveis_do_miolo(),
                ge.map(|g| (g.largura, g.altura)),
                pares[4]
                    .entorno
                    .as_ref()
                    .and_then(|e| e.config().niveis_do_miolo()),
            );
            let mut cab = String::from("| variante |");
            for (p, _) in PASSES {
                cab += &format!(" {p} |");
            }
            cab += " vista mín · med | entorno mín (miolo) | soma mín |";
            eprintln!("{cab}");
            eprintln!("|{}", "---|".repeat(PASSES.len() + 4));
            for (j, (nome, ..)) in VARIANTES.iter().enumerate() {
                let a = &amostras[j];
                let mut linha = format!("| {nome} |");
                for (_, f) in PASSES {
                    linha += &format!(
                        " {:.3} |",
                        minimo(&a.iter().map(|(t, _)| f(t)).collect::<Vec<_>>())
                    );
                }
                let tot: Vec<f64> = a.iter().map(|(t, _)| t.total()).collect();
                let ent: Vec<f64> = a
                    .iter()
                    .map(|(_, e)| e.as_ref().map_or(0.0, TemposMs::total))
                    .collect();
                let ent_m: Vec<f64> = a
                    .iter()
                    .map(|(_, e)| e.as_ref().map_or(0.0, |e| e.c3_miolo))
                    .collect();
                let soma: Vec<f64> = tot.iter().zip(&ent).map(|(a, b)| a + b).collect();
                linha += &format!(
                    " {:.3} · {:.3} | {:.3} ({:.3}) | {:.3} |",
                    minimo(&tot),
                    mediana(&tot),
                    minimo(&ent),
                    minimo(&ent_m),
                    minimo(&soma)
                );
                eprintln!("{linha}");
            }
            for (quem, f) in [("W5n", &f_vista[0]), ("W5o", &f_vista[1])] {
                let mut linha = format!("fases do miolo da vista ({quem}, mín):");
                let mut soma = 0.0;
                for i in 0..f[0].len() {
                    let ms = minimo(&f.iter().map(|v| v[i].1).collect::<Vec<_>>());
                    soma += ms;
                    linha += &format!(" {} {ms:.3} ms/{} desp ·", f[0][i].0, f[0][i].2);
                }
                eprintln!("{linha} soma {soma:.3}");
            }
        }
    }
    eprintln!("\n# loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}

/// ⭐ W5o — duas perguntas da 1.ª rodada: (A) a série do custo nos primeiros quadros de um solver
/// novo (o portão `a_nitidez_da_luz` lê o Low poucos quadros depois de criar a grade); (B) a `c2` do
/// High segue a VARIANTE ou a INSTÂNCIA (a ordem de criação)?
#[test]
#[ignore = "rodada de medição W5o (placa): a série dos primeiros quadros e a c2 por instância"]
fn medida_luz2d_w5o_serie_e_instancia() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!("# loadavg antes {}", crate::medida_luz2d_w4b::loadavg());
    let (_, vazia) = cenas().remove(0);
    let a = fabrica(FHD);
    let cfgs = |nivel| {
        let produto = config_do_produto(a, FHD, degrau_do(nivel)).expect("grade");
        let aritmetica = Config::padrao_para(&gpu.device, produto.grade).aritmetica;
        (
            Config {
                aritmetica,
                ..produto
            },
            config_do_entorno(a, FHD, degrau_do(nivel)).map(|c| Config { aritmetica, ..c }),
        )
    };
    // A. A série: um solver novo de cada vez, o total (vista + entorno) quadro a quadro.
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        let (vista, entorno) = cfgs(nivel);
        for v in [VARIANTES[1], VARIANTES[4]] {
            let mut p = Par::novo(&gpu, variante(vista, v), entorno.map(|e| variante(e, v)));
            let mut linha = format!("A {nivel:?} · {}:", v.0);
            for q in 0..40 {
                p.quadro(&gpu, &vazia);
                let (t, e) = p.tempos(&gpu);
                if q < 12 || q % 4 == 3 {
                    linha += &format!(
                        " {q}:{:.2}",
                        t.total() + e.as_ref().map_or(0.0, TemposMs::total)
                    );
                }
            }
            eprintln!("{linha}");
        }
    }
    // B. Seis instâncias no High, a ordem de criação trocada; a c2 de cada (o MÍNIMO de 7).
    let (vista, entorno) = cfgs(NivelDaLuz::Alta);
    let ordem = [4usize, 1, 4, 1, 0, 4];
    let mut pares: Vec<Par> = ordem
        .iter()
        .map(|&j| {
            let v = VARIANTES[j];
            Par::novo(&gpu, variante(vista, v), entorno.map(|e| variante(e, v)))
        })
        .collect();
    let mut c2 = vec![Vec::new(); pares.len()];
    for r in 0..RODADAS {
        for o in 0..pares.len() {
            let k = (o + r) % pares.len();
            for _ in 0..QUADROS {
                pares[k].quadro(&gpu, &vazia);
            }
            c2[k].push(pares[k].tempos(&gpu).0.c2.clone());
        }
    }
    for (i, &j) in ordem.iter().enumerate() {
        let niveis = c2[i][0].len();
        let por: Vec<f64> = (0..niveis)
            .map(|n| minimo(&c2[i].iter().map(|v| v[n]).collect::<Vec<_>>()))
            .collect();
        eprintln!(
            "B instância {i} ({}) · c2 {:.3} ms · por cascata {:?}",
            VARIANTES[j].0,
            por.iter().sum::<f64>(),
            por.iter()
                .map(|x| (x * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>()
        );
    }
    eprintln!("# loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}

/// ⭐ W5o — a `c2` do High depende do que a placa já alocou? Um processo por pergunta não serve (o
/// estado do alocador É a variável): cada caso cria o solver High DEPOIS de um lastro de `mb` MB
/// (texturas `Rgba16Float` vivas) — ou depois de criar e largar um Low (a sequência do portão).
/// Corre-se UM caso por processo: `PH2D_W5O_LASTRO=<mb>` ou `=low`.
#[test]
#[ignore = "rodada de medição W5o (placa): a c2 do High contra o lastro alocado antes"]
fn medida_luz2d_w5o_lastro() {
    let Some(casa) = crate::comum::gpu() else {
        return;
    };
    // `PH2D_W5O_DICAS`: o dispositivo de novo, com outras `MemoryHints` (blocos do alocador).
    let dicas = std::env::var("PH2D_W5O_DICAS").unwrap_or_default();
    const MB: u64 = 1024 * 1024;
    let hints = match dicas.as_str() {
        "" => None,
        "memoria" => Some(wgpu::MemoryHints::MemoryUsage),
        "256" => Some(wgpu::MemoryHints::Manual {
            suballocated_device_memory_block_size: 256 * MB..256 * MB,
        }),
        "4" => Some(wgpu::MemoryHints::Manual {
            suballocated_device_memory_block_size: 4 * MB..4 * MB,
        }),
        d => panic!("PH2D_W5O_DICAS = memoria | 256 | 4, não {d}"),
    };
    let gpu = match hints {
        None => casa,
        Some(memory_hints) => {
            let (device, queue) =
                pollster::block_on(casa.adapter.request_device(&wgpu::DeviceDescriptor {
                    label: Some("w5o dicas"),
                    required_features: casa.device.features(),
                    required_limits: casa.device.limits(),
                    memory_hints,
                    ..Default::default()
                }))
                .expect("device");
            ph2d_gpu::GpuContext {
                instance: casa.instance.clone(),
                adapter: casa.adapter.clone(),
                device: std::sync::Arc::new(device),
                queue: std::sync::Arc::new(queue),
            }
        }
    };
    let caso = std::env::var("PH2D_W5O_LASTRO").unwrap_or_else(|_| "0".into());
    let (_, vazia) = cenas().remove(0);
    let a = fabrica(FHD);
    let cfgs = |nivel| {
        let produto = config_do_produto(a, FHD, degrau_do(nivel)).expect("grade");
        let aritmetica = Config::padrao_para(&gpu.device, produto.grade).aritmetica;
        (
            Config {
                aritmetica,
                ..produto
            },
            config_do_entorno(a, FHD, degrau_do(nivel)).map(|c| Config { aritmetica, ..c }),
        )
    };
    let mut lastro = Vec::new();
    let mut vivo = None;
    let antes = |nivel| {
        let (v, e) = cfgs(nivel);
        let mut p = Par::novo(&gpu, v, e);
        for _ in 0..5 {
            p.quadro(&gpu, &vazia);
        }
        p
    };
    if caso == "low" || caso == "high" {
        drop(antes(if caso == "low" {
            NivelDaLuz::Baixa
        } else {
            NivelDaLuz::Alta
        }));
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    } else if caso == "low-vivo" {
        vivo = Some(antes(NivelDaLuz::Baixa));
    } else {
        let (mb, larga) = caso
            .strip_suffix("-largado")
            .map_or((caso.as_str(), false), |m| (m, true));
        let mb: u32 = mb
            .parse()
            .expect("PH2D_W5O_LASTRO = mb[-largado] | low | low-vivo | high");
        for _ in 0..mb / 32 {
            // 32 MB por textura (2048² × 8 B).
            lastro.push(gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("lastro"),
                size: wgpu::Extent3d {
                    width: 2048,
                    height: 2048,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }));
        }
        if larga {
            // Escritas uma vez (a memória tocada), largadas antes do High.
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            for t in &lastro {
                enc.clear_texture(t, &wgpu::ImageSubresourceRange::default());
            }
            gpu.queue.submit([enc.finish()]);
            lastro.clear();
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
    }
    let (v, e) = cfgs(NivelDaLuz::Alta);
    if std::env::var("PH2D_W5O_LISTA").is_ok() {
        let mut t: Vec<_> = ph2d_light2d_gpu::PassList::de(&v, true)
            .texturas
            .iter()
            .map(|t| (t.bytes(), t.nome, t.tamanho, t.formato, t.mips))
            .collect();
        t.sort_by_key(|x| std::cmp::Reverse(x.0));
        for (b, n, tam, f, m) in t.iter().take(14) {
            eprintln!(
                "T {:>7.1} MB {n} {tam:?} {f:?} mips {m}",
                *b as f64 / MB as f64
            );
        }
        let mut bs: Vec<_> = ph2d_light2d_gpu::PassList::de(&v, true)
            .buffers
            .iter()
            .map(|b| (b.bytes, b.nome))
            .collect();
        bs.sort_by_key(|x| std::cmp::Reverse(x.0));
        for (b, n) in bs.iter().take(10) {
            eprintln!("B {:>7.1} MB {n}", *b as f64 / MB as f64);
        }
    }
    let mut p = Par::novo(&gpu, v, e);
    let mut c2 = Vec::new();
    for _ in 0..12 {
        p.quadro(&gpu, &vazia);
        c2.push(p.tempos(&gpu).0.c2_total());
    }
    eprintln!(
        "C dicas {dicas:?} lastro {caso} ({} texturas, vivo {}) · c2 High mín {:.3} · med {:.3} ms",
        lastro.len(),
        vivo.is_some(),
        minimo(&c2),
        mediana(&c2)
    );
}

/// ⭐ W5o — INSTRUMENTO da prova ao bit: grava em `$PH2D_W5O_SAIDA` as fluências da vista pelo
/// caminho do PRODUTO (`LuzDoQuadro`, os sprites por texel, 80 quadros): a cena 3 com a câmera em
/// x = 2 (a luz de fora chega pelo ENTORNO — o portão `o_rebote_chega_de_fora_da_grade`) e a cena 5 na
/// vista de fábrica (o miolo dos corpos), cada uma no Low e no High. Controlos:
/// `PH2D_LIGHT2D_ENTORNO=0` e `PH2D_LIGHT2D_MIOLO=0` têm de mudar o ficheiro.
#[test]
#[ignore = "instrumento: grava um ficheiro"]
fn sonda_grava_as_fluencias_w5o() {
    use ph2d_app_light2d::LuzDoQuadro;
    use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef};
    use ph2d_render::Sprite;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let Ok(saida) = std::env::var("PH2D_W5O_SAIDA") else {
        return;
    };
    let mut r = crate::comum::renderer(&gpu);
    let branca = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let mut bytes = Vec::new();
    for (n, centro, altura) in [(3, [2.0_f32, 0.0], 2.0_f32), (5, [0.0, 0.0], 0.0)] {
        let mut w = World::new();
        assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
        let mut present = PresentWorld::new();
        let mut q = w.query::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform)>();
        for (e, s, g) in q.iter(&w) {
            let [ax, ay, bx, by, tx, ty] = g.affine();
            let mut i = crate::comum::quad(branca, [tx, ty]);
            i.size = s.size;
            i.anchor = s.anchor;
            i.basis = [ax, ay, bx, by];
            present.world_mut().spawn((i, SimRef(e)));
        }
        let camera = if altura > 0.0 {
            ph2d_render::Camera2d::new(centro, altura)
        } else {
            ph2d_render::Camera2d::default()
        };
        let a = camera.world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
        for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
            let mut l = LuzDoQuadro::com_degrau(degrau_do(nivel));
            for _ in 0..80 {
                assert!(l.preparar(&w, a, FHD));
                l.gravar(&gpu, &mut present, &r);
            }
            let s = l.solver().expect("solver");
            let f = crate::comum::fluencia_lida(&gpu, s);
            eprintln!(
                "cena {n} · {nivel:?}: {} texels · entorno {}",
                f.v.len(),
                std::env::var("PH2D_LIGHT2D_ENTORNO").unwrap_or_default()
            );
            bytes.extend(f.v.iter().flatten().flat_map(|x| x.to_le_bytes()));
        }
    }
    std::fs::write(saida, bytes).expect("gravar");
}

/// ⭐ W5o — INSTRUMENTO da prova ao bit do V-ciclo na base: a caixa do portão
/// `o_interior_do_opaco` no zoom 2,5 (o corpo enche a vista: os níveis grossos têm erro livre), pela
/// configuração do produto, no Low e no High; grava as fluências em `$PH2D_W5O_SAIDA`, com a base no
/// grupo desligada por `PH2D_W5O_BASE=0`.
#[test]
#[ignore = "instrumento: grava um ficheiro"]
fn sonda_grava_a_caixa_w5o() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let Ok(saida) = std::env::var("PH2D_W5O_SAIDA") else {
        return;
    };
    let base = std::env::var("PH2D_W5O_BASE").map_or(true, |v| v.trim() != "0");
    let cena = crate::o_interior_do_opaco::cena([0.0; 2]);
    let centro = vello::kurbo::Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let a = fabrica(FHD).then_scale_about(2.5, centro);
    let mut bytes = Vec::new();
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        let produto = config_do_produto(a, FHD, degrau_do(nivel)).expect("grade");
        let cfg = Config {
            aritmetica: Config::padrao_para(&gpu.device, produto.grade).aritmetica,
            miolo: Some(Miolo {
                base_no_grupo: base,
                ..MIOLO_PADRAO
            }),
            ..produto
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        for _ in 0..40 {
            s.quadro(&gpu, &cena);
        }
        let fl = crate::comum::fluencia_lida(&gpu, &s);
        eprintln!("caixa 2,5 · {nivel:?} · base {base}: {} texels", fl.v.len());
        bytes.extend(fl.v.iter().flatten().flat_map(|x| x.to_le_bytes()));
    }
    std::fs::write(saida, bytes).expect("gravar");
}
