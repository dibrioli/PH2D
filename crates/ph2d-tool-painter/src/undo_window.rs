//! **A JANELA VEM DE QUEM ESCREVE** — filho de [`super`], o outro lado do [`crate::undo_delta`].
//!
//! O commit de um traço custava **23,7 ms a 4096²** (doc 28 §5.16) e quase tudo era `PlaneDeltas::split`
//! **varrendo os dois endpoints de quatro planos — ~470 MB — para DERIVAR uma janela que quem escreveu já
//! conhecia**. Este módulo é o canal por onde ela chega.
//!
//! ## Por que isto é seguro, e onde a segurança mora
//!
//! ⚠️ O `diff_window` documenta a objeção certa: *"uma janela informada errado não falha: ela deixa fora
//! exatamente os texels que o undo depois não restaura"*. Três coisas respondem a ela, e nenhuma é uma
//! enumeração de chamadores:
//!
//! 1. **A declaração é o `mark_dirty`**, que **toda** escrita de canvas já faz — é ele que faz o pixel
//!    aparecer. Uma escrita que o pulasse já seria um defeito visível hoje (tinta que não sobe para a
//!    tela), então os dois modos de falha coincidem em vez de se esconderem.
//! 2. **A janela só é ACEITA se o snapshot for posterior ao último commit** ([`WriteWindow::hint_for`]).
//!    Ela acumula desde o commit anterior, então para qualquer `before` capturado depois dele ela é um
//!    **superconjunto** — e superconjunto é seguro por construção (a materialização reescreve texels que
//!    não mudaram com os valores que eles já tinham). Um `before` mais VELHO que o último commit é
//!    recusado e o commit varre, como sempre fez.
//! 3. **Em build de DEBUG o `split` CONFERE** (`undo_delta`): ele deriva a janela verdadeira e afirma que
//!    ela cabe na declarada. A suíte inteira desta crate roda em debug em ~4 s, e ela exercita fill,
//!    seleção, warp, sculpt, máscara, inpaint, clone, aquarela, Wet Paint e os shape editors — de modo
//!    que o invariante é **verificado a cada rodada, em todo gesto que algum teste encena**.

use crate::compositor::Region;

/// A união do que foi escrito desde o último commit estrutural, e desde QUANDO.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WriteWindow {
    /// O valor do contador de escritas quando esta janela foi zerada (isto é, no último commit).
    since: u64,
    /// A união de tudo que foi declarado desde então. `None` = nada foi declarado.
    rect: Option<Region>,
    /// Quantos acessos de escrita foram abertos e **não** declararam onde escreveram.
    ///
    /// ⚠️ É o que faz de "esquecer" um custo em tempo em vez de um custo em texels: enquanto ele for > 0
    /// a janela não é oferecida e o commit varre, como sempre fez. Zera no `reset` de cada commit, então
    /// um sítio esquecido degrada aquele passo e nada além dele.
    undeclared: u32,
}

impl WriteWindow {
    /// Um acesso de escrita foi aberto e ainda não disse onde escreveu.
    ///
    /// É este contador que faz de "esquecer" um custo em TEMPO em vez de um custo em TEXELS: enquanto
    /// ele for > 0 a janela não é oferecida e o commit varre, como sempre fez.
    pub const fn open_write(&mut self) {
        self.undeclared = self.undeclared.saturating_add(1);
    }

    /// Um acesso aberto declara a região que escreveu (`None` = não escreveu nada).
    pub fn mark(&mut self, r: Option<Region>) {
        self.undeclared = self.undeclared.saturating_sub(1);
        if let Some(r) = r {
            self.rect = Some(self.rect.map_or(r, |acc| union(acc, r)));
        }
    }

    /// Quantos acessos de escrita seguem abertos sem dizer onde escreveram.
    ///
    /// Readout de diagnóstico: enquanto for > 0 o commit **varre**, então este é o número que separa
    /// *"o commit custa extração"* de *"o commit custa varredura porque alguém não declarou"* — a
    /// pergunta que decide o próximo degrau do S3 (doc 28 §7).
    #[must_use]
    pub const fn undeclared(&self) -> u32 {
        self.undeclared
    }

