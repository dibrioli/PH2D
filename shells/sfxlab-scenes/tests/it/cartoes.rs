//! Cada linha da tabela e cada opção pelo gesto real: escreve só o seu campo, muda o som, vale um passo de desfazer.

use crate::{Mesa, leitura};
use ph2d_panel_sfxlab::tabela::{CAMPOS, Campo, Grupo, OPCOES};
use ph2d_panel_sfxlab::{ids, nome_da_camada};
use ph2d_sfx_recipe::{
    Body, EnvPoint, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, Recording, Source, Take, TakeId,
    Tone, Wave,
};
use std::collections::BTreeSet;

const ARRASTO: f32 = 40.0;
const SEGUNDOS: f32 = 0.25;

fn curva() -> Envelope {
    Envelope::hit(0.002, 0.1, 0.01)
}

fn receita(s: Source) -> Recipe {
    let mut r = Recipe::new(0x1234_5678_9ABC_DEF0, SEGUNDOS);
    r.add(0.5, s, curva());
    r
}

fn ruido() -> Source {
    Source::Noise(Noise {
        filter: NoiseFilter::Band,
        hz: 800.0,
        hz_end: 2000.0,
        q: 2.0,
    })
}

fn tom(wave: Wave) -> Source {
    Source::Tone(Tone {
        wave,
        hz: 440.0,
        hz_end: 880.0,
        glide: 0.1,
    })
}

const FM: Wave = Wave::Fm {
    ratio: 2.0,
    index: 1.0,
    index_end: 3.0,
};

fn corpo(law: ModeLaw) -> Source {
    Source::Body(Body {
        law,
        base_hz: 420.0,
        modes: 12,
        t60: 0.2,
        damping: 0.1,
        rolloff: 0.8,
        contact_ms: 1.0,
        noise: 0.2,
    })
}

/// Material que não é igual a si mesmo em nenhum trecho.
fn material(semente: u32) -> Take {
    let mut x = semente.max(1);
    let amostras = (0..48_000)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            (x as f32 / u32::MAX as f32) - 0.5
        })
        .collect();
    Take::new(48_000, 1, amostras)
}

fn gravacao(take: TakeId) -> Source {
    Source::Recording(Recording {
        take,
        semitones: 0.0,
        offset: 0.01,
    })
}

/// Uma mesa com a aba aberta em `r`, com dois materiais no projeto.
fn mesa_com(r: Recipe) -> (Mesa, u64) {
    let mut m = Mesa::nova();
    m.lib_mut().takes.add(material(1));
    m.lib_mut().takes.add(material(2));
    let id = m.abre("Teste", r);
    (m, id)
}

fn take_um() -> TakeId {
    material(1).id
}

/// Todos os estados que, juntos, mostram toda a tabela. `None` na seleção = o som inteiro.
fn estados() -> Vec<(Recipe, bool)> {
    let mut v: Vec<(Recipe, bool)> = [
        ruido(),
        tom(Wave::Saw),
        tom(FM),
        corpo(ModeLaw::Stretched { stretch: 1.5 }),
        gravacao(take_um()),
    ]
    .into_iter()
    .map(|s| (receita(s), true))
    .collect();
    v.push((receita(ruido()), false));
    v
}

/// Abre o estado e, no do som inteiro, tira a seleção pelo clique na linha do Mix.
fn prepara(r: Recipe, com_camada: bool) -> (Mesa, u64) {
    let (mut m, id) = mesa_com(r);
    if !com_camada {
        let linha = m
            .docs()
            .rect_de(id, ph2d_app_sfxlab::Alvo::Regua)
            .expect("régua");
        // A faixa do Mix fica logo abaixo da régua.
        let (x, y) = (linha.x + 20.0, linha.y + linha.h + 30.0);
        m.ponteiro_da_shell(ph2d_host::PointerKind::Down, x, y);
        m.ponteiro_da_shell(ph2d_host::PointerKind::Up, x, y);
        m.assenta();
        assert_eq!(
            m.docs().selecionada(id),
            None,
            "o clique no Mix não tirou a seleção"
        );
    }
    (m, id)
}

