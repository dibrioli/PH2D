# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-17 — F16–F17 (barra de controle, tracejado em pares, abas Drawing/Interface)

- Branch `line/UIUX` → `main` por `--ff-only`, tip `994950ef5f` (36 commits, nenhum merge commit). Merge-base
  antigo `0aab2c1cb6`; `main` de partida `db535d2a10` (a integração do Motion `=139`, 54 commits). Etiqueta de volta:
  `backup/UIUX-pre-integ-2026-10-17` (tip antigo `6cf2883cc5`). **Sem push.**
- Os 36 commits incluem também a F15 (Arrange/Align/Live Boolean na barra ④, cena 22), que nunca foi integrada à parte.

## 1 — O que entrou

| onda | o quê | handoff da linha |
|---|---|---|
| F15 | Arrange/Alinhar saem do painel para a barra ④; a ④ deixa de se esconder; interruptor e verbo da booleana viva | [F15](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F15_BARRA_DE_CONTROLE.md) |
| F16 | a fila ② vira a barra de controle do Illustrator (o traço); cena 23 | [F16](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F16_BARRA_DE_CONTROLE.md) |
| F16c | tracejado em até três pares e seta por ponta (`StrokeSpec`), fecho do padrão, Inner/Outer, painel do traço que rola (roda), cena 24 | [F16c](HANDOFF_CONTINUACAO_line_UIUX_2026-10-15_F16c_TRACEJADO.md) |
| F17 | painel do vetor em abas Drawing/Interface no título, interruptor no fim da ①, ordem do Figma, grupo Advanced, época da ordem gravada; cenas 25 e 26 | [F17](HANDOFF_CONTINUACAO_line_UIUX_2026-10-16_F17_ABAS.md) |

## 2 — Como se integrou

- `collision-surface.sh` (caminho absoluto): único ficheiro tocado pelos dois lados = `CLAUDE.md` (§5.1: Motion vs UI/UX,
  linhas diferentes). `git rebase main` **sem conflito**: 36/36, nenhum «Solved», nenhum marcador (varrido, incluindo `|||||||`).
- **Prova do rebase:** `git range-diff 0aab2c1cb6..backup main..HEAD` = 36 linhas `=`; e o multiconjunto `+/-` por ficheiro de
  cada commit, rebaseado contra original: **0 diferentes** em 36 (controlo positivo: commit 0 contra commit 1 difere).
- **Schema (reconta sobre o main novo):** o Motion não tocou no `PROJECT_SCHEMA` (`main` = `189`). A linha: `189 → 190`, tripla
  `(190, 13, 23)` (`VEC_SCENE` `22 → 23`), constante irmã `PROJECT_SCHEMA_TRACO_V22 = 189` (o `match` de `project_load_read` lê
  `< 190` pela forma antiga). Sem conflito ⇒ o `schema-recount.py` (que corre dentro do conflito) não teve o que contar; conferido à mão
  nos três sítios e nos dois lados. Registos de componentes: `108`/`109`/`109` iguais ao `main`. Nenhum contrato §6 tocado, nenhum ADR.
- **Censos da árvore combinada** (`censos-da-arvore-combinada.sh`): 118/118 verdes, controlo 12/12 censos.
- **`foundational-integrate.sh`:** re-sync dos geradores sem diff (nenhum commit `chore(sync)`), staleness verde,
  `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` verde, `nextest-impacted` com `PH2D_SUITE=fecho`:
  **20 503 / 20 503 verdes** (782 s, 8 690 saltados, 16 lentos, zero falhas, zero flakes — a linha sozinha dava 20 479; +24 são do Motion),
  incluindo os arch-gates da shell (`the_shell_only_shrinks`, `file_loc_caps`, tecto de LOC do workspace, `no_orphan_source_file`).
- Um tropeço do script: ele recusa árvore com ficheiro NÃO rastreado; os quatro `.log` das fotos (`fotos/F16_…`, `F16c_…`, `F17_…`)
  foram tirados para fora durante o gate e repostos. Nunca se comitam.

## 3 — ABERTO (copiado do §5 da F17, mais o que a integração acrescenta)

1. **Smoke do dono, pendente: cenas 23, 25 e 26** (a 24 e o painel do traço numa janela baixa já aprovados em 16/10).
   Comando: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_OBJECT_MODE_SMOKE=<23|25|26> ./target/smoke/ph2d-host-desktop`
   (binário `smoke` a compilar no primário). 25 = botão azul feito componente, aba Drawing com o ponto aceso na Interface;
   26 = a 25 + clique no interruptor da ① ⇒ aba Interface, palco tingido. Errado = sem o ponto/sem o palco tingido.
2. A placa só percorre UM par do tracejado (2–3 pares caem na cerca `FLAG_SO_CONFORME`; estender o shader é onda própria, medida).
3. O achado do profiler global das sondas do Motion (item 5 do F16c, do `ph2d-gpu`).
4. A pele Button desenha o botão do tema por cima da cor autorada — ver a cor do Hover no canvas é outra onda.
5. A ordem do Figma não tem ainda grupos visuais além do Advanced (Componente/Posição/… são só ordem).
6. Decisão humana nenhuma pendente da integração.
