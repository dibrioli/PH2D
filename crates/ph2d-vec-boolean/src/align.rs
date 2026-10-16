//! **O alinhamento do traço** — Inner e Outer, o *Align Stroke* do Illustrator.
//!
//! # Zero geometria nova, e isso é o desenho inteiro
//!
//! Um traço centrado de largura `w` cobre `[−w/2, +w/2]` em volta da borda. Então a faixa de
//! largura `2w` cobre `[−w, +w]` — e a metade de dentro é exactamente `banda ∩ interior`, a de
//! fora exactamente `banda − interior`. As duas peças já existem: [`crate::outline_stroke`]
//! produz a banda (com o tracejado e a quina que o renderer pinta), e [`crate::apply`] corta.
//! Escrever um traçador assimétrico seria uma **segunda resposta** a *"que forma tem este
//! traço?"*, e ela divergiria da primeira no dia em que a quina Miter mudasse.
//!
//! # ⛔ Só vale para o traço SÓLIDO — o tracejado anda nos TRILHOS (`BUGS_vector.md` #43)
//!
//! A banda dupla dobra também as PONTAS: cada ponto vira um disco de raio `w` cortado ao meio, e
//! todo vão menor que `2w` FECHA. Medido na cena 24 (traço-ponto-ponto, vãos de `1,5w`, ponta
//! redonda): Centre = **45 peças** (15 traços + 30 pontos de `0,787 w²`); Inner/Outer pela banda
//! dupla = **1 peça** de `117 w²` (95 % da faixa sólida). A lei do tracejado é a do Illustrator: o
//! padrão percorre o **trilho** — a borda da região encolhida (Inner) ou crescida (Outer) de `w/2`
//! — com a caneta AUTORADA, então traços e pontas ficam inteiros e de um lado só ([`rails`]).
//! O sólido fica com o recorte: é a banda exacta, e nas quinas convexas do Inner ela é recta
//! qualquer que seja a junta, o que o trilho com junta Round/Bevel não reproduz.
//!
//! # Por que aqui e não no `stroke_plan`
//!
//! O [`ph2d_vec_scene::stroke_plan`] é a porta única do *"o que este traço desenha?"*, e seria o
//! lugar natural. Ele **não pode**: a `ph2d-vec-boolean` já depende da `ph2d-vec-scene` (o
//! `expand.rs` consome o `stroke_plan`), então chamar a booleana de lá é o ciclo que o cargo
//! recusa por nome. O alinhamento mora um nível acima, e quem o consome são os **dois** leitores
//! do plano — o que PINTA (via `LiveGeometry`) e o que ASSA (o Outline Stroke) —, que é o que
//! impede o desenho e o bake de discordarem.
//!
//! # A lasca NÃO acontece aqui, e a medição corrigiu a afirmação oposta
//!
//! A 1ª versão deste cabeçalho dizia que o recorte reproduz o BUGS #16 (*subtrair uma curva dela
//! mesma depois da volta pelo arranjo deixa resíduo*) — **está errado, e a sonda mediu**: numa
//! forma de borda CURVA, Inner e Outer saem com **uma peça e ZERO lascas** em toda largura testada.
//!
//! O mecanismo é a diferença que eu não tinha visto. No Shape Builder os dois operandos
//! **compartilham a fronteira** (a borda da face é a borda da fonte, recalculada pelo arranjo), e
//! é aí que a tolerância deixa fiapo. Aqui eles **não se tocam**: a linha da forma passa pelo
//! MIOLO da banda — as duas trilhas dela ficam a `w` para cada lado —, então o corte é limpo por
//! construção. O filtro [`crate::expand::drop_slivers`] **fica** (a booleana é a mesma, e uma
//! entrada degenerada não é impossível), mas é **precaução medida, não a espinha**: a mutação que
//! o neutraliza **não sangra**, e isso está gateado e explicado em vez de escondido.

use ph2d_vec_scene::{
    Contour, Paint, StrokeAlign, StrokeSpec, VecPath, VecVertex, reverse_contour,
};

use crate::{BoolOp, apply, apply_many, expand::drop_slivers, outline_stroke};

