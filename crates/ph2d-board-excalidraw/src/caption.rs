//! W23 B5(2) — **os textos ao longo da linha de uma seta** no `.excalidraw` (doc 14 · doc 07 §rótulo).
//! O Excalidraw prende UM texto a uma seta: o nosso 1.º vai preso (com o `labelPosition` do next, a
//! fracção dele), os outros vão SOLTOS no sítio onde os desenhamos, marcados com o dono e o índice —
//! lá não seguem a seta; ao voltar, cada um regressa ao seu lugar na nossa.

use ph2d_board_model::{Board, Caption, Connector, Element, TextTurn};
use ph2d_board_route::{LABEL_WRAP, Routed};
use serde_json::{Value, json};

use crate::PH2D;
use crate::style::{Text, num};

/// Os textos de `c` (sobre a rota `r`) como elementos `text`; o 1.º preso a `me`.
pub(crate) fn write(
    board: &Board,
    el: &Element,
    c: &Connector,
    r: &Routed,
    me: &str,
) -> Vec<Value> {
    let st = &c.style;
    c.captions
        .iter()
        .enumerate()
        .map(|(i, cap)| {
            let f = r.caption_frame(cap.at, c.turn);
            let rect = [
                f.center[0] - LABEL_WRAP / 2.0,
                f.center[1] - st.font_size,
                LABEL_WRAP,
                2.0 * st.font_size,
            ];
            let bound = (i == 0).then_some(me);
            let mut t = crate::write::text(board, el, &cap.text, st, rect, bound, &[]);
            if i == 0 {
                // Sempre: sem ele o next põe o texto no meio dos PONTOS, não do comprimento (lei 12); o
                // 0.18.1 guarda-o sem o usar (lei 13, `label_oracle`).
                t["labelPosition"] = json!(cap.at);
            } else {
                t["id"] = json!(format!("{me}-text-{i}"));
                t["angle"] = json!(f.angle);
                t["customData"] = json!({ PH2D: { "caption_of": [el.id.0, i] } });
            }
            t
        })
        .collect()
}

/// A seta de um build de antes do formato 15 (o JSON com `label`): o rótulo fica UM texto a meio,
/// deitado na horizontal como o artista o via.
pub(crate) fn old_label(el: &mut Value) {
    // `get_mut`, nunca `[..]`: o índice de um `Value` cria a chave que falta (e o elemento já não lê).
    let Some(c) = el.get_mut("kind").and_then(|k| k.get_mut("Connector")) else {
        return;
    };
    let Some(label) = c.get("label").and_then(Value::as_str).map(str::to_owned) else {
        return;
    };
    if c.get("captions").is_none() {
        let caps = Connector::captions_of(label);
        c["captions"] = serde_json::to_value(caps).unwrap_or(Value::Null);
        c["turn"] = serde_json::to_value(TextTurn::Level).unwrap_or(Value::Null);
    }
}

/// O dono (o número do id) e o índice de um texto solto que é de uma seta nossa.
pub(crate) fn owner_of(v: &Value) -> Option<(u64, usize)> {
    let o = &v["customData"][PH2D]["caption_of"];
    Some((o[0].as_u64()?, usize::try_from(o[1].as_u64()?).ok()?))
}

/// A fracção do texto preso (o `labelPosition` do next; `null` = a meio).
pub(crate) fn label_position(v: &Value) -> Option<f64> {
    num(v, "labelPosition").map(|p| p.clamp(0.0, 1.0))
}

/// O texto preso a uma seta de FORA (pelos pontos `pts` dela): um texto, deitado na horizontal como
/// ele o desenha, na fracção do `labelPosition` ou, sem ele, no meio dos pontos.
pub(crate) fn foreign(c: &mut Connector, bound: Option<&Text>, pts: &[[f64; 2]]) {
    c.turn = TextTurn::Level;
    c.captions = bound
        .map(|t| {
            vec![Caption::new(
                t.text.clone(),
                t.at.unwrap_or_else(|| middle_of_points(pts)),
            )]
        })
        .unwrap_or_default();
}

/// ⭐ Onde o Excalidraw põe o texto de uma seta sem `labelPosition` (o 0.18.1 sempre; o next com
/// `null`), em fracção do comprimento: o ponto do MEIO se são ímpares, senão o meio do trecho do meio
/// (`label_oracle`, leis 12–13: a seta de 3 pontos assimétrica dá 0,75, não 0,5).
pub(crate) fn middle_of_points(pts: &[[f64; 2]]) -> f64 {
    let lens: Vec<f64> = pts
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect();
    let total: f64 = lens.iter().sum();
    let n = pts.len();
    if n < 2 || total <= 0.0 {
        return Caption::MIDDLE;
    }
    let upto = if n % 2 == 1 {
        lens[..n / 2].iter().sum::<f64>()
    } else {
        lens[..n / 2 - 1].iter().sum::<f64>() + lens[n / 2 - 1] / 2.0
    };
    upto / total
}

/// O que se mudou LÁ nos textos de uma seta nossa: o texto de cada um, e a fracção do preso.
pub(crate) fn edits(c: &mut Connector, bound: Option<&Text>, loose: &[(usize, Text)]) {
    if let Some(t) = bound {
        match c.captions.first_mut() {
            Some(cap) => {
                if cap.text != t.text {
                    cap.text.clone_from(&t.text);
                }
                if let Some(at) = t.at.filter(|a| (a - cap.at).abs() > 1e-9) {
                    cap.at = at;
                }
            }
            None => c.captions.push(Caption::new(
                t.text.clone(),
                t.at.unwrap_or(Caption::MIDDLE),
            )),
        }
    }
    for (i, t) in loose {
        if let Some(cap) = c.captions.get_mut(*i).filter(|cap| cap.text != t.text) {
            cap.text.clone_from(&t.text);
        }
    }
    c.drop_empty_captions();
}
