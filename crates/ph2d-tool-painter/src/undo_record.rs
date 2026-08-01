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
    /// **A porta ÚNICA por onde o cursor anda** — e ela zera o journal do passo.
    ///
    /// ⚠️ Os dois fatos são o MESMO fato: o cursor descreve *"o estado do último commit"* e o journal
    /// descreve *"os bytes velhos desde o último commit"*. Mover um sem zerar o outro deixa o journal
    /// falando de um passado velho demais — e isso **não falha**, só devolve o estado errado quando
    /// alguém acreditar nele. A rede de verificação (`audit_journal_matches_the_before`) nasceu
    /// vermelha exatamente assim, com o journal na tela virgem e o cursor já pintado.
    pub(super) fn set_cursor(&mut self, at: ModelSnapshot) {
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
            && let Some(cursor) = self.cursor.as_deref()
            && let Some(first_before) = top.materialize(cursor, true)
        {
            let old = self.undo.pop().expect("o topo que acabamos de ler");
            self.bytes -= old.heap_bytes();
            let cursor = after.clone();
            // O `first_before` é um estado MAIS VELHO (o começo do run), então a proveniência do journal
            // não bate e o guard recusa sozinho — a fonte é passada mesmo assim porque *quem decide é o
            // guard*, e uma lista de rotas que "não precisam perguntar" é a enumeração que apodrece.
            let src = self.relief_source(&first_before, &cursor);
            let entry = UndoEntry::split(*first_before, after, Some(kind), None, src);
            self.set_cursor(cursor);
            self.bytes += entry.heap_bytes();
            self.undo.push(entry);
            self.cap();
            return;
        }
        let cursor = after.clone();
        let src = self.relief_source(&before, &cursor);
        let entry = UndoEntry::split(before, after, Some(kind), None, src);
        self.set_cursor(cursor);
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        self.drop_redo();
        self.cap();
    }
}
