//! **A PILHA de FX raster de uma forma** — Blur / Outer Glow / Drop Shadow, encadeáveis.
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
    /// Desligado = a pilha o SALTA, como se não estivesse lá — mas os parâmetros ficam. Espelha o
    /// `FxEntry::enabled` da pilha de geometria: desarmar não pode custar números que o artista
    /// teria de lembrar.
    pub enabled: bool,
}

impl FxOp {
    /// Código de painel: um blur puro (borra o que chegou).
    pub const BLUR: u8 = 0;
    /// Código de painel: um brilho externo (a silhueta borrada, tingida, POR BAIXO do que chegou).
    pub const GLOW: u8 = 1;
    /// Código de painel: uma sombra projetada (o Glow deslocado por um offset).
    pub const DROP_SHADOW: u8 = 2;
    /// Quantos tipos existem — o painel oferece um "Add" por tipo, a partir daqui.
    pub const KINDS: usize = 3;

    /// O nome que o painel mostra. Mora AQUI (junto do `kind`) para não haver duas tabelas a
    /// discordar sobre qual índice é o quê.
    #[must_use]
    pub fn kind_name(kind: u8) -> &'static str {
        match kind {
            Self::GLOW => "Glow",
            Self::DROP_SHADOW => "Drop Shadow",
            _ => "Blur",
        }
    }

    /// Este degrau tinge (tem cor de halo)? Blur não — ele reusa os pixels que chegaram.
    #[must_use]
    pub fn tints(self) -> bool {
        self.kind == Self::GLOW || self.kind == Self::DROP_SHADOW
    }

    /// Só a Drop Shadow desloca; Blur e Glow ficam no lugar.
    #[must_use]
    pub fn displaces(self) -> bool {
        self.kind == Self::DROP_SHADOW
    }

    /// Este degrau contribui? Desligado, a pilha o salta (espelho do `FxEntry::is_active`).
    #[must_use]
    pub fn is_active(self) -> bool {
        self.enabled
    }

    /// O degrau que um "Add" recém-clicado deve criar, com defaults **VISÍVEIS** — armar no
    /// neutro seria um clique que não muda um pixel.
    #[must_use]
    pub fn new(kind: u8) -> Self {
        match kind {
            Self::GLOW => Self {
                kind,
                radius: 0.18,
                offset: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                opacity: 1.0,
                enabled: true,
            },
            Self::DROP_SHADOW => Self {
                kind,
                radius: 0.1,
                offset: [0.12, -0.12],
                color: [0.0, 0.0, 0.0, 1.0],
                opacity: 0.6,
                enabled: true,
            },
            _ => Self {
                kind: Self::BLUR,
                radius: 0.12,
                offset: [0.0, 0.0],
                color: [0.0, 0.0, 0.0, 1.0],
                opacity: 1.0,
                enabled: true,
            },
        }
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
    #[test]
    fn a_new_op_is_born_visible() {
        for kind in [FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW] {
            let o = FxOp::new(kind);
            assert_eq!(o.kind, kind);
            assert!(o.enabled, "nasce ligado");
            assert!(o.radius > 0.0, "raio zero não borraria nada ({kind})");
            assert!(o.opacity > 0.0, "opacidade zero seria invisível ({kind})");
        }
        assert!(
            FxOp::new(FxOp::DROP_SHADOW).offset != [0.0, 0.0],
            "uma sombra sem deslocamento é um glow escuro — o default tem de a mostrar"
        );
        assert!(
            FxOp::new(FxOp::BLUR).offset == [0.0, 0.0],
            "só a Drop Shadow desloca"
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
