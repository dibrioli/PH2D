//! **KELVINLETS REGULARIZADOS** — de Goes & James, *Regularized Kelvinlets:
//! Sculpting Brushes based on Fundamental Solutions of Elasticity*, SIGGRAPH
//! 2017 (Pixar).
//!
//! É o **l-mode** da família que AGARRA: em vez de um peso escalar multiplicado
//! por um vetor de gesto, o barro responde com a **solução fundamental da
//! elasticidade linear** — o campo de deslocamento de um sólido infinito puxado
//! num ponto. Plano: [`docs/3D/21_plano_modos_e_ferramentas.md`] §7 W5.
//!
//! # ⚠️ O que um campo FAZ que uma curva de falloff não pode fazer
//!
//! A lei do `S` é `base + gesto · w`, com `w` escalar: **todo vértice da pegada
//! anda na MESMA direção**, e o falloff só decide quanto. Um Kelvinlet é
//! **anisotrópico** — o termo `(r·f) r` faz o barro à FRENTE do puxão andar mais
//! que o barro ao LADO dele, que é o que um material elástico faz e o que uma
//! curva não tem como exprimir por não ter para onde apontar.
//!
//! ⇒ **O campo É o falloff.** Ele não multiplica a curva: ele a substitui. Quem
//! o consome tem de carregar o peso no ALVO (`unit_accum`), e a razão está
//! medida no doc do [`crate::Verb::Move`] — aplicar o perfil duas vezes deixou o
//! Grab movendo `0,12226` onde a referência move `0,22500`.
//!
//! # As quatro leis, e de onde elas vêm
//!
//! Todas saem de **uma** função ([`raw_grab`]) e da derivada direcional dela
//! ([`raw_affine`]) — não são quatro kernels, são um kernel e o gradiente dele:
//!
//! - **grab** (eq. 5 do paper): `u(r) = A(rε)·f + B(rε)·(r·f)·r`;
//! - **afim**: `u_F(r) = −(d·∇) u_grab(r; f)` com `F = f dᵀ`, que dá twist
//!   (`F` antissimétrica), scale (`F = s·I`) e pinch (`F` simétrica sem traço).
//!
//! ⚠️ **O `ε` é o que REGULARIZA:** `rε = √(|r|² + ε²)` nunca é zero, então o
//! campo é liso no centro em vez de singular — é isso, e só isso, que separa
//! este kernel do Kelvin de 1848 e o torna utilizável como pincel.
//!
//! # ⚠️ O material só tem UM número, e ele não é de gosto
//!
//! Depois da normalização o módulo de cisalhamento **cancela** (numerador e
//! denominador são homogéneos de grau 1 em `(a, b)`), então só a razão `b/a`
//! sobrevive — ou seja, só o **coeficiente de Poisson** [`POISSON`]. Ver o
//! doc dele: o valor foi MEDIDO, e a primeira escolha que eu fiz (`ν = 1/2`,
//! *"barro é incompressível"*) é **inexprimível** — ela zera o modo de escala.
//!
//! # ⚠️ O campo é ILIMITADO, e a pegada do motor não é
//!
//! Um Kelvinlet decai como `1/r`: a **um raio de pincel** do centro ele ainda
//! vale ~53 % do deslocamento do bico (medido em
//! `tests/measure_kelvinlet.rs`). Truncá-lo na pegada seria um degrau enorme na
//! borda do cursor. A cura é do próprio paper (§*multi-scale brushes*) e é o que
//! o [`Scales`] carrega: uma combinação de Kelvinlets de raios diferentes cujos
//! termos de campo distante **se cancelam**.

