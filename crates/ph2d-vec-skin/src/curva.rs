//! ⭐⭐⭐ **O QUE É UMA CURVA AQUI** — o irmão do *«o que é um ponto aqui»*, e a 2.ª saída da F26.
//!
//! # ⛔⛔⛔ O defeito que este módulo existe para curar, escrito por outra crate há meses
//!
//! O cabeçalho da [`ph2d_vec_envelope`] di-lo em duas linhas:
//!
//! > *«Só transformações **afins** comutam com a avaliação de Bézier … logo isto está errado:
//! > `for v in verts { v.anchor = warp(v.anchor) }` … A curva resultante **não é a imagem** da curva
//! > original. E erra de um jeito traiçoeiro: sob um mapa suave ela acerta a verdade em `t=0` e
//! > `t=1` **exatamente**, e no interior **nunca**.»*
//!
//! **A pele é um mapa não-afim** (os pesos variam com o ponto) e o [`crate::aplica`] deforma os
//! pontos de controlo — ⇒ ele é, à letra, o `for v in verts` daquele aviso. As consequências foram
//! as duas queixas do dono desta jornada:
//!
//! 1. *«pintar peso ENTRE os vértices não faz nada»* — porque só os NÓS são amostrados. **Medido:**
//!    uma mancha no meio de uma aresta move a arte `0,000000` pelo caminho dos pontos de controlo e
//!    `0,242375` por este.
//! 2. *«o ponto criado deforma a malha»* (F28) — o corte revelava o erro de interior que já lá
//!    estava. A F28 curou-o por compensação; aqui ele **deixa de existir**.
//!
//! # ⭐⭐ A lei: o peso varia ao longo do `t`, e nos NÓS ela é a de hoje
//!
//! Num segmento de `a` para `b`, o peso do ponto `C(t)` é a **mistura** das linhas dos dois nós,
//! `lerp(ra, rb, t)`, com as manchas pintadas somadas **no ponto** — a mesma lei que a F28 já usa
//! para dar peso a um nó novo. ⇒ em `t = 0` e `t = 1` ela é **exactamente** a de hoje, logo os nós
//! não se mexem e a paridade do repouso fica intacta.
//!
//! ⛔⛔ **É por isso que a pele NÃO é um [`ph2d_vec_envelope::Warp`]:** aquele contrato é por
//! POSIÇÃO, e a tabela do padrão-ouro é guardada **por ponto de controlo** — ela não tem forma
//! contínua no espaço. *Pedir um campo posicional obrigaria a guardar a malha do domínio no bind*
//! (degrau de schema, e peso no ficheiro) para responder à mesma pergunta que o `t` responde de
//! graça.
//!
//! # ⚠️ A derivada é por DIFERENÇA FINITA em `t`, e a razão está medida
//!
//! O fitter precisa de `(ponto, derivada)`, e a doc do [`ph2d_vec_envelope::Warp`] argumenta — com
//! razão — que uma derivada inconsistente faz o `fit_to_bezpath` **não convergir**. ⛔ Aqui a forma
//! fechada existiria mas atravessaria o `quota` de um osso que dobra e o bump de cada mancha, e
//! **cada um deles é um ramo** — a lei ficaria escrita duas vezes, e a segunda envelhece.
//!
//! ⇒ a derivada é a diferença central da **própria** função de amostragem, em `t` (um parâmetro em
//! `[0,1]`, com `h = 1e-6` ⇒ erro `~1e-12` relativo). *Ela é consistente com o `map` por
//! construção, que é o que o contrato de facto exige.* A sonda mediu que o fit converge.

use kurbo::{CubicBez, ParamCurve, Point, Vec2};
use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::{VecPath, VecVertex};


