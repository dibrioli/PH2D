//! Os gates da LEI do gatilho. ⚠️ A ponte (quem lê o teclado a sério) tem os dela na shell.

use super::*;
use crate::StableId;

/// A amostra de uma tecla que está em baixo há mais de um quadro.
const SEGURADA: Amostra = Amostra {
    pressed: true,
    just_pressed: false,
    just_released: false,
};
/// O quadro em que ela desceu.
const DESCEU: Amostra = Amostra {
    pressed: true,
    just_pressed: true,
    just_released: false,
};
/// O quadro em que ela subiu.
const SUBIU: Amostra = Amostra {
    pressed: false,
    just_pressed: false,
    just_released: true,
};
/// Solta há muito.
const SOLTA: Amostra = Amostra {
    pressed: false,
    just_pressed: false,
    just_released: false,
};

fn linha(edge: ActionEdge) -> ActionTriggerRow {
    ActionTriggerRow {
        action: "fire".into(),
        edge,
        signal: "shoot".into(),
    }
}

/// ⭐⭐⭐ **A razão de o `Press` ser o valor de fábrica**, e o CONTROLO que a torna legível: o
/// `Hold` fala nos mesmos quadros em que o `Press` se cala.
///
/// ⚠️ **Sem a metade do `Hold` este gate passaria sobre uma lei que nunca dispara** — *uma régua
/// que só vê a ausência não prova que a presença é possível*.
///
/// ⛔⛔ **E a 1.ª redacção CONTAVA e uma mutação sobreviveu-lhe:** ela afirmava *«o `Release` fala
/// UMA vez»*, e trocar a lei dele para `just_pressed` também fala uma vez — no quadro **errado**.
/// *Uma régua que conta QUANTOS nunca vê QUAIS* (a mesma forma que o pincel de contorno pagou,
/// §54 do `sculpt3d`) ⇒ o que se afirma é o PERFIL do toque, quadro a quadro.
#[test]
fn press_fala_uma_vez_por_toque_e_hold_fala_sempre() {
    // Um toque de quatro quadros: solta, desce, segura, sobe.
    let toque = [SOLTA, DESCEU, SEGURADA, SUBIU];
    let perfil = |edge| {
        let row = linha(edge);
        toque.map(|a| fala(&row, a))
    };
    assert_eq!(
        perfil(ActionEdge::Press),
        [false, true, false, false],
        "o Press fala SÓ no quadro em que ela desce"
    );
    assert_eq!(
        perfil(ActionEdge::Hold),
        [false, true, true, false],
        "o Hold fala nos dois quadros em que ela está em baixo"
    );
    assert_eq!(
        perfil(ActionEdge::Release),
        [false, false, false, true],
        "o Release fala SÓ no quadro em que ela sobe"
    );
}

/// ⚠️⚠️ **A acção que não existe fica CALADA — nas TRÊS arestas.**
///
/// A ponte devolve [`Amostra::default`] para um nome que o mapa não conhece (as três leituras da
/// `ph2d_input::Input` são `is_some_and`). O defeito mudo que isto impede é um `Release` a disparar
/// **em todo quadro** sobre uma acção que ninguém ligou — porque `!pressed` é trivialmente verdade.
#[test]
fn uma_accao_que_nao_existe_fica_calada_nas_tres_arestas() {
    for e in ActionEdge::ALL {
        assert!(
            !fala(&linha(e), Amostra::default()),
            "a aresta {e:?} falou sobre uma acção que o mapa não conhece"
        );
    }
}

/// A linha sem nome de sinal segue a tecla e fica calada — com o CONTROLO de que a mesma amostra
/// com nome fala.
#[test]
fn uma_linha_sem_nome_de_sinal_fica_calada() {
    let mut muda = linha(ActionEdge::Press);
    muda.signal = "   ".into();
    assert!(!fala(&muda, DESCEU), "uma linha sem nome publicou um sinal");
    assert!(
        fala(&linha(ActionEdge::Press), DESCEU),
        "o controlo: a MESMA amostra com nome tem de falar"
    );
}