/// **Quantas escalas o campo combina** — o `1/r` do paper, e as duas somas que
/// o cancelam.
///
/// ⚠️ **Não é uma preferência: é a única forma de um campo com suporte infinito
/// caber numa pegada finita.** As condições são aritméticas e estão nos gates —
/// `Σ wᵢ = 0` mata o termo `1/r`, e `Σ wᵢ εᵢ² = 0` mata o `1/r³` que sobra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub enum Scales {
    /// Um Kelvinlet só. Decai como `1/r` — o campo do paper, sem correção.
    Mono,
    /// Dois raios (`ε`, `2ε`) com pesos `+1, −1`. Decai como `1/r³`.
    Bi,
    /// Três raios (`ε`, `2ε`, `3ε`) com pesos `+5, −8, +3`. Decai como `1/r⁵`.
    ///
    /// ⚠️ **É o default porque a MEDIÇÃO o escolheu**, não porque é o maior:
    /// ver a tabela do resíduo de borda abaixo, em [`Scales::taps`].
    #[default]
    Tri,
}

impl Scales {
    /// Os `(peso, múltiplo de ε)` desta família.
    ///
    /// ⚠️ **Os múltiplos são `1, 2, 3` e os pesos saem de um SISTEMA**, não de
    /// uma tabela decorada: `5 − 8 + 3 = 0` e `5·1 − 8·4 + 3·9 = 0`. Trocar um
    /// múltiplo obriga a re-resolver o sistema, e o gate
    /// `the_far_field_cancels_by_construction` é quem recusa um par que não o
    /// satisfaz.
    #[must_use]
    pub const fn taps(self) -> &'static [(f32, f32)] {
        match self {
            Self::Mono => &[(1.0, 1.0)],
            Self::Bi => &[(1.0, 1.0), (-1.0, 2.0)],
            Self::Tri => &[(5.0, 1.0), (-8.0, 2.0), (3.0, 3.0)],
        }
    }
}

/// **O coeficiente de Poisson do barro — `1/2`, o material INCOMPRESSÍVEL.**
///
/// ⚠️ **Esta constante tem uma história de três atos, e o meio dela é uma
/// afirmação minha que a álgebra derrubou e a álgebra devolveu.**
///
/// 1. **O argumento físico:** barro é incompressível ⇒ `ν = 1/2`.
/// 2. **A refutação:** o ganho do bico do modo de escala vale `(5/2)a − 5b`, e
///    com `b = a/(4(1−ν))` isso é **exatamente zero em `ν = 1/2`** — um material
///    incompressível não pode ser dilatado. Escrevi `0,4` e uma tabela a
///    justificar.
/// 3. **A refutação da refutação, achada MEDINDO** (`poisson_shapes_the_scale_field`
///    devolveu a mesma linha para todo `ν`, de `0,00` a `0,49`): o campo de
///    escala **FATORA**. Usando `r² = rε² − ε²`, o colchete inteiro colapsa em
///    `(1 − 2b)·[ 1/rε³ + 3ε²/(2rε⁵) ]·r`, e o ganho do bico é
///    `(5/2)(1 − 2b)` — o **MESMO** fator. Depois de normalizar ele cancela:
///    *o modo de escala não pergunta de que material o barro é*, e o zero em
///    `ν = 1/2` é uma **singularidade removível da minha parametrização**, não
///    do modelo. Ver [`scale`].
///
/// ⇒ O argumento físico do ato 1 volta a valer, e agora ele PAGA:
///
/// | ν | anisotropia do grab | divergência do grab |
/// |---|---|---|
/// | 0,0 | 1,125× | 0,5000 |
/// | 0,3 | 1,200× | 0,2857 |
/// | 0,4 | 1,250× | 0,1667 |
/// | **0,5** | **1,333×** | **0,0000** |
///
/// Os dois eixos que sobraram melhoram **monotonicamente** até `1/2`: é lá que
/// o agarre é mais anisotrópico (o que o campo tem e uma curva não tem) e é lá
/// que ele **preserva volume**, que é o que separa apertar barro de encolhê-lo.
///
/// ⚠️ **E o `pinch` também colapsa lá:** o termo `(1 − 2b)/rε³` some, e o campo
/// dele passa a decair como `1/r⁵` **sem precisar de escalas nenhumas**.
pub const POISSON: f32 = 0.5;

