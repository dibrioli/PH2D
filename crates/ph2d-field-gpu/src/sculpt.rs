//! ⭐⭐⭐ **A ESCULTURA NO DISPOSITIVO** — a grade de voxels, amostrada no shader.
//!
//! # ⚠️ A lei é a do [`ph2d_field_mesh::SampledField::at`], linha a linha
//!
//! Ela tem duas metades, e a segunda é a que uma leitura rápida perde:
//!
//! 1. **fora da caixa** o valor é `max(distância à caixa, amostra na PAREDE)` — e não a distância à
//!    caixa crua. ⛔ A CPU pagou este defeito com uma foto do dono (*«uma face solta»*): a amostra
//!    de parede valia zero e o `max` não podia funcionar;
//! 2. **dentro** é trilinear com o índice **grampeado**, porque o ponto está na caixa por contrato.
//!
//! # ⚠️ O que a grade custa, e porque ela tem CACHE
//!
//! Uma grade de `128³` são `8 MB`. Enviá-la por quadro seria pagar `8 MB` de barramento para
//! responder à mesma pergunta — mais do que a imagem inteira que este passe veio poupar. ⇒ ela sobe
//! uma vez e fica, com a identidade a ser o **ponteiro do `Arc`** ([`crate::trace::Tracer`]).

use ph2d_field_eval::device::DeviceSculpt;

/// Quantos `f32` o cabeçalho de uma escultura ocupa no vector das constantes.
///
/// ⚠️ **`24` e não `21`**: os três do fim são enchimento, e ele existe para o cabeçalho seguinte
/// começar num múltiplo de quatro. *Um deslocamento que se lê à mão é mais fácil de conferir contra
/// o shader do que um apertado.*
pub const CABECALHO: usize = 24;

/// A escultura escrita em WGSL, com os cabeçalhos de fora.
pub struct SculptWgsl {
    /// As `N` funções `escultura_k`, mais a lei de amostragem partilhada.
    pub source: String,
    /// Os cabeçalhos, a partir do `const_base` que foi dado — ver [`CABECALHO`].
    pub consts: Vec<f32>,
}

/// ⭐⭐⭐ **O corpo das funções que a fita chama** — ver [`ph2d_field_eval::wgsl::ESCULTURA`].
///
/// `None` quando alguma escultura não souber entregar a grade. ⚠️ `Some` com fonte **vazia** quando
/// não há escultura nenhuma: é o caminho de sempre, e ele não declara função nenhuma.
#[must_use]
pub fn emit(sculpts: &[DeviceSculpt], const_base: usize) -> Option<SculptWgsl> {
    if sculpts.is_empty() {
        return Some(SculptWgsl {
            source: String::new(),
            consts: Vec::new(),
        });
    }
    let mut consts: Vec<f32> = Vec::with_capacity(sculpts.len() * CABECALHO);
    let mut offset: u32 = 0;
    for s in sculpts {
        let g = s.grid()?;
        let mut h = [0.0f32; CABECALHO];
        #[allow(clippy::cast_precision_loss)]
        {
            h[0] = g.dims[0] as f32;
            h[1] = g.dims[1] as f32;
            h[2] = g.dims[2] as f32;
        }
        h[3] = g.step;
        h[4..7].copy_from_slice(&g.origin);
        // ⚠️⚠️ **O deslocamento viaja nos BITS de um `f32`, e não no valor dele.** Uma grade de
        // `128³` tem `2 097 152` amostras e duas já passam de `2²⁴`, que é onde um `f32` deixa de
        // representar um inteiro exactamente. *Um índice guardado como número real é um índice que
        // arredonda para a amostra do lado.*
        h[7] = f32::from_bits(offset);
        for r in 0..3 {
            for c in 0..3 {
                // ⚠️ **`f64` → `f32` acontece AQUI**, e é a mesma divergência declarada da fita.
                #[allow(clippy::cast_possible_truncation)]
                {
                    h[8 + r * 3 + c] = s.inv_rot[r][c] as f32;
                }
            }
        }
        h[17..20].copy_from_slice(&s.translation);
        h[20] = s.scale;
        consts.extend_from_slice(&h);
        offset += u32::try_from(g.values.len()).ok()?;
    }

    let mut src = String::with_capacity(2048 + sculpts.len() * 256);
    src.push_str(LEI);
    for (i, _) in sculpts.iter().enumerate() {
        let h = const_base + i * CABECALHO;
        src.push_str(&format!(
            "fn {}{i}(p: vec3<f32>) -> f32 {{ return escultura_amostra({h}u, p); }}\n",
            ph2d_field_eval::wgsl::ESCULTURA
        ));
    }
    Some(SculptWgsl {
        source: src,
        consts,
    })
}

