//! **`ph2d-viewport3d` — a moldura de uma janela 3D, partilhada pelos DOIS módulos que a têm.**
//!
//! # Por que esta crate existe (e por que ela nasceu na linha do substrato)
//!
//! O piloto da W2 era tirar a família `field3d_*` da shell. O censo do acoplamento devolveu uma
//! coisa que o briefing não previa: **`sculpt3d_*` referencia `field3d_*` em sete ficheiros** —
//! `sculpt3d_navball`, `sculpt3d_viewports`, `sculpt3d_gizmo`, `sculpt3d_keys_view`,
//! `sculpt3d_scenes_viewports` — e o que ele usa de lá **não tem uma linha de campo implícito**:
//!
//! | o que a escultura consome | o que isso é |
//! |---|---|
//! | [`views::Standard`], [`views::view_for_key`] | as seis vistas nomeadas (`Numpad1/3/7`) |
//! | [`navball`] (`Ball`, `balls_from_basis`, `hits_widget`, `pick`) | o gizmo de navegação |
//! | [`layout`] (`Split`, `rects`, `hit`, `t_at`, `seam_*`) | as quatro viewports e as costuras |
//! | [`gizmo`] (`Handle`, `pick`, `project_with`) | o gizmo de transformação |
//! | [`view_menu::row_at`] | o menu de vistas |
//!
//! Isto é **moldura de janela 3D**, e está aqui por acidente de ordem: a modelagem escreveu-a
//! primeiro (W47–W52) e a escultura reutilizou-a em 10/09, quando ganhou as quatro viewports.
//!
//! ⛔ **Deixá-la dentro de `ph2d-app-field3d` fecharia a armadilha na linha seguinte:** a
//! `line/app-sculpt3d` teria de pôr `ph2d-app-sculpt3d` a depender de `ph2d-app-field3d` — uma
//! família irmã inteira, com as 17 cenas de smoke dela dentro — para desenhar uma bola de eixos.
//! *Duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra.*
//!
//! # ⚠️ O que ficou de fora, e a razão de cada um
//!
//! O corte foi medido ficheiro a ficheiro, e a régua foi *«isto pergunta alguma coisa ao estado da
//! família?»*:
//!
//! - **`gizmo_paint` ficou na família** — ele lê `field3d_smoke`, e pintar o gizmo *daquela cena* é
//!   trabalho da cena. O que veio foi a **lei** (geometria, projecção, picking), que não sabe que
//!   existe uma cena.
//! - **Quatro ficheiros de teste ficaram na família** (`layout`, `navball`, `view_menu`, `views`):
//!   eles exercitam estes módulos **através** do smoke do 3D, e é assim que devem ser exercitados —
//!   uma lei de moldura provada por um gesto real vale mais do que uma provada por um `Rect`
//!   escrito à mão. Só os dois **puros** (`area_tests`, `gizmo_drag_tests`) vieram.
//!
//! # ⏳ Dívida NOMEADA: o `Orbit` mora na crate de render do módulo de modelagem
//!
//! [`ph2d_field_render::Orbit`] é o tipo de câmera, e é dele que esta crate depende — logo a
//! escultura depende, por transitividade, da crate de render da **modelagem**. ⚠️ **Isso já era
//! verdade antes desta crate existir** (o `field3d_navball` sempre usou `Orbit`); o que muda é que
//! agora tem nome. A cura é um vocabulário 3D partilhado (`Orbit`/`Screen` numa folha própria) e
//! **não é desta wave** — ela mexeria na API de `ph2d-field-render`, que é do módulo de modelagem.

#![forbid(unsafe_code)]

pub mod gizmo;
pub mod layout;
pub mod navball;
pub mod navball_paint;
pub mod view_menu;
pub mod views;
