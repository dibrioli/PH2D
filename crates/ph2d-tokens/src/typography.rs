//! Typography scale tokens. Source: `docs/design/tokens.json` →
//! `typography.*`.
//!
//! Pixel sizes assume 1.0 device pixel ratio; widget code multiplies
//! by `dpr` at render time. OS text-scaling override (up to 200 %) is
//! honored at the same step.
//!
//! Canonical families: **Inter** (variable sans, with `Inter Display`
//! for titles) + **JetBrains Mono** (mono). Fallbacks below follow
//! the design stack (Apple → BlinkMacSystemFont → system-ui).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeToken {
    /// `xxs` — 10 px (badges, micro-tooltips).
    Xxs,
    /// `xs` — 11 px (status bar, mini chips).
    Xs,
    /// `sm` — 12 px (sidebar labels, dense lists).
    Sm,
    /// `base` — 13 px (default body, inputs).
    Base,
    /// `md` — 15 px (panel section headers).
    Md,
    /// `lg` — 18 px (page subheaders).
    Lg,
    /// `xl` — 24 px (page headers).
    Xl,
    /// `xl2` (`2xl` in JSON) — 32 px (section heroes).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 44 px (welcome screen hero).
    Xl3,
}

impl TypeToken {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::TYPOGRAPHY_SIZE_XXS,
            Self::Xs => crate::generated::TYPOGRAPHY_SIZE_XS,
            Self::Sm => crate::generated::TYPOGRAPHY_SIZE_SM,
            Self::Base => crate::generated::TYPOGRAPHY_SIZE_BASE,
            Self::Md => crate::generated::TYPOGRAPHY_SIZE_MD,
            Self::Lg => crate::generated::TYPOGRAPHY_SIZE_LG,
            Self::Xl => crate::generated::TYPOGRAPHY_SIZE_XL,
            Self::Xl2 => crate::generated::TYPOGRAPHY_SIZE_XL2,
            Self::Xl3 => crate::generated::TYPOGRAPHY_SIZE_XL3,
        }
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Base => "base",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
        }
    }
}

/// Font weight tokens (matches JSON `weight.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl FontWeight {
    pub const fn value(self) -> u16 {
        match self {
            Self::Regular => crate::generated::TYPOGRAPHY_WEIGHT_REGULAR,
            Self::Medium => crate::generated::TYPOGRAPHY_WEIGHT_MEDIUM,
            Self::Semibold => crate::generated::TYPOGRAPHY_WEIGHT_SEMIBOLD,
            Self::Bold => crate::generated::TYPOGRAPHY_WEIGHT_BOLD,
        }
    }
}

/// Line-height multipliers (matches JSON `line.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LineHeight {
    Tight,
    Snug,
    Normal,
    Loose,
}

impl LineHeight {
    pub const fn ratio(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_LINE_TIGHT,
            Self::Snug => crate::generated::TYPOGRAPHY_LINE_SNUG,
            Self::Normal => crate::generated::TYPOGRAPHY_LINE_NORMAL,
            Self::Loose => crate::generated::TYPOGRAPHY_LINE_LOOSE,
        }
    }
}

/// Letter-spacing tokens (matches JSON `track.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetterSpacing {
    Tight,
    Normal,
    Wide,
    Caps,
}

impl LetterSpacing {
    /// Em units (matches CSS `letter-spacing: Xem`).
    pub const fn em(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_TRACK_TIGHT,
            Self::Normal => crate::generated::TYPOGRAPHY_TRACK_NORMAL,
            Self::Wide => crate::generated::TYPOGRAPHY_TRACK_WIDE,
            Self::Caps => crate::generated::TYPOGRAPHY_TRACK_CAPS,
        }
    }
}

/// Font family chains. Strings constantes para uso direto em parley
/// font stack lookups (parley aceita comma-separated fallback chain).
pub const FONT_SANS: &str =
    "Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif";
pub const FONT_DISPLAY: &str = "'Inter Display', Inter, system-ui, sans-serif";
pub const FONT_MONO: &str = "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace";

