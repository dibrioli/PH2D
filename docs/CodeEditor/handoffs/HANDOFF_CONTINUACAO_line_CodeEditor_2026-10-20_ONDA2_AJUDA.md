# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: a Página de Ajuda (20/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; tudo desde I6 está na linha, e o [Acabamento](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_ACABAMENTO.md)
> foi o anterior; este vem por cima.
> Plano, oráculo e achados: [04_pagina_de_ajuda.md](../04_pagina_de_ajuda.md) (**§9 = achados do runtime e da API**) ·
> fila: [03_onda2.md](../03_onda2.md) §Próximo passo (item 5) · bancada:
> [ajuda_bancada.md](../ferramentas/medicoes_onda2/ajuda_bancada.md). O mecanismo vive nesses; aqui só o resumo.
> **Smoke do dono: PENDENTE.**

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` `100d34c8e0` · HEAD `e4e3158f1e` (antes do commit deste doc) · 79 commits da linha.
- Commits da onda (sobre `c02adf6bff`), **6**: `b1969ae584` (oráculo Godot + sonda do LSP + plano `04`) ·
  `9e0154a2c5` (o catálogo: crate `ph2d-code-help`) · `126579ac59` (a vista no painel, `help/`) ·
  `801e9e82db` (a página na família) · `0c3165622e` (cena `PH2D_CODE_SMOKE=5` + gate + bancada) ·
  `e4e3158f1e` (fotos refeitas). 140 ficheiros, +14 003 −570 (a maioria PNG e fixtures).

## 2. Superfície de colisão

`bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh` (linha inteira
contra o `main`; correr de novo no fecho — os números abaixo são o DELTA desta onda, que é o que se verificou):

- **Crate nova `ph2d-code-help`** (`Cargo.lock` +1 pacote); **dep nova** de `ph2d-app-code` nela.
- **`CENAS` 4 → 5** (`PH2D_CODE_SMOKE`), nos dois sítios (`ph2d-app-components/src/code_smoke.rs` e
  `ph2d-app-code/src/smoke.rs`); contador só desta família.
- **i18n:** `code.help.*` (módulo `code_help/`) novas; `code.help.page.*` e `code.help.back|forward|copy|sections`
  em `code_editor.rs`.
- **`ph2d-script`:** `api_help.rs` novo e `api_table::param_doc` (a ajuda de cada parâmetro).
- **Foundational intocado** (nenhum método novo em `DocumentSurface`); `PROJECT_SCHEMA`, registos e ADR **0**.
- **Shell:** `input_dispatch/letra_solta.rs` novo (guarda da letra solta, gate G9); `main.rs` 1066 → 1060
  (comentários aparados); chamada por quadro a `drain_clipboard_copy`.
- `surface.rs` moveu `docs.rs` 678 → 410 linhas; nasceram `help.rs`, `help_doc.rs`, `help_input.rs`, `nav.rs`.
- **CLAUDE.md §5.1:** a entrada do CodeEditor tem de passar a `PH2D_CODE_SMOKE=1..5` na integração (a linha não
  editou o CLAUDE.md).

## 3. O que a onda fez (resumo; mecanismo nos docs linkados)

- **Catálogo** (`ph2d-code-help`, `9e0154a2c5`): 225 nomes do runtime em 12 páginas Luau + `ph2d` / `motion` /
  `types` / `hooks`; **porta única** `resolve` (nome sob o cursor → página/âncora); exemplos que correm.
  `ph2d-script::api_help` e `luau_docs` passam a ler a ajuda de cada parâmetro. Cura de passagem: o `print` de um
  `integer` de 64 bits no console arredondava.
- **Vista** (`ph2d-panel-code/help/`, `126579ac59`): disposição pura, `HelpView` (cabeçalho ‹ ›, índice de
  secções, conteúdo virtualizado, links, exemplos com copiar, selecção e busca), a11y e sonda.
- **Família** (`801e9e82db`): Ctrl+clique/F12 abrem a página pela `resolve`; histórico ‹ › / Alt+←→; Ctrl+W; busca;
  copiar; Tab nos links. `goto`/`reach` perguntam à `resolve`. **Saíram** `Hints::pin` e `CodeDoc::documents` (o
  balão preso do I6 era redundante); o balão fecha ao carregar Ctrl.
- **Cena 5** (`0c3165622e`): `PH2D_CODE_SMOKE=5`, 11 passos (0–10), fotos
  `ferramentas/smoke/cena5/passo_0..10.png` (refeitas em `e4e3158f1e`). Cura da barra de busca: `chrome::button_w`
  mede os 3 botões de texto por `rect_for_label` (a lei inversa do orçamento).
- **Bancada** ([ajuda_bancada.md](../ferramentas/medicoes_onda2/ajuda_bancada.md)): Ctrl+clique → página em 1–2
  quadros; página de matemática de 14 611 px: layout ≈ 2,1 ms uma vez por largura, pintura 0,1 ms; um quadro com
  página 155 µs contra o editor a 7 500 linhas 412 µs.

## 4. O que um MERGE pode partir

- **`Hints::pin` e `CodeDoc::documents` apagados:** quem os chamar fora da linha não compila.
- **`CENAS` 4 → 5** nos dois sítios; outra linha que some cenas nesta família conflita aí.
- **Cargo.lock:** pacote novo `ph2d-code-help`; resolver à mão nunca — regenerar.
- **i18n:** chaves `code.help.*` novas; ordem/geração dos catálogos pode conflitar com outra linha que mexa em
  `code_editor.rs`.
- **Shell:** `input_dispatch/letra_solta.rs` (guarda: letra solta com Ctrl/aba de código não dispara atalho
  global) e o `drain_clipboard_copy` por quadro; `main.rs` perto do tecto da catraca `the_shell_only_shrinks`
  (1060): quem somar linhas lá reprova.
- **`ph2d-script`:** `api_table::param_doc` e `api_help.rs` — a tabela da API é lida pelo gate
  `toda_entrada_da_api_tem_ajuda`; entrada nova de API sem ajuda reprova.
- **`surface.rs`/`docs.rs`:** movidos (678 → 410); um merge textual de outra linha em `docs.rs` cai em sítio errado.

## 5. Testes

- **Todos verdes, por nome:**
  - `ph2d-code-help`: `censo` (3: `toda_global_do_runtime_tem_pagina`, `todo_texto_foi_escrito`,
    `cada_exemplo_corre_e_da_o_que_diz`), `resolver` (2), `ours` (1).
  - `ph2d-script`: `toda_entrada_da_api_tem_ajuda`, `o_print_de_um_integer_grande_nao_arredonda` (vermelho com a
    cura desligada).
  - `ph2d-app-code`: `ajuda` (G6–G8, vermelho com a abertura desligada); `oraculo_lsp` incluindo **G3: 206/206
    assinaturas = luau-lsp 1.70.1** (5 saltadas: `collectgarbage`, `debug.info`, `debug.traceback`, `os.date`,
    `table.insert` — o hover não traz nome) e `os_exemplos_da_api_nao_dao_aviso`;
    `dicas::carregar_no_ctrl_fecha_o_balao`; `despacho` G9 (vermelho com o `W` fora da lista da guarda);
    129 vizinhos da família verdes.
  - `ph2d-panel-code`: `pagina` (6; 8/8 mutações à mão sangraram) e `help::` (6).
  - `ph2d-app-components`: `a_cena_5_conduzida_passo_a_passo` (vermelho quando um link da mesma página empurra
    histórico), `a_cena_3`, `a_cena_4`; gate `cada_botao_da_barra_cabe_o_rotulo_inteiro`.
  - Shell: `file_loc_caps` e `architecture_the_shell_only_shrinks`.
- **Não corridos:** clippy, a suíte `nextest-impacted`, a prova de mutação formal — tudo no fecho da linha.
- Corridas: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/luau-lsp-fetch.sh && bash scripts/cargo-test-narrow.sh ph2d-code-help censo` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-app-code ajuda` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-app-code oraculo_lsp` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-panel-code pagina` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-app-components a_cena_5`.

