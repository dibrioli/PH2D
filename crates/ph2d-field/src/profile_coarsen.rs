//! ⭐⭐ **SIMPLIFICAR um perfil** — a responsabilidade que saiu do [`crate::profile`] em 2026-09-16,
//! quando o tecto de LOC disparou a `712/700` ao acrescentar os arcos.
//!
//! O vizinho responde *«o que É um perfil»* (o modelo, as duas vistas, a validação); este responde
//! *«como se faz um perfil mais barato»* (a decimação por orçamento de giro, para a
//! pré-visualização). São perguntas diferentes e mudam por motivos diferentes.
//!
//! ⚠️ **Toda saída daqui nasce por [`crate::Profile::new`], logo SEM arcos** — e é o que se quer: o
//! perfil grosso é uma aproximação declarada, e a decomposição exacta não sobrevive a uma decimação
//! que apaga vértices.

use crate::profile::Profile;

/// ⭐⭐⭐ **O MESMO CONTORNO, MAIS GROSSO — para a pré-visualização.**
///
/// # ⚠️ Por que ela existe
///
/// O traçado custa **`0,22 ms` por aresta do contorno**, medido, e esse custo é **cego aos pixels**:
/// numa imagem 4× menor ele cai `1,3×`. ⇒ a pré-visualização, que baixa a **resolução da tela** para
/// caber no orçamento de um quadro, **não baixava as arestas** — e subir o `Resolution` custava fps
/// enquanto a mão mexia (report do Enio, 2026-08-26).
///
/// ⭐ Esta é a **mesma lei que o módulo já ship**, aplicada onde faltava: *grosso a mexer, nítido ao
/// assentar*. O que o artista pediu em detalhe aparece quando ele **pára**, que é quando ele olha.
///
/// # ⚠️ Ela DECIMA, não recoze — e a diferença é o que a torna possível
///
/// Recozer exigiria a curva de origem, que vive na cena vetorial e **não** no documento. O que há
/// aqui é a polilinha já achatada, e um contorno achatado por **tolerância** tem os pontos densos
/// onde a curvatura é alta — então tirar um em cada `k` preserva o carácter da forma em vez de a
/// achatar por igual.
///
/// ⚠️ **Um contorno decimado pode auto-intersectar-se** numa feição fina, e é por isso que o
/// resultado passa pelo [`Profile::new`]: se ele recusar, volta o original. *Uma pré-visualização
/// que estraga a peça é pior do que uma lenta.*
///
/// ⛔ E ela **nunca sobe**: um `max_edges` maior do que o contorno devolve o próprio contorno, sem
/// inventar pontos que a curva não tem.
#[must_use]
pub fn coarsen(profile: &Profile, max_edges: usize) -> Profile {
    let total = profile.segment_count();
    if total <= max_edges || max_edges < 3 {
        return profile.clone();
    }
    // ⚠️ **O orçamento é o mesmo para todos os contornos**: um furo e a borda de fora têm de
    // encolher JUNTOS, senão o furo escapa da peça que o continha.
    let giro_total: f64 = profile.contours().iter().map(|c| total_abs_turn(c)).sum();
    // ⚠️ Um contorno FECHADO gira sempre `2π`, então isto não acontece — mas um `NaN` que viesse de
    // um ponto degenerado passaria por um `<= 0.0` ingénuo, e o orçamento sairia `NaN`.
    if !giro_total.is_finite() || giro_total <= 0.0 {
        return profile.clone();
    }
    coarsen_with_turn_budget(profile, giro_total / max_edges as f64)
}

