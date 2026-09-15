//! ⭐⭐⭐ **O ESTÚDIO** — o céu deixa de ser uma rampa lisa e passa a ter uma FONTE COM FORMA.
//!
//! # A premissa, MEDIDA antes de existir uma linha disto
//!
//! O [plano](../../../docs/Render3d/03_o_plano.md) diz que a `W3` é *«o que faz o metal existir»*.
//! Antes de a construir, a sonda `render_light_tests::measure_how_much_of_the_material_this_sky_lets_through`
//! mediu o que o céu de ontem — `A + B·y`, uma rampa linear — deixa ver, em BYTES sobre os pixels da
//! peça:
//!
//! | | espelho (`rugosidade 0,05`) | baço (`rugosidade 1,00`) |
//! |---|---:|---:|
//! | estrutura `\|∇²\|` do verde | `1,074` | `1,085` |
//! | quanto da peça vem da LÂMPADA | `2` de `154` | `146` de `212` |
//!
//! ⭐⭐⭐ **Um espelho e um pedaço de giz têm o MESMO contraste local.** Não é uma questão de grau: a
//! rampa filtrada por um lóbulo largo continua a ser uma rampa, logo **não há nada para reflectir** —
//! e uma lâmpada direccional é um delta, que um espelho reflecte num conjunto de medida nula (`2`
//! bytes de `154`). ⇒ *o metal deste app era uma bola cinzenta com um gradiente.*
//!
//! ⚠️ **E o controlo que o dono mais usa era o mais invisível:** num DIELÉCTRICO, arrastar o
//! `Roughness` de `0,30` a `0,05` movia `1 122` de `61 804` pixels (**`1,82 %`**), com `\|Δ\|` médio de
//! **`0,53` bytes** sobre a peça inteira.
//!
//! ⚠️ **A régua da estrutura tem CONTROLO**, senão um `\|∇²\|` que lê o mesmo número em tudo é
//! indistinguível de uma régua cega: sob um céu com ARESTA o espelho vai a `2,405` (`2,24×`) e o giz
//! fica em `1,221`. *Ela vê, e vê mais no espelho — que é o sentido certo.*
//!
//! # ⛔⛔ A recusa medida que existe sobre isto respondeu a OUTRA pergunta
//!
//! O [`ph2d_light::ENV_SLOPE`] carrega uma recusa por extenso: *«um ambiente de três zonas (céu,
//! horizonte claro, chão) — a forma de um HDRI de estúdio — dá contraste cima/baixo de `1,83×`
//! contra os `2,20×` deste … e ainda deixa `0,0042` de resíduo»*. ⚠️ **Ela foi medida sobre a
//! IRRADIÂNCIA** — quão bem o modelo reproduz a luz que uma DIFUSA recebe — e sobre isso continua a
//! valer inteira. *Uma recusa medida responde UMA pergunta* (`CLAUDE.md` §5.0): nada nela mede o que
//! um ESPELHO vê, e a rampa ganha aquela comparação por ser lisa exactamente pela razão por que perde
//! esta.
//!
//! ⇒ **a rampa FICA, com a lei dela intacta e byte a byte** (há gate), e o que se acrescenta é uma
//! **forma** por cima.
//!
//! # A lei, em duas decisões que não são números escolhidos
//!
//! O estúdio é `rampa + caixa de luz`, e a caixa é uma calote suave — uma **gaussiana esférica**
//! `e^{λ(ω·μ − 1)}`, que é a forma sem aresta mais barata que existe.
//!
//! 1. **O EIXO da caixa é o eixo da PRÓPRIA RAMPA** (`+y` em espaço de vista). O céu já é claro em
//!    cima e escuro em baixo; a caixa **afia** esse eixo em vez de trazer uma direcção nova. ⇒ zero
//!    constantes de direcção, e zero constantes de cor: ela é da cor média do céu
//!    ([`ph2d_light::ENV_BASE`]).
//! 2. **A ENERGIA sai do ambiente, não se soma a ele** — a caixa leva uma FRACÇÃO
//!    ([`SOFTBOX_SHARE`]) da energia total do céu e o termo constante desce exactamente o mesmo
//!    tanto. ⇒ a radiância média sobre a esfera fica igual (há gate) e o chão escurece **de graça**,
//!    porque a energia dele é que subiu.
//!
//! ⛔⛔ **E a frase que estava escrita aqui — *«logo o branco chapado do §8 não pode piorar»* — era
//! FALSA, e a medição derrubou-a na primeira corrida.** Média constante é uma afirmação sobre a
//! MÉDIA; o corte é uma afirmação sobre o **PICO**, e concentrar energia é precisamente subir o
//! pico. Medido na esfera do §6: o branco chapado vai de `5 180` para `7 972` pixels na vista
//! `Standard` a `0` stops.
//!
//! ⭐⭐⭐ **E a mesma tabela traz a saída, que é a que o §8 já tinha escrito:** na vista **`Neutral`**
//! o corte é **`0` em todas as vinte configurações medidas**, e a `Standard` a `−1` stop fica em
//! `114 → 115`. *A caixa de luz e a gestão de cor são o mesmo assunto — pôr uma fonte no céu é
//! exactamente o que torna a vista que não corta obrigatória*, e a escolha da omissão continua a ser
//! do dono (`docs/Render3d/05` §8).
//!
//! ⚠️ **E é por isso que as constantes foram escolhidas na `Neutral`:** sob a `Standard` um planalto
//! saturado tem `∇² = 0`, logo o corte **esconde** o efeito que a régua devia medir — a régua estaria
//! a ser lida através do defeito que ela acusa.
//!
//! # ⭐⭐⭐ O pré-filtro é uma TABELA, e a tabela é a própria DEFINIÇÃO
//!
//! O [`ph2d_material::Environment`] pede a radiância **já pré-filtrada** para um lóbulo GGX de
//! rugosidade `α`. Para a rampa isso tem forma fechada e exacta (o
//! [`crate::render_light::lobe_shrink`]); para uma calote, não tem.
//!
//! ⛔⛔ **E a forma fechada mais óbvia foi CONSTRUÍDA, MEDIDA e DEITADA FORA:** uma gaussiana
//! esférica convolvida com outra fecha em álgebra (o produto de duas SG é uma SG), e o `λ` do núcleo
//! pode ser escolhido para ter **exactamente o primeiro momento** do pré-filtro GGX — o que faz as
//! duas metades do céu concordarem em toda função linear. A álgebra batia a quadratura a `2,7e-6`.
//! ⚠️ **O MODELO é que não bate:** contra o pré-filtro GGX de verdade ela erra **`5 %` a `28 %`** na
//! faixa útil, e o pior é justamente onde o material de omissão vive — *a cauda do GGX é pesada e a
//! de uma gaussiana não é, e nenhum `λ` cura isso* (o melhor `λ` ajustado por varredura ainda deixa
//! `45 %` a `α = 0,1`). ⭐ **E a `α` é o QUADRADO da rugosidade** (`isotropic_alpha`), logo a
//! rugosidade de omissão `0,3` vive em `α = 0,09`, no pior pedaço da faixa.
//!
//! ⇒ a tabela. Ela é construída pela **definição** que o `mx_environment_prefilter` escreve
//! (`N = V = R`, amostragem por importância do GGX, peso `N·L`):
//!
//! ```text
//! m(α, ψ) = Σ (N·L) · caixa(ω_L) / Σ (N·L)
//! ```
//!
//! *Não há aproximação para declarar: a lei do produto e o oráculo dela são a mesma conta, e o gate
//! mede só a RESOLUÇÃO (a tabela contra uma quadratura `390×` mais densa).*
//!
//! ⚠️ **Os dois eixos são deformados, e cada um pela sua razão:** o da rugosidade é `√α` (isto é, a
//! rugosidade que o artista arrasta) e o do ângulo é `√(1 − cos ψ)`, que é **proporcional a `ψ`
//! junto do pico** e comprime a cauda. *Uma tabela uniforme em `cos ψ` tem `10°` de passo onde a
//! caixa tem `25°` de raio, e seria uma calote de seis degraus.* ⛔ E nenhum dos dois eixos precisa de
//! um `acos` em tempo de execução — só de dois `sqrt`.