## 6. Portões da onda (20/10)

Só os vizinhos por nome do §5. **Não correram:** clippy, a suíte, a mutação formal. A mutação do fecho deve cobrir
os ramos da `resolve`, o «mesma página não empurra histórico», o bit da aba de ajuda, a guarda da letra solta, o
balão fechado pelo Ctrl e `button_w`.

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, construído duas vezes; a 2.ª:
`Finished \`smoke\` profile [optimized] target(s) in 0.22s` (zero `Compiling`).

## 8. Smoke para o DONO — **PENDENTE**

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`.

A cena é conduzida em 11 passos (0–10); cada passo tem a foto do que deve aparecer em
`docs/CodeEditor/ferramentas/smoke/cena5/passo_0..10.png`. O dono confere: Ctrl+clique numa função da API abre a
página de ajuda numa aba (o que faz, cada argumento, exemplo, irmãs); voltar/avançar; busca; copiar o exemplo;
um link para a mesma página não empilha histórico; Ctrl+W fecha a aba. **Deu errado se:** o Ctrl+clique mostra só
um balão em vez da aba, ou o texto dos botões da busca sai cortado.

## 9. ABERTO

- **(a) Gate vermelho antigo, a curar ANTES de fechar:** `no_dependency_climbs_a_layer` reprova em
  `ph2d-app-components → ph2d-app-code [dev]` — vem dos gates das cenas 3/4 desta mesma linha, não desta onda.
- **(b) Achados do runtime/API** (`04` §9): `ph2d.wait` e `find_by_name` mortos, `spawn*`/`despawn`/`attach_script`
  com o erro da fila de 1024, `input` só de gamepad, manias de `state`, `loadstring` com ambiente partilhado (medir
  com dois scripts), `string.rep` 1e10. Decisão de produto do dono — não é desta onda.
- **(c) Diferenças declaradas:** as páginas de ajuda não estão na lista de scripts à esquerda (o Godot mostra);
  as abas de ajuda não persistem entre sessões.
- **(d)** Uma página re-disposta a outra largura mantém o scroll em pixels (o membro do topo desloca); o Godot não
  foi medido.
- **(e)** `docs/Components/ferramentas/fotografa_cena.sh` devia tirar duas capturas e guardar a 2.ª (o ecrã virtual
  pode devolver a 1.ª velha — foi por isso que as fotos foram refeitas).
- **(f)** O prefixo do nome do chunk do `eval` só existe no runner.
- **(g)** Herdados do handoff anterior: o Ctrl+M/Ctrl+W fora da aba e o balão com Ctrl **ficaram fechados** nesta
  onda; os restantes (A11Y_RENAME == A11Y_HOVER; árvore AccessKit sem adapter no sistema; secção Script do
  Inspector por fotografar; sonda do solver sem `--docs`; regra de Fatin) continuam, e os do
  [I13 §9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md#9-aberto).
- Integração: falta `clippy --workspace` e a suíte do fecho; CLAUDE.md §5.1 → `1..5`.

## 10. Próximo

Smoke do dono da cena 5 → `/pd-linha-fechar`: clippy + suíte + mutação (ramos da `resolve`, mesma página sem
histórico, bit da aba de ajuda, guarda da letra solta, balão no Ctrl, `button_w`) + curar o §9(a). Com isso a
fila do plano acaba; o que resta é o §9.
