//! ⭐⭐⭐ **O ATRITO, E O MATERIAL DE QUE A PEÇA É FEITA** — doc 109 §7, report do dono
//! (2026-09-13): *«os círculos não rotacionam com a colisão, talvez por falta de atrito.
//! Precisamos de parâmetros do material»*.
//!
//! ## Porque um círculo NÃO PODIA rodar, e a conta que o diz
//!
//! A correcção de não-penetração move as peças **ao longo da normal**, e o braço dela é
//! `r × n` ([`super::Contacto::braco`]). Num disco o ponto de contacto está sempre **sobre a
//! linha dos centros**, logo `r = ±R·n` e
//!
//! ```text
//!   r × n = ±R · (n × n) = 0          EXACTAMENTE zero, em todo contacto de disco
//! ```
//!
//! — zero por construção, nos três sítios que produzem um contacto de disco (disco×disco,
//! disco×caixa por dentro e por fora, e o ponto de suporte contra um plano). ⇒ *uma lei que só
//! empurra ao longo da normal não consegue rodar um círculo, qualquer que seja o número que se lhe
//! dê.* É o gate `a_disc_has_no_lever_on_the_normal_and_all_of_it_on_the_tangent`.
//!
//! ⚠️ **Zero em aritmética EXACTA, e ruído de cancelamento em `f32`:** o braço é `ponto − centro`,
//! e numa peça longe da origem isso subtrai dois números muito maiores do que a diferença. Medido
//! na cena `=115` (bola a `x ≈ 1,35`, 2 s de rampa): **`0,0000075°` acumulados** — sete milionésimos
//! de grau. *Uma barra de zero-ao-bit sobre isto mediria a aritmética, não a lei*, e é por isso que
//! os gates de cena usam `1e-3°` com este número escrito ao lado.
//!
//! ⭐⭐ **E a metade que faltava tem o braço MÁXIMO exactamente onde a outra tem zero.** Com
//! `t = perp(n)` vale a identidade
//!
//! ```text
//!   r × t = r · n
//! ```
//!
//! (basta expandir), e num disco `r · n = ±R`. *O braço da normal e o braço da tangente são as
//! duas coordenadas do mesmo vector: onde um é zero o outro é o raio inteiro.* É por isso que o
//! atrito não é um afinamento desta lei — é a única coisa que roda uma bola.
//!
//! ## A lei, e porque ela é POSICIONAL
//!
//! É a restrição de atrito do *Position Based Dynamics* (Müller et al. 2007 §3.5; a forma de
//! Macklin–Müller–Chentanez para corpo rígido): o **deslize** é o quanto os dois pontos de
//! contacto se mexeram **um em relação ao outro ao longo da tangente** desde o início do passo, e
//! o atrito é a correcção que o desfaz — repartida entre mover e RODAR pela massa efectiva ao
//! longo de `t`, exactamente como a metade normal —, limitada por **Coulomb**:
//!
//! ```text
//!   λt = clamp( deslize / (kt_a + kt_b),  ±μ · λn )      kt = w + invI · (r · n)²
//!   Δp = −t · λt · w            Δθ = −(r · n) · λt · invI
//! ```
//!
//! ⭐ **Numa bola pousada isto dá o rolamento de manual à primeira correcção:** `kt = w + 2w = 3w`,
//! logo a translação recebe `⅓` do deslize e a rotação `⅔` — e a soma no ponto de contacto é
//! exactamente `−deslize`. *A bola deixa de derrapar porque começou a rolar, não porque travou.*
//!
//! ⚠️ **O limite de Coulomb é o que a torna física e não cola:** `λn` numa pilha assente é a
//! penetração que a gravidade fez **naquele tique** (`~g·dt²`), então o atrito por tique é
//! `μ·g·dt²` — pequeno de propósito. Uma bola a `1 u/s` sob gravidade `4` e `μ = 0,6` passa de
//! derrapar a rolar em ~0,3 s, que é o que uma bola faz.
//!
//! ## O material
//!
//! [`Material`] é o par que todo motor tem (Rapier `friction`/`restitution`, Godot
//! `PhysicsMaterial`, Box2D `friction`/`restitution`), e **combina-se por PAR** pelas leis
//! clássicas do Box2D: o atrito pela média **geométrica** (`√(μa·μb)` — uma peça de gelo desliza
//! contra tudo, que é o que «gelo» quer dizer) e o salto pelo **maior** dos dois (uma bola
//! saltitante salta contra uma parede morta).
//!
//! ⚠️ **Ausente é `0`, e é isso que mantém toda corrente de hoje byte-idêntica** — sem as colunas
//! [`ph2d_nodegraph::attr::FRICTION_COLUMN`]/[`ph2d_nodegraph::attr::BOUNCE_COLUMN`] o material é
//! [`Material::LISO`], `μ = 0`, `λt = 0`, e nem uma soma é feita.

use ph2d_nodegraph::attr::{BOUNCE_COLUMN, Column, FRICTION_COLUMN, ROLLING_COLUMN, Stream};

use super::Colisor;

