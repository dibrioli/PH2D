#![forbid(unsafe_code)]
//! **O rig de lâmpadas — a casa ÚNICA.**
//!
//! # Por que esta crate deixou de estar vazia
//!
//! O Painter já ilumina relevo: quatro lâmpadas, material por-pixel, LUT especular, com paridade
//! CPU/GPU gateada. O módulo 3D precisa da MESMA luz — e a exigência que decide tudo está escrita em
//! `docs/3D/05.2`:
//!
//! > *"O mesmo rig de lâmpadas em editor e runtime. Um segundo modelo de luz no jogo faria o artista
//! > ajustar a arte contra uma iluminação que o jogador nunca veria — e é a classe de erro mais cara
//! > que existe, porque ninguém a percebe até o jogo estar pronto."*
//!
//! Duas cópias do rig não divergiriam no dia em que fossem escritas; divergiriam no dia em que alguém
//! consertasse uma. Então o rig tem um dono, e é esta crate.
//!
//! ⚠️ **Ela é a única peça NÃO-removível do módulo 3D**, e isso é deliberado (`docs/3D/02.3`): depois
//! que o Painter passa por aqui, arrancá-la quebra o Painter. Em troca, o Painter continua sendo o dono
//! de tudo o que é *dele* — o material, a LUT especular, o fold do relevo, a óptica por-pixel. **O que
//! mora aqui é só o que os dois consumidores têm de responder igual: quantas lâmpadas há, onde cada uma
//! está, e com que força.**
//!
//! # A fronteira, numa linha
//!
//! ```text
//! aqui:  o rig AUTORADO (azimute, elevação, intensidade, cor)  →  lâmpadas RESOLVIDAS (dir, half, tint)
//! lá:    o que uma lâmpada resolvida FAZ com um pixel  (material, LUT, wax, metal, albedo)
//! ```
//!
//! O corte é onde está o risco. A resolução é geometria pura e é **exatamente** onde um segundo
//! implementador erraria em silêncio — veja [`resolve`].
//!
//! # Por que dependemos da crate do pincel
//!
//! A conversão azimute→vetor passa pelo rotor de 1° do app
//! ([`ph2d_painter_brush::texture::rotate_by_degrees`]), que **acumula** uma rotação de um grau `d`
//! vezes. Isso não é `(cos θ, sin θ)`: é uma sequência numérica específica, com o erro de arredondamento
//! que ela acumula. Um `sin`/`cos` escrito aqui daria outro número nos últimos bits — e o app inteiro
//! (Jitter Rotate, Angle por-slot, a luz do impasto) gira por aquele.
//!
//! `ph2d-painter-brush` **não tem dependência nenhuma** (é folha), então folha→folha não inverte
//! arquitetura nenhuma. ⚠️ O rotor está lá por acidente de nascimento, não por pertencer ao pincel —
//! mas movê-lo é churn na crate mais quente do repo e **é decisão do dono dela**, não desta wave.

pub use ph2d_painter_brush::texture::rotate_by_degrees;

/// Quantas lâmpadas o rig comporta.
///
/// O número é do Krita (Phong Bumpmap), e é um **teto, não um alvo**: a lâmpada 0 é a principal, e o
/// artista que nunca abre o card tem exatamente uma. As outras nascem apagadas, que é o que mantém uma
/// tela que ninguém iluminou byte-idêntica ao build de uma lâmpada só. // CLAMP-OK
pub const MAX_LIGHTS: usize = 4;

/// Elevação mínima, em graus.
///
/// A 0 a lâmpada rasa a superfície, a resposta plana vai a zero, e um modelo RELATIVO — que divide
/// pela resposta plana — dividiria por ~0. // CLAMP-OK
pub const MIN_ELEV_DEG: u16 = 5;

