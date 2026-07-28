//! **A REDE DE VERIFICAÇÃO DO S3** — as perguntas que o journal tem de responder antes de virar a
//! FONTE do undo em vez de uma rede (doc 28 §7).
//!
//! Filho de [`super::runtime`] por responsabilidade, não por tamanho: o `runtime` é *o que o tool FAZ*,
//! e isto é *o que ele CONFERE sobre si mesmo*. Todas são `cfg(any(test, debug_assertions))` com irmãs
//! no-op em release, e todas só falam com `PH2D_UNDO_AUDIT=1` — porque **uma rede que varre o canvas
//! não pode viver dentro do relógio das coisas que ela observa** (§5.23: ligada por padrão ela derruba
//! dois gates de razão que medem *janela contra canvas*).
//!
//! ```text
//! PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter --lib -- --nocapture | grep S3-AUDIT
//! ```

use super::PainterTool;

impl PainterTool {
    /// **O CURSOR pode LARGAR o canvas?** — a pergunta que autoriza o último degrau do S3, medida antes
    /// de qualquer código ser escrito em cima dela.
    ///
    /// O `cursor` é um dono **permanente** do canvas (§5.20: dois donos em repouso), e `make_mut` copia
    /// com qualquer coisa acima de um — é ele, junto com o `stroke_undo`, que faz a primeira escrita de
    /// todo gesto pagar uma cópia do documento. Ele existe por dois motivos: ser a base do delta e ser o
    /// alvo da absorção. O primeiro o 3a já dissolveu (o vivo **é** o cursor em 92 de 92 undos); o
    /// segundo é o que esta rede pergunta.
    ///
    /// O cursor é o estado do **último commit**. O journal é zerado *nesse mesmo commit*
    /// ([`crate::undo::UndoController::set_cursor`]) e guarda os bytes velhos de **toda** escrita desde
    /// então. Logo, no instante em que um passo abre:
    ///
    /// ```text
    ///   cursor[i]  ==  journal.get(i).unwrap_or(vivo[i])
    /// ```
    ///
    /// Se isto vale, **o cursor não precisa ser SEGURADO — ele é reconstruível**, e a reconstrução custa
    /// zero no caso comum (journal vazio ⇒ o cursor É o vivo, pelo mesmo `Arc`).
    ///
    /// ```text
    /// PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter --lib -- --nocapture | grep cursor/
    /// ```
    #[cfg(any(test, debug_assertions))]
    pub(super) fn audit_cursor_is_reconstructible(&self) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        let Some(cursor) = self.undo.cursor_for_audit() else {
            return; // história vazia: não há cursor a reconstruir
        };
        if cursor.canvas_rgba.len() != self.canvas_rgba.len() {
            eprintln!("[S3-AUDIT] cursor/OUTRA-FORMA");
            return; // o canvas mudou de tamanho: o journal já se recusa a descrever o plano velho
        }
        let (mut differ, mut from_journal, mut first) = (0usize, 0usize, None);
        for (i, &want) in cursor.canvas_rgba.iter().enumerate() {
            let got = match self.undo.write_state.canvas_before(i) {
                Some(old) => {
                    from_journal += 1;
                    old
                }
                None => self.canvas_rgba[i],
            };
            if got != want {
                differ += 1;
                first.get_or_insert((i, got, want));
            }
        }
        eprintln!("[S3-AUDIT] cursor/RECONSTRUIDO: do-journal={from_journal} divergem={differ}");
        assert!(
            differ == 0,
            "o cursor NAO e reconstruivel de vivo+journal: {differ} byte(s) divergem, o 1o em \
             {first:?} (indice, reconstruido, cursor). Enquanto isto nao valer, o cursor tem de \
             SEGURAR o canvas — e segura-lo e o que faz a 1a escrita de todo gesto copiar o documento \
             (doc 28 §7)"
        );
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(super) fn audit_cursor_is_reconstructible(&self) {}

    /// **O RE-CENSO da premissa do S3** — *o estado vivo serve de base para o delta?* (doc 28 §7).
    ///
    /// Chamado no instante de todo undo/redo, com `PH2D_UNDO_AUDIT=1` a suíte inteira desta crate vira
    /// a rede: ela roda em debug em ~4 s e exercita fill, seleção, warp, sculpt, máscara, inpaint,
    /// clone, aquarela, Wet Paint e os shape editors, então o censo cobre **todo gesto que algum teste
    /// encena**. A medição de 2026-07-27 deu **81 chamadas · 79 concordam · 2 divergem**, e as duas
    /// exceções estão pinadas em `paint::undo_live_base_tests`.
    ///
    /// ⚠️ **`cargo test` CAPTURA o stderr de teste que passa** — sem `--nocapture` o censo sai vazio e
    /// *vazio parece concordância*. O controle positivo (imprimir também quando não diverge) é o que
    /// separou as duas coisas na primeira rodada.
    ///
    /// ```text
    /// PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT
    /// ```
    #[cfg(any(test, debug_assertions))]
    pub(super) fn audit_live_is_the_cursor(&self, tag: &str) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        let Some(cursor) = self.undo.cursor_for_audit() else {
            return;
        };
        let d = crate::undo_planes::PlaneDeltas::divergences(&self.snapshot_model(), cursor);
        eprintln!("[S3-AUDIT] {tag}: divergencias={} {d:?}", d.len());
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(super) fn audit_live_is_the_cursor(&self, _tag: &str) {}

