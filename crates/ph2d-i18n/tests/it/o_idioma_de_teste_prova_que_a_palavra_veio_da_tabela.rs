//! ⭐⭐⭐ **A PROVA QUE FALTAVA ÀS NOVE FATIAS DO HR-15.**
//!
//! Os **30** censos do HR-15 lêem o **FONTE de uma crate** e perguntam *«há aqui um literal com
//! cara de língua?»*. Nenhum deles pergunta *«a palavra que o artista LÊ saiu da tabela?»* — e as
//! duas não são a mesma pergunta: um rótulo esquecido no pintor pinta-se **exactamente igual** ao
//! que veio da tabela, logo *nada nesta árvore os conseguia distinguir*.
//!
//! O [`Idioma::Teste`] é a distinção. Este ficheiro afirma as **seis** propriedades de que o smoke
//! do dono depende — se alguma cair, o que ele vê no ecrã deixa de significar o que ele pensa.
//!
//! ⚠️ **Nenhum teste aqui escreve `PH2D_LANG`**: o ambiente é do PROCESSO e esta suíte corre em
//! paralelo, logo isso seria mais um membro da família de flakes de fan-out — e dos caros, porque
//! a vítima seria um teste noutro ficheiro. A porta é o [`tr_em`].

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ph2d_i18n::{Idioma, pseudo, tr, tr_em, tr_with_em};

/// A raiz do repo, a partir do manifesto desta crate.
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a crate vive em crates/<nome>")
        .to_path_buf()
}

/// Os ficheiros de tabela desta crate — **derivados do directório**, nunca uma lista à mão.
///
/// ⚠️ Uma lista escrita à mão ficaria a medir menos no dia em que alguém cortasse mais uma tabela
/// por assunto (aconteceu **três** vezes só em 2026-09-17), e o gate ficaria **verde a medir
/// menos**, que é a falha muda que o `CLAUDE.md` §5.0 nomeia sobre censos por prefixo.
///
/// ⛔⛔ **Uma TABELA é um ficheiro com `match key {`, e não todo `.rs` da pasta** — achado na 1.ª
/// corrida deste gate. A régua colhe todo `"x" =>` do ficheiro, e com prefixo **vazio** ela perde
/// o filtro que a protegia nos outros ~30 censos (lá o prefixo é `panel.…`): ela leu `"ingles" =>`
/// e `"xx" =>` do `match` de **variável de ambiente** do [`crate::idioma`] como se fossem chaves,
/// e os quatro gates de população acusaram-nas. *Um censo com o filtro vazio mede tudo o que a
/// forma dele casa, e a forma de um braço de `match` é a mesma em toda parte.*
fn tabelas() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut nomes: Vec<String> = std::fs::read_dir(&dir)
        .expect("src/ desta crate")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".rs"))
        .filter(|n| std::fs::read_to_string(dir.join(n)).is_ok_and(|c| c.contains("match key {")))
        .collect();
    nomes.sort();
    assert!(
        nomes.len() >= 30,
        "esta crate tinha 41 tabelas em 2026-09-17 e o gate lê {} — \
         a varredura do directório partiu-se, e um censo que varre menos fica VERDE a medir menos",
        nomes.len()
    );
    // ⭐ **O CASO QUE MOTIVOU A REGRA, guardado** — sem ele alguém «simplifica» a filtragem de
    //    volta para `.ends_with(".rs")` por parecer arbitrária, e os quatro gates de população
    //    voltam a acusar duas palavras que nunca foram chaves. (A mesma lei que o cabeçalho do
    //    `looks_like_a_key` da régua já cobra de si mesmo.)
    for fora in ["idioma.rs", "pseudo.rs", "formato.rs"] {
        assert!(
            !nomes.iter().any(|n| n == fora),
            "o {fora} não é uma tabela e entrou na varredura — \
             os braços de `match` dele passam a ler-se como chaves"
        );
    }
    nomes
        .into_iter()
        .map(|n| format!("crates/ph2d-i18n/src/{n}"))
        .collect()
}

/// Toda chave declarada em toda tabela desta crate.
///
/// O prefixo vazio faz a régua colher **todo** lado esquerdo de braço (`"x" =>`), que é a
/// população inteira.
fn chaves() -> BTreeSet<String> {
    let t: Vec<&str> = TABELAS
        .get_or_init(tabelas)
        .iter()
        .map(String::as_str)
        .collect();
    let ks = ph2d_label_census::keys::keys_declared(&repo(), &t, "");
    assert!(
        ks.len() >= 4_500,
        "a tabela tinha 4 986 chaves em 2026-09-17 e a extracção lê {} — \
         o gate ficaria verde sobre uma população que encolheu por acidente",
        ks.len()
    );
    ks
}

static TABELAS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

/// O texto fora dos marcadores — o que uma tradução traduziria.
fn fora_dos_marcadores(s: &str) -> String {
    let mut out = String::new();
    let mut resto = s;
    while let Some(abre) = resto.find('{') {
        out.push_str(&resto[..abre]);
        let depois = &resto[abre + 1..];
        match depois.find('}') {
            Some(fecha) => resto = &depois[fecha + 1..],
            None => {
                out.push('{');
                resto = depois;
            }
        }
    }
    out.push_str(resto);
    out
}