/// **Piso AMBIENTE** da difusa: o que uma face totalmente virada PARA LONGE da luz ainda devolve.
///
/// Tinta na sombra é mais escura; não é preta. Sem este piso o sombreamento vai a `0` e multiplica o
/// pixel a zero — que foi exatamente o que pôs as manchas pretas nas pontas dos traços do primeiro
/// smoke do impasto (a ponta de um traço é onde a altura cai de cheia a nada em um pixel, logo é a
/// encosta mais íngreme da tela, logo é o primeiro lugar onde um piso zero morde).
///
/// ⚠️ **Ele é LEI do modelo relativo, não material** — e por isso mora aqui e não no Painter: os dois
/// consumidores (tinta e forma) têm de dobrar a razão do MESMO jeito, senão a mesma lâmpada deixaria a
/// escultura mais escura na sombra que a pintura ao lado dela, e ninguém saberia dizer por quê.
/// Dobrado de modo que uma superfície PLANA ainda devolva exatamente `1.0` — o contrato de
/// byte-identidade sobrevive. // CLAMP-OK
pub const AMBIENT: f32 = 0.35;

/// Uma lâmpada, como o artista a autora.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    /// Acesa. A 0 é a principal e começa ligada; 1..3 começam **apagadas**, então uma tela nova é
    /// byte-idêntica ao build de uma lâmpada só.
    pub on: bool,
    /// Azimute em graus inteiros (`0..360`) — de que lado a lâmpada está.
    pub angle_deg: u16,
    /// Elevação em graus inteiros acima do plano (`MIN_ELEV_DEG..=90`).
    pub elev_deg: u16,
    /// Força (`0..2`). Uma lâmpada de preenchimento a 0,3 contra uma principal a 1 é o rig de todo dia.
    pub intensity: f32,
    /// A cor da lâmpada (RGB linear, `0..1`).
    pub color: [f32; 3],
}

impl Light {
    /// A PRINCIPAL — o default afinado pelo Enio (2026-07-12): superior-esquerda a 30°, a leitura que
    /// todo olho resolve como *saliente* em vez de *gravado*.
    pub const KEY: Self = Self {
        on: true,
        angle_deg: 230,
        elev_deg: 30,
        intensity: 1.0,
        color: [1.0, 1.0, 1.0],
    };

    /// Uma lâmpada que o artista ainda não acendeu. O ÂNGULO dela não é o da principal — quem marca
    /// "Enable" na lâmpada 2 e não vê nada mudar chama o interruptor de quebrado, com razão, e duas
    /// lâmpadas no mesmo lugar são uma lâmpada. Preenchimento: oposta à principal, baixa, fraca e fria.
    pub const FILL: Self = Self {
        on: false,
        angle_deg: 50, // oposta à principal (230 − 180)
        elev_deg: 25,
        intensity: 0.4,
        color: [0.85, 0.9, 1.0], // fria, contra uma principal quente
    };
}

impl Default for Light {
    fn default() -> Self {
        Self::KEY
    }
}

/// O rig inteiro, como o documento o guarda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightRig {
    pub lights: [Light; MAX_LIGHTS],
    /// Qual lâmpada o card está editando (`0..MAX_LIGHTS`). É estado de EDIÇÃO, não de aparência — não
    /// muda um pixel, e mora aqui para o painel continuar sendo função pura do snapshot.
    pub selected: u8,
}

impl Default for LightRig {
    fn default() -> Self {
        Self {
            lights: [Light::KEY, Light::FILL, Light::FILL, Light::FILL],
            selected: 0,
        }
    }
}

impl LightRig {
    /// A lâmpada que o card edita. Nunca entra em pânico com índice ruim — ele *clampa*, porque um
    /// snapshot que chega com seleção velha não pode derrubar o painel.
    #[must_use]
    pub fn current(&self) -> &Light {
        &self.lights[(self.selected as usize).min(MAX_LIGHTS - 1)]
    }

    /// [`Self::current`] mutável.
    pub fn current_mut(&mut self) -> &mut Light {
        &mut self.lights[(self.selected as usize).min(MAX_LIGHTS - 1)]
    }

    /// Se alguma lâmpada está acesa. O passe desiste em `false` — quem apaga todas recebe a superfície
    /// sem luz de volta, ao byte, em vez de uma divisão por zero.
    #[must_use]
    pub fn any_on(&self) -> bool {
        self.lights.iter().any(|l| l.on && l.intensity > 0.0)
    }
}

