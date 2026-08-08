//! Compact **text form** of a [`ColorRamp`] — the serialization the Motion Nodes
//! text-param channel carries (doc 32: a param that is *not one f32* lives as a
//! string on the `Graph`, never in the frozen `NodeManifest`). This is the
//! gradient's exact analog of `ph2d-curve`'s `serialize`/`parse` for the Curve
//! text param, and it exists for the SAME reason: `motion.color_ramp`'s Custom
//! ramp is multi-stop, and a variable-length list of coloured stops cannot be a
//! fixed set of `f32` params.
//!
//! **Format** — mirrors `ph2d-curve`'s (`c1 x:y:interp …`):
//! ```text
//! g1 <interp_u8> <pos>:<r>,<g>,<b> <pos>:<r>,<g>,<b> …
//! ```
//! - `g1` is the version tag (a later field is a new tag, never a silent extra
//!   token — [`parse_gradient`] rejects a malformed stop rather than ignoring it).
//! - `<interp_u8>` is [`RampInterp::to_u8`] — a **global** interp for the whole
//!   ramp (Blender's Color Ramp interpolation dropdown). It lives in the STRING,
//!   not a sibling `f32` param, because the GPU LUT fill ([`LutSpec::fill`]) only
//!   ever sees this string — the interp has to travel with the stops or the
//!   device bake could not match the CPU `eval`.
//! - each stop is `pos:r,g,b` with `{}`-formatted `f32` (Rust's shortest decimal
//!   that round-trips), in **linear** RGB — the wire space the `tint` column and
//!   the compositor use.
//!
//! ## `g2` — a alfa por stop (2026-08-08)
//!
//! O smoke reportou que *"a transparência das cores não está sendo respeitada no
//! motion"*, e a causa mais funda era esta: o formato **não tinha onde guardá-la**. O
//! `g1` dropava a alfa na serialização (`let [r, g, b, _a]`) e a doc a chamava de *"um
//! campo append-only futuro"* — este é o campo.
//!
//! ```text
//! g2 <interp_u8> <pos>:<r>,<g>,<b>,<a> …
//! ```
//!
//! ⚠️ **A versão é ESCOLHIDA pelo conteúdo, não fixada:** um ramp em que todo stop é
//! opaco serializa `g1`, **byte a byte** como antes — o mesmo padrão append-only dos
//! records do formato de grafo (*um documento que ninguém mutou é byte-idêntico*). Só o
//! ramp que de fato usa a alfa paga o header novo, e `parse_gradient` aceita os dois.
//!
//! Colour mode is always [`RampColorMode::Rgb`] and hue [`RampHue::Near`] in v1
//! (the two the node evaluates); an HSV/HSL custom ramp is a future token, not a
//! reinterpretation of an old string.

use crate::color_ramp::{ColorRamp, RampColorMode, RampInterp, RampStop};

/// Serialize a ramp to the compact text form (the inverse of [`parse_gradient`]).
///
/// Emite **`g1`** (três canais) quando todo stop é opaco e **`g2`** (quatro) quando
/// algum não é — ver o cabeçalho do módulo: é o que mantém byte-idêntico todo gradiente
/// já autorado. Colour mode/hue não são guardados.
#[must_use]
pub fn serialize_gradient(ramp: &ColorRamp) -> String {
    // ⚠️ `!= 1.0` e não `< 1.0`: uma alfa NaN ou fora de faixa também precisa viajar,
    // senão ela é silenciosamente saneada para opaca só por não caber no header velho.
    let translucent = ramp.stops().iter().any(|s| s.color[3] != 1.0);
    let mut s = format!(
        "{} {}",
        if translucent { "g2" } else { "g1" },
        ramp.interp.to_u8()
    );
    for stop in ramp.stops() {
        // `{}` on f32 is the shortest decimal that round-trips (Rust's Grisu/Ryū),
        // so parse-then-serialize is byte-stable.
        let [r, g, b, a] = stop.color;
        if translucent {
            s.push_str(&format!(" {}:{r},{g},{b},{a}", stop.pos));
        } else {
            s.push_str(&format!(" {}:{r},{g},{b}", stop.pos));
        }
    }
    s
}

