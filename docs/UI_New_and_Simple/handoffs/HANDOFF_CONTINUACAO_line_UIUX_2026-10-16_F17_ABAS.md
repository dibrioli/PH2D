# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 16/10 — F17 (abas Drawing/Interface no painel do vetor) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `0aab2c1cb6`. Anterior:
  [F16c](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F16c_TRACEJADO.md).
- Plano e decisões: [`28_plano_F17_abas_desenho_interface.md`](../vetor_nova_ui/28_plano_F17_abas_desenho_interface.md)
  (o porquê e as recusas); pesquisa: [`27`](../vetor_nova_ui/27_pesquisa_ajustes_de_criacao_de_ui.md).

## 1 — Commits desta janela

| commit | o quê |
|---|---|
| `686dbe79ee` | plano 28 |
| `2aacd13ec4` | F17 inteira (abas, interruptor da ①, ordem do Figma, grupo Advanced, tinto, época) — `git show -s 2aacd13ec4` |
| `37283d10e3` | cenas de smoke 25 e 26 |
| `5f6e8949fb` | tinto do palco `0,08 → 0,13` (escuro) e `0,07 → 0,10` (claro): medido nas fotos, o palco ia de `(86,93,109)` a `(93,102,121)`, indistinguível; agora `(98,107,128)` |
| `34dea5481b` | `cargo fmt` nos sete ficheiros da onda |
| `b3539ca4ab` | os dois censos desta onda: `"epoch.{}={e}"` na `NOT_LANGUAGE_TEXT` ao lado do `"theme.{}={}"`; as abas no FORA do `nenhuma_escolha_do_app_e_montada_a_mao` ao lado das do Painter |
| `8ccaeb349d` | régua do fecho do tracejado do Motion com a folga `FOLGA_DO_AJUSTE = 1e-4` — defeito do TESTE, herdado de `4a2776c969` (F16c) |
| `7fed1c82af` | a fixtura de `the_app_never_reshapes_a_still_screen` abre o grupo Advanced: fechado, moldava `1024` = o tecto e o controlo de vacuidade caía; aberto `1130`; mutação da cache a `clear` deu vermelho |
| `84363d7663` | clippy 1.98 `-D warnings` do workspace verde: `chunks_exact_to_as_chunks` ×3, `manual_is_multiple_of`, `ptr_arg`, `type_complexity` ×2 (`action_rows` do herói devolve `struct ActionRows`), `unused_imports` `InteractiveState` em `scroll.rs` (último uso saiu em `d2e190b0c7`), `needless_range_loop` em `seam.rs` |

## 2 — Decisões que um merge/leitor pode partir

- **Abas SÃO o título** (doca mínima `210` px ⇒ `132` px entre borda e X; «Vector» + abas não cabem).
- **Estado** `state::aba()`: thread-local da crate do painel, sessão, nasce `Drawing`. Escrita ÚNICA
  `state::set_aba`, chamada por `paint_tabs::apply_event` para três ids: `VECTOR_TAB_DRAWING`,
  `VECTOR_TAB_INTERFACE`, `VECTOR_VIEW_INTERFACE` (interruptor da ①). ⛔ Recusados: estado no `hero.view` com
  espelho (2.ª porta); chip «Interface ▸» separado (o ponto na aba diz o mesmo).
- **`Onde::{Drawing,Interface,Ambas}`** é o 3.º argumento de `Seccoes::seccao`. Grupo: `Seccoes::grupo`
  (cabeçalho `VECTOR_SECTION_ADVANCED`; membros pintam logo a seguir, pela ordem do artista entre eles).
- ⚠️ **`VECTOR_SECTIONS` é append-only** e ganhou `VECTOR_SECTION_ADVANCED`: 34 secções; o gate
  `every_section_header_is_registered_as_collapsible` conta `34`.
- `COLLAPSED_BY_DEFAULT` = Snap, Advanced, Boolean, States, Morph States. Frame/Clip/Layout/Anchors/
  Component/Widget passam a abrir (na aba deles).
- **Tinto do palco**: `ToolBars::stage_tint: Option<ColorToken>` (campo NOVO no editor-core) e token novo
  `StageTintUi` / `stage-tint-ui` (info com alfa; `tokens.json` forge/sunstone/blueprint + `derive.rs`).
- **Época da ordem gravada**: `WidgetStore::adopt_natural_order(painel, época, ids)` + linha `epoch.<id>=N` no
  `~/.ph2d/sections.txt`; `VECTOR_ORDER_EPOCH = 1`. A ordem real do dono tinha `vector.section.fill/stroke/snap/blend`
  e é fixture de gate.
