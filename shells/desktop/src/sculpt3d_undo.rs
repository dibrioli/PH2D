//! **DESFAZER e REFAZER** — tirar uma entrada de uma fila, aplicá-la, e pôr a
//! inversa na outra.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é *o
//! que a cena GRAVA* (lá: a fila, os verbos que produzem entradas) contra *como
//! uma entrada é DESFEITA* (aqui). O segundo é o que carrega as três leis de
//! endereçamento — o objeto, o nível e a janela —, e elas crescem juntas.

use super::{Entry, Sculpt3dScene, SculptStroke, StrokeUndo, swap_window};

impl Sculpt3dScene {
    /// Desfaz o último gesto. Devolve `false` se não havia nada.
    pub(crate) fn undo_stroke(&mut self) -> bool {
        self.step(true)
    }

    /// Refaz o último gesto desfeito. Devolve `false` se não havia nada.
    pub(crate) fn redo_stroke(&mut self) -> bool {
        self.step(false)
    }

    /// Tira uma entrada de uma fila, aplica, e põe a INVERSA na outra.
    ///
    /// ⚠️ As duas direções são esta função com as filas trocadas — não há um
    /// caminho de refazer para divergir do de desfazer.
    fn step(&mut self, undoing: bool) -> bool {
        let entry = if undoing {
            self.undo.pop()
        } else {
            self.redo.pop()
        };
        let Some(entry) = entry else {
            return false;
        };
        // ⚠️ **IR AO OBJETO CERTO VEM ANTES DE TUDO** — antes até do nível, que
        // é a mesma lei um degrau abaixo. Uma janela de traço é uma lista de
        // índices, e índices pertencem a uma MALHA: aplicá-los na peça que a mão
        // está trabalhando agora escreveria posições certas nos vértices errados
        // de um objeto que ninguém tocou, sem levantar erro nenhum. Desfazer
        // VOLTA ao objeto da edição, que é também o que o artista espera de um
        // Ctrl+Z.
        //
        // O clamp cobre um objeto que deixou de existir: a entrada é aplicada em
        // quem sobrou em vez de indexar fora da lista. É honestamente pior que
        // recusar, e é por isso que **apagar um objeto tem de limpar a fila** — a
        // wave que trouxer o delete traz essa metade.
        self.active = entry.object.min(self.objects.len() - 1);
        let object = self.active;
        let inverse = Entry {
            object,
            undo: self.apply_entry(entry.undo),
        };
        if undoing {
            self.redo.push(inverse);
        } else {
            self.undo.push(inverse);
        }
        true
    }

