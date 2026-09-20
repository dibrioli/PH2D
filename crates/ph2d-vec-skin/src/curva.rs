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

/// ⭐⭐⭐ **O CAMPO DO DOMÍNIO ESTÁ LIGADO?** — a porta de bissecção da wave de 2026-09-20.
///
/// Com ela a `0`, a lei da curva volta a misturar as linhas dos dois nós em linha recta, mesmo num
/// bind que guardou o campo. ⚠️ **Ela não apaga o campo do ficheiro** — ele fica lá, e voltar a
/// ligar não re-resolve nada.
///
/// ⚠️ **Lida no SÍTIO DE CHAMADA, uma vez por quadro**, pela mesma razão escrita na irmã acima: um
/// `var_os` por forma seria uma syscall dentro do laço do desenho, e um estado global posto para o
/// teste é um canal entre testes.
///
/// # ⛔⛔⛔ O que ela bissecta, RE-MEDIDO — a tabela que aqui esteve estava ERRADA
///
/// A tabela publicada nesta função até 2026-09-20 dizia **`8,7 %`** na linha do meio, e ela saiu de
/// uma fixtura **DEGENERADA**: a `campo_tests::pele` constrói os dois ossos por
/// [`ph2d_skeleton::SkinBone::new`], que crava `tendon: 0` («o neutro honesto de um osso SOZINHO»),
/// logo os **dois** ossos lêem a **mesma coluna** do campo e `5` dos `20` nós ficam com `w = 0` —
/// eles não se movem, e a diferença que a sonda lia era esse colapso, não a lei.
///
/// Re-medido pela [`super::campo_tests::diag_a_tabela_honesta_das_tres_linhas`], que corre as três
/// linhas com a fixtura corrigida (`pele_com_tendoes`, a única diferença) e mede **a CURVA** e não
/// os pontos de controlo:
///
/// | caso (rectângulo `40 × 10`, dois ossos) | fixtura degenerada | **honesta** | da peça |
/// |---|---:|---:|---:|
/// | como o artista desenha (`4` nós) · dobra `0,8` | `5,268` | `2,431` | `6,08 %` |
/// | **como o bind entrega** (`20` nós) · dobra `0,8` | `5,780` | **`0,128`** | **`0,32 %`** |
/// | como o bind entrega · dobra `1,5` · com peso pintado | `10,756` | `0,210` | `0,53 %` |
///
/// ⇒ **`8,7 %` era honestamente `0,32 %`, vinte e sete vezes menor.**
///
/// ⛔⛔ **E os números antigos não reproduzem em fixtura NENHUMA** (`5,175` contra `5,268`/`2,431`):
/// eles foram escritos de uma corrida que já não existe. *Uma tabela copiada à mão para um
/// doc-comment deixa de ter quem a contradiga; esta é derivada, e a sonda imprime as duas colunas
/// lado a lado exactamente para não voltar a ser.*
///
/// ⚠️⚠️ **A grandeza também estava errada:** os números velhos mediam ALÇAS, e **ligar o campo não
/// move uma única ÂNCORA** — `0,000` nas seis células da sonda. Uma alça que anda `δ` move a curva
/// no máximo `4/9 · δ`, logo medir alças **majora** o que o artista vê; medir âncoras dá zero.
/// *O desenho mede-se no DESENHO.*
///
/// ⚠️ **O erro de PESO é `0,0329`** depois de o bind subdividir (`0,3752` antes) — e a linha do
/// meio diz o resto: *a subdivisão do bind já põe nós onde o campo teria dito o mesmo*, e o que
/// sobra para o campo corrigir vale um terço de um por cento da peça.
#[must_use]
pub fn lei_do_campo_activa() -> bool {
    std::env::var("PH2D_SKIN_CAMPO").as_deref() != Ok("0")
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
    /// ⭐⭐⭐ **O CAMPO DO DOMÍNIO, quando o bind o guardou** — ver [`Self::pesos`].
    campo: Option<&'a crate::pesos::CampoDoDominio>,
}

