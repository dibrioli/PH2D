#![forbid(unsafe_code)]
//! **A LEI DO ESQUELETO** — *Linear Blend Skinning*: como um esqueleto move os pontos de uma
//! coisa presa a ele. ⛔ **Sem saber o que é essa coisa.**
//!
//! ```text
//! p' = Σ ŵ_j · (M_j · p)
//! ```
//!
//! Um ponto é movido pela mudança de referencial de **cada** osso, misturada pelos pesos dele. É a
//! *smooth skin* do Maya, o *Armature modifier* do Blender, o `Skin`/`Tendon` do Rive e a
//! *region binding* do Moho — e o LBS é a lei que toda a indústria assenta, porque a mistura de
//! afins ainda é um afim: **uma Bézier deformada continua a ser uma Bézier, exacta e editável.**
//!
//! # ⭐ Por que isto é um MÓDULO e não uma peça do vetor
//!
//! Porque nas quatro referências **um esqueleto só serve várias mídias**, e a metade que muda por
//! mídia é pequena:
//!
//! | Referência | O mesmo esqueleto deforma |
//! |---|---|
//! | Blender | malha, curva, texto, treliça **e desenho 2D** (Grease Pencil) |
//! | Moho | camada vectorial, **camada de imagem** e camada 3D |
//! | Rive | formas vectoriais **e** imagens (por malha) |
//! | Spine | malhas de imagem |
//!
//! ⇒ esta crate responde a *«para onde vai um PONTO»*, e cada mídia responde, do lado dela, a
//! *«o que é um ponto aqui»*. O 1.º cliente é a [`ph2d-vec-skin`], onde um vértice tem **três**
//! pontos (âncora e as duas alças) e cada um pesa pela posição DELE — o `CubicWeight` do Rive.
//! ⚠️ E é literalmente o que o Blender faz: o *Armature modifier* é um só, e o Grease Pencil tem a
//! implementação dele à parte porque *o que é um ponto* ali é outra coisa.
//!
//! # ⭐ O repouso é a IDENTIDADE, e cai da álgebra
//!
//! `M_j = S⁻¹ ∘ B_j ∘ rest_j⁻¹`, e `rest_j` **é** `S ∘ B_j` no instante em que se ligou ⇒ parado no
//! repouso todo `M_j` é a identidade, e `Σ ŵ_j · p = p` **seja qual for o peso**. A lei da casa
//! (*todo motor novo é no-op no ponto neutro*) não precisa aqui de uma guarda escrita à mão.

/// O afim aparece na assinatura de [`SkinBone::new`], então ele é re-exportado: um cliente que
/// tivesse de o alcançar por outra crate para chamar esta ficaria a depender de duas coisas para
/// usar uma.
pub use ph2d_affine::Xform;

/// **Um osso, do ponto de vista da pele** — tudo já no espaço LOCAL da coisa deformada.
///
/// ⚠️ Ele não sabe o que é uma entidade, um `Transform` ou uma hierarquia: a shell resolve a
/// cinemática (que é a propagação de `Transform` que a casa já corre) e entrega o resultado. É o
/// que mantém esta crate uma folha testável sem mundo ECS nenhum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkinBone {
    /// A origem do eixo de repouso, no espaço da coisa deformada.
    pub rest_a: [f64; 2],
    /// A ponta do eixo de repouso, no espaço da coisa deformada.
    pub rest_b: [f64; 2],
    /// O raio de influência, nas unidades da coisa deformada. `0` ⇒ este osso só ganha um ponto
    /// pelo desempate do órfão (ver [`Skin::point`]).
    pub radius: f64,
    /// `M_j` — leva um ponto do repouso para onde ESTE osso o quer agora.
    pub pose: Xform,
    /// ⭐ **Qual sub-osso de quantos**, para um osso que DOBRA ([`bend`]). `(0, 1)` ⇒ o osso
    /// inteiro, que é o que um osso recto é — e é o que [`SkinBone::new`] escreve.
    ///
    /// ⚠️ **Os sub-ossos de um osso partilham `rest_a`/`rest_b`/`radius`** (o eixo e o alcance são
    /// do OSSO) e distinguem-se só pela `pose` e por este par: é isso que faz a força total do osso
    /// não crescer com o número de segmentos (ver [`bend::share`]).
    ///
    /// ⚠️ **Eles são CONSECUTIVOS na pele**, e o desempate do órfão conta com isso para achar o
    /// grupo a partir de um membro — [`SkinBone::bent`] é a única porta que os produz.
    pub sub: (u8, u8),
    /// ⭐⭐⭐ **DE QUE OSSO AUTORADO este sub-osso é um pedaço** — o índice na lista de tendões que
    /// o chamador resolveu.
    ///
    /// ⚠️⚠️ **Ele existe porque os pesos passaram a poder ser GUARDADOS** ([`Skin::point_with`]).
    /// Enquanto a lei era derivada de uma distância, a pele não precisava de saber a que osso do
    /// ARTISTA cada pose correspondia: cada uma calculava o peso dela sozinha. Com os pesos do
    /// padrão-ouro resolvidos no bind, a tabela é por **osso autorado** e a pele tem `N` poses por
    /// osso — alguém tem de dizer qual é qual.
    ///
    /// ⛔ **É um campo do sub-osso, e não um vector paralelo dentro da [`Skin`]**, pela mesma razão
    /// que o [`SkinBone::sub`] o é: *«que pedaço de que osso sou eu»* é uma propriedade desta pose,
    /// e uma lista ao lado pode dessincronizar-se da outra numa edição.
    ///
    /// ⚠️ **Um osso saltado NÃO desloca este índice.** A resolução salta tendões cujo osso foi
    /// apagado; se isto fosse *«a minha posição na pele»* a tabela guardada passaria a apontar para
    /// o osso errado no instante em que alguém apagasse um osso — e a arte saltava sem nada
    /// reprovar. ⇒ é o índice do **tendão**, sempre.
    pub tendon: u32,
}

