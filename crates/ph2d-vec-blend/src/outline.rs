//! **O CONTORNO como cadeia de cúbicas, com o comprimento de arco acumulado** — o primitivo
//! compartilhado do motor: a correspondência ([`crate::matching`]) mede sobre ele, o compound
//! ([`crate::compound`]) tem um por contorno, e é ele que sabe cortar.
//!
//! Módulo irmão pelo teto de LOC.

use crate::{ARCLEN_EPS, MERGE_EPS, pt};
use kurbo::{CubicBez, ParamCurve, ParamCurveArclen, Point};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Um segmento do contorno, **com a parametrização normalizada**.
///
/// # A reta que entortava (o smoke do Enio: *"as intermediárias não representam a transição"*)
///
/// No documento, uma aresta RETA é a cúbica **degenerada** `(P0, P0, P3, P3)` — os handles
/// colapsados na âncora. Ela é geometricamente reta, mas a **parametrização dela não é uniforme**:
/// o ponto anda devagar, rápido, devagar. Duas retas cortadas em posições de arco diferentes
/// devolvem sub-cúbicas cujos pontos de controle caem em **frações diferentes** de cada aresta — e
/// o lerp de frações desalinhadas **tira o controle de cima da corda**. A aresta reta sai
/// **ondulada**.
///
/// Medido no quadrado→estrela do smoke: até **0,24 unidade** de desvio numa forma de tamanho 2 —
/// **12% da forma**. As duas pontas são polígonos, então todo intermediário TEM de ser um
/// polígono; o desvio devia ser zero.
///
/// O conserto não é "endireitar depois": é **não ter parametrização torta**. Uma reta entra aqui
/// na forma canônica (controles a ⅓ e ⅔ da corda), que é **afim em `t`** — e uma sub-cúbica de uma
/// curva afim é afim, com os controles nos mesmos ⅓ e ⅔ da sub-corda. Aí o lerp de duas retas é
/// uma reta, **por construção**, cortadas onde forem.
fn segment(a: &VecVertex, b: &VecVertex) -> CubicBez {
    let (p0, p3) = (pt(a.anchor), pt(b.anchor));
    if is_line(a, b) {
        let d = p3 - p0;
        return CubicBez::new(p0, p0 + d / 3.0, p0 + d * (2.0 / 3.0), p3);
    }
    CubicBez::new(p0, pt(a.out_handle), pt(b.in_handle), p3)
}

/// A aresta `a → b` é RETA? (os dois handles colapsados nas âncoras — a convenção do documento,
/// a mesma que a booleana usa.)
fn is_line(a: &VecVertex, b: &VecVertex) -> bool {
    close(a.out_handle, a.anchor) && close(b.in_handle, b.anchor)
}

pub(crate) fn close(p: [f64; 2], q: [f64; 2]) -> bool {
    (p[0] - q[0]).abs() <= COINCIDENT_EPS && (p[1] - q[1]).abs() <= COINCIDENT_EPS
}

/// Handle a esta distância da âncora conta como colapsado.
pub(crate) const COINCIDENT_EPS: f64 = 1e-9;

/// O contorno de uma forma como cadeia de cúbicas, com o comprimento de arco acumulado.
pub(crate) struct Outline {
    pub(crate) segs: Vec<CubicBez>,
    /// `cum[i]` = arco até o INÍCIO de `segs[i]`; `cum[n]` = o total.
    pub(crate) cum: Vec<f64>,
    pub(crate) total: f64,
    pub(crate) closed: bool,
}

impl Outline {
    /// O contorno **PRIMÁRIO** da geometria **COZIDA**.
    ///
    /// **Só os gates.** A produção nunca pergunta pelo primário sozinho — ela pede os anéis
    /// ([`crate::compound::rings`]), porque uma forma pode ter buraco, e foi exatamente por
    /// existir uma porta que devolvia "o contorno" no singular que a rosquinha virava disco.
    /// Os gates a mantêm porque quase todos medem formas de contorno único, onde ela É a forma.
    #[cfg(test)]
    pub(crate) fn of(path: &VecPath) -> Option<Outline> {
        Outline::of_contour(&path.cooked(), 0)
    }