/// **QUANTOS `ε` DE CAMPO A PEGADA TEM DE CONTER** — a ponte entre um campo de
/// suporte INFINITO e um octree que precisa de um raio para consultar.
///
/// `ε` **é o raio do pincel**, e a pegada é `KELVINLET_REACH · raio`
/// ([`crate::Brush::query_radius`]). Este é o número que decide **onde o campo é
/// cortado**, em unidades da escala dele.
///
/// ⚠️ **A PRIMEIRA versão desta constante estava do avesso, e foi um report do
/// Enio que a derrubou** (*"os modos B e L do grab estão bizarros"*). Ela dizia
/// `ε = raio / REACH` — a pegada mandando no campo — com o racional de que o
/// resíduo na borda do cursor tinha de ser pequeno. O resíduo ficava pequeno e o
/// preço era o produto: espremido a um terço do círculo, o agarre elástico saía
/// uma **AGULHA**. Medido pela porta do artista, ao lado do `s-mode` no mesmo
/// gesto (`measure_grab_modes`) —
///
/// | r/raio | `s-mode` | `l-mode`, `ε = raio/3` |
/// |---|---|---|
/// | 0,25 | 0,9742 | 0,5165 |
/// | 0,50 | 0,5473 | **0,0296** |
/// | 0,75 | 0,2117 | 0,0372 |
///
/// — e o modo inteiro movia **um terço** do barro do irmão (soma `3,060` contra
/// `8,712`). A meia-altura em `0,29·raio` estava escrita como *"a forma da
/// resposta elástica e não um defeito"*: era um defeito, e a nota que o
/// declarava aceitável é o que o manteve vivo.
///
/// ⚠️ **E o cerco tinha uma SEGUNDA metade, mais feia:** as escalas múltiplas
/// matam o campo distante por SUBTRAÇÃO, e subtração ultrapassa — a `Tri` cruza
/// zero AO LADO do puxão em `r/ε ≈ 1,5`. Com `ε = raio/3` isso caía dentro do
/// círculo, então **metade da pegada empurrava o barro para trás** (um anel
/// medido em `−3,8 %` do bico). Com `ε = raio` o cruzamento vai para
/// `1,5 · raio`, fora do que o artista lê como o pincel.
///
/// ⚠️ **A cura é a alternativa que este mesmo doc havia recusado — por
/// ESTIMATIVA, que é o que o §0 do `CLAUDE.md` proíbe.** Ela dizia *"custa ~9×
/// os vértices por dab"*, e ~9× não era um número medido. Medido
/// (`measure_grab_modes::what_a_three_times_wider_query_costs`, malha de 95 234
/// vértices, contra o K1 de 8 ms do ADR-0150) —
///
/// | raio do pincel | vértices na pegada | ms/dab | % do K1 |
/// |---|---|---|---|
/// | 0,1 | 161 | 0,013 | 0,2 % |
/// | 0,3 | 1 429 | 0,100 | 1,2 % |
/// | 0,5 | 4 035 | 0,547 | 6,8 % |
/// | 0,9 | 14 267 | 1,382 | 17,3 % |
/// | 1,5 (a malha INTEIRA) | 57 405 | 5,000 | 62,5 % |
///
/// ⇒ um pincel de detalhe com a pegada TRIPLICADA custa **1,2 % do K1**, e o
/// caso patológico — o modelo todo dentro da pegada — cabe com folga. *O teto
/// era do meu palpite, não do hardware.*
///
/// ⚠️ **O resíduo na borda da pegada continua sendo o que escolhe a FAMÍLIA**
/// (`the_rim_residual_is_what_chose_the_scale_family`; a `Tri` deixa `0,0347`
/// à frente e `−0,025` ao lado em `r/ε = 3`) — o que mudou é que esse corte
/// deixou de morar no círculo do cursor.
///
/// ⚠️ **O preço que FICA, nomeado:** o anel do cursor deixa de significar *o que
/// eu toco*. É a leitura do Elastic Deform do Blender, cujo pincel deforma bem
/// além do círculo desenhado, e quem quer o círculo literal tem o `s-mode` ao
/// lado no mesmo verbo.
pub const KELVINLET_REACH: f32 = 3.0;

