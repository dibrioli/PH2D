//! **O NOME QUE A INTERFACE MOSTRA** para cada verbo — irmão (`#[path]`) do
//! [`super::brush_verb`].
//!
//! ⚠️ **O corte foi por RESPONSABILIDADE e forçado pelo tecto de LOC** (o
//! ficheiro chegou a `704` com o [`crate::Verb::BoxTrim`]): ali fica *que verbos
//! existem e o que cada um FAZ*; aqui fica *como cada um se CHAMA*, que é
//! vocabulário de interface. ⛔ Nunca uma entrada no `FILE_OVERAGE_OK`.
//!
//! ⚠️ **A lista é EXAUSTIVA de propósito** (`match self` sem `_ =>`): um verbo
//! novo sem nome é **erro de compilação**, e não um chip com o rótulo do
//! vizinho.

impl crate::Verb {
    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, self.label_key())
    }

    /// ⭐⭐ **A CHAVE do rótulo** — `sculpt3d.verb.<variante>`, e é ela que a interface passa ao
    /// [`ph2d_i18n::tr`]. O texto vive na tabela (`ph2d-i18n/src/sculpt_engine.rs`), com a
    /// [`label`](Self::label) acima a lê-lo em inglês: *uma lei só, com um acessório derivado.*
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::BoxTrim => "sculpt3d.verb.box_trim",
            // ⚠️ **«Plane», e o nome é uma decisão de PRODUTO com duas cercas:**
            // (a) o dono pediu-o pela palavra *trim*, e esta casa já tem um
            // **Box Trim** que é outra coisa inteira (uma booleana no largar) —
            // dois chips com *trim* no nome seriam duas ferramentas
            // indistinguíveis na fileira, que é o defeito que o glifo
            // obrigatório de um painel existe para impedir; (b) o que este
            // pincel faz é ajustar um PLANO e puxar para ele, e é isso que o
            // artista precisa de ler para saber que os dois tectos são as duas
            // metades desse plano.
            Self::Plane => "sculpt3d.verb.plane",
            Self::Cloth => "sculpt3d.verb.cloth",
            Self::Draw => "sculpt3d.verb.draw",
            // ⭐ **O rótulo diz o EFEITO, e a espec §14.1 registra que o alvo
            // aprendeu isso do lado caro:** o primeiro nome deste pincel dizia
            // *como* ele funciona por dentro, e foi trocado antes de sair.
            Self::DrawSharp => "sculpt3d.verb.draw_sharp",
            Self::Inflate => "sculpt3d.verb.inflate",
            Self::Smooth => "sculpt3d.verb.smooth",
            Self::Sharpen => "sculpt3d.verb.sharpen",
            Self::Flatten => "sculpt3d.verb.flatten",
            Self::Fill => "sculpt3d.verb.fill",
            Self::Scrape => "sculpt3d.verb.scrape",
            Self::Clay => "sculpt3d.verb.clay",
            Self::Pinch => "sculpt3d.verb.pinch",
            Self::Magnify => "sculpt3d.verb.magnify",
            Self::Crease => "sculpt3d.verb.crease",
            Self::Blob => "sculpt3d.verb.blob",
            Self::Mask => "sculpt3d.verb.mask",
            // ⛔⛔ **INTEGRAÇÃO (20/09): estes três nasceram nesta linha com o RÓTULO no
            // lugar da CHAVE, e a fusão foi limpa e compilou.** A `line/UIUX`
            // migrou a fronteira dos motores para chave + tabela no mesmo dia, e
            // uma variante nova escrita no idioma antigo não é erro de
            // compilação — `label_key` devolve `&'static str` nos dois idiomas.
            // Quem os apanhou foi o censo `cada_rotulo_deste_motor_vem_da_tabela`
            // na ÁRVORE COMBINADA. ⚠️ E o `"Blur"` escapou ao censo do `ph2d-i18n`
            // porque o painel do VECTOR já tem uma ponte para essa palavra: as
            // pontes são indexadas pela PALAVRA e não pelo par `(crate, palavra)`,
            // logo uma coincidência de vocabulário entre dois motores esconde um
            // rótulo cru num terceiro. *Aqui a forma da chave apanha-o de graça.*
            Self::Paint => "sculpt3d.verb.paint",
            Self::Blur => "sculpt3d.verb.blur",
            Self::SmearColor => "sculpt3d.verb.smear_color",
            Self::Move => "sculpt3d.verb.move",
            Self::SnakeHook => "sculpt3d.verb.snake_hook",
            Self::Twist => "sculpt3d.verb.twist",
            Self::LocalScale => "sculpt3d.verb.local_scale",
            Self::ClayStrips => "sculpt3d.verb.clay_strips",
            Self::ClayThumb => "sculpt3d.verb.clay_thumb",
            Self::MultiplaneScrape => "sculpt3d.verb.multiplane_scrape",
            Self::SlideRelax => "sculpt3d.verb.slide_relax",
            Self::SurfaceSmooth => "sculpt3d.verb.surface_smooth",
            Self::Layer => "sculpt3d.verb.layer",
            Self::Pose => "sculpt3d.verb.pose",
            Self::Boundary => "sculpt3d.verb.boundary",
            Self::Density => "sculpt3d.verb.density",
            Self::EraseMultires => "sculpt3d.verb.erase_multires",
            Self::SmearMultires => "sculpt3d.verb.smear_multires",
            Self::SceneProject => "sculpt3d.verb.scene_project",
            Self::Thumb => "sculpt3d.verb.thumb",
            Self::Nudge => "sculpt3d.verb.nudge",
        }
    }
}