/// Uma lâmpada RESOLVIDA — a forma que todo consumidor sombreia.
///
/// É esta struct que atravessa a fronteira para a GPU (tinta) e para o passe da malha (forma). Nenhum
/// dos dois reconstrói `dir` a partir de graus: o rotor roda **uma vez**, aqui.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lamp {
    /// Direção unitária da luz; `z > 0` aponta para fora da superfície.
    pub dir: [f32; 3],
    /// Vetor médio entre a luz e a vista ortográfica `(0, 0, 1)`.
    pub half: [f32; 3],
    /// Cor PESADA: `intensity × color`. Peso e matiz nunca mais aparecem separados — toda soma a
    /// jusante é sobre este vetor, que é por que um rig de lâmpadas coloridas custa exatamente o que um
    /// rig de brancas custa.
    pub tint: [f32; 3],
}

/// O rig RESOLVIDO: as lâmpadas acesas, na ordem, mais o fato que decide o caminho rápido.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedRig {
    lamps: [Lamp; MAX_LIGHTS],
    n: usize,
    achromatic: bool,
}

impl ResolvedRig {
    /// As lâmpadas acesas. Nunca vazio — [`resolve`] devolve `None` nesse caso.
    #[must_use]
    pub fn lamps(&self) -> &[Lamp] {
        &self.lamps[..self.n]
    }

    /// Toda lâmpada é CINZA (`r == g == b`).
    ///
    /// Então a razão de todo canal é o mesmo número, e um consumidor pode computá-la **uma** vez em vez
    /// de três. Não é micro-otimização disfarçada: este é o rig DEFAULT (uma principal branca) e é o rig
    /// de todo mundo até alguém colorir uma lâmpada de propósito.
    #[must_use]
    pub fn achromatic(&self) -> bool {
        self.achromatic
    }
}

/// Resolve o rig autorado nas lâmpadas que os consumidores sombreiam.
///
/// `None` quando nenhuma lâmpada está acesa — aí o chamador deixa a superfície em paz em vez de dividir
/// por uma resposta plana igual a zero.
///
/// ⚠️ **O filtro `intensity > 0.0` é sobre o rig VAZIO, e é load-bearing:** com todas as lâmpadas em
/// potência zero, uma lista não-filtrada é não-vazia, isto devolveria `Some`, a resposta plana sairia
/// toda-zero, o piso do consumidor a viraria 1, a difusa ficaria 0 e a razão seria 0 — o que empurra
/// todo pixel iluminado para o piso ambiente. *Baixar as luzes até o fim ESCURECERIA a pintura a 35%
/// em vez de deixá-la sem luz.* É o filtro que faz `n == 0` ser verdade e o passe desistir.
#[must_use]
pub fn resolve(rig: &LightRig) -> Option<ResolvedRig> {
    const DARK: Lamp = Lamp {
        dir: [0.0; 3],
        half: [0.0; 3],
        tint: [0.0; 3],
    };
    let mut lamps = [DARK; MAX_LIGHTS];
    let mut n = 0usize;
    for l in rig.lights.iter().filter(|l| l.on && l.intensity > 0.0) {
        // Sem transcendental (HR-5): o rotor de 1° compartilhado, o mesmo por onde o Jitter Rotate do
        // pincel e o Angle por-slot giram.
        let elev = l.elev_deg.clamp(MIN_ELEV_DEG, 90);
        let az = rotate_by_degrees(l.angle_deg % 360);
        let el = rotate_by_degrees(elev);
        let (cos_e, sin_e) = (el[0], el[1]);
        let dir = [cos_e * az[0], cos_e * az[1], sin_e];
        let half = {
            let h = [dir[0], dir[1], dir[2] + 1.0]; // vista = (0, 0, 1), ortográfica
            let len = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt().max(1e-6);
            [h[0] / len, h[1] / len, h[2] / len]
        };
        let w = l.intensity.clamp(0.0, 2.0);
        let tint = [
            w * l.color[0].clamp(0.0, 1.0),
            w * l.color[1].clamp(0.0, 1.0),
            w * l.color[2].clamp(0.0, 1.0),
        ];
        lamps[n] = Lamp { dir, half, tint };
        n += 1;
    }
    if n == 0 {
        return None;
    }
    let achromatic = lamps[..n]
        .iter()
        .all(|l| l.tint[0] == l.tint[1] && l.tint[1] == l.tint[2]);
    Some(ResolvedRig {
        lamps,
        n,
        achromatic,
    })
}

#[cfg(test)]
#[path = "rig_tests.rs"]
mod rig_tests;
