//! **O QUE A CENA LEMBRA** — a pilha de níveis e a fila de desfazer.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados da
//! [`Sculpt3dScene`]. O corte é de responsabilidade: *o que a cena É e o que a
//! mão faz com ela* fica no pai; *o que ela guarda para poder voltar* fica aqui
//! — e as duas metades desta pergunta (a pilha e a fila) são a mesma coisa,
//! porque desde a multiresolução **acrescentar um nível é uma entrada de undo**.

use super::{Sculpt3dScene, SculptStroke};

/// **O estado anterior de um gesto** — a entrada de undo.
///
/// ⚠️ **Um enum e não um struct com bandeiras.** Ele começou como um `struct`
/// com `whole_mask: bool`, porque duas formas cabiam em um discriminante; com a
/// TERCEIRA (a topologia) o bool viraria um par de bandeiras cujas quatro
/// combinações incluem duas que não significam nada. Aqui esquecer um caso não
/// compila.
/// ⚠️ **Toda entrada de edição carrega o NÍVEL em que aconteceu**, e sem isso a
/// pilha torna o undo perigoso: os índices de um traço feito no nível 0 não
/// nomeiam nada no nível 2, então desfazê-lo de pé lá em cima escreveria as
/// posições certas nos vértices errados — em silêncio. Desfazer VOLTA ao nível
/// da edição, que é também o que o artista espera de um Ctrl+Z.
pub(super) enum StrokeUndo {
    /// A janela de um traço. Não há um segundo sistema a construir: a lei do
    /// traço já congela o `pre` por vértice tocado, e `touched` +
    /// `base_positions` É a janela.
    Stroke {
        level: usize,
        verts: Vec<u32>,
        positions: Vec<[f32; 3]>,
        /// As máscaras de antes, quando o traço PINTOU máscara.
        masks: Option<Vec<f32>>,
    },
    /// Uma operação de máscara mexeu na malha inteira: o estado anterior é o
    /// plano INTEIRO. ⚠️ O `None` aqui quer dizer *não havia máscara*, o que se
    /// desfaz REMOVENDO o plano — e é por isso que ele é um caso e não a
    /// ausência de um campo: desfazer um `Invert` sobre malha virgem tem de
    /// deixá-la virgem outra vez.
    Mask {
        level: usize,
        before: Option<Vec<f32>>,
    },
    /// **Um NÍVEL foi acrescentado** — desfazer é tirá-lo.
    ///
    /// ⚠️ **E ele deixou de guardar a malha inteira, que é o que a pilha
    /// compra.** Antes de existirem níveis, desfazer uma subdivisão exigia uma
    /// cópia do documento: a contagem de vértices mudava e não havia onde o
    /// estado anterior estivesse. Com a pilha, o nível de baixo **nunca foi
    /// tocado** — desfazer é descartar o topo e devolver a seleção.
    AddedLevel,
}

impl Sculpt3dScene {
    /// **SUBDIVIDE a malha uma vez** — quatro faces onde havia uma.
    ///
    /// ⚠️ **Isto encerra o traço em voo, e é obrigatório**: o `SculptStroke`
    /// carrega índices e um `pre` congelado da topologia ANTIGA, e o `begin`
    /// seguinte só reconstrói os vetores se a contagem mudou — o que ela mudou,
    /// mas a janela de undo pendente falaria de vértices que não existem mais.
    ///
    /// ⚠️ **O custo é do REBUILD, não da aritmética** (sonda
    /// `measure_subdivide`): numa malha de 24 mil vértices o gesto inteiro custa
    /// **7,3 ms**, dos quais 6,4 são reconstruir adjacência, octree e normais da
    /// malha quatro vezes maior — o plano e os canais são 0,7. É por isso que
    /// não há teto escrito aqui: o que aperta primeiro é a MEMÓRIA (~100 B por
    /// vértice de saída), e o log diz o número depois de cada nível.
    pub(super) fn subdivide(&mut self) -> bool {
        if !self.stack.add_level() {
            return false;
        }
        self.undo.push(StrokeUndo::AddedLevel);
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        true
    }

