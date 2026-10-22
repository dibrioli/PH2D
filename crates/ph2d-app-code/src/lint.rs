//! ⭐ **I7 — o campo do `self` que ninguém dá, e o «você quis dizer»** (`docs/CodeEditor/03_onda2.md`).
//!
//! - **O lint é nosso:** o `luau-lsp` infere o `self` pelo uso e aceita `self.ampiltude` em qualquer modo
//!   (`ferramentas/medicoes_onda2/I7_acoes_e_voce_quis_dizer.md` §3). Acusa-se o campo LIDO que o script
//!   não dá ao `self` nem declara por `ph2d.property`, nem o motor põe lá (`id`). Se o `self` foge (ver
//!   [`ph2d_code_syntax::SelfFields`]) ou o texto tem um erro de sintaxe, cala-se.
//! - **Quando:** na pausa da escrita do Godot (1,5 s; 0,5 s com um aviso à vista — os mesmos
//!   `idle_parse_delay` das cores), nunca na tecla: entre pausas, os avisos seguem as edições.
//! - **A distância** é a que explica as 177 sugestões do `rustc` (§4 da medição): o nome igual sem contar
//!   maiúsculas; senão o de menor Damerau (OSA) com distância ≤ `max(len, 3) / 3`. Os empatados vão
//!   todos, pela ordem das candidatas.

use crate::doc::CodeDoc;
use crate::docs::CodeDocs;
use crate::semantic::{IDLE_ERRORS_NS, IDLE_NS};
use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_explain::Explained;
use ph2d_code_syntax::ObjectCall;
use ph2d_panel_code::{Explanation, Problem, Severity};
use ph2d_script::api_table::HOOKS;
use ph2d_script::props::RESERVED_NAMES;
use std::ops::Range;
use std::sync::Arc;

/// A única sugestão, se for uma (empates não se escolhem).
fn single(near: &[String]) -> Option<&str> {
    match near {
        [one] => Some(one.as_str()),
        _ => None,
    }
}

/// A distância de Damerau restrita (OSA): trocar duas letras vizinhas custa 1.
fn osa(a: &[char], b: &[char]) -> usize {
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let c = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + c);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// As candidatas mais perto de `typo` (vazio = nenhuma perto o bastante), pela ordem de `candidates`.
pub(crate) fn did_you_mean<'a>(
    typo: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let candidates: Vec<&str> = candidates.into_iter().filter(|c| *c != typo).collect();
    let mut out: Vec<String> = Vec::new();
    let push = |c: &str, out: &mut Vec<String>| {
        if !out.iter().any(|o| o == c) {
            out.push(c.to_owned());
        }
    };
    for c in &candidates {
        if c.to_lowercase() == typo.to_lowercase() {
            push(c, &mut out);
        }
    }
    if !out.is_empty() {
        return out;
    }
    let t: Vec<char> = typo.chars().collect();
    let limit = t.len().max(3) / 3;
    let dist: Vec<usize> = candidates
        .iter()
        .map(|c| osa(&t, &c.chars().collect::<Vec<_>>()))
        .collect();
    let Some(best) = dist.iter().copied().filter(|d| *d <= limit).min() else {
        return out;
    };
    for (c, d) in candidates.iter().zip(&dist) {
        if *d == best {
            push(c, &mut out);
        }
    }
    out
}

/// Um `self.<campo>` lido que ninguém dá.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldLint {
    pub bytes: Range<usize>,
    pub name: String,
    /// As que ele queria dizer (pela lei da distância).
    pub near: Vec<String>,
}

/// Um nome de objecto (ou de mestre) escrito no script que a cena não confirma (item 8,
/// `docs/CodeEditor/07_arrastar_para_o_codigo.md` §3.6): nenhum objecto o tem, ou vários.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectLint {
    pub bytes: Range<usize>,
    pub explained: Explained,
}

