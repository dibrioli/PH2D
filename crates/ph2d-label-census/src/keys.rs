//! **A OUTRA METADE: a chave existe dos DOIS lados.**
//!
//! ⭐⭐⭐ Uma chave com erro de escrita PINTA O IDENTIFICADOR CRU na tela — e vaza a string: o
//! `ph2d_i18n::tr` de uma chave desconhecida faz `leak_key` (devolve o próprio identificador e faz
//! `Box::leak`), e num painel repintado a cada quadro isso é um vazamento POR QUADRO.
//!
//! ⭐ E o censo é dos dois lados porque os dois erros existem e a cura de cada um é oposta: uma chave
//! **usada e não declarada** pinta o identificador; uma **declarada e não usada** é uma órfã — e uma
//! órfã é onde alguém escreve, um dia, uma frase sobre um controlo que já não existe.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::source;

/// ⚠️ **Uma chave tem FORMA, e o censo tem de a conhecer.** A 1.ª redacção do gate aceitava qualquer
/// coisa sem espaços, e a primeira corrida acusou o PRÓPRIO ficheiro do gate: a mensagem de erro
/// dizia `"chrome.…"`, e o censo leu-a como uma chave em uso.
pub fn looks_like_a_key(k: &str, prefix: &str) -> bool {
    // ⛔⛔ **Um NOME DE FICHEIRO tem a mesma forma que uma chave** (2026-09-17): o censo do motor do
    //    pincel (`paint_brush.`) acusou `"paint_brush.rs"` — uma string noutra crate a NOMEAR um
    //    ficheiro — como *«usada e não declarada»*, e antes dele o mesmo censo leu
    //    `"…u.brush.density_detail"` (um CAMINHO DE CAMPO dentro de uma frase) como chave.
    //    ⇒ *duas coisas que um prefixo não distingue, e nenhuma delas é texto de UI.* A cura é da
    //    régua e não do prefixo: fugir para outro prefixo só adia até ao ficheiro que se chame
    //    assim.
    //
    // ⛔⛔ **E a 1.ª redacção desta cura criou um FALSO NEGATIVO, apanhado na mesma corrida:** ela
    //    listava `.png`/`.md`/`.json`/`.txt`/`.svg` e matou a chave REAL `shell.sheet_export.png`
    //    — ali o `png` é um **formato de exportação**, uma palavra que o artista lê num menu.
    //    *Quase toda extensão é também um rótulo legítimo neste app, que exporta em dezasseis
    //    formatos.* ⇒ a lista fica reduzida às extensões do CÓDIGO que este censo lê, que são as
    //    únicas que nunca são uma palavra de UI.
    const EXTENSOES: &[&str] = &[".rs", ".toml"];
    k.len() > prefix.len()
        && k.starts_with(prefix)
        && !EXTENSOES.iter().any(|e| k.ends_with(e))
        // ⚠️ **Uma chave acaba num NOME, nunca num ponto** (2026-09-13): um gate que escreve o
        //    PREFIXO de uma secção (`"panel.inspector.player."`, para separar as duas metades de um
        //    vocabulário) era lido como uma chave em uso, e o censo acusava o próprio gate.
        && !k.ends_with('.')
        // ⚠️⚠️ **NEM NUM SUBLINHADO, e é a MESMA coisa** (2026-09-17): um prefixo montado com
        //    `format!` não tem de acabar num ponto — o censo da escultura escreve
        //    `"panel.sculpt3d.cfilter_"` e cola-lhe o nome do knob, e a régua leu-o como uma chave
        //    em uso e acusou a tabela de não a declarar. ⇒ *a cura de 2026-09-13 curou UMA forma de
        //    prefixo e o mecanismo tem duas.* Medido antes de escrita: **nenhuma** das chaves
        //    declaradas neste repo acaba em `_`.
        && !k.ends_with('_')
        && k.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

fn code_of(p: &Path) -> Option<String> {
    fs::read_to_string(p)
        .ok()
        .map(|s| source::strip_comments(&s).into_iter().collect())
}

/// Todas as chaves `<prefix>…` **usadas** na árvore do repo, com o primeiro ficheiro onde aparecem.
///
/// ⚠️ As tabelas declaram; elas não usam — `tables` ficam de fora, senão o censo lê a própria fonte
/// como consumidor e os dois lados concordam sempre: um espelho não acusa. ⚠️ São VÁRIAS porque um
/// vocabulário pode estar partido por secção (o Inspector: a §14 num irmão, pelo tecto de LOC) — e
/// uma metade lida como consumidora daria por usadas todas as chaves que ela declara.
pub fn keys_used(repo: &Path, prefix: &str, tables: &[&str]) -> BTreeMap<String, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if p.is_dir() {
                if matches!(name, "target" | ".git" | "docs" | "Worktrees") {
                    continue;
                }
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let tables: Vec<PathBuf> = tables.iter().map(|t| repo.join(t)).collect();
    let mut files = Vec::new();
    walk(repo, &mut files);
    files.sort();
    let needle = format!("\"{prefix}");
    let mut out = BTreeMap::new();
    for p in files {
        if tables.contains(&p) {
            continue;
        }
        let Some(code) = code_of(&p) else {
            continue;
        };
        let mut i = 0usize;
        while let Some(k) = code[i..].find(&needle) {
            let start = i + k + 1;
            let Some(end) = code[start..].find('"') else {
                break;
            };
            let key = &code[start..start + end];
            if looks_like_a_key(key, prefix) {
                out.entry(key.to_string()).or_insert_with(|| {
                    p.strip_prefix(repo)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/")
                });
            }
            i = start + end;
        }
    }
    out
}

/// Todas as chaves `<prefix>…` **declaradas** nas tabelas `tables` — só o LADO ESQUERDO de um
/// braço (`"chave" =>`) conta, e as metades de um vocabulário partido somam-se.
pub fn keys_declared(repo: &Path, tables: &[&str], prefix: &str) -> BTreeSet<String> {
    let needle = format!("\"{prefix}");
    let mut out = BTreeSet::new();
    for table_rel in tables {
        let p = repo.join(table_rel);
        let code = code_of(&p).unwrap_or_else(|| panic!("a tabela {p:?} não se lê"));
        let mut i = 0usize;
        while let Some(k) = code[i..].find(&needle) {
            let start = i + k + 1;
            let Some(end) = code[start..].find('"') else {
                break;
            };
            let key = &code[start..start + end];
            // ⭐⭐ **DUAS formas de declarar, e a régua via UMA** (medido em 2026-09-17): a
            // maioria das tabelas é um `match key { "…" => "…" }`, mas três delas
            // (`node_options.rs`, `node_params.rs`, `node_params_motion.rs`) são **listas de
            // TUPLOS** `("…", "…"),`. Sobre elas o censo lia **zero declaradas** e dizia
            // *«dois conjuntos vazios concordam sempre»* — *uma régua que só conhece uma das
            // formas de escrever a mesma coisa mede o nada sobre a outra.*
            //
            // ⚠️ Aceitar a vírgula só é seguro porque isto lê **apenas ficheiros de TABELA**
            // (o `keys_used` exclui-os): num ficheiro de produto `tr_with("k", …)` também tem
            // vírgula a seguir, e ali isso seria um USO lido como declaração.
            let depois = code[start + end + 1..].trim_start();
            // ⛔⛔ **A vírgula SOZINHA não chega, e a 1.ª redacção desta cura partiu dois gates
            // em dez segundos:** num `match` o VALOR também acaba em vírgula
            // (`"k" => "  ...When Crouching",`), logo com o prefixo VAZIO — que é o que o gate do
            // idioma de teste usa — metade da tabela passava a ler-se como chave.
            // ⇒ um tuplo reconhece-se pelo PARÊNTESE que o abre, e essa é a cerca.
            let antes = code[..start - 1].trim_end();
            let tuplo = antes.ends_with('(') && depois.starts_with(',');
            if depois.starts_with("=>") || tuplo {
                out.insert(key.to_string());
            }
            i = start + end;
        }
    }
    out
}

#[cfg(test)]
mod tests_duas_formas_de_declarar {
    use std::io::Write;

    /// ⭐⭐⭐ **AS DUAS FORMAS, e o falso positivo que a 1.ª cura criou.**
    ///
    /// Três tabelas deste repo (`node_options.rs`, `node_params.rs`, `node_params_motion.rs`)
    /// são listas de TUPLOS e não `match`, e sobre elas o censo lia **zero declaradas** — *uma
    /// régua que só conhece uma das formas de escrever a mesma coisa mede o nada sobre a outra.*
    ///
    /// ⛔⛔ **E aceitar «vírgula a seguir» partiu dois gates em dez segundos:** num `match` o
    /// VALOR também acaba em vírgula, logo com o prefixo VAZIO metade da tabela passava a
    /// ler-se como chave. A cerca é o **parêntese** que abre o tuplo, e este teste é o que a
    /// impede de ser «simplificada» de volta.
    #[test]
    fn um_match_declara_a_chave_e_nunca_o_valor() {
        let dir = std::env::temp_dir().join(format!("ph2d-keys-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("o temporário cria");
        let f = dir.join("tabela.rs");
        let mut h = std::fs::File::create(&f).expect("o ficheiro abre");
        h.write_all(
            b"pub fn tr(key: &str) -> Option<&'static str> {\n              Some(match key {\n              \"a.chave\" => \"  ...When Crouching\",\n              _ => return None,\n              })\n}\n",
        )
        .expect("escreve");
        drop(h);

        let achadas = super::keys_declared(&dir, &["tabela.rs"], "");
        assert!(
            achadas.contains("a.chave"),
            "a chave de um `match` deixou de ser vista: {achadas:?}"
        );
        assert!(
            !achadas.contains("  ...When Crouching"),
            "o VALOR de um `match` está a ser lido como chave — é o falso positivo de 17/09, e \
             com o prefixo vazio ele leva metade da tabela: {achadas:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// ⭐ E a outra metade: uma lista de TUPLOS declara o PRIMEIRO elemento, nunca o segundo.
    #[test]
    fn um_tuplo_declara_o_primeiro_e_nunca_o_segundo() {
        let dir = std::env::temp_dir().join(format!("ph2d-keys-t-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("o temporário cria");
        let f = dir.join("tabela.rs");
        std::fs::write(
            &f,
            "pub const T: &[(&str, &str)] = &[\n    (\"a.chave\", \"Cylinder\"),\n];\n",
        )
        .expect("escreve");

        let achadas = super::keys_declared(&dir, &["tabela.rs"], "");
        assert!(
            achadas.contains("a.chave"),
            "a chave de um TUPLO não é vista — o censo mediria zero declaradas: {achadas:?}"
        );
        assert!(
            !achadas.contains("Cylinder"),
            "o VALOR de um tuplo está a ser lido como chave: {achadas:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod tests_forma_da_chave {
    use super::looks_like_a_key;

    /// ⭐⭐ **Os dois falsos positivos MEDIDOS em 2026-09-17**, guardados como controlo.
    ///
    /// Sem eles a régua re-escreve-se «simplificada» e volta a ler um caminho de campo e um nome
    /// de ficheiro como texto que o artista lê. *Uma regra sem o caso que a motivou é uma regra
    /// que alguém apaga por parecer arbitrária.*
    #[test]
    fn a_filename_is_not_a_key() {
        assert!(!looks_like_a_key("zz_fixtura.rs", "zz_fixtura."));
        assert!(!looks_like_a_key("zz_fixtura.physics.toml", "zz_fixtura."));
        // ⛔ O FALSO NEGATIVO que a 1.ª redacção criou: `png` aqui é um FORMATO, não um ficheiro.
        // ⚠️⚠️ **O prefixo da fixtura NÃO é o de nenhum censo vivo, e isso é a lei deste ficheiro:**
        //    a 1.ª redacção escreveu `"shell.sheet_export.png"` e o censo da SHELL leu a minha
        //    fixtura como uma chave em uso — *um censo que varre o repo inteiro lê o ficheiro do
        //    censo*, que é a armadilha que o doc do [`looks_like_a_key`] já narra uma vez.
        assert!(looks_like_a_key(
            "zz_fixtura.sheet_export.png",
            "zz_fixtura."
        ));
        assert!(looks_like_a_key("zz_fixtura.export.json", "zz_fixtura."));
        // ⚠️ O CONTROLO POSITIVO: uma chave legítima com um segmento parecido continua a passar.
        assert!(looks_like_a_key(
            "paint_brush.brush_blend.mix",
            "paint_brush."
        ));
        assert!(looks_like_a_key(
            "zz_fixtura.physics.hold_stiffness",
            "zz_fixtura."
        ));
    }

    /// ⚠️ **E as duas cercas que já lá estavam** — um prefixo não é uma chave, e uma chave não tem
    /// maiúsculas nem espaços.
    #[test]
    fn a_prefix_is_not_a_key_and_neither_is_a_sentence() {
        assert!(!looks_like_a_key(
            "zz_fixtura.inspector.player.",
            "zz_fixtura."
        ));
        assert!(!looks_like_a_key("zz_fixtura.Foo", "zz_fixtura."));
        assert!(!looks_like_a_key("zz_fixtura.a b", "zz_fixtura."));
    }
}

/// Um par declarado numa tabela de strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Par {
    /// A chave (`panel.x.y`).
    pub chave: String,
    /// O inglês que ela devolve, com as continuações de linha já juntas.
    pub texto: String,
    /// A linha do TEXTO, a contar de 1.
    pub linha: usize,
}

/// ⭐⭐⭐ **OS PARES DE UMA TABELA, NAS DUAS FORMAS QUE ESTE REPO USA.**
///
/// ⛔⛔ **Ela existe porque havia DOIS leitores da mesma tabela e só um aprendeu a segunda forma.**
/// O [`keys_declared`] acima aprendeu o TUPLO (`("k", "v"),`) em 2026-09-17; o leitor do gate
/// `a_tabela_inglesa_fala_ingles` ficou a conhecer só o `match` (`"k" => "v"`) — e assim
/// **1 432 entradas, 23 % da tabela**, nunca foram conferidas contra o português
/// (`node_options` 601 · `node_params_motion` 430 · `node_params` 398).
///
/// ⚠️⚠️ **E o piso de população não o podia dizer:** ele exige `>= 5 000` entradas, e a forma que a
/// régua conhece traz `4 815` + as outras tabelas — *um piso satisfeito pela forma que a régua
/// conhece não afirma nada sobre a forma que ela não conhece.*
///
/// # A cerca do TUPLO, que a 1.ª redacção da irmã pagou
///
/// Num `match` o VALOR também acaba em vírgula (`"k" => "v",`), logo *«há uma vírgula entre os
/// dois»* emparelharia o valor de uma linha com a chave da seguinte. ⇒ um tuplo reconhece-se pelo
/// **PARÊNTESE que o abre**, e é essa a cerca.
#[must_use]
pub fn declared_pairs_in(src: &str) -> Vec<Par> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    // `(texto, índice do `"` de abertura, índice logo após o `"` de fecho)`
    let mut ultima: Option<(String, usize, usize)> = None;
    while i < b.len() {
        if b[i] != '"' {
            i += 1;
            continue;
        }
        let inicio = i;
        i += 1;
        let mut s = String::new();
        while i < b.len() && b[i] != '"' {
            if b[i] == '\\' {
                i += 1;
                match b.get(i) {
                    // ⚠️⚠️ **A CONTINUAÇÃO NÃO INSERE NADA**, e é aqui que o leitor irmão erra:
                    // em Rust, `\` seguido de quebra come a quebra **e** o espaço em branco do
                    // início da linha seguinte, e acrescenta o VAZIO. O leitor do gate
                    // `a_tabela_inglesa_fala_ingles` empurra um `' '`, logo devolve
                    // `"uma frase  comprida"` (dois espaços) onde o binário tem um. Para um censo
                    // de LÍNGUA isso é inócuo — a tokenização não vê espaços a mais —, e para uma
                    // comparação EXACTA com o que um painel pintou é a diferença entre casar e
                    // acusar. *Um leitor que erra num espaço só é apanhado por quem compara ao bit.*
                    Some('\n') => {
                        i += 1;
                        while i < b.len() && b[i].is_whitespace() {
                            i += 1;
                        }
                        continue;
                    }
                    // ⛔⛔ **O `\u{…}` tem de ser DESCODIFICADO, senão a régua lê `u{25be}`.**
                    // Medido: a tabela declara `"+ Track  \u{25be}"` e o painel pinta `+ Track ▾`
                    // — sem esta linha o censo acusa um rótulo que veio da tabela. *Uma régua que
                    // lê o ESCAPE em vez do carácter compara duas coisas diferentes.*
                    Some('u') if b.get(i + 1) == Some(&'{') => {
                        let mut j = i + 2;
                        let mut hex = String::new();
                        while j < b.len() && b[j] != '}' {
                            hex.push(b[j]);
                            j += 1;
                        }
                        if let Some(c) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                        {
                            s.push(c);
                        }
                        i = j;
                    }
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('0') => s.push('\0'),
                    Some(c) => s.push(*c),
                    None => break,
                }
            } else {
                s.push(b[i]);
            }
            i += 1;
        }
        i += 1;
        if let Some((chave, abre, fim)) = ultima.take() {
            let meio: String = b[fim..inicio].iter().collect();
            let antes: String = b[..abre].iter().collect();
            let seta = meio.contains("=>");
            let tuplo = antes.trim_end().ends_with('(') && meio.trim() == ",";
            if seta || tuplo {
                let linha = b[..inicio].iter().filter(|c| **c == '\n').count() + 1;
                out.push(Par {
                    chave,
                    texto: s,
                    linha,
                });
                continue;
            }
        }
        ultima = Some((s, inicio, i));
    }
    out
}

#[cfg(test)]
mod tests_pares_das_duas_formas {
    use super::*;

    fn pares(src: &str) -> Vec<(String, String)> {
        declared_pairs_in(src)
            .into_iter()
            .map(|p| (p.chave, p.texto))
            .collect()
    }

    #[test]
    fn a_forma_match_e_a_forma_tuplo_leem_se_as_duas() {
        assert_eq!(
            pares("        \"a.b\" => \"Mix\",\n"),
            vec![("a.b".to_string(), "Mix".to_string())]
        );
        assert_eq!(
            pares("    (\"c.d\", \"Angle\"),\n"),
            vec![("c.d".to_string(), "Angle".to_string())]
        );
    }

    /// ⛔⛔ **A cerca do PARÊNTESE, e o fenómeno vive num ARRAY — não num `match`.**
    ///
    /// ⚠️ A 1.ª redacção deste controlo pôs duas linhas de `match` lado a lado e a mutação que
    /// apaga a cerca **SOBREVIVEU**: depois de emparelhar por `=>` o percurso segue sem guardar o
    /// valor, logo `"Mix"` nunca chega a ser candidato a chave. *Uma mutação que o corpus não
    /// discrimina lê-se exactamente como uma lei que não existe.*
    ///
    /// Quem discrimina é um **array** (`&["Paint", "Erase"]`, que as tabelas deste repo têm aos
    /// montes): ali há uma vírgula entre dois literais e o que abre é um `[`. Sem a cerca, a régua
    /// inventa o par `("Paint", "Erase")`.
    #[test]
    fn dois_literais_separados_por_virgula_num_array_nao_sao_um_par() {
        assert!(
            pares("const X: &[&str] = &[\"Paint\", \"Erase\"];\n").is_empty(),
            "um array não declara pares: {:?}",
            pares("const X: &[&str] = &[\"Paint\", \"Erase\"];\n")
        );
        // ⭐ E o CONTROLO do outro lado: com o parêntese, é um par.
        assert_eq!(
            pares("    (\"c.d\", \"Angle\"),\n"),
            vec![("c.d".to_string(), "Angle".to_string())]
        );
    }

    /// ⛔ **O `\u{…}` chega como CARÁCTER** — sem isto a régua compara `u{25be}` com `▾`.
    #[test]
    fn um_escape_unicode_chega_descodificado() {
        let v = pares("        \"a.b\" => \"+ Track  \\u{25be}\",\n");
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].1, "+ Track  \u{25be}");
        assert!(
            !v[0].1.contains('{'),
            "o escape sobreviveu cru: {:?}",
            v[0].1
        );
    }

    /// Duas entradas de `match` seguidas continuam a ler-se como duas.
    #[test]
    fn duas_entradas_de_match_leem_se_como_duas() {
        let v = pares("        \"a.b\" => \"Mix\",\n        \"e.f\" => \"Add\",\n");
        assert_eq!(v.len(), 2, "{v:?}");
        assert_eq!(v[0].1, "Mix");
        assert_eq!(v[1], ("e.f".to_string(), "Add".to_string()));
    }

    /// ⚠️ A continuação de linha faz parte do literal — sem ela a frase chega partida ao censo.
    #[test]
    fn uma_entrada_partida_em_duas_linhas_chega_inteira() {
        let v = pares("        \"a.b\" => \"uma frase \\\n             comprida\",\n");
        assert_eq!(v.len(), 1, "{v:?}");
        // ⚠️ UM espaço — o que vinha ANTES do `\`. A continuação não acrescenta nenhum.
        assert_eq!(v[0].1, "uma frase comprida");
    }
}