/// `b/a` em função do coeficiente de Poisson.
///
/// ⚠️ **Ela é uma FUNÇÃO e não um literal para que a degenerescência do
/// [`POISSON`] seja TESTÁVEL.** Escrita como constante, a frase *"o modo de
/// escala morre em ν = 1/2"* seria prosa que ninguém pode contradizer; escrita
/// assim, o gate a avalia.
#[inline]
const fn b_of(nu: f32) -> f32 {
    1.0 / (4.0 * (1.0 - nu))
}

/// `b/a` — a única coisa que o material contribui depois da normalização.
///
/// ⚠️ `a` fica **implícito em `1`**: ele multiplica numerador e denominador de
/// toda expressão normalizada deste módulo, então carregá-lo seria carregar um
/// fator que se cancela — e um fator que se cancela é onde uma refatoração
/// futura o esquece de um dos lados.
const B: f32 = b_of(POISSON);

/// `rε = √(|r|² + ε²)` — o raio REGULARIZADO, a peça inteira do paper.
#[inline]
fn r_eps(r: [f32; 3], eps: f32) -> f32 {
    (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + eps * eps).sqrt()
}

/// O Kelvinlet de AGARRE, **sem normalizar** — eq. 5 do paper com `a = 1`.
///
/// `u(r) = [ (a−b)/rε + a·ε²/(2·rε³) ]·f + [ b/rε³ ]·(r·f)·r`
#[inline]
fn raw_grab(r: [f32; 3], eps: f32, f: [f32; 3]) -> [f32; 3] {
    let re = r_eps(r, eps);
    let re3 = re * re * re;
    let iso = (1.0 - B) / re + eps * eps / (2.0 * re3);
    let rad = B / re3 * (r[0] * f[0] + r[1] * f[1] + r[2] * f[2]);
    [
        iso * f[0] + rad * r[0],
        iso * f[1] + rad * r[1],
        iso * f[2] + rad * r[2],
    ]
}

/// O ganho do bico de um agarre de raio `ε`: `(3a/2 − b)/ε`.
#[inline]
fn grab_tip(eps: f32) -> f32 {
    (1.5 - B) / eps
}

/// **O AGARRE** — o deslocamento de `r` quando o bico do pincel é puxado por
/// `f`, normalizado para que `grab(0) == f`.
///
/// ⚠️ **A normalização é o que faz o vértice sob o cursor seguir o dedo
/// EXATAMENTE**, e é ela que torna o campo comparável com a lei do `S`: os dois
/// modos entregam o mesmo deslocamento no centro, e tudo o que o chip troca é
/// **a forma com que a vizinhança acompanha**.
#[must_use]
pub fn grab(r: [f32; 3], eps: f32, f: [f32; 3], scales: Scales) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    let mut norm = 0.0f32;
    for &(w, m) in scales.taps() {
        let e = eps * m;
        let u = raw_grab(r, e, f);
        acc[0] += w * u[0];
        acc[1] += w * u[1];
        acc[2] += w * u[2];
        norm += w * grab_tip(e);
    }
    [acc[0] / norm, acc[1] / norm, acc[2] / norm]
}

/// Uma matriz `3×3` em ordem de LINHAS, e o que ela significa para o campo afim.
///
/// ⚠️ **O tipo é um alias e não uma struct de propósito:** o campo afim aceita
/// QUALQUER `F`, e as três famílias do paper são apenas três formas particulares
/// dela (antissimétrica · isotrópica · simétrica sem traço). Uma struct por
/// família esconderia que é uma lei só, e é a lei só que faz o twist não precisar
/// de `ν`.
type Mat3 = [[f32; 3]; 3];

#[inline]
fn mul(m: &Mat3, r: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * r[0] + m[0][1] * r[1] + m[0][2] * r[2],
        m[1][0] * r[0] + m[1][1] * r[1] + m[1][2] * r[2],
        m[2][0] * r[0] + m[2][1] * r[1] + m[2][2] * r[2],
    ]
}

