//! Os portões do compositor da óptica (J13b P5): a thread REAL, sem relógio fixo, contra [`compoe_a_otica`] no fio
//! do teste sobre o MESMO estado do motor; e o ciclo de vida (nenhuma thread sobrevive à sessão).

use super::super::super::otica_factos::{FactosDaOtica, InterruptoresDaOtica};
use super::super::SimWorker;
use super::*;
use ph2d_wet_paint::render::{PigmentVisual, RenderLayer};
use std::sync::atomic::Ordering;
use std::time::Duration;

/// 256²: o alcance do óleo já não cobre a folha — a caixa do compositor sai MENOR que a do fluido (B14.1).
const N: usize = 256;
const FORMA: FormaDaTela = FormaDaTela {
    ratio: 1,
    grade: (N, N),
    tela: (N, N),
};

fn factos() -> FactosDaOtica {
    FactosDaOtica {
        visual: PigmentVisual {
            paper: false,
            relevo: true,
        },
        oleo_adiado: false,
        oleo: OleoReal::default(),
        tem_grao: true,
        teste: InterruptoresDaOtica {
            curas_desligadas: (false, false),
            sem_cache: false,
            papel_liso_desligado: false,
            lei_da_caixa: None,
            so_na_cor: false,
        },
    }
}

/// Água e Real Oil, e o traço ABERTO: a água não anda (`sim_should_run` falso) — o estado fica fixo.
fn cena() -> Box<Engine> {
    let mut e = Engine::new(N, N);
    e.pointer_down(18.0, 20.0, None);
    for k in 0..24 {
        e.pointer_frame(18.0 + k as f64, 22.0 + (k % 5) as f64);
    }
    e.pointer_up();
    for _ in 0..8 {
        while e.step_stage().is_none() {}
    }
    e.layers[0].grid.oleo_real = true;
    e.pointer_down(40.0, 40.0, None);
    assert!(!e.sim_should_run(), "a fixture precisa da agua parada");
    let g = &e.layers[0].grid;
    assert!(
        g.susp.iter().chain(&g.sett).any(|&m| m > 0.0) && g.film.iter().any(|&f| f > 0.0),
        "a fixture nao tem tinta molhada — nao contem o fenomeno"
    );
    Box::new(e)
}

