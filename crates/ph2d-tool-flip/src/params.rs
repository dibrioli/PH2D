//! Flip-tool UI vocabulary — o modo de canvas (Select/Draw/Erase), o modo de
//! borracha, e os mapeamentos slider↔valor do brush, compartilhados pelo painel
//! docado (`ph2d-panel-flip`) e pela tool (`handle_panel_event`).
//!
//! Espelha `ph2d_tool_vector::params`: a tool é dona do estilo autoritativo,
//! projeta-o num [`FlipStyleSnapshot`] por frame (o shell publica → o painel lê),
//! e os dois lados concordam no mapa afim do slider (drag e tool em lock-step).

/// O gesto de canvas que a tool Flip executa. Espelha a arbitragem do Vector
/// (ADR-0112): **gizmo só no `Select`** (os modos de desenho não publicam
/// `GizmoView`, senão as alças comeriam o clique). O pill alterna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlipMode {
    /// Seta preta: seleciona e TRANSFORMA o objeto pelo gizmo. Não desenha.
    #[default]
    Select,
    /// Lápis: cada arrasto no canvas cria um traço novo no desenho ativo.
    Draw,
    /// Borracha: remove cobertura/traço (ver [`EraseMode`]).
    Erase,
    /// **Balde**: um clique preenche a região delimitada pelas linhas (W4).
    Fill,
    /// **Reshape**: esculpe o traço já desenhado (W5 — ver [`ReshapeKind`]).
    Reshape,
    /// **Edit**: seleciona TRAÇOS (W6 — o Edit Mode do GP).
    ///
    /// É um modo próprio, e **não** uma sobrecarga do [`FlipMode::Select`]: o Select é a
    /// arbitragem do ADR-0112 — ali quem manda é o **gizmo**, que move/gira/escala o
    /// objeto Flip inteiro. Se o clique do Select passasse a pegar traço, o usuário
    /// perderia o gizmo. É a mesma separação Object Mode × Edit Mode do Grease Pencil.
    Edit,
    /// **Colorize**: rabisca cores sobre as linhas; o corte LazyBrush transforma os
    /// rabiscos em regiões preenchidas num só solve (C2 — `docs/Flip/09_colorize.md`).
    Colorize,
}

impl FlipMode {
    /// **Todos os modos** — a lista que os gates varrem.
    ///
    /// Ela existe porque um modo NOVO escapava dos gates em silêncio: eles enumeravam os
    /// modos à mão, e o `Edit` (W6) entrou sem que o gate modal ou o do Size dissessem
    /// nada — apesar de o Edit mostrar controles dos dois. Um gate que não OBSERVA não
    /// dispara. Agora os gates varrem `ALL` e afirmam que a tabela deles a cobre INTEIRA,
    /// então o próximo modo quebra o teste no dia em que nascer — que é o único momento em
    /// que o custo de arrumar é baixo.
    pub const ALL: [FlipMode; 7] = [
        FlipMode::Select,
        FlipMode::Draw,
        FlipMode::Erase,
        FlipMode::Fill,
        FlipMode::Reshape,
        FlipMode::Edit,
        FlipMode::Colorize,
    ];
}

/// **O DOMÍNIO da seleção no modo Edit** (W8/§4.B — o `.selection` do GP vive em Point OU
/// Curve, `02_referencia §11`). O toggle da toolbar do GP vira três pills aqui:
/// `Stroke` = domínio Curve (clique pega o traço inteiro — o comportamento W6);
/// `Point` = domínio Point (clique pega UMA âncora; o traço vira a projeção `any()`);
/// `Segment` = o pedaço entre dois CRUZAMENTOS (§4.B).
///
/// A conversão de domínio na troca (broadcast/promoção) é feita pelo SHELL sobre o
/// documento — a tool só guarda a escolha (ela não tem, e não deve ter, acesso ao doc).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EditDomain {
    /// Traços inteiros (o domínio Curve do GP) — o default e o comportamento W6.
    #[default]
    Stroke,
    /// Âncoras individuais (o domínio Point do GP) — meio-traço selecionável.
    Point,
    /// **O pedaço entre dois cruzamentos** (§4.B). No DADO é o domínio Point (é o que a
    /// referência crava: *"segment→Point + pós-processo"*) — o que muda é a POLÍTICA de
    /// pick: um clique acende o trecho inteiro entre os traços que cruzam este.
    Segment,
}