/// O Kelvinlet AFIM, **sem normalizar** — a derivada direcional do agarre.
///
/// `u_F(r) = [ (a−b)/rε³ + 3aε²/(2rε⁵) ]·F r + [3b/rε⁵]·(rᵀFr)·r
///           − (b/rε³)·[ tr(F)·r + Fᵀ r ]`
///
/// ⚠️ **O último termo é o que quase ficou de fora**, e ele é o que separa as
/// três famílias: com `F` antissimétrica ele SOMA ao primeiro (`Fᵀ = −F`,
/// `tr = 0`) e o `b` desaparece do twist inteiro; com `F = s·I` ele subtrai
/// `4b`, e é dali que sai o zero em `ν = 1/2`.
fn raw_affine(r: [f32; 3], eps: f32, f: &Mat3) -> [f32; 3] {
    let re = r_eps(r, eps);
    let re2 = re * re;
    let re3 = re2 * re;
    let re5 = re3 * re2;
    let fr = mul(f, r);
    let c1 = (1.0 - B) / re3 + 1.5 * eps * eps / re5;
    let rfr = r[0] * fr[0] + r[1] * fr[1] + r[2] * fr[2];
    let c2 = 3.0 * B / re5 * rfr;
    let tr = f[0][0] + f[1][1] + f[2][2];
    // `Fᵀ r`, escrito por colunas em vez de transpor a matriz: transpor
    // alocaria uma cópia por vértice por dab, e a leitura por coluna é a mesma
    // aritmética.
    let ftr = [
        f[0][0] * r[0] + f[1][0] * r[1] + f[2][0] * r[2],
        f[0][1] * r[0] + f[1][1] * r[1] + f[2][1] * r[2],
        f[0][2] * r[0] + f[1][2] * r[1] + f[2][2] * r[2],
    ];
    let c3 = B / re3;
    [
        c1 * fr[0] + c2 * r[0] - c3 * (tr * r[0] + ftr[0]),
        c1 * fr[1] + c2 * r[1] - c3 * (tr * r[1] + ftr[1]),
        c1 * fr[2] + c2 * r[2] - c3 * (tr * r[2] + ftr[2]),
    ]
}

/// O ganho do bico de um campo afim de raio `ε`, para um `F` de cada família.
///
/// `∇u(0) = (1/ε³)·[ (5a/2 − b)·F − b·tr(F)·I − b·Fᵀ ]`, e cada família colapsa
/// esse colchete num múltiplo escalar do próprio `F` — é isso que permite
/// normalizar sem inverter uma matriz.
#[inline]
const fn affine_tip(kind: Affine, eps: f32) -> f32 {
    affine_gain(kind, B) / (eps * eps * eps)
}

/// O colchete `[ (5a/2 − b)·F − b·tr(F)·I − b·Fᵀ ]` colapsado num escalar, por
/// família — **parametrizado em `b` para que a degenerescência seja um GATE**.
#[inline]
const fn affine_gain(kind: Affine, b: f32) -> f32 {
    match kind {
        // `Fᵀ = −F`, `tr = 0` ⇒ `(5/2 − b) + b`. **O `b` some**: um twist não
        // pergunta se o material é compressível, porque não muda volume nenhum.
        Affine::Twist => 2.5,
        // `F = s·I` ⇒ `(5/2 − b) − 3b − b`. **Zero em `ν = 1/2`, e é por isso
        // que a [`scale`] não passa por aqui** — o numerador dela carrega o
        // MESMO fator, e o quociente `0/0` tem limite. Ver [`POISSON`].
        Affine::Scale => 2.5 - 5.0 * b,
        // `Fᵀ = F`, `tr = 0` ⇒ `(5/2 − b) − b`.
        Affine::Pinch => 2.5 - 2.0 * b,
    }
}

/// **O NÚCLEO RADIAL sem material** — `1/rε³ + 3ε²/(2rε⁵)`.
///
/// ⚠️ **Ele é o que o twist e a escala TÊM EM COMUM**, e a partilha não é uma
/// economia: é o fato de que os dois campos são o mesmo kernel aplicado a
/// vetores diferentes (`ω × r` e `s·r`). Foi a medição que o expôs — a
/// varredura de ν devolveu a mesma linha oito vezes.
#[inline]
fn radial(r: [f32; 3], eps: f32) -> f32 {
    let re = r_eps(r, eps);
    let re2 = re * re;
    let re3 = re2 * re;
    1.0 / re3 + 1.5 * eps * eps / (re3 * re2)
}