    /// **Troca de nível** — `up` sobe, senão desce. Devolve `false` na ponta.
    ///
    /// ⚠️ **Não é uma edição, então não entra na fila de undo**: o gesto é o
    /// próprio inverso, e uma pilha que registrasse navegação faria o Ctrl+Z
    /// gastar toques desfazendo o que o artista fez para OLHAR.
    pub(super) fn change_level(&mut self, up: bool) -> bool {
        let moved = if up {
            self.stack.higher()
        } else {
            self.stack.lower()
        };
        if moved {
            // O traço em voo fala de outra malha; a GPU precisa de tudo.
            self.stroke = SculptStroke::default();
            self.mesh_rebuilt();
        }
        moved
    }

    /// Fecha o traço e guarda o desfazer.
    pub(super) fn close_stroke(&mut self) {
        if self.stroke.touched().is_empty() {
            return;
        }
        self.undo.push(StrokeUndo::Stroke {
            level: self.stack.level(),
            verts: self.stroke.touched().to_vec(),
            positions: self.stroke.base_positions().to_vec(),
            masks: self
                .brush
                .verb
                .paints_mask()
                .then(|| self.stroke.base_masks().to_vec()),
        });
    }

    /// Desfaz o último traço. Devolve `false` se não havia nada.
    pub(super) fn undo_stroke(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        // ⚠️ **Voltar ao nível da edição vem PRIMEIRO.** Uma janela de traço
        // é uma lista de índices, e índices pertencem a uma topologia — aplicá-
        // los noutro nível escreve posições certas nos vértices errados sem
        // levantar erro nenhum.
        match entry {
            StrokeUndo::Stroke { level, .. } | StrokeUndo::Mask { level, .. } => {
                if self.stack.level() != level {
                    self.stack.select(level);
                    self.mesh_rebuilt();
                }
            }
            // ⚠️ **A MESMA lei, e ela não é simetria de estilo:** descartar o
            // topo só é possível DE PÉ nele, então desfazer uma subdivisão
            // depois de descer (`,`) era um **no-op silencioso** — o artista
            // apertava Ctrl+Z, a pilha ficava como estava, e a entrada era
            // consumida. Achado por mutação.
            StrokeUndo::AddedLevel => {
                let top = self.stack.level_count().saturating_sub(1);
                if self.stack.level() != top {
                    self.stack.select(top);
                }
            }
        }
        match entry {
            // Uma operação de máscara mexeu na malha inteira: o estado anterior
            // é o plano INTEIRO, e `None` quer dizer *não havia máscara* — o
            // que se desfaz REMOVENDO o plano, não zerando-o.
            StrokeUndo::Mask { before, .. } => {
                match before {
                    Some(m) => self.mesh_mut().put_masks(m),
                    None => {
                        self.mesh_mut().take_masks();
                    }
                }
                self.uploaded = false;
                self.edits += 1;
            }
            // Descartar o topo — o nível de baixo nunca foi tocado.
            StrokeUndo::AddedLevel => {
                self.stack.drop_top();
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
            }
            StrokeUndo::Stroke {
                verts,
                positions,
                masks,
                ..
            } => {
                if let Some(masks) = masks {
                    let out = self.mesh_mut().masks_mut();
                    for (&v, m) in verts.iter().zip(&masks) {
                        out[v as usize] = *m;
                    }
                } else {
                    {
                        let out = self.mesh_mut().positions_mut();
                        for (&v, p) in verts.iter().zip(&positions) {
                            out[v as usize] = *p;
                        }
                    }
                    // ⚠️ O `rebuild` inteiro, e não um `refresh_region`:
                    // desfazer devolve posições que o refit incremental já
                    // tinha "seguido" para outro lugar, e um refit sobre a
                    // volta deixaria caixas frouxas grandes demais acumulando a
                    // cada Ctrl+Z. Um undo é user-paced — é o lugar certo para
                    // pagar a resposta exata.
                    self.mesh_mut().rebuild();
                    self.mesh_rebuilt();
                }
            }
        }
        true
    }
}
