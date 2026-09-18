#![forbid(unsafe_code)]
//! **A SUPERFÍCIE OpenPBR — a lei de REFERÊNCIA, em CPU.**
//!
//! O `open_pbr_surface` é o modelo de superfície que a Academy Software Foundation publicou e que o
//! MaterialX 1.39.5 implementa — o *Principled BSDF* que o dono apontou, na versão que Blender,
//! Autodesk e Adobe assinaram (`docs/Render3d/00`). Esta crate é o **port Apache-2.0 do GLSL de
//! referência do MaterialX**, com a proveniência escrita ficheiro a ficheiro em `bsdf.rs` e
//! `indirect.rs`.
//!
//! # ⚠️ Por que existe uma lei em CPU se o MaterialX GERA o shader
//!
//! O MaterialX gera código de **GPU** (e a ponte para WGSL está medida, `docs/Render3d/04` §2). Mas o
//! modelador traça o campo **na CPU** e sombreia lá — e a lei da casa para um efeito com dois
//! motores é *uma lei, dois motores, unidos por gate de paridade*. ⇒ esta é a referência, e o WGSL
//! gerado vai ser provado contra ela; ⛔ nunca uma terceira redacção à mão.
//!
//! # O subconjunto desta fatia, e o que fica de fora COM MOTIVO
//!
//! | entradas | aqui? | porquê |
//! |---|---|---|
//! | `base_weight` · `base_color` · `base_diffuse_roughness` · `base_metalness` | ✅ | |
//! | `specular_weight` · `specular_color` · `specular_roughness` · `specular_ior` | ✅ | |
//! | `coat_weight` · `coat_color` · `coat_roughness` · `coat_ior` · `coat_darkening` | ✅ | |
//! | `emission_luminance` · `emission_color` | ✅ | |
//! | `*_roughness_anisotropy` · `geometry_tangent` | ⛔ | o traçador não tem tangentes |
//! | `subsurface_*` · `geometry_thin_walled` | ✅ | **17/09** (`docs/Render3d/10`): os DOIS caminhos — a parede fina e a maciça |
//! | `transmission_*` · `fuzz_*` · `thin_film_*` · `geometry_opacity` | ⛔ nesta fatia | cada um é uma closure com gate próprio a escrever; com peso zero a composição gerada dá-lhes contribuição **zero** |
//!
//! ⛔ **Os de fora NÃO existem na struct**, e não por arrumação: um campo que a lei não lesse seria um
//! controlo morto à espera de um painel que o mostrasse.
//!
//! # A régua: o oráculo, corrido
//!
//! `fixtures/materialx_openpbr_direct.txt` é o `GlslRenderer` do MaterialX a desenhar a esfera dele
//! com o shader gerado, sem interface, e a guardar a normal, a vista e a cor de cada pixel
//! (`docs/Render3d/ferramentas/fixture_openpbr.py`). ⭐ Antes de este port existir, uma réplica em
//! `f64` da composição bateu as 945 amostras de luz directa a **`1,05e-5`** relativo — que é a
//! precisão `f32` da placa, e é o chão da barra dos gates.
//!
//! # Convenções
//!
//! - Todo vector é **unitário** e todos no **mesmo** referencial (qualquer um): `n` a normal, `v` a
//!   direcção **para o observador**, `to_light` a direcção **para a luz**.
//! - A saída é **radiância linear da cena** — sem exposição e sem vista (essas são da
//!   `ph2d-view-transform`).
//! - Uma luz direcional entra pela **radiância que chega** (`cor × intensidade`, a convenção do
//!   MaterialX): uma difusa branca de frente para uma luz de `π` devolve `1`.

mod bsdf;
mod indirect;
mod subsurface;

use bsdf::{Bsdf, V3};

/// RGB linear.
pub type Rgb = [f32; 3];

