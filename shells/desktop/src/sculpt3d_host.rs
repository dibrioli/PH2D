//! ⭐⭐⭐ **ONDE A SHELL ATENDE A FAMÍLIA DA ESCULTURA** — os quinze invólucros, e nada mais.
//!
//! A família inteira vive em [`ph2d_app_sculpt3d`] desde a W2/L3 (112 ficheiros, ~31,9 k linhas).
//! O que ficou aqui são as funções que **desmontam o [`crate::app_state::AppGfx`]** e entregam à
//! família o que ela precisa — nunca o contrário.
//!
//! ## Por que os nomes não mudaram
//!
//! Os onze sítios de produto que chamam esta família (`input_dispatch`, `render_loop`,
//! `keyboard`, `keyboard_files`) continuam a escrever `self.sculpt3d_key(...)`. ⚠️ **Isso é
//! deliberado e tem preço medido:** renomeá-los era um diff de 11 sítios a mais sobre uma
//! fusão que já move 112 ficheiros, e dois dos gates de arquitectura deste repo nomeiam essas
//! funções **por string** — um deles entra em pânico com *«controlo positivo»* quando o nome
//! muda, que é a falha que se lê como *o gate partido* em vez de *a tabela desactualizada*.
//!
//! ## As três formas de empréstimo, e por que são três
//!
//! ⚠️⚠️ **O corte não é uniforme, e foi a MEDIÇÃO que o decidiu** — cada função pede o mínimo:
//!
//! | forma | quem | por quê |
//! |---|---|---|
//! | **`take` + devolução** | [`Self::sculpt3d_pointer_down`], [`Self::sculpt3d_wheel`] | elas perguntam à **porta do chrome**, e essa pergunta tem de ficar DENTRO da família (gate `the_scene_asks_the_one_chrome_door`). Para isso a família precisa do `&mut impl AppHost` inteiro — e `&mut self` mais `&mut self.gfx…sculpt3d` não coexistem. Tirar a cena do `AppGfx` durante a chamada dissolve o empréstimo duplo |
//! | **split borrow do `AppGfx`** | [`Self::sculpt3d_key`], [`Self::sculpt3d_entities_sync`] | elas precisam de DOIS campos do `AppGfx` ao mesmo tempo (a cena e o `hero_screen`/`sim`) e de **nenhuma** porta do host — logo desmontar basta |
//! | **campos soltos** | as restantes | precisam do device, do tamanho, do slot ou dos toasts, e de mais nada |
//!
//! ⛔⛔ **E o `take` só é seguro por uma propriedade MEDIDA que nada obrigava a valer:**
//! *nenhuma das seis portas do [`ph2d_app_host::AppHost`] lê `gfx.sculpt3d`.* Elas leem o
//! `hero_screen`, o índice de acerto e os modificadores. Se uma sétima porta — ou uma mudança
//! na `canvas_visible` — passasse a consultar a cena, ela veria `None` **durante o gesto**, e o
//! sintoma seria a escultura a sumir só enquanto o dedo está em baixo. É a armadilha §2.10 do
//! HOWTO (*«uma fronteira nova põe um elo novo na corrente que nenhum gate mede»*), e é por
//! isso que ela tem um: `the_host_never_reads_the_borrowed_scene`.

/// O executor da sonda do undo da escultura — aqui porque o `main.rs` está no tecto de LOC.
#[path = "sculpt3d_undo_probe.rs"]
mod undo_probe;

use crate::app_state::{App, AppGfx};
use ph2d_i18n::tr;

