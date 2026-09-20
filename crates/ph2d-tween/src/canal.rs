//! **O CANAL — que propriedade um tween escreve.**
//!
//! # ⛔ A lista é o que tem SINK hoje, e não o que seria bonito ter
//!
//! É a lei que o [`ph2d_ecs::SignalVerb`] já escreve para a tabela de acções: *um verbo cujo
//! consumidor não existe é um controlo morto com cara de feature*. Cada entrada abaixo escreve num
//! campo que existe e que alguém desenha.
//!
//! # ⚠️ A fronteira com o `PropKind` da timeline, declarada
//!
//! Esta lista **espelha** a do [`ph2d_timeline::PropKind`] menos o que não faz sentido num objecto
//! (`TimeRemap` é um relógio, `Morph` e os quatro de junta são de outros motores) e **mais o que a
//! timeline não tem**: a COR. Medido em 2026-09-19 — `13` canais e **ZERO** de cor.
//!
//! ⛔ *Duas listas de «o que se pode animar» divergem*, e a fronteira é esta: **a timeline anima o
//! que uma LANE endereça; o tween anima o que um OBJECTO carrega.** É a mesma frase que separa uma
//! cutscene de um componente, e é a razão de a cópia nascida numa corrida só ter uma delas.

use serde::{Deserialize, Serialize};

/// Ver o cabeçalho do módulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Canal {
    /// **A opacidade** — o `tint[3]` do `Sprite`, o modulate HERDADO (ADR-0071), que cascateia
    /// para os descendentes.
    ///
    /// ⭐ É o canal que a timeline também escreve (`PropKind::Opacity`), e é de propósito: o
    /// ledger já tem a entrada dele, e um fade autorado e um fade por componente escrevem o
    /// **mesmo** facto.
    #[default]
    Opacity,
    /// **A cor** — o `self_tint` do `Sprite`, o `self_modulate` do Godot, que **não** cascateia.
    ///
    /// ⚠️ **Ele e o [`Canal::Opacity`] são campos DIFERENTES de propósito** (ADR-0071 declara os
    /// dois como canónicos): um fade e um tint compõem sem se pisarem, e é isso que faz *«o
    /// inimigo pisca vermelho enquanto desvanece»* ser autorável sem uma lei nova.
    Tint,
    /// **A SILHUETA** — o `self_tint` **com o `tint_fill` ligado**: a textura desaparece e fica a
    /// forma chapada na cor.
    ///
    /// ⭐⭐ É o *flash de dano* que o levantamento nomeia (a linha do `Sprite` v4: *«tint_fill
    /// (flash de dano)»*), e ele **só funciona com [`super::AoAcabar::Rewind`]**: um `tint_fill`
    /// que ficasse ligado no fim deixaria o objecto como uma silhueta branca para sempre — a arte
    /// não volta por interpolação, ela volta por o motor **deixar de escrever**.
    ///
    /// ⚠️ A ponte escreve **dois** campos por este canal, e o ledger guarda os dois.
    Silhueta,
    /// A posição em `x` (metros) — o `translation.x` do `Transform`.
    PositionX,
    /// A posição em `y` (metros).
    PositionY,
    /// A escala em `x`.
    ScaleX,
    /// A escala em `y`.
    ScaleY,
    /// A rotação (radianos).
    Rotation,
}

impl Canal {
    /// Todos, em ordem — **a fonte da iteração**. ⛔ Nunca escreva a lista uma segunda vez.
    /// ⚠️ **APPEND-ONLY**: a posição é a tag e ela viaja no ficheiro.
    pub const ALL: [Canal; 8] = [
        Canal::Opacity,
        Canal::Tint,
        Canal::Silhueta,
        Canal::PositionX,
        Canal::PositionY,
        Canal::ScaleX,
        Canal::ScaleY,
        Canal::Rotation,
    ];

    /// O rótulo que o artista lê. Inglês (HR-15) — a mesma forma do `SignalVerb::label`, e a mesma
    /// dívida: os MOTORES escrevem o rótulo numa tabela de dados, e a fronteira de os derivar do
    /// id está declarada pela linha da UI.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Canal::Opacity => "tween.canal.opacity",
            Canal::Tint => "tween.canal.tint",
            Canal::Silhueta => "tween.canal.silhueta",
            Canal::PositionX => "tween.canal.position_x",
            Canal::PositionY => "tween.canal.position_y",
            Canal::ScaleX => "tween.canal.scale_x",
            Canal::ScaleY => "tween.canal.scale_y",
            Canal::Rotation => "tween.canal.rotation",
        }
    }

    /// **Quantas componentes deste canal contam** — `1` para um número, `4` para uma cor.
    ///
    /// ⚠️ **DERIVADO do canal, nunca uma segunda lista.** É o que decide se o painel pinta um
    /// campo ou quatro, e é a mesma lei do `SignalVerb::uses_arg`: um painel que mostra o que o
    /// canal não lê é um controlo morto; um que esconde o que ele lê é uma feature inalcançável.
    #[must_use]
    pub const fn aridade(self) -> usize {
        match self {
            Canal::Tint | Canal::Silhueta => 4,
            _ => 1,
        }
    }

    /// **Este canal escreve na APARÊNCIA (o `Sprite`) ou na POSE (o `Transform`)?**
    ///
    /// ⚠️ **Ela existe porque a chave do ledger é `(entidade, driver)`**, e a pose tem outros
    /// donos (o solver, a timeline, um script). Misturar as duas numa entrada só faria a mão de um
    /// engolir o autorado do outro — a frase que o `Driver::ScriptPose` e o `Driver::PrefabStage`
    /// já escrevem.
    #[must_use]
    pub const fn e_da_pose(self) -> bool {
        matches!(
            self,
            Canal::PositionX | Canal::PositionY | Canal::ScaleX | Canal::ScaleY | Canal::Rotation
        )
    }

    /// ⭐⭐⭐ **Este canal é uma COR?** — e é isto que decide se o painel pinta uma AMOSTRA ou um
    /// número.
    ///
    /// ⛔⛔ **Report do dono, 2026-09-19 (com foto):** *«por que usar cores em números se temos
    /// caixas selectoras?»*. O `de`/`para` de um `Tint` saía como **quatro campos numéricos** —
    /// `0 · 0 · 0 · 0` — e o app tem um selector de cor desde sempre (a secção *Color & Tint*, as
    /// duas amostras do emissor de partículas).
    ///
    /// ⚠️ **É DERIVADO da [`Self::aridade`], e não uma segunda lista:** um canal de cor é
    /// exactamente aquele cujas quatro componentes contam. *Duas listas sobre a mesma pergunta
    /// divergem no dia em que alguém acrescentar um canal.*
    #[must_use]
    pub const fn e_cor(self) -> bool {
        self.aridade() == 4
    }

    /// A posição em [`Self::ALL`] — a tag que o segmentado do painel usa.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O canal desta posição, ou o primeiro. ⚠️ **A POSIÇÃO NO ARRAY É A TAG**, e reordenar
    /// [`Self::ALL`] faria um clique escrever outro canal — e compila.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}