    /// O contorno `c` de uma geometria **JÁ COZIDA** (`0` = o primário).
    ///
    /// Recebe a cozida em vez de cozer aqui porque o chamador percorre TODOS os contornos: cozer
    /// por contorno cozeria a forma inteira uma vez por contorno.
    pub(crate) fn of_contour(cooked: &VecPath, c: usize) -> Option<Outline> {
        let (verts, closed) = cooked.contour(c)?;
        if verts.len() < 2 {
            return None;
        }
        let n = verts.len();
        let last = if closed { n } else { n - 1 };
        let mut segs = Vec::with_capacity(last);
        for i in 0..last {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            segs.push(segment(a, b));
        }
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        if total <= MERGE_EPS {
            return None; // forma degenerada: não há o que percorrer
        }
        Some(Outline {
            segs,
            cum,
            total,
            closed,
        })
    }

    /// As posições (arco normalizado, em [0,1)) das âncoras ORIGINAIS.
    pub(crate) fn anchors(&self) -> Vec<f64> {
        self.cum[..self.segs.len()]
            .iter()
            .map(|c| c / self.total)
            .collect()
    }

    /// O ponto no arco normalizado `s`.
    pub(crate) fn at(&self, s: f64) -> Point {
        let (i, t) = self.locate(s);
        self.segs[i].eval(t)
    }

    /// `n` pontos em posições de arco **igualmente espaçadas**, começando na origem do contorno.
    ///
    /// É a grade comum que torna a busca de fase uma **correlação circular**: amostradas assim, as
    /// duas formas comparam-se por um simples deslocamento de índice (`matching::phase_only`), e a
    /// fase deixa de estar presa às âncoras — que, num contorno suave, não significam nada.
    pub(crate) fn samples(&self, n: usize) -> Vec<Point> {
        (0..n).map(|k| self.at(k as f64 / n as f64)).collect()
    }

    /// (índice do segmento, `t` local) do arco normalizado `s`.
    pub(crate) fn locate(&self, s: f64) -> (usize, f64) {
        let arc = s.clamp(0.0, 1.0) * self.total;
        // O último segmento absorve o fim exato (`s == 1`).
        let i = match self.cum[1..].partition_point(|&c| c <= arc) {
            i if i >= self.segs.len() => self.segs.len() - 1,
            i => i,
        };
        let local = (arc - self.cum[i]).max(0.0);
        let seg = &self.segs[i];
        // O comprimento já está no acumulado — recalculá-lo aqui seria um `arclen` por consulta.
        let len = self.cum[i + 1] - self.cum[i];
        let t = if len <= MERGE_EPS {
            0.0
        } else {
            seg.inv_arclen(local.min(len), ARCLEN_EPS)
        };
        (i, t.clamp(0.0, 1.0))
    }