impl EditDomain {
    /// Os domínios na ordem das pills. Existe para o seam test **contar** — um domínio
    /// novo que não passe pelo seam é uma pill pintada e inerte, o bug nº 1 do projeto
    /// (o mesmo papel do [`ReshapeKind::ALL`] para os oito pincéis).
    pub const ALL: [EditDomain; 3] = [EditDomain::Stroke, EditDomain::Point, EditDomain::Segment];
}

/// Os pincéis de **escultura de traço** (W5). Espelha `ph2d_flip_reshape::ReshapeKind`
/// — mesmo precedente do [`FillMode`]: a tool e o painel são vocabulário de UI e não
/// carregam o solver; o shell traduz na fronteira.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReshapeKind {
    /// Alisa o tremor (o pincel mais usado).
    #[default]
    Smooth,
    /// Empurra na direção do movimento.
    Push,
    /// **Agarra** e carrega (o conjunto é congelado no toque).
    Grab,
    /// Aperta em direção ao cursor (Ctrl: infla).
    Pinch,
    /// Torce ao redor do cursor (Ctrl: inverte).
    Twist,
    /// Engrossa a linha (Ctrl: afina).
    Thickness,
    /// Aumenta a opacidade (Ctrl: apaga).
    Strength,
    /// Bagunça, perpendicular ao movimento.
    Randomize,
}

impl ReshapeKind {
    /// Os oito, na ordem do painel (2 linhas de 4).
    pub const ALL: [ReshapeKind; 8] = [
        ReshapeKind::Smooth,
        ReshapeKind::Push,
        ReshapeKind::Grab,
        ReshapeKind::Pinch,
        ReshapeKind::Twist,
        ReshapeKind::Thickness,
        ReshapeKind::Strength,
        ReshapeKind::Randomize,
    ];

    /// O rótulo do botão (inglês — a UI do app é inglês, sempre).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ReshapeKind::Smooth => "Smooth",
            ReshapeKind::Push => "Push",
            ReshapeKind::Grab => "Grab",
            ReshapeKind::Pinch => "Pinch",
            ReshapeKind::Twist => "Twist",
            ReshapeKind::Thickness => "Thicken",
            ReshapeKind::Strength => "Strength",
            ReshapeKind::Randomize => "Jitter",
        }
    }
}

/// Como o balde trata o que já está pintado — a semântica de balde de ANIMAÇÃO do
/// Toon Boom (`04 §3`). Espelha `ph2d_flip_fill::FillMode` (o painel/tool não
/// dependem do solver; o shell traduz).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FillMode {
    /// Preenche a região (o balde de sempre).
    #[default]
    Paint,
    /// Preenche **por baixo** do que já está pintado — colorir sem tocar na linha.
    PaintBehind,
    /// **Remove** o preenchimento sob o clique.
    Unpaint,
}

