//! **Fase do quadro: O PRÉ-QUADRO DO SCULPT3D** — o puxão do Grab, a escultura pendente do Ctrl+O, a
//! doação da forma e a ponte com a Hierarquia (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A ordem interna é lei escrita em cada comentário** (a ponte antes de a Hierarquia ser
//! desenhada). Quem põe e tira o barro é o modo do objecto (`fase_object_mode`, spec/06 F3). Sem a
//! `feature` `sculpt3d` o corpo é vazio.

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sculpt3d_pre_frame(&mut self) {
        // ⚠️ **O puxão do Grab é carimbado UMA vez por quadro, não por evento.**
        // Ver `Sculpt3dScene::pending_grab`: o alvo do `Grip::Hold` é função do
        // `pre` congelado e do puxão TOTAL, então os dabs intermediários são
        // byte-idênticos ao último — 16 deles custavam 17,9 ms onde um custa
        // 1,2. E ANTES do desenho, senão o barro que este quadro mostra é o do
        // quadro anterior.
        #[cfg(feature = "sculpt3d")]
        // ⭐ A shell PROCURA a cena (é ela que tem o `gfx`) e a família aplica a lei — desde
        // 2026-09-11 (W2/L3-A2) esta porta é uma função livre, não um `impl App`.
        if let Some(scene) = self.sculpt3d_scene_mut() {
            ph2d_app_sculpt3d::flush_grab(scene);
        }
        // A escultura que um Ctrl+O deixou pendente — ela espera o device, que o
        // load não tinha (ADR-0150 W8.3).
        #[cfg(feature = "sculpt3d")]
        self.sculpt3d_install_pending();
        // A DOAÇÃO: rasteriza a forma no tamanho que o Painter publicou no frame anterior e deixa o
        // plano no canal. Quase sempre não faz nada — sem cena armada sai no primeiro `if`.
        #[cfg(feature = "sculpt3d")]
        self.sculpt3d_donate_form();
        // ⭐⭐⭐ **A PONTE com a Hierarquia** (uma peça ⟺ uma entidade) — ver
        // [`ph2d_app_sculpt3d::entities`]. ⚠️ **Depois do `install_pending`**, para uma cena
        // instalada NESTE quadro já entrar na lista (e a peça nascida ter a entidade com que pede o
        // modo), e **antes** de a Hierarquia ser desenhada. Sem cena armada é um `return` imediato.
        #[cfg(feature = "sculpt3d")]
        self.sculpt3d_entities_sync();
    }
}