/// **DE QUE A PEÇA É FEITA** — o par que todo motor tem, mais o terceiro número que faz uma bola
/// parar sozinha. Cada um tem o tecto DELE, e nenhum é `0..1` por acidente: ver
/// [`ph2d_nodegraph::attr::BOUNCE_MAX`] e [`ph2d_nodegraph::attr::ROLLING_MAX`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// O coeficiente de Coulomb. `0` = gelo (o deslize não é oposto), `1` = lixa.
    pub atrito: f32,
    /// Quanto de um embate volta. `0` = morto (a lei de sempre), `2` = o dobro do que levou.
    pub salto: f32,
    /// ⭐ **O ATRITO DE ROLAMENTO** (doc 109 §7.10): quanto o contacto se opõe a ROLAR, e não a
    /// deslizar. `0` = a bola rola para sempre (a lei de antes desta coluna, ao bit).
    ///
    /// ⚠️ **Não se combina por par** — é da peça, e o porquê está no
    /// [`ph2d_nodegraph::attr::ROLLING_COLUMN`].
    pub rolar: f32,
}

/// Os tectos e a coerção vivem ao lado das COLUNAS (`ph2d_nodegraph::attr`), e não aqui: quem
/// declara e quem consome têm de ler o mesmo número. Ver [`ph2d_nodegraph::attr::BOUNCE_MAX`] —
/// é lá que está a tabela MEDIDA que abriu a faixa de `1` para `2`.
pub use ph2d_nodegraph::attr::{BOUNCE_MAX, FRICTION_MAX, ROLLING_MAX, material_coerce as coage};

impl Material {
    /// **O material da AUSÊNCIA** — gelo morto, que é a lei de antes do doc 109 §7, termo a termo.
    pub const LISO: Self = Self {
        atrito: 0.0,
        salto: 0.0,
        rolar: 0.0,
    };

    /// Lido de um valor autorado: um não-finito não é um pedido, e cada metade tem o tecto dela.
    fn de(atrito: f32, salto: f32, rolar: f32) -> Self {
        Self {
            atrito: coage(atrito, FRICTION_MAX),
            salto: coage(salto, BOUNCE_MAX),
            rolar: coage(rolar, ROLLING_MAX),
        }
    }
}

/// **O material que cada linha de um stream DECLAROU** — a porta única, irmã da
/// [`super::colisores`], **e `None` com a mesma força**: sem nenhuma das duas colunas ninguém
/// declarou material nenhum.
///
/// ⚠️ **`None` e `Some(LISO)` são coisas DIFERENTES, e a distinção é load-bearing:** o obstáculo do
/// `sim.collide` tem atrito próprio, e uma peça que não declarou nada tem de continuar a sofrê-lo
/// inteiro (a lei de sempre), enquanto uma peça que declarou `atrito = 0` **é gelo** e desliza
/// sobre ele. Colapsar as duas faria toda cena de hoje perder o atrito do chão.
pub fn materiais(s: &Stream) -> Option<Vec<Material>> {
    let coluna = |nome: &str| match s.get(nome) {
        Some(Column::Scalar(v)) if v.len() == s.count() => Some(v.clone()),
        _ => None,
    };
    let (a, b, c) = (
        coluna(FRICTION_COLUMN),
        coluna(BOUNCE_COLUMN),
        coluna(ROLLING_COLUMN),
    );
    if a.is_none() && b.is_none() && c.is_none() {
        return None;
    }
    Some(
        (0..s.count())
            .map(|i| {
                Material::de(
                    a.as_ref().map_or(0.0, |v| v[i]),
                    b.as_ref().map_or(0.0, |v| v[i]),
                    c.as_ref().map_or(0.0, |v| v[i]),
                )
            })
            .collect(),
    )
}

/// **O ATRITO DO PAR** — a média geométrica (Box2D). Uma peça de gelo (`0`) desliza contra tudo.
pub fn mu(a: f32, b: f32) -> f32 {
    (a.max(0.0) * b.max(0.0)).sqrt()
}

/// **O SALTO DO PAR** — o maior dos dois (Box2D). Uma bola saltitante salta contra uma parede
/// morta, que é o que o artista espera de uma bola saltitante.
///
/// ⚠️ O tecto é o [`BOUNCE_MAX`], **lido da coluna e não escrito outra vez**: o `1` que este `clamp`
/// tinha era o segundo sítio onde a faixa vivia, e teria sobrevivido à ordem do dono em silêncio.
pub fn salto(a: f32, b: f32) -> f32 {
    coage(a.max(b), BOUNCE_MAX)
}

/// **O multiplicador TANGENCIAL** — o que desfaz o `deslize`, limitado por Coulomb a `μ · λn`.
///
/// `soma` é `kt_a + kt_b`, a massa efectiva do par ao longo da tangente. Um par sem massa
/// efectiva, sem atrito ou sem contacto normal não tem atrito nenhum a repartir — e sair aqui é o
/// que mantém `μ = 0` sem uma divisão sequer.
pub fn lambda(deslize: f32, soma: f32, mu: f32, lambda_n: f32) -> f32 {
    if !soma.is_finite() || !deslize.is_finite() || soma <= 0.0 || mu <= 0.0 || lambda_n <= 0.0 {
        return 0.0;
    }
    let teto = mu * lambda_n;
    (deslize / soma).clamp(-teto, teto) // CLAMP-OK: teto >= 0 verificado acima
}