/// ⭐⭐⭐ **O CONTORNO ENGROSSADO ATÉ AO ERRO QUE SE TOLERA** (W85) — a forma que a
/// [`coarsen`] devia ter tido desde o início.
///
/// # Por que o ERRO, e não a contagem
///
/// Depois da W84 a decimação reparte **giro** (ver [`decimate_by_turn`]), e isso torna a contagem de
/// arestas uma consequência em vez de uma lei: o que o orçamento de giro fixa é o **erro da
/// normal**, que é metade do ângulo que uma corda substitui. ⇒ *pedir um erro é pedir a coisa que se
/// vê; pedir uma contagem é pedir um número que só a esperança liga ao que se vê.*
///
/// ⭐ **E é adaptativo à FORMA de graça:** um círculo gira `2π` e uma estrela de dez pontas gira
/// muito mais, então a estrela recebe mais arestas **porque tem mais direcção para gastar** — sem
/// uma regra própria a dizê-lo.
///
/// # ⚠️ De que recurso o número é
///
/// Medido (`measure_how_many_contour_edges_are_visible`, a régua é o **pixel sombreado** em níveis
/// de 8 bits, contra um contorno de `2048`):
///
/// | erro de normal p99 | pixel p99 | pixel máx |
/// |---:|---:|---:|
/// | `0,266°` | `1` | `1`–`2` |
/// | `0,529°` | `1` | `2`–`3` |
/// | `1,056°` | `3` | `4` |
/// | `2,110°` | `5` | `9`–`10` |
///
/// ⚠️ **E ela é INDEPENDENTE do tamanho da imagem** — os mesmos números a `640×360` e a `1600×900`.
/// ⛔ Isso **refuta** derivar o tecto do tamanho do pixel: o erro que se vê é **angular**, e um
/// ângulo não encolhe com a resolução da tela.
#[must_use]
pub fn coarsen_to_normal_error(profile: &Profile, max_error_rad: f32) -> Profile {
    // ⚠️ O `is_finite` primeiro: um `NaN` passa por um `<= 0.0` ingénuo e o orçamento sai `NaN`.
    if !max_error_rad.is_finite() || max_error_rad <= 0.0 {
        return profile.clone();
    }
    // ⚠️ O erro da normal é **metade** do giro que a corda substitui: a corda aponta para o meio do
    // arco, e as duas pontas dele afastam-se dela por metade do ângulo cada.
    coarsen_with_turn_budget(profile, f64::from(max_error_rad) * 2.0)
}

/// O corpo partilhado pelas duas portas acima — *uma lei, dois nomes para a mesma pergunta*.
fn coarsen_with_turn_budget(profile: &Profile, orcamento: f64) -> Profile {
    let total = profile.segment_count();
    if !orcamento.is_finite() || orcamento <= 0.0 {
        return profile.clone();
    }
    let thinner: Vec<Vec<[f32; 2]>> = profile
        .contours()
        .iter()
        .map(|c| {
            // ⚠️ Um contorno que já é pequeno fica INTEIRO: decimá-lo levá-lo-ia abaixo do triângulo,
            // e um furo de três lados é melhor do que um furo que desapareceu.
            if c.len() <= 8 {
                return c.clone();
            }
            decimate_by_turn(c, orcamento)
        })
        .collect();
    if thinner.iter().any(|c| c.len() < 3) {
        return profile.clone();
    }
    // ⚠️ A tolerância declarada sobe com a decimação: ela é o erro contra a curva de origem, e a
    // polilinha decimada erra mais. Mentir aqui envenenaria quem a usa para escolher uma grade.
    let ficou: usize = thinner.iter().map(Vec::len).sum();
    let passo_medio = (total as f32 / ficou.max(1) as f32).max(1.0);
    let tol = profile.tolerance() * passo_medio;
    Profile::new(thinner, profile.fill(), tol).unwrap_or_else(|_| profile.clone())
}

/// A curvatura total de um contorno fechado, em radianos e **sem sinal**.
///
/// ⚠️ **Sem sinal de propósito.** O giro *com* sinal de um contorno fechado é `±2π`, sempre — ele não
/// distingue um círculo de uma estrela. O que a decimação precisa de repartir é **quanta direcção**
/// a forma tem para gastar, e uma ponta de estrela gasta muito em pouco caminho.
fn total_abs_turn(c: &[[f32; 2]]) -> f64 {
    (0..c.len()).map(|i| turn_at(c, i)).sum()
}

