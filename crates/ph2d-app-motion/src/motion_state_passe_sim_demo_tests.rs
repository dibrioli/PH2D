//! Os gates da cena `=122` (doc 115 §15.2).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu** — as peças são `source.shape`, que lê um EXTERNAL
//! publicado por `motion_shape_gen::publish`; num cozedor virgem ele emite **zero** e os gates
//! mediriam o vazio. É o precedente que a `=110`, a `=114` e a `=121` já nomeiam.
//!
//! ⚠️ E o passe é chamado **aqui**, sobre a corrente cozida, como nos gates da `=121`: que a ROTA
//! do pump o chama já é provado pelos gates do `ph2d-eval-motion` (`passe_no_pump_tests`), e
//! repeti-lo aqui mediria a rota em vez da CENA.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// Os instantes em que a cena é medida. ⚠️ **Vários, e é metade do que esta cena afirma:** a
/// `=121` prova que o passe separa UMA vez; o que esta acrescenta é que ele o faz **em todo
/// quadro**, sobre peças que nunca param. Um gate num instante só não distinguiria as duas.
const INSTANTES: [f64; 5] = [0.0, 0.1, 0.6, 1.5, 3.0];

/// Corre a cena até `secs` e devolve a corrente final de cada fileira.
fn corre(secs: f64) -> Vec<Stream> {
    let mut m = MotionState::new();
    assert!(
        m.doc.graph.nodes().is_empty(),
        "o `MotionState` nasceu com nos: desligue o PH2D_GPU_COOK_DEMO para correr estes gates"
    );
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "um indice de tique"
    )]
    let ultimo = (secs * 60.0) as u64;
    let mut fim = vec![Stream::new(0); sinks.len()];
    for k in 0..=ultimo {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        for (i, sink) in sinks.iter().enumerate() {
            fim[i] = m
                .pump
                .cook
                .cook(&m.doc.graph, &m.registry, *sink, t)
                .expect("a cena cozinha")[0]
                .as_stream()
                .clone();
        }
        // ⚠️⚠️ **SEM ISTO O RELÓGIO DO ARNÊS FICA PARADO, e ele mente em silêncio.** A 1.ª
        // redacção só cozia em `t` crescente e lia a MESMA nuvem ao bit no instante zero e aos
        // 2,5 s — o que se lê como *«a cena está parada»* quando o que estava parado era o arnês.
        // (O molde é o `tests.rs` do próprio `motion.integrate`: cozer, depois `advance_tick`.)
        m.pump
            .cook
            .advance_tick(&m.doc.graph, &m.registry, t)
            .expect("o tique avanca");
    }
    fim
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a cena tem de emitir P"),
    }
}

/// Quantos pares se atravessam de forma VISÍVEL — a régua do doc 115 §9.1 (`2 %` do lado), e não
/// `contato(..).is_some()`, que conta os pares que apenas se TOCAM e acusaria a convergência.
fn pares_atravessados(s: &Stream) -> usize {
    const VISIVEL: f32 = 0.02;
    let p = posicoes(s);
    let Some(c) = ph2d_contact::colisores(s) else {
        return 0;
    };
    let mut n = 0;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let (Some(a), Some(b)) = (c[i], c[j]) else {
                continue;
            };
            if ph2d_contact::contato(&a, p[i], &b, p[j], false)
                .is_some_and(|m| m.penetracao > VISIVEL * super::LADO)
            {
                n += 1;
            }
        }
    }
    n
}

/// As varreduras de FÁBRICA — o que o artista recebe ao ligar o interruptor e não tocar em mais
/// nada. ⛔ Uma cena que só assenta com o knob no máximo é uma cena que o dono reprova.
fn de_fabrica() -> usize {
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "o default e' um inteiro pequeno e positivo, declarado no manifesto"
    )]
    let n = ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_DEFAULT as usize;
    n
}

/// ⭐⭐⭐ **AS PEÇAS TREMEM: a cena precisa de PLAY, e isso é gateado.**
///
/// ⚠️ Sem esta metade os outros gates mediriam um arranjo parado — e a cena, que existe para
/// mostrar o passe a correr **em todo quadro**, podia perder o `motion.wiggle` sem nada acusar.
#[test]
fn as_pecas_tremem_com_o_relogio() {
    let a = posicoes(&corre(0.0)[0]);
    let b = posicoes(&corre(0.6)[0]);
    let mexeu = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| (x[0] - y[0]).abs() > 1e-4 || (x[1] - y[1]).abs() > 1e-4)
        .count();
    assert!(
        mexeu >= a.len() / 2,
        "as pecas tem de TREMER com o relogio — so' {mexeu} de {} se mexeram",
        a.len()
    );
}

