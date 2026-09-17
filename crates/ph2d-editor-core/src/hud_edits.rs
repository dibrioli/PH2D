//! ⭐⭐⭐ **O vocabulário da secção HUD** (TOP-20 #20) — o que o painel mostra e o que ele edita.
//!
//! Irmão do [`crate::particles_edits`] e com a mesma forma: um `enum` por família de campo, uma
//! tabela que dá a ORDEM (a linha `i` mostra e edita o campo `i`), e um instantâneo que a shell
//! publica por quadro.
//!
//! # ⚠️ Uma secção para QUATRO componentes, e os blocos somem sozinhos
//!
//! Um objecto de HUD raramente tem os quatro: a raiz tem o [`ph2d_ecs::UiCanvas`], o rótulo tem o
//! `UiLabel`, o botão tem o `UiButton`. ⇒ o instantâneo diz **quais existem** e o pintor esconde os
//! blocos ausentes. *Mostrar sempre os doze campos entregaria nove controlos mortos.*

/// Os NÚMEROS da secção, pela ordem em que são pintados.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HudNumber {
    /// A largura da caixa de referência do canvas.
    RefWidth,
    /// A altura dela.
    RefHeight,
    /// O valor com que o contador começa **e com que renasce**.
    CounterStart,
}

/// A ordem dos números. ⚠️ A tabela de rótulos do pintor é indexada por esta, com gate a atar os
/// comprimentos.
pub const HUD_NUMBERS: [HudNumber; 3] = [
    HudNumber::RefWidth,
    HudNumber::RefHeight,
    HudNumber::CounterStart,
];

/// Os TEXTOS da secção, pela ordem em que são pintados.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HudText {
    /// O nome da fonte do rótulo — **um campo para três fontes** (contador, relógio ou etiqueta):
    /// qual delas é o `Source` que diz. Três campos deixariam dois sempre mortos.
    SourceName,
    /// O que vem antes do número.
    Prefix,
    /// O que vem depois.
    Suffix,
    /// O nome do sinal que o botão publica.
    Signal,
    /// O nome do contador.
    CounterName,
}

/// A ordem dos textos.
pub const HUD_TEXTS: [HudText; 5] = [
    HudText::SourceName,
    HudText::Prefix,
    HudText::Suffix,
    HudText::Signal,
    HudText::CounterName,
];

/// Uma edição da secção **HUD**.
#[derive(Clone, Debug, PartialEq)]
pub enum HudFieldEdit {
    /// Um número.
    Number(HudNumber, f32),
    /// Um texto.
    Text(HudText, String),
    /// Como a caixa de referência se acomoda (`0` = Keep · `1` = Stretch).
    Fit(u8),
    /// De onde o rótulo tira o número (o índice da fonte).
    Source(u8),
    /// O botão recusa o clique?
    Disabled(bool),
}

/// **O que o painel mostra do HUD** — o instantâneo que a shell publica por quadro.
///
/// ⚠️ **Os três `has_*` mandam no que é pintado**, e os campos de um bloco ausente não são lidos.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorHudInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// Tem [`ph2d_ecs::UiCanvas`]?
    pub has_canvas: bool,
    /// A largura da caixa de referência.
    pub ref_w: f32,
    /// A altura dela.
    pub ref_h: f32,
    /// `0` = Keep · `1` = Stretch.
    pub fit: u8,
    /// ⭐ **Há câmera de jogo na cena?** Sem ela o canvas fica onde o artista o pôs, e o painel
    /// DI-LO — *um HUD que não se cola e um HUD partido leem-se igual.*
    pub tem_camera: bool,
    /// Tem `UiLabel`?
    pub has_label: bool,
    /// O índice da fonte (`0` = Authored · `1` = Counter · `2` = Timer · `3` = Tag).
    pub source: u8,
    /// O nome da fonte.
    pub source_name: String,
    /// O que vem antes do número.
    pub prefix: String,
    /// O que vem depois.
    pub suffix: String,
    /// ⭐ **O que o rótulo mostra AGORA**, ou vazio se a fonte não existe na cena — o segundo
    /// aviso que os campos sozinhos não davam.
    pub vivo: String,
    /// Tem `UiButton`?
    pub has_button: bool,
    /// O nome do sinal.
    pub signal: String,
    /// Recusa o clique?
    pub disabled: bool,
    /// Tem `Counter`?
    pub has_counter: bool,
    /// O nome do contador.
    pub counter_name: String,
    /// Com que valor começa.
    pub counter_start: f32,
    /// ⭐ O valor de AGORA — leitura, nunca edição: ele é vivo e não é documento.
    pub counter_value: i64,
}

impl InspectorHudInfo {
    /// O valor de um número — a leitura que o painel faz pela [`HUD_NUMBERS`].
    #[must_use]
    pub fn number(&self, which: HudNumber) -> f32 {
        match which {
            HudNumber::RefWidth => self.ref_w,
            HudNumber::RefHeight => self.ref_h,
            HudNumber::CounterStart => self.counter_start,
        }
    }

    /// O texto de um campo — a irmã da [`Self::number`].
    #[must_use]
    pub fn text(&self, which: HudText) -> &str {
        match which {
            HudText::SourceName => &self.source_name,
            HudText::Prefix => &self.prefix,
            HudText::Suffix => &self.suffix,
            HudText::Signal => &self.signal,
            HudText::CounterName => &self.counter_name,
        }
    }

    /// **A que bloco pertence este número** — `true` quando o objecto o tem.
    #[must_use]
    pub fn mostra_numero(&self, which: HudNumber) -> bool {
        match which {
            HudNumber::RefWidth | HudNumber::RefHeight => self.has_canvas,
            HudNumber::CounterStart => self.has_counter,
        }
    }

    /// **A que bloco pertence este texto.**
    ///
    /// ⚠️ O `Source Name` some com a fonte `Authored`: ali não há nome nenhum a dar, e mostrá-lo
    /// seria um campo que não chega a consumidor nenhum.
    #[must_use]
    pub fn mostra_texto(&self, which: HudText) -> bool {
        match which {
            HudText::SourceName => self.has_label && self.source != 0,
            HudText::Prefix | HudText::Suffix => self.has_label,
            HudText::Signal => self.has_button,
            HudText::CounterName => self.has_counter,
        }
    }
}

#[cfg(test)]
#[path = "hud_edits_tests.rs"]
mod tests;
