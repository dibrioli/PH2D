//! **O FRAME DO PADRÃO** — a direção em que um alpha direcional é lido.
//!
//! ⚠️ **Módulo irmão do [`super`], e o corte é por RESPONSABILIDADE:** lá mora *o
//! que um padrão É* (as nove fórmulas, o hash, o ruído); aqui mora *em que
//! direção ele é lido*. As duas perguntas têm consumidores diferentes — o painel
//! pergunta a esta para OFERECER as duas pistas de eixo, e o motor pergunta à de
//! lá para saber que número sai do vértice.
//!
//! ⚠️ **E o corte tem preço, então ele é nomeado:** o [`AlphaFrame`] guarda campos
//! PRIVADOS e o `weight_at` do irmão os lê pela [`AlphaFrame::project`], que é
//! `pub(super)` — a fronteira do módulo é o que garante que ninguém construa um
//! frame torto por fora.

use ph2d_painter_brush::texture::rotate_by_degrees;

/// **A VISTA, entregue ao pincel** — o que faz de uma imagem um ESTÊNCIL preso ao
/// viewport em vez de um padrão colado no barro.
///
/// ⚠️ **Ele é a resposta a *"onde está a tela, deste objeto?"*, e por isso vem em
/// espaço LOCAL do objeto** — que é o espaço em que o `weight_at` recebe os
/// pontos. Converter no motor exigiria que ele conhecesse poses e câmeras; quem
/// as conhece é a cena, e ela responde uma vez por peça por quadro.
///
/// ⚠️ **`right` e `up` são ORTONORMAIS e continuam ortonormais em espaço local**
/// porque a pose deste módulo tem escala ESCALAR. Uma escala não-uniforme
/// cisalharia o par, e o frame deixaria de ser uma base — a mesma razão pela qual
/// o collider da física nomeia o skew como limite honesto em vez de fingir.
///
/// ⚠️ **Ele é o FRUSTUM, e não uma régua num ponto — e essa é a diferença que o
/// primeiro corte pagou.** A versão anterior guardava *quantas unidades de objeto
/// a altura da tela abrange*, um número que só é verdade numa PROFUNDIDADE; quem
/// o montava tinha de escolher onde perguntar, e os dois consumidores escolheram
/// pontos diferentes — o dab no ACERTO, o preview no CENTRO da peça. Medido na
/// cena do smoke: **+24,8%** de diferença de tamanho na frente do modelo e −16,6%
/// atrás, ou seja *a tinta que o artista via não era a que a ferramenta
/// depositava* (Enio, 2026-08-09). Guardando o olho e a RAZÃO, a profundidade
/// entra por vértice e **a pergunta "onde eu meço?" deixa de existir** — a
/// representação apaga o caso especial, como a bola limitada do Inflate apagou as
/// quatro cercas do Painter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaStencil {
    /// A direita da TELA, em espaço local, unitária.
    pub right: [f32; 3],
    /// O cima da TELA, em espaço local, unitário.
    pub up: [f32; 3],
    /// **O OLHO**, em espaço local — de onde as linhas de visão saem.
    pub eye: [f32; 3],
    /// **Quantas unidades de objeto a ALTURA da tela abrange POR UNIDADE DE
    /// PROFUNDIDADE** — `2·tan(fov/2)`, a razão que define o frustum.
    ///
    /// ⚠️ **É uma RAZÃO, e é por isso que ela não precisa da pose:** ela é
    /// adimensional, então vale igual em mundo e em espaço local por mais que a
    /// peça esteja escalada. A escala entra uma única vez, no [`Self::eye`].
    ///
    /// ⚠️ **E é ela que torna o carimbo imune ao ZOOM sem escolher um ponto:**
    /// aproximar não muda a razão — muda a profundidade de cada vértice —, então
    /// um ladrilho medido em fração de tela encolhe em unidades de objeto na
    /// proporção exata e fica do mesmo tamanho para quem olha, **em toda parte do
    /// modelo ao mesmo tempo**.
    pub height_per_depth: f32,
}

impl AlphaStencil {
    /// A vista CANÔNICA — a tela olhando por `−Z`, sem pose nenhuma.
    ///
    /// ⚠️ **Ela existe para quem NÃO tem câmera e ainda assim tem de desenhar o
    /// estêncil de frente**: o retrato do painel. Um swatch que montasse a
    /// própria base seria a segunda resposta a *"como um estêncil é lido?"*, e a
    /// que mente é a que o artista está olhando.
    ///
    /// ⚠️ **O olho a uma unidade e a razão em 1,0 não são números bonitos — eles
    /// tornam o retrato BYTE-IDÊNTICO ao que ele já desenhava.** O swatch amostra
    /// o plano `z = 0`, cuja profundidade contra este olho é exatamente `1`, e a
    /// divisão de perspectiva vira `1.0 / (1.0 · 1.0)`: multiplicar por um é
    /// exato em IEEE-754, então o retrato sai sem distorção nenhuma — que é o que
    /// um retrato deve ser.
    pub const CANONICAL: Self = Self {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        eye: [0.0, 0.0, 1.0],
        height_per_depth: 1.0,
    };
}