/// Os avisos de campo da aba e quando foram contados.
#[derive(Default)]
pub(crate) struct Lint {
    pub list: Vec<FieldLint>,
    pub objects: Vec<ObjectLint>,
    /// A (revisão do texto, geração da árvore, nomes da cena) da última contagem.
    counted: Option<(u64, u64, usize)>,
    /// A revisão vista no último quadro, e o relógio em que ela apareceu (a pausa conta daí).
    seen: Option<(u64, u128)>,
    /// Avança a cada mudança da lista (a chave dos problemas guardados, `CodeDoc::problems`).
    pub generation: u64,
}

impl Lint {
    /// Os avisos seguem as edições; um cujo nome uma edição TOCA larga-se (o nome mudou) — também
    /// a que escreve encostada a ele, antes ou depois.
    pub fn edited(&mut self, edits: &[TextEdit]) {
        self.generation += u64::from(!edits.is_empty());
        let follows = |e: &TextEdit, bytes: &mut Range<usize>| {
            if e.old_end_byte < bytes.start {
                let shift = |b: usize| b - e.old_end_byte + e.new_end_byte;
                *bytes = shift(bytes.start)..shift(bytes.end);
                true
            } else {
                e.start_byte > bytes.end
            }
        };
        for e in edits {
            self.list.retain_mut(|l| follows(e, &mut l.bytes));
            self.objects.retain_mut(|l| follows(e, &mut l.bytes));
        }
    }

    /// Os problemas a pintar.
    pub fn problems(&self, text: &Text) -> Vec<Problem> {
        self.list
            .iter()
            .filter(|l| l.bytes.end <= text.len())
            .map(|l| {
                let near = match l.near.as_slice() {
                    [one] => Some(one.as_str()),
                    _ => None,
                };
                let e = Explained::unknown_field(&l.name, near);
                Problem {
                    line: text.line_of_byte(l.bytes.start),
                    severity: Severity::Warning,
                    culprit: Some(l.bytes.clone()),
                    message: e.title(),
                    explanation: Some(Explanation {
                        kind: e.kind.id(),
                        title: e.title(),
                        why: e.why(),
                    }),
                }
            })
            .chain(
                self.objects
                    .iter()
                    .filter(|l| l.bytes.end <= text.len())
                    .map(|l| Problem {
                        line: text.line_of_byte(l.bytes.start),
                        severity: Severity::Warning,
                        culprit: Some(l.bytes.clone()),
                        message: l.explained.title(),
                        explanation: Some(Explanation {
                            kind: l.explained.kind.id(),
                            title: l.explained.title(),
                            why: l.explained.why(),
                        }),
                    }),
            )
            .collect()
    }
}

impl CodeDoc {
    /// Conta os avisos de campo de novo se o texto parou o bastante e a árvore é a dele.
    fn lint_tick(&mut self, now_ns: u128) {
        let rev = self.buffer.revision();
        let since = match self.lint.seen {
            Some((r, t)) if r == rev => t,
            _ => {
                // A 1.ª vez (aba aberta agora) conta já; uma revisão nova espera a pausa.
                let t = if self.lint.seen.is_none() { 0 } else { now_ns };
                self.lint.seen = Some((rev, t));
                t
            }
        };
        let key = (
            rev,
            self.syntax.generation(),
            Arc::as_ptr(&self.scene) as usize,
        );
        let idle = if self.lint.list.is_empty() && self.lint.objects.is_empty() {
            IDLE_NS
        } else {
            IDLE_ERRORS_NS
        };
        if self.lint.counted == Some(key)
            || self.syntax.is_stale()
            || now_ns < since.saturating_add(idle)
        {
            return;
        }
        self.lint.counted = Some(key);
        self.lint.list = self.field_lints();
        self.lint.objects = self.object_lints();
        self.lint.objects.extend(self.property_lints());
        self.lint.generation += 1;
    }

