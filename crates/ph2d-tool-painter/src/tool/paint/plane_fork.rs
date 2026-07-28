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
        // O plano ainda não existe (1ª pincelada da camada) ou tem outra forma: não há "antes" que o
        // journal possa descrever, e afirmar que descreve seria pior que não capturar.
        win.note_untracked_write();
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
        // O plano ainda não existe (1ª pincelada da camada) ou tem outra forma: não há "antes" que o
        // journal possa descrever, e afirmar que descreve seria pior que não capturar.
        win.note_untracked_write();
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
        // O plano ainda não existe (1ª pincelada da camada) ou tem outra forma: não há "antes" que o
        // journal possa descrever, e afirmar que descreve seria pior que não capturar.
        win.note_untracked_write();
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
/// ⚠️ **Captura o plano INTEIRO** (`None`), porque nenhum sítio conhece a sua região no momento do
/// fork — os laços de dab acumulam o `touched` *enquanto* escrevem. Isso é correto por construção (a
/// primeira captura de cada tile é a que vale) e, sendo debug-only, não custa nada em release. Quando
/// os sítios quentes aprenderem a passar a região, esta chamada é o lugar onde ela entra.
pub(super) fn fork_canvas<'a>(
    arc: &'a mut Arc<Vec<u8>>,
    win: &WriteState,
    width_px: u32,
) -> &'a mut Vec<u8> {
    win.capture_canvas(arc, width_px as usize * 4, None);
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
mod tests {
    use super::*;
    use crate::plane_copy::PAR_MIN_BYTES;
    use rayon::prelude::*;

    /// **A fork is a copy, and a copy has one right answer.** The parallel path must produce exactly what
    /// `Arc::make_mut` produces — value-identical, and uniquely owned afterwards so the caller may write.
    ///
    /// Run over a length that clears `PAR_MIN` (so the parallel path actually executes — the trap the
    /// ADR-0120 lesson names: an optimisation nobody exercises is green code that never runs) and one that
    /// does not.
    #[test]
    fn a_parallel_fork_is_byte_identical_to_the_serial_one() {
        for n in [PAR_MIN_BYTES / size_of::<f32>() + 1_000, 64] {
            let src: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect();

            let mut a = Arc::new(src.clone());
            let keep_a = Arc::clone(&a); // force the second-owner path
            let forked = fork_par(&mut a, &WriteState::default()).clone();

            let mut b = Arc::new(src.clone());
            let keep_b = Arc::clone(&b);
            let expected = Arc::make_mut(&mut b).clone();

            assert_eq!(forked, expected, "n = {n}");
            assert_eq!(forked, src, "n = {n}: the fork changed the contents");
            // The originals are untouched — the whole point of freezing one.
            assert_eq!(*keep_a, src);
            assert_eq!(*keep_b, src);
        }
    }

    /// **Um `Weak` vivo NÃO é um dono — e perguntar isso errado custou 4× no Wet Paint.**
    ///
    /// `Arc::get_mut` devolve `None` na presença de qualquer `Weak`; `Arc::make_mut` só **copia** com
    /// outro **strong** (com só `Weak` ele *move* o valor). Enquanto esta função perguntava pelo
    /// `get_mut`, ela copiava o plano inteiro sempre que alguém segurasse um `Weak` — e o guard de
    /// identidade do Wet Paint é exatamente isso (frente V, doc 28 §5.12), então o composite pagava uma
    /// cópia do documento **por movimento do mouse** enquanto o `make_mut` seguinte o movia de graça.
    ///
    /// ⚠️ O sintoma foi o gate de RAZÃO do move molhado voltando a **4,77×** — nunca uma falha de
    /// comportamento, porque as duas rotas dão os mesmos bytes. Este gate afirma a propriedade direto,
    /// para o defeito não depender de um relógio para aparecer.
    #[test]
    fn a_live_weak_is_not_an_owner_and_does_not_trigger_a_copy() {
        let mut a: Arc<Vec<f32>> = Arc::new(vec![1.0; PAR_MIN_BYTES / size_of::<f32>() + 1_000]);
        let watcher = Arc::downgrade(&a); // o guard de identidade do Wet Paint, em miniatura
        let before = a.as_ptr();
        let got = fork_par(&mut a, &WriteState::default());
        assert_eq!(
            got.as_ptr(),
            before,
            "um `Weak` vivo fez o fork COPIAR o plano — a pergunta e `strong_count`, nao `get_mut`"
        );
        // E o `Weak` foi desassociado, exatamente como o `Arc::make_mut` faria: quem o segurava tem de
        // enxergar que o buffer mudou de dono (e o guard do Wet Paint depende disso para se re-armar).
        assert!(watcher.upgrade().is_none() || Arc::strong_count(&a) == 1);
    }

