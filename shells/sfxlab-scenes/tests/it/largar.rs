//! O gesto é de quem recebeu a descida, onde quer que o ponteiro vá: pela ordem de eventos da shell.

use crate::Mesa;
use ph2d_app_sfxlab::{Alvo, AlvoE};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::board_view;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_host::PointerKind::{Down, Move, Up};
use ph2d_panel_sfxlab::tabela::{CAMPOS, Campo};
use ph2d_sfx_recipe::{Envelope, LayerId, Recipe, factory};
use ph2d_tool_registry::hash_node_id;

const VALOR_INICIAL: &str = "200";
/// Para a direita: a pega fica junto à borda esquerda do painel, e o limiar tem de passar dentro dele.
const PASSO_DO_LIMIAR: f32 = 20.0;
const ALVOS_MINIMOS: usize = 15;

fn metal() -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Metal Hit")
        .map(|(_, mut r)| {
            r.layers[0].envelope = Envelope::hit(0.002, 0.1, 0.01);
            r
        })
        .expect("som de fábrica")
}

fn attack() -> &'static Campo {
    CAMPOS
        .iter()
        .find(|c| c.id == hash_node_id("sfxlab_attack"))
        .expect("campo")
}

fn le(m: &mut Mesa, id: u64) -> f64 {
    let (r, l) = (m.receita(id), m.camada(id));
    attack().le(&r, l.as_ref()).expect("Attack visível")
}

/// O som aberto, a camada com o Attack à vista, e o Attack num valor com folga para os dois lados.
fn mesa() -> (Mesa, u64, LayerId) {
    let mut m = Mesa::nova();
    let id = m.abre("Metal Hit", metal());
    let camadas: Vec<LayerId> = m.receita(id).layers.iter().map(|l| l.id).collect();
    let camada = camadas
        .into_iter()
        .find(|c| {
            m.clica_na_aba(id, Alvo::Linha(*c));
            m.assenta();
            m.rect(attack().id).is_some()
        })
        .expect("nenhuma camada mostra o Attack");
    m.digita(attack().id, VALOR_INICIAL);
    m.espera(id);
    (m, id, camada)
}

fn centro(m: &mut Mesa, id: u64, alvo: Alvo) -> Option<(f32, f32)> {
    let r = m.docs().rect_de(id, alvo)?;
    Some((r.x + r.w * 0.5, r.y + r.h * 0.5))
}

fn pega(m: &Mesa, chip: NodeId) -> (f32, f32) {
    let p = handle_rect(m.rect(chip).expect("campo pintado"));
    (p.x + p.w * 0.5, p.y + p.h * 0.5)
}

/// Premir na pega e passar do limiar, ainda sobre o painel: o cursor já está preso.
fn agarra(m: &mut Mesa) {
    let (x, y) = pega(m, attack().id);
    assert!(!m.mao_da_shell(Down, x, y), "a pega é do painel");
    assert!(!m.mao_da_shell(Move, x + PASSO_DO_LIMIAR, y));
    assert!(!m.mao_da_shell(Move, x + PASSO_DO_LIMIAR * 2.0, y));
    assert!(m.hero.store.pointer_lock_wanted(), "o scrub não prendeu");
}

/// O efeito inteiro de largar o scrub em `(x, y)`, sobre a aba, descrito por `onde`.
fn solta_sobre_a_aba(m: &mut Mesa, id: u64, r0: &Recipe, (x, y): (f32, f32), onde: &str) {
    let sel = m.docs().selecionada(id);
    let toques = m.escuta.toques;
    agarra(m);
    let no_painel = le(m, id);
    assert!(
        !m.mao_da_shell(Move, x, y),
        "{onde}: a aba tomou o movimento"
    );
    assert!(
        board_view::pointer_look(&m.hero, x, y).is_none(),
        "{onde}: o ponteiro da aba a meio do arrasto de outro"
    );
    let arrastado = le(m, id);
    assert_ne!(arrastado, no_painel, "{onde}: o scrub parou sobre a aba");
    assert!(!m.mao_da_shell(Up, x, y), "{onde}: a aba tomou o largar");
    assert_eq!(
        m.hero.store.number_input_drag().map(|d| d.id),
        None,
        "{onde}: o arrasto não acabou"
    );
    assert!(
        !m.hero.store.pointer_lock_wanted(),
        "{onde}: o cursor continua preso e invisível"
    );
    m.espera(id);
    assert!(!m.docs().so_kept(id), "{onde}: o largar virou clique");
    assert_eq!(m.docs().selecionada(id), sel, "{onde}: mudou a seleção");
    assert_eq!(m.escuta.toques, toques, "{onde}: tocou");
    assert_eq!(le(m, id), arrastado, "{onde}: o largar mexeu no valor");
    assert_eq!(m.desfaz(false), Some(true), "{onde}: sem passo de desfazer");
    assert_eq!(m.receita(id), *r0, "{onde}: não foi um passo só");
    m.espera(id);
    // Fora da janela do duplo clique, que na pega repõe o valor de fábrica.
    m.assenta();
}

