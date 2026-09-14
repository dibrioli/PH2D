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
| `shells/desktop/src/render_loop/timeline_bridge.rs` (W8) | `maos_do_quadro` NOVA (o gizmo ∪ o esqueleto que a ferramenta Bone pousa); `run` troca `live_entity: Option<u64>` por `maos: &[u64]` | **muda a assinatura** (shell-interna) |
| `shells/desktop/src/render_loop/{fase_timeline_view,fase_timeline_drain,fase_frame_open}.rs` (W8) | o `TimelineView::dragging_entity` vira `maos: Vec<u64>` e atravessa a fase | sim |
| `shells/desktop/src/render_loop/autokey_pass.rs` (W8) | `run` ganha `skeleton: &SkeletonState`; `drag_now = gizmo.drag.is_some() \|\| skeleton.bone_pose.is_some()` | **muda comportamento**: um arrasto de osso passa a ser UM passo de undo |
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

---

## §8 — O que fica ABERTO

| item | estado |
|---|---|
| ✅ **Os fantasmas do onion desenhavam o quad de repouso** | **FECHADO pela W7** (a pose de mundo em `t`, a pele resolvida nele, a malha por fantasma) e **alcançável desde a W8** (o relógio do clip). ⚠️ A redacção fica aqui por contraste: ela dizia *«eles desenham o quad de repouso»* e a medição mostrou que **não havia fantasma nenhum** |
| ✅ **A UV do pintor** | **FECHADO pela W10**, e o item estava mal endereçado: o Painter não chamava nenhuma das duas portas de UV — ele tem afim próprio, do quad de repouso, logo o erro era em TODA a arte dobrada |
| ⏳ **A deformação do pincel é a do PEN-DOWN** | o `stroke_spec` é capturado ao abrir o traço (a mesma fotografia que o pincel de tecido tira dos obstáculos): um traço LONGO que atravesse compressões diferentes usa a do princípio. Para seguir por dab, a deformação tem de viajar no `StrokePoint`, como a pressão — medido e não feito |
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
| ⏳ **O AutoKey de um osso key só o SELECCIONADO** | com a corrente inteira congelada durante o arrasto, quem a IK moveu não recebe chave — o `autokey_pass` amostra `gizmo.iter_selected()`. É o modelo do Blender (keya-se o osso escolhido), mas **não foi medido contra ele** |
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
| `BASE=1d43da737 bash scripts/nextest-impacted.sh` | ✅ **14 639 passaram, 0 falharam** (10 428 saltados), `37,9 s` — a W11 traz as duas maiores crates do repo para a varredura, e nenhum membro da família de flakes de carga reprovou |
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets` | ⚠️ **correu em CACHE e não repete avisos** (a saída inteira é uma linha, `Finished`). Forçado o replay da única crate com aviso: `ph2d-preview-drive`, **pré-existente** e intocada por esta linha (§8). As crates desta linha foram corridas com replay: zero |
| `cargo machete` | ✅ nenhuma dependência por usar — a `ph2d-vector` e a `ph2d-asset` SAÍRAM da `ph2d-skeleton-live` |
| `bash scripts/doc-index.sh --check` | ✅ 19 índices em dia |
| provas de mutação | ✅ **21 RED** na asserção certa (8 na W2, 8 na W3, **5 na W8**), cada uma com o controlo `1 failed` — um filtro que casasse zero leria `0 passed; 0 failed`. ⚠️ A 1.ª redacção da 5.ª (o onion a aceitar um instante ausente) **não compilava**, e um erro de compilação não é um gate vermelho: foi reescrita numa forma que compila. **W9: mais 3** (a cláusula do Flip · o quarto ramo no canvas · o termo da preview). **W10: mais 4** e **W11: mais 5**, e ⚠️ **uma sobreviveu à primeira em cada uma** — as duas pelo mesmo motivo: citar a porta em vez de a ligar |
| tectos de LOC | ✅ ⚠️ **A W8 reprovou DOIS ficheiros de teste da shell** (`autokey_pass_tests` `612` · `timeline_onion_tests` `613`, tecto `600`): curados por **corte por responsabilidade** — o gate do osso saiu para o `autokey_bone_tests.rs` e o do relógio para um **sub-módulo** que herda as fixturas (`timeline_onion_clock_tests.rs`, o molde do `skin_at_time_tests` da W7). ⛔ Nunca subir o número. E o resto: ⚠️ **O `app_state.rs` bateu no tecto** (`1 023 / 1 019`) e a cura foi CORTE da prosa que eu tinha acrescentado — nunca subir o número |

⛔⛔ **O NÚMERO QUE O INTEGRADOR TEM DE VER:** a shell fecha esta linha em **`196 980`** linhas
contra o tecto de `196 990` do `the_shell_only_shrinks` — **`10` de folga**. Ele é um tecto que SOMA
entre linhas sem ninguém a contar (`CLAUDE.md` §5.0), e esta linha gastou-o quase todo em GATES.
⭐ **A cura MEDIDA está identificada e não foi feita, de propósito:** o `timeline_onion.rs` e os dois
ficheiros de teste dele são **~800 linhas que não são composição** — o motor do onion depende só de
crates (`ph2d-ecs`, `-render`, `-timeline`, `-skeleton-live`, `-poly2d`, `-vec-entities`) e só a
CHAMADA é da shell. Tirá-lo para `crates/ph2d-timeline-onion` é o molde do HOWTO e devolve ~800
linhas. ⛔ Enquanto isso não acontece, a cura de um vermelho é corte por responsabilidade — nunca
subir o número.
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

> ✅ **A PELE DE IMAGEM ENTROU NO PASSE DE SPRITES** (13/09, [handoff](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_PELE_NO_PASSE_DE_SPRITES_2026-09-13.md), [plano](docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md)): uma imagem presa ao esqueleto era uma **camada do Vello por cima do quadro** — fora da ordem, desenhada com o olho fechado, sem as propriedades da sprite, com costuras, e com a régua a ler a âncora CRUA. Hoje ela é uma **sprite do quadro desenhada como MALHA** (`SpriteMesh`, ⭐ **sem pipeline nova**: `N` triângulos entram como UMA tira nas `TriangleStrip` de sempre), e quem copia a instância (vidro do prefab, emissivo) ou aponta (picking, caixas, *View All*, UV do pintor) lê o que é DESENHADO. ⭐⭐ **O orçamento saiu do buffer do Vello e passou a sair do TEMPO do quadro:** `8 738 → 1 543` peças, de `1,08 µs` medidos por peça entregue. ⭐⭐⭐ **E o ONION de um rig passou a EXISTIR** (W7+W8): ele exigia que o seleccionado estivesse animado **e** desenhasse, e numa personagem riggada quem leva keys são os **ossos** (que não desenham) e quem desenha é a **imagem** (que não leva keys) ⇒ *zero fantasmas, sempre* — a nota antiga dizia *«eles desenham o quad de repouso»*, que é verdade **se houver fantasma**. Hoje um OSSO ghosta a arte que ele deforma, **dobrada** no instante de cada silhueta, e os instantes saem dos ossos ANIMADOS do esqueleto (o modo `Keys` é o de omissão e lia as keys do alvo desenhado). ⚠️ **E o relógio era o QUARTO palpite:** a chamada passava o `playhead` da CENA enquanto a aba Keys dirige o `clip_playhead` — com o cursor parado em `0` só o FUTURO tinha vizinhos (report do dono; arch-gate `the_onion_speaks_the_clip_clock`). ⭐⭐ **E a MÃO que pousa um osso passou a existir para o quadro:** o apply da timeline só conhecia a lista do gizmo de sprite, logo com a timeline aberta ele reescrevia a rotação do osso pela curva no quadro seguinte (`0,77 → 0,45`) — o osso voltava debaixo do dedo e o AutoKey lia `mundo == curva`. Hoje o que a mão segura é **o esqueleto inteiro** (a IK da ponta move a corrente toda) e um arrasto de osso é **UM** passo de undo. ⭐⭐ **E o GÉMEO DO FLIP fechou sem report** (W9): a condição da caixa de objecto era *«não estou na ferramenta vectorial»*, então com a ferramenta **Flip** a desenhar uma sprite de referência seleccionada publicava a caixa dela e **apagava o traço por cima da arte** (o ramo do Flip exige o mesmo `on_canvas`, em 9 sítios). Hoje a condição é *«nenhuma ferramenta AUTORA no canvas»* e o terceiro `if` por família morreu. ⛔ **A saída do Painter — isentar os ids do gizmo no hit-test — está RECUSADA aqui:** ali a caixa continua PINTADA e deixa de pegar. ⏳ Ficam o 9-slice e a folha desdobrada.