/// **O FRAME em que um padrão DIRECIONAL é lido** — três eixos ortonormais em
/// espaço de OBJETO.
///
/// ⚠️ **Ele é derivado UMA vez por dab, nunca por vértice**, e a razão é medida:
/// a conversão ângulo→vetor deste app é o rotor de um grau ACUMULADO
/// (`rotate_by_degrees`, o mesmo do Jitter Rotate e da luz do impasto), que é
/// **O(graus)** — até 359 iterações. Por vértice ele custaria mais que o padrão
/// inteiro que ele orienta.
///
/// ⚠️ **E ele não tem `Default`:** o único jeito de obter um é
/// [`crate::Brush::alpha_frame`], então *"esqueci de passar o frame do pincel"*
/// não é um erro que se comete distraído. É a lição do `ShapeFrame` do Painter
/// 2D, que existe pela mesma razão — lá um builder opcional fez a feature chegar
/// em 2 de 7 rotas, em silêncio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaFrame {
    /// **O EIXO: a direção em que o padrão VARIA.** O Strata empilha ao longo
    /// dele, as faixas do Scratches são indexadas por ele, e a trama do Weave é
    /// o plano perpendicular a ele.
    n: [f32; 3],
    /// A tangente, sempre no plano XY do objeto — ver [`AlphaFrame::placed`].
    t: [f32; 3],
    /// `n × t`, completando a base.
    b: [f32; 3],
    /// **ONDE o padrão POUSA no plano `t`–`b`**, em unidades de OBJETO.
    ///
    /// ⚠️ **O frame deixou de ser só ORIENTAÇÃO e virou COLOCAÇÃO** — *para onde
    /// o padrão aponta* e *onde ele pousa* —, e ele é o lugar certo para a
    /// segunda porque já é o objeto que atravessa [`crate::Brush::alpha_frame`]
    /// até o `weight_at`. Um parâmetro NOVO obrigaria todo chamador a passá-lo,
    /// e é exatamente assim que uma feature chega a algumas rotas e não a outras
    /// — a lição que a `ShapeFrame` do Painter 2D pagou (lá foram 2 de 7, em
    /// silêncio).
    ///
    /// ⚠️ **Em unidades de OBJETO, e não em LADRILHOS**, porque é a mesma régua
    /// do `alpha_scale`: mover o padrão de um `alpha_scale` anda exatamente um
    /// ladrilho, e **redimensionar não faz o padrão escorregar** — a escala
    /// re-escala em torno do ponto que o artista escolheu. Em ladrilhos o
    /// oposto seria verdade, e o artista veria o carimbo fugir do lugar cada vez
    /// que mexesse no tamanho.
    ///
    /// ⚠️ **`[0, 0]` é BYTE-IDÊNTICO ao mundo sem deslocamento:** `x - 0.0` é `x`
    /// ao bit em IEEE-754 para todo finito — inclusive `-0.0`.
    offset: [f32; 2],
    /// **De onde as linhas de visão saem**, em espaço de objeto — `[0, 0, 0]`
    /// para os nove procedurais, que não têm olho nenhum.
    eye: [f32; 3],
    /// **A razão do frustum**, ou `0.0` para dizer *"não há perspectiva aqui"*.
    ///
    /// ⚠️ **Zero é o desligado, e não um `Option`, porque o laço é QUENTE:** o
    /// `weight_at` corre por vértice sobre milhões deles, e um `Option` obrigaria
    /// a mesma pergunta a ser feita como `match` no meio da aritmética. Com o
    /// zero, o caminho dos nove reduz **literalmente** — `p − 0.0` é `p` ao bit e
    /// o ramo é o mesmo para a malha inteira, logo previsto pelo processador.
    persp: f32,
}

/// O piso da profundidade, em unidades de objeto.
///
/// ⚠️ **É guarda de REPRESENTAÇÃO, não de gosto:** a câmera pode entrar DENTRO do
/// modelo (é o que um zoom fundo faz), e ali os vértices atrás do olho têm
/// profundidade negativa — a divisão viraria o carimbo do avesso e o zero a faria
/// explodir. Quem está atrás do olho não é visto, então o que o padrão desenha
/// lá não é uma escolha de aparência: é o que impede um `inf` de escorrer para o
/// peso de um vértice.
const MIN_STENCIL_DEPTH: f32 = 1e-4;

