//! **As receitas e o material do projeto**, o campo `sfx_lab` do `ProjectFile` (blob `PH2DSFXL`, com versão
//! própria; vazio = nada a gravar). Fora do `ProjectState`, como os kanbans: o desfazer é da aba
//! (`docs/SfxLab/04_W3_plano.md` §2).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Envelope, Layer, LayerId, Recipe, Source, Take, Takes};

const MAGIC: &[u8; 8] = b"PH2DSFXL";
const VERSAO: u8 = 2;
/// W2, antes de `Layer::muted` e `Recipe::approved`.
const VERSAO_SEM_MUDO: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "component", derive(bevy_ecs::resource::Resource))]
pub struct SfxLibrary {
    pub recipes: BTreeMap<String, Recipe>,
    pub takes: Takes,
}

/// O blob não é uma biblioteca legível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryBlobError(pub String);

impl std::fmt::Display for LibraryBlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Serialize)]
struct Escrita<'a> {
    recipes: Vec<(&'a str, &'a Recipe)>,
    takes: Vec<(u32, u16, &'a [f32])>,
}

#[derive(Deserialize)]
struct Lida<R> {
    recipes: Vec<(String, R)>,
    takes: Vec<(u32, u16, Vec<f32>)>,
}

/// O formato congelado da versão 1: postcard é posicional, e o campo novo não pode aparecer na leitura dele.
#[derive(Deserialize)]
struct RecipeV1 {
    seed: u64,
    engine: u32,
    seconds: f32,
    layers: Vec<LayerV1>,
    next_layer: u64,
}

#[derive(Deserialize)]
struct LayerV1 {
    id: LayerId,
    gain: f32,
    start: f32,
    source: Source,
    envelope: Envelope,
}

impl From<RecipeV1> for Recipe {
    fn from(r: RecipeV1) -> Self {
        Self {
            seed: r.seed,
            engine: r.engine,
            seconds: r.seconds,
            layers: r
                .layers
                .into_iter()
                .map(|l| Layer {
                    id: l.id,
                    gain: l.gain,
                    start: l.start,
                    muted: false,
                    source: l.source,
                    envelope: l.envelope,
                })
                .collect(),
            next_layer: r.next_layer,
            approved: None,
        }
    }
}

fn le_corpo<R>(corpo: &[u8]) -> Result<Lida<Recipe>, LibraryBlobError>
where
    R: for<'de> Deserialize<'de> + Into<Recipe>,
{
    let lida: Lida<R> = postcard::from_bytes(corpo).map_err(|e| LibraryBlobError(e.to_string()))?;
    Ok(Lida {
        recipes: lida
            .recipes
            .into_iter()
            .map(|(n, r)| (n, r.into()))
            .collect(),
        takes: lida.takes,
    })
}

