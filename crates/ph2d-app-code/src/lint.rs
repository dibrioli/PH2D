//! I7 na aba: quando o lint corre sobre o texto de um script (a lei e o «você quis dizer» moram em
//! `ph2d_code_assist::lint`).

use crate::doc::CodeDoc;
use crate::docs::CodeDocs;
use crate::semantic::{IDLE_ERRORS_NS, IDLE_NS};
use ph2d_code_assist::lint::{FieldLint, ObjectLint, single};
pub(crate) use ph2d_code_assist::lint::{Lint, did_you_mean};
use ph2d_code_explain::Explained;
use ph2d_code_syntax::ObjectCall;
use ph2d_script::api_table::HOOKS;
use ph2d_script::props::RESERVED_NAMES;
use std::sync::Arc;

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
        self.lint.objects.extend(self.top_object_lints());
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

    /// Os `ph2d.object` que correm no topo fora de uma `ph2d.property` (doc CodeEditor 10) — não
    /// precisa da cena: no topo nenhum objecto existe ainda.
    fn top_object_lints(&self) -> Vec<ObjectLint> {
        self.syntax
            .stray_top_objects(self.buffer.text())
            .into_iter()
            .map(|bytes| ObjectLint {
                bytes,
                explained: Explained::object_at_top(),
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
