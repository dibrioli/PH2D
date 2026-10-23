//! O que a família pergunta à árvore sobre o script (CodeEditor Onda 2, I4): as propriedades
//! `ph2d.property` declaradas no topo (o `self.` da lista de completação — o `luau-lsp` não as liga ao
//! `self`) e se um byte está em CÓDIGO (a lista não abre a escrever numa string nem num comentário).

use crate::{HighlightKind, Syntax};
use ph2d_code_buffer::Text;

/// O tipo de um [`Symbol`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Property,
    Variable,
}

/// Um símbolo do script: o nome, o tipo e o byte do início do nome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub byte: usize,
}

/// A função da API que recebe um nome de objecto ([`Syntax::object_references`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectCall {
    /// `ph2d.object` — o objecto tem de existir (o editor acusa a falta).
    Object,
    /// `ph2d.find_by_name` — a pergunta «existe?» (`nil` é resposta).
    FindByName,
    /// `ph2d.spawn` — o nome de um MESTRE.
    Spawn,
}

impl ObjectCall {
    fn of(callee: &str) -> Option<Self> {
        match callee {
            "ph2d.object" => Some(Self::Object),
            "ph2d.find_by_name" => Some(Self::FindByName),
            "ph2d.spawn" => Some(Self::Spawn),
            _ => None,
        }
    }
}

/// Um nome de objecto escrito no script: o nome (sem escapes), os bytes DENTRO das aspas e a função.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectRef {
    pub name: String,
    pub range: std::ops::Range<usize>,
    pub call: ObjectCall,
}

/// O conteúdo de uma string Luau com os escapes simples desfeitos (`\"`, `\'`, `\\`).
fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(n) = chars.next()
        {
            out.push(n);
        } else {
            out.push(c);
        }
    }
    out
}

impl Syntax {
    /// Os nomes das `ph2d.property("<nome>", …)` do topo, pela ordem.
    #[must_use]
    pub fn declared_properties(&self, text: &Text) -> Vec<String> {
        self.property_declarations(text)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    }

    /// As `ph2d.property("<nome>", …)` do topo, pela ordem: o nome e os bytes dele (dentro das aspas).
    #[must_use]
    pub fn property_declarations(&self, text: &Text) -> Vec<(String, std::ops::Range<usize>)> {
        let root = self.tree().root_node();
        let mut c = root.walk();
        root.named_children(&mut c)
            .filter(|n| n.kind() == "function_call")
            .filter_map(|n| {
                let name = n.child_by_field_name("name")?;
                if text.slice(name.byte_range()) != "ph2d.property" {
                    return None;
                }
                let args = n.child_by_field_name("arguments")?;
                let first = args.named_child(0).filter(|a| a.kind() == "string")?;
                let content = first.child_by_field_name("content")?;
                Some((text.slice(content.byte_range()), content.byte_range()))
            })
            .collect()
    }

    /// Os símbolos do script, pela ordem do texto (o `@` da paleta, I14): as funções (de qualquer
    /// profundidade, com o nome inteiro — `M.soma`, `M:metodo`), as `ph2d.property` do topo e os locais
    /// do topo. O byte é o do INÍCIO do nome. Funcionam sem o assistente (o `documentSymbol` do
    /// `luau-lsp` não dá as propriedades, medição I9_I12_I14 §I14).
    #[must_use]
    pub fn symbols(&self, text: &Text) -> Vec<Symbol> {
        fn functions(n: tree_sitter::Node, text: &Text, out: &mut Vec<Symbol>) {
            if n.kind() == "function_declaration"
                && let Some(name) = n.child_by_field_name("name")
            {
                out.push(Symbol {
                    name: text.slice(name.byte_range()),
                    kind: SymbolKind::Function,
                    byte: name.start_byte(),
                });
            }
            let mut c = n.walk();
            for ch in n.named_children(&mut c) {
                functions(ch, text, out);
            }
        }
        let root = self.tree().root_node();
        let mut out = Vec::new();
        functions(root, text, &mut out);
        let mut c = root.walk();
        for n in root.named_children(&mut c) {
            if n.kind() != "variable_declaration" {
                continue;
            }
            let list = n
                .named_child(0)
                .filter(|a| a.kind() == "assignment_statement")
                .and_then(|a| a.named_child(0))
                .or_else(|| n.named_child(0))
                .filter(|l| l.kind() == "variable_list");
            let Some(list) = list else { continue };
            let mut lc = list.walk();
            for name in list.children_by_field_name("name", &mut lc) {
                out.push(Symbol {
                    name: text.slice(name.byte_range()),
                    kind: SymbolKind::Variable,
                    byte: name.start_byte(),
                });
            }
        }
        out.extend(
            self.property_declarations(text)
                .into_iter()
                .map(|(name, r)| Symbol {
                    name,
                    kind: SymbolKind::Property,
                    byte: r.start,
                }),
        );
        out.sort_by_key(|s| s.byte);
        out
    }