/// **Os parâmetros do artista** — os nomes e os valores padrão são os da nodedef
/// `ND_open_pbr_surface_surfaceshader` (há gate contra a própria nodedef, lida da fixture).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenPbr {
    pub base_weight: f32,
    pub base_color: Rgb,
    pub base_diffuse_roughness: f32,
    pub base_metalness: f32,
    pub specular_weight: f32,
    pub specular_color: Rgb,
    pub specular_roughness: f32,
    pub specular_ior: f32,
    pub coat_weight: f32,
    pub coat_color: Rgb,
    pub coat_roughness: f32,
    pub coat_ior: f32,
    pub coat_darkening: f32,
    pub emission_luminance: f32,
    pub emission_color: Rgb,
    /// ⭐ **Quanto da base é subsuperfície em vez de difusa** — as duas não somam, elas MISTURAM-SE
    /// (`mix` no grafo), logo `1` é subsuperfície pura. Ver [`crate::subsurface`].
    pub subsurface_weight: f32,
    /// A cor observada do meio que espalha.
    pub subsurface_color: Rgb,
    /// O comprimento do caminho livre médio, em unidades do MUNDO.
    pub subsurface_radius: f32,
    /// O multiplicador por canal do [`OpenPbr::subsurface_radius`] — é ele que faz o vermelho
    /// viajar mais fundo, que é o que dá a orelha acesa contra o sol.
    pub subsurface_radius_scale: Rgb,
    /// A fase: `0` espalha por igual, positivo para a frente, negativo para trás.
    pub subsurface_scatter_anisotropy: f32,
    /// ⭐⭐⭐ **A peça é uma PAREDE FINA?** — é este booleano que escolhe entre os dois caminhos da
    /// [`crate::subsurface`], e a escolha muda o fenómeno e não o grau.
    pub geometry_thin_walled: bool,
}

impl Default for OpenPbr {
    fn default() -> Self {
        Self {
            base_weight: 1.0,
            base_color: [0.8; 3],
            base_diffuse_roughness: 0.0,
            base_metalness: 0.0,
            specular_weight: 1.0,
            specular_color: [1.0; 3],
            specular_roughness: 0.3,
            specular_ior: 1.5,
            coat_weight: 0.0,
            coat_color: [1.0; 3],
            coat_roughness: 0.0,
            coat_ior: 1.6,
            coat_darkening: 1.0,
            emission_luminance: 0.0,
            emission_color: [1.0; 3],
            subsurface_weight: 0.0,
            subsurface_color: [0.8; 3],
            subsurface_radius: 1.0,
            subsurface_radius_scale: [1.0, 0.5, 0.25],
            subsurface_scatter_anisotropy: 0.0,
            geometry_thin_walled: false,
        }
    }
}

/// **O céu como fonte de luz**, visto por quem sombreia.
///
/// ⚠️ É a mesma divisão do método `PREFILTER` do MaterialX, e é por isso que a lei indirecta desta
/// crate se prova contra ele: o ambiente responde duas perguntas, e a superfície faz o resto.
pub trait Environment {
    /// A radiância que chega pela direcção `dir`, **já pré-filtrada** para um lóbulo GGX de
    /// rugosidade `alpha` (o `α` da NDF, não a rugosidade do artista).
    fn radiance(&self, dir: Rgb, alpha: f32) -> Rgb;
    /// A irradiância **normalizada** sobre a normal `n`, `E(n)/π` — a luz que uma difusa branca
    /// devolveria. Um céu uniforme de radiância `L` responde `L`.
    fn irradiance(&self, n: Rgb) -> Rgb;
}

/// **Um material pronto a sombrear** — as constantes que o grafo gerado calcula uma vez por
/// material, antes do laço das luzes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface {
    m: OpenPbr,
    /// O IOR que o `specular_weight` modula (`modulated_eta_s` no grafo).
    modulated_eta_s: f32,
    /// O `α` da reflexão principal, com o verniz a alargá-la.
    main_alpha: f32,
    coat_alpha: f32,
    coat_f0: f32,
    modulated_base_darkening: Rgb,
    coat_attenuation: Rgb,
    /// `subsurface_radius_scaled` do grafo — a escala por canal vezes o raio.
    subsurface_mfp: Rgb,
    /// `subsurface_color_nonnegative`.
    subsurface_colour: Rgb,
    /// `subsurface_thin_walled_brdf_factor` — ⚠️ sobre a cor **CRUA**, como o grafo a liga.
    thin_brdf_factor: Rgb,
    /// `subsurface_thin_walled_btdf_factor`.
    thin_btdf_factor: Rgb,
    /// ⭐⭐⭐ **A curvatura do ponto que se está a sombrear**, `1/raio`, em unidades do MUNDO.
    ///
    /// # ⚠️ Porque ela vive aqui, que é a struct do que é «por MATERIAL»
    ///
    /// Ela **não** é uma constante do material — é do PONTO. Vive aqui porque é exactamente onde o
    /// shader de referência a calcula: **antes** do laço das luzes, uma vez por pixel
    /// (`mx_subsurface_scattering_approx` deriva-a de `fwidth` no fragmento). ⇒ [`Surface`] é *o
    /// material NESTE ponto*, e quem sombreia um pixel chama [`Surface::at_curvature`] uma vez.
    ///
    /// ⚠️ **Zero é o valor de quem não a tem**, e ele é seguro por construção: o piso
    /// `max(curvature, 0.01)` do GLSL transforma-o num raio de `100` unidades de mundo — uma
    /// superfície praticamente plana, que é a leitura certa para *«não sei»*. ⛔ E com
    /// `subsurface_weight = 0` (a omissão) ninguém lê este campo.
    curvature: f32,
}

