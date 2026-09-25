# Integração da rodada 03 — `line/Vector` · `line/components` · `line/3DModeling` (2026-09-25)

> **Registo de UMA rodada.** Descreve o mundo no dia em que foi escrito; o estado vivo é o
> `CLAUDE.md` §5. Leitor: a próxima LLM integradora, ou quem bissecar um defeito que atravesse
> estas três linhas.

## §1 — O que entrou, e em que ordem

Rodada de **sete** linhas em três integradores. As quatro primeiras (UIUX, PainterWatercolor,
sculpt3d, motion-value) já estavam no `main` (`ebce91f83`) quando esta começou. Esta integrou as
outras três em **cascata**, na ordem medida pela sobreposição (a Vector é a que menos toca; a
3DModeling é a que mais colide com o que a UIUX e a sculpt3d já tinham trazido):

`main (ebce91f83)` → `line/Vector` (20 commits) → `line/components` (54) → `line/3DModeling` (112)

Total: **186** commits em `main..line/3DModeling`, **751** ficheiros, `+85 571 / −4 285`.
Chegam ao `main` por um único `git merge --ff-only line/3DModeling`.

Handoffs das linhas (os únicos lidos, por ordem do dono):
[Vector](../../Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_O_CAMPO_E_A_PESQUISA_2026-09-24.md) ·
[components](../../Components/handoffs/HANDOFF_INTEGRACAO_line_components_VIDA_2026-09-25.md) ·
[3DModeling](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-25.md).

## §2 — ⚠️ Os commits intermédios NÃO compilam sozinhos

O rebase fundiu TEXTO limpo sobre o `main` com a UIUX dentro, e o código novo das linhas falava a
API do Inspector de ANTES da refatoração dela. As curas estão todas nos **seis commits de
integração no topo** (`ae052fac2..a081994fd`). ⇒ **um `git bisect` que caia num commit entre
`ebce91f83` e `ae052fac2^` vai ler «não compila»** — isso é a rodada, não o defeito procurado.
Use `git bisect skip` nessa faixa.

## §3 — Os números que SOMAM (contados, nunca escolhidos)

| Contador | `main` | Depois | Delta e origem |
|---|---|---|---|
| `PROJECT_SCHEMA` | `160` | **`176`** | components `161–171` (+11), 3DModeling `172–176` (+5, **renumerados** com «(era N na linha)») |
| escada do schema | — | 6.ª faixa | `v144..v160` arquivados em `project_schema_history_v144.rs` (a fronteira é o `160`, o `main` de que a rodada nasceu) |
| registo `ph2d-ecs` / espelhos render·script | — | `108` / `109` | components +4, 3DModeling +1 (`Mesh3D`) |
| registo da FÍSICA | `37` | `40` | components +3 (o `collision-surface.sh` **não** o mostra) |
| `LIVE_SECTIONS` | — | `43` | components +4, 3DModeling +1 |
| casos de costura | — | `27` | — |
| cena do catavento | `=52` | **`=53`** | colisão MUDA com a tinta fina da sculpt3d (ver §4.1) |
| `nenhum_rotulo_do_app_pinta_nada` «inspector» | `85`/`81` | `87`/`83` | `Keep on Restart` · `Only This Object`; **medido** (a catraca tem as duas metades e fecha verde) |
| altura de abertura do painel da escultura | `2 051` | `2 186` | `+41` Bake Law, `+44` Lens, `+50` as duas pistas de lâmpada (conta fechada no gate) |

## §4 — O que só a árvore COMBINADA mostrou

1. **`=52` escrito pelas duas linhas para cenas diferentes** (`ae052fac2`): o `CENAS = 52` era o
   MESMO literal nos dois lados, e o rebase fundiu-o limpo com dois predicados a responder
   `Some("52")`. Renumeradas só as 48 linhas que a 3DModeling acrescentou.
