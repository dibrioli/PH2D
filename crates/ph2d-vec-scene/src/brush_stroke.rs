//! ⭐⭐⭐ **O PINCEL DE CONTORNO** (plano 36, W2 + W3-bis) — a arte que PERCORRE a linha, em vez da
//! tinta que ela revela.
//!
//! # Os dois modelos, e por que este ficheiro existe ao lado do padrão
//!
//! O [`crate::StrokePaint::Pattern`] é **normativo em SVG 2**: um traço com paint server é a
//! silhueta dele PREENCHIDA, então um tracejado são **buracos** no papel de parede e a arte não os
//! conhece. Isto é o outro modelo — o *Pattern Brush* do Illustrator —, e ele responde ao contrário
//! às MESMAS três perguntas: escala com a largura (W2), **reinicia em cada traço** (W3-bis), e tem
//! quinas (W5, por fazer).
//!
//! # ⭐ O motor já existia, e está pago
//!
//! [`crate::pattern_along`] (plano 23) copia um motivo ao longo de um [`ArcPath`], cada cópia
//! rodada para a tangente — **0,597 ms para 200 cópias × 40 vértices**, ~13× de folga sob o *kill*.
//! O que faltava não era o motor: era **endereçá-lo como uma propriedade do traço** em vez de uma
//! relação entre dois objectos.
//!
//! # ⚠️ A guia é o contorno COZIDO, e a lei é a mesma do traço
//!
//! Quem desenha o traço percorre a geometria **viva** (cantos vivos, largura viva, booleana). O
//! pincel tem de correr sobre a mesma — senão a arte anda por um caminho que ninguém vê.

use crate::arc_path::ArcPath;
use crate::pattern_path::{PatternSpec, pattern_along};
use crate::{BrushStroke, Contour, StrokeSpec, VecPath};

/// Teto de TRAÇOS que um contorno oferece ao pincel — **o recurso é TEMPO**, e o número é medido.
///
/// ⚠️ **O que o tracejado acrescenta ao custo NÃO são as cópias.** Numa fatia de traço de
/// comprimento `d` cabem `d/avanço` cópias, e a soma sobre `total/período` traços é no máximo
/// `total/avanço` — exactamente o mesmo que o contorno inteiro sem tracejado, que o `MAX_COPIES`
/// do [`crate::pattern_path`] já limita. O que cresce com o número de traços é o custo **FIXO por
/// fatia**: uma medida do bbox da arte (um passe sobre os vértices dela) e uma divisão.
///
/// MEDIDO (`measure_the_dashed_brush_recook`, release, arte de 40 vértices, guia de perímetro 200,
/// com o teto levantado a `32768` só para a varredura):
///
/// | traços | cópias | re-cook |
/// |---|---|---|
/// | 1 (sem tracejado) | 200 | 0,27 ms |
/// | 100 | 100 | 0,15 ms |
/// | 400 | 400 | 0,62 ms |
/// | 1 026 | 1 025 | 1,69 ms |
/// | 2 051 | 2 051 | 2,92 ms |
/// | **4 103** | 4 103 | **6,32 ms** |
/// | 8 205 | 8 205 | ⛔ **12,08 ms** |
///
/// ⇒ o joelho está entre `4 103` e `8 205` contra o *kill* de **8 ms** do re-cook por tecla
/// (plano 36 §4), e o teto fica em **4096**. ⭐ É o mesmo número do `MAX_COPIES` do
/// [`crate::pattern_path`] **por medição, não por simetria**: os dois limitam o mesmo recurso — o
/// trabalho de um re-cook — e calham em ordens de grandeza vizinhas.
///
/// ⚠️ **A condição em que o artista o encontra, e o que ele vê:** ele morde quando
/// `comprimento(contorno) > 4096 × período` — com uma largura de traço de `0,03` e um tracejado de
/// `(2, 2)` isso é um perímetro de **~490 unidades de mundo** —, e o que se vê é a arte a **parar a
/// meio do contorno, sem aviso**. É o mesmo sintoma que o `MAX_COPIES` já tem escrito, e a saída é
/// a mesma: o cache por-params do plano 23 §0, que tira o re-cook do quadro. ⛔ **Não** subir o
/// número com o custo onde está.
const MAX_DASHES: usize = 4096;

