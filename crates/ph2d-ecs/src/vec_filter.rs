//! **A PILHA de FX raster de uma forma** — Blur / Glow / Drop Shadow / Inner Shadow / Inner Glow /
//! Outline / Color Overlay, encadeáveis.
//!
//! Irmão de [`crate::VecOffset`]/[`crate::VecTextPath`]/[`crate::VecEnvelope`] no padrão que esta
//! linha já usou meia dúzia de vezes: o componente guarda a **relação** (que efeitos, em que
//! ordem, com que parâmetros) e a aparência é **derivada** dela — a shell a produz por frame. De
//! graça: **undo e save cobrem o FX sem uma linha a mais** (os dois capturam o mundo ECS, e este
//! componente está registrado no `ComponentRegistry`).
//!
//! # Por que uma PILHA LINEAR, e não um grafo
//!
//! O `<filter>` do SVG é um **DAG** de primitivas (`feGaussianBlur`/`feOffset`/`feComposite`/
//! `feMerge`…) — poderosíssimo, e **abandonado como interface** por todo mundo que tentou: o
//! Photoshop (Layer Styles), o After Effects (effect stack) e o Figma (Effects) convergiram numa
//! **lista ordenada** de efeitos por objeto. O DAG sobrevive no *runtime* (o arquivo SVG), nunca
//! na mão do artista. Nós já tínhamos a resposta em casa: a pilha de Live Path Effects
//! (ADR-0132) é exatamente isto no eixo da GEOMETRIA, e esta é a irmã dela no eixo dos PIXELS.
//!
//! O Rive — a referência do módulo — não tem pilha nenhuma (feather + blend, com sombra e brilho
//! DERIVADOS do feather). Poder encadear *sombra → borrão → brilho*, nessa ordem, com o resultado
//! de um alimentando o seguinte, é o que esta seção entrega e ele não.
//!
//! # O invariante que faz a pilha compor: **um op é imagem → imagem**
//!
//! É a mesma frase do `ph2d_vec_scene::effect` (*"um efeito é `VecPath -> VecPath`, puro — é POR
//! ISSO que a pilha compõe"*), traduzida para raster. A consequência prática e não-óbvia:
//! **Glow e Drop Shadow compõem o halo POR BAIXO da fonte DENTRO do próprio op**, e devolvem UMA
//! imagem. Um op que dissesse *"desenhe isto atrás da forma"* não teria como ser entrada do
//! seguinte — e foi exactamente esse o modelo da W1 (o `FxMode::Below`), que morreu aqui.
//!
//! # As unidades, e por que MUNDO
//!
//! `radius`/`offset` são de **MUNDO**. A textura-scratch é rasterizada na resolução do device (o
//! tamanho da forma NA TELA), então o kernel em pixels é `mundo × zoom` — o filtro fica
//! **resolution-crisp**: re-renderizado por frame no zoom atual, proporcional em toda escala (a
//! propriedade que o feather do Rive tem, por outra via). Guardar pixels congelaria o efeito num
//! zoom; guardar fração exigiria re-derivar a escala da seleção a cada frame (lição do `VecOffset`).
//!
//! # Pilha vazia = sem filtro
//!
//! Não há variante "None": esvaziar a pilha no painel **REMOVE** o componente (a lei do
//! `VecOffset`: um documento não acumula relações inertes que não desenham nada). Uma forma sem
//! `VecFilter` flui pelo `dispatch` **byte-idêntica** ao mundo de hoje.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;
pub use crate::vec_filter_kinds::FxKindSpec;
use crate::vec_filter_kinds::SPECS;

