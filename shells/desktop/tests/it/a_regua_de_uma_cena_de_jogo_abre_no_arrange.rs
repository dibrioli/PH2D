//! ⭐⭐⭐ **A régua que uma cena de JOGO abre mostra o relógio do JOGO** (2026-09-20).
//!
//! # ⛔⛔⛔ O defeito que este censo impede, medido
//!
//! Quinze prólogos das cenas de componentes abriam a régua com a MESMA justificação escrita —
//! *«o dono tem de ver que a corrida ANDA»* — e ela abre no separador de fábrica, o **`Keys`**.
//! Ali a régua mostra o relógio do **CLIPE** (`fase_timeline_drain`: *«o clip clock em Keys mode,
//! o da timeline em Arrange»*).
//!
//! Medido com uma sonda no `advance_ticks` e a foto da cena do fim de jogo: com o jogo a
//! **`5,750 s`** e `playing = true`, o painel lia **`Time(s) 0` · `Frame 0`** e o cursor colado ao
//! zero. Depois da porta: **`Time(s) 7,267` · `Frame 174`**.
//!
//! ⚠️ *Uma cena que abre uma prova e mostra a prova errada é pior que uma cena sem prova nenhuma.*
//!
//! # ⚠️ As DUAS metades, e nenhuma basta
//!
//! A primeira acusa a recaída (ninguém escreve o `insert` à mão); a **segunda** é a que impede o
//! censo de ficar verde por VÁCUO — se alguém apagasse a porta e todas as chamadas, a primeira
//! passaria a medir nada. *Um censo sem piso de população aprova o ficheiro vazio.*

const FICHEIROS: [(&str, &str); 2] = [
    (
        "components_scenes.rs",
        include_str!("../../src/components_scenes.rs"),
    ),
    (
        "components_scenes_suplentes.rs",
        include_str!("../../src/components_scenes_suplentes.rs"),
    ),
];

/// A agulha monta-se em runtime — ⛔ escrita como literal, este ficheiro acusar-se-ia a si mesmo
/// (a lei que o censo do HR-15 e a vassoura da parede já pagam).
fn agulha() -> String {
    format!("panel_visibility.insert({}timeline{}, true)", '"', '"')
}

/// ⛔ **Nenhuma cena de jogo abre a régua à mão** — ela passa pela porta.
#[test]
fn nenhuma_cena_de_jogo_abre_a_regua_a_mao() {
    let a = agulha();
    for (nome, fonte) in FICHEIROS {
        for (i, linha) in fonte.lines().enumerate() {
            // ⚠️⚠️ **Um COMENTÁRIO não é código, e esta régua já me acusou a mim:** a 1.ª
            // redacção reprovou na linha `38`, que é o doc-comment da PRÓPRIA porta a citar o
            // defeito que ela cura. *Uma régua textual que lê a prosa que EXPLICA a cura acusa a
            // cura* — a forma que a Fase B da física registou.
            let comentario = linha.trim_start().starts_with("//");
            // ⚠️ O corpo da PRÓPRIA porta é a única ocorrência legítima de CÓDIGO, e ela vive a 4
            // espaços de indentação (uma função de topo) contra os ≥ 8 de um prólogo.
            let dentro_da_porta = linha.starts_with("    hero.") && !linha.starts_with("     ");
            assert!(
                !linha.contains(&a) || comentario || dentro_da_porta,
                "{nome}:{} abre a régua à mão — use `abre_a_regua_da_corrida`, senão ela abre no \
                 separador `Keys`, que mostra o relógio do CLIPE e não o do jogo",
                i + 1
            );
        }
    }
}

/// ⭐ **E a porta é de facto USADA** — o piso que impede o gate de ficar verde por vácuo.
///
/// ⚠️ O número é o MEDIDO em 2026-09-20 (`7` + `8`), e ele só desce quando uma cena morre.
#[test]
fn a_porta_da_regua_tem_a_populacao_que_se_mediu() {
    let n: usize = FICHEIROS
        .iter()
        .map(|(_, f)| f.matches("abre_a_regua_da_corrida(hero)").count())
        .sum();
    assert!(
        n >= 15,
        "só {n} cenas passam pela porta — em 2026-09-20 eram 15; um censo sem piso aprova o vazio"
    );
}

/// ⚠️⚠️ **E a porta pede mesmo o separador** — sem esta metade ela seria um alias do `insert`.
///
/// **Mutação que deve sangrar:** apagar o `request_arrange_tab()` do corpo dela.
#[test]
fn a_porta_pede_o_separador_que_mostra_o_relogio_do_jogo() {
    let fonte = FICHEIROS[0].1;
    let corpo = fonte
        .split_once("pub(crate) fn abre_a_regua_da_corrida")
        .expect("a porta tem de existir")
        .1;
    let fim = corpo.find("\n}").expect("o fim do corpo");
    assert!(
        corpo[..fim].contains("request_arrange_tab()"),
        "a porta abre a régua e NÃO pede o Arrange: ela ficaria a mostrar o relógio do clipe"
    );
}
