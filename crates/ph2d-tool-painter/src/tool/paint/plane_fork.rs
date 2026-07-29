//! **Forking a canvas-sized plane, in parallel** — the hitch at the start of every stroke.
//!
//! A gesture that edits a plane in place needs two things at once: a **frozen** copy of what the stroke
//! started from (`pre`, so the render is idempotent and the knobs can re-render) and a **live** buffer to
//! write into. The session takes the frozen one as an `Arc` clone — refcount, not copy — and the first
//! write then calls `Arc::make_mut`, which sees a second owner and copies the whole plane. That copy is
//! genuinely necessary; there is no way to have a snapshot and a mutable buffer without one.
//!
//! What was not necessary is doing it on one thread. Measured on this machine, forking one `f32` plane:
//!
//! ```text
//!   2048²  (16,8 MB)   serial  0,54 ms    parallel  0,32 ms    1,7×
//!   4096²  (67,1 MB)   serial 10,88 ms    parallel  3,34 ms    3,3×
//! ```
//!
//! ⚠️ Note the shape of that table: **four times the data costs twenty times the time.** The cost is not
//! bandwidth, it is the fresh allocation — 67 MB of pages faulted in on first touch, one at a time. That
//! is also why the parallel version wins by more at 4K than at 2K: the faults spread across threads too.
//!
//! It is the whole of the measured set-up hitch. The sculpt's session open cost 12,9–15,1 ms at 4096²
//! against 5,7–6,0 at 2048², and one plane fork is 10,88 ms of it — the only cost in the module that grows
//! when the artist enlarges the canvas (every kernel here is bounded by the brush footprint and is flat in
//! canvas size).
//!
//! Three tools pay it, which is why this is a shared door rather than a sculpt detail: the **sculpt**
//! (`heights`, plus `covers`/`mats`/`canvas_rgba` when Inflate moves matter), the **Reshape** warp and the
//! **Smear** (both re-render the canvas and the three relief planes from a frozen session baseline).

use std::sync::Arc;

// O limiar mora com a cópia (`crate::plane_copy`), porque a DECISÃO daqui (*vale paralelizar?*) e a
// execução lá são a mesma pergunta, e duas cópias dela divergiriam. Ele é em BYTES: um plano de `[u8; 7]`
// move sete vezes a memória de um de `u8` com a mesma contagem.
use crate::plane_copy::worth_parallel;
use crate::undo::window::WriteState;

/// `Arc::make_mut` for a canvas-sized plane, with the copy parallelised.
///
/// Semantically **identical** to `Arc::make_mut` — same value, same aliasing rules, and the returned
/// buffer is uniquely owned either way. The only difference is which threads do the copying, so this is
/// byte-identical by construction: it is a copy, and a copy has one right answer.
///
/// When there is no second owner it delegates untouched — no allocation, no threads, no cost. When there
/// is one but the plane is small it also delegates, because rayon's fork would outweigh the memcpy.
///
/// ⚠️ **SEM CHAMADOR DE PRODUÇÃO, e por isso `cfg(test)`: ela é a REFERÊNCIA CONGELADA.** Todo sítio do
/// produto passa hoje por uma porta que sabe **nomear** o plano (`fork_canvas` / `fork_heights` /
/// `fork_covers` / `fork_mats`) — e nomear é o que separa um journal completo de um incompleto. Um
/// `pub(super)` órfão não é código morto silencioso: é uma **segunda resposta** esperando alguém
/// chamá-la, a lição que o `warp_axis` (doc 28 §5.11) e o `serial_side` (§5.16) já pagaram nesta linha.
///
/// Sob `cfg(test)` ela vira a coisa certa: o oráculo dos gates de byte-identidade e do `Weak`, e o corpo
/// que a sonda de custo mede — a metade de FORK que as quatro portas compartilham, sem a captura. E o
/// compilador passa a ser o guarda: um sítio de produto que a chame **não compila**, o que é mais forte
/// que qualquer arch-gate contando ocorrências.
#[cfg(test)]
pub(super) fn fork_par<'a, T>(arc: &'a mut Arc<Vec<T>>, win: &WriteState) -> &'a mut Vec<T>
where
    T: Copy + Send + Sync,
{
    // ⚠️ **Todo fork nasce NÃO-DECLARADO, e é isso que torna a janela segura.** O histórico paga ~470 MB
    // de varredura por traço a 4096² para DERIVAR uma janela que quem escreve já conhece (doc 28 §5.16);
    // quem quiser poupá-la chama `PainterTool::declare_wrote` com a região depois de escrever.
    //
    // A contagem é o mecanismo inteiro: enquanto houver um fork sem declaração o commit **varre**, que é
    // exatamente o que ele faz hoje. Então o modo de falha de um sítio novo — ou de um que esquece — é
    // *lento*, nunca *errado*. É a resposta à objeção que o `undo_delta::diff_window` documenta, e a
    // §5.17 mostra o que acontece com quem tenta respondê-la por um canal que não é o da escrita.
    //
    // ⚠️ A região quase sempre só é conhecida no FIM (os laços de dab acumulam o `touched` *enquanto*
    // escrevem), e é por isso que a declaração é uma CHAMADA SEPARADA em vez de um argumento daqui.
    open_access(win);
    // ⚠️ **Uma porta que não sabe nomear o plano deixa o journal de RELEVO incompleto.** Ele é um mapa
    // por camada e três tipos de elemento, então a captura tem de saber qual; sem isso a única resposta
    // honesta é *não descrevo este passo*, e o commit deriva como sempre (mesma política do contador de
    // acessos não-declarados). Cada sítio que passa por uma porta nomeada sai desta conta.
    win.note_untracked_write();
    fork_par_raw(arc)
}