impl OpenPbr {
    /// Calcula as constantes do material — a parte do grafo gerado que não depende da luz.
    #[must_use]
    pub fn prepare(&self) -> Surface {
        let m = *self;
        // A reflexão especular vê o IOR RELATIVO ao verniz, com a correcção da reflexão interna total.
        let ratio = m.specular_ior / m.coat_ior;
        let tir_fix = if ratio > 1.0 {
            ratio
        } else {
            m.coat_ior / m.specular_ior
        };
        let eta_s = bsdf::mix(m.specular_ior, tir_fix, m.coat_weight);
        let scaled_f0 = (m.specular_weight * bsdf::ior_to_f0(eta_s)).clamp(0.0, 0.99999);
        let epsilon = glsl_sign(eta_s - 1.0) * scaled_f0.sqrt();
        let modulated_eta_s = (1.0 + epsilon) / (1.0 - epsilon);

        let coat_f0 = bsdf::ior_to_f0(m.coat_ior);
        let k_coat = 1.0 - (1.0 - coat_f0) / (m.coat_ior * m.coat_ior);
        let e_metal = bsdf::scale3(m.base_color, m.specular_weight);
        let e_base = bsdf::mix3(m.base_color, e_metal, m.base_metalness);
        let darkening = e_base.map(|e| (1.0 - k_coat) / (1.0 - e * k_coat));

        let coat_affected = (2.0 * m.coat_roughness.powf(4.0) + m.specular_roughness.powf(4.0))
            .min(1.0)
            .powf(0.25);
        let effective = bsdf::mix(m.specular_roughness, coat_affected, m.coat_weight);

        Surface {
            m,
            modulated_eta_s,
            main_alpha: isotropic_alpha(effective),
            coat_alpha: isotropic_alpha(m.coat_roughness),
            coat_f0,
            modulated_base_darkening: bsdf::mix3(
                [1.0; 3],
                darkening,
                m.coat_weight * m.coat_darkening,
            ),
            coat_attenuation: bsdf::mix3([1.0; 3], m.coat_color, m.coat_weight),
            subsurface_mfp: bsdf::scale3(m.subsurface_radius_scale, m.subsurface_radius),
            subsurface_colour: m.subsurface_color.map(|x| x.max(0.0)),
            thin_brdf_factor: bsdf::scale3(
                m.subsurface_color,
                1.0 - m.subsurface_scatter_anisotropy,
            ),
            thin_btdf_factor: bsdf::scale3(
                m.subsurface_color,
                1.0 + m.subsurface_scatter_anisotropy,
            ),
            curvature: 0.0,
        }
    }
}

/// O `sign` do GLSL — `0` em `0`, ao contrário do `f32::signum`.
fn glsl_sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// `NG_open_pbr_anisotropy` com anisotropia zero: `α = r² · √(2 / (1 + 1))`.
fn isotropic_alpha(roughness: f32) -> f32 {
    const ANISO_INVERT: f32 = 1.0;
    roughness * roughness * (2.0 / (ANISO_INVERT * ANISO_INVERT + 1.0)).sqrt()
}

/// De onde vem a luz que uma closure avalia.
pub(crate) enum Closure<'a> {
    /// Uma luz directa, pela direcção **para** ela.
    Reflection(V3),
    /// O céu.
    Indirect(&'a dyn Environment),
}

impl Surface {
    /// A radiância que uma **luz direcional** devolve para o observador.
    #[must_use]
    pub fn direct(&self, n: Rgb, v: Rgb, to_light: Rgb, radiance: Rgb) -> Rgb {
        bsdf::mul3(radiance, self.compose(n, v, &Closure::Reflection(to_light)))
    }

