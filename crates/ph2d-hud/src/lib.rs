//! **O HUD: onde a raiz de um placar se cola à vista do jogo** (TOP-20 #20).
//!
//! A lei é uma só e responde a uma pergunta: *o artista desenhou o HUD numa caixa de referência —
//! para onde é que essa caixa vai, quando a vista do jogo tem outro tamanho?*
//!
//! # Porque isto é uma FOLHA de aritmética fechada
//!
//! A raiz do HUD é uma entidade como outra qualquer, e os filhos dela herdam pose por `Transform`.
//! Tudo o que falta é **a pose da raiz**, e ela é uma função pura de dois rectângulos. Não há
//! solver, não há convergência, não há estado entre quadros — é o mesmo argumento que o
//! [`ph2d_ecs::VecAnchors`] escreve para recusar o Cassowary.
//!
//! # O ORÁCULO (Godot 4.7.2, MIT, corrido sem interface)
//!
//! Medido em `docs/Components/ferramentas/godot_hud_probe.gd` (janela `720×450`, a referência a
//! variar), bloco L2 — `Window.content_scale_aspect`:
//!
//! | modo dele | a lei | a nossa |
//! |---|---|---|
//! | `ignore` | escala por EIXO, `J/R`, sem deslocamento | [`Fit::Stretch`] |
//! | `keep` | uniforme `min(Jx/Rx, Jy/Ry)`, **centrado** | [`Fit::Keep`] |
//! | `expand` | uniforme, **sem centrar** (canto) | ⛔ **não portado** — ver abaixo |
//! | `keep_width`/`keep_height` | redimensionam o VIEWPORT | ⛔ não portado |
//!
//! ⛔⛔ **O `expand` não é exprimível nesta composição, e a razão é geométrica.** Ali ele quer
//! dizer *«mantém o aspecto E deixa os filhos ancorados chegarem às bordas REAIS»* — e quem ancora
//! nesta casa é o [`ph2d_ecs::VecAnchors`], que mede contra a **caixa local da moldura**. Fazer os
//! filhos chegarem à borda exigiria **redimensionar a moldura por quadro**, que é escrever no
//! DOCUMENTO (a geometria de um `VecPath`), e o documento não se toca num passe derivado. Com a
//! caixa centrada — que é o idioma desta casa, num mundo Y-up — o `expand` e o `keep` dão a MESMA
//! imagem, e uma terceira entrada que não distingue nada é um controlo morto à nascença.
//!
//! ⛔ **Divergência DECLARADA:** o alvo **arredonda a banda do letterbox a pixel inteiro e encolhe
//! a escala para caber** (`keep`, ref `1280×360`: escala `y = 0,561111` e banda `124`, em vez de
//! `0,5625` e `123,75`). O nosso canvas vive em **metros** e não em pixels — aqui o valor é exacto,
//! e há gate a afirmá-lo pelos dois lados.

#![forbid(unsafe_code)]

/// **Como a caixa de referência se acomoda numa vista de outro tamanho.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum Fit {
    /// Uniforme (`min` dos dois factores) e **centrado** — o que sobra fica como banda. É o
    /// `keep` do alvo, e o valor de fábrica: um HUD que estica o texto lê-se como um defeito.
    #[default]
    Keep,
    /// Um factor **por eixo** — a caixa preenche a vista e a forma distorce. É o `ignore` do alvo.
    Stretch,
    /// Uniforme como o [`Fit::Keep`] — **e a caixa CRESCE no eixo curto**, de modo que um filho
    /// ancorado alcance as bordas REAIS da vista em vez de parar na banda do letterbox. É o
    /// `expand` do alvo.
    ///
    /// ⚠️ **A imagem do que está DENTRO da caixa de referência é a mesma do `Keep`** (a escala e a
    /// translação são idênticas — há gate). O que muda é só até onde uma ÂNCORA pode ir.
    ///
    /// ⚠️ **Append-only:** a posição é a tag do postcard, logo ele entra no FIM.
    Expand,
}