/// ⭐⭐⭐ **O KNOB DA CENA, EM TODO QUADRO: a fileira livre separa-se e a protegida NÃO.**
///
/// ⚠️ **As duas metades, e a segunda é a wave inteira** (o §10.4, aberto desde a W5): a primeira
/// sozinha provaria só que o passe funciona — que a `=121` já prova. O que esta cena acrescenta é
/// que uma fileira com `falloff = 0` fica **por separar com o interruptor LIGADO**, que é a forma
/// de um objecto dizer *«eu não colido»*.
///
/// ⚠️ E mede-se nos **cinco** instantes: a promessa não é *«separou»*, é *«fica separada»*.
#[test]
fn armada_a_fileira_livre_separa_se_em_todo_quadro_e_a_protegida_nao() {
    for t in INSTANTES {
        let fim = corre(t);
        let (livre, protegida) = (&fim[0], &fim[1]);

        let antes = pares_atravessados(livre);
        assert_eq!(
            antes,
            super::PECAS / 2,
            "t = {t}: TODO par tem de nascer atravessado, senao o interruptor nao tem o que curar"
        );

        let sep = ph2d_contact::passe::separa_o_que_se_desenha(livre, de_fabrica())
            .expect("a fileira livre tem o que separar");
        assert_eq!(
            pares_atravessados(&sep),
            0,
            "t = {t}: com as varreduras de FABRICA nenhum par pode ficar atravessado (eram {antes})"
        );

        // ⭐ A protegida: o passe ou devolve `None` (nada se mexeu) ou devolve a MESMA nuvem.
        let depois = ph2d_contact::passe::separa_o_que_se_desenha(protegida, de_fabrica())
            .map_or_else(|| posicoes(protegida), |s| posicoes(&s));
        assert_eq!(
            depois,
            posicoes(protegida),
            "t = {t}: a fileira com `falloff = 0` nao pode ser movida um bit pelo passe"
        );
    }
}

/// ⚠️ **O campo põe ZERO EXACTO, e o número é MEDIDO e não suposto.**
///
/// ⛔ A 1.ª redacção desta cena punha o campo em cima da fileira com `invert = 1`, e leu
/// `0,68 · 0,51 · 0,16 …` — *o `invert` dá uma RAMPA, não um interruptor*. Um campo que peça
/// nenhuma alcança é o que dá `0` **ao bit**, que é o que a fileira protegida precisa.
#[test]
fn o_campo_poe_zero_exacto_na_fileira_protegida() {
    let fim = corre(0.6);
    let Some(Column::Scalar(f)) = fim[1].get("falloff") else {
        panic!("a fileira protegida tem de trazer a coluna `falloff`");
    };
    assert!(
        f.iter().all(|v| *v == 0.0),
        "o campo tem de zerar a fileira AO BIT; veio {:?}",
        &f[..f.len().min(6)]
    );
    // ⭐ E o CONTROLO: a fileira livre não traz a coluna nenhuma — é a ausência que o passe lê
    // como `1`, e é ela que mantém byte-idêntica toda cena sem campo.
    assert!(
        fim[0].get("falloff").is_none(),
        "a fileira livre nao pode ter `falloff` — senao as duas nao diferem no knob que a cena diz"
    );
}

/// ⛔⛔ **NENHUM nó de colisão, e nenhum solver de contacto** — é o que torna a cena honesta.
///
/// ⚠️ A segunda metade é a que uma leitura rápida salta: o `sim.step` é **um dos três leitores do
/// colisor declarado**, logo numa simulação dele as peças já seriam separadas dentro do tique e o
/// passe não teria nada que fazer. *A cena mostraria o passe a não fazer nada e chamar-lhe-ia
/// sucesso* (doc 115 §15.3).
#[test]
fn a_cena_nao_tem_colisor_nenhum_nem_um_solver_de_contacto() {
    let mut m = MotionState::new();
    super::build(&mut m.doc, &m.registry).expect("a cena monta");
    for n in m.doc.graph.nodes() {
        assert!(
            !matches!(
                n.type_name.as_str(),
                "motion.collide" | "sim.collide" | "sim.step"
            ),
            "a cena do passe EM MOVIMENTO nao pode ter `{}` — o assunto dela e' que quem separa \
             e' o interruptor do sink, e mais nada",
            n.type_name
        );
    }
}

/// ⚠️ **As duas fileiras nascem DESARMADAS** — o primeiro quadro são os pares atravessados.
#[test]
fn as_duas_fileiras_nascem_com_o_interruptor_desligado() {
    let mut m = MotionState::new();
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    assert_eq!(sinks.len(), 2, "a cena tem DUAS fileiras");
    for (i, s) in sinks.iter().enumerate() {
        assert_eq!(
            ph2d_eval_motion::sink_collide_sweeps(&m.doc.graph, *s),
            0,
            "a fileira {i} tem de abrir desarmada"
        );
    }
}

/// ⭐ **O roteiro nomeia controlos que EXISTEM** — a lei que o tutorial do `#15` da
/// `line/components` já paga: *um passo que manda ligar `Collide` AFIRMA que essa linha está lá*.
#[test]
fn o_roteiro_nomeia_controlos_que_existem() {
    const ROTEIRO: &str = include_str!("motion_state_passe_sim_demo.rs");
    let m = MotionState::new();
    for (tipo, citado) in [
        ("motion.output", "Collide"),
        ("motion.falloff", "Center X"),
        ("motion.falloff", "Center Y"),
    ] {
        let tem: Vec<&str> = m
            .registry
            .param_ui(ph2d_nodegraph::node::NodeTypeId::of(tipo))
            .map(|h| h.iter().map(|x| ph2d_i18n::tr(x.label)).collect())
            .unwrap_or_default();
        assert!(
            tem.contains(&citado),
            "o roteiro manda mexer em `{citado}` e o cartao de `{tipo}` nao tem essa linha: {tem:?}"
        );
        assert!(
            ROTEIRO.contains(citado),
            "o gate cita `{citado}` e o roteiro nao — a regua deixou de medir o roteiro"
        );
    }
}
