//! **COMO UMA ENTRADA NASCE** — filho de [`super`] (`#[path]`, então ele enxerga os privados),
//! split pelo cap de LOC e pela linha de corte natural: aqui mora o que ACRESCENTA à história (o
//! cursor anda, o delta é partido, o cap morde), e no pai o que a CONSULTA.

use super::{CoalesceKind, ModelSnapshot, UndoController, UndoEntry};

impl UndoController {
    /// **De onde o RELEVO tira o lado `before`** — dos journals do passo, quando eles o descrevem (doc
    /// 28 §5.58.2, degrau 2).
    ///
    /// ⚠️ Ela **não decide nada**: quem decide é o guard, dentro de
    /// [`window::WriteState::with_relief_before`](crate::undo::window::WriteState::with_relief_before).
    /// Aqui só se junta o que a pergunta precisa — a proveniência vem do `before` (foi ele que abriu o
    /// passo) e a camada do `after` (é o estado VIVO do tool, o mesmo que a rede do audit consulta).
    ///
    /// ⚠️ **A camada sai do `ModelSnapshot`, e não de um parâmetro novo, de propósito:** o `LayerStack`
    /// já carrega a ativa, então perguntá-la aqui evita mudar a assinatura pública dos dois `record_*`
    /// — e evita que um chamador futuro possa passar uma camada que não é a do passo.
    fn relief_source<'a>(
        &'a self,
        before: &ModelSnapshot,
        after: &ModelSnapshot,
    ) -> Option<crate::undo_planes::ReliefSource<'a>> {
        Some(crate::undo_planes::ReliefSource {
            state: &self.write_state,
            writes: before.writes,
            layer: after.layers.active()?,
        })
    }
    /// **A entrada descreve o relevo?** — se não, a história é DESCARTADA e o cursor re-armado no
    /// estado de agora (degrau 4, doc 28 §5.60).
    ///
    /// O `before` de um traço ELIDE o relevo: ele o descreve pelo journal em vez de o segurar, e é
    /// isso que faz o 1º dab escrever no lugar. O preço é que **a aposta pode ser perdida** — se o
    /// journal não descrever o passo (camadas misturadas, uma escrita que ele não soube capturar, um
    /// plano trocado por inteiro), o estado de antes **não existe em lugar nenhum**: nem na entrada,
    /// nem no tool, que o sobrescreveu.
    ///
    /// ⚠️ **Guardar a entrada assim mesmo seria o pior dos três desfechos.** Ela sairia com o relevo
    /// como `OnlyAfter` ou `Unchanged` — *"não existia antes"* / *"nada mudou"* — e desfazê-la
    /// **apagaria** o relevo, em silêncio, com todos os gates verdes. Descartar a história é a única
    /// resposta honesta, e o `debug_assert` a torna LOUD em toda a suíte: se ela disparar num caminho
    /// real, é o desenho da elisão que está errado, não este guard.
    ///
    /// `true` = quem chama já terminou (a entrada foi descartada).
    fn discard_if_relief_is_lost(&mut self, entry: &UndoEntry, cursor: &ModelSnapshot) -> bool {
        if !entry.relief_indescribable() {
            return false;
        }
        // O readout do journal vai JUNTO — em QUALQUER perfil: *"indescritível"* admite quatro causas
        // (misturado, incompleto, camada errada, plano trocado) e elas pedem correções diferentes.
        // Um descarte de história é evento real, e um log que não diz por quê manda quem o lê adivinhar.
        eprintln!(
            "[painter-undo] o relevo deste passo nao e' descritivel ({}): historico descartado",
            self.write_state.relief_state()
        );
        debug_assert!(
            false,
            "o `before` elidiu o relevo e o journal recusou — ver undo_planes::relief_maps (doc 28 \
             §5.60). A entrada seria guardada dizendo que o relevo nao existia antes, e desfaze-la o \
             APAGARIA."
        );
        self.clear();
        self.set_cursor(cursor.clone());
        true
    }

    /// **A base do RELEVO para materializar o TOPO da história** — o cursor REIDRATADO pelas suas
    /// testemunhas, e `None` quando ela não pode ser oferecida honestamente.
    ///
    /// ⚠️ **Duas bases erradas foram MEDIDAS antes desta.** O cursor elide o relevo (degrau 4), então
    /// materializar contra ele devolve `None` e as rotas internas morreriam caladas; e passar o VIVO
    /// no lugar dele está **errado**, com número: num run coalescido o gate
    /// `a_coalesced_run_recomposes_the_delta` sangrou com `32.0` onde o estado adjacente tinha `26.0`
    /// — a base tem de ser *o estado ADJACENTE à entrada, e nem sempre ele é o de agora*.
    ///
    /// A base certa é o cursor, e ela é **recuperável** enquanto ninguém tiver escrito relevo desde o
    /// commit: aí o plano vivo é o mesmo objeto E os mesmos bytes que a testemunha aponta. Quem
    /// responde *"ninguém escreveu?"* é o journal
    /// ([`relief_untouched`](crate::undo::window::WriteState::relief_untouched)) — uma única captura
    /// derruba a oferta, e as duas rotas caem nos seus early-outs de sempre (a absorção sai calada, o
    /// run coalescido começa uma entrada nova). *Menos esperto nunca, errado jamais.*
    pub(super) fn base_for_top(&self) -> Option<ModelSnapshot> {
        let mut base = self.cursor.as_deref()?.clone();
        if base.relief_elided.is_empty() {
            return Some(base); // nada elidido: o cursor já É a base
        }
        if !self.write_state.relief_untouched() {
            return None;
        }
        let el = base.relief_elided.clone();
        el.rehydrate_into(&mut base).then_some(base)
    }

    /// **A porta ÚNICA por onde o cursor anda** — e ela zera o journal do passo.
    ///
    /// ⚠️ Os dois fatos são o MESMO fato: o cursor descreve *"o estado do último commit"* e o journal
    /// descreve *"os bytes velhos desde o último commit"*. Mover um sem zerar o outro deixa o journal
    /// falando de um passado velho demais — e isso **não falha**, só devolve o estado errado quando
    /// alguém acreditar nele. A rede de verificação (`audit_journal_matches_the_before`) nasceu
    /// vermelha exatamente assim, com o journal na tela virgem e o cursor já pintado.
    /// ⚠️ **E ela ELIDE o relevo** (degrau 4, doc 28 §5.60). O cursor era o **segundo dono permanente**
    /// dos três planos (medido na §5.14: dois donos em repouso, `undo.clear()` levava a um), então
    /// enquanto ele os segurasse a 1ª escrita de todo traço forkaria o documento — e o `before`
    /// elidido não adiantaria nada. *Só as duas elisões juntas levam a contagem a um* (§5.59), e a
    /// testemunha do `before` é quem torna isso observável: com o cursor segurando, o fold forka, o
    /// plano vivo vira outro objeto e o `Weak` recusa.
    ///
    /// ⚠️ **Quem lê o relevo do cursor passa o VIVO no lugar** (degrau 3): `undo`/`redo` já recebiam;
    /// `absorb_foreign_writes` e a extensão do run coalescido passaram a receber nesta wave, porque
    /// materializar contra um cursor sem planos devolve `None` e as duas rotas morreriam em silêncio.
    pub(super) fn set_cursor(&mut self, mut at: ModelSnapshot) {
        #[cfg(test)]
        let elide = self.elide_cursor;
        #[cfg(not(test))]
        let elide = true;
        if elide {
            at.relief_elided =
                crate::undo::elide::ElidedRelief::of(&at.heights, &at.covers, &at.mats);
            at.heights.clear();
            at.covers.clear();
            at.mats.clear();
        }
        self.cursor = Some(Box::new(at));
        self.write_state.reset_journal();
    }

    /// [`Self::record_structural`] com a **janela declarada por quem escreveu** (ver
    /// [`window::WriteWindow`]): `Some(rect)` poupa o `split` de varrer os planos para derivá-la; `None`
    /// é sempre correto, só mais caro.
    pub fn record_structural_hinted(
        &mut self,
        before: ModelSnapshot,
        after: ModelSnapshot,
        hint: Option<crate::compositor::Region>,
    ) {
        self.absorb_foreign_writes(&before);
        // O cursor tem de ser o `after` COMPLETO, então ele é tirado ANTES do split (que o esvazia). É
        // um clone de `Arc`s, não de pixels.
        let cursor = after.clone();
        // ⚠️ **A ORDEM aqui carrega peso: o split vem ANTES do `set_cursor`.** O delta do relevo sai dos
        // journals do passo, e o `set_cursor` **os zera** (os dois fatos são o mesmo fato — ver a doc
        // dele). Instalar o cursor primeiro deixaria o journal vazio quando o delta fosse partido, e o
        // guard cairia no caminho de sempre — silenciosamente, com todos os gates verdes.
        let src = self.relief_source(&before, &cursor);
        let entry = UndoEntry::split(before, after, None, hint, src);
        if self.discard_if_relief_is_lost(&entry, &cursor) {
            return;
        }
        self.set_cursor(cursor);
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        self.drop_redo();
        self.cap();
    }

    /// Record a COALESCIBLE structural transition: when the newest undo entry carries the SAME
    /// [`CoalesceKind`] (and no redo branch intervenes), the run extends — the top entry keeps its
    /// original `before` and adopts this `after` — so N repeated same-kind actions undo as ONE step.
    /// Otherwise it pushes a fresh entry (which starts a new run).
    pub fn record_structural_coalesced(
        &mut self,
        kind: CoalesceKind,
        before: ModelSnapshot,
        after: ModelSnapshot,
    ) {
        self.absorb_foreign_writes(&before);
        if self.redo.is_empty()
            && let Some(top) = self.undo.last()
            && top.kind == Some(kind)
            // ⚠️ Estender o run RECOMPÕE o delta: o `before` da entrada está esvaziado, então ele é
            // materializado do cursor ANTIGO (que é o `after` de que o delta partiu) e re-partido contra
            // o `after` novo. Concatenar os dois deltas seria a alternativa, e ela erra quando as duas
            // janelas se sobrepõem — o segundo passo escreveria por cima do primeiro na ordem errada.
            && let Some(base) = self.base_for_top()
            && let Some(cursor) = self.cursor.as_deref()
            && let Some(first_before) = top.materialize(cursor, &base, true)
        {
            let old = self.undo.pop().expect("o topo que acabamos de ler");
            self.bytes -= old.heap_bytes();
            let cursor = after.clone();
            // O `first_before` é um estado MAIS VELHO (o começo do run), então a proveniência do journal
            // não bate e o guard recusa sozinho — a fonte é passada mesmo assim porque *quem decide é o
            // guard*, e uma lista de rotas que "não precisam perguntar" é a enumeração que apodrece.
            let src = self.relief_source(&first_before, &cursor);
            let entry = UndoEntry::split(*first_before, after, Some(kind), None, src);
            if self.discard_if_relief_is_lost(&entry, &cursor) {
                return;
            }
            self.set_cursor(cursor);
            self.bytes += entry.heap_bytes();
            self.undo.push(entry);
            self.cap();
            return;
        }
        let cursor = after.clone();
        let src = self.relief_source(&before, &cursor);
        let entry = UndoEntry::split(before, after, Some(kind), None, src);
        if self.discard_if_relief_is_lost(&entry, &cursor) {
            return;
        }
        self.set_cursor(cursor);
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        self.drop_redo();
        self.cap();
    }
}
