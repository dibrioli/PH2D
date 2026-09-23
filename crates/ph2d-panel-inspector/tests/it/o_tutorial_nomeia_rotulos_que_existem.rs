//! ⭐⭐⭐ **Todo r&oacute;tulo que o TUTORIAL cita existe no painel** (TOP-20 #15, 2026-09-15).
//!
//! # Porque este gate existe
//!
//! Um passo de tutorial que diz *«carregue em `+ Add State`»* n&atilde;o &eacute; uma
//! instru&ccedil;&atilde;o: &eacute; uma **afirma&ccedil;&atilde;o de que esse texto est&aacute; na
//! tela**. Ela &eacute; escrita num ficheiro (o tutorial) e decidida noutro (o pintor da
//! sec&ccedil;&atilde;o) — logo envelhece sozinha, e **nada reprova**.
//!
//! ⚠️ Esta classe j&aacute; custou uma jornada a este repo: quando um selector do Motion deixou de
//! **ciclar** no clique, os **tr&ecirc;s** tutoriais em PDF continuaram a ensinar o gesto antigo, e
//! o dono bateria nisso no primeiro que tocasse. O texto de um smoke ou de um tutorial &eacute;
//! **superf&iacute;cie de produto** (`CLAUDE.md` §0.8) — a &uacute;nica cuja correc&ccedil;&atilde;o
//! nada media.
//!
//! # O que ele mede, e o que NÃO mede
//!
//! ⛔ Ele **n&atilde;o** mede que o r&oacute;tulo chega a pixel — isso pede um arn&ecirc;s de
//! pintura que esta crate n&atilde;o tem, e est&aacute; nomeado como aberto no handoff da wave. Ele
//! mede a metade que apanha a podrid&atilde;o real: *o texto que o tutorial p&otilde;e entre aspas
//! ainda existe no pintor*.
//!
//! ⚠️ **A unidade &eacute; o `<code class="ui">`, e a marca &eacute; deliberada:** o tutorial cita
//! tamb&eacute;m nomes da CENA (`Door`, `botao`, `door_opening`), que n&atilde;o s&atilde;o
//! r&oacute;tulos do painel — varrer todo `<code>` acusaria esses e o gate teria de ganhar uma lista
//! de excep&ccedil;&otilde;es, que &eacute; a forma como um censo apodrece.
//!
//! ⚠️ **`include_str!` e n&atilde;o `read_to_string`:** um caminho que deixe de existir tem de
//! **falhar a compilar**, e n&atilde;o s&oacute; quando o teste correr (a armadilha do «g&eacute;meo
//! em runtime», medida na W2).

/// A fonte do tutorial, tal como o gerador de PDF a l&ecirc;.
const TUTORIAL: &str =
    include_str!("../../../../docs/Components/tutoriais/src/01_maquina_de_estados.html");

/// Os pintores que produzem os r&oacute;tulos citados — a sec&ccedil;&atilde;o e o cabe&ccedil;alho
/// do Inspector (de onde vem o bot&atilde;o *Add Component*).
const PINTORES: [&str; 5] = [
    include_str!("../../src/sections/statemachine.rs"),
    // ⚠️ **O irmão das SETAS** — o ficheiro passou o tecto de 600 na migração do HR-15 e as rows
    //    das transições mudaram-se para cá. Um gate que lesse só o pai acusava metade do tutorial.
    include_str!("../../src/sections/statemachine_setas.rs"),
    include_str!("../../src/paint_head.rs"),
    include_str!("../../src/sections/anchors.rs"),
    // ⚠️⚠️ **O CARTÃO DO TOPO** — a frase da selecção múltipla deixou de ser pintada pela secção
    //    e passou a ser pintada UMA vez pelo painel (2026-09-22: ela vivia em **vinte e uma**
    //    secções e o artista lia-a uma vez por componente). O tutorial cita-a, e sem este ficheiro
    //    o gate acusava-a de órfã sobre um produto CERTO.
    include_str!("../../src/paint_cards.rs"),
];

/// ⚠️⚠️ **A TABELA entra no gate porque o texto MUDOU DE SÍTIO** (`line/UIUX`, 2026-09-16): desde
/// a migração do HR-15 o pintor escreve `tr("panel.inspector.…")` e a FRASE vive aqui. Um gate que
/// só lesse o pintor acusaria como órfão todo rótulo VIVO — e, pior, ficaria **verde sobre uma
/// chave** se a frase mudasse na tabela sem ninguém tocar no pintor (o defeito que os sete gates
/// repontados da `line/UIUX` pagaram).
///
/// ⭐ **Por isso a régua tem DUAS metades:** a frase citada tem de ser o texto de uma chave, **e**
/// essa chave tem de aparecer num pintor. Nenhuma das duas sozinha afirma o que o tutorial promete.
const TABELAS: [&str; 2] = [
    include_str!("../../../ph2d-i18n/src/inspector_game.rs"),
    include_str!("../../../ph2d-i18n/src/inspector.rs"),
];

