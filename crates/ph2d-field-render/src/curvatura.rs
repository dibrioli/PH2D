//! ⭐⭐⭐ **A CURVATURA DA SUPERFÍCIE** — a grandeza geométrica que a subsuperfície MACIÇA lê.
//!
//! # ⚠️ Porque ela sai do CAMPO e não de derivadas de ECRÃ
//!
//! O renderizador de referência do MaterialX estima-a por `length(fwidth(N)) / length(fwidth(P))` —
//! uma diferença finita entre píxeis vizinhos. ⛔⛔ **Aqui isso era inexprimível sem partir uma
//! propriedade que esta linha tem em `100,000 %`:** o `fwidth` da placa é por **quad de `2×2`** e a
//! diferença do traçador de CPU seria **por pixel**, logo os dois motores deixariam de dar o mesmo
//! byte. ⇒ a curvatura tem de ser função do PONTO, e é.
//!
//! ⭐ E medido, o estimador dele é ruído: sobre a esfera do oráculo (curvatura verdadeira `1`) a
//! curvatura implícita de cada amostra espalha-se de **`0,02` a `54`**, e **não é a tesselação** —
//! uma esfera `256×128` nossa devolve a mesma dispersão (`ph2d-material/src/tests.rs`, `MACICOS`).
//! *A nossa é uma propriedade da peça; a dele é uma propriedade do ecrã.*
//!
//! # ⭐⭐⭐ A lei, e porque ela não custa amostras novas de estêncil
//!
//! Num campo de distância `|∇f| = 1`, logo a curvatura média do conjunto de nível é `H = ∇²f / 2`.
//! E o Laplaciano sai da **MESMA soma** que a normal já percorre — ver [`crate::march::Stencil`],
//! cujo doc já diz que as duas leis são somas sobre os mesmos deslocamentos:
//!
//! ```text
//! Σⱼ f(p + ε·dⱼ) = n·f(p) + ε·∇f·Σdⱼ + (ε²/2)·Σ dⱼᵀ H dⱼ + O(ε³)
//! ```
//!
//! Nos dois estêncis `Σdⱼ = 0` e `Σ dⱼdⱼᵀ = c·I`, logo o termo do meio morre e o último é
//! `c·∇²f`:
//!
//! | estêncil | `c` | `∇²f` |
//! |---|---:|---|
//! | [`Stencil::Central6`] (`\|d\| = 1`) | `2` | `(Σfⱼ − 6·f(p)) / ε²` |
//! | [`Stencil::Tetra4`] (`\|d\| = √3`) | `4` | `(Σfⱼ − 4·f(p)) / (2ε²)` |
//!
//! ⇒ o que falta às amostras que a marcha já pagou é **uma só**: a do CENTRO.
//!
//! # ⛔ E o estêncil desta lei é FIXO no de quatro, mesmo quando a normal usa seis
//!
//! A [`crate::march::Stencil`] é uma escolha de QUADRO (a normal pode ser lida por `6` ou por `4`
//! amostras), e o traçador do dispositivo lê sempre pelo de `4`. ⇒ deixar esta lei seguir aquela
//! escolha faria os dois motores responderem curvaturas diferentes no mesmo ponto **no dia em que o
//! quadro escolhesse o outro**, e a paridade cairia sem uma linha de lei ter mudado.
//!
//! # ⚠️ O sinal, e a divergência que ele declara
//!
//! A referência devolve um **comprimento**, logo é sempre `≥ 0`: ela não distingue uma bossa de uma
//! cova. Esta porta devolve `|H|` pela mesma razão, e o gate mede-o. ⛔ A diferença real fica noutro
//! sítio e está **declarada**: a dela é a curvatura normal na direcção do ECRÃ (logo muda ao rodar
//! a câmera) e a nossa é a **média** (logo não muda). *A nossa é a que um artista espera.*

use ph2d_field_eval::hybrid::Hybrid;

