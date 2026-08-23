//! ⭐⭐⭐ **O MAPA DO RECTÂNGULO** — num patch de quatro lados a fronteira deixa de
//! ser **presa** por comprimento de arco e passa a **deslizar**.
//!
//! ⛔⛔⛔ **CONSTRUÍDO, MEDIDO e DESLIGADO no mesmo dia** — e ele é o que faltava
//! para o defeito ter **nome**. Ver [`RECTANGLE_MAP`] para a tabela que o rejeita e o
//! fim deste doc para o que ele PROVOU.
//!
//! # ⛔ A premissa que o mandou construir, e ela era um artefacto de régua
//!
//! A conclusão de 2026-08-23 dizia: *as faces de patch `n = 4` medem `0,0°` de
//! enviesamento no domínio e `12°` na superfície ⇒ a grade é encomendada perfeita e
//! chega torta ⇒ o defeito nasce no MAPA.* ⚠️ **O `0,0°` era a mediana de um balde
//! VAZIO** — ver [`crate::FillReport::domain_cells`] e `tests/rulers.rs`. Com as
//! duas réguas corrigidas, a mesma esfera lisa a `d = 0,55`:
//!
//! | fronteira PRESA | régua partida | ⭐ régua corrigida |
//! |---|---|---|
//! | domínio dos **rectângulos** | `0,0°` (nada medido) | **`1,0°`** |
//! | superfície dos **rectângulos** | `12°` | **`16°`** |
//! | superfície dos **leques** | `18°` | **`19°`** |
//!
//! ⇒ ⭐ **a premissa sobreviveu à correcção** (a grade do rectângulo nasce quase
//! recta e chega a `16°`), mas o número que ela dava estava errado, e o leque deixou
//! de ser o termo dominante que a régua partida sugeria.
//!
//! ⚠️ **E trocar o operador do mapa não movia nada:** valor médio contra cotangente,
//! `12° → 12°` na coluna isolada. *Dois Laplacianos diferentes dão o mesmo número* —
//! porque os dois obedecem à MESMA fronteira pregada.
//!
//! # ⭐ A dedução: o operador é inocente, a CONDIÇÃO DE FRONTEIRA é que não é
//!
//! O [`crate::param`] resolve o problema **de Dirichlet puro**: cada vértice de
//! malha da fronteira é **pregado** na aresta do polígono pela sua fracção de `τ`, e
//! o interior é a extensão harmónica disso. ⚠️ **Um mapa harmónico com a fronteira
//! pregada não é conforme** — ele é conforme apenas se a correspondência de fronteira
//! for exactamente a certa, e a fracção de comprimento de arco não é a certa.
//! *Trocar o operador do interior não pode curar uma condição de fronteira errada:
//! os dois operadores obedecem à mesma fronteira.*
//!
//! # ⭐⭐⭐ A construção: dois problemas MISTOS, e a conformalidade sai de graça
//!
//! ```text
//!     ∇²u = 0   com  u = −1 no lado 3,  u = +1 no lado 1,  ∂u/∂n = 0 nos lados 0 e 2
//!     ∇²v = 0   com  v = −1 no lado 0,  v = +1 no lado 2,  ∂v/∂n = 0 nos lados 1 e 3
//! ```
//!
//! ⭐⭐ **Porque isto é conforme, e não é opinião.** Seja `f = u + i·w` o
//! completamento analítico de `u`; o problema misto acima é o do *quadrilátero
//! extremal*, e `f` leva o patch **conformemente** sobre o rectângulo `[−1,1] × [0,H]`,
//! onde `H` é o **módulo conforme** da peça. O segundo problema tem solução única
//! `v = 2w/H − 1` — a mesma conjugada, normalizada. ⇒ `(u, v)` é **um mapa conforme
//! composto com uma escala num eixo**, e uma escala de eixo **preserva a
//! ortogonalidade de direcções alinhadas com os eixos**. A grade do domínio é
//! alinhada com os eixos por construção. ⇒ *ela volta ORTOGONAL à superfície, e o
//! módulo — que este ficheiro nem calcula — só mexeria no ASPECTO, que já mede `1,26`
//! contra `1,22` do oráculo.*
//!
//! # ⚠️ Aqui os pesos COTANGENTES são obrigatórios, e é por isso que sozinhos não valiam
//!
//! A condição natural (`∂/∂n = 0`) de um Laplaciano discreto é **não fixar** o
//! vértice de bordo e deixá-lo na média ponderada dos vizinhos. ⭐ Isso é a condição
//! de Neumann **do operador que se usou** — e só o cotangente é o Laplace–Beltrami da
//! superfície. Com pesos de valor médio o bordo deslizaria para o sítio errado.
//! ⇒ *as duas mudanças só significam alguma coisa JUNTAS*, e é exactamente por isso
//! que medir o cotangente sozinho — com a fronteira pregada — disse «não move nada».
//!
//! # ⛔ As redes, e são TRÊS contagens, não três esperanças
//!
//! O teorema de Tutte não se aplica aqui (nem fronteira convexa pregada, nem pesos
//! garantidamente positivos), então nada disto se assume:
//!
//! 1. **Peso total não-positivo** num vértice ⇒ [`solve`] recusa. Um Gauss–Seidel
//!    sobre soma negativa reflecte em vez de mediar, e diverge.
//! 2. **Monotonia ao longo de cada lado livre** ⇒ [`solve`] recusa se falhar. O `uv`
//!    de um ponto de SAÍDA é interpolado entre os dois vértices de malha à volta
//!    dele; se a coordenada livre recuar, dois pontos de saída trocam de ordem e a
//!    malha rasga ao longo da fronteira do patch.
//! 3. **Triângulos virados no domínio** ⇒ o [`crate::param`] recua para o mapa de
//!    sempre, com a mesma rede que o interior alinhado já usava
//!    ([`crate::aligned::flipped`]).
//!
//! # ⭐⭐⭐ O QUE ELE PROVOU, e é o valor desta construção
//!
//! Esfera lisa, `d = 0,55`, com as réguas corrigidas:
//!
//! | | fronteira PRESA | ⭐ fronteira DESLIZA |
//! |---|---|---|
//! | domínio dos **rectângulos** | `1,0°` | ⛔ **`12,4°`** |
//! | superfície dos **rectângulos** | **`16°`** | `14°` |
//! | superfície dos **leques** | `19°` | `19°` |
//! | patches que deslizaram | `0/5` | `5/5` |
//!
//! ⭐⭐⭐ **Leia as duas primeiras linhas juntas.** Com a fronteira presa, a superfície
//! mede `16°` sobre um domínio de `1,0°` — **`15°` que aparecem do nada e não tinham
//! nome**. Com a fronteira a deslizar, a superfície mede `14°` sobre um domínio de
//! `12,4°` — **`1,6°` de folga**. *A quase-igualdade é a prova de que o mapa é de
//! facto conforme*: um mapa que preserva ângulos entrega na superfície o ângulo que o
//! domínio encomendou.
//!
//! ⇒ ⛔⛔ **A conformalidade não REDUZ o enviesamento; ela MUDA-O DE SÍTIO.** Presa,
//! o mapa carrega-o; a deslizar, a fronteira carrega-o. O total fica em `14°–16°` nos
//! dois casos. **Logo o enviesamento não é propriedade do mapa — é propriedade do
//! PATCH e da forma como o arco é subdividido.**
//!
//! # ⭐⭐⭐ E é aqui que a CLASSE do algoritmo reaparece
//!
//! Os `12,4°` são a **discordância conforme entre lados opostos**: o ponto `k` do
//! lado 0 e o ponto `k` do lado 2 são postos por **comprimento de arco** (`τ`), e a
//! correspondência que a conformalidade pede entre esses dois lados **não é
//! «fracções iguais»**. A linha de grade que os une nasce inclinada, e nenhuma
//! construção do interior a pode endireitar.
//!
//! ⚠️ **O oráculo não tem este problema, e não é por afinação:** ele tem **UMA**
//! parametrização global, e os pontos de subdivisão de um arco são onde as isolinhas
//! inteiras o cruzam. ⇒ os dois patches que partilham o arco concordam **por
//! construção**. Nós subdividimos por `τ` e resolvemos cada patch **em separado**.
//! *É a mesma diferença de classe — local contra global — que motivou o pivô do
//! ADR-0162, um nível abaixo.*
//!
//! ⛔ **A cura, portanto, não é outro mapa por patch.** É uma das duas:
//! *(a)* fazer a subdivisão do arco sair de um acordo entre os dois patches vizinhos
//! (ponto fixo sobre o layout), ou *(b)* a parametrização global quantizada — os
//! inteiros já vêm do F4, então o que resta dela é **linear**.
//!
//! ⚠️ **Clean-room:** o problema misto do quadrilátero extremal e a geração de grade
//! por mapa harmónico são matemática clássica (Winslow 1966; a fórmula cotangente é
//! Pinkall–Polthier 1993). Nenhuma linha vem de fonte GPL — ver ADR-0162.