impl SkinBone {
    /// **A porta única da composição** — `S⁻¹ ∘ B ∘ rest⁻¹`, na ordem certa.
    ///
    /// ⚠️ **A ordem é o defeito clássico** e por isso ela vive numa função só: `Xform::then(outer)`
    /// aplica `self` PRIMEIRO, então a leitura da esquerda para a direita é o inverso da fórmula.
    /// Escrita em dois sítios, ela diverge no primeiro que alguém refactorar.
    ///
    /// `None` quando o repouso é singular (uma forma ou um osso escalados a zero no bind) — o
    /// chamador salta o osso, e os outros renormalizam sozinhos.
    #[must_use]
    pub fn new(
        rest: Xform,
        length: f64,
        strength: f64,
        bone_world: Xform,
        shape_world_inv: Xform,
    ) -> Option<Self> {
        let rest_inv = rest.inverse()?;
        let rest_a = rest.apply([0.0, 0.0]);
        let rest_b = rest.apply([length, 0.0]);
        // ⭐ **O raio mede-se no eixo JÁ no espaço da forma**, e não como `strength · length`: assim
        // ele carrega a escala do bind de graça, e um esqueleto ligado a uma forma escalada 3×
        // alcança 3× mais longe — que é o que o artista vê.
        let span = (rest_b[0] - rest_a[0]).hypot(rest_b[1] - rest_a[1]);
        Some(Self {
            rest_a,
            rest_b,
            radius: (span * strength).max(0.0),
            pose: rest_inv.then(&bone_world).then(&shape_world_inv),
            sub: (0, 1),
            // ⚠️ **`0` é o neutro honesto de um osso SOZINHO**, que é o que esta porta constrói —
            // quem tem uma lista atribui o índice pela [`SkinBone::bent`], a única porta do produto.
            tendon: 0,
        })
    }

