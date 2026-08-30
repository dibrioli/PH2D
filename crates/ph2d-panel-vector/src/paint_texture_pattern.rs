//! A secção **PATTERN** do painel Vector (plano 33, W5) — módulo irmão do [`super`] (teto de LOC).
//!
//! ⚠️⚠️ **NÃO confundir com o [`super::paint_patternpath`].** Aquele é o *Pattern Along Path*
//! (plano 23): um MOTIVO copiado ao longo de uma guia. Este é a **TINTA** de uma forma — uma arte
//! repetida num reticulado, dentro dela.
//!
//! # Esta secção é a única porta do produto para afinar um padrão
//!
//! Sem ela o padrão nasce e **fica como nasceu**: o motor existiria, gateado e smokado, e o artista
//! teria uma imagem repetida que não consegue tocar. Foi exactamente esse o buraco entre a W4 e um
//! produto.
//!
//! # Ela só sobe quando há um padrão
//!
//! `current_texture_pattern()` devolve `None` para toda forma que não tem um — e então o cabeçalho
//! nem aparece. É a lei do `Join Selected Bodies` e da secção do Pattern on Path: *um controlo que
//! não se aplica é ruído, e um botão que recusa é pior que um botão que falta.*

use super::*;

/// Os quatro reticulados, na ordem em que o artista os pensa: o neutro, os dois tijolos, a colmeia.
const TILES: [(usize, &str); 4] = [(0, "Grid"), (1, "Brick"), (2, "Column"), (3, "Hex")];
/// As três leis de repetição.
const MODES: [(usize, &str); 3] = [(0, "Tile"), (1, "Mirror"), (2, "Clamp")];

/// Os dois sujeitos, e o cabeçalho de cada um. ⚠️ Índice = o `slot` da família de ids.
const SECOES: [(ph2d_a11y::NodeId, &str); 2] = [
    (ids::VECTOR_SECTION_TEXPAT, "panel.vector.section.texpat"),
    (
        ids::VECTOR_SECTION_TEXPAT_STROKE,
        "panel.vector.section.texpat_stroke",
    ),
];

/// O id do controlo `knob` da secção da tinta `slot` — o atalho local da fábrica do editor-core.
#[must_use]
pub fn kid(slot: usize, knob: ids::TexPatKnob) -> ph2d_a11y::NodeId {
    ids::texpat_id(slot, knob)
}

/// ⭐⭐ **A gémea: que `(tinta, controlo)` este id nomeia** (`None` se não é da secção).
///
/// ⚠️ **Uma porta, três consumidores** — o `populate` regista por aqui, o `event_clicks` deixa
/// passar por aqui e a shell resolve o SLOT por aqui. Três listas escritas à mão divergiriam no
/// primeiro knob novo, e a que o artista vê é a que envelhece.
#[must_use]
pub fn texpat_knob_of(id: ph2d_a11y::NodeId) -> Option<(usize, ids::TexPatKnob)> {
    (0..ids::TEXPAT_SLOTS).find_map(|slot| {
        ids::TexPatKnob::ALL
            .iter()
            .find(|k| kid(slot, **k) == id)
            .map(|k| (slot, *k))
    })
}

