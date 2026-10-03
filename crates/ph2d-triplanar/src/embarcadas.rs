//! ⭐ **O pacote embutido** — sete texturas do Poly Haven (CC0, `texturas/LICENSE-CC0.txt`), a 1k,
//! sem modificação: cor, normal (OpenGL) e rugosidade.

use crate::{LADO, Mapas, Mipmaps};

/// Uma textura do pacote, na ordem do painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Embarcada {
    Tijolo,
    Madeira,
    Pedra,
    Chapa,
    Ferrugem,
    Couro,
    Betao,
}

macro_rules! ficheiros {
    ($n:literal) => {
        (
            include_bytes!(concat!("../texturas/", $n, "_diff_1k.jpg")).as_slice(),
            include_bytes!(concat!("../texturas/", $n, "_nor_gl_1k.jpg")).as_slice(),
            include_bytes!(concat!("../texturas/", $n, "_rough_1k.jpg")).as_slice(),
        )
    };
}

impl Embarcada {
    /// As sete, na ordem do painel.
    pub const TODAS: [Self; 7] = [
        Self::Tijolo,
        Self::Madeira,
        Self::Pedra,
        Self::Chapa,
        Self::Ferrugem,
        Self::Couro,
        Self::Betao,
    ];

    /// O nome do ficheiro no Poly Haven.
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Tijolo => "red_brick",
            Self::Madeira => "wood_table_001",
            Self::Pedra => "rock_boulder_dry",
            Self::Chapa => "metal_plate",
            Self::Ferrugem => "rusty_metal_02",
            Self::Couro => "brown_leather",
            Self::Betao => "concrete_floor_worn_001",
        }
    }

    /// ⭐ O tamanho REAL de um ladrilho, em metros (o `dimensions` do Poly Haven) — a omissão do
    /// tamanho na peça.
    #[must_use]
    pub fn tamanho_real(self) -> f32 {
        match self {
            Self::Tijolo => 1.4,
            Self::Madeira => 1.5,
            Self::Pedra => 1.8,
            Self::Chapa => 0.5,
            Self::Ferrugem => 1.0,
            Self::Couro => 0.4,
            Self::Betao => 3.0,
        }
    }

    fn bytes(self) -> (&'static [u8], &'static [u8], &'static [u8]) {
        match self {
            Self::Tijolo => ficheiros!("red_brick"),
            Self::Madeira => ficheiros!("wood_table_001"),
            Self::Pedra => ficheiros!("rock_boulder_dry"),
            Self::Chapa => ficheiros!("metal_plate"),
            Self::Ferrugem => ficheiros!("rusty_metal_02"),
            Self::Couro => ficheiros!("brown_leather"),
            Self::Betao => ficheiros!("concrete_floor_worn_001"),
        }
    }

    /// ⭐ Decodifica os três ficheiros e monta os [`Mapas`] (com os mips). Custa dezenas de ms: quem
    /// chama guarda o resultado.
    ///
    /// # Errors
    /// Se um JPG embutido não decodificar (um defeito do pacote; há gate).
    pub fn mapas(self) -> Result<Mapas, String> {
        let (cor, nor, rug) = self.bytes();
        let cor = jpg(cor)?;
        let nor = jpg(nor)?;
        let rug = jpg(rug)?;
        Ok(Mapas {
            cor: Mipmaps::de_rgba8(cor.0, cor.1, &cor.2, LADO, true),
            nrh: junta_nrh(&nor, Some(&rug), LADO),
            tem_normal: true,
            tem_rugosidade: true,
        })
    }
}

/// Uma imagem decodificada: `(largura, altura, pixels RGBA8 com a origem em cima)`.
pub type Imagem = (u32, u32, Vec<[u8; 4]>);

fn jpg(b: &[u8]) -> Result<Imagem, String> {
    use ph2d_imageio::ImageImporter;
    let img = ph2d_imageio_jpeg::JpegImporter
        .import(b, &ph2d_imageio::ImportOpts::default())
        .map_err(|e| format!("{e}"))?;
    match img {
        ph2d_imageio::DecodedImage::Flat(b) => {
            Ok((b.width, b.height, b.pixels.iter().map(|p| p.0).collect()))
        }
        _ => Err("o JPG não é uma imagem plana".to_owned()),
    }
}

/// ⭐ O mapa `nrh` a partir de uma normal (rgb) e de uma rugosidade (o r dela, cinzenta), cada uma
/// do seu tamanho. Sem rugosidade, `a = 0`; sem normal quem chama passa a plana `(128, 128, 255)`.
#[must_use]
pub fn junta_nrh(nor: &Imagem, rug: Option<&Imagem>, lado: u32) -> Mipmaps {
    let n = Mipmaps::de_rgba8(nor.0, nor.1, &nor.2, lado, false);
    let r = rug.map(|r| Mipmaps::de_rgba8(r.0, r.1, &r.2, lado, false));
    let base: Vec<[u8; 4]> = n
        .nivel(0)
        .iter()
        .enumerate()
        .map(|(i, c)| [c[0], c[1], c[2], r.as_ref().map_or(0, |r| r.nivel(0)[i][0])])
        .collect();
    // Os mips refazem-se dos bytes juntos (média de números, sem curva), já virados: desvira-se
    // para entrar pela mesma porta.
    let topo: Vec<[u8; 4]> = (0..lado as usize)
        .flat_map(|j| {
            let r = (lado as usize - 1 - j) * lado as usize;
            base[r..r + lado as usize].to_vec()
        })
        .collect();
    Mipmaps::de_rgba8(lado, lado, &topo, lado, false)
}