/// **O PERFIL ESCALAR das duas famílias b-livres** — o número por que `ω × r` e
/// `s·r` são multiplicados, e vale `1` no bico.
///
/// ⚠️ **Ele é público porque um giro elástico entregue como DESLOCAMENTO INCHA
/// o barro.** O `v(r)` não depende da escala, então ele sai do somatório e o que
/// sobra é um escalar — mas somar `perfil · (ω × r)` à posição põe o vértice
/// **fora** da circunferência, em `|r|·√(1 + (θ·perfil)²)`. É a MESMA objeção
/// que o braço do [`crate::Verb::Twist`] já documenta contra interpolar pela
/// corda, pelo outro lado da igualdade: ali a forma encolhia, aqui ela cresce.
///
/// **MEDIDO** a meio raio do bico (`measure_field_verbs`), onde o perfil vale
/// `0,577`:
///
/// | θ (rad) | deslocamento | ângulo |
/// |---|---|---|
/// | 0,25 | 1,0104 | **1,0000** |
/// | 0,50 | 1,0408 | **1,0000** |
/// | 1,00 | 1,1546 | **1,0000** |
/// | 2,00 | **1,5271** | **1,0000** |
///
/// ⚠️ **E o primeiro número que escrevi aqui era `+12 %` a meio radiano, sem
/// medição** — a conta com `perfil = 1`, que só vale no BICO, onde `r = 0` e não
/// há raio nenhum a inflar. O expoente é `θ·perfil`, e é a sonda que o diz.
///
/// ⇒ Um verbo que GIRA (ou que ESCALA ao longo de um raio) consome o **perfil**
/// e mantém a própria geometria exata; o campo decide só *quanto cada vértice
/// acompanha*. Quem não tem essa estrutura — o [`pinch`], cuja `F` produz um
/// termo `(r·F r)r` que não é múltiplo de `F r` — consome o vetor.
#[must_use]
pub fn rigid_profile(r: [f32; 3], eps: f32, scales: Scales) -> f32 {
    let mut num = 0.0f32;
    let mut norm = 0.0f32;
    for &(w, m) in scales.taps() {
        let e = eps * m;
        num += w * radial(r, e);
        norm += w * 2.5 / (e * e * e);
    }
    num / norm
}

/// **Onde a indicadora começa a aterrissar** — fração da pegada.
///
/// ⚠️ **`0,75` é o JOELHO de uma varredura, e a minha primeira escolha (`0,5`)
/// foi reprovada por um gate que eu não tinha previsto.** O degrau morre a
/// QUALQUER hold (a janela vale `0` com derivada `0` em `t = 1` por
/// construção), então o que o hold decide não é o degrau — é **quanto do campo
/// externo sobrevive**, e o `pinch` espirra pela normal predominantemente na
/// metade de FORA:
///
/// | HOLD | degrau no aro | retorno de volume do pinch |
/// |---|---|---|
/// | 0,50 | 0,02 % | **0,1686 — reprova** |
/// | 0,65 | 0,03 % | **0,2468 — reprova** |
/// | **0,75** | **0,06 %** | **passa** |
/// | 0,85 | 0,15 % | passa |
///
/// ⇒ o primeiro valor em que o `pinch` mantém o termo de traço zero, com o
/// degrau ainda **26× menor que o do próprio s-mode** (1,57 %), que shipa e que
/// ninguém reportou. Abaixo de `t = 0,75` — os primeiros 2,25 raios de pincel,
/// onde vive todo o gesto — a janela vale `1,0` EXATO e o traço é
/// **byte-idêntico** ao campo cru.
pub const RIM_HOLD: f32 = 0.75;