/// Faixa do slider **Gap Closure** (alcance da extensão, em px de tela).
pub const GAP_MAX_PX: f64 = 40.0;
/// Faixa do **Grow/Shrink** (px de tela): o offset assinado do contorno a partir do
/// **EIXO** da linha (BUGS #14). O default 0 já é exato em qualquer espessura e zoom;
/// isto é só o ajuste estilístico (o "off-register" para +, o vão deliberado para −).
pub const GROW_MIN: f64 = -8.0;
pub const GROW_MAX: f64 = 8.0;
/// Faixa do **Trap** — o raio da *trapped ball*, em px de TELA (`0` = desligado).
///
/// Uma bola de raio `r` sela um vão de até `2r`, então o teto de 20 px casa com a
/// capacidade do controle irmão: o Gap Closure alcança 40 px, e `2 × 20 = 40`. Os dois
/// resolvem o mesmo problema por caminhos diferentes, e não faria sentido um alcançar
/// menos que o outro.
///
/// **Não é um teto de recurso** (CLAUDE.md §0.0): o custo da bola é a EDT, que é O(área)
/// e **independe do raio** — subir o Trap não custa mais, e a medição da grade está em
/// `docs/Flip/09_colorize.md` §7.1. O que limita é o SENTIDO: uma bola de raio `r` não
/// entra numa região mais estreita que `2r`, então um Trap alto torna as regiões finas
/// do desenho impreenchíveis (e o balde diz isso, com o erro `BallTooFat`).
pub const TRAP_MAX_PX: f64 = 20.0;
/// Faixa do **Precision** (pixels do buffer por px de tela; 1 = resolução da tela).
pub const PRECISION_MIN: f64 = 0.5;
pub const PRECISION_MAX: f64 = 4.0;

/// Como a borracha age (GP `erase.cc`): `Soft` reduz opacidade (default, mais
/// "pintura"), `Hard` corta a cobertura, `Stroke` apaga o traço inteiro tocado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EraseMode {
    #[default]
    Soft,
    Hard,
    Stroke,
}

/// Largura do traço em pixels de tela (a faixa inclusiva que o slider Size cobre).
///
/// **O teto é 256 px** (smoke do Enio 2026-07-13: *"parece que o máximo ainda é muito
/// pequeno"*). Os 64 px do 1º corte eram um pincel de *linha*; o Flip precisa também da
/// **marca** — o traço largo de marcador, o preenchimento à mão, o borrão de sombra —, e
/// 64 px num monitor de 4K é um risco fino. O mesmo Size é o raio da borracha e o do
/// pincel de escultura, então o teto vale para os três.
///
/// (A faixa é linear de propósito: o painel liga o slider ao chip por um mapa **afim**
/// — `display = track·scale + offset` —, e uma curva quadrática faria o número no chip
/// divergir do knob no 1º arrasto. Para o valor exato existe a caixa: digitar 7 dá 7.)
pub const WIDTH_MIN_PX: f64 = 1.0;
pub const WIDTH_MAX_PX: f64 = 256.0;

/// **Quantos "px" de Size cabem em UMA unidade de mundo** (ADR-0114 §4.C.6).
///
/// O Size é uma medida do **MUNDO**, não da tela — Enio 2026-07-17: *"a largura do traço
/// está relativa ao zoom do canvas e não é fixa no mundo"*. Isto **reverte** a escolha de
/// 2026-07-11 (pincel absoluto em px de tela), e é a escolha certa: um traço é ARTE, e
/// arte não muda de espessura porque o artista aproximou a câmera. Dar zoom agora amplia
/// o desenho como uma foto — e é assim que todo app de desenho de documento se comporta
/// (o traço de 2 pt do Illustrator tem 2 pt em qualquer zoom).
///
/// **Ela também cura uma incoerência do próprio documento:** as POSIÇÕES sempre foram
/// unidades de mundo enquanto as LARGURAS eram px de tela. Misturar as duas já tinha
/// custado um bug real ao balde (`flip_fill::boundaries`: *"uma linha de 3 unidades de
/// mundo (≈324 px!)"*), que precisava converter na mão. Com o Size em mundo, os dois
/// eixos do `Point` falam a mesma língua e a conversão SOME.
///
/// O número é redondo de propósito: **Size 100 = 1 unidade de mundo**. Na vista default
/// (`height_world = 10`) ele lê perto de px, que é a intuição que o slider já vendia.
pub const SIZE_PX_PER_WORLD: f32 = 100.0;

