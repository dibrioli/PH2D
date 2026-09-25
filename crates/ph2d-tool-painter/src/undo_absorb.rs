//! **A reconciliação da história com escritas que NÃO passaram por ela** — filho de [`super`]
//! (`#[path]`, então ele enxerga os privados do controller), split pelo cap de LOC.
//!
//! Uma única porta, chamada na entrada dos dois `record_*`. O porquê está no cabeçalho de [`super`]:
//! o escorrido do Wet Paint, que o smoke do Enio achou em 2026-07-26.

use super::{ModelSnapshot, UndoController, UndoEntry};
use std::sync::Arc;

#[cfg(test)]
thread_local! {
    /// **Quantas vezes a absorção de fato RE-PARTIU o topo neste thread.**
    ///
    /// ⚠️ Ela pop-a e push-a uma entrada, então `undo_depth()` **não muda** e `retained_bytes()` pode
    /// não mudar: um gate que os observasse ficaria verde sobre um disparo espúrio — foi exatamente o
    /// que aconteceu com a 1ª versão do `a_clean_pen_down_does_not_wake_the_absorption`, que
    /// SOBREVIVEU à mutação. A pergunta é *"ela disparou?"*, e a forma honesta de a fazer é contar
    /// (o idioma do `RELIEF_FROM_JOURNAL`).
    pub(crate) static ABSORB_FIRED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };

    /// **Força o caminho CARO** — a metade de controlo do gate que afirma que o barato guarda a MESMA
    /// entrada (`undo_absorb_tests`). Sem ela, o gate compararia o caminho barato consigo mesmo.
    pub(crate) static ONLY_THE_FULL_PATH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    /// **Quantas absorções re-partiram o canvas pelo caminho BARATO** — sem ele, o gate de igualdade
    /// ficaria verde com o caminho barato nunca a correr (os dois lados seriam o caro).
    pub(crate) static CHEAP_FIRED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

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
    pub(crate) fn absorb_foreign_writes(&mut self, before: &ModelSnapshot) {
        let Some(cursor) = self.cursor.as_deref() else {
            return; // história vazia: não há topo a corrigir
        };
        // `split` esvazia o que recebe, então ele come CÓPIAS — e uma cópia de `ModelSnapshot` é um
        // punhado de refcounts.
        let (mut a, mut b) = (cursor.clone(), before.clone());
        // ⚠️ **O RELEVO não participa do detector, e a razão é medida.** O cursor o ELIDE (degrau 4),
        // então ele não tem como testemunhar sobre bytes que não segura: comparado com um `before` que
        // os segura, toda camada sai como `OnlyAfter` = *"apareceu agora"*, `heap_bytes()` nunca é
        // zero, e **a absorção dispara em TODO pen-down** — um re-split + materialize completos por
        // traço. Medido a 4096²: pen-down **5,70 → 36,2 ms**, e a atribuição só apareceu depois de a
        // hipótese óbvia (o alocador reciclando os planos liberados) ser REFUTADA por medição — segurar
        // os planos não devolvia um milissegundo.
        //
        // E a exclusão é honesta, não um atalho: esta porta existe para a escrita de CANVAS que não
        // registrou entrada (o escorrido do Wet Paint). Uma escrita de relevo fora da história é outra
        // pergunta, e quem a responde é o journal — `base_for_top` recusa a base quando ela existe.
        for m in [&mut a, &mut b] {
            m.heights.clear();
            m.covers.clear();
            m.mats.clear();
            m.relief_elided = crate::undo::elide::ElidedRelief::default();
        }
        let detected = crate::undo_planes::PlaneDeltas::split(&mut a, &mut b, None, None);
        if detected.heap_bytes() == 0 {
            return; // o topo já termina onde este passo começa: o caso comum, e ele não custa nada
        }
        // ⚠️ **A base do RELEVO vem da porta única** ([`Self::base_for_top`]): o cursor elide os planos
        // (degrau 4) e materializar contra ele devolveria `None`, matando a absorção em silêncio. Ela
        // recusa quando alguém escreveu relevo desde o commit, e aí a absorção sai calada — que é a
        // mesma resposta que ela já dava quando o cursor não descrevia o topo.
        let Some(base) = self.base_for_top() else {
            return;
        };
        let Some(cursor) = self.cursor.as_deref() else {
            return;
        };
        let Some(mut top) = self.undo.pop() else {
            return;
        };
        let (kind, old_bytes) = (top.kind, top.heap_bytes());
        // ⭐ **O CANVAS vai pelo caminho barato quando ele dá a MESMA resposta** (medido 2026-09-24:
        // a materialização era uma cópia de 67 MB e o re-split voltava a varrer os 67 MB para achar
        // uma janela que o topo e o detector já conheciam — `~9–11 ms` de um pen-down de `~25`). A
        // prova de que é a mesma entrada está no cabeçalho de `undo_delta_absorb.rs`, e o gate a
        // compara com o caminho caro campo a campo. Os OUTROS planos seguem pela porta de sempre: o
        // escorrido só escreve canvas, e um segundo caminho para dezanove planos seria a segunda
        // lista que nasce incompleta.
        let stride = crate::undo_delta::Strides::of(top.before.canvas_size.0).rgba;
        let cheap = top.planes.canvas().absorbed(
            &cursor.canvas_rgba,
            &before.canvas_rgba,
            detected.canvas(),
            stride,
        );
        #[cfg(test)]
        let cheap = cheap.filter(|_| !ONLY_THE_FULL_PATH.with(std::cell::Cell::get));
        // Com o canvas já calculado, o topo materializa-se SEM ele (é essa a cópia que se poupa) —
        // e depois é devolvido intacto se a materialização recusar.
        let taken = cheap
            .is_some()
            .then(|| std::mem::take(top.planes.canvas_mut()));
        let Some(mut first_before) = top.materialize(cursor, &base, true) else {
            // O cursor não descreve mais o delta do topo. Não há como esticá-lo honestamente, e mentir
            // aqui seria pior que a divergência: sai calado, exatamente como o `undo` faz.
            if let Some(canvas) = taken {
                *top.planes.canvas_mut() = canvas;
            }
            self.undo.push(top);
            return;
        };
        if cheap.is_some() {
            // O mesmo `Arc` dos dois lados: o `split` vê-os idênticos e não varre o canvas.
            first_before.canvas_rgba = Arc::clone(&before.canvas_rgba);
        }
        #[cfg(test)]
        ABSORB_FIRED.with(|c| c.set(c.get() + 1));
        #[cfg(test)]
        if cheap.is_some() {
            CHEAP_FIRED.with(|c| c.set(c.get() + 1));
        }
        self.bytes -= old_bytes;
        let mut entry = UndoEntry::split(*first_before, before.clone(), kind, None, None);
        if let Some(canvas) = cheap {
            *entry.planes.canvas_mut() = canvas;
        }
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        // O cursor anda junto: o topo agora termina no estado que este passo encontrou.
        self.set_cursor(before.clone());
    }
}
