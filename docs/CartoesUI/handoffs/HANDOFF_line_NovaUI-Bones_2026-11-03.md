# Handoff — linha `NovaUI-Bones`: família 5, o painel Bones em cartões (03/11/2026)

Worktree `Worktrees/line-NovaUI-Bones`, branch `line/NovaUI-Bones`, base `line/NovaUI` `11b6952897`.
**Não integrada, sem push.** Leitor: a próxima LLM / o integrador. Plano-mãe: [06](../06_COMO_PORTAR_PARA_O_MAIN.md) §3.
Portado de `line/TrelloUITeste` `45908badd2` (base `4e648ec2b4`). Commits: `df740ec55f` · `b590724917` · `15afd509d3` · `b844fe6913` + este handoff.
Sem `docs/`: 39 ficheiros, +3 254 −236. `.rs` da shell: +82 −85 (saldo −3).

## 1 — O que a linha entrega

1. **O painel Bones em cartões**: Tool, Weight Brush, Skin, Bone, Angle Limit, Smart Bone, IK (`cards.rs` + `cards_osso.rs`, partidos pelo tecto de 200 linhas por fn; `cards_presos.rs`, `state_preso.rs`). `PH2D_CARDS=0` = o corpo antigo.
2. **Length em px do documento (D27)**: `knobs::{px_per_unit, spec_on_panel, apply_from_panel}`, com a cola em `fase_skeleton_verbs` e `publica_o_osso`.
3. **O cartão solto preso a um osso** (T7/T44): retrato (`publica_presos`), corda (`ph2d_card_rope::anda(BONES)`, `bone_middle`), alfinete durável.
4. **A cura do defeito da foto do dono**: Object e Action em Smart Bone saem do estilo antigo (rótulo por cima, caixa na largura do cartão, `Widget::Button`/`Widget::Dropdown`). **Curve Tip** era a mesma família e foi curada no mesmo bloco. Já não há `c.legacy` no painel. Foto: [`bones_smart_bone_antes_depois.png`](../maquete/bones_smart_bone_antes_depois.png).
5. **A cena `PH2D_VEC_BONE_SMOKE=9`** (no `main` o `=8` é o losango; o 9 conta-se no roteador): montagem em `ph2d-skeleton-demo::cartoes`. `PH2D_VEC_BONE_SMOKE_SMART=1` nasce com Smart Bone (a foto não clica no `+`). `Prologo.transform_na_mao` arma o Transform.

Fotos: [`bones_aprovada_x_portada.png`](../maquete/bones_aprovada_x_portada.png) · [`bones_cartao_preso_corda.png`](../maquete/bones_cartao_preso_corda.png).

## 2 — Resoluções à mão contra a linha (o porte não era só aplicar)

| Choque | Resolução |
|---|---|
| `paint.rs`: F41 (largar a edição ao mudar de osso) × `semeia` | os dois, nessa ordem |
| `set_current_bone_subject`: `Option<u64>` no `main` × par `(u64, String)` no teste | fica o do `main` (10 testes e a shell usam-no); o nome entra por `set_current_bone_name`. **O retrato guarda os dois.** |
| `knobs::apply_from_panel` × `comprimento::aplica` (30/10) | o Length em px passa por `aplica`: também empurra os filhos |
| A caixa `SMART_CLIP`/`TIP` era `InteractiveState::Dropdown`, que não emite `Click` | `seam::lista`: nos cartões é `Button`, a lista abre por `dropdown::on_click`. **Sem isto a lista não abria; quem achou foi o teste de clique real.** |
| `tests/seam.rs`: 24 de 31 vermelhos (na linha de teste eram os «16 do skeleton», deixados) | a causa era uma só: uma pintura não mostra os cartões, que se medem ao pintar e abrem com mola. `trait Assenta` no teste; a caixa abre pelo mesmo caminho do hero (`aplica`). Agora medem os cartões, 32/32. |

## 3 — Superfície de colisão

- **Desta linha:** `ph2d-panel-skeleton`, `ph2d-app-skeleton`, `ph2d-skeleton-live` (`painel.rs` novo: `live_bone`, `bone_subject`, `metros_por_unidade`, movidos para a catraca de `ph2d-app-skeleton` passar), `ph2d-skeleton-demo` (`cartoes.rs`, +3 deps).
- **Fora dela, à mão:**
  - `ph2d-app-vec`: `lib.rs` (1 `mod`), `smoke_bone.rs` (nível 9; comentário do braço em S cortado como na linha de teste), `smoke_bone_cartoes.rs`, `smoke_bone_envelope.rs` (`transform_na_mao`). A linha do Inspector confirmou que não mexe nestes, salvo `lib.rs`.
  - `ph2d-editor-core/src/panel/cards_linhas.rs`: lista vazia não abre caixa; o segmento sem espaço larga o ícone (§5).
  - `ph2d-i18n/src/cartoes.rs`: `card.bones.list_title`.
  - `ph2d-panel-registry-init`: `Cargo.toml` (dev-deps `card-stack`, `card-rope`, `skeleton`), `tests/it/main.rs` (4 `mod`) e os testes da §6.
  - `shells/desktop`: `fase_skeleton_verbs.rs`, `vec_bone_smoke.rs`, `fase_selection_mirror_bone_focus.rs`, `Cargo.toml`.