/// Text-rendering strategy. Orthogonal to [`crate::Theme`] — qualquer
/// combinação Theme × TextRendering é válida. Lida via thread-local
/// `paint::text_rendering()` dentro de `paint_text*` em
/// `ph2d-editor-core`.
///
/// Três presets:
///
/// - `Default` — pipeline histórico: snap-Y only, sem snap-X, hint=true,
///   sem boost.
/// - `CrispHeavy` — snap-X integral + boost +300/+200/+150
///   (Medium 500 → ExtraBold 800 a 11 px) **com hint OFF**: deixa o
///   eixo variable wght fluir sem quantização → letras visivelmente
///   mais cheias, qualidade "Linear/Notion-pro".
/// - `CrispHeavyPlus` — variação experimental de CrispHeavy (2026-05-25
///   "tempero final"): mesmo boost + hint=off de CrispHeavy, mas com
///   **half-pixel snap-X** (preserva ~50 % do kerning) e
///   **letter-spacing -0.01 em em corpos ≤16 px** (aperta a densidade
///   que ExtraBold abre). Pensada para A/B test contra `CrispHeavy` —
///   se ficar perceptualmente melhor, promove pra default e descarta o
///   canonical `CrispHeavy`.
///
/// ⛔ **Um preset de texto NÃO escolhe o anti-aliasing do passe.** O
/// `CrispHeavyPlus` tinha um 4.º ingrediente (`Msaa16` no Vello pass) e
/// ele foi retirado em 2026-08-30 — a nota no fim de
/// [`TextRenderingParams`] diz o mecanismo. Nenhum preset novo pode
/// trazê-lo de volta.
///
/// Adicionar preset = (a) novo variant aqui, (b) caso novo em
/// [`Self::params`] / [`Self::id`] / [`Self::display_name`] / [`Self::next`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextRendering {
    /// Pipeline histórico — Vello AA analítico + snap-Y, sem boost,
    /// sem snap-X.
    #[default]
    Default,
    /// Snap-X + hint **OFF** + boost +300/+200/+150 → ExtraBold @ 11 px,
    /// axis variable flowing sem quantização do autohinter.
    CrispHeavy,
    /// Experimental: CrispHeavy + half-pixel snap-X + letter-spacing
    /// -0.01em em corpos ≤16 px. A/B vs CrispHeavy.
    CrispHeavyPlus,
    /// ⭐⭐ **O que o Vello 0.10 traz e os outros três NÃO conseguem exprimir:**
    /// engrossar o contorno **só no eixo X**.
    ///
    /// Os presets acima pedem massa ao **eixo `wght` da fonte** — e um eixo de peso
    /// engorda os traços verticais **e** os horizontais juntos, porque foi assim que o
    /// desenhador o desenhou. A `Scene::font_embolden` dilata a **outline** depois de
    /// desenhada, com `x` e `y` independentes (`Diagonal2`) ⇒ dá para engrossar as
    /// hastes verticais — que é o que sustenta a legibilidade a 11-12 px — **sem**
    /// engordar as barras horizontais, que é o que faz o texto pequeno fechar os olhos
    /// das letras.
    ///
    /// ⚠️ **E funciona em fonte NÃO-variável**, onde o `weight_boost` é literalmente
    /// inerte: sem eixo `wght` não há o que empurrar.
    ///
    /// ⚠️ **O número é PIXELS, não ems** — a dilatação corre sobre a outline já
    /// escalada ao corpo do run (`glyph_cache.rs`: `DrawSettings::unhinted(size)` e
    /// depois `expand_path(amount)`) ⇒ o mesmo valor engrossa relativamente **mais** o
    /// texto pequeno, que é a direcção certa, e é preciso ser minúsculo.
    ///
    /// ⛔ **O valor de partida NÃO está medido, e isso está dito de propósito.** Não há
    /// como o medir aqui: o arnês de teste desta casa não carrega fontes, e texto sem
    /// glifos não produz tinta nenhuma. Quem o mede é o dono, na bancada, a olhar.
    CrispEmbolden,
}

/// Estratégia de snap horizontal do glyph origin. Trade-off entre
/// preservar kerning subpixel (None) e alinhar stems ao pixel grid (Full).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapX {
    /// Sem snap — `g.x` fica fracionário. Preserva 100 % do kerning.
    None,
    /// Snap a 0.5 px — preserva ~50 % do kerning, ganha ~80 % do snap.
    Half,
    /// Snap a 1 px (inteiro) — perde kerning subpixel, stems alinhados.
    Full,
}

