//! **O que um degrau NOVO é, tipo por tipo** — os defaults do "Add".
//!
//! Irmão de [`super::vec_filter`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo
//! diz *o que um degrau É* (os campos, os tetos, as portas que respondem sobre ele), este diz *com
//! que números ele NASCE*.
//!
//! ⚠️ **A lei é uma:** os defaults são **VISÍVEIS**, nunca neutros — armar um degrau no neutro seria
//! um clique que não muda um pixel, e o artista concluiria que o "Add" não funcionou. A exceção é o
//! que o degrau não controla: um valor que o kernel ignora fica no `BLANK`.

use super::vec_filter::{BLANK, FxOp};

impl FxOp {
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
            Self::COLOR_ADJUST => Self {
                kind,
                // Um QUARTO DE VOLTA de matiz. "Add" tem de mudar a tela, e dos três knobs a
                // matiz é o que NOMEIA o efeito; um quarto de volta é longe o bastante para se
                // ler como *outra cor* sem ser a complementar (meia volta), que lê como inversão.
                //
                // ⚠️ Numa arte CINZENTA este default não desenha nada, e isso é honesto: girar a
                // matiz de um pixel sem croma É nada. Quem quiser mexer em cinza tem o Brilho, e
                // é por isso que os três são oferecidos juntos.
                hue: 0.25,
                ..BLANK
            },
            Self::DUOTONE => Self {
                kind,
                // O par clássico de duotone — sombra FRIA, luz QUENTE. "Add" tem de mudar a tela, e
                // duas pontas neutras (preto→branco) seriam a identidade em cinza: o degrau
                // desenharia a própria entrada dessaturada e leria como quebrado.
                color: [0.10, 0.12, 0.35, 1.0],
                color_b: [1.0, 0.86, 0.62, 1.0],
                ..BLANK
            },
            Self::LUMA_TO_ALPHA => Self { kind, ..BLANK },
            // A rampa nasce preto→branco — o mesmo default do Gradient Map do Painter (e do
            // Photoshop), porque é o que se lê como *"a minha arte mapeada na minha rampa"*.
            //
            // ⚠️ **Ela NÃO é neutra, e a nota anterior afirmava que era.** Eu escrevi *"a
            // IDENTIDADE em luma, um degrau novo não pode mudar o desenho"*; medido pelo gate
            // `no_stops_is_the_painters_empty_ramp_which_is_not_the_two_stop_default`, um cinza de
            // display **129 entra e 204 sai** — o `t` sai do `L` do OKLab e a mistura acontece em
            // luz LINEAR, então não há como duas pontas lineares reconstruírem a curva sRGB. Um
            // Gradient Map é um RECOLORIDOR, como o Duotone e o Color Overlay: adicioná-lo muda o
            // desenho **por desenho**, e o que o default tem de ser é PREVISÍVEL.
            Self::GRADIENT_MAP => {
                let mut stops = [[0.0, 0.0, 0.0, 1.0]; Self::MAX_GRADIENT_STOPS];
                stops[1] = [1.0, 1.0, 1.0, 1.0];
                let mut stop_pos = [0.0; Self::MAX_GRADIENT_STOPS];
                stop_pos[1] = 1.0;
                Self {
                    kind,
                    stops,
                    stop_pos,
                    stop_count: 2,
                    // ⚠️ **Linear EXPLÍCITO, e é a QUARTA instância da mesma doença.** O `BLANK`
                    // compartilhado nasce com `mode: MODE_CONTOUR`, que vale `1` — o default bom
                    // da família do falloff, onde esse número significa *Contour*. Aqui `1` é
                    // *Smooth*, então herdar o blank fazia todo Gradient Map nascer com easing
                    // por-trecho sem ninguém o pedir: o mesmo `1` querendo dizer coisas diferentes
                    // em tipos diferentes, que esta wave já curou no PLANO de passes e no gate do
                    // trilho.
                    //
                    // ⚠️ **E o preço não era só o desenho:** em Smooth o `+` **não pode** ser
                    // neutro — o easing é por SEGMENTO, então dividir um segmento reforma a curva
                    // (medido: 25 níveis de byte). Em Linear ele é neutro ao nível de byte, que é o
                    // que permite ao artista ganhar um ponto de controle sem ganhar uma edição.
                    mode: 0,
                    ..BLANK
                }
            }
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
