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