/// ⭐ **O DOMÍNIO DO RECTÂNGULO é o quadrado alinhado com os eixos `[−1,1]²`**, e
/// não o losango que o [`crate::domain::corners_for`] devolve para `n = 4`.
///
/// ⚠️ **Ele tem de caber no mesmo alcance**, porque o balde de localização do
/// [`crate::param`] mapeia `[−1,1]` em células e não sabe de polígonos. *Um domínio
/// maior faria a localização de ponto falhar na borda, silenciosamente.*
pub(crate) const EDGE: f32 = 1.0;

/// ⛔⛔ **DESLIGADO — MEDIDO E REJEITADO** (2026-08-23), e a rejeição é das
/// **esculturas**, não da esfera.
///
/// ⚠️ **Com `false` o achatamento é byte-idêntico ao de sempre**, e patches de `n ≠ 4`
/// nunca passam por aqui em nenhum dos dois casos.
///
/// # ⭐ Onde ele GANHA — a esfera lisa (`d = 0,55`)
///
/// Enviesamento da superfície nas faces de rectângulo: **`16° → 14°`**, com o domínio
/// a subir de `1,0°` para `12,4°`. *Ver o doc do módulo: essa subida é a prova, não o
/// defeito.*
///
/// # ⛔ Onde ele PERDE, e é onde o artista olha — a orelha (`d = 1,0`)
///
/// | | presa | ⭐ desliza |
/// |---|---|---|
/// | aspecto p50 | **`1,98`** | ⛔ `2,15` |
/// | faces `> 4×` | **3 558** | ⛔ **7 646** |
/// | faces `> 60°` | **9 159** | ⛔ **14 794** |
/// | dobras | **171** | ⛔ **267** |
/// | aresta máxima | **`5,5 %`** | ⛔ `9,9 %` |
/// | detalhe perdido p95 | **`0,222 %`** | ⛔ `0,410 %` |
///
/// O gancho acompanha (`> 4×`: 581 → 747; dobras: 18 → 26); a enrugada não se move.
///
/// # ⚠️ O mecanismo da perda, e ele é o irmão do ganho
///
/// Um mapa conforme é fiel ao **ângulo** e não à **área**. Com a fronteira a
/// deslizar, os pontos de bordo deixam de estar a espaçamento regular no domínio, e
/// num patch geometricamente distorcido isso paga-se em **tamanho de célula** — que é
/// exactamente o que as colunas de aspecto, área e aresta máxima medem. *Numa esfera
/// lisa não há distorção para pagar; numa orelha há.*
///
/// ⇒ **Fica desligado e fica no código**: ele é a testemunha de controlo de toda
/// tabela desta investigação, e é a única forma de voltar a medir o domínio contra a
/// superfície com um mapa que se sabe conforme.
///
/// ⭐ **A sonda que decide é `what_does_the_chain_do_to_a_plain_sphere`**
/// (`shells/desktop/src/sculpt3d_field_follow.rs`) para a esfera e
/// `what_shape_are_our_quads` (`sculpt3d_quad_shape.rs`) para as esculturas, com as
/// colunas separadas por valência — porque *um número que soma duas populações
/// opostas esconde as duas*.
pub(crate) const RECTANGLE_MAP: bool = false;

