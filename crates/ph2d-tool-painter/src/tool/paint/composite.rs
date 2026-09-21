//! **Composite Brush** — run Brush · Smear · Blur · Erase together as a reorderable **5-layer** stack.
//!
//! An upgrade to the Brush tool (a panel checkbox, not a rail tool): when on, one stroke applies all
//! the armed operations, each with its own Strength — and, since 2026-09-20 (ordem do dono), its own
//! **colour**, **hardness** and **stamp size**. The layers occupy FIXED positions numbered 1 (top) …
//! 5 (bottom); the tool at each position is reordered with the panel's up/down buttons. The stroke
//! runs the stack **bottom → top** (position 5 first), so each operation processes the canvas as
//! modified by the one below it — e.g. Brush(3) → Smear(2) → Blur(1) paints, then smears that, then
//! blurs the result; Blur(3) → Smear(2) → Brush(1) blurs the canvas, smears it, then paints clean
//! strokes on top (untouched by the blur/smear below). Split from `paint.rs` for the LOC cap.
//!
//! ⭐⭐⭐ **E a ordem é do TRAÇO, não do LOTE** (2026-09-20, report do dono três vezes): com duas ou
//! mais camadas activas a pilha corre **por CAMADA sobre a história inteira do gesto**, numa
//! recomposição regional a partir da tela do pen-down. O mecanismo, o preço e as três excepções
//! vivem em [`super::composite_pilha`]; aqui ficam o MODELO (o que uma camada é) e a FIAÇÃO.
//!
//! The shared brush parameters (Size / Shape / Grain / Falloff / Tiling / Jitter / Symmetry / Stroke)
//! drive every layer's dab geometry; the colour-family parameters (Blend / ramps / Randomize /
//! Accumulate) only affect the pigment-laying layers — so the panel keeps every control visible in
//! composite mode (see `BrushSettings::paints_no_color`).
//!
//! ## As DUAS posições novas nascem DESLIGADAS, e a pilha de hoje é byte-idêntica
//!
//! `4` e `5` nascem com `strength = 0`, que é como o motor pula uma camada ⇒ o traço de quem não
//! mexe em nada é o mesmo ao bit. O mesmo vale para os dois campos novos: `color: None` = *a cor do
//! pincel* (o caminho por onde o Randomize Color continua a passar) e `size: 1.0` = *o tamanho do
//! pincel*, e nos dois casos a lista de dabs entregue à rota é **a mesma fatia**, sem uma cópia.
//!
//! ## Por que uma camada MAIOR fica com MENOS dabs da mesma lista (e não com um percurso próprio)
//!
//! O motor de traço emite dabs a `spacing × diâmetro` de distância, resolvidos com o raio do
//! PINCEL. Medido em 2026-09-20 ([`super::diag_composite_cinco_camadas`]): reutilizar essa lista
//! numa camada `4×` maior custa **`×3,7` a `×4,1`** — ela paga quatro vezes mais dabs do que
//! precisa (o Smear vai de `95` para `387 ms` num traço de 720 px). ⇒ uma camada maior
//! **SUBAMOSTRA** a lista por comprimento de arco ([`PainterTool::camada_dabs`]), ficando com o
//! subconjunto que o espaçamento DELA pede. É exacto (os dabs escolhidos são os do próprio
//! percurso, não pontos inventados) e custa um acumulador por camada.
//!
//! ⚠️ **Uma camada MENOR fica com a lista inteira, e isso é uma decisão com número:** o vão entre
//! dabs é `spacing × 2r`, logo ela só sai em CONTAS quando `size < spacing` — que é exactamente o
//! piso do controlo. Acima dele a lista cheia é mais densa do que o espaçamento dela pede, o que
//! custa MENOS do que um percurso próprio (menos dabs, cada um de área `size²`) e endurece um pouco
//! a borda, que é a doença já medida no doc 25 §13.10.

use super::PaintMode;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::Dab;

/// **Quantas posições a pilha tem.** ⚠️ Ela é lida pelo motor, pelo instantâneo, pelos ids e pelo
/// cartão — *um `3` escrito à mão em qualquer um deles é a segunda resposta que esta const existe
/// para não haver* (a extensão de 2026-09-20 encontrou exactamente um, o `N_LAYERS` do painel).
pub(crate) const N_CAMADAS: usize = 5;

