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
}