use ph2d_material::Rgb;
use std::sync::LazyLock;

/// **O RAIO ANGULAR da caixa de luz**, em graus. Medido — ver `docs/Render3d/05`.
pub const SOFTBOX_RADIUS_DEG: f32 = 25.0;

/// **Que fracção da energia total do céu vive na caixa.** Medido — ver `docs/Render3d/05`.
///
/// ⭐ `0,20` em `4,7 %` da esfera faz a caixa **`5,4×`** mais clara do que o céu à volta dela.
pub const SOFTBOX_SHARE: f32 = 0.20;

/// **A largura do BORDO da caixa**, em graus.
///
/// ⭐⭐ **Não é um gosto: é a RESOLUÇÃO da tabela, e o número saiu de uma conta que o gate depois
/// confirmou.** Interpolar um `smoothstep` de largura `W` com passo `h` erra `0,75·(h/W)²`; com o
/// passo do eixo do ângulo (`~0,22°`, ver [`ANGLE_N`]) e a barra de `2e-3` do gate sai `W ≥ 5,8°`.
///
/// ⚠️⚠️ **A primeira redacção disto pôs `3°` «≈ 3 células» e o gate REPROVOU com `4,95e-2`** — vinte e
/// cinco vezes a barra, e sempre no bordo. *Uma aresta mais dura do que a tabela representa não é uma
/// aresta: é o degrau da tabela a passar por uma.*
pub const SOFTBOX_RIM_DEG: f32 = 6.0;

