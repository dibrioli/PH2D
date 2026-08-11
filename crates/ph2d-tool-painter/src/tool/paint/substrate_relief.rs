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

use ph2d_painter_brush::texture::{self, ImageMask, TexDabBasis, TextureSettings};
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
pub(super) struct Substrate<'a> {
    tex: TextureSettings,
    basis: TexDabBasis,
    /// A amplitude em PIXELS de relevo (`depth × MAX_TOOTH_PX`).
    amp_px: f32,
    /// A **Roughness** do papel — a largura do realce, o mesmo eixo do `SpecLut` que a tinta usa.
    rough: f32,
    /// Os PIXELS do papel, quando o slot é uma **imagem** (o gesto *"Use as Brush Paper"*).
    ///
    /// ⚠️ **Sem ela o `TextureKind::Image` é INERTE, e isso era um controle morto que eu shipei.** O
    /// `patterns::sample_kind` responde `None => 1.0` (*"kind is Image but no pixels supplied"*), e um
    /// dente CONSTANTE tem gradiente zero — logo a luz não desenha nada. O artista escolhia o próprio
    /// papel digitalizado e o slider Relief não movia um pixel. Os outros dois consumidores do slot (o
    /// `watercolor_render` e o `dab_route` do Wet Paint) já passavam a imagem; este era o terceiro, e
    /// nasceu esquecido.
    image: Option<ImageMask<'a>>,
}

impl<'a> Substrate<'a> {
    /// Resolve o substrato para este passe, ou `None` quando ele está desligado.
    ///
    /// `None` é o neutro e ele é **byte-idêntico**: quem chama não soma inclinação nenhuma, e o
    /// `shade_over` devolve `([1;3], [0;3])` na primeira linha.
    pub(super) fn resolve(
        tex: TextureSettings,
        depth: f32,
        rough: f32,
        image: Option<ImageMask<'a>>,
    ) -> Option<Self> {
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
            image,
        })
    }

    /// A altura do dente naquele texel, em PIXELS de relevo.
    fn tooth_px(&self, x: i64, y: i64) -> f32 {
        // `sample` devolve `0..1` (vale do tooth .. pico). `center`/`radius` são inertes sob `Tiled`
        // (a coordenada é `p / TEX_TILE_BASE_PX`), e é por isso que um substrato não precisa de dab.
        let t = texture::sample(
            &self.tex,
            &self.basis,
            x,
            y,
            [0.0, 0.0],
            1.0,
            self.image.as_ref(),
        );
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

    /// A altura do dente **na unidade do plano de relevo da TINTA** — cargas, não pixels.
    ///
    /// ⚠️ **A conversão do consumidor entra AQUI, e é o ponto todo.** O buffer de altura da tinta é
    /// medido em CARGAS e a luz o multiplica por `DEPTH_UNIT_PX` para chegar a pixels; o dente já é
    /// medido em pixels. Entregá-lo cru inclinaria o papel `DEPTH_UNIT_PX` vezes demais — literalmente
    /// [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]], que este módulo paga
    /// dividindo na saída em vez de esperar que o outro lado adivinhe.
    ///
    /// ⚠️ **É esta a porta que os DOIS produtores atravessam** (o laço da CPU e o fold que sobe para a
    /// GPU), porque ela é somada dentro do `ReliefFields::height_at`. A versão anterior publicava uma
    /// INCLINAÇÃO, que só o laço da CPU sabia somar — e o produto compõe na GPU.
    pub(super) fn tooth_loads(&self, x: i64, y: i64) -> f32 {
        #[cfg(test)]
        TOOTH_SAMPLES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.tooth_px(x, y) / DEPTH_UNIT_PX
    }
}