impl SegmentoDaPele<'_> {
    /// Os pesos do ponto `p`, com a linha do nó misturada em `t`.
    ///
    /// # ⭐⭐⭐ Quando o bind guardou o CAMPO, a mistura não é usada (2026-09-20)
    ///
    /// A mistura `lerp(ra, rb, t)` é uma **recta entre dois nós**, e o campo verdadeiro atravessa
    /// uma junta num **«S»** — medido num rectângulo de `40 × 10`, a recta erra até **`0,3709`**
    /// numa aresta que cruza a junta e **`0,0000`** nas que não cruzam ([`crate::pesos::CampoDoDominio`]
    /// tem a tabela). Com o campo vivo, a linha deste ponto **lê-se do domínio** e o erro vai a zero
    /// por construção: ele é o mesmo padrão-ouro de que a tabela dos nós foi amostrada.
    ///
    /// ⭐⭐ **Nos NÓS as duas leis coincidem AO BIT, e não por sorte:** a linha guardada do nó `k`
    /// *é* este campo amostrado na âncora dele, pela mesma [`crate::pesos::amostra_achatada`] com o
    /// mesmo ponto. ⇒ *ligar o campo não move um nó.*
    ///
    /// ⚠️ **Fora da malha o campo devolve `None` e a mistura VOLTA** — ⛔ nunca zeros. Uma alça vive
    /// fora da forma por definição (ela é uma tangente), e ali a resposta de sempre é a certa.
    ///
    /// ⚠️ **As correcções à mão entram por baixo das duas**, no mesmo sítio: elas são uma mancha no
    /// ESPAÇO e já eram somadas no ponto — o que esta wave muda é a BASE sobre que elas pousam.
    fn pesos(&self, p: [f64; 2], t: f64) -> Vec<f64> {
        let mut w = self.pele.scratch();
        if let Some(linha) = self.campo.and_then(|c| c.linha(p)) {
            self.pele
                .weights_corrected(p, Some(&linha), &mut w, self.correcoes);
            return w;
        }
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
        w
    }

    /// A mistura de `p` por uma linha de pesos **já resolvida**.
    ///
    /// ⚠️ **Ela é AFIM em `p` para `w` fixo** — `R(θ̄)·(p − c) + Σ wᵢMᵢ(c)`, onde `c` e `θ̄` só
    /// dependem de `w` ([`ph2d_skeleton::centro`]). É essa propriedade que a [`Self::direccao`]
    /// usa para extrair a parte LINEAR do afim de um nó por duas avaliações.
    fn mistura(&self, p: [f64; 2], w: &[f64]) -> Point {
        let q = if self.rigido {
            self.pele.blend(p, w)
        } else {
            self.pele.blend_linear(p, w)
        };
        Point::new(q[0], q[1])
    }

    /// A imagem de `C(t)`.
    fn ponto(&self, t: f64) -> Point {
        let c = self.src.eval(t);
        let p = [c.x, c.y];
        let w = self.pesos(p, t);
        self.mistura(p, &w)
    }

    /// ⭐⭐⭐ **A DIRECÇÃO EM QUE A ARTE SAI DO NÓ DE PARTIDA** (`fim = false`) **ou CHEGA AO DE
    /// CHEGADA** (`fim = true`) — o **eixo** daquela alça, e a peça que faz a colinearidade ser
    /// exacta em vez de aproximada.
    ///
    /// Ela é `R_k · û`, onde `û` é a direcção da alça na FONTE e `R_k` é a parte linear do afim
    /// **daquele nó** — a mesma que a lei ingénua aplica às três metades do vértice. ⇒ as duas
    /// alças que se encontram no nó `k` são mapeadas pela **MESMA** `R_k` a partir de duas
    /// direcções que a fonte já tinha colineares ⇒ *continuam colineares, ao bit, e o `kind` do
    /// vértice deixa de ser uma promessa.*
    ///
    /// ⛔⛔ **A 1.ª redacção tirava-a da cúbica JÁ deformada (`ja.p₁ − ja.p₀`) e isso não serve:**
    /// aquele vector mistura DOIS nós (`B(p₁) − A(p₀)` quando a alça é degenerada), logo os dois
    /// extremos de uma aresta recta recebiam a **mesma recta** e o segmento não conseguia arquear.
    /// *O que faz uma recta arquear é exactamente as duas pontas serem rodadas por afins
    /// DIFERENTES.*
    ///
    /// ⛔⛔ **E uma ALÇA DEGENERADA devolve ZERO de propósito — ela não é um caso a remendar.**
    /// Uma alça em cima da âncora (a arte que a caneta desenha sem arrastar) não carrega tangente
    /// nenhuma, e o nó onde ela chega é um **CANTO**: ali não há continuidade para conciliar, e o
    /// ajuste livre é a resposta mais fiel. ⚠️ A 1.ª redacção inventava-lhe uma direcção por
    /// cascata (`C''`, depois a corda) e **nenhum gate conseguia matar essa linha** — nos nós em
    /// que ela era lida, a outra metade do nó tinha comprimento zero e a
    /// [`reconcilia`] saltava-os na mesma. *Uma linha que a mutação não consegue matar não é lei.*
    fn direccao(&self, fim: bool) -> Vec2 {
        let u = if fim {
            self.src.p2 - self.src.p3
        } else {
            self.src.p1 - self.src.p0
        };
        if u.hypot() <= 0.0 {
            return Vec2::ZERO;
        }
        let (t, c) = if fim {
            (1.0, self.src.p3)
        } else {
            (0.0, self.src.p0)
        };
        let p = [c.x, c.y];
        let w = self.pesos(p, t);
        versor(self.mistura([p[0] + u.x, p[1] + u.y], &w) - self.mistura(p, &w))
    }
}