/// **A ATERRISSAGEM DA INDICADORA** — o que faz o campo chegar a ZERO na borda
/// da pegada em vez de ser decapitado nela.
///
/// ⚠️ **O degrau que isto remove foi MEDIDO, e a nota que o autorizava era
/// minha.** O campo do paper tem suporte INFINITO; a família [`Scales::Tri`]
/// mata os termos `1/r` e `1/r³`, mas em `r/ε = 3` o deslocamento ainda vale
/// **3,47 %** do bico (`the_rim_residual_is_what_chose_the_scale_family`), e a
/// indicadora o cortava a zero em distância nenhuma:
///
/// | | bico | último anel DENTRO | 1º anel FORA | degrau | vértices na costura |
/// |---|---|---|---|---|---|
/// | s-mode | 0,34398 | 0,00539 | 0 | 1,57 % | 10 |
/// | l-mode | 0,33241 | 0,00962 | 0 | **2,90 %** | **114** |
///
/// ⚠️ **E o que tornou isto VISÍVEL foi mover o `ε`, não o resíduo.** Enquanto
/// `ε = raio/3` a pegada era `raio` e o corte caía **no anel do cursor**, onde
/// um degrau lê como *a borda do pincel* — a §7.11 pôs `ε = raio` e a pegada em
/// `3·raio`, e o MESMO degrau mudou-se para onde nada explica que o pincel
/// acabou, numa costura **11× mais longa**. O gate continuou verde (`0,0347 <
/// 0,036`) porque ele mede o resíduo e o CERTIFICA; a mensagem dele dizia *"a
/// borda do CURSOR"*, e essa frase deixou de ser verdade sem ninguém a
/// reconferir.
///
/// ⛔ **Alargar o alcance NÃO é a cura, e isso é medição:** `r/ε = 4` deixa
/// `1,19 %`, `5` deixa `0,48 %`, `6` deixa `0,215 %` — nunca zero, e a contagem
/// de vértices da pegada cresce com `r²`. A janela dá **exatamente zero, por
/// construção, a qualquer alcance**, por uma multiplicação.
///
/// ⚠️ **Ela mora aqui e é aplicada pelo CONSUMIDOR**, de propósito: o campo do
/// paper é infinito por desenho e fica intacto; quem tem pegada finita é o
/// traço, e é dele a concessão. E como os cinco verbos de campo consomem a
/// curva **linearmente** (o `grab` e o `pinch` dobram-na na força, que é linear
/// nela), um fator na indicadora escala os cinco **por construção** — não há
/// lista de verbos a apodrecer.
///
/// `C¹` nas duas pontas (valor e derivada nulos em `t = 1`) e sem
/// transcendental (HR-5).
#[must_use]
pub fn rim_landing(t: f32) -> f32 {
    if t <= RIM_HOLD {
        return 1.0;
    }
    if t >= 1.0 {
        return 0.0;
    }
    let s = (t - RIM_HOLD) / (1.0 - RIM_HOLD);
    1.0 - s * s * (3.0 - 2.0 * s)
}

/// O campo `K(r)·v(r)` normalizado pelo ganho `5/2` que as duas famílias
/// b-livres partilham — a porta do [`twist`] e da [`scale`].
///
/// ⚠️ **Ele DELEGA ao [`rigid_profile`] em vez de somar o vetor**, e é isso que
/// impede as duas leituras do mesmo campo de divergirem: o verbo que gira e a
/// função da biblioteca leem o mesmo número, e não dois somatórios com a mesma
/// álgebra e arredondamentos diferentes.
fn rigid(r: [f32; 3], eps: f32, scales: Scales, v: impl Fn([f32; 3]) -> [f32; 3]) -> [f32; 3] {
    let vr = v(r);
    let p = rigid_profile(r, eps, scales);
    [p * vr[0], p * vr[1], p * vr[2]]
}

/// Qual das três formas de `F` um campo afim carrega.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Affine {
    /// `F` antissimétrica — gira em torno de um eixo. Sem mudança de volume, e
    /// **sem dependência de [`POISSON`]**.
    Twist,
    /// `F = s·I` — dilata (ou colapsa) radialmente.
    Scale,
    /// `F` simétrica e sem traço — aperta num eixo e espirra no outro.
    Pinch,
}

