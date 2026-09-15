//! Gates da lei do `StateMachine` — **as SETE leis do oráculo**, mais as que são nossas e
//! declaradas. O corpus medido está em `docs/Components/12_plano_state_machine.md` §2.

use super::*;

fn estado(nome: &str, entra: &str, sai: &str) -> MachineState {
    MachineState {
        name: nome.into(),
        on_enter: entra.into(),
        on_exit: sai.into(),
    }
}

fn seta(from: u8, on: &str, to: u8) -> Transition {
    Transition {
        from,
        on: on.into(),
        to,
    }
}

/// Uma porta: `Fechada → A abrir → Aberta`, e uma seta de volta.
fn porta() -> StateMachine {
    StateMachine {
        states: vec![
            estado("Fechada", "porta_fechada", ""),
            estado("A abrir", "porta_a_abrir", ""),
            estado("Aberta", "porta_aberta", "porta_sai_aberta"),
        ],
        transitions: vec![seta(0, "botao", 1), seta(1, "fim", 2), seta(2, "botao", 0)],
        initial: 0,
    }
}

/// ⭐⭐ **Entrar no inicial é ENTRAR — e uma vez só.**
#[test]
fn entrar_no_estado_inicial_anuncia_se_e_nao_se_repete() {
    let m = porta();
    let mut rt = born(&m);
    let a = advance(&m, &mut rt, &[]);
    assert_eq!(a.emitted, vec!["porta_fechada".to_string()]);
    assert_eq!(a.steps, 0, "anunciar a entrada nao e' atravessar uma seta");
    let b = advance(&m, &mut rt, &[]);
    assert!(
        b.emitted.is_empty(),
        "so' se entra uma vez: {:?}",
        b.emitted
    );
}

/// ⭐⭐⭐ **O sinal move a máquina, e sair e entrar são NOMES diferentes.**
#[test]
fn um_sinal_move_a_maquina_e_anuncia_a_saida_e_a_entrada() {
    let m = porta();
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]); // consome a entrada no inicial
    let a = advance(&m, &mut rt, &["botao"]);
    assert_eq!(rt.current, 1, "«Fechada» + botao ⇒ «A abrir»");
    assert_eq!(a.steps, 1);
    // A «Fechada» é calada ao sair (`on_exit` vazio) ⇒ só a entrada fala.
    assert_eq!(a.emitted, vec!["porta_a_abrir".to_string()]);

    let b = advance(&m, &mut rt, &["fim"]);
    assert_eq!(rt.current, 2);
    assert_eq!(b.emitted, vec!["porta_aberta".to_string()]);

    // E de «Aberta» a saída FALA, então a ordem é saída → entrada.
    let c = advance(&m, &mut rt, &["botao"]);
    assert_eq!(rt.current, 0);
    assert_eq!(
        c.emitted,
        vec!["porta_sai_aberta".to_string(), "porta_fechada".to_string()],
        "a saida vem ANTES da entrada — e sao dois nomes, nunca um campo de fase"
    );
}

/// ⭐⭐⭐ **A ARBITRAGEM: a primeira satisfeita ganha** (divergência declarada do oráculo, que tem
/// um campo `priority` e desempata pela ÚLTIMA declarada).
///
/// **Mutação que deve sangrar:** trocar o `.find(` por um `.rev().find(`.
#[test]
fn a_primeira_transicao_satisfeita_ganha() {
    let m = StateMachine {
        states: vec![
            estado("A", "", ""),
            estado("B", "", ""),
            estado("C", "", ""),
        ],
        // As DUAS ouvem o mesmo sinal. A ordem da tabela é a precedência.
        transitions: vec![seta(0, "s", 1), seta(0, "s", 2)],
        initial: 0,
    };
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    advance(&m, &mut rt, &["s"]);
    assert_eq!(
        rt.current, 1,
        "ganha a PRIMEIRA linha, que e' a que o artista ve' em cima"
    );
}

/// ⭐⭐⭐ **Uma cadeia resolve-se NUM tique — quando os sinais lá estão.**
///
/// É a lei 5 do oráculo (uma cadeia `A→B→C→D→E` atravessa-se inteira num avanço), portada para uma
/// entrada que é **evento** e não nível: lá as condições ficam verdadeiras, aqui cada hop gasta o
/// seu sinal.
///
/// **Mutação que deve sangrar:** trocar o `loop` por um único passo.
#[test]
fn uma_cadeia_resolve_se_num_tique_quando_os_sinais_la_estao() {
    let m = StateMachine {
        states: (0..5)
            .map(|i| estado(&format!("S{i}"), &format!("entrou{i}"), ""))
            .collect(),
        transitions: (0..4).map(|i| seta(i, &format!("s{i}"), i + 1)).collect(),
        initial: 0,
    };
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    let a = advance(&m, &mut rt, &["s0", "s1", "s2", "s3"]);
    assert_eq!(rt.current, 4, "A→B→C→D→E num avanco so'");
    assert_eq!(a.steps, 4);
    assert_eq!(a.emitted, vec!["entrou1", "entrou2", "entrou3", "entrou4"]);
}

