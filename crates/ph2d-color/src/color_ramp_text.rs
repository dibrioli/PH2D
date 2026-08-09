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
//! ## `g3` — o ESPAÇO de interpolação (2026-08-09)
//!
//! O `g1`/`g2` fixavam [`RampColorMode::Rgb`] + [`RampHue::Near`] e o cabeçalho anterior
//! deste módulo já dizia o que fazer: *"an HSV/HSL custom ramp is a future token, not a
//! reinterpretation of an old string"*. Este é o token.
//!
//! ```text
//! g3 <interp_u8> <mode_u8> <hue_u8> <pos>:<r>,<g>,<b>,<a> …
//! ```
//!
//! ⚠️ **O motor SEMPRE soube interpolar em HSV/HSL** ([`ColorRamp::mix2`]/`cubic` ramificam
//! em `color_mode` desde que a crate nasceu, e o `unwrap_hues` do caminho cúbico existe só
//! para isso) — o que faltava era **onde guardar a escolha**. Um azul→amarelo em RGB passa
//! por um cinza morto no meio; em HSV ele percorre o arco de matiz e passa por verde/ciano
//! saturados. A capacidade estava construída e era **inexprimível no formato**, que é
//! exatamente a forma do defeito que o `g2` fechou para a alfa.
//!
//! ⚠️ **A versão continua ESCOLHIDA pelo conteúdo:** `g3` sai apenas quando
//! `mode != Rgb || hue != Near`. Um gradiente que ninguém levou para fora do RGB serializa
//! `g1`/`g2` **byte a byte** como antes — a política append-only dos records do formato de
//! grafo. E a régua é o que a rampa PRECISA EXPRIMIR (os campos), não o que hoje muda um
//! pixel: o matiz não pinta nada em RGB, mas guardá-lo é o que faz a escolha do artista
//! **sobreviver a um desvio** por RGB e voltar (há gate).
//!
//! ⚠️ **`g3` carrega a alfa SEMPRE**, porque a versão é quem diz a aridade do stop — a regra
//! que o `g2` instalou. A alternativa (um `g3` de três canais e um `g4` de quatro) dobraria
//! os tokens a cada campo novo; aqui cada versão nova é o **superset**, e as antigas
//! sobrevivem por causa dos documentos que já as usam.
//!
//! ⚠️ **E o dispositivo herda o espaço de graça:** o LUT da GPU é assado na CPU por
//! [`ColorRamp::bake_into`] → [`ColorRamp::eval`], pela MESMA `parse_gradient` — então não
//! há WGSL a escrever, e não há segunda expressão da lei para divergir.

use crate::color_ramp::{ColorRamp, RampColorMode, RampHue, RampInterp, RampStop};

