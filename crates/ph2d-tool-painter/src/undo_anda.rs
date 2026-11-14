//! **O passo de undo/redo** — filho de [`super`] (`#[path]`, enxerga os privados do controller): a base, a
//! materialização, as pilhas e o cursor; e a tela que o tool ENTREGA para ser escrita no lugar (B17.5).

use std::sync::Arc;

use super::{ModelSnapshot, UndoController};

impl UndoController {
    /// [`Self::undo`] com a TELA do tool entregue (B17.5): quem chama larga o `Arc` dela (o de `live` também) e a
    /// história larga o do cursor, então a tela tem um dono só e os tiles do passo vão NO LUGAR — o `par_clone` do
    /// plano inteiro era o custo do Ctrl+Z (3,33 ms Digital a 4K). `*tela` fica vazio no sucesso (o restauro instala
    /// a do modelo devolvido) e volta intacto na recusa.
    pub fn undo_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
    ) -> Option<Box<ModelSnapshot>> {
        self.anda_na_tela(live, tela, true)
    }

    /// [`Self::undo_na_tela`] para o redo.
    pub fn redo_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
    ) -> Option<Box<ModelSnapshot>> {
        self.anda_na_tela(live, tela, false)
    }

    fn anda_na_tela(
        &mut self,
        live: &mut ModelSnapshot,
        tela: &mut Arc<Vec<u8>>,
        want_before: bool,
    ) -> Option<Box<ModelSnapshot>> {
        // A tela que a sessão molhada vigia pelo ponteiro tem de mudar de ponteiro: fica a cópia.
        if self.write_state.vigia(tela) || !Arc::ptr_eq(&live.canvas_rgba, tela) {
            return self.anda(live, None, want_before).ok();
        }
        live.canvas_rgba = Arc::default();
        match self.anda(live, Some(std::mem::take(tela)), want_before) {
            Ok(m) => Some(m),
            Err(t) => {
                if let Some(t) = t {
                    *tela = t;
                }
                live.canvas_rgba = Arc::clone(tela);
                None
            }
        }
    }

    /// O passo comum: `dono` = a tela do tool entregue ([`Self::undo_na_tela`]); `Err` devolve-a, intacta, na recusa.
    pub(super) fn anda(
        &mut self,
        live: &ModelSnapshot,
        dono: Option<Arc<Vec<u8>>>,
        want_before: bool,
    ) -> Result<Box<ModelSnapshot>, Option<Arc<Vec<u8>>>> {
        let pilha = if want_before { &self.undo } else { &self.redo };
        // A tela elidida do cursor volta pelo journal sobre o vivo (B17.4).
        let (Some(entry), Some(cursor)) = (pilha.last(), self.cursor.as_deref()) else {
            return Err(dono);
        };
        let held = self.with_canvas_held(cursor, dono.as_ref().unwrap_or(&live.canvas_rgba));
        let r = match (held, dono) {
            (None, d) => Err(d),
            (Some(held), None) => entry.materialize(&held, live, want_before).ok_or(None),
            (Some(mut held), Some(d)) => {
                // Os donos que a história controla largam a tela (o `held` e o cursor); o do tool, se é a mesma.
                let tela = std::mem::take(&mut held.canvas_rgba);
                let cursor_segura = Arc::ptr_eq(&cursor.canvas_rgba, &tela);
                let d = (!Arc::ptr_eq(&d, &tela)).then_some(d);
                if cursor_segura && let Some(c) = self.cursor.as_deref_mut() {
                    c.canvas_rgba = Arc::default();
                }
                let pilha = if want_before { &self.undo } else { &self.redo };
                pilha
                    .last()
                    .expect("conferida acima")
                    .materialize_tomando(&held, live, want_before, tela)
                    .map_err(|t| Some(d.unwrap_or(t)))
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
