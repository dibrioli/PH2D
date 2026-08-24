//! `ph2d-field-profile` — **o desenho do editor vetorial virando perfil de sólido** ([ADR-0161]).
//!
//! Uma crate, uma pergunta: *como é que um `VecPath` se torna um [`Profile`]?* É aqui que o fluxo do
//! MoI renasce sobre a caneta que a casa já tem — desenha-se o contorno no editor de vetores e
//! extruda-se ou revoluciona-se.
//!
//! # ⭐ O arredondamento de quina do perfil vem de graça, e é de propósito
//!
//! O cozimento parte de [`VecPath::cooked`], que é a geometria **já com os Live Corners aplicados**
//! ([ADR-0121]). Logo o *corner widget* do editor vetorial é o arredondamento das arestas verticais
//! da extrusão — o módulo 3D não tem, e não deve ter, uma segunda resposta para "arredondar a quina
//! de um contorno". *Uma quina, um dono.*
//!
//! A pilha de Live Path Effects ([ADR-0132]) entra pelo mesmo caminho: `cooked()` já a correu.
//!
//! # ⚠️ O que é COZIDO aqui não é reversível, e é por isso que a tolerância viaja junto
//!
//! O que sai é uma polilinha. A curva original fica no documento vetorial, que continua a ser a
//! **fonte**; o [`Profile`] é o **cozido**, e leva dentro de si a tolerância com que foi feito, para
//! que "este perfil está bom?" tenha resposta sem adivinhação.
//!
//! # O eixo Y
//!
//! ⚠️ **A conversão não vira nem espelha nada**: o `(x, y)` do path é o `(x, y)` do perfil, e há
//! gate a fixá-lo. Se o plano de desenho de uma ferramenta tiver o Y para baixo, quem espelha é a
//! **ferramenta**, na hora de escolher o plano — não esta função, que não sabe de que plano o
//! desenho veio.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0121]: ../../../docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use kurbo::{BezPath, PathEl, Point};
use ph2d_field::{FillRule, Profile, ProfileError};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Tolerância de achatamento como **fração da maior dimensão do perfil**, usada por
/// [`cook_path_auto`].
///
/// # ⭐⭐ A régua certa é a NORMAL, não a silhueta (medido 2026-08-23, ADR-0161 W54)
///
/// Enio, no smoke da W53, com duas fotos: *"contudo sem ajustes de resolução"* — a silhueta lisa e a
/// **luz em degraus**. O primeiro número deste `const` foi escolhido pela **flecha** (o erro de
/// posição), e a flecha é a grandeza que o olho **menos** vê aqui:
///
/// | fração | arestas | flecha (% de D) | salto de NORMAL | extrusão | torno | px com degrau |
/// |---:|---:|---:|---:|---:|---:|---:|
/// | `1e-3` (o primeiro) | 56 | **0,079 %** | **6,43°** | 53,3 ms | 65,5 ms | **4 417** |
/// | `3e-4` | 96 | 0,027 % | 3,75° | 92,5 ms | 104,7 ms | 6 765 |
/// | **`1e-4`** (shipa) | **168** | 0,009 % | **2,14°** | **139,3 ms** | **178,8 ms** | **79** |
/// | `3e-5` | 305 | 0,003 % | 1,18° | 237,3 ms | 286,0 ms | 86 |
/// | `1e-5` | 528 | 0,0009 % | 0,68° | 409,3 ms | 511,1 ms | 70 |
///
/// (círculo de raio 0,5 num torno e numa extrusão, 640×480, mediana de 7, `load < 3`.)
///
/// ⭐ **A silhueta erra 0,079 % da peça — invisível. A normal salta 6,43° — e é isso que a luz
/// mostra.** O `1e-4` é o **joelho**: os degraus caem **56×** (4 417 → 79 pixels), e o passo
/// seguinte custa **+70 %** para não melhorar nada (79 → 86, ruído).
///
/// ⚠️ **A coluna dos degraus é acoplada ao limiar** com que é contada (3° entre pixels vizinhos), e
/// ela diz isso alto ao **não ser monótona**: em `3e-4` há mais facetas e cada uma ainda passa dos
/// 3°, então o número **sobe** antes de colapsar. O que decide é o **salto de normal**; a contagem
/// é a ilustração de o limiar ter sido cruzado, não a prova.
///
/// ⚠️ **O limiar de 3° veio das duas fotos do Enio** — é onde o nosso sombreamento mostra o degrau.
/// É um oráculo de aparência, e está declarado como tal.
///
/// # ⛔ A tabela ANTERIOR está desmentida, e não era o perfil
///
/// Ela dizia *"64 arestas → 24,1 ms"* (2026-08-19) e justificava o `1e-3` com *"o baseline do módulo
/// custa 25 ms, então ~64 arestas é o orçamento"*. Medido hoje, a **mesma** extrusão com 56 arestas
/// custa **53,3 ms** — o traçado ficou **~2,4× mais caro** desde então, e ninguém o reconferiu.
/// ⚠️ Isso é um achado por explicar (o anti-serrilhado adaptativo re-amostra cada pixel de borda
/// **quatro** vezes, e entrou depois), **não** uma consequência desta wave. Ver `docs/3DModeling/06`
/// §55.
///
/// ⚠️ E o **preço interativo** que aquele orçamento protegia deixou de ser este número: desde a W24 a
/// resolução do preview sai do **relógio** (grosso enquanto a mão mexe, nítido ao assentar) e desde a
/// W32 o traçado **cede à mão**. O que a tabela acima mede é o traçado **assente**, que se paga uma
/// vez. *Quem move o número que tornava algo inalcançável tem de reconferir a nota* — e foram a W24 e
/// a W32 que o moveram.
///
/// ⚠️ **É uma FRAÇÃO e não um absoluto** porque a mesma peça desenhada em milímetros ou em metros
/// tem de sair com a mesma qualidade — uma tolerância absoluta faria a unidade do documento decidir
/// a suavidade da forma.
pub const TOLERANCE_RATIO: f64 = 1e-4;