/// **Um degrau da pilha** — um efeito com os parâmetros dele.
///
/// Plain data `Copy`: o fold da GPU o lê por valor, o painel o desenha, o undo o compara. Os
/// campos que um `kind` não usa ficam quietos (o painel não os oferece: knob morto é knob que
/// ensina a desconfiar dos vivos).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FxOp {
    /// Que efeito: [`FxOp::BLUR`] · [`FxOp::GLOW`] · [`FxOp::DROP_SHADOW`].
    pub kind: u8,
    /// O raio de suavização em unidades de MUNDO (o `stdDev` do gaussiano). A shell o converte
    /// para pixels do device pelo zoom.
    pub radius: f32,
    /// O deslocamento em unidades de MUNDO (só a Drop Shadow o lê).
    pub offset: [f32; 2],
    /// A cor do halo, RGBA reta em `[0,1]` (o Blur a ignora — ele borra os próprios pixels).
    pub color: [f32; 4],
    /// A intensidade/opacidade DESTE degrau, em `[0,1]`.
    pub opacity: f32,
    /// O MODO deste degrau — o índice em [`FxKindSpec::modes`]. Zero (o 1º modo) para todo tipo
    /// que não oferece escolha.
    pub mode: u8,
    /// Desligado = a pilha o SALTA, como se não estivesse lá — mas os parâmetros ficam. Espelha o
    /// `FxEntry::enabled` da pilha de geometria: desarmar não pode custar números que o artista
    /// teria de lembrar.
    pub enabled: bool,
    /// **A LEI DE MISTURA deste degrau** — o código de `ph2d_painter_effects::BlendMode`
    /// (`0` = Normal, o neutro). Só os tipos com [`FxKindSpec::takes_blend`] o leem.
    ///
    /// ⚠️ **Um `u8` CRU, e não o enum.** É o mesmo desenho que o `LayerCompositor` já ship: o
    /// `ph2d-ecs` não depende do `ph2d-painter-effects`, e não devia — este componente é um
    /// DOCUMENTO, e um documento não tem por que arrastar a crate de efeitos de pintura para dentro
    /// do grafo de dependências de toda cena. O nome de cada código é publicado à UI pela shell,
    /// que alcança os dois lados (o padrão do `set_filter_kinds`).
    ///
    /// ⚠️ **Código desconhecido cai em Normal** (o `from_u8` do enum já o faz), então um arquivo de
    /// uma versão futura desenha o efeito sem lei exótica em vez de não desenhar nada.
    pub blend: u8,
    /// **O TAMANHO das ondulações do ruído**, em unidades de MUNDO (o `baseFrequency` do SVG, pelo
    /// avesso: ali é frequência, aqui é comprimento — um artista pensa em *quão grandes são os
    /// caroços*, não em quantos cabem por unidade).
    ///
    /// ⚠️ **Em MUNDO como o `radius`, e é isso que torna o padrão zoom-invariante:** a coordenada
    /// de ruído é `pixel/escala_px` com `escala_px = escala_mundo × zoom`, ou seja o zoom cancela.
    /// Guardado em pixels, dar zoom re-sortearia a textura.
    pub scale: f32,
    /// **Quantas OITAVAS** o ruído soma (o `numOctaves`). Cada uma tem metade do tamanho e metade
    /// da amplitude da anterior: 1 é uma ondulação limpa, 4 já tem grão fino dentro dos caroços.
    pub detail: u8,
    /// **Qual das infinitas realizações** — o `seed`. Não muda a estatística do campo, só qual
    /// desenho ele é; é o botão *"me dá outro"* que todo artista de AE aperta primeiro.
    pub seed: u8,
    /// **Quanto a silhueta ENGORDA**, em unidades de MUNDO — e o sinal é a metade que importa:
    /// positivo cresce, negativo encolhe, **zero não faz nada**.
    ///
    /// ⚠️ **UM número com sinal, e não um par (modo, quantidade)** — é onde a pesquisa converge e
    /// não é gosto: o SVG (`operator="erode|dilate"`) e o Photoshop (*Minimum*/*Maximum*, dois
    /// itens de menu) partem em dois porque um é formato declarativo e o outro herdou a UI de 1990;
    /// quem desenhou isto como CONTROLE — o *Simple Choker* do AE, o *Dilate/Erode* do Blender, o
    /// *Offset Path* do Illustrator — usa **um slider com sinal**. Crescer e encolher são a mesma
    /// operação em sinais opostos, e quem afina um choke quer atravessar o zero sem trocar de modo.
    pub grow: f32,
}