/// **Size (o número do slider) → largura em unidades de MUNDO** — a porta ÚNICA da
/// conversão (ADR-0114 §4.C.6).
///
/// Todo sítio que AUTORA com o Size passa por aqui: o traço (`flip_draw`), a reescrita da
/// seleção no Edit (`flip_select`), o raio da borracha (`flip_erase`) e o do sculpt
/// (`flip_reshape`). O anel do cursor faz o caminho de volta (mundo → tela) porque ele
/// mostra na TELA o que vai acontecer no MUNDO. Duas cópias da regra divergem — e aqui
/// divergir significa a ferramenta desenhar numa espessura e o anel prometer outra.
#[must_use]
pub fn size_to_world(size_px: f64) -> f32 {
    (size_px as f32) / SIZE_PX_PER_WORLD
}

/// Mapa afim do slider Size → chip px (o painel usa em `link_slider_number_mapped`:
/// `px = track·SCALE + OFFSET`). Mantém painel e tool em lock-step com
/// [`slider_to_px`]/[`px_to_slider`].
pub const WIDTH_SLIDER_OFFSET: f32 = WIDTH_MIN_PX as f32;
pub const WIDTH_SLIDER_SCALE: f32 = (WIDTH_MAX_PX - WIDTH_MIN_PX) as f32;
/// Mapa do slider Opacity → chip (`0..1` → `0..100 %`).
pub const OPACITY_SLIDER_SCALE: f32 = 100.0;

/// Slider normalizado `0..=1` → largura px `MIN..=MAX`.
#[must_use]
pub fn slider_to_px(track: f32) -> f64 {
    WIDTH_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (WIDTH_MAX_PX - WIDTH_MIN_PX)
}