    /// `byte` está no TOPO do script: fora do corpo de qualquer função (o nome de uma função a meio de
    /// escrever, `function up|`, ainda é topo).
    #[must_use]
    pub fn at_top_level(&self, text: &Text, byte: usize) -> bool {
        let byte = byte.min(text.len());
        let root = self.tree().root_node();
        let mut n = root.descendant_for_byte_range(byte, byte);
        while let Some(node) = n {
            // Depois do INÍCIO dos parâmetros e até ao `end` (o `body` acaba na última instrução: uma
            // linha vazia antes do `end` ainda é dentro). ⚠️ O início, não o fim: numa árvore só
            // deslocada (o reparse ainda no worker) o tree-sitter estica o `)` por cima do que se
            // escreveu logo a seguir a ele — medido: `parameters 13..25` sobre `(self)\n  on_`.
            if matches!(node.kind(), "function_declaration" | "function_definition")
                && let Some(params) = node.child_by_field_name("parameters")
            {
                let close = node
                    .child(node.child_count().saturating_sub(1))
                    .filter(|c| c.kind() == "end")
                    .map_or(node.end_byte(), |c| c.start_byte());
                if params.start_byte() < byte && byte <= close {
                    return false;
                }
            }
            n = node.parent();
        }
        true
    }

    /// ⭐ **Os nomes de objecto escritos no script** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
    /// §3.5–8): o 1.º argumento LITERAL de `ph2d.object`, `ph2d.find_by_name` e `ph2d.spawn` (o mestre),
    /// a qualquer profundidade, pela ordem do texto. Um nome montado (`"Por" .. "ta"`, uma variável)
    /// não conta: o renomear só reescreve o que se lê, e o problema só acusa o que se sabe.
    #[must_use]
    pub fn object_references(&self, text: &Text) -> Vec<ObjectRef> {
        fn walk(n: tree_sitter::Node, text: &Text, out: &mut Vec<ObjectRef>) {
            if n.kind() == "function_call"
                && let Some(name) = n.child_by_field_name("name")
                && let Some(call) = ObjectCall::of(&text.slice(name.byte_range()))
                && let Some(first) = n
                    .child_by_field_name("arguments")
                    .and_then(|a| a.named_child(0))
                    .filter(|a| a.kind() == "string")
                && let Some(content) = first.child_by_field_name("content")
            {
                out.push(ObjectRef {
                    name: unescape(&text.slice(content.byte_range())),
                    range: content.byte_range(),
                    call,
                });
            }
            let mut c = n.walk();
            for ch in n.named_children(&mut c) {
                walk(ch, text, out);
            }
        }
        let mut out = Vec::new();
        walk(self.tree().root_node(), text, &mut out);
        out
    }

