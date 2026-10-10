# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-10 — F8: ENQUADRAR TUDO

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F7e, 10/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7e_EDIT_SHAPE.md) — citada,
> não reescrita. Lei, oráculos e recusas: [`11_enquadrar_tudo.md`](../vetor_nova_ui/11_enquadrar_tudo.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `2a925859d` (o `main` não andou — sem rebase)
  · F0–F7e continuam por integrar e vão JUNTAS com esta jornada.
- Commits desta jornada: `0f38cdfcf..HEAD` (7).
- **O pedido (dono, 10/10):** *«enquadrar tudo»* — o desenho inteiro cabe na tela num gesto (no
  iPad, o beliscão rápido do Procreate).

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; a frase ganha «enquadrar»; o smoke `…|6..13` → `…|6..14`.

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **os oráculos** | tldraw 5.5.2 (COM interface: variante `~/Apps/tldraw-oracle/www_ui`) e Excalidraw `next`, pelo Playwright, 11 cenas nossas × 3 janelas × 3 câmaras, tecla e API; 66 fixtures (4,6 MB) | `vetor_nova_ui/oraculo_enquadrar/` |
| ⭐ **a lei** | `frame`/`frame_within` (caixa → vista na ÁREA, margem, zoom pela lei da câmera), `along` (ponto fixo, `ln h`), `ViewFlight` (testemunha, `fresh`/`abandon`/`arbitrate`, a memória do beliscão ao contrário); `Camera2d::{view, set_view, clamp_height}` (a roda pergunta o mesmo clamp) | `ph2d-render/src/camera_frame.rs` (novo) |
| **as caixas** | o que a cena DESENHA: imagens do mundo de apresentação, formas da `FrameOrder`, Flip não escondido; a seleção (primária + extras, grupos pelos filhos desenhados) | `ph2d-app-vec/src/frame_boxes.rs` (novo); `ph2d-app-flip::gizmo_view::{world_aabb, drawn_aabbs}` |
| **o pedido** | `view_request::resolve` (All · FrameSelected · Selected · Camera · Previous) e `ViewState` (o voo + o pedido adiado UM quadro) | `ph2d-app-vec/src/view_request.rs` (novo) |
| **a área** | `canvas_area::frameable` = a visível sem a faixa da pílula ⑤ | `ph2d-app-host/src/canvas_area.rs` |
| **a margem** | `float_bar::frame_margin()` = `Lg` + uma linha da ④ + 2·`Xs` = 44 px lógicos (derivada; meio da faixa dos oráculos) | `ph2d-editor-core/.../float_bar.rs` |
| **as portas** | *Ver ▸ Frame All · quick pinch* / *Frame Selection*; `TouchOut::QuickPinch` no reconhecedor; `ViewFocusKind::{FrameSelected, Previous}`; `GizmoView::world_aabb` | ids/menu/i18n/`rail_tools.rs`; `interaction/touch.rs`; `gizmo/world_box.rs` (novo) |
| **a shell** | `hero_intents/view.rs` SAI; `fase_view_frame.rs` (novo, só junta as fontes); `canvas_zoom::publish` (roda + voo); o toque pede pelo barramento | **+100 −109** |
| **cena 14** | a 9 + três formas FORA da tela + o clique real em *Ver ▸ Enquadrar tudo*; foto [`f8_01`](../vetor_nova_ui/fotos/f8_01_cena14_tres_formas_fora_da_tela_e_ver_enquadrar_tudo.png) | `ph2d-app-vec/src/vector_mode_frame.rs` (estágios 60–63) |

## §2 — Medido e decidido (técnico, delegado)

Tabelas em [`11_enquadrar_tudo.md`](../vetor_nova_ui/11_enquadrar_tudo.md) §2.

- **⛔ Premissa do briefing derrubada:** o enquadrar EXISTIA (`ViewFocusKind::All`, o 3.º estado do
  botão VIEW): contava só imagens, encaixava na JANELA `×1,1` e saltava. A grep do prompt não o via.
  Curado no mesmo verbo (uma porta).
- **Os dois oráculos são a mesma lei com parâmetros diferentes** — o gate reproduz ≥ 200 corridas de
  tecla dos dois ao centésimo de px. Os nossos parâmetros: a área visível (como o Excalidraw), 44 px,
  o zoom da câmera (como o tldraw), cena vazia → vista de omissão (como o Excalidraw).
- **O voo:** `Role::Viewpoint` (a mola da roda; assenta ~0,38 s, o tldraw ~0,33 s; sobrevive ao
  *reduced motion*). ⛔ Um gate da shell apanhou o 1.º entalhe da roda PERDIDO a meio de um voo ⇒
  `ViewFlight::arbitrate`: o gesto mais novo ganha.
- **O pedido resolve-se um quadro depois** — medido nas fotos: a `PH2D_VEC_BONE_MEDIA_SMOKE=1/2`
  pede no quadro em que monta, e as mídias presas a ossos saíam cortadas.
- **`F`/`Home` e as ~10 cenas de smoke que pedem `Selected` NÃO mudaram** (centrar, saltar): o
  *Enquadrar seleção* é um pedido novo. Teclas novas: nenhuma (decisão do dono; proposta no §7).
- **Beliscão rápido:** fronteira = `LONG_PRESS_NS` (0,5 s, a do toque de 2 dedos), varrida 100–1200 ms;
  nenhum número novo.

## §3 — Foundational tocado e contratos

- **`ph2d-render`:** `pub mod camera_frame`; `Camera2d::clamp_height` (o `zoomed` delega). Dev-dep
  `serde_json = "1"` (já no lockfile; `Cargo.lock` muda 1 linha).
- **`ph2d-editor-core`:** `ViewFocusKind::FrameSelected`, `::Previous` (enum público — quem fizer
  `match` exaustivo noutra linha parte); `TouchOut::QuickPinch { closing }` (idem); ids
  `MENUBAR_VIEW_FRAME_ALL` = `hash_node_id("menubar_view_frame_all")`, `MENUBAR_VIEW_FRAME_SELECTED` =
  `hash_node_id("menubar_view_frame_selected")`; `menu_bar::OWN_ROWS` 11 → 13; `float_bar::frame_margin`;
  `GizmoView::world_aabb`.
- **`ph2d-i18n`:** `chrome.menu.frame_all`, `chrome.menu.frame_selected` (+2).
- **`ph2d-app-host`:** `canvas_area::frameable`. **`ph2d-app-flip`:** `gizmo_view::{world_aabb, drawn_aabbs}`.
- **`ph2d-app-vec`:** `frame_boxes`, `view_request`, `vector_mode::draw_shape` passa a `pub(crate)`.
- **Contratos congelados (§6): nenhum.** `Tool=12`/`PanelEvent=4`/Nodes/Vector intocados.

## §4 — Superfície de colisão e contadores

- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+14**. Sem schema, sem `PROJECT_SCHEMA`, sem registos.
- **Shell:** +100 −109 nesta jornada (185 693 → 185 684); `App` sem campo novo (o estado mora no
  `AppGfx::view`); `hierarchy::dispatch` perdeu `view_focus_kind` e `present`.
- **Em silêncio:** outra linha que chame `hero_intents::drain_view_focus` (saiu) ou passe
  `view_focus_kind` ao `hierarchy::dispatch`; quem dependa do `All` antigo (só imagens, janela, salto):
  as três cenas que o pediam (`PH2D_VEC_BONE_SMOKE=2`, `PH2D_VEC_BONE_MEDIA_SMOKE=1|2`) foram
  refotografadas — inteiras, melhores — e as notas delas corrigidas.

## §5 — Gate final

Uma corrida do `verificador` (`BASE=2a925859d`, HEAD `bca5cf3a8`): warnings-deny ✓ · machete ✓ ·
standalone-optional ✓ · workflow-packages ✓ · censos 114/114 e 12/12 ✓ · `doc-index` ✓ · `adr-index` ✓
· `nextest-impacted` **17 349 / 17 353**. ✗ clippy (1 `manual_contains`), ✗ fmt (6 blocos), ✗ 3 tectos
deterministas (`gizmo/paint.rs` 721 > 700; `main.rs::new` 213 > 212; `App` 174 campos > 173), ✗
`motion_path_perf` = flake de carga (load ~27; 3× sozinho, verde). **Curas** (`bc76f0714`): o
`world_aabb` em `gizmo/world_box.rs` (paint.rs 696), o pedido adiado no `AppGfx::view` (a `App` sem
campo), clippy e fmt. Depois: os três tectos, `the_shell_only_shrinks`, a ponte do toque e o fmt ✓;
clippy `-D warnings --all-targets` nas 7 crates tocadas ✓. O `nextest-impacted` inteiro NÃO foi
re-corrido depois das curas (tectos, testes e a regra de arbitragem — dirigidos todos verdes).

### Mutação

Agente `mutacao` (verde visto antes, âncoras pré-voadas, restauro `git checkout` + `touch`, árvore
limpa no fim): **31 de 37 sangraram à primeira** — a lei inteira (o eixo apertado, a margem, o centro
da ÁREA, o clamp, a área estreita, o sinal do Y, o ponto fixo), o voo (a testemunha, a sementeira, a
memória, o `landed`, o `take`, o `fresh`, a arbitragem), o beliscão (o rápido, o pan, o sentido, o
levantar, o reset), a margem, o menu, a pílula, as caixas (o escondido, a ordem, o Flip), o pedido (a
memória, o adiamento, o centrar, a cena vazia). Das 6 sobreviventes: **G1** instrumento (o teste
mudara de módulo e o filtro `aabb` não o apanhava); **R11** e **T2** buracos reais — testemunhas
`framing_the_selection_after_everything_disarms_the_way_back` e `a_quick_twist_is_not_a_quick_pinch`;
**Q1/Q2** as duas costuras da shell — gate de texto `the_shell_asks_through_the_deferral_and_the_wheel_through_the_arbitration`
(na crate); **T7** redundante (o afastamento no tique; o do levantar já o grava) — **retirado do
código**. As 5 re-mutadas: as 5 sangram. **Final: 36/36 + 1 retirada.**

## §6 — Smoke do dono

Foto [`f8_01`](../vetor_nova_ui/fotos/f8_01_cena14_tres_formas_fora_da_tela_e_ver_enquadrar_tudo.png):
as cinco formas (três nasceram fora da tela) à vista, centradas na área entre a Hierarquia e o painel
Vector, acima da pílula do rodapé, a 48 %. E [`f8_02`](../vetor_nova_ui/fotos/f8_02_bone_media_1_as_tres_midias_inteiras_no_frame_all_curado.png):
o *Frame All* curado na cena das mídias presas a ossos.

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=14 ./target/smoke/ph2d-host-desktop`
2. A cena desenha um rectângulo e uma elipse, depois três formas FORA da tela, e clica sozinha em
   **View ▸ Frame All**.
3. Tem de acontecer: a vista ANDA (não salta) até as cinco formas caberem, com folga em volta; nenhuma
   por baixo da coluna de ferramentas nem da pílula do rodapé.
4. Errado: alguma forma cortada ou fora da tela, ou a vista a saltar.
5. Na mesma janela: dê zoom/arraste para longe e use **View ▸ Frame All**; selecione uma forma e use
   **View ▸ Frame Selection**; `F` continua a só centrar. Num ecrã de toque: um beliscão RÁPIDO
   (juntar os dedos e levantar) enquadra tudo; o mesmo gesto ao contrário volta à vista de antes;
   a pinça devagar continua a só dar zoom.

## §7 — O que fica ABERTO

- **Teclas (pergunta ao dono):** pelo perfil Illustrator, `Ctrl+0` = *Enquadrar tudo*; e `F` passar
  a *Enquadrar seleção* (enquadrar com zoom, como Unity/Maya/Blender) em vez de só centrar.
- O botão VIEW da régua continua o ciclo antigo (Selected → Camera → All).
- Um dedo real ainda não fez o beliscão rápido (provado com a mão sintética, como a F6).
- O Flip numa receita por editar: as caixas do Flip filtram o escondido pela árvore, não a receita.

## §8 — Binário de smoke (2.ª corrida)

(preenchido no fecho)

## §9 — `agent-loop-profile.sh`

(preenchido no fecho)
