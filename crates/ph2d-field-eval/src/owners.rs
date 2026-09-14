//! ⭐⭐⭐ **DE QUEM É ESTE PONTO** — a lei que responde *«que folha o artista apontou?»*, num sítio só.
//!
//! # Como se pergunta a um CAMPO quem ele é
//!
//! Uma malha traz a resposta consigo (cada triângulo sabe de que objecto é). Um campo implícito não:
//! o que se avalia é **um número**, e o número da peça já é a união de todos os nós. ⇒ pergunta-se a
//! cada folha o valor do campo **dela** ali, e ganha a de **menor módulo**. Numa superfície de união
//! o vencedor vale `~0` e os outros valem a distância a que estão, então a resposta não é apertada:
//! ela é a diferença entre tocar e não tocar.
//!
//! # ⛔⛔ E porque isto deixou de ser um detalhe da SELECÇÃO
//!
//! A lei viveu dentro do `ph2d-app-field3d::pick`, porque a pergunta só se fazia **num clique**. O
//! **material por objecto** faz dela uma pergunta **por pixel** (`docs/Render3d/05` §8), e nesse dia
//! ela passou a ter dois leitores — o clique e o sombreamento. *Uma lei escrita em dois sítios ainda
//! não é uma lei: só uma PORTA é.*
//!
//! ⚠️ **A recusa do *id-buffer* continua de pé, e esta porta não a viola:** ela não arrasta um
//! segundo canal por cada passo da marcha — resolve o dono **depois**, uma vez por ponto.
//!
//! # ⭐⭐ A BOLA à frente, e a margem que ela obriga
//!
//! Perguntar a todas as folhas custa `O(folhas)` por ponto, e medido a `640×360` com `16` folhas
//! isso é `507 ns/px` — um quadro inteiro. A cura é a bola que a casa **já deriva**
//! ([`crate::bounds::bounding_ball`]): numa união o dono vale `~0`, logo o ponto está **sobre** a
//! superfície da própria folha, logo **dentro** da bola dela. Quem não a contém não pode ganhar.
//! Medido: `13,3 ms → 1,6 ms` com a **mesma** resposta, e `1,0` folha visitada por pixel.
//!
//! ⚠️⚠️ **A MARGEM não é folga, é obrigatória.** A marcha pára quando o campo desce abaixo de uma
//! tolerância — isto é, **ligeiramente fora** da superfície. Um filtro exacto (`d² ≤ r²`) rejeita a
//! resposta certa **em todo o lado**: medido, `26 216` de `26 216` pixels caíam na rede.
//!
//! ⚠️ **E a rede tem de existir:** uma mistura suave **empurra** a superfície para fora das bolas das
//! folhas (o [`crate::bounds`] escreve-o), e ali nenhuma bola contém o ponto. Quando nenhuma
//! qualifica, pergunta-se a **todas** — a resposta nunca muda, só o preço.

use crate::{Field, bounds::Ball};
use ph2d_field::FieldDoc;

/// **As folhas de uma peça, prontas a responder de quem é um ponto.**
///
/// ⚠️ **Quem as coloca é o CHAMADOR**, e não esta struct: numa árvore com grupos a pose de uma folha
/// é a composição das dos antepassados, e quem a sabe é quem tem a hierarquia
/// (`ph2d_field_ecs::world_xform`). Uma folha avaliada com a pose **local** responderia sobre um
/// sítio onde ela não está — e o erro **cresce com o aninhamento**, logo passaria despercebido numa
/// peça plana e escolheria o objecto errado numa peça agrupada.
pub struct Owners {
    leaves: Vec<(Ball, Field)>,
    margin: f32,
}

