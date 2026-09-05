//! **ASSAR UM AFIM na geometria de um caminho** — módulo irmão de [`super::path_ops`] pelo teto de
//! LOC, e o corte é por RESPONSABILIDADE: o `path_ops` **move e mede** um caminho no frame dele;
//! aqui um afim entra na geometria e o frame **desaparece**.
//!
//! ⚠️ É o que reconcilia frames diferentes antes de uma operação de geometria (booleana, merge,
//! offset): os operandos vêm de entidades com `Transform` distintos, e um resultado só pode viver
//! num frame.

use crate::{Paint, StrokeSpec, VecPath};

/// **Assa** o afim `x` na geometria de `path`: âncoras, handles e a geometria do
/// gradiente passam a estar no frame de destino.
///
/// É o que reconcilia frames diferentes antes de uma operação de geometria
/// (booleana, merge, offset): os operandos vêm de entidades com `Transform`
/// distintos, e um resultado só pode viver num frame. Assando os operandos no
/// MUNDO, o resultado nasce em world-space — e a entidade nova dele, na identidade,
/// o desenha exatamente onde as formas de origem estavam.
///
/// Identidade é no-op.
pub fn bake_xform(path: &mut VecPath, x: &crate::Xform) {
    if x.is_identity() {
        return;
    }
    let f = |p: [f64; 2]| x.apply(p);
    path.for_each_vert_mut(|v| {
        v.anchor = f(v.anchor);
        v.in_handle = f(v.in_handle);
        v.out_handle = f(v.out_handle);
    });
    transform_fill_geometry(path, f, x.mean_scale(), x);
}

/// Aplica a transformação de ponto `f` (a MESMA das âncoras) à geometria world-space
/// do gradiente do fill, e escala por `radius_scale` **todo comprimento escalar do
/// path**: o raio do gradiente radial e o `corner_radius` de cada vértice.
///
/// Os dois são a mesma espécie de quantidade — **um comprimento que não é um ponto** —
/// e é por isso que moram na mesma função. Um op que transforma a geometria transforma
/// os pontos com `f` e os comprimentos com `radius_scale`; se os dois comprimentos
/// vivessem em funções separadas, o próximo op novo escalaria um e esqueceria o outro,
/// e a quina de uma forma escalada arredondaria errado sem nada ficar vermelho.
///
/// Sob escala NÃO-uniforme um raio escalar é indefinido (a quina viraria elíptica); o
/// fator médio dos eixos é a mesma aproximação que o gradiente radial já faz.
/// No-op para `Solid` / sem fill.
pub(crate) fn transform_fill_geometry(
    path: &mut VecPath,
    f: impl Fn([f64; 2]) -> [f64; 2],
    radius_scale: f64,
    x: &crate::Xform,
) {
    if radius_scale != 1.0 {
        path.for_each_vert_mut(|v| v.corner_radius *= radius_scale);
    }
    // ⛔⛔⛔ **A LARGURA DO TRAÇO É O TERCEIRO COMPRIMENTO, e ficou de fora da lista** — report do
    // Enio de 2026-08-30 (*"a proporção muda no stroke (estica/achata o tile)"*).
    //
    // ⚠️⚠️ **O doc acima PREVIU este defeito, palavra por palavra:** *"se os dois comprimentos
    // vivessem em funções separadas, o próximo op novo escalaria um e esqueceria o outro"*. A
    // largura nunca esteve em função nenhuma — medido: sob um `scale_path` UNIFORME de `2×` o
    // ladrilho da estampa duplicava (`4,0 × 2,0`) e a banda ficava em `1,0000`.
    //
    // ⚠️ **E ela tem a lei da CANETA, não a do CAMINHO**: `√|det|` (a média geométrica), que é a
    // decisão do dono no bug #27 — *"quando engrossa, engrossa por igual nos dois eixos"*. Sob
    // `(3, 1)` as duas médias dão `2,000` e `1,732`, e usar a do caminho poria a banda e o motivo
    // outra vez em desacordo, um passo mais subtil.
    let caneta = x.uniform_scale();
    if let Some(s) = path.stroke.as_mut() {
        s.width *= caneta;
    }
    // ⭐⭐⭐ **E A PILHA DE APARÊNCIA INTEIRA** (v20). ⛔ As três cicatrizes deste ficheiro são todas
    // *código que diz `fill` e devia dizer **tinta***; uma pilha de N tintas é a quarta
    // oportunidade de repetir a conta, e a cura é a mesma: a lei de cada espécie mora numa função
    // ([`transform_paint`] / [`transform_stroke`]) que o chão e as camadas PARTILHAM, em vez de
    // uma cópia que envelhece.
    for e in &mut path.paints {
        match &mut e.kind {
            crate::PaintKind::Fill(p) => transform_paint(p, &f, radius_scale),
            crate::PaintKind::Stroke(s) => transform_stroke(s, x, caneta),
        }
    }
    if let Some(p) = path.fill.as_mut() {
        transform_paint(p, &f, radius_scale);
    }
    // ⭐⭐⭐ **E A ESTAMPA DO CONTORNO TAMBÉM** (auditoria de 2026-08-30).
    //
    // ⛔ A lei acima estava escrita **só para o preenchimento**: este `match` lê `path.fill` e mais
    // nada. ⇒ rodar ou escalar uma forma cujo CONTORNO tem estampa deixava-a exactamente onde
    // estava — o ângulo, o tamanho e a origem dela não seguiam a forma —, enquanto a do
    // preenchimento seguia. Uma forma com as DUAS tintas estampadas rodava com metade do desenho.
    //
    // ⚠️ *Uma lei escrita para uma das duas tintas não é uma lei — é um acidente que ainda não foi
    // encontrado.* É a terceira vez que esta linha paga a mesma conta (o colector do ficheiro e o
    // memo do assado foram as outras duas), e as três tinham a mesma forma: código que diz `fill` e
    // devia dizer *tinta*.
    // ⛔⛔ **A ESTAMPA DO TRAÇO SEGUE A CANETA, não a forma** (report de 2026-08-30).
    //
    // O preenchimento estica com a forma — é a lei dele, e está certa. Um traço **não estica**: a
    // caneta é uniformizada por `√|det|` desde o bug #27. ⇒ com a lei do preenchimento aqui, o
    // motivo esticava dentro de uma banda que não esticava. Medido: com `(3, 1)`, o ladrilho ia a
    // um aspecto **3,00×** o autorado enquanto a banda ficava redonda.
    //
    // ⚠️ **A conta é a MESMA função** — o que muda é o afim que ela recebe: a parte CONFORME, que
    // devolve um afim já conforme **ao bit** (logo o caminho comum não se mexe).
    if let Some(pat) = path.stroke.as_mut().and_then(StrokeSpec::pattern_mut) {
        let u = x.uniform_part();
        transform_pattern(pat, &|p| u.apply(p));
    }
}

