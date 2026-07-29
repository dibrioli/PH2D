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
    /// **A PROVENIÊNCIA do journal** — o valor do contador de escritas quando ele foi zerado, ou `None`
    /// se ninguém abriu passo desde o último commit.
    ///
    /// ⚠️ É ela que torna a migração segura, e pela mesma política do `undeclared`: o journal só serve
    /// de lado `before` de um passo se tiver sido zerado **exatamente** onde aquele passo começou. Um
    /// sítio que ainda não abre passo deixa o journal ancorado num ponto mais VELHO, a pergunta
    /// [`Self::journal_describes_step_at`] responde `false`, e o commit cai no caminho de sempre —
    /// *lento, nunca errado*. Sem ela, um sítio esquecido daria o lado `before` de um passado que não é
    /// o do passo, e o undo devolveria pixels que nunca existiram.
    #[cfg(any(test, debug_assertions))]
    journal_since: std::cell::Cell<Option<u64>>,
    /// Os bytes velhos dos três planos de RELEVO — ver [`ReliefJournals`].
    #[cfg(any(test, debug_assertions))]
    relief: std::cell::RefCell<ReliefJournals>,
}

/// **Os journals dos três planos de relevo, mais de QUE camada eles falam.**
///
/// ⚠️ O relevo não é um plano, é um **mapa por camada** (`heights`/`covers`/`mats` são
/// `BTreeMap<LayerId, …>`), e dois planos de camadas diferentes têm a mesma FORMA. O `arm` do
/// [`TileJournal`](crate::undo::journal::TileJournal) descarta a grade quando as dimensões mudam, mas
/// aqui elas **não** mudam: um passo que tocasse o relevo de duas camadas misturaria os bytes das duas
/// no mesmo índice, com confiança e em silêncio. Daí a camada ser lembrada, e um segundo dono marcar o
/// journal como **misturado** em vez de tentar reconciliar.
///
/// ⚠️ **`incomplete` é a mesma política do contador de acessos não-declarados do S1**: uma escrita que
/// passa pela porta GENÉRICA (`fork_par`) não sabe dizer de que plano é, então o journal do relevo não
/// pode afirmar que descreve o passo. Ele responde `false`, o commit deriva como sempre, e cada sítio
/// que aprende a sua porta nomeada passa a pagar só a região — *lento, nunca errado*.
#[cfg(any(test, debug_assertions))]
#[derive(Debug, Default)]
struct ReliefJournals {
    layer: Option<crate::layers::LayerId>,
    mixed: bool,
    incomplete: bool,
    /// **Este plano foi escrito quando ainda não tinha forma de canvas** — a 1ª pincelada de uma camada,
    /// ou um plano de outro documento.
    ///
    /// ⚠️ **Não é o mesmo que incompleto, e conflatá-los custava 202 passos da suíte.** Um plano que não
    /// existia no começo do passo **não tem *antes* a descrever**; o motor de delta já chama isso de
    /// `OnlyAfter` e nunca consulta janela nenhuma. Incompleto é a outra coisa: alguém escreveu bytes
    /// cujo valor velho o journal não guardou.
    ///
    /// ⚠️ **A promoção só é honesta porque a rede a confere** (`audit_relief_absence`): *journal vazio
    /// para este plano ⟺ o `before` do passo não o tem em forma de canvas*. Sem esse ⟺, marcar estes
    /// passos como DESCREVE seria uma afirmação que o oráculo **não pode contradizer** — a forma exata
    /// do gate vazio da §5.23.
    absent: [bool; 3],
    heights: crate::undo::journal::TileJournal<f32>,
    covers: crate::undo::journal::TileJournal<u8>,
    mats: crate::undo::journal::TileJournal<ph2d_painter_brush::material::MaterialBytes>,
}

