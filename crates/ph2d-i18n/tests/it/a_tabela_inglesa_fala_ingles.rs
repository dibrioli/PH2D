//! ⭐⭐⭐ **A TABELA INGLESA DO APP TEM DE FALAR INGLÊS** — a régua que faltava, e a dívida que ela
//! achou à primeira corrida.
//!
//! # ⛔⛔ Porque as trinta catracas do HR-15 não viam isto
//!
//! Elas perguntam *«esta palavra veio da TABELA?»*. Nenhuma pergunta *«e a tabela está em que
//! LÍNGUA?»* — e em 2026-09-19, sobre `5 226` entradas, **oito** delas eram frases em português
//! que o artista lê na caixa de saída da escultura. ⚠️ Elas viviam **fora dos marcadores do script
//! de migração**, logo nenhuma passagem automática lhes tocou: *um texto escrito à mão depois da
//! migração entra na tabela pela porta que a migração não guarda.*
//!
//! Traduzidas por **ordem do dono** (*«tudo em inglês»*, 2026-09-19), posta com a medição na mão.
//!
//! # ⚠️ O que esta régua É, declarado
//!
//! Um **PISO**, e não um detector de português. Ela acusa por tokens da classe fechada e por
//! MORFOLOGIA (`-ção/-cao`, `-ões/-oes`, `-ão/-ao`, `-mente`, `-ando/-endo/-indo`) — e uma frase
//! curta sem nenhum dos dois passa (`"colorize a recalcular"` passa-lhe ao lado). *Uma régua
//! heurística que se declara é utilizável; uma que se julga completa é uma licença.*
//!
//! ⛔⛔ **E ela custou a lição do LEITOR:** a 1.ª redacção lia as entradas com um `.` que não casa
//! quebra de linha, e **toda entrada com `\` de continuação evaporava** — ela leu `3` onde estavam
//! `8`, e as cinco que faltavam eram as mais compridas. *Um censo textual tem de saber todas as
//! formas do que lê, e a continuação de linha é uma delas.*
//!
//! # ⛔ A metade que NÃO é defeito: o diagnóstico de CONSOLA
//!
//! O terminal é do DONO (`CLAUDE.md` §0.8), e a prosa dele é a língua dele. As chaves de consola
//! estão isentas **com o mecanismo**, e o gate reprova se uma isenção deixar de descrever alguma
//! coisa.

use std::path::PathBuf;

/// ⭐ As chaves cujo texto **nunca chega ao ecrã** — diagnóstico que só o terminal mostra.
///
/// ⚠️ Cada uma diz PORQUÊ, e o censo de obsolescência abaixo exige que ela ainda exista.
const SO_CONSOLA: &[(&str, &str)] = &[(
    "shell.undo_app.",
    "as cinco razoes de um passo de undo NAO ter sido registado. Elas so' correm com o log LIGADO \
     (`Self::undo_log_on()`), e o leitor delas e' quem esta' a ler o terminal -- o Enio ou a LLM \
     seguinte. A prosa de terminal e' a lingua DELE (CLAUDE.md §0.8).",
)];

/// ⭐⭐ **A lei da LÍNGUA vem da PORTA** ([`ph2d_label_census::portuguese_tokens`]), nunca de uma
/// cópia aqui: ela tem dois consumidores — esta tabela e as cenas da conferência do Motion —, e
/// duas cópias divergiriam no dia da primeira palavra nova.
use ph2d_label_census::portuguese_tokens as portugues;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Todas as entradas `"chave" => "texto"` da tabela.
///
/// ⚠️ **A continuação de linha (`\` + quebra) faz parte do literal** — ver o cabeçalho.
fn entradas() -> Vec<(String, usize, String, String)> {
    let mut out = Vec::new();
    let mut fich: Vec<PathBuf> = std::fs::read_dir(raiz())
        .expect("a pasta src/ da tabela existe")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    fich.sort();
    for p in fich {
        let src = std::fs::read_to_string(&p).expect("le o ficheiro da tabela");
        let nome = p.file_name().unwrap().to_string_lossy().to_string();
        for (chave, texto, linha) in pares(&src) {
            out.push((nome.clone(), linha, chave, texto));
        }
    }
    out
}

