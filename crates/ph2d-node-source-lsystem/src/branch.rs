//! **O ESQUELETO VIRA RAMOS** — a decomposição que faz um tronco ser UMA forma contínua e não
//! uma pilha de retângulos.
//!
//! Report do Enio (2026-08-30): *"as formas crescem sempre separadas e não crescem como um
//! objeto só. O tronco deve ter uma estrutura única e não vários retângulos soltos
//! sobrepostos."*
//!
//! # A lei é unânime, e não é nossa
//!
//! Um ramo **não é uma sequência de segmentos desenhados**: é uma CURVA (o eixo) com uma
//! FUNÇÃO DE RAIO ao longo dela, e a superfície é esse perfil varrido pelo eixo. As quatro
//! referências dizem-no com quatro vocabulários (estudo completo no
//! [doc 95](../../../docs/Motion%20Nodes/95_estudo_ramificacao_continua_e_instancias.md)):
//!
//! - **cpfg**, o interpretador dos autores do ABOP — que é a nossa referência: `@Gs`/`@Gc(n)`/
//!   `@Ge(n)` põem pontos de controlo, e `@Gr(...)` dá o perfil de raio entre eles;
//! - **Houdini** *L-System SOP*: emite **polilinha** + largura, e quem faz o tubo é outro nó
//!   (*PolyWire* / *Sweep*);
//! - **SpeedTree**: *spine* com perfil de raio;
//! - **Blender** *Sapling*: curvas com *bevel* + *taper*.
//!
//! ⇒ **O varrimento é uma etapa PRÓPRIA, em todas.** Este módulo é a primeira metade dela: a
//! que decide *quais pontos formam um ramo*. A segunda — transformar a polilinha num contorno
//! preenchido — é da shell, que tem o `power_stroke` (⛔ um nó não alcança a biblioteca vetorial,
//! e é essa cerca que deixa o cook memoizar e repetir ao bit — ADR-0154).
//!
//! # ⭐ A JUNÇÃO, que é o que separa isto de «cortar a cadeia em pedaços»
//!
//! No cpfg, `[` guarda *"o último ponto de controlo antes do ramo"* e o filho **liga-se a
//! ele**. É por isso que uma bifurcação não tem buraco. Aqui a polilinha de um ramo filho
//! **começa na posição do pai E com a largura do pai**, e afina para a largura própria ao longo
//! do primeiro passo — o **colar** que faz as duas silhuetas coincidirem naquele ponto.
//!
//! ⚠️⚠️ **A largura da junção era `min(pai, filho)` e o smoke reprovou-a** (Enio, 2026-08-30:
//! *"não há continuidade perfeita entre um tronco e seus ramos"*): com o mínimo, um galho fino
//! nasce **fino no meio da silhueta grossa do tronco**, e o degrau vê-se. *A restrição do
//! SpeedTree é «o raio nunca EXCEDE o do pai ali» — igualar satisfá-la, e o `min` era a minha
//! leitura conservadora dela, não a lei.*
//!
//! ⚠️ **Isso não muda nada de autorado**: aquele ponto não existia antes desta wave (não havia
//! fita nenhuma), então a regra escolhe um número NOVO, não substitui um número do artista.
//!
//! # ⭐ E a PONTA afina, se lhe pedirem
//!
//! O `tip_taper` leva a largura do ÚLTIMO ponto de um ramo **terminal** a zero. `0` devolve a
//! ponta de sempre, byte a byte. É o *taper* que o Blender Sapling (*"taper the tip to become
//! thinner and thinner"*) e o SpeedTree têm — e sem ele um raminho acaba como um palito
//! cortado a direito, que foi a outra metade do mesmo report.
//!
//! # ⚠️ Só `F` e `G` entram numa fita
//!
//! O `J`/`K`/`M` são âncoras de instância (uma folha, uma flor) e não têm osso — enfiá-las na
//! polilinha faria a fita dar um passo de comprimento zero e o perfil de largura ganhar uma
//! parada duplicada. O `f`/`g` já **corta** a cadeia por construção (o elemento seguinte nasce
//! raiz), então nem chega aqui.