/// **O tecto do tamanho de uma camada, e o recurso dele é o QUADRO.**
///
/// Medido em 2026-09-20 ([`super::diag_composite_cinco_camadas`], `--release`, canvas `1024²`,
/// traço de 720 px = 362 eventos), com o percurso próprio de cada camada:
///
/// | raio | Brush | Smear | Blur |
/// |---|---|---|---|
/// | 24 (`1×`) | `5,18` | `67,59` | `8,70` |
/// | 96 (`4×`) | `6,52` | `95,12` | **`87,40`** |
///
/// O Blur é `~r²` (a convolução é `área × 2(2k+1)` com `k = 0,34·raio`), e a `4×` ele custa
/// `87,40 / 362 = 0,241 ms` por evento de ponteiro. Um rato de `1000 Hz` entrega `~16` eventos por
/// quadro ⇒ **`3,9 ms`, 23 % de um quadro de 16,7, para UMA camada**. A `8×` seriam `~15,5 ms` — o
/// quadro inteiro. ⇒ o tecto é **`4`**, e ele nomeia o relógio, não o conforto.
pub const MAX_TAMANHO_DA_CAMADA: f32 = 4.0;

/// **Quantas OPERAÇÕES o chip de uma camada cicla** — `Brush · Smear · Blur · Erase`.
///
/// ⛔ Ela existe porque o `% 4` do ciclo era um literal ao lado do `match` de quatro braços do
/// [`CompositeOp::to_u8`], e **o painel precisa do mesmo número** para medir a coluna do chip (a
/// largura sai do nome mais largo das operações, não de um palpite). *Três respostas à mesma
/// pergunta, e só a do ciclo é que fica vermelha quando alguém acrescenta uma quinta.*
///
/// ⚠️ Quem a mantém honesta é o `o_painel_tem_um_nome_por_operacao`: ele exige **nomes DISTINTOS**
/// para `0..N_OPERACOES`, o que apanha de uma vez uma contagem grande de mais e um braço em falta
/// no `op_name` do painel (que cairia no `_ => Brush` e daria dois nomes iguais).
pub const N_OPERACOES: usize = 4;

/// One of the composite operations. Wire discriminant (`to_u8`) travels in the panel snapshot so
/// the panel can label each row; the panel maps it back to a name.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompositeOp {
    Brush,
    Smear,
    Blur,
    /// **Apaga** — a mesma rota de depósito do `Brush` com o blend forçado a `EraseAlpha`.
    ///
    /// ⚠️ Ela **não** é o modo borracha da ferramenta: o `composite_active()` exige `!eraser`, e
    /// isso fica. Aqui a borracha é uma CAMADA, e o que a separa do pincel é uma linha
    /// (`brush.blend`). ⭐ Ela não tinge o que apaga porque o `BrushBlend::lays_pigment` — a cura de
    /// 2026-09-20 — já o impede no carimbo.
    Erase,
}

impl CompositeOp {
    /// Wire discriminant for the panel snapshot (`0` Brush · `1` Smear · `2` Blur · `3` Erase).
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::Brush => 0,
            Self::Smear => 1,
            Self::Blur => 2,
            Self::Erase => 3,
        }
    }

    /// Esta operação **deposita a cor do pincel**? (ou seja, a fileira de cor da camada tem sujeito)
    ///
    /// ⛔ O `Erase` responde `false`: ele escreve no ALFA e devolve o RGB do destino letra por letra
    /// — uma cor ali seria um controlo morto, que é a espécie que o §5.0 do `CLAUDE.md` nomeia.
    pub(crate) fn deposita_cor(self) -> bool {
        matches!(self, Self::Brush)
    }
}