/// **A ALTURA que uma cópia recebe** — derivada da largura da faixa, e multiplicada pelo `scale`.
///
/// ⚠️⚠️ **É aqui que os dois modelos divergem, e é deliberado.** O plano 35 §2.3 fixou que uma
/// TINTA **não** escala com a largura (*"a largura decide a faixa; o padrão decide o que a
/// preenche"*) — a queixa clássica do Illustrator, do lado certo. Um pincel é o oposto **porque ele
/// É a faixa**: engrossar o traço engrossa a arte, que é o que o *Pattern Brush* faz.
#[must_use]
pub fn brush_height(b: &BrushStroke, width: f64) -> f64 {
    let h = width * b.scale;
    if h.is_finite() && h > 0.0 { h } else { 0.0 }
}

/// A arte **escalada** para a altura da faixa, centrada em zero.
///
/// ⚠️ **Um factor ÚNICO nos dois eixos.** Escalar só a altura esmagaria o motivo, e um pincel que
/// deforma a arte é o *Art Brush*, não o *Pattern Brush* — outra ferramenta, com outro nome, que o
/// plano 36 §3.4 deixa fora de propósito.
///
/// `None` quando a arte não tem altura que se meça (um ponto, um caminho vazio): não há factor
/// honesto, e desenhar nada é melhor que dividir por quase-zero.
#[must_use]
fn art_at_height(art: &VecPath, h: f64) -> Option<VecPath> {
    // ⚠️ `<=` e não `!(_ > _)`: os dois recusam o NaN, e é a forma que o `dash_fit`
    // desta crate já escolheu — *duas formas de recusar a mesma coisa lêem-se como duas leis*.
    if h <= 0.0 || h.is_nan() {
        return None;
    }
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in art.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    let alt = hi[1] - lo[1];
    if !seen || alt <= 0.0 || alt.is_nan() {
        return None;
    }
    let k = h / alt;
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let map = |p: [f64; 2]| [(p[0] - c[0]) * k, (p[1] - c[1]) * k];
    let mut out = art.clone();
    let escala = |verts: &mut Vec<crate::VecVertex>| {
        for v in verts.iter_mut() {
            v.anchor = map(v.anchor);
            v.in_handle = map(v.in_handle);
            v.out_handle = map(v.out_handle);
            // ⚠️ **O raio de quina é um COMPRIMENTO LOCAL** e escala junto — esquecê-lo faria a
            // quina viva de uma cópia grande ficar do tamanho da de uma pequena.
            v.corner_radius *= k;
        }
    };
    escala(&mut out.verts);
    for c in &mut out.subpaths {
        escala(&mut c.verts);
    }
    Some(out)
}

/// ⭐⭐⭐ **AS FATIAS de arco que um contorno oferece ao pincel** (plano 36, W3-bis) — a lei do
/// tracejado, pura.
///
/// - **Sem tracejado**: uma fatia, o contorno inteiro.
/// - **Com tracejado**: uma fatia por TRAÇO, e **a arte reinicia em cada uma** — que é a lei do
///   *Pattern Brush* do Illustrator, e é o que o Enio pediu ao juntar as duas coisas
///   (*"mas não posso usar o dash com pattern?"*).
///
/// ⚠️ **O `dash` chega já ENCAIXADO** (`[traço, vão]` em comprimento, saído da
/// [`crate::dash_fit::dash_lengths_for`]) — o mesmo par que o traçador usaria para desenhar a
/// linha. *Se esta função reescalasse por conta própria, a arte cairia noutros sítios que os traços
/// da mesma forma, e as duas respostas divergiriam no primeiro ajuste de largura.*
///
/// ⚠️ **A última fatia é TRUNCADA no fim do contorno**, exactamente como um traçador faz: um
/// composto tem um par de tracejado só (fitado ao contorno mais longo — vide
/// [`crate::dash_fit::longest_contour`]), então os outros anéis acabam a meio de um traço.
///
/// Vazio quando não há contorno que se meça — um comprimento nulo não tem fatia nenhuma, e isso é
/// diferente de *"uma fatia de comprimento nulo"*.
#[must_use]
pub fn brush_spans(total: f64, dash: Option<[f64; 2]>) -> Vec<(f64, f64)> {
    if total <= 0.0 || total.is_nan() {
        return Vec::new();
    }
    let inteiro = vec![(0.0, total)];
    let Some([traco, vao]) = dash else {
        return inteiro;
    };
    let periodo = traco + vao;
    // ⚠️ `<=` e não `!(_ > _)`: a forma que o `dash_fit` desta crate já escolheu, e as duas
    // recusam o NaN — *duas formas de recusar a mesma coisa lêem-se como duas leis*.
    if traco <= 0.0 || periodo <= 0.0 || traco.is_nan() || periodo.is_nan() {
        return inteiro;
    }
    let mut out = Vec::new();
    let mut s = 0.0;
    while s < total && out.len() < MAX_DASHES {
        out.push((s, (s + traco).min(total)));
        s += periodo;
    }
    out
}