/// Amostras no eixo da rugosidade (`√α`), de `0` a `1`.
pub const ROUGH_N: usize = 49;
/// Amostras no eixo do ângulo (`√(1 − cos ψ)`), de `0` a `√2` — passo `~0,22°` em toda a faixa.
pub const ANGLE_N: usize = 513;
/// Direcções por célula na construção.
///
/// ⭐⭐ **O número saiu da única régua que o dono vê: PIXELS.** Contra uma tabela de `8 192`
/// amostras, esta devolve `\|Δ\| médio 0,041 byte` e **pior caso `3` bytes** sobre oito materiais.
/// ⚠️ A régua intermédia dizia outra coisa — `7,4e-3` de desvio contra a definição, *vinte e cinco
/// vezes* o que a primeira redacção da barra pedia —, e subir para `2 048` amostras compra `0,034`
/// de um byte por **`4×` o preço da construção**. *Uma régua intermédia não sabe quando parar; a do
/// consumidor sabe.*
const SAMPLES: u32 = 512;

/// **A FORMA da caixa de luz** — um disco com o bordo esbatido.
///
/// ⛔⛔ **A gaussiana esférica foi construída, MEDIDA e DEITADA FORA por esta régua:** ela pré-filtra
/// em forma fechada, o que era o argumento inteiro a favor dela — e um espelho sob uma gaussiana
/// ganha `9 %` de estrutura, porque *uma gaussiana não tem aresta nenhuma*. **A aresta é o efeito.**
/// Uma caixa de luz real é um rectângulo de bordo nítido, e é isso que um cromado mostra. ⇒ com a
/// tabela a fazer o pré-filtro, a forma passou a ser livre, e o disco é a forma certa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Softbox {
    pub radius_deg: f32,
    pub rim_deg: f32,
}

impl Softbox {
    /// A caixa do produto.
    pub const PRODUCT: Self = Self {
        radius_deg: SOFTBOX_RADIUS_DEG,
        rim_deg: SOFTBOX_RIM_DEG,
    };

    /// As duas fronteiras do bordo, **em cosseno**, resolvidas uma vez.
    ///
    /// ⚠️ **A construção avalia a forma `49 × 513 × 512` vezes**, e a primeira redacção resolvia
    /// `to_radians().cos()` DUAS vezes em cada uma delas — dois transcendentais por amostra, dentro
    /// do laço mais quente desta crate. *Uma constante dobrada dentro de um laço corre onde o laço
    /// corre* (a mesma forma que a memória `a_constant_folded_into_a_tree` já regista).
    #[must_use]
    fn profile(&self) -> Profile {
        let meio = self.rim_deg * 0.5;
        Profile {
            dentro: f64::from(self.radius_deg - meio).to_radians().cos(),
            fora: f64::from(self.radius_deg + meio).to_radians().cos(),
        }
    }