/// **Um ramo**: a linha de centro e a largura em cada ponto dela.
///
/// ⚠️ **Dados simples de propósito** — sem `VecPath`, sem `WidthStops`, sem nada da biblioteca
/// vetorial. Quem constrói a forma é a shell; este módulo vive dentro de um nó, e um nó que
/// alcançasse o motor de desenho deixaria o cook de ser memoizável.
#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    /// A linha de centro, da junção (ou da base) até à ponta. **Sempre ≥ 2 pontos** — um ramo
    /// de um ponto não é uma fita, e é descartado.
    pub points: Vec<[f32; 2]>,
    /// A largura em cada ponto — o mesmo comprimento que [`Self::points`].
    pub widths: Vec<f32>,
}

impl Branch {
    /// O comprimento de arco acumulado até cada ponto, normalizado a `[0, 1]`.
    ///
    /// ⚠️ **Por ARCO e não por índice**, porque é assim que um perfil de largura é definido
    /// nesta casa (`WidthStop::pos` é *"fracção do comprimento de ARCO"*, e o doc dele explica
    /// porquê: *picar uma aresta em vinte pedaços não pode mover o ponto mais grosso do
    /// traço*). Um L-System pica os ramos de forma desigual — o `"` encurta o passo a cada
    /// geração —, então indexar por posição na lista poria a mesma planta com perfis
    /// diferentes conforme a profundidade.
    ///
    /// Um ramo de comprimento zero (todos os pontos no mesmo sítio) devolve tudo em `0`.
    #[must_use]
    pub fn arc_fractions(&self) -> Vec<f32> {
        let n = self.points.len();
        let mut acc = Vec::with_capacity(n);
        let mut total = 0.0f32;
        acc.push(0.0);
        for i in 1..n {
            let (a, b) = (self.points[i - 1], self.points[i]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            total += (dx * dx + dy * dy).sqrt();
            acc.push(total);
        }
        if total <= 0.0 {
            return vec![0.0; n];
        }
        acc.iter().map(|s| s / total).collect()
    }
}

/// Uma raiz não tem pai — o mesmo sentinela que a tartaruga escreve na coluna `parent`.
const NO_PARENT: i32 = -1;

/// Este elemento desenha um osso? (⇒ entra numa fita)
fn draws(sym: f32) -> bool {
    matches!(sym as i32 as u8, b'F' | b'G')
}

/// O índice de pai como inteiro, ou [`NO_PARENT`].
fn parent_of(parent: &[f32], i: usize) -> i32 {
    let p = parent.get(i).copied().unwrap_or(-1.0);
    if p < 0.0 { NO_PARENT } else { p as i32 }
}

/// **A decomposição** — o esqueleto (`P` · `parent` · `size` · `sym`) em ramos.
///
/// Um ramo é uma corrida máxima de elementos em que cada um é o **único filho que desenha** do
/// anterior. Uma bifurcação termina o ramo do pai e **começa** um por filho, cada um a partir da
/// posição do pai (a lei da junção, acima).
///
/// ⚠️ **As quatro colunas chegam por fatia, não por `Stream`** — é o que torna esta lei
/// testável sem cozinhar um grafo, e é também o que a deixa servir o `rig.*`, que publica
/// exactamente as mesmas colunas ([doc 92](../../../docs/Motion%20Nodes/92_o_que_o_mini_cavalry_tem_e_nos_nao.md)
/// §2 item 8: *sem um desenhador de esqueleto, os cinco `rig.*` são difíceis de autorar*).
///
/// Colunas mais curtas que `p` leem-se com o neutro (`parent = -1`, `size = 1`, `sym = F`), que
/// é o caso de quem chamar isto com um esqueleto sem `sym`.
#[must_use]
pub fn branches(
    p: &[[f32; 2]],
    parent: &[f32],
    size: &[[f32; 2]],
    sym: &[f32],
    tip_taper: f32,
) -> Vec<Branch> {
    let n = p.len();
    // Sem `sym` tudo desenha — é o esqueleto genérico (o do `rig.*`), que não tem alfabeto.
    let draws_i = |i: usize| sym.get(i).copied().is_none_or(draws);
    let width_i = |i: usize| size.get(i).map_or(1.0, |s| s[0]);

    // Quantos filhos que DESENHAM cada elemento tem, e qual é o único (quando é um só).
    // ⚠️ Uma folha (`J`) filha do tronco **não** conta: ela não interrompe a fita, e contá-la
    // partiria o tronco em dois exactamente onde o artista pôs uma folha.
    let mut kids = vec![0u32; n];
    let mut only_kid = vec![NO_PARENT; n];
    for i in 0..n {
        if !draws_i(i) {
            continue;
        }
        let par = parent_of(parent, i);
        if par == NO_PARENT || par as usize >= n {
            continue;
        }
        let par = par as usize;
        kids[par] += 1;
        only_kid[par] = if kids[par] == 1 { i as i32 } else { NO_PARENT };
    }

    let mut out = Vec::new();
    for start in 0..n {
        if !draws_i(start) {
            continue;
        }
        let par = parent_of(parent, start);
        let par_ok = par != NO_PARENT && (par as usize) < n && draws_i(par as usize);
        // Continua a fita do pai? Só se ele desenha e este é o ÚNICO filho dele.
        if par_ok && kids[par as usize] == 1 {
            continue;
        }

        let mut points = Vec::new();
        let mut widths = Vec::new();
        // ⭐⭐ **A JUNÇÃO: o filho começa no ponto do pai E COM A LARGURA DO PAI.**
        //
        // ⚠️ **Era `min(pai, filho)` até ao smoke de 2026-08-30, e o report do Enio nomeia
        // exactamente o que isso produz:** *"não há continuidade perfeita entre um tronco e
        // seus ramos"*. Com o mínimo, um galho fino nasce fino **no meio da silhueta grossa do
        // tronco** — as duas superfícies encostam-se num degrau, e o degrau vê-se.
        //
        // Tomando a largura do PAI ali, os dois contornos coincidem naquele ponto e a união
        // fecha: o galho abre num **colar** e afina para a largura dele ao longo do primeiro
        // passo. É o que a referência descreve — o SpeedTree crava que *"o raio nunca pode
        // exceder o do pai no ponto onde o galho nasceu"*, e IGUALAR satisfaz a restrição; o
        // `min` era a minha leitura conservadora dela, não a lei.
        if par_ok {
            let par = par as usize;
            points.push(p[par]);
            widths.push(width_i(par));
        }

        let mut cur = start;
        let terminal;
        loop {
            points.push(p[cur]);
            widths.push(width_i(cur));
            let next = only_kid[cur];
            if next == NO_PARENT || kids[cur] != 1 {
                // ⭐ **Uma ponta é TERMINAL quando nada continua a partir dela** — não quando o
                // ramo acaba. Um ramo que acaba numa BIFURCAÇÃO passa a espessura aos filhos, e
                // afiná-lo ali abriria um buraco no meio da árvore.
                terminal = kids[cur] == 0;
                break;
            }
            cur = next as usize;
        }

        // ⭐ **O AFINAMENTO DA PONTA** (report do Enio, 2026-08-30: *"as pontas não têm opção de
        // afinar"*). `0` devolve a largura de sempre — **byte a byte** —, `1` leva a ponta a
        // zero. É o *taper* que o Blender Sapling e o SpeedTree têm, e é por isso que uma
        // referência desenha um raminho e não um palito cortado.
        //
        // ⚠️ **Só na ponta TERMINAL**, e só no ÚLTIMO ponto: o afinamento é uma propriedade do
        // fim do ramo, não uma segunda lei de largura a competir com o `!` da gramática.
        if terminal
            && tip_taper > 0.0
            && let Some(w) = widths.last_mut()
        {
            *w *= (1.0 - tip_taper).clamp(0.0, 1.0);
        }

        // ⚠️ Um ramo de UM ponto não é uma fita. Acontece de verdade: uma raiz cuja única
        // continuação é uma folha (`J`) fica sem filho que desenhe. Emiti-lo daria uma
        // polilinha degenerada ao `power_stroke`, que devolve vazio — e um vazio silencioso
        // lê-se como *"o ramo sumiu"*.
        if points.len() >= 2 {
            out.push(Branch { points, widths });
        }
    }
    out
}

#[cfg(test)]
#[path = "branch_tests.rs"]
mod tests;