    /// ⭐⭐⭐ **A GÉMEA FOSCA — a mesma superfície sem o lóbulo especular.**
    ///
    /// # ⛔⛔⛔ Ela existe por uma MEDIÇÃO, e o report do dono está nela
    ///
    /// Um recolhedor de hemisfério (o ricochete de `ph2d_field_render::bounce`) mede a luz que a
    /// cena devolve somando `N` direcções FIXAS. O lóbulo especular de uma superfície é uma
    /// **quase-delta** em direcção, e `N = 48` não a amostra: ela cai numa direcção para um pixel e
    /// não para o vizinho.
    ///
    /// ⚠️⚠️ **E o sintoma não é «um pouco de ruído» — é o estimador deixar de CONVERGIR.** Medido na
    /// peça que o dono fotografou (o vaso da cena `=5`), a quebra de segunda diferença no `p99`:
    ///
    /// | direcções | com especular | **só difusa** |
    /// |---:|---:|---:|
    /// | `16` | `3,912` | **`0,969`** |
    /// | `32` | `2,457` | **`0,827`** |
    /// | `48` | `0,872` | **`0,588`** |
    /// | `96` | **`6,292`** | **`0,450`** |
    ///
    /// ⇒ com o especular a sequência é **caótica** (`96` direcções leem PIOR que `48`); sem ele é
    /// **monótona**, que é o que um estimador consistente faz. *Um refinamento que acumula direcções
    /// ao longo de quadros assentes precisa de que mais direcções signifiquem melhor.*
    ///
    /// ⚠️ **Divergência DECLARADA, e não um defeito:** a luz que sai de um ponto inclui mesmo o
    /// especular dele. O que este produto não faz é **transportá-lo** por um recolhedor difuso — é
    /// a mesma fronteira que o cabeçalho do `ph2d_field_render::bounce` já declara do lado de cá
    /// (*«a parte DIFUSA … o especular indirecto continua a ser o céu»*), agora também do lado de lá.
    ///
    /// ⭐ **Um METAL devolve ~nada por aqui, e isso está certo:** com `specular_weight = 0` a camada
    /// dele fica sem resposta, e um espelho não tem luz difusa para dar. A `base_metalness` **não**
    /// se mexe — zerá-la transformaria um metal numa superfície difusa da cor dele, inventando luz
    /// que não existe.
    #[must_use]
    pub fn matte(&self) -> Self {
        OpenPbr {
            specular_weight: 0.0,
            ..self.m
        }
        .prepare()
    }

    /// ⭐⭐⭐ **O mesmo material, NESTE ponto da peça** — a curvatura que a subsuperfície maciça lê.
    ///
    /// `curvature` é `1/raio` em unidades do MUNDO (para uma esfera de raio `R`, é `1/R`). Ver o doc
    /// do campo [`Surface::curvature`] para porque ela mora numa struct que se diz «por material», e
    /// o do [`crate::subsurface::thick`] para porque ela **não** pode vir de derivadas de ecrã nesta
    /// casa.
    ///
    /// ⚠️ Com `subsurface_weight = 0` ou com a peça declarada parede fina, ninguém a lê — e chamar
    /// isto é então byte-idêntico a não chamar.
    #[must_use]
    pub fn at_curvature(self, curvature: f32) -> Self {
        Self { curvature, ..self }
    }

    /// ⭐⭐⭐ **Este material LÊ a curvatura?** — a porta que decide se alguém paga por ela.
    ///
    /// Só o caminho MACIÇO da subsuperfície a lê: a parede fina é a lambertiana do lado de lá e não
    /// sabe nada sobre a forma da peça. ⇒ com a omissão (`subsurface_weight = 0`) a resposta é
    /// `false` e o traçador não gasta uma amostra de campo.
    ///
    /// ⚠️ **Ela é a MESMA pergunta que o `com_a_curvatura` do WGSL faz** (`ss_color_weight.a <= 0`
    /// ou `ss_brdf_thin.a > 0.5`), e é por isso que existe aqui em vez de em cada chamador: duas
    /// redacções divergiriam no dia em que a lei ganhasse um terceiro caminho.
    #[must_use]
    pub fn reads_curvature(&self) -> bool {
        self.m.subsurface_weight > 0.0 && !self.m.geometry_thin_walled
    }

    /// A radiância que o **céu** devolve para o observador.
    #[must_use]
    pub fn indirect(&self, n: Rgb, v: Rgb, env: &dyn Environment) -> Rgb {
        self.compose(n, v, &Closure::Indirect(env))
    }