- ⚠️ **Catracas (DELTA)**: abertura do vetor `887 → 612` (medido); tokens `3116 → 3141` (uma fileira por token).
- ⚠️ **Cenas 25/26 usam os estágios `121..=129`** (o `246` era da cena 24, que tem quatro formas e vai a `247`);
  gate novo `as_faixas_de_estagios_nao_se_cruzam`.
- Os seams das secções de UI usam `crate::host_na_interface()` (`tests/it/main.rs`). Os censos da coluna ①
  aceitam «a aba trocou» como efeito do interruptor. `seleccao_tem_interface` e `ui_states_state` ficaram
  `pub` (condições de espera da cena).
- Sem bump de `PROJECT_SCHEMA`: a aba é sessão; a época vive no `sections.txt`, fora do projeto.

## 3 — Smoke (por fazer pelo dono)

Cena 25: `PH2D_OBJECT_MODE_SMOKE=25` — botão RoundRect azul feito componente pelo *Create* do painel, pele
Button, estados Default+Hover gravados; estrela amarela ao lado; aba Drawing com o ponto aceso na Interface.
Cena 26: `PH2D_OBJECT_MODE_SMOKE=26` — a 25 + o clique no interruptor da ① ⇒ aba Interface, palco tingido.
Comando (precisa do binário `smoke` compilado): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=25 ./target/smoke/ph2d-host-desktop` (idem `=26`). Errado = a cena abre sem o ponto na aba ou, na 26, sem o palco tingido.
Fotos: [25](../vetor_nova_ui/fotos/F17_cena25_aba_drawing.png) · [26](../vetor_nova_ui/fotos/F17_cena26_aba_interface.png).
Notas: na 25 o cabeçalho ADVANCED fica abaixo da dobra do painel com um objecto seleccionado (rola); o botão
vestido desenha-se como botão do tema com o nome «Path 0 (1)» — a cor azul só aparece no painel e na barra
(lei da pele de hoje, não desta onda). Os `.log` irmãos das fotos ficam não rastreados.

## 4 — Verificação

- Gates novos: `a_ordem_natural_e_a_do_figma`, `cada_seccao_vive_na_aba_certa` (lib); `as_abas_do_titulo` ×4
  (seam pelo ponteiro real: `cada_aba_pinta_so_as_suas_seccoes`, `as_duas_portas_escrevem_a_mesma_aba`,
  `o_grupo_avancado_dobra_os_sete_juntos`, `a_ordem_gravada_do_dono_cede_a_ordem_do_figma_uma_vez`);
  `a_epoca_esquece_a_ordem_gravada_uma_vez` (editor-core lib); cenas: `as_cenas_25_e_26_comecam_nas_abas`,
  `as_faixas_de_estagios_nao_se_cruzam`, `vector_mode_interface_tests` ×3.
- **Prova de mutação: 8/8 sangraram** (filtro sem a aba; membros fora do grupo; `on` do interruptor fixo; tinto na
  aba errada; época sem o teste; época sem gravar; paint sem adopt; Component fora do topo).
- `ph2d-panel-vector --test it` 266+ verdes; `ph2d-tokens`, `ph2d-i18n` verdes; `cargo check --workspace --tests`
  limpo; clippy do workspace verde; fmt limpo.
- Suíte de fecho, 1.ª corrida: `20 473/20 479` (6 falhas): 4 curadas (`b3539ca4ab` ×2, `8ccaeb349d`, `7fed1c82af`);
  2 flakes de carga sem diff da linha — `ph2d-timeline` `motion_path_perf` e `ph2d-light2d-gpu` `g4` (passam sozinhos;
  ver [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).
- 2.ª corrida: <a preencher>

## 5 — ABERTO

1. Smoke do dono das cenas 25/26 (e da 23, ainda sem confirmação explícita).
2. `CLAUDE.md` §5.1 UI/UX: já aponta para este handoff («F17, na linha 16/10»); na integração, trocar a etiqueta
   para «no main» com a data.
3. A placa só percorre UM par do tracejado (herdado da F16c, item 4 do handoff anterior).
4. O achado do profiler global das sondas do Motion (item 5 do anterior, do `ph2d-gpu`).
5. A pele Button desenha o botão do tema por cima da cor autorada — ver a cor do Hover no canvas é outra onda.
6. A ordem do Figma não tem ainda grupos visuais além do Advanced (Componente/Posição/… são só ordem).