/// O degrau neutro sobre o qual os defaults de cada tipo são escritos.
const BLANK: FxOp = FxOp {
    kind: FxOp::BLUR,
    radius: 0.0,
    offset: [0.0, 0.0],
    color: [0.0, 0.0, 0.0, 1.0],
    opacity: 1.0,
    mode: FxOp::MODE_CONTOUR,
    enabled: true,
    blend: FxOp::BLEND_NORMAL,
    scale: 0.25,
    detail: 3,
    seed: 0,
    grow: 0.0,
};

impl FxOp {
    /// Código de painel: um blur puro (borra o que chegou).
    pub const BLUR: u8 = 0;
    /// Código de painel: um brilho externo (a silhueta borrada, tingida, POR BAIXO do que chegou).
    pub const GLOW: u8 = 1;
    /// Código de painel: uma sombra projetada (o Glow deslocado por um offset).
    pub const DROP_SHADOW: u8 = 2;
    /// Código de painel: a sombra que cai **para dentro** — o alfa INVERTIDO, borrado, deslocado e
    /// mascarado pela própria forma. É o que faz um recorte parecer FUNDO (Photoshop/Figma).
    pub const INNER_SHADOW: u8 = 3;
    /// Código de painel: o brilho de dentro (a Inner Shadow sem deslocamento).
    pub const INNER_GLOW: u8 = 4;
    /// Código de painel: o **contorno** — o halo de borda DURA, largura autorada, por baixo da
    /// forma. É o traço de sticker, e é o que uma Gaussiana sozinha não desenha.
    pub const OUTLINE: u8 = 5;
    /// Código de painel: o **feather** — a borda fica macia SEM borrar o miolo, por uma rampa de
    /// largura autorada CENTRADA na fronteira. É o headline do Rive, e o que um borrão não faz (ele
    /// mistura a COR também).
    pub const FEATHER: u8 = 6;
    /// Código de painel: o **bevel** — a borda ganha relevo: a face virada para a luz clareia, a
    /// oposta escurece, e o efeito morre para o miolo. O "3D" do Layer Style.
    pub const BEVEL: u8 = 7;
    /// Código de painel: repinta o que chegou com uma cor, **sem borrar e sem mover cobertura**.
    /// Pontual ⇒ margem zero e UM dispatch.
    pub const COLOR_OVERLAY: u8 = 8;
    /// Código de painel: a **turbulência** — a imagem é DEFORMADA por um campo de ruído
    /// procedural. Amount pequeno = borda rasgada/orgânica (o *Roughen*); Amount grande = a forma
    /// inteira liquefaz. É o `feTurbulence` + `feDisplacementMap` do SVG **num degrau só**, que é
    /// como o AE (*Turbulent Displace*) e todo mundo depois dele o embrulharam.
    pub const TURBULENCE: u8 = 9;
    /// Código de painel: **Grow / Shrink** — a silhueta ENGORDA ou AFINA, por um limiar no campo
    /// de distância. É o `feMorphology` do SVG (`dilate`/`erode`), o *Simple Choker* do AE e o
    /// *Minimum/Maximum* do Photoshop, num knob com sinal.
    ///
    /// ⚠️ **É o único da família do campo que mede a IMAGEM, não a FORMA** — ver
    /// [`Self::measures_the_image`].
    pub const MORPHOLOGY: u8 = 10;
    /// Quantos tipos existem — o painel oferece um "Add" por tipo, a partir daqui.
    pub const KINDS: usize = 11;

    /// Modo de queda: **a PROXIMIDADE do outro lado** (a silhueta borrada — o modelo do
    /// Photoshop). Lê como PROFUNDIDADE: uma parte fina escurece INTEIRA, porque tudo nela está
    /// perto de fora, e uma reentrância quase não recebe efeito, porque o outro lado ali subtende
    /// um ângulo pequeno.
    pub const MODE_PROXIMITY: u8 = 0;
    /// Modo de queda: **a DISTÂNCIA à borda** — uma banda de largura constante ao longo de TODO o
    /// contorno, reentrâncias incluídas. É o que "sombra interna" desenha em quem olha a forma, e é
    /// o default dos degraus de DENTRO.
    pub const MODE_CONTOUR: u8 = 1;