impl Owners {
    /// Compila **uma fita por folha**, a partir de documentos de um nó já **posto no mundo**.
    ///
    /// ⚠️ **A compilação é um JIT e mora AQUI, fora de qualquer laço.** Uma versão que compilasse por
    /// ponto não seria lenta, seria impossível: `300` pixels de um laço de selecção numa peça de `5`
    /// folhas custariam `1 800` compilações.
    ///
    /// `margin` é a tolerância com que o ponto foi produzido — ver a nota do módulo. Quem marcha
    /// sabe-a; esta porta não a pode adivinhar.
    #[must_use]
    pub fn new(placed: &[FieldDoc], reg: &crate::hybrid::Registry, margin: f32) -> Self {
        Self {
            leaves: placed
                .iter()
                .map(|d| {
                    (
                        crate::bounds::bounding_ball(d, reg).unwrap_or(Ball::EMPTY),
                        Field::new(d),
                    )
                })
                .collect(),
            margin,
        }
    }

    /// Quantas folhas esta peça tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Não há folha nenhuma?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// ⭐⭐⭐ **A folha a que este ponto pertence** — `None` só quando não há folhas.
    ///
    /// Ver a nota do módulo: a bola à frente, a margem, e a rede quando nenhuma bola o contém.
    #[must_use]
    pub fn at(&self, p: [f32; 3]) -> Option<usize> {
        self.at_counting(p).0
    }

    /// ⭐⭐⭐ **AS DUAS FOLHAS QUE DISPUTAM ESTE PONTO, e quanto** — `(dono, rival, t)`.
    ///
    /// `t` é o peso do **rival**, em `0..=0,5`: `0` quer dizer *«o dono ganha de longe»* e `0,5`
    /// *«estão empatadas»*. O `rival` é igual ao `dono` quando há uma folha só.
    ///
    /// # ⛔⛔ Porque ela existe: a fronteira de COR não é uma silhueta
    ///
    /// O anti-serrilhado do traçado corre nas **bordas** — pixels em que umas sub-amostras acertam a
    /// peça e outras não. Uma fronteira de **cor** no meio da peça não é nenhuma dessas: ali **todas**
    /// acertam, não há registo de borda nenhum, e a cor muda de um pixel para o outro **a pique**.
    /// Medido (`docs/Render3d/05` §19): um degrau de até **`236` bytes** numa banda de `1`–`2` px,
    /// ao lado de uma silhueta suavizada. *A peça fica com o contorno liso e uma escada por dentro.*
    ///
    /// # ⭐ A largura sai da GEOMETRIA, e não de um número escolhido
    ///
    /// A fronteira é onde `|f_dono| = |f_rival|`. Andando `s` perpendicular a ela, uma distância
    /// cresce `~s` e a outra decresce `~s` ⇒ a diferença `d = |f_rival| − |f_dono|` varia **`2 s`**.
    /// Logo `s = d/2`, e para a transição medir **um pixel** basta dividir por `width`:
    ///
    /// ```text
    /// t = máx(0, ½ − d / (2·width))
    /// ```
    ///
    /// ⚠️ `width` é o **tamanho de um pixel em mundo**, que quem marcha sabe e esta porta não pode
    /// adivinhar — a mesma lei da `margin` no [`Owners::new`].
    ///
    /// ⛔ **Isto NÃO substitui o [`Owners::at`]:** um clique quer **uma** folha, e meia folha não é
    /// uma resposta. As duas perguntas são diferentes e têm duas portas.
    #[must_use]
    pub fn mix_at(&self, p: [f32; 3], width: f32) -> (usize, usize, f32) {
        let mut best: Option<(f32, usize)> = None;
        let mut second: Option<(f32, usize)> = None;
        // ⛔⛔ **A MARGEM DA MISTURA É UM PIXEL, e não a da marcha** — foi isto que fez a 1.ª
        // versão desta porta não mexer numa união DURA. A bola à frente filtra quem não pode
        // ganhar, e a margem dela (`self.margin`) é a tolerância com que o **ponto** foi produzido:
        // uns `2e-4`. ⚠️ Mas a pergunta aqui não é *«quem pode GANHAR?»* — é *«quem pode estar a
        // menos de um PIXEL de ganhar?»*, e um pixel mede `4,4e-3`, **vinte vezes mais**.
        //
        // Medido: no pior pixel de uma união dura o rival vinha filtrado e a mistura lia
        // `(0, 0, 0.0)` — *o segundo melhor era o próprio dono*, que é como esta porta diz «não há
        // rival». A cura é uma linha, e sem ela a lei inteira era muda naquele caso.
        self.visit(p, self.margin.max(width), |i, v| match best {
            Some((bv, _)) if v >= bv => {
                if second.is_none_or(|(sv, _)| v < sv) {
                    second = Some((v, i));
                }
            }
            _ => {
                second = best;
                best = Some((v, i));
            }
        });
        let Some((bv, bi)) = best else {
            return (0, 0, 0.0);
        };
        let Some((sv, si)) = second else {
            return (bi, bi, 0.0);
        };
        // ⚠️ `width <= 0` devolve o degrau a pique, que é o comportamento de antes desta porta — um
        // chamador sem escala de pixel não pode receber uma mistura inventada.
        if width <= 0.0 {
            return (bi, si, 0.0);
        }
        (bi, si, (0.5 - (sv - bv) / (2.0 * width)).max(0.0))
    }

