# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I15 (19/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; I6/I11, I7, I8, I8b, I10, I9+I12+I14 e [I13](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md)
> estão na linha; este vem por cima. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha `| I15 |`) ·
> medições: [I15_bancada.md](../ferramentas/medicoes_onda2/I15_bancada.md) e
> [I15_solver.md](../ferramentas/medicoes_onda2/I15_solver.md). O mecanismo vive nesses; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` `100d34c8e0` · HEAD `aca1bc0ed1` (antes do commit deste doc) · 61 commits da linha.
- Commits da onda (sobre `42fef754b6`), **3**: `1fc8801149` feat (cena `PH2D_CODE_SMOKE=3` conduzida e
  fotografada) · `944dff18b0` test+docs (verificador antigo × novo sobre os scripts reais — fica o antigo) ·
  `aca1bc0ed1` test+perf (a bancada completa com o `luau-lsp` fixado + as três curas). 35 ficheiros,
  +1582 −52 (16 deles são fotos).

## 2. Superfície de colisão

`bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh` (linha inteira
contra o `main`):

```
merge-base 100d34c8e0 · 61 commit(s) · 317 arquivo(s)
PROJECT_SCHEMA 191 = base · tripla (191, 13, 23) = base · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20 = base
registo de componentes: ph2d-ecs 108 · render 109 · script 109 = base
contrato congelado (§6): intocado · ADR: nenhum (próximo livre 0189)
Cargo.lock: 2 pacotes novos na linha — ph2d-code-explain, ph2d-code-format (de ondas anteriores)
marcadores de conflito: nenhum · tectos de LOC: nenhum ficheiro da linha passa do teto
```

**Contadores como DELTA desta onda:** schema/registo/ADR **0**; **cena de smoke +1** (`PH2D_CODE_SMOKE`
2 → 3; contador só desta família); `Cargo.lock` **+1 linha** (a dev-dep `ph2d-app-code` em
`ph2d-app-components`, sem pacote novo); comandos da `TABLE` **0**.

**Foundational tocado:** nenhum (`ph2d-editor-core` e a shell intocados). Toca `ph2d-panel-code/src/glyphs.rs`
(+9 −3: o ponto da calha por `partition_point`; `frame.problems` tem de vir ORDENADO por linha, com `debug_assert`).

## 3. O que a onda fez (resumo; mecanismo nos docs linkados)

- **Cena 3** (`1fc8801149`): `onda.luau` com `self.ampiltude` de propósito ⇒ as formas param por erro do motor; a
  aba abre e a STAR vem seleccionada com amplitude própria 2. **Porta das fotos**
  `PH2D_CODE_SMOKE_PASSO=<0..7>` (`ph2d-app-code/src/smoke_drive.rs`): conduz o roteiro pelas teclas REAIS da aba
  (`DocumentSurface::key`, `go_to`), **uma letra por quadro** — medido: `ph2d.` escrito num só quadro depois do
  Enter numa linha de comentário lia-se como comentário (árvore velha) e a lista nunca abria; cada passo acaba
  num quadro. `PH2D_CODE_SMOKE_FOTO=cena` traz a aba Scene para a frente no fim.
- **Gate** `a_cena_3_conduzida_passo_a_passo` (`ph2d-app-components/tests/it/a_cena_3_do_editor_conduzida.rs`):
  o mesmo roteiro sobre a cena montada como a shell; confere formas paradas · «Did you mean amplitude?» · lista =
  toda a API `ph2d` · snippet `ph2d.set(entity, "x", value)` · rotação · caixa do rename «keeps the value of
  1 object» · `height` = 2 próprio da STAR no instantâneo do Inspector.
- **Fotos** (`docs/CodeEditor/ferramentas/smoke/cena3/`): `passo_0..7` na aba, `cena_0/2/5/7` na aba Scene.
- **Solver antigo × novo** (`944dff18b0`, sonda `solver_scripts_reais` em `ph2d-code-lsp`): 12 scripts reais × 2
  modos. O novo é 4,4× mais lento nos diagnósticos (pior p50 `2,7–3,1` × `0,6–0,7` ms), ACUSA código válido no modo
  normal e troca as classes `1030` do estrito por `1039` (em duplicado) e `1041`, sem explicação escrita ⇒
  **fica o antigo** (critério de morte do plano). `ServerConfig::new` mantém `Solver::Old`.
- **Bancada completa** (`aca1bc0ed1`): célula `Lista::Real` (`luau-lsp` fixado, lista + assinatura + cores + dicas,
  afirma as duas janelinhas por tecla): p50 `1,41–1,80` ms (51 linhas) e `1,66–2,08` (7 500), p99.9 ≤ 9,7 — dentro
  de um quadro. Pintura `+0,17` ms a 7 500 linhas (293 diagnósticos) que as curas NÃO moveram (§9).
- **Curas** (`aca1bc0ed1`): ponto da calha por `partition_point` sobre `problems` ordenado por linha; globais
  desconhecidos extraídos 1× por quadro e cortados pela busca; tokens semânticos ORDENADOS na chegada (o
  `partition_point` de `spans` assumia a ordem do servidor).

## 4. O que um MERGE pode partir

- **`CENAS` 2 → 3** em `ph2d-app-components/src/code_smoke.rs` e `ph2d-app-code/src/smoke.rs` (conferidos em
  teste): outra linha que mexa em `PH2D_CODE_SMOKE` conflita. **CLAUDE.md §5.1**, a entrada do CodeEditor diz
  `PH2D_CODE_SMOKE=1|2` ⇒ tem de passar a `1..3` na integração (a linha não editou o CLAUDE.md).
- **`ph2d_app_code::smoke_drive`**: módulo `pub` novo; `frame()` chama `smoke_drive::frame(docs)` e, só com as
  variáveis de ambiente, pode fazer `hero.documents.activate(None)` no fim do quadro.
- **`semantic::spans`** mudou de assinatura: `unresolved: &[Range<usize>]` em vez de `&[Diag]` (chamadores fora da
  linha quebram).
- **Dev-dep nova** `ph2d-app-code` em `ph2d-app-components` (`Cargo.toml` + `Cargo.lock`); a shell compõe as
  duas, a dependência é só de teste.
- **`ProblemsFrame.problems` ordenado por linha** (`ph2d-panel-code/src/glyphs.rs`): outro produtor de `Problem`
  que não ordene reprova o `debug_assert`.
- `code_smoke::ONDA_LUAU` é script novo e entra no gate `os_scripts_da_cena_saem_limpos_do_formatador`.

## 5. Testes

- Gate da cena **1**: `a_cena_3_conduzida_passo_a_passo` (acima) · `os_scripts_da_cena_saem_limpos_do_formatador`
  (agora com `ONDA_LUAU`) · unitário do roteador do smoke (`3 → Some(3)`, `4 → None`) · cores **7/7** verdes.
- Sondas `#[ignore]`: `solver_scripts_reais` (`ph2d-code-lsp`) e `bancada::tecla_pixel` (`ph2d-app-code`).
- **Mutação: não corrida. Clippy: não corrido** (só no fecho da linha).
- Corridas: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/luau-lsp-fetch.sh && bash scripts/cargo-test-narrow.sh ph2d-app-components a_cena_3` ·
  `… bash scripts/ph2d-run.sh cargo test -p ph2d-code-lsp --profile smoke --test it solver_scripts_reais -- --ignored --nocapture` ·
  `… PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`.

## 6. Portões da onda (19/10)

Só os vizinhos por nome do §5. **Não correram:** clippy, a suíte `nextest-impacted`, a mutação — tudo no fecho da
linha.

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.33s` (zero `Compiling`).

