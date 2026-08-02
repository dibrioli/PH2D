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

use ph2d_mesh::{DetachedLevel, Reversal, Stamped};

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
    /// byte-a-byte como estava, e **descer escreve no de baixo** (o carimbo do
    /// que foi esculpido em cima). Depois de uma viagem para baixo COM trabalho,
    /// um redo por
    /// recomputação devolveria uma malha PARECIDA — que é a pior forma de
    /// errado, porque ninguém vê. Carregar o nível é exato, e **não é cópia**:
    /// ele é movido para fora da pilha e movido de volta.
    ///
    /// ⚠️ `Box` porque um nível é o maior objeto do módulo, e sem ele todo
    /// elemento das duas filas mediria isso.
    DroppedLevel(Box<DetachedLevel>),
    /// **O artista DESCEU** de `from` para o nível de baixo, e a descida
    /// CARIMBOU na base o que ele tinha esculpido em cima.
    ///
    /// ⚠️ **Descer é uma EDIÇÃO, e é por isso que ela entra na história.** A
    /// versão anterior tratava a troca de nível como navegação — *olhar não é
    /// editar* —, e a frase era falsa: a descida escreve na malha de baixo. Sem
    /// registro, o carimbo ficava sem inverso, e o Ctrl+Z de uma subdivisão
    /// devolvia o artista a uma base que ele nunca autorou.
    Descended { from: usize, stamped: Box<Stamped> },
    /// **O artista SUBIU** de `from` para o nível de cima. Desfazer é descer — e
    /// é exato de graça, porque subir só escreve no topo valores que
    /// `(base, detalhe)` já determinam.
    Ascended { from: usize },
    /// **Um nível foi reconstruído EMBAIXO** — aplicá-la é tirá-lo.
    ///
    /// ⚠️ Ela carrega a RENUMERAÇÃO, não a malha: inserir uma base renumera todo
    /// nível acima, e desfazer é despermutar. São 4 B por vértice por nível
    /// (um terço de um plano de posições) contra o clone da pilha inteira.
    ReversedLevel(Box<Reversal>),
    /// **Os buracos foram TAPADOS** — aplicá-la é truncar a malha de volta.
    ///
    /// ⚠️ **Dois `usize` e nada mais, e não é um atalho: é uma propriedade do
    /// algoritmo.** Um remendo é geometria NOVA colada na beira — nenhum vértice
    /// nem face que já existia é tocado —, então desfazer é descartar o fim. Há
    /// gate afirmando o *só acrescenta*, e é ele que sangra no dia em que o
    /// preenchimento mexer num vértice antigo: nesse dia este `usize` para de
    /// ser suficiente, e o gate diz isso antes do artista.
    FilledHoles { verts: usize, faces: usize },
    /// **A malha foi RECONSTRUÍDA por voxelização** — e a entrada carrega a
    /// malha inteira de antes.
    ///
    /// ⚠️ **É a única operação do módulo sem representação mais barata, e isso é
    /// uma propriedade dela, não preguiça.** Todas as outras compartilham
    /// estrutura com o estado anterior — o traço congela só os vértices
    /// tocados, tapar buraco só ACRESCENTA, subdividir deixa o nível de baixo
    /// intacto. Um remesh não partilha nada: nem a contagem de vértices, nem a
    /// de faces, nem a correspondência entre elas. O "antes" é a malha, e
    /// guardá-la é o preço honesto.
    ///
    /// ⚠️ E ela é a **própria inversa**: aplicar é TROCAR a malha viva pela
    /// carregada e devolver a que estava. Sem casos, sem um `Unremeshed` do
    /// outro lado — a mesma forma do `Mask`.
    Remeshed(Box<ph2d_mesh::Mesh>),
    /// **O preenchimento foi desfeito** — aplicá-la é tapar de novo.
    UnfilledHoles,
    /// **A reconstrução foi desfeita** — aplicá-la é refazê-la.
    ///
    /// ⚠️ **Ela não carrega nada, e isso é o oposto do [`Self::DroppedLevel`].**
    /// Lá o redo TEM de trazer o nível, porque recomputá-lo depende de uma base
    /// que o carimbo já mudou. Aqui reverter é função pura da malha, e desfazer
    /// devolve a malha ao bit (permutar move dados, não os computa) — então
    /// refazer é chamá-la de novo e receber o mesmo resultado. Há gate.
    UnreversedLevel,
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

/// **Uma entrada da fila: o que desfazer, e EM QUEM.**
///
/// ⚠️ O `object` não é metadado — ele é o que impede o defeito que uma cena com
/// mais de uma peça inventa: esculpir A, esculpir B, e o Ctrl+Z aplicar a janela
/// de A na malha de B. Um campo, num lugar só, em vez de repetido em cada
/// variante do [`StrokeUndo`] — a lista que nasce incompleta no dia em que
/// aparece a quarta.
pub(super) struct Entry {
    pub(super) object: usize,
    pub(super) undo: StrokeUndo,
}

/// **DESFAZER e REFAZER** — ver [`undo`]. Filho (`#[path]`) pelo motivo dos
/// outros: o corte é de responsabilidade.
#[path = "sculpt3d_undo.rs"]
mod undo;

