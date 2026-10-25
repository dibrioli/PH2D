//! **O modelo da §11 Animation** (spec
//! [`08_animation_inline.md`](../../../../docs/Sprite_projeto/08_animation_inline.md)) — snapshot
//! e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros cinco.
//!
//! # Duas metades com donos diferentes, outra vez
//!
//! A seção tem a **biblioteca** (as animações desta sprite: nome, intervalo, ritmo) e o
//! **tocador** (o que está a tocar agora, a que velocidade). São dois componentes distintos no
//! motor, e a UI diz qual é qual — a mesma lição que a §12 pagou ao misturar as âncoras deste
//! objeto com a âncora do pai.
//!
//! # ⚠️ O que este snapshot NÃO tem, e porquê
//!
//! Não há uma cópia do frame atual em vírgula flutuante nem do progresso: os dois são derivados
//! do estado inteiro, e derivá-los aqui daria uma **segunda** resposta a «em que ponto está esta
//! animação». O snapshot traz os inteiros; quem os apresenta calcula na hora.

/// Uma animação, como o Inspector a lê.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAnimRow {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub frame_ms: u32,
    /// A posição em `ph2d_ecs::AnimDirection::ALL`.
    pub direction_tag: u8,
    /// `0` = repete para sempre; `n` = toca `n` ciclos.
    pub repeat: u32,
    pub hold_ms: u32,
    pub repeat_delay_ms: u32,
    /// O nome do sinal ao ACABAR (spec §8.10). Vazio = calada.
    pub signal_on_finish: String,
    /// O nome do sinal ao fechar um CICLO. Vazio = calada.
    pub signal_on_loop: String,
    /// **A duração de cada célula do intervalo**, em ms — `0` = herda o `frame_ms`, vazio = todas
    /// herdam (spec §8.12).
    ///
    /// ⚠️ **O VETOR, e não um `bool`.** A primeira versão trazia só *«tem ritmo próprio?»*, porque
    /// o Inspector não o editava; o Enio pediu o campo (*«se não tiver um parâmetro de duração
    /// para cada quadro, crie»*), e um campo que EDITA precisa de ler o valor. O predicado passou a
    /// ser derivado ([`Self::has_per_frame_timing`]) — *uma fonte, e a pergunta calcula-se dela*.
    pub per_frame_ms: Vec<u32>,
    /// A lista de quadros (`SpriteAnimFrames`), na ordem tocada. Vazia = o intervalo `from..=to`.
    pub frames: Vec<u32>,
}

impl InspectorAnimRow {
    /// Esta animação declara o ritmo de alguma célula? **Derivado do vetor**, nunca guardado ao
    /// lado dele — duas respostas à mesma pergunta divergem no dia em que uma for esquecida.
    #[must_use]
    pub fn has_per_frame_timing(&self) -> bool {
        self.per_frame_ms.iter().any(|&v| v > 0)
    }

    /// O intervalo cabe na grelha que a sprite tem?
    ///
    /// ⚠️ **Derivado contra o `cells` do snapshot, nunca guardado.** Mexer em `hframes` encolhe a
    /// grelha debaixo de uma animação gravada, e uma bandeira «válida» envelheceria em silêncio —
    /// a mesma lei do `kind()` de uma âncora e do `mount_index()` da §12.
    pub fn fits(&self, cells: u32) -> bool {
        cells > 0 && self.from.min(self.to) < cells
    }

    /// Quantas células esta animação percorre, dentro da grelha de hoje.
    /// **Esta animação toca na grelha de hoje?** Uma lista de quadros só toca se TODAS as células
    /// cabem (a lei de `ph2d_ecs::tag_cells`: aparar tocaria outra animação); um intervalo, se o
    /// início cabe.
    #[must_use]
    pub fn plays_on(&self, cells: u32) -> bool {
        if self.frames.is_empty() {
            self.fits(cells)
        } else {
            cells > 0 && self.frames.iter().all(|&c| c < cells)
        }
    }

    pub fn span(&self, cells: u32) -> u32 {
        if !self.fits(cells) {
            return 0;
        }
        let lo = self.from.min(self.to);
        let hi = self.from.max(self.to).min(cells - 1);
        hi - lo + 1
    }
}