/// ⭐⭐⭐ **A LEI DA CURVA ESTÁ LIGADA?** — a porta única da F30, e ela tem **dois** leitores que não
/// se conhecem: o `recook` do quadro e o gesto que acrescenta um ponto
/// ([`ph2d_skeleton_live::ponto_novo`]).
///
/// ⛔⛔ **O segundo leitor não é zelo — é uma lei que se INVERTE.** A F28 compensa o ponto novo para
/// o desenho não saltar, e essa compensação é feita contra a curva que a lei **ingénua** desenha.
/// Com a lei da curva ligada o desenho é a imagem verdadeira da fonte, logo partir a fonte **não o
/// move** (medido: `0,0000 %`) e compensar passa a **estragar** (medido: `11,11 %` da peça).
/// *Uma cura fica errada no dia em que o defeito que ela curava deixa de existir.*
///
/// ⚠️ `PH2D_SKIN_CURVE=0` volta ao caminho dos pontos de controlo, e lá a compensação volta a ser
/// obrigatória.
///
/// ⛔⛔ **Ela é lida no SÍTIO DE CHAMADA e a lei viaja como PARÂMETRO daí para baixo** — nunca num
/// estado global. A 1.ª redacção pôs um átomo com uma porta `forcar_lei` para os gates medirem o
/// outro lado, e o doc dela dizia *«o nextest corre um processo por teste»*: verdade para o
/// `nextest`, **falsa** para o `cargo test`, que corre os testes em THREADS do mesmo processo — o
/// guarda de um vazava para outro e a suíte reprovava em conjunto e passava sozinha. *Um estado
/// global posto para o teste é um canal entre testes.*
#[must_use]
pub fn lei_da_curva_activa() -> bool {
    std::env::var("PH2D_SKIN_CURVE").as_deref() != Ok("0")
}

/// Um segmento da arte visto **através** da pele — a curva paramétrica `t ↦ blend(C(t), w(t))`.
///
/// Ela nunca é materializada: o fitter amostra-a.
struct SegmentoDaPele<'a> {
    src: CubicBez,
    pele: &'a Skin,
    /// A linha de pesos do nó de PARTIDA, ou `None` para a lei derivada.
    ra: Option<&'a [f64]>,
    /// A linha de pesos do nó de CHEGADA.
    rb: Option<&'a [f64]>,
    correcoes: &'a [Correccao],
    /// A mistura: rígida (o produto) ou linear (o controlo dos gates).
    rigido: bool,
}

impl SegmentoDaPele<'_> {
    /// A imagem de `C(t)`.
    fn ponto(&self, t: f64) -> Point {
        let c = self.src.eval(t);
        let p = [c.x, c.y];
        let mut w = self.pele.scratch();
        match (self.ra, self.rb) {
            // ⭐ A MISTURA das duas linhas no mesmo `t` — a lei da F28 para o nó novo, aplicada
            // agora a **todo** ponto da curva. Em `t = 0` e `t = 1` ela é a linha do nó, ao bit.
            (Some(a), Some(b)) => {
                let mistura: Vec<f64> = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| (y - x).mul_add(t, *x))
                    .collect();
                self.pele
                    .weights_corrected(p, Some(&mistura), &mut w, self.correcoes);
            }
            // ⛔ Sem tabela a lei é a DERIVADA, e ela já é função da posição — não há o que misturar.
            _ => self.pele.weights_corrected(p, None, &mut w, self.correcoes),
        }
        let q = if self.rigido {
            self.pele.blend(p, &w)
        } else {
            self.pele.blend_linear(p, &w)
        };
        Point::new(q[0], q[1])
    }

}


