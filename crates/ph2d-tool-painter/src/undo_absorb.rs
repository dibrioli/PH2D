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

    /// **Quantas vezes o detector perguntou ao canvas só dentro da janela DECLARADA.**
    pub(crate) static DETECTED_WITHIN: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };

    /// Quantas absorções correram com o vidro do reset SEGURADO (ADR-0204, o caminho caro).
    pub(crate) static VIDRO_SEGURADO: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };

    /// Quantas absorções correram com o vidro COMPOSTO com o journal (o caminho do produto).
    pub(crate) static VIDRO_COMPOSTO: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };

    /// A rota do vidro do escorrido — o gate compara a composta com a segurada; a sonda mede as três.
    pub(crate) static ROTA_DO_VIDRO: std::cell::Cell<RotaDoVidro> =
        const { std::cell::Cell::new(RotaDoVidro::Composta) };
}

/// Por onde a absorção leva o vidro que o escorrido escreveu depois do commit (ADR-0204).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RotaDoVidro {
    /// O produto: o delta do topo composto com o journal, nas caixas dos dois.
    Composta,
    /// O oráculo da composta: o cursor segura o vidro do reset (três varreduras do plano inteiro).
    #[cfg(test)]
    Segurada,
    /// Só a sonda: o vidro elidido não testemunha (o custo e a resposta ERRADA de antes de `69158c3890`).
    #[cfg(test)]
    Antes,
}

