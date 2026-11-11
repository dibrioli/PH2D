//! **A ESCADA do `PROJECT_SCHEMA`** — o número do formato de arquivo, e como
//! ele chegou onde está.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e por LOC:** o irmão [`super::project`]
//! responde *"o que um arquivo de projeto contém, e como ele vai e volta do
//! disco"*; este responde *"que versão ele é, e por que"*. O `project.rs`
//! cruzou o teto de 600 do HR-18 com o degrau v77, e a escada é a metade que
//! cresce **um parágrafo por wave** — separá-la é o corte que não volta.
//!
//! ⚠️ **A escada mora COLADA à constante de propósito**, e a razão está escrita
//! no degrau v69: ele chegou ao `main` com a linha da escada AUSENTE, e *quem
//! conta o próximo degrau lê a escada, não o literal*. Mover as duas juntas
//! preserva isso; mover só o literal seria o defeito outra vez.
//!
//! ⚠️ **E o valor se CONTA contra o `main` do dia, nunca se escolhe** — esta
//! colisão passa **muda** quando duas linhas escrevem o MESMO número, porque o
//! git não sabe o que ele significa.

/// Versão do formato de arquivo de projeto. Bump ⇒ hard-break, sem migração, até o app ser funcional (CLAUDE §0.11).
///
/// ⚠️ **Os degraus de v2 a v168 estão ARQUIVADOS**, verbatim, em SETE arquivos por faixa:
/// `project_schema_history` (`v2`..`v82`, ARQUIVADO verbatim em `docs/archive/project-schema/escada_v2_a_v82.md`), `project_schema_history_v83` (ARQUIVADO em `docs/archive/project-schema/escada_v83_a_v98.md`)
/// (`v83`..`v98`), `project_schema_history_v99` (`v99`..`v111`, ARQUIVADO em `docs/archive/project-schema/escada_v99_resto.md`),
/// [`super::project_schema_history_v112`] (`v112`..`v128`),
/// [`super::project_schema_history_v128`] (`v128`..`v144`) e
/// [`super::project_schema_history_v144`] (`v144`..`v160`) e
/// [`super::project_schema_history_v160`] (`v160`..`v168`). O corte é por IDADE e o tecto de 600
/// LOC do HR-18 é quem o pede — sete vezes até hoje, e a de 2026-09-17 foi a primeira em que o
/// gatilho não foi uma linha mas a ACUMULAÇÃO de uma rodada: seis linhas puseram dezasseis degraus,
/// e nenhuma estoura o tecto sozinha.
///
/// ⚠️⚠️ **A fronteira de cada faixa é MEDIDA, nunca escolhida:** ela é o `PROJECT_SCHEMA` do
/// `main` de que a rodada VIVA nasceu (`git merge-base`), porque abaixo dele estão rodadas
/// FECHADAS e acima está o que alguém a contar o próximo degrau precisa de ver.
/// O que se lê para contar o próximo degrau é a ponta, e a ponta é o que ficou aqui.
///
/// `168 → 181`: arquivados em `docs/archive/project-schema/escada_v168_a_v181.md` (`171 → 176` nos
/// `escada_v171_a_v172.md` e `escada_v172_a_v176.md`).
///
/// # `181 → 182` — **o OBJECTO VETORIAL** (spec/06 F3 ▸ Vector, 2026-10-04)
///
/// `VecObject` passa a REGISTADO (o ECS `+1` e os dois espelhos `+1`): o contentor das formas, o
/// marcador do `ObjectKind::Vector`. ⛔ **Sem degrau de migração** — um v181 é recusado em voz alta;
/// as formas soltas que um projecto antigo traria ganham cada uma o seu objecto no 1.º quadro
/// (escolha do dono), e é essa a porta, não uma migração.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `182 → 183` — **o 3D SAI do PH2D** (`line/poda-3d`, ADR-0179, 2026-10-05)
///
/// Ordem do dono: o modelador, a escultura e o Render3d saem inteiros. Do `ProjectFile` saem os
/// campos da escultura (o documento opaco com `SCULPT_DOC_VERSION`), do modelador (o documento do
/// campo com `FIELD_DOC_VERSION`) e as formas assadas (`baked_forms`). Do registo saem os nove do
/// campo (`FieldObject` · `FieldNode` · `FieldPose` · `FieldMods` · `FieldMaterial` · `FieldVerb` ·
/// `FieldProfileSource` · `FieldLight` · `FieldTexture` — o registo do campo deixa de existir) e,
/// do ECS, `BakedForm` · `Sculpt3dPieceRef` · `Mesh3D` (ECS `-3`, espelhos `-3`). ⛔ **Sem degrau de
/// migração**, pela decisão de sempre — um v182 é recusado em voz alta; o postcard é POSICIONAL e
/// um campo a menos no meio do `ProjectFile` seria lido errado em silêncio.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `183 → 184` — **o OBJECTO VETORIAL sai: cada forma é um objecto** (spec/06 F3 ▸ Vector, 2.ª
/// volta, 2026-10-05)
///
/// Escolha do dono: sem contentor — a forma (`VecPathRef`) é o objecto. `VecObject` deixa o registo
/// (ECS `-1`, espelhos `-1`). ⛔ **Sem degrau de migração**, pela decisão de sempre — um v183 é
/// recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `184 → 185` — **os QUADROS** (`line/MiroClone`, W0, 2026-10-05; integrada sobre o `184` do
/// objecto vetorial em 2026-10-07)
///
/// Campo novo no FIM do `ProjectFile`: `boards: Vec<u8>`, o `BoardSet` com versão PRÓPRIA
/// (`ph2d_board_model::FORMAT_VERSION`) — campos novos lá dentro não voltam a subir este número.
/// ⛔ **Sem degrau de migração**: um v184 é recusado em voz alta (postcard é posicional).
/// Registos de componentes: `0` (os quadros não são entidades).
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `185 → 186` — **o PAPEL e o VIDRO do documento pintado** (`line/PainterWatercolor`, 2026-10-05 → 07;
/// integrada sobre o `185` dos quadros em 2026-10-07)
///
/// `PaintedDocument` ganha, no FIM, `papel: Option<[u8; 3]>` (pedido do dono: a cor do papel nos
/// quatro meios, invisível nas camadas) e `vidros` — o alfa de cada canal dos texels que a aguada
/// pintou, selado com o píxel (o estado da arte do vidrado: a aguada é um filtro sobre o papel de
/// cor, doc 48, BUGS #46). As migrações congeladas (v95, v128) passam a ler o documento pelo
/// `PaintedDocumentSemPapel` (o layout de antes) e nascem sem papel e sem vidro. ⛔ **Sem degrau de
/// migração**, pela decisão de sempre — um v185 é recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `186 → 187` — **o ESQUELETO é um OBJECTO** (`line/Vector`, A14, 2026-10-05)
///
/// `ph2d_skeleton_ecs::Skeleton` passa a REGISTADO (o registo do esqueleto `7 → 8`; o ECS e os dois
/// espelhos NÃO se mexem — o esqueleto regista os seus, como a física). ⛔ **Sem degrau de migração**,
/// pela decisão de sempre — um v186 é recusado em voz alta. Os ossos soltos que um projecto antigo
/// traria ganham cada raiz o seu esqueleto no 1.º quadro (`ph2d_app_skeleton::loose`, pose ao bit):
/// é essa a porta, como a das formas soltas no `182`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `187 → 188` — **o `Motion Input`** (`line/motion-value`, ciclo 19, 2026-10-10)
///
/// `ph2d_ecs::MotionInput` REGISTADO (ECS `+1`; os dois espelhos chamam o registador do ECS e não
/// se mexem): o objecto que publica uma entrada para o grafo do Motion (doc 128 §3.6). ⛔ **Sem
/// degrau de migração**, pela decisão de sempre — um v187 é recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `188 → 189` — **o `Macro`** (`line/Macros-Automação`, W4, 2026-10-13)
///
/// `ph2d_ecs::Macro` REGISTADO (ECS `+1`; os dois espelhos chamam o registador do ECS e não se
/// mexem): a gravação de um objecto (ADR-0186). O conteúdo é um blob com versão PRÓPRIA (o cabeçalho
/// do `MacroDoc`), que muda sem tocar este número. ⛔ **Sem degrau de migração**, pela decisão de
/// sempre — um v188 é recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `189 → 190` — **o tracejado em PARES e a seta por PONTA** (`line/UIUX`, F16c, 2026-10-15)
///
/// O `StrokeSpec` muda de forma (`VEC_SCENE_SCHEMA_VERSION` `22 → 23`): `dash` passa a até três pares
/// e `marker_scale` a `marker_scale_start`/`_end`. ⭐ **COM leitura do anterior, sem tipo congelado**
/// (ordem do dono: *«ficheiros antigos têm de continuar a abrir iguais»*): só o `StrokeSpec` tem forma
/// antiga, e o leitor abre o âmbito `ph2d_vec_scene::StrokeLayoutV22` para todo ficheiro `< 190`
/// (`project_load_read`); as formas guardadas dentro do Envelope e da pele são regravadas no load.
/// Gate sobre BYTES gravados pelo código v189: `project_traco_v189_tests`. Registos de componentes: `0`.
///
/// # `190 → 191` — **o script mora no PROJETO** (`line/CodeEditor`, T7, ADR-0188 §2, 2026-10-15)
///
/// Campo novo no FIM do `ProjectFile`: `scripts: Vec<u8>`, a `ScriptLibrary` (`ScriptId → { nome
/// único, texto }`) no blob `PH2DSCRIPTS` com versão PRÓPRIA. O blob do `LuauScript` passa a v3
/// (`source: ScriptSource { None, Library(id), Unimported(caminho) }`, era o caminho). ⭐ **COM
/// migração** (decisão do dono: o projeto antigo importa o ficheiro, sem perda): os braços `190` e
/// `189` (o último de forma antiga do traço) leem o ficheiro com o tipo vivo acrescentando o `0` do `scripts` vazio (o campo novo é o ÚLTIMO e o
/// postcard é posicional), passa cada blob v2 do `LuauScript` a v3, e o load importa cada caminho
/// legível (absoluto · pasta do projeto · directório corrente); o ilegível fica `Unimported` e o
/// cartão recusa-o. Registos de componentes: `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `191 → 192` — **a opacidade do objeto é canal de ESTADO** (`line/UIUX`, F22b, 2026-10-19)
///
/// O `ObjectPose.opacity` já viajava no ficheiro, mas o `capture` gravava sempre `1` e o `install`
/// não a escrevia: o Show nunca mexia na opacidade. Desde aqui grava e escreve (dono: *«passe a
/// animar a opacidade»*). ⭐ **COM migração, sem forma nova:** os bytes lêem-se com o tipo vivo, e
/// todo ficheiro `<= 191` passa cada pose gravada à opacidade que a forma tem no documento
/// (`ph2d_app_vec::ui_state_edit::opacity_from_scene`) — o Show desses projetos continua a não a
/// mexer. Registos de componentes: `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `192 → 193` — **as marcas do artista** (`line/Macros-Automação`, W9, 2026-10-17)
///
/// Campo novo no FIM do `ProjectFile`: `marcas_do_artista: Vec<u8>` — por imagem com macro, onde o
/// artista escreveu depois da gravação (`docs/Macros/07_plano_W9.md`), blob com versão PRÓPRIA (o
/// formato de `PalcosDoPainter::marcas_a_gravar`). ⭐ **COM leitura dos anteriores:** o campo novo é o
/// ÚLTIMO e o postcard é posicional — os braços `192`/`191`, `190` e `189` acrescentam o `0` do `Vec` vazio
/// (um projecto antigo abre sem marcas: o clipe toca por cima de tudo, como na W8). Gate:
/// `project_marcas_tests`. Registos de componentes: `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `193 → 194` — **as animações do personagem** (`line/GameEngineFuncional`, N1, 2026-10-25)
///
/// UM componente registado novo: `ph2d::physics::CharacterAnimations` (que animação toca em cada
/// estado do `PlatformPlayer`, `docs/GameEngineFuncional/05_N1_plano.md` §3). ⭐ **COM leitura do
/// anterior:** a forma do `ProjectFile` não muda, e um `193` não tem o componente — lê-se com o tipo
/// vivo, no braço do degrau actual. Registos: física `+1`, espelhos `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `194 → 195` — **o Draw Mode Tiled** (`line/GameEngineFuncional`, plano 14, 2026-10-26)
///
/// UM componente registado novo: `ph2d::ecs::TiledDraw` (a sprite enche o seu tamanho com ladrilhos da
/// imagem, `docs/GameEngineFuncional/14_tiled_com_tamanho_plano.md`). ⭐ **COM leitura dos anteriores:** a
/// forma do `ProjectFile` não muda; um `194`/`193` lê-se com o tipo vivo. Registos: ecs `+1`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `195 → 196` — **cada uso da imagem guarda a sua configuração** (`line/GameEngineFuncional`, plano 15, 2026-10-27)
///
/// UM componente registado novo: `ph2d::ecs::ImageUseMemory` (o que Still · Repeated · Animated tinham quando o
/// artista os deixou). ⭐ **COM leitura dos anteriores:** a forma do `ProjectFile` não muda; um `195`/`194`/`193`
/// lê-se com o tipo vivo. Registos: ecs `+1`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `196 → 197` — **o uso Region** (`line/GameEngineFuncional`, plano 16, 2026-10-27)
///
/// UM componente registado novo: `ph2d::ecs::ImageRegion` (o marcador do uso e a grelha de snap; a área é o
/// `SpriteRegion`). O `ImageUseMemory` ganha o lugar `region` no FIM e lê a forma de 3 lugares (o seu
/// `Deserialize`). ⭐ **COM leitura dos anteriores:** um `196`…`193` lê-se com o tipo vivo. Registos: ecs `+1`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `197 → 198` — **o formato do jogo** (`line/GameEngineFuncional`, plano 17, 2026-10-27)
///
/// Campo novo no FIM do `ProjectFile`: `jogo: Vec<u8>` — a resolução que o jogo mostra e quando a moldura da
/// câmera se vê (`crate::project_settings::collect_jogo`), blob com versão PRÓPRIA. ⭐ **COM leitura dos
/// anteriores:** os braços `197`…`189` acrescentam o `0` do `Vec` vazio (= o formato de fábrica, 16:9). Gate:
/// `project_jogo_tests`. Registos de componentes: `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `198 → 199` — **a pilha de CONSTRAINTS** (`line/BlenderConstraints`, W1, ADR-0192, 2026-10-25)
///
/// `Constraints` REGISTADO (`ph2d-constraints-ecs`, ECS e espelhos `0`): um binário v198 descartaria a
/// pilha. A forma do `ProjectFile` não muda: o `198` lê-se com o tipo vivo. A tripla NÃO vê este degrau.
///
/// # `199 → 200` — **o id estável por VÉRTICE** (`line/BlenderConstraints`, W4.2, 2026-10-26)
///
/// `VecVertex.id: VertId` e `VecPath.next_vert` no FIM (`VEC_SCENE_SCHEMA_VERSION` `23 → 24`): o endereço
/// das regras de CAD e do Copy Point. ⭐ **COM leitura dos anteriores, sem tipo congelado:** o leitor
/// abre o âmbito `ph2d_vec_scene::WireLayout` com a versão do fio do ficheiro (`fio_do_projeto`: `22`
/// até ao `189`, `23` até ao `199`) e um path antigo abre numerado `1..=n`; as formas guardadas dentro
/// do Envelope e da pele regravam-se no load (`migra_fio`). Gate sobre BYTES gravados pelo código v23:
/// `vec_v23_fixture` (cena) e `a_v23_*_source_is_rewritten_with_ids` (blobs). Registos de componentes:
/// `0`. ⚠️ **A tripla VÊ este degrau** — a forma da `VecScene` muda.
///
/// # `200 → 201` — **a COLA entre objectos** (`line/BlenderConstraints`, W5, ADR-0193, 2026-10-27)
///
/// `Colas` REGISTADO (`ph2d-constraints-ecs`): um binário v200 descartaria as colas. A forma do
/// `ProjectFile` não muda: o `200` lê-se com o tipo vivo. A tripla NÃO vê este degrau.
///
/// # `201 → 202` — **o arranjo do HUD** (W5.5, ADR-0194, 27/10): `UiArranjo` REGISTADO, como a cola.
///
/// # `202 → 203` — **o «Show notice» por timer** (W6, 28/10): `TimersQuiet` REGISTADO, como a cola.
///
/// # `203 → 204` — **a mesa de som é documento** (`line/UIUX`, F30 §B2, 2026-10-25)
///
/// Campo novo no FIM do `ProjectFile`: `mesa: Vec<u8>` — `ph2d_audio::Mesa::to_bytes`, blob com versão
/// PRÓPRIA (`MESA_VERSAO`; vazio = a de fábrica), o AUTORADO da mesa (a curva da Cena não entra).
/// ⭐ **COM leitura dos anteriores:** o v203 acrescenta o `0` do `Vec` vazio, e os braços `202`..`189`
/// mais um (um projecto antigo abre com a mesa no padrão, que é o que a mesa era em toda sessão).
/// Gate: `project_mesa_tests`. Registos de componentes: `0`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `204 → 207` — **a identidade dos itens** (`line/UIUX`, F30 §A1 · F31 §B · F32, 2026-10-25)
///
/// `205`: `item: ItemId` no fim de cada item das listas de nove componentes. `206`: `VecFilter::ids`
/// e o `id` de cada entrada da pilha do Vetor (`VecScene` `24 → 25`, a tripla VÊ-o; lê-se no âmbito
/// `PilhaLayoutV23`; o `24` é o id por vértice do `199 → 200`). `207`: `Tween::relogio`,
/// `PathFollow::relogio_item` e `ObjectPose::filter_ids` (as poses `<= 206` no âmbito
/// `ph2d_ui_state::PoseLayoutV196`). ⭐ **COM migração:** a forma do `ProjectFile` não muda; os blobs
/// passam pelas formas congeladas (`ph2d_ecs::item_id::migra`, um componente pode subir vários degraus)
/// e o passe dá os ids. Desenho e porquê: `docs/UI_New_and_Simple/F30_plano.md` §A1, `F31_plano.md` §1,
/// `F32_plano.md` §1. Gates: `project_itens_tests`. Registos de componentes: `0`.
///
/// # `207 → 208` — **os Kanbans** (`line/TrelloClone`, W1, ADR-0195, 2026-10-27)
///
/// `kanbans: Vec<u8>` no FIM do `ProjectFile` (formato em `ph2d_app_kanban::project`); do `204` ao `207`
/// lê-se com um `0`, os anteriores com mais (ver `falta_no_fim`). Gate: `project_marcas_tests`. Registos de
/// componentes: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `208 → 209` — **os Pixel Sprites** (`line/PixelArt`, P0, ADR-0196, 2026-10-25; reposto sobre o `208` do main)
///
/// Campo novo no FIM do `ProjectFile`: `pixel_sprites: Vec<u8>` (depois de `kanbans`) — os documentos de
/// pixels dos objectos Pixel Sprite, blob `PH2DPIXEL` com versão PRÓPRIA (`ph2d_pixel_core::codec`). ⭐ **COM
/// leitura dos anteriores:** o `falta_no_fim` acrescenta o `0` do `Vec` vazio a todo degrau `<= 208` (um
/// projecto antigo abre sem Pixel Sprites), e os braços `192`/`191` e `190`/`189` levam cinco. Gate:
/// `project_pixel_tests`. Registos de componentes: ecs `+1` (`PixelSprite`), espelhos `+1`.
///
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `209 → 210` — **o Follow Value sobre uma CAIXA** (`line/UIUX`, F41, 2026-10-30)
///
/// O `TimelineDoc` v25 pôs o canal de uma caixa (`View::Toggle`) em `0` = desligada, `1` = ligada; as
/// faixas que um Follow Value (`ph2d_constraints::Valor`) guarda sobre o canal de uma caixa estavam na
/// convenção antiga. ⭐ **COM migração de CONTEÚDO:** a forma do `ProjectFile` não muda; num `<= 209` cada
/// componente com conversão registada (`ComponentRegistry::register_caixas_v209`, hoje só `Constraints`)
/// passa pela lei `Valor::desvira_caixas`. Gate sobre bytes gravados pelo código de antes:
/// `ph2d-app-constraints` `tests/it/valor_caixa_v209.rs`; o braço: `project_caixas_tests`. Registos de
/// componentes: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `210 → 211` — **o tendão guarda o comprimento do bind** (`line/UIUX`, F41, 2026-10-30)
///
/// O `Length` animado empurra o osso seguinte e ESTICA o desenho preso (dono 30/10, como o
/// Blender/Spine): `ph2d_skeleton_ecs::Tendon` ganhou `comprimento: f64` no FIM (o do osso ao
/// prender). ⭐ **COM migração de FORMA:** a `SkinBind` gravada num `<= 210` lê-se pela forma
/// congelada `skin_bind_v210::SkinBindV210` (degrau registado por `register_skeleton_components` →
/// `ph2d_ecs::item_id::regista_degrau`; o mesmo serve as macros, `migra_blob`) com `comprimento = 0` =
/// não estica — o ficheiro antigo continua igual. Gate sobre bytes montados campo a campo:
/// `ph2d-skeleton-live` `skin_bind_v210_tests`. Registos de componentes: `0`. ⚠️ **A tripla NÃO vê
/// este degrau.**
///
/// # `211 → 212` — **a tinta de base escondida pelo olho** (`line/UIUX`, F44b, 2026-11-01)
///
/// `VecPath::fill_hidden`/`stroke_hidden` no FIM (`VEC_SCENE_SCHEMA_VERSION` `25 → 26`): o olho do
/// Fill/Stroke de base guarda a tinta fora do `fill`/`stroke` ([`ph2d_vec_scene::PaintSlot`]). ⭐ **COM
/// leitura dos anteriores, sem tipo congelado:** o `fio_do_projeto` passa a dizer `24` até ao `205` e
/// `25` até ao `211` (os dois `None`); as formas do Envelope e da pele regravam-se no load
/// (`migra_fio`). Gate sobre bytes gravados pelo código v25: `vec_v25_fixture` (cena). Registos de
/// componentes: `0`. ⚠️ **A tripla VÊ este degrau** — a forma da `VecScene` muda.
///
/// # `212 → 213` — **a tinta de base tem LUGAR na pilha** (`line/UIUX`, F45, 2026-11-01)
///
/// `VecPath::base_order` no FIM (`VEC_SCENE_SCHEMA_VERSION` `26 → 27`): o Fill e o Stroke de base
/// ocupam uma posição entre as camadas — um preenchimento cobre um traço e vice-versa
/// ([`ph2d_vec_scene::BaseOrder`]). ⭐ **COM leitura dos anteriores:** o `fio_do_projeto` diz `26` até ao
/// `212` e a base nasce no lugar de antes (a mesma aparência). Gate sobre bytes gravados pelo código
/// v26: `vec_v26_fixture` (cena). Registos de componentes: `0`. ⚠️ **A tripla VÊ este degrau.**
///
/// # `213 → 214` — **o traço por cima dos Filtros** (`line/UIUX`, F47, 2026-11-02)
///
/// `StrokeSpec::above_effects` no FIM (`VEC_SCENE_SCHEMA_VERSION` `27 → 28`), na base e em cada camada
/// Stroke da pilha. ⭐ **COM leitura dos anteriores:** o `fio_do_projeto` diz `27` até ao `213` e todo
/// traço abre com `false` (a aparência de antes). Gate sobre bytes gravados pelo código v27:
/// `vec_v27_fixture` (cena). Registos de componentes: `0`. ⚠️ **A tripla VÊ este degrau.**
///
/// # `214 → 215` — **o som é asset do PROJETO** (`line/GameEngineFuncional`, GEF 21 E2, 2026-11-02)
///
/// Campo novo no FIM do `ProjectFile`: `sounds: Vec<u8>` — a `ph2d_ecs::ProjectSounds` (nome → bytes do
/// ficheiro), blob `PH2DSONS` com versão PRÓPRIA (vazio = sem sons); o `AudioSource2D.sound` passa a ser o
/// NOME de um deles (a forma do componente não muda). ⭐ **COM leitura dos anteriores:** o `falta_no_fim`
/// acrescenta o `0` a todo `<= 214` e os braços `192`/`191` e `190`/`189` levam seis; o load importa cada
/// `sound` que se lê como caminho (`ph2d_app_components::sound_project::open`). Gate:
/// `project_sounds_tests`. Registos de componentes: `0`. ⚠️ **A tripla NÃO vê este degrau.**
/// (Escrito `213 → 214` na linha; integração `2026-11-02c`: a `line/UIUX` ficou com o `214`.)
///
/// # `215 → 216` — **os estilos de cor** (`line/NovaUI-Inspector`, CartoesUI D28/D30, 2026-11-03)
///
/// A `VecScene` `28 → 29` (`estilos` no fim). ⭐ **COM leitura dos anteriores:** `fio_do_projeto` diz `28`
/// até ao `215` e a cena abre sem estilos. Gate: `vec_v28_fixture`. Registos: `0`. ⚠️ **A tripla VÊ.**
///
/// # `216 → 217` — **as receitas de som** (`line/SfxLab`, W2, 2026-11-05)
///
/// `sfx_lab: Vec<u8>` no FIM do `ProjectFile`: a `SfxLibrary`, blob `PH2DSFXL` com versão PRÓPRIA. ⭐ **COM leitura dos
/// anteriores:** `falta_no_fim` e os quatro braços antigos levam mais um `0`. Gate: `project_sfx_tests`. Registos: `0`.
///
/// # `217 → 218` — **os nomes das camadas de colisão** (`line/GameEngineFuncional02`, ficha f1, onda 2, 2026-11-07)
///
/// `physics_layers: Vec<u8>` no FIM do `ProjectFile`: a `LayerNames`, blob `PH2DLAYN` com versão PRÓPRIA (vazio = sem
/// nomes). ⭐ **COM leitura dos anteriores:** `falta_no_fim` e os braços antigos levam mais um `0`. O número da camada
/// continua a identidade. Gate: `project_layer_names_tests`. Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `218 → 219` — **32 camadas de colisão** (`line/GameEngineFuncional02`, ficha f1, onda 5, 2026-11-07)
///
/// `PhysicsSettings.layer_matrix` passa de `[u8; 8]` a `[u32; 32]`, no MEIO do fio; os `<= 218` leem-se sob `LayerRowsV218`
/// (`LayerMatrix::rows_from_legacy`). `LayerNames` v2 (32 nomes + mostradas); os `<= 218` mostram as oito. Gate: `project_layer_names_tests`.
///
/// # `219 → 220` — **a forma de colisão é um objeto filho** (`line/GameEngineFuncional02`, B2, 2026-11-08)
///
/// Migração de CONTEÚDO, a forma do ficheiro é a mesma: num `<= 219` cada corpo ganha a filha «Collision Shape» com o
/// `Collider` e o lado-forma (`ph2d_physics_ecs::migra_formas_no_retrato`) e as trilhas dela seguem-na. Gate:
/// `project_formas_tests`. Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `220 → 221` — **a exceção de camadas da forma** (`line/GameEngineFuncional02`, B9, 2026-11-08)
///
/// Componente NOVO registado, `ph2d::physics::CollisionOverride { include, exclude }`, no lado-forma; o fio
/// dos existentes é o mesmo, um `<= 220` abre como está (sem o componente = a linha da camada). Gate:
/// `um_v220_abre_sem_excecao_de_camadas`. Registos: `+1`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `221 → 222` — **o início e o aviso viram campos** (`line/GameEngineFuncional`, dono 07/11, §0.11)
///
/// `SpriteAnimator.start: Option<String>` e `SpriteAnimations.1: Option<[u32; 2]>` (o aviso do corte velho) no FIM;
/// os componentes `SpriteAnimStart` e `SpriteCutNotice` saem do registo (`120 → 118`). SEM leitura dos anteriores.
/// Registos de componentes: `-2`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `222 → 223` — **a cor da animação** (`line/GameEngineFuncional`, editor de animações, onda 1)
///
/// `AnimationTag.color: u8` no FIM: o índice da paleta do editor, escolhido ao inserir. SEM leitura dos anteriores
/// (§0.11). ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `223 → 224` — **as várias folhas do Animated Sprite** (`line/GameEngineFuncional`, plano GEF 26)
///
/// `SpriteAnimFrames` guarda `FrameRef` (folha + célula, ou vazio) em vez de `u32`; `SpriteGrid.sheet: u8` e
/// `AnimationTag.sheet: u8` no FIM; o componente `SpriteSheets` entra no registo (`118 → 119`). SEM leitura dos
/// anteriores (§0.11). ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `224 → 225` — **as camadas com tinta do Wet** (`line/PainterWatercolor`, 2026-11-08)
///
/// `PaintedDocument` ganha, no FIM, `tintas_molhadas` (dono 08/11: no Digital o relevo do papel vê-se na
/// tinta; só a do Wet, que já traz o papel nos píxeis, o esconde). SEM leitura dos anteriores no
/// arquivo (§0.11, a GEF a recusa); o documento da macro sobe `1 → 2` e lê o `1`. Gate:
/// `project_tintas_molhadas_tests`. Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `225 → 226` — **o acerto de cada quadro** (`line/GameEngineFuncional`, GEF 28 Align, onda A)
///
/// `ExtraSheet.offsets: CellOffsets` no FIM; o componente `CellOffsets` (o da folha `0`) entra no registo. SEM leitura
/// dos anteriores (§0.11). `SpriteAnimFrames` `Some(vec![])` passa a querer dizer «não toca nada» (guarda-se, não se
/// apaga), sem mudar o formato. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `226 → 227` — **a forma «Outline»** (`line/GameEngineFuncional02`, W2b, 2026-11-09)
///
/// `ColliderShape::Outline { half_x, half_y }` ANEXADA e o componente NOVO registado
/// `ph2d::physics::ColliderOutline { rings, picture_px, detail_q4, alpha_cutoff, min_island_px }` no
/// lado-forma. SEM leitura dos anteriores (§0.11). Registos: `+1`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `227 → 228` — **as passadas do Aseprite no ping-pong** (`line/PixelArt`, P14a-3)
///
/// `AnimationTag.acaba_na_ponta_de_chegada: bool` no FIM: `repeat` N do Aseprite conta passadas, o motor ciclos; N
/// ímpar ⇒ o último ciclo é meio (`ph2d_ecs::tag_do_aseprite`). SEM leitura dos anteriores (§0.11). Registos: `0`.
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `228 → 229` — **a forma que segue a slice** (`line/PixelArt`, P14b-1)
///
/// O componente `ph2d::pixel::SegueSlice { slice: u32 }` entra no registo (`+1`): um `ComponentBlob` de `type_id`
/// desconhecido recusa o load inteiro num binário anterior, e o degrau diz «outra versão». O `Collider` de quem o
/// tem grava a forma e o offset no ponto neutro (derivados da slice). O blob Pixel sobe `MINOR 6 → 7`
/// (`Sprite.pivot_slice`; versão própria, campo desconhecido saltado). SEM leitura dos anteriores (§0.11).
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `229 → 230` — **o normal map do Sprite do motor** (`line/PixelArt`, P14 Sprite com normal, 2026-11-09)
///
/// `LibraryDoc.normals: Vec<([u8; 32], [u8; 32])>` no FIM: a normal é da IMAGEM (`cor → normal`, por conteúdo;
/// `ph2d_app_sprite_anim::normal_map`), no `ProjectState` (undo e ficheiro); os píxeis da normal são os de qualquer
/// imagem (átlas: `SavedAsset`; fora dele: os pixels de sprite). SEM leitura dos anteriores (§0.11). Gates:
/// `the_library_travels_in_the_file`, `a_normal_da_textura_individual_vai_e_volta_pelo_projeto`. Registos: `0`.
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `230 → 231` — **o pivô e as formas por célula** (`line/PixelArt`, P14e D1, 2026-11-09)
///
/// Dois componentes no registo (`+2`): `ph2d::ecs::SpriteCells` (pivô e retângulos de colisão por célula do Sprite
/// do motor, o padrão da Unity; o da folha é o padrão) e `ph2d::pixel::SegueCelula` (a filha que segue a célula;
/// `shape`/`offset` do `Collider` derivados, como o `SegueSlice`). SEM leitura dos anteriores (§0.11). Gates:
/// `ph2d-pixel-cena inspector_abre::`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `231 → 232` — **«Opacity» numa Sprite é o `Sprite::opacity`** (`line/TimelinePrincipal`, doc 17 onda 2, 2026-11-10)
///
/// O sentido gravado muda: a curva `PropKind::Opacity` de uma sprite conduz `Sprite::opacity` (era o `tint[3]`); o
/// canal OBJETO `11` sai (não se reusa) e o grupo OBJETO do tint leva o alfa (`TINTA + 3`). SEM leitura dos
/// anteriores (§0.11). Gates: `cada_campo_novo_mostra_o_seu_grupo`, `a_swatch_do_tint_grava_a_cor_e_o_alfa`, `cada_campo_le_o_que_escreveu`. Registos: `0`.
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `232 → 233` — **a caixa de cada habilidade do Platform Player** (`line/GameEngineFuncional`, 2026-11-10)
///
/// `PlatformPlayer` ganha sete `bool` no FIM (`walls_on`, `dash_on`, `crouch_on`, `swim_on`, `ledge_on`,
/// `glide_on`, `fall_limit_on`): desligada, a lei recebe a habilidade inerte e os números ficam guardados (antes
/// `0` desligava). Ficha `docs/GameEngineFuncional/componentes/03_character.md`. SEM leitura dos anteriores (§0.11).
/// Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `233 → 234` — **a curva do pulo no canvas** (`line/GameEngineFuncional`, 2026-11-10)
///
/// `PlatformPlayer` ganha `show_jump_arc: bool` no FIM (a caixa «Show Jump Arc»; só o editor a lê). Ficha
/// `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`. SEM leitura dos anteriores (§0.11). Registos: `0`.
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `234 → 235` — **o Input Map é documento** (`line/InputMap`, R2c, 2026-11-10)
///
/// `InputMap.controls: PlayerControls` (a tela do jogador) no FIM do mapa; o mapa sai do `ProjectFile` e entra
/// no `ProjectState` depois de `tags` (undo e ficheiro, um só dono); `game_id: u64` no FIM do `ProjectFile`
/// (sorteado quando o projecto nasce). SEM leitura dos anteriores (§0.11). Gates: `input_map_tests::`,
/// `project_file_round_trips_through_postcard`. Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `235 → 236` — **o Look da forma de colisão** (`line/GameEngineFuncional02`, onda 8, 2026-11-10)
///
/// O componente `ph2d::physics::ShapeLook { pivot, tint, opacity }` entra no registo (`+1`) no lado-forma: o pivô da
/// forma em fração da meia-extensão (a forma cresce em volta dele) e a cor do overlay. SEM leitura dos anteriores
/// (§0.11). Gates: `ph2d-physics-ecs forma_pivot::`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `236 → 237` — **o traço da forma de colisão** (`line/GameEngineFuncional02`, onda 8.6, 2026-11-10)
///
/// `ShapeLook` ganha `largura`, `tracejado`, `traco: [traço, vão]` (px de tela) e `traco_proprio` (dono 10/11: Dash e
/// a largura do stroke no Look); sem `traco_proprio` vale o traço do tipo do corpo. Registos: `0`. SEM leitura dos
/// anteriores (§0.11). Gates: `ph2d-physics-ecs shape_look::`, `ph2d-panel-physics o_traco_proprio`. ⚠️ **A tripla NÃO
/// vê este degrau.**
///
/// # `237 → 238` — **a curva Custom com pontos** (`line/TimelinePrincipal`, acréscimo 13, 2026-11-13)
///
/// `ph2d_anim::Interp` ganha a variante `Spline(EaseSpline)` no FIM (pontos criados/apagados no editor do cartão
/// da chave; tecto `SPLINE_MAX_POINTS = 8`). SEM leitura dos anteriores (§0.11). Gates: `ph2d-anim curve_spline::`,
/// `ph2d-panel-timeline cartao_bezier::`. Registos: `0`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `238 → 239` — **os tipos de alça da curva Custom** (`line/TimelinePrincipal`, acréscimo 14, 2026-11-13)
///
/// `ph2d_anim::EaseSpline` ganha `kinds: u16` (o `HandleKind` Corner·Smooth·Symmetric de cada ponto, 2 bits por
/// ponto, os do Vector). SEM leitura dos anteriores (§0.11). Gates: `ph2d-anim curve_spline::`. Registos: `0`.
/// ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `239 → 240` — **a montagem de juntas** (`line/JointEditor`, W1, 2026-11-12)
///
/// O componente `ph2d::physics::JointAssembly { template, parts, linked }` entra no registo (`+1`): a montagem
/// é um objeto («Car 1») que diz de que gabarito veio e que objeto ocupa cada papel (ADR-0201). `PhysicsJoint`
/// ganha `assembly: u64`, `role: u16` e `one_chain: bool` (o Pin resolvido como uma corrente, W2) no FIM. SEM leitura dos anteriores (§0.11). Gates: `ph2d-app-physics
/// joint_assembly::`, `ph2d-joint-templates the_template_ids_are_pinned`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `240 → 241` — **os enums do pincel pelo NOME** (`line/PainterWatercolor`, B50.1, 2026-11-10)
///
/// O JSON do pincel (preset e trilha da macro) grava `pixel`, `hose.mode`, `*.lei.mode`, `dual.modo` e
/// `pen_tilt.rotation` pelo nome da variante (antes o índice); o preset sobe `ph2d_brush_presets::SCHEMA 1 → 2`.
/// SEM leitura dos anteriores (§0.11). Gate: `os_enums_do_preset_gravam_o_nome`. ⚠️ **A tripla NÃO vê este degrau.**
///
/// # `241 → 242` — **o fator de aparência do documento pintado** (`line/PainterWatercolor`, B34.1, 2026-11-11)
///
/// `PaintedDocument.fator_aparencia: f32` no FIM (lado curto ÷ 1080, nunca < 1; nasce com o documento e viaja com
/// ele): a base de «a aparência acompanha o tamanho da tela» (decisão 6 do dono). SEM leitura dos anteriores
/// (§0.11). Gate: `o_fator_nasce_do_tamanho_e_viaja_com_o_documento`. ⚠️ **A tripla NÃO vê este degrau.**
pub(crate) const PROJECT_SCHEMA: u32 = 242;