fn visiveis(m: &mut Mesa, id: u64) -> Vec<&'static Campo> {
    let r = m.receita(id);
    let l = m.camada(id);
    CAMPOS
        .iter()
        .filter(|c| c.le(&r, l.as_ref()).is_some())
        .collect()
}

fn sentido(c: &Campo, v: f64) -> f32 {
    if v + f64::from(ARRASTO) * c.scrub > c.faixa.1 {
        -ARRASTO
    } else {
        ARRASTO
    }
}

#[test]
fn scrub_de_cada_campo_escreve_so_o_seu_campo_muda_o_som_e_vale_um_passo() {
    let mut vistos = BTreeSet::new();
    for (r, com_camada) in estados() {
        let (mut m, id) = prepara(r, com_camada);
        for c in visiveis(&mut m, id) {
            let (r0, l0) = (m.receita(id), m.camada(id));
            let h0 = m.hash(id);
            let v0 = c.le(&r0, l0.as_ref()).expect("visível");
            let dx = sentido(c, v0);
            m.agarra(c.id, dx);
            m.solta(c.id, dx);
            m.espera(id);
            let (r1, l1) = (m.receita(id), m.camada(id));
            let v1 = c.le(&r1, l1.as_ref()).expect("visível");
            assert!(
                (v1 - v0).abs() > 1e-6,
                "{}: o scrub não escreveu ({v0})",
                c.chave
            );
            for (i, (a, b)) in leitura(&r0, l0.as_ref(), CAMPOS)
                .into_iter()
                .zip(leitura(&r1, l1.as_ref(), CAMPOS))
                .enumerate()
            {
                if CAMPOS[i].id != c.id {
                    assert_eq!(a, b, "{} mexeu em {}", c.chave, CAMPOS[i].chave);
                }
            }
            assert_ne!(m.hash(id), h0, "{}: o som não mudou", c.chave);
            assert_eq!(m.desfaz(false), Some(true), "{}: sem passo", c.chave);
            assert_eq!(m.receita(id), r0, "{}: o desfazer não devolveu", c.chave);
            assert_eq!(
                m.desfaz(false),
                Some(false),
                "{}: mais de um passo",
                c.chave
            );
            m.espera(id);
            assert_eq!(m.hash(id), h0, "{}: o som não voltou", c.chave);
            vistos.insert(c.id);
        }
    }
    let faltam: Vec<_> = CAMPOS
        .iter()
        .filter(|c| !vistos.contains(&c.id))
        .map(|c| c.chave)
        .collect();
    assert!(faltam.is_empty(), "nenhum estado mostra: {faltam:?}");
}

#[test]
fn esc_a_meio_do_scrub_devolve_e_nao_deixa_passo() {
    for (r, com_camada) in estados() {
        let (mut m, id) = prepara(r, com_camada);
        for c in visiveis(&mut m, id) {
            let (r0, l0) = (m.receita(id), m.camada(id));
            let v0 = c.le(&r0, l0.as_ref()).expect("visível");
            m.agarra(c.id, sentido(c, v0));
            assert!(
                m.hero.text_entry_focused(),
                "{}: no scrub o Esc tem de ir ao campo, não à aba",
                c.chave
            );
            let (r1, l1) = (m.receita(id), m.camada(id));
            assert_ne!(c.le(&r1, l1.as_ref()), Some(v0), "{}: não mexeu", c.chave);
            m.esc();
            m.quadro();
            assert_eq!(m.receita(id), r0, "{}: o Esc não devolveu", c.chave);
            assert_eq!(
                m.desfaz(false),
                Some(false),
                "{}: o Esc deixou passo",
                c.chave
            );
        }
    }
}