/// **Até onde a borracha de uma camada chega** — ordem do dono, 2026-09-20: *«uma opção em erase:
/// se a borracha atua só no próprio traço do Brush ou se ela apaga também a camada da imagem
/// abaixo»*.
///
/// ⛔ O valor de fábrica é [`Self::Tudo`], que é o comportamento de sempre — a lei da casa manda
/// tudo o que é novo shipar desligado, e é ele que mantém a imagem de hoje **byte-idêntica**.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum EscopoDaBorracha {
    /// Come o alfa de tudo o que estiver por baixo, a imagem incluída.
    #[default]
    Tudo,
    /// Devolve o pixel ao que ele era **antes deste traço** — ou seja, apaga só a tinta que a
    /// própria pilha acabou de pôr. Mecanismo e álgebra em
    /// [`PainterTool::aplica_borracha_do_traco`].
    Traco,
}

/// **Quantos escopos o chip da borracha cicla.** ⚠️ Ele é lido pelo ciclo do clique E pela largura
/// medida do chip no painel — *duas respostas à mesma pergunta divergem no dia em que houver um
/// terceiro*.
pub const N_ESCOPOS_DA_BORRACHA: usize = 2;

impl EscopoDaBorracha {
    /// O discriminante que viaja no instantâneo do painel (`0` Tudo · `1` Traço).
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::Tudo => 0,
            Self::Traco => 1,
        }
    }
}

/// One composite stack layer: which operation sits here, its Strength (`0..1`; `0` = the layer is a
/// no-op and is skipped), its colour and its stamp size. Stored in a fixed `[_; N_CAMADAS]` in
/// display order (index 0 = layer 1 = top).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct CompositeLayer {
    pub op: CompositeOp,
    pub strength: f32,
    /// Até onde a borracha desta posição chega — inerte em toda operação que não é a borracha.
    pub erase_scope: EscopoDaBorracha,
    /// A cor DESTA camada, ou `None` = **a cor do pincel**.
    ///
    /// ⚠️ O `None` não é «sem cor», é *«segue quem manda»* — e é ele que mantém o **Randomize
    /// Color** vivo: com uma cor autorada a camada carimba um valor fixo e a variação por-dab que o
    /// motor escreveu no `Dab::color` é substituída. *Uma cor autorada por omissão apagaria um
    /// motor inteiro sem ninguém pedir.*
    pub color: Option<[f32; 3]>,
    /// A **dureza** DESTA camada, ou `None` = *a dureza do pincel*.
    ///
    /// ⭐ Ordem do dono, 2026-09-20: *«Hardness para cada um da lista»*. Ela é `Option` e não um
    /// multiplicador — ao contrário do [`Self::size`] — porque a dureza já é uma fracção `0..1`
    /// com significado absoluto: *«metade da dureza do pincel»* não é uma frase que o artista
    /// pense, e um multiplicador tornaria o ponto neutro dependente de onde o pincel está.
    ///
    /// ⚠️ **O `None` é o que mantém o controlo `Hardness` do pincel VIVO** para esta camada — a
    /// mesma lei do [`Self::color`], e é por isso que as duas têm o mesmo botão de volta.
    pub hardness: Option<f32>,
    /// O tamanho do carimbo DESTA camada, como **multiplicador do raio do pincel** (`1.0` = o
    /// tamanho do pincel).
    ///
    /// ⚠️ **Multiplicador e não pixels, de propósito:** a camada é uma variação do pincel que está
    /// na mão, e um número absoluto seria uma segunda resposta ao lado do slider `Size` — as duas a
    /// divergirem no dia em que uma ganhar um clamp. Os dois extremos são DERIVADOS, nunca
    /// escolhidos: ver [`PainterTool::tamanho_da_camada`].
    pub size: f32,
}

impl Default for CompositeLayer {
    fn default() -> Self {
        Self {
            op: CompositeOp::Brush,
            strength: 0.0,
            erase_scope: EscopoDaBorracha::Tudo,
            color: None,
            hardness: None,
            size: 1.0,
        }
    }
}

impl PainterTool {
    /// Whether the composite stack drives this stroke: the checkbox is on AND the active operation is
    /// the plain Brush (not a Smear/Blur/Eraser rail tool — composite is a Brush-tool upgrade).
    pub(crate) fn composite_active(&self) -> bool {
        self.paint.composite_enabled
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
    }