- ⚠️ **`fase_selection_mirror_bone_focus.rs` é tocado também pela linha do Inspector** (`9b05cb020d`: `let (win, cam)`, `chave::resolve`, bloco `anda(INSP)`). As duas primeiras linhas e a linha do `Cargo.toml` foram escritas iguais e no mesmo sítio. **Fique com os dois blocos:** `publica_presos` + `anda(BONES)` daqui, `anda(INSP)` de lá. O `publica_presos` só compila com esta linha, por isso esta parte do alfinete ficou aqui, embora a cola retrato/alfinete seja da outra: a parte do Inspector não foi tocada.
- `Cargo.lock`: regenerar, nunca resolver à mão.

## 4 — O que precisa da linha do Inspector

- **Cursor preso do campo** (`ph2d-cursor-preso`, `sync_cursor_preso`): vem de lá. Sem ela, o arrasto no ícone funciona mas o cursor não fica preso. Esperado até à integração.
- ⚠️ **`shells/desktop/src/cursor_preso.rs` existe nas DUAS linhas (conflito add/add, de propósito): fica o da linha do Inspector, inteiro.** O daqui só traz a seta ↔ (§9) para o smoke desta linha. O `mod cursor_preso;` e as duas linhas em `despacho_metodos_janela_e_vetor.rs` foram escritos iguais aos de lá.
- Os testes `o_bloco5_abertos` e `o_bloco7_duas_copias` chamam `publica_presos` **e** o Inspector: não compilam em nenhuma das duas linhas sozinha. Entram na integração. O mesmo para a metade Inspector de `toda_dica_dos_cartoes`; a metade Bones está em `o_smart_bone_em_cartoes`.
- O número do nível `=9` e o `NIVEIS` só andam aqui.

## 5 — Decisões de produto tomadas por mim (o dono pode reverter)

1. Cena `=9`, não `=8`: o prompt dizia 8, mas esse número já é outra cena.
2. Numa coluna onde ícone + palavra não cabem nos segmentos, **a palavra fica e o ícone sai** (Create/Transform/Weight na coluna estreita): é regra de `Widget::Choice`, vale para todos os cartões. Motivo: o censo de letras comidas tem dívida 0 no Bones e não se sobe.
3. **Ordem** Tool · Bone · Angle Limit · Smart Bone · IK, como no prompt; a maquete põe IK antes de Smart Bone.
4. **Mix 1.00, não `Mix (%) 100`** como na maquete: o número é o do documento, e foi o que a linha de teste mostrou ao dono.
5. **Softness e Bend em linhas separadas**: com quatro opções (Mixed) o Bend não cabe ao lado.
6. **Object é um botão de texto centrado** (azul enquanto espera o clique), não um chip com seta: ele arma a escolha no canvas, não abre lista.
7. **Título `BONES`** é uma chave própria, não o título do painel em maiúsculas.

## 6 — Testes e provas

Novo: `o_smart_bone_em_cartoes` (clique real no hero), `an_empty_action_list_paints_no_box` (`seam`). Trazidos: `a_lista_de_cartoes_flutua_no_hero`, `o_alfinete_com_corda`, `o_cartao_solto_bloco4`, `o_cartao_solto_no_canvas`, `cartoes_notas`.

**Mutação, 8/8 a sangrar:** rótulo do Object · linha antiga na Action · sem `declare` (2) · `lista` como `Dropdown` · lista vazia · `v / k` no Length · sem abandonar a edição. A 1.ª rodada deu 6/8: o rótulo do Object e a lista vazia sobreviviam, o teste não os media. **Os dois testes foram refeitos e as duas mutações agora sangram.**

O vermelho do segmento estreito e do título foi visto antes da cura (`nenhum_rotulo`: 3 reprovados, 2 corridas iguais; depois 14/14).

## 7 — Saída dos portões