/// ⚠️ **A ordem é a da identidade, nunca a do mundo.** A fixtura spawna a identidade `9` ANTES da
/// `2`, que é o que torna o gate observável — com as duas por ordem crescente ele passaria mesmo
/// sem a ordenação.
#[test]
fn os_disparos_saem_pela_ordem_da_identidade() {
    let mut w = World::new();
    for id in [9_u64, 2] {
        w.spawn((
            StableId(id),
            SignalOnAction(vec![ActionTriggerRow {
                action: "fire".into(),
                edge: ActionEdge::Press,
                signal: format!("tiro{id}"),
            }]),
        ));
    }
    let disparos = dispara(&mut w, &|_| DESCEU);
    let nomes: Vec<&str> = disparos.iter().map(|d| d.signal.as_str()).collect();
    assert_eq!(
        nomes,
        ["tiro2", "tiro9"],
        "os disparos têm de sair pela ordem da identidade"
    );
}

/// ⭐ **Cada linha fala pela SUA acção** — o gate que uma leitura única do teclado deixaria passar.
#[test]
fn cada_linha_le_a_propria_accao() {
    let mut w = World::new();
    w.spawn((
        StableId(1),
        SignalOnAction(vec![
            ActionTriggerRow {
                action: "fire".into(),
                edge: ActionEdge::Press,
                signal: "tiro".into(),
            },
            ActionTriggerRow {
                action: "jump".into(),
                edge: ActionEdge::Press,
                signal: "salto".into(),
            },
        ]),
    ));
    let disparos = dispara(&mut w, &|nome| {
        if nome == "fire" { DESCEU } else { SOLTA }
    });
    assert_eq!(disparos.len(), 1, "só a linha do `fire` devia falar");
    assert_eq!(disparos[0].signal, "tiro");
    assert_eq!(disparos[0].row, 0, "a linha que falou é a de índice 0");
}

/// ⛔⛔ **O CENSO da ausência: este componente NÃO tem estado vivo.**
///
/// As cinco irmãs registadas desta linha têm um `*Runtime` não-registado e uma entrada no
/// [`crate::rewind_runtime`]. Esta não tem nenhum dos dois **de propósito** (a aresta é trabalho do
/// input), e sem este gate a ausência lê-se como esquecimento — que é exactamente o report que o
/// `#14` pagou quando o `projectile_state` ficou fora do `rebuild_from_rest`.
///
/// ⚠️ **Ele mede o FICHEIRO da lei e o do renascer**, com piso de população nos dois: uma varredura
/// partida devolve zero e lê-se como aprovada.
///
/// ⛔⛔ **E ele reprovou na 1.ª corrida sobre produto CERTO** — porque o doc-comment da lei
/// **explica** que não há `SignalOnActionRuntime`, e a régua lia a explicação como a coisa. É a
/// família que este repo já tem escrita (*«uma régua textual a varrer `\bApp\b` lê o doc-comment
/// que EXPLICA a cura e acusa 93 ficheiros de 133»*, W2/L2 Fase B) ⇒ **a prosa sai primeiro**, e o
/// piso de população passa a ser medido DEPOIS do corte, senão apagar todo o código deixaria o
/// gate verde com o cabeçalho a segurar o número.
#[test]
fn o_gatilho_nao_guarda_estado_e_isso_e_declarado() {
    /// Só o CÓDIGO: um comentário que descreve a ausência não é a presença dela.
    fn sem_prosa(src: &str) -> String {
        src.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    let lei = sem_prosa(include_str!("signal_on_action.rs"));
    let renascer = sem_prosa(include_str!("rewind_runtime.rs"));
    assert!(
        lei.len() > 1_000 && renascer.len() > 1_000,
        "piso de população DEPOIS do corte da prosa: {} e {} bytes de código",
        lei.len(),
        renascer.len()
    );
    assert!(
        !lei.contains("SignalOnActionRuntime"),
        "nasceu um estado vivo: ou a lei mudou, ou este gate tem de ser reescrito com o porquê"
    );
    assert!(
        !renascer.contains("SignalOnAction"),
        "o gatilho entrou no renascer — ele não tem o que repor, e entrar lá seria a 2.ª resposta \
         a «ela já estava premida?»"
    );
    // O controlo positivo: as irmãs que TÊM estado vivo estão lá.
    assert!(
        renascer.contains("Timer") && renascer.contains("Factory"),
        "controlo: o ficheiro do renascer deixou de nomear as irmãs — a varredura mede outra coisa"
    );
}
