//! **O SUBSTRATO ACENDE** — o dente do papel vira uma superfície com relevo, para qualquer meio.
//!
//! ## Por que isto existe (Enio, 2026-08-10)
//!
//! O emboss do Wet Paint imprime o dente do papel DENTRO da cor do pigmento e é a coisa que faz uma
//! aguada parecer que assentou sobre papel. O Digital não tinha nada equivalente — e a sonda
//! [`super::emboss_probe`] mediu por quê: **tinta digital opaca SATURA**, o desvio do alfa no miolo de
//! um traço duro é `0,00`, e um gradiente sobre um platô é zero. Sem um campo, não há o que sombrear.
//!
//! ## O campo é o PAPEL, e ele já estava aqui
//!
//! ⚠️ **A doc 19 estimou este item como *"uma extração arquitetural com um ADR na frente"*, porque
//! media o papel pelos cinco arquivos `watercolor_*` em que ele nasceu. Medido de novo, a parte que
//! importa já é NEUTRA:** os três papéis assados (`PaperCold`/`PaperRough`/`PaperHot`, tiles 256²) são
//! `TextureKind`s de `ph2d-painter-brush`, amostrados por `texture::sample` como qualquer padrão, e o
//! slot `BrushSpec::paper` é campo do `BrushSpec` neutro. O que é `watercolor_*` são os SETTERS e o
//! consumidor do wash — não o dente.
//!
//! Logo este módulo lê o dente **sem uma linha de aquarela**, e a barreira `watercolor_is_untouched_*`
//! nem entra em jogo. O `watercolor_noise::paper_height` (o *fallback* procedural da doc 19 §1.1) fica
//! onde está, deliberadamente: os `TextureKind` do próprio slot já cobrem o caso.
//!
//! ## O sombreamento é o PADRÃO-OURO, e ele também já estava aqui
//!
//! O estado da arte para *"uma imagem de altura vira superfície iluminada"* é **bump mapping**
//! (Blinn 1978): normal por diferenças centrais, normalizada, sombreada por Lambert + um realce.
//! Krita implementa exatamente isso no `Phong Bumpmap`
//! (`normal = normalize(−∂h/∂x, −∂h/∂y, 8)`, depois `Ia + Kd·N·L + Ks·(R·V)^n`); ArtRage chama-o
//! *Canvas Lighting* e o expõe como ângulo + intensidade; o Corel Painter resolve a **outra** metade
//! (a tinta só assenta nos PICOS do grão, com um controle de *penetração*).
//!
//! ⚠️ **O emboss do Wet Paint é a versão CRUA disso:** `((m_r−m_l)·0,5 + (m_d−m_u)·1,0)·k` é
//! `dot(∇m, L)` com `L = (1, 2)` **não normalizado**, sem normal, sem realce e sem material. Ele
//! funciona porque o CAMPO dele é bom, não porque a lei seja.
//!
//! Então a lei aqui é a do [`super::impasto_shade::Rig`], que já é o padrão-ouro **e** resolve a
//! armadilha que o doc-comment dele nomeia (*"metade dos filtros de emboss já escritos"* escurecem a
//! pintura inteira no instante em que a luz acende): o sombreamento é **RELATIVO** — dividido pela
//! resposta de uma superfície PLANA do mesmo material —, então dente plano multiplica por exatamente
//! `1` e soma exatamente `0`, e sai byte-idêntico.
//!
//! ## As duas metades compõem, e o repo já tinha uma
//!
//! * **Depósito** (o Corel): o slot **Grain** já modula quanto pigmento assenta — medido, ele leva o
//!   desvio do alfa no miolo de `0,00` para `24,33`. A tinta já entra no dente.
//! * **Sombreamento** (o Krita/ArtRage): é o que este módulo acrescenta.
//!
//! ## Uma diferença NOMEADA em relação ao Wet Paint
//!
//! Lá o emboss vive DENTRO do pigmento (`la > 0`), então papel nu fica limpo. Aqui o dente é uma
//! **superfície**, e ela acende com ou sem tinta — que é o que o ArtRage faz e o que a doc 19 §1.3
//! antecipou como *"semântica nova, mas legítima: a cobertura do papel é 1"*. Isso é deliberado e é
//! metade do *"ou melhor"* do pedido: gatear por tinta exigiria um plano de cobertura por-canvas (o
//! ciclo de vida inteiro do §10.4), e **medir antes de limitar** manda não construí-lo antes de um
//! smoke pedir.

use ph2d_painter_brush::texture::{self, TexDabBasis, TextureSettings};
use ph2d_painter_brush::{TextureKind, TextureMapping};

use super::impasto_light::DEPTH_UNIT_PX;