/// Parse the compact text form. Returns `None` for anything malformed OR for
/// fewer than two stops — a one-stop "gradient" has nothing to interpolate, so
/// the caller falls back to a sensible default (`ColorRamp::default()`), exactly
/// as `ph2d-curve::parse` returns `None` on a degenerate curve.
#[must_use]
pub fn parse_gradient(s: &str) -> Option<ColorRamp> {
    let mut it = s.split_whitespace();
    // `g1` = três canais (alfa implícita 1.0); `g2` = quatro.
    let with_alpha = match it.next()? {
        "g1" => false,
        "g2" => true,
        _ => return None,
    };
    let interp = RampInterp::from_u8(it.next()?.parse::<u8>().ok()?);
    let mut stops = Vec::new();
    for tok in it {
        let (pos_str, rgb_str) = tok.split_once(':')?;
        let pos = pos_str.parse::<f32>().ok()?;
        let mut c = rgb_str.split(',');
        let r = c.next()?.parse::<f32>().ok()?;
        let g = c.next()?.parse::<f32>().ok()?;
        let b = c.next()?.parse::<f32>().ok()?;
        // Em `g1` um 4º canal segue MALFORMADO (não é dado extra a ignorar); em `g2` ele
        // é obrigatório — a versão diz quantos canais o stop tem, e um stop com o número
        // errado de canais é um stop que ninguém escreveu.
        let a = if with_alpha {
            c.next()?.parse::<f32>().ok()?
        } else {
            1.0
        };
        if c.next().is_some() {
            return None;
        }
        if !(pos.is_finite() && r.is_finite() && g.is_finite() && b.is_finite() && a.is_finite()) {
            return None;
        }
        stops.push(RampStop::new(pos, [r, g, b, a]));
    }
    if stops.len() < 2 {
        return None;
    }
    Some(ColorRamp::new(stops, RampColorMode::Rgb, interp))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A ALFA POR STOP SOBREVIVE AO ROUND-TRIP** — o defeito de 2026-08-08.
    ///
    /// ⚠️ Nasceu VERMELHO: o `g1` dropava a alfa na serialização, então um stop
    /// translúcido voltava opaco e a transparência era **inexprimível no formato**, que é
    /// a camada mais funda do que o smoke reportou.
    #[test]
    fn a_translucent_stop_survives_the_round_trip() {
        let ramp = ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 0.25]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 0.8]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        );
        let text = serialize_gradient(&ramp);
        assert!(
            text.starts_with("g2 "),
            "um ramp translúcido pede o header novo: {text}"
        );
        let back = parse_gradient(&text).expect("round-trip");
        assert_eq!(back.stops()[0].color[3], 0.25);
        assert_eq!(back.stops()[1].color[3], 0.8);
    }

    /// **E um ramp OPACO segue byte-idêntico ao que já shipava** — a política append-only:
    /// só quem usa a alfa paga o header novo.
    #[test]
    fn an_opaque_ramp_still_serializes_as_v1() {
        let ramp = ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        );
        assert_eq!(serialize_gradient(&ramp), "g1 2 0:1,0,0 1:0,0,1");
    }

    /// Um `g1` antigo continua legível, com a alfa implícita — e um 4º canal nele segue
    /// MALFORMADO, porque a versão é quem diz quantos canais um stop tem.
    #[test]
    fn v1_is_still_read_and_its_stop_arity_is_still_enforced() {
        let r = parse_gradient("g1 2 0:1,0,0 1:0,0,1").expect("v1 parses");
        assert_eq!(r.stops()[0].color[3], 1.0);
        assert!(parse_gradient("g1 2 0:1,0,0,0.5 1:0,0,1").is_none());
        assert!(parse_gradient("g2 2 0:1,0,0 1:0,0,1").is_none());
    }

    /// Round-trip: serialize then parse gives back the same stops + interp,
    /// bit-for-bit (the shortest-decimal `{}` form is stable).
    #[test]
    fn parse_is_the_inverse_of_serialize() {
        let ramp = ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
                RampStop::new(0.5, [0.0, 1.0, 0.0, 1.0]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Ease,
        );
        let s = serialize_gradient(&ramp);
        let back = parse_gradient(&s).expect("round-trips");
        assert_eq!(back.interp, RampInterp::Ease, "interp survives");
        assert_eq!(back.len(), 3, "stop count survives");
        for (a, b) in ramp.stops().iter().zip(back.stops()) {
            assert_eq!(a.pos, b.pos, "pos byte-stable");
            assert_eq!(a.color[..3], b.color[..3], "rgb byte-stable");
            assert_eq!(b.color[3], 1.0, "alpha is implicit 1.0");
        }
    }

    /// A serialized string that we then parse and re-serialize is byte-identical
    /// (the format is canonical — no drift on re-save).
    #[test]
    fn serialize_is_the_inverse_of_parse() {
        let s = "g1 0 0:1,0,0 0.5:0,1,0 1:0,0,1";
        let ramp = parse_gradient(s).unwrap();
        assert_eq!(serialize_gradient(&ramp), s, "canonical round-trip");
    }

    /// Malformed / degenerate strings return `None` so the caller uses its
    /// default — never a half-built ramp.
    #[test]
    fn malformed_and_degenerate_return_none() {
        assert!(parse_gradient("").is_none(), "empty");
        assert!(
            parse_gradient("c1 0 0:0,0,0 1:1,1,1").is_none(),
            "wrong tag"
        );
        assert!(parse_gradient("g1 0 0.5:0.5,0.5,0.5").is_none(), "one stop");
        assert!(
            parse_gradient("g1 0 0:1,0,0,9 1:0,0,1").is_none(),
            "a 4th channel is malformed"
        );
        assert!(
            parse_gradient("g1 0 0:1,0 1:0,0,1").is_none(),
            "a missing channel is malformed"
        );
        assert!(
            parse_gradient("g1 0 nan:1,0,0 1:0,0,1").is_none(),
            "non-finite pos rejected"
        );
    }

    /// The interp `u8` travels in the string (it is the ONLY place the GPU LUT
    /// fill can read it). A different interp yields a different string and a
    /// different parsed ramp.
    #[test]
    fn interp_rides_the_string() {
        // `RampInterp::to_u8`: Ease=0, Linear=2 (Blender's menu order).
        let linear = "g1 2 0:0,0,0 1:1,1,1";
        let ease = "g1 0 0:0,0,0 1:1,1,1";
        assert_eq!(parse_gradient(linear).unwrap().interp, RampInterp::Linear);
        assert_eq!(parse_gradient(ease).unwrap().interp, RampInterp::Ease);
        assert_ne!(linear, ease, "the interp is a distinguishing token");
    }
}
