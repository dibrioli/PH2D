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

/// **O que um tipo de degrau É** — a tabela que decide os controles, a margem e o número de
/// passes. UMA tabela, e todos os consumidores a leem: o painel (que rows oferecer), o passe da
/// GPU (quanto espalhar, quantos dispatches) e o próprio `FxOp` (`tints`/`displaces`).
///
/// ⚠️ **É por isto que ela existe.** A W2 tinha três tipos e o painel decidia por `kind == 2` /
/// `kind != 0` espalhado pelo `paint`. Com sete tipos essa aritmética apodrece na primeira adição
/// — e o modo de falha é um knob morto (ou um knob que falta) que nenhum gate vê.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FxKindSpec {
    /// O nome que o painel mostra (e o rótulo do card).
    pub name: &'static str,
    /// O rótulo do raio, ou `None` se o tipo não tem raio nenhum (o Color Overlay é pontual).
    /// `Some` também significa *este op custa DOIS dispatches* (a Gaussiana separável).
    pub radius_label: Option<&'static str>,
    /// Os rótulos do par de offset, ou `None` se o tipo não desloca nada. ⚠️ **É um par de nomes,
    /// não um booleano:** o mesmo vetor é *para onde a sombra cai* na Drop Shadow e *de onde a luz
    /// vem* no Bevel, e um rótulo errado é um knob que mente sobre o que faz.
    pub offset_labels: Option<(&'static str, &'static str)>,
    /// O rótulo da cor, ou `None` se o tipo não tinge (o Blur reusa os pixels que recebeu).
    pub color_label: Option<&'static str>,
    /// **Espalha para FORA da silhueta que recebeu?** É esta a pergunta da MARGEM da textura: um
    /// halo externo cresce a imagem, um efeito de DENTRO não cresce nada.
    pub grows: bool,
    /// O halo nasce do alfa **INVERTIDO** e é mascarado pela forma — ou seja, mora DENTRO dela.
    pub inner: bool,
    /// Os MODOS que este tipo oferece, na ordem dos códigos, ou vazio se ele não tem escolha a
    /// fazer. O painel pinta um chip por modo e o **índice no slice É o código** — como a `SPECS`,
    /// a lista é a tabela.
    pub modes: &'static [&'static str],
}