- **Verdes:** clippy `-D warnings` em `panel-skeleton`, `app-skeleton`, `skeleton-live`, `skeleton-demo`, `host-desktop`, `editor-core`, `i18n`, `panel-registry-init`; censos da árvore 12/12 (132 testes); `ph2d-panel-skeleton` inteira (48, antes da chave do título; `paint.rs` mudou 1 linha depois); `the_shell_only_shrinks`, `architecture_panel_loc_cap`, `architecture_workspace_file_loc_cap`, `architecture_widget_loc_cap`, `architecture_cycle_prevention`, `no_magic_numeric`, `hr12_widgets_a11y`, `tooltip`, `node_id_collisions`; `nenhum_rotulo` 14/14; `a_lista_de_cartoes_flutua` 4/4; `o_smart_bone` 4/4; `o_alfinete_com_corda` 4/4; `o_cartao_solto` 14/14; `os_campos_de_texto_tem_nome`; `a_docked_panel_never_reaches_the_drawing_area`; `a_panel_paints_where_its_tab_says`; `knobs` 9; `comprimento` 3 + 3; `skin_live` 27; `smoke_bone` 59; `ph2d-editor-core cards` 20; `ph2d-panel-widget-gallery cartoes` 38.
- **Vermelhos que NÃO são desta linha:** `cargo fmt --check` em `shells/desktop/src/project_sounds_tests.rs:41`; `cargo machete` acusa `ph2d-board-internet`, `ph2d-board-sync` (`app-kanban`), `rustc-hash` (`wgpu-core` vendor) e `ph2d-panel-pixel-preview` (`host-desktop`). Nenhuma dependência acrescentada aqui aparece.
- **Não corrido:** impactados e suíte completa (só na integração). `o_que_o_artista_nao_alcanca` como filtro não casou nenhum teste, apesar de o módulo existir: **não está coberto por esta rodada.**
- **Sem prova de tela:** a lista de Action aberta só está provada por clique em teste (a foto não clica), e a da cena do smoke depende dos clips dela.
- `agent-loop-profile`: `test:check` 2,5x (alvo ≤ 1) · edições sem conferir 1,6 % · contexto relido 226 mil · contexto no início 61 mil.

## 8 — Aberto

- Família 3 (mola do Kanban) e as cenas `PH2D_VEC_CARTOES_SMOKE`: fora desta linha.
- Maquete sem par no produto: o olho no Angle Limit.
- `CLAUDE.md` §5.1 não foi tocado: o integrador troca o link do módulo.

## 9 — Smoke do dono (03/11): OK, com duas curas

O dono aprovou o resto e achou duas coisas; as duas estão curadas e ele ainda não as reviu no app.

1. **Arrastar no ícone de Segments não fazia nada.** Causa: o campo arrasta em contínuo (0,1 por px), o documento guarda inteiro e `semeia` repõe-no a cada quadro; a base do Move seguinte era esse valor, e o resíduo perdia-se (`round(round(v) + d) == round(v)`). O `as u8` ainda cortava: subir ficava preso e descer caía no 1.º px. **O IK Chain é a mesma família** e foi curado no mesmo bloco.
   - Cura em `pointer_move.rs`: num campo de cartão a base é o acumulador enquanto o documento estiver a menos de um passo dele (só arredondou); além disso ele bateu num piso/tecto e a base volta ao valor, para a inversão não ter zona morta. Vale para todo campo de cartão, sem novo campo em `CardFieldSpec` (a linha do Inspector constrói-o). Campo contínuo: acumulador = valor.
   - `knobs::apply` (Segments) e o Chain em `fase_bone_smart_and_knobs.rs` arredondam em vez de cortar. Efeito colateral: escrever `2.7` dá 3, não 2.
2. **A seta ↔ sobre o ícone ou a letra não aparecia aqui.** `ew_resize_under` existia sem chamador nesta linha; quem a liga é a linha do Inspector. Trazida só a chamada, com mais uma condição: a seta fica a meio do arrasto (`pointer_lock_wanted`), porque aqui o cursor ainda não se esconde.

Testes em `o_smart_bone_em_cartoes` (8/8): contagem nos dois sentidos e inversão no piso, Chain, Length a 1 por px, seta na pega e na letra e não no valor. **Antes da cura:** Segments ficava em 4 (esperado 12) e Chain em 2 (esperado 10). **Mutação, 5/5 a sangrar:** sem acumulador · acumulador sem o teste do piso · Segments a cortar · pega sem seta · seta também no valor.

Verdes depois: clippy `-D warnings` em `panel-registry-init`, `app-skeleton`, `editor-core`, `host-desktop`; `the_shell_only_shrinks` (2); `architecture_workspace_file_loc_cap`; `no_magic_numeric`; `ph2d-editor-core cards` 20; `ph2d-panel-widget-gallery cartoes` 38; `nenhum_rotulo` 14; `o_cartao_solto` 14; `number_drag` 29. A prova da seta para na função que decide: **a troca do cursor na janela só se vê no app**.
