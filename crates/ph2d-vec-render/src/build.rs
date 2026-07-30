//! **De `VecPath` a `BezPath`** — os construtores de desenho (módulo irmão, teto de LOC).
//!
//! Aqui mora a única tradução entre o modelo do documento (âncora + dois handles, estilo Rive) e a
//! curva que o Vello encoda. Três perguntas, três portas: o desenho INTEIRO (o traço leva tudo), só
//! os contornos FECHADOS (o preenchimento — um contorno aberto não tem interior) e só os ABERTOS (as
//! linhas de construção).
//!
//! ⚠️ **Cozer e CONSTRUIR são passos separados de propósito** (W0.3 do plano 25): `cooked()` roda a
//! pilha de Live Path Effects e o arredondamento de quina, e o [`crate::draw_path`] precisa de até
//! DOIS desenhos da mesma forma. Cozer por construção fazia a pilha correr **duas vezes por forma
//! por frame**; hoje o cozido é feito uma vez e as construções leem dele
//! ([`build_contours`]). Gateado por CONTAGEM em `encode_cost_tests`.

use ph2d_vec_scene::VecPath;
use ph2d_vector::BezPath;

use crate::pt;

/// Constrói o `BezPath` (world-space) de um path editável: para CADA contorno
/// (primário + `subpaths`), `move_to` na 1ª âncora, depois uma cúbica por segmento
/// usando `out_handle(i)` e `in_handle(i+1)`; fecha com uma cúbica final se
/// `closed`. Um compound vira um só `BezPath` de vários sub-caminhos — é a
/// [`Fill`] rule que decide o que é buraco.
pub fn build_bezpath(path: &VecPath) -> BezPath {
    build_path(path, None)
}

/// O path do **PREENCHIMENTO** — só os contornos FECHADOS.
///
/// **Um contorno aberto não tem interior.** Ele é uma linha de construção: as três arestas
/// internas do cubo isométrico, a boca da base do cone, a tampa do cilindro, as barras da
/// sub-rotina, a cruz da junção. Essas coisas se DESENHAM, não se preenchem.
///
/// Sem esta distinção, o preenchimento **fecha cada contorno aberto implicitamente** (é a
/// semântica de fill de qualquer rasterizador) e a corda que fecha a linha de construção
/// vira uma região com winding próprio — que, com `NonZero`, CANCELA a silhueta onde
/// coincide. Foi exatamente o que o Enio fotografou no cubo: as arestas internas
/// `V1 → M → V3`, fechadas pela corda `V3 → V1`, abriam um triângulo escuro no meio da face
/// direita. O cone e o cilindro tinham a mesma doença em forma de lente (o arco aberto
/// fechado pela sua corda), só que menos visível.
///
/// O traço (`build_bezpath`) continua levando TUDO — é ele que desenha as linhas de
/// construção, que é a razão de elas existirem.
#[must_use]
pub fn build_fill_bezpath(path: &VecPath) -> BezPath {
    build_path(path, Some(true))
}

/// As **linhas de construção** — só os contornos ABERTOS. É o complemento exato de
/// [`build_fill_bezpath`]: o que dá volume ao sólido (as arestas internas do cubo, a boca
/// do cone, a tampa do cilindro) e que o preenchimento tem de ignorar.
///
/// Vazio para as 40 formas que não têm sub-contorno aberto.
#[must_use]
pub fn build_lines_bezpath(path: &VecPath) -> BezPath {
    build_path(path, Some(false))
}

/// `want`: `None` = todos os contornos · `Some(true)` = só os fechados · `Some(false)` = só
/// os abertos.
///
/// A geometria COZIDA: as quinas com `corner_radius` já viraram arco. O documento guarda a quina
/// afiada + o raio; o que se PINTA é isto. (Os overlays de âncora continuam na fonte — ver
/// `draw_overlays` — senão o usuário veria dois vértices onde autorou um.)
fn build_path(path: &VecPath, want: Option<bool>) -> BezPath {
    let cooked = path.cooked();
    #[cfg(test)]
    crate::encode_cost_tests::count_cook();
    build_contours(&cooked, want)
}

/// Os contornos de um path **JÁ COZIDO** → `BezPath`.
///
/// ⚠️ **Separado do [`build_path`] de propósito:** cozinhar roda a pilha de Live Path Effects
/// (ADR-0132) e o arredondamento de quina, e o [`draw_path`] precisa de ATÉ DOIS desenhos da mesma
/// forma (o preenchimento e o traço). Cozer por desenho fazia a pilha inteira correr **duas vezes
/// por forma por frame**; agora o cozido é feito uma vez e os dois leem dele.
pub(crate) fn build_contours(path: &VecPath, want: Option<bool>) -> BezPath {
    // O CONTADOR do gate de orçamento (`encode_cost_tests`): quantos desenhos este frame construiu.
    // Contar é exato onde cronometrar não é — a razão entre um encode de FILL e um de STROKE não
    // isola uma construção de caminho, porque os dois não fazem o mesmo trabalho no Vello (medido).
    #[cfg(test)]
    crate::encode_cost_tests::count_build();
    let mut bp = BezPath::new();
    for c in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(c) else {
            continue;
        };
        if want.is_some_and(|w| w != closed) {
            continue;
        }
        let Some(first) = verts.first() else {
            continue;
        };
        bp.move_to(pt(first.anchor));
        for pair in verts.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            bp.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
        }
        if closed && verts.len() >= 2 {
            let last = verts.last().unwrap();
            bp.curve_to(pt(last.out_handle), pt(first.in_handle), pt(first.anchor));
            bp.close_path();
        }
    }
    bp
}