/// **A faixa de tinta de um traço ALINHADO**, já em forma preenchida com a cor dele.
///
/// `None` significa *"não há alinhamento a executar — pinte o traço como sempre"*, e cobre quatro
/// casos que se leem igual na tela: sem traço · [`StrokeAlign::Centre`] · largura zero · e a
/// geometria que **não delimita um interior** (ver abaixo). `Some` é a faixa; `Some` vazio seria a
/// aniquilação, e a distinção é a mesma que a [`ph2d_vec_render::LiveGeometry`] faz — colapsar as
/// duas faria a forma reaparecer inteira no instante em que o recorte a mata.
///
/// ⚠️ **Só caminhos inteiramente FECHADOS.** *Dentro* e *fora* são perguntas sobre uma região, e
/// um contorno aberto não tem uma; um caminho MISTO (uma silhueta mais uma linha de construção)
/// também não, porque a linha teria de ser desenhada centrada enquanto a silhueta é recortada — e
/// então uma forma só teria duas leis de traço ao mesmo tempo. A recusa é aqui e o painel a
/// espelha pela mesma [`StrokeAlign::needs_a_region`], para o botão nunca prometer o que a
/// geometria devolve como nada.
///
/// ⚠️ **Falha do motor devolve `None`, não vazio.** Um sweep que desiste tem de deixar a arte na
/// tela do jeito que estava; devolver vazio a apagaria em silêncio.
#[must_use]
pub fn aligned_stroke(path: &VecPath) -> Option<Vec<VecPath>> {
    let s = path.stroke.as_ref()?;
    // Lê a COZIDA (o raio vivo, ADR-0121, e os efeitos, ADR-0132): é ela que a banda traça.
    if !s.is_aligned() || !path.all_contours_closed() {
        return None;
    }
    // O tracejado anda nos trilhos (ver o cabeçalho). Sem trilho — a forma é mais fina que `w` e
    // o Inner a aniquila — cai no recorte, que ao menos pinta a faixa.
    if s.dash_lengths().is_some()
        && let Some(ink) = rails(path, s).map(|r| outline_stroke(&r))
        && !ink.is_empty()
    {
        return Some(ink);
    }
    // A banda DUPLA: o mesmo traço com o dobro da largura, que cobre `[−w, +w]`.
    let band = outline_stroke(&VecPath {
        stroke: Some(doubled(s)),
        ..path.clone()
    });
    if band.is_empty() {
        return None; // o motor não respondeu — a arte fica como estava
    }
    // O operando de recorte: a MESMA forma sem traço, e carregando o estilo de TINTA. O
    // `apply_many` toma o estilo do path do TOPO (o último), então dá-lo aqui é o que faz o
    // recorte sair já com a cor do traço — em vez de a carimbar depois, que seria confiar duas
    // vezes na mesma decisão.
    let region = VecPath {
        fill: Some(Paint::Solid(s.color())),
        stroke: None,
        ..path.clone()
    };
    let op = match s.align {
        StrokeAlign::Inner => BoolOp::Intersect,
        StrokeAlign::Outer => BoolOp::Subtract,
        // `is_aligned()` já respondeu por este caso; o braço existe porque o `match` é a porta.
        StrokeAlign::Centre => return None,
    };
    // ⚠️ `Subtract` preserva o de TRÁS: a banda entra primeiro e a região por cima.
    Some(drop_slivers(
        band.iter().flat_map(|b| apply(b, &region, op)).collect(),
    ))
}