    /// ⭐⭐⭐ **O MESMO OSSO, PARTIDO POR UMA CURVA** — os `N` sub-ossos de um *bendy bone*,
    /// empurrados para `out` **em ordem** (o desempate do órfão conta com isso).
    ///
    /// ⚠️⚠️ **O PONTO NEUTRO É BYTE-IDÊNTICO, e por dois caminhos que se encontram:** com
    /// `segments <= 1` **ou** com a curvatura recta esta função empurra exactamente o que a
    /// [`SkinBone::new`] devolve, sem passar pela fábrica de frames. *Uma recta não precisa de `N`
    /// ossos para a desenhar* — e colapsá-la é o que separa «o caminho antigo continua igual» de
    /// «o caminho antigo continua parecido».
    ///
    /// ⚠️ A pose de cada sub-osso é `S⁻¹ ∘ B ∘ F_k ∘ rest⁻¹`: o frame entra **do lado do osso**,
    /// entre o repouso e o mundo, porque ele é um afim do espaço LOCAL do osso. Pô-lo do lado de
    /// fora dobraria a arte no espaço da forma, e a dobra deixaria de seguir o osso quando ele
    /// rodasse.
    pub fn bent(
        rest: Xform,
        spec: bend::BoneSpec,
        bone_world: Xform,
        shape_world_inv: Xform,
        tendon: u32,
        out: &mut Vec<Self>,
    ) {
        let Some(base) = Self::new(
            rest,
            spec.length,
            spec.strength,
            bone_world,
            shape_world_inv,
        ) else {
            return;
        };
        let base = Self { tendon, ..base };
        if spec.is_rigid() {
            out.push(base);
            return;
        }
        let n = bend::segments_of(spec.segments);
        // O inverso já existiu dentro da `new` — se ela devolveu algo, este `else` é inalcançável.
        let Some(rest_inv) = rest.inverse() else {
            return;
        };
        // ⚠️ **UMA chamada para os `n` frames**, e não `n` chamadas ao [`bend::frame`]: desde a
        // equalização por arco (2026-09-16) cada frame precisa da tabela de comprimento, e pedi-la
        // por sub-osso construiria `n` tabelas para as mesmas `n` respostas.
        for (k, frame) in bend::frames(spec).into_iter().enumerate() {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "n <= MAX_SEGMENTS = 32, logo o índice cabe num u8"
            )]
            let k = k as u8;
            out.push(Self {
                pose: rest_inv
                    .then(&frame)
                    .then(&bone_world)
                    .then(&shape_world_inv),
                sub: (k, n),
                ..base
            });
        }
    }
}

/// **A pele de UMA coisa** — os ossos a que ela está presa, já resolvidos para este quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct Skin {
    bones: Vec<SkinBone>,
}

impl Skin {
    /// `None` sem osso nenhum — uma pele vazia não é a identidade, é a **ausência** de pele, e o
    /// chamador tem de deixar a forma em paz em vez de a passar por um mapa que não existe.
    #[must_use]
    pub fn new(bones: Vec<SkinBone>) -> Option<Self> {
        (!bones.is_empty()).then_some(Self { bones })
    }