/// ⭐⭐⭐ **A ARTE SEGUE O PESO ENTRE OS NÓS — corrigindo as ALÇAS, e sem limiar nenhum.**
///
/// A lei de hoje ([`crate::aplica_corrigido`]) corre **sempre e primeiro**: ela acerta nos NÓS por
/// construção (ali o peso é o do próprio ponto) e erra no INTERIOR de cada segmento, porque a pele
/// é um mapa **não-afim**. O que falta é exactamente o que as duas alças de uma cúbica governam ⇒
/// esta função ajusta-as, por mínimos quadrados, contra a curva verdadeira.
///
/// # ⛔⛔⛔ Porque ela substituiu o REFIT (report do dono, 2026-09-19, com duas fotos)
///
/// *«Em determinado momento da deformação as alças sofrem uma mudança e o path muda repentinamente,
/// como se o handle mudasse de tipo.»*
///
/// A redacção anterior perguntava *«o desvio passa da tolerância?»* e, se sim, **refazia o contorno
/// inteiro** com a `kurbo::fit_to_bezpath`. Um booleano sobre uma grandeza contínua é um **degrau**,
/// e medido numa dobra a passos de `0,01 rad` ele não dá um salto: dá **CHATTER** — a decisão
/// oscila entre quadros vizinhos a partir de `1,44 rad`, e cada oscilação troca a representação de
/// **todos** os contornos (nós, alças e contagem). *O artista arrasta a âncora e a forma pisca.*
///
/// ⭐⭐ **A lei nova não tem decisão nenhuma para tomar**, e é isso que a torna contínua:
///
/// - **a correcção é da DIFERENÇA, e não da curva** — o que se ajusta é `verdade(t) − ingénuo(t)`.
///   Onde o mapa é afim sobre o segmento (em repouso, e em toda aresta cujos dois nós têm o mesmo
///   peso e que nenhuma mancha toca) essa diferença é **exactamente zero**, o segundo membro do
///   sistema é zero e as alças ficam **byte-idênticas**. ⇒ o defeito que obrigou o limiar a existir
///   — `binding_a_shape_moves_nothing` a acusar `40/3` em repouso, a elevação `(⅓, ⅔)` de uma recta
///   — **não pode acontecer aqui**;
/// - **os NÓS não se mexem**: eles já estão certos, e são as extremidades fixas do ajuste;
/// - **o `kind` e o `corner_radius` sobrevivem**, porque nenhum vértice nasce nem morre;
/// - e o resultado é **linear** na diferença amostrada, logo contínuo na pose.
///
/// ⚠️ O parâmetro `tolerancia` **saiu**: não há o que tolerar quando não há decisão. Quem governa a
/// fidelidade é [`AMOSTRAS`].
pub fn aplica_pela_curva(pele: &Skin, path: &mut VecPath, pesos: &[f64], correcoes: &[Correccao]) {
    aplica_pela_curva_com(pele, path, pesos, correcoes, true);
}

/// **A lei da curva com a mistura como PARÂMETRO** — ver [`crate::aplica_corrigido_com`].
pub fn aplica_pela_curva_com(
    pele: &Skin,
    path: &mut VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
) {
    let fonte = path.clone();
    crate::aplica_corrigido_com(pele, path, pesos, correcoes, rigido);
    let ossos = if pesos.is_empty() {
        0
    } else {
        pesos.len() / (fonte.verts_all().count() * 3).max(1)
    };
    let mut base = 0usize;
    for c in 0..fonte.contour_count() {
        let Some((verts, fechado)) = fonte.contour(c) else {
            continue;
        };
        let n = verts.len();
        let segs = if fechado { n } else { n.saturating_sub(1) };
        for k in 0..segs {
            let s = SegmentoDaPele {
                src: cubica(verts, k, n),
                pele,
                ra: linha(pesos, ossos, base + k),
                rb: linha(pesos, ossos, base + (k + 1) % n),
                correcoes,
                rigido,
            };
            let Some((alvo, _)) = path.contour_mut(c) else {
                continue;
            };
            if alvo.len() != n {
                continue;
            }
            // ⚠️ A cúbica INGÉNUA lê-se **antes** de as alças serem escritas — e as duas que este
            // segmento escreve são exactamente as duas que ele lê. *Cada alça pertence a um
            // segmento só, logo não há ordem que as faça interferir.*
            let ja = cubica(alvo, k, n);
            let (d1, d2) = correccao_das_alcas(&s, &ja);
            let j = (k + 1) % n;
            alvo[k].out_handle = [alvo[k].out_handle[0] + d1.x, alvo[k].out_handle[1] + d1.y];
            alvo[j].in_handle = [alvo[j].in_handle[0] + d2.x, alvo[j].in_handle[1] + d2.y];
        }
        base += n;
    }
}

/// Quantas amostras interiores por segmento alimentam o ajuste das alças.
///
/// ⚠️ **Duas bastariam para fechar o sistema** (são duas incógnitas); mais amostras repartem o erro
/// em vez de o zerar em dois pontos e deixar a curva fugir entre eles.
///
/// ⛔ **A REGRA DO PONTO MÉDIO — `(i + ½)/N` — é uma escolha, não uma lei**, e está medido: a
/// mutação que a troca por `i/N` **sobrevive**, porque a amostra em `t = 0` tem `B₁ = B₂ = 0` e não
/// entra no sistema. *Fica escrito para ninguém procurar o gate que a defende.*
pub const AMOSTRAS: usize = 8;