/// Snapshot da §11 da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAnimInfo {
    pub entity_bits: u64,
    /// A biblioteca inteira, na ordem em que o componente a guarda.
    pub rows: Vec<InspectorAnimRow>,
    /// O TOCADOR está anexado? Sem ele a seção mostra só o botão que o anexa.
    pub player_present: bool,
    /// **Quantas células a grelha desta sprite tem** (`hframes × vframes`) — o pool.
    pub cells: u32,
    /// O nome da animação a tocar. Vazio = nenhuma.
    pub current: String,
    pub playing: bool,
    pub autoplay: bool,
    /// A velocidade em múltiplos — o `Q16.16` do motor, convertido para leitura.
    pub speed: f32,
    /// `0` = herdar; `1..=4` = a direção de `AnimDirection::ALL` mais um.
    pub direction_override_tag: u8,
    /// `0` = herdar · `1` = ligado · `2` = desligado.
    pub loop_override_tag: u8,
    /// O frame que está no ecrã agora (`Sprite::frame`).
    pub frame: u32,
    /// Quantas entidades estão selecionadas.
    ///
    /// ⚠️ **A §11 NÃO se espalha sobre a seleção** (uma animação identifica-se pelo nome, e o
    /// índice que a edição carrega só significa alguma coisa na biblioteca da primária), e por
    /// isso este número é o que a seção usa para o **dizer**. Sem ele, marcar cinco goblins e
    /// renomear uma animação muda **um** e cala-se — o artista descobre semanas depois.
    pub selected_count: usize,
    /// O cartão «Animation» e o editor (plano GameEngineFuncional 07).
    pub card: super::AnimCardInfo,
}

impl InspectorAnimInfo {
    /// O índice, na biblioteca, da animação a tocar. **Derivado do nome.**
    pub fn current_index(&self) -> Option<usize> {
        if self.current.is_empty() {
            return None;
        }
        self.rows.iter().position(|r| r.name == self.current)
    }

    /// **O tocador aponta para uma animação que já não existe** — ou que já não cabe na grelha.
    ///
    /// ⚠️ Duas causas, um estado, pela mesma razão do `mount_dangling` da §12: apagar a animação
    /// e encolher a grelha por baixo dela produzem a mesma coisa aos olhos do artista — nada
    /// acontece —, e ele precisa de o poder ler.
    pub fn current_dangling(&self) -> bool {
        if self.current.is_empty() {
            return false;
        }
        match self.current_index() {
            None => true,
            Some(i) => !self.rows[i].plays_on(self.cells),
        }
    }

    /// **A POSIÇÃO da cabeça de leitura no curso da barra, `0..1`.**
    ///
    /// ⚠️ **É posição, e não progresso.** A barra media `(passo+1) / total` enquanto só informava;
    /// desde que ela se arrasta (2026-08-23) tem de medir `passo / (total-1)`, senão o polegar não
    /// pousa em cima do frame — o primeiro frame desenhar-se-ia já com uma fatia preenchida.
    ///
    /// ⚠️ **Esta função e a [`scrub_cell`](Self::scrub_cell) são UMA lei em dois sentidos, e viviam
    /// em TRÊS sítios** (o pintor, o `sync` e o despacho, cada um com a sua cópia). Uma mutação que
    /// mudou só a do pintor **sobreviveu** a toda a suíte — foi isso que as trouxe para aqui. O
    /// gate que as prende é `the_scrub_position_and_the_cell_are_inverses`.
    ///
    /// `None` quando não há animação a tocar ou ela não cabe na grelha — não há barra que mostrar.
    #[must_use]
    pub fn scrub_position(&self) -> Option<f32> {
        let (step, span) = self.progress()?;
        Some(if span > 1 {
            step as f32 / (span - 1) as f32
        } else {
            0.0
        })
    }

