//! ⭐⭐ **A LUZ DO RENDER** — as lâmpadas do rig e o céu de estúdio, em espaço de VISTA.
//!
//! # ⛔ Nenhuma lei nova de luz
//!
//! As lâmpadas são as da `ph2d-light` (o rig é do DOCUMENTO — *«a mesma lâmpada acende a tinta ao
//! lado»*, e inventar um segundo rig aqui é o erro que aquela crate existe para impedir) e o céu é o
//! `env_ambient` dela. O que este ficheiro faz é **traduzir**, e as duas traduções têm um preço
//! escrito:
//!
//! 1. **o sinal de `y`:** o rig é autorado no referencial do CANVAS, com `y` para BAIXO; o G-buffer do
//!    modelador está em espaço de VISTA, com `y` para CIMA. ⚠️ A casa já pagou este sinal uma vez
//!    (*«sem esta negação a mesma lâmpada acende a pintura por cima e a escultura por baixo»*), e o
//!    oráculo dele é um RENDER — há gate a fazê-lo.
//! 2. **o `π`:** o rig promete que *uma superfície plana de frente devolve `1`*; o MaterialX recebe
//!    a radiância que chega, e uma difusa branca de frente para uma luz `L` devolve `L/π`. ⇒ a
//!    radiância é `π × intensidade × cor`.
//!
//! # ⚠️ O céu é ANCORADO NA TELA — e é isso que o torna um estúdio
//!
//! O `env_ambient` diz-o por escrito: as lâmpadas são de tela, e *«um estúdio cujo céu gira enquanto
//! as luzes ficam paradas não é um estúdio»*. Um céu de MUNDO é outra decisão, e chega com um rig de
//! mundo (a `W3` do plano).
//!
//! # ⭐⭐⭐ A radiância pré-filtrada é a da direcção MÉDIA do lóbulo (14/09)
//!
//! O céu é LINEAR na altura, e a média de uma função linear sobre um lóbulo é o valor dela na
//! direcção **média** — que é a espelhada só no limite de `α → 0`. Um lóbulo largo puxa a média para
//! o equador, e o céu lido fica mais claro (ou mais escuro) do que a luz que de facto chega.
//!
//! ⚠️⚠️ **A redacção anterior deste parágrafo dizia que o erro «não foi medido» e que ele «é da forma
//! do céu, não do material» — as duas metades estavam erradas** (medido 13/09, `docs/Render3d/05`
//! §8): quem manda na magnitude é o **`α` do material**, e o erro chega a `41,8 %` do termo da altura
//! com a rugosidade no máximo.
//!
//! ⛔ **E ela ficou VIVA no dia em que o material passou a ser autorável** (14/09, §12): a nota do §8
//! dizia, à letra, *«ela torna-se visível no dia em que houver material por objecto»* — num metal a
//! `roughness 1` são `p95 = 23` e `max = 33` bytes. *Quem move o número que tornava algo inalcançável
//! tem de reconferir a nota* (`CLAUDE.md` §0.0).
//!
//! ⇒ ver [`lobe_shrink`], e a §16 do `docs/Render3d/05`.

use ph2d_field_render::Lamp;

