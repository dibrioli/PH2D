//! ⭐⭐⭐ **A FRONTEIRA DOS MOTORES** — o rótulo que uma crate PUBLICA e outra PINTA.
//!
//! # ⛔⛔ As duas réguas que já existiam são cegas a ele, e por construção
//!
//! | régua | vê o literal? | sabe se ele é PINTADO? |
//! |---|---|---|
//! | [`crate::language_literals`] (lexical, 30 gates) | sim, **se a crate estiver na lista** | não |
//! | `scripts/censo-texto-pintado.py` (de porta) | sim | sim — **dentro da MESMA crate** |
//!
//! Um rótulo que atravessa a fronteira escapa às duas: do lado do motor não há pintor nenhum a
//! seguir, e do lado do painel **o literal não existe**. ⇒ *um censo cuja crate não é DONA do texto
//! que ela pinta fica verde sobre texto cru* — o que o cabeçalho do `ph2d_i18n::sculpt_engine` já
//! escrevia em prosa depois de o defeito aparecer **três vezes**, todas achadas por uma fotografia
//! do dono, e que nenhum instrumento enumerava.
//!
//! # A população: o que o motor PUBLICA
//!
//! Um literal com cara de língua que sai da crate por uma das duas portas:
//!
//! - **[`Publicacao::Funcao`]** — ele é devolvido por uma `fn … -> &str` (com `'static` ou sem);
//! - **[`Publicacao::Campo`]** — ele é o valor de um campo `label` / `name` / `title` / `text` num
//!   literal de struct, que é como um CATÁLOGO publica (o `ShapeDesc` do vector, os moldes do
//!   L-System).
//!
//! # ⛔ A cegueira que esta régua já pagou, e que o teste guarda
//!
//! A 1.ª redacção exigia `-> &'static str` e lia **zero** nos dez nomes de ferramenta
//! (`fn label(&self) -> &str`, a assinatura que o contrato `Tool` impõe — *uma função de trait não
//! escolhe o tipo de retorno dela*). ⇒ a régua aceita as duas formas, e
//! `a_regua_ve_a_assinatura_que_o_contrato_impoe` é o controlo.
//!
//! # ⭐ Ela erra para o lado ALTO, como a irmã
//!
//! Um nome publicado que ninguém pinta aparece aqui — e é *a terceira espécie* do `CLAUDE.md` §5.0:
//! um **ÓRFÃO** (cura: apagar) lê-se igual a um **MORTO** (cura: ligar). A régua não os separa e não
//! tem como: quem julga é quem lê a lista, e escreve o mecanismo ao lado.

use std::path::Path;

use crate::{Literal, language_literals};

/// ⭐⭐ **O fonte sem COMENTÁRIOS** — a porta para quem pergunta *«este ficheiro NOMEIA um tipo?»*.
///
/// ⛔⛔ Sem ela a pergunta erra para o lado alto de uma maneira que não se cura por afinação: dois
/// painéis do Inspector citam `SignalVerb::uses_arg` numa **linha de prosa** e seriam acusados de
/// o chamar. *Um tipo nomeado num comentário não pode ser chamado.*
///
/// ⚠️ Ela devolve o texto com os comentários **apagados**, e não removidos: as posições e as linhas
/// ficam onde estavam, que é o que um censo precisa para dizer `ficheiro:linha`.
#[must_use]
pub fn sem_comentarios(src: &str) -> String {
    crate::source::strip_comments(src).into_iter().collect()
}

/// Por que porta o literal sai da crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Publicacao {
    /// Devolvido por uma função cujo retorno é um `&str` — a assinatura, para quem lê a lista.
    Funcao(String),
    /// O valor de um campo de catálogo (`label`, `name`, `title`, `text`).
    Campo(String),
}

/// Um nome que o motor publica.
#[derive(Clone, Debug)]
pub struct Publicado {
    /// O literal, como a régua lexical o vê.
    pub literal: Literal,
    /// Por que porta ele sai.
    pub via: Publicacao,
}

/// Os campos de um literal de struct que são vocabulário de interface.
const CAMPOS: &[&str] = &["label", "name", "title", "text"];