impl BodyCtx<'_> {
    /// Secção **PATTERN** da tinta `slot` — a lei do padrão de textura da forma selecionada.
    ///
    /// ⭐⭐ **UMA função, dois sujeitos** (plano 35, wave F; Enio 2026-08-28: *"cada seção deve ter
    /// seus ajustes próprios"*). O plano §2.4 recusava duplicar a secção — *"onze fileiras a
    /// dobrar, e as duas divergiriam no primeiro knob novo"* — e a recusa estava certa sobre o
    /// **CÓDIGO** e errada sobre a **UI**: um alvo escondido num chip faz o artista mexer num knob
    /// e ver o outro sujeito mudar. ⇒ a UI duplica, o código não, e a divergência morre no tipo.
    pub(crate) fn texture_pattern_section(&mut self, slot: usize, y: f32) -> f32 {
        let Some(p) = state::current_texture_pattern(slot) else {
            return y;
        };
        let (sec_id, sec_label) = SECOES[slot.min(SECOES.len() - 1)];
        let (mut y, collapsed) = self.section_header(sec_id, tr(sec_label), y);
        if collapsed {
            return y;
        }
        let kid = |k| kid(slot, k);
        // ⭐⭐⭐ **A ARTE SUMIU** — dito ANTES dos dois botões que a repõem. Ver [`Self::missing_art_hint`].
        y = self.missing_art_hint(p.art_missing, y);
        // A ARTE — trocar a imagem sem trocar a lei. ⚠️ O mesmo botão que o chip *Pattern* aciona
        // quando a forma ainda não tem padrão: uma porta, dois gatilhos.
        y = self.action_button(kid(ids::TexPatKnob::Source), "Source...", y);
        // ⭐ **A ARTE pode ser uma FORMA do documento** (W7) — o modelo do Figma. O gesto é o de
        // duas mãos que a casa já tem: aperta, e o clique seguinte no canvas escolhe.
        y = self.action_button(kid(ids::TexPatKnob::PickShape), "Use Shape...", y);

        // ⭐⭐ **UM PARÂMETRO QUE O MODO NÃO USA NÃO APARECE** (Enio, 2026-08-27).
        //
        // No `Clamp` há **uma** cópia, enquadrada na forma: o reticulado, o desfasamento, o tamanho
        // e o vão não têm quem os leia. Mostrá-los seria oferecer quatro knobs mortos — o defeito
        // que o [doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md) catalogou
        // dezanove vezes, e que já custou a esta secção o Offset na colmeia.
        //
        // ⚠️ **Esconder NÃO é apagar**: a lei fica no documento e volta inteira ao sair do `Clamp`
        // — é o par da decisão de o enquadramento ser DERIVADO e nunca escrito.
        let repete = p.mode != 2;
        if repete {
            // O RETICULADO.
            let tiles: Vec<(ph2d_a11y::NodeId, &str, bool)> = TILES
                .iter()
                .map(|(i, l)| {
                    (
                        kid(ids::TexPatKnob::Tile(*i as u8)),
                        *l,
                        usize::from(p.kind) == *i,
                    )
                })
                .collect();
            y = self.segmented("Tile", &tiles, y);
        }

        // O DESFASAMENTO — só com Brick/Column. ⚠️ Na grade ele não tem sentido, e na COLMEIA ele é
        // **fixo** em meio passo (é isso que a torna colmeia): oferecê-lo ali seria um knob que o
        // modelo ignora.
        if repete && matches!(p.kind, 1 | 2) {
            let denom = self.live_number(kid(ids::TexPatKnob::OffsetNum), p.offset_denom);
            let track = self.live_track(kid(ids::TexPatKnob::Offset), denom_track(p.offset_denom));
            y = self.slider_row(
                "Offset",
                kid(ids::TexPatKnob::Offset),
                kid(ids::TexPatKnob::OffsetNum),
                track,
                denom,
                &format!("1/{}", denom.round() as i64),
                y,
            );
        }

        // O TAMANHO e o VÃO — os dois só existem enquanto o padrão REPETE.
        if repete {
            // ⭐⭐ O TAMANHO, **os DOIS eixos** (Enio, 2026-08-27: poder achatar a arte de
            // propósito). Era um número só — o lado maior, com o aspecto sempre preservado.
            //
            // ⚠️ **A protecção não desapareceu, mudou de lei imposta para gesto escolhido**: o
            // cadeado nasce LIGADO, e com ele mexer num eixo leva o outro pelo mesmo factor. Ele
            // preserva a razão **ACTUAL** e não a natural da arte — voltar ao aspecto da imagem
            // desfaria o achatamento que o artista acabou de autorar.
            for (axis, label, sid, nid) in [
                (
                    0usize,
                    "Width",
                    kid(ids::TexPatKnob::Width),
                    kid(ids::TexPatKnob::WidthNum),
                ),
                (
                    1,
                    "Height",
                    kid(ids::TexPatKnob::Height),
                    kid(ids::TexPatKnob::HeightNum),
                ),
            ] {
                let v = self.live_number(nid, p.size[axis]);
                let track = self.live_track(sid, size_track(p.size[axis]));
                y = self.slider_row(label, sid, nid, track, v, &format!("{v:.2}"), y);
            }
            // ⚠️ O cadeado vem DEPOIS dos dois números que ele liga — ele descreve o que acontece
            // *àquelas duas linhas*, e um controlo que descreve o que está acima dele lê-se onde
            // está.
            y = self.checkbox_row(
                kid(ids::TexPatKnob::Lock),
                tr("panel.vector.texpat.lock"),
                p.lock_aspect,
                y,
            );

            // O VÃO — bipolar; negativo é a sobreposição.
            let gap = self.live_number(kid(ids::TexPatKnob::GapNum), p.gap);
            let gap_track = self.live_track(kid(ids::TexPatKnob::Gap), gap_track(p.gap));
            y = self.slider_row(
                "Gap",
                kid(ids::TexPatKnob::Gap),
                kid(ids::TexPatKnob::GapNum),
                gap_track,
                gap,
                &format!("{gap:.2}"),
                y,
            );
        }

        // ⭐ A POSIÇÃO — onde, dentro de uma repetição, a arte começa.
        //
        // ⚠️ Ela vive aqui porque as três alças de canvas do W6 foram RETIRADAS (Enio, 2026-08-27:
        // *"não ficou legal. vamos retirar e deixar os ajustes apenas no painel"*). O tamanho e a
        // rotação já tinham fileira; a posição não tinha nenhuma, e sem estas duas retirar as alças
        // teria tirado ao artista uma coisa que ele fazia.
        //
        // ⚠️ Dentro do `repete` pela mesma lei dos outros: no `Clamp` a colocação é DERIVADA (uma
        // cópia enquadrada na forma) e a fase não tem quem a leia.
        if repete {
            for (axis, label, sid, nid) in [
                (
                    0usize,
                    "Shift X",
                    kid(ids::TexPatKnob::ShiftX),
                    kid(ids::TexPatKnob::ShiftXNum),
                ),
                (
                    1,
                    "Shift Y",
                    kid(ids::TexPatKnob::ShiftY),
                    kid(ids::TexPatKnob::ShiftYNum),
                ),
            ] {
                let pct = self.live_number(nid, p.shift_pct[axis]);
                let track = self.live_track(sid, shift_track(p.shift_pct[axis]));
                y = self.slider_row(label, sid, nid, track, pct, &format!("{pct:.0}%"), y);
            }
        }

        // O ÂNGULO do PADRÃO (não o da forma). ⚠️ Ele vale em TODOS os modos: no `Clamp` roda a
        // cópia enquadrada.

        let angle = self.live_number(kid(ids::TexPatKnob::AngleNum), p.angle_deg);
        let angle_track = self.live_track(kid(ids::TexPatKnob::Angle), angle_track(p.angle_deg));
        y = self.slider_row(
            "Angle",
            kid(ids::TexPatKnob::Angle),
            kid(ids::TexPatKnob::AngleNum),
            angle_track,
            angle,
            &format!("{angle:.0}"),
            y,
        );

        // A REPETIÇÃO.
        let modes: Vec<(ph2d_a11y::NodeId, &str, bool)> = MODES
            .iter()
            .map(|(i, l)| {
                (
                    kid(ids::TexPatKnob::Mode(*i as u8)),
                    *l,
                    usize::from(p.mode) == *i,
                )
            })
            .collect();
        let y = self.segmented("Repeat", &modes, y);
        self.texpat_seam_hint(p.mode, p.wrap_seam_visible, y)
    }

    /// ⭐⭐⭐ **O app MEDIU que esta arte não encaixa consigo própria, e diz-lo** (plano 33, W10).
    ///
    /// # Porque isto existe
    ///
    /// Um ladrilho cujo salto na volta passa o joelho medido mostra **uma aresta dura em cada
    /// fronteira**. O artista vê-a e não tem como saber que a causa é a arte, nem que o remédio
    /// está no chip mesmo acima. *Uma ferramenta que ignora em silêncio é pior que uma que recusa.*
    ///
    /// # ⚠️ Porque só no `Tile`
    ///
    /// A dica fica **debaixo do controlo que a resolve**, e só quando ela tem sujeito:
    /// - `Tile` (`0`) — as cópias encostam cruas ⇒ o salto vê-se ⇒ **fala**;
    /// - `Mirror` (`1`) — cada repetição é o espelho da anterior, então a junta **fecha por
    ///   construção** (medido: salto `0`, costura `0`) ⇒ cala, e é ele o remédio que a frase aponta;
    /// - `Clamp` (`2`) — há **uma** cópia e não há junta nenhuma ⇒ cala.
    ///
    /// ⛔ Um aviso que aparece no modo que o cura ensinaria o artista a ignorá-lo.
    /// ⭐⭐⭐ **A forma que servia de arte foi apagada, e o painel diz-lo** (plano 33, W11).
    ///
    /// # Porque isto existe
    ///
    /// Sem ela a estampa volta a **cor chapada** — indistinguível de um preenchimento sólido que
    /// alguém escolheu de propósito. A secção sobe inteira, com reticulado, tamanho, vão e rotação
    /// a oferecerem-se, e **nenhum deles tem efeito**: não há ladrilho para arrumar.
    ///
    /// ⚠️ **Ela vem ANTES dos dois botões de arte, e é por isso que fica no topo.** *Source…* e
    /// *Use Shape…* são a reparação; ler o problema imediatamente acima do gesto que o resolve é o
    /// que separa um aviso útil de uma queixa. ⛔ Pô-la no fim mandaria o artista procurar.
    ///
    /// ⚠️ Um pincel na mesma situação **não** ganha aviso: lá o `art` é um `Option`, e a casa já
    /// decidiu que *"um id que aponta para uma forma apagada é um pincel sem arte"* — o rótulo do
    /// botão muda para *"Pick Shape…"* e diz o estado sozinho. Uma estampa não tem essa saída,
    /// porque o `PatternSource` **não tem variante vazia**.
    fn missing_art_hint(&mut self, sumiu: bool, y: f32) -> f32 {
        if sumiu {
            return self.hint_line(tr("panel.vector.texpat.art_missing.hint"), y);
        }
        y
    }

    fn texpat_seam_hint(&mut self, mode: u8, visivel: bool, y: f32) -> f32 {
        if mode == 0 && visivel {
            return self.hint_line(tr("panel.vector.texpat.seam.hint"), y);
        }
        y
    }
}

