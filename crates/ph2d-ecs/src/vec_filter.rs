//! **O FX raster de uma forma** — Blur / Outer Glow / Drop Shadow, resolution-crisp.
//!
//! Irmão de [`crate::VecOffset`]/[`crate::VecTextPath`]/[`crate::VecEnvelope`] no padrão que esta
//! linha já usou meia dúzia de vezes: o componente guarda a **relação** (que efeito, com que raio,
//! deslocamento, cor) e a aparência é **derivada** dela — a shell a produz por frame. De graça:
//! **undo e save cobrem o FX sem uma linha a mais** (os dois capturam o mundo ECS, e este
//! componente está registrado no `ComponentRegistry`).
//!
//! # Por que NÃO é um `PathEffect` da pilha (ADR-0132) — e a razão é do TIPO
//!
//! Um efeito da pilha é `VecPath -> VecPath`, avaliado DENTRO da `ph2d-vec-scene`
//! ([`crate`] não a alcança, mas veja `ph2d_vec_scene::effect::run_stack`) — o modelo puro de
//! documento, sem kurbo, sem GPU. Um FX raster produz **PIXELS**, não uma curva: você **borra**,
//! **desloca** e **tinge** o que a forma rasteriza, e não há `VecPath` que represente isso. É a
//! MESMA fronteira que o `VecOffset` documenta para a geometria, um degrau adiante: lá o offset
//! precisava do motor booleano (ciclo de crate); aqui o filtro precisa do rasterizador e do
//! compositor GPU, que vivem no shell. Pôr o filtro na pilha faria o `cooked()` — a resposta a
//! *"o que este documento desenha?"* — depender de alguém ter instalado um pipeline de GPU em
//! runtime, uma porta que pode não ter sido aberta, falhando em silêncio.
//!
//! Logo o FX raster é um **post-pass orquestrado no SHELL** (onde `VelloPass` + `LayerCompositor`
//! estão em escopo): a shell isola a forma na própria textura, roda o filtro texture→texture, e
//! recompõe no z dela. Plano em `docs/Vector Module/24_plano_fx_raster.md`.
//!
//! # As unidades, e por que MUNDO
//!
//! `radius`/`offset` são de **MUNDO**. A textura-scratch é rasterizada na resolução do device (o
//! tamanho da forma NA TELA), então o kernel em pixels é `mundo × zoom` — o filtro fica
//! **resolution-crisp**: re-renderizado por frame no zoom atual, proporcional em toda escala (a
//! propriedade que o feather do Rive tem, por outra via). Guardar pixels congelaria o efeito num
//! zoom; guardar fração exigiria re-derivar a escala da seleção a cada frame (a lição do `VecOffset`).
//!
//! # Ausência = sem filtro
//!
//! Não há variante "None" guardada: escolher *None* no painel **REMOVE** o componente (a lei do
//! `VecOffset`: um documento não acumula relações inertes que não desenham nada). Uma forma sem
//! `VecFilter` flui pelo `dispatch` **byte-idêntica** ao mundo de hoje — o caminho comum não paga
//! nada.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O FX raster de uma forma.** A entidade que o carrega também tem um [`crate::VecPathRef`]: o
/// `VecPath` dela continua a curva AUTORADA (o modo Node a edita); o resultado filtrado é DESENHO,
/// que a shell produz por frame e injeta no z da fonte.
///
/// ⚠️ **W1 é UM filtro por forma** (o corte inicial que fecha o Rive-headline). A composição de
/// vários filtros numa cadeia é a W2 (o grafo componível) — e é por isso que os params moram num
/// struct plano com um `kind`, não em campos por-efeito: a W2 troca isto por uma lista sem quebrar
/// o save (o componente cunha `stable_type_id` próprio; apender NÃO é postcard posicional aqui
/// porque a chave é o NOME, não a posição — mas a W2 decide a forma da lista).
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecFilter {
    /// Que efeito, no código do painel: `0` Blur · `1` Outer Glow · `2` Drop Shadow. *None* não é
    /// um valor — é a ausência do componente.
    pub kind: u8,
    /// O raio de suavização em unidades de MUNDO (o `stdDev` do gaussiano). A shell o converte para
    /// pixels do device pelo zoom. `0` não acontece: o painel remove o componente antes.
    pub radius: f32,
    /// O deslocamento da sombra em unidades de MUNDO (só Drop Shadow; Blur/Glow ignoram).
    pub offset: [f32; 2],
    /// A cor da sombra/brilho, RGBA reta em `[0,1]` (Blur ignora — ele borra os pixels da própria
    /// forma).
    pub color: [f32; 4],
    /// A intensidade/opacidade do efeito em `[0,1]`.
    pub opacity: f32,
}

impl SimComponent for VecFilter {}

impl VecFilter {
    /// Código de painel: um blur puro (borra a própria forma).
    pub const BLUR: u8 = 0;
    /// Código de painel: um brilho externo (a forma borrada, tingida, atrás dela).
    pub const GLOW: u8 = 1;
    /// Código de painel: uma sombra projetada (a forma borrada, deslocada, tingida, atrás dela).
    pub const DROP_SHADOW: u8 = 2;

    /// Um filtro novo.
    #[must_use]
    pub fn new(kind: u8, radius: f32, offset: [f32; 2], color: [f32; 4], opacity: f32) -> Self {
        Self { kind, radius, offset, color, opacity }
    }

    /// A sombra/glow tem cor e (para a sombra) deslocamento; o Blur não os lê.
    #[must_use]
    pub fn tints(self) -> bool {
        self.kind == Self::GLOW || self.kind == Self::DROP_SHADOW
    }

    /// Só a Drop Shadow desloca; Blur e Glow ficam no lugar.
    #[must_use]
    pub fn displaces(self) -> bool {
        self.kind == Self::DROP_SHADOW
    }
}