/// **A amplitude do dente, em pixels de relevo, no Depth máximo.**
///
/// ⚠️ **MEDIDO contra a âncora do próprio modelo, não escolhido.** O emboss do Wet Paint declara o
/// próprio teto — ele faz `clamp(emb, −40, +40)` em níveis de cor —, então *"o mais parecido possível
/// com o Wet Paint"* tem um número, e não é uma opinião: o Depth máximo daqui pousa na mesma ordem.
///
/// Medido pela sonda `probe_substrate_depth_ladder`, a excursão de luminância no Depth máximo é
/// **LINEAR na amplitude** (`3,0 px → 128 níveis`, um quarto do curso por quarto do slider), logo
/// `1,0 px → ~43 níveis`. O primeiro valor que escrevi foi `3,0` e a medição o reprovou: 128 níveis é
/// **metade da faixa inteira** — aquilo não é papel, é chapa ondulada.
pub(super) const MAX_TOOTH_PX: f32 = 1.0;

/// ⛔ **MEDIDO E REJEITADO — não reconstrua: um realce especular no papel.**
///
/// A primeira versão deu ao papel um `shine` próprio para que a Roughness governasse a LARGURA do
/// brilho, como faz a Roughness da tinta. A prova de mutação a matou: tirar o material do papel (que o
/// devolve ao `shine: 0.0` do `Material::NEUTRAL`) **não move um texel**. O mecanismo está escrito no
/// `impasto_shade`: o realce de cada lâmpada tem a resposta PLANA subtraída e é clampado em zero
/// (`the_glint_only_ever_adds_light`), e num dente de ~1 px a normal quase não sai do plano — logo
/// `spec − flat_spec` é nulo em QUALQUER expoente. O especular é uma feature de relevo GROSSO (a
/// tinta), não de papel.
///
/// O substrato RESOLVIDO para um passe: o dente, a amplitude e o material dele.
///
/// Construído uma vez por passe (`resolve`) porque a base de rotação e o *snap* de Size não são
/// baratos por-texel — o mesmo motivo pelo qual o `watercolor_render` os iça para fora do laço.
pub(super) struct Substrate {
    tex: TextureSettings,
    basis: TexDabBasis,
    /// A amplitude em PIXELS de relevo (`depth × MAX_TOOTH_PX`).
    amp_px: f32,
    /// A **Roughness** do papel — a largura do realce, o mesmo eixo do `SpecLut` que a tinta usa.
    rough: f32,
}

impl Substrate {
    /// Resolve o substrato para este passe, ou `None` quando ele está desligado.
    ///
    /// `None` é o neutro e ele é **byte-idêntico**: quem chama não soma inclinação nenhuma, e o
    /// `shade_over` devolve `([1;3], [0;3])` na primeira linha.
    pub(super) fn resolve(tex: TextureSettings, depth: f32, rough: f32) -> Option<Self> {
        let amp_px = depth.clamp(0.0, 1.0) * MAX_TOOTH_PX;
        if amp_px <= 0.0 || !tex.is_active() {
            return None;
        }
        // ⚠️ **Ancorado no CANVAS, sempre.** Um substrato é a superfície sob tudo: se ele seguisse o
        // dab (`ViewPlane`/`Stencil`), o dente andaria com a pincelada e leria como ruído por-traço em
        // vez de papel. É a mesma escolha que o `watercolor_settings` faz ao armar o slot.
        let mut tex = tex;
        tex.mapping = TextureMapping::Tiled;
        tex.flow = false;
        tex.rake = false;
        let mut rng = 0u64; // Tiled não sorteia offset; o rng existe só para a assinatura
        let basis = texture::dab_basis(
            &tex,
            &mut rng,
            [0.0, 0.0],
            ph2d_painter_brush::footprint::FootprintDeform::identity(),
        );
        Some(Self {
            tex,
            basis,
            amp_px,
            rough: rough.clamp(0.0, 1.0),
        })
    }

    /// A altura do dente naquele texel, em PIXELS de relevo.
    fn tooth_px(&self, x: i64, y: i64) -> f32 {
        // `sample` devolve `0..1` (vale do tooth .. pico). `center`/`radius` são inertes sob `Tiled`
        // (a coordenada é `p / TEX_TILE_BASE_PX`), e é por isso que um substrato não precisa de dab.
        let t = texture::sample(&self.tex, &self.basis, x, y, [0.0, 0.0], 1.0, None);
        // **ROUGHNESS = a ÍNGREMEZA do dente** — um ganho de contraste em torno do meio, recortado.
        //
        // ⚠️ **Esta NÃO foi a primeira leitura, e a medição derrubou a primeira.** Eu a implementei
        // como o expoente do realce (a Roughness da TINTA, *"how BROAD the highlight is"*) e o gate
        // nasceu com **0 texels movidos**: o realce é subtraído da resposta PLANA e clampado em zero
        // (`the_glint_only_ever_adds_light`), e num dente de ~1 px a normal quase não sai do plano —
        // então `spec − flat_spec ≈ 0` em QUALQUER expoente. Um controle assim é morto por
        // construção, e a casa extermina essa espécie.
        //
        // A leitura que funciona é a das próprias referências: o Corel Painter chama de **Contrast**
        // (*"a steepness of the paper grain"*) e o ArtRage de **Roughness** (*"rougher grain, affecting
        // paint strokes more"*) — em nenhum dos dois é um expoente de brilho. Um papel áspero tem
        // paredes íngremes e platôs; um satinado ondula de leve. O recorte nos extremos é a feature,
        // não um defeito: é o que faz picos e poços chatos.
        let gain = 0.5 + self.rough * 2.5;
        let t = (0.5 + (t - 0.5) * gain).clamp(0.0, 1.0);
        t * self.amp_px
    }