/// A metade de DECLARAÇÃO de toda porta: um acesso de escrita foi aberto e ainda não disse onde
/// escreveu. Compartilhada pelas quatro portas para que nenhuma nasça sem ela.
fn open_access(win: &WriteState) {
    let mut w = win.get();
    w.open_write();
    win.set(w);
}

/// A região de um `Region` (pixels) em ELEMENTOS de um plano de `k` elementos por pixel.
fn elems(
    area: Option<crate::compositor::Region>,
    k: usize,
) -> Option<(usize, usize, usize, usize)> {
    area.map(|r| {
        let (x, y) = (r.x as usize, r.y as usize);
        (x * k, y, (x + r.w as usize) * k, y + r.h as usize)
    })
}

/// **A porta do RELEVO (altura)** — o fork mais a captura no journal daquela CAMADA.
///
/// Três portas e não uma genérica porque o journal é **tipado** (`f32` / `u8` / `[u8; 7]`) e keyed por
/// camada: quem escreve tem de dizer as duas coisas, e dizer é o que separa esta porta da genérica.
pub(super) fn fork_heights<'a>(
    arc: &'a mut Arc<Vec<f32>>,
    win: &WriteState,
    layer: crate::layers::LayerId,
    size: (u32, u32),
    area: Option<crate::compositor::Region>,
) -> &'a mut Vec<f32> {
    if arc.len() == (size.0 as usize) * (size.1 as usize) {
        win.capture_heights(layer, arc, size.0 as usize, elems(area, 1));
    } else {
        // O plano ainda não existe (1ª pincelada da camada) ou tem outra forma. **Isso não é o mesmo que
        // o journal estar incompleto** — um plano que não existia não tem *antes* a descrever, e o motor
        // de delta já chama isso de `OnlyAfter`. Quem decide qual das duas é `ReliefJournals::note_absent`.
        win.note_absent_relief(layer, crate::undo::window::ReliefPlane::Heights);
    }
    open_access(win);
    fork_par_raw(arc)
}

/// Ver [`fork_heights`].
pub(super) fn fork_covers<'a>(
    arc: &'a mut Arc<Vec<u8>>,
    win: &WriteState,
    layer: crate::layers::LayerId,
    size: (u32, u32),
    area: Option<crate::compositor::Region>,
) -> &'a mut Vec<u8> {
    if arc.len() == (size.0 as usize) * (size.1 as usize) {
        win.capture_covers(layer, arc, size.0 as usize, elems(area, 1));
    } else {
        // Ver o `else` de [`fork_heights`].
        win.note_absent_relief(layer, crate::undo::window::ReliefPlane::Covers);
    }
    open_access(win);
    fork_par_raw(arc)
}

/// Ver [`fork_heights`].
pub(super) fn fork_mats<'a>(
    arc: &'a mut Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    win: &WriteState,
    layer: crate::layers::LayerId,
    size: (u32, u32),
    area: Option<crate::compositor::Region>,
) -> &'a mut Vec<ph2d_painter_brush::material::MaterialBytes> {
    if arc.len() == (size.0 as usize) * (size.1 as usize) {
        win.capture_mats(layer, arc, size.0 as usize, elems(area, 1));
    } else {
        // Ver o `else` de [`fork_heights`].
        win.note_absent_relief(layer, crate::undo::window::ReliefPlane::Mats);
    }
    open_access(win);
    fork_par_raw(arc)
}

/// **A porta do CANVAS** — o fork, mais a captura dos bytes velhos no journal do passo.
///
/// Existe separada do [`fork_par`] genérico porque o journal precisa saber **de que plano** são os
/// bytes, e o canvas é o único plano com dono único no tool (os de relevo são mapas por camada). É
/// também a porta que o arch-gate mira: um sítio novo que chame o `fork_par` cru para o canvas fica
/// vermelho.
///
/// ⚠️ **`area` é a região que este sítio vai escrever, ou `None` quando ele não sabe** — e as duas
/// respostas são corretas, com preços diferentes. `None` captura o plano INTEIRO: **67,11 MB a 4096²**,
/// exatamente `n × 4` (doc 28 §7), o número que tornava a troca do S3 *lateral em vez de positiva*. Uma
/// região é o que o journal precisa reter para valer a pena.
///
/// ⚠️ **Superconjunto é seguro; subconjunto perde a edição em silêncio.** Quem passa uma região promete
/// apenas *conter* o que escreveu — os sítios de depósito usam
/// [`super::region::dabs_bounds`], que soma a footprint máxima de cada dab
/// **antes** do laço. É por isso que a lista de dabs que chega aqui tem de ser a FINAL: o Tiling e a
/// Symmetry expandem cópias (`tiling::tiled_dabs`), e elas estão na lista.
pub(super) fn fork_canvas<'a>(
    arc: &'a mut Arc<Vec<u8>>,
    win: &WriteState,
    width_px: u32,
    area: Option<crate::compositor::Region>,
) -> &'a mut Vec<u8> {
    win.capture_canvas(arc, width_px as usize * 4, elems(area, 4));
    open_access(win);
    fork_par_raw(arc)
}