/// ⭐ **Os valores das grades, concatenados na ordem de [`emit`]** — o que sobe para a placa.
///
/// ⚠️ A ordem **é** o contrato: os deslocamentos do cabeçalho saem desta mesma travessia.
#[must_use]
pub fn grid_values(sculpts: &[DeviceSculpt]) -> Option<Vec<f32>> {
    let mut v = Vec::new();
    for s in sculpts {
        v.extend_from_slice(s.grid()?.values);
    }
    Some(v)
}

/// Quantos `f32` as grades desta peça ocupam no armazém — sem as copiar.
///
/// ⚠️ **É a origem da região da grade de longe** ([`crate::longe`]), que vem logo a seguir; uma
/// peça sem escultura dá `0`. Uma escultura sem grade dá `None`, e aí a peça não vai à placa.
#[must_use]
pub fn grid_len(sculpts: &[DeviceSculpt]) -> Option<usize> {
    let mut n = 0usize;
    for s in sculpts {
        n += s.grid()?.values.len();
    }
    Some(n)
}

/// ⭐ **A identidade das grades desta peça** — o que um cache compara para saber se elas mudaram.
///
/// ⚠️ **É o ponteiro do `Arc`, e quem o guarda guarda também o `Arc`**: sem a referência forte, a
/// escultura podia morrer e o alocador devolver o mesmo endereço a outra — *um cache que compara
/// endereços tem de impedir que eles sejam reciclados.*
#[must_use]
pub fn identity(sculpts: &[DeviceSculpt]) -> Vec<usize> {
    sculpts
        .iter()
        .map(|s| std::sync::Arc::as_ptr(&s.field).cast::<()>() as usize)
        .collect()
}

/// A lei de amostragem, partilhada por todas as esculturas da peça.
const LEI: &str = r"
// ⭐⭐⭐ A `SampledField::at`, em WGSL. `h` é o deslocamento do cabeçalho em `k`.
fn escultura_amostra(h: u32, p: vec3<f32>) -> f32 {
    let dims = vec3<u32>(u32(k[h]), u32(k[h + 1u]), u32(k[h + 2u]));
    let step = k[h + 3u];
    let origem = vec3<f32>(k[h + 4u], k[h + 5u], k[h + 6u]);
    let off = bitcast<u32>(k[h + 7u]);
    let t = vec3<f32>(k[h + 17u], k[h + 18u], k[h + 19u]);
    let sc = k[h + 20u];

    // ⚠️ **A pose desfaz-se AQUI**, e a segunda metade dela é o `* sc` do fim: sem ele o campo
    // deixa de ser uma distância assim que houver escala, e todo raio de filete mente.
    let q = (p - t) / sc;
    let l = vec3<f32>(
        dot(q, vec3<f32>(k[h +  8u], k[h +  9u], k[h + 10u])),
        dot(q, vec3<f32>(k[h + 11u], k[h + 12u], k[h + 13u])),
        dot(q, vec3<f32>(k[h + 14u], k[h + 15u], k[h + 16u])));

    let lo = origem;
    let hi = origem + vec3<f32>(f32(dims.x - 1u), f32(dims.y - 1u), f32(dims.z - 1u)) * step;
    // `distance_to_box`: só as componentes que saem contam.
    let fora = length(max(max(lo - l, l - hi), vec3<f32>(0.0)));
    if (fora > 0.0) {
        // ⛔⛔ **O `max` com a amostra na PAREDE, e não a distância à caixa crua.** A CPU pagou este
        // defeito com uma foto do dono: sem a amostra de parede, um passo para fora da caixa lia a
        // distância à caixa (`+0,016`) onde por dentro o campo valia `+0,083` — e a marcha parava
        // em cheio numa superfície que não existe.
        return max(fora, escultura_trilinear(dims, origem, step, off, clamp(l, lo, hi))) * sc;
    }
    return escultura_trilinear(dims, origem, step, off, l) * sc;
}
";

