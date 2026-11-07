//! **Os nomes das camadas de colisão** (ficha f1, onda 2) e **quais se mostram** (onda 5): uma tabela por projeto, ao
//! lado da regra do mundo. O número continua a ser a identidade; sem nome dado, a camada mostra-o. Sem nomes de fábrica.

use crate::{Collider, LayerMatrix, MAX_LAYERS, RaySensor};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

const MAGIC: &[u8; 8] = b"PH2DLAYN";
const VERSION: u32 = 2;
const VERSION_DE_OITO: u32 = 1;

/// A camada 0 existe sempre: é onde todo objeto nasce.
const SEMPRE: u32 = 1;
/// O que um projeto até o schema 218 mostrava: as oito, sempre.
const AS_OITO: u32 = 0xFF;

/// `.0` = o nome de cada camada (vazio = o número). `.1` = as mostradas à mão: o «+ Layer», e as oito de um projeto antigo.
#[derive(Resource, Clone, PartialEq, Eq, Debug)]
pub struct LayerNames(pub [String; MAX_LAYERS], pub u32);

impl Default for LayerNames {
    fn default() -> Self {
        Self(Default::default(), SEMPRE)
    }
}

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

    /// O «+ Layer»: a camada passa a mostrar-se, mesmo sem nome nem objeto.
    pub fn show(world: &mut World, layer: u8) {
        if usize::from(layer) < MAX_LAYERS {
            let mut t = Self::of(world);
            t.1 |= 1 << layer;
            t.install(world);
        }
    }

    /// A próxima camada livre entre as não visíveis de `visiveis`; `None` nas 32.
    #[must_use]
    pub fn next_free(visiveis: u32) -> Option<u8> {
        let livre = (!visiveis).trailing_zeros();
        u8::try_from(livre)
            .ok()
            .filter(|&l| usize::from(l) < MAX_LAYERS)
    }

    /// As linhas para o ficheiro: nenhuma no projeto de fábrica, que fica byte a byte.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if *self == Self::default() {
            return Vec::new();
        }
        let corpo = postcard::to_allocvec(&(VERSION, &self.0, self.1)).unwrap_or_default();
        [&MAGIC[..], &corpo].concat()
    }

    /// Lê [`Self::to_bytes`] e a forma de oito nomes. Recusa o que não entende, em voz alta: nunca abre com nomes errados.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let corpo = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| String::from("layer names: bad magic"))?;
        let (versao, resto) =
            postcard::take_from_bytes::<u32>(corpo).map_err(|e| format!("layer names: {e}"))?;
        match versao {
            VERSION_DE_OITO => {
                let oito: [String; 8] =
                    postcard::from_bytes(resto).map_err(|e| format!("layer names: {e}"))?;
                let mut t = Self(Default::default(), AS_OITO);
                for (n, velho) in t.0.iter_mut().zip(oito) {
                    *n = velho;
                }
                Ok(t)
            }
            VERSION => {
                let (nomes, mostradas): ([String; MAX_LAYERS], u32) =
                    postcard::from_bytes(resto).map_err(|e| format!("layer names: {e}"))?;
                Ok(Self(nomes, mostradas))
            }
            v => Err(format!("layer names: version {v}")),
        }
    }

    /// Um projeto até o schema 218: os mesmos nomes, e as oito de sempre à vista (`vazio` = sem nomes).
    pub fn upgrade_v218(bytes: &[u8]) -> Result<Vec<u8>, String> {
        let mut t = Self::from_bytes(bytes)?;
        t.1 |= AS_OITO;
        Ok(t.to_bytes())
    }

    /// ⭐ **As camadas visíveis, em bits:** a 0, as mostradas à mão, as que têm nome, as que têm objeto (`em_uso`) e as
    /// de linha diferente do padrão. Nada configurado fica escondido.
    #[must_use]
    pub fn visible(&self, em_uso: u32, matriz: &LayerMatrix) -> u32 {
        let mut v = SEMPRE | self.1 | em_uso;
        for i in 0..MAX_LAYERS {
            if !self.0[i].is_empty() || matriz.row(i) != u32::MAX {
                v |= 1 << i;
            }
        }
        v
    }
}