/// O campo afim normalizado para que `∇u(0) == F`.
fn affine(r: [f32; 3], eps: f32, f: &Mat3, kind: Affine, scales: Scales) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    let mut norm = 0.0f32;
    for &(w, m) in scales.taps() {
        let e = eps * m;
        let u = raw_affine(r, e, f);
        acc[0] += w * u[0];
        acc[1] += w * u[1];
        acc[2] += w * u[2];
        norm += w * affine_tip(kind, e);
    }
    [acc[0] / norm, acc[1] / norm, acc[2] / norm]
}

/// **A TORÇÃO** — `∇u(0) r = ω × r`, ou seja o barro sob o bico gira como um
/// corpo rígido de velocidade angular `ω` e o resto acompanha elasticamente.
///
/// ⚠️ **Não pergunta de que material o barro é** — o `b` cancela na lei afim
/// para uma `F` antissimétrica, e o gate
/// `the_twist_has_a_closed_form_without_the_material` o afirma contra a forma
/// geral.
#[must_use]
pub fn twist(r: [f32; 3], eps: f32, omega: [f32; 3], scales: Scales) -> [f32; 3] {
    rigid(r, eps, scales, |r| {
        [
            omega[1] * r[2] - omega[2] * r[1],
            omega[2] * r[0] - omega[0] * r[2],
            omega[0] * r[1] - omega[1] * r[0],
        ]
    })
}

/// **A ESCALA** — `∇u(0) r = s·r`: o barro sob o bico dilata pela fração `s`.
///
/// ⚠️ **Ela NÃO passa pela lei afim geral, e a razão é a única singularidade
/// deste módulo.** Com `F = s·I` o numerador inteiro fatora em `(1 − 2b)` e o
/// ganho do bico vale `(5/2)(1 − 2b)`: em `ν = 1/2` os dois são zero, e a lei
/// geral computaria `0/0`. O quociente tem **limite**, ele é o [`radial`], e é
/// ele que esta função avalia — a mesma resposta para todo `ν`, e definida
/// justamente onde a outra rota não está.
#[must_use]
pub fn scale(r: [f32; 3], eps: f32, s: f32, scales: Scales) -> [f32; 3] {
    rigid(r, eps, scales, |r| [s * r[0], s * r[1], s * r[2]])
}

/// **O APERTO** — `F = s·(n nᵀ − ½(I − n nᵀ))`: estica pela fração `s` ao longo
/// de `n` e aperta a METADE disso no plano perpendicular.
///
/// ⚠️ **O traço é zero por construção**, e é isso que o torna um aperto em vez
/// de um encolhimento: o que sai de lado tem de sair por algum lugar, e sai pela
/// normal. Um pinch que só aperta lateralmente — o que o `s-mode` faz — remove
/// volume, e o vinco que ele deixa não é o de um material.
///
/// ⚠️ **`n` tem de chegar UNITÁRIO.** Não há normalização aqui de propósito: o
/// chamador já tem a normal de área do dab e re-normalizá-la por vértice
/// pagaria uma raiz por vértice por dab para responder o que já se sabe.
#[must_use]
pub fn pinch(r: [f32; 3], eps: f32, n: [f32; 3], s: f32, scales: Scales) -> [f32; 3] {
    // `n nᵀ − ½(I − n nᵀ)` == `1,5·n nᵀ − ½·I`.
    let h = 1.5 * s;
    let f: Mat3 = [
        [h * n[0] * n[0] - 0.5 * s, h * n[0] * n[1], h * n[0] * n[2]],
        [h * n[1] * n[0], h * n[1] * n[1] - 0.5 * s, h * n[1] * n[2]],
        [h * n[2] * n[0], h * n[2] * n[1], h * n[2] * n[2] - 0.5 * s],
    ];
    affine(r, eps, &f, Affine::Pinch, scales)
}

#[cfg(test)]
#[path = "kelvinlet_tests.rs"]
mod tests;
