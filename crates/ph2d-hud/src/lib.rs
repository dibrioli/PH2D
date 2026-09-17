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
