# HANDOFF DE INTEGRAÇÃO — `line/Vector` · A PELE DE IMAGEM ENTRA NO PASSE DE SPRITES

> **2026-09-13** · DIRETRIZ §1.5.9. A linha fecha aqui e **PARA** — não integra, não pusha
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Continuação directa do
> [handoff do ATLAS](HANDOFF_INTEGRACAO_line_Vector_O_ATLAS_E_O_QUADRO_2026-09-13.md), cujo §8 deixou
> aberto *«as costuras»* e *«a ordem de profundidade da pele»* — os dois eram o mesmo defeito, e o
> diagnóstico que o nomeia é a [F6-h da fila](../01_a_fila.md).

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| merge-base com `main` | `1d43da737` (o `main` enviado, CI verde) |
| commits desta jornada | **4 de código** — `335fe893a` W1 · `552b64e18` W2 · `074455744` W3 · `7353a733a` W4+W5 (mais os 11 do handoff anterior, no mesmo ramo) |
| plano | [`docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`](../03_plano_a_pele_no_passe_de_sprites.md) §5, W1–W5 — **as cinco fechadas** |
| contadores partilhados | **zero** (`PROJECT_SCHEMA` 128 · `VEC_SCENE_SCHEMA` 22 · `FLIP_SCHEMA` 13 · `DOC_VERSION` 18 · `FIELD_DOC_VERSION` 22 · os três registos 85/86/86 — todos iguais ao `main`) |
| contrato congelado (§6) | **intocado** · zero ADR · nenhum pacote externo novo |

---

## §2 — O que a linha entrega, em uma frase

**Uma imagem presa ao esqueleto deixou de ser uma camada do Vello por cima do quadro e passou a ser
uma SPRITE do quadro, desenhada como malha** — com rank, com o olho da Hierarquia, com tinta,
opacidade, mistura e recorte, sem costuras, e com o orçamento derivado do tempo do quadro.

As cinco waves:

| W | o quê |
|---|---|
| **W1** | o primitivo `SpriteMesh` no passe de sprites (`ph2d-render`), sem pipeline nova |
| **W2** | a extracção emite a sprite presa, e a malha posada é posta na instância dela |
| **W3** | os outros consumidores: quem COPIA a instância leva a malha; quem APONTA lê o que é desenhado |
| **W4** | o orçamento re-medido — `8 738` (do buffer do Vello) → **`1 543`** (do tempo do quadro) |
| **W5** | a cena que ENSINA a ordem e o olho |
| **W6** | a caixa do gizmo da sprite deixa de engolir o rig — a lei do ADR-0112 vira UMA porta |
| **W7** | **o ONION vê a pele**: a pose de mundo num instante, a pele resolvida nele, a malha por fantasma, e o escopo que um OSSO define |
| **W8** | **os dois relatos do smoke da W7**: o onion passa a falar o relógio do CLIP (só aparecia a silhueta do futuro), e a MÃO que pousa um osso passa a existir para o quadro (com a timeline aberta o osso não se transformava e o AutoKey não cunhava nada) |
| **W9** | **o gémeo do Flip da W6, fechado SEM report**: a condição da caixa de objecto passa a ser *«nenhuma ferramenta AUTORA no canvas»*, e o terceiro `if` por família morre |
| **W10** | **o pincel segue a arte DOBRADA**: a porta de canvas (`ph2d_render::mesh_uv`) e o Painter a consultá-la — as duas portas que sabiam da malha não tinham chamador de produto |
| **W11** | **o pincel PAGA a deformação**: o dab nasce como a elipse que a malha endireita, e sai redondo no ECRÃ |
| **W11b** | **a matriz nascia numa BASE MISTA** (2.º report: *«sem melhorias»*) — as duas metades tinham gate e a JUNÇÃO não, porque as fixturas das duas eram alinhadas aos eixos |
| **W12** | **a deformação debaixo de um dab mede-se AO TAMANHO DO DAB** (3.º report: *«quase bom … talvez artefato inevitável»*) — uma malha é afim por TRIÂNGULO, e perguntar num PONTO dava ao dab inteiro a deformação de um pedaço dele |
| **W12b** | **o motor do onion SAIU da shell** — a cura nomeada do tecto `the_shell_only_shrinks`, que as waves do pincel tinham deixado com UMA linha de folga |
| **W13** | **o diagnóstico completo do *«só fica redondo onde não temos deformação»*** — a régua da W12 era cega a uma marca AMASSADA; a cura foi medida (grelha amostrada) e **RECUSADA pelo dono** |
| **W14** | **o AutoKey grava a pose que a MÃO fez** — a população era a SELECÇÃO, e agarrar um osso não o selecciona: a pose feita com IK ia para a animação por um osso só |
| **W14b** | **com uma restrição VIVA o sujeito da autoria é o ALVO da âncora** — os ossos são derivados e o ledger salta-os com razão; a mão do quadro passa a trazer o alvo |
| **W14c** | **a ORDEM DO QUADRO** — a malha de uma imagem presa era construída ANTES de o solver de IK escrever a pose, e o apply repunha a curva mesmo a tempo: o gizmo mostrava a IK e a arte mostrava a curva, para sempre |
| **W15** | **os três itens do report do IK** — o painel mentia sobre os CINCO números (nunca semeados do documento), a corrente governada passou a ter uma FAIXA na tela, e o *«bend só funciona a 2»* foi medido e **refutado** |

---

## §3 — Foundational / partilhado tocado, e por quê

