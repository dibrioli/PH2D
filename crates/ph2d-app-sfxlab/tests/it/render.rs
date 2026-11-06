//! Quando rende e o que volta: o velho nunca passa, a tela nunca espera, o desfazer devolve o som.

use crate::{Mesa, verdade};
use ph2d_app_sfxlab::{Alvo, Pedido, RenderJob};
use ph2d_sfx_recipe::{Body, Envelope, ModeLaw, Recipe, Source, Takes, factory};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// O pior corpo do motor: 120 s, 1 080 modos sem amortecer (9,4 s em `03_W2_medicao.md`).
fn pior_corpo() -> Recipe {
    let mut r = Recipe::new(1, 120.0);
    r.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Stretched { stretch: 1.0 },
            base_hz: 20.0,
            modes: 1080,
            t60: 120.0,
            damping: 0.0,
            rolloff: 0.0,
            contact_ms: 0.1,
            noise: 0.0,
        }),
        Envelope::default(),
    );
    r
}

fn fabrica(nome: &str) -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == nome)
        .map(|(_, r)| r)
        .expect("fábrica")
}

fn pedido(r: &Recipe) -> Pedido {
    Pedido {
        receita: r.clone(),
        solo: None,
        takes: Arc::new(Takes::default()),
    }
}

fn espera_o(job: &RenderJob, prazo: Duration) -> ph2d_app_sfxlab::Feito {
    let fim = Instant::now() + prazo;
    loop {
        if let Some(r) = job.poll() {
            return r.expect("render");
        }
        assert!(Instant::now() < fim, "nada chegou");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn render_velho_nunca_substitui_o_novo() {
    // O velho já pronto na fila, sem ninguém o ter lido, ainda não vale.
    let job = RenderJob::default();
    let (a, b) = (fabrica("Laser"), fabrica("Whoosh"));
    job.submit(pedido(&a), 1);
    std::thread::sleep(Duration::from_millis(500));
    job.submit(pedido(&b), 2);
    let f = espera_o(&job, Duration::from_secs(60));
    assert_eq!(f.ticket, 2);
    assert_eq!(f.pcm.samples, verdade(&b, &Default::default()));

    // O pior corpo, logo seguido de uma edição barata: chega a barata, e bem antes dos 9 s do corpo.
    let job = RenderJob::default();
    job.submit(pedido(&pior_corpo()), 1);
    std::thread::sleep(Duration::from_millis(50));
    let t0 = Instant::now();
    job.submit(pedido(&a), 2);
    let f = espera_o(&job, Duration::from_secs(60));
    assert_eq!(f.ticket, 2);
    assert_eq!(f.pcm.samples, verdade(&a, &Default::default()));
    assert!(t0.elapsed() < Duration::from_secs(8), "{:?}", t0.elapsed());
}

#[test]
fn editar_nao_bloqueia_a_tela() {
    let mut m = Mesa::new();
    m.poe("Worst", pior_corpo());
    let t0 = Instant::now();
    let id = m.abre("Worst");
    assert!(
        m.docs().em_voo(id),
        "o pior corpo não pode ter terminado aqui"
    );

    // Um clique real manda um pedido novo sobre o pior corpo em voo.
    let mudo = Alvo::Mudo(m.camada(id, 0));
    m.clica(id, mudo);
    m.clica(id, mudo);
    let gasto = t0.elapsed();
    assert!(m.docs().em_voo(id));
    assert!(gasto < Duration::from_secs(3), "a tela esperou {gasto:?}");
}

#[test]
fn desfazer_devolve_a_receita_e_o_som() {
    let mut m = Mesa::new();
    m.poe("Impact", fabrica("Impact"));
    let id = m.abre("Impact");
    m.espera(id);
    let (r0, h0) = (m.receita(id), m.hash(id));

    // Um gesto contínuo inteiro é um passo só.
    let onda = Alvo::Onda(m.camada(id, 1));
    let de = m.centro(id, onda);
    m.arrasta(de, (de.0 + 90.0, de.1), 6);
    m.espera(id);
    let (r1, h1) = (m.receita(id), m.hash(id));
    assert_ne!(r1, r0);
    assert_ne!(h1, h0, "arrastar o início tem de mudar o som");
    assert_eq!(m.lib().recipes["Impact"], r1, "o mundo recebe a edição");

    // Uma edição discreta é outro passo.
    m.clica(id, Alvo::Mudo(r0.layers[0].id));
    m.espera(id);
    let (r2, h2) = (m.receita(id), m.hash(id));
    assert_ne!(h2, h1);

    assert_eq!(m.desfaz(false), Some(true));
    m.espera(id);
    assert_eq!((m.receita(id), m.hash(id)), (r1.clone(), h1));
    assert_eq!(m.desfaz(false), Some(true));
    m.espera(id);
    assert_eq!((m.receita(id), m.hash(id)), (r0, h0), "o gesto é um passo");
    let agora = m.receita(id);
    assert_eq!(m.lib().recipes["Impact"], agora);

    assert_eq!(m.desfaz(true), Some(true));
    m.espera(id);
    assert_eq!((m.receita(id), m.hash(id)), (r1, h1));
    assert_eq!(m.desfaz(true), Some(true));
    m.espera(id);
    assert_eq!((m.receita(id), m.hash(id)), (r2, h2));
}

#[test]
fn abrir_projeto_adota_a_receita_e_zera_o_desfazer() {
    let mut m = Mesa::new();
    m.poe("Shared", fabrica("Laser"));
    let id = m.abre("Shared");
    let c = m.camada(id, 0);
    m.clica(id, Alvo::Mudo(c));
    assert_eq!(m.desfaz(false), Some(true));
    let c = m.camada(id, 0);
    m.clica(id, Alvo::Mudo(c));

    // Outro projeto: o mesmo nome, outra receita.
    let outra = fabrica("Whoosh");
    m.poe("Shared", outra.clone());
    m.quadro();
    assert_eq!(m.receita(id), outra, "a receita do mundo é adotada");
    assert_eq!(m.desfaz(false), Some(false), "o desfazer recomeça");
    assert_eq!(m.receita(id), outra);
    m.espera(id);
    let certo = verdade(&outra, m.lib());
    assert_eq!(m.docs().pcm(id).expect("pcm"), &*certo);

    // O nome sumiu do projeto: a aba fecha.
    m.mundo
        .resource_mut::<ph2d_sfx_recipe::SfxLibrary>()
        .recipes
        .clear();
    m.quadro();
    assert!(m.docs().ids().is_empty());
}

#[test]
fn som_novo_tem_nome_unico_toca_e_a_semente_vem_do_nome() {
    let mut m = Mesa::new();
    let a = m.som_novo();
    let b = m.som_novo();
    assert_ne!(a, b);
    let (na, nb) = (
        m.docs().nome(a).expect("a").to_owned(),
        m.docs().nome(b).expect("b").to_owned(),
    );
    assert_ne!(na, nb);
    let (ra, rb) = (m.receita(a), m.receita(b));
    assert_eq!(ra.seed, ph2d_app_sfxlab::semente_do_nome(&na));
    assert_ne!(ra.seed, rb.seed);
    assert_eq!(m.lib().recipes[&na], ra);
    assert!(matches!(ra.layers[0].source, Source::Body(_)));
    m.espera(b);
    assert!(
        crate::pico(m.docs().pcm(b).expect("pcm")) > 0.05,
        "tem de se ouvir"
    );
}

#[test]
fn a_cena_3_abre_o_som_da_cena_pela_fabrica() {
    let mut m = Mesa::new();
    let mut mundo = std::mem::take(&mut m.mundo);
    m.docs().abre_a_cena(&mut mundo);
    m.docs().abre_a_cena(&mut mundo);
    m.mundo = mundo;
    m.quadro();
    m.quadro();
    let id = m.activa();
    assert_eq!(m.docs().nome(id), Some(ph2d_app_sfxlab::SOM_DA_CENA));
    assert_eq!(m.receita(id), fabrica(ph2d_app_sfxlab::SOM_DA_CENA));
    assert_eq!(
        m.docs().ids().len(),
        1,
        "um pedido repetido não abre duas abas"
    );
}
