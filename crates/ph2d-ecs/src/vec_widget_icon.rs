//! **O ícone ESCOLHIDO de um botão de ícone** — a segunda rota do plano UI/UX (W8b, §6.2).
//!
//! Um `IconButton` autorado desenha, por default, a **forma que o veste**: num editor vetorial a
//! resposta nativa a *qual ícone?* é *o que você desenhou*. Este componente é o **override**: o
//! artista escolhe um glifo do catálogo do editor, e a forma passa a ser só a moldura.
//!
//! # PRESENÇA é a escolha, e a ausência é o desenho
//!
//! O idioma dos overrides desta casa (`GravityScale`, `Ccd`, `ZIndexOverride`): quem não tem o
//! componente segue o default. Aqui o default é *o desenho do artista*, então tirar o componente
//! é literalmente **voltar ao desenho** — sem um segundo campo a dizer qual das duas rotas está
//! activa, e sem um estado em que as duas discordem.
//!
//! # Componente PRÓPRIO, e não um campo no [`crate::VecWidget`] — a diferença é um BUMP
//!
//! O blob de um componente é postcard **posicional**: apendar `icon` ao `VecWidget` moveria o
//! layout dele e obrigaria a subir o `PROJECT_SCHEMA`, o que **recusa todo projeto já salvo**. Um
//! componente novo cunha a própria chave (`blake3(NOME)[..8]`) e **não move nada** — é o
//! precedente exacto do `PhysicsJoint` (W3) e do `GravityScale` (W8) do módulo de física.
//!
//! # ⚠️ O SLUG, nunca o número — e isto não é preferência
//!
//! O discriminante de `IconId` **é a posição alfabética do arquivo SVG** (o
//! `enum_order_matches_svgs` da `ph2d-editor-core` pina isso), então acrescentar
//! `docs/design/icons/blob.svg` empurra **todo ícone depois de `blob`** uma casa. Um número
//! guardado aqui passaria a nomear outro glifo — em silêncio, em todo projeto já salvo. O slug é o
//! nome do arquivo: ele não se move quando um vizinho nasce.
//!
//! E um slug que este build não conhece **degrada para o desenho**, que é o mesmo canal de
//! compatibilidade do `kind` do [`crate::VecWidget`]: nunca recusar o arquivo.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O glifo escolhido**, pelo slug canônico (`ph2d_editor_core::icons::IconId::slug`).
#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecWidgetIcon {
    /// O slug kebab-case do ícone — `"play"`, `"chevron-down"`, `"trash"`.
    pub slug: String,
}

impl SimComponent for VecWidgetIcon {}