impl SfxLibrary {
    /// O blob a gravar: vazio quando não há nada, e o projeto sem receitas continua igual ao byte.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.recipes.is_empty() && self.takes.is_empty() {
            return Vec::new();
        }
        let corpo = Escrita {
            recipes: self.recipes.iter().map(|(n, r)| (n.as_str(), r)).collect(),
            takes: (self.takes.iter())
                .map(|t| (t.rate, t.channels, &*t.samples))
                .collect(),
        };
        let mut out = MAGIC.to_vec();
        out.push(VERSAO);
        out.extend(postcard::to_allocvec(&corpo).unwrap_or_default());
        out
    }

    /// Lê o blob de [`Self::to_bytes`]. O endereço do material é recalculado, e toda receita passa pela lei dos valores.
    ///
    /// # Errors
    /// Cabeçalho, versão ou corpo ilegíveis.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LibraryBlobError> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let corpo = bytes
            .strip_prefix(MAGIC.as_slice())
            .ok_or_else(|| LibraryBlobError("not a sound recipe library".into()))?;
        let (&versao, corpo) = corpo
            .split_first()
            .ok_or_else(|| LibraryBlobError("sound recipe library without a version".into()))?;
        let lida = match versao {
            VERSAO => le_corpo::<Recipe>(corpo)?,
            VERSAO_SEM_MUDO => le_corpo::<RecipeV1>(corpo)?,
            _ => {
                return Err(LibraryBlobError(format!(
                    "sound recipe library version {versao}, this app reads up to {VERSAO}"
                )));
            }
        };
        let mut out = Self::default();
        for (nome, mut r) in lida.recipes {
            r.sanitize();
            out.recipes.insert(nome, r);
        }
        for (rate, channels, samples) in lida.takes {
            out.takes.add(Take::new(rate, channels, samples));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheia() -> SfxLibrary {
        let mut lib = SfxLibrary::default();
        let id = lib.takes.add(Take::new(44_100, 2, vec![0.25, -0.25]));
        lib.recipes
            .insert("Take".into(), crate::factory::recorded(id, 0.1));
        for (n, r) in crate::factory::all() {
            lib.recipes.insert(n.into(), r);
        }
        lib
    }

    /// Bytes gravados pelo código de W2, antes de `muted` e `approved` existirem. Nunca se regrava.
    #[test]
    fn blob_v1_abre_na_v2() {
        let lida = SfxLibrary::from_bytes(include_bytes!("fixtures/library_v1.bin")).unwrap();
        let mut lib = SfxLibrary::default();
        let id = lib
            .takes
            .add(Take::new(44_100, 2, vec![0.25, -0.25, 0.5, -0.5]));
        let mut r = Recipe::new(7, 0.25);
        r.add(
            0.5,
            crate::Source::Noise(crate::Noise {
                filter: crate::NoiseFilter::Band,
                hz: 800.0,
                hz_end: 200.0,
                q: 2.0,
            }),
            crate::Envelope::hit(0.01, 0.1, 0.01),
        );
        r.add(
            0.75,
            crate::Source::Recording(crate::Recording {
                take: id,
                semitones: -5.0,
                offset: 0.0,
            }),
            crate::Envelope::default(),
        );
        r.layers[1].start = 0.05;
        lib.recipes.insert("Saved Before W3".into(), r);
        assert_eq!(lida, lib);
    }

    #[test]
    fn mudo_e_aprovado_voltam_iguais_do_blob() {
        let mut lib = cheia();
        let r = lib.recipes.get_mut("Impact").unwrap();
        r.layers[0].muted = true;
        r.approved = Some(crate::Approved {
            sound: "Impact.wav".into(),
            blake3: [7; 32],
            engine: crate::ENGINE_V1,
        });
        assert_eq!(SfxLibrary::from_bytes(&lib.to_bytes()).unwrap(), lib);
    }

    #[test]
    fn a_biblioteca_volta_igual_e_a_vazia_nao_escreve_nada() {
        assert!(SfxLibrary::default().to_bytes().is_empty());
        assert_eq!(SfxLibrary::from_bytes(&[]).unwrap(), SfxLibrary::default());
        let lib = cheia();
        assert_eq!(SfxLibrary::from_bytes(&lib.to_bytes()).unwrap(), lib);
    }

    #[test]
    fn blob_ilegivel_e_recusado_com_o_motivo() {
        assert!(SfxLibrary::from_bytes(b"lixo").is_err());
        let mut outra_versao = cheia().to_bytes();
        outra_versao[8] = 99;
        let e = SfxLibrary::from_bytes(&outra_versao).unwrap_err();
        assert!(e.0.contains("99"), "{e}");
    }

    /// O mesmo áudio entra uma vez, e o endereço muda com qualquer bit.
    #[test]
    fn o_material_e_endereçado_pelo_conteudo() {
        let mut t = Takes::default();
        let a = t.add(Take::new(48_000, 1, vec![0.5, 0.25]));
        assert_eq!(a, t.add(Take::new(48_000, 1, vec![0.5, 0.25])));
        assert_eq!(t.iter().count(), 1);
        assert_ne!(a, Take::new(44_100, 1, vec![0.5, 0.25]).id);
        assert_ne!(a, Take::new(48_000, 1, vec![0.5, 0.26]).id);
    }
}

/// A porta do projeto: a biblioteca do mundo em bytes e de volta.
#[cfg(feature = "component")]
pub mod project {
    use super::{LibraryBlobError, SfxLibrary};
    use bevy_ecs::world::World;

    /// O blob a gravar.
    #[must_use]
    pub fn save(world: &World) -> Vec<u8> {
        world
            .get_resource::<SfxLibrary>()
            .map(SfxLibrary::to_bytes)
            .unwrap_or_default()
    }

    /// # Errors
    /// O blob é ilegível: o load recusa o arquivo inteiro, como os sons.
    pub fn open(bytes: &[u8]) -> Result<SfxLibrary, LibraryBlobError> {
        SfxLibrary::from_bytes(bytes)
    }

    /// Substitui a biblioteca do documento anterior.
    pub fn install(world: &mut World, lib: SfxLibrary) {
        world.insert_resource(lib);
    }
}