/// ⭐⭐⭐ **POR QUE UM PATCH NÃO DESLIZOU** — ver [`solve`].
///
/// ⚠️ **A ordem dos variants é a ordem das colunas** em
/// [`crate::FillReport::slid_refused`]; a `Flipped` não nasce aqui — é a rede do
/// [`crate::param`], que conta os triângulos virados **depois** de o mapa fechar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// O patch não tem quatro lados, ou a fronteira não indexa a triangulação.
    NotAQuad,
    /// Um vértice reclamado pelos **dois** lados opostos: o patch está pinçado.
    Pinched,
    /// Algum vértice tem soma de pesos cotangentes `≤ 0` — Gauss–Seidel diverge.
    NegativeWeights,
    /// A coordenada livre recua ao longo de um lado; dois pontos de saída trocariam
    /// de ordem e a malha rasgaria na fronteira do patch.
    NotMonotone,
}

impl Refusal {
    /// A coluna dela em [`crate::FillReport::slid_refused`].
    pub(crate) fn slot(self) -> usize {
        match self {
            Self::NotAQuad => 0,
            Self::Pinched => 1,
            Self::NegativeWeights => 2,
            Self::NotMonotone => 3,
        }
    }
}

/// **O QUE O MAPA DEVOLVE.**
pub(crate) struct Slid {
    /// Por vértice local, o ponto no quadrado `[−1,1]²`.
    pub(crate) uv: Vec<[f32; 2]>,
    /// Quantas rondas de Gauss–Seidel gastou.
    pub(crate) rounds: usize,
    /// O resíduo com que parou.
    pub(crate) residual: f32,
}