impl Sculpt3dScene {
    /// **A porta única por onde uma edição entra na história.**
    ///
    /// ⚠️ **Ela limpa a fila de refazer, e é por isso que é uma porta.** Uma
    /// edição nova torna o futuro que estava guardado impossível de alcançar (o
    /// estado de que ele partia deixou de existir), e enumerar os sítios que
    /// gravam — hoje três: o traço, a máscara, a subdivisão — é a lista que
    /// nasce incompleta no dia em que aparece o quarto. Aqui o quarto nasce
    /// certo.
    pub(super) fn record(&mut self, undo: StrokeUndo) {
        let object = self.active;
        self.undo.push(Entry { object, undo });
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
        if !self.obj_mut().stack.add_level() {
            return false;
        }
        self.record(StrokeUndo::AddedLevel);
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        true
    }

    /// **RECONSTRÓI o nível de baixo** a partir da base — o des-subdividir.
    /// `false` se a base não é uma subdivisão, ou se o artista não está nela.
    ///
    /// ⚠️ **É o único jeito de uma malha IMPORTADA ganhar multiresolução.** O
    /// `subdivide` só acrescenta para cima, então um OBJ denso nasce com um
    /// nível: dá para esculpir a pele e **não** para mover a forma grande. O
    /// preço é que a malha inteira é renumerada — ver o `multires_reverse.rs`.
    pub(super) fn reverse_level(&mut self) -> bool {
        let Some(rev) = self.obj_mut().stack.reverse() else {
            return false;
        };
        self.record(StrokeUndo::ReversedLevel(Box::new(rev)));
        // O traço em voo fala de índices que a renumeração acabou de mover, e a
        // GPU tem de receber a malha inteira: os vértices trocaram de lugar.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        true
    }

    /// **TAPA todo buraco da malha.** Devolve o relatório, ou `None` se não
    /// havia buraco — ou se a pilha tem mais de um nível.
    ///
    /// ⚠️ **A recusa com pilha é estrutural, não cautela.** Tapar muda a
    /// TOPOLOGIA da base, e todo nível acima é `subdivide` dela: o detalhe deles
    /// passaria a descrever uma malha que não existe mais. Descer não resolve —
    /// o que resolveria é reconstruir a pilha, que é outra operação. O log diz
    /// para tapar ANTES de subdividir.
    pub(super) fn close_holes(&mut self) -> Option<ph2d_mesh::HoleFill> {
        if self.obj().stack.level_count() != 1 {
            return None;
        }
        let report = ph2d_mesh::fill_holes(self.obj_mut().stack.mesh_mut());
        if report.is_noop() {
            return Some(report);
        }
        self.record(StrokeUndo::FilledHoles {
            verts: report.verts_before(),
            faces: report.faces_before(),
        });
        // A contagem de vértices mudou: o traço em voo fala de outra malha e os
        // buffers do device mudaram de TAMANHO.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Some(report)
    }

    /// **RECONSTRÓI a malha por voxelização** — o botão do W7.
    ///
    /// Devolve `None` com a pilha de multires montada, e a recusa é da MESMA
    /// família da de tapar buraco: um remesh troca a topologia inteira, e todo
    /// nível acima é `subdivide` da base — o detalhe deles passaria a descrever
    /// uma malha que não existe mais. ⚠️ **A alternativa seria achatar a pilha
    /// em silêncio**, e isso é destruir trabalho que o artista autorou sem
    /// dizer; o log nomeia a recusa e o conserto.
    pub(super) fn remesh(&mut self, resolution: u32) -> Option<ph2d_sdf::RemeshReport> {
        if self.obj().stack.level_count() != 1 {
            return None;
        }
        let (out, report) = ph2d_sdf::remesh(self.obj().stack.mesh(), resolution).ok()?;
        let previous = core::mem::replace(self.obj_mut().stack.mesh_mut(), out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais,
        // e os buffers do device mudaram de tamanho.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Some(report)
    }

    /// **Troca de nível** — `up` sobe, senão desce. Devolve `false` na ponta.
    ///
    /// ⚠️ **Ela ENTRA na história, e a versão anterior estava errada.** O doc
    /// dela dizia *"não é uma edição: o gesto é o próprio inverso"* — e a
    /// segunda metade é falsa, porque **descer ESCREVE na malha de baixo** (o
    /// carimbo do que foi esculpido em cima). Uma mutação fora da história é uma
    /// mutação sem inverso: o Enio reportou o sintoma como *artefatos na malha*,
    /// e ele era exatamente isto — desfazer uma subdivisão devolvia o artista a
    /// uma base que ele nunca autorou.
    pub(super) fn change_level(&mut self, up: bool) -> bool {
        let from = self.obj().stack.level();
        let entry = if up {
            if !self.obj_mut().stack.higher() {
                return false;
            }
            StrokeUndo::Ascended { from }
        } else {
            let Some(stamped) = self.obj_mut().stack.lower() else {
                return false;
            };
            StrokeUndo::Descended {
                from,
                stamped: Box::new(stamped),
            }
        };
        self.record(entry);
        // O traço em voo fala de outra malha; a GPU precisa de tudo.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        true
    }

    /// Fecha o traço e guarda o desfazer.
    pub(super) fn close_stroke(&mut self) {
        if self.stroke.touched().is_empty() {
            return;
        }
        let entry = StrokeUndo::Stroke {
            level: self.obj().stack.level(),
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