/// **A lei de UMA tinta de preenchimento** — a geometria world-space dela segue a forma.
///
/// ⭐ Extraída em 2026-09-05 (v20) porque passou a ter N chamadores: o chão da pilha e cada camada
/// de preenchimento dela. *Uma lei com dois donos escrita duas vezes é a forma exacta das três
/// cicatrizes que este ficheiro carrega.*
fn transform_paint(p: &mut Paint, f: &impl Fn([f64; 2]) -> [f64; 2], radius_scale: f64) {
    match p {
        Paint::Linear { start, end, .. } => {
            *start = f(*start);
            *end = f(*end);
        }
        Paint::Radial { center, radius, .. } => {
            *center = f(*center);
            *radius *= radius_scale;
        }
        Paint::MultiPoint { points } => {
            for pt in points {
                pt.pos = f(pt.pos);
            }
        }
        Paint::Pattern(pat) => transform_pattern(pat, f),
        Paint::Solid(_) => {}
    }
}

/// **A lei de UM contorno** — a largura sofre a CANETA (`√|det|`) e a estampa dele segue a parte
/// conforme, nunca a do caminho. Ver as duas notas longas no corpo da [`transform_fill_geometry`].
fn transform_stroke(s: &mut StrokeSpec, x: &crate::Xform, caneta: f64) {
    s.width *= caneta;
    if let Some(pat) = s.pattern_mut() {
        let u = x.uniform_part();
        transform_pattern(pat, &|p| u.apply(p));
    }
}

/// A pose de UM padrão sob o afim `f` — a lei que as duas tintas partilham.
///
/// ⭐⭐ **O padrão SONDA o afim, e por isso é o único preenchimento desta casa que conserva a
/// ORIENTAÇÃO.** As imagens dos dois eixos unitários dizem tudo: o ângulo do eixo x é a rotação, e o
/// comprimento de cada imagem é a escala NAQUELE eixo. É exacto para qualquer afim, e melhor do que
/// o `radius_scale` médio que o gradiente radial recebe — não por esmero, mas porque um radial do
/// peniko **é circular** e não tem onde guardar um ângulo, enquanto o padrão tem.
///
/// ⚠️ **Um espelho vira uma meia-volta**, e a aproximação é nomeada: `atan2` de um eixo invertido
/// dá `π`, e o `PatternFill` não tem campo de reflexão onde guardar a diferença.
fn transform_pattern(pat: &mut crate::PatternFill, f: &impl Fn([f64; 2]) -> [f64; 2]) {
    let o = f(pat.origin);
    let ax = f([pat.origin[0] + 1.0, pat.origin[1]]);
    let ay = f([pat.origin[0], pat.origin[1] + 1.0]);
    let (dx, dy) = ([ax[0] - o[0], ax[1] - o[1]], [ay[0] - o[0], ay[1] - o[1]]);
    pat.angle += dx[1].atan2(dx[0]);
    pat.size = [
        pat.size[0] * dx[0].hypot(dx[1]),
        pat.size[1] * dy[0].hypot(dy[1]),
    ];
    pat.origin = o;
}

#[cfg(test)]
#[path = "path_bake_xform_tests.rs"]
mod tests;
