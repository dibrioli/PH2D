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
}