#[test]
fn digitar_um_numero_escreve_o_campo_e_vale_um_passo() {
    for (r, com_camada) in estados() {
        let (mut m, id) = prepara(r, com_camada);
        for c in visiveis(&mut m, id) {
            let (r0, l0) = (m.receita(id), m.camada(id));
            let v0 = c.le(&r0, l0.as_ref()).expect("visível");
            let alvo = if v0 + c.passo * 3.0 > c.faixa.1 {
                v0 - c.passo * 3.0
            } else {
                v0 + c.passo * 3.0
            };
            m.digita(c.id, &format!("{alvo}"));
            m.quadro();
            let (r1, l1) = (m.receita(id), m.camada(id));
            let v1 = c.le(&r1, l1.as_ref()).expect("visível");
            assert!(
                (v1 - alvo).abs() < 0.01,
                "{}: pedi {alvo}, ficou {v1}",
                c.chave
            );
            m.hero.store.set_focus(None);
            assert_eq!(m.desfaz(false), Some(true), "{}", c.chave);
            assert_eq!(
                m.desfaz(false),
                Some(false),
                "{}: mais de um passo",
                c.chave
            );
        }
    }
}

#[test]
fn cada_opcao_de_filter_wave_e_law_escolhe_a_si_e_muda_o_som() {
    for (grupo, fonte, n) in [
        (Grupo::Filter, ruido(), 4),
        (Grupo::Wave, tom(Wave::Sine), 5),
        (Grupo::Law, corpo(ModeLaw::Membrane), 3),
    ] {
        let dele: Vec<_> = OPCOES.iter().filter(|o| o.grupo == grupo).collect();
        assert_eq!(dele.len(), n, "{grupo:?}");
        let (mut m, id) = mesa_com(receita(fonte));
        for o in dele {
            let l = m.camada(id).expect("camada");
            if (o.aceso)(&l) == Some(true) {
                continue;
            }
            let h0 = m.hash(id);
            m.clica(o.id);
            m.espera(id);
            let l = m.camada(id).expect("camada");
            assert_eq!((o.aceso)(&l), Some(true), "{}: não acendeu", o.chave);
            assert_ne!(m.hash(id), h0, "{}: o som não mudou", o.chave);
            m.assenta();
        }
    }
}

#[test]
fn clicar_de_novo_no_fm_nao_apaga_o_que_o_artista_ajustou() {
    let (mut m, id) = mesa_com(receita(tom(FM)));
    let r0 = m.receita(id);
    let fm = OPCOES
        .iter()
        .find(|o| o.chave == "sfxlab.card.wave.fm")
        .expect("FM");
    m.clica(fm.id);
    m.quadro();
    assert_eq!(m.receita(id), r0);
    assert_eq!(m.desfaz(false), Some(false));
}

#[test]
fn ruido_branco_nao_oferece_hz_nem_q() {
    let (mut m, _) = mesa_com(receita(ruido()));
    let hz = CAMPOS
        .iter()
        .find(|c| c.chave == "sfxlab.card.hz")
        .expect("hz")
        .id;
    assert!(m.rect(hz).is_some(), "o filtro Band tem Hz");
    let branco = OPCOES
        .iter()
        .find(|o| o.chave == "sfxlab.card.filter.white")
        .expect("White");
    m.clica(branco.id);
    m.assenta();
    assert!(
        m.rect(hz).is_none(),
        "o Hz do ruído branco é controle morto"
    );
}