/// ⭐⭐ **AS CÓPIAS que um pincel põe sobre `guia`** — a porta única do modelo B.
///
/// - `guia` é o contorno **cozido** da forma (o que o traço de facto percorre);
/// - `art` é a forma que se repete;
/// - `width` é a largura do traço, de que a altura da arte é derivada;
/// - `dash` é o `[traço, vão]` **já encaixado** do traçador, ou `None` para linha contínua.
///
/// ⚠️ **O encaixe é LIGADO** (`fit_span`): num contorno fechado, a cauda que sobra é um vão
/// encostado a uma cópia inteira **sempre na mesma quina** — o defeito que o `dash_fit` já curou
/// para o tracejado, e a mesma porta o cura aqui.
///
/// ⭐⭐ **E o comprimento que o avanço fecha é o do TRAÇO quando há tracejado**, não o do contorno:
/// é isso que faz cada traço começar e acabar com uma cópia inteira. ⚠️ **O alvo é UM para todas as
/// fatias**, de propósito — assim todo traço leva o mesmo ritmo, e a fatia truncada do fim de um
/// anel simplesmente leva menos cópias em vez de mudar de cadência à vista.
///
/// Vazio quando não há o que desenhar (arte degenerada, contorno sem comprimento, largura zero) —
/// e quem chama pinta a **cor de recurso**, que é desenho certo, não desistência.
#[must_use]
pub fn brush_copies(
    guia: &Contour,
    art: &VecPath,
    b: &BrushStroke,
    width: f64,
    dash: Option<[f64; 2]>,
) -> Vec<VecPath> {
    let Some(escalada) = art_at_height(art, brush_height(b, width)) else {
        return Vec::new();
    };
    let Some(arc) = ArcPath::from_contour(&guia.verts, guia.closed) else {
        return Vec::new();
    };
    let total = arc.total();
    let alvo = dash.map_or(total, |[traco, _]| traco);
    let mut out = Vec::new();
    for (inicio, fim) in brush_spans(total, dash) {
        out.extend(pattern_along(
            &escalada,
            &arc,
            &PatternSpec {
                spacing: b.spacing,
                offset: b.offset,
                flip: b.flip,
                rotation_deg: b.rotation_deg,
                fit_span: Some(alvo),
                start_offset: inicio,
                end_offset: fim,
            },
        ));
    }
    out
}

/// **Todas as cópias de um pincel sobre TODOS os contornos de `path`.**
///
/// ⚠️ **Recebe o `StrokeSpec` INTEIRO, e não `(pincel, largura)`.** A arte, a largura da faixa e o
/// tracejado saem os três do mesmo traço; passá-los soltos abriria a porta a desenhar a arte de um
/// traço na largura de outro, e o compilador não teria como dizê-lo.
///
/// ⚠️ **O tracejado sai pela porta do TRAÇADOR** ([`crate::dash_fit::dash_lengths_for`], que mede um
/// caminho **já cozido** — que é o que esta função recebe). *Uma segunda medição poria a arte
/// noutra cadência que a linha desenharia, e o artista veria dois tracejados sobre a mesma forma.*
///
/// ⚠️ **Contorno a contorno, e não o mais longo.** O `dash_fit` escolhe o contorno mais longo
/// porque o traçador recebe **um** par `[traço, vão]` para o caminho inteiro; o encaixe do AVANÇO
/// não tem essa restrição — cada contorno recebe as suas cópias e **fecha exactamente**, o que é
/// estritamente melhor. *Uma limitação herdada sem se perguntar se ela ainda existe é uma limitação
/// inventada.*
///
/// Vazio quando o traço não é um pincel — a pergunta *"que cópias este traço põe?"* não tem resposta
/// num traço sólido, e devolver as de um pincel inventado seria pior que devolver nada.
#[must_use]
pub fn brush_along_path(path: &VecPath, art: &VecPath, s: &StrokeSpec) -> Vec<VecPath> {
    let Some(b) = s.brush() else {
        return Vec::new();
    };
    let dash = crate::dash_fit::dash_lengths_for(path, s);
    let mut out = Vec::new();
    let principal = Contour {
        verts: path.verts.clone(),
        closed: path.closed,
    };
    for c in std::iter::once(&principal).chain(path.subpaths.iter()) {
        out.extend(brush_copies(c, art, b, s.width, dash));
    }
    out
}

#[cfg(test)]
#[path = "brush_stroke_engine_tests.rs"]
mod tests;