/// O versor de `v`, ou o vector NULO quando `v` não tem direcção.
///
/// ⚠️ **`Vec2::normalize` de um vector nulo devolve `NaN`**, e um `NaN` numa alça apaga a forma.
///
/// ⛔⛔ **A cerca NÃO TEM GATE, e isso está MEDIDO — fica declarada.** Para o `NaN` chegar ao
/// desenho é preciso que a parte linear do afim de um nó seja singular **na direcção daquela
/// alça** *e* que a correcção livre reviva a alça. Nas duas fixturas construídas para o provocar —
/// a pele inteira colapsada, e um eixo colapsado com o tendão posto à mão — a alça colapsa junto e
/// a [`reconcilia`] salta o nó **antes** de olhar para o eixo, logo a mutação que troca isto por
/// `v / n` **sobrevive**.
///
/// ⚠️ *Uma linha que a mutação não mata não é lei* — e esta fica na mesma, porque o modo de falha
/// dela é a forma do artista **DESAPARECER**. O gate que faltaria precisa de uma pose singular
/// numa direcção e de uma correcção não-nula na mesma alça; quem o construir apaga esta nota.
fn versor(v: Vec2) -> Vec2 {
    let n = v.hypot();
    if n > 0.0 { v / n } else { Vec2::ZERO }
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
/// ⛔⛔⛔ **A TERCEIRA daquelas linhas era VERDADE SOBRE O CAMPO E MENTIRA SOBRE O DESENHO, e o
/// dono reportou-o no dia seguinte** (*«muitas irregularidades … mau tratamento das alças dos
/// handles»*). O `kind` sobrevivia como BYTE e a **geometria deixava de o honrar**: com as duas
/// alças de um nó corrigidas por sistemas independentes elas paravam de ser colineares, e o nó que
/// o artista desenhou LISO virava uma QUINA de `28,62°` a `120°`. *Um campo que sobrevive e uma
/// propriedade que se mantém são coisas diferentes, e só a segunda é o desenho.*
///
/// ⇒ desde 2026-09-19 corre um **quarto passe**, a [`reconcilia`]: as duas alças de cada nó voltam
/// a rodar JUNTAS, e a colinearidade passa a ser exacta. Medido na barra da cena — quebra
/// `0,000°` em todas as dobras, e o desvio à curva verdadeira **melhora** ao mesmo tempo
/// (`0,03371 → 0,01900` a `90°`, contra `0,03371` da lei ingénua), porque conciliar a tangente não
/// tira graus de liberdade ao ajuste: **redistribui-os**.
///
/// ⚠️ O parâmetro `tolerancia` **saiu**: não há o que tolerar quando não há decisão. Quem governa a
/// fidelidade é [`AMOSTRAS`].
pub fn aplica_pela_curva(pele: &Skin, path: &mut VecPath, pesos: &[f64], correcoes: &[Correccao]) {
    aplica_pela_curva_com(pele, path, pesos, correcoes, true, None);
}

/// **A lei da curva com a mistura como PARÂMETRO** — ver [`crate::aplica_corrigido_com`].
pub fn aplica_pela_curva_com(
    pele: &Skin,
    path: &mut VecPath,
    pesos: &[f64],
    correcoes: &[Correccao],
    rigido: bool,
    campo: Option<&crate::pesos::CampoDoDominio>,
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
        // ⭐ O EIXO de cada alça — `(entrada, saída)` por nó. Ver [`reconcilia`].
        let mut eixo = vec![(Vec2::ZERO, Vec2::ZERO); n];
        for k in 0..segs {
            let s = SegmentoDaPele {
                src: cubica(verts, k, n),
                pele,
                ra: linha(pesos, ossos, base + k),
                rb: linha(pesos, ossos, base + (k + 1) % n),
                correcoes,
                rigido,
                campo,
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
            eixo[k].1 = s.direccao(false);
            eixo[j].0 = s.direccao(true);
        }
        reconcilia(path, c, &eixo, fechado);
        base += n;
    }
}

/// ⭐⭐⭐ **A TANGENTE DE UM NÓ VOLTA A SER UMA SÓ** — o passe que cura o report do dono.
///
/// # ⛔⛔⛔ O defeito, medido na barra da cena (2026-09-19)
///
/// *«Ainda temos muitas irregularidades na deformação de vetores. Certamente um mau tratamento das
/// alças dos handles.»*
///
/// A [`correccao_das_alcas`] resolve **cada segmento sozinho**, e as duas alças que se encontram num
/// nó saem de dois sistemas que não se conhecem ⇒ deixam de ser colineares, e o nó que o artista
/// desenhou LISO vira uma QUINA:
///
/// | dobra | quebra da tangente nos nós, p50 | máx |
/// |---|---|---|
/// | `30°` | `2,58°` | `5,86°` |
/// | `60°` | `5,08°` | `13,00°` |
/// | `90°` | `7,30°` | `20,92°` |
/// | `120°` | `9,05°` | `28,62°` |
///
/// ⚠️ **O controlo é o que nomeia a causa:** a lei INGÉNUA ([`crate::aplica_corrigido`]) mede
/// `0,000°` em **todas** aquelas dobras, porque as três metades de um vértice passam pelo MESMO
/// afim e *um afim preserva colinearidade*. ⇒ *a quebra não vinha da pele: vinha do ajuste.*
///
/// # ⭐⭐ A lei: as duas alças de um nó RODAM JUNTAS
///
/// Cada alça já tem um **eixo** — a direcção que o afim daquele nó dá à tangente da fonte
/// ([`SegmentoDaPele::direccao`]). O ajuste livre afastou-a do eixo por um ângulo; este passe faz as
/// duas metades concordarem num ângulo só (a média pesada pelo COMPRIMENTO de cada alça, porque é o
/// braço mais longo que manda no desenho) e roda cada uma para lá. ⇒ *o ângulo entre as duas fica
/// exactamente o que o afim do nó lhe deu* — um nó liso continua liso, um canto mantém o canto.
///
/// ⛔⛔ **Por que RODAR e não reescrever a alça a partir do eixo:** `|h|·versor(h)` **não** é `h` em
/// vírgula flutuante, e em repouso o ajuste é zero ⇒ reescrever devolveria uma forma a mexer-se ao
/// último bit num quadro em que nada se moveu. Uma rotação de `0` é a identidade **ao bit**
/// (`cos 0 = 1`, `sin 0 = 0`), e é isso que mantém a
/// [`tests::em_repouso_as_alcas_ficam_byte_identicas`].
///
/// ⛔⛔ **Uma alça sem EIXO fica fora da média E fora da rotação** — a alça que a fonte tem em cima
/// da âncora (toda arte que a caneta desenha sem arrastar) não carrega tangente, e o nó onde ela
/// chega é um **CANTO**: ali não há continuidade para conciliar, e o ajuste livre é a resposta mais
/// fiel. ⚠️ **São DUAS cercas e nenhuma é zelo**, com um gate cada: deixá-la entrar na média
/// envenena o ângulo comum com um `atan2(0, 0)` e roda a **outra** metade do nó para um sítio que
/// ninguém pediu ([`tests::num_no_misto_a_metade_sem_eixo_nao_entra_na_media`]); deixá-la ser
/// escrita roda-a para um ângulo que ela não tem. Numa forma só de arestas rectas o passe é, por
/// isso, **inerte** ([`tests::numa_forma_de_arestas_rectas_o_passe_nao_toca_em_nada`]).
///
/// ⚠️ **A cerca `soma.1 <= 0.0` NÃO tem gate e fica declarada**: para a divisão importar era
/// preciso um nó em que alguma metade tenha eixo e **comprimento zero ao mesmo tempo**, e o
/// comprimento de uma alça com eixo é `‖R·û‖` mais a correcção livre — zero só por cancelamento
/// exacto. *Uma linha que a mutação não mata não é lei; esta fica porque o que ela evita é um
/// `0/0` a chegar ao `sin_cos`.*
fn reconcilia(path: &mut VecPath, c: usize, eixo: &[(Vec2, Vec2)], fechado: bool) {
    let Some((alvo, _)) = path.contour_mut(c) else {
        return;
    };
    if alvo.len() != eixo.len() {
        return;
    }
    let n = alvo.len();
    for k in 0..n {
        // ⚠️ Num caminho ABERTO as pontas têm uma alça só, e ali não há nada a conciliar.
        if !fechado && (k == 0 || k == n - 1) {
            continue;
        }
        let a = Point::new(alvo[k].anchor[0], alvo[k].anchor[1]);
        let hs = [
            Point::new(alvo[k].in_handle[0], alvo[k].in_handle[1]) - a,
            Point::new(alvo[k].out_handle[0], alvo[k].out_handle[1]) - a,
        ];
        let es = [eixo[k].0, eixo[k].1];
        // O desvio de cada metade ao eixo dela, e o comprimento com que ela pesa.
        let mut soma = (0.0_f64, 0.0_f64);
        let mut desvio = [0.0_f64; 2];
        for i in 0..2 {
            // ⛔⛔ **SEM EIXO não há o que conciliar** — uma alça em cima da âncora na FONTE não
            // carrega tangente, e o nó onde ela chega é um CANTO. ⚠️ **E não há segunda metade a
            // guardar o comprimento**, porque ela seria morta: uma alça de comprimento zero entra
            // na média com peso zero e sai da rotação como zero (rodar o vector nulo dá o vector
            // nulo). *Uma linha que a mutação não mata não é lei.*
            if es[i].hypot() <= 0.0 {
                continue;
            }
            let l = hs[i].hypot();
            desvio[i] = (hs[i].atan2() - es[i].atan2()).rem_euclid(std::f64::consts::TAU);
            if desvio[i] > std::f64::consts::PI {
                desvio[i] -= std::f64::consts::TAU;
            }
            soma = (desvio[i].mul_add(l, soma.0), soma.1 + l);
        }
        if soma.1 <= 0.0 {
            continue;
        }
        let comum = soma.0 / soma.1;
        for i in 0..2 {
            if es[i].hypot() <= 0.0 {
                continue;
            }
            // ⭐ A rotação é a DIFERENÇA para o ângulo comum — em repouso ela é `0`, e `cos 0 = 1`
            // com `sin 0 = 0` devolve a alça **ao bit**.
            let (si, co) = (comum - desvio[i]).sin_cos();
            let g = Vec2::new(
                si.mul_add(-hs[i].y, co * hs[i].x),
                si.mul_add(hs[i].x, co * hs[i].y),
            );
            let q = [a.x + g.x, a.y + g.y];
            if i == 0 {
                alvo[k].in_handle = q;
            } else {
                alvo[k].out_handle = q;
            }
        }
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
/// ⛔⛔ **O determinante NÃO precisa de guarda, e isso foi medido:** a matriz depende **só dos
/// `t`**, logo ela é a MESMA em todo segmento de toda forma — uma constante. A 1.ª redacção tinha
/// um `if det.abs() < 1e-12 { return None }` e a mutação que o apagava **sobreviveu**, porque
/// aquele ramo é inalcançável. *Uma linha que a mutação não consegue matar não é lei, é comentário
/// com sintaxe de código.*
///
/// ⛔⛔ **E não há cerca de `NaN` aqui, também por medição.** A tentação é guardar contra uma
/// diferença não-finita — mas tudo o que chegasse assim já teria passado pela
/// [`crate::aplica_corrigido`], que corre **antes** e escreve o `NaN` no desenho sem nos perguntar
/// nada: *uma cerca a jusante do sítio onde o estrago acontece protege o quê?*
///
/// ⚠️⚠️ **ELA SOZINHA PARTE A TANGENTE DOS NÓS, e é por isso que a [`reconcilia`] corre a seguir.**
/// Cada segmento resolve o seu sistema **sozinho**, logo as duas alças que se encontram num nó saem
/// de dois ajustes que não se conhecem. *Lida isolada, esta função está certa e o desenho fica
/// errado* — ver o report e a tabela em [`reconcilia`].
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
    let det = a12.mul_add(-a12, a11 * a22);
    ((b1 * a22 - b2 * a12) / det, (b2 * a11 - b1 * a12) / det)
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

/// ⭐ **Os gates da CONCILIAÇÃO das alças**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "curva_alcas_tests.rs"]
mod alcas_tests;