#[cfg(any(test, debug_assertions))]
impl ReliefJournals {
    /// Uma captura chegou para `l`. Se ela é de outra camada, o que estava guardado descreve OUTRO
    /// plano — e o journal se declara misturado em vez de responder um byte do lugar errado.
    fn note_layer(&mut self, l: crate::layers::LayerId) {
        match self.layer {
            None => self.layer = Some(l),
            Some(cur) if cur == l => {}
            Some(_) => {
                self.mixed = true;
                self.heights.reset();
                self.covers.reset();
                self.mats.reset();
            }
        }
    }

    /// **Uma escrita chegou a um plano SEM forma de canvas.** Duas coisas cabem aqui e só uma é dívida.
    ///
    /// Se o journal daquele plano ainda está **vazio**, o plano não existia no começo do passo: não há
    /// *antes*, e não descrever o que não existe é a resposta certa (o `absent` registra o fato para a
    /// rede o conferir). Se ele **já tem tiles**, o plano ERA canvas-shaped mais cedo neste passo e
    /// perdeu a forma no meio — aí a escrita atual é genuinamente indescritível, e o passo inteiro se
    /// declara incompleto, como antes.
    fn note_absent(&mut self, l: crate::layers::LayerId, which: ReliefPlane) {
        let empty = match which {
            ReliefPlane::Heights => self.heights.is_empty(),
            ReliefPlane::Covers => self.covers.is_empty(),
            ReliefPlane::Mats => self.mats.is_empty(),
        };
        if !empty {
            self.incomplete = true;
            return;
        }
        // O passo TOCOU o relevo desta camada (ele criou o plano), então a camada é registrada: sem
        // isso `speaks_for` recusaria, e um passo cujo único relevo é um plano novo cairia em
        // SEM-RELEVO — que descreve outra coisa.
        self.note_layer(l);
        self.absent[which as usize] = true;
    }

    /// `true` se estes journals podem responder pela camada `l`.
    fn speaks_for(&self, l: crate::layers::LayerId) -> bool {
        !self.mixed && !self.incomplete && self.layer == Some(l)
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
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
    /// **Os bytes que os quatro journals de fato retêm neste passo** — canvas + os três de relevo.
    ///
    /// É o número que decide se o journal pode sair do `cfg(debug)` para o release: a troca do S3
    /// substitui um **fork do plano inteiro** por uma **captura da região**, e ela só é positiva se a
    /// região couber onde o plano não cabia. `crate::tool::paint::measure_journal_cost` a mede num traço
    /// real; aqui está a porta que ela lê.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn journal_heap_bytes(&self) -> usize {
        let r = self.relief.borrow();
        self.canvas.borrow().heap_bytes()
            + r.heights.heap_bytes()
            + r.covers.heap_bytes()
            + r.mats.heap_bytes()
    }

    #[cfg(any(test, debug_assertions))]
    pub(crate) fn canvas_before(&self, i: usize) -> Option<u8> {
        self.canvas.borrow().get(i)
    }

    /// O passo fechou: o journal esquece o que capturou.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn reset_journal(&self) {
        self.canvas.borrow_mut().reset();
        self.relief.borrow_mut().reset();
        self.journal_since.set(None);
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn reset_journal(&self) {}

    /// **Um passo de undo COMEÇA aqui** — o journal esquece o intervalo anterior e passa a descrever
    /// exatamente o que vier a partir de `writes`.
    ///
    /// ⚠️ A distinção que isto instala é o que separa o journal-como-rede do journal-como-FONTE. Ancorado
    /// no último **commit**, ele mistura as escritas estrangeiras (o escorrido do Wet Paint) com as do
    /// passo, e nos tiles que as duas tocam a primeira captura é a da gota — o lado `before` sairia do
    /// passo *anterior*. Ancorado no **passo**, ele responde a pergunta certa.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn begin_step(&self, writes: u64) {
        self.canvas.borrow_mut().reset();
        self.relief.borrow_mut().reset();
        self.journal_since.set(Some(writes));
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn begin_step(&self, _writes: u64) {}