/// Parâmetros derivados de [`TextRendering`] consumidos pelo painter
/// de texto e pelo `effective_weight` em `ph2d-text`. Cada preset
/// expõe o mesmo formato → caller é independente do número de variants.
// PartialEq only (not Eq) because `letter_spacing_em_dense: f32` is
// not `Eq` (float NaN semantics). Compare via `==` is fine for our
// purposes; we never use TextRenderingParams as a HashMap key.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextRenderingParams {
    /// Weight bump (CSS units) aplicado a corpos ≤12 px.
    pub weight_boost_body: u16,
    /// Weight bump aplicado em 12 < size ≤ 16 px (dense lists).
    pub weight_boost_dense: u16,
    /// Weight bump aplicado em 16 < size ≤ 20 px (mid headings).
    pub weight_boost_mid: u16,
    /// Estratégia de snap horizontal da origem do glyph.
    pub snap_x: SnapX,
    /// Quando `true`, passa `hint(true)` para o Vello — autohinter
    /// skrifa snapa stems ao pixel grid (mais "crisp" mas QUANTIZA
    /// diferenças de wght a 11-12 px: presets que diferem por <1 px
    /// de massa de stem ficam visualmente idênticos pós-hint).
    /// Quando `false`, deixa o eixo wght variable fluir livremente
    /// — strokes ficam ligeiramente mais "soft" mas a variação entre
    /// presets é visível.
    pub hint: bool,
    /// Letter-spacing (em ems) aplicado a corpos ≤16 px. Negativo
    /// aperta a densidade horizontal — útil em presets com ExtraBold
    /// para compensar o "abrir" natural do weight. `0.0` = sem ajuste.
    pub letter_spacing_em_dense: f32,
    /// **Dilatação sintética da outline, em PIXELS**, `(x, y)` — o `font_embolden` do
    /// Vello 0.10. `(0.0, 0.0)` desliga-a, e é o que os três presets históricos
    /// declaram: com zero, o Vello nem sequer entra no caminho da expansão
    /// (`glyph_cache.rs` compara contra `Diagonal2::new(0,0)`), logo eles ficam
    /// **byte-idênticos** ao que shipava. Há gate a afirmá-lo.
    pub embolden_px: (f32, f32),
    //
    // ⛔ AQUI VIVIA `prefer_msaa: bool` (removido 2026-08-30). NÃO o
    // reintroduza, nem com outro nome.
    //
    // O `AaConfig` do Vello é escolhido **por PASSE**, não por caminho
    // de desenho: `RenderParams::antialiasing_method` vale para tudo o
    // que aquele `Scene` contém. O nosso passe de chrome carrega o
    // chrome **e** a arte vectorial do documento no MESMO `Scene`, logo
    // uma preferência de TEXTO escolhia o AA dos VECTORES do artista.
    //
    // O preço estava escrito no nosso próprio código, ao lado da
    // bandeira que o ligava: «MSAA16 produced visible stippling on thin
    // (1-1.5 px) vector strokes at near-axis angles»
    // (`ph2d-render/src/vello_pass.rs`). O `CrispHeavyPlus` ligava
    // `Msaa16` e a justificação concedia o defeito na mesma frase
    // («the problem case is vectors») — certa sobre os glifos, cega
    // quanto a `AaConfig` ser por passe. Report do dono do produto:
    // «manchas animadas parecendo TV antiga» à volta das formas
    // vectoriais (`docs/Atualizar Stack/04_registro.md` §22.2).
    //
    // ⇒ a arte do documento manda no passe (`AaConfig::Area`, que é
    // analítico, melhor em traço fino e mais barato), e este preset
    // fica com os três ingredientes que são de facto sobre texto:
    // snap-X, letter-spacing denso e o boost de peso.
    //
    // ⚠️ A alternativa real — chrome e documento em DOIS passes, cada
    // um com o seu `AaConfig` — é arquitectura (segundo alvo, segundo
    // `Renderer`, composição), **não** uma bandeira neste struct.
}