/// **A porta da TROCA DE PLANO** — o campo `canvas_rgba` passa a hospedar, por um trecho, um plano
/// que **não é a tela**.
///
/// Dois sítios fazem isso, e pelo mesmo motivo: pintar por todo o pipeline de stamp sem tocar a tela.
/// A **máscara** troca o seu scratch para dentro do campo; o **gate de proteção** troca o plano
/// `free`. Enquanto a troca está de pé, um `fork_canvas` captura bytes do plano ERRADO — e como *a
/// primeira captura de cada tile é a que vale*, a poluição é **permanente**: a projeção que escreve a
/// tela logo depois encontra o tile já tomado, não recaptura, e o journal jura que a tela começou o
/// passo com os bytes do scratch. Foi exatamente isso que o censo do degrau 1 pegou (11 dos 12).
///
/// A cura é um contador de profundidade, não um guard com `Drop`: os dois sítios seguram `&mut self`
/// por dentro do trecho trocado, e um `Drop` estenderia o empréstimo até o fim do escopo (os 14
/// `E0499` que o S1 já mediu). Cada chamada é **uma** troca, então a paridade das chamadas é a
/// própria paridade das trocas.
pub(crate) fn swap_canvas_plane(
    canvas: &mut Arc<Vec<u8>>,
    other: &mut Arc<Vec<u8>>,
    w: &WriteState,
) {
    std::mem::swap(canvas, other);
    w.toggle_foreign_plane();
}

/// **A porta da SUBSTITUIÇÃO** — o plano inteiro é trocado por outro (Fill, crop, resize, o Reset do
/// warp, um bind de documento), em vez de escrito no lugar.
///
/// Um fork não tem o que capturar aqui: não há escrita incremental, o plano simplesmente deixa de
/// existir. O journal tem de guardar o plano velho **inteiro** antes de ele ir embora, senão o passo
/// perde tudo que não foi capturado até então — e perde em silêncio, que é o modo de falha que o
/// `diff_window` documenta e teme.
///
/// ⚠️ **Custa exatamente o que o fork custa hoje**, então nunca é regressão; e é debug-only enquanto
/// o journal for rede de verificação.
///
/// ⚠️ Uma substituição que muda a FORMA (crop/resize) é recusada pelo journal (o stride não mede o
/// plano velho) — e é correto: um plano de outra forma já força `Whole` no motor de delta, então não
/// há janela a preservar. Fazer o journal *saber* que não sabe é o próximo degrau.
impl crate::tool::PainterTool {
    pub(crate) fn replace_canvas(&mut self, new: Arc<Vec<u8>>) {
        let stride = self.source_size.0 as usize * 4;
        self.undo
            .write_state
            .capture_canvas(&self.canvas_rgba, stride, None);
        self.canvas_rgba = new;
    }
}

/// O fork cru — a metade de POSSE, sem a de declaração. Privado: quem escreve passa pelo guard.
fn fork_par_raw<T>(arc: &mut Arc<Vec<T>>) -> &mut Vec<T>
where
    T: Copy + Send + Sync,
{
    // ⚠️ **`strong_count`, NUNCA `Arc::get_mut`** — e a diferença custou uma regressão de 4× no Wet
    // Paint antes de ser vista.
    //
    // `get_mut` devolve `None` se existir **qualquer `Weak`**; `make_mut` só COPIA se existir outro
    // **strong** (com apenas `Weak` vivo ele *move* o valor, que é o que a frente V mediu em 0,0000 ms).
    // Perguntar pelo `get_mut` é portanto fazer uma pergunta MAIS ESTRITA que a do copiador — e o guard
    // de identidade do Wet Paint é precisamente um `Weak`, então esta função passou a copiar o canvas
    // inteiro **a cada movimento do mouse** com o `make_mut` logo abaixo movendo-o de graça.
    //
    // A pergunta certa é a que o copiador faz: *há outro dono FORTE?* Ela é um palpite sobre HOW, nunca
    // sobre WHETHER — quem decide de fato continua sendo o `make_mut` da última linha.
    if Arc::strong_count(arc) > 1 && worth_parallel::<T>(arc.len()) {
        *arc = Arc::new(crate::plane_copy::par_clone(arc));
    }
    // Now either uniquely owned (we just replaced it) or small/unshared — so this never copies twice.
    Arc::make_mut(arc)
}

#[cfg(test)]
#[path = "plane_fork_tests.rs"]
mod tests;
