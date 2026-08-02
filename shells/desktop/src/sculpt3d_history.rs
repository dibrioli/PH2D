//! **O QUE A CENA LEMBRA** — a pilha de níveis e as duas filas de desfazer.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados da
//! [`Sculpt3dScene`]. O corte é de responsabilidade: *o que a cena É e o que a
//! mão faz com ela* fica no pai; *o que ela guarda para poder voltar* fica aqui
//! — e as duas metades desta pergunta (a pilha e as filas) são a mesma coisa,
//! porque desde a multiresolução **acrescentar um nível é uma entrada de undo**.
//!
//! # Desfazer e refazer são a MESMA operação
//!
//! Toda entrada é uma **troca**: aplicá-la instala o estado que ela carrega e
//! devolve, na mesma passada, o estado que estava ali — que é exatamente a
//! entrada inversa. Então não há um motor de desfazer e outro de refazer para
//! divergirem; há [`Sculpt3dScene::apply_entry`], e as duas direções só diferem
//! em **de que fila se tira e para qual se empurra**.
//!
//! ⚠️ E é por isso que o `positions`/`masks` de um traço saem por
//! `mem::replace`: ler-e-escrever num passo não é economia, é a garantia de que
//! o que volta para a outra fila é *o que de fato estava lá* — uma segunda
//! leitura, feita depois da escrita, devolveria o valor recém-instalado.

use ph2d_mesh::DetachedLevel;

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
    /// **Um NÍVEL foi acrescentado** — aplicá-la é tirá-lo.
    ///
    /// ⚠️ **E ela deixou de guardar a malha inteira, que é o que a pilha
    /// compra.** Antes de existirem níveis, desfazer uma subdivisão exigia uma
    /// cópia do documento: a contagem de vértices mudava e não havia onde o
    /// estado anterior estivesse. Com a pilha, o nível de baixo **nunca foi
    /// tocado** — desfazer é descartar o topo e devolver a seleção.
    AddedLevel,
    /// **Um nível foi tirado** — aplicá-la é recolocá-lo. É a inversa da de
    /// cima, e ela **carrega** o nível.
    ///
    /// ⚠️ **Refazer NÃO é subdividir de novo**, e a diferença é medível: uma
    /// subdivisão recomputada só reproduz o nível enquanto o de baixo estiver
    /// byte-a-byte como estava, e **descer escreve no de baixo**
    /// (`copy_shared_down`). Depois de uma única viagem para baixo, um redo por
    /// recomputação devolveria uma malha PARECIDA — que é a pior forma de
    /// errado, porque ninguém vê. Carregar o nível é exato, e **não é cópia**:
    /// ele é movido para fora da pilha e movido de volta.
    ///
    /// ⚠️ `Box` porque um nível é o maior objeto do módulo, e sem ele todo
    /// elemento das duas filas mediria isso.
    DroppedLevel(Box<DetachedLevel>),
}

/// **Instala `values` nos índices `verts` e devolve o que estava lá** — uma
/// TROCA, numa passada só.
///
/// ⚠️ É a peça que faz desfazer e refazer serem a mesma operação, e ela é uma
/// função solta para poder ser TESTADA: o resto do caminho vive numa
/// [`Sculpt3dScene`], que precisa de um device, e o defeito que ela previne não
/// tem sintoma imediato. Uma segunda leitura feita *depois* da escrita
/// devolveria o valor recém-instalado, a fila oposta guardaria um estado que
/// nunca existiu, e o artista só descobriria no **segundo** Ctrl+Shift+Z.
fn swap_window<T: Copy>(out: &mut [T], verts: &[u32], values: &[T]) -> Vec<T> {
    verts
        .iter()
        .zip(values)
        .map(|(&v, x)| std::mem::replace(&mut out[v as usize], *x))
        .collect()
}

