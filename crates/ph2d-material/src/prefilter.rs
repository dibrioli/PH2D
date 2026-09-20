//! ⭐⭐⭐ **O PRIMEIRO MOMENTO do pré-filtro GGX** — o número que o [`crate::wgsl::EnvLobe`] já pedia
//! pelo nome e que esta crate não dava.
//!
//! # ⛔⛔ Porque ele mora AQUI agora, e a recusa que ele contraria
//!
//! O doc do [`crate::wgsl::EnvLobe`] escreve, campo a campo, `lobe_shrink(clamp(main_alpha, EPS, 1))`
//! — ou seja, **o contrato desta crate nomeia esta função** e mandava cada consumidor escrevê-la. Com
//! **um** consumidor isso era dívida; com **dois** é a lei escrita em dois sítios, que é a forma que
//! esta casa já pagou meia dúzia de vezes.
//!
//! ⚠️ **E há uma recusa escrita a dizer o contrário**, no `ph2d-app-field3d/src/render_light.rs`:
//! *«a cura mora AQUI, e não na `ph2d-material` … "avaliar na direcção média" só é exacto porque
//! ESTE céu é linear»*. ⭐ **Ela continua verdadeira, e é sobre OUTRA coisa:** a recusa é sobre a
//! **aplicação** (*ler o céu na direcção média*), que de facto só é exacta para um céu linear e fica
//! com cada céu. O que se mudou é o **NÚMERO** — `c(α)` é o coeficiente de grau `1` do núcleo do
//! pré-filtro, e um núcleo não sabe que céu vai convolver.
//!
//! ⇒ *a promessa fica onde estava; o coeficiente mora onde o contrato já o nomeava.*

/// ⭐⭐⭐ **O ENCOLHIMENTO DO LÓBULO** — `E[ω]·R` sob a distribuição do pré-filtro GGX.
///
/// # O que ele É
///
/// A distribuição do pré-filtro (o *split-sum* de Karis, que é a que o `mx_environment_prefilter`
/// assume) é **simétrica em torno da direcção espelhada `R`**, logo a componente perpendicular da
/// média cancela-se e sobra `E[ω] = c(α)·R`, com `c ≤ 1`.
///
/// ⚠️ **É a convolução em harmónicos esféricos, não uma heurística:** uma função de grau `1`
/// convolvida com um núcleo simétrico é a mesma função de grau `1`, escalada pelo coeficiente de grau
/// `1` do núcleo — e `c(α)` **é** esse coeficiente.
///
/// # ⛔ O que ele NÃO é
///
/// Ele **não** autoriza ler um céu qualquer na direcção média. Isso é exacto para um céu **linear na
/// direcção** (`L(ω) = A + B·(ω·k)`) e para mais nada: sobre um céu com feições, a média de uma
/// função não-linear não é a função da média. *Essa promessa é de quem tem o céu, e esta crate não
/// tem nenhum.*
///
/// # A forma fechada, e de onde ela sai
///
/// Com `N = V = R`, as amostras são `L = 2(N·h)h − N` com `h` do GGX, pesadas por `N·L` e
/// **descartadas** em `N·L ≤ 0` — é o descarte que faz `c` deixar de ser trivial quando o lóbulo passa
/// do hemisfério:
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
/// `c(1) = 2/3` — que é **exactamente** o `Â₁` do lóbulo cosseno (Ramamoorthi & Hanrahan 2001).
/// *A rugosidade máxima do GGX é o hemisfério cosseno, e as duas metades da casa chegam ao mesmo
/// número por caminhos diferentes.*
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

/// A sonda e os gates desta lei — ver o cabeçalho deles.
#[cfg(test)]
#[path = "prefilter_tests.rs"]
mod prefilter_tests;