/// A cerca da [`AMOSTRAS`], em TEMPO DE COMPILAÇÃO e ao lado do que guarda.
///
/// ⚠️ São **duas** incógnitas (as duas alças): com menos de duas amostras o sistema não fecha, e o
/// determinante seria zero. ⭐ Como é uma const, quem a editar para um valor mudo **não compila** —
/// um `assert!` de teste sobre uma constante é dobrado pelo compilador antes de correr, e o clippy
/// di-lo em voz alta.
const _: () = assert!(AMOSTRAS >= 2);

/// ⭐⭐⭐ **O AJUSTE DAS DUAS ALÇAS** — mínimos quadrados com as pontas PRESAS.
///
/// Uma cúbica é **linear nos pontos de controlo**: `C(t) = B₀P₀ + B₁P₁ + B₂P₂ + B₃P₃`. Com `P₀` e
/// `P₃` fixos (os nós, que já estão certos), mover só as alças dá `ΔC(t) = B₁ΔP₁ + B₂ΔP₂`, e o
/// `ΔP` que melhor segue a diferença medida sai de um sistema `2×2` cuja matriz **só depende dos
/// `t`** — logo é constante, e a solução é **linear** na diferença. *É daí que vem a continuidade.*
///
fn correccao_das_alcas(s: &SegmentoDaPele<'_>, ja: &CubicBez) -> (Vec2, Vec2) {
    let (mut a11, mut a12, mut a22) = (0.0_f64, 0.0_f64, 0.0_f64);
    let (mut b1, mut b2) = (Vec2::ZERO, Vec2::ZERO);
    for i in 0..AMOSTRAS {
        #[expect(clippy::cast_precision_loss, reason = "i < AMOSTRAS, um punhado")]
        let t = (i as f64 + 0.5) / AMOSTRAS as f64;
        let u = 1.0 - t;
        let (w1, w2) = (3.0 * u * u * t, 3.0 * u * t * t);
        let d = s.ponto(t) - ja.eval(t);
        a11 = w1.mul_add(w1, a11);
        a12 = w1.mul_add(w2, a12);
        a22 = w2.mul_add(w2, a22);
        b1 += d * w1;
        b2 += d * w2;
    }
    // ⛔⛔ **O determinante NÃO precisa de guarda, e isso foi medido:** a matriz depende **só dos
    // `t`**, logo ela é a MESMA em todo segmento de toda forma — uma constante. A 1.ª redacção
    // tinha um `if det.abs() < 1e-12 { return None }` e a mutação que o apagava **sobreviveu**,
    // porque aquele ramo é inalcançável. *Uma linha que a mutação não consegue matar não é lei, é
    // comentário com sintaxe de código.*
    let det = a12.mul_add(-a12, a11 * a22);
    let (d1, d2) = (
        (b1 * a22 - b2 * a12) / det,
        (b2 * a11 - b1 * a12) / det,
    );
    // ⛔⛔ **E não há cerca de `NaN` aqui, também por medição.** A tentação é guardar contra uma
    // diferença não-finita — mas tudo o que chegasse assim já teria passado pela
    // [`crate::aplica_corrigido`], que corre **antes** e escreve o `NaN` no desenho sem nos
    // perguntar nada: *uma cerca a jusante do sítio onde o estrago acontece protege o quê?* Medido:
    // com uma mancha de centro `NaN` a forma desaparece **com ou sem** a cerca, e a mutação que a
    // apagava sobrevivia. ⇒ ela sai, e quem a quiser tem de a pôr onde o `NaN` entra.
    (d1, d2)
}

/// A linha de pesos do nó `k` (índice PLANO, na tabela guardada), ou `None` quando não há tabela.
fn linha(pesos: &[f64], ossos: usize, k: usize) -> Option<&[f64]> {
    (ossos > 0).then(|| pesos.get(k * 3 * ossos..k * 3 * ossos + ossos))?
}

/// O segmento `k` como cúbica do kurbo. Fechado: o último liga de volta ao primeiro.
fn cubica(verts: &[VecVertex], k: usize, n: usize) -> CubicBez {
    let (a, b) = (&verts[k], &verts[(k + 1) % n]);
    CubicBez::new(
        Point::new(a.anchor[0], a.anchor[1]),
        Point::new(a.out_handle[0], a.out_handle[1]),
        Point::new(b.in_handle[0], b.in_handle[1]),
        Point::new(b.anchor[0], b.anchor[1]),
    )
}

#[cfg(test)]
#[path = "curva_tests.rs"]
mod tests;