    /// ⭐ **Os nomes de propriedade escritos no script** (doc CodeEditor 08 §3.4): o 2.º argumento
    /// LITERAL de `ph2d.get`/`ph2d.set` (a pose ou `"health.max"`), pela ordem do texto —
    /// `(nome sem escapes, bytes DENTRO das aspas)`.
    #[must_use]
    pub fn property_references(&self, text: &Text) -> Vec<(String, std::ops::Range<usize>)> {
        fn walk(
            n: tree_sitter::Node,
            text: &Text,
            out: &mut Vec<(String, std::ops::Range<usize>)>,
        ) {
            if n.kind() == "function_call"
                && let Some(name) = n.child_by_field_name("name")
                && matches!(&*text.slice(name.byte_range()), "ph2d.get" | "ph2d.set")
                && let Some(second) = n
                    .child_by_field_name("arguments")
                    .and_then(|a| a.named_child(1))
                    .filter(|a| a.kind() == "string")
                && let Some(content) = second.child_by_field_name("content")
            {
                out.push((
                    unescape(&text.slice(content.byte_range())),
                    content.byte_range(),
                ));
            }
            let mut c = n.walk();
            for ch in n.named_children(&mut c) {
                walk(ch, text, out);
            }
        }
        let mut out = Vec::new();
        walk(self.tree().root_node(), text, &mut out);
        out
    }

    /// O byte onde começa a 1.ª linha do CORPO da função do topo `name` (a linha a seguir ao
    /// cabeçalho), ou `None` se o script não a tem. É onde o Ctrl de um arrasto no topo guarda o
    /// objecto em `self` (o `init` é o `@onready` do Godot, §3.2).
    #[must_use]
    pub fn top_function_body(&self, text: &Text, name: &str) -> Option<usize> {
        let root = self.tree().root_node();
        let mut c = root.walk();
        let f = root.named_children(&mut c).find(|n| {
            // ⚠️ `local function init` tem o mesmo nó, mas o motor só chama GLOBAIS (`module.hook`).
            n.kind() == "function_declaration"
                && n.child(0).is_none_or(|k| k.kind() != "local")
                && n.child_by_field_name("name")
                    .is_some_and(|m| text.slice(m.byte_range()) == name)
        })?;
        let header_end = f.child_by_field_name("parameters")?.end_byte();
        let line = text.line_of_byte(header_end);
        Some(text.byte_of_line(line + 1))
    }