impl AlphaFrame {
    /// O frame de um eixo autorado em GRAUS (azimute `0..360`, elevação `0..=90`).
    ///
    /// ⚠️ **A REPRESENTAÇÃO APAGA O CASO ESPECIAL, e é isso que a torna a certa.**
    /// A receita óbvia — ortonormalizar contra um "para cima" de referência — tem
    /// um POLO onde o eixo encosta na referência, e ali o frame SALTA: arrastar a
    /// elevação faria a trama girar noventa graus de repente, com o artista
    /// segurando o slider. Aqui a tangente sai do próprio azimute, e ela é
    /// unitária e perpendicular ao eixo **por identidade**, em qualquer elevação:
    ///
    /// ```text
    /// n = (cos_e·cos_az,  cos_e·sen_az,  sen_e)
    /// t = (   −sen_az,       cos_az,       0  )
    /// n·t = −cos_e·cos_az·sen_az + cos_e·sen_az·cos_az + 0 = 0
    /// ```
    ///
    /// Sem ramo, sem limiar, sem polo — e no zênite (`elev = 90`, onde o azimute
    /// não move mais o eixo) ele passa a ROLAR o padrão, que é exatamente o
    /// controle que faltaria ali.
    ///
    /// ⚠️ **O rotor é o do APP INTEIRO**, e não um `sin`/`cos` escrito aqui: a
    /// sequência dele é específica (uma rotação de 1° acumulada), e a luz, o
    /// Jitter Rotate e o Angle por-slot giram por ela. Um segundo caminho daria
    /// outro número nos últimos bits — a razão que a `ph2d-light` escreveu no
    /// próprio `Cargo.toml` quando pagou esta mesma aresta.
    /// ⚠️ **`pub(crate)` de propósito:** de fora, o único caminho para um frame é
    /// [`crate::Brush::alpha_frame`], então um chamador não consegue construir um
    /// que DISCORDE do pincel que ele passa ao lado. Duas portas para *"em que
    /// direção este padrão corre?"* divergiriam no primeiro sítio novo.
    #[must_use]
    /// ⚠️ **UM construtor, e não dois.** A versão sem deslocamento existiu por
    /// meia hora e ficou **sem chamador** no instante em que o
    /// [`crate::Brush::alpha_frame`] passou a pousar o padrão — e um
    /// `pub(crate)` sem chamador não é código morto silencioso, é uma SEGUNDA
    /// resposta a *"como se constrói um frame?"* esperando alguém a chamar.
    /// Quem quer o frame não-deslocado passa `[0, 0]`, que é byte-idêntico.
    pub(crate) fn placed(az_deg: u16, elev_deg: u16, offset: [f32; 2]) -> Self {
        let az = rotate_by_degrees(az_deg % 360);
        let el = rotate_by_degrees(elev_deg.min(MAX_AXIS_ELEV_DEG));
        let (cos_e, sin_e) = (el[0], el[1]);
        let n = [cos_e * az[0], cos_e * az[1], sin_e];
        let t = [-az[1], az[0], 0.0];
        // `b = n × t` — unitário porque `n` e `t` são unitários e perpendiculares.
        let b = [
            n[1] * t[2] - n[2] * t[1],
            n[2] * t[0] - n[0] * t[2],
            n[0] * t[1] - n[1] * t[0],
        ];
        Self {
            n,
            t,
            b,
            offset,
            // ⚠️ **O olho na origem e a razão em zero REDUZEM literalmente**, e é
            // isso que mantém os nove procedurais byte-idênticos: `p − 0.0` é `p`
            // ao bit para todo finito, e o ramo de perspectiva não é tomado.
            eye: [0.0; 3],
            persp: 0.0,
        }
    }