## 8. Smoke para o DONO

> Precisa do assistente instalado: `bash scripts/luau-lsp-fetch.sh` (já está no cache desta máquina). As fotos de
> cada passo estão em `docs/CodeEditor/ferramentas/smoke/cena3/` (`passo_0.png` … `passo_7.png` na aba do código;
> `cena_0.png`, `cena_2.png`, `cena_5.png`, `cena_7.png` na aba da cena).

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke`. Abre a aba **onda.luau** e a **Star** vem selecionada. Clique na aba **Scene**, lá em cima: as formas estão **PARADAS** (o script tem um erro de digitação); o quadrado cinzento não tem script. Volte em **onda.luau**; no console aparecem as linhas vermelhas do erro (o botão **Clear** limpa).
2. Na linha 21 a palavra «ampiltude» tem uma linha ondulada e um pontinho na margem. Clique dentro dela e carregue **Alt+Enter**: abre um balão com o botão **Did you mean amplitude** e uma prévia. **Enter** aplica. **Ctrl+S**. Na aba **Scene** os três sobem e descem, e a estrela sobe o dobro (é o valor próprio dela, 2).
3. No fim da linha 22 (`-- 2) escreva aqui…`) carregue **Enter**, escreva `ph2d.` e espere: abre a lista de tudo o que o `ph2d` sabe fazer, com a explicação ao lado. Escreva `se`: o `set` sobe ao topo; **Enter** → fica `ph2d.set(entity, "x", value)` com `entity` selecionado e um balão com a ajuda dos argumentos. Escreva `self.id`, **Tab**, `rotation`, **Tab**, `self.t`, **Tab** (se abrir uma lista enquanto escreve, **Enter** escolhe). **Ctrl+S**: na aba **Scene** a estrela e o hexágono giram (o círculo gira também, mas é redondo, não se nota).
4. Clique dentro de `"amplitude"` na linha 9 e carregue **F2**: aparece uma caixinha «Enter to rename · keeps the value of 1 object». Escreva `height` e **Enter**: a linha 9 e a linha 21 mudam juntas. No **Inspector**, abra a secção **Script**: a estrela mostra `height` = 2 como valor próprio dela.
5. **Deu errado se:** a lista não abre depois de `ph2d.`; o Alt+Enter não oferece nada; depois de renomear o valor da estrela volta a 1 ou o Inspector ainda diz `amplitude`; as formas não mexem depois do Ctrl+S.

## 9. ABERTO

- A secção **Script** do Inspector expandida **não foi fotografada** (o ecrã virtual ignora cliques sintéticos); o
  gate confere o instantâneo.
- **Pintura `+0,17` ms** a 7 500 linhas com 293 diagnósticos / 297 dicas (51 linhas: 0,65–0,81; 7 500: 0,82–1,01):
  as curas da calha, dos globais e das cores não a moveram (0,822 → 0,847). Suspeita, **não medida**:
  `CodeDoc::problems()` reconstruído e ordenado a cada quadro (`doc.rs` ~359); a fusão em `problems.rs` ~217–262
  é O(D×R). Medir (fases por quadro) antes de curar.
- A sonda do solver **não passou `--docs`** (só mudaria o texto da documentação, não os diagnósticos).
- A regra de Fatin nas caudas (p99/p99.9) reprova ora numa corrida ora noutra (ruído do par 0,6–1,0 ms), como já
  no I1; o p50 e o limite do quadro são a régua.
- Os herdados do [handoff do I13 §9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md#9-aberto)
  continuam (abas fechadas com ● sem formatar; «Format on Save» por sessão; avisos das deps do StyLua; o CI sem
  fetch dos dois binários).
- Integração: falta `clippy --workspace` e a suíte do fecho; CLAUDE.md §5.1 `1|2` → `1..3`.

## 10. Próximo

**Acabamento** (item 4 do plano): o ponteiro em **mão** com Ctrl carregado + sublinhado sobre o que o Ctrl+clique
alcança (a MESMA regra do `goto.rs`: sítio no script · `self.<propriedade>` · documentação da API); e a **lista de
problemas** (painel que lê o MESMO `Vec<Problem>` da calha, cada linha com a explicação do I8 e a linha do script,
clique leva o cursor, leitor de ecrã). Depois: smoke do dono e `/pd-linha-fechar` (clippy + suíte + mutação).