/// A óptica no fio do teste sobre uma cópia do motor (o sujo + a dívida, como o worker os junta).
fn sincrona(e: &Engine, divida: Option<CaixaCel>, planos: &mut PlanosDaOtica) -> Option<CaixaCel> {
    let mut c = e.clone();
    if let Some((x0, y0, x1, y1)) = divida {
        c.merge_dirty_otica(x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    }
    let caixa = caixa_da_grade(c.take_dirty(), FORMA.grade);
    let layers: Vec<RenderLayer<'_>> = c
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let entrada = EntradaDaOtica {
        factos: factos(),
        marca: MarcaDaAgua::de(&c),
        trabalho: caixa.map(|caixa| TrabalhoDaOtica {
            caixa,
            params: c.sim.gather_params(&c.tuning),
        }),
        ratio: FORMA.ratio,
        grade: FORMA.grade,
        tela: FORMA.tela,
        revisao: c.revisao(),
    };
    planos.campo_cache = None;
    let s = compoe_a_otica(&entrada, &layers, planos);
    assert!(s.oleo.is_some(), "a fixture nao corre o Real Oil");
    s.caixas.map(|c| c.caixa_cel)
}

fn espera(w: &SimWorker) -> RespostaDaOtica {
    let o = w.otica.as_ref().expect("o compositor existe");
    let t0 = Instant::now();
    loop {
        if let Some(r) = o.try_recv() {
            return r;
        }
        assert!(
            t0.elapsed() < Duration::from_secs(10),
            "o compositor nao respondeu ao pedido"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn entrega(w: &SimWorker, e: Box<Engine>) {
    w.want.store(false, Ordering::Release);
    w.to_worker.send(e).expect("o worker aceita o motor");
}

fn traz(w: &SimWorker) -> Box<Engine> {
    w.want.store(true, Ordering::Release);
    w.back
        .recv_timeout(Duration::from_secs(10))
        .expect("o worker devolve o motor")
}

fn sobe() -> SimWorker {
    forca_no_teste(Some(true));
    let w = SimWorker::spawn(None);
    forca_no_teste(None);
    w
}

/// **A imagem do compositor, aplicada, é a óptica síncrona sobre o mesmo motor — ao byte.** Cinco rondas na
/// thread real: o sujo inteiro, um dab (a cópia INCREMENTAL do espelho), só a dívida, a água a correr (B14.1: o
/// compositor recompõe só as células que o passo MUDOU, a síncrona a caixa do fluido inteira — os planos ao byte) e
/// o motor limpo («nada»).
#[test]
fn a_imagem_do_compositor_e_a_otica_sincrona_ao_byte() {
    let w = sobe();
    let mut e = cena();
    let (mut controlo, mut referencia) = (
        PlanosDaOtica::da_grade(FORMA.grade),
        PlanosDaOtica::da_grade(FORMA.grade),
    );
    type Prepara = fn(&mut Engine);
    let rondas: [(&str, Prepara, Option<CaixaCel>); 5] = [
        ("sujo inteiro", Engine::mark_dirty_full, None),
        (
            "um dab",
            |e| {
                for k in 1..=8 {
                    e.pointer_frame(40.0 + k as f64, 40.0 + 0.5 * k as f64);
                }
            },
            None,
        ),
        ("so a divida", |_| {}, Some((10, 12, 30, 28))),
        (
            "a agua corre",
            |e| {
                e.pointer_up();
                for _ in 0..3 {
                    while e.step_stage().is_none() {}
                }
                e.pointer_down(40.0, 40.0, None);
            },
            None,
        ),
        ("limpo", |_| {}, None),
    ];
    let mut imagens = 0;
    for (k, (nome, prepara, divida)) in rondas.into_iter().enumerate() {
        prepara(&mut e);
        let esperada = sincrona(&e, divida, &mut referencia);
        entrega(&w, e);
        let epoca = 100 + k as u64;
        let antigo = w.otica.as_ref().unwrap().pede(PedidoDeImagem {
            factos: factos(),
            epoca,
            divida,
            forma: FORMA,
            com_veu: true,
        });
        assert!(antigo.is_none(), "{nome}: um pedido velho ficou na caixa");
        let resposta = espera(&w);
        e = traz(&w);
        match (resposta, esperada) {
            (RespostaDaOtica::Nada { epoca: ep, .. }, None) => assert_eq!(ep, epoca, "{nome}"),
            (RespostaDaOtica::Imagem(img), Some(caixa)) => {
                assert_eq!(img.epoca, epoca, "{nome}");
                assert!(img.oleo.is_some(), "{nome}: sem Real Oil");
                let c = img.caixas.first().expect("caixas");
                let env = envolvente(&img);
                if nome == "a agua corre" {
                    let (a, b) = (env, caixa);
                    assert!(
                        a.0 >= b.0 && a.1 >= b.1 && a.2 <= b.2 && a.3 <= b.3 && a != b,
                        "{nome}: a caixa {a:?} nao e menor que a do fluido {b:?}"
                    );
                } else {
                    assert_eq!(env, caixa, "{nome}: a caixa composta");
                }
                assert!(
                    c.sombra.is_some() && c.veu.is_some(),
                    "{nome}: sombra e veu"
                );
                img.aplica(&mut controlo, FORMA.grade);
                imagens += 1;
            }
            (_, esperada) => panic!("{nome}: resposta errada (esperava caixa {esperada:?})"),
        }
        assert!(
            controlo.pigment == referencia.pigment,
            "{nome}: o pigmento difere"
        );
        assert!(
            controlo.sombra == referencia.sombra,
            "{nome}: a sombra difere"
        );
        assert!(controlo.grao == referencia.grao, "{nome}: o grao difere");
    }
    assert_eq!(imagens, 4, "quatro imagens e um «nada»");
    assert!(
        referencia.pigment.iter().any(|&b| b > 0) && referencia.sombra.iter().any(|&b| b > 0),
        "a comparacao foi sobre planos vazios"
    );
}

fn envolvente(img: &Imagem) -> CaixaCel {
    img.caixas
        .iter()
        .map(|c| c.caixa_cel)
        .reduce(|a, b| super::super::super::otica_no_tique::une(Some(a), b))
        .expect("caixas")
}

/// Duas pinceladas molhadas longe uma da outra (o topo e o fundo da folha) e o óleo; o traço aberto junto à de cima.
fn cena_de_duas_faixas() -> Box<Engine> {
    let mut e = Engine::new(N, N);
    for y in [24.0, 226.0] {
        e.pointer_down(116.0, y, None);
        for k in 0..24 {
            e.pointer_frame(116.0 + k as f64, y + (k % 5) as f64);
        }
        e.pointer_up();
    }
    e.layers[0].grid.oleo_real = true;
    e.pointer_down(128.0, 26.0, None);
    assert!(!e.sim_should_run(), "a fixture precisa da agua parada");
    Box::new(e)
}

/// **B14.1: duas faixas vivas, duas caixas** — a água corre nas duas pinceladas e as linhas entre elas não mudam: o
/// compositor compõe uma caixa por faixa (a síncrona, a envolvente das duas), e os planos ficam ao byte da síncrona.
#[test]
fn duas_faixas_vivas_compoem_duas_caixas_ao_byte_da_sincrona() {
    let w = sobe();
    let mut e = cena_de_duas_faixas();
    let (mut controlo, mut referencia) = (
        PlanosDaOtica::da_grade(FORMA.grade),
        PlanosDaOtica::da_grade(FORMA.grade),
    );
    type Prepara = fn(&mut Engine);
    let rondas: [(&str, Prepara, usize); 3] = [
        ("sujo inteiro", Engine::mark_dirty_full, 1),
        (
            "a agua corre nas duas",
            |e| {
                e.pointer_up();
                for _ in 0..3 {
                    while e.step_stage().is_none() {}
                }
                e.pointer_down(128.0, 26.0, None);
            },
            2,
        ),
        (
            "e continua a correr",
            |e| {
                e.pointer_up();
                for _ in 0..2 {
                    while e.step_stage().is_none() {}
                }
                e.pointer_down(128.0, 26.0, None);
            },
            2,
        ),
    ];
    for (k, (nome, prepara, quantas)) in rondas.into_iter().enumerate() {
        prepara(&mut e);
        let esperada = sincrona(&e, None, &mut referencia).expect("a sincrona compoe");
        entrega(&w, e);
        let epoca = 200 + k as u64;
        w.otica.as_ref().unwrap().pede(PedidoDeImagem {
            factos: factos(),
            epoca,
            divida: None,
            forma: FORMA,
            com_veu: true,
        });
        let resposta = espera(&w);
        e = traz(&w);
        let RespostaDaOtica::Imagem(img) = resposta else {
            panic!("{nome}: o compositor nao compos");
        };
        let caixas: Vec<CaixaCel> = img.caixas.iter().map(|c| c.caixa_cel).collect();
        eprintln!("{nome}: sincrona {esperada:?} -> compositor {caixas:?}");
        assert_eq!(
            caixas.len(),
            quantas,
            "{nome}: {caixas:?} (a sincrona {esperada:?})"
        );
        let area = |c: CaixaCel| (c.2 + 1 - c.0) * (c.3 + 1 - c.1);
        let (nossa, dela) = (caixas.iter().map(|&c| area(c)).sum::<usize>(), area(esperada));
        eprintln!("{nome}: {dela} -> {nossa} celulas compostas");
        if quantas == 2 {
            assert!(nossa < dela, "{nome}: as faixas custam {nossa} >= {dela}");
            assert!(
                caixas[0].3 < caixas[1].1,
                "{nome}: as faixas sobrepoem-se {caixas:?}"
            );
        }
        img.aplica(&mut controlo, FORMA.grade);
        assert!(controlo.pigment == referencia.pigment, "{nome}: o pigmento difere");
        assert!(controlo.sombra == referencia.sombra, "{nome}: a sombra difere");
        assert!(controlo.grao == referencia.grao, "{nome}: o grao difere");
        assert!(controlo.suave == referencia.suave, "{nome}: o suave difere");
    }
}

/// **O compositor acaba com a sessão** — o mesmo `Drop` que fecha o worker (com o motor COM ele) fecha-o.
#[test]
fn a_finished_session_does_not_leave_the_compositor_running() {
    let mut w = sobe();
    let fio = w
        .otica
        .as_mut()
        .and_then(|o| o.fio.take())
        .expect("o compositor subiu");
    let mut e = Engine::new(32, 32);
    e.sim.gravity_override = Some([0.0, 1.0]);
    entrega(&w, Box::new(e));
    let want = Arc::clone(&w.want);
    std::thread::sleep(Duration::from_millis(30));
    assert!(!fio.is_finished(), "o compositor devia estar vivo");
    drop(w);
    let t0 = Instant::now();
    while !(fio.is_finished() && Arc::strong_count(&want) == 1)
        && t0.elapsed() < Duration::from_millis(500)
    {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(Arc::strong_count(&want), 1, "o worker sobreviveu a sessao");
    assert!(
        fio.is_finished(),
        "o compositor sobreviveu a sessao: segura um espelho orfao"
    );
}

/// O controlo: sem o interruptor não há compositor (o caminho de hoje, e o ritmado nunca o tem).
#[test]
fn sem_o_interruptor_nao_ha_compositor() {
    forca_no_teste(Some(false));
    let w = SimWorker::spawn(None);
    forca_no_teste(Some(true));
    let r = SimWorker::spawn(Some(&super::super::RitmoDaAgua {
        estagios: vec![1],
        nao_volta: vec![false],
    }));
    forca_no_teste(None);
    assert!(w.otica.is_none());
    assert!(r.otica.is_none(), "o worker ritmado nunca tem compositor");
}