/// Os marcadores de um texto, em ordem e com as chavetas.
fn marcadores(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut resto = s;
    while let Some(abre) = resto.find('{') {
        let depois = &resto[abre + 1..];
        match depois.find('}') {
            Some(fecha) => {
                out.push(format!("{{{}}}", &depois[..fecha]));
                resto = &depois[fecha + 1..];
            }
            None => resto = depois,
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 1 · A propriedade que o smoke do dono lê

/// ⭐⭐⭐ **TODA palavra que sai da tabela vem deformada** — logo o que ficar em inglês normal no
/// ecrã está, por construção, escrito no código.
///
/// ⚠️ **É esta a frase que o smoke afirma**, e sem ela o smoke não afirma nada: se UMA chave
/// escapasse, o dono veria uma palavra inglesa que veio da tabela e reportaria um defeito que não
/// existe — ou, pior, aprenderia a ignorar palavras inglesas no ecrã.
#[test]
fn toda_palavra_da_tabela_sai_deformada() {
    let mut com_letras = 0usize;
    let mut escaparam: Vec<String> = Vec::new();
    for k in chaves() {
        let pseudo_ = tr_em(Idioma::Teste, &k);
        let corpo = fora_dos_marcadores(pseudo_);
        if fora_dos_marcadores(tr_em(Idioma::Ingles, &k))
            .chars()
            .any(|c| c.is_ascii_alphabetic())
        {
            com_letras += 1;
        }
        if corpo.chars().any(|c| c.is_ascii_alphabetic()) {
            escaparam.push(format!("{k} → {pseudo_:?}"));
        }
    }
    // ⚠️ **O piso é o CONTROLO de vacuidade:** um valor sem letras nenhumas (um glifo, um número)
    //    passa a asserção trivialmente, e sem este piso o gate ficaria verde se a régua deixasse
    //    de ver letras.
    assert!(
        com_letras >= 4_000,
        "só {com_letras} valores têm letras ASCII — a régua deixou de as ver e o gate ficou vácuo"
    );
    assert!(
        escaparam.is_empty(),
        "{} chave(s) saem do idioma de teste com letra inglesa FORA de um marcador — \
         o smoke do dono deixa de conseguir distinguir tabela de código:\n  {}",
        escaparam.len(),
        escaparam.join("\n  ")
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 2 · O que a deformação não pode partir

/// ⛔ **Um marcador sai VERBATIM, na mesma ordem** — senão o [`tr_with_em`] deixa de o achar e a
/// frase sai com `{n}` escrito na tela.
#[test]
fn um_marcador_atravessa_a_deformacao_intacto() {
    let mut com_marcador = 0usize;
    for k in chaves() {
        let ingles = tr_em(Idioma::Ingles, &k);
        let esperados = marcadores(ingles);
        if esperados.is_empty() {
            continue;
        }
        com_marcador += 1;
        assert_eq!(
            marcadores(tr_em(Idioma::Teste, &k)),
            esperados,
            "a chave {k:?} perde ou reordena marcadores no idioma de teste"
        );
    }
    assert!(
        com_marcador >= 150,
        "só {com_marcador} chaves têm marcador — a extracção partiu-se e este gate mede o nada"
    );
}

/// ⭐ **E a substituição continua a funcionar sobre o modelo deformado**, com o VALOR verbatim.
///
/// ⚠️ O valor é texto do ARTISTA (um nome, um número) e não se traduz — deformá-lo seria mentir
/// sobre dados dele.
#[test]
fn a_frase_deformada_ainda_recebe_os_valores() {
    let chave = "panel.tags.verb.delete_subtree";
    let ingles = tr_em(Idioma::Ingles, chave);
    assert!(
        ingles.contains("{tags}") && ingles.contains("{objects}"),
        "a fixtura deixou de ter os dois marcadores: {ingles:?} — escolha outra chave"
    );
    let saida = tr_with_em(
        Idioma::Teste,
        chave,
        &[("tags", &3_i32), ("objects", &"Boss")],
    );
    assert!(
        saida.contains('3') && saida.contains("Boss"),
        "o valor não chegou à frase deformada: {saida:?}"
    );
    assert!(
        !saida.contains('{'),
        "sobrou um marcador por substituir na frase deformada: {saida:?}"
    );
    // O CONTROLO: o valor sai como o artista o escreveu, sem um acento nosso em cima.
    assert!(
        !saida.contains("Ɓóšš"),
        "o VALOR foi deformado — ele é texto do artista: {saida:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 3 · A chave desconhecida

/// ⛔⛔ **A chave que a tabela não conhece volta CRUA, mesmo no idioma de teste.**
///
/// Meio repo pergunta *«esta chave existe?»* com `tr(k) != k` (o [`ph2d_i18n::TextKey::key`]
/// tem-no escrito). Deformar o caminho da falha partiria **todos** esses censos de uma vez, e
/// trocaria um identificador cru — feio de propósito, reconhecível — por um acentuado.
#[test]
fn a_chave_desconhecida_fica_crua_no_idioma_de_teste() {
    const NAO_EXISTE: &str = "panel.isto.nao.existe.em.tabela.nenhuma";
    assert_eq!(tr_em(Idioma::Teste, NAO_EXISTE), NAO_EXISTE);
    assert_eq!(tr_em(Idioma::Ingles, NAO_EXISTE), NAO_EXISTE);

    // ⭐ **A PREMISSA da detecção, MEDIDA:** ela separa «respondeu» de «não conhece» comparando o
    //    valor com a chave, logo uma chave cujo VALOR fosse a própria chave sairia por engano sem
    //    deformação. Zero em 2026-09-17 — e este gate é o que o mantém verdade.
    let iguais: Vec<String> = chaves()
        .into_iter()
        .filter(|k| tr_em(Idioma::Ingles, k) == *k)
        .collect();
    assert!(
        iguais.is_empty(),
        "{} chave(s) têm por VALOR a própria chave, e a detecção de «desconhecida» \
         deixa-as passar sem deformar — o smoke leria «está no código» sobre texto da tabela:\n  {}",
        iguais.len(),
        iguais.join("\n  ")
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 4 · O controlo: o caminho de omissão não se mexeu

/// ⭐ **O inglês é o MESMO PONTEIRO de sempre** — o idioma novo não custa uma cópia, uma
/// comparação de strings nem um byte diferente no caminho que ship.
///
/// ⚠️ Sem este controlo, toda a fatia poderia ter mudado o produto em silêncio e os outros cinco
/// gates ficariam verdes na mesma: eles medem o idioma de TESTE.
#[test]
fn o_ingles_continua_a_ser_o_mesmo_ponteiro() {
    for k in chaves() {
        let a = tr(&k);
        let b = tr_em(Idioma::Ingles, &k);
        assert!(
            std::ptr::eq(a, b),
            "a chave {k:?} deixou de devolver a mesma tabela — {a:?} contra {b:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 5 · O alongamento

/// ⭐⭐ **O alongamento medido nos quatro editores é ENTREGUE** — se ele encolher, a tensão de
/// disposição some e o smoke deixa de mostrar onde o texto não cabe.
#[test]
fn cada_palavra_cresce_o_que_a_medicao_mandou() {
    let mut curtos = 0usize;
    for k in chaves() {
        let base = fora_dos_marcadores(tr_em(Idioma::Ingles, &k))
            .chars()
            .count();
        if base == 0 {
            continue;
        }
        if base <= 6 {
            curtos += 1;
        }
        let saida = tr_em(Idioma::Teste, &k);
        let depois = fora_dos_marcadores(saida).chars().count();
        // O alvo do balde, menos os dois parênteses que já lá estão.
        let alvo = match base {
            0..=6 => base * 200,
            7..=12 => base * 183,
            13..=20 => base * 170,
            21..=35 => base * 161,
            36..=70 => base * 148,
            _ => base * 137,
        }
        .div_ceil(100);
        assert!(
            depois >= alvo,
            "a chave {k:?} cresce {base} → {depois} e o balde pede {alvo}: {saida:?}"
        );
    }
    // ⚠️ **O balde que mais cresce é o que mais importa** (os rótulos de coluna apertada), e é o
    //    mais fácil de perder se alguém «simplificar» a tabela para um número único.
    assert!(
        curtos >= 200,
        "só {curtos} valores têm 6 caracteres ou menos — a população que dobra desapareceu, \
         e é ela que mede as colunas apertadas deste app"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 6 · O custo

/// ⛔ **A deformação é feita UMA vez por palavra** — senão o `tr` vaza um `Box::leak` **por
/// quadro**, que é o defeito que o doc da própria chave desconhecida já nomeia.
#[test]
fn a_mesma_palavra_deformada_e_a_mesma_memoria() {
    let k = "panel.timeline.title";
    let a = tr_em(Idioma::Teste, k);
    let b = tr_em(Idioma::Teste, k);
    assert!(
        std::ptr::eq(a, b),
        "duas chamadas devolveram memória diferente ({a:?} / {b:?}) — \
         o idioma de teste passa a vazar uma cópia por quadro"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// 7 · A lei, à mão

/// A lei em três exemplos que se conferem de cabeça — a tabela de cima prova a população, esta
/// prova que a aritmética é a que está escrita.
#[test]
fn a_lei_escrita_a_mao() {
    // 3 caracteres, balde 1..6 ⇒ alvo 6, extra 3, menos 2 parênteses ⇒ 1 de enchimento.
    assert_eq!(pseudo::deforma("Add"), "[Áðð·]");
    // O marcador sai verbatim e não conta: base = 0 ⇒ alvo 0 ⇒ sem enchimento.
    assert_eq!(pseudo::deforma("{n}"), "[{n}]");
    // Um `{` sem par é TEXTO, e conta — a mesma tolerância que o `tr_with` tem.
    // 2 caracteres ⇒ alvo 4, extra 2, e os dois parênteses já o pagam ⇒ sem enchimento.
    assert_eq!(pseudo::deforma("{n"), "[{ñ]");
}
