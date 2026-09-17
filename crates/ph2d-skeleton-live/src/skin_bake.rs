//! ⭐⭐⭐ **A MALHA FINA ASSADA NO BIND** — a W1 da F9 (`docs/Skeleton/01_a_fila.md`).
//!
//! # O que muda de sítio
//!
//! Hoje a densidade da pele é uma decisão do **QUADRO**: o `Smooth` refina a malha a cada quadro,
//! contra uma tolerância em pixels de ecrã e dentro de um orçamento de peças
//! ([`crate::skin_budget::SKIN_FRAME_PIECES`]). Numa cena com muita arte presa o orçamento não
//! chega para ninguém e o `Smooth` fica **igual ao `Fast`** — o report que abriu a F9.
//!
//! ⇒ a densidade passa a ser uma decisão do **BIND**: assa-se uma vez, em repouso, onde o campo de
//! pesos curva ([`ph2d_poly2d::refine_rest_by_attrs`]), e o quadro só **posa** o que já está lá.
//!
//! # ⚠️ Esta wave NÃO muda o que se vê, e isso é o pedido
//!
//! A fila manda a W1 entregar *«a malha fina no bind, com a régua da silhueta a mesma de hoje»* —
//! porque quem paga a densidade nova é o **quadro** enquanto a deformação estiver na CPU, e só a W2
//! (o *vertex shader*) a torna de graça. ⇒ a porta nasce **DESLIGADA** e o caminho de omissão é
//! byte-idêntico: [`assar_no_bind`] devolve `None` sem a env var.
//!
//! ⛔ **Um interruptor que nasce ligado antes de a W2 pagar por ele** poria mais vértices para a
//! CPU deformar por quadro — exactamente o recurso que a F9 existe para libertar.

use ph2d_poly2d::{AttrLaw, Mesh2d, RefineOptions, hermite_attrs, refine_rest_by_attrs};

/// ⭐ **O ERRO QUE A ASSADURA TOLERA, em pixels da ARTE** — a mesma barra que o `Smooth` do quadro
/// usa (`0,5 px`), para as duas leis prometerem a mesma coisa.
pub const TOLERANCIA_PX: f64 = 0.5;

/// ⭐⭐⭐ **A TOLERÂNCIA DA ASSADURA, em fracção de PESO** — `TOLERANCIA_PX / diagonal da arte`.
///
/// # Porque ela é DERIVADA e não uma constante
///
/// O critério mede curvatura de PESO (adimensional) e o que o artista vê são PIXELS. A ponte entre
/// os dois é o teorema do [`ph2d_poly2d::refine_rest_by_attrs`]:
///
/// ```text
/// |erro| ≤ τ · dispersão      (dispersão = quanto duas poses de osso afastam o mesmo ponto)
/// ```
///
/// ⇒ a tolerância de peso que entrega `0,5 px` é `0,5 / dispersão_máxima`. E a dispersão máxima que
/// um osso pode produzir é **a extensão da própria arte**: um osso que rode `180°` à volta de um
/// pivô a `d` da arte desloca o ponto `2 d`, e `d` não passa de meia diagonal. *Uma constante aqui
/// entregaria meio pixel numa arte e cinco noutra.*
///
/// # ⛔⛔ O número que eu tinha escrito era INERTE, e foi a arte REAL que o disse
///
/// A 1.ª redacção fixava `0,02`, tirado de uma escada medida sobre uma fixtura **sintética**. Sobre
/// a arte da cena do dono (`512 × 320`, `2 430` peças, pesos BBW por 3 ossos) o pior desvio de peso
/// de TODA aresta já é **`0,0154`** — ⇒ a `0,02` a assadura não partia **nada**, e o knob era um
/// controlo morto. A causa é boa: a malha do bind **já é graduada pelas articulações** desde
/// 2026-09-10, logo a densidade já está onde o campo vira.
///
/// A escada na arte REAL (sonda `escada_da_assadura_na_arte_real`, em `ph2d-app-vec`):
///
/// | `τ` | peças | × entrada | desvio |
/// |---:|---:|---:|---:|
/// | `0,0200` | `2 430` | `1,00×` | `0,0154` (**nada parte**) |
/// | `0,0100` | `2 556` | `1,05×` | `0,0100` |
/// | `0,0050` | `3 254` | `1,34×` | `0,0050` |
/// | `0,0020` | `5 894` | `2,43×` | `0,0020` |
/// | `0,0010` | `11 516` | `4,74×` | `0,0010` |
///
/// ⇒ com a diagonal de `604 px` desta arte, `τ = 0,5/604 ≈ 8,3e-4` e a assadura pede **`~5×`** a
/// malha do bind (`~13 000` peças).
///
/// ⭐⭐ **E é esse número que PROVA que a porta tem de ficar fechada até à W2:** o orçamento de CPU
/// do quadro inteiro é `SKIN_FRAME_PIECES = 8 738` — *uma imagem assada não caberia nele sozinha*.
/// Na placa, `13 000` triângulos custam `~0,15 %` de um quadro (a tabela da W0-b). *A assadura não
/// é cara: o que é caro é deformá-la na CPU.*
#[must_use]
pub fn tolerancia_do_bind(mesh: &Mesh2d) -> f64 {
    let diagonal = f64::from(mesh.size[0]).hypot(f64::from(mesh.size[1]));
    if diagonal > 0.0 {
        TOLERANCIA_PX / diagonal
    } else {
        // Uma arte sem tamanho não tem pixels para tolerar; o critério fica adimensional puro.
        TOLERANCIA_PX
    }
}