/// Por que um path não pôde virar perfil.
#[derive(Clone, Debug, PartialEq)]
pub enum CookError {
    /// Um contorno **aberto**. Um perfil delimita área, e um contorno aberto não delimita nada.
    ///
    /// ⚠️ Recusa em vez de ignorar: saltar o contorno aberto em silêncio daria um sólido que é *quase*
    /// o desenho, e a diferença só apareceria como um buraco que não fechou.
    OpenContour { contour: u32 },
    /// O path não tem contorno nenhum com pontos.
    Empty,
    /// A polilinha saiu, e o documento a recusou. Ver [`ProfileError`].
    Rejected(ProfileError),
}

/// **Coze um path do editor vetorial num perfil**, com a tolerância dada em unidades do documento.
///
/// # Errors
/// Ver [`CookError`].
pub fn cook_path(path: &VecPath, tolerance: f64) -> Result<Profile, CookError> {
    // A geometria COZIDA: Live Corners e a pilha de efeitos já correram. É a forma que está na tela,
    // e é ela que tem de virar sólido.
    let path = &*path.cooked();

    let mut contours: Vec<Vec<[f32; 2]>> = Vec::with_capacity(path.contour_count());
    for c in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(c) else {
            continue;
        };
        if verts.is_empty() {
            continue;
        }
        if !closed {
            return Err(CookError::OpenContour { contour: c as u32 });
        }
        if verts.len() < 2 {
            // Um contorno "fechado" de um ponto só não tem aresta nenhuma — deixa-se cair, e o
            // `Profile` recusa depois se não sobrar nada.
            continue;
        }
        contours.push(flatten_contour(verts, tolerance));
    }
    if contours.is_empty() {
        return Err(CookError::Empty);
    }
    Profile::new(contours, fill_rule(path.fill_rule), tolerance as f32).map_err(CookError::Rejected)
}