    /// `true` se esta janela (e o journal ao lado dela) descrevem um passado **posterior** a
    /// `snapshot_writes` — a metade de PROVENIÊNCIA do [`Self::hint_for`], perguntada sozinha.
    ///
    /// Um `before` mais VELHO que o último commit vê escritas que a janela já esqueceu, então nada do
    /// que ela diz vale para ele; a rede de verificação precisa exatamente desta pergunta, sem a
    /// metade de "alguém deixou acesso aberto".
    #[must_use]
    pub const fn covers(&self, snapshot_writes: u64) -> bool {
        snapshot_writes >= self.since
    }

    /// Zera a janela: daqui para a frente ela descreve o que vier depois de `writes`.
    pub fn reset(&mut self, writes: u64) {
        self.since = writes;
        self.rect = None;
        self.undeclared = 0;
    }

    /// A janela que um commit pode usar para um `before` capturado com o contador em `snapshot_writes` —
    /// ou `None` (varra), que é sempre a resposta correta, só mais cara.
    ///
    /// ⚠️ **A comparação é `>=`, e é ela que torna a declaração segura.** A janela acumula desde o último
    /// commit; se o snapshot é posterior a esse ponto, tudo que ele não viu está aqui dentro. Se for
    /// ANTERIOR (uma transação aninhada, um shape aberto atravessando um layer op), há escritas fora
    /// dela e a única resposta honesta é varrer.
    pub const fn hint_for(&self, snapshot_writes: u64) -> Option<Region> {
        if self.undeclared == 0 && snapshot_writes >= self.since {
            self.rect
        } else {
            None
        }
    }
}

