//! **O perfil de LARGURA** do traço (ADR-0145 · plano 25 §5) — irmão de [`super`] pelo teto de
//! 700 LOC, e o corte é por responsabilidade: aqui mora tudo que responde *"que forma a largura
//! tem ao longo do caminho?"* — a faixa dos multiplicadores, o mapa slider↔multiplicador, o
//! default que os knobs mostram, e o **catálogo de perfis nomeados** (W2b).
//!
//! O que fica no pai é o vocabulário de ESTILO (cor, largura única, pontas, dash): um traço tem
//! largura mesmo sem perfil, e o perfil não sabe de cor nenhuma.

use ph2d_vec_scene::WidthProfile;

/// **Power Stroke** — os multiplicadores do perfil de largura.
///
/// `0` é alcançável e significa **sem tinta ali** (o traço vira um bico, que é o desenho a
/// nanquim que a feature existe para dar). O teto `3` é ergonômico: o perfil MULTIPLICA a
/// largura que o artista já escolheu no slider de Width, e triplicá-la cobre o gesto de
/// engrossar sem que o slider fique inutilizável na faixa fina, que é onde ele mora.
pub const WPROFILE_MIN: f64 = 0.0;
pub const WPROFILE_MAX: f64 = 3.0;
pub const WPROFILE_SLIDER_SCALE: f32 = (WPROFILE_MAX - WPROFILE_MIN) as f32;
pub const WPROFILE_SLIDER_OFFSET: f32 = WPROFILE_MIN as f32;

/// O perfil DEFAULT do Power Stroke: afina nas duas pontas e engrossa no meio.
///
/// Não é neutro de propósito — o botão RECUSA o perfil uniforme (aí a operação é o Outline
/// Stroke), então nascer em `1·1·1` daria um botão que não faz nada no primeiro clique. Este
/// perfil é a pincelada de nanquim, que é o que a feature existe para dar.
pub const WPROFILE_DEFAULT_START: f64 = 0.25;
pub const WPROFILE_DEFAULT_MID: f64 = 1.6;
pub const WPROFILE_DEFAULT_END: f64 = 0.25;
/// Onde o ponto grosso senta, em fração de ARCO. No meio.
pub const WPROFILE_DEFAULT_POS: f64 = 0.5;

/// Os quatro acima **como perfil** — o que os sliders mostram antes de o artista tocar em nada.
///
/// Existe para que a pergunta *"que trilho este slider exibe quando o store está vazio?"* tenha
/// UMA resposta: quem pinta a fileira e quem decide se um perfil do catálogo está ACESO leem o
/// mesmo default, pela mesma [`preset_tracks`]. Quatro `unwrap_or` soltos são quatro chances de
/// um deles ficar para trás.
pub const WPROFILE_DEFAULT: WidthProfile = WidthProfile {
    start: WPROFILE_DEFAULT_START,
    mid: WPROFILE_DEFAULT_MID,
    end: WPROFILE_DEFAULT_END,
    position: WPROFILE_DEFAULT_POS,
};

/// Normalized track `0..=1` → multiplicador `MIN..=MAX`.
#[must_use]
pub fn slider_to_wprofile(track: f32) -> f64 {
    WPROFILE_MIN + f64::from(track.clamp(0.0, 1.0)) * (WPROFILE_MAX - WPROFILE_MIN)
}
/// Multiplicador → normalized track (inverse of [`slider_to_wprofile`]).
#[must_use]
pub fn wprofile_to_slider(m: f64) -> f32 {
    ((m.clamp(WPROFILE_MIN, WPROFILE_MAX) - WPROFILE_MIN) / (WPROFILE_MAX - WPROFILE_MIN)) as f32
}

/// **Os quatro trilhos que ESTE perfil é** — `[start, mid, end, position]`, na ordem em que o
/// painel pinta os sliders.
///
/// ⚠️ **Porta única, e ela é load-bearing por um motivo aritmético.** Quem ESCREVE um preset nos
/// sliders (o espelho da seleção, o clique do catálogo) e quem PERGUNTA *"qual preset está
/// aceso?"* têm de falar a mesma língua — e a língua tem de ser o **trilho**, nunca o
/// multiplicador: o ida-e-volta `slider_to_wprofile(wprofile_to_slider(1.0))` dá
/// `1.0000000298…` (o trilho é `f32`), então uma comparação em multiplicadores **nunca**
/// acenderia a linha. Em trilhos a igualdade é exata por construção, sem tolerância inventada.
#[must_use]
pub fn preset_tracks(p: &WidthProfile) -> [f32; 4] {
    [
        wprofile_to_slider(p.start),
        wprofile_to_slider(p.mid),
        wprofile_to_slider(p.end),
        // A posição é a fração crua do trilho — o domínio dela JÁ é `[0,1]`, não há faixa a
        // remapear (é o que o `preset_from_store` lê de volta).
        #[allow(clippy::cast_possible_truncation)]
        {
            p.position.clamp(0.0, 1.0) as f32
        },
    ]
}

/// **Qual perfil do catálogo estes quatro trilhos SÃO**, ou `None` se nenhum (W2b).
///
/// ⚠️ **A resposta é DERIVADA, nunca guardada.** Não existe campo *"preset corrente"* em lugar
/// nenhum do app, e é isso que mantém a fileira honesta: arrastar um slider ou uma alça do Width
/// Tool apaga todas as linhas sozinho, porque a forma deixou de ser qualquer uma delas. Um campo
/// guardado continuaria dizendo *Taper* sobre uma curva que o artista já mudou.
///
/// A comparação é exata **em trilho** — ver [`preset_tracks`] para o porquê de não poder ser em
/// multiplicador.
#[must_use]
pub fn active_preset(tracks: &[f32; 4]) -> Option<usize> {
    ph2d_vec_scene::WIDTH_PRESETS
        .iter()
        .position(|p| preset_tracks(&p.profile) == *tracks)
}
