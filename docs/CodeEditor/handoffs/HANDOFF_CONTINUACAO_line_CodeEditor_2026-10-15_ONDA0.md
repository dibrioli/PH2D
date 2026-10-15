# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 0 fechada (15/10/2026)

> Troca de janela, **não** integração. Leitor: a próxima LLM. Recomendação: abrir **janela NOVA** (CLAUDE.md §2: uma janela por onda) e começar por `/pd-linha-assumir` (FASE 0 do [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md): `cd` + `pwd` + `git branch --show-current` ANTES de ler código).

## 1. Identidade

- Worktree `Worktrees/line-CodeEditor` · branch `line/CodeEditor` · merge-base `0aab2c1cb6` · HEAD `ebd135c70b` (antes deste handoff) · 22 commits.
- **Não integrada e NÃO integrável sozinha.** A Onda 0 tira o caminho do editor externo (o script mora no projeto, [ADR-0188](../../architecture/decisions/0188-o-editor-de-scripts-mora-no-ph2d-e-o-script-mora-no-projeto.md)) e o editor interno só chega na Onda 1: integrar a Onda 0 só é regressão de produto. Integração só por ordem do Enio, depois da Onda 1.

## 2. Decisões do dono (15/10) — [00_decisoes_do_dono.md](../00_decisoes_do_dono.md)

Script dentro do projeto · `luau-lsp` empacotado · IA desligada até a ÚLTIMA onda (depois, a mais completa possível) · scripts podem referenciar outros objetos (arrastar da Hierarchy) e tudo o que facilite. Quem mudar uma pergunta ao dono antes.

## 3. Feito: Onda 0, T1–T7

A tabela por item (commits, desvios, achados medidos) e as Medições (T4) vivem em [01_onda0.md](../01_onda0.md) §Estado e §Medições — não se copiam aqui. Correções do gate depois da suíte impactada:

| Commit | O quê |
|---|---|
| `f172709530` | o aviso de import chega ao ecrã como toast (chaves i18n `app.components.script_project.*`) |
| `af86fbbe2a` | `ime` é dono da `Composition` (DAG de módulos do editor-core) |
| `5525476208` | `scene_library_tests.rs` cortado do `scene_tests` (teto 700) |
| `289051a95d` | `ph2d-a11y` `NodeBuilder::value`; o valor a11y do `TextInput` inclui a composição |
| `7ca6e49e41` | `api_table`: campo `name`→`ident`; censo dos rótulos do motor |

Fixtura do smoke: `ebd135c70b` [`script_quebrado.luau`](../ferramentas/smoke/script_quebrado.luau).

## 4. Estado dos gates

- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` verde depois das correções.
- `nextest-impacted`: 18588/18593 verdes; os 5 vermelhos foram corrigidos e re-corridos verdes **POR NOME**. **A suíte inteira NÃO foi re-corrida depois das correções** — corra-a no próximo ponto de cadência ou no fecho (`PH2D_SUITE=fecho`).
- `censos-da-arvore-combinada` 116/117 — o 1 vermelho era o #1 (corrigido). `the_shell_only_shrinks` verde.
- **Shell: delta ≈ +220 linhas** (T6 +50, T7 +217 com 139 de teste, +3 do toast); `TETO_LOC` 189 041. As ondas seguintes põem código em CRATES (plano: `ph2d-app-code`, `ph2d-code-buffer`, `ph2d-code-syntax`; `ph2d-code-lsp` já existe), nunca na shell.

## 5. Superfície foundational tocada (para o handoff de integração futuro)

`ph2d-editor-core` (módulo `ime`, composição no `WidgetStore`, pintura e a11y do `TextInput`) · `ph2d-a11y` (`NodeBuilder::value`) · `ph2d-text` (fonte mono, `mono_contexts`, arrays `UI_FONTS`) · `ph2d-tokens` (`FONT_MONO`) · `ph2d-i18n` (chaves) · `shells/desktop` (roteio de IME, `set_ime_cursor_area`, `PROJECT_SCHEMA` 189→190 + braço 189, `ProjectFile.scripts`, toast) · `tools/ph2d-bindgen` (metade Luau vinda da `api_table` do `ph2d-script`; agora compila o Luau C++) · comentário em `.github/workflows/spike.yml` · `Cargo.lock` (crates novas `ph2d-script-doc`, `ph2d-code-lsp`).

**Números provisórios a RECONTAR na integração** (CLAUDE.md §5.0: soma-se, não se escolhe): ADR-0188 (0187 já é da `line/MiroClone`) · `PROJECT_SCHEMA` 190 (o braço 189 depende de `scripts` ser o ÚLTIMO campo do `ProjectFile` — não acrescente campo depois dele sem rever o braço).

## 6. ABERTO — carregado para a Onda 1 ou depois

- Seletor de scripts já existentes na biblioteca (digitar o nome funciona).
- Re-Browse do mesmo ficheiro cria «bob 2.luau».
- Sem UI para apagar scripts sem uso; o Inspector ainda diz «file».
- IME: o TextEdit do Board não recebe `Commit` em X11/IME (`board_keys.rs`, da MiroClone); texto vetorial sem composição visível; Combobox/NumberInput sem composição visível; X11+XIM engole a tecla morta (sem acento visível ali).
- `pcall` pode apanhar o erro de orçamento (Onda 4: orçamento de instruções). Falhas de `ph2d.set` (campo desconhecido / não numérico) não levam linha.
- Catálogo MCP: `attach_script` ainda descreve bytecode. `drain_spawns` sem chamador. `docs/design/tokens.json` com `font-mono` velho. `ph2d-text` tem um `tests/` solto citado noutros sítios.
- `luau-lsp`: solver novo ≈7× mais lento a 3 000 linhas ⇒ ficar no antigo, re-medir a cada versão fixada. Completion logo após tecla custa uma rechecagem ⇒ pedir assíncrono; hover só na pausa do rato.

## 7. Próximo: Onda 1

Definição em [relatório §6](../reports/Editor%20de%20scripts%20estado%20da%20arte.md) (Onda 1): aba de documento + botão «Edit script» no cartão Script, buffer, layout por linha com cache, calha, tree-sitter-luau, buscar/substituir, undo agrupado, salvar → recarga direta, console, captura de teclado no topo da cadeia, AccessKit por linha, `PH2D_CODE_SMOKE`. **Mais** os itens movidos da Onda 0: fixturas do oráculo Godot (auto-indent/auto-brace) e a bancada tecla→pixel. **Primeiro considere devolver JÁ um caminho de edição** (o editor interno em si), porque a regressão da §1 só fecha com ele.

## 8. Smoke (binário já compilado no worktree, `--profile smoke`; 2.º build: «Finished in 0.23s», zero Compiling)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_SCRIPT_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Selecione o objeto com o cartão Script; o `bob.luau` já vive dentro do projeto (sem ficheiro solto).
3. Deve ver o erro do script com a LINHA no cartão (usar a fixtura `script_quebrado.luau`); num campo de texto, `´` + `a` mostra o acento sublinhado e insere `á` UMA vez.
4. Deu errado se: o cartão mostra só uma frase cortada sem linha, o script aparece vazio, ou o `á` entra duplicado.

## 9. Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`, colado verbatim)

