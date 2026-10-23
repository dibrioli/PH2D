//! **A régua da borda nos níveis de cima** (handoff W6g §41 a / §53 / §54.1 — os +3,62 ms da vista a 2160p Alta).
//! À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_borda_topo -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Cena 5, `[-4.2, -1.8]` h 3,8, a configuração do APP. Num processo: a vista do produto `P` com o céu
//! (`somar_entorno(None)`), com a borda e a borda outra vez (o controlo), e uma vista por forma de [`FORMAS`]
//! (`PH2D_BORDA_TOPO_FORMAS=a,b`; a 2160p Alta só cabe UMA ao lado de `P`). [`RODADAS`] em ordem rodada, o MÍNIMO
//! da `c2`; depois as fases por nível (nunca antes: a vida com fases muda o regime, W6g §51 c). ⚠️ A soma e as
//! subidas de `P` ficam no regime lento (a `identidade` não): compare-as só com a `identidade`; a descida não tem
//! regime (handoff W6h §13). `PH2D_BORDA_TOPO_REGISTOS=<pasta>` escreve os módulos para `spikes/regua-registos`;
//! `PH2D_BORDA_TOPO_SEM_CEU=1` tira o céu; `PH2D_BORDA_TOPO_ALVO=<l>x<a>` / `_NIVEL=low|medium|high`.

use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
use ph2d_app_light2d::{config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;

use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5v::{App, cena_do_smoke};

const RODADAS: usize = 9;
const AQUECE: usize = 3;
const QUADROS: usize = 6;
const RODADAS_FASES: usize = 4;

/// A leitura do arquivo no `de_fora`; o índice continua vivo (o `select` sobre ele).
const LEITURA: &str = "r = desempacotar(arquivo_dos_r[hrcs[iq()].fora_base[m / 4u][m % 4u] + i]).r;";
/// O fim do `de_fora` (a soma das duas sondas do arquivo).
const FIM: &str = "    return soma * arco(n, j);\n}";

/// As formas de diagnóstico do `de_fora` (`PH2D_BORDA_TOPO_FORMAS=a,b`): cada uma devolve outra luz.
const FORMAS: [(&str, &str); 6] = [
    // O controlo do regime: o módulo do produto, por `modulo_rc`, na 2.ª vista.
    ("identidade", ""),
    // F1: só o ponto `b` na borda; o resto do `de_fora` sai do módulo.
    ("so_b", "    // b no canónico do entorno"),
    // F2: mais o nível, a coluna e o recuo.
    ("ate_coluna", "    let vf = ce.y - recuo * passo.y - 0.5;"),
    // F3: tudo menos a leitura do arquivo.
    ("sem_ler", ""),
    // O índice do arquivo por um escalar do uniforme (a luz errada; mede só a forma).
    ("base_escalar", "hrcs[iq()].fora_base[m / 4u][m % 4u]"),
    // F4: a soma (o `r0_na_sonda`) sem o `de_fora`; a descida igual ao produto.
    ("soma_sem_fora", ""),
];

/// Como a vista do app acima de 3 GiB (`quadro_renasce`, as páginas novas): nasce, solta-se e renasce nos
/// blocos largados — o regime rápido do driver, o mesmo para o produto e cada forma.
fn renascido(gpu: &ph2d_gpu::GpuContext, c: Config, nome: &str) -> Solver {
    let s = Solver::novo(gpu, c.clone()).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
    drop(s);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    Solver::novo(gpu, c).unwrap_or_else(|e| panic!("{nome}: {e:?}"))
}

/// O módulo do produto com a forma `nome` (`Config::modulo_rc`).
fn forma(nome: &str, c: Config) -> Config {
    let mut s =
        ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento, c.hrc_niveis_em_textura())
            .to_string();
    let corta = |s: &mut String, de: &str, por: &str| {
        assert_eq!(s.matches(de).count(), 1, "a agulha: {de}");
        let i = s.find(de).expect("de");
        let f = i + s[i..].find(FIM).expect("o fim do de_fora") + FIM.len();
        s.replace_range(i..f, por);
    };
    match nome {
        "identidade" => {}
        "so_b" => corta(
            &mut s,
            FORMAS[1].1,
            "    return cas.ceu.rgb * arco(n, j) * (1.0 + 1e-30 * (b.x + b.y));\n}",
        ),
        "ate_coluna" => corta(
            &mut s,
            FORMAS[2].1,
            "    return ceu * arco(n, j) * (1.0 + 1e-30 * (recuo + passo.y + f32(pai)));\n}",
        ),
        "sem_ler" => {
            assert_eq!(s.matches(LEITURA).count(), 1, "a agulha da leitura");
            s = s.replacen(LEITURA, "r = select(ceu, 0.5 * ceu, i == 7u);", 1);
        }
        "base_escalar" => {
            let de = FORMAS[4].1;
            assert_eq!(s.matches(de).count(), 1, "a agulha da base");
            s = s.replacen(de, "hrcs[iq()].fora_base[0][0]", 1);
        }
        "soma_sem_fora" => {
            let de = "    return de_fora(u, i32(v), 0u, 0u);\n}";
            assert_eq!(s.matches(de).count(), 1, "a agulha da soma");
            s = s.replacen(de, "    return cas.ceu.rgb * arco(0u, 0u);\n}", 1);
        }
        n => panic!("forma {n}: não há"),
    }
    Config {
        modulo_rc: Some(Box::leak(s.into_boxed_str())),
        ..c
    }
}