| ficheiro | o quê | aditivo? |
|---|---|---|
| `ph2d-render/src/sprite_mesh.rs` (NOVO na W1) | o `SpriteMesh`, a costura em tira, o `uv_at`, o `drawn_mesh`/`covers`/`uv_under`, o `LiftedInstances`, o `tag_lifted` | sim |
| `ph2d-render/src/sprite/instance.rs` | bits `8..31` do `flip_uv` = a marca da malha (só CPU) | sim |
| `ph2d-render/src/sprite_collect.rs` · `renderer.rs` · `renderer_draw.rs` · `clip_pass.rs` | a recolha marca a instância com a malha; o `DrawRun` ganha `mesh`; os TRÊS passes desenham por `sprite_mesh::draw_run`; `render_lifted_instances` novo | sim (sem malha, byte-idêntico) |
| `ph2d-render/src/picking.rs` (+ `picking_tests.rs`, corte mecânico) | todo pick, caixa, laço e UV lêem o que é DESENHADO; `scene_sprites_bbox_world` novo | **muda comportamento** de uma sprite com malha, e do *View All* (§6.5) |
| `ph2d-poly2d/src/refine.rs` | **só doc** (a tabela de custo do Vello sai) | — |
| `ph2d-skeleton-live/src/skin_image.rs` | o `attach_skin_meshes`, o `is_skinned_image`, o `ppm` na régua, o orçamento novo; **saem** `draw_skinned_images`, `SkinImageCache`, `stable_image`, `triangle_xform` | muda o produto (é a cura) |
| `ph2d-skeleton-live/Cargo.toml` | saem `ph2d-asset` e `ph2d-vector` | — |
| `ph2d-app-skeleton/src/state.rs` | sai o campo `skin_image_cache` (o `SkeletonState` fica com SETE) | — |
| `ph2d-app-vec/src/smoke_bone.rs` | a barra que ensina a ordem + as duas portas derivadas | sim |
| `shells/desktop/**` | o extract emite a sprite presa · a `fase_sim_extract` põe a malha · o overlay do Vello sai · o vidro/emissivo/*View All* pela porta nova · o `ppm` no *Bind* e no smoke | ver §6 |
| `ph2d-ecs/src/transform_inverse.rs` (W7) | `parent_world_transform_with` / `world_transform_with`: a MESMA travessia com a fonte da pose local injectada (estática, zero custo no caminho vivo) | sim |
| `ph2d-timeline/src/pose.rs` (W7) | `world_pose_at` / `_into`: o `pose_at` de cada elo, composto pela travessia acima | sim |
| `ph2d-skeleton-live` (W7) | `skin_of_in`/`skin_of_with`, `deform_field_with`, `posed_sprite_mesh`, `bone_index` público, `skinned_images_of_skeleton` | sim (a pele viva é a mesma) |
| `ph2d-render` `sprite_collect.rs` + `renderer_draw.rs` (W7) | o `extra` do passe é uma `LiftedInstances` (leva malhas), e não uma fatia crua | **muda a assinatura** de `render_with_extra`/`render_with_streams` |
| `ph2d-skeleton-demo/src/lib.rs` (W7) | `seed_arm_swing` — a acção do braço, no clip ABERTO | sim |
| `ph2d-render/src/sprite_mesh.rs` (W11) | `warp_under`: a deformação local do triângulo, adimensional | sim |
| `ph2d-painter-brush/src/canvas_warp.rs` (W11, NOVO) | `warped_dab`: `W⁻¹ · E` nos três números do motor (raio · achatamento · ângulo), sem transcendentais | sim (identidade ⇒ no-op ao bit) |
| `ph2d-tool-painter` (W11) | `PainterTool::set_canvas_warp` + a composição no `stroke_spec` (a porta do *Grid Stamp*) | **muda comportamento** só com deformação |
| `ph2d-render/src/picking.rs` (W10) | `MeshUv` + `mesh_uv`: a porta de canvas de três estados, sobre o `uv_query` que já existia | sim |
| `shells/desktop/src/input_dispatch/painter_canvas_input.rs` (W10) | o `deliver_canvas_pointer` consulta a porta antes do afim do quad | **muda comportamento** de uma sprite com malha (é a cura); `Quad` deixa o resto byte-idêntico |
| `shells/desktop/src/render_loop/snapshots.rs` (W9) | o `flip_gizmo_on` SAI de `publish`/`publish_gizmo` (o 3.º `if` por família morre) | **muda a assinatura** (shell-interna) |
| `shells/desktop/src/render_loop/fase_snapshots_publish.rs` (W9) | a condição do `object_gizmo_on` ganha a cláusula da ferramenta Flip | **é a cura** — a caixa de OUTRA família deixa de matar o traço do Flip |
| `shells/desktop/src/render_loop/timeline_onion.rs` (W8) | `collect_onion_ghosts` passa a receber `live_clip_t: Option<f64>` — `None` = o clip activo não tem instante único aqui ⇒ **zero fantasmas** | **muda a assinatura** (shell-interna) |
| `shells/desktop/src/render_loop/fase_canvas_overlays.rs` (W8) | o relógio do onion passa a ser `self.timeline_view.clip_time` (era `self.playhead.time()`) | **é a cura do 1.º relato** |
| `ph2d-panel-skeleton/src/{section,paint}.rs` (W15) | os cinco campos numéricos passam a ser SEMEADOS do documento, por UMA tabela que quem pinta e quem semeia percorrem | **é a cura** — o painel mostrava `0` em todos |
| `ph2d-app-skeleton/src/goal.rs` (W15) | `chains` NOVA — as juntas da corrente governada, em mundo, pela MESMA população do solver | sim |
| `ph2d-skeleton-render/src/goal.rs` (W15) | `draw_chains` NOVA — a faixa por baixo dos ossos governados + o X na raiz | sim |
| `shells/desktop/src/render_loop/fase_skeleton_drives.rs` (W14c, NOVO) | o osso inteligente + a âncora de IK saem da fase de CANVAS para a metade da SIMULAÇÃO, entre o apply e o extract | **é a cura** |
| `ph2d-app-skeleton/src/goal.rs` (W14b) | `target_of` NOVA — o alvo que a âncora de um osso persegue, extraída do `drag_anchor` (a mesma procura, dois consumidores) | sim |
| `shells/desktop/src/render_loop/timeline_bridge.rs` (W14b) | a `maos_do_quadro` traz também o **ALVO** de cada âncora do esqueleto segurado | **muda comportamento**: o apply deixa de escrever por cima do alvo, e o AutoKey passa a cunhá-lo |
| `shells/desktop/src/render_loop/autokey_pass.rs` (W14) | a população sai para a porta `populacao` e passa a ser **selecção ∪ mão**; `run` recebe `&SimWorld` | **é a cura** |
| `shells/desktop/src/render_loop/timeline_bridge.rs` (W8) | `maos_do_quadro` NOVA (o gizmo ∪ o esqueleto que a ferramenta Bone pousa); `run` troca `live_entity: Option<u64>` por `maos: &[u64]` | **muda a assinatura** (shell-interna) |
| `shells/desktop/src/render_loop/{fase_timeline_view,fase_timeline_drain,fase_frame_open}.rs` (W8) | o `TimelineView::dragging_entity` vira `maos: Vec<u64>` e atravessa a fase | sim |
| `shells/desktop/src/render_loop/autokey_pass.rs` (W8) | `run` ganha `skeleton: &SkeletonState`; `drag_now = gizmo.drag.is_some() \|\| skeleton.bone_pose.is_some()` | **muda comportamento**: um arrasto de osso passa a ser UM passo de undo |
| `ph2d-render/src/sprite_mesh_warp.rs` (W12, NOVO) | `warp_over`: a deformação que a malha faz **sobre o disco que o dab ocupa** (mínimos quadrados, `8` amostras), com atalho ao bit quando o dab cabe na facete | sim |
| `ph2d-render/src/sprite_mesh.rs` (W12) | a álgebra do triângulo sai para `warp_of` (UMA conta, duas entradas); ⛔ **`warp_under` APAGADA** — ficou sem chamador, e uma porta sem chamador lê-se como lei ausente | **muda a assinatura** (crate-interna) |
| `ph2d-render/src/picking.rs` (W12) | `mesh_uv` e o `uv_query` ganham `footprint_uv: [f32; 2]` — `[0, 0]` = *«não vou pintar»* (o picking e as caixas) | **muda a assinatura** (pública) |
| `ph2d-tool-painter` (W12) | `PainterTool::dab_footprint_px` — o raio do dab ANTES da composição da deformação | sim |
| `shells/desktop/src/input_dispatch/painter_canvas_input.rs` (W12) | o painter é obtido ANTES da pergunta à malha, para o footprint viajar nela | **é a cura** |
| `crates/ph2d-timeline-onion/` (W12b, CRATE NOVA) | o motor do onion (`1 014` linhas) sai da shell — ele só depende de crates irmãs; a shell fica com a CHAMADA | sim (byte-idêntico) |
| `shells/desktop/src/render_loop/snapshots.rs` (W6) | o `vec_gizmo_on` vira `object_gizmo_on` e sobe para UMA porta no topo do `build_view`: **nenhuma família** publica caixa de objecto fora do Select da ferramenta vectorial (os dois `if` por família saem) | **muda comportamento** — uma SPRITE e um GRUPO deixam de publicar caixa naqueles modos (§6.8) |

---

## §4 — Símbolos que podem COLIDIR

### Itens públicos NOVOS

- `ph2d_render::{SpriteMesh, LiftedInstances}` · `SpriteMesh::uv_at` ·
  `SpriteRenderer::render_lifted_instances` · `picking::scene_sprites_bbox_world`
- `ph2d_skeleton_live::skin_image::{attach_skin_meshes, is_skinned_image}`
- `ph2d_app_vec::smoke_bone::{painted_arm_rect, overlap_bar}`

### ⛔ Itens públicos APAGADOS

- `ph2d_skeleton_live::skin_image::{draw_skinned_images, SkinImageCache, triangle_xform}` — a shell
  re-exporta o módulo inteiro (`crate::skeleton_skin_image`), então **uma linha paralela que os use
  parte aqui**.
- `ph2d_app_skeleton::state::SkeletonState::skin_image_cache` (campo).
- `render_loop::sim_extract::skinned_image` (era `pub(super)`) — virou
  `skin_image::is_skinned_image`, e quem a chamava era a `fase_selection_mirror_skin`.

### Assinaturas MUDADAS (todas com `pixels_per_meter`)

`skin_live::bind_image` · `skin_image::{pixel_to_local, joints_in_image, deform_field}` ·
`app_vec::smoke_bone::bind`. E, por tipo: `present_frost::lift` e `sprite_emissive::collect` passam a
escrever num `ph2d_render::LiftedInstances` (os campos `App::{emissive,frost}_instances` mudam de
`Vec<RenderInstance>` para ele).

---

## §5 — Gates novos, e as provas de mutação

**W1** (`ph2d-render`): a costura `5N − 2` · o triângulo fora de alcance saltado · a volta
`local → quad_pos` · a marca na recolha e a limpeza da fatia de fora · a malha que não se desenha
deixa o quad. **GPU:** a malha de 2 triângulos em repouso É o quad (tinta, opacidade, espelho,
âncora deslocada) e ⭐ **a arte TRANSLÚCIDA numa malha `8×8` não tem costura** — `0` px de diferença,
onde o caminho do Vello punha `10 580`.

**W2** (`ph2d-skeleton-live` + shell): em repouso cada pixel é lido onde o quad o lê (4 casos:
controlo · não centrada + offset · espelho X + offset · não centrada + espelho Y) · só a instância
BASE do quad da sprite recebe a malha · o orçamento por quadro sobre a malha · `uv_at` contra o
`QUAD_STRIP` · três gates de texto da shell reescritos.
**Oito mutações, oito RED na asserção certa** (controlo `1 failed` em cada): âncora crua (`21 px` ao
lado) · sem espelho (o pixel `0` lê o `40`) · sem `Without<SlicePatchMirror>` · sem a comparação do
quad (`2` malhas) · `v` invertido no `uv_at` · tecto por imagem (`288` contra `144`) · a guarda de
volta ao extract · a chamada apagada.

**W3** (`ph2d-render`): a instância levantada leva a malha e o reuso nunca devolve a do quadro
anterior · o `collect_from` leva a malha e a alteração do `keep` · o `tag_lifted` marca e a fatia
crua limpa · a UV de repouso debaixo da malha posada · o picking onde a malha é desenhada · a caixa
da malha e o *View All* com âncora · a UV do pintor · a malha que o passe recusa aponta como o quad.
**Oito mutações, oito RED.**

**W4**: a outra ponta do tecto é uma lei de COMPILAÇÃO (`const _: () = assert!(SKIN_FRAME_PIECES >=
1_000)`) — uma fatia mais fina, ou um custo por peça maior medido outra vez, **para a build**.

**W5** (`ph2d-app-vec`): a barra atravessa o braço pintado e não o tapa, em qualquer
`pixels_per_meter`.

**W7** (4 crates + shell): a fonte injectada devolve a travessia viva **ao bit**, com o controlo de
uma fonte que mente nas DUAS pontas (`ph2d-ecs`) · doc vazio ⇒ `world_pose_at == world_transform`, e
uma key no PAI move o filho em MUNDO e **não** a pose local dele (`ph2d-timeline`) · posar a pele sem
tocar no mundo dá o que o mundo daria, e o mundo fica onde estava (`ph2d-skeleton-live`, com o
oráculo a ser o PRÓPRIO produto) · a malha do fantasma dobra a arte e **mantém a UV de repouso** · a
fatia de fora COM malha é marcada e as duas metades do quadro partilham a costura (`ph2d-render`) ·
um OSSO seleccionado ghosta a arte que ele deforma, com os dois controlos (nada animado · o osso de
OUTRO esqueleto) · e os fantasmas de instantes diferentes trazem malhas DIFERENTES (shell).
**Oito mutações, oito RED.** ⚠️ **Uma delas SOBREVIVEU à primeira** — `local_of(entity)` →
`world.get::<Transform>(entity)` — e nomeou o buraco: numa FOLHA a cadeia de ancestrais já basta para
a resposta mudar, logo o controlo tem de estar numa **RAIZ**. O gate ganhou essa metade.

**W6** (shell, `snapshots_object_gizmo_tests`): uma sprite seleccionada **não** publica caixa fora do
Select da ferramenta vectorial, e as **extras** de uma multi-selecção obedecem à mesma porta.
⚠️ **Cada metade traz o CONTROLO `object_gizmo_on = true` ao lado** — um `present` sem espelho
devolve `None` sempre, e sem o controlo o gate ficaria verde a medir o vazio. Mutação (`if false` no
lugar do guarda): **2 de 2 RED**, na asserção certa.

**W8** (shell + `ph2d-editor-core`): **sem instante de clip o onion não publica fantasma nenhum**,
com o controlo (a MESMA cena com instante ghosta) · **arch-gate
`the_onion_speaks_the_clip_clock`** — varre o `src/` inteiro da shell e exige que a chamada leve
`self.timeline_view.clip_time` e **nenhum** dos três relógios crus, com controlo de população
(exactamente UMA chamada) · **a pose que a mão pôs sobrevive ao apply**, com o controlo da mão vazia
(ali a curva TEM de escrever) · **de um osso vai o esqueleto inteiro**, com o guarda do sujeito que
não é osso e a metade que prova que **a mão do gizmo não se perdeu** · e **posar um osso abre UMA
bracket de arrasto**, com o controlo do gesto ausente.
**Cinco mutações, cinco RED:** o relógio da cena de volta · a porta a esquecer o OSSO · a porta a
esquecer o GIZMO · o autokey a esquecer a mão no osso · o onion a aceitar um instante ausente.
⚠️ **O arch-gate reprovou primeiro sobre o COMENTÁRIO que explica a cura** (ele nomeia o relógio
errado para dizer que ele saiu): *um censo textual que não separa prosa de código mente nos DOIS
sentidos*, e a varredura passou a apagar os `//` antes de aplicar a lei.
⚠️ **Ele vive na `ph2d-editor-core`, não em `shells/desktop/tests/it/` ao lado do irmão
`the_motion_path_is_offered_only_on_the_keys_tab`** — a razão está escrita no cabeçalho dele e é
MEDIDA: a shell tinha `90` linhas de folga contra o `the_shell_only_shrinks`, e um gate de 90 linhas
lá dentro estouraria a catraca que existe para ela só encolher.

⛔⛔ **E a W9 EXPIROU um gate ALHEIO que estava certo** — o `the_gizmo_is_not_published_while_the_preview_runs`
(que o §9.5 deste handoff já avisava ler a expressão à letra): ele citava a condição INTEIRA
achatada, e a cláusula do Flip entrou **no meio dela**. Curado na FORMA, não no literal: hoje a
agulha é o **termo** (`&&!self.ui_preview.is_on(),`) e exige **unicidade**. Mutação: RED.

**W9** (`ph2d-editor-core`, dois arch-gates): a condição da caixa **nomeia as duas ferramentas de
autoria**, com controlo positivo (a chamada existe) · e o **censo dos ramos que recebem o
`on_canvas`** — um quarto acorda a lei. **Duas mutações, duas RED** (apagar a cláusula do Flip ·
acrescentar um quarto ramo). ⚠️ Os dois vivem na `ph2d-editor-core` pela razão medida do
`the_onion_speaks_the_clip_clock`, e ⛔ **nenhum deles alcança a ferramenta NOVA que ninguém ligar** —
isso é dívida nomeada no §8.

**W10** (`ph2d-render` + um arch-gate): os **três estados** da porta, com o **controlo da sprite sem
malha** (sem ele, uma porta que respondesse `Use` a toda gente passaria) · e o fio (a porta de canvas
exige janela e GPU). **Quatro mutações, quatro RED** — ⚠️ **uma SOBREVIVEU à primeira**: um
`let malha = MeshUv::Quad;` ao lado de um `let _ = mesh_uv(..)` deixava o arch-gate verde sobre o
defeito inteiro. *Citar uma porta não é consultá-la* ⇒ ele exige a **ligação**, não a menção.

**W11** (3 crates + o fio): o no-op ao bit (com controlo) · o eixo comprimido a pedir o dobro do
raio · **a elipse pintada a voltar REDONDA ao ecrã** (a régua é o produto) · o triângulo colapsado
recusado · a deformação adimensional (controlo: o `posed_arm` só translada) · o `stroke_spec` a
pagá-la (controlo: a identidade) · e o fio. **Cinco mutações, cinco RED** — ⚠️ **uma sobreviveu**,
pela segunda vez no dia, porque o arch-gate exigia a MENÇÃO da porta e não a LIGAÇÃO à resposta.
⛔⛔ **E um gate apanhou uma promessa minha a ser falsa:** sem o atalho da identidade a decomposição
devolvia `1,0000006` e `0,39999998` em repouso — *«byte a byte» não é uma promessa que uma raiz
quadrada cumpra: é um `if`*.

**W11b** (`ph2d-render`): **a JUNÇÃO** — a matriz publicada é a que a malha faz, e a elipse que a
lei do pincel tira dela volta REDONDA ao ecrã. A fixtura nasce da resposta (escolhe-se a deformação
de ecrã, constrói-se o triângulo que a produz). **Duas mutações, duas RED**, a primeira sendo o
próprio erro do report. ⚠️ A crate ganhou um **dev-dep** para a lei canónica do pincel — a forma dos
dois que ela já tinha.

**W12** (`ph2d-render` + o fio): um dab que cabe numa facete recebe a facete **ao bit** (controlo: um
dab grande TEM de mover a resposta) · a marca é mais redonda ao tamanho do dab em **todas** as seis
células medidas, com margem `0,02` · **a elipse AUTORADA chega ao ecrã como o artista a desenhou** ·
as amostras fora da malha respondem pela facete que deixaram · e o fio (a DERIVAÇÃO do footprint).
**Cinco mutações, cinco RED** — ⚠️⚠️ **TRÊS sobreviveram à primeira**, e as três nomearam buracos:

1. ⛔⛔ **Um dab REDONDO não consegue medir a BASE do ajuste.** Trocar a base por um espelho
   multiplica a matriz por um factor **ORTOGONAL** — mesmos valores singulares, mesmos eixos —, logo
   com `flatten = 0` a resposta é a mesma **ao bit**. É a W11b uma volta mais fundo: lá a fixtura
   alinhada aos eixos não media a base, aqui é o **pincel** que não a mede. Com `flatten 0,4` e
   `angle 30°` o eixo maior chega a `29,8°`–`38,3°` com a lei certa e a `152,6°`–`164,2°` com a base
   trocada.
2. ⛔ **Nomear um argumento não é alimentá-lo** — `let footprint_uv = [0.0, 0.0];` com a chamada
   intacta passava no arch-gate (a mesma forma da mutação da W10: *citar uma porta não é
   consultá-la*). Ele exige agora a **derivação** do raio do pincel.
3. ⛔ **Um gate só com o raio grande não pina o raio de amostragem** — amostrar a METADE do raio
   melhora os dabs grandes na mesma, e só a coluna do raio PEQUENO a apanha (ganho **zero** ali).

**W14** (a shell): a pose que a mão fez entra INTEIRA (red-first `(2, 3)`) · um osso que a mão segura
e que **não se moveu** não cunha nada (`(2, 3, 3)`) · e a porta `populacao` com os três casos dela
(selecção ∪ mão sem repetidos · a mão do GIZMO sem estar na selecção · o conduzido por um MOTOR
saltado mesmo estando na mão) · e o **primeiro gesto de um rig NOVO**, sem curva nenhuma.
⚠️ **Essa última fixtura quase fabricou um defeito que não existe:** de UM quadro ela lê *«não grava
nada»* sobre um passe são — sem curva, quem mede a mudança é a **BASELINE**, e *um arrasto são DOIS
quadros*. **Cinco mutações, cinco RED** — ⚠️ **duas só morreram depois de a
população sair para uma PORTA**, e a quinta apanhou uma lei (*pré-visualização não é autoria*) que
vivia **sem gate** desde a auditoria de 2026-09-08.

**W14b** (a shell + a `ph2d-app-skeleton`): arrastar a âncora de IK grava **a âncora**. ⚠️ **A
fixtura quase fabricou um defeito pela SEGUNDA vez nesta wave:** com `keys_mode = false` (o default
do construtor) o gate falha **com a cura aplicada** — uma âncora de trajectória é geometria do CLIP e
o documento recusa-a fora da aba *Keys*, que é a de omissão do app. **Duas mutações, duas RED** (o
alvo fora da mão · o `target_of` a devolver o alvo errado — esta última também acordou dois gates de
costura do esqueleto, o que diz que a porta é a mesma que o hit-test usa).

**W12b** (a mudança de crate): ⛔ **o `every_member_inherits_the_workspace_lints` apanhou a armadilha
do HOWTO à primeira** — a crate nova não herdava os lints da workspace, logo o `unsafe` ficava
PERMITIDO nela em todo alvo. Prova exacta do movimento: **`15` testes antes, `15` depois**
(14 + 1 `#[ignore]`).

---

## §6 — Coisas que uma leitura rápida do diff entende ao contrário

1. **«A W2 só tirou uma guarda do extract.»** Não: a guarda saindo faz a sprite EXISTIR no quadro
   (rank, visibilidade, propriedades), e a malha é posta **depois** do extract, na instância que ele
   emitiu. É a ordem que é o contrato — antes do extract não há instância nenhuma para a receber.
2. **O `render_instances_only` continua a LIMPAR marcas, e isso é deliberado:** a fatia crua do glow
   do Motion não traz malhas, e uma marca herdada indexaria as malhas de outra chamada. Quem leva
   malhas é o irmão `render_lifted_instances`.
3. **O `SKIN_FRAME_PIECES` desceu `5,7×` (`8 738 → 1 543`) e isso NÃO é regressão:** o número velho
   media o buffer do Vello, que este caminho não gasta; a `8 738` peças o quadro custaria `9,4 ms`.
4. **O picking mudou para TODA sprite, não só para as presas** — mas sem malha o caminho é o de
   sempre, e os `13` testes antigos (movidos verbatim para `picking_tests.rs`) continuam a medi-lo.
5. **O *View All* muda de resultado em cenas sem pele nenhuma**, e é a cura de um defeito
   anterior à malha: a shell reconstruía à mão um quad centrado no PIVÔ, ignorando a âncora e a base.
6. **`sprite_world_to_uv` numa sprite presa devolve `None` fora da malha** (antes devolvia a UV do
   quad de repouso). O `_unclamped` — o que o Painter usa — continua a responder pela lei do quad
   fora dela: limite NOMEADO no §8.
7. **A `ph2d-vector` sair das dependências da `ph2d-skeleton-live` não é arrumação:** é o fim do
   número que ela sustentava. O mesmo para a `ph2d-asset`, que saiu com a cache de imagens.
8. **A W6 não «esconde o gizmo no modo Osso»: ela põe a SPRITE e o GRUPO dentro de uma lei que já
   existia** (ADR-0112, *«o gizmo de objecto só existe fora da ferramenta vectorial, ou no Select
   dela»*). Ela estava escrita **por família**, e as duas que faltavam são exactamente as que a W2
   fez nascer debaixo de um gesto de autoria. ⛔ **Nenhum gesto se perde:** naqueles modos o
   `ramo_ferramenta_vetorial` consome todo press de canvas, logo a caixa só era alcançável onde ela
   bloqueava o ramo. E a **selecção fica armada** — é isso que mantém o *Bind* com sujeito.
13. **O `GhostTarget` tem `relogios` porque a pergunta «quem MOVE isto?» vale em DOIS eixos:** o
   ESCOPO (quem é ghostado) e os INSTANTES (quando). O segundo foi achado antes do smoke — o modo
   `Keys`, que é o de **omissão**, lia as keyframes do alvo desenhado, e a imagem de um rig não tem
   nenhuma. *Curar metade de uma pergunta deixa o recurso mudo na configuração de fábrica.*
10. **A W7 não «fez os fantasmas dobrarem»: ela fez um rig TER fantasmas.** A leitura anterior era
   *«eles desenham o quad de repouso»*, que é verdade **se houver fantasma** — e num rig normal não
   havia: o onion exigia que o seleccionado estivesse animado **e** desenhasse, e numa personagem
   riggada quem leva keys são os ossos (que não desenham) e quem desenha é a imagem (que não leva
   keys). *Duas guardas que se excluem uma à outra desligam o recurso sem nunca o dizer.*
11. **O `extra` do passe mudar de `&[RenderInstance]` para `&LiftedInstances` NÃO é arrumação:** é o
   par instância+malha a viajar junto. Um vector paralelo ao lado da fatia é o padrão que o
   `corner_radius` proíbe por escrito, e o stream do Motion continua a entrar **sem malha nenhuma**.
12. **O `ghost_instance` passou a ler a pose de MUNDO, e isso fechou uma nota antiga de graça:** o
   ADR-0142 dizia *«rigs parenteados são wave futura»* porque ele lia o `pose_at` LOCAL. Para uma
   raiz as duas respostas são as mesmas — os nove gates do onion passam sem uma linha mudada.
23. **A W11b não é «um sinal trocado»: é uma BASE.** A `2×2` tinha linhas em `y`-para-cima e
   colunas em `v`-para-baixo; a conjugação pelo espelho nega os termos fora da diagonal, o que é
   invisível numa deformação DIAGONAL e espelha uma RODADA. ⛔ E as fixturas das duas metades eram
   alinhadas aos eixos — *uma fixtura alinhada aos eixos não mede uma base*.

21. **A W11 não muda o pincel de ninguém:** com a arte em repouso (ou sem malha) a `canvas_warp`
   devolve o spec do artista **ao bit**, por um `if` explícito — e é isso que mantém os goldens e a
   paridade do Painter de pé.
22. **O `dab_flatten`/`dab_angle_deg` do PAINEL não se mexem:** o `stroke_spec` devolve uma CÓPIA
   (é o que o *Grid Stamp* já fazia), logo o artista continua a ver os números dele.

19. **A W10 não «arranjou a UV fora da malha»** (que é o que a fila dizia): ela pôs o Painter a
   falar com a porta que sabe da malha. O defeito era em **TODA** a arte dobrada, não só fora dela —
   o Painter nunca chamou nenhuma das duas portas de UV, ele tem afim próprio.
20. **`MeshUv::Quad` não é um caso degenerado: é o que mantém o Painter intacto.** O afim dele
   carrega a grelha da folha, o *Repeat Image* e a margem do gizmo de deformação; uma porta que
   respondesse a toda sprite apagaria os três.

17. **A W9 não muda NADA para o objecto Flip, e isso é álgebra:** a caixa dele era
   `object_gizmo_on_antigo ∧ flip_ok` e hoje é `object_gizmo_on_novo = vec_ok ∧ flip_ok ∧ ¬preview` —
   a mesma expressão. O que muda é a caixa das **outras** famílias sob a ferramenta Flip.
18. **O Painter NÃO tem este defeito, e a saída dele não serve aqui:** ele não usa o `on_canvas` —
   tem porta própria (`chrome_hit::pointer_over_chrome`) que **isenta** os ids do gizmo. ⛔ Copiar
   essa isenção para o Flip deixaria a caixa PINTADA e sem pegar, que é a alça morta.

14. **A W8 não «acrescentou um relógio ao onion»: ela apagou o QUARTO.** O quadro já escolhia entre
   os três relógios em dois sítios (o dreno e o autokey), e a chamada do onion escrevia à mão um
   palpite que não era nenhum deles. ⇒ a cura é ler o que a vista **já publica**
   (`TimelineViewSnapshot::clip_time`), não escolher melhor.
15. **O `maos: &[u64]` no lugar do `live_entity: Option<u64>` não é generalização preventiva:**
   agarrar a PONTA de um osso faz cinemática inversa e move a corrente toda, logo uma mão com um
   elemento deixaria metade do esqueleto a brigar com o dedo — e **qual** metade depende da alça.
16. **O `drag_now` do autokey passar a somar o osso muda o UNDO, não a autoria:** sem ele o gesto
   ainda cunharia chaves (cada quadro é uma «edição discreta»), mas cada quadro abriria um passo
   próprio. *A cura do 2.º relato criava um defeito novo se esta metade ficasse de fora.*

9. **O `vec_gizmo_on` não foi só renomeado:** ele mudou de sítio (de dois `if` dentro dos ramos para
   UMA porta no topo do `build_view`) e de alcance (todas as famílias). O nome antigo mentia desde
   que o Flip entrou com o gémeo dele.

---

## §7 — As premissas que a medição derrubou

1. ⛔ **«O custo do `Smooth` é o da tabela do `refine.rs`»** (`10`–`16 %` de um quadro a `7 776`
   peças) — aquela tabela media o caminho do **Vello**. No caminho vivo uma peça entregue custa
   `1,08 µs`, e `7 776` peças custam `8,4 ms`.
2. ⛔ **«A malha só interessa a quem desenha.»** Três consumidores COPIAVAM a instância (o vidro do
   prefab, o emissivo, os fantasmas do onion) e dois liam o QUAD de repouso (o picking, o *View
   All*) — cinco sítios onde a imagem aparecia ou era apontada onde ela não está.
3. ⛔ **«Descodificar a malha por quadro é de graça»** — `0,134 µs` por peça, `13 %` do custo do
   `Fast`. É a maior parcela depois da deformação.
4. ⚠️ **«Basta esperar por uma máquina calma.»** A carga de FUNDO desta workstation é **`~7`** sem
   ninguém compilar (o editor, o `rust-analyzer`, o `sccache`, as outras sessões) — a espera por
   `load < 4` esgotou `20 min` e mediu a `8,01`. O instrumento que sobrevive é o **MÍNIMO de `N`
   corridas** (o custo quando o escalonador deu o núcleo), com a mediana ao lado a dizer quanto a
   máquina estava a roubar.
5. ⛔ **«O quinto defeito da F6-h era só a âncora.»** Era a âncora **e o espelho**: o shader espelha a
   UV do quad, logo a régua da imagem tem de espelhar a POSIÇÃO — espelhar as duas não espelha nada.
6. ⛔⛔ **«Os consumidores da instância são cinco, e a W3 fechou-os»** (§7.2) — havia um **sexto**, e
   ele não copiava nem lia: ele passou a **EXISTIR**. A caixa do gizmo pede um espelho no presente,
   logo enquanto a imagem presa não emitia instância ela não tinha caixa nenhuma, e a ausência lia-se
   como a do objecto inteiro. ⇒ *ao fazer uma coisa existir no quadro, o censo não é «quem a copia?»
   mas «quem passa a ter resposta onde antes não tinha nenhuma?»* — e a resposta nova chegou a um
   `hit_index` e matou o gesto que vivia por cima dela.

7. ⛔⛔ **«Curar o onion custa três peças»** (a redacção da W3) — custa **quatro**, e a que faltava é
   a que decide se o recurso existe: o **ESCOPO**. As três previstas (a pose de mundo em `t`, a pele
   que a aceita, a malha por fantasma) estavam certas e bastavam para *desenhar* um fantasma que
   ninguém chegava a pedir.
8. ⛔ **«O `bone_index` constrói-se por chamada, e é de propósito»** — verdade para quem tem UMA
   coisa na mão, e o doc dele já dizia que um LAÇO o partilha. O onion é `N` artes × `M` instantes,
   e o `attach_skin_meshes` (que também é um laço) herda a porta nova.

9. ⛔⛔ **«A W7 fechou o onion de um rig»** — ela fechou o MOTOR e deixou o RELÓGIO. Os nove gates
   dela passam-lhe o instante à mão, e o único sítio que o escolhe é uma fase do `render_frame` que
   nenhum deles alcança: na configuração de fábrica (aba **Keys**) o número entregue ficava parado em
   `0` e **só o futuro tinha vizinhos**. *Um gate que recebe o instante como argumento mede tudo
   menos de onde ele veio.*
10. ⛔ **«Posar um osso é um gesto do esqueleto, e a timeline não tem nada com isso.»** Tem: o apply
   escreve a pose de TODA entidade keyada, e a única lista de excepção que ele conhecia era a do
   gizmo de sprite. Medido nesta jornada com a sonda: `0,77 → 0,45` no quadro seguinte ao arrasto.
   *Um gesto novo herda os inimigos do antigo, e ninguém lhe dá a lista.*

11. ⛔⛔ **«O resíduo que sobra é inevitável — é a natureza das deformações do mesh»** (a hipótese do
   dono no 3.º report, e a minha antes de medir). **Metade errada:** a maior parte dele era a
   deformação ser lida num PONTO e aplicada a um dab que cobria vários triângulos — e isso tem cura,
   com `0,10`–`0,23` de redondeza medidos. *«Fora de escopo porque é inalcançável» é uma afirmação
   sobre um número que outra pessoa pode mudar* (§0.0). A metade certa é o que sobra depois: **uma
   elipse por dab**, que é uma troca de resolução e não uma parede.
12. ⛔⛔ **«O próximo item é a deformação POR DAB»** (o que a W11 deixou escrito na fila). Medido:
   vale `≤ 0,05` e **em sinal ambíguo**, um quarto do que o footprint compra — porque o
   `stamp_dabs_inner` já relê o `stroke_spec` ao vivo e só o RAIO fica congelado no pen-down.
   *Um item de fila escrito por quem acabou de curar a porta ao lado descreve o resíduo daquela
   porta, não o maior defeito.*
13. ⛔ **«A `warp_under` é a porta da deformação num ponto e fica»** — depois da W12 ela tinha **zero
   chamadores**, e este repo já escreveu que *uma porta sem chamador e uma lei ausente produzem o
   mesmo app*. Foi APAGADA; o que fica é a `warp_of` (a álgebra, dado o triângulo), com duas
   entradas.

---

## §8 — O que fica ABERTO

| item | estado |
|---|---|
| ✅ **Os fantasmas do onion desenhavam o quad de repouso** | **FECHADO pela W7** (a pose de mundo em `t`, a pele resolvida nele, a malha por fantasma) e **alcançável desde a W8** (o relógio do clip). ⚠️ A redacção fica aqui por contraste: ela dizia *«eles desenham o quad de repouso»* e a medição mostrou que **não havia fantasma nenhum** |
| ✅ **A UV do pintor** | **FECHADO pela W10**, e o item estava mal endereçado: o Painter não chamava nenhuma das duas portas de UV — ele tem afim próprio, do quad de repouso, logo o erro era em TODA a arte dobrada |
| ⚠️ **A deformação do pincel é a do PEN-DOWN** | ⛔⛔ **MEDIDO na W12 e vale ~zero:** com o pen-down do outro lado do braço a redondeza da marca muda `≤ 0,05`, **em sinal ambíguo**, contra os `0,10`–`0,23` que o footprint compra. A razão é que o `stamp_dabs_inner` já relê o `stroke_spec` **ao vivo**, logo só o RAIO fica congelado. Fica como **inconsistência declarada** — dois leitores do mesmo spec em instantes diferentes —, não como cura à espera. *Um item de fila escrito por quem acabou de curar a porta ao lado descreve o resíduo daquela porta, não o maior defeito.* |
| ⛔⛔ **UMA elipse por dab NÃO chega — e o dono DECIDIU deixar como está** (2026-09-14) | 4.º report: *«só fica redondo onde não temos deformação»*. Com a régua certa (o PERFIL visto do ecrã, não o `maior/menor` — que é **cego a uma marca amassada**) e a fixtura certa (leque forte + malha grossa), a lei que shipa lê `1,3`–`2,6` de fora de redondo com `8`–`28 %` de ondulação. ⭐ **A cura está MEDIDA e não construída:** a lei do dab passa a ser o mapa **AMOSTRADO** (grelha `8×8` ⇒ `1,07`–`1,14`; `24×24` ⇒ `1,02`–`1,05`), que reproduz um afim **exactamente** ⇒ zero regressão fora de arte deformada. ⛔ **O preço é que ela vive no `FootprintDeform::apply`, o choke point de TODO pincel do app** (~12 sítios de cobertura na crate mais quente do repo) ⇒ **decisão do dono, e ele decidiu: *«Deixe como está»*** — o caminho recomendado é `Deform = Smooth` + pincel médio (`~1,15`). ⚠️ **Registado como RECUSA, não como pendência.** As duas alavancas e o limite delas (um pincel muito grande fica em `2,15` em toda malha) estão na [fila W13](../01_a_fila.md). ⛔ Uma lei QUADRÁTICA foi construída, medida e REFUTADA (o mapa tem DOBRAS, não curvatura) |
| ⏳ **(a redacção anterior, por contraste)** | depois da W12 sobra `≈1,1`–`1,2` de redondeza no pior regime (pincel grande × malha grossa × leque forte), e é **inerente**: sobre um footprint em que a deformação varia, nenhum afim único a descreve. ⛔ **Não é uma parede** — os dois diminuidores medidos são a malha mais fina (`Smooth`) e o pincel menor. Dividir o dab em sub-dabs é a saída teórica e **não foi medida** |
| ⏳ **O CHROME do Painter fica no repouso** | a curva, a linha, o gizmo de deformação, os gizmos de selecção, os crachás e a humidade desenham-se em posições de IMAGEM pelo mesmo afim; numa arte dobrada ficam no sítio de repouso. ⚠️ **Não piorou com a W10** (antes a tinta estava errada com eles), e o anel do pincel segue o ponteiro — só o TAMANHO dele sai do afim |
| ⏳ **O conta-gotas do BgRemoval** | usa uma caixa alinhada aos eixos que ignora rotação **e** malha — mais antigo e mais cru que tudo isto |
| ⚠️ **9-slice e folha desdobrada** | a malha só conhece o quad da sprite; essas desenham-se SEM deformar, com aviso único no stderr |
| ⏳ **A FATIA do quadro (`1/10`)** | é a única escolha do `SKIN_FRAME_PIECES`; medir outra é `PH2D_SKIN_PIECES=<n>` |
| ⏳ **O custo do onion no EXTREMO** | medido: uma peça de fantasma vale `0,158 µs`, logo `16` fantasmas (o máximo dos dois sliders) sobre uma pele no tecto dela (`1 543` peças) são **`~3,9 ms`, `23 %` de um quadro**. ⛔ Não se corta nada — deitar fora os mais distantes é decisão de PRODUTO; o que fica é o número no `PH2D_BONE_LOG` |
| ⛔ **Uma forma VECTORIAL presa não tem fantasma** | ela é desenhada pelo Vello e não tem `RenderInstance`, e o passe que desenha fantasmas é o de sprites. Limite NOMEADO — curá-lo é um fantasma de Vello, outra máquina |
| ✅ **O gémeo do Flip da W6** | **FECHADO pela W9** (sem report, medido): a condição passou a ser *«nenhuma ferramenta autora no canvas»* e o 3.º `if` por família morreu |
| ⏳ **A condição vive no FIO** | ela é uma expressão numa fase do `render_frame` que nenhum teste de unidade alcança, e o arch-gate que a protege é **textual**: apanha a regressão e o crescimento da população, **não** a ferramenta de autoria NOVA que ninguém ligar. A cura de fundo é a ferramenta DECLARAR se autora no canvas — e isso é o `Tool`, contrato **congelado** (§6) |
| ⚠️ **DUAS caixas de sprite** | a do gizmo sai do `sheet_grid_overlay::gizmo_box(sprite, …)` (o quad) e a do `ph2d_editor_core::gizmo` sai do `ph2d_render::selection_bbox_world` (que a W3 tornou ciente da malha): numa imagem presa e DOBRADA elas discordam. Hoje só a segunda é lida (o *View All* e o contorno do realce) |
| ⚠️ **Vermelho PRÉ-EXISTENTE, não desta linha** | `ph2d-preview-drive/src/lib.rs:493` — clippy `len` sem `is_empty`. A crate é intocada por esta linha (último commit dela: `21c403c20`, 12/09) |
| ⏳ **O onion no Arrange com PILHA** | o `clip_time` responde `None` quando o clip activo toca **zero ou duas** vezes ali ⇒ nenhum fantasma. É a resposta honesta (não existe um «agora» de que o passado seja vizinho) e **não** foi smokada: uma pilha com uma strip só devolve o tempo local, que é o caso comum |
| ✅ **O AutoKey de um osso keyava só o SELECCIONADO** | **FECHADO pela W14**, e a redacção antiga estava a ser generosa: não é *«o modelo do Blender»*, é **perda de dados** — agarrar um osso **não** o selecciona, logo a pose feita com IK podia ir para a animação por um osso só, ou por **nenhum**. Medido red-first: `(2, 3)` numa corrente de dois dobrada inteira. A população passou a ser **selecção ∪ mão** (a porta `autokey_pass::populacao`), com o DIFF a impedir que isso espalhe chaves |
| ⏳ **O tecto da shell tem `1 016` linhas de folga** | a W12b moveu `1 014` (o motor do onion). A próxima candidata **não** está medida — e o tecto é a grandeza que soma entre linhas sem ninguém a contar (`CLAUDE.md` §5.0) |
| ⏳ como no handoff anterior | o mapa dobra sobre si em dobras fortes · F4 *«undo tem poucos passos»* |

---

## §9 — O que o INTEGRADOR faz

1. `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` nesta worktree (colado no
   §10): **nenhum contador partilhado se move**.
2. ⚠️ **A `ph2d-render` é de TODOS.** O `DrawRun` ganhou um campo (`mesh`) e o `clip_pass` mudou de
   assinatura: uma linha paralela que construa `DrawRun` à mão ou chame o `encode_clip_groups` parte
   aqui — e o `sprite.wgsl` **não** foi tocado.
3. ⚠️ **Os símbolos APAGADOS do §4** partem uma linha que os use (a shell re-exporta o módulo inteiro).
4. ⚠️ **O picking e o *View All* mudam de resultado** para sprites com âncora/rotação (§6.5): um
   golden de outra linha que os contenha muda de valor, e a mudança **é a cura**.
5. ⚠️ **A W6 toca o `snapshots.rs` da shell** (o parâmetro `vec_gizmo_on` → `object_gizmo_on` e uma
   porta nova no `build_view`): uma linha paralela que edite o mesmo passe funde com conflito de
   MESMO SÍMBOLO. O sítio de chamada (`fase_snapshots_publish.rs`) **não** muda — e não pode: o gate
   `the_gizmo_is_not_published_while_the_preview_runs` lê aquela expressão à letra.

---

## §10 — O portão batched, e o smoke

Régua = merge-base `1d43da737`.

| passo | resultado |
|---|---|
| `BASE=1d43da737 bash scripts/nextest-impacted.sh` | ✅ **14 693 passaram, 0 falharam** — o delta contra a W12 (`14 644`) são os gates novos das W14a–c e da W15, **mais** as 38 suítes que a W15 traz para a varredura (ela toca a `ph2d-skeleton`, que é folha de meia dezena de crates)|
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets` | ⚠️ **correu em CACHE e não repete avisos** (a saída inteira é uma linha, `Finished`). Forçado o replay da única crate com aviso: `ph2d-preview-drive`, **pré-existente** e intocada por esta linha (§8). As crates desta linha foram corridas com replay: zero |
| `cargo machete` | ✅ nenhuma dependência por usar — a `ph2d-vector` e a `ph2d-asset` SAÍRAM da `ph2d-skeleton-live` |
| `bash scripts/doc-index.sh --check` | ✅ 19 índices em dia |
| provas de mutação | ✅ **21 RED** na asserção certa (8 na W2, 8 na W3, **5 na W8**), cada uma com o controlo `1 failed` — um filtro que casasse zero leria `0 passed; 0 failed`. ⚠️ A 1.ª redacção da 5.ª (o onion a aceitar um instante ausente) **não compilava**, e um erro de compilação não é um gate vermelho: foi reescrita numa forma que compila. **W9: mais 3** (a cláusula do Flip · o quarto ramo no canvas · o termo da preview). **W10: mais 4** e **W11: mais 5**, e ⚠️ **uma sobreviveu à primeira em cada uma** — as duas pelo mesmo motivo: citar a porta em vez de a ligar |
| tectos de LOC | ✅ ⚠️ **A W8 reprovou DOIS ficheiros de teste da shell** (`autokey_pass_tests` `612` · `timeline_onion_tests` `613`, tecto `600`): curados por **corte por responsabilidade** — o gate do osso saiu para o `autokey_bone_tests.rs` e o do relógio para um **sub-módulo** que herda as fixturas (`timeline_onion_clock_tests.rs`, o molde do `skin_at_time_tests` da W7). ⛔ Nunca subir o número. E o resto: ⚠️ **O `app_state.rs` bateu no tecto** (`1 023 / 1 019`) e a cura foi CORTE da prosa que eu tinha acrescentado — nunca subir o número |

⭐⭐⭐ **O NÚMERO QUE O INTEGRADOR TEM DE VER, e ele MUDOU:** a W12 levou a shell a **`196 989`** de
`196 990` — **UMA** linha de folga —, e a **W12b fez a cura que este parágrafo prescrevia**: o motor
do onion (`timeline_onion.rs` + os dois ficheiros de teste, **`1 014`** linhas) saiu para
[`crates/ph2d-timeline-onion`](../../../crates/ph2d-timeline-onion/), porque ele não é composição —
nada nele pergunta pela `App`. A shell fecha esta linha em **`196 500`**, com **`490`** de folga.
⚠️ **Prova exacta do movimento: `15` testes antes, `15` depois** (14 + 1 `#[ignore]`), contados nos
dois lados — *um `git mv` que perde um teste fica VERDE em `check`, `clippy` e nas três suítes*.
⛔ E continua a valer: a cura de um vermelho é corte por responsabilidade, nunca subir o número.
⚠️ **E é por isso que o arch-gate `the_onion_speaks_the_clip_clock` vive na `ph2d-editor-core`** e não
ao lado do irmão em `shells/desktop/tests/it/`: a razão está escrita no cabeçalho dele.

⚠️ **O `collision-surface.sh` marca `✗` em dois ficheiros da shell** (`app_state.rs` `1019/600`,
`main.rs` `1118/600`): são as folgas NUMERADAS do `file_loc_caps`, que é o gate a sério — o mapa
compara contra o tecto genérico de `600`.

### O smoke COMPILADO (nesta worktree, o mesmo pacote e perfil que o dono corre)

```text
=== build 1
    Finished `smoke` profile [optimized] target(s) in 15.60s
=== build 2
    zero linhas `Compiling` · Finished in 0.21s
```

### O smoke do dono

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 PH2D_BONE_LOG=1 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O que ele mostra que a cena de antes não mostrava:** a **barra azul** passa por cima do braço
pintado (a imagem presa está na ORDEM do quadro), e o **olho** da linha *«Painted arm»* na Hierarquia
esconde-a. Com a camada do Vello, a imagem ficava à frente da barra e continuava desenhada com o olho
fechado.

⚠️ **E o que a W11 acrescenta:** com o braço bem DOBRADO, o risco pintado tem de sair com a
**espessura do anel do cursor** — e não uma lasca fina onde o leque comprime a arte.

⚠️ **E o que a W15 acrescenta (os três itens do report do IK):** com a âncora criada, o painel
*Bones* tem de mostrar **`Chain = 2`** (e `Mix = 1`, `Softness = 0`, mais o comprimento e a força do
osso) — ⛔ até 2026-09-14 os cinco mostravam `0`. Mudar o `Chain` para `3` ou `4` tem de **mover a
faixa** que marca a corrente governada no canvas (o X marca onde ela pára), e o **Bend** tem de
continuar a inverter o joelho com `Chain` maior que `2`.

⚠️ **E o que a W14c acrescenta (as duas fotos, *«ao acrescentar o IK o osso perde influência sobre a
ponta da malha»*):** na mesma cena, com a âncora criada, arraste-a e veja a **ARTE** dobrar com os
ossos. ⛔ Até 2026-09-14 o gizmo do osso ia para a pose resolvida e a arte ficava na curva — a malha
era construída antes de o solver escrever.

⚠️ **E o que a W14b acrescenta (o report seguinte, *«ainda não funciona para IK»*):** na mesma cena,
arraste o **losango da âncora** (a restrição que a cena cria na ponta do braço) com o **AutoKey**
ligado. A corrente segue-o; ao arrastar o cursor do tempo para longe e voltar, **o braço tem de
voltar à mesma pose** — porque o que ficou gravado é a **âncora**, e os ossos derivam dela. ⛔ Até
2026-09-14 o gesto não gravava nada: os ossos são conduzidos pelo solver (e o AutoKey salta-os, com
razão) e o alvo não estava na lista de quem o passe olha.

⚠️ **E o que a W14 acrescenta (a fase seguinte, escolhida pelo dono):** com *Window → Timeline*
aberta e o **AutoKey** ligado, puxe a **PONTA** do braço com a ferramenta **Bone** — a corrente
inteira dobra. Ao largar, **todos** os ossos que se moveram têm de ter chave (a linha de cada um na
timeline ganha um losango). ⛔ Até 2026-09-14 só o osso **seleccionado** a recebia, e a pose
desaparecia ao arrastar o cursor do tempo para longe e voltar.

⚠️ **E o que a W12 acrescenta, que é o que o 3.º report pedia:** repita o toque com o pincel **bem
grande** (o anel a cobrir vários triângulos da arte dobrada). A marca tem de sair redonda **também
aí** — ⛔ até 2026-09-14 um pincel grande sobre uma malha grossa saía **menos** redondo do que sairia
sem correcção nenhuma, porque o dab inteiro levava a deformação de um pedaço dele.

⚠️ **E o que a W10 acrescenta:** na MESMA cena, dobre o braço pintado (arraste o corpo de um osso),
escolha a linha *«Painted arm»*, pegue no **Painter** e pinte sobre a arte **dobrada**. A tinta tem de
sair debaixo do ponteiro. ⛔ Até 2026-09-14 ela caía deslocada exactamente pela deformação, e um
clique sobre o quad de REPOUSO (onde nada se desenha) começava um traço invisível.

⚠️ **E o que a W9 acrescenta:** com a MESMA cena, escolha a linha *«Painted arm»* na Hierarquia,
pegue na ferramenta **Flip** (modo *Draw*) e desenhe **por cima do braço pintado**. Tem de desenhar.
⛔ Até 2026-09-14 o traço não começava ali — a caixa do gizmo da sprite seleccionada cobria a arte e
o ramo do Flip exige `on_canvas`.

⚠️ **E o que a W7+W8 acrescentam a ele** (a cena imprime a linha que nomeia o osso): abrir
*Window → Timeline*, escolher na **Hierarchy** a linha de osso que o terminal nomeia, arrastar o
cursor da timeline para o meio e ligar **Onion** — têm de aparecer **duas** silhuetas do braço
pintado, uma esverdeada (passado) e uma azulada (futuro), as três formas DIFERENTES. ⛔ Se só
aparecer a azulada, o relógio do onion voltou a ser o da cena (W8). E com o AutoKey ligado, arrastar
o CORPO desse osso tem de o mover e cunhar chave — se ele voltar debaixo do dedo, a mão deixou de
existir para o apply (W8).

---

## §11 — A UMA LINHA proposta para o `CLAUDE.md §5` (entrada **Vector**, depois da linha do ATLAS)

> ✅ **A PELE DE IMAGEM ENTROU NO PASSE DE SPRITES** (13/09, [handoff](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_PELE_NO_PASSE_DE_SPRITES_2026-09-13.md), [plano](docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md)): uma imagem presa ao esqueleto era uma **camada do Vello por cima do quadro** — fora da ordem, desenhada com o olho fechado, sem as propriedades da sprite, com costuras, e com a régua a ler a âncora CRUA. Hoje ela é uma **sprite do quadro desenhada como MALHA** (`SpriteMesh`, ⭐ **sem pipeline nova**: `N` triângulos entram como UMA tira nas `TriangleStrip` de sempre), e quem copia a instância (vidro do prefab, emissivo) ou aponta (picking, caixas, *View All*, UV do pintor) lê o que é DESENHADO. ⭐⭐ **O orçamento saiu do buffer do Vello e passou a sair do TEMPO do quadro:** `8 738 → 1 543` peças, de `1,08 µs` medidos por peça entregue. ⭐⭐⭐ **E o ONION de um rig passou a EXISTIR** (W7+W8): ele exigia que o seleccionado estivesse animado **e** desenhasse, e numa personagem riggada quem leva keys são os **ossos** (que não desenham) e quem desenha é a **imagem** (que não leva keys) ⇒ *zero fantasmas, sempre* — a nota antiga dizia *«eles desenham o quad de repouso»*, que é verdade **se houver fantasma**. Hoje um OSSO ghosta a arte que ele deforma, **dobrada** no instante de cada silhueta, e os instantes saem dos ossos ANIMADOS do esqueleto (o modo `Keys` é o de omissão e lia as keys do alvo desenhado). ⚠️ **E o relógio era o QUARTO palpite:** a chamada passava o `playhead` da CENA enquanto a aba Keys dirige o `clip_playhead` — com o cursor parado em `0` só o FUTURO tinha vizinhos (report do dono; arch-gate `the_onion_speaks_the_clip_clock`). ⭐⭐ **E a MÃO que pousa um osso passou a existir para o quadro:** o apply da timeline só conhecia a lista do gizmo de sprite, logo com a timeline aberta ele reescrevia a rotação do osso pela curva no quadro seguinte (`0,77 → 0,45`) — o osso voltava debaixo do dedo e o AutoKey lia `mundo == curva`. Hoje o que a mão segura é **o esqueleto inteiro** (a IK da ponta move a corrente toda) e um arrasto de osso é **UM** passo de undo. ⭐⭐ **E o GÉMEO DO FLIP fechou sem report** (W9): a condição da caixa de objecto era *«não estou na ferramenta vectorial»*, então com a ferramenta **Flip** a desenhar uma sprite de referência seleccionada publicava a caixa dela e **apagava o traço por cima da arte** (o ramo do Flip exige o mesmo `on_canvas`, em 9 sítios). Hoje a condição é *«nenhuma ferramenta AUTORA no canvas»* e o terceiro `if` por família morreu. ⛔ **A saída do Painter — isentar os ids do gizmo no hit-test — está RECUSADA aqui:** ali a caixa continua PINTADA e deixa de pegar. ⏳ Ficam o 9-slice e a folha desdobrada. ⭐⭐⭐ **E o PINCEL PASSOU A PINTAR SOBRE A ARTE DOBRADA, em três reports** (W10–W12): a tinta caía deslocada **em toda a arte** porque o Painter mapeia o ponteiro pelo afim do **quad de repouso** e as duas portas que sabiam da malha tinham **zero chamadores de produto** (W10); a FORMA continuava a ser a da textura, e onde o leque comprime um disco chegava ao ecrã como uma **lasca** (W11); e a matriz nascia numa **base MISTA** — linhas em `y` para CIMA, colunas em `v` para BAIXO —, o que nega os termos fora da diagonal e espelha a elipse de uma arte **RODADA**, com as fixturas das duas metades alinhadas aos eixos e portanto cegas (W11b). ⭐⭐⭐ **E a W12 fechou o que o dono chamou de *«talvez artefato inevitável»*: uma malha é afim por TRIÂNGULO, e a deformação era lida num PONTO**  — um dab que cobre vários triângulos levava, por inteiro, a deformação de um pedaço dele, e com um pincel grande sobre uma malha grossa isso deixava a marca **menos redonda do que não corrigir nada** (`1,383` contra `1,188`). Hoje a porta responde pelo melhor afim **sobre o disco que o dab ocupa** (`1,18`), e um dab que cabe numa facete recebe a facete **ao bit**. ⚠️ **Três mutações sobreviveram à primeira, e a melhor delas é uma lei:** *um dab REDONDO não consegue medir a BASE* — um factor ortogonal tem os mesmos valores singulares e os mesmos eixos, logo o espelho é invisível ao círculo unitário; só uma elipse **autorada** o separa (`30°` contra `155°`). ⏳ O resíduo que fica é **uma elipse por dab**, e é troca de resolução, não parede. ⭐⭐ **E o tecto da shell foi curado pela cura que o handoff anterior prescrevia e não fez:** o motor do onion (`1 014` linhas, zero `App`) saiu para [`ph2d-timeline-onion`](crates/ph2d-timeline-onion/) — shell `196 989 → 195 974`. 
