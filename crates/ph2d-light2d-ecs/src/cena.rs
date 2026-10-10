//! **A `CenaDeLuz`** — a resposta da coleta e a entrada de todo solver. Dado puro, coordenadas do
//! mundo em metros, cores lineares.

/// Cor linear RGB.
pub type Rgb = [f32; 3];

/// Quem emite. Todo emissor com área é **opaco**: o raio termina no primeiro emissor que acerta.
#[derive(Clone, Debug, PartialEq)]
pub enum Emissor {
    /// Disco de `raio > 0` com radiância uniforme.
    Disco {
        centro: [f32; 2],
        raio: f32,
        radiancia: Rgb,
        alcance_m: f32,
    },
    /// O limite `raio → 0` do disco: contribui `intensidade · queda(r) / r` pela única direção que
    /// o liga ao ponto. Não bloqueia ninguém.
    Ponto {
        centro: [f32; 2],
        intensidade: Rgb,
        alcance_m: f32,
    },
    /// Polígono simples (o quad de um sprite emissivo), radiância uniforme.
    Poligono {
        vertices: Vec<[f32; 2]>,
        radiancia: Rgb,
        alcance_m: f32,
    },
}

/// Quem bloqueia: polígono simples; cada ENTRADA do raio multiplica por `1 − opacidade`.
#[derive(Clone, Debug, PartialEq)]
pub struct Oclusor {
    pub vertices: Vec<[f32; 2]>,
    pub opacidade: f32,
    /// ⭐ W5c — o albedo LINEAR com que o corpo devolve a luz (o rebote, plano §2.4): só num corpo
    /// OPACO que recebe luz; zero = não reflete (o translúcido, o `NaoRecebeLuz`).
    pub albedo: Rgb,
}

/// A cena, em ordem estável (a da coleta: `StableId`, depois `Entity`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CenaDeLuz {
    pub emissores: Vec<Emissor>,
    pub oclusores: Vec<Oclusor>,
    /// Radiância do céu (soma das luzes `Ambiente`), que chega por toda direção que escapa.
    pub ceu: Rgb,
    /// ⭐ Quantas `Luz` a cena TEM (visíveis), acesas ou não — a GUARDA do quadro (W3): sem nenhuma,
    /// o solver não corre e o quadro é o de sempre. Uma sprite que brilha não conta (ela ilumina
    /// quando a cena tem luz; sozinha deixaria todo projeto antigo com brilho no escuro).
    pub luzes: u32,
    /// ⭐ As entidades com `NaoRecebeLuz` (ordem do `StableId`) — o receptor não as multiplica.
    pub nao_recebem: Vec<bevy_ecs::entity::Entity>,
    /// ⭐ Os sprites que emitem ou bloqueiam, com o QUAD que os representa acima (W3): a C0 da placa
    /// troca esse quad pelo alfa de cada texel da textura; o gabarito continua a ver o quad.
    pub sprites: Vec<SpriteDeLuz>,
    /// ⭐ Os caminhos VETORIAIS que bloqueiam (W3) — a geometria não está no `World` (vive na
    /// `VecScene`): a C0 da placa desenha-os pelo Vello na grade; o gabarito NÃO os vê.
    pub vetores: Vec<VetorDeLuz>,
    /// ⭐ Cada `Luz` que virou emissor, com o índice dela em [`Self::emissores`] (W4): a UI diz o
    /// que a captura fez com ESTA lâmpada.
    pub luzes_emissoras: Vec<(bevy_ecs::entity::Entity, usize)>,
}

/// Um caminho vetorial que bloqueia a luz (W3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VetorDeLuz {
    pub entidade: bevy_ecs::entity::Entity,
    /// O `VecPathId` do caminho (o `VecPathRef` da entidade).
    pub caminho: u64,
    /// A opacidade por entrada (`BloqueiaLuz`).
    pub opacidade: f32,
    /// ⭐ W5c — o caminho devolve a luz (opaco e recebe luz); o albedo é a cor DELE, que a camada do
    /// Vello já traz.
    pub reflete: bool,
}

/// Um sprite que entra na luz (W3) — a ponte entre o quad da cena e a textura que a C0 amostra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteDeLuz {
    pub entidade: bevy_ecs::entity::Entity,
    /// Onde está o quad dele nesta cena.
    pub forma: FormaDoSprite,
    /// O multiplicador da cor do texel na emissão (`tint∘self_tint`·α·`SpriteEmissive`); zero num
    /// oclusor.
    pub emissao: Rgb,
    /// A opacidade por entrada (1 num emissor, que é opaco; a do `BloqueiaLuz` num oclusor).
    pub opacidade: f32,
    /// ⭐ W5c — o multiplicador da cor do texel no ALBEDO do rebote (`tint∘self_tint`); zero = não
    /// reflete (emissor, translúcido, `NaoRecebeLuz`).
    pub albedo: Rgb,
}

/// O índice do quad de um sprite em [`CenaDeLuz::emissores`] ou [`CenaDeLuz::oclusores`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormaDoSprite {
    Emissor(usize),
    Oclusor(usize),
}
