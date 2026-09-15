//! ⭐⭐⭐ **O OSSO QUE DOBRA** (*B-Bone* do Blender, *Bendy Bone* do Spine) — um osso cujo eixo é
//! uma **Bézier cúbica** em vez de um segmento recto.
//!
//! # A lei não muda; muda quem PRODUZ
//!
//! A [`crate::Skin`] já mistura `N` poses **rígidas** por peso, que é exactamente o que um osso
//! curvo é: o Blender diz *«Segments = N»* e fabrica `N` ossos virtuais. ⇒ esta crate não ganha
//! matemática de pele nenhuma — ela ganha uma **fábrica de sub-ossos**, e a pele ganha só a regra
//! de *que fatia do eixo cada sub-osso governa* ([`share`]).
//!
//! # ⭐ O ponto neutro é exacto POR CONSTRUÇÃO, não por tolerância
//!
//! As alças nascem nos **terços** do eixo, onde a cúbica degenera no segmento por identidade
//! polinomial:
//!
//! ```text
//! B(t) = linear(t) + 3(1−t)²t·inn + 3(1−t)t²·out
//! ```
//!
//! Com `inn = out = 0` os dois termos de correcção são `0.0` e `B(t) = (L·t, 0)` **ao bit**. E o
//! frame de cada sub-osso é construído a partir da razão `(c₁−c₀)/(x₁−x₀)`, cujo numerador e
//! denominador saem da **mesma expressão** quando a curva é recta ⇒ a razão é `1.0` exacta, o
//! afim é [`Xform::IDENTITY`] exacto, e a translação é zero exacta.
//!
//! ⚠️ **E a fábrica colapsa mesmo assim:** [`crate::SkinBone::bent`] devolve UM osso quando a
//! curvatura é recta, seja qual for o número de segmentos. *Uma recta não precisa de `N` ossos
//! para a desenhar, e colapsá-la é o que torna o caminho antigo byte-idêntico em vez de
//! byte-parecido.*
//!
//! # ⛔ O que este módulo NÃO decide
//!
//! De onde vêm as alças. Aqui elas são **autoradas** (dois deslocamentos). Derivá-las dos ossos
//! vizinhos — o *Handle Type: Auto* do Blender — é uma lei sobre a HIERARQUIA, e a hierarquia não
//! existe nesta crate de propósito (ver o cabeçalho da [`crate`]).

use crate::Xform;

/// **As duas alças de curvatura de um osso**, em espaço LOCAL dele (o eixo é o `+x`, a origem é a
/// raiz), como **deslocamento a partir do terço** onde a alça recta mora.
///
/// ⚠️ São deslocamentos e não posições absolutas por causa do ponto neutro: `[0, 0]` tem de ser
/// *«este osso é recto»*, e com posições absolutas o neutro seria `L/3` — um valor que depende do
/// comprimento e que ninguém acerta ao escrever um `Default`.
/// ⚠️ **Ele atravessa o ficheiro** (é campo do `Bone` do lado do ECS), então deriva `serde` aqui —
/// a mesma decisão, e pela mesma razão, do [`crate::BendSide`]: declará-lo outra vez do lado do ECS
/// com uma conversão no meio seria a *segunda porta* que duas definições do mesmo conceito abrem.
#[derive(Copy, Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Bend {
    /// Deslocamento da alça da RAIZ, a partir de `(L/3, 0)`.
    pub inn: [f64; 2],
    /// Deslocamento da alça da PONTA, a partir de `(2L/3, 0)`.
    pub out: [f64; 2],
}

/// ⭐⭐ **O OSSO COMO O ARTISTA O AUTOROU** — comprimento, alcance, segmentos e curvatura, sem esta
/// crate saber o que é uma entidade, um componente ou uma hierarquia.
///
/// ⚠️ **Ele existe porque os quatro campos viajam SEMPRE juntos**, e porque escrevê-los soltos numa
/// assinatura já os pôs a `8` argumentos — *o número de parâmetros de uma porta é a forma mais
/// barata de um agrupamento em falta se anunciar*. O `Bone` do lado do ECS é o espelho dele, e a
/// conversão mora num sítio só.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoneSpec {
    /// O comprimento do eixo, no espaço LOCAL do osso.
    pub length: f64,
    /// O alcance, como fracção do eixo já no espaço da coisa deformada (ver [`crate::SkinBone::new`]).
    pub strength: f64,
    /// Quantos sub-ossos. `1` (e `0`) ⇒ o osso de sempre. Saturado por [`segments_of`].
    pub segments: u8,
    /// A curvatura autorada. [`Bend::STRAIGHT`] ⇒ o osso de sempre, seja qual for `segments`.
    pub curve: Bend,
}