/// O bbox que contém os dois. Irmão do `tool::paint::region::union_region`, que é privado ao módulo de
/// pintura; esta cópia existe porque o histórico não pode depender do interior do tool.
const fn union(a: Region, b: Region) -> Region {
    let x0 = if a.x < b.x { a.x } else { b.x };
    let y0 = if a.y < b.y { a.y } else { b.y };
    let ax = a.x + a.w;
    let bx = b.x + b.w;
    let ay = a.y + a.h;
    let by = b.y + b.h;
    let x1 = if ax > bx { ax } else { bx };
    let y1 = if ay > by { ay } else { by };
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// **O estado de ESCRITA de um passo**, com mutabilidade interior — a janela declarada e, ao lado
/// dela, o [`journal`](crate::undo::journal) que captura os bytes velhos na hora da escrita.
///
/// A mutabilidade interior não é conveniência: um mesmo gesto tem vários planos abertos ao mesmo tempo
/// (o Inflate escreve altura, cobertura, material e RGBA numa tacada) e cada um carrega um acesso vivo.
/// Com `&mut` eles se excluiriam; com `Cell`/`RefCell` compartilham a mesma declaração.
///
/// Isto torna o tool `!Sync`, o que é livre: `Tool: std::any::Any` e nada no editor exige `Sync`.
///
/// ⚠️ **Os dois campos respondem à MESMA pergunta por dois caminhos** (*o que este passo escreveu?*) —
/// a janela responde ONDE, o journal responde O QUÊ ESTAVA LÁ. Ficam juntos de propósito: separá-los
/// em dois campos do tool seria abrir espaço para um sítio declarar num e esquecer o outro.
#[derive(Debug, Default)]
pub struct WriteState {
    win: std::cell::Cell<WriteWindow>,
    /// Quantas trocas de plano estão de pé — enquanto for ímpar, o campo `canvas_rgba` do tool hospeda
    /// um plano que **não é a tela** e o journal do canvas não pode capturar dele.
    ///
    /// Contador e não booleano porque a paridade é o invariante: cada
    /// [`swap_canvas_plane`](crate::tool::paint::plane_fork::swap_canvas_plane) é **uma** troca, e
    /// aninhá-las (o gate por dentro da máscara) tem de voltar ao plano certo na ordem inversa.
    foreign_plane: std::cell::Cell<u32>,
    /// Os bytes velhos do CANVAS, por tile. ⚠️ **Só em debug/test** enquanto o journal for uma REDE de
    /// verificação e não a fonte do undo: capturar *e* forkar seria pagar as duas coisas (doc 28 §7).
    #[cfg(any(test, debug_assertions))]
    canvas: std::cell::RefCell<crate::undo::journal::TileJournal<u8>>,
}

impl WriteState {
    /// A janela declarada até agora.
    #[must_use]
    pub fn get(&self) -> WriteWindow {
        self.win.get()
    }

    /// Reescreve a janela.
    pub fn set(&self, w: WriteWindow) {
        self.win.set(w);
    }

    /// Uma troca de plano começou — ou terminou. Ver
    /// [`swap_canvas_plane`](crate::tool::paint::plane_fork::swap_canvas_plane), a única porta.
    pub(crate) fn toggle_foreign_plane(&self) {
        self.foreign_plane.set(self.foreign_plane.get() ^ 1);
    }

    /// `true` enquanto o campo `canvas_rgba` hospeda um plano que não é a tela.
    ///
    /// Consumidor: a captura, que é debug-only — daí o `cfg`. O `toggle` acima **não** é gateado: ele é
    /// chamado pela porta de troca em qualquer perfil, e um contador que só existe em debug faria a
    /// porta ter duas formas.
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn on_foreign_plane(&self) -> bool {
        self.foreign_plane.get() & 1 == 1
    }

    /// **Captura o canvas antes de ele ser escrito** — `area` em ELEMENTOS (`x*4` para RGBA), `None`
    /// quando o sítio não sabe onde vai escrever.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn capture_canvas(
        &self,
        buf: &[u8],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        if self.on_foreign_plane() {
            return; // estes bytes são do scratch da máscara / do plano `free` — não da tela
        }
        self.canvas.borrow_mut().capture(buf, stride, area);
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn capture_canvas(
        &self,
        _buf: &[u8],
        _stride: usize,
        _area: Option<(usize, usize, usize, usize)>,
    ) {
    }

    /// O byte que o elemento `i` do canvas tinha no início do passo, se o tile dele foi capturado.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn canvas_before(&self, i: usize) -> Option<u8> {
        self.canvas.borrow().get(i)
    }

    /// O passo fechou: o journal esquece o que capturou.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn reset_journal(&self) {
        self.canvas.borrow_mut().reset();
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn reset_journal(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn r(x: u32, y: u32, w: u32, h: u32) -> Region {
        Region { x, y, w, h }
    }

    #[test]
    fn the_window_is_the_union_of_what_was_declared() {
        let mut w = WriteWindow::default();
        assert_eq!(w.hint_for(0), None, "nada declarado = varra");
        w.open_write();
        w.mark(Some(r(10, 20, 5, 5)));
        w.open_write();
        w.mark(Some(r(30, 10, 5, 5)));
        assert_eq!(w.hint_for(0), Some(r(10, 10, 25, 15)));
    }

    /// **Um `before` mais VELHO que o último commit é recusado** — é a metade que impede a janela de ser
    /// pequena demais, e sem ela uma transação aninhada perderia os texels escritos antes do reset.
    #[test]
    fn a_snapshot_older_than_the_last_commit_gets_no_hint() {
        let mut w = WriteWindow::default();
        w.open_write();
        w.mark(Some(r(0, 0, 4, 4)));
        w.reset(7); // o commit
        w.open_write();
        w.mark(Some(r(100, 100, 4, 4)));
        assert_eq!(w.hint_for(7), Some(r(100, 100, 4, 4)), "capturado NO reset");
        assert_eq!(w.hint_for(9), Some(r(100, 100, 4, 4)), "capturado DEPOIS");
        assert_eq!(w.hint_for(6), None, "capturado ANTES do reset");
    }
}
