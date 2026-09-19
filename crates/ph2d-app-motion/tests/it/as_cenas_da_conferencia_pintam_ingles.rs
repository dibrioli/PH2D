//! ⭐⭐⭐ **AS CENAS DA CONFERÊNCIA PINTAM INGLÊS** — por ordem do dono, e com a régua que faltava.
//!
//! # ⛔⛔⛔ Porque nada acusava isto
//!
//! O gate da família (`every_word_this_family_shows_comes_from_the_string_table`) **isenta as
//! cenas por desenho**, e a isenção está escrita em dois sítios: no cabeçalho da tabela
//! `app.motion` (*«o que NÃO está aqui, de propósito: as CENAS»*) e no próprio gate, com piso de
//! população. *Uma cena guarda o texto DELA* — e isso é certo.
//!
//! ⚠️ **Mas essa isenção também apagava a pergunta da LÍNGUA.** Medido em 2026-09-19: estas cinco
//! cenas pintavam **23 palavras em português** no canvas (`ANTES`, `DEPOIS`, `ALVO`, `MIRA`,
//! `RASTRO`, `CORTE`, `BANDA`, `RAMPA`, `FORMA`, `SOLTA`, `DESVIA`, `BORDA`, `APARADO`,
//! `PICOTADO`), contra a lei de que *toda string que o artista LÊ é inglês*. Nenhuma catraca as
//! via, porque isento de *«vem da tabela?»* não é isento de *«em que língua?»*.
//!
//! ⛔ **E a pergunta TINHA de ir ao dono antes da cura:** uma daquelas palavras foi escolhida por
//! ele num smoke (`SOLTA`, e não «RASGA», 2026-08-21). A resposta foi **«tudo em inglês»**
//! (2026-09-19), e é ela que este ficheiro guarda.
//!
//! # ⚠️ A régua lê o FONTE, e a razão é o alcance
//!
//! O que interessa é o par *rótulo de coluna* (`label(g, "…")`) **mais** a tabela `ROW_LABELS` — e
//! duas das cinco tabelas são privadas do módulo delas. Ler o fonte por [`include_str!`] alcança as
//! cinco e **deixa de compilar** no dia em que um ficheiro mudar de sítio, que é o modo de falha
//! barulhento (`CLAUDE.md` §5.0).

use ph2d_label_census::portuguese_tokens;

/// Os cinco ficheiros, com o fonte embutido. ⚠️ `include_str!` e não leitura em runtime: o gémeo em
/// runtime só falha **quando o teste corre**, e um filtro esconde-o.
const CENAS: &[(&str, &str)] = &[
    (
        "goal",
        include_str!("../../src/motion_state_conferencia_demos_goal.rs"),
    ),
    (
        "operator",
        include_str!("../../src/motion_state_conferencia_demos_operator.rs"),
    ),
    (
        "rank",
        include_str!("../../src/motion_state_conferencia_demos_rank.rs"),
    ),
    (
        "sim",
        include_str!("../../src/motion_state_conferencia_demos_sim.rs"),
    ),
    (
        "style",
        include_str!("../../src/motion_state_conferencia_demos_style.rs"),
    ),
];

/// As palavras que uma cena PINTA: os rótulos de coluna e a tabela das linhas.
///
/// ⚠️ **Só o CÓDIGO** — as linhas de `//!`/`///` são prosa para a próxima LLM, e ela é em
/// português de propósito.
fn palavras_pintadas(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for linha in src.lines() {
        let t = linha.trim_start();
        if t.starts_with("//") {
            continue;
        }
        let colhe = |marca: &str, out: &mut Vec<String>| {
            if let Some(p) = t.find(marca) {
                for parte in t[p + marca.len()..].split('"').skip(1).step_by(2) {
                    out.push(parte.to_string());
                }
            }
        };
        colhe("label(g,", &mut out);
        if t.contains("ROW_LABELS") && t.contains('[') {
            colhe("= [", &mut out);
        }
    }
    out
}

/// ⭐⭐⭐ **NENHUMA PALAVRA QUE ESTAS CENAS PINTAM ESTÁ EM PORTUGUÊS.**
#[test]
fn o_rotulo_de_cena_que_o_artista_le_esta_em_ingles() {
    let mut total = 0usize;
    let mut maus = Vec::new();
    for (nome, src) in CENAS {
        let palavras = palavras_pintadas(src);
        // ⛔ PISO POR CENA: um ficheiro que deixe de ser lido devolve zero palavras e passaria.
        assert!(
            palavras.len() >= 4,
            "a cena `{nome}` pinta {} palavra(s) — em 2026-09-19 eram 4 a 6. Ou ela mudou de \
             forma, ou o leitor partiu-se",
            palavras.len()
        );
        total += palavras.len();
        for p in palavras {
            let pt = portuguese_tokens(&p);
            if !pt.is_empty() {
                maus.push(format!("{nome}: {p:?} {pt:?}"));
            }
        }
    }
    assert!(
        total >= 23,
        "as cinco cenas pintam {total} palavras — em 2026-09-19 eram 23"
    );
    assert!(
        maus.is_empty(),
        "estas palavras são pintadas no canvas em português — o artista lê-as assim:\n  {}",
        maus.join("\n  ")
    );
}

/// ⛔⛔ **O CONTROLO POSITIVO da LEITURA** — sem ele o gate acima passa sobre um leitor cego.
///
/// ⚠️ Ele afirma as duas metades que a extracção tem de acertar: o rótulo de COLUNA (uma chamada
/// solta) e a TABELA das linhas (uma lista). *Um leitor que só veja uma das duas lê metade das
/// palavras e cala-se sobre a outra.*
#[test]
fn a_leitura_ve_as_duas_formas_que_uma_cena_usa() {
    let goal = CENAS.iter().find(|(n, _)| *n == "goal").unwrap().1;
    let p = palavras_pintadas(goal);
    assert!(
        p.contains(&"BEFORE".to_string()) && p.contains(&"AFTER".to_string()),
        "o leitor não viu os rótulos de COLUNA: {p:?}"
    );
    assert!(
        p.contains(&"TARGET".to_string()) && p.contains(&"AIM".to_string()),
        "o leitor não viu a TABELA das linhas: {p:?}"
    );
    // ⛔ E o negativo: a PROSA do cabeçalho é portuguesa de propósito e NÃO entra.
    assert!(
        !p.iter().any(|w| w.contains("linha")),
        "o leitor colheu prosa de comentário: {p:?}"
    );
}