    /// **With no second owner it does not allocate at all.** The common case inside a stroke is that the
    /// plane was already forked by an earlier dab; paying a copy per dab instead of per stroke is the
    /// regression this guards.
    #[test]
    fn an_unshared_plane_is_not_copied() {
        let mut a: Arc<Vec<f32>> = Arc::new(vec![1.0; PAR_MIN_BYTES / size_of::<f32>() + 1_000]);
        let before = a.as_ptr();
        let got = fork_par(&mut a, &WriteState::default());
        assert_eq!(got.as_ptr(), before, "an unshared plane was copied anyway");
    }

    /// **The fast path has to be proven by the clock, because nothing else can see it.**
    ///
    /// The two gates above cannot tell the branches apart, and that is not an oversight — it is the
    /// point: a fork is a copy, so the parallel path is *semantically identical* by construction. There
    /// is no value, no pointer and no refcount that differs (`Arc::make_mut` also leaves a uniquely
    /// owned buffer at a fresh address). A behavioural gate here would be the serial path measured
    /// against itself and green forever — the trap ADR-0120 documented and ADR-0124 then hit a second
    /// time, hiding in its own undo oracle.
    ///
    /// So the claim is timed, and asserted as a **RATIO** rather than a wall-clock bar: `ci-test`
    /// compiles at `opt-level=1` and this machine is documented as drifting ~3× across a session, so an
    /// absolute millisecond bar would be measuring the profile and the weather. The ratio survives both.
    ///
    /// Measured at 4096² (67 MB): serial 10,88 ms, parallel 3,34 ms — 3,3×. The bar is set well under
    /// that so a loaded machine cannot flake it, while a fork that silently stopped being parallel
    /// (delegating straight to `Arc::make_mut`) lands at 1,0× and fails.
    #[test]
    #[ignore = "perf measurement — run with --release --ignored"]
    fn the_parallel_fork_is_actually_faster_than_the_serial_one() {
        use std::time::Instant;
        const N: usize = 4096 * 4096;
        let src: Arc<Vec<f32>> = Arc::new(vec![0.5; N]);
        let best = |mut f: Box<dyn FnMut() -> f64>| (0..3).map(|_| f()).fold(f64::MAX, f64::min);

        let s = Arc::clone(&src);
        let serial = best(Box::new(move || {
            let mut a = Arc::clone(&s);
            let _keep = Arc::clone(&s);
            let t0 = Instant::now();
            let m = Arc::make_mut(&mut a);
            std::hint::black_box(&m[0]);
            t0.elapsed().as_secs_f64() * 1000.0
        }));
        let p = Arc::clone(&src);
        let parallel = best(Box::new(move || {
            let mut a = Arc::clone(&p);
            let _keep = Arc::clone(&p);
            let t0 = Instant::now();
            let m = fork_par(&mut a, &WriteState::default());
            std::hint::black_box(&m[0]);
            t0.elapsed().as_secs_f64() * 1000.0
        }));
        eprintln!(
            "[fork] 4096² plane: serial {serial:.2} ms · parallel {parallel:.2} ms · \
             {:.1}×",
            serial / parallel
        );
        assert!(
            serial / parallel > 1.5,
            "the parallel fork bought {:.1}× over `Arc::make_mut` (serial {serial:.2} ms, parallel \
             {parallel:.2} ms). Below 1.5× the fast path is not running — check that `PAR_MIN` still \
             sits under a canvas-sized plane",
            serial / parallel
        );
    }

    /// **A porta do pen-down** — que o depósito de PIGMENTO atravesse esta função, e não o `make_mut` cru.
    ///
    /// ⚠️ Este gate é arquitetural porque o defeito é **invisível ao comportamento**: as duas rotas produzem
    /// os mesmos bytes (é o que o gate acima prova), então trocar uma pela outra deixa a suíte inteira verde e
    /// custa **três vezes o tempo** no gesto que o artista mais sente. Medido no pen-down a 4096², pincel
    /// digital: **10,3 ms com o `make_mut` cru contra 3,6 ms por aqui** (`measure_impasto_cost::
    /// the_first_stroke_latency`); o pen-down do impasto, 18,6 -> 12,2.
    ///
    /// ⚠️ **O escopo deixou de ser o `stamp_cache` (2026-07-26).** A doc antiga dizia que o pen-down era
    /// *"o único sítio onde o `Arc` do canvas tem um segundo dono garantido"* — e isso é **falso**:
    /// medido (`measure_stroke_owners`), em repouso o canvas tem **dois donos**, porque o `cursor` do
    /// histórico é um dono **permanente**. Logo a PRIMEIRA escrita de **todo** gesto forka, e os outros
    /// 23 sítios (fill, smear, blur, clone, seleção, warp, máscara, inpaint, aquarela, e o composite do
    /// Wet Paint, que roda a cada TICK) pagavam a cópia **serial**. O gate irmão abaixo cobre a crate.
    #[test]
    fn the_pigment_deposit_forks_the_canvas_in_parallel() {
        let src = include_str!("stamp_cache.rs");
        // Controle positivo: o alvo tem de EXISTIR, senão o gate passa por não achar nada (a falha que o
        // `the_shape_slot_goes_through_the_shape_door` do Flow pegou em si mesmo).
        let through = src.matches("fork_canvas(").count();
        assert!(
            through >= 5,
            "controle: o stamp_cache tem de escrever o canvas pela porta paralela ({through} sitios)"
        );
        let raw = src.matches("Arc::make_mut(&mut self.canvas_rgba)").count();
        assert_eq!(
            raw, 0,
            "o deposito de pigmento nao pode forkar o canvas SERIALMENTE: {raw} sitio(s) com `make_mut` cru \
             (as duas rotas dao os mesmos bytes, entao isto nao acende em teste de comportamento nenhum \
             -- custa 3x o tempo do pen-down e passa despercebido)"
        );
    }