    /// O ÂNGULO SÓLIDO da forma, `∫ forma dω` — integrado, nunca uma fórmula por forma.
    ///
    /// ⭐ É isto que deixa a forma ser livre: quem muda o disco não tem uma segunda conta para
    /// acertar.
    #[must_use]
    fn solid_angle(&self) -> f64 {
        const N: usize = 1 << 16;
        let p = self.profile();
        let passo = 2.0 / N as f64;
        let soma: f64 = (0..N)
            .map(|i| p.value(-1.0 + (i as f64 + 0.5) * passo))
            .sum();
        2.0 * std::f64::consts::PI * soma * passo
    }
}

/// A forma com as fronteiras já em cosseno.
#[derive(Clone, Copy, Debug)]
struct Profile {
    dentro: f64,
    fora: f64,
}

impl Profile {
    /// ⚠️ **O `smoothstep` corre no COSSENO e não no ângulo** — é aqui que a forma é definida, e não
    /// há uma segunda definição em graus para discordar dela.
    #[must_use]
    fn value(&self, cos_psi: f64) -> f64 {
        // ⚠️ Um disco que cobre a esfera inteira (`raio 180°, bordo 0`) tem `dentro == fora` e a
        // divisão é `0/0`. Ele é o CONTROLO do gate da normalização, logo o caso degenerado tem de
        // responder — e a resposta é «tudo dentro».
        if self.dentro <= self.fora {
            return f64::from(u8::from(cos_psi >= self.fora));
        }
        let t = ((cos_psi - self.fora) / (self.dentro - self.fora)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// O eixo do ângulo: `u = √(1 − cos ψ)`, e o passo dele.
fn angle_axis(cos_psi: f32) -> f32 {
    (1.0 - cos_psi.clamp(-1.0, 1.0)).max(0.0).sqrt()
}

/// **O PRÉ-FILTRO DA CAIXA**, tabelado — a média da calote de amplitude `1` sobre o lóbulo.
///
/// ⭐ Uma fonte UNIFORME devolve `1` em qualquer célula das duas tabelas, e é isso que mantém o
/// *furnace test* de pé sem uma linha nova: a normalização é a mesma dos dois lados.
#[derive(Debug)]
pub struct BoxPrefilter {
    /// `ROUGH_N × ANGLE_N` — o especular.
    spec: Vec<f32>,
    /// `ANGLE_N` — o difuso (o lóbulo cosseno).
    diff: Vec<f32>,
    /// Que fracção da energia total do céu vive nela.
    share: f32,
    /// A amplitude da calote — ver [`amplitude`]. Mora aqui porque é constante por tabela, e um
    /// `exp` por pixel para a redescobrir seria três por pixel.
    amp: f32,
}

impl BoxPrefilter {
    /// Constrói a tabela de uma caixa `shape` que leva `share` da energia do céu.
    ///
    /// ⚠️ **A sequência é de Hammersley, logo determinística** — a tabela é a mesma em toda máquina e
    /// o gate não flaka.
    #[must_use]
    pub fn build(shape: Softbox, share: f32) -> Self {
        Self::build_with_samples(shape, share, SAMPLES)
    }

    /// [`Self::build`] com a contagem de amostras em aberto — a porta das sondas que mediram a
    /// const. ⚠️ Existe para a MEDIÇÃO ser feita pela mesma lei do produto, nunca por uma réplica.
    #[must_use]
    pub fn build_with_samples(shape: Softbox, share: f32, samples: u32) -> Self {
        let perfil = shape.profile();
        let mut spec = vec![0.0_f32; ROUGH_N * ANGLE_N];
        for ri in 0..ROUGH_N {
            let rough = ri as f64 / (ROUGH_N - 1) as f64;
            let a2 = (rough * rough) * (rough * rough);
            // As amostras do lóbulo, no referencial em que `N = V = R = +z`. Só `x` e `z` interessam:
            // a caixa entra no plano `x-z` e a lei é simétrica em `y`.
            let mut lobo: Vec<(f64, f64, f64)> = Vec::with_capacity(samples as usize);
            for i in 0..samples {
                let u1 = (f64::from(i) + 0.5) / f64::from(samples);
                let u2 = f64::from(i.reverse_bits()) / f64::from(u32::MAX);
                let ch2 = (1.0 - u1) / (1.0 + (a2 - 1.0) * u1);
                let ch = ch2.max(0.0).sqrt();
                let sh = (1.0 - ch2).max(0.0).sqrt();
                let phi = 2.0 * std::f64::consts::PI * u2;
                let h = [sh * phi.cos(), sh * phi.sin(), ch];
                let l = [2.0 * ch * h[0], 2.0 * ch * h[1], 2.0 * ch * h[2] - 1.0];
                if l[2] > 0.0 {
                    lobo.push((l[0], l[2], l[2]));
                }
            }
            let peso: f64 = lobo.iter().map(|s| s.2).sum();
            for ai in 0..ANGLE_N {
                let u = std::f64::consts::SQRT_2 * ai as f64 / (ANGLE_N - 1) as f64;
                let cos_psi = (1.0 - u * u).clamp(-1.0, 1.0);
                let sin_psi = (1.0 - cos_psi * cos_psi).max(0.0).sqrt();
                let soma: f64 = lobo
                    .iter()
                    .map(|&(lx, lz, w)| w * perfil.value(sin_psi * lx + cos_psi * lz))
                    .sum();
                spec[ri * ANGLE_N + ai] = (soma / peso) as f32;
            }
        }

        // O difuso: a mesma calote sob o lóbulo cosseno recortado. Grelha de Fibonacci sobre a
        // esfera, `E(n)/π` — e uma fonte uniforme devolve `1`, como no especular.
        const NS: usize = 40_000;
        let phi_d = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        let dirs: Vec<[f64; 3]> = (0..NS)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f64 + 0.5) / NS as f64;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let a = phi_d * i as f64;
                [r * a.cos(), y, r * a.sin()]
            })
            .collect();
        let mut diff = vec![0.0_f32; ANGLE_N];
        for (ai, d) in diff.iter_mut().enumerate() {
            let u = std::f64::consts::SQRT_2 * ai as f64 / (ANGLE_N - 1) as f64;
            let cos_psi = (1.0 - u * u).clamp(-1.0, 1.0);
            let sin_psi = (1.0 - cos_psi * cos_psi).max(0.0).sqrt();
            // A caixa em `+y`; a normal a `ψ` dela, no plano `x-y`.
            let n = [sin_psi, cos_psi, 0.0];
            let soma: f64 = dirs
                .iter()
                .map(|w| {
                    let cn = w[0] * n[0] + w[1] * n[1];
                    if cn <= 0.0 {
                        0.0
                    } else {
                        cn * perfil.value(w[1])
                    }
                })
                .sum();
            *d = (soma * 4.0 * std::f64::consts::PI / (NS as f64 * std::f64::consts::PI)) as f32;
        }
        Self {
            spec,
            diff,
            share,
            amp: (f64::from(share) * 4.0 * std::f64::consts::PI / shape.solid_angle()) as f32,
        }
    }