impl BoneSpec {
    /// O osso recto de sempre — o que todo osso era antes de existir curvatura.
    #[must_use]
    pub fn straight(length: f64, strength: f64) -> Self {
        Self {
            length,
            strength,
            segments: 1,
            curve: Bend::STRAIGHT,
        }
    }

    /// `true` quando este osso produz **um** sub-osso — por não ter segmentos ou por ser recto.
    #[must_use]
    pub fn is_rigid(&self) -> bool {
        segments_of(self.segments) == 1 || self.curve.is_straight()
    }
}

impl Bend {
    /// O osso recto — o ponto neutro.
    pub const STRAIGHT: Self = Self {
        inn: [0.0, 0.0],
        out: [0.0, 0.0],
    };

    /// `true` quando as duas alças estão no terço ⇒ a curva **é** o segmento, ao bit.
    #[must_use]
    pub fn is_straight(&self) -> bool {
        self.inn == [0.0, 0.0] && self.out == [0.0, 0.0]
    }
}

/// ⭐ **O TECTO DE SUB-OSSOS**, e o recurso dele é o **custo por PONTO da pele**.
///
/// [`crate::Skin::weights_at`] percorre **todos** os ossos por ponto, então `N` segmentos
/// multiplicam por `N` o custo do osso que os tem — e o recurso é o **orçamento do quadro da PELE
/// inteira**, não o do osso. Medido por [`crate::bend_tests::bend_measure_the_ceiling`] (`--release`,
/// `load 4,47`): 4 ossos, UM deles curvo, 20 000 pontos, mediana de 5 corridas.
///
/// | segmentos | ossos na pele | ms | × o recto | % de um quadro de 16,7 ms |
/// |---:|---:|---:|---:|---:|
/// | 1 | 4 | 0,337 | 1,00 | **2,0 %** |
/// | 2 | 5 | 0,426 | 1,26 | 2,6 % |
/// | 4 | 7 | 0,579 | 1,72 | 3,5 % |
/// | 8 | 11 | 0,902 | 2,68 | 5,4 % |
/// | 16 | 19 | 1,559 | 4,62 | 9,3 % |
/// | **32** | **35** | **2,989** | **8,87** | **17,9 %** |
/// | 64 | 35 | 2,984 | 8,85 | 17,9 % |
///
/// ⭐ **A última linha é a saturação a ser observada**: pedir `64` devolve `35` ossos e o mesmo
/// relógio que `32`, que é a prova de que este tecto existe no caminho de execução e não só no doc.
///
/// **Onde o `32` vem:** a `32` um osso curvo sobre uma malha generosa cabe em `17,9 %` do quadro,
/// logo **dois** deles ainda deixam metade dele livre; a `64` um só já pede `~35 %` e um par come o
/// quadro. O joelho está entre os dois, e `32` é o último degrau em que a feature é composível.
/// ⛔ **Não é o número do Blender copiado** (ele para em 32 por decisão dele) — é o que esta lei
/// mede nesta máquina, e o `64` da tabela existe para o mostrar.
///
/// ⚠️ Acima dele [`segments_of`] **satura em silêncio**, de propósito: um osso com 200 segmentos não
/// é erro do artista, é um slider que alguém arrastou até ao fim.
pub const MAX_SEGMENTS: u8 = 32;

/// Quantos sub-ossos um pedido de `segments` produz de facto: pelo menos `1`, no máximo
/// [`MAX_SEGMENTS`].
#[must_use]
pub fn segments_of(segments: u8) -> u8 {
    segments.clamp(1, MAX_SEGMENTS)
}

/// **O ponto da curva** no parâmetro `t ∈ [0,1]`, em espaço local do osso.
///
/// ⚠️ Escrito como *«a recta MAIS a correcção das alças»* e não pela base de Bernstein crua: é essa
/// forma que faz `Bend::STRAIGHT` devolver `(L·t, 0)` **ao bit** (somar `0.0` a um finito é exacto).
#[must_use]
pub fn point_at(length: f64, bend: Bend, t: f64) -> [f64; 2] {
    let s = 1.0 - t;
    let (w_inn, w_out) = (3.0 * s * s * t, 3.0 * s * t * t);
    [
        length * t + (w_inn * bend.inn[0] + w_out * bend.out[0]),
        w_inn * bend.inn[1] + w_out * bend.out[1],
    ]
}

