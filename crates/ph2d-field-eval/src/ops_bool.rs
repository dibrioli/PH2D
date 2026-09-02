//! ⭐⭐ **QUEM SE JUNTA A QUEM, E COM QUE FORMA** — os operadores booleanos e o carácter deles.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::ops`] responde a *«que forma cada primitiva é»*; este responde a *«como duas formas se
//! encontram»*. O arquivo passou as `700` linhas do gate de LOC da workspace quando o **chanfro por
//! forma** entrou (Enio, 2026-08-30). ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **Intersecção e subtração são De Morgan**, e não fórmulas próprias: `A ∩ B = ¬(¬A ∪ ¬B)`. Uma
//! fórmula a mais seria a segunda resposta à mesma pergunta — e as duas divergiriam no dia em que um
//! carácter novo nascesse.

use crate::ops::{length2, neg};
use fidget::context::Tree;
use std::f64::consts::FRAC_1_SQRT_2;

pub fn union_sharp(a: &Tree, b: &Tree) -> Tree {
    a.min(b.clone())
}

/// União com filete **exato** de raio `r`.
///
/// `max(r, min(a,b)) − ‖(max(r−a, 0), max(r−b, 0))‖`
///
/// Preserva a propriedade de distância onde `a` e `b` a têm, e por isso **o raio pedido é o raio
/// entregue** — medido a 0,00 % nos gates desta crate.
pub fn union_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    let ux = (Tree::constant(r) - a.clone()).max(0.0);
    let uy = (Tree::constant(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - length2(&ux, &uy)
}

/// ⚠️ **Duas faces mais paralelas do que isto não são uma quina** — quem chamar com `|cos| ≥` isto
/// tem um defeito a montante, e o recorte existe para o **valor não virar `inf` na fita**.
///
/// O multiplicador da [`union_round_at`] é `1/sin φ`, e aqui ele vale `70,7×` — cerca de **30×**
/// além da quina mais aguda do catálogo (a ponta da estrela, `2,29×`). ⇒ *o número é uma rede
/// numérica, não um teto de produto*: nenhuma forma desta casa chega perto dele.
const COS_FACES_MAX: f64 = 0.9999;

/// ⭐⭐⭐ **A UNIÃO ARREDONDADA COM O ÂNGULO DAS FACES DENTRO — o arco VERDADEIRO em qualquer quina.**
///
/// # O achado que ela cura (W104, §102.4 do doc 06)
///
/// A [`union_round`] mede `‖(u, v)‖` com Pitágoras, e Pitágoras só é a distância euclidiana se os
/// dois gradientes forem **ortogonais**. Fora dos 90° o vértice recuava `(1 − 1/√2)·r/sin α` em vez
/// do `r·(1/sin α − 1)` de um arco de raio `r` — numa ponta de estrela (`α = 19,2°`), **2,29× menos
/// filete do que o número no slider diz**.
///
/// # A conta, derivada (⚠️ não copiada — a régua é o valor analítico, como no resto desta crate)
///
/// `u = (r − a)⁺` e `v = (r − b)⁺` são as **projecções** de `Δ = centro − p` sobre as duas normais
/// unitárias. Para duas direcções com `n_a · n_b = c`, o comprimento verdadeiro de `Δ` é
///
/// ```text
/// ‖Δ‖ = √( (u² + v² − 2·c·u·v) / (1 − c²) )
/// ```
///
/// ⭐ **Com `c = 0` ela É a [`union_round`], termo a termo** — e o caminho fica literalmente o mesmo
/// (o `if` abaixo devolve a outra função), de modo que toda forma ortogonal continua **byte a byte**
/// o que era.
///
/// # ⭐ Por que `α` e `c` são o mesmo facto
///
/// Numa intersecção de meios-espaços com ângulo diedro interno `2α`, as normais **exteriores**
/// fazem `c = n_a · n_b = −cos 2α`. Substituindo, o recuo no vértice sai
/// `r·(√(2/(1+c)) − 1) = r·(1/sin α − 1)` — **exactamente** o arco. Os chamadores passam o que a
/// geometria deles conhece: o prisma já calculava o cosseno das paredes vizinhas, e a estrela o
/// meio-ângulo da ponta.
///
/// # ⛔⛔ Ela NÃO é a «cura pelo raio» que a W104 mediu e rejeitou
///
/// Aquela passava `r·(1 − sin α)/(1 − 1/√2)` ao operador — *mudava o raio* para acertar o recuo, e
/// por isso deixava a **largura** da mistura diferente em cada aresta; onde duas larguras diferentes
/// se encontram nasce o vinco que partiu o prisma (`0,0 %` → `5,4 %` de aresta viva). Aqui o raio
/// **é** o raio: o que muda é a métrica com que as duas distâncias se combinam, e ela é a mesma dos
/// dois lados da quina.
///
/// # ⛔⛔⛔ E o RECORTE tem de ser GEOMÉTRICO — a 1.ª versão desta função deslocava a FACE
///
/// Escrever `√((u⁺² + v⁺² − 2c·u⁺·v⁺)/(1 − c²))` dá o arco certo no vértice **e move a face plana**:
/// onde só uma face está activa (`v⁺ = 0`) ela devolve `u/√(1−c²)` em vez de `u`, e o zero deixa de
/// estar na face. Medido: o prisma foi de `0,0 %` para **`3,3 %`** de superfície sobre um vinco
/// (pior `67,6°`) e a quebra de curvatura de `0,15` para `2,14`. *O `max(0)` da fórmula publicada
/// não é «tirar o negativo» — é o ponto mais próximo do CONE, e num referencial oblíquo cortar o
/// coeficiente não é projectar.*
///
/// ⇒ o recorte muda de coordenadas. Com `Δ = centro − p`, escreva-o na **base das duas normais**,
/// `Δ = s·n_a + t·n_b`:
///
/// ```text
/// s = (u − c·v)/(1 − c²)          t = (v − c·u)/(1 − c²)          u = s + c·t     v = t + c·s
/// s⁺ = max(s + c·min(t, 0), 0)    t⁺ = max(t + c·min(s, 0), 0)
/// ‖Δ‖ = √( s⁺² + t⁺² + 2·c·s⁺·t⁺ )
/// ```
///
/// ⭐⭐ **E `s + c·min(t, 0)` é `min(s, u)` num diedro obtuso e `max(s, u)` num agudo** — uma
/// operação em vez de três, com a escolha feita ao **compilar** (o `c` é constante da forma). O
/// preço da lei nova sobre a antiga fica assim (medido, `measure_prism_sides` e
/// `measure_star_points`):
///
/// | forma | nós | ns/ponto |
/// |---|---|---|
/// | prisma hexagonal | `134 → 187` (`1,40×`) | `5,65 → 8,15` (`1,44×`) |
/// | estrela de 5 pontas | `275 → 371` (`1,35×`) | `9,59 → 12,31` (`1,28×`) |
///
/// ⭐ **E o cilindro, a caixa, a esfera, o toro e o elipsóide não mexem um bit** — as quinas deles
/// são ortogonais, logo nem chegam a entrar aqui. *O preço é pago pelas três formas que de facto
/// tinham o defeito.* ⚠️ Sem a simplificação acima seriam `212` e `416` nós.
///
/// ⭐⭐ **As três regiões saem de uma expressão só, e coincidem nas fronteiras:** dentro do cone
/// (`s, t ≥ 0`) ela é `‖Δ‖` exacto; do lado em que a face `B` deixou de contar (`t < 0`) o
/// `s + c·min(t,0)` **é** `u`, logo a lei degenera na distância à face `A` — e em `t = 0` as duas
/// leituras são o mesmo número (`s = u` ali). *Sem esse termo o operador tem o arco certo e a face
/// no sítio errado, que é pior do que o defeito que ele veio curar.*
///
/// ⚠️ **Com `c = 0` tudo isto é a identidade**: `s = u`, `t = v`, os dois `min(·,0)` são zero, e a
/// expressão é `√(u⁺² + v⁺²)` — a fórmula publicada, termo a termo.
///
/// # ⛔⛔ A HISTÓRIA: as duas curas anteriores, medidas e rejeitadas (W104)
///
/// A interseção arredondada publicada é `length2(r + a, r + b) − r`, e o zero dela é
/// `(a+r)² + (b+r)² = r²` — o círculo de raio `r` à volta do ponto que dista `r` das duas faces,
/// que é **exactamente** o centro do filete. ⚠️ Mas `length2(a, b)` só é a distância euclidiana se
/// os dois gradientes forem **ortogonais**. ⇒ o vértice recua `(1 − 1/√2)·r/sin α` em vez do
/// `r·(1/sin α − 1)` de um arco verdadeiro:
///
/// | meio-ângulo interno `α` | recuo do operador | recuo de um arco de raio `r` | razão |
/// |---|---|---|---|
/// | **45°** (quina recta) | `0,414 r` | `0,414 r` | **1,00** |
/// | 30° | `0,586 r` | `1,000 r` | 1,71 |
/// | **19,2°** (ponta de estrela) | `0,892 r` | `2,046 r` | **2,29** |
///
/// ⇒ o mesmo número dá filetes de tamanhos diferentes conforme o ângulo da quina, e uma ponta muito
/// aguda arredonda **2,3× menos** do que se pediu.
///
/// # ⛔⛔ As duas curas, CONSTRUÍDAS e REJEITADAS pela sonda de arestas
///
/// A régua é `measure_sharp_edges` (fração da superfície sobre um vinco, com o filete a metade do
/// limite):
///
/// | construção | prisma | estrela |
/// |---|---|---|
/// | **o operador, tal como shipa** | **`0,0 %` · 2°** | **`0,1 %` · 35°** |
/// | canto **exato** (`min(max(f1,f2,corda), disco)` no referencial `(u,w)` do par de planos) | `0,4 %` · 31° | `1,8 %` · 61° |
/// | raio **compensado** pelo ângulo (`r·(1−sin α)/(1−1/√2)`) | `5,4 %` · 50° | `0,2 %` · 48° |
///
/// ⭐ **O canto exato dá o arco certo e é 1-Lipschitz** — e crava no **vértice de 3 vias**, onde uma
/// quina lateral encontra o aro: ele é `min`/`max` de ramos com troca **dura**, e os dois filetes
/// que ali se encontram não concordam. *O operador é LISO, e a suavidade dele no vértice vale mais
/// do que a exactidão dele na aresta.*
///
/// ⭐ **A compensação dá o recuo certo** — e parte o prisma pela razão simétrica: ela torna o recuo
/// igual **fazendo a largura da mistura diferente** em cada aresta, e onde duas misturas de larguras
/// diferentes se encontram nasce o mesmo vinco. *«Arredondar por igual» tem duas leituras — o recuo
/// e a largura — e só uma delas sobrevive a um vértice.*
///
/// # ⭐⭐⭐ E a W107 dissolveu esta recusa — o operador passou a SABER o ângulo
///
/// As duas curas acima mexiam no **raio** para acertar o recuo, e por isso mudavam a **largura** da
/// mistura; a lei que shipa desde 2026-09-02 não mexe no raio nenhum — ela troca a métrica com que
/// as duas distâncias se combinam, e ela é a mesma dos dois lados da quina
/// ([`union_round_at`]). ⇒ o recuo é o do arco verdadeiro em **qualquer** ângulo, agudo ou obtuso.
///
/// ⛔ **A compensação por raio (`r·max(1, (1 − sin α)/(1 − 1/√2))`) MORREU com ela**, e com ela os
/// três sítios que a chamavam. *Uma cura que precisa de um `max(1, ·)` para não partir metade dos
/// casos estava a dizer que era meia cura.*
///
/// ⚠️ **§0.0:** *quem move o número que tornava algo inalcançável tem de reconferir a nota* — foi a
/// própria régua de CURVATURA da W104-ter (`3,71` sem compensar, `1,19` compensado) que mediu a lei
/// nova, e ela lê `0,15` na estrela.
pub fn union_round_at(a: &Tree, b: &Tree, r: f64, cos_faces: f64) -> Tree {
    if cos_faces == 0.0 || !cos_faces.is_finite() {
        // ⭐ **O caminho de sempre, ao bit** — nem um nó a mais na árvore para quem é ortogonal.
        return union_round(a, b, r);
    }
    let c = cos_faces.clamp(-COS_FACES_MAX, COS_FACES_MAX);
    let inv = (1.0 - c * c).recip();
    let (u, v) = (Tree::constant(r) - a.clone(), Tree::constant(r) - b.clone());
    let s = (u.clone() - v.clone() * Tree::constant(c)) * Tree::constant(inv);
    let t = (v.clone() - u.clone() * Tree::constant(c)) * Tree::constant(inv);
    // ⭐⭐ **O recorte cabe numa operação, e o SINAL de `c` escolhe qual** (ver a nota acima):
    // `s + c·min(t, 0)` vale `s` quando `t ≥ 0` e `u` quando `t < 0`, e `u = s + c·t` — logo é
    // `min(s, u)` num diedro obtuso (`c > 0`, onde `u ≤ s` exactamente quando `t ≥ 0`) e `max(s, u)`
    // num agudo, em que a desigualdade se inverte com o sinal. ⚠️ **A escolha é de COMPILAÇÃO** (`c`
    // é uma constante da forma), então não é uma ramificação na fita — e as duas metades têm gate:
    // a estrela mede o lado agudo, o prisma e o octaedro o obtuso.
    let recorta = |x: Tree, y: Tree| if c > 0.0 { x.min(y) } else { x.max(y) };
    let sp = recorta(s, u).max(0.0);
    let tp = recorta(t, v).max(0.0);
    // ⚠️ `sp² + tp² + 2c·sp·tp` escrito como `(sp+tp)² − 2(1−c)·sp·tp` — a mesma expressão, um nó
    // a menos, e a fita deste módulo é avaliada milhões de vezes por quadro.
    let cruz = sp.clone() * tp.clone();
    let quad = (sp + tp).square() - cruz * Tree::constant(2.0 * (1.0 - c));
    a.min(b.clone()).max(r) - crate::ops::safe_sqrt(quad)
}

/// ⭐⭐⭐ **A UNIÃO ARREDONDADA DE N SUPERFÍCIES, numa operação só** — e o «numa só» é a razão de ela
/// existir.
///
/// `max(r, min aᵢ) − ‖(r − a₁)⁺, …, (r − aₙ)⁺‖` — a extensão directa da [`union_round`], que é o
/// caso `n = 2`.
///
/// # ⛔⛔ Ela nasceu de um report do Enio (2026-08-30): *«o fillet só muda a posição do chamfer»*
///
/// Compor chanfro-e-filete tem três superfícies (as duas faces **e o plano do corte**), e fazê-lo
/// com duas misturas **encaixadas** paga caro por nada: cada nível soma um quadrado na lei de
/// Cauchy–Schwarz, e o campo passa a subir mais depressa que a distância — medido `‖∇f‖ = 4,89` num
/// octaedro, com a marcha a atravessar a superfície.
///
/// ⛔ **E a tentativa de a evitar por «encolher-chanfrar-deslocar» NÃO ARREDONDA**, o que é a lei
/// que a W104 já tinha medido e escrito neste módulo: *deslocar um semiespaço dá outro semiespaço,
/// sem canto para arredondar*. Medido: com o chanfro em `0,12`, o giro da normal na quina fica
/// **cravado em `45,000°`** para qualquer filete — só a posição dela desliza. *É exactamente o que o
/// report diz.*
///
/// ⭐ Aqui as três entram **ao mesmo tempo**: o `length` é sobre todas, e num ponto qualquer só as
/// que estão a menos de `r` contribuem. ⇒ o tecto é `√(quantas estão activas)` — `√2` numa aresta,
/// `√3` num vértice de três — em vez de crescer com o comprimento da corrente.
pub fn union_round_n(pecas: &[Tree], r: f64) -> Tree {
    match pecas {
        [] => Tree::constant(0.0),
        [a] => a.clone(),
        // ⚠️ **Raio zero cai no caminho DURO**, pela razão da [`union`]: com `r = 0` o termo
        // `(r − aᵢ)⁺` é positivo por DENTRO da peça, e o resultado deixa de ser o `min` exactamente
        // onde ele tem de ser. *Não é optimização — é correcção*, e foi o que uma corrida do censo
        // apanhou com o chanfro sozinho.
        _ if r <= 0.0 => {
            let mut menor = pecas[0].clone();
            for p in &pecas[1..] {
                menor = menor.min(p.clone());
            }
            menor
        }
        [a, b] => union_round(a, b, r),
        _ => {
            let mut menor = pecas[0].clone();
            for p in &pecas[1..] {
                menor = menor.min(p.clone());
            }
            let mut soma: Option<Tree> = None;
            for p in pecas {
                let u = (Tree::constant(r) - p.clone()).max(0.0).square();
                soma = Some(match soma {
                    None => u,
                    Some(acc) => acc + u,
                });
            }
            let dist = soma.map_or_else(|| Tree::constant(0.0), |s| s.max(1.0e-30).sqrt());
            menor.max(r) - dist
        }
    }
}

/// ⭐ A intersecção arredondada **com o ângulo das faces dentro** — o dual de De Morgan da
/// [`union_round_at`].
///
/// ⚠️ **`cos_faces` NÃO troca de sinal na negação**, e essa é a razão de haver uma lei só: negar as
/// duas expressões vira **as duas** normais, e o produto interno de dois vectores virados é o mesmo.
/// *A quina convexa de uma intersecção e o entalhe côncavo de uma união são a mesma cunha vista dos
/// dois lados.*
pub fn intersection_round_at(a: &Tree, b: &Tree, r: f64, cos_faces: f64) -> Tree {
    neg(&union_round_at(&neg(a), &neg(b), r, cos_faces))
}

/// A intersecção arredondada de N superfícies — o dual de De Morgan da [`union_round_n`].
pub fn intersection_round_n(pecas: &[Tree], r: f64) -> Tree {
    let negadas: Vec<Tree> = pecas.iter().map(neg).collect();
    neg(&union_round_n(&negadas, r))
}

/// União **suave** (smooth-min polinomial), alcance `k`.
///
/// ⚠️ **NÃO preserva a propriedade de distância**, e o `k` **não é um raio**: medido, entrega 3/4
/// do número pedido. Quem o levar à UI com a etiqueta "raio" mente 25 %, sempre.
pub fn union_smooth(a: &Tree, b: &Tree, k: f64) -> Tree {
    let half = Tree::constant(0.5);
    let h = (half.clone() + half * (b.clone() - a.clone()) / Tree::constant(k))
        .max(0.0)
        .min(1.0);
    let mixed = b.clone() + (a.clone() - b.clone()) * h.clone();
    mixed - Tree::constant(k) * h.clone() * (Tree::constant(1.0) - h)
}

/// ⭐⭐⭐ **União com CHANFRO de alcance `r`** (W99) — o corte reto a 45°, em vez do arco.
///
/// `min(min(a, b), (a + b − r) · √½)`
///
/// # ⚠️ Por que ela é UMA linha, e por que isso não é sorte
///
/// O plano do chanfro num canto de 90° é `a + b = r`, e a distância de um ponto a ele é
/// `(a + b − r)/√2` — **exacta**, não aproximada. O `min` com o canto vivo é o que a limita à região
/// onde ela de facto é a superfície mais próxima. *No CAD, filete e chanfro são duas máquinas com
/// modos de falha diferentes; aqui são a mesma conta com um termo trocado, e nenhuma pode falhar.*
///
/// # ⚠️ Ela é sempre um MINORANTE, e a marcha depende disso
///
/// O resultado é `min(min(a,b), …)`, logo **nunca maior** que `min(a, b)` — que já é um minorante
/// da distância à união. ⇒ andar o valor do campo continua a ser seguro, mesmo onde o termo do
/// chanfro tem gradiente acima de `1` (ele tem, quando as duas normais se alinham). *Um passo é
/// seguro porque o valor é menor que a distância, não porque o gradiente é unitário.*
pub fn union_chamfer(a: &Tree, b: &Tree, r: f64) -> Tree {
    let corte = (a.clone() + b.clone() - Tree::constant(r)) * Tree::constant(FRAC_1_SQRT_2);
    a.min(b.clone()).min(corte)
}

/// Intersecção, com o mesmo caráter de mistura da união — **por De Morgan**.
pub fn intersection(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    neg(&union(&neg(a), &neg(b), blend))
}

/// Subtração (`a` menos `b`): intersectar `a` com o complemento de `b`.
pub fn difference(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    intersection(a, &neg(b), blend)
}

/// O caráter de mistura já resolvido em número, para os três operadores partilharem um caminho só.
#[derive(Clone, Copy, Debug)]
pub enum Blended {
    Sharp,
    Exact(f64),
    /// ⭐⭐⭐ **O CHANFRO** (W99) — o corte reto a 45º. Ver [`union_chamfer`].
    Chamfer(f64),
    Organic(f64),
}

/// A união, escolhendo a fórmula pelo caráter. **Os outros dois operadores passam por aqui** — é o
/// que garante que "arredondar" signifique a mesma coisa nas três operações.
pub fn union(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    match blend {
        // ⚠️ Raio zero cai no caminho DURO de propósito: `union_round(_, _, 0.0)` seria
        // algebricamente equivalente, mas passaria por um `max`/`length` a mais em cada avaliação,
        // e o traçado avalia milhões de vezes por quadro.
        Blended::Sharp => union_sharp(a, b),
        Blended::Exact(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Chamfer(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Organic(k) if k <= 0.0 => union_sharp(a, b),
        Blended::Exact(r) => union_round(a, b, r),
        Blended::Chamfer(r) => union_chamfer(a, b, r),
        Blended::Organic(k) => union_smooth(a, b, k),
    }
}