/// O vidro na absorção: `Elidido` não testemunha; `Segurado` vem no cursor; `Composto` vem do journal.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Vidro {
    Elidido,
    Segurado,
    Composto,
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
    ///
    /// # O VIDRO pelo journal (ADR-0204)
    ///
    /// O cursor elide o vidro (não segura os bytes), então ele não testemunha no detector. Mas o escorrido
    /// do Wet ESCREVE o vidro depois do commit, e essa escrita é do topo como a da tela. Só quando o journal
    /// do vidro capturou algo desde o reset (o caso comum não paga nada) o delta do vidro do topo é COMPOSTO
    /// com o journal nas caixas dos dois ([`StoredMap::composto_com_o_journal`](crate::undo_delta::StoredMap)):
    /// `antes0 → R` seguido de `R → vivo`. Onde a forma não compõe, o caminho que SEGURA o vidro do reset
    /// (`journal.get(i).unwrap_or(vivo[i])`) — o detector, a base e a re-partição do oráculo, com três varreduras
    /// do plano inteiro (medido +34 ms no pen-down a 4096², `watercolor_vidro_custo_sonda`).
    pub(crate) fn absorb_foreign_writes(&mut self, before: &ModelSnapshot) {
        crate::sonda_partes::parte("antes_da_absorcao");
        if !self.o_escorrido_escreveu_o_vidro(before) {
            self.absorb_foreign_writes_inner(before, Vidro::Elidido);
            crate::sonda_partes::parte("absorcao");
            return;
        }
        #[cfg(test)]
        let rota = ROTA_DO_VIDRO.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let rota = RotaDoVidro::Composta;
        if rota == RotaDoVidro::Composta
            && self
                .absorb_foreign_writes_inner(before, Vidro::Composto)
                .is_some()
        {
            #[cfg(test)]
            VIDRO_COMPOSTO.with(|c| c.set(c.get() + 1));
            crate::sonda_partes::parte("absorcao");
            return;
        }
        #[cfg(test)]
        if rota == RotaDoVidro::Antes {
            self.absorb_foreign_writes_inner(before, Vidro::Elidido);
            return;
        }
        let held = self.cursor_com_o_vidro_do_reset(before);
        crate::sonda_partes::parte("vidro_do_reset");
        let Some(held) = held else {
            self.absorb_foreign_writes_inner(before, Vidro::Elidido);
            crate::sonda_partes::parte("absorcao");
            return;
        };
        #[cfg(test)]
        VIDRO_SEGURADO.with(|c| c.set(c.get() + 1));
        let elidido = self.cursor.replace(Box::new(held));
        if self.absorb_foreign_writes_inner(before, Vidro::Segurado) != Some(true) {
            self.cursor = elidido;
        }
        crate::sonda_partes::parte("absorcao");
    }

    /// O escorrido pode ter escrito o vidro depois do commit — `false` (sem custo) quando o journal do vidro
    /// está calado, a chave desligada, o cursor o segura, ou o `before` também o elide (o commit: a absorção
    /// do escorrido já correu no pen-down).
    fn o_escorrido_escreveu_o_vidro(&self, before: &ModelSnapshot) -> bool {
        self.cursor.as_deref().is_some_and(|cursor| {
            crate::undo::window::vidro_pelo_journal()
                && !cursor.relief_elided.vidros.is_empty()
                && before.relief_elided.vidros.is_empty()
                && !self.write_state.vidro_untouched()
        })
    }

    /// O cursor com o vidro do último reset segurado (o caminho caro, onde a composição não serve).
    fn cursor_com_o_vidro_do_reset(&self, before: &ModelSnapshot) -> Option<ModelSnapshot> {
        let cursor = self.cursor.as_deref()?;
        let vidros = self
            .write_state
            .vidro_no_reset(&cursor.relief_elided.vidros, &before.vidros)?;
        let mut c = cursor.clone();
        c.vidros = vidros;
        c.relief_elided.vidros.clear();
        Some(c)
    }

    /// `Some(true)` = re-partiu o topo (e o cursor andou). `None` só no `Vidro::Composto`, e antes de tocar em
    /// nada: a forma não compõe, quem chama corre o caminho que segura o vidro.
    fn absorb_foreign_writes_inner(&mut self, before: &ModelSnapshot, modo: Vidro) -> Option<bool> {
        let Some(cursor) = self.cursor.as_deref() else {
            return Some(false); // história vazia: não há topo a corrigir
        };
        // O vidro do topo seguido do journal (ADR-0204): o mapa `antes0 → vivo`, e se o escorrido o mudou.
        let vidro_novo = match (modo, self.undo.last()) {
            (Vidro::Composto, Some(top)) => Some(
                self.write_state
                    .vidro_composto(top.planes.vidros(), &before.vidros)?,
            ),
            (Vidro::Composto, None) => return Some(false),
            _ => None,
        };
        let pingou_no_vidro = vidro_novo.as_ref().is_some_and(|(_, p)| *p);
        // `split` esvazia o que recebe, então ele come CÓPIAS — e uma cópia de `ModelSnapshot` é um
        // punhado de refcounts.
        let (mut a, mut b) = (cursor.clone(), before.clone());
        // A TELA elidida (B17.4): o mesmo reset dos dois lados ⇒ a mesma tela, sem a ler; senão ela volta pelo journal.
        let live = before
            .canvas_elided
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .unwrap_or_else(|| Arc::clone(&before.canvas_rgba));
        let mesma_tela = matches!(
            (&cursor.canvas_elided, &before.canvas_elided),
            (Some(x), Some(y)) if x.ptr_eq(y)
        );
        if mesma_tela {
            let p = Arc::new(Vec::new());
            a.canvas_rgba = Arc::clone(&p);
            b.canvas_rgba = p;
        } else {
            let (Some(ca), Some(cb)) = (
                self.canvas_held(cursor, &live),
                self.canvas_held(before, &live),
            ) else {
                return Some(false); // a testemunha morreu: não há o que comparar honestamente
            };
            a.canvas_rgba = ca;
            b.canvas_rgba = cb;
        }
        a.canvas_elided = None;
        b.canvas_elided = None;
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
            // O vidro descrito pelo journal (ADR-0204) também não testemunha aqui, pela mesma razão.
            if crate::undo::window::vidro_pelo_journal() && modo != Vidro::Segurado {
                m.vidros.clear();
            }
            m.relief_elided = crate::undo::elide::ElidedRelief::default();
        }
        // ⭐ **O CANVAS é perguntado só dentro da janela DECLARADA** (`crate::undo::window`), quando
        // ela existe e descreve um passado que inclui o do cursor: o escorrido do Wet Paint declara a
        // região que escreve (`wetpaint::composite`), e a janela acumula desde o último commit — um
        // superconjunto de tudo o que mudou desde o cursor. A resposta é a MESMA do detector de sempre
        // (a janela exacta, varrida no recorte — `StoredPlane::split_exact_within`), e em DEBUG a
        // verdadeira é derivada e tem de caber na declarada. Medido a 4096²: o detector varria a tela
        // inteira em `3–8 ms` por pen-down para achar um escorrido do tamanho de um pincel. Sem janela
        // (um acesso ficou por declarar, ou o cursor é mais velho que ela) é o caminho de sempre.
        let stride = crate::undo_delta::Strides::of(a.canvas_size.0).rgba;
        let declared = self
            .write_state
            .get()
            .hint_for(cursor.writes)
            .and_then(|r| crate::undo_delta::PlaneWindow::from_region(r, a.canvas_size.0, stride));
        #[cfg(test)]
        let declared = declared.filter(|_| !ONLY_THE_FULL_PATH.with(std::cell::Cell::get));
        let (canvas, _dentro) = crate::undo_delta::StoredPlane::split_exact_within(
            &mut a.canvas_rgba,
            &mut b.canvas_rgba,
            stride,
            declared,
        );
        #[cfg(test)]
        if _dentro {
            DETECTED_WITHIN.with(|c| c.set(c.get() + 1));
        }
        // Os dois lados do canvas já foram consumidos (esvaziados): o `split` dos outros planos passa
        // por eles sem ler um byte, e o plano do canvas que vale é o de cima.
        let mut detected = crate::undo_planes::PlaneDeltas::split(&mut a, &mut b, None, None);
        *detected.canvas_mut() = canvas;
        crate::sonda_partes::parte("abs_detector");
        if detected.heap_bytes() == 0 && !pingou_no_vidro {
            return Some(false); // o topo já termina onde este passo começa: o caso comum, e ele não custa nada
        }
        // ⚠️ **A base do RELEVO vem da porta única** ([`Self::base_for_top`]): o cursor elide os planos
        // (degrau 4) e materializar contra ele devolveria `None`, matando a absorção em silêncio. Ela
        // recusa quando alguém escreveu relevo desde o commit, e aí a absorção sai calada — que é a
        // mesma resposta que ela já dava quando o cursor não descrevia o topo.
        // O vidro composto não lê o da base (a materialização corre sem ele, abaixo).
        let Some(base) = self.base_for_top_com(Some(&live), modo == Vidro::Composto) else {
            return Some(false);
        };
        let Some(cursor) = self.cursor.as_deref() else {
            return Some(false);
        };
        let (Some(cursor), Some(before_h)) = (
            self.with_canvas_held(cursor, &live),
            self.with_canvas_held(before, &live),
        ) else {
            return Some(false);
        };
        crate::sonda_partes::parte("abs_base_e_telas");
        let cursor = &cursor;
        let Some(mut top) = self.undo.pop() else {
            return Some(false);
        };
        let (kind, old_bytes, transacao) = (top.kind, top.heap_bytes(), top.transacao);
        // ⭐ **O CANVAS vai pelo caminho barato quando ele dá a MESMA resposta** (medido 2026-09-24:
        // a materialização era uma cópia de 67 MB e o re-split voltava a varrer os 67 MB para achar
        // uma janela que o topo e o detector já conheciam — `~9–11 ms` de um pen-down de `~25`). A
        // prova de que é a mesma entrada está no cabeçalho de `undo_delta_absorb.rs`, e o gate a
        // compara com o caminho caro campo a campo. Os OUTROS planos seguem pela porta de sempre: o
        // escorrido só escreve canvas, e um segundo caminho para dezanove planos seria a segunda
        // lista que nasce incompleta.
        let stride = crate::undo_delta::Strides::of(top.before.canvas_size.0).rgba;
        // Com o vidro composto e a tela que o escorrido NÃO tocou (`R == vivo` na tela), o delta da tela é o do
        // topo tal qual (`antes0 → R`): nem a cópia da tela nem o re-split dela (medido 3,5 + 7,6 ms a 4096²).
        let cheap = if modo == Vidro::Composto
            && matches!(detected.canvas(), crate::undo_delta::StoredPlane::Unchanged)
        {
            Some(top.planes.canvas().clone())
        } else {
            let c = top.planes.canvas().absorbed(
                &cursor.canvas_rgba,
                &before_h.canvas_rgba,
                detected.canvas(),
                stride,
            );
            #[cfg(test)]
            let c = c.filter(|_| !ONLY_THE_FULL_PATH.with(std::cell::Cell::get));
            c
        };
        // Com o canvas já calculado, o topo materializa-se SEM ele (é essa a cópia que se poupa) —
        // e depois é devolvido intacto se a materialização recusar.
        let taken = cheap
            .is_some()
            .then(|| std::mem::take(top.planes.canvas_mut()));
        // O vidro composto já é a resposta: o topo materializa-se sem ele (sem a cópia do plano inteiro).
        let vidro_tirado = vidro_novo
            .is_some()
            .then(|| std::mem::take(top.planes.vidros_mut()));
        let Some(mut first_before) = top.materialize(cursor, &base, true) else {
            // O cursor não descreve mais o delta do topo. Não há como esticá-lo honestamente, e mentir
            // aqui seria pior que a divergência: sai calado, exatamente como o `undo` faz.
            if let Some(canvas) = taken {
                *top.planes.canvas_mut() = canvas;
            }
            if let Some(v) = vidro_tirado {
                *top.planes.vidros_mut() = v;
            }
            self.undo.push(top);
            // O composto devolve a pergunta intacta ao caminho que segura o vidro.
            return (modo != Vidro::Composto).then_some(false);
        };
        crate::sonda_partes::parte("abs_materializa");
        if cheap.is_some() {
            // O mesmo `Arc` dos dois lados: o `split` vê-os idênticos e não varre o canvas.
            first_before.canvas_rgba = Arc::clone(&before_h.canvas_rgba);
        }
        if vidro_novo.is_some() {
            // Idem para o vidro: o `split` dá `Unchanged`, e o mapa composto entra no lugar.
            first_before.vidros = before_h.vidros.clone();
            first_before.relief_elided.vidros.clear();
        }
        #[cfg(test)]
        ABSORB_FIRED.with(|c| c.set(c.get() + 1));
        #[cfg(test)]
        if cheap.is_some() {
            CHEAP_FIRED.with(|c| c.set(c.get() + 1));
        }
        self.bytes -= old_bytes;
        let mut entry = UndoEntry::split(*first_before, before_h, kind, None, None);
        crate::sonda_partes::parte("abs_split");
        entry.transacao = transacao;
        // A água absorvida não é do artista — salvo a da sessão dele (W12): a tinta que ela espalhou
        // depois do registo é deste passo.
        entry.marca = self.marca.estende_pela_agua(top.marca.take());
        if let Some(canvas) = cheap {
            *entry.planes.canvas_mut() = canvas;
        }
        if let Some((v, _)) = vidro_novo {
            *entry.planes.vidros_mut() = v;
        }
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        // O cursor anda junto: o topo agora termina no estado que este passo encontrou.
        self.set_cursor(before.clone());
        Some(true)
    }
}
