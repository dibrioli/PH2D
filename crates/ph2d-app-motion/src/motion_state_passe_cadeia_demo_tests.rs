//! Os gates da cena `=123` — A CADEIA (doc 115 §18).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu** — as peças são `source.shape`, que lê um EXTERNAL
//! publicado por `motion_shape_gen::publish`; num cozedor virgem ele emite **zero** e os gates
//! mediriam o vazio. É o precedente que a `=110`, a `=114`, a `=121` e a `=122` já nomeiam.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// Cozinha a cena uma vez e devolve a corrente do sink. ⭐ **Um instante só, e é de propósito:**
/// esta cena não se mexe (ver o cabeçalho do módulo) — o que ela afirma é a ESCADA, e medir cinco
/// instantes de uma cena parada mediria a mesma coisa cinco vezes.
fn corre() -> Stream {
    let mut m = MotionState::new();
    assert!(
        m.doc.graph.nodes().is_empty(),
        "o `MotionState` nasceu com nos: desligue o PH2D_GPU_COOK_DEMO para correr estes gates"
    );
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    m.pump
        .cook
        .cook(&m.doc.graph, &m.registry, sinks[0], 0.0)
        .expect("a cena cozinha")[0]
        .as_stream()
        .clone()
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a cena tem de emitir P"),
    }
}

/// Quantos pares se atravessam de forma VISÍVEL — a mesma régua das cenas irmãs (`2 %` do lado), e
/// não `contato(..).is_some()`, que conta os pares que apenas se TOCAM e acusaria a convergência.
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

/// ⭐⭐⭐ **A ESCADA — que é a cena inteira.** O que o dono vê nos passos (2) a (4) do roteiro:
/// ligar o interruptor abre a cadeia um pouco, o tecto HERDADO não a fecha, e o tecto do SLIDER
/// fecha-a.
///
/// ⚠️⚠️ **As quatro metades são o gate, e nenhuma sobra:**
/// - sem a fixtura CONTER o fenómeno, todas as outras são verdes por vácuo;
/// - sem o degrau de fábrica ainda atravessar, o passo (3) do roteiro ensinaria o contrário;
/// - sem o `64` ainda atravessar, esta cena passaria com o tecto ANTIGO — e *um gate que passa com
///   o número antigo não mediu a mudança*;
/// - sem o `1024` fechar, a cena não tem o que mostrar.
#[test]
fn a_escada_das_varreduras_e_o_que_esta_cena_mostra() {
    let cru = corre();
    let antes = pares_atravessados(&cru);
    assert!(
        antes >= super::PECAS,
        "a fixtura tem de CONTER o fenomeno — a cadeia crua traz {antes} pares atravessados"
    );
    let fica = |v: usize| {
        ph2d_contact::passe::separa_o_que_se_desenha(&cru, v)
            .map_or(antes, |s| pares_atravessados(&s))
    };

    // ⭐ O tecto do SLIDER lê-se do REGISTO — é o número que o CARTÃO oferece, que é exactamente o
    // que o roteiro promete. ⛔ Uma segunda cópia da const seria a divergência que o gate da shell
    // existe para impedir, um nível acima.
    let m = MotionState::new();
    let slider_max = m
        .registry
        .param_ui(ph2d_nodegraph::node::NodeTypeId::of("motion.output"))
        .and_then(|h| {
            h.iter()
                .find(|x| ph2d_i18n::tr(x.label) == "Collide Sweeps")
                .map(|x| x.max)
        })
        .expect("o cartao do sink oferece `Collide Sweeps`");
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "os dois sao inteiros pequenos e positivos, declarados no manifesto"
    )]
    let (fabrica, slider) = (
        ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_DEFAULT as usize,
        slider_max as usize,
    );

    let de_fabrica = fica(fabrica);
    assert!(
        de_fabrica < antes,
        "ligar o interruptor tem de ABRIR a cadeia: {antes} -> {de_fabrica}"
    );
    assert!(
        de_fabrica > 0,
        "e a {fabrica} varreduras ela NAO pode fechar — o passo (3) do roteiro promete que nao"
    );
    let no_herdado = fica(64);
    assert!(
        no_herdado > 0,
        "CONTROLO DO TECTO ANTIGO: a 64 varreduras (o herdado, ate' 18/09) a cadeia ainda se \
         atravessa — se fecha, esta cena deixou de medir a mudanca do doc 115 §18"
    );
    assert_eq!(
        fica(slider),
        0,
        "e a {slider} (o topo do slider de hoje) ela tem de FECHAR — e' o passo (4) do roteiro"
    );
}