    /// A **emissão** — a luz que a própria superfície dá, e que o verniz filtra pelo Fresnel dele.
    ///
    /// # ⭐ A guarda do zero, e porque ela é `== 0.0` e não `<= 0.0`
    ///
    /// Esta chamada corre **por amostra** e, com o material de omissão, faz um `forward_facing`, um
    /// `dot`, um `clamp` e um `powf(5)` para somar `[0,0,0]` — medido em `docs/Render3d/05` §8:
    /// **`4,6 %`** do relógio de sombreamento a produzir nada.
    ///
    /// ⚠️ **`== 0.0` é a única forma de a guarda ser byte-idêntica para TODA entrada.** Com
    /// `luminance == 0` o `scale3` zera as três componentes e tudo a jusante é uma multiplicação por
    /// zero, logo o resultado é `[0,0,0]` por construção — a guarda **observa** a álgebra, não a
    /// muda. ⛔ Um `<= 0.0` **mudaria** a resposta para uma luminância negativa, e esta crate é o
    /// **port fiel** do GLSL de referência: uma entrada fora da faixa da nodedef é assunto de quem
    /// autora, e não desta lei.
    #[must_use]
    pub fn emission(&self, n: Rgb, v: Rgb) -> Rgb {
        let m = &self.m;
        if m.emission_luminance == 0.0 {
            return [0.0; 3];
        }
        let uncoated = bsdf::scale3(m.emission_color, m.emission_luminance);
        let nf = bsdf::forward_facing(n, v);
        let ndv = bsdf::dot(nf, v).clamp(bsdf::EPS, 1.0);
        let x = (1.0 - ndv).clamp(0.0, 1.0).powf(5.0);
        let fresnel = bsdf::mix3([1.0 - self.coat_f0; 3], [0.0; 3], x);
        let coated = bsdf::mul3(bsdf::mul3(uncoated, m.coat_color), fresnel);
        bsdf::mix3(uncoated, coated, m.coat_weight)
    }

    /// ⭐⭐⭐ **A subsuperfície escolhida** — o `selected_subsurface` do grafo.
    ///
    /// # ⭐ A guarda do zero, e porque ela OBSERVA a álgebra em vez de a mudar
    ///
    /// Com `subsurface_weight == 0` o `mix` a jusante devolve a difusa **ao bit**
    /// (`bg + (fg − bg)·0 = bg`), logo tudo o que esta função produzisse era multiplicado por zero.
    /// ⇒ sair cedo é byte-idêntico, e poupa o laço de `32` termos do Burley em **todo** material que
    /// não pediu subsuperfície — que é o de omissão. ⚠️ É a mesma guarda que a [`Surface::emission`]
    /// já declara, pela mesma razão medida: aquela custava `4,6 %` do relógio a produzir `[0,0,0]`.
    ///
    /// # ⚠️ E o SELECTOR é um ramo, não um `mix` — o que isso compra, e o que o torna legítimo
    ///
    /// O grafo escreve `mix(fg = parede fina, bg = maciça, mix = float(thin_walled))` e **avalia as
    /// duas**. Com o selector a valer exactamente `0` ou `1`, o `mix` devolve um dos lados ao bit,
    /// logo o ramo dá o mesmo número — e poupa o Burley em toda peça de parede fina.
    ///
    /// ⛔ **Isto só é verdade porque nenhum dos lados pode ser `NaN` ou `Inf`:** `x + (NaN − x)·0`
    /// é `NaN`, não `x`. É o `acos` cortado da [`crate::subsurface`] que o garante, e há gate a
    /// medir o ramo contra o `mix` sobre uma grelha.
    fn subsurface(&self, n: V3, v: V3, c: &Closure<'_>) -> Bsdf {
        let m = &self.m;
        if m.subsurface_weight == 0.0 {
            return Bsdf {
                response: [0.0; 3],
                throughput: [0.0; 3],
            };
        }
        if m.geometry_thin_walled {
            let reflection = bsdf::mul_color(
                diffuse(
                    1.0,
                    self.subsurface_colour,
                    m.base_diffuse_roughness,
                    n,
                    v,
                    c,
                    false,
                ),
                self.thin_brdf_factor,
            );
            let transmission = bsdf::mul_color(
                subsurface::translucent(1.0, self.subsurface_colour, n, c),
                self.thin_btdf_factor,
            );
            subsurface::mix_bsdf(reflection, transmission, 0.5)
        } else {
            subsurface::thick(
                1.0,
                self.subsurface_colour,
                self.subsurface_mfp,
                self.curvature,
                n,
                v,
                c,
            )
        }
    }