    /// Modo de ruído: a soma COM SINAL das oitavas (o `fractalNoise` do SVG) — ondulações macias.
    pub const MODE_SMOOTH: u8 = 0;
    /// Modo de ruído: a soma dos MÓDULOS (o `turbulence` do SVG) — vincos onde as oitavas cruzam
    /// o zero.
    ///
    /// ⚠️ **Colide numericamente com [`Self::MODE_CONTOUR`], e a colisão é REAL, não teórica.** O
    /// `mode` é um índice na lista do TIPO, então o mesmo `1` quer dizer coisas diferentes em
    /// tipos diferentes — e o `plan_of` da `ph2d-render` roteava por `mode == MODE_CONTOUR` para
    /// QUALQUER tipo com modos, o que mandaria uma turbulência *Creased* para o campo de
    /// distância. Quem pergunta *"como este degrau é executado?"* tem de olhar o TIPO primeiro;
    /// há gate lá a exigir isso nos dois modos.
    pub const MODE_CREASED: u8 = 1;

    /// **Quantas oitavas o ruído pode somar.**
    ///
    /// ⚠️ **Não é teto de CUSTO — isso foi medido e é falso.** Na RTX, a 512², somar de 1 a 12
    /// oitavas move o passe de 0,058 para 0,12 ms, o que é a própria dispersão da medição: o laço
    /// é aritmética pura sobre um registro, e ela desaparece ao lado da largura de banda da
    /// textura (`measure_the_turbulence_octave_cost`).
    ///
    /// O teto é de **REPRESENTAÇÃO**: cada oitava tem metade do tamanho e metade da amplitude da
    /// anterior, então a `n`-ésima desenha detalhe de `Size/2ⁿ⁻¹` com peso `1/2ⁿ⁻¹`. Medido no que
    /// o artista vê — quanto a BORDA anda ao acrescentar a oitava, com Size 40 px
    /// (`measure_what_an_extra_octave_still_moves`):
    ///
    /// | oitava | a borda anda |
    /// |---|---|
    /// | 4 → 5 | 0,072 px |
    /// | 5 → 6 | 0,044 px |
    /// | **6 → 7** | **0,019 px** |
    /// | 9 → 10 | 0,002 px |
    ///
    /// A partir da sétima o desenho muda menos de um cinquenta avos de pixel, e o detalhe que ela
    /// acrescenta já é menor que o texel que o amostra — ele não aparece, ele **serrilha**.
    pub const MAX_DETAIL: u8 = 6;

    /// A lei de mistura NEUTRA (`BlendMode::Normal`). Um degrau nasce aqui, e nela o degrau é
    /// **byte-idêntico** ao mundo pré-blend — é isso que torna esta wave uma adição e não uma
    /// mudança de aparência.
    pub const BLEND_NORMAL: u8 = 0;

    /// Quantas leis de mistura um degrau oferece — os códigos `0..BLEND_KINDS`.
    ///
    /// ⚠️ **VINTE, e o `BlendMode` do Rust tem 22** — os dois que ficam de fora (`Behind` e
    /// `Clear`, códigos 20 e 21) não são leis de COR: são operações de **COBERTURA** (o próprio
    /// `apply` do Rust os desvia antes da função de mistura). Um degrau de FX aplica a sua lei
    /// exactamente onde a cobertura já está decidida pela lei DELE — o `inner_tint` existe para
    /// *não mover a cobertura*, e foi um bug real resolvido —, então os dois não teriam onde
    /// pousar. Oferecê-los e depois os dobrar em Normal no dispositivo seria a opção que despacha
    /// e mente; há gate na shell (o único lugar que vê os dois lados) a exigir que este número
    /// continue a ser o dos códigos não-de-cobertura.
    pub const BLEND_KINDS: u8 = 20;