/// **RESOLVE OS DOIS PROBLEMAS MISTOS.** `sides[i]` é a cadeia ordenada de vértices
/// **locais** do lado `i`, do canto de entrada ao de saída (o último de um lado é o
/// primeiro do seguinte). `nb` são os pesos **cotangentes**.
///
/// ⚠️ **A recusa é uma resposta e não uma falha** — o chamador fica com o achatamento
/// de sempre. Ver as três redes no doc do módulo.
///
/// ⛔⛔ **E ela diz PORQUÊ**, desde 2026-08-23. Sem o motivo, `deslizou 1/2` não
/// distingue *«o mapa não serve para este patch»* de *«uma das redes é severa demais»* —
/// e foi sobre um `1/2` mudo que eu escrevi *«não é o mapa»*, tendo medido **um patch em
/// seis**. *Um numerador sem motivo é a mesma omissão que um numerador sem denominador.*
pub(crate) fn solve(
    nb: &[Vec<(u32, f32)>],
    sides: &[Vec<u32>],
    rounds_cap: usize,
    tol: f32,
) -> Result<Slid, Refusal> {
    if sides.len() != 4 {
        return Err(Refusal::NotAQuad);
    }
    let nv = nb.len();
    let mut uv = vec![[0.0f32; 2]; nv];
    let mut fix = vec![[false; 2]; nv];
    // O eixo `0` vem dos lados 3 e 1; o eixo `1` dos lados 0 e 2.
    for &(side, axis, value) in &[
        (3usize, 0usize, -EDGE),
        (1, 0, EDGE),
        (0, 1, -EDGE),
        (2, 1, EDGE),
    ] {
        for &v in &sides[side] {
            let v = v as usize;
            if v >= nv {
                return Err(Refusal::NotAQuad);
            }
            // ⛔ **Um vértice reclamado pelos DOIS lados opostos** é um patch
            // pinçado: o quadrado não o pode representar, e escrever por cima daria
            // um domínio que ninguém consegue ler.
            if fix[v][axis] && (uv[v][axis] - value).abs() > f32::EPSILON {
                return Err(Refusal::Pinched);
            }
            uv[v][axis] = value;
            fix[v][axis] = true;
        }
    }
    // ⛔ Rede 1 — ver o doc do módulo.
    for (v, list) in nb.iter().enumerate() {
        if (fix[v][0] && fix[v][1]) || list.is_empty() {
            continue;
        }
        if list.iter().map(|&(_, k)| k).sum::<f32>() <= 0.0 {
            return Err(Refusal::NegativeWeights);
        }
    }

    let (mut rounds, mut residual) = (0usize, f32::INFINITY);
    for r in 0..rounds_cap {
        let mut worst = 0.0f32;
        for v in 0..nv {
            for c in 0..2 {
                if fix[v][c] || nb[v].is_empty() {
                    continue;
                }
                let (mut s, mut sw) = (0.0f32, 0.0f32);
                for &(w, k) in &nb[v] {
                    s = k.mul_add(uv[w as usize][c], s);
                    sw += k;
                }
                if sw <= 0.0 {
                    continue;
                }
                let next = s / sw;
                if !next.is_finite() {
                    return Err(Refusal::NegativeWeights);
                }
                worst = worst.max((next - uv[v][c]).abs());
                uv[v][c] = next;
            }
        }
        rounds = r + 1;
        residual = worst;
        if worst < tol {
            break;
        }
    }

    // ⛔ Rede 2 — a coordenada livre de cada lado tem de ser monótona no sentido em
    // que a cadeia corre. Ver o doc do módulo.
    for &(side, axis, rising) in &[
        (0usize, 0usize, true),
        (1, 1, true),
        (2, 0, false),
        (3, 1, false),
    ] {
        let mut last = if rising {
            -f32::INFINITY
        } else {
            f32::INFINITY
        };
        for &v in &sides[side] {
            let x = uv[v as usize][axis];
            if rising && x < last || !rising && x > last {
                return Err(Refusal::NotMonotone);
            }
            last = x;
        }
    }
    Ok(Slid {
        uv,
        rounds,
        residual,
    })
}