2. **A costura do Inspector** (`302d9eaed`): `tween::warn → rows::aviso`, `BTN_H →
   caixa_do_botao`/`abaixo_do_botao`, a `Seccao` medida nas chamadas que a pedem, e os campos de
   texto da vida ganham rótulo (a porta `text_row` deixou de aceitar um campo só de dica).
3. **Duas catracas por ACUMULAÇÃO**: a escada do schema (`688` contra `700` → 6.ª faixa,
   `d7eecb0da`) e `the_shell_only_shrinks` (`198 208` contra `196 990`). Esta curou-se a MOVER
   (`ec27cd326`): duas crates-folha novas, **`ph2d-timeline-persist`** (a timeline dentro do
   projecto) e **`ph2d-sheet-bounds`** (confinar uma peça à folha) — shell a `196 742`. ⚠️ O
   `timeline_orphan_tests.rs` FICA na shell (exercita uma cena dela, HOWTO §2.6).
4. **Oito vermelhos de costura do `nextest-impacted`** (`a081994fd`), nenhum de lei: um tecto de
   LOC (`panel.rs` da escultura → `panel_pontes.rs`), uma função da shell a `203` linhas
   (`frame_gfx::of`), o gate do shader com o endereço velho, a secção LIVE MESH **sem gate que a
   pintasse** (nasce `a_seccao_live_mesh_esta_viva`), três rótulos com a regra no nome (dois eram
   PLACEHOLDERS escolhidos por função — a régua ganhou a 2.ª porta de placeholder), oito caixas de
   marcar da vida pela porta `paint_check_row`, e a tabela `PORTAS` do Inspector armado.
5. **O índice da memória** (`f41b38c1a`): a união das linhas passou o tecto (`22 080` contra
   `22 000`) com sete contagens de família paradas; recontadas, `21 872` bytes.

## §5 — Portão da árvore combinada (ponta `a081994fd`)

- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` — limpo.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero avisos.
- `BASE=main nextest-impacted` — **19 278 / 19 278** verdes, zero TIMEOUT/SIGSEGV.
- `censos-da-arvore-combinada.sh` — **127 / 127** verdes (controlo do filtro `12 de 12`).
- Os dois gates que leem o `CLAUDE.md` (`architecture_docs_*`) — `6 / 6`.
- A bateria de GPU (`#[ignore]`, o CI **não** a corre) do `ph2d-mesh-render` + `ph2d-form-pbr`,
  com adaptador: `221 / 223` à primeira. ⛔ **Os dois vermelhos vinham da PRÓPRIA linha**
  (`probe_wire_continuity`: o arame do miolo a `62 %` contra `70`, o rasante a `57,4 %` contra
  `78`) — o `main` lê `9 / 9` e a ponta ORIGINAL da `line/3DModeling` (`4c4c18ce7`, antes do
  rebase) lê os MESMOS dois números. A causa: a sonda tirava a luz do `Shade::default()`, e a
  linha trocou a luz de fábrica de `Matcap(0)` para `Pbr` (`c185de0a3`). Com a luz escrita por
  nome volta a `9 / 9` — cura no commit do portão, com o porquê no fonte.

## §6 — Resíduo de processo

- Um bloco `merge=text` temporário em `.git/info/attributes` (contra o Mergiraf a apagar
  remoções em ficheiros de contagem) foi **retirado** no fim.
- ⏳ **ABERTO e nomeado (da `line/3DModeling`):** a sonda do arame prova a CONTINUIDADE em
  profundidade sobre o matcap; **ninguém mediu se o arame se LÊ tão bem sobre a luz PBR de
  fábrica** — a régua `ink_in` é um contraste de luma, e sobre a superfície PBR rasante o mesmo
  arame lê `57–62 %` de tinta visível. É outra pergunta, com outra régua, e é da linha.
- Os abertos de cada linha continuam nos handoffs delas (a foto do dono de 25/09 da 3DModeling;
  *«não vejo a bala»* na `=2` da vida).