    /// **A INVERSA**: que célula (absoluta) corresponde a `v` no curso da barra.
    ///
    /// ⚠️ `round`, e não truncar: com truncagem a última célula só apareceria no pixel final da
    /// trilha, e cada uma das outras ocuparia uma fatia deslocada de meia célula.
    #[must_use]
    pub fn scrub_cell(&self, v: f32) -> Option<u32> {
        let row = self.rows.get(self.current_index()?)?;
        let seq = self.playing_cells(row)?;
        Some(seq[self.scrub_step(v, seq.len())])
    }

    /// O passo (`0..n`) que a posição `v` da barra escolhe — a inversa de [`Self::scrub_position`].
    fn scrub_step(&self, v: f32, n: usize) -> usize {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let step = (v.clamp(0.0, 1.0) * n.saturating_sub(1) as f32).round() as usize;
        step
    }

    /// **A edição que o arrasto da barra emite.** Num intervalo é a célula (`SetFrame`); numa lista de
    /// quadros é o PASSO (`SetStep`) — a mesma célula pode estar na lista duas vezes, e `SetFrame` não
    /// saberia qual das duas.
    #[must_use]
    pub fn scrub_edit(&self, v: f32) -> Option<AnimFieldEdit> {
        let row = self.rows.get(self.current_index()?)?;
        let seq = self.playing_cells(row)?;
        let step = self.scrub_step(v, seq.len());
        Some(if row.frames.is_empty() {
            AnimFieldEdit::SetFrame(seq[step])
        } else {
            AnimFieldEdit::SetStep(u32::try_from(step).ok()?)
        })
    }

    /// As células que `row` toca hoje, ou `None` se não toca (fora da grelha). Ver [`InspectorAnimRow::plays_on`].
    fn playing_cells(&self, row: &InspectorAnimRow) -> Option<Vec<u32>> {
        row.plays_on(self.cells)
            .then(|| super::row_cells(row, self.cells))
            .filter(|s| !s.is_empty())
    }

    /// O passo da animação a tocar que está no ecrã: numa lista, o cursor (`card.step`) se ele aponta
    /// para a célula visível; num intervalo, `frame - from`. `None` fora da animação.
    fn current_step(&self, row: &InspectorAnimRow) -> Option<usize> {
        let seq = self.playing_cells(row)?;
        if row.frames.is_empty() {
            let lo = row.from.min(row.to);
            return (self.frame >= lo && ((self.frame - lo) as usize) < seq.len())
                .then(|| (self.frame - lo) as usize);
        }
        let step = self.card.step as usize;
        if seq.get(step) == Some(&self.frame) {
            return Some(step);
        }
        seq.iter().position(|&c| c == self.frame)
    }

    /// **Onde, no vetor de ritmo, mora a célula que a barra está a mostrar.**
    ///
    /// ⚠️ **Derivado, e é a mesma célula que o arrasto escolhe** (`scrub_cell`): o controlo de
    /// duração por-quadro não precisa de um segundo selector, porque a barra **já** é o selector.
    /// *Um painel que pede duas vezes «qual quadro?» é um painel em que os dois podem discordar.*
    ///
    /// `None` quando não há animação a tocar, ou quando o frame no ecrã caiu fora do intervalo
    /// dela — não há célula desta animação a que a duração se possa referir.
    #[must_use]
    pub fn this_frame_index(&self) -> Option<usize> {
        let row = self.rows.get(self.current_index()?)?;
        self.current_step(row)
    }

    /// **Quanto dura a célula que a barra mostra**, em ms. `0` = herda o `frame_ms` da animação.
    ///
    /// ⚠️ **`0` é «não declarado», e não «instantâneo»** — é a mesma convenção do `Repeat (0 =
    /// forever)` que esta seção já usa, e é o que permite ao vetor ser **esparso**: declarar a
    /// duração de UMA célula não obriga a escrever as outras sete.
    #[must_use]
    pub fn this_frame_ms(&self) -> Option<u32> {
        let i = self.this_frame_index()?;
        let row = self.rows.get(self.current_index()?)?;
        Some(row.per_frame_ms.get(i).copied().unwrap_or(0))
    }

