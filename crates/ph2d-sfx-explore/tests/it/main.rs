//! A lei da exploração, sem tela.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_audio::dsp::maior_momento;
use ph2d_sfx_dsp::{TETO_DA_ESCUTA, render};
use ph2d_sfx_explore::lei::{LEIS, lei_de};
use ph2d_sfx_explore::{
    Exploracao, FILHOS, MAX_OUVIDOS, Ninhada, Sessao, TAMANHO_MUTATE, TAMANHO_ROLL, varia,
};
use ph2d_sfx_lab::medida::{ALVO_DA_ESCUTA, VOLUME_MUDO, escuta_nivelada};
use ph2d_sfx_lab::tabela::{CAMPOS, Onde};
use ph2d_sfx_recipe::{
    Body, Como, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, SAMPLE_RATE, Source, Takes, factory,
};

fn fabrica(nome: &str) -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == nome)
        .map(|(_, r)| r)
        .expect("som de fábrica")
}

fn pcm(r: &Recipe) -> Vec<f32> {
    render(r, r.seed, r.engine, &Takes::default())
        .expect("render")
        .samples
}

fn sem_nada() -> Arc<Takes> {
    Arc::new(Takes::default())
}

/// Ruído travado ao lado de um corpo livre: o ruído depende da semente, e é aí que uma trava de mentira se denuncia.
fn ruido_travado() -> Recipe {
    let mut r = Recipe::new(77, 0.5);
    let ruido = r.add(
        0.5,
        Source::Noise(Noise {
            filter: NoiseFilter::White,
            hz: 800.0,
            hz_end: 800.0,
            q: 1.0,
        }),
        Envelope::default(),
    );
    r.add(
        0.5,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 300.0,
            modes: 8,
            t60: 0.3,
            damping: 1.0,
            rolloff: 1.0,
            contact_ms: 1.0,
            noise: 0.1,
        }),
        Envelope::default(),
    );
    r.layer_mut(ruido).unwrap().travada = true;
    r
}

fn so_a_camada(r: &Recipe, i: usize) -> Vec<f32> {
    let id = r.layers[i].id;
    pcm(&r.solo(id))
}

#[test]
fn fixado_nao_muda_em_mil_sorteios() {
    let base = ruido_travado();
    let travada = so_a_camada(&base, 0);
    let livre = so_a_camada(&base, 1);
    let mut mudou_a_livre = 0;
    for n in 1..=1000 {
        let v = varia(&base, n, TAMANHO_ROLL);
        assert_eq!(v.layers[0], base.layers[0], "sorteio {n} mexeu nos campos");
        assert_eq!(
            v.seed, base.seed,
            "o sorteio não pode mudar a semente da receita"
        );
        if n <= 50 {
            assert_eq!(
                so_a_camada(&v, 0),
                travada,
                "sorteio {n} mudou o ruído da camada travada"
            );
            if so_a_camada(&v, 1) != livre {
                mudou_a_livre += 1;
            }
        }
    }
    assert_eq!(mudou_a_livre, 50, "controle: a camada livre tem de mudar");
}

#[test]
fn o_campo_travado_nao_muda_e_nao_desloca_o_sorteio_dos_outros() {
    let base = fabrica("Metal Hit");
    let hz = CAMPOS
        .iter()
        .find(|c| c.chave.contains("base"))
        .expect("campo da base");
    let solto = varia(&base, 5, TAMANHO_MUTATE);
    let mut com_trava = base.clone();
    com_trava.layers[0].travas.push(hz.id.0);
    let preso = varia(&com_trava, 5, TAMANHO_MUTATE);
    let (b, s, p) = (&base.layers[0], &solto.layers[0], &preso.layers[0]);
    assert_eq!(hz.le(&preso, Some(p)), hz.le(&base, Some(b)));
    assert_ne!(hz.le(&solto, Some(s)), hz.le(&base, Some(b)), "controle");
    for c in CAMPOS
        .iter()
        .filter(|c| c.id != hz.id && matches!(c.onde, Onde::Camada { .. }))
    {
        assert_eq!(c.le(&preso, Some(p)), c.le(&solto, Some(s)), "{}", c.chave);
    }
}

#[test]
fn todo_campo_de_camada_tem_lei() {
    for c in CAMPOS {
        let e_de_camada = matches!(c.onde, Onde::Camada { .. });
        assert_eq!(lei_de(c.id).is_some(), e_de_camada, "{}", c.chave);
    }
    let de_camada = CAMPOS
        .iter()
        .filter(|c| matches!(c.onde, Onde::Camada { .. }))
        .count();
    assert_eq!(LEIS.len(), de_camada, "lei de campo que não existe");
}

