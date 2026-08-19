//! **O EMPACOTADOR** — N imagens nomeadas viram UMA folha com N regiões.
//!
//! É a metade da ferramenta de criar hand-packed que **não depende** de como o artista arranja as
//! regiões: seja o arranjo automático, seja um que ele mexeu à mão, alguém tem de compor os
//! pixels numa imagem só e produzir os retângulos. Esta é essa função, e ela é pura.
//!
//! ## Um empacotador só no projeto
//!
//! Usa o **mesmo** `rect_packer` (heurística Skyline) que o atlas dinâmico usa desde o M14.4d.
//! Um segundo algoritmo aqui seria uma segunda resposta a *"como o PH2D arruma retângulos"*, e as
//! duas divergiriam no dia em que uma fosse afinada.
//!
//! ## Determinismo é requisito, não conforto
//!
//! As entradas são **ordenadas antes de empacotar** (altura ↓, largura ↓, nome), então o mesmo
//! conjunto de sprites produz sempre a mesma folha, byte-a-byte — HR-5. Isso é o que torna a
//! folha exportável, re-importável e comparável: sem isso, empacotar duas vezes daria dois
//! arquivos diferentes com o mesmo conteúdo, e o `git diff` de um projeto ficaria ruidoso para
//! sempre.
//!
//! ⚠️ A ordem de EMPACOTAMENTO (por altura) não é a ordem das REGIÕES no documento: aquela é por
//! **nome**, e é ela que faz o índice ser uma referência estável ([`crate::AuthoredSheet::new`]).
//! São duas ordens com dois propósitos, de propósito.
//!
//! ## O tamanho da folha é MEDIDO, não escolhido
//!
//! Tenta 64², 128², 256²… e para na **primeira** potência de dois em que tudo cabe. Não há um
//! `SHEET_SIZE` fixo: o teto (`PackOptions::max_size`) é do *dispositivo* — a maior textura que a
//! GPU aceita —, e quem o passa é quem sabe. Um limite que não diz de que recurso é seria um
//! palpite à espera de um smoke (`CLAUDE.md` §0).

use crate::AuthoredSheet;

/// Uma imagem a empacotar.
pub struct PackInput {
    /// O nome que a região terá na folha — o que o artista vê e por que procura.
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// RGBA8 justo: `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// Como empacotar.
#[derive(Copy, Clone, Debug)]
pub struct PackOptions {
    /// Pixels transparentes entre regiões vizinhas.
    ///
    /// ⚠️ **Não é estética.** A amostragem bilinear lê meio texel para fora da borda, então duas
    /// regiões coladas sangram uma na outra ao mínimo zoom. O `region_filter_clip` do sprite
    /// defende o caso comum, mas ele não existe em toda engine que vai ler esta folha exportada —
    /// e o padding defende-a em qualquer uma. `2` é o valor que o Aseprite e o TexturePacker
    /// oferecem por omissão.
    pub padding: u32,
    /// A maior folha aceitável, em pixels de lado.
    ///
    /// ⚠️ **É um limite do DISPOSITIVO** (`wgpu::Limits::max_texture_dimension_2d`), e por isso
    /// vem de quem o conhece em vez de estar escrito aqui.
    pub max_size: u32,
}

impl Default for PackOptions {
    fn default() -> Self {
        Self {
            padding: 2,
            // O mínimo que a especificação do WebGPU garante em qualquer adaptador. Quem tem mais
            // passa mais; ninguém tem menos.
            max_size: 8192,
        }
    }
}

/// Por que um empacotamento não pôde ser feito.
#[derive(Debug, PartialEq, Eq)]
pub enum PackError {
    /// Nenhuma entrada.
    Empty,
    /// Uma imagem sozinha já não cabe no teto do dispositivo — nenhum arranjo resolve.
    TooLarge {
        name: String,
        width: u32,
        height: u32,
        max: u32,
    },
    /// O conjunto não cabe no teto, mesmo cabendo cada peça.
    DoesNotFit { count: usize, max: u32 },
    /// `rgba.len()` não bate com `width * height * 4`.
    PixelCountMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "nothing to pack"),
            Self::TooLarge {
                name,
                width,
                height,
                max,
            } => write!(
                f,
                "'{name}' is {width}x{height}, larger than the {max}x{max} device limit — \
                 no arrangement fits it"
            ),
            Self::DoesNotFit { count, max } => write!(
                f,
                "{count} images do not fit in {max}x{max} — pack fewer, or shrink them"
            ),
            Self::PixelCountMismatch {
                name,
                expected,
                found,
            } => write!(
                f,
                "'{name}' declares {expected} pixel bytes but carries {found}"
            ),
        }
    }
}

