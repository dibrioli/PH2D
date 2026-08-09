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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaStencil {
    /// A direita da TELA, em espaço local, unitária.
    pub right: [f32; 3],
    /// O cima da TELA, em espaço local, unitário.
    pub up: [f32; 3],
    /// **Quantas unidades de OBJETO a ALTURA da tela abrange**, na profundidade
    /// do modelo.
    ///
    /// ⚠️ **É esta a régua que torna o estêncil imune ao ZOOM**, e ela é UM
    /// número de propósito: aproximar reduz quantas unidades de objeto cabem na
    /// tela, então um ladrilho medido em fração-de-tela encolhe em unidades de
    /// objeto na mesma proporção — e fica do mesmo tamanho para quem olha. Duas
    /// grandezas separadas (pixels e uma altura de viewport) seriam dois números
    /// que precisam concordar.
    pub view_units: f32,
}

impl AlphaStencil {
    /// A vista CANÔNICA — a tela olhando por `−Z`, sem pose nenhuma.
    ///
    /// ⚠️ **Ela existe para quem NÃO tem câmera e ainda assim tem de desenhar o
    /// estêncil de frente**: o retrato do painel. Um swatch que montasse a
    /// própria base seria a segunda resposta a *"como um estêncil é lido?"*, e a
    /// que mente é a que o artista está olhando.
    pub const CANONICAL: Self = Self {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        view_units: 1.0,
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
}

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
        Self { n, t, b, offset }
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
        // ⚠️ **O deslocamento é FRAÇÃO DA VISTA e vira unidades de objeto aqui**,
        // porque é aqui que a régua da vista existe. Convertê-lo no chamador
        // seria a conversão feita em dois sítios — e o segundo nasceria sem ela.
        Self {
            n,
            t,
            b,
            offset: [offset[0] * s.view_units, offset[1] * s.view_units],
        }
    }

    /// O eixo autorado — o que o overlay desenharia e o que o gate mede.
    #[must_use]
    pub fn axis(&self) -> [f32; 3] {
        self.n
    }

    /// O ponto `p` nas coordenadas do frame: `(ao longo de t, ao longo de b, ao
    /// longo do EIXO)`.
    pub(super) fn project(&self, p: [f32; 3]) -> [f32; 3] {
        // ⚠️ **O deslocamento sai das DUAS coordenadas do plano e NUNCA do
        // eixo.** Ele descreve onde o carimbo pousa NA superfície; subtraí-lo do
        // `q[2]` moveria o padrão ao longo da direção de projeção, que para uma
        // imagem não significa nada e para o Strata seria um terceiro controle
        // que ninguém pediu.
        [
            (p[0] * self.t[0] + p[1] * self.t[1] + p[2] * self.t[2]) - self.offset[0],
            (p[0] * self.b[0] + p[1] * self.b[1] + p[2] * self.b[2]) - self.offset[1],
            p[0] * self.n[0] + p[1] * self.n[1] + p[2] * self.n[2],
        ]
    }
}

/// O zênite. Acima dele o eixo desceria do outro lado, e o azimute já cobre os
/// 360° do outro hemisfério — dois caminhos para a mesma direção.
pub const MAX_AXIS_ELEV_DEG: u16 = 90;