    /// A cadeia de cúbicas que sai de cortar este contorno nas posições `cuts`.
    ///
    /// **O percurso é CÍCLICO, e isso não é detalhe.** As posições de B saem de `wrap(s − fase)`,
    /// então elas são uma **rotação** da ordem crescente, não a ordem crescente: exatamente uma
    /// peça atravessa a origem do contorno. (Nunca *mais* de uma, e nunca uma âncora: a origem de
    /// B **é** uma âncora de B, e toda âncora de B está em `cuts` — é o que garante que cada peça
    /// caiba num único segmento e seja uma sub-cúbica EXATA.)
    ///
    /// Devolve **uma peça por corte** — sempre. É o que faz A e B saírem pareados 1-a-1.
    ///
    /// # O segmento se decide pelo MEIO da peça, nunca pela borda dela
    ///
    /// **Toda** borda de peça é uma âncora — é assim que este motor foi desenhado (a união inclui
    /// as âncoras das duas formas). E uma âncora é exatamente a fronteira entre dois segmentos:
    /// perguntar "de quem é este ponto?" ali é perguntar de que lado de um empate o `f64` caiu.
    ///
    /// A 1ª versão perguntava pela borda, e o smoke do Enio mostrou o preço: quando o corte era
    /// atribuído ao segmento **anterior** (`t≈1`) em vez do seguinte (`t≈0`), a peça colapsava num
    /// **ponto** — e a aresta correspondente **sumia do pareamento**. Medido no quadrado→estrela:
    /// **3 das 10 arestas da estrela viravam pontos**. As intermediárias deixavam de representar a
    /// transição, e uma delas chegou a explodir na tela.
    ///
    /// O **meio** da peça não é fronteira de nada: ele cai dentro de um único segmento, sempre.
    /// # A ORIGEM tem de ESTAR na lista — e quem depende disso é quem garante
    ///
    /// O parágrafo acima ("nenhuma peça atravessa a origem") não é uma observação: é um
    /// **invariante**, e ele vale porque a origem do contorno **é** uma âncora, e toda âncora está
    /// nos cortes. Quando ele falha, esta função **trunca** a peça que atravessaria a origem em
    /// `1.0` — e o arco entre a origem e o corte seguinte fica **sem peça nenhuma**: uma aresta
    /// inteira some do pareamento, em silêncio.
    ///
    /// E ele FALHAVA. Os cortes de B são as **imagens** dos cortes de A, e a imagem que devia cair
    /// exatamente na origem volta do ida-e-volta `map_backward` → `map_forward` como
    /// **`1 − 3,7e-14`** (medido, círculo → heptágono): o `f64` não fecha o ciclo. Aí a peça
    /// `[1−ε, 1]` é um ponto, e a aresta que começava na origem desaparece.
    ///
    /// O invariante agora é **estabelecido aqui**, e não meramente esperado do chamador: um corte a
    /// menos de `MERGE_EPS` da volta completa **é** a origem. É a mesma lição que já custou uma
    /// reprovação neste motor — *nunca deixar um `f64` decidir de que lado de um empate ele caiu* —,
    /// só que na fronteira que fecha o ciclo.
    pub(crate) fn cut(&self, cuts: &[f64]) -> Vec<CubicBez> {
        let m = cuts.len();
        let mut out = Vec::with_capacity(m);
        let at_origin = |s: f64| if s >= 1.0 - MERGE_EPS { 0.0 } else { s };
        for k in 0..m {
            let s0 = at_origin(cuts[k]);
            // O corte seguinte, ciclicamente. Se ele "voltou" (é a peça que fecha o contorno), o
            // fim dela é o fim do percurso.
            let next = at_origin(cuts[(k + 1) % m]);
            let s1 = if next <= s0 + MERGE_EPS { 1.0 } else { next };
            let i = self.segment_at(0.5 * (s0 + s1));
            let (t0, t1) = (self.local_t(i, s0), self.local_t(i, s1));
            out.push(self.segs[i].subsegment(t0.min(t1)..t1.max(t0)));
        }
        out
    }

    /// O segmento que contém o arco normalizado `s`. `s` tem de cair **dentro** de um segmento —
    /// os chamadores passam o MEIO de uma peça, nunca uma borda.
    pub(crate) fn segment_at(&self, s: f64) -> usize {
        let arc = s.clamp(0.0, 1.0) * self.total;
        self.cum[1..]
            .partition_point(|&c| c <= arc)
            .min(self.segs.len() - 1)
    }

    /// O `t` local do arco normalizado `s` **dentro do segmento `i`** (fora dele, satura em 0/1 —
    /// que é o certo: a borda da peça é a borda do segmento).
    pub(crate) fn local_t(&self, i: usize, s: f64) -> f64 {
        let arc = s.clamp(0.0, 1.0) * self.total;
        let len = self.cum[i + 1] - self.cum[i];
        if len <= MERGE_EPS {
            return 0.0;
        }
        let local = (arc - self.cum[i]).clamp(0.0, len);
        self.segs[i].inv_arclen(local, ARCLEN_EPS).clamp(0.0, 1.0)
    }

    /// O contorno percorrido ao CONTRÁRIO (a saída do "forma vira do avesso").
    pub(crate) fn reversed(&self) -> Outline {
        let segs: Vec<CubicBez> = self
            .segs
            .iter()
            .rev()
            .map(|s| CubicBez::new(s.p3, s.p2, s.p1, s.p0))
            .collect();
        let mut cum = Vec::with_capacity(segs.len() + 1);
        let mut total = 0.0;
        for s in &segs {
            cum.push(total);
            total += s.arclen(ARCLEN_EPS);
        }
        cum.push(total);
        Outline {
            segs,
            cum,
            total,
            closed: self.closed,
        }
    }
}