    /// **As portas NOMEADAS ensinam o journal; a genérica declara que não sabe.**
    ///
    /// O relevo é um mapa por camada e três tipos de elemento, então a captura tem de saber **qual** —
    /// e é essa exigência que separa as duas famílias de porta. Quem passa pela genérica não fica
    /// errado, fica **lento**: o journal se declara incompleto e o commit deriva como sempre.
    ///
    /// ⚠️ Mutação que sangra: tirar o `capture_heights` do [`fork_heights`] (o journal não sabe do byte
    /// velho) · tirar o `note_untracked_write` do [`fork_par`] (o journal jura descrever um passo que
    /// tem escrita fora dele).
    #[test]
    fn the_named_relief_doors_teach_the_journal_and_the_generic_one_does_not() {
        const W: u32 = 64;
        let n = (W as usize) * (W as usize);
        let layer = crate::layers::LayerId(7);

        let w = WriteState::default();
        let mut h: Arc<Vec<f32>> = Arc::new((0..n).map(|i| i as f32).collect());
        let keep = Arc::clone(&h); // um segundo dono, como o `stroke_undo` de um passo real
        fork_heights(&mut h, &w, layer, (W, W), None)[10] = -1.0;

        assert_eq!(w.relief_state(), "DESCREVE");
        assert_eq!(
            w.heights_before(layer, 10),
            Some(10.0),
            "o journal tem de guardar o valor VELHO, nao o que acabou de ser escrito"
        );
        assert_eq!(
            keep[10], 10.0,
            "controle: o fork preservou o plano congelado"
        );
        assert!(
            w.heights_before(crate::layers::LayerId(8), 10).is_none(),
            "o journal nao pode responder por OUTRA camada"
        );

        let w2 = WriteState::default();
        let mut h2: Arc<Vec<f32>> = Arc::new(vec![1.0; n]);
        let _ = fork_par(&mut h2, &w2);
        assert_eq!(
            w2.relief_state(),
            "INCOMPLETO",
            "a porta generica nao sabe de que plano e — o journal tem de se declarar incompleto"
        );
    }

    /// **O FOLD escreve o relevo pelas portas nomeadas** — arch-gate, porque o defeito é invisível ao
    /// comportamento: as duas rotas produzem os mesmos bytes, e a diferença é só se o journal aprende.
    ///
    /// O fold é a metade cara do pen-up (9,25 ms de fork nos três planos) e é o alvo do S3; um sítio
    /// que voltasse ao `fork_par` deixaria o passo INCOMPLETO e o commit derivando, com toda a suíte
    /// verde.
    #[test]
    fn the_relief_fold_goes_through_the_named_doors() {
        let src = include_str!("impasto_live.rs");
        for door in ["fork_heights(", "fork_covers(", "fork_mats("] {
            assert!(
                src.contains(door),
                "controle: o fold tem de escrever o relevo pela porta {door}"
            );
        }
        let generic = src.matches("plane_fork::fork_par(").count();
        assert_eq!(
            generic, 0,
            "o fold tem {generic} escrita(s) de relevo pela porta GENERICA: o journal nao aprende de \
             que plano sao os bytes e o passo inteiro fica INCOMPLETO"
        );
    }