/// Serialize a ramp to the compact text form (the inverse of [`parse_gradient`]).
///
/// A versão é a **menor que exprime a rampa** — ver o cabeçalho do módulo: **`g1`** (três
/// canais) quando todo stop é opaco, **`g2`** (quatro) quando algum não é, e **`g3`**
/// (com espaço + matiz no header) quando ela sai do RGB. É o que mantém byte-idêntico
/// todo gradiente já autorado.
#[must_use]
pub fn serialize_gradient(ramp: &ColorRamp) -> String {
    // ⚠️ `!= 1.0` e não `< 1.0`: uma alfa NaN ou fora de faixa também precisa viajar,
    // senão ela é silenciosamente saneada para opaca só por não caber no header velho.
    let translucent = ramp.stops().iter().any(|s| s.color[3] != 1.0);
    // ⚠️ A pergunta é sobre os CAMPOS, não sobre o que hoje pinta: o matiz é inerte em
    // RGB, e guardá-lo assim mesmo é o que faz a escolha do artista sobreviver a um
    // desvio por RGB (o gate `a_hue_chosen_in_hsv_survives_a_detour_through_rgb`).
    let spaced = ramp.color_mode != RampColorMode::Rgb || ramp.hue != RampHue::Near;
    let mut s = if spaced {
        // `g3` é o superset: alfa sempre, mais os dois tokens de header.
        format!(
            "g3 {} {} {}",
            ramp.interp.to_u8(),
            ramp.color_mode.to_u8(),
            ramp.hue.to_u8()
        )
    } else {
        format!(
            "{} {}",
            if translucent { "g2" } else { "g1" },
            ramp.interp.to_u8()
        )
    };
    // A aridade do stop é função da VERSÃO, não da translucidez: o `g3` é quatro canais
    // mesmo com todo stop opaco (a regra que o `g2` instalou — a versão diz a aridade).
    let four_channel = translucent || spaced;
    for stop in ramp.stops() {
        // `{}` on f32 is the shortest decimal that round-trips (Rust's Grisu/Ryū),
        // so parse-then-serialize is byte-stable.
        let [r, g, b, a] = stop.color;
        if four_channel {
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
    // `g1` = três canais (alfa implícita 1.0); `g2` = quatro; `g3` = quatro **mais** o
    // espaço de interpolação e o caminho de matiz no header.
    let (with_alpha, spaced) = match it.next()? {
        "g1" => (false, false),
        "g2" => (true, false),
        "g3" => (true, true),
        _ => return None,
    };
    let interp = RampInterp::from_u8(it.next()?.parse::<u8>().ok()?);
    // ⚠️ Os dois tokens do espaço são LIDOS antes dos stops e só existem no `g3` — um
    // header curto demais é malformado, não um `g3` com defaults implícitos: a versão é
    // quem promete os campos, exatamente como promete a aridade do stop.
    let (color_mode, hue) = if spaced {
        (
            RampColorMode::from_u8(it.next()?.parse::<u8>().ok()?),
            RampHue::from_u8(it.next()?.parse::<u8>().ok()?),
        )
    } else {
        (RampColorMode::Rgb, RampHue::Near)
    };
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
    let mut ramp = ColorRamp::new(stops, color_mode, interp);
    // ⚠️ `ColorRamp::new` não recebe o matiz (ele nasce `Near`), então ele é escrito no
    // campo público depois — e é por isso que o round-trip do `g3` tem gate próprio: um
    // construtor que ignora um campo em silêncio é como o `hue` voltaria sempre `Near`.
    ramp.hue = hue;
    Some(ramp)
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

    /// **O ESPAÇO DE INTERPOLAÇÃO SOBREVIVE AO ROUND-TRIP** — o defeito de 2026-08-09.
    ///
    /// ⚠️ Nasceu VERMELHO: `parse_gradient` cravava `RampColorMode::Rgb` e `ColorRamp::new`
    /// crava `RampHue::Near`, então uma rampa em HSV voltava em RGB e a escolha era
    /// **inexprimível no formato** — o motor sabia interpolar em HSV desde sempre.
    #[test]
    fn a_ramp_that_leaves_rgb_survives_the_round_trip() {
        let mut ramp = ColorRamp::new(
            vec![
                RampStop::new(0.0, [0.0, 0.0, 1.0, 1.0]),
                RampStop::new(1.0, [1.0, 1.0, 0.0, 1.0]),
            ],
            RampColorMode::Hsv,
            RampInterp::Linear,
        );
        ramp.hue = RampHue::Ccw;
        let text = serialize_gradient(&ramp);
        assert!(
            text.starts_with("g3 "),
            "sair do RGB pede o header novo: {text}"
        );
        let back = parse_gradient(&text).expect("round-trip");
        assert_eq!(back.color_mode, RampColorMode::Hsv);
        assert_eq!(back.hue, RampHue::Ccw);
        assert_eq!(back.interp, RampInterp::Linear, "o interp não se perde");
    }

    /// **E A CAPACIDADE CHEGA AO CONSUMIDOR** — o gate de ponta a ponta, e o único que
    /// mede o que o artista VÊ.
    ///
    /// ⚠️ O oráculo é a APARÊNCIA, não a flag: azul→amarelo em RGB cruza pelo cinza (os
    /// dois são complementares, então o meio tem R≈G≈B) e em HSV percorre o arco de matiz
    /// e continua **saturado**. A régua é `max−min` dos canais, que é a definição de
    /// saturação do HSV — vem de fora do nosso código, e é isso que a torna oráculo.
    /// Um gate que só comparasse `back.color_mode` ficaria verde com o `eval` ignorando o
    /// campo.
    #[test]
    fn the_string_carries_the_space_all_the_way_to_the_colour() {
        let stops = " 0:0,0,1,1 1:1,1,0,1";
        let sat = |s: &str| {
            let c = parse_gradient(s).expect("parses").eval(0.5);
            c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
        };
        // `RampInterp::to_u8`: Linear = 2. `RampColorMode`: Rgb = 0, Hsv = 1. `RampHue`: Near = 0.
        let rgb = sat(&format!("g1 2{}", " 0:0,0,1 1:1,1,0"));
        let hsv = sat(&format!("g3 2 1 0{stops}"));
        assert!(
            rgb < 0.05,
            "azul→amarelo em RGB tem de morrer no cinza (medido {rgb})"
        );
        assert!(
            hsv > 0.9,
            "e em HSV tem de continuar saturado (medido {hsv})"
        );
    }

    /// **UM MATIZ ESCOLHIDO EM HSV SOBREVIVE A UM DESVIO POR RGB.**
    ///
    /// ⚠️ Este gate é a REGRA de escolha de versão: ela pergunta pelos CAMPOS da rampa, não
    /// por quais deles pintam hoje. O matiz é inerte em RGB (o braço `Rgb` do `mix2` nunca
    /// chama `lerp_hue`), então uma regra por *liveness* dropava o `Ccw` aqui e o artista
    /// perdia a escolha só por ter passado pelo RGB no ciclo do botão.
    #[test]
    fn a_hue_chosen_in_hsv_survives_a_detour_through_rgb() {
        let mut ramp = parse_gradient("g3 2 1 3 0:0,0,1,1 1:1,1,0,1").expect("hsv/ccw");
        ramp.color_mode = RampColorMode::Rgb;
        let detour = serialize_gradient(&ramp);
        let back = parse_gradient(&detour).expect("round-trip");
        assert_eq!(back.color_mode, RampColorMode::Rgb);
        assert_eq!(back.hue, RampHue::Ccw, "o matiz escolhido não se perde");
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
        // ⚠️ Um `g3` sem os dois tokens de espaço é MALFORMADO, não um `g3` com defaults
        // implícitos: a versão promete os campos do header como promete a aridade do stop.
        // Sem isto, `it.next()?` comeria o primeiro STOP como se fosse o modo, e a rampa
        // parsearia com um stop a menos — em silêncio.
        assert!(
            parse_gradient("g3 2 0:0,0,1,1 1:1,1,0,1").is_none(),
            "a g3 with no space tokens is malformed"
        );
        assert!(
            parse_gradient("g3 2 1 0:0,0,1,1 1:1,1,0,1").is_none(),
            "a g3 missing the hue token is malformed"
        );
        assert!(
            parse_gradient("g3 2 1 0 0:0,0,1 1:1,1,0").is_none(),
            "g3 stops are four channels"
        );
    }

    /// **E UMA RAMPA QUE NUNCA SAIU DO RGB SEGUE BYTE-IDÊNTICA** — a política append-only,
    /// agora com três versões: só quem de fato usa o espaço paga o header maior.
    #[test]
    fn a_ramp_that_never_left_rgb_keeps_the_version_it_always_had() {
        for s in [
            "g1 2 0:1,0,0 1:0,0,1",
            "g2 2 0:1,0,0,0.25 1:0,0,1,0.8",
            "g1 0 0:1,0,0 0.5:0,1,0 1:0,0,1",
        ] {
            let ramp = parse_gradient(s).expect("parses");
            assert_eq!(serialize_gradient(&ramp), s, "canonical round-trip: {s}");
        }
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