/// Os modos de QUEDA. Duas leis diferentes sobre *o que é "perto da borda"*, e a diferença é
/// visível em qualquer forma com reentrância.
///
/// ⚠️ Chamavam-se `INNER_MODES` porque só os degraus de dentro os ofereciam — e o nome era um
/// acidente de quem chegou primeiro, não uma propriedade da escolha: a pergunta *"perto da borda é
/// pouco fora por perto, ou é pouca DISTÂNCIA até ela?"* é a mesma para um halo externo.
pub const FALLOFF_MODES: [&str; 2] = ["Proximity", "Contour"];

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
    /// Quantos tipos existem — o painel oferece um "Add" por tipo, a partir daqui.
    pub const KINDS: usize = 9;

    /// Modo de queda: **a PROXIMIDADE do outro lado** (a silhueta borrada — o modelo do
    /// Photoshop). Lê como PROFUNDIDADE: uma parte fina escurece INTEIRA, porque tudo nela está
    /// perto de fora, e uma reentrância quase não recebe efeito, porque o outro lado ali subtende
    /// um ângulo pequeno.
    pub const MODE_PROXIMITY: u8 = 0;
    /// Modo de queda: **a DISTÂNCIA à borda** — uma banda de largura constante ao longo de TODO o
    /// contorno, reentrâncias incluídas. É o que "sombra interna" desenha em quem olha a forma, e é
    /// o default dos degraus de DENTRO.
    pub const MODE_CONTOUR: u8 = 1;

    /// **A tabela dos tipos.** Indexada pelo `kind`; a ordem É a dos códigos acima (há gate).
    pub const SPECS: [FxKindSpec; Self::KINDS] = [
        FxKindSpec {
            name: "Blur",
            radius_label: Some("Radius"),
            offset_labels: None,
            color_label: None,
            grows: true,
            inner: false,
            modes: &[],
        },
        FxKindSpec {
            name: "Glow",
            radius_label: Some("Radius"),
            offset_labels: None,
            color_label: Some("Color"),
            grows: true,
            inner: false,
            // O halo externo faz a MESMA escolha que os de dentro: a silhueta borrada (Proximity)
            // ou uma banda de largura constante ao longo do contorno (Contour). Numa forma com
            // reentrância elas desenham coisas visivelmente diferentes — a ponta de uma estrela
            // brilha nas duas, o vão entre pontas só na segunda.
            modes: &FALLOFF_MODES,
        },
        FxKindSpec {
            name: "Drop Shadow",
            radius_label: Some("Radius"),
            offset_labels: Some(("Offset X", "Offset Y")),
            color_label: Some("Color"),
            grows: true,
            inner: false,
            modes: &[],
        },
        FxKindSpec {
            name: "Inner Shadow",
            radius_label: Some("Radius"),
            offset_labels: Some(("Offset X", "Offset Y")),
            color_label: Some("Color"),
            grows: false,
            inner: true,
            modes: &FALLOFF_MODES,
        },
        FxKindSpec {
            name: "Inner Glow",
            radius_label: Some("Radius"),
            offset_labels: None,
            color_label: Some("Color"),
            grows: false,
            inner: true,
            modes: &FALLOFF_MODES,
        },
        FxKindSpec {
            name: "Outline",
            // ⚠️ "Width", não "Radius": o contorno se estende EXATAMENTE este tanto a partir de uma
            // aresta reta (o corte duro mora no nível `Φ(−1)` da Gaussiana — há gate que mede).
            radius_label: Some("Width"),
            offset_labels: None,
            color_label: Some("Color"),
            grows: true,
            inner: false,
            modes: &[],
        },
        FxKindSpec {
            name: "Feather",
            radius_label: Some("Feather"),
            offset_labels: None,
            color_label: None,
            grows: true,
            inner: false,
            modes: &[],
        },
        FxKindSpec {
            name: "Bevel",
            radius_label: Some("Depth"),
            offset_labels: Some(("Light X", "Light Y")),
            color_label: Some("Shadow"),
            grows: false,
            inner: true,
            modes: &[],
        },
        FxKindSpec {
            name: "Color Overlay",
            radius_label: None,
            offset_labels: None,
            color_label: Some("Color"),
            grows: false,
            inner: false,
            modes: &[],
        },
    ];

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
mod tests {
    use super::*;

    fn stack(kinds: &[u8]) -> VecFilter {
        VecFilter {
            ops: kinds.iter().map(|k| FxOp::new(*k)).collect(),
        }
    }

