//! ⭐⭐⭐ **O NÚMERO VIVO na row do dope-sheet** — report do Enio, 2026-09-04: *"o painel não mostra
//! as propriedades animadas (os números não mudam em tempo real com a animação)"*.
//!
//! É o que o After Effects põe na coluna de valor: ao lado do nome da propriedade, o que ela vale
//! AGORA. Com o cursor a andar, ele anda — e é assim que se vê que a curva está a chegar ao objecto.
//!
//! # De onde vem o número (e de onde NÃO vem)
//!
//! Do MUNDO, publicado pela shell em [`ph2d_timeline::TrackValues`]. ⛔ **Não** da curva desta row,
//! que o painel já sabe amostrar: um readout tirado da curva concordaria com ela mesmo quando o
//! objecto a ignora, que é exactamente o outro defeito reportado no mesmo dia (uma forma filtrada
//! ficou opaca com a curva a dizer `0`).
//!
//! # Por que um módulo irmão
//!
//! O `tracks.rs` estava a **9 linhas** do teto de 600 (HR-18) quando esta wave chegou. *A cura de
//! um teto é o corte para um irmão, nunca uma isenção* — e o corte por RESPONSABILIDADE é este: lá
//! mora a row (o twirl, os hits, os diamantes); aqui, o que se escreve na coluna de nomes.

use ph2d_editor_core::paint::paint_text_elided;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::{DisplayAngle, LengthDisplay};
use ph2d_timeline::{PropKind, TimelineViewSnapshot, TrackView};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

/// A fatia reservada ao número, no fim da coluna de nomes.
///
/// **Medida, não escolhida:** o texto mais largo que [`text`] emite tem 6 caracteres (`-12.34`), e
/// a `TypeToken::Xs` é de 11 px com o dígito do Inter a avançar ~`0,6 em` ⇒ ~40 px. É também o
/// piso que [`fits`] exige para o NOME (o dobro), então a coluna só mostra o número quando ainda
/// sobra bem mais do que isto para quem a row nomeia.
///
/// ⚠️ **A coluna de nomes cresceu por este mesmo número** ([`crate::tracks::LABEL_COL_W`]): o
/// número foi **acrescentado**, não trocado pelo nome — quem já lia *"Fade · Opacity"* inteiro
/// continua a lê-lo inteiro.
pub(crate) const VALUE_W: f32 = 40.0; // LITERAL-PX-OK: 6 digits at TypeToken::Xs (11px)

/// Os caracteres que [`VALUE_W`] leva — o orçamento do número com a unidade ([`com_unidade`]).
const VALUE_CHARS: usize = 6;

/// **Há espaço para o número sem comer o nome?** — a coluna é arrastável e o piso dela
/// ([`crate::geom`]) deixa-a chegar a 56 px, onde um número apagaria metade do nome.
///
/// ⚠️ A regra é *o nome fica com pelo menos o dobro do número*, e não um segundo número escolhido:
/// no tamanho de fábrica (176) sobram 148 px e a leitura aparece com 104 px de nome — o mesmo que
/// ele tinha antes de esta coluna existir; encolhida ao piso (56), ela sai de cena em vez de
/// espremer quem a row identifica.
pub(crate) fn fits(name_w: f32) -> bool {
    name_w >= VALUE_W * 2.0
}

/// **O número como ele se lê numa fatia de 6 caracteres.**
///
/// ⚠️ A precisão CEDE antes da largura: `1234.57` não cabe e seria cortado a `1234.…`, que perde
/// o algarismo mais significativo dos que sobram. A banda escolhe as casas decimais para o número
/// caber inteiro — *um readout cortado mente sobre a ordem de grandeza; um arredondado não*.
///
/// A banda de baixo (`< 100`) fica com as MESMAS duas casas do editor de gráfico, que é onde a
/// esmagadora maioria destes canais vive (opacidade `0..1`, escala à volta de `1`, radianos até
/// `±3,14`).
/// ⭐ **O texto da coluna do valor**: um canal discreto vale um índice, e escreve-se a PALAVRA que ele
/// nomeia, nunca o índice (o texto «nenhum» fica vazio); o resto é o número (F27 F).
/// Um comprimento ou um ângulo leva a unidade do projecto ([`com_unidade`]).
pub(crate) fn valor_da_row(
    values: &ph2d_timeline::TrackValues,
    track: &TrackView,
    v: f32,
    u: Unidades,
) -> String {
    match values.palavra(track.target.get(), v) {
        Some(p) => dita(p).to_owned(),
        None if track.prop.is_text() => String::new(),
        None => com_unidade(track.prop, f64::from(v), u).unwrap_or_else(|| text(v)),
    }
}

/// As unidades em que o artista lê: o comprimento e o ângulo do projecto (as mesmas do canvas e do Inspector).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Unidades {
    comprimento: LengthDisplay,
    angulo: DisplayAngle,
}

impl Unidades {
    pub(crate) fn of(p: &ProjectSettings) -> Self {
        Self {
            comprimento: LengthDisplay::of(p),
            angulo: p.display_angle,
        }
    }
}