#[test]
fn a_mesma_variacao_da_o_mesmo_som_e_outra_da_outro() {
    let base = fabrica("Impact");
    assert_eq!(
        pcm(&varia(&base, 9, TAMANHO_ROLL)),
        pcm(&varia(&base, 9, TAMANHO_ROLL))
    );
    assert_ne!(
        pcm(&varia(&base, 9, TAMANHO_ROLL)),
        pcm(&varia(&base, 10, TAMANHO_ROLL))
    );
}

#[test]
fn refazer_uma_nao_muda_as_outras() {
    let mut x = Exploracao::default();
    let takes = sem_nada();
    x.semeia(&fabrica("Metal Hit"), 0.3, &takes);
    espera_todos(&mut x, &takes);
    let antes: Vec<_> = (0..FILHOS).map(|i| filho(&x, i)).collect();
    x.redimensiona(3, 0.9, &takes);
    espera_todos(&mut x, &takes);
    for (i, de_antes) in antes.iter().enumerate() {
        if i == 3 {
            assert_ne!(&filho(&x, i), de_antes, "o arrastado tem de mudar");
        } else {
            assert_eq!(&filho(&x, i), de_antes, "filho {i}");
        }
    }
}

fn filho(x: &Exploracao, i: usize) -> Vec<f32> {
    x.grade.as_ref().unwrap().filhos[i]
        .as_ref()
        .expect("filho")
        .arquivo
        .clone()
}

fn espera_todos(x: &mut Exploracao, takes: &Arc<Takes>) {
    let fim = Instant::now() + Duration::from_secs(120);
    loop {
        let base = x.grade.as_ref().unwrap().base.clone();
        let _ = x.serve(&base, false, takes);
        let g = x.grade.as_ref().unwrap();
        if g.filhos.iter().all(Option::is_some) && !x.em_voo() {
            let _ = x.serve(&base, false, takes);
            return;
        }
        assert!(Instant::now() < fim, "a ninhada não chegou");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn arrastar_de_volta_devolve_a_base() {
    let base = fabrica("Impact");
    let longe = varia(&base, 3, 0.7);
    assert_ne!(pcm(&longe), pcm(&base), "controle");
    assert_eq!(varia(&base, 3, 0.0), base);
    let takes = sem_nada();
    let mut x = Exploracao::default();
    x.semeia(&base, 0.7, &takes);
    espera_todos(&mut x, &takes);
    x.redimensiona(2, 0.0, &takes);
    x.redimensiona(5, 1.4, &takes);
    x.redimensiona(5, 0.7, &takes);
    espera_todos(&mut x, &takes);
    assert_eq!(filho(&x, 2), pcm(&base));
    assert_eq!(
        filho(&x, 5),
        pcm(&varia(&base, numero_do_filho(&x, 5), 0.7))
    );
}

fn numero_do_filho(x: &Exploracao, i: usize) -> u64 {
    match x.grade.as_ref().unwrap().como(i) {
        Como::Variacao(n) => n,
        c => panic!("{c:?}"),
    }
}

#[test]
fn sorteio_nunca_e_mudo_nem_quebra_o_motor() {
    let (mut pior, mut mudos) = (f32::MAX, 0);
    for (nome, base) in factory::all() {
        for n in 1..=250u64 {
            for tamanho in [TAMANHO_MUTATE, TAMANHO_ROLL] {
                let v = varia(&base, n, tamanho);
                assert_eq!(v.seconds, base.seconds, "{nome}: a duração não se sorteia");
                let s = pcm(&v);
                assert!(s.iter().all(|x| x.is_finite()), "{nome} {n}");
                let vol = maior_momento(&s, SAMPLE_RATE);
                pior = pior.min(vol);
                if vol < VOLUME_MUDO {
                    mudos += 1;
                }
            }
        }
    }
    assert_eq!(mudos, 0, "o mais baixo: {pior} LUFS");
}

#[test]
fn filhos_tocam_no_mesmo_volume() {
    let base = fabrica("Impact");
    let mut sem_nivelar = Vec::new();
    let mut nivelados = Vec::new();
    for n in 1..=FILHOS as u64 {
        let s = pcm(&varia(&base, n, TAMANHO_ROLL));
        sem_nivelar.push(maior_momento(&s, SAMPLE_RATE));
        nivelados.push(maior_momento(
            &escuta_nivelada(&s, SAMPLE_RATE),
            SAMPLE_RATE,
        ));
    }
    let abre = |v: &[f32]| {
        v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min)
    };
    assert!(
        abre(&sem_nivelar) > 3.0,
        "a fixture não contém o fenômeno: {sem_nivelar:?}"
    );
    assert!(abre(&nivelados) < 0.5, "{nivelados:?}");
    for v in nivelados {
        assert!((v - ALVO_DA_ESCUTA).abs() < 0.5, "{v}");
    }
}

#[test]
fn nivelar_nao_passa_do_teto_nem_muda_o_arquivo() {
    // Pico alto com volume baixo: um estalo. Levado ao alvo, passaria do teto sem o limitador DEPOIS do ganho.
    let mut estalo = vec![0.0f32; 48_000];
    estalo[100] = 0.9;
    estalo[101] = -0.9;
    let copia = estalo.clone();
    let ouvido = escuta_nivelada(&estalo, SAMPLE_RATE);
    assert_eq!(estalo, copia, "o arquivo não muda");
    let pico = ouvido.iter().fold(0.0f32, |a, b| a.max(b.abs()));
    assert!(pico <= TETO_DA_ESCUTA, "{pico}");
    assert!(
        maior_momento(&estalo, SAMPLE_RATE) < ALVO_DA_ESCUTA,
        "a fixture tem de pedir ganho"
    );
    // Silêncio não sobe até o alvo.
    let piso = vec![1.0e-6f32; 48_000];
    assert_eq!(escuta_nivelada(&piso, SAMPLE_RATE), piso);
}

#[test]
fn render_velho_da_ninhada_nunca_entra() {
    let takes = sem_nada();
    let mut lento = Recipe::new(1, 120.0);
    lento.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 20.0,
            modes: 500,
            t60: 60.0,
            damping: 0.0,
            rolloff: 0.0,
            contact_ms: 1.0,
            noise: 0.0,
        }),
        Envelope::default(),
    );
    let rapido = fabrica("Wood Knock");
    let mut n = Ninhada::default();
    n.pede(0, lento, takes.clone());
    n.pede(0, rapido.clone(), takes);
    let fim = Instant::now() + Duration::from_secs(60);
    let f = loop {
        if let Some((0, r)) = n.recebe().into_iter().next() {
            break r.expect("render");
        }
        assert!(Instant::now() < fim, "o novo não chegou");
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(f.receita, rapido);
    assert_eq!(f.arquivo, pcm(&rapido));
    assert!(n.recebe().is_empty(), "o velho não pode passar");
}