    /// **O commit recebeu a janela, ou varreu?** — o readout que separa os dois custos possíveis dos
    /// 12,2 ms que sobraram do S2 (doc 28 §5.19): *extração* de uma janela pequena contra *varredura*
    /// dos quatro planos porque algum sítio deixou um acesso sem declarar.
    ///
    /// ⚠️ O contador é **um só para todos os planos** (uma `WriteState` no tool), então um único sítio
    /// esquecido tira a janela de TODOS eles. É por isso que a pergunta se faz aqui, no consumidor, e
    /// não por sítio: o que decide é o total.
    ///
    /// `PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT`
    /// (⚠️ o `--nocapture` é parte da medição — o `cargo test` engole o stderr de teste que passa).
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn audit_commit_window(
        &self,
        snapshot_writes: u64,
        hint: Option<crate::compositor::Region>,
    ) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        let w = self.undo.write_state.get();
        let verdict = match hint {
            Some(r) => format!("JANELA {}x{} em ({},{})", r.w, r.h, r.x, r.y),
            None => "VARRE".to_string(),
        };
        eprintln!(
            "[S3-AUDIT] commit: {verdict} · nao-declarados={} · snapshot_writes={snapshot_writes}",
            w.undeclared()
        );
    }

    /// **A REDE DO JOURNAL — todo escritor de canvas passa pela porta?** (doc 28 §7, degrau S3).
    ///
    /// O journal captura os bytes velhos do canvas na primeira vez que a porta
    /// ([`super::paint::plane_fork::fork_canvas`]) é atravessada, e é zerado em cada commit. Logo o
    /// que ele guarda é, por construção, **o canvas como ele estava no ÚLTIMO COMMIT** — e quem
    /// carrega esse estado hoje é o `cursor` do histórico. Esta função afirma essa igualdade.
    ///
    /// ⚠️ **O alvo é o `cursor`, não o `before` do passo, e a primeira versão errou isso.** Mirado no
    /// `before` a rede disparou em 16 testes na 1ª rodada, com o journal em 255 (tela virgem) contra
    /// um `before` já pintado: o journal descreve um passado *mais antigo*, porque escritas entre dois
    /// commits (um preview re-stampado, um tick de água) o armam antes de o passo abrir. Isso **não é
    /// defeito** — é a mesma reconciliação que o
    /// [`crate::undo::UndoController::absorb_foreign_writes`] faz hoje por diff, e que o journal
    /// passa a fazer **por construção**: ele capturou aquelas escritas também.
    ///
    /// ⚠️ **O que ela pega é o escritor que escreve sem passar pela porta**: aí a captura pega bytes
    /// já modificados e o journal passa a descrever um passado que nunca existiu. É a única forma de
    /// ele estar errado enquanto captura o plano inteiro, e é justamente o que precisa estar provado
    /// antes de ele virar a FONTE do undo em vez de uma rede.
    ///
    /// ⚠️ **A rede só se aplica a um passo que COMEÇA no cursor**, e a razão está no
    /// [`crate::undo::UndoController::absorb_foreign_writes`]: se o `before` que chega não é o
    /// cursor, alguém escreveu no meio — e o histórico responde a isso **re-partindo a entrada do
    /// topo e movendo o cursor**. Até essa reconciliação acontecer o journal está ancorado num
    /// commit que deixou de descrever a tela, então a pergunta que ele responderia é sobre outro
    /// passado. Perguntar mesmo assim seria afirmar mais do que o produto promete.
    ///
    /// O discriminante é o MESMO fato que a absorção usa (o `before` é o cursor?), perguntado por
    /// **identidade de `Arc`**: no caso comum nada escreveu desde o commit, então o `before` clona o
    /// ponteiro do cursor e a igualdade é exata e barata.
    ///
    /// ⚠️ **E há agora um alvo MAIS FORTE, quando o sítio abre o passo** (doc 28 §5.26): com o journal
    /// ancorado no início do passo (`begin_undo_step`) ele descreve o **`before` daquele passo**, que é
    /// exatamente o lado que o commit precisa. Aí a rede compara contra o `before` — sem a escapatória
    /// do `ptr_eq`, porque não há mais um vão entre o commit e o passo em que a gota pudesse morar.
    /// Enquanto o sítio não abrir o passo, cai no alvo antigo (o cursor) e o commit segue derivando.
    ///
    /// ```text
    /// PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter --lib 2>&1 | grep "journal do canvas"
    /// ```
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn audit_journal_matches_the_before(&self, before: &crate::undo::ModelSnapshot) {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some()) {
            return;
        }
        // O alvo FORTE: o journal foi zerado no início DESTE passo, então ele descreve o `before` dele.
        if self
            .undo
            .write_state
            .journal_describes_step_at(before.writes)
        {
            self.audit_journal_against("PASSO", &before.canvas_rgba);
            self.audit_relief_journals(before);
            return;
        }
        if !self.undo.write_state.get().covers(before.writes) {
            return; // um `before` anterior ao último reset: o journal não fala do passado dele
        }
        let Some(cursor) = self.undo.cursor_for_audit() else {
            return; // história vazia: não há "último commit" com que comparar
        };
        if !std::sync::Arc::ptr_eq(&before.canvas_rgba, &cursor.canvas_rgba) {
            return; // escrita estrangeira: a absorção move o cursor, e só depois dela há o que conferir
        }
        self.audit_journal_against("COMMIT", &cursor.canvas_rgba);
    }

    /// **A mesma rede, para os TRÊS planos de relevo** — o fold é a metade cara do pen-up (9,25 ms de
    /// fork), e ele é quem o S3 tem de alcançar depois do canvas.
    ///
    /// ⚠️ O relevo é um **mapa por camada**, então a pergunta carrega a camada: os journals recusam
    /// responder por outra (`speaks_for`), e um passo que tocou duas se declara misturado em vez de
    /// devolver bytes do plano errado.
    #[cfg(any(test, debug_assertions))]
    fn audit_relief_journals(&self, before: &crate::undo::ModelSnapshot) {
        let Some(layer) = self.layers.active() else {
            return;
        };
        if !self
            .undo
            .write_state
            .relief_describes_step_at(before.writes, layer)
        {
            eprintln!("[S3-AUDIT] relevo/{}", self.undo.write_state.relief_state());
            return;
        }
        /// `(conhecidos, divergentes)` de um plano do mapa contra o que o journal diz dele.
        fn tally<T: Copy + PartialEq>(
            want: Option<&std::sync::Arc<Vec<T>>>,
            get: impl Fn(usize) -> Option<T>,
        ) -> (usize, usize) {
            let (mut known, mut differ) = (0usize, 0usize);
            for (i, &w) in want.into_iter().flat_map(|v| v.iter().enumerate()) {
                if let Some(got) = get(i) {
                    known += 1;
                    differ += usize::from(got != w);
                }
            }
            (known, differ)
        }
        let w = &self.undo.write_state;
        let (kh, dh) = tally(before.heights.get(&layer), |i| w.heights_before(layer, i));
        let (kc, dc) = tally(before.covers.get(&layer), |i| w.covers_before(layer, i));
        let (km, dm) = tally(before.mats.get(&layer), |i| w.mats_before(layer, i));
        eprintln!(
            "[S3-AUDIT] relevo/PASSO: h={kh}/{dh} c={kc}/{dc} m={km}/{dm} (conhecidos/divergem)"
        );
        assert!(
            dh + dc + dm == 0,
            "os journals de relevo descrevem um passado diferente do `before` do passo: heights {dh}, \
             covers {dc}, mats {dm} divergem. Algo escreveu num plano de relevo SEM passar pelas portas \
             `fork_heights`/`fork_covers`/`fork_mats` (doc 28 §7)"
        );
    }

    /// A comparação em si, para os dois alvos — **uma porta**, porque duas cópias dela divergiriam
    /// sobre o que "o journal está certo" significa, e o alvo forte nasceria com a asserção fraca.
    ///
    /// Conta também **quantos elementos o journal de fato responde**: um journal vazio concorda com
    /// qualquer coisa, e *zero divergências sobre zero bytes* é o gate vazio que a §5.23 já pagou uma
    /// vez. O readout separa as duas leituras.
    #[cfg(any(test, debug_assertions))]
    fn audit_journal_against(&self, tag: &str, want: &[u8]) {
        let (mut differ, mut known, mut first) = (0usize, 0usize, None);
        for (i, &w) in want.iter().enumerate() {
            if let Some(got) = self.undo.write_state.canvas_before(i) {
                known += 1;
                if got != w {
                    differ += 1;
                    first.get_or_insert((i, got, w));
                }
            }
        }
        eprintln!("[S3-AUDIT] journal/{tag}: conhecidos={known} divergem={differ}");
        assert!(
            differ == 0,
            "o journal do canvas descreve um passado diferente do alvo {tag}: {differ} byte(s) \
             divergem, o 1o em {first:?} (indice, journal, alvo). Isso significa que algo escreveu no \
             canvas SEM passar pela porta `fork_canvas` — o journal capturou bytes ja modificados, e \
             como FONTE do undo ele devolveria um estado que nunca existiu (doc 28 §7)"
        );
    }

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(crate) fn audit_journal_matches_the_before(&self, _before: &crate::undo::ModelSnapshot) {}

    #[cfg(not(any(test, debug_assertions)))]
    #[expect(clippy::unused_self, reason = "no-op em release; a rede é de debug")]
    pub(crate) fn audit_commit_window(
        &self,
        _snapshot_writes: u64,
        _hint: Option<crate::compositor::Region>,
    ) {
    }
}