/// Tier limits para [`crisp_weight_boost_for`] (px). Discrete tiers
/// evitam micro-shifts visíveis no Inter variable weight axis.
const CRISP_BOOST_TIER_BODY_MAX: f32 = 12.0;
const CRISP_BOOST_TIER_DENSE_MAX: f32 = 16.0;
const CRISP_BOOST_TIER_MID_MAX: f32 = 20.0;

impl TextRendering {
    /// ⭐ **Todos os modos, na ordem do ciclo do menu** — a lista canónica, como o
    /// `SliderDesign::ALL` e o `PropertyBoxState::ALL`.
    ///
    /// ⚠️ Ela existe porque a alternativa mordeu: o teste do ciclo tinha a contagem **no nome**
    /// (`..._cycles_three_states`) e o corpo escrito à mão, então o 4.º preset **partiu o teste de
    /// outra pessoa** só por existir. *Uma contagem literal num gate faz cada feature nova editar o
    /// teste de alguém.* Agora os gates derivam daqui, e um variant que não venha a esta lista é
    /// apanhado pelo ciclo.
    pub const ALL: [Self; 4] = [
        Self::Default,
        Self::CrispHeavy,
        Self::CrispHeavyPlus,
        Self::CrispEmbolden,
    ];

    /// Cycle entre as opções (toggle do menu).
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::CrispHeavy,
            Self::CrispHeavy => Self::CrispHeavyPlus,
            Self::CrispHeavyPlus => Self::CrispEmbolden,
            Self::CrispEmbolden => Self::Default,
        }
    }

    /// Stable identifier (matches future tokens.json key).
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::CrispHeavy => "crisp_heavy",
            Self::CrispHeavyPlus => "crisp_heavy_plus",
            Self::CrispEmbolden => "crisp_embolden",
        }
    }

    /// Human-readable display name (menu items).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::CrispHeavy => "Crisp Heavy",
            Self::CrispHeavyPlus => "Crisp Heavy +",
            Self::CrispEmbolden => "Crisp Embolden",
        }
    }

    /// Parâmetros do preset (boost tiers + snap-X flag). Único lugar
    /// onde cada preset declara seu shape — `effective_weight` e
    /// `paint_text*` consomem isso sem conhecer os variants
    /// individualmente.
    pub const fn params(self) -> TextRenderingParams {
        match self {
            Self::Default => TextRenderingParams {
                weight_boost_body: 0,
                weight_boost_dense: 0,
                weight_boost_mid: 0,
                snap_x: SnapX::None,
                hint: true,
                letter_spacing_em_dense: 0.0,
                // Os três presets históricos ficam byte-idênticos: com zero o Vello nem
                // entra no caminho da expansão de outline.
                embolden_px: (0.0, 0.0),
            },
            Self::CrispHeavy => TextRenderingParams {
                weight_boost_body: 300,
                weight_boost_dense: 200,
                weight_boost_mid: 150,
                snap_x: SnapX::Full,
                // hint=false aqui é deliberado: a 11-12 px o autohinter
                // do skrifa colapsa diferenças de wght <1 px ao mesmo
                // pixel grid; desligar libera o eixo variable a fluir
                // → CrispHeavy fica visualmente "pro" (ExtraBold real).
                hint: false,
                letter_spacing_em_dense: 0.0,
                // Os três presets históricos ficam byte-idênticos: com zero o Vello nem
                // entra no caminho da expansão de outline.
                embolden_px: (0.0, 0.0),
            },
            Self::CrispHeavyPlus => TextRenderingParams {
                // Mesmo boost de CrispHeavy.
                weight_boost_body: 300,
                weight_boost_dense: 200,
                weight_boost_mid: 150,
                // Half-pixel snap — preserva ~50 % do kerning vs Full.
                snap_x: SnapX::Half,
                hint: false,
                // Aperta densidade dos corpos pequenos compensando o
                // "abrir" do ExtraBold.
                letter_spacing_em_dense: -0.01,
                // Os três presets históricos ficam byte-idênticos: com zero o Vello nem
                // entra no caminho da expansão de outline.
                embolden_px: (0.0, 0.0),
                // ⛔ Este preset ligava `AaConfig::Msaa16` no passe do
                // Vello. Retirado em 2026-08-30 — ver a nota longa no
                // fim de `TextRenderingParams`: o `AaConfig` é por
                // PASSE, e este passe também carrega os vectores.
            },
            Self::CrispEmbolden => TextRenderingParams {
                // ⚠️ **Sem boost de peso, de propósito.** Este preset existe para
                // responder a UMA pergunta — *o que a dilatação de outline faz que o
                // eixo `wght` não faz?* — e somá-la ao boost misturaria as duas causas.
                // Se ganhar, a composição é a wave seguinte.
                weight_boost_body: 0,
                weight_boost_dense: 0,
                weight_boost_mid: 0,
                // Snap inteiro: a dilatação é sobre a outline, e uma origem fraccionária
                // borra exactamente a nitidez que ela vem comprar.
                snap_x: SnapX::Full,
                // `hint: false` pela mesma razão do CrispHeavy — o autohinter quantiza
                // massa de haste a 11-12 px, e apagaria a diferença que este preset É.
                hint: false,
                letter_spacing_em_dense: 0.0,
                // ⛔ **PALPITE DECLARADO, não medição.** `0,2 px` no X é da ordem de
                // meia haste a 12 px; o Y fica a zero porque é a barra horizontal que
                // fecha os olhos das letras no corpo pequeno. Não há como o medir aqui
                // (o arnês não tem fontes), então o número é o ponto de partida para o
                // dono olhar na bancada — e é ele que o fixa.
                embolden_px: (0.2, 0.0),
            },
        }
    }
}

