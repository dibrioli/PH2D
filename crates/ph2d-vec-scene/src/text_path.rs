//! **O referencial de um glifo que cavalga um caminho** — [`GlyphFrame`].
//!
//! Este é o motor do *texto em caminho* (doc 22, W2), e ele **não sabe o que é um glifo**: recebe
//! uma posição de arco e devolve o afim que leva o **espaço de texto** ao mundo. Quem sabe o que
//! é uma letra — a fonte, o avanço, o ascender — é o shell; quem sabe o que é um arco é esta
//! crate. Foi essa divisão que manteve o [`crate::arc_path`] sem opinião sobre amostragem, e é a
//! mesma aqui.
//!
//! # Espaço de texto
//!
//! `apply([dx, dy])`, com `dx` = deslocamento **ao longo da linha** a partir do ponto de âncora do
//! glifo, e `dy` = altura **acima da baseline**. É o sistema em que um glifo já está desenhado, o
//! que torna a conversão uma multiplicação e nada mais.
//!
//! # A âncora é o MEIO do glifo, e isso é normativo
//!
//! O caminho é amostrado no **centro** do glifo (`pen + avanço/2`), não na borda esquerda — a
//! especificação do SVG diz `mid = x + advance / 2 + offset` e roda em torno desse ponto. Não é
//! preciosismo: ancorar pela borda esquerda faz a letra girar **para fora** da linha, e é a
//! origem de metade dos artefatos que se veem em curva apertada.
//!
//! # ⚠️ O que este módulo NÃO faz, e a razão
//!
//! O Illustrator vende **cinco** orientações — *Rainbow · Skew · 3D Ribbon · Stair Step ·
//! Gravity*. Só a **Rainbow** está aqui, porque só ela tem especificação **aberta e normativa**:
//! ela é o `<textPath>` do SVG, e o comportamento é verificável contra a spec.
//!
//! As outras quatro eu conheço pelo *nome* e pela descrição de segunda mão. Duas delas — Skew e
//! 3D Ribbon — são **duais** (uma preserva as arestas verticais do glifo, a outra as
//! horizontais), e implementar a trocada produziria um efeito coerente, bonito, e **com o rótulo
//! errado**. É exatamente o defeito que o Painter pagou quando `Falloff::Sharp` foi armado no
//! lugar de `Pow4` porque o *identificador* casava com a palavra da UI. Um efeito que faz outra
//! coisa que não a que o nome promete é pior do que um efeito que falta.
//!
//! Elas entram quando a spec estiver **fixada por uma fonte que se possa citar**, não por memória.
//!
//! # Rigidez
//!
//! A Rainbow é uma **rotação** — `x_axis` e `y_axis` saem unitários e ortogonais. É isso que faz
//! o glifo chegar ao mundo sem cisalhar nem esticar, e é isso que dispensa qualquer refit: um
//! afim **comuta** com a avaliação de Bézier ([`crate::arc_path`] e o ADR-0129 §1), então a curva
//! transformada É a imagem da curva. O envelope precisa de `fit_to_bezpath` porque o mapa dele
//! não é afim; aqui não há o que aproximar.

use crate::arc_path::ArcPath;

/// O referencial de um glifo sobre o caminho: uma origem e os dois eixos que levam o **espaço de
/// texto** ao mundo.
///
/// Para a Rainbow os eixos são **ortonormais** (é uma rotação pura); o tipo não o impõe, para que
/// as orientações não-rígidas caibam aqui quando a spec delas for fixada.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GlyphFrame {
    /// Onde o ponto `[0, 0]` do espaço de texto cai no mundo.
    pub origin: [f64; 2],
    /// A imagem do vetor `[1, 0]` — a direção "ao longo da linha".
    pub x_axis: [f64; 2],
    /// A imagem do vetor `[0, 1]` — a direção "acima da baseline".
    pub y_axis: [f64; 2],
}