    /// Os nomes de objecto/mestre escritos que a cena não confirma (item 8). Sem cena lida (nenhum
    /// objecto nem mestre), cala-se: «não sei a cena» não é «a cena não tem nada».
    fn object_lints(&self) -> Vec<ObjectLint> {
        let scene = &*self.scene;
        if scene.objects.is_empty() && scene.masters.is_empty() {
            return Vec::new();
        }
        let objects = || scene.objects.keys().map(String::as_str);
        self.syntax
            .object_references(self.buffer.text())
            .into_iter()
            .filter_map(|r| {
                let explained = match (r.call, scene.objects.get(&r.name)) {
                    (ObjectCall::Spawn, _) if scene.masters.contains(&r.name) => return None,
                    (ObjectCall::Spawn, _) => {
                        let near = did_you_mean(&r.name, scene.masters.iter().map(String::as_str));
                        Explained::unknown_master(&r.name, single(&near))
                    }
                    (_, Some(&n)) if n > 1 => Explained::object_twice(&r.name, n),
                    (_, Some(_)) | (ObjectCall::FindByName, None) => return None,
                    (ObjectCall::Object, None) => {
                        let near = did_you_mean(&r.name, objects());
                        Explained::unknown_object(&r.name, single(&near))
                    }
                };
                Some(ObjectLint {
                    bytes: r.range,
                    explained,
                })
            })
            .collect()
    }

    /// Os nomes escritos em `ph2d.get/set` que nem a pose nem um componente têm (doc CodeEditor 08
    /// §3.4) — não precisa da cena: o catálogo é o mesmo para todas.
    fn property_lints(&self) -> Vec<ObjectLint> {
        let mut known = ph2d_script::api_table::property_names();
        known.extend(ph2d_script::io::POSE_FIELDS.iter().map(|f| (*f).to_owned()));
        self.syntax
            .property_references(self.buffer.text())
            .into_iter()
            .filter(|(name, _)| !known.contains(name))
            .map(|(name, bytes)| {
                let near = did_you_mean(&name, known.iter().map(String::as_str));
                ObjectLint {
                    bytes,
                    explained: Explained::unknown_property(&name, single(&near)),
                }
            })
            .collect()
    }

    fn field_lints(&self) -> Vec<FieldLint> {
        let t = self.buffer.text();
        let hooks: Vec<&str> = HOOKS.iter().map(|h| h.ident).collect();
        let Some(f) = self.syntax.self_fields(t, &hooks) else {
            return Vec::new();
        };
        let mut known = self.syntax.declared_properties(t);
        known.extend(f.assigned);
        known.extend(RESERVED_NAMES.iter().map(|s| (*s).to_owned()));
        f.reads
            .into_iter()
            .filter(|(name, _)| !known.contains(name))
            .map(|(name, bytes)| FieldLint {
                near: did_you_mean(&name, known.iter().map(String::as_str)),
                bytes,
                name,
            })
            .collect()
    }
}

impl CodeDocs {
    /// Um quadro: cada aba recontar os avisos de campo na pausa.
    pub(crate) fn lint(&mut self) {
        let now = self.clock.0;
        for d in &mut self.docs {
            d.lint_tick(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lei_do_rustc() {
        let campos = ["amplitude", "speed", "t", "id", "base_x", "base_y"];
        let dym = |t| did_you_mean(t, campos);
        assert_eq!(dym("ampiltude"), ["amplitude"], "troca de vizinhas = 1");
        assert_eq!(dym("aplitde"), ["amplitude"], "2 edições num nome de 7");
        assert_eq!(dym("AMPLITUDE"), ["amplitude"], "maiúsculas primeiro");
        assert_eq!(dym("ampl"), Vec::<String>::new(), "4 edições: longe");
        assert_eq!(dym("tt"), ["t"]);
        assert_eq!(dym("base_z"), ["base_x", "base_y"], "empates: todos");
        assert_eq!(
            dym("speed"),
            Vec::<String>::new(),
            "o próprio nome não se sugere"
        );
        assert_eq!(
            did_you_mean("pirnt", ["pairs", "print", "ipairs"]),
            ["print"]
        );
        assert_eq!(
            did_you_mean("gte", ["get", "set"]),
            ["get"],
            "o Python falha esta"
        );
    }
}