impl std::error::Error for PackError {}

/// Empacota `inputs` numa folha única.
///
/// A folha resultante já vem com as regiões ordenadas por nome (o construtor de
/// [`AuthoredSheet`] garante), e os pixels compostos.
pub fn pack(
    id: u32,
    sheet_name: String,
    inputs: Vec<PackInput>,
    opts: PackOptions,
) -> Result<AuthoredSheet, PackError> {
    if inputs.is_empty() {
        return Err(PackError::Empty);
    }
    for i in &inputs {
        let expected = (i.width as usize)
            .saturating_mul(i.height as usize)
            .saturating_mul(4);
        if i.rgba.len() != expected {
            return Err(PackError::PixelCountMismatch {
                name: i.name.clone(),
                expected,
                found: i.rgba.len(),
            });
        }
        // Cada peça leva o padding consigo, então é o tamanho INFLADO que tem de caber.
        let (w, h) = (i.width + opts.padding, i.height + opts.padding);
        if w > opts.max_size || h > opts.max_size {
            return Err(PackError::TooLarge {
                name: i.name.clone(),
                width: i.width,
                height: i.height,
                max: opts.max_size,
            });
        }
    }
    // ⚠️ A ORDEM DE EMPACOTAMENTO: altura ↓, depois largura ↓, depois nome. As duas primeiras são
    // a heurística clássica (peças altas primeiro deixam menos buraco na skyline); a terceira é o
    // desempate que torna o resultado DETERMINÍSTICO — sem ela, duas peças do mesmo tamanho
    // trocariam de lugar conforme a ordem de chegada, e a mesma cena daria folhas diferentes.
    let mut order: Vec<usize> = (0..inputs.len()).collect();
    order.sort_by(|&a, &b| {
        let (x, y) = (&inputs[a], &inputs[b]);
        y.height
            .cmp(&x.height)
            .then(y.width.cmp(&x.width))
            .then(x.name.cmp(&y.name))
    });

    // A menor potência de dois em que tudo cabe. Começa no menor lado que a maior peça exige.
    let biggest = inputs
        .iter()
        .map(|i| (i.width + opts.padding).max(i.height + opts.padding))
        .max()
        .unwrap_or(1);
    let mut size = 64u32.max(biggest.next_power_of_two());
    let placed = loop {
        if let Some(p) = try_pack(&inputs, &order, size, opts.padding) {
            break p;
        }
        if size >= opts.max_size {
            return Err(PackError::DoesNotFit {
                count: inputs.len(),
                max: opts.max_size,
            });
        }
        size = (size * 2).min(opts.max_size);
    };

    // Compõe os pixels. A folha nasce **transparente**, e é isso que faz o padding ser padding em
    // vez de lixo: o que houver entre as regiões é alfa zero.
    let mut rgba = vec![0u8; (size as usize) * (size as usize) * 4];
    for (idx, (x, y)) in &placed {
        blit(&mut rgba, size, &inputs[*idx], *x, *y);
    }
    Ok(AuthoredSheet::new(
        id,
        sheet_name,
        size,
        size,
        rgba,
        placed
            .iter()
            .map(|(idx, (x, y))| {
                let i = &inputs[*idx];
                (i.name.clone(), [*x, *y, i.width, i.height])
            })
            .collect::<Vec<_>>(),
    ))
}

