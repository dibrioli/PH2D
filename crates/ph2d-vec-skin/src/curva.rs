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
/// ⭐⭐⭐ **O CAMPO É LIDO COM DERIVADA CONTÍNUA?** — a porta de bissecção da cura de 2026-09-20.
///
/// Report do dono, com o arco que ele esperava marcado a verde sobre a foto: *«a imagem vetorial
/// deforma mal, com várias curvas ao longo do caminho. Baixa qualidade para um app pro»*.
///
/// # A causa, que está no PAPER e não é nossa
///
/// O *Bounded Biharmonic Weights* diz que os pesos são **`C¹` nas alças e `C∞` em todo o resto** —
/// e discretiza o problema **com elementos finitos LINEARES**. ⇒ o campo verdadeiro é liso e as
/// facetas são da **discretização**: um campo linear por triângulo tem gradiente constante lá
/// dentro e um SALTO em cada aresta, e o contorno da peça do dono atravessa **119** deles.
///
/// # ⛔⛔⛔ ELA NASCE DESLIGADA, e a razão é a MEDIÇÃO e não a prudência
///
/// A cura funciona **no campo** e **não chega ao que o artista vê**:
///
/// | a `90°` em S, na barra da cena do dono | baricêntrica | **`C¹`** |
/// |---|---:|---:|
/// | ondulações da LEI (o campo sozinho, ponto a ponto) | `68` | **`22`** |
/// | a amplitude delas (`\|k\|` p90) | `2,76` | **`1,62`** |
/// | **ondulações do CAMINHO VECTORIAL — o que se desenha** | **`12`** | **`12`** |
///
/// ⇒ *o ajuste das cúbicas já alisava abaixo do que o campo contribui*, e a `4,3×` menos
/// facetas no campo correspondem a **zero** no desenho. O preço é `0,363 ms` por forma por
/// quadro só para derivar os gradientes (`2,2 %` de um quadro a uma forma presa, `17 %` a oito),
/// mais `~50 %` por consulta.
///
/// ⚠️⚠️ **A frase que estava aqui — *«os `12` que sobram são do AJUSTE; a cura que falta é um
/// ajuste com continuidade GLOBAL»* — acertou a METADE e prescreveu a cura ERRADA.** Eram mesmo do
/// ajuste, e não do ajuste em si: eram da **conciliação das alças**, um passe que corria a seguir
/// a ele e que foi apagado em 2026-09-20 (a tabela está em [`aplica_pela_curva`]). *A cura não era
/// mais continuidade — era menos.*
///
/// ⭐⭐⭐ **E com a conciliação fora a previsão da última linha desta nota cumpriu-se: o campo
/// VOLTOU a ser o tecto, e agora ela chega ao desenho.** Re-medido pela mesma porta, `90°` em S,
/// contra o padrão-ouro lido com a MESMA lei dos dois lados:
///
/// | a `90°` em S, na barra da cena do dono | baricêntrica | **`C¹`** |
/// |---|---:|---:|
/// | **excesso de curvatura do desenho** (`κ+` p90, graus de quina) | `3,16°` | **`1,98°`** |
/// | serpentina p50 | `0,001727` | `0,001789` |
/// | quebra da tangente nos nós (máx) | `13,65°` | `15,86°` |
///
/// ⛔ **E ela continua DESLIGADA, agora por um número e não por ausência de efeito:** `3,16°` de
/// excesso sobre a janela de `B_H = 0,05` é uma flecha de `~0,14 %` da espessura da barra —
/// abaixo do que se vê —, e o preço é `17 %` de um quadro a oito formas presas. *Uma cura
/// invisível não paga um sexto do quadro.*
///
/// ⚠️ **Repare no que ela NÃO compra:** a quebra da tangente **não melhora** (`13,65 → 15,86`).
/// Ela não vem de o campo ser `C⁰` — vem de cada segmento ser ajustado sozinho, e **o CHÃO do
/// modelo paga-a igual**.
///
/// ⚠️ **Lida no SÍTIO DE CHAMADA e uma vez por forma**, pela mesma razão escrita nas duas irmãs
/// acima: um `var_os` por amostra seria uma syscall dentro do laço do desenho.
#[must_use]
pub fn lei_c1_activa() -> bool {
    std::env::var("PH2D_SKIN_C1").as_deref() == Ok("1")
}

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
    /// ⭐⭐⭐ **O CAMPO DO DOMÍNIO, quando o bind o guardou** — ver [`Self::pesos`].
    campo: Option<&'a crate::pesos::CampoDoDominio>,
    /// ⭐⭐⭐ **O ÍNDICE da malha do campo** — derivado UMA vez por forma pelo chamador. Sem ele a
    /// consulta varre TODOS os triângulos, e isso é `95 %` do custo de amostrar um ponto
    /// ([`crate::pesos::IndiceDoCampo`]).
    indice: Option<&'a crate::pesos::IndiceDoCampo>,
    /// ⭐⭐⭐ **A leitura `C¹` do MESMO campo** — ver [`crate::pesos_suave`].
    ///
    /// ⛔⛔ **Os DOIS coexistem de propósito, e a 1.ª redacção não os separou:** ela punha só o
    /// suave, e com `PH2D_SKIN_C1=0` o campo deixava de ser consultado **de todo** — a porta da
    /// leitura desligava em silêncio a porta do CAMPO, que é outra wave. A sonda leu `24` em vez
    /// de `12` e denunciou-a. *Uma porta de bissecção que desliga duas coisas não bissecta nada.*
    suave: Option<&'a crate::pesos_suave::CampoSuave<'a>>,
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
        // ⭐ A leitura `C¹` primeiro; a baricêntrica é o que sobra quando a porta a desliga.
        let lida = self
            .suave
            .and_then(|s| s.linha(p))
            .or_else(|| self.campo.and_then(|c| c.linha_com(p, self.indice)));
        if let Some(linha) = lida {
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

    /// ⭐⭐⭐ **A IMAGEM E A TANGENTE** — o par que um fitter adaptativo exige.
    ///
    /// A derivada é a **diferença central da própria função de amostragem**, em `t`. ⛔ A forma
    /// fechada existiria, mas atravessaria o `quota` de um osso que dobra e o bump de cada mancha,
    /// e **cada um deles é um ramo** — a lei ficaria escrita duas vezes e a segunda envelhece.
    /// *Uma diferença central da MESMA função é consistente com ela por construção, que é o que o
    /// contrato do fitter de facto exige* (o cabeçalho da [`ph2d_vec_envelope::Warp`] mede o
    /// sintoma de a quebrar: o fit **não converge**).
    ///
    /// ⚠️ **Nas pontas a diferença é de UM LADO só**, e não por zelo: fora de `[0, 1]` a mistura
    /// `lerp(ra, rb, t)` extrapola a tabela de pesos, e uma tangente tirada de pesos que não
    /// existem é ruído com cara de derivada.
    fn ponto_e_tangente(&self, t: f64) -> (Point, Vec2) {
        const H: f64 = 1e-6;
        let (a, b) = ((t - H).max(0.0), (t + H).min(1.0));
        let (pa, pb) = (self.ponto(a), self.ponto(b));
        (self.ponto(t), (pb - pa) / (b - a))
    }

    /// A imagem de `C(t)`.
    fn ponto(&self, t: f64) -> Point {
        let c = self.src.eval(t);
        let p = [c.x, c.y];
        let w = self.pesos(p, t);
        self.mistura(p, &w)
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
/// ⛔⛔⛔ **E ENTRE 2026-09-19 E 2026-09-20 CORREU UM QUARTO PASSE AQUI — a `reconcilia` — QUE FOI
/// APAGADO, porque era ELE a queixa seguinte do dono** (*«não fica bom. Muito curvado»*, com foto).
///
/// A história, porque ela é a lição: ao ver que as duas alças de um nó saíam de dois sistemas
/// independentes e deixavam de ser colineares (um nó LISO virava uma quina de `28,62°`), escrevi um
/// passe que as rodava de volta para um ângulo comum. Ele curava a quina — e **era ele, sozinho, a
/// serpentina**. Medido na barra da cena, `90°` em S, `54` nós, pela porta do produto:
///
/// | | serpentina p50 | desvio ao ouro (máx) | **κ excesso p90** | quebra máx |
/// |---|---:|---:|---:|---:|
/// | o ajuste **+ a conciliação** | `0,016016` | `0,00546` | `3,86°` | `0,000°` |
/// | **só o ajuste** (o que corre hoje) | **`0,001727`** | **`0,00341`** | **`3,16°`** | `13,65°` |
/// | a lei do **RIVE** (sem ajuste nenhum) | `0,005554` | `0,02939` | `21,05°` | `0,000°` |
/// | o **CHÃO** do modelo | `0,001752` | `0,00334` | `3,18°` | — |
///
/// ⭐⭐⭐ **Só o ajuste É o chão do modelo** — ele acerta-o às três casas nas três colunas, em todas
/// as dobras de `30°` a `120°`. *O ajuste nunca foi o defeito.* A conciliação pega na solução
/// ÓPTIMA e roda-a para fora do óptimo por um desvio diferente em cada nó ⇒ segmentos vizinhos
/// ficam empurrados para lados opostos, que é à letra o que se lê como uma linha a serpentear.
///
/// ⛔⛔⛔ **E o `13,65°` que fica NÃO é um defeito do ajuste: é a VERDADE.** Os pesos do
/// padrão-ouro são lidos por coordenadas baricêntricas sobre a malha BBW, logo o mapa é `C⁰` e a
/// arte deformada **tem mesmo** pequenos bicos onde o contorno atravessa uma aresta da malha — o
/// [`lei_c1_activa`] escreve o mecanismo. A prova é a coluna do EXCESSO de curvatura, que mede
/// contra o padrão-ouro em vez de contra uma curva ideal imaginária: ali o ajuste lê `3,16°` e o
/// chão lê `3,18°`. *As quinas do ajuste são as quinas que a lei tem, e a conciliação apagava uma
/// feição da verdade por `9,3×` de serpentina.*
///
/// ⛔⛔ **E isto REFUTA a alavanca que eu próprio tinha nomeado** — *«a cura que falta é um ajuste
/// com continuidade GLOBAL»*: um ajuste global com `G¹` duro é **mais** restrito que o óptimo livre
/// por segmento, e como a verdade não é `G¹` ele só pode afastar o desenho do padrão-ouro. A célula
/// foi construída e medida (a colinearidade metida DENTRO do sistema, cada alça presa ao eixo do
/// próprio nó): `κ excesso p90 14,48°` contra os `3,16°` do livre. *Impor uma suavidade que a lei
/// não tem é pagar fidelidade por nada.*
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
    // ⭐⭐⭐ **OS GRADIENTES DO CAMPO DERIVAM-SE UMA VEZ POR FORMA, aqui.** A leitura `C¹` precisa
    // do gradiente de cada peso em cada vértice da malha, e ele é `O(V·B)`: derivá-lo por AMOSTRA
    // de curva seria uma varredura da malha dentro do laço do desenho.
    //
    // ⚠️ `PH2D_SKIN_C1=0` volta à leitura baricêntrica de sempre, e é por onde se bissecta um
    // report — ver [`lei_c1_activa`].
    let suave = campo
        .filter(|_| lei_c1_activa())
        .and_then(crate::pesos_suave::CampoSuave::novo);
    let suave_ref = suave.as_ref();
    // ⭐⭐⭐ **O ÍNDICE DA MALHA, derivado UMA VEZ POR FORMA** — a mesma disciplina dos gradientes
    // acima, e pela mesma razão elevada ao quadrado: sem ele cada amostra varre os `878`
    // triângulos da malha, o que é `95 %` do custo de amostrar um ponto.
    let indice = campo.and_then(|c| crate::pesos::IndiceDoCampo::novo(&c.malha));
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
                campo,
                indice: indice.as_ref(),
                suave: suave_ref,
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

impl kurbo::ParamCurveFit for SegmentoDaPele<'_> {
    fn sample_pt_tangent(&self, t: f64, _sign: f64) -> kurbo::CurveFitSample {
        // `sign` escolhe de que lado de uma descontinuidade amostrar. Ignorá-lo é coerente com o
        // `break_cusp` abaixo: não há lado a escolher.
        let (p, tangent) = self.ponto_e_tangente(t);
        kurbo::CurveFitSample { p, tangent }
    }

    fn sample_pt_deriv(&self, t: f64) -> (Point, Vec2) {
        self.ponto_e_tangente(t)
    }

    /// ⚠️ **Nenhuma cúspide, e aqui isso é uma AFIRMAÇÃO MAIS FRACA que a da irmã** — a
    /// [`ph2d_vec_envelope`] prova que nenhum mapa dela dobra; a pele **pode** dobrar (o mapa dobra
    /// sobre si mesmo em dobras fortes, e isso está declarado em aberto no §5 do `CLAUDE.md`).
    /// ⛔ O custo de o deixar em `None` é o que a doc do kurbo diz — cúspide perdida vira mais
    /// subdivisão, *«generally not a disaster»* —, e é por isso que o **tecto de segmentos** da
    /// [`refit_pela_curva`] não é decoração.
    fn break_cusp(&self, _range: core::ops::Range<f64>) -> Option<f64> {
        None
    }
}

/// **Como o campo é LIDO** — que campo, e com que continuidade.
///
/// ⚠️ **Ela existe porque a `c1` viaja como PARÂMETRO aqui** (ao contrário da irmã
/// [`aplica_pela_curva_com`], que a lê do ambiente lá dentro — dívida nomeada no gate
/// `a_leitura_c1_cura_o_campo_e_nao_chega_ao_desenho`), e porque um oitavo argumento solto é o
/// que o clippy recusa com razão: *dois booleanos e um `Option` soltos numa assinatura são três
/// oportunidades de os trocar de ordem.*
#[derive(Clone, Copy)]
pub struct LeituraDoCampo<'a> {
    /// O campo do domínio, quando o bind o guardou.
    pub campo: Option<&'a crate::pesos::CampoDoDominio>,
    /// A leitura com derivada contínua ([`crate::pesos_suave`]).
    pub c1: bool,
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
/// ⚠️⚠️ **CADA SEGMENTO RESOLVE O SEU SISTEMA SOZINHO, e é DE PROPÓSITO.** As duas alças que se
/// encontram num nó saem de dois ajustes que não se conhecem, e a tangente ali parte-se um pouco
/// (`p50 0,97°`, máx `13,65°` a `90°` na barra da cena). ⛔ **Conciliá-las custa `9,3×` de
/// serpentina e foi medido e APAGADO** — a tabela e o mecanismo estão em [`aplica_pela_curva`].
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

/// ⭐⭐⭐ **O SEGUNDO CORPO — as duas rotas MEDIDAS**, num irmão. Ver o cabeçalho dele: uma foi
/// recusada pelo preço e a outra é a ideia do dono, `7,7×` mais barata e melhor.
#[path = "curva_segundo_corpo.rs"]
mod segundo_corpo;
pub use segundo_corpo::{Bake, refit_pela_curva, refit_pelo_bake};

#[cfg(test)]
#[path = "curva_tests.rs"]
mod tests;

/// ⭐ **Os gates da CONCILIAÇÃO das alças**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "curva_alcas_tests.rs"]
mod alcas_tests;