/// O track `0..1` de um tamanho de mundo. ⚠️ O MESMO mapa que o `event` e o `populate` usam.
#[must_use]
pub(crate) fn size_track(size: f64) -> f32 {
    (((size - crate::TEXPAT_SIZE_MIN) / (crate::TEXPAT_SIZE_MAX - crate::TEXPAT_SIZE_MIN))
        .clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de um vão bipolar (`0.5` = encostado).
#[must_use]
pub(crate) fn gap_track(gap: f64) -> f32 {
    (((gap + crate::TEXPAT_GAP_MAX) / (2.0 * crate::TEXPAT_GAP_MAX)).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de uma fase em percentagem (unipolar `0..100`). ⚠️ O MESMO mapa que o `event` e o
/// `populate` usam — a fronteira única.
#[must_use]
pub(crate) fn shift_track(pct: f64) -> f32 {
    ((pct / crate::TEXPAT_SHIFT_MAX).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de um ângulo em graus (unipolar `0..360`).
#[must_use]
pub(crate) fn angle_track(deg: f64) -> f32 {
    ((deg / crate::TEXPAT_ANGLE_MAX).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` do denominador do desfasamento.
#[must_use]
pub(crate) fn denom_track(n: f64) -> f32 {
    (((n - crate::TEXPAT_DENOM_MIN) / (crate::TEXPAT_DENOM_MAX - crate::TEXPAT_DENOM_MIN))
        .clamp(0.0, 1.0)) as f32
}
