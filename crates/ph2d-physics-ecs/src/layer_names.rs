//! **Os nomes das camadas de colisão** (ficha f1, onda 2): uma tabela por projeto, ao lado da regra do mundo. O número
//! 0–7 continua a ser a identidade; sem nome dado, a camada mostra-o. Sem nomes de fábrica.

use crate::MAX_LAYERS;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

const MAGIC: &[u8; 8] = b"PH2DLAYN";
const VERSION: u32 = 1;

#[derive(Resource, Clone, PartialEq, Eq, Debug, Default)]
pub struct LayerNames(pub [String; MAX_LAYERS]);

impl LayerNames {
    /// A tabela do projeto aberto (as de fábrica, se nunca foi renomeada).
    #[must_use]
    pub fn of(world: &World) -> Self {
        world.get_resource::<Self>().cloned().unwrap_or_default()
    }

    /// Põe esta tabela no mundo, se mudou.
    pub fn install(&self, world: &mut World) {
        if Self::of(world) != *self {
            world.insert_resource(self.clone());
        }
    }

    /// O nome da camada `layer` passa a `name`, sem espaços nas pontas; vazio volta ao número.
    pub fn rename(world: &mut World, layer: u8, name: &str) {
        let mut t = Self::of(world);
        if let Some(n) = t.0.get_mut(usize::from(layer)) {
            name.trim().clone_into(n);
            t.install(world);
        }
    }

    /// O que se lê: o nome, ou o número sem nome dado.
    #[must_use]
    pub fn label(&self, layer: usize) -> String {
        match self.0.get(layer) {
            Some(n) if !n.is_empty() => n.clone(),
            _ => layer.to_string(),
        }
    }

    /// As linhas para o ficheiro: nenhuma sem nomes, o projeto que nunca renomeou fica byte a byte.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.0.iter().all(String::is_empty) {
            return Vec::new();
        }
        let corpo = postcard::to_allocvec(&(VERSION, &self.0)).unwrap_or_default();
        [&MAGIC[..], &corpo].concat()
    }

    /// Lê [`Self::to_bytes`]. Recusa o que não entende, em voz alta: nunca abre com nomes errados.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let corpo = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| String::from("layer names: bad magic"))?;
        let (versao, nomes): (u32, [String; MAX_LAYERS]) =
            postcard::from_bytes(corpo).map_err(|e| format!("layer names: {e}"))?;
        if versao != VERSION {
            return Err(format!("layer names: version {versao}"));
        }
        Ok(Self(nomes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_nomes_vao_e_voltam_e_sem_nomes_nao_ha_bytes() {
        assert!(LayerNames::default().to_bytes().is_empty());
        assert_eq!(LayerNames::from_bytes(&[]), Ok(LayerNames::default()));
        let mut t = LayerNames::default();
        t.0[2] = "Ground".into();
        assert_eq!(LayerNames::from_bytes(&t.to_bytes()).as_ref(), Ok(&t));
        assert_eq!(t.label(2), "Ground");
        assert_eq!(t.label(3), "3", "sem nome dado, o número");
        assert!(LayerNames::from_bytes(b"lixo").is_err());
    }

    #[test]
    fn renomear_apara_e_vazio_volta_ao_numero() {
        let mut w = World::new();
        LayerNames::rename(&mut w, 1, "  Enemy ");
        assert_eq!(LayerNames::of(&w).label(1), "Enemy");
        LayerNames::rename(&mut w, 1, "");
        assert_eq!(LayerNames::of(&w).label(1), "1");
        LayerNames::rename(&mut w, 99, "ninguém");
        assert_eq!(LayerNames::of(&w), LayerNames::default());
    }
}