    /// A inclinação do dente **na unidade que o [`super::impasto_shade::Rig`] espera**.
    ///
    /// ⚠️ **A conversão do consumidor entra AQUI, e é o ponto todo.** O `shade_over` faz
    /// `−dh × DEPTH_UNIT_PX` para chegar a pixels, porque o buffer de altura da TINTA é medido em
    /// cargas; o dente já é medido em pixels. Entregar a inclinação crua inclinaria o papel
    /// `DEPTH_UNIT_PX` vezes demais — literalmente
    /// [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]], que este módulo paga
    /// dividindo na entrada em vez de esperar que o outro lado adivinhe.
    pub(super) fn slope_at(&self, x: i64, y: i64) -> (f32, f32) {
        let dx = (self.tooth_px(x + 1, y) - self.tooth_px(x - 1, y)) * 0.5;
        let dy = (self.tooth_px(x, y + 1) - self.tooth_px(x, y - 1)) * 0.5;
        (dx / DEPTH_UNIT_PX, dy / DEPTH_UNIT_PX)
    }
}

/// O dente default de um papel — o que o artista arma ao ligar o relevo sem ter escolhido um papel.
/// `PaperCold` (*cold press*) é o papel de aquarela mais comum e o meio-termo dos três assados;
/// deixar `None` faria o interruptor ligar e não mostrar nada, que é o controle morto que esta casa
/// recusa.
fn default_paper() -> TextureSettings {
    TextureSettings {
        kind: TextureKind::PaperCold,
        mapping: TextureMapping::Tiled,
        ..TextureSettings::default()
    }
}

impl crate::tool::PainterTool {
    /// O SUBSTRATO vigente — o dente do papel como superfície, ou `None` quando desligado.
    ///
    /// ⚠️ **É a TERCEIRA razão para este passe existir, e a forma é a da doação:** ela também não passa
    /// pelo `impasto_show` (aquele bit pergunta *"mostrar o relevo da TINTA?"*, e um papel não é relevo
    /// de tinta) e também precisa correr num documento que não tem `heights` nenhum — que é literalmente
    /// o caso do Digital, o meio para o qual isto foi pedido.
    pub(super) fn substrate(&self) -> Option<Substrate> {
        Substrate::resolve(
            self.paint.brush.paper,
            self.paint.substrate_depth,
            self.paint.substrate_rough,
        )
    }

    /// Escreve um campo do PAPEL em todos os slots de pincel.
    ///
    /// ⚠️ **O papel é do CANVAS e o slot é do PINCEL — o fan-out é o que reconcilia os dois.** Sem ele,
    /// trocar de modo de pintura trocaria o papel debaixo da obra, que é a falha de duas-portas na sua
    /// forma mais cara: o substrato mudaria por um gesto que não fala de substrato nenhum. É o mesmo
    /// remédio (e o mesmo precedente) do `set_material_field` e do `toggle_brush_impasto`.
    fn set_paper_field(&mut self, write: impl Fn(&mut ph2d_painter_brush::BrushSpec)) {
        write(&mut self.paint.brush);
        for b in self.paint.brush_by_mode.iter_mut() {
            write(b);
        }
    }

    /// **Depth do substrato** (`0` = desligado). Ligar sem papel escolhido arma o [`default_paper`].
    pub fn set_substrate_depth(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.paint.substrate_depth = v;
        if v > 0.0 && !self.paint.brush.paper.is_active() {
            self.set_paper_field(|b| b.paper = default_paper());
        }
    }

    /// **Roughness do substrato** — quão LARGO é o realce sobre o dente.
    pub fn set_substrate_roughness(&mut self, v: f32) {
        self.paint.substrate_rough = v.clamp(0.0, 1.0);
    }

    /// O Depth vigente do substrato — o painel lê por aqui.
    #[must_use]
    pub fn substrate_depth(&self) -> f32 {
        self.paint.substrate_depth
    }

    /// A Roughness vigente do substrato.
    #[must_use]
    pub fn substrate_roughness(&self) -> f32 {
        self.paint.substrate_rough
    }

    /// Roteia as duas rows do substrato do canal genérico do painel para os setters acima. Devolve
    /// `true` quando consumiu o evento; chamada pelo `handle_panel_event`.
    pub(crate) fn route_substrate_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SUBSTRATE_RELIEF => {
                self.set_substrate_depth(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SUBSTRATE_ROUGHNESS => {
                self.set_substrate_roughness(*v as f32);
                true
            }
            _ => false,
        }
    }
}