/// ⚠️ **A cena nasce DESARMADA** — nenhum override no sink. Sem isto o primeiro quadro mostrava o
/// resultado e nunca a causa, e a cena deixava de poder divergir do valor de fábrica no dia em que
/// ele mudasse.
#[test]
fn a_cadeia_nasce_com_o_interruptor_desligado_e_com_as_varreduras_de_fabrica() {
    let mut m = MotionState::new();
    let sinks = super::build(&mut m.doc, &m.registry).expect("a cena monta");
    assert_eq!(
        ph2d_eval_motion::sink_collide_sweeps(&m.doc.graph, sinks[0]),
        0,
        "a cena nao pode autorar o interruptor — quem liga e' o artista"
    );
    // ⚠️ **E o CONTROLO: armado, o número que corre é o de FÁBRICA** — é isso que prova que a cena
    // não autorou as varreduras. Sem esta metade, uma cena que escrevesse `collide_iterations`
    // passaria na de cima (o interruptor continua desligado) e mentiria no dia em que o valor de
    // fábrica mudasse.
    m.doc
        .graph
        .set_param(sinks[0], ph2d_eval_motion::SINK_COLLIDE_PARAM, 1.0);
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "o default e' um inteiro pequeno e positivo"
    )]
    let fabrica = ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_DEFAULT as usize;
    assert_eq!(
        ph2d_eval_motion::sink_collide_sweeps(&m.doc.graph, sinks[0]),
        fabrica,
        "armada, a cena tem de correr as varreduras de FABRICA — ela nao pode autorar o numero"
    );
}

/// ⛔ **Nenhum nó de colisão em lado nenhum** — o assunto da wave é o app separar SOZINHO.
#[test]
fn a_cena_nao_tem_colisor_nenhum() {
    let mut m = MotionState::new();
    super::build(&mut m.doc, &m.registry).expect("a cena monta");
    for n in m.doc.graph.nodes() {
        let t = n.type_name.as_str();
        assert!(
            !t.contains("collide") && !t.contains("contact"),
            "a cena montou um `{t}` — ela existe para provar que nao precisa de nenhum"
        );
    }
}

/// ⚠️ **O roteiro nomeia controlos que EXISTEM** — um passo que manda mexer num knob é uma
/// AFIRMAÇÃO de que ele está na tela, e ela envelhece sozinha.
#[test]
fn o_roteiro_nomeia_controlos_que_existem() {
    const ROTEIRO: &str = include_str!("motion_state_passe_cadeia_demo.rs");
    let m = MotionState::new();
    for (tipo, citado) in [
        ("motion.output", "Collide"),
        ("motion.output", "Collide Sweeps"),
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
    // ⭐ E os DOIS números que o roteiro promete são os do produto — senão ele ensina uma escada
    // que o slider não tem.
    let slider_max = m
        .registry
        .param_ui(ph2d_nodegraph::node::NodeTypeId::of("motion.output"))
        .and_then(|h| {
            h.iter()
                .find(|x| ph2d_i18n::tr(x.label) == "Collide Sweeps")
                .map(|x| x.max)
        })
        .expect("o cartao do sink oferece `Collide Sweeps`");
    for n in [slider_max, ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_MAX] {
        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "os dois tectos sao inteiros pequenos e positivos"
        )]
        let texto = (n as usize).to_string();
        assert!(
            ROTEIRO.contains(&texto),
            "o roteiro tem de dizer `{texto}` — ele ensina a escada, e a escada e' a do produto"
        );
    }
}