impl Fit {
    /// Todos, na ordem em que o selector os mostra — que é a ordem da tag.
    ///
    /// ⚠️ **A lista é a FONTE**, e o índice do painel deriva dela. A 1.ª redacção do Inspector
    /// mapeava `0`/`1` **à mão** (`u8::from(fit == Stretch)`), e um modo novo teria existido, com
    /// lei e gates, **sem o artista lhe chegar** — o defeito que o `Density` da escultura e o verbo
    /// do FIM DE JOGO pagaram, cada um por um array escrito à mão.
    pub const ALL: [Self; 3] = [Self::Keep, Self::Stretch, Self::Expand];

    /// O rótulo que o artista lê. Inglês (HR-15).
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Keep => "hud.fit.keep",
            Self::Stretch => "hud.fit.stretch",
            Self::Expand => "hud.fit.expand",
        }
    }

    /// O índice deste modo no selector.
    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|f| *f == self).unwrap_or(0)
    }

    /// O modo de um índice do selector. ⚠️ Fora de alcance cai no de fábrica — um ficheiro
    /// estragado não escolhe um modo que ninguém autorou.
    #[must_use]
    pub fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }
}

/// **A caixa em que o artista desenhou o HUD**, em unidades de mundo, CENTRADA na entidade.
///
/// ⚠️ Centrada, e não com a origem num canto: é o idioma desta casa (um mundo Y-up, poses no
/// centro), e é o que faz o centramento do [`Fit::Keep`] cair de graça na translação.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Canvas {
    ref_w: f32,
    ref_h: f32,
    /// Ver [`Fit`].
    pub fit: Fit,
}

impl Canvas {
    /// A caixa de referência, ou `None` se ela não for um rectângulo utilizável.
    ///
    /// ⚠️ **A recusa é a lei, não uma cerca defensiva:** um lado `0` faria o factor de escala ser
    /// uma divisão por zero — `inf` — e a pose conduzida levaria o HUD inteiro para fora de
    /// qualquer vista, em silêncio. Quem chama trata o `None` dizendo-o em voz alta.
    #[must_use]
    pub fn new(ref_w: f32, ref_h: f32, fit: Fit) -> Option<Self> {
        (ref_w.is_finite() && ref_h.is_finite() && ref_w > 0.0 && ref_h > 0.0).then_some(Self {
            ref_w,
            ref_h,
            fit,
        })
    }

    /// A largura da caixa de referência.
    #[must_use]
    pub fn ref_w(&self) -> f32 {
        self.ref_w
    }

    /// A altura da caixa de referência.
    #[must_use]
    pub fn ref_h(&self) -> f32 {
        self.ref_h
    }
}

/// **A vista do jogo**, exactamente na forma que a fase da câmera já devolve: centro e
/// meia-janela, em metros.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct View {
    /// O centro da vista, em mundo.
    pub center: [f32; 2],
    /// Meia largura e meia altura, em metros. ⚠️ **Meia**, porque é o que o
    /// `fase_game_camera` devolve — converter na fronteira seria a segunda resposta à mesma
    /// pergunta.
    pub half: [f32; 2],
}

/// **A pose que a raiz do HUD tem de ter neste quadro.**
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Placement {
    /// Escala por eixo.
    pub scale: [f32; 2],
    /// Translação de mundo.
    pub translate: [f32; 2],
}

/// **A porta única.** A caixa de referência sobre a vista, pela regra do [`Fit`].
///
/// ⚠️ **A translação é SEMPRE o centro da vista**, nos dois modos: é isso que faz o centramento do
/// `keep` não precisar de aritmética nenhuma. O que o oráculo chama de *deslocamento* é, aqui, a
/// BANDA que sobra — e ela é derivada, nunca somada à pose (ver [`bands`]).
#[must_use]
pub fn place(canvas: &Canvas, view: View) -> Placement {
    let fx = (2.0 * view.half[0]) / canvas.ref_w;
    let fy = (2.0 * view.half[1]) / canvas.ref_h;
    let scale = match canvas.fit {
        Fit::Keep => {
            let s = fx.min(fy);
            [s, s]
        }
        Fit::Stretch => [fx, fy],
        // ⭐ **A POSE do `Expand` é a do `Keep`, ao bit** — o que ele muda é só até onde uma
        // ÂNCORA pode ir (a caixa efectiva), e não como a caixa é desenhada. Há gate.
        Fit::Expand => {
            let s = fx.min(fy);
            [s, s]
        }
    };
    Placement {
        scale,
        translate: view.center,
    }
}