    /// **O journal descreve o passo que começou em `writes`?** — a pergunta que autoriza usá-lo como
    /// lado `before`. Ver [`Self::journal_since`].
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn journal_describes_step_at(&self, writes: u64) -> bool {
        self.journal_since.get() == Some(writes)
    }

    /// **Captura o relevo antes de ele ser escrito** — um por plano, porque cada um tem um tipo de
    /// elemento e o journal é tipado. `area` em ELEMENTOS (o relevo carrega um por pixel).
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn capture_heights(
        &self,
        layer: crate::layers::LayerId,
        buf: &[f32],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        let mut r = self.relief.borrow_mut();
        r.note_layer(layer);
        r.heights.capture(buf, stride, area);
    }

    /// Ver [`Self::capture_heights`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn capture_covers(
        &self,
        layer: crate::layers::LayerId,
        buf: &[u8],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        let mut r = self.relief.borrow_mut();
        r.note_layer(layer);
        r.covers.capture(buf, stride, area);
    }

    /// Ver [`Self::capture_heights`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn capture_mats(
        &self,
        layer: crate::layers::LayerId,
        buf: &[ph2d_painter_brush::material::MaterialBytes],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        let mut r = self.relief.borrow_mut();
        r.note_layer(layer);
        r.mats.capture(buf, stride, area);
    }

    /// **Uma escrita que os journals de relevo não conseguem contabilizar** — a porta genérica (que não
    /// sabe de que plano é) ou um plano cuja forma eles não descrevem. Ver [`ReliefJournals::incomplete`].
    ///
    /// ⚠️ **`cfg(test)`, e ela SEGUE a porta genérica** — era a metade de declaração dela, e o
    /// `fork_par` virou referência congelada quando os dez sítios migraram (§5.29). O doc que morava
    /// aqui dizia *"não é gateado por `cfg` porque a porta genérica o chama em qualquer perfil"*, e essa
    /// frase morreu junto com a porta: em produção o único caminho até `incomplete` é o
    /// [`ReliefJournals::note_absent`], quando um plano que **existia** perde a forma no meio do passo.
    #[cfg(test)]
    pub(crate) fn note_untracked_write(&self) {
        self.relief.borrow_mut().incomplete = true;
    }
}

/// Qual dos três planos de relevo — o índice que [`ReliefJournals::absent`] usa.
///
/// Existe porque as três portas fazem a MESMA pergunta sobre planos de tipos diferentes, e um enum é o
/// que deixa a resposta ser dada uma vez em vez de três.
///
/// ⚠️ **NÃO é gateado por `cfg`**, e a tentativa de gateá-lo não compilou em `--release`: ele atravessa a
/// assinatura de [`WriteState::note_absent_relief`], que as portas chamam em **qualquer** perfil. Um tipo
/// que só existe em debug daria duas formas às portas — o mesmo motivo do `note_untracked_write`.
/// ⚠️ E `cargo clippy -p` roda em **debug**: só o check de release pega isto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReliefPlane {
    Heights = 0,
    Covers = 1,
    Mats = 2,
}

impl WriteState {
    /// **O plano `which` da camada `layer` foi escrito sem ter forma de canvas.** Ver
    /// [`ReliefJournals::note_absent`], que decide qual das duas coisas isso significa.
    ///
    /// ⚠️ Não é gateado por `cfg` pela mesma razão do [`Self::note_untracked_write`]: as três portas o
    /// chamam em qualquer perfil, e um canal que só existisse em debug daria duas formas às portas.
    pub(crate) fn note_absent_relief(&self, layer: crate::layers::LayerId, which: ReliefPlane) {
        #[cfg(any(test, debug_assertions))]
        {
            self.relief.borrow_mut().note_absent(layer, which);
        }
        #[cfg(not(any(test, debug_assertions)))]
        {
            let _ = (layer, which);
        }
    }

    /// Quais planos de relevo este passo escreveu **sem que houvesse um antes** — o que a rede confere
    /// contra o `before` (`audit_relief_absence`). Índices de [`ReliefPlane`].
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn relief_absent(&self) -> [bool; 3] {
        self.relief.borrow().absent
    }