```
  ✗ paralelismo de ferramenta              1.16/passo   alvo: >= 1,5  (11% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                132   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                384 : 144   alvo: <= 1,0  razao 2.7x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.0%   alvo: <= 2%  (8 de 837; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         285 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               64 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## Retorno do smoke do dono (15/10)

- Erro de `script_quebrado.luau` lido no cartão: `line 3 · Expected identifier when parsing expression, got '='`. Está CORRECTO (a 2.ª `=` de `local x = = 1` é o símbolo inesperado), mas o dono: *«não gostei do = pois é confuso para os olhos»*. ⇒ **Onda 1/2, primeiro item de mensagens:** (1) o símbolo culpado nunca vai cru entre aspas simples no meio da frase — vai destacado (chip/realce com tokens) e, no editor, sublinhado no lugar exacto da linha; (2) as mensagens mais frequentes do compilador ganham texto escrito à mão em `ph2d-i18n` (ex.: «Something is missing before this =»), com a mensagem crua do Luau ao dispor no hover/console. Medir quais são as frequentes no registo de códigos de erro (plano §6 Onda 2).

## Nitidez do texto (dono 15/10: «pode aplicar a mesma nitidez» · «aumentar um pouco o tamanho da font da mensagem de erro»)

- Causa medida (`123d126168`, sonda `ph2d-app-components/tests/it/mede_as_duas_barras_do_igual.rs`): o preset de fábrica `CrispHeavyPlus` pintava com hint OFF; e com hint ON o vello 0.10 (`Engine::AutoFallback`) corria o interpretador TrueType sobre o `prep` inerte de 7 bytes (só dropout) do Noto/Atkinson/JetBrains Mono — nada fazia.
- Cura: `af019f98fc` tira o `prep` inerte no registo (`ph2d_text::without_prep_table`, recusa fontes com `fpgm`/instruções; avanços bit-idênticos; `read-fonts` 0.41.0 já estava no lock) · `6df92e82c2` `CrispHeavyPlus.hint = true` (o `CrispHeavy` do menu fica sem hint: é a identidade dele) + gate de placa `as_duas_barras_do_igual_tem_o_mesmo_pico_no_preset_de_fabrica` (tolerância 3 %; antes 5,2–28,3 %, depois ≤ 1,6 %; ⚠️ sem adaptador imprime «no adapter» e não mede — no CI só corre o gate de CPU `every_registered_font_reaches_the_autohinter`) · `679ce98a93` o erro Luau do cartão Script sobe `TypeToken::Sm → Base` (`rows::aviso_em`, gate `o_aviso_num_degrau_acima_cabe_na_altura_que_devolve`). Peso medido: tinta −3,3 % a +5 %, stems verticais ±2 % (`docs/CodeEditor/ferramentas/medicoes/peso_hint.txt`).
