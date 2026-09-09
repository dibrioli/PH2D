//! ⭐⭐⭐ **COMO UMA CHAPA SE MONTA** — a laje, as paredes, e os planos do chanfro do aro.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::ops`] responde *«que forma é esta primitiva»*; estas três funções respondem
//! *«como é que um perfil 2D vira uma chapa com aresta tratada»*, e são chamadas por **39** sítios
//! em nove módulos. ⛔ *Split, nunca allowlist* — o `ops.rs` estava a `630` das `700` linhas do gate
//! e a W142 precisava de espaço para o segundo perfil da estrela.

use crate::ops_joint::{Edge, intersection_joint, intersection_joint_n};
use fidget::context::Tree;

/// ⭐⭐⭐ **A LEI DAS TRÊS FORMAS DA W101, numa frase:** *um sólido de parede reta é a interseção de
/// uma laje com meias-fatias, e `max` de funções 1-Lipschitz é 1-Lipschitz.*
///
/// # ⚠️ Por que ela existe, e por que NÃO é a fórmula da referência
///
/// O `sdCappedCone` publicado é **exato em toda parte**, e paga por isso com **ramificações**
/// (`(q.y<0)?r1:r2`, e o sinal `(cb.x<0 && ca.y<0)?-1:1`). Esta crate compila para uma fita da
/// `fidget`, e as ramificações que ela tem — `compare`/`and`/`or` — produzem funções
/// **descontínuas**: o gradiente por diferenciação automática deixa de existir na fronteira delas, e
/// quem consome esse gradiente é a extração da malha (sem normal não há QEF) e a marcha. É a mesma
/// razão pela qual o [`LENGTH_FLOOR`] existe, um nível acima.
///
/// ⭐ **O que se perde e o que se ganha, dito com precisão.** `max(a, b)` de duas distâncias exatas
/// é:
/// - **exato na superfície** — o zero de `max` é exatamente a fronteira da interseção;
/// - **exato no interior** — a distância à parede mais próxima é o `max` das perpendiculares;
/// - **um SUBESTIMADOR no exterior**, junto às quinas onde duas paredes não são ortogonais.
///
/// Subestimar é **seguro** para a marcha de esferas (nunca ultrapassa) e custa passos, não
/// correção. E `‖∇f‖ ≤ 1` não é esperança: o máximo de funções 1-Lipschitz é 1-Lipschitz, por
/// definição. ⇒ *o passo da marcha não muda por causa destas formas* — e o gate
/// `every_primitive_honours_the_march` mede-o, forma a forma, derivado de `PrimitiveKind::ALL`.
///
/// ⚠️ **É a MESMA aritmética que o `box_raw` faz**, com uma diferença: ali as três paredes são
/// ortogonais, então o termo exterior (`length` das partes positivas) é exato pelo Pitágoras. Aqui
/// a parede inclina, o Pitágoras deixaria de valer, e por isso o exterior fica no `max`.
///
/// # ⛔⛔⛔ **E O `round` DESTA FAMÍLIA ERA INERTE — medido na W103, três primitivas depois**
///
/// A receita `offset(max(A, B), r)` com as fontes encolhidas **não arredonda nada**, e a álgebra
/// di-lo numa linha: `{max(A,B) − r < 0}` é `{A < r} ∩ {B < r}` — a interseção das duas peças
/// **dilatadas separadamente**, e não a dilatação da interseção. Cada peça é um semiespaço (uma laje,
/// uma parede): dilatar um semiespaço dá outro semiespaço, **sem canto para arredondar**. O que o
/// recuo tira, o deslocamento repõe — e o aro fica **exatamente tão vivo como estava**.
///
/// ⭐ **Por que funciona na caixa e no cilindro:** ali a fonte é o `box_raw`/`cylinder_raw`, que é a
/// distância **exata** (o termo `length` das partes positivas), e a dilatação de uma distância exata
/// É o corpo com os cantos redondos. *A receita nunca foi «encolher e deslocar»: era «encolher uma
/// distância EXATA e deslocar».*
///
/// `walls` são as meias-fatias já **normalizadas** (gradiente unitário); `half_height` é a laje em Z.
///
/// ⭐⭐ **A laje contra as paredes, com a aresta do aro tratada** — a porta do chanfro de toda a
/// família das chapas (Enio, 2026-08-30: *«em todas as peças temos fillet para as bordas
/// arredondadas mas não temos um slider para chamfer»*).
///
/// ⚠️ **Com `chamfer = 0` isto é o caminho de sempre, ao bit** — ver [`crate::ops_joint`].
pub(crate) fn slab_and_walls(walls: &Tree, half_height: f64, e: Edge) -> Tree {
    slab_and_walls_from(walls, walls, half_height, e)
}