impl App {
    /// **Onde a cena mora** — `AppGfx.sculpt3d`, que nasce `None`.
    ///
    /// ⚠️ Cinco sítios de produto a chamam. Ela fica na shell porque é a shell que é dona do
    /// `AppGfx`: a família recebe a cena, nunca a procura.
    pub(crate) fn sculpt3d_scene_mut(&mut self) -> Option<&mut ph2d_app_sculpt3d::Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }

    /// Arma a cena do smoke, uma vez.
    pub(crate) fn sculpt3d_smoke(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            // ⚠️ O `None` viaja: a família tem o próprio guarda estático, e ele **não** pode
            // ser armado por uma corrida sem janela.
            return ph2d_app_sculpt3d::input::smoke(&mut None, None);
        };
        let size = gfx.surface.size();
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        ph2d_app_sculpt3d::input::smoke(
            &mut gfx.sculpt3d,
            Some((&device, (size.width, size.height))),
        );
    }

    /// O cursor que a costura da divisão 3D pede.
    ///
    /// ⚠️ **Dois empréstimos PARTILHADOS, logo nem `take` nem desmontagem**: o host é `&self` e
    /// a cena sai de `&self.gfx`, e duas leituras coexistem.
    pub(crate) fn sculpt3d_seam_cursor(&self) -> Option<winit::window::CursorIcon> {
        let scene = self.gfx.as_ref()?.sculpt3d.as_ref()?;
        ph2d_app_sculpt3d::input::seam_cursor(self, scene)
    }

    /// A roda aproxima. **Empresta a cena** — ver o cabeçalho.
    pub(crate) fn sculpt3d_wheel(&mut self, steps: f32) -> bool {
        self.com_a_cena_emprestada(|host, scene| {
            ph2d_app_sculpt3d::input::wheel(host, scene, steps)
        })
    }

    /// O botão apertou. **Empresta a cena** — ver o cabeçalho.
    pub(crate) fn sculpt3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        // ⭐ Com o Painter a pintar a peça, o botão ESQUERDO é dele (`painter_na_malha`);
        // os outros continuam a navegar a vista.
        if button == winit::event::MouseButton::Left
            && self
                .painter_tool_mut()
                .is_some_and(|p| p.on_screen_canvas())
        {
            return false;
        }
        let tomou = self.com_a_cena_emprestada(|host, scene| {
            ph2d_app_sculpt3d::input_down::pointer_down(host, scene, button)
        });
        if tomou {
            self.a_escultura_toma_o_canvas();
        }
        tomou
    }

    /// ⭐⭐ **Um gesto da escultura no canvas LARGA a ferramenta em mãos** — a 4.ª linha da tabela
    /// do dono do canvas ([`ph2d_app_field3d::mode`], com o report de 2026-09-16 e a medição).
    /// ⚠️ Só com o barro na tela: um aperto sem barro (a costura das vistas) não é escultura.
    fn a_escultura_toma_o_canvas(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let clay = gfx
            .sculpt3d
            .as_ref()
            .is_some_and(ph2d_app_sculpt3d::Sculpt3dScene::clay_on_screen);
        let Some(neutral) = gfx.tools.default_tool_id() else {
            return;
        };
        let owner = ph2d_app_field3d::mode::Owner {
            tool: gfx.tools.active().map(ph2d_editor_core::Tool::id),
            clay,
        };
        if clay && ph2d_app_field3d::mode::clay_takes_the_canvas(&owner, &neutral) {
            gfx.tools.set_active(&neutral);
            gfx.toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
                "shell.sculpt3d_host.sculpting_took_the_canvas",
            )));
            self.title_dirty = true;
        }
    }

    /// ⭐⭐ **O EMPRÉSTIMO, num sítio só.**
    ///
    /// ⚠️⚠️ **A devolução é INCONDICIONAL e é isso que a torna segura.** A família tem dezenas
    /// de `return` cedo — `pointer_down` sozinho tem mais de vinte — e um `take` escrito lá
    /// dentro perderia a escultura no primeiro deles, **em silêncio**. Aqui não há caminho
    /// entre o `take` e a devolução por onde um `return` da família possa passar: ela devolve
    /// um `bool` e quem escreve o `Some` de volta é esta função.
    ///
    /// ⛔ Um `panic!` da família ainda perderia a cena — e isso é aceite: um pânico aqui derruba
    /// o `winit` inteiro, e não há `catch_unwind` neste caminho.
    fn com_a_cena_emprestada(
        &mut self,
        gesto: impl FnOnce(&mut Self, &mut ph2d_app_sculpt3d::Sculpt3dScene) -> bool,
    ) -> bool {
        let Some(mut scene) = self.gfx.as_mut().and_then(|g| g.sculpt3d.take()) else {
            return false;
        };
        let tomou = gesto(self, &mut scene);
        self.gfx
            .as_mut()
            .expect("o `gfx` existe: a cena acabou de sair de dentro dele")
            .sculpt3d = Some(scene);
        tomou
    }

    /// As teclas da cena 3D. **Desmonta o `AppGfx`** — ela precisa da cena E do `hero_screen`.
    pub(crate) fn sculpt3d_key(
        &mut self,
        code: winit::keyboard::KeyCode,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        // ⚠️ **Colhidos com `&self`, ANTES do empréstimo mutável** — e o `text_focused` é hoje
        // UMA leitura onde eram duas (a guarda geral e o `Delete` perguntavam cada um por si,
        // e nada obrigava as respostas a concordar).
        let factos = ph2d_app_sculpt3d::keys::keys_delete::DeleteFacts {
            clay_on_screen: self.sculpt3d_clay_on_screen(),
            text_focused: self.text_entry_focused(),
            over_panel: crate::forwarding::cursor_over_hero_panel(
                self.gfx.as_ref(),
                self.last_pointer.0,
                self.last_pointer.1,
            ),
            vector_has_selection: self.vec.pen.selected_vert().is_some()
                || !self.vec.pen.selected_paths().is_empty(),
        };
        // ⚠️ A RAZÃO e não o `bool`: com ela a família diz porque recusou o `Ctrl+Z` (16/09).
        let morta = self.sculpt3d_keys_dead_reason();
        let App {
            gfx, sculpt3d_req, ..
        } = self;
        let Some(gfx) = gfx.as_mut() else {
            return false;
        };
        let Some(scene) = gfx.sculpt3d.as_mut() else {
            return false;
        };
        ph2d_app_sculpt3d::keys::key(
            scene,
            sculpt3d_req,
            gfx.hero_screen.as_mut(),
            ph2d_app_sculpt3d::keys::KeyPress { code, ctrl, shift },
            &factos,
            morta,
        )
    }

    /// `Ctrl+Alt+Q` divide a janela em quatro.
    pub(crate) fn sculpt3d_quad_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        let mods = ph2d_app_host::AppHost::mods(self);
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        ph2d_app_sculpt3d::keys_view::quad_key(scene, code, mods)
    }

    /// O pill SCULPT entra ou sai do barro.
    pub(crate) fn sculpt3d_apply_toggle(&mut self) {
        let App {
            gfx, sculpt3d_req, ..
        } = self;
        match gfx.as_mut() {
            Some(gfx) => {
                let size = gfx.surface.size();
                let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
                ph2d_app_sculpt3d::mode::apply_toggle(
                    &mut gfx.sculpt3d,
                    sculpt3d_req,
                    Some((&device, (size.width, size.height))),
                )
            }
            None => ph2d_app_sculpt3d::mode::apply_toggle(&mut None, sculpt3d_req, None),
        }
    }

    /// Instala a escultura que um load deixou pendente.
    pub(crate) fn sculpt3d_install_pending(&mut self) {
        let App { gfx, sculpt3d, .. } = self;
        match gfx.as_mut() {
            Some(gfx) => {
                let size = gfx.surface.size();
                let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
                ph2d_app_sculpt3d::doc::install_pending(
                    &mut gfx.sculpt3d,
                    &mut sculpt3d.pending,
                    Some((&device, (size.width, size.height))),
                )
            }
            // ⚠️ **Sem janela a pendência tem de SOBREVIVER** — é por isso que o `None` chega à
            // família em vez de esta função sair cedo: ela é que sabe não a tomar.
            None => ph2d_app_sculpt3d::doc::install_pending(&mut None, &mut sculpt3d.pending, None),
        }
    }

    /// A sincronia da Hierarquia ⇄ escultura, por quadro.
    pub(crate) fn sculpt3d_entities_sync(&mut self) {
        let App {
            gfx,
            sculpt3d: shell,
            ..
        } = self;
        let Some(gfx) = gfx.as_mut() else {
            return;
        };
        let hier_sel = gfx.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
        let AppGfx { sim, sculpt3d, .. } = gfx;
        let Some(scene) = sculpt3d.as_mut() else {
            return;
        };
        ph2d_app_sculpt3d::entities::entities_sync(sim, scene, shell, hier_sel, &mut |sim| {
            // ⛔ **A folha do nome único NÃO é da família** — ela serve o Flip, o vetor, a
            // física e a Hierarquia, e é alvo de linha própria. Quem a possui passa-a.
            ph2d_unique_name::unique_name(sim, tr("shell.sculpt3d_host.sculpt"))
        });
    }

    /// A doação chega à tinta do Painter.
    pub(crate) fn sculpt3d_donate_form(&mut self) {
        let App {
            gfx, donated_form, ..
        } = self;
        let Some(gfx) = gfx.as_mut() else {
            return;
        };
        // ⚠️ O `GpuContext` e a cena vivem os dois no `AppGfx`; desmontá-lo separa-os sem o
        // clone de dois `Arc` que a versão anterior desta chamada pagava por quadro.
        let AppGfx {
            sculpt3d, surface, ..
        } = gfx;
        let Some(scene) = sculpt3d.as_mut() else {
            return;
        };
        ph2d_app_sculpt3d::donation::donate_form(scene, donated_form, surface.gpu());
    }

    /// Escreve a cena num arquivo escolhido pelo artista.
    pub(crate) fn sculpt3d_export(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            sculpt3d, toasts, ..
        } = gfx;
        ph2d_app_sculpt3d::export::export(sculpt3d.as_ref(), toasts);
    }

    /// Lê cada arquivo de malha soltado e põe-no na cena.
    pub(crate) fn sculpt3d_import_files(&mut self, paths: &[std::path::PathBuf]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        let AppGfx {
            sculpt3d, toasts, ..
        } = gfx;
        ph2d_app_sculpt3d::import::import_files(
            paths,
            sculpt3d,
            (&device, (size.width, size.height)),
            toasts,
        );
    }

    /// Escolher um arquivo de malha e importá-lo (`Ctrl+Shift+O`).
    pub(crate) fn sculpt3d_pick_and_import(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        let AppGfx {
            sculpt3d, toasts, ..
        } = gfx;
        ph2d_app_sculpt3d::import::pick_and_import(
            sculpt3d,
            (&device, (size.width, size.height)),
            toasts,
        );
    }
}