/// **A BANDA que sobra de cada lado** (metade da diferença, por eixo) — o número que o
/// `get_final_transform` do alvo devolve como deslocamento.
///
/// ⚠️ Ela existe para ser **medida contra o oráculo** e para o painel a poder mostrar; ⛔ ela
/// **não** entra na pose — somá-la seria descentrar o que já está centrado.
#[must_use]
pub fn bands(canvas: &Canvas, view: View) -> [f32; 2] {
    let p = place(canvas, view);
    [
        (2.0 * view.half[0] - canvas.ref_w * p.scale[0]) / 2.0,
        (2.0 * view.half[1] - canvas.ref_h * p.scale[1]) / 2.0,
    ]
}

/// **A CAIXA EFECTIVA do canvas neste quadro, em unidades LOCAIS dele** — a de referência,
/// crescida pela banda que sobra de cada lado.
///
/// `[x0, y0, x1, y1]`, Y-up e **centrada na origem**, que é o idioma da caixa de referência.
///
/// # ⭐⭐ Porque ela existe: a âncora estava INERTE, e não por ligar
///
/// O [handoff do #20](../../../docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md)
/// §7 escreve *«as quatro âncoras do `VecAnchors` não estão ligadas ao canvas»* — e a sonda do §5.0
/// corrige a redacção: **elas estão ligadas e são inertes.** O `VecAnchors::delta_local` responde
/// *«a moldura mudou de tamanho?»*, e a caixa do canvas é **a mesma em toda janela** — o que muda é
/// a ESCALA da raiz. ⇒ *a régua media uma grandeza que não se mexe*, e o delta era `0,0` por
/// subtracção de iguais.
///
/// Esta porta dá-lhe a grandeza que **de facto** muda. Medido (ref `32 × 18`, `Fit::Keep`):
///
/// | vista | banda | caixa efectiva em `x` | `dmax` de um filho preso à direita |
/// |---|---|---|---|
/// | `16:9` | `0,0` | `[−16, 16]` | **`0,000`** |
/// | `21:9` | `5,0` | `[−21, 21]` | `5,000` |
/// | `4:3` | `0,0` (em `y`: `3,0`) | `[−16, 16]` | `0,000` (em `y`: `3,000`) |
///
/// ⭐ **O neutro é EXACTO**: no aspecto da própria caixa a banda é zero, a efectiva **É** a de
/// referência, e o delta sai `0,0` por subtracção de iguais ⇒ o que se desenha é **byte-idêntico**
/// ao de antes desta porta.
///
/// # ⛔⛔ E ela DISSOLVE a recusa declarada do `Fit::Expand`
///
/// O cabeçalho desta crate recusa o `expand` porque *«fazer os filhos chegarem à borda exigiria
/// redimensionar a moldura por quadro, que é escrever no DOCUMENTO»*. ⚠️ **A premissa caiu:** a
/// caixa efectiva é **derivada por quadro**, exactamente como a pose da raiz, e não toca no
/// documento. *Quem move o número que tornava algo inalcançável tem de reconferir a nota*
/// (`CLAUDE.md` §0.0) — e é esta porta que o move.
///
/// ⚠️ **Com `Fit::Stretch` ela devolve a caixa de referência ao bit**, e isso é a lei e não um
/// caso por cobrir: ali não há banda nenhuma — a caixa já preenche a vista nos dois eixos.
#[must_use]
pub fn effective_box(canvas: &Canvas, view: View) -> [f32; 4] {
    let p = place(canvas, view);
    let b = bands(canvas, view);
    // A banda é de MUNDO; a caixa é LOCAL ⇒ ela atravessa a escala da raiz.
    //
    // ⚠️ A escala nunca é zero: o `Canvas::new` recusa uma referência não-positiva, e a vista vem
    // de uma câmera com meia-extensão positiva. A guarda existe para o caso degenerado não dar
    // `inf` em silêncio — um `NaN` aqui viajaria até à pose de todo filho ancorado.
    // ⛔⛔ **SÓ o `Expand` cresce, e o oráculo é quem o diz** (bloco L4, medido 2026-09-19 com a
    // janela `720×450` do headless):
    //
    // | aspecto | ref | a caixa que o filho ancorado lê | o canto dele no ecrã |
    // |---|---|---|---|
    // | `keep` | `640×360` | `(640, 360)` — **a referência** | `(720, 428)`, a `22 px` da borda |
    // | `keep` | `1280×360` | `(1280, 360)` | `(720, 326)`, a `124` |
    // | `expand` | `640×360` | **`(640, 400)`** | `(720, 450)` — **a borda** |
    // | `expand` | `1280×360` | `(1280, 800)` | `(720, 450)` |
    // | `expand` | `320×480` | `(768, 480)` | `(720, 450)` |
    //
    // ⚠️⚠️ A 1.ª redacção desta porta crescia no `Keep`, e isso fazia o `Keep` comportar-se como o
    // `expand` do alvo — uma divergência **silenciosa** que retirava a capacidade de confinar o HUD
    // à área segura. *O modo que o artista escolhe tem de significar o que o alvo diz que
    // significa.*
    if canvas.fit != Fit::Expand {
        return [
            -canvas.ref_w / 2.0,
            -canvas.ref_h / 2.0,
            canvas.ref_w / 2.0,
            canvas.ref_h / 2.0,
        ];
    }
    let local = |banda: f32, escala: f32| if escala > 0.0 { banda / escala } else { 0.0 };
    let hx = canvas.ref_w / 2.0 + local(b[0], p.scale[0]);
    let hy = canvas.ref_h / 2.0 + local(b[1], p.scale[1]);
    [-hx, -hy, hx, hy]
}

