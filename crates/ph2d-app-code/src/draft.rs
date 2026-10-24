//! ⭐ **1b — a propriedade aparece no Inspector SEM salvar** (ordem do dono 23/10, o `@export` do Godot).
//!
//! O oráculo ([`ferramentas/oraculo_export_sem_salvar`](../../../docs/CodeEditor/ferramentas/oraculo_export_sem_salvar/saida.txt),
//! duas corridas iguais) diz a lei: o Inspector segue o texto da aba **na pausa da escrita** (1,5 s; 0,5 s
//! com o texto partido), nunca na tecla; com o texto partido fica a **última lista boa**; o disco não muda.
//! Aqui cada aba colhe as declarações do seu texto pela VM da casa ([`ScriptHost::draft_decls`]) e o
//! quadro publica-as em [`ScriptDrafts`], que o cartão Script lê antes da cena.

use crate::docs::CodeDocs;
use crate::semantic::{IDLE_ERRORS_NS, IDLE_NS};
use ph2d_script::{ScriptDrafts, ScriptHost};

/// O que uma aba colheu do seu texto.
#[derive(Default)]
pub(crate) struct Draft {
    /// A revisão vista e quando (ns) — a pausa conta a partir daí.
    seen: Option<(u64, u128)>,
    /// A revisão colhida por último.
    harvested: Option<u64>,
    /// A última colheita falhou (o texto está partido).
    broken: bool,
}

impl CodeDocs {
    /// Colhe o texto que parou e publica a lista de cada script aberto ou estacionado.
    pub(crate) fn drafts(&mut self, world: &mut ph2d_ecs::World, host: Option<&ScriptHost>) {
        let now = self.clock.0;
        let mut changed = false;
        if let Some(h) = host {
            for d in &mut self.docs {
                let rev = d.buffer.revision();
                let since = match d.draft.seen {
                    Some((r, t)) if r == rev => t,
                    _ => {
                        d.draft.seen = Some((rev, now));
                        now
                    }
                };
                let idle = if d.draft.broken {
                    IDLE_ERRORS_NS
                } else {
                    IDLE_NS
                };
                // A 1.ª vez (aba aberta agora) e o texto SALVO (o Ctrl+S, o renomear que grava)
                // colhem já: senão a lista velha da aba taparia a da cena recarregada. Uma revisão
                // por salvar espera a pausa.
                let due = d.draft.harvested.is_none()
                    || !d.buffer.is_dirty()
                    || now >= since.saturating_add(idle);
                if d.draft.harvested == Some(rev) || !due {
                    continue;
                }
                d.draft.harvested = Some(rev);
                // Partido: a lista que havia fica (a última boa); nenhuma = a cena responde.
                match h.draft_decls(&d.title, &d.buffer.text().to_string()) {
                    Ok(list) => {
                        d.draft.broken = false;
                        if self.drafts.get(&d.script) != Some(&list) {
                            self.drafts.insert(d.script, list);
                            changed = true;
                        }
                    }
                    Err(_) => d.draft.broken = true,
                }
            }
        }
        let (docs, parked) = (&self.docs, &self.parked);
        let before = self.drafts.len();
        self.drafts
            .retain(|id, _| docs.iter().any(|d| d.script == *id) || parked.contains_key(id));
        changed |= self.drafts.len() != before;
        if changed || !world.contains_resource::<ScriptDrafts>() {
            world.insert_resource(ScriptDrafts(self.drafts.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_script::ScriptId;

    const S: u128 = 1_000_000_000;

    fn nomes(w: &ph2d_ecs::World, id: ScriptId) -> Option<Vec<String>> {
        w.get_resource::<ScriptDrafts>()?
            .0
            .get(&id)
            .map(|l| l.iter().map(|d| d.name.clone()).collect())
    }

    fn escreve(docs: &mut CodeDocs, texto: &str) {
        docs.docs[0].buffer.replace_all(texto);
    }

    #[test]
    fn a_lista_segue_o_texto_por_salvar_na_pausa_e_guarda_a_ultima_boa() {
        let host = ScriptHost::new().unwrap();
        let mut w = ph2d_ecs::World::new();
        let mut docs = CodeDocs::default();
        let id = ScriptId(7);
        docs.open(id, "bob.luau", "ph2d.property(\"vida\", 10)\n");
        let em = |docs: &mut CodeDocs, w: &mut ph2d_ecs::World, t: u128| {
            docs.set_clock(t, true);
            docs.drafts(w, Some(&host));
        };
        em(&mut docs, &mut w, S);
        assert_eq!(
            nomes(&w, id),
            Some(vec!["vida".into()]),
            "a aba aberta colhe já"
        );

        escreve(
            &mut docs,
            "ph2d.property(\"vida\", 10)\nph2d.property(\"velocidade\", 3)\n",
        );
        em(&mut docs, &mut w, 2 * S);
        em(&mut docs, &mut w, 2 * S + IDLE_NS - 1);
        assert_eq!(nomes(&w, id), Some(vec!["vida".into()]), "nunca na tecla");
        em(&mut docs, &mut w, 2 * S + IDLE_NS);
        assert_eq!(
            nomes(&w, id),
            Some(vec!["vida".into(), "velocidade".into()]),
            "na pausa de 1,5 s, sem salvar"
        );

        escreve(
            &mut docs,
            "ph2d.property(\"vida\", 10)\nph2d.property(\"escudo\", 1)\nlocal = =\n",
        );
        em(&mut docs, &mut w, 5 * S);
        em(&mut docs, &mut w, 5 * S + IDLE_NS);
        assert_eq!(
            nomes(&w, id),
            Some(vec!["vida".into(), "velocidade".into()]),
            "partido: fica a última lista boa"
        );

        escreve(
            &mut docs,
            "ph2d.property(\"vida\", 10)\nph2d.property(\"escudo\", 1)\n",
        );
        em(&mut docs, &mut w, 8 * S);
        em(&mut docs, &mut w, 8 * S + IDLE_ERRORS_NS);
        assert_eq!(
            nomes(&w, id),
            Some(vec!["vida".into(), "escudo".into()]),
            "curado: 0,5 s, a pausa de quem tinha erro à vista"
        );

        escreve(&mut docs, "ph2d.property(\"altura\", 1)\n");
        docs.docs[0].buffer.mark_saved();
        em(&mut docs, &mut w, 9 * S);
        assert_eq!(
            nomes(&w, id),
            Some(vec!["altura".into()]),
            "salvo: colhe no mesmo quadro (a cena recarrega já)"
        );

        docs.docs.clear();
        em(&mut docs, &mut w, 10 * S);
        assert_eq!(nomes(&w, id), None, "aba fechada: a cena responde");
    }

    #[test]
    fn um_texto_que_nunca_carregou_nao_publica_nada() {
        let host = ScriptHost::new().unwrap();
        let mut w = ph2d_ecs::World::new();
        let mut docs = CodeDocs::default();
        docs.open(ScriptId(3), "x.luau", "local = =\n");
        docs.set_clock(S, true);
        docs.drafts(&mut w, Some(&host));
        assert_eq!(nomes(&w, ScriptId(3)), None);
    }
}