/// ⭐⭐⭐ **O LEITOR É AGORA UMA PORTA PARTILHADA** — `ph2d_label_census::keys::declared_pairs_in`.
///
/// ⛔⛔ **A cópia que morava aqui conhecia SÓ a forma `match`** (`"k" => "v"`), e a irmã
/// `keys_declared` aprendeu o TUPLO (`("k", "v"),`) em 2026-09-17. ⇒ **1 432 entradas — 23 % da
/// tabela — nunca foram conferidas contra o português** (`node_options` 601 ·
/// `node_params_motion` 430 · `node_params` 398).
///
/// ⚠️⚠️ **E o piso de população não o podia dizer:** ele exige `>= 5 000`, e a forma que a régua
/// conhecia já traz `4 815` + o resto. *Um piso satisfeito pela forma que a régua conhece não
/// afirma nada sobre a forma que ela não conhece.* A cura é uma PORTA, nunca a terceira cópia.
///
/// ⚠️ A porta também corrige um espaço: a continuação de linha do Rust não insere NADA, e esta
/// cópia empurrava um `' '`. Inócuo para a língua, decisivo para quem compara ao bit.
fn pares(src: &str) -> Vec<(String, String, usize)> {
    ph2d_label_census::keys::declared_pairs_in(src)
        .into_iter()
        .map(|p| (p.chave, p.texto, p.linha))
        .collect()
}

/// ⭐⭐⭐ **NENHUMA FRASE QUE O ARTISTA LÊ ESTÁ EM PORTUGUÊS.**
#[test]
fn nenhuma_frase_que_o_artista_le_esta_em_portugues() {
    let todas = entradas();
    // ⛔ PISO DE POPULAÇÃO: uma leitura que encolhesse devolveria zero acusados e leria-se como
    //    aprovação — a forma exacta que o censo por prefixo de nome deste repo já pagou.
    //
    // ⛔⛔ **E o piso SOBE com a porta, senão ele vira uma licença.** Ele esteve em `5 000` enquanto
    //    o leitor conhecia só a forma `match`, que sozinha traz `4 815` — ou seja, perder o TUPLO
    //    outra vez passaria despercebido. Hoje a leitura é de **6 706** entradas e o piso fica em
    //    `6 400`: acima do que a forma antiga produz sozinha, que é a única posição em que ele
    //    afirma alguma coisa sobre a forma nova.
    assert!(
        todas.len() >= 6_400,
        "a régua leu {} entradas da tabela — em 2026-09-19 eram 6 706, das quais 1 432 na forma \
         TUPLO. Ou a tabela encolheu, ou o leitor partiu-se (a CONTINUAÇÃO de linha e a forma \
         TUPLO foram as duas que já o morderam)",
        todas.len()
    );
    let mas: Vec<String> = todas
        .iter()
        .filter(|(_, _, chave, _)| !SO_CONSOLA.iter().any(|(p, _)| chave.starts_with(p)))
        .filter_map(|(f, l, chave, texto)| {
            let pt = portugues(texto);
            (!pt.is_empty()).then(|| format!("{f}:{l}  {chave}  {pt:?}\n      {texto:?}"))
        })
        .collect();
    assert!(
        mas.is_empty(),
        "a tabela INGLESA do app tem {} entrada(s) em português — o artista lê-as assim:\n  {}",
        mas.len(),
        mas.join("\n  ")
    );
}