    /// Quantos ossos esta pele tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bones.len()
    }

    /// Sempre `false` — [`Skin::new`] recusa a pele vazia. Existe para o `clippy::len_without_is_empty`
    /// e para dizer isso em voz alta.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }

    /// Os ossos, para quem quer medir (as sondas e os gates).
    #[must_use]
    pub fn bones(&self) -> &[SkinBone] {
        &self.bones
    }

    /// Um rascunho de pesos do tamanho certo — o que [`Skin::point`] pede emprestado.
    ///
    /// Existe para o chamador não ter de saber que o comprimento é `len()`: um `vec![0.0; n]` com
    /// o `n` errado é um `debug_assert` em debug e pesos lidos fora do sítio em release.
    #[must_use]
    pub fn scratch(&self) -> Vec<f64> {
        vec![0.0; self.bones.len()]
    }

    /// **Os pesos NORMALIZADOS de um ponto**, escritos em `w` (que tem de ter `len()` casas).
    ///
    /// Devolve `false` quando o ponto é **órfão** — fora do raio de todo osso. Nesse caso `w` sai
    /// com `1` no osso mais próximo e `0` no resto: o *point binding* do Moho, e a razão está no
    /// [doc 47 §2.4](../../../docs/Vector%20Module/47_o_desenho_ganha_ossos.md) — com suporte
    /// infinito (`1/d²`) um ponto longe de tudo passa a seguir a **média do esqueleto**, e a aba de
    /// um chapéu atrasa-se atrás da cabeça.
    pub fn weights_at(&self, p: [f64; 2], w: &mut [f64]) -> bool {
        debug_assert_eq!(w.len(), self.bones.len());
        let mut soma = 0.0;
        let (mut perto, mut perto_d2) = (0usize, f64::INFINITY);
        for (i, b) in self.bones.iter().enumerate() {
            let (u, d2) = project_to_segment(p, b.rest_a, b.rest_b);
            if d2 < perto_d2 {
                (perto, perto_d2) = (i, d2);
            }
            // O bump `(1 − x²)²` com `x = d/r`: `1` no eixo, **`0` E derivada `0`** na borda. É a
            // continuidade C¹ que faz um ponto atravessar a fronteira de influência sem estalo.
            let peso = if b.radius > 0.0 {
                let x2 = d2 / (b.radius * b.radius);
                if x2 < 1.0 {
                    let t = 1.0 - x2;
                    t * t
                } else {
                    0.0
                }
            } else {
                0.0
            };
            // ⭐ A QUOTA do sub-osso. Num osso recto ela é `1.0` ao bit, logo `peso * 1.0 == peso` e
            // este caminho continua byte-idêntico ao que existia antes de os ossos dobrarem.
            let peso = peso * bend::share(b.sub.0, b.sub.1, u);
            w[i] = peso;
            soma += peso;
        }
        if soma > 0.0 {
            for v in w.iter_mut() {
                *v /= soma;
            }
            return true;
        }
        // ⛔ **Nada de podar pesos pequenos.** Uma catraca por baixo (`w < 1/256 ⇒ 0`) devolveria
        // exactamente o estalo que o bump C¹ existe para evitar: o osso saltaria de `1/256` para
        // zero no meio do movimento. O suporte já é finito — não há cauda para cortar.
        let vencedor = &self.bones[perto];
        let n = usize::from(vencedor.sub.1);
        if n <= 1 {
            for (i, v) in w.iter_mut().enumerate() {
                *v = f64::from(u8::from(i == perto));
            }
            return false;
        }
        // ⚠️ **Um osso CURVO ganha o desempate inteiro, e reparte-o pelos sub-ossos dele.** Dar o
        // ponto ao sub-osso mais próximo em vez de ao grupo prenderia um órfão à ponta errada da
        // curva: eles partilham o eixo de repouso, logo a distância é a MESMA nos `n` e quem
        // decidisse seria a ordem da lista.
        let base = perto - usize::from(vencedor.sub.0);
        let (u, _) = project_to_segment(p, vencedor.rest_a, vencedor.rest_b);
        for (i, v) in w.iter_mut().enumerate() {
            *v = match i.checked_sub(base) {
                Some(k) if k < n => {
                    bend::share(u8::try_from(k).unwrap_or(u8::MAX), vencedor.sub.1, u)
                }
                _ => 0.0,
            };
        }
        false
    }

    /// Onde este ponto vai parar. `w` é o rascunho de [`Skin::scratch`], reusado por ponto para a
    /// pele inteira não alocar uma vez por ponto.
    #[must_use]
    pub fn point(&self, p: [f64; 2], w: &mut [f64]) -> [f64; 2] {
        self.weights_at(p, w);
        self.blend(p, w)
    }

    /// ⭐⭐⭐ **OS PESOS DE UM OSSO AUTORADO, REPARTIDOS PELOS SUB-OSSOS DELE** — escritos em `w`.
    ///
    /// `por_tendao[j]` é a fracção do ponto que pertence ao osso **que o artista desenhou**; esta
    /// função traduz isso para a pele resolvida, que tem `N` poses por osso quando ele dobra.
    ///
    /// ⭐ **A partição é a MESMA lei de sempre** ([`bend::share`]) — um osso recto devolve `1.0` ao
    /// bit, logo o caminho de um rig sem curvatura é a cópia directa da tabela. *A lei dos pesos
    /// mudou; a lei de como um osso curvo os reparte não.*
    ///
    /// ⚠️ **Um tendão cujo osso foi APAGADO não tem pose nenhuma na pele, e o peso dele some** —
    /// é por isso que a soma é renormalizada aqui em vez de se confiar na tabela: apagar um osso
    /// não pode apagar a arte, que é a mesma decisão que o [`Skin::weights_at`] já tomava.
    ///
    /// ⛔ Uma tabela curta (ou vazia) devolve tudo a zero, e a renormalização não tem por onde
    /// pegar ⇒ **o ponto fica onde está**. É a leitura honesta de *«esta malha não traz pesos»*, e
    /// nunca um salto para a origem.
    pub fn weights_from(&self, p: [f64; 2], por_tendao: &[f64], w: &mut [f64]) {
        debug_assert_eq!(w.len(), self.bones.len());
        let mut soma = 0.0;
        for (i, b) in self.bones.iter().enumerate() {
            let quota = if b.sub.1 <= 1 {
                1.0
            } else {
                let (u, _) = project_to_segment(p, b.rest_a, b.rest_b);
                bend::share(b.sub.0, b.sub.1, u)
            };
            let peso = por_tendao.get(b.tendon as usize).copied().unwrap_or(0.0) * quota;
            w[i] = peso;
            soma += peso;
        }
        if soma > 0.0 {
            for v in w.iter_mut() {
                *v /= soma;
            }
        }
    }

    /// ⭐⭐⭐ **[`Skin::point`] COM OS PESOS JÁ SABIDOS** — a porta do padrão-ouro.
    ///
    /// ⚠️⚠️ **Ela existe porque os *Bounded Biharmonic Weights* NÃO são função de uma posição.**
    /// Eles são a solução de um problema variacional sobre a arte inteira, resolvido uma vez ao
    /// prender; perguntá-los ponto a ponto por quadro seria re-resolver o problema por pixel. ⇒ o
    /// consumidor traz a tabela, e esta função só a reparte e mistura.
    ///
    /// ⛔ **A mistura é a MESMA** ([`Skin::blend`]), e é isso que garante que as duas leis de peso
    /// entregam geometria pela mesma aritmética — *duas misturas seriam duas artes*.
    #[must_use]
    pub fn point_with(&self, p: [f64; 2], por_tendao: &[f64], w: &mut [f64]) -> [f64; 2] {
        self.weights_from(p, por_tendao, w);
        self.blend(p, w)
    }

    /// ⭐⭐⭐ **OS PESOS DE UM PONTO, COM AS CORRECÇÕES DO ARTISTA** — a porta ÚNICA por onde as
    /// duas leis de peso passam.
    ///
    /// `guardados` escolhe a lei: `Some(tabela)` é o padrão-ouro ([`Skin::weights_from`]), `None` é
    /// a euclidiana ([`Skin::weights_at`]). ⛔ Com `correcoes` vazio o resultado é **byte-idêntico**
    /// ao das duas — o laço nem corre e nada é renormalizado uma segunda vez.
    ///
    /// ⚠️ **Ela existe para a correcção não ter de ser escrita duas vezes**, uma por lei: o artista
    /// corrige *aquele ponto*, e de que lei veio o peso que ele está a corrigir não é pergunta dele.
    pub fn weights_corrected(
        &self,
        p: [f64; 2],
        guardados: Option<&[f64]>,
        w: &mut [f64],
        correcoes: &[Correccao],
    ) {
        match guardados {
            Some(t) => self.weights_from(p, t, w),
            None => {
                self.weights_at(p, w);
            }
        }
        self.corrige(p, w, correcoes);
    }

    /// [`Skin::weights_corrected`] seguido da mistura — a porta que uma mídia chama por ponto.
    #[must_use]
    pub fn point_corrected(
        &self,
        p: [f64; 2],
        guardados: Option<&[f64]>,
        w: &mut [f64],
        correcoes: &[Correccao],
    ) -> [f64; 2] {
        self.weights_corrected(p, guardados, w, correcoes);
        self.blend(p, w)
    }

    /// ⭐⭐⭐ **A CORRECÇÃO À MÃO** — o artista soma (ou tira) peso a um osso, num sítio.
    ///
    /// # ⚠️ Porque ela é uma MANCHA no espaço e não uma tabela por vértice
    ///
    /// *Uma tabela indexada por ordem de varredura é o vector paralelo que o
    /// `VecVertex::corner_radius` proíbe por escrito*: dezenas de operações inserem, apagam,
    /// invertem e soldam vértices, e cada uma teria de se lembrar de mexer nela. Uma mancha é
    /// **ancorada na geometria** — ela diz *«aqui»*, e continua a dizer «aqui» depois de o artista
    /// mexer no desenho.
    ///
    /// # ⚠️ O bump é o MESMO da lei euclidiana
    ///
    /// `(1 − x²)²` com `x = d/raio`: `1` no centro, **`0` E derivada `0`** na borda. ⛔ Uma queda
    /// linear deixaria uma aresta visível no sítio exacto onde o artista pintou — o estalo que a
    /// continuidade C¹ da casa existe para não ter.
    ///
    /// # ⚠️ O sujeito é o TENDÃO e não o sub-osso
    ///
    /// O artista corrige *o osso que ele desenhou*; um osso que dobra tem `N` poses, e a correcção
    /// reparte-se por elas pela **mesma** lei que já reparte o peso ([`bend::share`]). ⛔ Corrigir
    /// um sub-osso seria expor ao artista uma divisão que ele não fez.
    ///
    /// ⚠️ **Renormaliza no fim, e só se alguma mancha alcançou o ponto** — senão isto não seria um
    /// no-op sobre a lei que já normalizou.
    fn corrige(&self, p: [f64; 2], w: &mut [f64], correcoes: &[Correccao]) {
        if correcoes.is_empty() {
            return;
        }
        let mut mexeu = false;
        for c in correcoes {
            if c.raio <= 0.0 || !c.delta.is_finite() {
                continue;
            }
            let d2 = (p[0] - c.centro[0]).powi(2) + (p[1] - c.centro[1]).powi(2);
            let x2 = d2 / (c.raio * c.raio);
            if x2 >= 1.0 {
                continue;
            }
            let t = 1.0 - x2;
            let bump = t * t;
            for (i, b) in self.bones.iter().enumerate() {
                if b.tendon != c.tendon {
                    continue;
                }
                w[i] = (c.delta * bump)
                    .mul_add(self.quota(b, p), w[i])
                    .clamp(0.0, 1.0);
                mexeu = true;
            }
        }
        if !mexeu {
            return;
        }
        let soma: f64 = w.iter().sum();
        // ⛔ **Soma zero deixa `w` como está** — o artista tirou tudo, e a mistura devolve o ponto
        // INTACTO (a lei do [`Skin::blend`]). ⚠️ Dividir por zero daria `NaN` em toda a arte.
        if soma > 0.0 {
            for v in w.iter_mut() {
                *v /= soma;
            }
        }
    }

    /// A fracção deste sub-osso no osso autorado a que ele pertence — `1.0` ao bit num osso recto.
    ///
    /// ⚠️ **Uma função e não três cópias:** a mesma conta vive no [`Skin::weights_at`], no
    /// [`Skin::weights_from`] e agora na correcção, e três cópias divergiriam no dia em que a
    /// repartição mudasse — com o sintoma a ser um osso curvo a corrigir-se de outra maneira do
    /// que se pesa.
    fn quota(&self, b: &SkinBone, p: [f64; 2]) -> f64 {
        if b.sub.1 <= 1 {
            return 1.0;
        }
        let (u, _) = project_to_segment(p, b.rest_a, b.rest_b);
        bend::share(b.sub.0, b.sub.1, u)
    }

    /// A mistura `Σ ŵ_j · (M_j · p)` — a única aritmética que move um ponto, seja de onde vierem
    /// os pesos.
    ///
    /// ⚠️ **Com todos os pesos a zero devolve o ponto INTACTO** (e não a origem): é a resposta
    /// certa a *«nenhum osso reclama este ponto»*, e o que a torna segura é que as duas portas de
    /// peso renormalizam antes de chegar aqui.
    /// ⭐⭐⭐ **A MISTURA COM OS PESOS JÁ SABIDOS** — `Σ ŵ_i · (M_i · p)`.
    ///
    /// ⚠️ **Ela é pública porque há uma mídia em que o peso NÃO é do ponto que se está a mover.**
    /// Num caminho vectorial o peso é do **NÓ**, e a âncora e as duas alças dele movem-se pelos
    /// pesos da âncora ([`ph2d_vec_skin`], ordem do dono de 2026-09-19) — quem chama já tem o `w` e
    /// só precisa de mover um ponto por ele. ⛔ As portas que **calculam** o peso
    /// ([`Skin::point`], [`Skin::point_with`], [`Skin::point_corrected`]) continuam a ser o caminho
    /// normal: esta só serve a quem tem uma resposta a *«de quem é este peso»* que não é «do ponto».
    ///
    /// ⛔ **Soma zero devolve o ponto INTACTO** e nunca a origem: é a leitura honesta de *«nenhum
    /// osso manda aqui»*, e é o que impede uma arte de saltar para o zero do mundo quando o artista
    /// tira todo o peso com o pincel.
    ///
    /// ⛔⛔⛔ **ELA JÁ NÃO É O CAMINHO DO PRODUTO, e desde 2026-09-19 chama-se `_linear` por isso.**
    /// Interpolar POSIÇÕES dá a **CORDA** do arco: um ponto entre dois ossos que divergem `θ` é
    /// puxado para `cos(θ/2)` da distância à junta, e a arte colapsa (medido: `0,5000` a `120°`
    /// sobre um raio de `1`). Quem move um ponto chama a [`Skin::blend`], que roda em torno da
    /// JUNTA — ver [`crate::centro`]. ⚠️ Esta fica porque a lei nova a usa para a **translação** do
    /// centro, e porque é o CONTROLO de todos os gates que medem a cura.
    #[must_use]
    pub fn blend_linear(&self, p: [f64; 2], w: &[f64]) -> [f64; 2] {
        let mut out = [0.0, 0.0];
        let mut soma = 0.0;
        for (b, &peso) in self.bones.iter().zip(w.iter()) {
            if peso == 0.0 {
                continue;
            }
            let q = b.pose.apply(p);
            out[0] += peso * q[0];
            out[1] += peso * q[1];
            soma += peso;
        }
        if soma == 0.0 { p } else { out }
    }

    /// **A PORTA DE UMA MÍDIA** — deforma uma sequência de pontos em lugar, com um rascunho só.
    ///
    /// ⚠️ É o que cada mídia chama depois de responder à pergunta dela (*«o que é um ponto aqui»*):
    /// o vetor passa por aqui as três metades de cada vértice, e um cliente raster passará os
    /// vértices de uma malha. ⛔ Um cliente que faça o laço à mão paga o `scratch` por ponto —
    /// esta função existe para isso não ser possível por descuido.
    pub fn deform_points<'a>(&self, pts: impl IntoIterator<Item = &'a mut [f64; 2]>) {
        let mut w = self.scratch();
        for p in pts {
            *p = self.point(*p, &mut w);
        }
    }
}