/// Os pares `("chave", "texto")` das tabelas, já com os escapes resolvidos.
fn chaves_e_textos() -> Vec<(String, String)> {
    let mut v = Vec::new();
    for t in TABELAS {
        // ⚠️⚠️ **Duas FORMAS de braço, e a segunda é a das frases longas**: o `rustfmt` escreve
        //    `"k" => {` e põe o texto na linha seguinte. Um parser que só conhecesse a forma de
        //    uma linha lia ZERO avisos — exactamente as frases que o tutorial cita.
        let mut bloco: Option<String> = None;
        for linha in t.lines() {
            let linha = linha.trim();
            if let Some(k) = bloco.take() {
                if let Some(resto) = linha.strip_prefix('"')
                    && let Some(f) = resto.rfind('"')
                {
                    v.push((k, desescapa(&resto[..f])));
                }
                continue;
            }
            let Some(resto) = linha.strip_prefix('"') else {
                continue;
            };
            let Some(fim) = resto.find('"') else { continue };
            let chave = &resto[..fim];
            let Some(depois) = resto[fim + 1..].trim_start().strip_prefix("=>") else {
                continue;
            };
            let depois = depois.trim_start();
            let Some(texto) = depois.strip_prefix('"') else {
                if depois.starts_with('{') {
                    bloco = Some(chave.to_string());
                }
                continue;
            };
            let Some(fim_t) = texto.rfind('"') else {
                continue;
            };
            v.push((chave.to_string(), desescapa(&texto[..fim_t])));
        }
    }
    v
}

/// **Traduz os escapes `\u{XXXX}` do fonte Rust para o caracter real.**
///
/// ⚠️ Sem isto o gate comparava `The clock is stopped — it…` (o que o artista l&ecirc;) com
/// `The clock is stopped \u{2014} it…` (o que est&aacute; escrito no ficheiro) e acusava um
/// r&oacute;tulo VIVO. *Uma r&eacute;gua que compara a forma escrita com a forma lida mede o
/// codificador, n&atilde;o o produto.*
fn desescapa(fonte: &str) -> String {
    let mut out = String::with_capacity(fonte.len());
    let mut resto = fonte;
    while let Some(i) = resto.find("\\u{") {
        out.push_str(&resto[..i]);
        let tail = &resto[i + 3..];
        match tail.find('}') {
            Some(j) => {
                match u32::from_str_radix(&tail[..j], 16)
                    .ok()
                    .and_then(char::from_u32)
                {
                    Some(c) => out.push(c),
                    None => out.push_str(&resto[i..i + 3 + j + 1]),
                }
                resto = &tail[j + 1..];
            }
            None => {
                out.push_str(&resto[i..]);
                return out;
            }
        }
    }
    out.push_str(resto);
    out
}

/// Os r&oacute;tulos que o tutorial afirma estarem na tela, pela ordem em que aparecem.
/// ⚠️⚠️ **LIMITE NOMEADO: ele lê o HTML CRU.** Uma entidade (`&middot;`) nunca casa o texto da
/// tabela, que traz o carácter. ⇒ um rótulo com `·` cita-se pela metade ASCII — foi o que a frase
/// da selecção fez em 2026-09-22. *Curar isto é des-escapar entidades, e ninguém mediu se vale.*
fn rotulos_citados() -> Vec<String> {
    const ABRE: &str = "<code class=\"ui\">";
    let mut v = Vec::new();
    let mut resto = TUTORIAL;
    while let Some(i) = resto.find(ABRE) {
        let tail = &resto[i + ABRE.len()..];
        let j = tail
            .find("</code>")
            .expect("um <code class=\"ui\"> sem fecho");
        v.push(tail[..j].to_string());
        resto = &tail[j..];
    }
    v
}

/// ⭐⭐⭐ **Cada r&oacute;tulo citado existe num pintor.**
///
/// **Muta&ccedil;&atilde;o que deve sangrar:** mudar `"+ Add State"` para `"+ New State"` no
/// pintor da sec&ccedil;&atilde;o — o tutorial passa a ensinar um bot&atilde;o que n&atilde;o
/// existe, e hoje isso reprova.
#[test]
fn o_tutorial_so_cita_rotulos_que_o_painel_pinta() {
    let pintores: Vec<String> = PINTORES.iter().map(|s| desescapa(s)).collect();
    let citados = rotulos_citados();
    // ⚠️ **PISO DE POPULAÇÃO** — sem ele, uma marca renomeada faz o gate varrer ZERO e ficar
    // verde a medir nada (a falha MUDA que a W2 mediu ao mover ficheiros).
    assert!(
        citados.len() >= 14,
        "o tutorial deveria citar pelo menos 14 rotulos de tela; achei {} — a marca \
         `<code class=\"ui\">` mudou de nome?",
        citados.len()
    );
    let tabela = chaves_e_textos();
    // ⚠️ **PISO DA TABELA** — se o parser dela deixar de casar (um `match` reescrito, um texto em
    //    várias linhas), ela devolve VAZIO e o gate passa a medir só o pintor, em silêncio.
    assert!(
        tabela.len() >= 100,
        "li {} pares chave/texto nas tabelas — o formato do `match` mudou e este gate passaria a \
         medir só os literais do pintor",
        tabela.len()
    );
    let vivo = |r: &String| {
        // (a) ainda escrito no pintor — os rótulos que não são língua, e os que a migração não tocou
        pintores.iter().any(|p| p.contains(r.as_str()))
            // (b) ou é o TEXTO de uma chave que um pintor de facto usa
            || tabela
                .iter()
                .any(|(k, t)| t.contains(r.as_str()) && pintores.iter().any(|p| p.contains(k.as_str())))
    };
    let orfaos: Vec<&String> = citados.iter().filter(|r| !vivo(r)).collect();
    assert!(
        orfaos.is_empty(),
        "o tutorial cita rotulos que nenhum pintor do Inspector produz: {orfaos:?}"
    );
}

/// ⚠️ **A metade JUSTA** — sem ela, um `contains` sobre uma string vazia passaria sempre, e uma
/// marca que extraísse `""` deixaria o gate de cima verde sobre um tutorial inteiro podre.
#[test]
fn nenhum_rotulo_citado_e_vazio() {
    for r in rotulos_citados() {
        assert!(
            !r.trim().is_empty(),
            "um <code class=\"ui\"> vazio no tutorial: ele afirma que a tela mostra NADA"
        );
    }
}