    /// O carácter antes de `byte` é código (nem string nem comentário).
    #[must_use]
    pub fn in_code(&self, text: &Text, byte: usize) -> bool {
        let byte = byte.min(text.len());
        if byte == 0 {
            return true;
        }
        let prev = text.prev_grapheme(byte);
        !self
            .highlights(text, prev..byte)
            .iter()
            .any(|h| matches!(h.kind, HighlightKind::String | HighlightKind::Comment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_nomes_de_objecto_a_qualquer_profundidade_e_so_os_literais() {
        let src = "local a = ph2d.object(\"Porta\")\nfunction update(self)\n  if ph2d.find_by_name('Caixa Grande') then\n    ph2d.spawn(\"Bala\", 1, 2)\n  end\n  ph2d.object(nome)\n  ph2d.object(\"Por\" .. \"ta\")\n  ph2d.object(\"Diz \\\"oi\\\"\")\n  ph2d.get(\"Porta\")\nend\n";
        let t = Text::new(src);
        let got: Vec<(String, ObjectCall, String)> = Syntax::new(&t)
            .object_references(&t)
            .into_iter()
            .map(|r| (r.name, r.call, t.slice(r.range)))
            .collect();
        assert_eq!(
            got,
            [
                ("Porta".into(), ObjectCall::Object, "Porta".into()),
                (
                    "Caixa Grande".into(),
                    ObjectCall::FindByName,
                    "Caixa Grande".into()
                ),
                ("Bala".into(), ObjectCall::Spawn, "Bala".into()),
                (
                    "Diz \"oi\"".into(),
                    ObjectCall::Object,
                    "Diz \\\"oi\\\"".into()
                ),
            ]
        );
    }

    #[test]
    fn o_corpo_da_funcao_do_topo_comeca_na_linha_a_seguir_ao_cabecalho() {
        let src = "local x = 1\nfunction init(self)\n  x = 2\nend\nfunction update(self) end\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        assert_eq!(
            s.top_function_body(&t, "init"),
            Some(src.find("  x = 2").unwrap())
        );
        assert_eq!(s.top_function_body(&t, "nada"), None);
        let local = Text::new("local function init(self)\nend\n");
        assert_eq!(
            Syntax::new(&local).top_function_body(&local, "init"),
            None,
            "um local function init não é o gancho"
        );
    }

    #[test]
    fn as_propriedades_do_topo_e_so_elas() {
        let t = Text::new(
            "ph2d.property(\"amplitude\", 1.0)\nph2d.property('speed', 2)\nlocal x = ph2d.property(\"nao\", 1)\nfunction init(self)\n  ph2d.property(\"dentro\", 1)\nend\n",
        );
        let s = Syntax::new(&t);
        assert_eq!(s.declared_properties(&t), ["amplitude", "speed"]);
    }

    #[test]
    fn os_simbolos_pela_ordem_do_texto() {
        let src = "ph2d.property(\"speed\", 2)\nlocal LIMITE, outro: number = 3, 4\nlocal function ajuda(v)\n  local dentro = 1\n  return v\nend\nlocal M = {}\nfunction M.soma(a) end\nfunction M:metodo() end\nX = 1\nlocal f = function() end\n";
        let t = Text::new(src);
        let got: Vec<(String, SymbolKind)> = Syntax::new(&t)
            .symbols(&t)
            .into_iter()
            .map(|s| (s.name, s.kind))
            .collect();
        let want = [
            ("speed", SymbolKind::Property),
            ("LIMITE", SymbolKind::Variable),
            ("outro", SymbolKind::Variable),
            ("ajuda", SymbolKind::Function),
            ("M", SymbolKind::Variable),
            ("M.soma", SymbolKind::Function),
            ("M:metodo", SymbolKind::Function),
            ("f", SymbolKind::Variable),
        ];
        assert_eq!(
            got,
            want.map(|(n, k)| (n.to_owned(), k)),
            "o local de dentro e a atribuição a um global não são símbolos"
        );
    }

    #[test]
    fn o_topo_e_fora_do_corpo_das_funcoes() {
        let src = "ph2d.property(\"a\", 1)\nupd\nfunction init(self)\n  x = 1\n  \nend\nlocal f = function()\n  y = 2\nend\nfunction up\n";
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let at = |p: &str| src.find(p).unwrap() + p.len();
        assert!(s.at_top_level(&t, at("\nupd")));
        assert!(!s.at_top_level(&t, at("  x")));
        assert!(
            !s.at_top_level(&t, at("x = 1\n  ")),
            "a linha vazia antes do `end`"
        );
        assert!(!s.at_top_level(&t, at("  y")));
        assert!(s.at_top_level(&t, at("function up")));
        assert!(s.at_top_level(&t, at("end\n")));
    }

    #[test]
    fn numa_arvore_so_deslocada_o_corpo_continua_dentro() {
        let mut b = ph2d_code_buffer::Buffer::new("function init(self)\nend\n");
        let mut s = Syntax::new(b.text());
        b.set_selection(ph2d_code_buffer::Selection::single(
            ph2d_code_buffer::Range::cursor(19),
        ));
        let edits = b.insert("\n  on_");
        s.edit_deferred(&edits);
        assert!(s.is_stale());
        assert!(
            !s.at_top_level(b.text(), 22),
            "o `on_` está no corpo do `init`"
        );
    }

    #[test]
    fn strings_e_comentarios_nao_sao_codigo() {
        let t = Text::new("local a = \"ab\" -- cd\nlocal b\n");
        let s = Syntax::new(&t);
        let at = |p: &str| t.slice(0..t.len()).find(p).unwrap() + p.len();
        assert!(s.in_code(&t, at("local a")));
        assert!(!s.in_code(&t, at("\"ab")));
        assert!(!s.in_code(&t, at("-- cd")));
        assert!(s.in_code(&t, at("local b")));
    }
}