#[test]
#[ignore = "medição de placa: a borda nos níveis de cima (W6g §54.1)"]
fn medida_luz2d_borda_topo() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = std::env::var("PH2D_BORDA_TOPO_ALVO").map_or((3840, 2160), |v| {
        let (l, a) = v.split_once('x').expect("<l>x<a>");
        (l.parse().expect("largura"), a.parse().expect("altura"))
    });
    let nivel = match std::env::var("PH2D_BORDA_TOPO_NIVEL").as_deref() {
        Ok("low") => NivelDaLuz::Baixa,
        Ok("medium") => NivelDaLuz::Media,
        _ => NivelDaLuz::Alta,
    };
    let a = Camera2d::new([-4.2, -1.8], 3.8)
        .world_to_screen_affine(ph2d_host::WindowSize::new(alvo.0, alvo.1));
    eprintln!(
        "P placa {:?} · lado máx {}",
        gpu.adapter.get_info().name,
        gpu.device.limits().max_texture_dimension_2d
    );
    let cena = cena_do_smoke(5);
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..cena.clone()
    };
    // Como o app: a fina engrossa até caber no lado da placa (a 2160p, `k ≥ 1`).
    let tetos = Tetos {
        lado: gpu.device.limits().max_texture_dimension_2d,
        recusou: None,
    };
    let k = fina_que_cabe(a, alvo, degrau_do(nivel), tetos).expect("nada cabe na placa");
    let dg = degrau_do(nivel).com_fina_engrossada(k);
    let mut app = {
        let g = grade_do_quadro(a, alvo, dg).expect("a grade");
        // A ordem do app: a vista nasce, o entorno monta-se, e a vista renasce com ele vivo (com a vista a renascer
        // ANTES do entorno, o acumulado dela ficava no regime lento: a soma 12,6 contra 5,6 ms, §13).
        let cv = config_da_vista(&gpu.device, g, a, dg);
        let mut vista = Solver::novo(&gpu, cv.clone()).expect("solver da vista");
        let (i, f) = regiao_da_janela(vista.grade_da_fluencia(), a, alvo);
        vista.regiao_fina(i, f);
        let mut entorno = config_do_entorno(&gpu.device, a, alvo, dg)
            .map(|c| Solver::novo(&gpu, c).expect("solver do entorno"));
        if let Some(e) = entorno.as_mut() {
            let g = vista.config().grade;
            e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ]),
            );
        }
        let (i, f) = regiao_da_janela(vista.grade_da_fluencia(), a, alvo);
        drop(vista);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let mut vista = Solver::novo(&gpu, cv).expect("solver da vista");
        vista.regiao_fina(i, f);
        App { vista, entorno }
    };
    let entorno = app.entorno.as_ref().expect("o entorno (com o arquivo da borda)");
    assert!(
        entorno.config().arquivo_da_borda.is_some(),
        "o entorno sem o arquivo: não há borda"
    );
    let pedidas = std::env::var("PH2D_BORDA_TOPO_FORMAS").unwrap_or_else(|_| "sem_ler".into());
    let formas: Vec<&'static str> = pedidas
        .split(',')
        .map(|n| {
            FORMAS
                .iter()
                .find(|f| f.0 == n)
                .unwrap_or_else(|| panic!("forma {n}: não há"))
                .0
        })
        .collect();
    let mut solvers: Vec<Solver> = formas
        .iter()
        .map(|nome| {
            let mut s = renascido(&gpu, forma(nome, app.vista.config().clone()), nome);
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
            s.regiao_fina(i, f);
            assert!(s.somar_entorno(&gpu, Some(entorno)));
            s
        })
        .collect();
    // As variantes: (nome, o solver — `None` = a vista do produto —, a borda ligada).
    let mut vars: Vec<(&'static str, Option<usize>, bool)> =
        vec![("céu", None, false), ("borda", None, true), ("controlo", None, true)];
    vars.extend(formas.iter().enumerate().map(|(k, n)| (*n, Some(k), true)));
    // `PH2D_BORDA_TOPO_SEM_CEU=1`: a vista do produto nunca troca de estado (o controlo da troca, §13).
    if std::env::var("PH2D_BORDA_TOPO_SEM_CEU").as_deref() == Ok("1") {
        vars.retain(|v| v.2);
    }
    let c = app.vista.config();
    let rotulo = format!("{}×{} {nivel:?} fina k {k}", alvo.0, alvo.1);
    eprintln!(
        "O {rotulo} · grade {}×{} · formas {formas:?} · load {}",
        c.grade.largura,
        c.grade.altura,
        loadavg().trim()
    );
    if let Ok(pasta) = std::env::var("PH2D_BORDA_TOPO_REGISTOS") {
        registos(&pasta, &app.vista, &formas);
    }

    // Um quadro da variante `v`: o entorno, depois a vista dela com o estado da borda certo.
    let mut estado_p: Option<bool> = None;
    let mut quadro = |app: &mut App, solvers: &mut [Solver], v: (&str, Option<usize>, bool)| {
        let e = app.entorno.as_mut().expect("entorno");
        e.quadro(&gpu, &sem_ceu);
        let vista = match v.1 {
            Some(k) => &mut solvers[k],
            None => {
                if estado_p != Some(v.2) {
                    assert!(app.vista.somar_entorno(&gpu, v.2.then_some(&*e)));
                    estado_p = Some(v.2);
                }
                &mut app.vista
            }
        };
        vista.quadro(&gpu, &cena);
        vista.tempos(&gpu).map_or(0.0, |t| t.c2_total())
    };

    let nv = vars.len();
    let mut c2: Vec<Vec<f64>> = vec![Vec::new(); nv];
    for r in 0..RODADAS {
        for k in 0..nv {
            let iv = (k + r) % nv;
            for _ in 0..AQUECE {
                quadro(&mut app, &mut solvers, vars[iv]);
            }
            for _ in 0..QUADROS {
                let t = quadro(&mut app, &mut solvers, vars[iv]);
                c2[iv].push(t);
            }
        }
        eprintln!("R rodada {r} · load {}", loadavg().trim());
    }
    let min: Vec<f64> = c2.iter().map(|v| minimo(v)).collect();
    for (iv, v) in vars.iter().enumerate() {
        eprintln!(
            "C {rotulo} · {} · c2 mín {:.3} · med {:.3} · − céu {:+.3} · − borda {:+.3}",
            v.0,
            min[iv],
            mediana(&c2[iv]),
            min[iv] - min[0],
            min[iv] - min[1]
        );
    }

    // As fases por nível (o modo de fases liga em todas; a `c2` dos tempos fica 0).
    assert!(app.vista.medir_fases_da_hrc(&gpu));
    for s in &mut solvers {
        assert!(s.medir_fases_da_hrc(&gpu));
    }
    type PorNivel = Vec<(&'static str, u32, Vec<f64>)>;
    let mut fases: Vec<PorNivel> = vec![Vec::new(); nv];
    for r in 0..RODADAS_FASES {
        for k in 0..nv {
            let iv = (k + r) % nv;
            for q in 0..AQUECE + QUADROS {
                quadro(&mut app, &mut solvers, vars[iv]);
                if q < AQUECE {
                    continue;
                }
                let s = vars[iv].1.map_or(&app.vista, |k| &solvers[k]);
                for (etapa, n, ms, _) in s.fases_da_hrc(&gpu).expect("fases") {
                    match fases[iv].iter_mut().find(|o| o.0 == etapa && o.1 == n) {
                        Some(o) => o.2.push(ms),
                        None => fases[iv].push((etapa, n, vec![ms])),
                    }
                }
            }
        }
    }
    let ler = |iv: usize, etapa: &str, n: u32| {
        fases[iv]
            .iter()
            .find(|o| o.0 == etapa && o.1 == n)
            .map_or(0.0, |o| minimo(&o.2))
    };
    let mut chaves: Vec<(&'static str, u32)> = fases[0].iter().map(|o| (o.0, o.1)).collect();
    chaves.sort_by_key(|k| (k.0, k.1));
    let nomes: Vec<&str> = vars.iter().map(|v| v.0).collect();
    eprintln!("F {rotulo} · colunas: {}", nomes.join(" · "));
    let mut soma = vec![0.0_f64; nv];
    for (etapa, n) in chaves {
        let v: Vec<f64> = (0..nv).map(|iv| ler(iv, etapa, n)).collect();
        for (s, x) in soma.iter_mut().zip(&v) {
            *s += x;
        }
        let todas = matches!(etapa, "hrc_desce" | "hrc_soma");
        if !todas && v.iter().all(|x| (x - v[1]).abs() < 0.03) {
            continue;
        }
        let l: Vec<String> = v.iter().map(|x| format!("{x:.3}")).collect();
        eprintln!("F {rotulo} · {etapa} n {n:>2} · {}", l.join(" · "));
    }
    let l: Vec<String> = soma.iter().map(|x| format!("{x:.3}")).collect();
    eprintln!(
        "F {rotulo} · soma das fases · {} · load {}",
        l.join(" · "),
        loadavg().trim()
    );
}

/// Para a régua de registos (`spikes/regua-registos <pasta>`): o módulo de cada variante e as constantes da
/// holográfica do produto, nas entradas da descida e da soma.
fn registos(pasta: &str, vista: &Solver, formas: &[&'static str]) {
    let pasta = std::path::PathBuf::from(pasta);
    std::fs::create_dir_all(&pasta).expect("a pasta");
    let c = vista.config();
    let constantes = vista.constantes_da_hrc().expect("a holográfica");
    let produto = ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento, c.hrc_niveis_em_textura());
    let mut modulos = vec![("produto", produto.to_string())];
    modulos.extend(
        formas
            .iter()
            .map(|n| (*n, forma(n, c.clone()).modulo_rc.expect("a forma").to_string())),
    );
    for (nome, fonte) in modulos {
        std::fs::write(pasta.join(format!("{nome}.wgsl")), fonte).expect("o módulo");
        for entrada in ["hrc_desce", "hrc_soma"] {
            let mut l = vec![format!("modulo {nome}.wgsl"), format!("entrada {entrada}")];
            l.extend(constantes.iter().map(|(k, x)| format!("{k} {x}")));
            std::fs::write(pasta.join(format!("{nome}_{entrada}.txt")), l.join("\n") + "\n")
                .expect("as constantes");
        }
    }
    eprintln!("REGUA · {} · {} constantes", pasta.display(), constantes.len());
}