impl Sculpt3dScene {
    /// **A porta única por onde uma edição entra na história.**
    ///
    /// ⚠️ **Ela limpa a fila de refazer, e é por isso que é uma porta.** Uma
    /// edição nova torna o futuro que estava guardado impossível de alcançar (o
    /// estado de que ele partia deixou de existir), e enumerar os sítios que
    /// gravam — hoje três: o traço, a máscara, a subdivisão — é a lista que
    /// nasce incompleta no dia em que aparece o quarto. Aqui o quarto nasce
    /// certo.
    pub(super) fn record(&mut self, entry: StrokeUndo) {
        self.undo.push(entry);
        self.redo.clear();
    }

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
        self.record(StrokeUndo::AddedLevel);
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        true
    }

    /// **Troca de nível** — `up` sobe, senão desce. Devolve `false` na ponta.
    ///
    /// ⚠️ **Não é uma edição, então não entra na fila de undo**: o gesto é o
    /// próprio inverso, e uma pilha que registrasse navegação faria o Ctrl+Z
    /// gastar toques desfazendo o que o artista fez para OLHAR. Pela mesma razão
    /// ela não passa pelo [`Sculpt3dScene::record`]: olhar não pode apagar um
    /// refazer guardado.
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
        let entry = StrokeUndo::Stroke {
            level: self.stack.level(),
            verts: self.stroke.touched().to_vec(),
            positions: self.stroke.base_positions().to_vec(),
            masks: self
                .brush
                .verb
                .paints_mask()
                .then(|| self.stroke.base_masks().to_vec()),
        };
        self.record(entry);
    }

    /// Desfaz o último gesto. Devolve `false` se não havia nada.
    pub(super) fn undo_stroke(&mut self) -> bool {
        self.step(true)
    }

    /// Refaz o último gesto desfeito. Devolve `false` se não havia nada.
    pub(super) fn redo_stroke(&mut self) -> bool {
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
        let inverse = self.apply_entry(entry);
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
                if self.stack.level() != level {
                    self.stack.select(level);
                    self.mesh_rebuilt();
                }
            }
            // ⚠️ **A MESMA lei, e ela não é simetria de estilo:** tirar ou pôr um
            // nível só é possível DE PÉ no topo, então desfazer uma subdivisão
            // depois de descer (`,`) era um **no-op silencioso** — o artista
            // apertava Ctrl+Z, a pilha ficava como estava, e a entrada era
            // consumida. Achado por mutação.
            StrokeUndo::AddedLevel | StrokeUndo::DroppedLevel(_) => {
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
            StrokeUndo::Mask { level, before } => {
                let now = self.mesh().masks().map(<[f32]>::to_vec);
                match before {
                    Some(m) => self.mesh_mut().put_masks(m),
                    None => {
                        self.mesh_mut().take_masks();
                    }
                }
                self.uploaded = false;
                self.edits += 1;
                StrokeUndo::Mask { level, before: now }
            }
            // Tirar o topo — o nível de baixo nunca foi tocado. O que sai vira a
            // inversa, inteiro.
            StrokeUndo::AddedLevel => {
                let gone = self.stack.drop_top();
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
                self.stack.push_level(*level);
                self.stroke = SculptStroke::default();
                self.mesh_rebuilt();
                StrokeUndo::AddedLevel
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

#[cfg(test)]
mod tests {
    use super::swap_window;

    /// ⚠️ **A troca devolve o que ESTAVA lá, não o que ela instalou.**
    ///
    /// É o dente do modelo inteiro: se ela devolvesse o valor novo, o desfazer
    /// funcionaria (o estado certo é instalado) e o refazer seria um no-op que
    /// **consome** a entrada — a forma de "o redo às vezes não faz nada" que
    /// nenhum gate de contagem vê.
    #[test]
    fn the_window_swap_returns_what_was_there_not_what_it_installed() {
        let mut plane = [10.0f32, 11.0, 12.0, 13.0];
        let verts = [3u32, 1];

        let was = swap_window(&mut plane, &verts, &[99.0, 98.0]);
        assert_eq!(
            plane,
            [10.0, 98.0, 12.0, 99.0],
            "instalou nos índices certos"
        );
        assert_eq!(
            was,
            vec![13.0, 11.0],
            "e colheu o que estava lá, na ordem dos índices"
        );

        // E ela é a própria inversa: aplicar o que voltou restaura o começo — que
        // é literalmente o que a fila oposta faz.
        let back = swap_window(&mut plane, &verts, &was);
        assert_eq!(plane, [10.0, 11.0, 12.0, 13.0], "a volta restaura");
        assert_eq!(back, vec![99.0, 98.0], "e devolve o que o refazer precisa");
    }
}
