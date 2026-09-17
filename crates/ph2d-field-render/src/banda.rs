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

/// ⭐⭐⭐ **A ESTRUTURA DE MÉDIA FREQUÊNCIA do resíduo** — a régua que apanha o que o dono chamou
/// «reflexo mal feito», e que as duas de cima NÃO apanhavam.
///
/// # ⛔ Porque os terraços a um pixel não bastaram
///
/// Uma soma de projecções deslocadas da peça (a recolha por pixel com direcções fixas) desenha
/// **ondulações de 4–8 px** sobre uma face plana — cada uma pequena, o conjunto uma imagem. A
/// segunda diferença a **um** pixel lê cada ondulação como quase nada, e o `p99` dela nem separava
/// a lei doente da sã (`0,17` contra `0,13`). *O olho integra a uma escala, e a régua tem de a ter.*
///
/// A grandeza: o resíduo `canal − referência`, ao longo de cada linha, menos a versão dele alisada
/// numa janela de `2·janela + 1` pixels — o que sobra é a parte que NÃO é desvio suave de nível.
/// Devolve o RMS disso em fracção da média da referência. Um desvio de nível lê `~0`; uma imagem
/// esborratada da peça lê alto.
#[must_use]
pub fn estrutura(
    g: &Gbuffer,
    canal: &dyn Fn(usize) -> f32,
    referencia: &dyn Fn(usize) -> f32,
    janela: usize,
) -> f32 {
    let (w, h) = (g.width as usize, g.height as usize);
    let (mut media, mut n_ref) = (0.0f64, 0usize);
    for i in 0..w * h {
        if g.hit[i] {
            media += f64::from(referencia(i));
            n_ref += 1;
        }
    }
    if n_ref == 0 {
        return 0.0;
    }
    media /= n_ref as f64;
    let (mut soma2, mut n) = (0.0f64, 0usize);
    let largura = 2 * janela + 1;
    for y in 0..h {
        // Um troço contíguo de pixels acertados de cada vez.
        let mut x = 0;
        while x < w {
            if !g.hit[y * w + x] {
                x += 1;
                continue;
            }
            let ini = x;
            while x < w && g.hit[y * w + x] {
                x += 1;
            }
            let fim = x;
            if fim - ini < largura {
                continue;
            }
            let r: Vec<f64> = (ini..fim)
                .map(|xx| f64::from(canal(y * w + xx) - referencia(y * w + xx)))
                .collect();
            for c in janela..r.len() - janela {
                let liso = r[c - janela..=c + janela].iter().sum::<f64>() / largura as f64;
                let medio = r[c] - liso;
                soma2 += medio * medio;
                n += 1;
            }
        }
    }
    if n == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let v = ((soma2 / n as f64).sqrt() / media.max(1e-9)) as f32;
    v
}