#[test]
fn envelope_estranho_mostra_a_frase_e_nao_os_tres_campos() {
    let forma: Vec<_> = CAMPOS
        .iter()
        .filter(|c| c.cartao == ph2d_panel_sfxlab::tabela::Cartao::Forma)
        .collect();
    assert_eq!(forma.len(), 3);
    let estranho = Envelope {
        points: [(0.0, 0.0), (0.05, 1.0), (0.1, 0.3), (0.2, 0.8), (0.25, 0.0)]
            .map(|(t, v)| EnvPoint { t, v, exp: false })
            .to_vec(),
    };
    let quatro = Envelope {
        points: [(0.0, 0.2), (0.05, 1.0), (0.1, 0.3), (0.2, 0.0)]
            .map(|(t, v)| EnvPoint { t, v, exp: false })
            .to_vec(),
    };
    for e in [estranho, quatro, Envelope::default()] {
        let mut r = receita(ruido());
        r.layers[0].envelope = e.clone();
        let (mut m, id) = mesa_com(r);
        for c in &forma {
            assert!(m.rect(c.id).is_none(), "{} mente sobre {e:?}", c.chave);
        }
        assert!(m.rect(ids::FORMA_HIT).is_some(), "sem o verbo de trocar");
        assert_eq!(
            m.receita(id).layers[0].envelope,
            e,
            "pintar não pode escrever"
        );
        // O verbo troca a curva, e aí sim os três aparecem: um passo de desfazer.
        m.clica(ids::FORMA_HIT);
        m.assenta();
        for c in &forma {
            assert!(m.rect(c.id).is_some(), "{} não apareceu", c.chave);
        }
        assert_eq!(m.desfaz(false), Some(true));
        assert_eq!(m.receita(id).layers[0].envelope, e);
    }
}

#[test]
fn a_curva_que_veio_de_hit_mostra_os_tres_e_escreve_hit() {
    let (mut m, id) = mesa_com(receita(ruido()));
    let ataque = CAMPOS
        .iter()
        .find(|c| c.chave == "sfxlab.card.attack_ms")
        .expect("attack");
    m.digita(ataque.id, "5");
    m.quadro();
    let e = m.receita(id).layers[0].envelope.clone();
    assert_eq!(
        e,
        Envelope::hit(0.005, 0.1, 0.01),
        "o campo escreve pela porta do hit"
    );
}

#[test]
fn soltar_o_scrub_toca_uma_vez_com_play_on_change() {
    let (mut m, id) = mesa_com(receita(ruido()));
    m.docs()
        .rect_de(id, ph2d_app_sfxlab::Alvo::PlayOnChange)
        .expect("caixa");
    m.clica_na_aba(id, ph2d_app_sfxlab::Alvo::PlayOnChange);
    let c = CAMPOS
        .iter()
        .find(|c| c.chave == "sfxlab.card.start_ms")
        .expect("start");
    m.agarra(c.id, ARRASTO);
    m.espera(id);
    assert_eq!(m.escuta.toques, 0, "o scrub não toca a cada passo");
    m.solta(c.id, ARRASTO);
    m.espera(id);
    m.quadro();
    assert_eq!(m.escuta.toques, 1, "soltar toca");
}

#[test]
fn o_material_se_escolhe_na_lista_com_nomes_legiveis() {
    let (mut m, id) = mesa_com(receita(gravacao(take_um())));
    let h0 = m.hash(id);
    let outro = {
        let Source::Recording(g) = m.receita(id).layers[0].source else {
            panic!("gravação")
        };
        m.lib_mut()
            .takes
            .iter()
            .map(|t| t.id)
            .find(|t| *t != g.take)
            .expect("o outro material")
    };
    m.clica(ids::MATERIAL);
    m.assenta();
    let indice = m
        .lib_mut()
        .takes
        .iter()
        .position(|t| t.id == outro)
        .expect("índice");
    m.clica(ids::opcao_de_material(indice));
    m.espera(id);
    let Source::Recording(g) = m.receita(id).layers[0].source else {
        panic!("gravação")
    };
    assert_eq!(g.take, outro);
    assert_ne!(m.hash(id), h0, "trocar de material muda o som");
    assert_eq!(m.desfaz(false), Some(true));
}

#[test]
fn o_nome_da_camada_e_o_tipo_numerado_quando_repete_e_reordenar_nao_renomeia() {
    let mut r = receita(ruido());
    r.add(0.5, tom(Wave::Sine), curva());
    r.add(0.5, ruido(), curva());
    let nomes =
        |r: &Recipe| -> Vec<String> { r.layers.iter().map(|l| nome_da_camada(r, l)).collect() };
    assert_eq!(nomes(&r), ["Noise", "Tone", "Noise 2"]);
    let primeira = r.layers.remove(0);
    r.layers.push(primeira);
    assert_eq!(nomes(&r), ["Tone", "Noise 2", "Noise"]);
}