/// ⭐⭐⭐ **O FRAME DO SUB-OSSO `k`** — o afim, em espaço LOCAL do osso, que leva o osso recto para
/// onde a curva o põe.
///
/// Ele manda o nó `k` do eixo para o nó `k` da curva e o nó `k+1` para o `k+1` ⇒ a posição é
/// contínua em cada junta, e a arte não abre fenda. ⛔ **Não é só rotação:** com rotação pura o nó
/// seguinte aterra à distância do EIXO e não à da CORDA, e cada junta ganharia um degrau do tamanho
/// da diferença.
///
/// ⚠️⚠️ **A escala é só no EIXO do osso, nunca uniforme** — e a diferença é o que o artista vê: uma
/// escala uniforme faria a arte **ENGORDAR** onde a corda é mais longa que o eixo (medido: `+87 %`
/// de espessura nas pontas de um arco com as alças a `0,6 L`). Com a escala axial a espessura é
/// preservada **ao bit** e o que estica é o comprimento, que é o que um osso a dobrar faz.
///
/// ⏳ **LIMITE DECLARADO — o esticão VARIA ao longo do osso**, porque os nós saem do parâmetro e não
/// do comprimento de arco. Medido com as alças a `0,2 L` e 8 segmentos: `1,129` nas pontas contra
/// `1,003` no meio (**13 %**); a `0,6 L` a variação vai a **87 %**. A cura publicada é a
/// **equalização por comprimento de arco** (o `equalize_cubic_bezier` da referência), que torna o
/// esticão constante — e que custa exactamente a exactidão do ponto neutro deste ficheiro, porque um
/// somatório de cordas não devolve `L` ao bit. ⇒ fica por medir num smoke, não por escrever.
#[must_use]
pub fn frame(length: f64, segments: u8, bend: Bend, k: u8) -> Xform {
    let n = segments_of(segments);
    let k = k.min(n - 1);
    let (t0, t1) = (f64::from(k) / f64::from(n), f64::from(k + 1) / f64::from(n));
    // ⚠️ `x` sai da MESMA expressão que a componente `x` da curva recta (`length * t`), e é isso que
    // faz as razões darem `1.0` e `0.0` ao bit no ponto neutro.
    let (x0, x1) = (length * t0, length * t1);
    let (c0, c1) = (point_at(length, bend, t0), point_at(length, bend, t1));
    let a = x1 - x0;
    // `(cos·s, sin·s)` = a corda por unidade de eixo: já traz a rotação E o esticão axial juntos.
    let (cs, ss) = if a != 0.0 {
        ((c1[0] - c0[0]) / a, (c1[1] - c0[1]) / a)
    } else {
        (1.0, 0.0)
    };
    // A coluna do `y` é a rotação PURA (o versor perpendicular à corda) ⇒ a espessura não muda.
    let norma = cs.hypot(ss);
    let (cos, sin) = if norma > 0.0 {
        (cs / norma, ss / norma)
    } else {
        (1.0, 0.0)
    };
    Xform([cs, ss, -sin, cos, c0[0] - cs * x0, c0[1] - ss * x0])
}

/// ⭐⭐ **A QUOTA DE UM SUB-OSSO SOBRE O EIXO** — a partição da unidade que reparte o peso **do
/// osso** pelos sub-ossos dele.
///
/// `u` é a fracção do eixo de repouso sob o ponto (o `t` de [`crate::project_to_segment`]). A quota
/// é a função-chapéu centrada no meio de cada sub-osso, achatada nas duas pontas:
///
/// ```text
/// x = clamp(u·n − ½, 0, n−1)      quota_k = max(0, 1 − |x − k|)
/// ```
///
/// ⭐ **Σ quota = 1 para todo `u`**, o que é o que mantém a força total do osso igual à de antes: um
/// osso partido em 8 não pode ganhar 8× de influência contra o vizinho. ⛔ É por isso que os
/// sub-ossos **partilham o eixo de repouso do osso inteiro** em vez de cada um ter o seu — com
/// eixos próprios o `bump` de cada um somaria, e partir um osso passaria a engordá-lo.
///
/// ⚠️ Com `n = 1` devolve `1.0` **ao bit** (e o ramo curto existe para isso ser óbvio, não porque a
/// conta geral falhasse: ela também dá `1.0` exacto).
#[must_use]
pub fn share(k: u8, n: u8, u: f64) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    let n_f = f64::from(n);
    let x = (u * n_f - 0.5).clamp(0.0, n_f - 1.0);
    (1.0 - (x - f64::from(k)).abs()).max(0.0)
}