/// A linha declara uma função cujo retorno é um `&str`?
///
/// ⚠️ **Sem exigir `pub`**: o que decide é a assinatura, não a visibilidade — um `impl Trait for T`
/// não escreve `pub` nos métodos dele, e os dez nomes de ferramenta vivem exactamente aí.
fn devolve_str(linha: &str) -> bool {
    let t = linha.trim_start();
    if !abre_funcao(linha) {
        return false;
    }
    let Some((_, dep)) = t.split_once("->") else {
        return false;
    };
    let dep = dep.trim();
    dep == "&str"
        || dep == "&'static str"
        || dep.starts_with("&str ")
        || dep.starts_with("&'static str ")
}

/// A linha é a declaração de UMA função (qualquer uma)?
fn abre_funcao(linha: &str) -> bool {
    let t = linha.trim_start();
    t.starts_with("fn ")
        || t.starts_with("pub fn ")
        || t.starts_with("const fn ")
        || t.starts_with("pub const fn ")
        || t.starts_with("pub(crate) fn ")
        || t.starts_with("pub(crate) const fn ")
        || t.starts_with("pub(super) fn ")
}

/// A linha é `<campo>: "…"` para um dos [`CAMPOS`]?
fn campo_de_catalogo(linha: &str) -> Option<String> {
    let t = linha.trim_start();
    let (nome, resto) = t.split_once(':')?;
    let nome = nome.trim();
    if !CAMPOS.contains(&nome) {
        return None;
    }
    resto
        .trim_start()
        .starts_with('"')
        .then(|| nome.to_string())
}

/// ⭐⭐ **Os nomes que os `.rs` debaixo de `src_root` publicam.**
///
/// ⚠️ A varredura lexical por baixo é a [`language_literals`], **memoizada por raiz** — chamar isto
/// depois de um censo lexical da mesma raiz não relê a árvore.
/// ⭐⭐ **O custo é por FICHEIRO, não por literal** — e a 1.ª redacção fazia o contrário: ela lia e
/// partia em linhas o ficheiro **uma vez por literal**, o que no `ph2d-i18n` (165 literais num
/// `lib.rs` grande) era o mesmo trabalho 165 vezes. *Um gate que se aproxima do tecto de morte do
/// executor vira membro da família de flakes de fan-out sem uma linha de lógica mudar.*
///
/// ⚠️ A varredura por baixo devolve os literais **agrupados por ficheiro** (ela ordena-os); se um
/// dia deixar de o fazer, este laço lê o ficheiro outra vez — mais lento, nunca errado.
#[must_use]
pub fn published_names(src_root: &Path) -> Vec<Publicado> {
    let todos = language_literals(src_root);
    let mut out = Vec::new();
    let mut i = 0;
    while i < todos.len() {
        let rel = todos[i].rel.clone();
        let mut j = i;
        while j < todos.len() && todos[j].rel == rel {
            j += 1;
        }
        if let Ok(raw) = std::fs::read_to_string(src_root.join(&rel)) {
            let linhas: Vec<&str> = raw.lines().collect();
            for l in &todos[i..j] {
                if let Some(p) = publica_em(l, &linhas) {
                    out.push(p);
                }
            }
        }
        i = j;
    }
    out
}

/// A régua sobre UM literal e o texto do ficheiro dele — a porta dos testes de controlo.
#[must_use]
pub fn publica(l: &Literal, src: &str) -> Option<Publicado> {
    publica_em(l, &src.lines().collect::<Vec<&str>>())
}