    /// **O frame de um ESTÊNCIL** — a tela é o plano, e o eixo é a direção de
    /// quem olha.
    ///
    /// ⚠️ **O `roll` é o MESMO `Pattern Angle` do padrão procedural, e ele muda
    /// de significado com o modo — o que é honesto, porque a pergunta muda:** num
    /// campo 3-D ele diz *para que lado os estratos empilham*; num carimbo preso
    /// à tela ele diz *quanto o carimbo está torto na tela*. As duas são a mesma
    /// palavra do artista (*gire isto*), e é por isso que continuam UMA row em
    /// vez de duas que ele teria de escolher entre.
    ///
    /// ⚠️ **A ELEVAÇÃO não entra, e não é omissão:** o eixo de um estêncil é a
    /// vista, por definição. Um segundo controle que o inclinasse tiraria o
    /// carimbo da frente — que é exatamente o que este modo existe para impedir —,
    /// e por isso a row dele **some** quando há uma imagem armada.
    pub(crate) fn stencil(s: &AlphaStencil, roll_deg: u16, offset: [f32; 2]) -> Self {
        let r = rotate_by_degrees(roll_deg % 360);
        // O giro acontece DENTRO do plano da tela: a base nova é a antiga
        // rodada, e o eixo (a normal) não é tocado por construção.
        let t = [
            r[0] * s.right[0] + r[1] * s.up[0],
            r[0] * s.right[1] + r[1] * s.up[1],
            r[0] * s.right[2] + r[1] * s.up[2],
        ];
        let b = [
            r[0] * s.up[0] - r[1] * s.right[0],
            r[0] * s.up[1] - r[1] * s.right[1],
            r[0] * s.up[2] - r[1] * s.right[2],
        ];
        // `n = t × b` — o eixo sai da PRÓPRIA base, e não de um terceiro vetor
        // guardado ao lado: assim ele não pode discordar dela.
        let n = [
            t[1] * b[2] - t[2] * b[1],
            t[2] * b[0] - t[0] * b[2],
            t[0] * b[1] - t[1] * b[0],
        ];
        // ⚠️ **O deslocamento fica em FRAÇÃO DA VISTA, sem conversão nenhuma** —
        // e a ausência é a wave: a [`Self::project`] devolve fração de tela para
        // um estêncil, então o deslocamento já está na régua do resultado. A
        // versão anterior o multiplicava pela régua de UM ponto, e por isso o
        // carimbo pousava em lugares diferentes conforme quem o montava.
        Self {
            n,
            t,
            b,
            offset,
            eye: s.eye,
            persp: s.height_per_depth,
        }
    }

    /// O eixo autorado — o que o overlay desenharia e o que o gate mede.
    #[must_use]
    pub fn axis(&self) -> [f32; 3] {
        self.n
    }

    /// O ponto `p` nas coordenadas do frame: `(ao longo de t, ao longo de b, ao
    /// longo do EIXO)`.
    ///
    /// ⚠️ **Para um ESTÊNCIL as duas primeiras saem em FRAÇÃO DA TELA**, não em
    /// unidades de objeto — é a divisão de perspectiva, e é ela que faz do
    /// carimbo uma folha presa ao viewport em vez de um plano colado ao barro na
    /// profundidade em que alguém escolheu medir. Dois pontos que caem no MESMO
    /// pixel recebem a MESMA coordenada de carimbo, por construção, seja qual for
    /// a profundidade de cada um.
    ///
    /// ⚠️ **E os nove procedurais atravessam isto ao BIT:** com `eye = [0,0,0]` a
    /// subtração é a identidade em IEEE-754 e com `persp = 0` o ramo não é
    /// tomado, então a expressão que sobra é literalmente a de antes.
    pub(super) fn project(&self, p: [f32; 3]) -> [f32; 3] {
        let d = [p[0] - self.eye[0], p[1] - self.eye[1], p[2] - self.eye[2]];
        let x = d[0] * self.t[0] + d[1] * self.t[1] + d[2] * self.t[2];
        let y = d[0] * self.b[0] + d[1] * self.b[1] + d[2] * self.b[2];
        let z = d[0] * self.n[0] + d[1] * self.n[1] + d[2] * self.n[2];
        // ⚠️ **O deslocamento sai das DUAS coordenadas do plano e NUNCA do
        // eixo.** Ele descreve onde o carimbo pousa NA superfície; subtraí-lo do
        // `q[2]` moveria o padrão ao longo da direção de projeção, que para uma
        // imagem não significa nada e para o Strata seria um terceiro controle
        // que ninguém pediu.
        if self.persp > 0.0 {
            // ⚠️ **O eixo aponta para QUEM OLHA** (ele é `t × b`, e a base da
            // tela é destra), então o que está à frente da câmera tem `z`
            // NEGATIVO — a profundidade é o negativo dele. Ler o sinal errado
            // aqui põe o carimbo atrás do observador, que é a única forma de
            // este ramo falhar em silêncio.
            let depth = (-z).max(MIN_STENCIL_DEPTH);
            let k = 1.0 / (depth * self.persp);
            [x * k - self.offset[0], y * k - self.offset[1], z]
        } else {
            [x - self.offset[0], y - self.offset[1], z]
        }
    }
}

/// O zênite. Acima dele o eixo desceria do outro lado, e o azimute já cobre os
/// 360° do outro hemisfério — dois caminhos para a mesma direção.
pub const MAX_AXIS_ELEV_DEG: u16 = 90;
