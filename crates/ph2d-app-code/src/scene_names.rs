//! Os nomes da cena que as abas leem: a lei mora em [`ph2d_code_bancada::names`] (o que o editor
//! sabe da cena); aqui só o quadro que os segue.

pub(crate) use ph2d_code_bancada::names::{SceneNames, Shared, property_items, scene_items};
use ph2d_ecs::World;
use std::sync::Arc;

impl crate::docs::CodeDocs {
    /// Um quadro: os nomes da cena (só com abas abertas); as abas passam a ler a cópia nova quando
    /// eles mudam — e o lint reconta (a chave dele é a cópia).
    pub(crate) fn follow_scene(&mut self, world: &mut World) {
        if self.docs.is_empty() {
            return;
        }
        let now = SceneNames::read(world);
        if *self.scene != now {
            self.scene = Arc::new(now);
        }
        for d in &mut self.docs {
            if !Arc::ptr_eq(&d.scene, &self.scene) {
                d.scene = Arc::clone(&self.scene);
            }
        }
    }
}