/// ⭐ **O valor de um canal com grandeza** ([`ph2d_ecs::canal::unidade`]), na unidade do projecto e com o sufixo:
/// o canal guarda metros num comprimento e graus num ângulo. `None` = número nu (quem chama formata como sempre).
pub(crate) fn com_unidade(prop: PropKind, v: f64, u: Unidades) -> Option<String> {
    use ph2d_ecs::canal::{Unidade, unidade};
    let PropKind::Field(k) = prop else {
        return None;
    };
    // O grau cola-se ao número e o radiano já traz o espaço; o comprimento leva-o.
    let (n, espaco, sufixo) = match unidade(k) {
        Unidade::Nenhuma => return None,
        Unidade::Comprimento => (u.comprimento.value(v), " ", u.comprimento.suffix()),
        Unidade::Angulo => (
            u.angulo.from_radians_f64(v.to_radians()),
            "",
            u.angulo.suffix(),
        ),
    };
    // A unidade divide a MESMA fatia: a precisão cede primeiro, depois o espaço (`1234px`, nunca `1234 p…`).
    #[expect(clippy::cast_possible_truncation, reason = "a fatia mostra f32")]
    let n = n as f32;
    let resta = |e: &str| VALUE_CHARS.saturating_sub(e.chars().count() + sufixo.chars().count());
    let (num, e) = [espaco, ""]
        .into_iter()
        .find_map(|e| cabe(n, resta(e)).map(|s| (s, e)))
        .unwrap_or_else(|| (curto(n, 0), ""));
    Some(format!("{num}{e}{sufixo}"))
}

/// O número com até duas casas que cabe em `largura` caracteres, sem zeros à direita.
fn cabe(v: f32, largura: usize) -> Option<String> {
    (0..=2)
        .rev()
        .map(|casas| curto(v, casas))
        .find(|s| s.chars().count() <= largura)
}

/// `v` com `casas` decimais e sem zeros à direita.
fn curto(v: f32, casas: usize) -> String {
    let s = format!("{v:.casas$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        &s
    };
    if s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}

/// O que uma palavra diz no ecrã: a chave pela tabela de strings, o nome do artista como está.
pub(crate) fn dita(p: &ph2d_timeline::Palavra) -> &str {
    match p {
        ph2d_timeline::Palavra::Chave(c) => ph2d_i18n::tr(c),
        ph2d_timeline::Palavra::DoArtista(s) => s,
    }
}

pub(crate) fn text(v: f32) -> String {
    /// Acima desta DÉCADA a 2.ª casa decimal deixa de caber nos 6 caracteres.
    const TWO_DECIMALS_UP_TO: f32 = 100.0; // LITERAL-PX-OK: decade, not a screen measure
    /// E acima desta, nenhuma cabe.
    const ONE_DECIMAL_UP_TO: f32 = 10_000.0; // LITERAL-PX-OK: decade, not a screen measure
    let a = v.abs();
    if a < TWO_DECIMALS_UP_TO {
        format!("{v:.2}")
    } else if a < ONE_DECIMAL_UP_TO {
        format!("{v:.1}")
    } else {
        format!("{v:.0}")
    }
}

/// **O NOME e o VALOR da row, na coluna de nomes.**
///
/// Os dois numa função só porque a largura de um é o que sobra do outro — separá-los seria a
/// segunda função a decidir a mesma repartição, e a que envelhecesse escreveria por cima da outra.
///
/// `nome` chega já rotulado (`(texto, cor)`); a geometria é `(x do nome, fim da coluna, y da row)`.
/// O número é no-op quando a shell não publicou nenhum para esta row — ver
/// [`ph2d_timeline::TrackValues::get`] para as três causas.
///
/// ⚠️ **Elidido, nunca quebrado.** No `paint_text` o `max_width` é orçamento de QUEBRA: um nome um
/// pixel largo demais virava duas linhas e transbordava para a row de baixo.
pub(crate) fn paint_name_and_value(
    ctx: &mut PaintCtx,
    theme: Theme,
    snap: &TimelineViewSnapshot,
    track: &TrackView,
    nome: (&str, ColorToken),
    geo: (f32, f32, f32),
) {
    let (texto, cor) = nome;
    let (label_x, right, y) = geo;
    let all_w = (right - label_x - Spacing::Xs.px()).max(0.0);
    let mostra = fits(all_w);
    let name_w = if mostra {
        (all_w - VALUE_W - Spacing::Xs.px()).max(0.0)
    } else {
        all_w
    };
    let font = TypeToken::Sm.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        texto,
        label_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        name_w,
        resolve(cor, theme),
    );
    let Some(v) = snap.values.get(track.target.get()).filter(|_| mostra) else {
        return;
    };
    let valor = valor_da_row(&snap.values, track, v, Unidades::of(ctx.host.project()));
    let font = TypeToken::Xs.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &valor,
        right - VALUE_W - Spacing::Xs.px(),
        y + (ROW_H_PX - font) * 0.5,
        font,
        VALUE_W,
        // ⚠️ `Text3` de propósito: o nome é que identifica a row, e um número em `Text1` ao lado
        // dele passaria a ser a primeira coisa que o olho lê.
        resolve(ColorToken::Text3, theme),
    );
}

#[cfg(test)]
#[path = "tracks_value_tests.rs"]
mod tests;