/// **O que um rótulo do HUD mostra**, antes de virar texto.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Valor {
    /// Uma contagem — pontos, vidas, quantos inimigos restam.
    Inteiro(i64),
    /// Um tempo que falta, em segundos.
    Segundos(f32),
}

/// **O número, em texto.**
///
/// ⚠️ **Um tempo nunca sai NEGATIVO:** um relógio que já tocou mostra `0.0`, não `-1.3`. O valor
/// negativo existe no motor (é quanto ele passou do fim) e mostrá-lo seria ensinar ao jogador uma
/// coisa que não é sobre o jogo.
///
/// ⚠️ **Uma casa decimal, e o ponto é o separador**: este texto é CONTEÚDO do jogo (fica ao lado do
/// prefixo que o artista escreveu), não uma etiqueta da interface do editor — a vírgula decimal do
/// `ph2d-i18n` governa o segundo caso, não este.
#[must_use]
pub fn formata(v: Valor) -> String {
    match v {
        Valor::Inteiro(n) => n.to_string(),
        Valor::Segundos(s) => format!("{:.1}", s.max(0.0)),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// **O que o dedo fez** — os dois momentos que decidem um botão.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gesto {
    /// O dedo pousou.
    Baixo,
    /// O dedo levantou.
    Cima,
}

/// **A lei do clique de um botão**, medida no oráculo (Godot 4.7.2, bloco L3 de
/// `godot_hud_probe.gd`): ele dispara **uma vez, ao LARGAR**, e só se o carregar **e** o largar
/// caírem no MESMO botão.
///
/// `premido` é a memória do gesto (em que botão o dedo pousou) e `sob_o_cursor` o que está debaixo
/// dele agora — `None` quando não há botão alcançável ali. Devolve a memória NOVA e se há sinal a
/// publicar.
///
/// | o gesto | o alvo |
/// |---|---|
/// | baixo DENTRO, cima DENTRO | ⭐ publica |
/// | baixo DENTRO, cima FORA | não publica |
/// | baixo FORA, cima DENTRO | não publica |
///
/// ⚠️ **Um `Baixo` fora limpa a memória**, e isso é a lei e não uma cerca: sem isso um gesto
/// abandonado noutro sítio ficaria armado, e o largar seguinte publicaria um botão em que o dedo
/// nunca pousou.
///
/// ⚠️ **O desactivado não chega aqui:** quem resolve `sob_o_cursor` já responde `None` para um botão
/// desactivado ou sem nome — são as duas maneiras de um botão não ser um botão, e separá-las daria
/// dois sítios para decidir.
#[must_use]
pub fn clique<T: Copy + PartialEq>(
    gesto: Gesto,
    premido: Option<T>,
    sob_o_cursor: Option<T>,
) -> (Option<T>, bool) {
    match gesto {
        Gesto::Baixo => (sob_o_cursor, false),
        Gesto::Cima => (None, premido.is_some() && premido == sob_o_cursor),
    }
}
