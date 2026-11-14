//! **O passo de undo/redo** — filho de [`super`] (`#[path]`, enxerga os privados do controller): a base, a
//! materialização, as pilhas e o cursor; e a tela e o relevo que o tool ENTREGA para serem escritos no lugar
//! (B17.5).

use std::sync::Arc;

use super::{ModelSnapshot, UndoController};
use crate::undo_planes::{Base, Relevo};

/// As bases que o tool entregou ao passo; a recusa devolve-as intactas.
#[derive(Default)]
pub(super) struct Dono {
    tela: Option<Arc<Vec<u8>>>,
    relevo: Option<Relevo>,
}

/// O relevo do tool é, chave a chave, o MESMO `Arc` do `live`? (Só então o do `live` o representa.)
fn mesmo_relevo(a: &Relevo, live: &ModelSnapshot) -> bool {
    fn igual<T>(
        a: &std::collections::BTreeMap<crate::layers::LayerId, Arc<Vec<T>>>,
        b: &std::collections::BTreeMap<crate::layers::LayerId, Arc<Vec<T>>>,
    ) -> bool {
        a.len() == b.len()
            && a.iter()
                .all(|(k, v)| b.get(k).is_some_and(|w| Arc::ptr_eq(v, w)))
    }
    igual(&a.heights, &live.heights) && igual(&a.covers, &live.covers) && igual(&a.mats, &live.mats)
}

impl UndoController {
    /// [`Self::undo`] com a TELA e o RELEVO do tool entregues (B17.5): quem chama larga os `Arc` deles (os de `live`
    /// também) e a história larga os do cursor, então cada plano tem um dono só e os tiles do passo vão NO LUGAR — o
    /// `par_clone` do plano inteiro era o custo do Ctrl+Z (3,33 ms Digital, 10,48 ms Impasto a 4K). No sucesso `*tela`
    /// e `*relevo` ficam vazios (o restauro instala os do modelo devolvido); na recusa voltam intactos.
    pub(crate) fn undo_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
        relevo: &mut Relevo,
    ) -> Option<Box<ModelSnapshot>> {
        self.anda_na_tela(live, tela, relevo, true)
    }

    /// [`Self::undo_na_tela`] para o redo.
    pub(crate) fn redo_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
        relevo: &mut Relevo,
    ) -> Option<Box<ModelSnapshot>> {
        self.anda_na_tela(live, tela, relevo, false)
    }

    fn anda_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
        relevo: &mut Relevo,
        want_before: bool,
    ) -> Option<Box<ModelSnapshot>> {
        // A tela que a sessão molhada vigia pelo ponteiro tem de mudar de ponteiro: fica a cópia.
        let tela_vai = !self.write_state.vigia(tela) && Arc::ptr_eq(&live.canvas_rgba, tela);
        let relevo_vai = mesmo_relevo(relevo, live);
        let dono = Dono {
            tela: tela_vai.then(|| {
                live.canvas_rgba = Arc::default();
                std::mem::take(tela)
            }),
            relevo: relevo_vai.then(|| {
                *relevo = Relevo::default();
                Relevo::tira(live)
            }),
        };
        match self.anda(live, dono, want_before) {
            Ok(m) => Some(m),
            Err(d) => {
                if let Some(t) = d.tela {
                    *tela = t;
                    live.canvas_rgba = Arc::clone(tela);
                }
                if let Some(r) = d.relevo {
                    live.heights.clone_from(&r.heights);
                    live.covers.clone_from(&r.covers);
                    live.mats.clone_from(&r.mats);
                    *relevo = r;
                }
                None
            }
        }
    }

    /// O passo comum: `dono` = as bases do tool entregues ([`Self::undo_na_tela`]); `Err` devolve-as, intactas.
    pub(super) fn anda(
        &mut self,
        live: &ModelSnapshot,
        dono: Dono,
        want_before: bool,
    ) -> Result<Box<ModelSnapshot>, Dono> {
        let pilha = if want_before { &self.undo } else { &self.redo };
        // A tela elidida do cursor volta pelo journal sobre o vivo (B17.4).
        let (Some(entry), Some(cursor)) = (pilha.last(), self.cursor.as_deref()) else {
            return Err(dono);
        };
        let held = self.with_canvas_held(cursor, dono.tela.as_ref().unwrap_or(&live.canvas_rgba));
        let r = match (held, dono) {
            (None, d) => Err(d),
            (
                Some(held),
                Dono {
                    tela: None,
                    relevo: None,
                },
            ) => entry
                .materialize(&held, live, want_before)
                .ok_or_else(Dono::default),
            (Some(mut held), Dono { tela: d, relevo }) => {
                // Os donos que a história controla largam as bases (o `held` e o cursor); o do tool, se é o mesmo.
                let tela_vai = d.is_some();
                let (tela, d) = match d {
                    Some(d) => {
                        let tela = std::mem::take(&mut held.canvas_rgba);
                        if Arc::ptr_eq(&cursor.canvas_rgba, &tela)
                            && let Some(c) = self.cursor.as_deref_mut()
                        {
                            c.canvas_rgba = Arc::default();
                        }
                        let d = (!Arc::ptr_eq(&d, &tela)).then_some(d);
                        (tela, d)
                    }
                    None => (Arc::clone(&held.canvas_rgba), None),
                };
                let relevo_vai = relevo.is_some();
                let relevo = match relevo {
                    Some(r) => {
                        #[cfg(debug_assertions)]
                        crate::undo_planes::PlaneDeltas::debug_relief_base_is_the_cursor(&held, &r);
                        larga_o_relevo(&mut held);
                        if let Some(c) = self.cursor.as_deref_mut() {
                            larga_o_relevo(c);
                        }
                        r
                    }
                    None => Relevo::de(live),
                };
                let pilha = if want_before { &self.undo } else { &self.redo };
                pilha
                    .last()
                    .expect("conferida acima")
                    .materialize_tomando(&held, live, want_before, Base { tela, relevo })
                    .map_err(|b| Dono {
                        tela: tela_vai.then(|| d.unwrap_or(b.tela)),
                        relevo: relevo_vai.then_some(b.relevo),
                    })
            }
        };
        let restore = match r {
            Ok(m) => m,
            Err(d) => {
                // O cursor não descreve mais o plano que a entrada deltou (o canvas mudou de forma sob o
                // histórico). Um undo que devolve pixels quase-certos é pior que um que se recusa: descarta.
                eprintln!("[painter-undo] o cursor nao casa com o delta: historico descartado");
                debug_assert!(
                    false,
                    "cursor incoerente com o delta — ver undo_delta::StoredPlane::side"
                );
                self.clear();
                return Err(d);
            }
        };
        let entry = if want_before {
            self.undo.pop()
        } else {
            self.redo.pop()
        }
        .expect("o topo que acabamos de ler");
        self.marca_anda(&entry, !want_before);
        if want_before {
            self.redo.push(entry);
        } else {
            self.undo.push(entry);
        }
        // O cursor ANDA com a história: o estado que acabamos de instalar é o adjacente do novo topo.
        self.set_cursor((*restore).clone());
        Ok(restore)
    }
}

/// O snapshot larga o relevo de impasto que segura, e as testemunhas `Weak` dele (que impediriam o `get_mut`). O
/// cursor é substituído no sucesso e descartado na recusa, então nada lê o que ele largou.
fn larga_o_relevo(m: &mut ModelSnapshot) {
    m.heights.clear();
    m.covers.clear();
    m.mats.clear();
    m.relief_elided.heights.clear();
    m.relief_elided.covers.clear();
    m.relief_elided.mats.clear();
}