    /// O percurso partilhado por [`Owners::at_counting`] e [`Owners::mix_at`] — a bola à frente, a
    /// rede, e `|f_i(p)|` de quem sobrevive às duas.
    ///
    /// ⚠️ **Uma travessia, dois leitores:** as duas perguntas diferem só no que fazem com os valores,
    /// e escrever o filtro duas vezes seria a segunda cópia de uma lei que já se paga cara uma vez
    /// (a margem, a rede, o `d² ≤ (r+margem)²`).
    fn visit(&self, p: [f32; 3], margem: f32, mut f: impl FnMut(usize, f32)) {
        let dentro = |b: &Ball| {
            let d = [p[0] - b.center[0], p[1] - b.center[1], p[2] - b.center[2]];
            let r = b.radius + margem;
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r * r
        };
        // ⚠️ **A rede é uma pergunta, não um `if` escondido:** com uma mistura suave nenhuma bola
        // contém o ponto, e aí o filtro não filtra — ele responderia `None` sobre uma peça visível.
        let filtra = self.leaves.len() > 1 && self.leaves.iter().any(|(b, _)| dentro(b));
        for (i, (b, campo)) in self.leaves.iter().enumerate() {
            if filtra && !dentro(b) {
                continue;
            }
            f(
                i,
                campo
                    .at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs() as f32,
            );
        }
    }

    /// ⭐⭐ **A MESMA resposta, e QUANTAS folhas ela custou** — o preço, observável.
    ///
    /// ⛔⛔ **Ela existe porque um gate sobre a RESPOSTA é cego ao PREÇO, e a rede esconde-o.**
    /// Medido em 2026-09-13: apagar a margem do filtro deixou os quatro gates desta lei **verdes** —
    /// sem margem nenhuma bola contém o ponto, a rede dispara, e a resposta sai **certa** pelo
    /// caminho caro. *Uma optimização cuja ausência não se vê na saída precisa de um gate sobre o
    /// trabalho, não sobre o resultado.*
    ///
    /// ⚠️ Ela **não** é um `#[cfg(test)]`: quem sombreia um quadro quer poder dizer quantas folhas
    /// pagou, e um número que só existe em teste não descreve o produto.
    #[must_use]
    pub fn at_counting(&self, p: [f32; 3]) -> (Option<usize>, usize) {
        if self.leaves.is_empty() {
            return (None, 0);
        }
        let mut best: Option<(f32, usize)> = None;
        let mut visitadas = 0usize;
        self.visit(p, self.margin, |i, v| {
            visitadas += 1;
            if best.is_none_or(|(melhor, _)| v < melhor) {
                best = Some((v, i));
            }
        });
        (best.map(|(_, i)| i), visitadas)
    }
}

#[cfg(test)]
#[path = "owners_tests.rs"]
mod tests;