    /// **QUEM o campo de duração edita: `(linha, célula)`** — e as duas saem da mesma lei.
    ///
    /// ⚠️ **A linha é a da animação que a BARRA mostra (`current`), e não a que a lista tem
    /// selecionada** — e a diferença foi um defeito reportado (Enio, 2026-08-23: *«o tempo volta a
    /// 0 no enter»*). O campo mora na zona do TOCADOR (barra, Playing, Speed, Rewind), que fala da
    /// animação a tocar; a zona da BIBLIOTECA, por baixo, é que fala da linha selecionada. Escrever
    /// o valor numa e lê-lo da outra faz o campo **reverter sozinho**, que é exactamente como o
    /// defeito se via.
    ///
    /// ⇒ **Uma função devolve as duas**, para que quem escreve não possa usar um índice de linha
    /// diferente do que produziu a célula.
    #[must_use]
    pub fn this_frame_target(&self) -> Option<(u8, u32)> {
        let row = u8::try_from(self.current_index()?).ok()?;
        let cell = u32::try_from(self.this_frame_index()?).ok()?;
        Some((row, cell))
    }

    /// A posição do frame atual dentro da animação a tocar, para a barra: `(passo, total)`.
    ///
    /// `None` quando não há animação a tocar, ou quando ela não cabe — não há barra que mostrar.
    pub fn progress(&self) -> Option<(u32, u32)> {
        let row = self.rows.get(self.current_index()?)?;
        let n = u32::try_from(self.playing_cells(row)?.len()).ok()?;
        // ⚠️ O frame pode estar FORA da animação (a grelha mudou, ou ela acabou de ser escolhida e
        // ainda não correu um tique). Saturar é honesto: a barra mostra o princípio (ou o fim).
        let step = match self.current_step(row) {
            Some(s) => u32::try_from(s).ok()?,
            None if row.frames.is_empty() => {
                self.frame.saturating_sub(row.from.min(row.to)).min(n - 1)
            }
            None => 0,
        };
        Some((step, n))
    }
}