/// ⭐⭐⭐ **UM SINAL É GASTO por quem o ouve** — a lei que um gate vermelho impôs ao meu desenho.
///
/// ⚠️⚠️ **Sem ela, uma porta `Aberta` ia a `Fechada` e logo a `A abrir` com UM toque**, porque a
/// mesma seta de `botao` que a fecha é ouvida outra vez pelo estado onde ela acabou de entrar. É a
/// diferença entre a entrada do oráculo (um **nível**, que fica verdadeiro) e a nossa (um
/// **evento**, que acontece).
///
/// **Mutação que deve sangrar:** apagar o `gasto[i] = true;`. ⛔ Ela **pendura** — e é por isso que
/// este gate conta os passos em vez de olhar só o estado final.
#[test]
fn um_sinal_e_gasto_por_quem_o_ouve_e_um_toque_da_um_passo() {
    let m = porta();
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    advance(&m, &mut rt, &["botao"]); // Fechada → A abrir
    advance(&m, &mut rt, &["fim"]); // A abrir → Aberta
    let a = advance(&m, &mut rt, &["botao"]); // Aberta → Fechada, e PARA
    assert_eq!(a.steps, 1, "um toque = um passo");
    assert_eq!(
        rt.current, 0,
        "a porta fecha e FICA fechada — com o sinal a valer duas vezes ela reabria no mesmo tique"
    );

    // E o MESMO sinal duas vezes no mesmo tique move duas vezes: «aconteceu duas vezes» é isso.
    let mut rt2 = born(&m);
    advance(&m, &mut rt2, &[]);
    let b = advance(&m, &mut rt2, &["botao", "fim"]);
    assert_eq!(b.steps, 2, "dois eventos distintos, dois passos");
    assert_eq!(rt2.current, 2);
}

/// ⭐⭐ **Um CICLO não pendura** — a lei 6 do oráculo, que aqui cai de graça: o laço termina porque
/// a lista de sinais é **finita**, e não por um `MAX_DEPTH`.
///
/// ⚠️ **A fixtura fecha o ciclo com DOIS sinais no mesmo tique**, que é a única forma de o
/// exercitar depois do consumo — com um só, o consumo já o pára ao primeiro passo.
#[test]
fn um_ciclo_termina_porque_os_sinais_acabam() {
    let m = StateMachine {
        states: vec![estado("A", "", ""), estado("B", "", "")],
        transitions: vec![seta(0, "s", 1), seta(1, "t", 0)],
        initial: 0,
    };
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    let a = advance(&m, &mut rt, &["s", "t", "s", "t"]);
    assert_eq!(a.steps, 4, "quatro eventos, quatro passos — e depois PARA");
    assert_eq!(rt.current, 0);

    // E um tique com um sinal só anda **um** passo, como o oráculo mede — depois o estado onde
    // ele aterrou já não escuta aquele nome, e o tique seguinte não move nada.
    let mut rt2 = born(&m);
    advance(&m, &mut rt2, &[]);
    assert_eq!(advance(&m, &mut rt2, &["s"]).steps, 1);
    assert_eq!(rt2.current, 1);
    assert_eq!(
        advance(&m, &mut rt2, &["s"]).steps,
        0,
        "o «B» escuta `t`, nao `s` — um sinal que ninguem escuta nao move nada"
    );
}

/// ⛔ **Uma transição para SI MESMO é recusada** — a lei 7 do oráculo (`has_transition` = `false`).
#[test]
fn uma_transicao_para_si_mesmo_e_recusada() {
    let m = StateMachine {
        states: vec![estado("A", "entrouA", "saiuA")],
        transitions: vec![seta(0, "s", 0)],
        initial: 0,
    };
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    let a = advance(&m, &mut rt, &["s"]);
    assert_eq!(a.steps, 0, "entrar onde ja' se esta' nao e' entrar");
    assert!(a.emitted.is_empty(), "e nao anuncia nada: {:?}", a.emitted);
}

/// ⛔ **Um sinal que ninguém escuta não move nada**, e **um nome vazio nunca dispara** — o espelho
/// da lei do produtor.
#[test]
fn um_sinal_desconhecido_e_uma_seta_sem_nome_nao_movem_nada() {
    let mut m = porta();
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    assert_eq!(advance(&m, &mut rt, &["outra_coisa"]).steps, 0);

    m.transitions[0].on = String::new();
    let mut rt2 = born(&m);
    advance(&m, &mut rt2, &[]);
    assert_eq!(
        advance(&m, &mut rt2, &[""]).steps,
        0,
        "uma seta sem nome NUNCA dispara — nem com um sinal de nome vazio"
    );
}

/// ⚠️ **Um índice que aponta para fora não é um caso especial a lembrar: é um estado que não pode
/// existir.** Apagar estados recua ao inicial.
#[test]
fn apagar_estados_recua_ao_inicial_em_vez_de_apontar_para_fora() {
    let m = porta();
    let mut rt = born(&m);
    advance(&m, &mut rt, &[]);
    advance(&m, &mut rt, &["botao"]);
    advance(&m, &mut rt, &["fim"]);
    assert_eq!(rt.current, 2);

    let mut curta = m.clone();
    curta.states.truncate(1);
    assert!(reconcile(&curta, &mut rt), "ele tem de mexer");
    assert_eq!(rt.current, 0);
    assert!(
        !rt.started,
        "renascer e' renascer: ele volta a anunciar a entrada"
    );
    assert!(!reconcile(&curta, &mut rt), "e a segunda vez nao mexe");
}

/// ⚠️ **Uma máquina VAZIA não parte nada** — o artista anexa o componente antes de escrever nele.
#[test]
fn uma_maquina_vazia_nao_parte_nada() {
    let m = StateMachine::default();
    let mut rt = born(&m);
    let a = advance(&m, &mut rt, &["s"]);
    assert_eq!(a, Advanced::default());
}

/// ⚠️ **Um `initial` fora de alcance nasce no 0** em vez de estourar — a mesma lei do `reconcile`.
#[test]
fn um_inicial_fora_de_alcance_nasce_no_primeiro() {
    let m = StateMachine {
        states: vec![estado("A", "", "")],
        transitions: vec![],
        initial: 9,
    };
    assert_eq!(born(&m).current, 0);
}