/// Os quatro vértices do tetraedro — ver o cabeçalho para porque não é o estêncil do quadro.
const OFFSETS: [[f32; 3]; 4] = [
    [1.0, -1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
];

/// ⭐⭐⭐ **O PASSO desta lei, e ele NÃO é o da normal** — a fracção do tamanho da peça.
///
/// # ⛔⛔ A premissa que a medição derrubou
///
/// A 1.ª redacção deste módulo dizia *«`eps` é o mesmo que a normal usou; dois passos diferentes
/// dariam duas respostas para a mesma pergunta»*. **É falso, e o gate reprovou em voz alta:** com o
/// passo da normal a face de uma caixa lia curvatura **`1,49`** e a esfera errava `109 %`.
///
/// *Uma primeira diferença e uma segunda não têm o mesmo passo óptimo.* A primeira divide por `ε` e
/// a segunda por **`ε²`**, logo o cancelamento em `f32` entra `1/ε` vezes mais cedo — e o óptimo da
/// segunda é `~(ulp)^{1/4}` contra `^{1/3}` da primeira.
///
/// # ⭐ E o óptimo é uma FRACÇÃO DA PEÇA, medida — não um epsilon absoluto
///
/// Varrido em três raios (pior erro relativo sobre 32 pontos da superfície):
///
/// | `ε / raio` | `R = 0,4` | `R = 1,0` | `R = 2,5` |
/// |---:|---:|---:|---:|
/// | `0,0004` | `0,534` | `0,397` | `0,404` |
/// | `0,0016` | `0,033` | `0,022` | `0,025` |
/// | **`0,0064`** | **`0,0041`** | **`0,0041`** | **`0,0047`** |
/// | `0,0256` | `0,027` | `0,013` | `0,013` |
/// | `0,1024` | `0,054` | `0,054` | `0,054` |
///
/// ⭐ **O vale está no MESMO sítio nos três** — é isso que faz dele uma lei e não uma calibração:
/// abaixo dele manda o cancelamento (`1/ε²`), acima manda a truncagem (`O(ε²)`).
///
/// ⚠️ `escala` é o tamanho da PEÇA (o raio da bola que a envolve), e **não** o da vista: a curvatura
/// é uma propriedade da peça, e um passo que seguisse o zoom daria duas curvaturas para o mesmo
/// ponto — exactamente o defeito de ecrã que este módulo existe para não ter.
#[must_use]
pub fn eps_para(escala: f32) -> f32 {
    const FRACCAO: f32 = 0.0064;
    (escala.abs() * FRACCAO).max(crate::PRECISION_FLOOR)
}

/// ⭐ **A curvatura de cada ponto**, `|H|` em `1/unidade de mundo` — para uma esfera de raio `R`,
/// `1/R`.
///
/// `eps` é o passo da diferença, e quem o deriva é a [`eps_para`] — leia lá porque ele **não** é o
/// da normal.
///
/// Devolve um vector do tamanho de `pontos`. ⚠️ Um ponto onde a avaliação falha lê `0`, que o piso
/// do GLSL (`max(κ, 0,01)`) transforma num raio de `100` — *a leitura certa para «não sei» é uma
/// superfície quase plana*, e não um `NaN` a atravessar o material.
#[must_use]
pub fn curvaturas(eval: &mut Hybrid, pontos: &[[f32; 3]], eps: f32) -> Vec<f32> {
    // ⚠️ O `is_nan` é explícito: a intenção é **recusar o NaN**, e um `<=` sozinho não o apanha.
    if pontos.is_empty() || eps.is_nan() || eps <= 0.0 {
        return vec![0.0; pontos.len()];
    }
    // As cinco amostras de cada ponto, num lote só — a avaliação em lote é o que esta crate paga
    // barato (a mesma razão do estêncil da normal).
    let n = pontos.len();
    let mut xs = Vec::with_capacity(n * 5);
    let mut ys = Vec::with_capacity(n * 5);
    let mut zs = Vec::with_capacity(n * 5);
    for p in pontos {
        for d in OFFSETS {
            xs.push(d[0].mul_add(eps, p[0]));
            ys.push(d[1].mul_add(eps, p[1]));
            zs.push(d[2].mul_add(eps, p[2]));
        }
        xs.push(p[0]);
        ys.push(p[1]);
        zs.push(p[2]);
    }
    let Ok(v) = eval.eval(&xs, &ys, &zs) else {
        return vec![0.0; n];
    };
    let denom = 2.0 * eps * eps;
    (0..n)
        .map(|i| {
            let b = i * 5;
            let soma = v[b] + v[b + 1] + v[b + 2] + v[b + 3];
            let laplaciano = (soma - 4.0 * v[b + 4]) / denom;
            (laplaciano * 0.5).abs()
        })
        .collect()
}

/// ⭐⭐ **A curvatura de cada pixel que acertou** — `0` nos que não acertaram.
///
/// ⚠️ **Ela corre só quando algum material da cena a LÊ**, e quem decide é o chamador: com a
/// subsuperfície maciça desligada (a omissão) o vector fica **vazio** e o sombreamento lê `0` sem
/// pagar amostra nenhuma. *Uma grandeza que ninguém lê não se calcula* — é a mesma bandeira que a
/// assadura do chão já respeita.
#[must_use]
pub fn do_gbuffer(eval: &mut Hybrid, g: &crate::Gbuffer, eps: f32) -> Vec<f32> {
    let vivos: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
    let pontos: Vec<[f32; 3]> = vivos.iter().map(|&i| g.point[i]).collect();
    let k = curvaturas(eval, &pontos, eps);
    let mut out = vec![0.0f32; g.hit.len()];
    for (slot, &i) in k.iter().zip(&vivos) {
        out[i] = *slot;
    }
    out
}