impl GlyphFrame {
    /// **O referencial Rainbow** no arco `s`: o glifo cavalga o caminho rigidamente.
    ///
    /// - `s` — a posição de arco do **centro** do glifo (§ do módulo).
    /// - `dy` — o desvio da baseline, em unidades de mundo, **positivo para a ESQUERDA do sentido
    ///   de marcha**. Numa linha da esquerda para a direita isso é **acima da baseline**, que é a
    ///   convenção do texto — e é essa a razão de ser este o sinal, e não "para fora da curva".
    ///   *Fora* depende do **winding**: num círculo anti-horário a esquerda aponta para o centro.
    ///   É aqui que entra o *Align to Path* (Baseline · Center · Ascender · Descender): cada um é
    ///   um `dy` diferente, computado das **métricas da fonte** — que esta crate não conhece nem
    ///   deve conhecer.
    /// - `flip` — põe o texto **do outro lado**, a ler no sentido oposto.
    ///
    /// `None` numa **cúspide**, onde não há tangente: sem direção não há referencial, e inventar
    /// um punha o glifo num ângulo que ninguém autorou. Quem chama decide (saltar o glifo, ou
    /// herdar o referencial do anterior) — esta função não escolhe por ele.
    #[must_use]
    pub fn on_path(path: &ArcPath, s: f64, dy: f64, flip: bool) -> Option<Self> {
        // Virar o texto é percorrer o caminho ao contrário: o arco conta da outra ponta e a
        // tangente inverte. A normal inverte JUNTO (é a perpendicular da tangente), então o `dy`
        // passa para o outro lado da curva sozinho — que é exatamente o que "flip" significa, e
        // é por isso que não há um segundo sinal a lembrar.
        let (point, tangent) = if flip {
            let (p, t) = path.frame_at(path.total() - s);
            (p, [-t[0], -t[1]])
        } else {
            path.frame_at(s)
        };
        // O [`ArcPath`] devolve o vetor NULO numa cúspide — não um unitário qualquer.
        if tangent[0] == 0.0 && tangent[1] == 0.0 {
            return None;
        }
        // A normal é a tangente rodada um quarto de volta: troca de eixo e sinal, sem
        // transcendental (HR-5) — a mesma construção do Zig Zag.
        let normal = [-tangent[1], tangent[0]];
        Some(Self {
            origin: [
                normal[0].mul_add(dy, point[0]),
                normal[1].mul_add(dy, point[1]),
            ],
            x_axis: tangent,
            y_axis: normal,
        })
    }

    /// O MESMO referencial com a origem deslocada `dx` **ao longo da linha**.
    ///
    /// Existe por causa da âncora ao meio (§ do módulo): o caminho é amostrado no CENTRO do
    /// glifo, mas um contorno de glifo é desenhado a partir do **pen origin** dele. Recuar a
    /// origem meio avanço é exatamente equivalente a somar `-avanço/2` a todo `local[0]` —
    /// `apply([x − h, y]) = origem + eixo_x·(x − h) + eixo_y·y = (origem − eixo_x·h) + …` — e
    /// fazê-lo UMA vez por glifo, em vez de por ponto de contorno, mantém o `apply` a puro afim.
    ///
    /// Note que isto **não** é o mesmo que amostrar o caminho meio avanço atrás: ali a curva
    /// mudaria de direção entre os dois pontos, aqui o glifo inteiro partilha o referencial do
    /// seu centro — que é o que o mantém rígido.
    #[must_use]
    pub fn shifted_along(&self, dx: f64) -> Self {
        Self {
            origin: [
                self.x_axis[0].mul_add(dx, self.origin[0]),
                self.x_axis[1].mul_add(dx, self.origin[1]),
            ],
            ..*self
        }
    }

    /// Leva um ponto do **espaço de texto** ao mundo.
    ///
    /// `local[0]` = ao longo da linha a partir da âncora do glifo · `local[1]` = acima da
    /// baseline. Um glifo inteiro passa por aqui ponto a ponto — âncoras **e alças** —, e como o
    /// mapa é afim a curva que sai é a imagem exata da que entrou.
    #[must_use]
    pub fn apply(&self, local: [f64; 2]) -> [f64; 2] {
        [
            self.x_axis[0].mul_add(local[0], self.y_axis[0].mul_add(local[1], self.origin[0])),
            self.x_axis[1].mul_add(local[0], self.y_axis[1].mul_add(local[1], self.origin[1])),
        ]
    }
}

#[cfg(test)]
#[path = "text_path_tests.rs"]
mod tests;