    /// **Aplica uma entrada e devolve a inversa dela.**
    fn apply_entry(&mut self, entry: StrokeUndo) -> StrokeUndo {
        // ⚠️ **Ir ao nível certo vem PRIMEIRO.** Uma janela de traço é uma lista
        // de índices, e índices pertencem a uma topologia — aplicá-los noutro
        // nível escreve posições certas nos vértices errados sem levantar erro
        // nenhum.
        match entry {
            StrokeUndo::Stroke { level, .. } | StrokeUndo::Mask { level, .. } => {
                if self.obj().stack.level() != level {
                    self.obj_mut().stack.select(level);
                    self.mesh_rebuilt();
                }
            }
            // ⚠️ **A MESMA lei, e ela não é simetria de estilo:** tirar ou pôr um
            // nível só é possível DE PÉ no topo, então desfazer uma subdivisão
            // depois de descer (`,`) era um **no-op silencioso** — o artista
            // apertava Ctrl+Z, a pilha ficava como estava, e a entrada era
            // consumida. Achado por mutação.
            StrokeUndo::AddedLevel | StrokeUndo::DroppedLevel(_) => {
                let top = self.obj().stack.level_count().saturating_sub(1);
                if self.obj().stack.level() != top {
                    self.obj_mut().stack.select(top);
                }
            }
            // ⚠️ Uma troca de nível é aplicada DE ONDE ELA POUSOU — descer se
            // desfaz do nível de baixo, subir do de cima. E com todas as trocas
            // registradas este `select` nunca tem o que fazer: a ordem LIFO já
            // devolveu a pilha ao lugar. Ele fica como rede, e o `lower` dele é o
            // único que não é gravado — seguro porque um passeio sem escultura
            // carimba exatamente nada.
            StrokeUndo::Descended { from, .. } => {
                let landed = from.saturating_sub(1);
                if self.obj().stack.level() != landed {
                    self.obj_mut().stack.select(landed);
                }
            }
            StrokeUndo::Ascended { from } => {
                if self.obj().stack.level() != from + 1 {
                    self.obj_mut().stack.select(from + 1);
                }
            }
            // ⚠️ A MESMA lei outra vez, e aqui os dois lados pousam em níveis
            // DIFERENTES: reverter deixa o artista no nível 1 (a malha que ele
            // tinha, com uma base nova embaixo) e desfazê-la o deixa no 0.
            StrokeUndo::ReversedLevel(_) => {
                if self.obj().stack.level() != 1 {
                    self.obj_mut().stack.select(1);
                }
            }
            // ⚠️ Tapar buraco só existe em pilha de UM nível, então este `select`
            // nunca tem o que fazer: a ordem LIFO já devolveu a pilha ao lugar
            // antes de a entrada ser alcançada. Ele fica como rede, do mesmo
            // jeito que o do `Descended`.
            StrokeUndo::UnreversedLevel
            | StrokeUndo::FilledHoles { .. }
            | StrokeUndo::UnfilledHoles
            | StrokeUndo::Remeshed(_) => {
                if self.obj().stack.level() != 0 {
                    self.obj_mut().stack.select(0);
                }
            }
        }
        match entry {
            // Uma operação de máscara mexeu na malha inteira: o estado anterior
            // é o plano INTEIRO, e `None` quer dizer *não havia máscara* — o
            // que se desfaz REMOVENDO o plano, não zerando-o.
            StrokeUndo::Mask { level, before } => {
                let now = self.mesh().masks().map(<[f32]>::to_vec);
                match before {
                    Some(m) => self.mesh_mut().put_masks(m),
                    None => {
                        self.mesh_mut().take_masks();
                    }
                }
                self.obj_mut().uploaded = false;
                self.edits += 1;
                StrokeUndo::Mask { level, before: now }
            }
            // Tirar o topo — o nível de baixo nunca foi tocado. O que sai vira a
            // inversa, inteiro.
            StrokeUndo::AddedLevel => {
                let gone = self.obj_mut().stack.drop_top();
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                match gone {
                    Some(level) => StrokeUndo::DroppedLevel(Box::new(level)),
                    // Só alcançável se a pilha tiver um nível só, e aí não havia
                    // nada a tirar: a inversa honesta é *nada foi tirado*.
                    None => StrokeUndo::AddedLevel,
                }
            }
            StrokeUndo::DroppedLevel(level) => {
                self.obj_mut().stack.push_level(*level);
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                StrokeUndo::AddedLevel
            }
            // As duas trocas de nível, e elas são a inversa uma da outra: descer
            // se desfaz devolvendo o carimbo E subindo; subir se desfaz descendo,
            // o que produz o carimbo que a inversa vai precisar.
            StrokeUndo::Descended { from, stamped } => {
                self.obj_mut().stack.undo_descent(&stamped);
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                StrokeUndo::Ascended {
                    from: from.saturating_sub(1),
                }
            }
            StrokeUndo::Ascended { from } => {
                let stamped = self.obj_mut().stack.lower();
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                match stamped {
                    Some(s) => StrokeUndo::Descended {
                        from: from + 1,
                        stamped: Box::new(s),
                    },
                    // Só alcançável se a pilha já estivesse no 0 — e aí não houve
                    // descida: a inversa honesta é *nada foi feito*.
                    None => StrokeUndo::Ascended { from },
                }
            }
            // Tapar buraco e o desfazer dele. ⚠️ O `truncate` VALIDA, e a recusa
            // devolve a mesma entrada em vez de consumi-la — a malha ficou como
            // estava, então a única coisa capaz de desfazer continua na fila.
            StrokeUndo::FilledHoles { verts, faces } => {
                let ok = self.mesh_mut().truncate(verts, faces).is_ok();
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                if ok {
                    StrokeUndo::UnfilledHoles
                } else {
                    StrokeUndo::FilledHoles { verts, faces }
                }
            }
            // A troca. ⚠️ Ela **não valida** como o `truncate` do irmão acima
            // porque não há o que validar: a malha carregada é uma malha
            // inteira e coerente, e instalá-la não pode deixar índice apontando
            // para fora.
            StrokeUndo::Remeshed(previous) => {
                let now = core::mem::replace(self.obj_mut().stack.mesh_mut(), *previous);
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                StrokeUndo::Remeshed(Box::new(now))
            }
            StrokeUndo::UnfilledHoles => {
                let report = ph2d_mesh::fill_holes(self.mesh_mut());
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                if report.is_noop() {
                    StrokeUndo::UnfilledHoles
                } else {
                    StrokeUndo::FilledHoles {
                        verts: report.verts_before(),
                        faces: report.faces_before(),
                    }
                }
            }
            // A reconstrução e o desfazer dela. ⚠️ O `false` do `unreverse` não
            // é ignorado: devolver a MESMA entrada mantém a reversão viva na
            // fila oposta, em vez de consumir a única coisa capaz de desfazê-la.
            StrokeUndo::ReversedLevel(rev) => {
                let undone = self.obj_mut().stack.unreverse(&rev);
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                if undone {
                    StrokeUndo::UnreversedLevel
                } else {
                    StrokeUndo::ReversedLevel(rev)
                }
            }
            StrokeUndo::UnreversedLevel => {
                let again = self.obj_mut().stack.reverse();
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                match again {
                    Some(r) => StrokeUndo::ReversedLevel(Box::new(r)),
                    None => StrokeUndo::UnreversedLevel,
                }
            }
            StrokeUndo::Stroke {
                level,
                verts,
                positions,
                masks,
            } => {
                if let Some(masks) = masks {
                    let now = swap_window(self.mesh_mut().masks_mut(), &verts, &masks);
                    StrokeUndo::Stroke {
                        level,
                        verts,
                        positions,
                        masks: Some(now),
                    }
                } else {
                    let now = swap_window(self.mesh_mut().positions_mut(), &verts, &positions);
                    // ⚠️ O `rebuild` inteiro, e não um `refresh_region`:
                    // desfazer devolve posições que o refit incremental já
                    // tinha "seguido" para outro lugar, e um refit sobre a
                    // volta deixaria caixas frouxas grandes demais acumulando a
                    // cada Ctrl+Z. Um undo é user-paced — é o lugar certo para
                    // pagar a resposta exata.
                    self.mesh_mut().rebuild();
                    self.mesh_rebuilt();
                    StrokeUndo::Stroke {
                        level,
                        verts,
                        positions: now,
                        masks: None,
                    }
                }
            }
        }
    }
}