#[test]
fn soltar_o_arrasto_do_campo_sobre_a_aba_termina_o_arrasto_e_nao_clica() {
    let (mut m, id, _) = mesa();
    let r0 = m.receita(id);
    let kept = centro(&mut m, id, Alvo::Ex(AlvoE::SoKept)).expect("Kept only pintado");
    solta_sobre_a_aba(&mut m, id, &r0, kept, "Kept only");
    // O filtro continua a responder ao clique dele.
    m.clica_na_aba(id, Alvo::Ex(AlvoE::SoKept));
    assert!(m.docs().so_kept(id), "o clique de verdade liga o filtro");
}

#[test]
fn soltar_o_arrasto_do_campo_sobre_qualquer_alvo_da_aba_nao_clica() {
    let (mut m, id, l) = mesa();
    let r0 = m.receita(id);
    let alvos = [
        Alvo::Play,
        Alvo::Stop,
        Alvo::PlayOnChange,
        Alvo::UseInGame,
        Alvo::Regua,
        Alvo::Linha(l),
        Alvo::Alca(l),
        Alvo::Mudo(l),
        Alvo::Solo(l),
        Alvo::Onda(l),
        Alvo::Trava(l),
        Alvo::Add,
        Alvo::Ex(AlvoE::Roll),
        Alvo::Ex(AlvoE::Mutate),
        Alvo::Ex(AlvoE::Next),
        Alvo::Ex(AlvoE::Keep),
        Alvo::Ex(AlvoE::Lock),
        Alvo::Ex(AlvoE::Explore),
        Alvo::Ex(AlvoE::SoKept),
    ];
    let mut vistos = 0;
    for alvo in alvos {
        if let Some(p) = centro(&mut m, id, alvo) {
            solta_sobre_a_aba(&mut m, id, &r0, p, &format!("{alvo:?}"));
            vistos += 1;
        }
    }
    assert!(vistos >= ALVOS_MINIMOS, "só {vistos} alvos pintados");
    // A área vazia: a faixa do Mix, abaixo da régua, onde o clique tira a seleção.
    let regua = m.docs().rect_de(id, Alvo::Regua).expect("régua");
    let vazio = (regua.x + 20.0, regua.y + regua.h + 30.0);
    solta_sobre_a_aba(&mut m, id, &r0, vazio, "área vazia");
    // E o cartão aberto não ficou por causa dos largares.
    assert!(
        m.docs()
            .rect_de(id, Alvo::Item(ph2d_app_sfxlab::Fonte::Noise))
            .is_none()
    );
}

/// O outro sentido: o gesto nasce na aba, o largar cai no painel. É da aba, e o campo não sabe de nada.
fn gesto_da_aba_solto_no_painel(m: &mut Mesa, id: u64, de: (f32, f32), onde: &str) {
    let (x, y) = pega(m, attack().id);
    let v0 = le(m, id);
    assert!(
        m.mao_da_shell(Down, de.0, de.1),
        "{onde}: a descida é da aba"
    );
    m.mao_da_shell(Move, x, y);
    m.mao_da_shell(Up, x, y);
    assert_eq!(
        m.hero.store.number_input_drag().map(|d| d.id),
        None,
        "{onde}"
    );
    assert_ne!(
        m.hero.store.focus_id(),
        Some(attack().id),
        "{onde}: o largar focou o campo"
    );
    assert!(!m.mao_da_shell(Move, x, y), "{onde}: o gesto não fechou");
    assert_eq!(le(m, id), v0, "{onde}: o campo mexeu");
    m.espera(id);
}

#[test]
fn o_gesto_que_nasce_na_aba_e_solta_no_painel_fecha_sem_tocar_no_campo() {
    let (mut m, id, l) = mesa();
    let onda = centro(&mut m, id, Alvo::Onda(l)).expect("onda");
    let antes = m.receita(id);
    gesto_da_aba_solto_no_painel(&mut m, id, onda, "Onda");
    assert_ne!(m.receita(id), antes, "a Onda arrastada não moveu");
    assert_eq!(m.desfaz(false), Some(true), "a Onda sem passo");
    assert_eq!(m.receita(id), antes, "a Onda em mais de um passo");
    m.espera(id);

    let alca = centro(&mut m, id, Alvo::Alca(l)).expect("alça");
    gesto_da_aba_solto_no_painel(&mut m, id, alca, "Alça");
    while m.receita(id) != antes {
        assert_eq!(m.desfaz(false), Some(true));
    }
    m.espera(id);

    // A prensa num botão: largar noutro sítio não é clique em ninguém.
    let kept = centro(&mut m, id, Alvo::Ex(AlvoE::SoKept)).expect("Kept only");
    gesto_da_aba_solto_no_painel(&mut m, id, kept, "prensa");
    assert!(!m.docs().so_kept(id), "o largar longe da prensa clicou");

    // A grade do Explore: o filho arrastado e a mudança.
    m.clica_na_aba(id, Alvo::Ex(AlvoE::Explore));
    m.espera(id);
    m.assenta();
    for alvo in [Alvo::Ex(AlvoE::Filho(0)), Alvo::Ex(AlvoE::Mudanca)] {
        let de = centro(&mut m, id, alvo).expect("o Explore pinta");
        gesto_da_aba_solto_no_painel(&mut m, id, de, &format!("{alvo:?}"));
    }
}
