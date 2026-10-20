# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 19/10 — F23 metade A (o painel lê a curva que a tela desenha) feita; metade B (o losango no campo) começada

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (sem rebase). Anterior:
  [F22](HANDOFF_CONTINUACAO_line_UIUX_2026-10-19_F22_O_GESTO_VENCE_A_MAQUINA.md) (§6 = F22b).
- Fecha **F22 ABERTO 4** (opacidade conduzida pela timeline no painel) na parte do VALOR; a parte «o campo diz que é animado» é a metade B, aberta.
- Plano: [`34_plano_F23_o_campo_diz_que_e_animado.md`](../vetor_nova_ui/34_plano_F23_o_campo_diz_que_e_animado.md) (§1 = defeitos, §3 = desenho, §9 = feito e o que falta).

## 1 — Commits

| commit | o quê |
|---|---|
| `70e61530fc` | docs: plano 34 (o defeito settle × autokey medido vermelho) |
| `4fce8515fa` | feat: metade A (D1–D4 do plano §1) |
| (este commit) | `channel_mark` + `key_mark` (sem consumidor) + plano §9 + handoff |

## 2 — Superfície de colisão e decisões que um merge pode partir

**`ph2d-app-vec` `driven_style.rs`**
- `settle_to_authored(sim, map, scene, conduzida: impl Fn(u64) -> bool)`: assinatura nova (4.º argumento); salta quem tem binding de `PropKind::Opacity`.
- `live_layers(sim, map, store, view)` = timeline (`resolve`/`apply`) + rows (`widget_drive`): a porta única da tela (`fase_vector_view_and_drives`) e do painel. O alias `vec_widget_drive` saiu do `main.rs`; os gates `the_draw_pass_publishes_what_the_rows_drive` e `the_driven_style_is_read_before_it_settles` passam a ler `live_layers` (um merge que reintroduza o alias parte-os).
- `follow_the_hand(sim, map, sel, v)`: chamado em `fase_transform_ops` ao lado do `set_opacity`.
- `paint_stack::published(scene, sel, vivo: &VecViewState)` pergunta ao `ph2d_vec_scene::object_alpha`; o misto compara os valores vivos. Chamador na forma antiga não compila.

**Shell `timeline_bridge_keys.rs`**
- `Mundo { world, vec: Option<&VecScene> }` (`From<&World>`; `Mundo::com_vetor`). `sample_prop_value`, `key_value_for`, `key_authoring_solo`, `publish_track_values`, `autokey_pass::sample_pose` aceitam `impl Into<Mundo>`.
- `autokey_pass::run` ganhou `vec: &VecScene`: chamador antigo parte (5 em `autokey_bone_tests.rs` já atualizados).

**`ph2d-timeline` (metade B, sem consumidor)**
- Novos `channel_mark.rs` + `channel_mark_tests.rs`; `lib.rs` exporta-os. `autokey::curve_seen` (pub(crate)) é a porta única da curva; o diff do autokey passou a usá-la — um merge em `autokey.rs` deve manter essa chamada.

**`ph2d-editor-core` (metade B, sem consumidor)**
- Novos `widget/property_box/key_mark.rs` + `key_mark_tests.rs`; `property_box/mod.rs` declara-os. Estado `thread_local` (`set_key_marks`).

**O que um merge pode partir**
- Chamador de `settle_to_authored`/`autokey_pass::run`/`published` na forma antiga.
- Qualquer escritor novo de `VecDrivenStyle.alpha` que não passe por `follow_the_hand` volta a perder a edição à curva.
- `PROJECT_SCHEMA` e contratos §6 intocados. `intent.rs` está a 695/700 LOC: não acrescente intents (a metade B apaga chave por `SelectSingle` + `DeleteSelection`).

## 3 — Smoke (do dono)

Sem cena nova nesta metade (a cena 38 é da metade B). Manual:
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Crie uma forma, abra a aba **Keys**, anime a opacidade de 20 % a 100 % (duas chaves) e pare o relógio a meio, com a forma selecionada.
3. Tem de acontecer: a tela fica no valor da curva (não pula para o valor gravado) e a linha **Opacity** do Appearance diz o mesmo valor (≈ 60), não o gravado.
4. Com **Auto Key** ligado, parar o relógio fora de uma chave NÃO cria chave. Digite 30 em Opacity ⇒ cria uma chave a 30 nesse tempo.
5. Errado = a forma saltar para o valor gravado ao parar, o painel dizer 100, ou aparecer uma chave nova só por parar.

## 4 — Verificação

Bateria por nome (verificador, 19/10), toda verde: app-vec `driven_style` 7 · `appearance` 5 · `paint_stack` 10 · `ui_state` 46 · `vector_bridge` 45; timeline `channel_mark` 5 · `autokey` 28 · `it a_vector_path_fades` 4; editor-core `key_mark` 2 · `architecture_*` (shell_only_shrinks, file_loc_cap, panel_wiring_parity, foundation DAG) + `icons` 9 · `no_magic_numeric` 3 · `no_literal_color` 3; shell `--bin` autokey/timeline_bridge/timeline_preview/opacidade_vetor/fase_timeline_key_insert 102 · `--test it` the_driven_style/the_draw_pass/authored_intents 9. `fmt` limpo. clippy, mutação e `nextest-impacted`: **não corridos** (fecho da linha). O `key_mark` é `pub` (re-exportado em `widget::key_mark`) para o clippy `-D warnings` não o acusar de morto enquanto a metade B não o liga.

