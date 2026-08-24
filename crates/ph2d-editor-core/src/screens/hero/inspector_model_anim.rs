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
    /// **Esta animação tem ritmo PRÓPRIO por célula?** (spec §8.12, o que um `.ase` traz.)
    ///
    /// ⚠️ Um **facto**, e não o vetor: o Inspector não o edita — a §8.8 põe essa edição no editor
    /// de timeline futuro. O que ele tem de fazer é **dizer que ele existe**, senão o campo
    /// `Frame ms` mente sobre a animação e ninguém sabe porquê.
    pub per_frame_timing: bool,
}

impl InspectorAnimRow {
    /// O intervalo cabe na grelha que a sprite tem?
    ///
    /// ⚠️ **Derivado contra o `cells` do snapshot, nunca guardado.** Mexer em `hframes` encolhe a
    /// grelha debaixo de uma animação gravada, e uma bandeira «válida» envelheceria em silêncio —
    /// a mesma lei do `kind()` de uma âncora e do `mount_index()` da §12.
    pub fn fits(&self, cells: u32) -> bool {
        cells > 0 && self.from.min(self.to) < cells
    }

    /// Quantas células esta animação percorre, dentro da grelha de hoje.
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
            Some(i) => !self.rows[i].fits(self.cells),
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
        let span = row.span(self.cells);
        if span == 0 {
            return None;
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let step = (v.clamp(0.0, 1.0) * (span - 1) as f32).round() as u32;
        Some(row.from.min(row.to) + step)
    }

    /// A posição do frame atual dentro da animação a tocar, para a barra: `(passo, total)`.
    ///
    /// `None` quando não há animação a tocar, ou quando ela não cabe — não há barra que mostrar.
    pub fn progress(&self) -> Option<(u32, u32)> {
        let row = self.rows.get(self.current_index()?)?;
        let span = row.span(self.cells);
        if span == 0 {
            return None;
        }
        let lo = row.from.min(row.to);
        // ⚠️ O frame pode estar FORA do intervalo (a grelha mudou, ou a animação acabou de ser
        // escolhida e ainda não correu um tique). Saturar é honesto: a barra mostra o princípio.
        let step = self.frame.saturating_sub(lo).min(span - 1);
        Some((step, span))
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
            per_frame_timing: false,
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
}