/// Uma edição da §11.
///
/// ⚠️ **As da BIBLIOTECA carregam o índice da animação; as do TOCADOR não carregam nenhum** —
/// elas falam do estado, que é um só por sprite. É a mesma divisão que a §12 fez entre as âncoras
/// deste objeto e a montagem no pai.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimFieldEdit {
    /// Cria uma animação com o próximo nome livre, cobrindo a grelha inteira.
    Add,
    /// Retira a animação deste índice.
    Remove(u8),
    /// Renomeia. ⚠️ Nome inválido ou repetido é **recusado com aviso**, nunca em silêncio.
    Rename(u8, String),
    /// `(animação, primeira célula)`.
    From(u8, u32),
    /// `(animação, última célula)`.
    To(u8, u32),
    /// `(animação, ms por frame)`.
    FrameMs(u8, u32),
    /// `(animação, ms de pausa no último frame)`.
    HoldMs(u8, u32),
    /// `(animação, ms de pausa entre ciclos)`.
    DelayMs(u8, u32),
    /// `(animação, ciclos — `0` = para sempre)`.
    Repeat(u8, u32),
    /// `(animação, posição em `AnimDirection::ALL`)`.
    Direction(u8, u8),
    /// `(animação, nome do sinal ao ACABAR)` — vazio cala a animação.
    SignalOnFinish(u8, String),
    /// `(animação, nome do sinal ao fechar um CICLO)` — vazio cala a animação.
    SignalOnLoop(u8, String),
    /// **`(animação, índice DENTRO do intervalo, ms)`** — a duração de UMA célula (spec §8.12).
    ///
    /// ⚠️ O índice é relativo ao `from` da animação, e não à grelha: é assim que o vetor se
    /// reindexa sozinho quando o intervalo muda. `0` limpa a declaração (a célula volta a herdar
    /// o `frame_ms`).
    FrameMsAt(u8, u32, u32),

    /// Anexa o TOCADOR a esta sprite.
    AddPlayer,
    /// **Escolhe a animação a tocar** — o clique numa linha da lista.
    ///
    /// ⚠️ Ao contrário da §12, isto **é** uma edição da cena: numa biblioteca de animações, a que
    /// se vê e a que toca são a mesma (o `AnimationPlayer` do Godot faz assim), e separá-las
    /// pediria um segundo controlo que duplicaria a lista.
    SetCurrent(String),
    Playing(bool),
    Autoplay(bool),
    /// A velocidade em múltiplos. A conversão para `Q16.16` mora na shell, num sítio só.
    Speed(f32),
    /// `0` = herdar; `1..=4` = `AnimDirection::ALL` mais um.
    DirectionOverride(u8),
    /// `0` herdar · `1` ligado · `2` desligado.
    LoopOverride(u8),
    /// Repõe o ciclo no princípio.
    Rewind,
    /// **Põe a cabeça de leitura nesta CÉLULA** — o arrasto da barra de frames.
    ///
    /// ⚠️ **Célula absoluta, e não um passo dentro do intervalo.** O `Sprite::frame` é absoluto, e
    /// um passo relativo obrigaria o commit a re-derivar o `lo` de uma tag que pode ter mudado
    /// entre o gesto e o commit.
    ///
    /// ⚠️ **Agarrar a barra PAUSA a reprodução**, e isso é do verbo: enquanto o tique também
    /// escreve o `Sprite::frame`, o dedo e o relógio disputam o mesmo campo e a imagem pisca entre
    /// os dois. *Quem pega no volante conduz.*
    SetFrame(u32),

    // ── O cartão «Animation» e o editor (plano GameEngineFuncional 07) ──────────────────────────
    /// A lista de quadros da animação `row`, na ordem tocada. Vazia = volta ao intervalo.
    Frames(u8, Vec<u32>),
    /// O modo do ciclo (`AnimCycleMode::tag`) — `repeat` e `direction` num só passo de undo.
    CycleMode(u8, u8),
    /// Quem escolhe a animação (`AnimChooser::tag`).
    Chooser(u8),
    /// A animação do estado `0..4` (Parado · Correr · Saltar · Cair) no modo «O personagem».
    CharacterState(u8, String),
    /// A partir de que velocidade (m/s) o personagem corre.
    CharacterRunFrom(f32),
    /// Virar a imagem ao andar para trás.
    CharacterFlip(bool),
    /// Parar e mostrar o PASSO `n` da animação que toca (a barra e a faixa de quadros numa lista, onde a
    /// mesma célula pode aparecer duas vezes e `SetFrame(célula)` seria ambíguo).
    SetStep(u32),
    /// «Editar animações…»: abrir o editor de animações desta sprite numa aba. Não mexe na cena — quem o
    /// trata é a shell, que abre a aba (`ph2d-app-sprite-anim`).
    OpenEditor,
    /// A grelha da folha, colunas × linhas (a proposta confirmada, ou o campo do cartão).
    Grid(u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, from: u32, to: u32) -> InspectorAnimRow {
        InspectorAnimRow {
            name: name.into(),
            from,
            to,
            frame_ms: 100,
            direction_tag: 0,
            repeat: 0,
            hold_ms: 0,
            repeat_delay_ms: 0,
            signal_on_finish: String::new(),
            signal_on_loop: String::new(),
            per_frame_ms: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn info(cells: u32, current: &str) -> InspectorAnimInfo {
        InspectorAnimInfo {
            entity_bits: 1,
            rows: vec![row("idle", 0, 1), row("walk", 2, 5)],
            player_present: true,
            cells,
            current: current.into(),
            playing: false,
            autoplay: false,
            speed: 1.0,
            direction_override_tag: 0,
            loop_override_tag: 0,
            frame: 0,
            selected_count: 1,
            card: crate::AnimCardInfo::default(),
        }
    }

    /// **O intervalo é lido contra a grelha de HOJE.**
    #[test]
    fn the_range_is_measured_against_the_grid_that_exists() {
        let r = row("walk", 2, 5);
        assert_eq!(r.span(8), 4);
        assert_eq!(r.span(4), 2, "a grelha encolheu: o fim recua");
        assert_eq!(r.span(2), 0, "a grelha encolheu abaixo do inicio");
        assert!(!r.fits(2));
        assert_eq!(r.span(0), 0);
    }

    /// **O «pendurado» tem DUAS causas e um estado** — o nome sumiu, ou a grelha encolheu.
    #[test]
    fn a_current_animation_can_dangle_two_different_ways() {
        assert!(
            !info(8, "").current_dangling(),
            "sem escolha nao ha' o que pendurar"
        );
        assert!(!info(8, "walk").current_dangling());
        assert!(
            info(8, "run").current_dangling(),
            "o nome nao esta' na biblioteca"
        );
        assert!(
            info(2, "walk").current_dangling(),
            "a tag existe e a grelha encolheu debaixo dela"
        );
    }

    /// **A POSIÇÃO e a CÉLULA são inversas uma da outra, em toda célula do intervalo.**
    ///
    /// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE** (2026-08-23): a régua da barra vivia em
    /// três sítios — o pintor, o `sync` e o despacho — e trocar a do PINTOR de posição para
    /// progresso passava a suíte inteira, porque nada ligava o que se desenha ao que o clique
    /// produz. O polegar deixaria de pousar em cima do frame e nenhum teste diria nada.
    ///
    /// ⇒ Uma lei, dois sentidos, e a ida-e-volta é a afirmação.
    #[test]
    fn the_scrub_position_and_the_cell_are_inverses() {
        let mut i = info(8, "walk"); // walk = 2..=5 sobre 8 celulas
        for cell in 2..=5u32 {
            i.frame = cell;
            let pos = i.scrub_position().expect("ha' barra");
            assert!(
                (0.0..=1.0).contains(&pos),
                "a posicao tem de caber no curso: {pos}"
            );
            assert_eq!(
                i.scrub_cell(pos),
                Some(cell),
                "ida-e-volta partida na celula {cell} (posicao {pos})"
            );
        }
        // ⚠️ As DUAS PONTAS do curso alcancam as duas pontas do intervalo — e e' a metade que a
        // regua de PROGRESSO nao dava: com ela o minimo do widget ja' valia uma celula.
        i.frame = 2;
        assert_eq!(
            i.scrub_position(),
            Some(0.0),
            "a primeira celula fica no ZERO do curso"
        );
        i.frame = 5;
        assert_eq!(i.scrub_position(), Some(1.0), "e a ultima no fim");
        assert_eq!(i.scrub_cell(0.0), Some(2));
        assert_eq!(i.scrub_cell(1.0), Some(5));
        // Fora do curso: fixa, nunca extrapola.
        assert_eq!(i.scrub_cell(-3.0), Some(2));
        assert_eq!(i.scrub_cell(9.0), Some(5));
        // Sem animacao a tocar (ou fora da grelha) nao ha' barra nem celula.
        assert_eq!(info(8, "").scrub_position(), None);
        assert_eq!(info(8, "").scrub_cell(0.5), None);
        assert_eq!(info(2, "walk").scrub_position(), None);
        assert_eq!(info(2, "walk").scrub_cell(0.5), None);
    }

    /// A barra de progresso conta **passos dentro do intervalo**, e satura em vez de mentir.
    #[test]
    fn the_progress_bar_counts_steps_inside_the_range_and_saturates() {
        let mut i = info(8, "walk"); // walk = 2..=5, 4 passos
        i.frame = 2;
        assert_eq!(i.progress(), Some((0, 4)));
        i.frame = 4;
        assert_eq!(i.progress(), Some((2, 4)));
        // ⚠️ Fora do intervalo (a grelha mudou, ou a escolha é deste quadro): satura.
        i.frame = 0;
        assert_eq!(i.progress(), Some((0, 4)));
        i.frame = 99;
        assert_eq!(i.progress(), Some((3, 4)));
        // Sem animação, ou sem intervalo, não há barra.
        assert_eq!(info(8, "").progress(), None);
        assert_eq!(info(2, "walk").progress(), None);
    }

    /// **O CAMPO DE DURAÇÃO APONTA PARA A CÉLULA QUE A BARRA MOSTRA** — e para nenhuma outra.
    ///
    /// ⚠️ O índice é relativo ao `from` da animação: uma tag `4..7` com o frame no 5 edita a
    /// **segunda** entrada do vetor, não a sexta. É isso que faz o vetor reindexar-se sozinho
    /// quando o artista mexe no intervalo, em vez de apontar para o sítio errado em silêncio.
    #[test]
    fn the_duration_field_points_at_the_cell_the_bar_shows() {
        let mut i = info(8, "attack");
        i.rows = vec![row("attack", 4, 7)];
        i.frame = 4;
        assert_eq!(i.this_frame_index(), Some(0), "o `from` e' o indice ZERO");
        i.frame = 5;
        assert_eq!(i.this_frame_index(), Some(1));
        i.frame = 7;
        assert_eq!(i.this_frame_index(), Some(3));
    }

    /// **Fora do intervalo não há campo** — não existe célula desta animação a que a duração se
    /// possa referir, e um campo que edita «a célula 9 de uma animação que vai até à 7» escreveria
    /// num sítio que ninguém vê.
    #[test]
    fn outside_the_range_there_is_no_cell_to_edit() {
        let mut i = info(8, "attack");
        i.rows = vec![row("attack", 4, 7)];
        i.frame = 2;
        assert_eq!(i.this_frame_index(), None);
        assert_eq!(i.this_frame_ms(), None);
        i.frame = 4;
        assert!(i.this_frame_index().is_some(), "e dentro dele ha'");
    }

    /// **O valor mostrado é o da célula, e `0` quer dizer «herda»** — a mesma convenção do
    /// `Repeat (0 = forever)` que esta seção já usa, e o que torna o vetor ESPARSO possível.
    #[test]
    fn the_shown_value_is_the_cells_own_and_zero_means_inherit() {
        let mut i = info(8, "attack");
        let mut r = row("attack", 4, 7);
        r.per_frame_ms = vec![0, 250];
        assert!(r.has_per_frame_timing(), "um valor > 0 conta");
        i.rows = vec![r];
        i.frame = 4;
        assert_eq!(i.this_frame_ms(), Some(0), "declarado como herdar");
        i.frame = 5;
        assert_eq!(i.this_frame_ms(), Some(250));
        i.frame = 6;
        assert_eq!(i.this_frame_ms(), Some(0), "fora do vetor tambem herda");
    }

    /// **Um vetor só de zeros NÃO é ritmo próprio** — senão o aviso *«this animation has per-frame
    /// timing»* ficaria colado à animação depois de o artista limpar tudo.
    /// ⭐ Uma LISTA de quadros (plano GameEngineFuncional 07): a barra conta passos da lista, o arrasto
    /// emite `SetStep` (a mesma célula pode repetir-se) e a duração aponta para o passo.
    #[test]
    fn a_frame_list_is_walked_by_step_not_by_cell() {
        let mut i = info(8, "walk");
        i.rows[1].frames = vec![6, 2, 6, 7];
        i.frame = 6;
        i.card.step = 2;
        assert_eq!(i.progress(), Some((2, 4)), "o SEGUNDO 6, pelo cursor");
        assert_eq!(i.this_frame_index(), Some(2));
        assert_eq!(i.scrub_position(), Some(2.0 / 3.0));
        assert_eq!(i.scrub_edit(1.0), Some(AnimFieldEdit::SetStep(3)));
        assert_eq!(i.scrub_cell(1.0 / 3.0), Some(2));
        i.card.step = 9; // um cursor velho: a célula visível decide
        assert_eq!(i.this_frame_index(), Some(0));
        i.rows[1].frames = vec![6, 9];
        assert!(i.current_dangling(), "uma célula fora da grelha cala a lista inteira");
        assert_eq!(i.progress(), None);
    }

    #[test]
    fn a_range_still_scrubs_by_cell() {
        let i = info(8, "walk");
        assert_eq!(i.scrub_edit(1.0), Some(AnimFieldEdit::SetFrame(5)));
    }

    #[test]
    fn a_vector_of_zeros_is_not_per_frame_timing() {
        let mut r = row("walk", 0, 3);
        r.per_frame_ms = vec![0, 0, 0];
        assert!(!r.has_per_frame_timing());
    }
}
