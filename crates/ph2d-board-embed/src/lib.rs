//! W18 ([ADR-0185]) — **a lei do vídeo no quadro**: que texto é um link de um vídeo do YouTube ([`youtube`]),
//! o endereço do tocador oficial ([`player_url`]) e da miniatura ([`thumbnail_url`]), o tamanho com que nasce
//! ([`natural_size`]) e quando o tocador pode estar VIVO no canvas ([`live_allowed`], as regras do YouTube).
//!
//! A tabela das formas é a do oráculo (Excalidraw 0.18.1, `excalidraw_oracle/saidas/w18`), com os defeitos
//! dele corrigidos: o `t=1m30s` vira segundos (o embed só os entende), `/live/<id>` e `?feature=share&v=<id>`
//! são vídeos, o domínio em maiúsculas também, e o id tem de ter os 11 caracteres do YouTube.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use ph2d_board_model::Video;

/// O lado mínimo, em píxeis de ecrã, de um tocador do YouTube visível (*Required Minimum Functionality*).
pub const MIN_LIVE_SIDE: f64 = 200.0;

/// O tamanho com que um vídeo nasce no quadro (o do oráculo: 560×315, 16:9; um `/shorts/` 315×560).
const LANDSCAPE: [f64; 2] = [560.0, 315.0];

/// ⭐ O vídeo do YouTube que `text` é, ou `None` (o texto fica texto). O texto inteiro tem de ser o link
/// (com espaços à volta): uma frase com um link dentro não é um vídeo (oráculo `Y22`).
#[must_use]
pub fn youtube(text: &str) -> Option<Video> {
    let t = text.trim();
    if t.is_empty() || t.chars().any(char::is_whitespace) {
        return None;
    }
    let rest = strip_scheme(t);
    let (host, path_q) = rest.split_once('/').unwrap_or((rest, ""));
    let host = host.to_ascii_lowercase();
    let host = host.split(':').next().unwrap_or_default();
    let (path, query) = path_q.split_once('?').unwrap_or((path_q, ""));
    let (path, frag) = path.split_once('#').unwrap_or((path, ""));
    let (query, qfrag) = query.split_once('#').unwrap_or((query, ""));
    let param = |k: &str| {
        query
            .split('&')
            .chain(frag.split('&'))
            .chain(qfrag.split('&'))
            .find_map(|kv| kv.split_once('=').filter(|(a, _)| *a == k).map(|(_, v)| v))
    };
    let seg: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let (id, vertical) = match host {
        "youtu.be" | "www.youtu.be" => (*seg.first()?, false),
        "youtube.com"
        | "www.youtube.com"
        | "m.youtube.com"
        | "music.youtube.com"
        | "youtube-nocookie.com"
        | "www.youtube-nocookie.com" => match seg.as_slice() {
            ["watch"] => (param("v")?, false),
            ["shorts", id] => (*id, true),
            ["embed" | "live" | "v", id] => (*id, false),
            _ => return None,
        },
        _ => return None,
    };
    if !is_id(id) {
        return None;
    }
    let start = param("t").or_else(|| param("start")).map_or(0, seconds);
    Some(Video::YouTube {
        id: id.to_owned(),
        start,
        vertical,
    })
}

fn strip_scheme(t: &str) -> &str {
    for s in ["https://", "http://", "HTTPS://", "HTTP://"] {
        if let Some(r) = t.strip_prefix(s) {
            return r;
        }
    }
    t
}

/// Os 11 caracteres de um id do YouTube (`[A-Za-z0-9_-]`).
fn is_id(id: &str) -> bool {
    id.len() == 11
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// `90`, `90s`, `1m30s`, `1h2m3s` → segundos; o que não se lê é o começo.
fn seconds(t: &str) -> u32 {
    if let Ok(n) = t.parse::<u32>() {
        return n;
    }
    let (mut total, mut num) = (0u32, 0u32);
    for c in t.chars() {
        match c {
            '0'..='9' => {
                num = num
                    .saturating_mul(10)
                    .saturating_add(c.to_digit(10).unwrap_or(0))
            }
            'h' => (total, num) = (total.saturating_add(num.saturating_mul(3600)), 0),
            'm' => (total, num) = (total.saturating_add(num.saturating_mul(60)), 0),
            's' => (total, num) = (total.saturating_add(num), 0),
            _ => return 0,
        }
    }
    total.saturating_add(num)
}

/// O link canónico de um vídeo (o que se grava e o que «Abrir no navegador» abre).
#[must_use]
pub fn canonical_link(v: &Video) -> String {
    match v {
        Video::YouTube {
            id,
            start,
            vertical: true,
        } if *start == 0 => {
            format!("https://www.youtube.com/shorts/{id}")
        }
        Video::YouTube { id, start: 0, .. } => format!("https://www.youtube.com/watch?v={id}"),
        Video::YouTube { id, start, .. } => {
            format!("https://www.youtube.com/watch?v={id}&t={start}s")
        }
    }
}

/// O tocador OFICIAL (o domínio sem cookies do YouTube), a começar no início gravado; `enablejsapi` deixa
/// a página que o leva pausá-lo e retomá-lo pela API oficial (`postMessage`).
#[must_use]
pub fn player_url(v: &Video) -> String {
    match v {
        Video::YouTube { id, start, .. } => format!(
            "https://www.youtube-nocookie.com/embed/{id}?autoplay=1&playsinline=1&enablejsapi=1&start={start}"
        ),
    }
}

/// A miniatura do vídeo (a imagem que o YouTube mostra antes do play).
#[must_use]
pub fn thumbnail_url(v: &Video) -> String {
    match v {
        Video::YouTube { id, .. } => format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"),
    }
}

/// O tamanho com que o vídeo nasce no quadro.
#[must_use]
pub fn natural_size(v: &Video) -> [f64; 2] {
    match v {
        Video::YouTube { vertical: true, .. } => [LANDSCAPE[1], LANDSCAPE[0]],
        Video::YouTube { .. } => LANDSCAPE,
    }
}

/// Onde o vídeo está no ecrã, para decidir se o tocador pode estar vivo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnScreen {
    /// A largura e a altura do retângulo do vídeo em píxeis de ecrã.
    pub size: [f64; 2],
    /// O ângulo do elemento (rad).
    pub angle: f64,
    /// Algum elemento está à frente de alguma parte dele.
    pub covered: bool,
    /// O retângulo inteiro cabe na vista do quadro.
    pub whole: bool,
}

/// ⭐ O tocador pode estar VIVO no canvas (senão, a miniatura): as regras do YouTube para o tocador
/// embutido — nada à frente de nenhuma parte dele, a aparência dele não se modifica (sem rodar), e pelo
/// menos [`MIN_LIVE_SIDE`] × [`MIN_LIVE_SIDE`] visíveis — e inteiro à vista.
#[must_use]
pub fn live_allowed(s: OnScreen) -> bool {
    !s.covered
        && s.whole
        && s.angle == 0.0
        && s.size[0] >= MIN_LIVE_SIDE
        && s.size[1] >= MIN_LIVE_SIDE
}

#[cfg(test)]
mod tests;
