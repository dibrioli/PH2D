//! ⭐⭐⭐ **O ÍNDICE DA MEMÓRIA é lido por TODO agente antes da primeira palavra — e ele estava a ser
//! cortado a meio, em silêncio.**
//!
//! Medido em 2026-09-17, depois da rodada de seis linhas: o `project-memory/MEMORY.md` estava em
//! **224 linhas / 36,5 KB**, o carregador levou **`26 007` bytes** (a linha `158`) e **não disse
//! nada a quem o lia** — um terço do índice era invisível para toda a gente.
//!
//! # ⛔⛔ O que o estourou não foi crescimento: foi a FUSÃO
//!
//! Seis linhas reescreveram a **mesma** entrada, cada uma à sua maneira, e o merge guardou todas:
//! `60` ponteiros repetidos sobre `51` ficheiros, com **zero** linhas iguais byte a byte. *Nenhuma
//! varredura de duplicados textuais as via* — elas só se leem como duplicadas depois de se perguntar
//! **para que ficheiro** cada uma aponta.
//!
//! # ⚠️ E a CONTAGEM de uma família não se escreve: CONTA-SE
//!
//! Quatorze famílias declaravam um número parado, a pior a dizer `8` sobre `18` entradas reais — e
//! três delas declaravam números **diferentes** na mesma página (`Ofício de gate (52)` e `(47)`;
//! `Provas de mutação (7)`, `(8)` e `(11)`). É a lei do §5.0 — *número que soma entre linhas se
//! CONTA, nunca se escolhe* — aplicada ao índice que a enuncia.
//!
//! ⚠️ **O recurso é o BYTE e não a linha** (§0.0): o corte medido foi `26 007` bytes, e é esse o
//! número que este gate defende, com margem.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// O que o carregador de facto levou, medido em 2026-09-17 sobre o ficheiro que ele cortou.
const ORCAMENTO_MEDIDO: usize = 26_007;
/// A margem: o índice cresce entre rodadas, e a cura é sempre DESCER entradas para a família.
const TECTO: usize = 22_000;
/// ⚠️ Piso de população — sem ele um `MEMORY.md` que ninguém encontra passa este gate a medir nada.
const PISO_DE_PONTEIROS: usize = 120;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn memoria() -> PathBuf {
    raiz().join("project-memory")
}

/// Os ficheiros que uma linha do índice aponta.
fn alvos(linha: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes: Vec<char> = linha.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '(' {
            let resto: String = bytes[i + 1..].iter().collect();
            if let Some(fim) = resto.find(')') {
                let alvo = &resto[..fim];
                if alvo.ends_with(".md")
                    && alvo.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.'
                    })
                {
                    out.push(alvo.to_string());
                }
            }
        }
        i += 1;
    }
    out
}

/// Quantas entradas uma família tem — CONTADAS no ficheiro dela, nunca escritas à mão.
fn entradas_da_familia(corpo: &str) -> usize {
    let depois_do_cabecalho = corpo.splitn(3, "---").last().unwrap_or(corpo);
    depois_do_cabecalho
        .lines()
        .filter(|l| l.starts_with("- "))
        .count()
}

fn indice() -> String {
    std::fs::read_to_string(memoria().join("MEMORY.md")).expect("o índice da memória existe")
}

/// ⭐ **Um ponteiro por ficheiro.** Duas redacções do mesmo ponteiro não são duas memórias: são a
/// mesma, e a segunda gasta o orçamento de quem vem a seguir.
#[test]
fn cada_memoria_e_apontada_uma_vez_so() {
    let texto = indice();
    let mut quantas: BTreeMap<String, usize> = BTreeMap::new();
    for linha in texto.lines().filter(|l| l.starts_with("- ")) {
        let mut vistos_nesta = Vec::new();
        for alvo in alvos(linha) {
            if !vistos_nesta.contains(&alvo) {
                vistos_nesta.push(alvo.clone());
                *quantas.entry(alvo).or_default() += 1;
            }
        }
    }
    assert!(
        quantas.len() >= PISO_DE_PONTEIROS,
        "o índice aponta {} ficheiros — abaixo do piso de {PISO_DE_PONTEIROS}. \
         Ou a memória encolheu de repente, ou esta varredura deixou de a ler.",
        quantas.len()
    );
    let repetidos: Vec<String> = quantas
        .iter()
        .filter(|&(_, &n)| n > 1)
        .map(|(f, n)| format!("  {n}×  {f}"))
        .collect();
    assert!(
        repetidos.is_empty(),
        "ficheiros apontados mais de uma vez no `MEMORY.md` ({}):\n{}\n\n\
         ⚠️ Isto é a assinatura de uma FUSÃO, não de descuido: duas linhas reescreveram a mesma \
         entrada e o merge guardou as duas. Cura: uma redacção só, a mais informativa.",
        repetidos.len(),
        repetidos.join("\n")
    );
}