#[test]
fn roll_parte_do_ponto_de_partida_e_mutate_anda() {
    let base = fabrica("Metal Hit");
    let mut s = Sessao::default();
    let r1 = s.roll(&base);
    let r2 = s.roll(&r1);
    assert_eq!(
        r2,
        varia(&base, 2, TAMANHO_ROLL),
        "Roll parte do mesmo ponto"
    );
    let m1 = s.mutate(&r2);
    assert_eq!(m1, varia(&r2, 3, TAMANHO_MUTATE));
    let m2 = s.mutate(&m1);
    assert_eq!(m2, varia(&m1, 4, TAMANHO_MUTATE), "Mutate anda");
    let n = s.next(&m2);
    assert_eq!(
        n,
        varia(&m1, 5, TAMANHO_MUTATE),
        "Next joga fora e tenta do mesmo ponto"
    );
    s.keep(&n);
    let depois = s.roll(&n);
    assert_eq!(
        depois,
        varia(&n, 6, TAMANHO_ROLL),
        "Keep vira o ponto de partida"
    );
}

#[test]
fn som_vazio_sorteia_um_de_fabrica() {
    let mut s = Sessao::default();
    let r = s.roll(&Recipe::new(1, 1.0));
    assert!(!r.layers.is_empty());
    assert!(pcm(&r).iter().any(|x| x.abs() > 1.0e-4));
}