/// O ângulo, em radianos, entre a aresta que **chega** ao vértice `i` e a que **sai** dele.
fn turn_at(c: &[[f32; 2]], i: usize) -> f64 {
    let n = c.len();
    if n < 3 {
        return 0.0;
    }
    let (p, q, r) = (c[(i + n - 1) % n], c[i], c[(i + 1) % n]);
    let a = [f64::from(q[0] - p[0]), f64::from(q[1] - p[1])];
    let b = [f64::from(r[0] - q[0]), f64::from(r[1] - q[1])];
    let cross = a[0] * b[1] - a[1] * b[0];
    let dot = a[0] * b[0] + a[1] * b[1];
    // `atan2` do produto vectorial contra o escalar — estável mesmo com arestas muito curtas, que
    // é onde uma versão por `acos` do normalizado devolve `NaN`.
    cross.atan2(dot).abs()
}

/// ⭐⭐⭐ **A decimação por GIRO** — mantém um vértice quando o ângulo acumulado desde o último
/// mantido chega ao orçamento.
///
/// # ⛔ O que ela substitui, e por que a anterior estava errada
///
/// A versão até 2026-08-27 tirava **um em cada `k`** vértices, com este raciocínio no doc do
/// [`coarsen`]: *«um contorno achatado por tolerância tem os pontos densos onde a curvatura é alta —
/// então tirar um em cada `k` preserva o carácter da forma»*. ⭐ Isso é **verdade para curvatura**,
/// que é distribuída por muitos vértices.
///
/// ⚠️ **Uma QUINA não é curvatura distribuída: é um vértice só, com todo o ângulo dentro.** Um passo
/// por índice apaga-a com probabilidade `(k−1)/k`, e o que fica no lugar é um bisel — *e se ela
/// sobrevive depende de o índice dela ser divisível pelo passo, o que é uma lotaria.*
///
/// ⛔ **Medido** (`measure_whether_the_preview_decimation_eats_corners`, uma estrela de 5 pontas com
/// 400 pontos, traçada a `640×360`):
///
/// | tecto | passo | pixels que mudam | normal p99 | normal máx |
/// |---:|---:|---:|---:|---:|
/// | `336` | `2` | `0` | `0,034°` | `0,048°` |
/// | **`168`** | **`3`** | **`509` (`0,87 %`)** | **`28,1°`** | **`126,8°`** |
/// | `84` | `5` | `0` | `0,034°` | `0,048°` |
///
/// ⭐ As quinas caem em múltiplos de `40`: com passo `2` e `5` elas sobrevivem, com `3` **três em
/// cada cinco morrem**. E o `PREVIEW_MAX_EDGES` que ship é justamente `168`.
///
/// # ⭐ Por que o GIRO é a grandeza certa
///
/// O erro de uma corda que substitui um arco é fixado pelo **ângulo** que o arco varre, não pelo
/// número de pontos que ele tinha. E o erro que se **vê** é o da **normal**, que é esse mesmo ângulo
/// (medido: a normal p99 de um círculo decimado é exactamente `∝ 1/n`). ⇒ repartir o giro por igual
/// distribui o erro por igual, e um vértice que sozinho gasta o orçamento — uma quina — é mantido
/// **por construção**, sem uma regra própria a dizê-lo.
fn decimate_by_turn(c: &[[f32; 2]], orcamento: f64) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(c.len());
    let mut acc = 0.0f64;
    for i in 0..c.len() {
        let t = turn_at(c, i);
        if out.is_empty() || acc + t >= orcamento {
            out.push(c[i]);
            acc = 0.0;
        } else {
            acc += t;
        }
    }
    out
}