/// ⭐⭐⭐ **UMA CORRECÇÃO DE PESO FEITA À MÃO** — uma mancha no espaço que soma (ou tira) peso a um
/// osso, onde a conta automática errou.
///
/// ⚠️ Ver [`Skin::corrige`] para o mecanismo, o bump e a razão de ela ser uma MANCHA e não uma
/// tabela por vértice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correccao {
    /// ⭐ **O TENDÃO** — o índice do osso **que o artista desenhou** na lista que o chamador
    /// resolveu, o mesmo espaço do [`SkinBone::tendon`].
    ///
    /// ⚠️ **E não o sub-osso:** o artista corrige o osso que ele vê, e a repartição por sub-ossos
    /// de um osso que dobra é feita pela lei, não por ele.
    pub tendon: u32,
    /// O centro da mancha, **no espaço da coisa deformada** (o mesmo dos eixos de repouso) — logo
    /// ela fica onde o artista a pôs, mesmo que ele mexa no desenho depois.
    pub centro: [f64; 2],
    /// O raio, nas unidades da coisa deformada. `<= 0` ⇒ a mancha não alcança nada.
    pub raio: f64,
    /// Quanto somar ao peso deste osso no CENTRO da mancha. Negativo TIRA — **o sinal é a
    /// direcção**, e a lei nunca pergunta por um modo.
    ///
    /// ⛔⛔ **A frase que estava aqui — *«não há um segundo modo «apagar» a lembrar»* — era sobre a
    /// TELA e foi REVOGADA pelo dono** (2026-09-19: *«no lugar de valores negativos em Brush
    /// Strength prefiro botões Add e Subtract»*). ⚠️ *O que ele revogou foi a superfície, não o
    /// dado:* a força é uma MAGNITUDE no painel e a direcção sai de dois botões, e é o
    /// `WeightDirection::delta` que volta a juntar as duas num sinal antes de chegar aqui. **Este
    /// campo não mudou um bit.**
    pub delta: f64,
}