/// Tenta arrumar tudo numa folha `size × size`. `None` = não coube.
fn try_pack(
    inputs: &[PackInput],
    order: &[usize],
    size: u32,
    padding: u32,
) -> Option<Vec<(usize, (u32, u32))>> {
    let mut packer = rect_packer::DensePacker::new(size as i32, size as i32);
    let mut out = Vec::with_capacity(order.len());
    for &idx in order {
        let i = &inputs[idx];
        // `false` = sem rotação: uma região rodada obrigaria o consumidor a saber disso, e o
        // formato do Aseprite que exportamos não tem como dizê-lo sem entrar no campo `rotated`
        // que o nosso próprio leitor ignora.
        let r = packer.pack(
            (i.width + padding) as i32,
            (i.height + padding) as i32,
            false,
        )?;
        out.push((idx, (r.x as u32, r.y as u32)));
    }
    Some(out)
}

/// Copia uma imagem para dentro da folha, linha a linha.
fn blit(sheet: &mut [u8], sheet_w: u32, src: &PackInput, x: u32, y: u32) {
    let row = (src.width as usize) * 4;
    for r in 0..src.height as usize {
        let from = r * row;
        let to = (((y as usize + r) * sheet_w as usize) + x as usize) * 4;
        sheet[to..to + row].copy_from_slice(&src.rgba[from..from + row]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(name: &str, w: u32, h: u32, fill: u8) -> PackInput {
        PackInput {
            name: name.to_string(),
            width: w,
            height: h,
            rgba: vec![fill; (w * h * 4) as usize],
        }
    }

    fn opts() -> PackOptions {
        PackOptions {
            padding: 0,
            max_size: 1024,
        }
    }

    #[test]
    fn nothing_to_pack_is_an_error_not_an_empty_sheet() {
        assert_eq!(pack(0, "s".into(), Vec::new(), opts()), Err(PackError::Empty));
    }

    #[test]
    fn every_input_becomes_a_region() {
        let sheet = pack(
            1,
            "s".into(),
            vec![img("b", 16, 16, 1), img("a", 16, 16, 2)],
            opts(),
        )
        .expect("pack");
        assert_eq!(sheet.regions.len(), 2);
        // As regiões saem ordenadas por NOME (a ordem do documento), não pela de empacotamento.
        assert_eq!(sheet.region(0).unwrap().name, "a");
        assert_eq!(sheet.region(1).unwrap().name, "b");
    }

    /// ⚠️ **A lei que torna a folha exportável.** Sem o desempate por nome, duas peças do mesmo
    /// tamanho trocariam de lugar conforme a ordem de chegada, e empacotar a mesma cena duas
    /// vezes daria dois arquivos diferentes — o `git diff` do projeto ficaria ruidoso para sempre.
    #[test]
    fn the_same_images_always_produce_the_same_sheet() {
        let a = pack(
            1,
            "s".into(),
            vec![img("x", 16, 16, 1), img("y", 16, 16, 2)],
            opts(),
        )
        .expect("pack");
        let b = pack(
            1,
            "s".into(),
            vec![img("y", 16, 16, 2), img("x", 16, 16, 1)],
            opts(),
        )
        .expect("pack");
        assert_eq!(a, b, "a MESMA cena, declarada ao contrario, e' a mesma folha");
    }

    /// Nenhuma região pode sobrepor outra — é a propriedade inteira de um empacotador.
    #[test]
    fn regions_never_overlap() {
        let inputs: Vec<PackInput> = (0..12)
            .map(|i| img(&format!("r{i:02}"), 10 + i * 3, 7 + i * 5, i as u8))
            .collect();
        let sheet = pack(1, "s".into(), inputs, opts()).expect("pack");
        for (i, a) in sheet.regions.iter().enumerate() {
            for b in sheet.regions.iter().skip(i + 1) {
                let (ax, ay, aw, ah) = (a.rect[0], a.rect[1], a.rect[2], a.rect[3]);
                let (bx, by, bw, bh) = (b.rect[0], b.rect[1], b.rect[2], b.rect[3]);
                let disjoint = ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay;
                assert!(disjoint, "'{}' sobrepoe '{}'", a.name, b.name);
            }
        }
    }

    /// Toda região tem de caber DENTRO da folha — o mesmo invariante que o `validate` do
    /// documento cobra no encode, provado aqui na origem.
    #[test]
    fn every_region_lies_inside_the_sheet() {
        let inputs: Vec<PackInput> = (0..9).map(|i| img(&format!("r{i}"), 40, 30, 0)).collect();
        let sheet = pack(1, "s".into(), inputs, opts()).expect("pack");
        for r in &sheet.regions {
            assert!(r.rect[0] + r.rect[2] <= sheet.width, "{} sai em x", r.name);
            assert!(r.rect[1] + r.rect[3] <= sheet.height, "{} sai em y", r.name);
        }
    }

    /// A folha é a MENOR potência de dois que serve — não um tamanho fixo.
    #[test]
    fn the_sheet_grows_only_as_much_as_needed() {
        let small = pack(1, "s".into(), vec![img("a", 8, 8, 0)], opts()).expect("pack");
        assert_eq!((small.width, small.height), (64, 64), "piso de 64");
        let big = pack(1, "s".into(), vec![img("a", 300, 300, 0)], opts()).expect("pack");
        assert_eq!((big.width, big.height), (512, 512));
    }

    /// Uma peça maior que o teto do dispositivo não tem arranjo que a salve — e a mensagem diz
    /// QUAL peça, senão o artista fica a adivinhar entre cem.
    #[test]
    fn an_image_bigger_than_the_device_limit_names_itself() {
        let e = pack(
            1,
            "s".into(),
            vec![img("gigante", 2048, 16, 0)],
            PackOptions {
                padding: 0,
                max_size: 1024,
            },
        )
        .unwrap_err();
        assert!(matches!(e, PackError::TooLarge { ref name, .. } if name == "gigante"));
        assert!(e.to_string().contains("gigante"));
    }

    #[test]
    fn a_set_that_cannot_fit_says_so() {
        let inputs: Vec<PackInput> = (0..8).map(|i| img(&format!("r{i}"), 500, 500, 0)).collect();
        let e = pack(
            1,
            "s".into(),
            inputs,
            PackOptions {
                padding: 0,
                max_size: 1024,
            },
        )
        .unwrap_err();
        assert!(matches!(e, PackError::DoesNotFit { count: 8, max: 1024 }));
    }

    /// Os PIXELS chegam ao sítio certo — não basta o retângulo estar certo.
    #[test]
    fn the_pixels_land_where_the_region_says() {
        let sheet = pack(1, "s".into(), vec![img("a", 4, 4, 0xAB)], opts()).expect("pack");
        let r = sheet.region(0).unwrap().rect;
        for row in 0..r[3] {
            for col in 0..r[2] {
                let i = (((r[1] + row) * sheet.width + r[0] + col) * 4) as usize;
                assert_eq!(&sheet.rgba[i..i + 4], &[0xAB; 4], "pixel {row},{col}");
            }
        }
    }

    /// ⚠️ O espaço entre regiões é TRANSPARENTE. Se a folha nascesse com lixo, o padding —
    /// que existe para impedir sangramento — sangraria ele próprio.
    #[test]
    fn the_gap_between_regions_is_transparent() {
        let sheet = pack(
            1,
            "s".into(),
            vec![img("a", 8, 8, 0xFF), img("b", 8, 8, 0xFF)],
            PackOptions {
                padding: 4,
                max_size: 1024,
            },
        )
        .expect("pack");
        // Um canto que nenhuma região reclama tem de estar a zero.
        let inside_any = |x: u32, y: u32| {
            sheet.regions.iter().any(|r| {
                x >= r.rect[0] && x < r.rect[0] + r.rect[2] && y >= r.rect[1]
                    && y < r.rect[1] + r.rect[3]
            })
        };
        let mut checked = 0;
        for y in 0..sheet.height {
            for x in 0..sheet.width {
                if !inside_any(x, y) {
                    let i = ((y * sheet.width + x) * 4) as usize;
                    assert_eq!(&sheet.rgba[i..i + 4], &[0, 0, 0, 0], "vazio em {x},{y}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "o teste tem de ter olhado para algum vazio");
    }

    #[test]
    fn pixels_that_do_not_match_the_declared_size_are_refused() {
        let bad = PackInput {
            name: "torto".into(),
            width: 4,
            height: 4,
            rgba: vec![0; 8],
        };
        assert_eq!(
            pack(1, "s".into(), vec![bad], opts()),
            Err(PackError::PixelCountMismatch {
                name: "torto".into(),
                expected: 64,
                found: 8,
            })
        );
    }
}