/// **Quantas vezes o dente foi amostrado** — o oráculo da auditoria de custo (Enio, 2026-08-10).
///
/// ⚠️ **Uma CONTAGEM, e não um relógio, porque esta máquina é COMPARTILHADA**: a primeira tabela do
/// breakdown do papel media *"Roughness dobra o custo"* sob `load average 37` (sete `rustc` de outras
/// linhas), e a segunda corrida a desmentiu — 422 contra 426 ms, ruído. O número de amostragens por
/// texel é um fato sobre o CÓDIGO, e nenhuma carga o move.
#[cfg(test)]
pub(super) static TOOTH_SAMPLES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// **A chave do substrato** — tudo o que decide o dente, numa forma que se compara.
///
/// `TextureSettings` é `Copy + PartialEq` (ela existe para o `BrushSpec` ser `Copy`), então a chave é
/// um punhado de bytes e compará-la por frame não é um custo que se meça. Ver
/// [`crate::tool::PainterTool::reconcile_substrate`] para o porquê de ela existir.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct SubstrateKey {
    tex: TextureSettings,
    depth: f32,
    rough: f32,
    image: u64,
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
    pub(super) fn substrate(&self) -> Option<Substrate<'_>> {
        Substrate::resolve(
            self.paint.brush.paper,
            self.paint.substrate_depth,
            self.paint.substrate_rough,
            self.paint.paper_image.as_ref().map(|i| i.as_mask()),
        )
    }

    /// **O que o substrato É, em forma comparável** — a chave do reconcile.
    ///
    /// ⚠️ **Ela lê EXATAMENTE o que a [`Self::substrate`] lê, e mora ao lado dela por isso.** Uma
    /// chave que enumera *alguns* dos ingredientes é a forma exata de o próximo knob nascer sem
    /// invalidar nada — e o modo de falha disso não é um erro, é um RETÂNGULO na tela do artista.
    ///
    /// A imagem entra pela **versão monotônica** e nunca pelo endereço do buffer, o mesmo cuidado que
    /// o `PaperKey` do Wet Paint já toma (ADR-0124: um `Arc` novo pode nascer no endereço de um morto).
    fn substrate_key(&self) -> SubstrateKey {
        SubstrateKey {
            tex: self.paint.brush.paper,
            depth: self.paint.substrate_depth,
            rough: self.paint.substrate_rough,
            image: self.paint.paper_image_version,
        }
    }

    /// **O substrato mudou desde a última composição? Então a composição inteira está velha.**
    ///
    /// ⚠️ **É a correção dos dois primeiros reports do smoke de 2026-08-10, e eles são UM mecanismo.**
    /// O Enio viu *"retângulos que marcam por onde o pincel passou"* e *"ao aumentar o Relief o papel
    /// não é atualizado em tempo real, só ao usar o pincel"* — as duas metades de a mesma coisa: o
    /// dente do papel cobre a TELA INTEIRA, e nenhum knob que o produz derrubava o confinamento.
    ///
    /// O produto já dizia a lei, no doc do [`crate::tool::PainterTool::preview_gpu_region`]: *"`None`
    /// significa que a mudança não foi confinada"*, e o do fold de planos a repete como a condição de
    /// SOLIDEZ de um upload parcial (*"correto exatamente quando o relevo composto FORA da região não
    /// mudou"*). Um Relief novo muda o relevo composto em todo texel; confiná-lo ao rastro do pincel
    /// deixa o papel novo dentro do retângulo e o papel velho em volta — que é literalmente a foto.
    ///
    /// ⚠️ **Uma TESTEMUNHA, não uma lista de setters.** Os knobs que alimentam o dente são nove hoje
    /// (os dois do substrato + kind/size/angle/mapping/offset/params/reset do slot Paper) e sete deles
    /// moram num arquivo sobre a AQUARELA, escrito quando o papel só temperava o pigmento molhado. Uma
    /// linha por setter é a regra que o décimo nasce sem — e é o mesmo argumento que o
    /// `gpu_lane_stale` faz sobre si mesmo, duas dúzias de linhas acima na mesma crate.
    ///
    /// **Não invalida na PRIMEIRA vez**, e isso não é economia: `invalidate_composite` levanta o
    /// `edited_since_bind`, então um documento que ninguém tocou contaria como editado só por ter sido
    /// drenado uma vez.
    pub(crate) fn reconcile_substrate(&mut self) {
        let now = self.substrate_key();
        match self.lit_substrate {
            Some(was) if was == now => {}
            Some(_) => {
                self.lit_substrate = Some(now);
                self.invalidate_composite();
            }
            None => self.lit_substrate = Some(now),
        }
    }

    /// Escreve um campo do PAPEL em todos os slots de pincel.
    ///
    /// ⚠️ **O papel é do CANVAS e o slot é do PINCEL — o fan-out é o que reconcilia os dois.** Sem ele,
    /// trocar de modo de pintura trocaria o papel debaixo da obra, que é a falha de duas-portas na sua
    /// forma mais cara: o substrato mudaria por um gesto que não fala de substrato nenhum. É o mesmo
    /// remédio (e o mesmo precedente) do `set_material_field` e do `toggle_brush_impasto`.
    pub(super) fn set_paper_field(&mut self, write: impl Fn(&mut ph2d_painter_brush::BrushSpec)) {
        write(&mut self.paint.brush);
        for b in self.paint.brush_by_mode.iter_mut() {
            write(b);
        }
    }

    /// **Carimba o papel VIVO em todos os slots de pincel** — a porta que os setters do slot atravessam.
    ///
    /// ⚠️ **MEDIDO, não suposto (2026-08-10).** Os sete setters do slot Paper escreviam só o pincel
    /// vivo, e isso era invisível enquanto o papel só temperava o pigmento MOLHADO. Desde que o dente
    /// virou uma superfície do canvas, ele deixou de ser: com Paper Size em 5, pegar a **Faca** (que
    /// tem slot próprio) devolvia o Size a 1 e a excursão de luminância caía de **166 para 75 níveis**
    /// — o papel mudando debaixo da obra porque o artista pegou uma ferramenta.
    ///
    /// É a mesma lei que o `set_substrate_depth` já honrava e que o doc do [`Self::set_paper_field`]
    /// enuncia: **o papel é do CANVAS, o slot é do PINCEL.**
    pub(super) fn sync_paper_across_slots(&mut self) {
        let p = self.paint.brush.paper;
        self.set_paper_field(|b| b.paper = p);
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