    /// A média da calote sobre o lóbulo GGX de rugosidade `α`, na direcção a `cos ψ` do eixo dela.
    #[must_use]
    pub fn specular(&self, alpha: f32, cos_psi: f32) -> f32 {
        let r = alpha.clamp(0.0, 1.0).sqrt() * (ROUGH_N - 1) as f32;
        let a = angle_axis(cos_psi) / core::f32::consts::SQRT_2 * (ANGLE_N - 1) as f32;
        let (r0, a0) = ((r as usize).min(ROUGH_N - 2), (a as usize).min(ANGLE_N - 2));
        let (fr, fa) = (r - r0 as f32, a - a0 as f32);
        let l = |ri: usize, ai: usize| self.spec[ri * ANGLE_N + ai];
        let baixo = l(r0, a0) + (l(r0, a0 + 1) - l(r0, a0)) * fa;
        let cima = l(r0 + 1, a0) + (l(r0 + 1, a0 + 1) - l(r0 + 1, a0)) * fa;
        baixo + (cima - baixo) * fr
    }

    /// As duas tabelas e as duas constantes, para quem as leva ao dispositivo — ver
    /// [`crate::studio_wgsl`].
    ///
    /// ⚠️ **Elas saem daqui e não de uma reconstrução**: a tabela é o produto de uma sequência de
    /// Hammersley com uma contagem de amostras MEDIDA, e refazê-la do outro lado seria uma segunda
    /// resposta à mesma pergunta.
    #[must_use]
    pub fn tables(&self) -> (&[f32], &[f32], f32, f32) {
        (&self.spec, &self.diff, self.share, self.amp)
    }

