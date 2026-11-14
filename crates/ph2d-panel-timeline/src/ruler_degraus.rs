//! ⭐ **Os riscos da régua pelo zoom** (dono 13/11: «poucas marcações e quase invisíveis»): três
//! degraus — o rotulado (com número), o do meio e o miúdo (um quadro, quando cabe) —, escolhidos pelo
//! espaço em píxeis, nunca fixos em segundos. Cada degrau menor divide o maior, para os riscos caírem
//! uns sobre os outros.

/// O espaço mínimo entre dois números da régua.
const ROTULO_MIN_PX: f64 = 90.0; // LITERAL-PX-OK: «0.75» com folga de um número ao seguinte
/// O espaço mínimo entre dois riscos miúdos (abaixo disto viram cinza corrido).
const MIUDO_MIN_PX: f64 = 6.0; // LITERAL-PX-OK: um risco de 1 px com 5 px de vão
/// O espaço mínimo entre dois riscos do meio.
const MEIO_MIN_PX: f64 = 24.0; // LITERAL-PX-OK: o meio só se distingue do miúdo com vão largo

/// Os passos com número, em segundos, do mais fino ao mais largo.
const ROTULOS: [f64; 13] = [
    0.1,   // LITERAL-PX-OK: segundos (tempo), não medida
    0.25,  // LITERAL-PX-OK: segundos
    0.5,   // LITERAL-PX-OK: segundos
    1.0,   // LITERAL-PX-OK: segundos
    2.0,   // LITERAL-PX-OK: segundos
    5.0,   // LITERAL-PX-OK: segundos
    10.0,  // LITERAL-PX-OK: segundos
    15.0,  // LITERAL-PX-OK: segundos
    30.0,  // LITERAL-PX-OK: segundos
    60.0,  // LITERAL-PX-OK: segundos
    120.0, // LITERAL-PX-OK: segundos
    300.0, // LITERAL-PX-OK: segundos
    600.0, // LITERAL-PX-OK: segundos
];

/// Os passos dos riscos, em segundos.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Degraus {
    pub(crate) rotulo: f64,
    pub(crate) meio: Option<f64>,
    pub(crate) miudo: Option<f64>,
}

/// `a` é múltiplo inteiro de `b`?
fn divide(a: f64, b: f64) -> bool {
    let q = a / b;
    q >= 1.0 && (q - q.round()).abs() < 1e-6
}

/// Os degraus para `px_por_s` píxeis por segundo a `fps` quadros por segundo.
pub(crate) fn degraus(px_por_s: f64, fps: f64) -> Degraus {
    let px = |s: f64| s * px_por_s;
    let rotulo = ROTULOS
        .into_iter()
        .find(|&s| px(s) >= ROTULO_MIN_PX)
        .unwrap_or(ROTULOS[ROTULOS.len() - 1]);
    let quadro = (fps > 0.0).then(|| 1.0 / fps);
    // O miúdo: o quadro se cabe e divide o rotulado; senão a fração mais fina que cabe.
    let miudo = quadro
        .into_iter()
        .chain([10.0, 5.0, 4.0, 2.0].map(|n| rotulo / n)) // LITERAL-PX-OK: divisores do passo (contagem)
        .find(|&s| px(s) >= MIUDO_MIN_PX && divide(rotulo, s));
    let meio = Some(rotulo / 2.0)
        .filter(|&s| px(s) >= MEIO_MIN_PX && miudo.is_none_or(|m| m < s && divide(s, m)));
    Degraus {
        rotulo,
        meio,
        miudo,
    }
}

/// O número de um risco rotulado: inteiro sem casas, fração sem zeros a mais («0.25», «1.5»).
pub(crate) fn texto(t: f64) -> String {
    if (t - t.round()).abs() < 1e-6 {
        return format!("{}", t.round() as i64);
    }
    let s = format!("{t:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ No zoom de fábrica (~480 px/s a 24 fps) há número a cada ¼ s e um risco por quadro; os
    /// degraus dividem-se. *Mutação: o miúdo fixo em 0,5 s (a régua de antes) ⇒ vermelho.*
    #[test]
    fn no_zoom_de_fabrica_ha_um_risco_por_quadro() {
        let d = degraus(480.0, 24.0);
        assert_eq!(d.rotulo, 0.25);
        assert_eq!(d.miudo, Some(1.0 / 24.0));
        assert!(
            d.meio
                .is_some_and(|m| divide(d.rotulo, m) && divide(m, 1.0 / 24.0))
        );
    }

    /// ⭐ Em qualquer zoom: os números não se pisam, os miúdos não viram cinza corrido, e cada degrau
    /// divide o de cima. *Mutação: tirar o `divide` do miúdo ⇒ vermelho a 30 fps.*
    #[test]
    fn em_qualquer_zoom_os_degraus_cabem_e_se_dividem() {
        for fps in [12.0, 24.0, 25.0, 30.0, 60.0] {
            for pps in [4.0, 15.0, 40.0, 100.0, 250.0, 480.0, 1200.0, 4000.0] {
                let d = degraus(pps, fps);
                assert!(
                    d.rotulo * pps >= ROTULO_MIN_PX || d.rotulo == 600.0,
                    "{fps} {pps}"
                );
                if let Some(m) = d.miudo {
                    assert!(
                        m * pps >= MIUDO_MIN_PX && divide(d.rotulo, m),
                        "{fps} {pps} {d:?}"
                    );
                }
                if let Some(m) = d.meio {
                    assert!(divide(d.rotulo, m), "{fps} {pps} {d:?}");
                }
            }
        }
    }

    /// ⭐⭐ Dono 13/11: «quase invisíveis». Nos cartões, todo risco ≥ 3:1 contra a régua (WCAG 1.4.11)
    /// em todo tema; medido antes: o traço `ReguaTraco` dava 1,3:1. *Mutação: o fraco = `ReguaTraco` ⇒
    /// vermelho.*
    #[test]
    fn os_riscos_se_veem_contra_a_regua_em_todo_tema() {
        use ph2d_editor_core::panel::cards::ferramenta;
        let rgb = |c: ph2d_vector::Color| {
            let [r, g, b, _] = c.to_rgba8().to_u8_array();
            ph2d_tokens::Color::from_hex(u32::from_be_bytes([0, r, g, b]))
        };
        ferramenta::force_cards_on_this_thread(Some(true));
        for theme in ph2d_tokens::Theme::ALL {
            let fundo = rgb(crate::folha_tinta::tinta(
                ph2d_tokens::TimelineTinta::FitaRegua,
                theme,
            ));
            let (forte, fraco) = super::super::tons_dos_riscos(theme);
            for (nome, cor) in [("forte", forte), ("fraco", fraco)] {
                let r = rgb(cor).contrast_ratio(&fundo);
                assert!(r >= 3.0, "{theme:?}: {nome} {r:.2}:1");
            }
        }
        ferramenta::force_cards_on_this_thread(None);
    }

    #[test]
    fn o_numero_da_regua_nao_tem_zeros_a_mais() {
        assert_eq!(texto(2.0), "2");
        assert_eq!(texto(0.25), "0.25");
        assert_eq!(texto(1.5), "1.5");
        assert_eq!(texto(0.1 * 3.0), "0.3");
    }
}