/// ⭐⭐⭐ **O IMPULSO DE ROLAMENTO** — o que faz uma bola a rolar **parar sozinha** (doc 109 §7.10).
///
/// O atrito tangencial trava quem **derrapa**; uma bola que já rola tem velocidade zero no ponto de
/// contacto, logo ele não tem nada a opor e ela rola para sempre. O que a trava na vida real é a
/// **deformação** do contacto, que a literatura modela como um binário limitado pela normal:
///
/// ```text
///   |τ| ≤ μr · jn · R          ⇒   jr = clamp( L,  ±μr·jn·R ),   L = ω / invI
/// ```
///
/// — a mesma forma do `rollingResistance` do Box2D v3 e do `rolling_resistance` do Rapier.
/// `L` é o **momento angular que a peça tem**, então o `clamp` diz que o pior caso é parar a
/// rotação neste tique: ⛔ ele nunca a inverte, e é por isso que o tecto não tem divergência a
/// temer (`ph2d_nodegraph::attr::ROLLING_MAX`).
///
/// ⚠️ **Ele corre DEPOIS do tangencial e lê o `ω` já corrigido por ele** — os dois escrevem a mesma
/// grandeza, e lidos do mesmo `ω` o rolamento desfaria parte do giro que o atrito acabou de dar.
#[must_use]
pub fn rolamento(momento: f32, mu_r: f32, lambda_n: f32, braco: f32) -> f32 {
    let teto = mu_r * lambda_n * braco.abs();
    if !teto.is_finite() || !momento.is_finite() || teto <= 0.0 {
        return 0.0;
    }
    momento.clamp(-teto, teto) // CLAMP-OK: teto > 0 verificado acima
}

/// **O DESLIZE que o atrito opõe** — o que o solver precisa de saber para o medir.
///
/// ⚠️ **Sem isto não há atrito, e a ausência é honesta**: o deslize é *«quanto os pontos de
/// contacto se mexeram um em relação ao outro desde o início do passo»*, e um consumidor sem
/// passo (o `motion.collide`, que reautora as posições a cada quadro) não tem esse «antes» —
/// inventá-lo daria uma força a partir de um número que não é um movimento.
#[derive(Clone, Copy)]
pub struct Deslize<'a> {
    /// Onde cada peça estava ANTES de o passo a mover.
    pub antes: &'a [[f32; 2]],
    /// Quanto cada peça já rodou neste passo por outra causa que não o contacto (o `spin`
    /// integrado), em GRAUS. Uma bola que chega a girar derrapa contra o chão mesmo parada, e sem
    /// isto o atrito não a veria.
    pub girou_antes: &'a [f32],
    /// De que cada peça é feita.
    pub material: &'a [Material],
}

/// **AS PEÇAS que o solver separa** — a fotografia do que cada uma é.
///
/// ⚠️ Uma struct e não sete argumentos: a lista cresceu com o material, e um `&[f32]` a mais numa
/// chamada posicional é a forma clássica de o `pesos` e o `inv_inercia` trocarem de lugar sem o
/// compilador dizer nada.
#[derive(Clone, Copy)]
pub struct Pecas<'a> {
    pub colisores: &'a [Option<Colisor>],
    /// `inv_mass`: `0` é um obstáculo (não se move, não roda).
    pub pesos: &'a [f32],
    /// Quanto cada peça roda por unidade de binário; `0` trava.
    pub inv_inercia: &'a [f32],
    /// O deslize a opor, ou `None` — ver [`Deslize`].
    pub deslize: Option<Deslize<'a>>,
}

impl<'a> Pecas<'a> {
    /// As peças sem material nenhum — a lei de antes do §7, que é o que os gates de geometria
    /// pura afirmam.
    pub fn novas(
        colisores: &'a [Option<Colisor>],
        pesos: &'a [f32],
        inv_inercia: &'a [f32],
    ) -> Self {
        Self {
            colisores,
            pesos,
            inv_inercia,
            deslize: None,
        }
    }

    /// O material da peça `i`, ou [`Material::LISO`] quando ninguém o declarou.
    pub(crate) fn material(&self, i: usize) -> Material {
        self.deslize
            .and_then(|d| d.material.get(i).copied())
            .unwrap_or(Material::LISO)
    }
}

/// **O que uma corrida do solver ESCREVE, além das posições.**
pub struct Saida<'a> {
    /// Quanto cada peça rodou, em GRAUS — o chamador soma-o à coluna `rot`.
    pub giro: &'a mut [f32],
    /// O salto efectivo de cada peça neste tique: o maior [`salto`] entre os pares que ela tocou.
    /// `0` onde nada lhe tocou, que é a lei de sempre.
    pub salto: &'a mut [f32],
}