    /// **A composição do grafo gerado**, closure a closure, na ordem em que o shader a escreve.
    ///
    /// ⚠️ Os ramos com peso zero desta fatia (película fina, transmissão, subsuperfície, fuzz) ficam
    /// ESCRITOS como o grafo os deixa — um `Bsdf::NONE` ou uma soma com resposta zero —, porque o
    /// throughput deles entra nas somas (`mx_add_bsdf`) e reescrever a álgebra «simplificada» mudaria
    /// um número que o oráculo mede.
    fn compose(&self, n: V3, v: V3, c: &Closure<'_>) -> V3 {
        let m = &self.m;
        let metal = bsdf::add(
            Bsdf::NONE,
            schlick(
                m.specular_weight,
                bsdf::scale3(m.base_color, m.base_weight),
                m.specular_color,
                self.main_alpha,
                n,
                v,
                c,
            ),
        );
        let base_fg = bsdf::mul_float(metal, m.base_metalness);

        let dielectric_reflection = bsdf::add(
            Bsdf::NONE,
            dielectric(
                1.0,
                m.specular_color,
                self.modulated_eta_s,
                self.main_alpha,
                n,
                v,
                c,
            ),
        );
        // A transmissão com peso zero: resposta zero, throughput um.
        let transmission = bsdf::mul_float(Bsdf::NONE, 0.0);
        let base_colour = m.base_color.map(|x| x.max(0.0));
        let diffuse = diffuse(
            m.base_weight,
            base_colour,
            m.base_diffuse_roughness,
            n,
            v,
            c,
            true,
        );
        let opaque = subsurface::mix_bsdf(self.subsurface(n, v, c), diffuse, m.subsurface_weight);
        let substrate = bsdf::add(transmission, bsdf::mul_float(opaque, 1.0));
        let dielectric_base = bsdf::layer(dielectric_reflection, substrate);

        let base_substrate = bsdf::add(
            base_fg,
            bsdf::mul_float(dielectric_base, 1.0 - m.base_metalness),
        );
        let attenuated = bsdf::mul_color(
            bsdf::mul_color(base_substrate, self.modulated_base_darkening),
            self.coat_attenuation,
        );
        let coat = dielectric(
            m.coat_weight,
            [1.0; 3],
            m.coat_ior,
            self.coat_alpha,
            n,
            v,
            c,
        );
        let coat_layer = bsdf::layer(coat, attenuated);
        bsdf::layer(Bsdf::NONE, coat_layer).response
    }
}

fn dielectric(weight: f32, tint: V3, ior: f32, alpha: f32, n: V3, v: V3, c: &Closure<'_>) -> Bsdf {
    match c {
        Closure::Reflection(l) => bsdf::dielectric_reflection(weight, tint, ior, alpha, n, v, *l),
        Closure::Indirect(env) => indirect::dielectric(weight, tint, ior, alpha, n, v, *env),
    }
}

/// O metal: `generalized_schlick` com `color90 = 1` e expoente `5`, como o grafo o liga.
fn schlick(
    weight: f32,
    color0: V3,
    color82: V3,
    alpha: f32,
    n: V3,
    v: V3,
    c: &Closure<'_>,
) -> Bsdf {
    const COLOR90: V3 = [1.0; 3];
    const EXPONENT: f32 = 5.0;
    match c {
        Closure::Reflection(l) => {
            bsdf::schlick_reflection(weight, color0, color82, COLOR90, EXPONENT, alpha, n, v, *l)
        }
        Closure::Indirect(env) => indirect::schlick(
            weight, color0, color82, COLOR90, EXPONENT, alpha, n, v, *env,
        ),
    }
}

/// O Oren-Nayar, com a compensação de energia a ser um ARGUMENTO — ver
/// [`bsdf::oren_nayar_plain`]: o grafo liga-a na base e deixa-a por escrever na parede fina da
/// subsuperfície, onde a nodedef lhe dá `false`.
fn diffuse(
    weight: f32,
    color: V3,
    roughness: f32,
    n: V3,
    v: V3,
    c: &Closure<'_>,
    energy_compensation: bool,
) -> Bsdf {
    match c {
        Closure::Reflection(l) => {
            bsdf::oren_nayar_reflection(weight, color, roughness, n, v, *l, energy_compensation)
        }
        Closure::Indirect(env) => {
            indirect::oren_nayar(weight, color, roughness, n, v, *env, energy_compensation)
        }
    }
}

pub mod wgsl;

#[cfg(test)]
mod tests;