/// ⭐⭐⭐ **O ENCOLHIMENTO DO LÓBULO** — `E[ω]·R` sob a distribuição do pré-filtro GGX.
///
/// # Porque UM escalar cura, e cura EXACTAMENTE
///
/// Este céu é **linear na altura** (`L(ω) = A + B·ω.y`), e a média de uma função linear sobre uma
/// distribuição é a função avaliada na **direcção média** dela. A distribuição do pré-filtro é
/// simétrica em torno da espelhada `R`, logo a componente perpendicular cancela e sobra
/// `E[ω] = c(α)·R`. ⇒ `média(A + B·ω.y) = A + B·c(α)·R.y`.
///
/// ⚠️ **É a convolução em harmónicos esféricos, e não uma heurística:** uma função de grau `1`
/// convolvida com um núcleo simétrico é a mesma função de grau `1` escalada pelo coeficiente de grau
/// `1` do núcleo — e `c(α)` **é** esse coeficiente.
///
/// ⛔⛔ **A cura mora AQUI, e não na `ph2d-material`**, e a razão não é a de sempre (*«ela é o port
/// fiel do GLSL»*, que também vale): *«avaliar na direcção média»* só é **exacto** porque **este** céu
/// é linear. Sobre um céu com feições a lei é outra, e escrevê-la lá seria prometer, a quem trouxer o
/// céu seguinte, uma exactidão que ela não tem.
///
/// # A forma fechada, e de onde ela sai
///
/// Com `N = V = R` (a suposição do *split-sum*, que é a que o `mx_environment_prefilter` faz), as
/// amostras são `L = 2(N·h)h − N` com `h` do GGX, pesadas por `N·L` e descartadas em `N·L ≤ 0`:
///
/// ```text
/// c(α) = Σ (N·L)² / Σ (N·L)
/// ```
///
/// Integrando em `ξ` com `cos²θ_h = (1−ξ)/(1+(a−1)ξ)` e `a = α²`, com `k = a−1`, `m = a+1` e
/// `L = ln(2a/m)`:
///
/// ```text
/// c = [ k(3a+1) − 4am·L ] / { k · [ 2a·L − 2a + m ] }
/// ```
///
/// ⭐ **Dois controlos que não são coincidência:** `c(0) = 1` (o lóbulo colapsa na espelhada) e
/// `c(1) = 2/3` — que é **exactamente** o `(2/3)·k` do lóbulo cosseno que o [`ph2d_light::ENV_SLOPE`]
/// já carrega, e que este ficheiro desfaz com o `RAW` logo abaixo. *A rugosidade máxima do GGX é o
/// hemisfério cosseno, e as duas metades da casa chegam ao mesmo número por caminhos diferentes.*
///
/// ⚠️ **A vizinhança de `a = 1` é singularidade REMOVÍVEL**, e numericamente instável: o numerador e o
/// denominador vão os dois a zero como `k³`, logo o cancelamento come a precisão. Abaixo de
/// `|k| = 1e-3` devolve-se o limite (`2/3`) — o desvio ali é `< 1e-4`, contra um efeito que se mede
/// em dezenas de bytes. Há gate contra a quadratura, nos dois lados da costura.
#[must_use]
pub fn lobe_shrink(alpha: f32) -> f32 {
    let a = f64::from(alpha.clamp(0.0, 1.0)).powi(2);
    if a <= 0.0 {
        return 1.0;
    }
    let k = a - 1.0;
    if k.abs() < 1.0e-3 {
        return 2.0 / 3.0;
    }
    let m = a + 1.0;
    let l = (2.0 * a / m).ln();
    let den = k * (2.0 * a * l - 2.0 * a + m);
    if den == 0.0 {
        return 1.0;
    }
    ((k * (3.0 * a + 1.0) - 4.0 * a * m * l) / den) as f32
}

/// **O céu de estúdio**, em espaço de vista.
#[derive(Clone, Copy, Debug, Default)]
pub struct StudioSky;

impl ph2d_material::Environment for StudioSky {
    fn radiance(&self, dir: [f32; 3], alpha: f32) -> [f32; 3] {
        // O `ENV_SLOPE` da `ph2d-light` já vem convolvido com o lóbulo cosseno (`(2/3)·k`); a
        // radiância quer o `k` cru.
        const RAW: f32 = 1.5;
        // ⭐⭐⭐ **A ALTURA É A DA DIRECÇÃO MÉDIA DO LÓBULO** — ver [`lobe_shrink`] e o topo deste
        // ficheiro.
        //
        // ⛔⛔ **O `alpha` chegava aqui e era DEITADO FORA** (`_alpha`), e isso é a segunda espécie
        // de controlo morto do `CLAUDE.md` §5.0 — *o consumidor que projecta o valor fora*: o fio
        // está inteiro, o valor chega, e quem o recebe descarta-o. ⚠️ Nenhuma sonda de *«quem lê
        // este campo?»* o vê, porque ele **é** lido — está na assinatura.
        let up = lobe_shrink(alpha) * dir[1];
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * up)
        })
    }

    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        // Vista (`y` para cima) → canvas (`y` para baixo).
        ph2d_light::env_ambient([n[0], -n[1], n[2]])
    }
}

/// **As lâmpadas acesas de um rig**, em espaço de vista e como radiância.
#[must_use]
pub fn lamps(rig: &ph2d_light::LightRig) -> Vec<Lamp> {
    let Some(resolved) = ph2d_light::resolve(rig) else {
        return Vec::new();
    };
    resolved
        .lamps()
        .iter()
        .map(|l| Lamp {
            to_light: [l.dir[0], -l.dir[1], l.dir[2]],
            radiance: l.tint.map(|t| t * core::f32::consts::PI),
        })
        .collect()
}

#[cfg(test)]
#[path = "render_light_tests.rs"]
mod tests;