/// ⛔⛔ **O CONTROLO POSITIVO — a régua vê o que ela afirma ver.**
///
/// ⚠️ As fixturas são as OITO frases como elas estavam antes da cura de 2026-09-19. Sem esta
/// metade, um `portugues()` que devolvesse sempre vazio deixava o gate acima verde para sempre.
#[test]
fn a_regua_ve_as_oito_frases_que_ela_curou() {
    const ANTES: &[&str] = &[
        "ha' uma pilha de multiresolucao montada -- J reverte-a",
        "a malha aqui ja' esta' no ponto que o Detail pede -- mova o slider (ou a tecla U) para \
         pedir outra densidade, ou aumente o pincel com ] para alcancar mais peca",
        "{nome} precisa de uma pilha de multiresolucao -- sem um nivel ABAIXO nao ha' deslocamento \
         nenhum (K subdivide, ',' desce)",
        "{nome} trabalha a BEIRA de uma peca aberta -- esta peca e' fechada, e a regiao dele comeca \
         na borda (experimente uma tigela, ou apague faces para abrir uma boca)",
        "{nome} precisa de OUTRA peca A' VISTA -- a{plural} que ha' esta' escondida (abra o olho \
         dela na Hierarquia, ou saia do isolamento)",
        "{nome} precisa de OUTRA peca na cena -- ele empurra o barro ate' encostar nela, e aqui so' \
         ha' uma",
        "nao ha' peca nenhuma para cortar",
        "ha' uma pilha de multiresolucao montada -- J reverte-a e o corte volta",
    ];
    for f in ANTES {
        assert!(
            !portugues(f).is_empty(),
            "a régua não vê português em {f:?} — ela deixaria esta frase voltar"
        );
    }
    // ⛔ O CONTROLO NEGATIVO: o inglês da tabela não pode ser acusado. ⚠️ Estas seis são reais, e a
    //    última é a armadilha — `Dash` e `Range` são palavras que a cena do Motion passou a pintar.
    for f in [
        "a multiresolution stack is mounted -- J reverts it",
        "there is no piece to cut",
        "Canonical widget showcase \u{b7} reference for peripheral agents",
        "Select an entity to inspect its components",
        "Snap to grid",
        "Dash \u{b7} Range \u{b7} Ramp \u{b7} Shape \u{b7} Trim \u{b7} Cull",
    ] {
        assert!(
            portugues(f).is_empty(),
            "a régua acusou INGLÊS em {f:?}: {:?}",
            portugues(f)
        );
    }
}

/// ⛔⛔ **O CENSO DE OBSOLESCÊNCIA das isenções** — sem ele a lista vira licença.
#[test]
fn nenhuma_isencao_de_consola_ficou_orfa() {
    let todas = entradas();
    let orfas: Vec<&str> = SO_CONSOLA
        .iter()
        .filter(|(p, _)| !todas.iter().any(|(_, _, chave, _)| chave.starts_with(p)))
        .map(|(p, _)| *p)
        .collect();
    assert!(
        orfas.is_empty(),
        "estas isenções já não descrevem chave nenhuma — APAGUE-AS: {orfas:?}"
    );
    // ⭐ E a metade JUSTA: uma isenção que já não abriga português nenhum não tem razão de ser.
    for (p, _) in SO_CONSOLA {
        let abriga = todas
            .iter()
            .filter(|(_, _, chave, _)| chave.starts_with(p))
            .any(|(_, _, _, texto)| !portugues(texto).is_empty());
        assert!(
            abriga,
            "a isenção {p:?} já não abriga uma frase em português — se aquelas chaves passaram a \
             inglês, ela deixou de ser precisa"
        );
    }
}

/// ⛔⛔⛔ **O LEITOR JUNTA AS CONTINUAÇÕES DE LINHA — e sem esta metade ele mente CALADO.**
///
/// A 1.ª redacção desta régua lia as entradas com um `.` que não casa quebra de linha: **toda
/// entrada com `\` de continuação era truncada**, e ela leu `3` das `8` frases em português — as
/// cinco que faltavam eram, precisamente, as mais compridas.
///
/// ⚠️ **Nenhum dos outros três testes deste ficheiro o apanha:** o piso conta ENTRADAS (que não
/// mudam), o controlo positivo usa fixturas escritas à mão (que não passam pelo leitor), e a
/// tabela está curada (logo não há português que se perca). *Uma cegueira do LEITOR precisa de uma
/// régua sobre o leitor.*
#[test]
fn a_leitura_junta_uma_entrada_partida_em_duas_linhas() {
    let todas = entradas();
    let (_, _, _, texto) = todas
        .iter()
        .find(|(_, _, chave, _)| chave == "app.sculpt3d.recusa.trabalha_a_beira_de_uma_peca_aberta")
        .expect("a chave da recusa do contorno existe na tabela");
    assert!(
        texto.starts_with("{nome} works the RIM"),
        "o começo da entrada mudou: {texto:?}"
    );
    assert!(
        texto.ends_with("open a mouth)"),
        "⛔ a entrada chegou TRUNCADA — o leitor deixou de juntar a continuação de linha: {texto:?}"
    );
    // ⛔ E o CONTROLO: uma entrada de UMA linha continua inteira.
    let (_, _, _, curta) = todas
        .iter()
        .find(|(_, _, chave, _)| chave == "app.sculpt3d.trim_aplica.sem_peca_para_cortar")
        .expect("a chave do corte sem peça existe");
    assert_eq!(curta, "there is no piece to cut");
}