    /// **De que é feito um fork, por TIPO e por TAMANHO** — a medição que mostrou que
    /// `par_iter().copied().collect()` **não é** uniformemente melhor que um memcpy.
    #[test]
    #[ignore = "medicao — rode com --release --ignored"]
    fn what_a_fork_costs_by_element_type_and_size() {
        use std::time::Instant;
        fn best(mut f: impl FnMut() -> f64) -> f64 {
            (0..5).map(|_| f()).fold(f64::MAX, f64::min)
        }
        fn go<T: Copy + Send + Sync + Default + PartialEq>(name: &str, n: usize) {
            let src: Arc<Vec<T>> = Arc::new(vec![T::default(); n]);
            let a = best(|| {
                let s = Arc::clone(&src);
                let t0 = Instant::now();
                let v: Vec<T> = (*s).clone();
                let dt = t0.elapsed().as_secs_f64() * 1000.0;
                std::hint::black_box(v.len());
                dt
            });
            let b = best(|| {
                let s = Arc::clone(&src);
                let t0 = Instant::now();
                let v: Vec<T> = s.par_iter().copied().collect();
                let dt = t0.elapsed().as_secs_f64() * 1000.0;
                std::hint::black_box(v.len());
                dt
            });
            let c = best(|| {
                let s = Arc::clone(&src);
                let t0 = Instant::now();
                let mut v: Vec<T> = vec![T::default(); s.len()];
                v.par_chunks_mut(1 << 16)
                    .zip(s.par_chunks(1 << 16))
                    .for_each(|(d, x)| d.copy_from_slice(x));
                let dt = t0.elapsed().as_secs_f64() * 1000.0;
                std::hint::black_box(v.len());
                dt
            });
            let mb = (n * std::mem::size_of::<T>()) as f64 / (1024.0 * 1024.0);
            println!(
                "{name:<16} {mb:>7.0} MB  clone {a:>7.3}  par_collect {b:>7.3}  par_memcpy {c:>7.3}"
            );
        }
        println!();
        for side in [2048usize, 4096] {
            let n = side * side;
            println!("-- tela {side}x{side} --");
            go::<u8>("canvas rgba u8", n * 4);
            go::<u8>("covers u8", n);
            go::<f32>("heights f32", n);
            go::<[u8; 7]>("mats [u8;7]", n);
        }
        println!();
    }

    /// **NENHUM sítio da crate forka o canvas serialmente** — o irmão de escopo largo do gate acima.
    ///
    /// ⚠️ Um gate por-arquivo protege o arquivo que alguém lembrou de listar. Este varre `tool/paint/**`
    /// inteiro, então o sítio 24 nasce coberto — que é exatamente como os 23 nasceram descobertos.
    ///
    /// ⚠️ **O literal é só o NOME da porta, e isso é deliberado.** A primeira versão casava a chamada
    /// inteira (`fork_canvas(&mut self.canvas_rgba, &self.undo.write_state, self.source_size.0)`) e
    /// morreu no `cargo fmt`, que quebra três argumentos em quatro linhas: o gate passou a achar ZERO
    /// e só o controle positivo o salvou de virar verde-sobre-nada. Um gate ancorado em LAYOUT é um
    /// proxy que expira; o que ele tem de afirmar é a PORTA.
    ///
    /// **Prosa é isenta, código não.** O literal aparece em doc-comments explicando a diferença entre as
    /// duas rotas, e um gate que os proibisse mandaria apagar a explicação; ele conta só linhas de código.
    /// Arquivos de teste e de medição também são isentos — eles CHAMAM `make_mut` de propósito, para
    /// medir a rota lenta contra a rápida.
    #[test]
    fn no_site_in_the_crate_forks_the_canvas_serially() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tool/paint");
        let mut scanned = 0usize;
        let mut through = 0usize;
        let mut offenders: Vec<String> = Vec::new();
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).expect("src/tool/paint existe") {
                let p = e.expect("entrada legivel").path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                if !name.ends_with(".rs")
                    || name.contains("test")
                    || name.starts_with("measure_")
                    || name == "plane_fork.rs"
                {
                    continue; // gates e sondas chamam a rota lenta DE PROPOSITO
                }
                let src = std::fs::read_to_string(&p).expect("fonte legivel");
                scanned += 1;
                through += src.matches("fork_canvas(").count();
                for (i, line) in src.lines().enumerate() {
                    let t = line.trim_start();
                    if t.starts_with("//") {
                        continue; // prosa: explicar a rota lenta e o que os docs FAZEM
                    }
                    if t.contains("Arc::make_mut(&mut self.canvas_rgba)") {
                        offenders.push(format!("{name}:{}", i + 1));
                    }
                }
            }
        }
        // Controle positivo, nas DUAS pontas: sem ele o gate passa por não ter achado arquivo nenhum.
        assert!(scanned > 20, "controle: so {scanned} arquivos varridos");
        assert!(
            through >= 20,
            "controle: so {through} escritas de canvas pela porta paralela — o scanner esta cego"
        );
        assert!(
            offenders.is_empty(),
            "estes sitios forkam o canvas SERIALMENTE (a primeira escrita de todo gesto copia a tela \
             inteira, ~3x mais devagar, e as duas rotas dao os mesmos bytes — nenhum teste de \
             comportamento acende): {offenders:?}"
        );
    }
}