/// Largura px → slider normalizado `0..=1` (inverso de [`slider_to_px`]), pra
/// semear o knob a partir da largura autoritativa da tool.
#[must_use]
pub fn px_to_slider(px: f64) -> f32 {
    (((px - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32).clamp(0.0, 1.0)
}

/// Slider `0..=1` → fração `0..=1` (hardness, opacity, smoothing). Identidade
/// clampada — o mapa afim trivial mantém painel e tool em lock-step.
#[must_use]
pub fn slider_to_unit(track: f32) -> f32 {
    track.clamp(0.0, 1.0)
}

/// O snapshot que o painel docado pinta (a tool projeta o estilo por frame).
/// `Default` espelha os defaults da `FlipTool` (o painel pinta isto antes do 1º
/// push do shell).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlipStyleSnapshot {
    /// Cor do traço (sRGB8) — a mesma que o picker OKLCH devolve.
    pub stroke: [u8; 4],
    /// Largura em px de tela.
    pub width_px: f64,
    /// Dureza da borda `0..=1` (1 = borda dura).
    pub hardness: f32,
    /// Opacidade do traço `0..=1`.
    pub opacity: f32,
    /// Intensidade do active smoothing `0..=1` (o "assentar" da cauda).
    pub smoothing: f32,
    /// Modo de canvas atual (o painel destaca o botão ativo).
    pub mode: FlipMode,
    /// Modo de borracha atual (só relevante em `FlipMode::Erase`).
    pub erase: EraseMode,
    /// **O raio EFETIVO da borracha, em px de tela** — já resolvido contra o link
    /// (§4.C): linkado = o `width_px` do pincel, deslinkado = o valor próprio dela.
    ///
    /// Quem apaga e quem desenha o anel do cursor leem ESTE campo e mais nenhum: a
    /// pergunta *"que raio a borracha usa?"* tem uma resposta só, e ela é respondida
    /// no [`crate::FlipTool::ui_snapshot`] — duas cópias divergiriam.
    pub erase_px: f64,
    /// A força EFETIVA da borracha `0..=1` (idem: linkada = `opacity` do pincel).
    pub erase_strength: f32,
    /// O Size da borracha segue o do pincel? (só desenha o toggle de link; quem
    /// precisa do NÚMERO usa `erase_px`.) Default **ligado** — a borracha sempre
    /// compartilhou o Size, e deslinkar é opt-in.
    pub link_size: bool,
    /// A Strength da borracha segue a do pincel? (idem.)
    pub link_strength: bool,
    /// Cor do PREENCHIMENTO (sRGB8) — distinta da cor do traço: colorir usa outra
    /// paleta que desenhar, e obrigar a trocar a cor do traço para pintar seria
    /// hostil.
    pub fill_color: [u8; 4],
    /// Modo do balde.
    pub fill_mode: FillMode,
    /// **O traço nasce PREENCHIDO** (só relevante em [`FlipMode::Draw`]) — o material
    /// `stroke + fill` do Grease Pencil, que é como o Suzanne é desenhado.
    ///
    /// Quando ligado, o traço carrega o próprio preenchimento: o fill é a triangulação
    /// dos **pontos dele**, então linha e cor são UMA geometria — esculpir a linha move
    /// a cor exatamente junto, no mesmo frame, sem re-preencher nada. (O balde continua
    /// existindo para colorir regiões delimitadas por VÁRIOS traços.)
    pub draw_filled: bool,
    /// O pincel de escultura ativo (só relevante em [`FlipMode::Reshape`]).
    ///
    /// O Reshape **não tem raio nem força próprios**: usa o `width_px` (Size) e a
    /// `opacity` (Strength) — exatamente como a borracha. Um 2º par de sliders para
    /// as mesmas duas grandezas seria estado duplicado, e o usuário teria de
    /// re-ajustar o pincel a cada troca de modo.
    pub reshape: ReshapeKind,
    /// O DOMÍNIO da seleção (só relevante em [`FlipMode::Edit`]) — traço ou ponto (W8).
    pub edit_domain: EditDomain,
    /// Alcance do Gap Closure em px de tela (`0` = desligado).
    pub gap_px: f64,
    /// Grow/Shrink em px de tela: offset assinado do contorno a partir do EIXO da
    /// linha (o shell converte para px de buffer via `precision`).
    pub grow: f64,
    /// Precision: resolução do buffer do balde (pixels por px de tela).
    pub precision: f64,
    /// **Trap**: raio da trapped ball em px de tela (`0` = desligado — o balde de
    /// sempre, ao bit). O shell converte para px de buffer via `precision`.
    pub trap: f64,
    /// Cor que o PRÓXIMO rabisco do Colorize semeia (sRGB8) — paleta própria (C2),
    /// distinta da cor do balde: cada rabisco carrega a cor com que foi desenhado.
    pub colorize_color: [u8; 4],
}

impl Default for FlipStyleSnapshot {
    fn default() -> Self {
        // Espelha os defaults da `FlipTool` (o painel pinta isto antes do 1º
        // push do shell). As consts vivem no `tool.rs` (evita ciclo params↔tool),
        // então repetimos os valores aqui — cobertos por um teste no `tool.rs`.
        Self {
            stroke: [240, 240, 245, 255],
            width_px: 6.0,
            hardness: 1.0,
            opacity: 1.0,
            smoothing: 0.5,
            mode: FlipMode::Select,
            erase: EraseMode::Soft,
            // Linkados por default ⇒ os efetivos são os do pincel.
            erase_px: 6.0,
            erase_strength: 1.0,
            link_size: true,
            link_strength: true,
            fill_color: [230, 190, 120, 255],
            fill_mode: FillMode::Paint,
            draw_filled: false,
            reshape: ReshapeKind::Smooth,
            edit_domain: EditDomain::Stroke,
            gap_px: 0.0,
            grow: 0.0,
            trap: 0.0,      // opt-in: o default é o balde do W4, sem mudança nenhuma
            precision: 1.6, // = DEFAULT_PRECISION (tool.rs); o teste-espelho cobre
            colorize_color: [200, 90, 90, 255],
        }
    }
}
