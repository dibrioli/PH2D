//! Os gates da cena `=121` (doc 115 W5).
//!
//! ⚠️⚠️ **Eles medem o BARRO e não o documento.** Uma cena está certa como DADOS e é impossível
//! como GESTO — foi o que o `#15` da `line/components` pagou com o report *«não apareceu a secção»*
//! —, e aqui o defeito simétrico seria uma cena cujas peças **não se sobrepõem**: o interruptor não
//! faria nada, e ela ensinaria o contrário do que acontece.

use crate::motion_state::MotionState;
use ph2d_contact::Colisor;
use ph2d_nodegraph::attr::{Column, Stream};

/// Coze a cena e devolve a corrente que o sink desenha.
///
/// ⛔⛔ **A 1.ª redacção lia ZERO peças, e a causa está escrita no cabeçalho da cena irmã:** o
/// `source.shape` não carrega a geometria dele — ela é um EXTERNO que a shell publica
/// (`motion_shape_gen::publish`), e *num cozedor virgem aquele nó emite zero*. ⇒ o cozimento tem de
/// correr no `pump.cook`, que é onde os externos vivem, e depois do `publish`.
///
/// ⚠️ É a mesma família da 1.ª mentira do arnês do §11.2 — *um documento vazio lê-se como um
/// documento barato*, e as duas vezes o que a curou foi o **piso de população** abaixo.
fn cozida(m: &mut MotionState) -> Stream {
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    let sink = *sinks.first().expect("um sink");
    crate::motion_shape_gen::publish(m, 0.0);
    m.pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("a cena coze")[0]
        .as_stream()
        .clone()
}

/// Quantos pares se atravessam de facto — a mesma régua do doc 115 §9.1 (a penetração VISÍVEL, e
/// não `contato(..).is_some()`, que conta os pares que só se TOCAM e acusaria a própria convergência).
fn pares_atravessados(p: &[[f32; 2]], c: &[Option<Colisor>]) -> usize {
    const VISIVEL: f32 = 0.02;
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

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a cena tem de emitir P"),
    }
}

/// ⭐⭐⭐ **A CENA ENSINA O QUE DIZ: desarmada as peças ATRAVESSAM-SE, armada elas ASSENTAM.**
///
/// ⚠️ **As duas metades, e a primeira é a que uma leitura rápida salta:** sem o amontoado a cena
/// não tem nada para o interruptor curar, e o passo (1) do roteiro seria uma mentira. *Uma cena de
/// smoke que ensina o contrário do que acontece é pior que uma cena ausente* (§5.0).
#[test]
fn a_cena_amontoa_as_pecas_e_o_interruptor_separa_as() {
    let mut m = MotionState::new();
    let s = cozida(&mut m);
    let n = s.count();
    assert_eq!(
        n,
        super::PECAS,
        "a fileira tem de carimbar {} peças",
        super::PECAS
    );
    let c = ph2d_contact::colisores(&s).expect("as peças DECLARAM a caixa delas");
    let p = posicoes(&s);

    let antes = pares_atravessados(&p, &c);
    assert!(
        antes >= n / 2,
        "a cena tem de NASCER amontoada — {antes} pares atravessados em {n} peças"
    );

    // ⭐⭐⭐ **As varreduras são as de FÁBRICA e não um número escolhido para o gate passar.** É o
    // que o artista recebe ao ligar o interruptor sem tocar em mais nada, e é a única barra que diz
    // alguma coisa sobre o smoke. ⛔ Uma cena que só assenta com o knob no máximo é uma cena que o
    // dono reprova.
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "o default e' um inteiro pequeno e positivo, declarado no manifesto"
    )]
    let de_fabrica = ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_DEFAULT as usize;
    let separado = ph2d_contact::passe::separa_o_que_se_desenha(&s, de_fabrica)
        .expect("o passe tem o que separar");
    let depois = pares_atravessados(&posicoes(&separado), &c);
    assert_eq!(
        depois, 0,
        "com as {de_fabrica} varreduras de FABRICA, nenhuma peça pode continuar a atravessar \
         outra (eram {antes})"
    );
}

/// ⛔⛔ **NENHUM nó de colisão no grafo, e é isso que a cena existe para mostrar.**
///
/// ⚠️ Sem este gate, alguém «arranja» a cena acrescentando um `motion.collide` no dia em que ela
/// parecer partida — e ela passa a demonstrar exactamente o contrário da ordem do dono.
#[test]
fn o_grafo_da_cena_nao_tem_no_de_colisao_nenhum() {
    let mut m = MotionState::new();
    super::build(&mut m.doc, &m.registry).expect("a cena monta");
    for n in m.doc.graph.nodes() {
        assert!(
            !matches!(n.type_name.as_str(), "motion.collide" | "sim.collide"),
            "a cena do PASSE nao pode ter `{}` — o assunto dela e' nao haver nenhum",
            n.type_name
        );
    }
}

/// ⚠️ **A cena nasce DESARMADA** — o primeiro quadro é o amontoado, senão o artista vê o resultado
/// e nunca a causa.
#[test]
fn a_cena_nasce_com_o_interruptor_desligado() {
    let mut m = MotionState::new();
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    assert_eq!(
        ph2d_eval_motion::sink_collide_sweeps(&m.doc.graph, sinks[0]),
        0,
        "a cena tem de abrir desarmada"
    );
}

/// ⭐ **O roteiro nomeia controlos que EXISTEM.** *Um passo que manda ligar `Collide` no cartão do
/// Output AFIRMA que essa linha está lá*, e ela envelhece sozinha no dia em que o param mudar de
/// rótulo — a lei que o tutorial do `#15` já paga na `line/components`.
#[test]
fn o_roteiro_nomeia_controlos_que_existem() {
    const ROTEIRO: &str = include_str!("motion_state_passe_demo.rs");
    let m = MotionState::new();
    // ⭐ Os rótulos saem do REGISTO, que é de onde o painel os pinta — uma lista escrita à mão
    // aqui seria a segunda resposta à mesma pergunta, e divergiria no dia do renome.
    let hints = m
        .registry
        .param_ui(ph2d_nodegraph::node::NodeTypeId::of("motion.output"))
        .expect("o Output declara hints");
    let rotulos: Vec<&str> = hints.iter().map(|h| ph2d_i18n::tr(h.label)).collect();
    for citado in ["Collide", "Collide Sweeps"] {
        assert!(
            rotulos.contains(&citado),
            "o roteiro manda ligar `{citado}` e o cartao do Output nao tem essa linha: {rotulos:?}"
        );
        assert!(
            ROTEIRO.contains(citado),
            "o gate cita `{citado}` e o roteiro nao — a regua deixou de medir o roteiro"
        );
    }
}