/// ⭐⭐ **A contagem de uma família sai do ficheiro dela.** Um `(N)` escrito à mão envelhece no dia
/// seguinte, e quem o lê decide com ele.
#[test]
fn a_contagem_de_cada_familia_e_a_do_ficheiro() {
    let texto = indice();
    let mut conferidas = 0;
    let mut erradas = Vec::new();
    let mut sem_contagem: Vec<String> = Vec::new();
    for linha in texto.lines().filter(|l| l.starts_with("- ")) {
        for alvo in alvos(linha) {
            if !alvo.starts_with("reference_topic_") {
                continue;
            }
            // ⚠️ **O número vive DENTRO do texto do link, e é o PRIMEIRO `(N)` dele** — não o
            // último parêntese antes do `](`, que numa linha rica é a cauda da explicação
            // (`… (2 de 8 sobreviveram)`), nem o último número do texto, que pode ser um `1.ª`.
            let Some(fim) = linha.find(&format!("]({alvo})")) else {
                continue;
            };
            let Some(abre_texto) = linha[..fim].rfind('[') else {
                continue;
            };
            let texto = &linha[abre_texto + 1..fim];
            let Some(declarado) = primeiro_numero_entre_parentesis(texto) else {
                sem_contagem.push(format!("  {alvo}"));
                continue;
            };
            let corpo = std::fs::read_to_string(memoria().join(&alvo))
                .unwrap_or_else(|_| panic!("a família `{alvo}` é apontada e não existe"));
            let real = entradas_da_familia(&corpo);
            conferidas += 1;
            if declarado != real {
                erradas.push(format!(
                    "  {alvo}: o índice diz {declarado}, o ficheiro tem {real}"
                ));
            }
        }
    }
    assert!(
        conferidas >= 10,
        "só {conferidas} famílias declaram contagem — a varredura do `(N)` deixou de casar"
    );
    assert!(
        erradas.is_empty(),
        "contagens de família paradas ({}):\n{}\n\n\
         ⚠️ Conte no ficheiro (as linhas que começam por `- `) e escreva o número medido.",
        erradas.len(),
        erradas.join("\n")
    );
    // ⭐ **E a metade que impede a cura barata:** apagar o `(N)` de uma linha calaria a metade de
    // cima sem nada acusar — *toda família declara a contagem dela*.
    assert!(
        sem_contagem.is_empty(),
        "famílias apontadas SEM declarar a contagem ({}):\n{}\n\n\
         ⚠️ Escreva `(N)` no texto do link, com o N contado no ficheiro.",
        sem_contagem.len(),
        sem_contagem.join("\n")
    );
}

/// O primeiro `(N)` de um texto — ver a nota no chamador.
fn primeiro_numero_entre_parentesis(texto: &str) -> Option<usize> {
    let mut resto = texto;
    while let Some(i) = resto.find('(') {
        let depois = &resto[i + 1..];
        let j = depois.find(')')?;
        let dentro = &depois[..j];
        if !dentro.is_empty() && dentro.chars().all(|c| c.is_ascii_digit()) {
            return dentro.parse().ok();
        }
        resto = &depois[j + 1..];
    }
    None
}

/// ⭐⭐⭐ **O índice cabe no que o carregador leva.** ⛔ A cura de um estouro é DESCER entradas para o
/// `reference_topic_*` da secção — nunca apagar uma memória, e nunca subir este número sem medir
/// outra vez o corte.
#[test]
fn o_indice_cabe_no_orcamento_do_carregador() {
    let bytes = indice().len();
    assert!(
        bytes <= TECTO,
        "o `MEMORY.md` tem {bytes} bytes contra o alvo de {TECTO} (o carregador levou \
         {ORCAMENTO_MEDIDO} na medição de 2026-09-17, e CORTA o resto em silêncio).\n\
         Cura: descer as entradas mais longas para o `reference_topic_*` da secção delas."
    );
}