/// ⭐⭐ **A TOLERÂNCIA DE UM NÍVEL** (W55) — a lei que traduz o número do artista.
///
/// O nível `1` é o joelho que a tabela do [`TOLERANCE_RATIO`] mediu, e cada nível **divide** a
/// tolerância por ele. Numa curva suave a contagem de arestas anda com `tol^-1/2`, então o nível `4`
/// não custa quatro vezes: custa **duas** (medido — ver [`ph2d_field::MAX_PROFILE_RESOLUTION`]).
///
/// ⚠️ **O piso é 1 e não é conforto:** abaixo do joelho estão exactamente os degraus de luz que a
/// W54 acabou de matar, e oferecê-los seria devolver o defeito com um rótulo por cima. Quem quiser
/// mais barato tem o preview grosso, que já sai do relógio e não da autoria.
///
/// ⚠️ **Clampa em vez de recusar** porque é uma função **pura de leitura**: quem valida a escrita é
/// [`ph2d_field_ecs::set_param`], e uma segunda recusa aqui daria duas respostas à mesma pergunta.
#[must_use]
pub fn tolerance_ratio_for(level: u32) -> f64 {
    f64::from(level.clamp(1, ph2d_field::MAX_PROFILE_RESOLUTION)).recip() * TOLERANCE_RATIO
}

/// **Coze o path no NÍVEL dado**, com a tolerância derivada do tamanho do desenho.
///
/// É a porta normal: uma tolerância absoluta obriga quem chama a saber a escala do documento, e
/// errar nisso é ou um perfil facetado ou um traçado dez vezes mais caro.
///
/// # Errors
/// Ver [`CookError`].
pub fn cook_path_at(path: &VecPath, level: u32) -> Result<Profile, CookError> {
    let cooked = path.cooked();
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for c in 0..cooked.contour_count() {
        let Some((verts, _)) = cooked.contour(c) else {
            continue;
        };
        for v in verts {
            // ⚠️ Os HANDLES entram na conta, e não só as âncoras: uma curva pode sair da caixa das
            // âncoras, e uma tolerância derivada de uma caixa pequena demais faz o achatamento
            // trabalhar de mais exatamente onde a curva é mais larga.
            for p in [v.anchor, v.in_handle, v.out_handle] {
                for k in 0..2 {
                    min[k] = min[k].min(p[k]);
                    max[k] = max[k].max(p[k]);
                }
            }
        }
    }
    let span = (max[0] - min[0]).max(max[1] - min[1]);
    let ratio = tolerance_ratio_for(level);
    // Um desenho sem extensão não tem escala de onde tirar tolerância; o `Profile` recusa-o logo a
    // seguir, e um número positivo qualquer chega lá.
    let tolerance = if span.is_finite() && span > 0.0 {
        span * ratio
    } else {
        ratio
    };
    cook_path(&cooked, tolerance)
}

/// O mesmo no nível de omissão ([`ph2d_field::DEFAULT_PROFILE_RESOLUTION`]) — a porta de quem ainda
/// não tem opinião sobre finura, que é toda a gente até abrir o painel.
///
/// # Errors
/// Ver [`CookError`].
pub fn cook_path_auto(path: &VecPath) -> Result<Profile, CookError> {
    cook_path_at(path, ph2d_field::DEFAULT_PROFILE_RESOLUTION)
}

fn fill_rule(r: ph2d_vec_scene::FillRule) -> FillRule {
    match r {
        ph2d_vec_scene::FillRule::NonZero => FillRule::NonZero,
        ph2d_vec_scene::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

/// Um contorno fechado de vértices cúbicos → polilinha.
fn flatten_contour(verts: &[VecVertex], tolerance: f64) -> Vec<[f32; 2]> {
    let mut bez = BezPath::new();
    bez.move_to(pt(verts[0].anchor));
    for i in 0..verts.len() {
        let a = &verts[i];
        let b = &verts[(i + 1) % verts.len()];
        // ⚠️ O segmento de FECHO (último → primeiro) entra pelo `%` — é o mesmo laço, e não um caso
        // à parte depois dele. Um caso à parte é onde este tipo de conversão costuma perder a
        // aresta que fecha a figura.
        if a.out_handle == a.anchor && b.in_handle == b.anchor {
            // Reta exata: não passa pelo achatamento. Não é otimização de gosto — mandar uma cúbica
            // degenerada ao flattener é pedir-lhe uma decisão sobre uma curva que não existe.
            bez.line_to(pt(b.anchor));
        } else {
            bez.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
        }
    }
    bez.close_path();

    let mut out: Vec<[f32; 2]> = Vec::new();
    kurbo::flatten(bez, tolerance, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => out.push([p.x as f32, p.y as f32]),
        // `flatten` só emite retas; o fecho é implícito no `Profile` (que não repete o 1º ponto).
        _ => {}
    });
    out
}

fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[cfg(test)]
mod tests;