Já observado: D2 medido VERMELHO no quadro real (4 testes de `autokey_opacidade_vetor_tests.rs` vermelhos com a cura desligada, 6/6 verdes com ela). A sonda corre em `keys_mode = true`; com a pilha vazia a vista «cena» não toca o clip. Metade B: 5 testes em `channel_mark_tests.rs`, 2 em `key_mark_tests.rs`. Prova de mutação da metade B: por fazer (§5).

## 5 — ABERTO

1. **Metade B (plano §9, «O que falta»):** (1) trocar `paint_decorator_dot` por `paint_decorator_for` em `property_row.rs:219` e `:375` e nas linhas do Inspector; (2) coluna de decorator nas linhas slider+chip (`DECORATOR_W`, `row.rs:58`) — Appearance `paint_appearance.rs:72`/`paint_rows.rs:23`, `VECTOR_MORPH_T` `paint_blend.rs:124`, `INSP_SPRITE_OPACITY` `color_tint.rs:228`; barra de controlo `bars_control.rs:172` sem coluna (decidir); (3) o clique: registo genérico `key_mark_at` consultado no press, ou id derivado por campo (`NodeId(x ^ SAL)`, `ids/live_sections.rs:208`) + braço em `event_clicks.rs:190` e `event.rs` `SINGLE_ID_CLICKS`; o seam test `MockPanelHost::click_at` (`ph2d-ui-testkit/src/gestures.rs:38`) é o DoD, pois `architecture_panel_wiring_parity` não vê ids derivados; (4) a shell: tabela `(campo, PropKind)` por painel, `channel_mark` com o `t_src` do autokey, `set_key_marks` 1×/quadro, clique → `AddKey` (`key_value_for`/`key_insert_time`) ou `SelectSingle` + `DeleteSelection`.
2. Cena 38 (estágios livres 42..=49, 57..=59, 64..=69): opacidade 20→100 % + posição animada, relógio a meio ⇒ losango vazio em Opacity e X; numa chave ⇒ cheio. Foto `fotos/F23_cena38.png`. Smoke do dono desta cena: por fazer.
3. Mutação no fecho: settle sem `conduzida` (já 4 vermelhos) · `follow_the_hand` no-op · `or_else` do gravado em `sample_prop_value` · `published` a ler `p.opacity.get()` · `curve_seen` com solo/pilha trocados · meio quadro → 0 · `key_mark_of` com `min`.
4. **Fora, com mecanismo:** o `rest` de `read_prop_kind` (`ph2d-timeline`) para opacidade de vetor nunca conduzido continua 1,0 (o leitor não alcança o documento vetorial; só pesa num clip da pilha que não cobre o canal). Tracejado de vários pares na placa: `ph2d-shape-gpu/src/eixo.rs:128`, `MAX_DASH_PAIRS = 3`, 2–3 pares caem ao Vello — família render.
5. **Correção de estado:** o flake `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` JÁ está promovido (`docs/DevOps/FLAKES_DE_CARGA.md:57` + `.config/nextest.toml:85`): F21 ABERTO 7 / F22 ABERTO 8 estavam desatualizados.
6. Checkbox/toggle no canvas (F18 ABERTO 3): decisão visual do dono. F20 ABERTO 6, F18 4, F16–17 seguem herdados.
7. `CLAUDE.md` §5.1 UI/UX: **não** alterado; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…37` → `…38` se a cena 38 entrar).
8. `nextest-impacted` não corrido nesta onda.

## 6 — Depois do handoff: a cena do Auto Key, o smoke e a ordem do dono (19/10)

- Report do dono: *«opacity ainda não funciona com auto key, não grava quadros, assim como as outras propriedades dos desenhos vetoriais»*. Commit `b3a889a629`: cena `PH2D_AUTOKEY_SMOKE=1|2` (`ph2d-app-vec/src/autokey_cena.rs`, chamada em `fase_object_mode.rs`; `=2` mexe sozinha) e a sonda `PH2D_AUTOKEY_LOG=1` (`autokey_pass.rs` `sonda`: painel/armado/população, e por entidade capturando/solo/t/casa/pose/fila — uma linha quando muda).
- **Medido no app** (`=2`, tela virtual): Opacity ← 30 ⇒ chave 0,30 no quadro 12 (2 → 3 chaves); X do objecto +1 ⇒ chave de X (2 → 3). Fotos `fotos/F23_autokey_1_inicio.png` (Opacity 60 no Appearance e na barra = a curva) e `F23_autokey_2_gravou.png`.
- **O mecanismo do resto do report:** a Posição X/Y do painel do vetor (`despacho_vetor_alinhar_e_forma.rs` → `translate_path_world`), o arrasto e o empurrão da ferramenta Vetor (`selection.rs:257` `translate_path`) movem os PONTOS; o `Transform` que a timeline anima não muda e o Auto Key nada vê. Só o arrasto do gizmo de objecto (`vec_snap.rs` `slide_entity_world`) mexe no `Transform`.
- ✅ **Smoke do dono: OK (19/10)** — a cena `=1`, Opacity 30 ⇒ chave.
- ⛔⛔ **ORDEM DO DONO (19/10), a próxima onda:** *«O auto key deve gravar absolutamente tudo neste app, não só as transformações»* (memória `project_autokey_records_everything_2026_10_19.md`). Toda propriedade editável vira canal animável e o Auto Key armado grava-a — inclusive a edição dos PONTOS (não é cura «mover o objecto em vez dos pontos»). Hoje a timeline só anima os 19 `PropKind` (`ph2d-timeline/src/prop.rs:21`, fio congelado só por APPEND): cor, largura e tracejado do traço, cantos, geometria e o resto não têm canal. Comece por inventariar as propriedades editáveis por família e medir o modelo de canais (Rive / After Effects: cada propriedade tem chave); o losango da metade B (§5) passa a valer para todas.