/// A régua com as linhas já partidas — a porta que o laço por ficheiro usa.
fn publica_em(l: &Literal, linhas: &[&str]) -> Option<Publicado> {
    let i = l.line.checked_sub(1)?;
    let atual = linhas.get(i)?;
    if let Some(campo) = campo_de_catalogo(atual) {
        return Some(Publicado {
            literal: l.clone(),
            via: Publicacao::Campo(campo),
        });
    }
    // Sobe até à declaração de função mais próxima. ⛔ Pára num `}` de coluna zero (fim do bloco de
    // topo): sem essa cerca, um literal solto herdaria a assinatura da função ACIMA dele.
    for j in (0..=i).rev() {
        let linha = linhas[j];
        if devolve_str(linha) {
            return Some(Publicado {
                literal: l.clone(),
                via: Publicacao::Funcao(linha.trim().to_string()),
            });
        }
        if abre_funcao(linha) || linha.starts_with('}') {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::language_literals_in;

    fn publicados(src: &str) -> Vec<(String, Publicacao)> {
        language_literals_in("x.rs", src)
            .iter()
            .filter_map(|l| publica(l, src))
            .map(|p| (p.literal.text, p.via))
            .collect()
    }

    /// ⛔⛔ **A cegueira que esta régua pagou na 1.ª redacção.** Ela exigia `-> &'static str` e lia
    /// ZERO nos dez nomes de ferramenta — `fn label(&self) -> &str` é a assinatura que o contrato
    /// `Tool` (§6, congelado) impõe, e *uma função de trait não escolhe o tipo de retorno dela*.
    #[test]
    fn a_regua_ve_a_assinatura_que_o_contrato_impoe() {
        let elidida = publicados(
            "impl Tool for T {\n    fn label(&self) -> &str {\n        \"Bg Removal\"\n    }\n}\n",
        );
        assert_eq!(
            elidida.len(),
            1,
            "a assinatura elidida publica: {elidida:?}"
        );
        assert_eq!(elidida[0].0, "Bg Removal");
        let estatica = publicados(
            "impl V {\n    pub fn label(self) -> &'static str {\n        \"Wet Paint\"\n    }\n}\n",
        );
        assert_eq!(
            estatica.len(),
            1,
            "a assinatura estática publica: {estatica:?}"
        );
    }

    /// ⭐ A 2.ª porta: um CATÁLOGO publica por campo, não por função.
    #[test]
    fn a_regua_ve_um_campo_de_catalogo() {
        let v =
            publicados("pub const SHAPES: &[D] = &[D {\n    label: \"Banner\",\n    n: 3,\n}];\n");
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].1, Publicacao::Campo("label".into()));
    }

    /// ⛔ **O controlo NEGATIVO, e ele é metade do valor:** um literal dentro de uma função que
    /// devolve outra coisa não é publicado — sem esta cerca a régua acusaria todo o motor.
    #[test]
    fn uma_funcao_que_devolve_outra_coisa_nao_publica() {
        assert!(publicados("fn f(&self) -> String {\n    format!(\"Wet Paint\")\n}\n").is_empty());
        assert!(
            publicados("fn f(&self) -> Option<&str> {\n    Some(\"Wet Paint\")\n}\n").is_empty()
        );
    }

    /// ⛔ **Um tipo nomeado num COMENTÁRIO não pode ser chamado** — o controlo da porta que a régua
    /// do pintor usa para não acusar prosa.
    #[test]
    fn a_porta_sem_comentarios_apaga_a_prosa_e_guarda_as_linhas() {
        let src = "// fala de SignalVerb\nlet x = OutroTipo::A;\n";
        let limpo = sem_comentarios(src);
        assert!(
            !limpo.contains("SignalVerb"),
            "a prosa sobreviveu: {limpo:?}"
        );
        assert!(
            limpo.contains("OutroTipo"),
            "o código foi apagado: {limpo:?}"
        );
        assert_eq!(
            limpo.lines().count(),
            src.lines().count(),
            "as linhas mudaram de número — um censo diria o endereço errado"
        );
    }

    /// ⛔⛔ **A cerca do `}` de coluna zero.** Sem ela um literal solto a seguir a uma função que
    /// devolve `&str` HERDA a assinatura dela, e a régua publica texto que ninguém devolve.
    #[test]
    fn um_literal_depois_do_bloco_nao_herda_a_assinatura_de_cima() {
        let src = "impl V {\n    fn label(&self) -> &str {\n        \"Digital\"\n    }\n}\n\nconst OUTRO: &str = \"Watercolor\";\n";
        let v = publicados(src);
        assert_eq!(v.len(), 1, "só o que a função devolve: {v:?}");
        assert_eq!(v[0].0, "Digital");
    }
}