/// Distância AO QUADRADO de `p` ao segmento `a..b` (a raiz nunca é precisa: a lei compara com
/// `r²` e a mistura só usa razões).
#[must_use]
pub fn dist2_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    project_to_segment(p, a, b).1
}

/// ⭐⭐ **A distância AO QUADRADO de `p` a uma POLILINHA** — o corpo de um osso que dobra.
///
/// ⚠️ **Ela existe para o DEDO ler a mesma linha que o desenho pinta.** Com dois nós devolve
/// exactamente o [`dist2_to_segment`] daquele troço, ao bit — logo um osso recto continua a ser
/// agarrado como sempre foi. ⛔ *Desenhar por um mapa e agarrar por outro é um controlo morto sob o
/// dedo*, e num osso arqueado a diferença entre a corda e o corpo é o osso inteiro.
#[must_use]
pub fn dist2_to_polyline(p: [f64; 2], pts: &[[f64; 2]]) -> f64 {
    pts.windows(2)
        .map(|w| dist2_to_segment(p, w[0], w[1]))
        .fold(f64::INFINITY, f64::min)
}

/// ⭐ **A PROJECÇÃO de `p` no segmento `a..b`** — `(fracção do eixo, distância ao quadrado)`.
///
/// ⚠️ **Ela existe porque o osso que DOBRA precisa dos dois números**, e o [`dist2_to_segment`]
/// deitava fora um deles depois de o calcular. *Uma segunda função a recalcular a mesma projecção
/// divergiria desta no primeiro `clamp` que alguém mexesse* — por isso a antiga passou a ser esta,
/// com a componente que não usa descartada.
#[must_use]
pub fn project_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> (f64, f64) {
    let (abx, aby) = (b[0] - a[0], b[1] - a[1]);
    let (apx, apy) = (p[0] - a[0], p[1] - a[1]);
    let len2 = abx * abx + aby * aby;
    // Osso de comprimento zero ⇒ a distância é ao PONTO. (Um osso assim não influencia ninguém
    // pelo raio — só pode ganhar o desempate do órfão —, e aqui isso resolve-se sozinho.)
    let t = if len2 > 0.0 {
        ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (apx - t * abx, apy - t * aby);
    (t, dx * dx + dy * dy)
}

/// ⭐ **A régua da DOBRA** — quanto da arte a pele vira do avesso. Crate-irmã do [`Skin`] por
/// responsabilidade: *onde um ponto vai parar* e *o mapa continua a ser injectivo* são duas
/// perguntas, e a segunda nunca tinha instrumento.
/// ⭐⭐⭐ **O OSSO QUE DOBRA** — a fábrica de sub-ossos de um *bendy bone*. Crate-irmã da [`Skin`]
/// por responsabilidade: *que poses existem* e *como elas se misturam* são duas perguntas, e a
/// segunda não muda uma linha por a primeira passar a dar `N` respostas.
/// ⭐⭐⭐ **O CENTRO DE ROTAÇÃO** — a cura do entalhe do cotovelo.
pub mod centro;
pub mod bend;
/// ⭐ Os gates do osso que dobra.
#[cfg(test)]
#[path = "bend_tests.rs"]
mod bend_tests;
pub mod fold;
/// ⭐ Os gates e a sonda da régua da dobra.
#[cfg(test)]
#[path = "fold_tests.rs"]
mod fold_tests;
mod reach;
/// ⭐ Os gates do **osso inteligente** — o mesmo corte: *que instante de uma acção este ângulo
/// pede* é a lei do Smart Bone, não a do alcance.
#[cfg(test)]
#[path = "reach_action_tests.rs"]
mod reach_action_tests;
/// ⭐ Os gates do **LADO da dobra numa corrente longa** — o mesmo corte, e ele nasceu de um report
/// do dono (*«IK Bend só funciona se o IK Chain for 2»*): *para que lado dobra uma corrente de N*
/// não é a mesma pergunta que *onde ela chega*.
#[cfg(test)]
#[path = "reach_chain_side_tests.rs"]
mod reach_chain_side_tests;
/// ⭐ **Uma passagem da corrente** — irmão do [`reach`] pelo tecto de LOC, cortado por
/// responsabilidade: *que pose se pede* e *como a corrente varre* são duas perguntas.
mod reach_fabrik;
/// ⭐ Os gates da **parede de uma junta** — irmão do [`reach_tests`] pelo teto de LOC, cortado por
/// RESPONSABILIDADE: *até onde este osso dobra* não é uma pergunta sobre alcance.
#[cfg(test)]
#[path = "reach_limit_tests.rs"]
mod reach_limit_tests;
/// ⭐ **O modo MISTO** — irmão do [`reach`] pelo tecto de LOC, cortado por responsabilidade: ele
/// não é uma variante do FABRIK, é outro solver (descida junta a junta com a parede de cada lado).
mod reach_mixed;
/// ⭐ **Como uma corrente ganha e guarda um LADO** — irmão do [`reach`] pelo tecto de LOC,
/// cortado por responsabilidade: *onde ela chega* e *para que lado dobra* são duas perguntas.
mod reach_side;
#[cfg(test)]
#[path = "reach_tests.rs"]
mod reach_tests;
pub use reach::{
    BendSide, DEFAULT_ITERATIONS, FULL_TURN, MAX_ITERATIONS, Reach, action_time, bend_side_of,
    blend_angle, clamp_to_limit, dominant_side, reach, side_of, softened_distance, wrap_pi,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "correccao_tests.rs"]
mod correccao_tests;
