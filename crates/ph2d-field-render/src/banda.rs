//! ⭐⭐⭐ **A RÉGUA DOS TERRAÇOS** — o instrumento que o report do dono de 2026-09-17 obrigou a
//! existir: *«funciona, é rápido, mas é de baixa qualidade (como se fosse muitas sombras duras)»*,
//! com a foto dos arcos concêntricos dentro do vaso.
//!
//! # ⛔⛔ Porque nenhuma régua desta crate o via
//!
//! As que existiam medem **exactidão** — o sangramento de cor da caixa de Cornell contra a
//! referência convergida, e o erro da média. As duas são cegas a isto por construção: um pico que
//! está `65×` alto em `1 %` dos pixels e ligeiramente baixo nos outros **sai na média**, e a média
//! é o que elas leem. *O olho não vê médias — vê o vizinho.*
//!
//! ⇒ a grandeza é a **QUEBRA**: a segunda diferença ao longo de uma linha. Ela é cega a toda rampa
//! suave (a irradiância indirecta sobe e desce numa parede curva, e isso é sinal verdadeiro) e
//! acende exactamente onde a resposta **salta**, que é o que desenha um arco.

use crate::Gbuffer;

/// ⭐⭐⭐ **A AMPLITUDE DOS TERRAÇOS de um canal**, em unidades da própria escala dele — `(p99, máx)`.
///
/// Para cada trio horizontal de pixels que a suavização **já declarou ser a mesma superfície** (a
/// mesma guarda de normal, [`crate::OCCLUSION_BLUR_COS`]), a quebra é `|c[i-1] − 2·c[i] + c[i+1]|`.
///
/// ⚠️ **O denominador é UM número para a imagem inteira** (a média do canal sobre os pixels
/// acertados) e nunca o valor local: com um denominador local, um terraço no fundo de um furo
/// escuro lê-se infinito e a régua passa a medir a escuridão em vez da quebra.
///
/// ⚠️⚠️ **A leitura é a CAUDA.** Um terraço vive na linha onde uma direcção troca de resposta, que
/// é `~1 %` dos pixels — uma média dilui-o até ao invisível. É o mesmo «extremo global» que o
/// [`crate::OCCLUSION_REACH`] documenta, do lado oposto.
///
/// ⭐ **O controlo natural dela é a OCLUSÃO**: as duas metades do hemisfério correm o mesmo conjunto
/// de direcções, na mesma cena, no mesmo `k`, e passam pelo mesmo borrão. Uma régua que as leia lado
/// a lado não pode ser acusada de medir a cena, a câmera, o número de direcções ou o borrão.
#[must_use]
pub fn terracos(g: &Gbuffer, canal: &dyn Fn(usize) -> f32) -> (f32, f32) {
    let (w, h) = (g.width as usize, g.height as usize);
    let escala = {
        let mut soma = 0.0f64;
        let mut n = 0usize;
        for i in 0..w * h {
            if g.hit[i] {
                soma += f64::from(canal(i));
                n += 1;
            }
        }
        if n == 0 {
            return (0.0, 0.0);
        }
        #[allow(clippy::cast_possible_truncation)]
        let e = (soma / n as f64) as f32;
        e.max(1e-6)
    };
    let mesma = |a: usize, b: usize| {
        let (u, v) = (g.normal[a], g.normal[b]);
        u[0] * v[0] + u[1] * v[1] + u[2] * v[2] >= crate::OCCLUSION_BLUR_COS
    };
    let mut quebras: Vec<f32> = Vec::new();
    for y in 0..h {
        for x in 1..w.saturating_sub(1) {
            let (a, b, c) = (y * w + x - 1, y * w + x, y * w + x + 1);
            if !g.hit[a] || !g.hit[b] || !g.hit[c] || !mesma(a, b) || !mesma(b, c) {
                continue;
            }
            quebras.push((canal(a) - 2.0 * canal(b) + canal(c)).abs() / escala);
        }
    }
    if quebras.is_empty() {
        return (0.0, 0.0);
    }
    quebras.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p99 = quebras[(quebras.len() * 99) / 100];
    (p99, *quebras.last().unwrap_or(&0.0))
}