    /// **A tabela dos tipos**, indexada pelo `kind` — mora no irmão [`crate::vec_filter_kinds`],
    /// que é o arquivo que cresce a cada wave. Re-exportada aqui porque `FxOp::SPECS` é a porta
    /// que a shell publica ao painel, e mover a tabela não pode mover a porta.
    pub const SPECS: [FxKindSpec; Self::KINDS] = SPECS;

    /// A spec de um `kind`. **Porta única** — quem pergunta *"este tipo tem offset / cresce / borra?"*
    /// pergunta aqui, nunca por aritmética de índice espalhada pelos consumidores.
    ///
    /// Um código desconhecido (arquivo de uma versão futura) cai no Blur, que é o tipo mais inerte:
    /// borra o que chegou e não inventa cor nenhuma.
    #[must_use]
    pub fn spec(kind: u8) -> &'static FxKindSpec {
        let i = kind as usize;
        if i < Self::KINDS {
            &Self::SPECS[i]
        } else {
            &Self::SPECS[Self::BLUR as usize]
        }
    }

    /// O nome que o painel mostra. Mora na [`Self::SPECS`] (junto do `kind`) para não haver duas
    /// tabelas a discordar sobre qual índice é o quê.
    #[must_use]
    pub fn kind_name(kind: u8) -> &'static str {
        Self::spec(kind).name
    }

    /// Este degrau tinge (tem cor)? Blur não — ele reusa os pixels que chegaram.
    #[must_use]
    pub fn tints(self) -> bool {
        Self::spec(self.kind).color_label.is_some()
    }

    /// Este degrau lê o par de offset? (a direção da sombra, ou a da luz.)
    #[must_use]
    pub fn displaces(self) -> bool {
        Self::spec(self.kind).offset_labels.is_some()
    }

    /// **Este degrau tem uma lei de mistura a honrar?** Vista da [`FxKindSpec::takes_blend`].
    ///
    /// ⚠️ **Porta ÚNICA, com dois consumidores que fazem perguntas diferentes:** o painel a
    /// consulta para OFERECER o controle, e o produtor da GPU para o HONRAR. Sem a segunda metade,
    /// um arquivo com `blend` sobrevivente num tipo que deixou de o tomar desenharia uma lei que
    /// a UI não mostra — e sem a primeira, um knob morto. (O `is_hinge` dos joints é o precedente
    /// exato.)
    #[must_use]
    pub fn takes_blend(self) -> bool {
        Self::spec(self.kind).takes_blend
    }

    /// A lei de mistura que este degrau de facto aplica — [`Self::BLEND_NORMAL`] em todo tipo que
    /// não a toma. É esta que o produtor manda ao dispositivo.
    #[must_use]
    pub fn blend_code(self) -> u8 {
        if self.takes_blend() {
            self.blend
        } else {
            Self::BLEND_NORMAL
        }
    }

    /// **Este degrau lê um campo de ruído procedural?** Vista da [`FxKindSpec::noise_labels`], e a
    /// mesma porta com dois consumidores do [`Self::takes_blend`]: o painel a consulta para
    /// OFERECER os três knobs, o produtor da GPU para os HONRAR.
    #[must_use]
    pub fn reads_noise(self) -> bool {
        Self::spec(self.kind).noise_labels.is_some()
    }

    /// **Este degrau engorda/afina a silhueta?** Vista da [`FxKindSpec::grow_label`], e a mesma
    /// porta com dois consumidores do [`Self::takes_blend`]: o painel a consulta para OFERECER o
    /// knob, o produtor da GPU para o HONRAR.
    #[must_use]
    pub fn reads_grow(self) -> bool {
        Self::spec(self.kind).grow_label.is_some()
    }

    /// **Contra o QUÊ este degrau mede a distância — a IMAGEM que recebeu, ou a FORMA?**
    ///
    /// Quatro tipos da família do campo (contorno, feather, bevel, os de dentro em modo Contour)
    /// são efeitos de **BORDA DA FORMA**: eles perguntam *a que distância da silhueta autorada eu
    /// estou*, e é por isso que empilhá-los recorta pela forma. A morfologia é a única definida
    /// sobre a **IMAGEM** — o `feMorphology` do SVG dilata *a entrada dele* —, e é isso que faz
    /// `Outline → Grow` engordar o contorno em vez de o recortar de volta à silhueta.
    ///
    /// ⚠️ **Porta única, e o preço de não a ter era mudo:** com geometria disponível o produtor
    /// resolve o campo pelo pé EXATO dos segmentos da forma, então um degrau que pergunta pela
    /// imagem receberia a resposta sobre a forma — o desenho certo para quatro tipos, e a resposta
    /// errada para o quinto, sem erro nenhum.
    #[must_use]
    pub fn measures_the_image(kind: u8) -> bool {
        kind == Self::MORPHOLOGY
    }

    /// As oitavas que este degrau de facto soma — pelo menos uma, no máximo [`Self::MAX_DETAIL`].
    /// **Porta única**: o painel mostra este número e o produtor manda este número.
    #[must_use]
    pub fn detail_clamped(self) -> u8 {
        self.detail.clamp(1, Self::MAX_DETAIL)
    }

    /// Este degrau contribui? Desligado, a pilha o salta (espelho do `FxEntry::is_active`).
    #[must_use]
    pub fn is_active(self) -> bool {
        self.enabled
    }

    /// Zera o modo de quem não tem modos (porta única do `new`).
    fn with_default_mode(mut self) -> Self {
        if Self::spec(self.kind).modes.is_empty() {
            self.mode = 0;
        }
        self
    }

    /// O degrau que um "Add" recém-clicado deve criar, com defaults **VISÍVEIS** — armar no
    /// neutro seria um clique que não muda um pixel.
    #[must_use]
    pub fn new(kind: u8) -> Self {
        match kind {
            Self::GLOW => Self {
                kind,
                radius: 0.18,
                color: [1.0, 1.0, 1.0, 1.0],
                // ⚠️ **Proximity, e não o Contour do `BLANK`.** O Glow SEMPRE foi a silhueta
                // borrada; ganhar uma opção não pode repintar o que "Add Glow" quer dizer para
                // quem já o usa. (Um Glow salvo antes desta wave carrega `mode = 0` — que é
                // exatamente este —, então nenhum arquivo muda de aparência.)
                mode: Self::MODE_PROXIMITY,
                ..BLANK
            },
            Self::DROP_SHADOW => Self {
                kind,
                radius: 0.1,
                offset: [0.12, -0.12],
                opacity: 0.6,
                ..BLANK
            },
            Self::INNER_SHADOW => Self {
                kind,
                radius: 0.08,
                offset: [0.08, -0.08],
                opacity: 0.75,
                ..BLANK
            },
            Self::INNER_GLOW => Self {
                kind,
                radius: 0.1,
                color: [1.0, 1.0, 1.0, 1.0],
                opacity: 0.8,
                ..BLANK
            },
            Self::OUTLINE => Self {
                kind,
                radius: 0.06,
                ..BLANK
            },
            Self::FEATHER => Self {
                kind,
                radius: 0.12,
                ..BLANK
            },
            Self::BEVEL => Self {
                kind,
                radius: 0.1,
                // A luz vem de cima-à-esquerda (a convenção de todo Layer Style).
                offset: [-0.1, 0.1],
                opacity: 0.9,
                ..BLANK
            },
            Self::COLOR_OVERLAY => Self {
                kind,
                // Sem raio (o tipo é pontual) e numa cor FORTE: o clique tem de mudar a tela.
                color: [0.95, 0.25, 0.35, 1.0],
                ..BLANK
            },
            Self::TURBULENCE => Self {
                kind,
                // Amount e Size **MEDIDOS** no smoke (`=35`): ondulações da ordem de um quinto da
                // forma, deslocando ~6% dela. Menos que isto lê como borda suja; mais, e o "Add"
                // já entrega a forma liquefeita.
                radius: 0.08,
                scale: 0.25,
                detail: 3,
                // Smooth: o `fractalNoise` do SVG, a lei que não tem vinco. Um default com vinco
                // seria escolher o efeito mais dramático como ponto de partida.
                mode: Self::MODE_SMOOTH,
                ..BLANK
            },
            Self::MORPHOLOGY => Self {
                kind,
                // ENGORDA por default, e da ordem de grandeza do contorno (0,06): "Add" tem de
                // mudar a tela, e a direção que se lê como *efeito* é a que cresce — encolher uma
                // forma para dentro pode não deixar rastro nenhum numa silhueta fina.
                grow: 0.06,
                ..BLANK
            },
            _ => Self {
                kind: Self::BLUR,
                radius: 0.12,
                ..BLANK
            },
        }
        // Um tipo sem modos guarda ZERO — um número guardado que não seleciona nada é a semente do
        // "este campo quer dizer o quê aqui?" seis meses depois.
        .with_default_mode()
    }
}