#[test]
fn heard_recupera_o_ouvido_ha_cem_sorteios() {
    let base = fabrica("Impact");
    let mut s = Sessao::default();
    let mut atual = base;
    let mut primeiro = None;
    for i in 0..101 {
        atual = s.roll(&atual);
        let como = s.como(&atual);
        assert!(matches!(como, Como::Roll(_)));
        if i != 50 {
            s.ouviu(&atual, como);
        }
        if i == 0 {
            primeiro = Some(atual.clone());
        }
    }
    assert_eq!(s.ouvidos.len(), 100, "o que não tocou não entra");
    let primeiro = primeiro.unwrap();
    assert_eq!(s.ouvidos[0].receita, primeiro);
    assert_eq!(pcm(&s.ouvidos[0].receita), pcm(&primeiro));
    s.ouviu(&primeiro, Como::Mao);
    assert_eq!(s.ouvidos.len(), 100, "o mesmo som não repete");
}

#[test]
fn o_teto_do_heard_guarda_as_estrelas() {
    let base = Recipe::new(1, 0.1);
    let mut s = Sessao::default();
    let com_semente = |n: u64| Recipe {
        seed: n,
        ..base.clone()
    };
    s.ouviu(&com_semente(0), Como::Mao);
    s.alterna_estrela(0);
    s.ouviu(&com_semente(1), Como::Mao);
    // A busca pelo igual é linear: enche direto, sem pagar 10 000² no teste.
    let modelo = s.ouvidos[1].clone();
    s.ouvidos.extend((2..MAX_OUVIDOS as u64).map(|n| {
        let mut o = modelo.clone();
        o.receita.seed = n;
        o
    }));
    s.ouviu(&com_semente(99_999), Como::Mao);
    assert_eq!(s.ouvidos.len(), MAX_OUVIDOS);
    assert!(s.ouvidos[0].estrela, "a estrela fica");
    assert_eq!(s.ouvidos[1].receita.seed, 2, "sai o mais velho sem estrela");
}

#[test]
fn reabrir_nao_repete_o_que_ja_se_ouviu() {
    let base = fabrica("Metal Hit");
    let mut s = Sessao::default();
    let r = s.roll(&base);
    s.ouviu(&r, Como::Roll(1));
    let mut outra = Sessao::default();
    outra.adota(s.ouvidos.clone());
    assert_ne!(outra.roll(&base), r);
}

#[test]
fn n_depois_da_oitava_traz_oito_novas() {
    let takes = sem_nada();
    let mut x = Exploracao::default();
    let base = fabrica("Metal Hit");
    x.semeia(&base, TAMANHO_MUTATE, &takes);
    espera_todos(&mut x, &takes);
    let primeiro = filho(&x, 0);
    for i in 0..FILHOS {
        x.proximo_filho(&takes);
        assert_eq!(x.grade.as_ref().unwrap().ultimo, Some(i));
        assert!(x.serve(&base, false, &takes).is_some());
    }
    assert_eq!(x.sessao.ouvidos.len(), FILHOS);
    x.proximo_filho(&takes);
    assert_eq!(x.grade.as_ref().unwrap().ultimo, Some(0));
    espera_todos(&mut x, &takes);
    assert_ne!(filho(&x, 0), primeiro);
}

#[test]
fn desfazer_fora_da_grade_refaz_as_oito_em_volta_do_som_novo() {
    let takes = sem_nada();
    let mut x = Exploracao::default();
    let a = fabrica("Metal Hit");
    let b = fabrica("Wood Knock");
    x.semeia(&a, TAMANHO_MUTATE, &takes);
    let _ = x.serve(&b, true, &takes);
    assert_eq!(
        x.grade.as_ref().unwrap().base,
        a,
        "no meio do gesto não refaz"
    );
    let _ = x.serve(&b, false, &takes);
    assert_eq!(x.grade.as_ref().unwrap().base, b);
}

/// A corrida que o cancelamento não cobre: o velho terminou e já está no canal quando o novo é pedido.
#[test]
fn render_velho_que_ja_terminou_tambem_nao_entra() {
    let takes = sem_nada();
    let (velho, novo) = (fabrica("Wood Knock"), fabrica("Laser"));
    let mut n = Ninhada::default();
    n.pede(0, velho, takes.clone());
    let fim = Instant::now() + Duration::from_secs(60);
    while n.em_voo() {
        assert!(Instant::now() < fim, "o velho não terminou");
        std::thread::sleep(Duration::from_millis(1));
    }
    n.pede(0, novo.clone(), takes);
    while n.em_voo() {
        assert!(Instant::now() < fim, "o novo não terminou");
        std::thread::sleep(Duration::from_millis(1));
    }
    let chegou = n.recebe();
    assert_eq!(chegou.len(), 1, "só o novo passa");
    assert_eq!(chegou[0].1.as_ref().expect("render").receita, novo);
}