    /// **Reordenar troca DOIS vizinhos, e as pontas são no-ops** — subir na primeira linha e descer
    /// na última não fazem nada, e o painel nem desenha essas setas. Aqui prova-se que, mesmo se
    /// alguém as despachasse, a pilha não se deforma (um `swap` fora de faixa entraria em pânico).
    #[test]
    fn reordering_swaps_neighbours_and_the_ends_are_no_ops() {
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW]);
        assert!(f.move_down(0));
        assert_eq!(
            f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
            vec![FxOp::GLOW, FxOp::BLUR, FxOp::DROP_SHADOW]
        );
        assert!(f.move_up(2));
        assert_eq!(
            f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
            vec![FxOp::GLOW, FxOp::DROP_SHADOW, FxOp::BLUR]
        );
        let before = f.clone();
        assert!(!f.move_up(0), "subir na primeira linha não faz nada");
        assert!(!f.move_down(2), "descer na última não faz nada");
        assert!(!f.move_down(9), "nem uma linha que não existe");
        assert_eq!(f, before, "e nenhuma delas pode deformar a pilha");
    }

    /// **Uma pilha só desenha se algum degrau estiver LIGADO** — a porta única que o produtor e a
    /// remoção do componente perguntam. Vazia e toda-desligada são o mesmo fato para quem desenha.
    #[test]
    fn a_stack_is_active_only_while_some_op_is_enabled() {
        assert!(!VecFilter::default().is_active(), "vazia não desenha nada");
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW]);
        assert!(f.is_active());
        f.ops[0].enabled = false;
        assert!(f.is_active(), "um degrau ligado basta");
        f.ops[1].enabled = false;
        assert!(!f.is_active(), "toda desligada é o mesmo que vazia");
    }

    /// **Os defaults de cada tipo são VISÍVEIS** — armar no neutro seria um clique que não muda um
    /// pixel, e o artista concluiria que o botão está quebrado.
    ///
    /// ⚠️ O laço varre **todos** os tipos e pergunta o que exigir à [`FxOp::SPECS`]: um tipo novo
    /// entra neste gate sem que ninguém o acrescente aqui, que é o oposto da lista escrita à mão
    /// que a W2 tinha (e que teria ficado verde sobre os quatro tipos desta wave).
    #[test]
    fn a_new_op_is_born_visible() {
        for kind in 0..FxOp::KINDS as u8 {
            let o = FxOp::new(kind);
            let s = FxOp::spec(kind);
            assert_eq!(
                o.kind, kind,
                "o Add do tipo {kind} tem de criar o tipo {kind}"
            );
            assert!(o.enabled, "nasce ligado ({})", s.name);
            assert!(
                o.opacity > 0.0,
                "opacidade zero seria invisível ({})",
                s.name
            );
            if s.radius_label.is_some() {
                assert!(o.radius > 0.0, "raio zero não desenharia nada ({})", s.name);
            }
            if s.offset_labels.is_some() {
                assert!(
                    o.offset != [0.0, 0.0],
                    "uma sombra sem deslocamento é um glow — o default tem de a mostrar ({})",
                    s.name
                );
            } else {
                assert!(
                    o.offset == [0.0, 0.0],
                    "só quem tem offset nasce deslocado ({})",
                    s.name
                );
            }
        }
    }

    /// **Quem tem modos nasce no default declarado; quem não tem nasce em ZERO.** Um número
    /// guardado que não seleciona nada é a semente de "este campo quer dizer o quê aqui?".
    #[test]
    fn only_the_kinds_with_modes_carry_one() {
        for kind in 0..FxOp::KINDS as u8 {
            let o = FxOp::new(kind);
            let s = FxOp::spec(kind);
            if s.modes.is_empty() {
                assert_eq!(o.mode, 0, "{} não tem modos e nasceu em {}", s.name, o.mode);
            } else {
                assert!(
                    (o.mode as usize) < s.modes.len(),
                    "{} nasceu num modo que a tabela não oferece ({})",
                    s.name,
                    o.mode
                );
            }
        }
        // Os dois de dentro nascem em CONTOUR — a banda que segue o contorno é o que "sombra
        // interna" desenha para quem olha a forma; a proximidade fica como a outra opção.
        for kind in [FxOp::INNER_SHADOW, FxOp::INNER_GLOW] {
            assert_eq!(FxOp::new(kind).mode, FxOp::MODE_CONTOUR);
            assert_eq!(FxOp::spec(kind).modes.len(), 2);
        }
    }

    /// **A tabela é indexada pelo CÓDIGO, e cada tipo tem nome próprio.** Um `SPECS` fora de ordem
    /// daria ao painel o nome de um tipo e os controles de outro — e nada pareceria quebrado até
    /// alguém procurar o Offset da Drop Shadow no card do Glow.
    #[test]
    fn the_spec_table_is_indexed_by_the_kind_code() {
        assert_eq!(FxOp::SPECS.len(), FxOp::KINDS);
        for (name, kind) in [
            ("Blur", FxOp::BLUR),
            ("Glow", FxOp::GLOW),
            ("Drop Shadow", FxOp::DROP_SHADOW),
            ("Inner Shadow", FxOp::INNER_SHADOW),
            ("Inner Glow", FxOp::INNER_GLOW),
            ("Outline", FxOp::OUTLINE),
            ("Feather", FxOp::FEATHER),
            ("Bevel", FxOp::BEVEL),
            ("Color Overlay", FxOp::COLOR_OVERLAY),
        ] {
            assert_eq!(FxOp::kind_name(kind), name, "o código {kind} é o {name}");
        }
        let mut names: Vec<&str> = FxOp::SPECS.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FxOp::KINDS, "dois tipos com o mesmo nome");
        // Um código de uma versão futura cai no tipo mais inerte, nunca em pânico.
        assert_eq!(FxOp::kind_name(200), "Blur");
    }

    /// **QUEM oferece a escolha de queda, e o que cada um arma ao nascer.**
    ///
    /// ⚠️ Este gate existe porque o seam do painel é dirigido pela TABELA: apagar os modos do Glow
    /// deixa-o verde (ele passa a esperar zero chips, coerentemente), então ele não pode testemunhar
    /// que a capacidade EXISTE. O fato mora aqui, e é aqui que se pina.
    #[test]
    fn the_falloff_choice_is_offered_where_it_means_something() {
        for kind in [FxOp::GLOW, FxOp::INNER_SHADOW, FxOp::INNER_GLOW] {
            assert_eq!(
                FxOp::spec(kind).modes,
                &FALLOFF_MODES,
                "{} tinha de oferecer Proximity/Contour",
                FxOp::kind_name(kind)
            );
        }
        // ⚠️ **O Glow nasce em Proximity, e os de DENTRO em Contour.** O Glow SEMPRE foi a silhueta
        // borrada — ganhar uma opção não pode repintar o que "Add Glow" quer dizer para quem já o
        // usa —, e um Glow salvo antes desta wave carrega `mode = 0`, que é exatamente este.
        assert_eq!(FxOp::new(FxOp::GLOW).mode, FxOp::MODE_PROXIMITY);
        assert_eq!(FxOp::new(FxOp::INNER_SHADOW).mode, FxOp::MODE_CONTOUR);
        assert_eq!(FxOp::new(FxOp::INNER_GLOW).mode, FxOp::MODE_CONTOUR);
        // E quem não oferece escolha guarda ZERO — um número que não seleciona nada é a semente do
        // "este campo quer dizer o quê aqui?".
        for kind in 0..FxOp::KINDS as u8 {
            if FxOp::spec(kind).modes.is_empty() {
                assert_eq!(FxOp::new(kind).mode, 0, "{}", FxOp::kind_name(kind));
            }
        }
    }

    /// **`tints`/`displaces` são VISTAS da tabela, não uma segunda opinião.** Foi a divergência
    /// entre elas e o `paint` que a tabela veio matar.
    #[test]
    fn the_predicates_are_views_of_the_table() {
        for kind in 0..FxOp::KINDS as u8 {
            let o = FxOp::new(kind);
            let s = FxOp::spec(kind);
            assert_eq!(o.tints(), s.color_label.is_some(), "tints() do {}", s.name);
            assert_eq!(
                o.displaces(),
                s.offset_labels.is_some(),
                "displaces() do {}",
                s.name
            );
        }
        // E as duas metades que decidem a MARGEM da textura: quem mora dentro não cresce nada.
        for kind in [
            FxOp::INNER_SHADOW,
            FxOp::INNER_GLOW,
            FxOp::BEVEL,
            FxOp::COLOR_OVERLAY,
        ] {
            assert!(
                !FxOp::spec(kind).grows,
                "{} desenha só dentro do que recebeu — margem seria textura paga a troco de nada",
                FxOp::kind_name(kind)
            );
        }
        for kind in [
            FxOp::BLUR,
            FxOp::GLOW,
            FxOp::DROP_SHADOW,
            FxOp::OUTLINE,
            FxOp::FEATHER,
        ] {
            assert!(FxOp::spec(kind).grows, "{} espalha", FxOp::kind_name(kind));
        }
        assert!(
            FxOp::spec(FxOp::COLOR_OVERLAY).radius_label.is_none(),
            "o Color Overlay é PONTUAL — um raio nele seria knob morto (e um dispatch a mais)"
        );
    }

    /// O teto é respondido pela pilha, não contado no chamador.
    #[test]
    fn the_ceiling_is_the_stacks_own_answer() {
        let mut f = VecFilter::default();
        while f.has_room() {
            f.ops.push(FxOp::new(FxOp::BLUR));
        }
        assert_eq!(f.ops.len(), VecFilter::MAX_OPS);
    }
}
