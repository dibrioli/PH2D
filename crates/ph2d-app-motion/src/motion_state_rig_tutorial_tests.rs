//! Os gates do TUTORIAL do ciclo 9 (`09_coisas_que_se_seguram`) — irmão dos da cena `=120` pelo
//! teto de 700 LOC, e o corte é por RESPONSABILIDADE: lá *o que a cena faz*, aqui *o que o PDF
//! afirma dela* (os cartões, as linhas e as figuras que o texto nomeia).

use crate::motion_state::MotionState;

/// A FONTE do tutorial deste ciclo — lida, para que os nomes do gate e os do texto não possam
/// divergir em silêncio.
const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/09_coisas_que_se_seguram.html");

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (ciclo 9, passo 7 — doc 103 §1).
///
/// ⛔⛔ **Um passo que manda clicar numa linha AFIRMA que ela está no cartão** — e o modo de falha
/// é o pior possível: o dono procura, não encontra, e conclui que o programa está partido.
///
/// ⚠️⚠️ **E este ciclo tem uma armadilha que os anteriores não tinham: a cena tem QUATRO
/// `motion.drive`** (o ângulo do FK, o ângulo da pose de cada uma das duas peles, e o quinhão por
/// osso). *Um passo que diz «o cartão Drive» num grafo com quatro é um passo que o dono não
/// consegue executar* ⇒ os dois que o tutorial nomeia têm rótulo próprio, e é pelo rótulo que
/// este gate os procura.
#[test]
fn every_row_the_rig_tutorial_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("120", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let pedidos: &[(&str, &[&str])] = &[
        ("Verlet Rope: a corda", &["Gravity"]),
        (
            "Wave: o campo",
            // ⚠️⚠️ **`Height Drives` e não `Height Channel`** — o param chama-se `height_channel`
            // e o RÓTULO que o artista lê é outro. *Foi este gate que o apanhou*, sobre um
            // tutorial já impresso em PDF: o passo mandava procurar uma linha que a tela nunca
            // mostrou, e o modo de falha é o dono a concluir que o programa está partido.
            &["Height Drives", "Rows", "Cols", "Spacing"],
        ),
        ("Strength: quanto a restricao puxa", &["Radius"]),
        ("Drive: o angulo de cada junta", &["Scale"]),
        ("Range: que ossos puxam", &["End"]),
    ];
    for (titulo, linhas) in pedidos {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == *titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!("o tutorial nomeia o cartao `{titulo}` e a cena tem {havia:?}")
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        for l in *linhas {
            assert!(
                rows.contains(l),
                "o tutorial manda mexer em `{l}` no cartao `{titulo}`, e o cartao mostra {rows:?}"
            );
            assert!(
                TUTORIAL.contains(&format!("<code>{l}</code>")),
                "o gate defende a linha `{l}` e o tutorial nunca a nomeia"
            );
        }
        assert!(
            TUTORIAL.contains(titulo),
            "o gate defende o cartao `{titulo}` e o tutorial nunca o nomeia"
        );
    }
}

/// ⭐ **CADA FIGURA QUE O TUTORIAL MOSTRA EXISTE.**
///
/// ⛔ Uma `<img>` partida num PDF é um rectângulo vazio com uma legenda por baixo: o leitor lê a
/// legenda e acredita nela. *A legenda é a afirmação; a figura é a prova, e sem ela sobra só a
/// afirmação.*
#[test]
fn every_figure_the_rig_tutorial_shows_exists() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    let mut n = 0;
    for pedaco in TUTORIAL.split("src=\"../fig/").skip(1) {
        let nome = pedaco.split('"').next().expect("o nome da figura");
        assert!(
            dir.join(nome).is_file(),
            "o tutorial mostra `{nome}` e a figura nao existe — corra `write_the_rig_figures`"
        );
        n += 1;
    }
    assert_eq!(n, 6, "o tutorial tem seis figuras, achei {n}");
}

/// ⭐⭐ **O TUTORIAL NOMEIA A CENA CERTA** — e o roteador tem esse nível.
///
/// ⛔ Um tutorial que manda correr `=119` sobre a cena do ciclo 9 abre a cena de OUTRO ciclo, e o
/// dono passa o smoke inteiro a procurar panos que não existem.
#[test]
fn the_rig_tutorial_opens_the_scene_this_cycle_built() {
    assert!(
        TUTORIAL.contains("PH2D_GPU_COOK_DEMO=120"),
        "o tutorial tem de abrir a cena `=120`"
    );
    // ⚠️ **O TECTO é `const`, então a metade dele é de COMPILAÇÃO** — baixar o `MAX_DEMO_LEVEL`
    // deixa de compilar, que é mais forte do que reprovar.
    const _: () = assert!(crate::motion_state::demo_router::MAX_DEMO_LEVEL >= 120);
    // ⭐ E a metade que MEDE: o nível tem de estar na TABELA das cenas de ciclo — um tecto alto
    // com a tabela sem a linha abre o canvas em branco.
    assert!(
        crate::motion_state::demo_router::is_cycle_scene("120"),
        "o `120` tem de estar na tabela das cenas de ciclo"
    );
}

/// ⭐⭐⭐ **OS NÚMEROS DA SECÇÃO 7 SÃO OS QUE A MEDIÇÃO DEU** — e este gate é o que impede que eles
/// envelheçam em silêncio.
///
/// ⚠️⚠️ **Uma tabela de custo num PDF é a afirmação mais fácil de deixar apodrecer do repo:** ela
/// não compila, não corre e ninguém a relê. Aqui ela é **derivada** da mesma fonte que a §7 do doc
/// 114 — as constantes abaixo — e o gate exige que cada uma apareça no texto.
///
/// ⛔ **Ele NÃO re-mede:** medir num gate faria dele mais um membro da família de flakes de carga
/// (`CLAUDE.md` §5.0), e o que se defende aqui é a HONESTIDADE do texto, não o relógio.
#[test]
fn the_cost_table_the_rig_tutorial_prints_is_the_one_that_was_measured() {
    // As leituras da §7 do doc 114 (RELEASE, `load 22,88`, 2026-09-17).
    let medidos: &[(&str, &str)] = &[
        ("0,010", "o campo a 60x60"),
        ("1,32", "o campo a 512x512"),
        ("2,26", "o corpo mole a 512x512"),
        ("13,77", "o bando de 3 600 na CPU"),
        ("82,6", "a fraccao de um quadro que o bando come"),
    ];
    for (n, o_que) in medidos {
        assert!(
            TUTORIAL.contains(n),
            "a seccao 7 tem de trazer `{n}` ({o_que}) — o numero saiu da medicao, nao de memoria"
        );
    }
    // ⭐ E a frase que dá sentido ao maior deles: sem ela `13,77 ms` lê-se como *«este grupo é
    // lento»*, quando é o preço de o ÚNICO nó com placa não a estar a usar.
    assert!(
        TUTORIAL.contains("única") || TUTORIAL.contains("único"),
        "a seccao 7 tem de dizer que o `Boids` e' o UNICO do grupo com caminho na placa — \
         sem isso o numero grande lê-se como um defeito do grupo inteiro"
    );
}