/// **A pilha de FX raster de uma forma.** A entidade que a carrega também tem um
/// [`crate::VecPathRef`]: o `VecPath` dela continua a curva AUTORADA (o modo Node a edita); o
/// resultado filtrado é DESENHO, que a shell produz por frame e injeta no z da fonte.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecFilter {
    /// Os degraus, **na ordem em que se aplicam** (o primeiro recebe a forma nua). A ordem é a
    /// feature: `Shadow → Blur` e `Blur → Shadow` desenham coisas diferentes.
    pub ops: Vec<FxOp>,
}

impl SimComponent for VecFilter {}

impl VecFilter {
    /// O teto de degraus numa pilha.
    ///
    /// **O recurso que aperta é a TELA do painel, não a GPU** — e isso está MEDIDO, não suposto
    /// (`ph2d-render/tests/fx_stack_gpu.rs::the_cost_of_a_stack_is_linear_in_the_number_of_ops`,
    /// RTX, 512×512, sigma 8 px): `0 degraus 0,082 ms · 1 → 0,084 · 2 → 0,149 · 3 → 0,220 ·
    /// 4 → 0,336 · 6 → **0,429 ms**`. O custo é linear, ~0,07 ms por degrau, e uma pilha CHEIA
    /// custa **2,6 % de um frame de 60 fps**. Cada degrau, em compensação, é um card de 4-6 linhas
    /// no painel — seis já enchem a coluna. É a mesma razão do `MAX_PATH_EFFECTS` da pilha de
    /// geometria, com o número um degrau acima porque aqui o card é mais raso.
    pub const MAX_OPS: usize = 6;

    /// Uma pilha com um degrau — o que o 1º "Add" produz.
    #[must_use]
    pub fn single(op: FxOp) -> Self {
        Self { ops: vec![op] }
    }

    /// A pilha desenha alguma coisa? Vazia (ou toda desligada) = a forma sai nua, e a shell não
    /// produz imagem nenhuma. **Porta única**: quem coze e quem decide se há FX perguntam aqui.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.ops.iter().any(|o| o.is_active())
    }

    /// Há espaço para mais um degrau?
    #[must_use]
    pub fn has_room(&self) -> bool {
        self.ops.len() < Self::MAX_OPS
    }

    /// Troca `row` com o vizinho de cima. `false` (e nada muda) na primeira linha.
    pub fn move_up(&mut self, row: usize) -> bool {
        if row == 0 || row >= self.ops.len() {
            return false;
        }
        self.ops.swap(row - 1, row);
        true
    }

    /// Troca `row` com o vizinho de baixo. `false` (e nada muda) na última linha.
    pub fn move_down(&mut self, row: usize) -> bool {
        if row + 1 >= self.ops.len() {
            return false;
        }
        self.ops.swap(row, row + 1);
        true
    }
}

#[cfg(test)]
#[path = "vec_filter_tests.rs"]
mod tests;