/// ⭐⭐⭐ **A INTERPOLAÇÃO TRILINEAR de uma grade no armazém `grades`** — partilhada pela escultura
/// e pela grade de LONGE ([`crate::longe`]).
///
/// ⚠️ **Ela vive fora da [`LEI`] porque a grade de longe existe sem escultura nenhuma** — e uma
/// segunda cópia dela seria a segunda resposta a *«que valor tem a grade neste ponto?»*, com o
/// `fma` a divergir no último bit da primeira. É o [`crate::trace_wgsl::leis`] que a põe no texto,
/// **sempre**.
pub(crate) const TRILINEAR: &str = r"
// ⚠️ **O índice é GRAMPEADO porque o ponto está na caixa por contrato** — ver a nota da CPU: a
// guarda que ali havia devolvia zero NA PAREDE, que é a definição de uma superfície.
fn escultura_eixo(v: f32, origem: f32, inv: f32, n: u32) -> vec2<f32> {
    let top = f32(n - 1u);
    let cru = (v - origem) * inv;
    // ⚠️ O `NaN` cai no `else`, como na CPU: a comparação negada apanha-o sem um ramo próprio.
    var g = 0.0;
    if (cru >= 0.0) { g = min(cru, top); }
    // ⚠️ `max(n, 2u) - 2u` e não `n - 2u`: uma grade de um plano só faria o `u32` dar a volta.
    let i = min(u32(floor(g)), max(n, 2u) - 2u);
    return vec2<f32>(f32(i), g - f32(i));
}

fn escultura_trilinear(dims: vec3<u32>, origem: vec3<f32>, step: f32, off: u32, p: vec3<f32>) -> f32 {
    let inv = 1.0 / step;
    let ex = escultura_eixo(p.x, origem.x, inv, dims.x);
    let ey = escultura_eixo(p.y, origem.y, inv, dims.y);
    let ez = escultura_eixo(p.z, origem.z, inv, dims.z);
    let rx = dims.x;
    let rxy = dims.x * dims.y;
    let base = off + u32(ex.x) + u32(ey.x) * rx + u32(ez.x) * rxy;
    let c000 = grades[base];
    let c100 = grades[base + 1u];
    let c010 = grades[base + rx];
    let c110 = grades[base + rx + 1u];
    let c001 = grades[base + rxy];
    let c101 = grades[base + rxy + 1u];
    let c011 = grades[base + rxy + rx];
    let c111 = grades[base + rxy + rx + 1u];
    // ⚠️ **`fma` e não `mix`**: a CPU escreve `(b − a).mul_add(t, a)`, que é UMA operação fundida.
    // O `mix` da WGSL é `a·(1−t) + b·t` — a mesma álgebra, outro último bit.
    let x00 = fma(c100 - c000, ex.y, c000);
    let x10 = fma(c110 - c010, ex.y, c010);
    let x01 = fma(c101 - c001, ex.y, c001);
    let x11 = fma(c111 - c011, ex.y, c011);
    let y0 = fma(x10 - x00, ey.y, x00);
    let y1 = fma(x11 - x01, ey.y, x01);
    return fma(y1 - y0, ez.y, y0);
}
";