    /// A irradiância normalizada da calote, na normal a `cos ψ` do eixo dela.
    #[must_use]
    pub fn diffuse(&self, cos_psi: f32) -> f32 {
        let a = angle_axis(cos_psi) / core::f32::consts::SQRT_2 * (ANGLE_N - 1) as f32;
        let a0 = (a as usize).min(ANGLE_N - 2);
        let fa = a - a0 as f32;
        self.diff[a0] + (self.diff[a0 + 1] - self.diff[a0]) * fa
    }
}

/// A tabela do estúdio que o produto usa — construída uma vez, no primeiro quadro de Render.
static PREFILTER: LazyLock<BoxPrefilter> =
    LazyLock::new(|| BoxPrefilter::build(Softbox::PRODUCT, SOFTBOX_SHARE));

/// **O ESTÚDIO** — a rampa da [`ph2d_light`] mais a caixa de luz.
#[derive(Clone, Copy, Debug)]
pub struct Studio<'a> {
    /// `None` é a rampa nua — o céu de antes desta wave, ao bit.
    pub softbox: Option<&'a BoxPrefilter>,
}

impl Studio<'static> {
    /// O estúdio do produto.
    #[must_use]
    pub fn of_the_product() -> Self {
        Self {
            softbox: Some(&PREFILTER),
        }
    }

    /// A rampa nua, sem caixa nenhuma — o céu de antes desta wave.
    #[must_use]
    pub fn bare_ramp() -> Self {
        Self { softbox: None }
    }
}

impl Studio<'_> {
    /// **A radiância pré-filtrada**, em espaço de vista.
    #[must_use]
    pub fn radiance(&self, dir: [f32; 3], alpha: f32) -> Rgb {
        // O `ENV_SLOPE` da `ph2d-light` já vem convolvido com o lóbulo cosseno (`(2/3)·k`); a
        // radiância quer o `k` cru.
        const RAW: f32 = 1.5;
        // ⭐ **A MESMA associação da lei que ela substitui** — `up` primeiro, depois
        // `RAW * ENV_SLOPE[i] * up`. Trocar a ordem das multiplicações move o resultado UM ULP, e é
        // isso que separa «o céu de ontem» de «quase o céu de ontem» (há gate ao bit).
        let up = crate::render_light::lobe_shrink(alpha) * dir[1];
        // ⚠️ **O eixo da caixa é `+y`**, logo `cos ψ` é a própria componente `y` da direcção.
        let base = self
            .softbox
            .map_or(1.0, |t| 1.0 - t.share + t.amp * t.specular(alpha, dir[1]));
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT
                * (ph2d_light::ENV_BASE[i] * base + RAW * ph2d_light::ENV_SLOPE[i] * up)
        })
    }

    /// **A irradiância normalizada** — a mesma repartição, com o lóbulo cosseno.
    #[must_use]
    pub fn irradiance(&self, n: [f32; 3]) -> Rgb {
        // Vista (`y` para cima) → canvas (`y` para baixo): a porta da rampa mora na `ph2d-light` e
        // é a MESMA que a tinta e a escultura usam.
        let rampa = ph2d_light::env_ambient([n[0], -n[1], n[2]]);
        let Some(t) = self.softbox else {
            return rampa;
        };
        let delta = t.amp * t.diffuse(n[1]) - t.share;
        [0, 1, 2].map(|i| rampa[i] + ph2d_light::AMBIENT * ph2d_light::ENV_BASE[i] * delta)
    }
}

/// ⭐ **O estúdio É um céu** — e é esta a única implementação da lei; o
/// [`crate::render_light::StudioSky`] é a porta que o produto chama e não tem lei nenhuma dentro.
impl ph2d_material::Environment for Studio<'_> {
    fn radiance(&self, dir: [f32; 3], alpha: f32) -> Rgb {
        Studio::radiance(self, dir, alpha)
    }
    fn irradiance(&self, n: [f32; 3]) -> Rgb {
        Studio::irradiance(self, n)
    }
}

#[cfg(test)]
#[path = "studio_tests.rs"]
mod tests;

/// ⏱️ As SONDAS vivem no irmão, por assunto e pelo tecto de LOC — ver [`probes`].
#[cfg(test)]
#[path = "studio_probe_tests.rs"]
mod probes;