/// FontWeight boost (CSS units) dado os params do preset + tamanho
/// renderizado. Returns 0 acima de 20 px (boost não traz ganho
/// perceptual nesse range, independente do preset).
///
/// Caller soma o resultado ao [`FontWeight`] nominal e clampa a
/// `[100, 900]` (skrifa também enforça downstream).
pub const fn crisp_weight_boost_for(params: TextRenderingParams, font_size_px: f32) -> u16 {
    if font_size_px <= CRISP_BOOST_TIER_BODY_MAX {
        params.weight_boost_body
    } else if font_size_px <= CRISP_BOOST_TIER_DENSE_MAX {
        params.weight_boost_dense
    } else if font_size_px <= CRISP_BOOST_TIER_MID_MAX {
        params.weight_boost_mid
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_strictly_increasing() {
        let scale = [
            TypeToken::Xxs,
            TypeToken::Xs,
            TypeToken::Sm,
            TypeToken::Base,
            TypeToken::Md,
            TypeToken::Lg,
            TypeToken::Xl,
            TypeToken::Xl2,
            TypeToken::Xl3,
        ];
        for w in scale.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn weight_values_match_css_standard() {
        assert_eq!(FontWeight::Regular.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
    }

    #[test]
    fn line_height_ratios_in_typography_norm() {
        assert!(LineHeight::Tight.ratio() < LineHeight::Normal.ratio());
        assert!(LineHeight::Normal.ratio() < LineHeight::Loose.ratio());
    }

    #[test]
    fn text_rendering_default_is_default_variant() {
        assert_eq!(TextRendering::default(), TextRendering::Default);
    }

    /// **O ciclo do menu percorre a lista canónica e volta ao princípio.**
    ///
    /// ⚠️ **Derivado do [`TextRendering::ALL`], nunca escrito à mão.** A 1.ª redacção chamava-se
    /// `..._cycles_three_states` e enumerava os três — então o 4.º preset partiu-a só por existir,
    /// e a correcção «óbvia» (trocar três por quatro) deixaria a mesma armadilha armada para o 5.º.
    #[test]
    fn text_rendering_cycles_through_every_mode_and_returns() {
        let mut cur = TextRendering::ALL[0];
        for expected in TextRendering::ALL.iter().skip(1) {
            cur = cur.next();
            assert_eq!(
                cur, *expected,
                "o ciclo saltou ou trocou a ordem de um modo"
            );
        }
        assert_eq!(
            cur.next(),
            TextRendering::ALL[0],
            "o ciclo nao volta ao principio: o ultimo modo e' um beco sem saida no menu"
        );
    }

    #[test]
    fn text_rendering_ids_stable() {
        assert_eq!(TextRendering::Default.id(), "default");
        assert_eq!(TextRendering::CrispHeavy.id(), "crisp_heavy");
        assert_eq!(TextRendering::CrispHeavyPlus.id(), "crisp_heavy_plus");
    }

    #[test]
    fn text_rendering_display_names() {
        assert_eq!(TextRendering::Default.display_name(), "Default");
        assert_eq!(TextRendering::CrispHeavy.display_name(), "Crisp Heavy");
        assert_eq!(
            TextRendering::CrispHeavyPlus.display_name(),
            "Crisp Heavy +"
        );
    }

    #[test]
    fn text_rendering_default_params_are_identity() {
        let p = TextRendering::Default.params();
        assert_eq!(p.weight_boost_body, 0);
        assert_eq!(p.weight_boost_dense, 0);
        assert_eq!(p.weight_boost_mid, 0);
        assert_eq!(p.snap_x, SnapX::None);
        assert!(p.hint);
        assert_eq!(p.letter_spacing_em_dense, 0.0);
    }

    #[test]
    fn crisp_heavy_params_are_pro_quality() {
        let p = TextRendering::CrispHeavy.params();
        assert_eq!(p.weight_boost_body, 300);
        assert_eq!(p.weight_boost_dense, 200);
        assert_eq!(p.weight_boost_mid, 150);
        assert_eq!(p.snap_x, SnapX::Full);
        assert!(
            !p.hint,
            "CrispHeavy MUST disable hint — see typography docstring"
        );
    }

    #[test]
    fn crisp_heavy_plus_differs_from_heavy_in_two_axes() {
        // The whole point of CrispHeavyPlus: same boost as CrispHeavy
        // but 2 axes flipped to A/B-test the "tempero final".
        //
        // ⚠️ Eram TRÊS até 2026-08-30. O terceiro era `prefer_msaa`, e
        // ele não era sobre texto: escolhia o `AaConfig` do PASSE
        // inteiro, vectores do artista incluídos. Ver a nota no fim de
        // `TextRenderingParams`.
        let h = TextRendering::CrispHeavy.params();
        let p = TextRendering::CrispHeavyPlus.params();
        // Same boost (so the A/B isolates the 3 changes, not the weight).
        assert_eq!(p.weight_boost_body, h.weight_boost_body);
        assert_eq!(p.weight_boost_dense, h.weight_boost_dense);
        assert_eq!(p.weight_boost_mid, h.weight_boost_mid);
        // 2 axes flipped.
        assert_ne!(p.snap_x, h.snap_x, "Plus must use Half snap, not Full");
        assert_eq!(p.snap_x, SnapX::Half);
        assert!(p.letter_spacing_em_dense < 0.0, "Plus tightens body");
    }

    #[test]
    fn crisp_weight_boost_monotonically_decreases_in_size() {
        // For all presets, boost is non-increasing as size grows.
        let sizes = [10.0_f32, 11.0, 12.0, 13.0, 15.0, 17.0, 19.0, 22.0, 32.0];
        for preset in [
            TextRendering::Default,
            TextRendering::CrispHeavy,
            TextRendering::CrispHeavyPlus,
        ] {
            let params = preset.params();
            let mut prev = u16::MAX;
            for s in sizes {
                let b = crisp_weight_boost_for(params, s);
                assert!(b <= prev, "preset={preset:?} size={s} prev={prev} curr={b}");
                prev = b;
            }
        }
    }

    #[test]
    fn crisp_weight_boost_typetoken_coverage_for_heavy() {
        // Anchor the tiers against the real TypeToken sizes for the
        // canonical CrispHeavy preset.
        let p = TextRendering::CrispHeavy.params();
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xxs.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xs.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Sm.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Base.px()), 200);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Md.px()), 200);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Lg.px()), 150);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl.px()), 0);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl2.px()), 0);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl3.px()), 0);
    }

    #[test]
    fn crisp_weight_boost_default_preset_always_zero() {
        // Default's params produce 0 boost at every size — the preset
        // is a true identity.
        let p = TextRendering::Default.params();
        for s in [10.0_f32, 11.0, 15.0, 20.0, 32.0, 44.0] {
            assert_eq!(crisp_weight_boost_for(p, s), 0);
        }
    }
}