/// ⭐⭐⭐ **A MESMA CHAPA, com um perfil PRÓPRIO para o plano do chanfro** (W142).
///
/// # ⛔⛔ Ela nasceu do report do Enio de 08/09: *«o fillet não pega todas as arestas após usar chamfer»*
///
/// O plano do chanfro do aro é `(tampa + paredes + c)·√½` — ele **SOMA** o campo das paredes, e por
/// isso **herda todas as cristas desse campo**. E o campo de um perfil 2D tem uma crista: o **eixo
/// medial**. Numa quina convexa arredondada de raio `r`, o eixo medial começa no **centro do arco**,
/// isto é, à profundidade `r` abaixo da superfície.
///
/// ⇒ **a faceta do chanfro atravessa o eixo medial sempre que `chamfer > round`**, e ali ela deixa
/// de ser um plano: passa a ser um *telhado*, com uma cumeeira sobre a bissetriz da quina.
///
/// ⭐⭐⭐ **E é por isso que o filete não a alcança:** a cumeeira vive **DENTRO DE UMA PEÇA** da
/// mistura, e uma mistura arredonda **entre** peças. *Nenhum filete arredonda uma aresta que esteja
/// dentro de uma peça só.*
///
/// # ⭐ A cura, e por que o número dela não é escolhido
///
/// O plano do chanfro passa a ser construído sobre um perfil cujas quinas estão arredondadas a
/// `round + chamfer`: aí o eixo medial começa `round` **abaixo** do ponto mais fundo da faceta, que
/// nunca o toca. ⭐⭐ E a fronteira interior da faceta — o novo bordo da tampa — fica com raio
/// `(round + chamfer) − chamfer = round` **em planta**, que é exactamente o filete que o artista
/// pediu. *A conta é o filete do vértice escrito no perfil, não uma suavização.*
///
/// ⚠️ **A parede continua a ser a autorada.** Só as `arestas` da mistura n-ária — que é onde os
/// planos do chanfro nascem — vêem o perfil liso; o corpo vê o do artista, e é por isso que a
/// silhueta da peça não se mexe.
///
/// **MEDIDO** na estrela (fracção da superfície sobre um vinco, `probe_the_pair_grid`), com o
/// chanfro a metade do limite:
///
/// | filete | antes | depois |
/// |---|---:|---:|
/// | `0,25·limite` | `11,02 %` | **`0,00 %`** |
/// | `0,50·limite` | `4,35 %` | **`0,00 %`** |
/// | `0,875·limite` | `0,35 %` | **`0,00 %`** |
///
/// ⚠️ **Quem não tem segundo perfil passa o MESMO nos dois** ([`slab_and_walls`]), e a saída é
/// byte-idêntica — a cura é por forma, porque só a forma sabe re-arredondar o próprio contorno.
pub(crate) fn slab_and_walls_from(
    walls: &Tree,
    walls_for_chamfer: &Tree,
    half_height: f64,
    e: Edge,
) -> Tree {
    let slab = Tree::z().abs() - Tree::constant(half_height);
    if e.chamfer <= 0.0 {
        return intersection_joint(&slab, walls, e);
    }
    // ⭐ **As DUAS tampas com SINAL, e não o `|z| − h` dobrado** — a dobra tem um vinco em `z = 0`
    // que o plano do chanfro carrega para a superfície quando o filete lá chega: era isso que
    // punha uma costura no EQUADOR de um cilindro (`19,0°`, contra `1,5°` só com filete).
    let tampa = [
        Tree::z() - Tree::constant(half_height),
        -Tree::z() - Tree::constant(half_height),
    ];
    intersection_joint_n(
        &[tampa[0].clone(), tampa[1].clone(), walls.clone()],
        &[
            (tampa[0].clone(), walls_for_chamfer.clone()),
            (tampa[1].clone(), walls_for_chamfer.clone()),
        ],
        e,
    )
}

/// ⭐⭐⭐ **UMA CHAPA INTEIRA NUMA MISTURA SÓ** — as peças do perfil 2D, as arestas que elas formam,
/// e as duas tampas.
///
/// # ⛔ Ela é a [`slab_and_walls`] sem o encaixe
///
/// A [`slab_and_walls`] recebe o perfil **já composto**, e por isso a mistura do aro herda a costura
/// interna dele e põe-na no aro — é o defeito que o 3.º report do Enio nomeou. Quando o perfil é uma
/// **intersecção** de peças, o chamador tem-nas na mão e pode entregá-las: aí não há composta
/// nenhuma a entrar, e o aro sai tão liso quanto o filete sozinho o faria.
///
/// ⚠️ **Só serve a perfis que são INTERSECÇÃO.** Um perfil feito por **união** (a cruz, a
/// engrenagem, o coração, a estrela) não é exprimível numa intersecção arredondada, e continua a
/// entrar composto pela [`slab_and_walls`] — está nomeado na catraca do
/// `the_chamfer_never_makes_an_edge_worse_than_the_fillet_alone`, e é a população que a
/// [`slab_and_walls_from`] serve.
///
/// ⚠️ **As tampas entram com SINAL**, e cada peça do perfil forma uma aresta com cada uma delas: um
/// perfil de `k` peças dá `k` arestas laterais implícitas mais `2k` de aro.
pub(crate) fn plate_joint_n(
    corpo2d: &[Tree],
    arestas2d: &[(Tree, Tree)],
    half_height: f64,
    e: Edge,
) -> Tree {
    let tampa = [
        Tree::z() - Tree::constant(half_height),
        -Tree::z() - Tree::constant(half_height),
    ];
    let mut corpo = corpo2d.to_vec();
    corpo.extend(tampa.iter().cloned());
    let mut arestas = arestas2d.to_vec();
    for p in corpo2d {
        for t in &tampa {
            arestas.push((p.clone(), t.clone()));
        }
    }
    intersection_joint_n(&corpo, &arestas, e)
}