    /// O que os journals de relevo dizem sobre o elemento `i` da camada `layer`, ou `None` quando eles
    /// não falam por ela (mistura, plano sem nome, tile não capturado).
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn heights_before(&self, layer: crate::layers::LayerId, i: usize) -> Option<f32> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.heights.get(i)).flatten()
    }

    /// Ver [`Self::heights_before`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn covers_before(&self, layer: crate::layers::LayerId, i: usize) -> Option<u8> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.covers.get(i)).flatten()
    }

    /// Ver [`Self::heights_before`].
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn mats_before(
        &self,
        layer: crate::layers::LayerId,
        i: usize,
    ) -> Option<ph2d_painter_brush::material::MaterialBytes> {
        let r = self.relief.borrow();
        r.speaks_for(layer).then(|| r.mats.get(i)).flatten()
    }

    /// As três irmãs de release das capturas acima — no-ops, como a do canvas. Sem elas as portas
    /// nomeadas teriam de carregar o `cfg` dentro do corpo, e uma porta com duas formas é o começo de
    /// duas respostas.
    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn capture_heights(
        &self,
        _layer: crate::layers::LayerId,
        _buf: &[f32],
        _stride: usize,
        _area: Option<(usize, usize, usize, usize)>,
    ) {
    }

    /// Ver [`Self::capture_heights`].
    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn capture_covers(
        &self,
        _layer: crate::layers::LayerId,
        _buf: &[u8],
        _stride: usize,
        _area: Option<(usize, usize, usize, usize)>,
    ) {
    }

    /// Ver [`Self::capture_heights`].
    #[cfg(not(any(test, debug_assertions)))]
    #[expect(
        clippy::unused_self,
        reason = "no-op em release; o journal é rede de debug"
    )]
    pub(crate) fn capture_mats(
        &self,
        _layer: crate::layers::LayerId,
        _buf: &[ph2d_painter_brush::material::MaterialBytes],
        _stride: usize,
        _area: Option<(usize, usize, usize, usize)>,
    ) {
    }

    /// **Por que os journals de relevo não descrevem o passo** — o readout que separa *"não havia o que
    /// descrever"* de *"havia e eles não sabem"*, e só o segundo é dívida de migração.
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn relief_state(&self) -> &'static str {
        let r = self.relief.borrow();
        match () {
            () if r.mixed => "MISTURADO", // duas camadas num passo
            // ⚠️ **A causa deste estado MUDOU e a frase que morava aqui virou falsa.** Ela dizia
            // *"alguém escreveu pela porta genérica"*, e a porta genérica é `cfg(test)` desde que os
            // dez sítios de sculpt/warp migraram: em produção ela não existe. O único produtor que
            // sobrou é o `else` das três portas nomeadas — o plano não tinha forma de canvas na hora
            // da escrita (a primeira pincelada de uma camada, ou um plano de outro documento).
            //
            // ⚠️ Medido: **202 passos da suíte**, todos por essa via. Não é a mesma coisa que a antiga,
            // e a diferença decide o S3: um plano que **não existia** no começo do passo não tem
            // *antes* a descrever — o motor de delta já chama isso de `OnlyAfter`. Separar as duas é o
            // próximo degrau; enquanto elas dividem um estado, este número parece dívida e não é.
            () if r.incomplete => "INCOMPLETO",
            () if r.layer.is_none() => "SEM-RELEVO", // o passo não tocou relevo nenhum
            () => "DESCREVE",
        }
    }

    /// **Os journals de relevo descrevem o passo que começou em `writes`, para a camada `layer`?**
    #[cfg(any(test, debug_assertions))]
    #[must_use]
    pub(crate) fn relief_describes_step_at(
        &self,
        writes: u64,
        layer: crate::layers::LayerId,
    ) -> bool {
        self.journal_describes_step_at(writes) && self.relief.borrow().speaks_for(layer)
    }
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
