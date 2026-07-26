//! **A reconciliação da história com escritas que NÃO passaram por ela** — filho de [`super`]
//! (`#[path]`, então ele enxerga os privados do controller), split pelo cap de LOC.
//!
//! Uma única porta, chamada na entrada dos dois `record_*`. O porquê está no cabeçalho de [`super`]:
//! o escorrido do Wet Paint, que o smoke do Enio achou em 2026-07-26.

use super::{ModelSnapshot, UndoController, UndoEntry};

impl UndoController {
    /// **Absorve, na entrada do topo, o que foi escrito no canvas SEM entrada de undo.**
    ///
    /// # O invariante, e por que ele passou a ser estabelecido em vez de assumido
    ///
    /// O delta se sustenta enquanto **`entry[i].after` for o estado logo antes do commit `i+1`** — é o
    /// contrato da história linear, e o cabeçalho deste módulo o registrava como *"não há repro
    /// conhecido"*. **O smoke do Enio (2026-07-26) achou o repro**, e ele não era um defeito de outro
    /// sistema: era a água.
    ///
    /// Um traço de **Wet Paint** grava a entrada no pen-up, e a sim **continua correndo** — cada tick
    /// composita pigmento novo no `canvas_rgba` **sem gravar entrada nenhuma**. É literalmente o que um
    /// *escorrido* é: tinta que chega onde o traço não passou, depois que ele acabou. O traço SEGUINTE
    /// adota esse canvas como o seu `before`, e aí o escorrido virou história **de ninguém**: desfazer o
    /// traço que o causou reescreve só a JANELA daquele traço, e o escorrido está fora dela.
    ///
    /// ⚠️ **A cura não é fazer a sim gravar entradas** (seriam 40 passos de undo por segundo) nem adiar o
    /// commit até a tinta secar (decisão de produto, em aberto). É fechar o vão **no lugar onde o
    /// invariante é consumido**: se o estado que chega como `before` não é o `cursor` (que É o `after` do
    /// topo), então alguém escreveu no meio — e essa escrita pertence ao passo anterior, porque foi ele
    /// que a causou. A entrada do topo é re-partida para terminar **aqui**.
    ///
    /// # Custo: zero no caso comum, e é por construção
    ///
    /// A pergunta *"alguém escreveu?"* é feita pelo MESMO `PlaneDeltas::split` que o commit usa — sobre
    /// clones dos dois snapshots, que são clones de `Arc`, não de pixels. Ele começa por `Arc::ptr_eq`
    /// em cada plano, então **sem escrita estrangeira ele não lê um byte**. Reusar essa porta em vez de
    /// escrever uma segunda lista de planos é deliberado: uma segunda lista é a que nasce incompleta
    /// quando o vigésimo plano aparecer.
    pub(super) fn absorb_foreign_writes(&mut self, before: &ModelSnapshot) {
        let Some(cursor) = self.cursor.as_deref() else {
            return; // história vazia: não há topo a corrigir
        };
        // `split` esvazia o que recebe, então ele come CÓPIAS — e uma cópia de `ModelSnapshot` é um
        // punhado de refcounts.
        let (mut a, mut b) = (cursor.clone(), before.clone());
        if crate::undo_planes::PlaneDeltas::split(&mut a, &mut b).heap_bytes() == 0 {
            return; // o topo já termina onde este passo começa: o caso comum, e ele não custa nada
        }
        let Some(top) = self.undo.last() else {
            return;
        };
        let (kind, Some(first_before)) = (top.kind, top.materialize(cursor, true)) else {
            // O cursor não descreve mais o delta do topo. Não há como esticá-lo honestamente, e mentir
            // aqui seria pior que a divergência: sai calado, exatamente como o `undo` faz.
            return;
        };
        let old = self.undo.pop().expect("o topo que acabamos de ler");
        self.bytes -= old.heap_bytes();
        let entry = UndoEntry::split(*first_before, before.clone(), kind);
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        // O cursor anda junto: o topo agora termina no estado que este passo encontrou.
        self.cursor = Some(Box::new(before.clone()));
    }
}
