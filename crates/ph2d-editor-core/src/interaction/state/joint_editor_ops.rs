//! O pedido da aba Joint Editor (plano `docs/JointEditor/01_plano.md` §W3): quem pede é o botão «Open in
//! Joint Editor» do Inspector; quem abre é a família `ph2d-app-joint-editor`.

use super::WidgetStore;

impl WidgetStore {
    /// Pede a aba aberta na montagem `assembly` (o `StableId`; `0` = nenhuma em particular).
    pub fn request_joint_editor(&mut self, assembly: u64) {
        self.joint_editor_request = Some(assembly);
    }

    /// Lê e limpa o pedido.
    pub fn take_joint_editor_request(&mut self) -> Option<u64> {
        self.joint_editor_request.take()
    }
}