/// **Os TRILHOS de um traço alinhado tracejado**: a borda da região encolhida (Inner) ou crescida
/// (Outer) de `w/2`, já com o traço CENTRADO autorado — percorrê-los com a caneta de largura `w`
/// pinta exactamente o lado pedido, com traços e pontas inteiros.
///
/// ⚠️ **No sentido da REGIÃO, nunca do contorno:** é `região ∓ banda(w)`, então o furo de um
/// composto cresce no Inner e encolhe no Outer sem ninguém perguntar a orientação dele.
///
/// ⚠️ **O padrão começa onde o do caminho começa**: o sweep devolve cada contorno a partir de um
/// vértice qualquer e no sentido dele; cada trilho é rodado para o vértice mais perto do início do
/// contorno de origem e virado para o mesmo sentido — senão o traço-ponto-ponto do Inner sairia
/// noutra fase e ao contrário do Centre. O AJUSTE do padrão (`dash_fit`) é o do comprimento do
/// trilho, pela porta de sempre do `outline_stroke`.
fn rails(path: &VecPath, s: &StrokeSpec) -> Option<VecPath> {
    let cooked = path.cooked();
    // Só a geometria: o trilho não pode re-cozer os efeitos da fonte.
    let plain = VecPath {
        verts: cooked.verts.clone(),
        closed: cooked.closed,
        subpaths: cooked.subpaths.clone(),
        fill_rule: cooked.fill_rule,
        fill: Some(Paint::Solid(s.color())),
        ..VecPath::default()
    };
    let centred = StrokeSpec {
        align: StrokeAlign::Centre,
        ..s.clone()
    };
    let band = outline_stroke(&VecPath {
        stroke: Some(StrokeSpec {
            dash: None,
            ..centred.clone()
        }),
        ..plain.clone()
    });
    if band.is_empty() {
        return None;
    }
    let op = match s.align {
        StrokeAlign::Inner => BoolOp::Subtract,
        StrokeAlign::Outer => BoolOp::Union,
        StrokeAlign::Centre => return None,
    };
    let operands: Vec<&VecPath> = std::iter::once(&plain).chain(band.iter()).collect();
    let mut loops: Vec<Vec<VecVertex>> = apply_many(&operands, op)
        .into_iter()
        .flat_map(|p| std::iter::once(p.verts).chain(p.subpaths.into_iter().map(|c| c.verts)))
        .filter(|l| l.len() >= 2)
        .collect();
    let origins: Vec<&[VecVertex]> = (0..plain.contour_count())
        .filter_map(|c| plain.contour(c).map(|(v, _)| v))
        .filter(|v| !v.is_empty())
        .collect();
    for l in &mut loops {
        same_start(l, &origins);
    }
    let mut it = loops.into_iter();
    Some(VecPath {
        verts: it.next()?,
        closed: true,
        subpaths: it.map(Contour::new_closed).collect(),
        stroke: Some(centred),
        ..VecPath::default()
    })
}

/// Roda e vira o trilho `rail` para começar junto do início do contorno de origem mais próximo e
/// percorrer no mesmo sentido dele.
fn same_start(rail: &mut Vec<VecVertex>, origins: &[&[VecVertex]]) {
    let d2 = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
    let near = |start: [f64; 2], r: &[VecVertex]| {
        r.iter()
            .enumerate()
            .map(|(i, v)| (i, d2(v.anchor, start)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    };
    let Some(origin) = origins.iter().min_by(|a, b| {
        let da = near(a[0].anchor, rail).map_or(f64::INFINITY, |x| x.1);
        let db = near(b[0].anchor, rail).map_or(f64::INFINITY, |x| x.1);
        da.total_cmp(&db)
    }) else {
        return;
    };
    if signed_area(rail).signum() != signed_area(origin).signum() {
        reverse_contour(rail);
    }
    if let Some((i, _)) = near(origin[0].anchor, rail) {
        rail.rotate_left(i);
    }
}

/// A área com sinal do polígono das âncoras — só o SENTIDO interessa.
fn signed_area(l: &[VecVertex]) -> f64 {
    l.iter()
        .zip(l.iter().cycle().skip(1))
        .map(|(a, b)| a.anchor[0] * b.anchor[1] - b.anchor[0] * a.anchor[1])
        .sum()
}

/// O mesmo traço com o **dobro da largura** — o insumo dos dois alinhamentos.
///
/// ⚠️ **O tracejado é compensado, e sem isso o Inner sai com a cadência errada.** O campo `dash`
/// guarda MÚLTIPLOS da largura (é o que faz engrossar a linha alongar traço e vão na proporção),
/// então dobrar a largura dobraria o dash em COMPRIMENTO — e o recorte devolveria metade da
/// espessura com o dobro do passo. Halvar os multiplicadores mantém o `dash_lengths()` idêntico,
/// que é a grandeza que o olho vê.
fn doubled(s: &StrokeSpec) -> StrokeSpec {
    StrokeSpec {
        width: s.width * 2.0,
        dash: s.dash.map(|p| p.scaled(0.5)),
        ..s.clone()
    }
}

#[cfg(test)]
#[path = "align_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "align_dash_tests.rs"]
mod dash_tests;