/// As camadas com algum objeto: cada `Collider` e cada `RaySensor`.
#[must_use]
pub fn layers_in_use(world: &World) -> u32 {
    let mut uso = 0u32;
    let bit = |l: u8| 1u32 << u32::from(l).min(MAX_LAYERS as u32 - 1);
    if let Some(mut q) = world.try_query::<&Collider>() {
        uso = q.iter(world).fold(uso, |u, c| u | bit(c.layer));
    }
    if let Some(mut q) = world.try_query::<&RaySensor>() {
        uso = q.iter(world).fold(uso, |u, r| u | bit(r.layer));
    }
    uso
}

/// ⭐ **Quantos objetos há em cada camada** (a lista da aba «Collision Layers», onda 9a): um objeto conta
/// uma vez, na camada do seu `Collider`; o que só tem `RaySensor` conta na camada do sensor.
#[must_use]
pub fn objects_per_layer(world: &World) -> [u32; MAX_LAYERS] {
    let mut n = [0u32; MAX_LAYERS];
    let at = |l: u8| usize::from(l).min(MAX_LAYERS - 1);
    if let Some(mut q) = world.try_query::<&Collider>() {
        for c in q.iter(world) {
            n[at(c.layer)] += 1;
        }
    }
    if let Some(mut q) =
        world.try_query_filtered::<&RaySensor, bevy_ecs::query::Without<Collider>>()
    {
        for r in q.iter(world) {
            n[at(r.layer)] += 1;
        }
    }
    n
}

/// ⭐ **As camadas visíveis deste projeto** (a porta para o painel, o Inspector e a fila «Collides with»).
#[must_use]
pub fn visible_layers(world: &World, matriz: &LayerMatrix) -> u32 {
    LayerNames::of(world).visible(layers_in_use(world), matriz)
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

    /// Os bytes da forma v1 escritos à mão: a partir do tipo vivo o teste provaria a hipótese dele.
    #[test]
    fn a_forma_de_oito_nomes_abre_com_as_oito_a_vista() {
        let nomes: [String; 8] = std::array::from_fn(|i| {
            if i == 2 {
                "Ground".into()
            } else {
                String::new()
            }
        });
        let corpo = postcard::to_allocvec(&(1u32, &nomes)).unwrap();
        let t = LayerNames::from_bytes(&[&MAGIC[..], &corpo].concat()).expect("v1 abre");
        assert_eq!(t.label(2), "Ground");
        assert_eq!(t.visible(0, &LayerMatrix::all()), 0xFF);
    }

    #[test]
    fn um_projeto_v218_sem_nomes_mostra_as_oito_e_um_novo_so_a_zero() {
        let velho = LayerNames::upgrade_v218(&[]).unwrap();
        let velho = LayerNames::from_bytes(&velho).unwrap();
        assert_eq!(velho.visible(0, &LayerMatrix::all()), 0xFF);
        assert_eq!(LayerNames::default().visible(0, &LayerMatrix::all()), 1);
    }

    #[test]
    fn visivel_e_nome_objeto_linha_ou_a_mao() {
        let mut t = LayerNames::default();
        let all = LayerMatrix::all();
        t.0[12] = "Fx".into();
        assert_eq!(t.visible(0, &all), 1 | 1 << 12, "nome");
        assert_eq!(
            LayerNames::default().visible(1 << 9, &all),
            1 | 1 << 9,
            "objeto"
        );
        let mut m = all;
        m.set(20, 25, false);
        assert_eq!(
            LayerNames::default().visible(0, &m),
            1 | 1 << 20 | 1 << 25,
            "linha"
        );
        let mut w = World::new();
        LayerNames::show(&mut w, 3);
        assert_eq!(visible_layers(&w, &all), 1 | 1 << 3, "a mão");
        assert_eq!(LayerNames::next_free(1 | 1 << 3), Some(1));
        assert_eq!(LayerNames::next_free(0xF), Some(4));
        assert_eq!(LayerNames::next_free(u32::MAX), None, "passou de 32");
    }

    #[test]
    fn o_show_vai_e_volta_e_o_objeto_faz_a_camada_voltar() {
        let mut t = LayerNames::default();
        t.1 |= 1 << 30;
        assert_eq!(LayerNames::from_bytes(&t.to_bytes()), Ok(t));
        let mut w = World::new();
        let e = w
            .spawn(Collider {
                layer: 17,
                ..Collider::default()
            })
            .id();
        assert_eq!(visible_layers(&w, &LayerMatrix::all()), 1 | 1 << 17);
        w.despawn(e);
        assert_eq!(visible_layers(&w, &LayerMatrix::all()), 1);
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