    /// Toggle the Composite Brush on/off (panel checkbox). Plain state — touches no pixels.
    pub fn toggle_composite(&mut self) {
        self.paint.composite_enabled = !self.paint.composite_enabled;
    }

    /// The Composite Brush enable flag (the panel snapshot mirrors it to show the card + hide Strength).
    #[must_use]
    pub fn composite_enabled(&self) -> bool {
        self.paint.composite_enabled
    }

    /// Set the Strength (`0..1`) of the composite layer at `pos` (`0` = layer 1 … `4` = layer 5).
    pub fn set_composite_layer_strength(&mut self, pos: usize, t: f32) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].strength = t.clamp(0.0, 1.0);
        }
    }

    /// Escolhe a OPERAÇÃO da camada em `pos` (o chip da fileira: Brush · Smear · Blur · Erase).
    pub fn set_composite_layer_op(&mut self, pos: usize, op: u8) {
        if pos >= N_CAMADAS {
            return;
        }
        self.paint.composite[pos].op = match op {
            1 => CompositeOp::Smear,
            2 => CompositeOp::Blur,
            3 => CompositeOp::Erase,
            _ => CompositeOp::Brush,
        };
    }

    /// Escreve a COR autorada da camada em `pos`. O picker chega aqui; o `None` chega pelo
    /// [`Self::clear_composite_layer_color`] (o clique-direito na amostra).
    pub fn set_composite_layer_color(&mut self, pos: usize, rgb: [f32; 3]) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].color = Some(rgb);
        }
    }

    /// Devolve a camada em `pos` à **cor do pincel** (e ao Randomize Color com ela).
    pub fn clear_composite_layer_color(&mut self, pos: usize) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].color = None;
        }
    }

    /// Escreve a DUREZA autorada da camada em `pos` (`0..1`).
    ///
    /// ⚠️ Ela não tem piso derivado como o tamanho: a faixa É o domínio da lei do perfil do dab, e
    /// os dois extremos são alcançáveis (um disco duro e um esfumado).
    pub fn set_composite_layer_hardness(&mut self, pos: usize, t: f32) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].hardness = Some(t.clamp(0.0, 1.0));
        }
    }

    /// Devolve a camada em `pos` à **dureza do pincel**.
    pub fn clear_composite_layer_hardness(&mut self, pos: usize) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].hardness = None;
        }
    }

    /// Escreve o TAMANHO da camada em `pos` (multiplicador do raio do pincel).
    ///
    /// ⚠️ O clamp aqui é o da PISTA (`0..MAX_TAMANHO_DA_CAMADA`); o piso que depende do Spacing é
    /// aplicado na LEITURA ([`Self::tamanho_da_camada`]), porque ele muda quando o artista mexe no
    /// Spacing e um valor gravado com o piso de ontem mentiria hoje.
    pub fn set_composite_layer_size(&mut self, pos: usize, t: f32) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].size = t.clamp(0.0, MAX_TAMANHO_DA_CAMADA);
        }
    }

    /// **O tamanho EFECTIVO da camada `pos`, com os dois extremos DERIVADOS** (§0.0 — um limite diz
    /// de que recurso ele é).
    ///
    /// - **Piso = o `spacing` do pincel.** O vão entre dois dabs é `spacing × 2r`, logo uma camada
    ///   de raio `size × r` só deixa de os sobrepor quando `size < spacing`: abaixo disso a lista
    ///   partilhada sai em CONTAS. *O piso não é um número escolhido — é o mesmo número que o
    ///   artista vê no controlo Spacing.*
    /// - **Tecto = [`MAX_TAMANHO_DA_CAMADA`]**, medido (ver a const).
    #[must_use]
    pub(crate) fn tamanho_da_camada(&self, pos: usize) -> f32 {
        let piso = self.paint.brush.spacing.clamp(1e-3, 1.0);
        self.paint.composite[pos]
            .size
            .clamp(piso, MAX_TAMANHO_DA_CAMADA)
    }

    /// Move the layer at `pos` one position UP (toward layer 1 / top) — swaps the tool with its upper
    /// neighbour. The position NUMBERS stay fixed; only which tool sits where changes. No-op at the top.
    pub fn move_composite_layer_up(&mut self, pos: usize) {
        if (1..N_CAMADAS).contains(&pos) {
            self.paint.composite.swap(pos, pos - 1);
        }
    }

    /// Move the layer at `pos` one position DOWN (toward layer 5 / bottom). No-op at the bottom.
    pub fn move_composite_layer_down(&mut self, pos: usize) {
        if pos + 1 < N_CAMADAS {
            self.paint.composite.swap(pos, pos + 1);
        }
    }

    /// The per-position operation discriminants — for the panel snapshot.
    pub(crate) fn composite_ops_u8(&self) -> [u8; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].op.to_u8())
    }

    /// The per-position layer Strengths — for the panel snapshot.
    pub(crate) fn composite_strengths(&self) -> [f32; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].strength)
    }

    /// As cores por posição, **já resolvidas** — a autorada, ou a do pincel quando não há.
    ///
    /// ⚠️ O painel recebe a cor RESOLVIDA (é o que a amostra tem de pintar) mais a bandeira de quem
    /// é autorada ([`Self::composite_color_authored`]): sem a segunda, uma camada que segue o
    /// pincel e uma que por acaso tem a mesma cor leem-se iguais na tela.
    pub(crate) fn composite_colors(&self) -> [[f32; 3]; N_CAMADAS] {
        std::array::from_fn(|i| {
            self.paint.composite[i]
                .color
                .unwrap_or(self.paint.brush.color)
        })
    }

    /// Quais posições têm cor AUTORADA (o resto segue o pincel) — para a amostra do painel.
    pub(crate) fn composite_color_authored(&self) -> [bool; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].color.is_some())
    }

    /// As durezas por posição, **já resolvidas** — a autorada, ou a do pincel quando não há.
    ///
    /// ⚠️ O par resolvida + autorada existe pela MESMA razão da cor: sem a bandeira, uma camada que
    /// segue o pincel e uma que por acaso autorou o mesmo número leem-se iguais na tela — e o botão
    /// de volta apareceria onde não há nada para limpar.
    pub(crate) fn composite_hardnesses(&self) -> [f32; N_CAMADAS] {
        std::array::from_fn(|i| {
            self.paint.composite[i]
                .hardness
                .unwrap_or(self.paint.brush.hardness)
        })
    }

    /// Quais posições têm dureza AUTORADA (o resto segue o pincel).
    pub(crate) fn composite_hardness_authored(&self) -> [bool; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].hardness.is_some())
    }

    /// Os tamanhos por posição **como o artista os autorou** (sem o piso do Spacing) — o painel
    /// pinta o que ele escreveu, e a lei aplica o piso ao ler ([`Self::tamanho_da_camada`]).
    pub(crate) fn composite_sizes(&self) -> [f32; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].size)
    }

    /// Escolhe até onde a borracha da posição `pos` chega (`0` Tudo · `1` Traço).
    pub fn set_composite_layer_erase_scope(&mut self, pos: usize, escopo: u8) {
        if pos < N_CAMADAS {
            self.paint.composite[pos].erase_scope = match escopo {
                1 => EscopoDaBorracha::Traco,
                _ => EscopoDaBorracha::Tudo,
            };
        }
    }

    /// Os escopos da borracha por posição, para o instantâneo do painel.
    pub(crate) fn composite_erase_scopes(&self) -> [u8; N_CAMADAS] {
        std::array::from_fn(|i| self.paint.composite[i].erase_scope.to_u8())
    }

    /// Route the Composite-card panel events (enable checkbox, per-position reorder buttons, per-position
    /// Strength sliders). Returns `true` iff consumed — chained ahead of the big `handle_panel_event` match.
    pub(crate) fn route_composite_event(&mut self, event: &PanelEvent) -> bool {
        match event {
            PanelEvent::Click(id) => {
                if *id == crate::ids::PAINTER_BRUSH_COMPOSITE_ENABLE {
                    self.toggle_composite();
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_UP
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_up(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_DOWN
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_down(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_OP
                    .iter()
                    .position(|x| x == id)
                {
                    // O chip CICLA as quatro operações — é o gesto que torna as posições novas
                    // alcançáveis (sem ele uma camada nasceria presa ao que o default declarou).
                    let proximo = (usize::from(self.paint.composite[p].op.to_u8() + 1) % N_OPERACOES) as u8;
                    self.set_composite_layer_op(p, proximo);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR
                    .iter()
                    .position(|x| x == id)
                {
                    self.clear_composite_layer_color(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR
                    .iter()
                    .position(|x| x == id)
                {
                    self.clear_composite_layer_hardness(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE
                    .iter()
                    .position(|x| x == id)
                {
                    // O chip CICLA os escopos — o mesmo gesto do chip da operação, e pela mesma
                    // razão: o `PanelEvent` é contrato congelado (§6) e não tem clique-direito.
                    let proximo =
                        (usize::from(self.paint.composite[p].erase_scope.to_u8() + 1)
                            % N_ESCOPOS_DA_BORRACHA) as u8;
                    self.set_composite_layer_erase_scope(p, proximo);
                    return true;
                }
                false
            }
            PanelEvent::SetValue(id, v) => {
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH
                    .iter()
                    .position(|x| x == id)
                {
                    self.set_composite_layer_strength(p, *v as f32);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS
                    .iter()
                    .position(|x| x == id)
                {
                    // A pista e o campo são a MESMA grandeza (`0..1`) — sem projecção nenhuma.
                    self.set_composite_layer_hardness(p, *v as f32);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_SIZE
                    .iter()
                    .position(|x| x == id)
                {
                    // ⚠️ A pista é `0..1` e o campo é um MULTIPLICADOR até `MAX_TAMANHO_DA_CAMADA`.
                    self.set_composite_layer_size(p, *v as f32 * MAX_TAMANHO_DA_CAMADA);
                    return true;
                }
                false
            }
            // ── A cor de uma camada, pelo picker partilhado: `"r,g,b"` em 8 bits nativos, a mesma
            //    rota (e o mesmo formato) do `PAINTER_COLOR_THUMB` do pincel. ──
            PanelEvent::SelectOption(id, value) => {
                let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_COLOR
                    .iter()
                    .position(|x| x == id)
                else {
                    return false;
                };
                let mut it = value.split(',');
                if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next())
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                {
                    self.set_composite_layer_color(p, [r, g, b].map(|v| f32::from(v) / 255.0));
                }
                true
            }
            _ => false,
        }
    }

    /// **A lista de dabs DESTA camada** — o raio escalado, a cor resolvida e, quando a camada é
    /// MAIOR que o pincel, a lista SUBAMOSTRADA pelo espaçamento dela.
    ///
    /// Devolve `None` quando nada muda (`size == 1` e sem cor autorada) — e aí o chamador entrega a
    /// **mesma fatia**, sem cópia nem alocação. *É isso que faz a pilha de hoje continuar
    /// byte-idêntica.*
    ///
    /// ⚠️ **A subamostragem é por ARCO e o acumulador é POR CAMADA**, em `composite_arco`: o
    /// `Dab::arc_len` é cumulativo desde o pen-down, logo a regra *«guarda este dab se ele está a
    /// pelo menos `spacing × 2 × r_camada` do último que guardei»* atravessa os lotes sem descontinuidade
    /// — sem esse estado, cada lote recomeçaria a contar e a camada grande carimbaria duas vezes na
    /// fronteira entre dois eventos.
    ///
    /// ⚠️ **O `stroke_radius_px` escala junto com o `radius_px`**, e não é decoração: ele é a
    /// unidade em que o **Flow** da Shape divide a coordenada ao longo do traço, e deixá-lo para
    /// trás re-fasearia o padrão da silhueta só nesta camada.
    /// A porta do gate para [`Self::camada_dabs`] — ⚠️ ela existe porque a metade que importa (*a
    /// camada grande recebe MENOS dabs*) não é observável no barro: uma faixa mais larga desenha-se
    /// igual com 40 dabs ou com 10. *Uma propriedade de CUSTO mede-se contando, não olhando.*
    #[cfg(test)]
    pub(super) fn camada_dabs_para_teste(&mut self, pos: usize, dabs: &[Dab]) -> Option<Vec<Dab>> {
        self.camada_dabs(pos, dabs)
    }

    fn camada_dabs(&mut self, pos: usize, dabs: &[Dab]) -> Option<Vec<Dab>> {
        let layer = self.paint.composite[pos];
        let escala = self.tamanho_da_camada(pos);
        let cor = layer.color.filter(|_| layer.op.deposita_cor());
        if (escala - 1.0).abs() < f32::EPSILON && cor.is_none() {
            return None;
        }
        let vao = if escala > 1.0 {
            // ⚠️ `spacing` é fracção do DIÂMETRO, e o raio da camada é `escala × radius_px` do dab.
            Some(self.paint.brush.spacing.max(1e-3) * 2.0 * escala)
        } else {
            None // menor que o pincel ⇒ fica com a lista inteira (ver o cabeçalho do módulo)
        };
        let mut saida = Vec::with_capacity(dabs.len());
        for d in dabs {
            if let Some(vao) = vao {
                let passo = vao * d.radius_px;
                let ultimo = self.paint.composite_arco[pos];
                if ultimo.is_finite() && d.arc_len - ultimo < passo {
                    continue;
                }
                self.paint.composite_arco[pos] = d.arc_len;
            }
            let mut n = *d;
            n.radius_px *= escala;
            n.stroke_radius_px *= escala;
            if let Some(c) = cor {
                n.color = c;
            }
            saida.push(n);
        }
        Some(saida)
    }

    /// Stamp a dab batch through the composite stack (bottom → top). Each operation reuses its own route
    /// (`stamp_dabs_inner` / `stamp_dabs_smear` / `stamp_dabs_blur`) with the layer's Strength swapped
    /// into the brush spec; a zero-Strength layer is skipped. The per-op routes each read/write the
    /// canvas in place, so an upper op sees the lower op's result (the "affects the combination" order).
    ///
    /// ⛔⛔ **O `stroke_mask` é TROCADO por camada, e sem isso um segundo Brush não pinta.** O cap de
    /// Accumulate é um mapa de cobertura **por TRAÇO**; com duas camadas a partilhá-lo, a primeira
    /// leva-o ao tecto e a segunda deposita **ZERO** — medido em 2026-09-20 antes desta wave: com
    /// Strength `0,6` a tinta lia `61,04` com uma camada e `61,04` com duas, quando dois passes
    /// dariam `~86` (`1 − 0,4²`). *O número era exactamente o cap, ao décimo.* A troca é um
    /// `mem::swap` (`O(1)`), e cada camada só paga o mapa dela se de facto o armar.
    pub(super) fn stamp_dabs_composite(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        // ⚠️ As listas por camada resolvem-se UMA vez e ANTES de qualquer decisão: o
        // [`Self::camada_dabs`] consome o acumulador de arco daquela posição (estado por TRAÇO), e
        // chamá-lo duas vezes para o mesmo lote entregaria duas subamostragens diferentes.
        let camadas: [Vec<Dab>; N_CAMADAS] = std::array::from_fn(|pos| {
            if self.paint.composite[pos].strength <= 0.0 {
                Vec::new()
            } else {
                self.camada_dabs(pos, dabs)
                    .unwrap_or_else(|| dabs.to_vec())
            }
        });
        // ⛔ **Com menos de duas camadas activas não há ordem para arrumar** — o caminho é o de
        // sempre, byte-idêntico, e nem a fotografia do `pre` é paga.
        if self.camadas_activas() < 2 {
            // ⚠️ **No máximo UMA posição faz alguma coisa aqui**, logo não há ordem para escolher —
            // e um `for … .rev()` neste braço seria uma linha que a mutação não consegue matar
            // (medido: invertê-lo não move um bit). O `find` diz a mesma coisa e diz que é UMA.
            if let Some(pos) = (0..N_CAMADAS).find(|&p| self.paint.composite[p].strength > 0.0) {
                self.aplica_camada(pos, &camadas[pos]);
            }
            return;
        }
        self.recompoe_a_pilha(camadas);
    }
}