/// Quantas vezes a malha de entrada a assadura pode crescer. ⚠️ `8×` e não `4,5×`: a escada acima é
/// de UMA arte, e o tecto é uma cerca contra uma arte cujo campo de pesos curve mais — quem a
/// atingir sai com `travado_pelo_orcamento` e não em silêncio.
pub const CRESCIMENTO_MAX: usize = 8;

/// ⭐ **A porta está ligada?** — `PH2D_SKIN_BAKE=1`, lido uma vez.
///
/// ⚠️ Nasce DESLIGADA: ver o cabeçalho.
fn ligada() -> bool {
    static LIGADA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADA.get_or_init(|| {
        std::env::var("PH2D_SKIN_BAKE").is_ok_and(|v| {
            let v = v.trim();
            v == "1" || v.eq_ignore_ascii_case("sim") || v.eq_ignore_ascii_case("on")
        })
    })
}

/// ⭐⭐⭐ **ASSA A MALHA DO BIND** — `None` quando não há nada a fazer, e aí o chamador guarda o que
/// já tinha (o caminho de omissão, byte-idêntico).
///
/// `None` em quatro casos, todos legítimos e nenhum silencioso para quem lê este doc:
/// - a porta está desligada (o caminho de omissão desta wave);
/// - a malha não traz pesos (`ossos == 0`) — a 1.ª mídia resolve pela lei derivada, e ali não há
///   campo de atributos para perseguir;
/// - a malha está vazia;
/// - a assadura não partiu nada (o campo de pesos já era linear em toda aresta) ⇒ devolver a
///   entrada seria uma cópia inútil.
///
/// ⚠️ **Os pesos saem RE-INTERPOLADOS pela mesma lei de Hermite** que o `Smooth` usa por quadro —
/// os vértices novos nascem com o peso que aquela lei lhes dá, e não com uma segunda derivação.
/// *Duas leis para a mesma pergunta divergiriam no dia em que uma ganhasse uma cerca.*
#[must_use]
pub fn assar_no_bind(mesh: &Mesh2d, pesos: &[f64], ossos: usize) -> Option<(Mesh2d, Vec<f64>)> {
    if !ligada() {
        return None;
    }
    assar(mesh, pesos, ossos)
}

/// ⭐⭐ **O TRABALHO, sem a porta** — a metade que os gates medem.
///
/// ⚠️ **Ela existe separada por causa do `OnceLock` da porta:** a env var é lida UMA vez por
/// processo, logo um gate que a escrevesse mediria o que o vizinho já tinha fixado — a armadilha
/// que este repo já pagou (*«`env VAR=` define a variável VAZIA, e o CONTROLO passou a correr a
/// mesma lei que devia contradizer»*). ⇒ *a lei é parâmetro e a porta é um `if` de uma linha.*
#[must_use]
pub fn assar(mesh: &Mesh2d, pesos: &[f64], ossos: usize) -> Option<(Mesh2d, Vec<f64>)> {
    if ossos == 0 || mesh.rest.is_empty() || mesh.tris.is_empty() {
        return None;
    }
    let lei = AttrLaw::Hermite { values: ossos };
    let tau = tolerancia_do_bind(mesh);
    let attrs = hermite_attrs(mesh, pesos, ossos);
    let (assada, attrs, rel) = refine_rest_by_attrs(
        mesh,
        &attrs,
        ossos * 3,
        lei,
        RefineOptions {
            tolerance_px: tau,
            max_pieces: mesh.tris.len().saturating_mul(CRESCIMENTO_MAX),
            adaptativo: true,
        },
    );
    // ⚠️ **A linha sai ANTES da desistência, e é a diferença entre medir e adivinhar:** *«o
    // assador não partiu nada»* e *«o assador nem correu»* dão o mesmo silêncio, e a 1.ª medição
    // desta wave leu exactamente isso — nenhuma linha no log, e duas explicações possíveis.
    // Ela é presa à porta (e não a `assar`) para os gates, que chamam a lei directamente, ficarem
    // calados.
    if ligada() {
        eprintln!(
            "[bone] assadura do bind: {} -> {} pecas ({:.2}x), desvio {:?} de \
             {tau:.2e} (pesos: {} vertices x {ossos} ossos)",
            mesh.tris.len(),
            assada.tris.len(),
            assada.tris.len() as f64 / mesh.tris.len() as f64,
            rel.desvio,
            mesh.rest.len()
        );
    }
    if assada.tris.len() == mesh.tris.len() {
        return None;
    }
    // ⚠️ **Os VALORES saem de dentro do formato de Hermite** — os `2 × ossos` gradientes que viajam
    // atrás deles descrevem a mesma curvatura uma segunda vez e não se guardam: o
    // [`crate::skinned_mesh::SkinnedMesh`] declara `pesos[v · ossos + j]`, e escrever lá outra
    // coisa seria a terceira grandeza que aquele doc proíbe.
    let mut so_valores = Vec::with_capacity(assada.rest.len() * ossos);
    for v in 0..assada.rest.len() {
        let base = v * ossos * 3;
        so_valores.extend_from_slice(attrs.get(base..base + ossos)?);
    }
    if rel.travado_pelo_orcamento {
        eprintln!(
            "[bone] a assadura do bind gastou o tecto de {} pecas ({} -> {}, desvio {:?}): o campo \
             de pesos desta arte curva mais do que a tolerancia {tau:.2e} pede",
            mesh.tris.len() * CRESCIMENTO_MAX,
            mesh.tris.len(),
            assada.tris.len(),
            rel.desvio
        );
    }
    Some((assada, so_valores))
}

#[cfg(test)]
#[path = "skin_bake_tests.rs"]
mod tests;
