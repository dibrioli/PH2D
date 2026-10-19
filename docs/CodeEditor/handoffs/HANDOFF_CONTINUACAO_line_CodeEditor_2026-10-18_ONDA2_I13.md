# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I13 (18/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; I6/I11, I7, I8, I8b, [I10](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I10.md) e
> [I9+I12+I14](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I9_I12_I14.md) estão na linha; este vem por
> cima. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha `| I13 |`) · medição e oráculo:
> [I13.md](../ferramentas/medicoes_onda2/I13.md) (§«O nosso, lado a lado»). O mecanismo vive nesses; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` `100d34c8e0` · HEAD `2e30941d3b` (antes do commit deste doc) · 56 commits da linha.
- Commits da onda (sobre `0515496c56`), **2**: `956e14ddba` docs (oráculo VS Code 1.142 + extensão StyLua 1.7.2;
  medição `ferramentas/medicoes_onda2/I13.md`, fotos `I13/`, script `ferramentas/oraculo_i13/corre.sh`) ·
  `2e30941d3b` feat (39 ficheiros, +1810 −185).

## 2. Superfície de colisão

`bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh` (linha inteira
contra o `main`):

```
merge-base 100d34c8e0 · 56 commit(s) · 291 arquivo(s)
PROJECT_SCHEMA 191 = base · tripla (191, 13, 23) = base · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20 = base
registo de componentes: ph2d-ecs 108 · render 109 · script 109 = base
contrato congelado (§6): intocado · ADR: nenhum (próximo livre 0189) · cena de smoke: nenhuma
Cargo.lock: 2 pacotes novos na linha — ph2d-code-explain (onda anterior), ph2d-code-format (esta)
marcadores de conflito: nenhum · tectos de LOC: nenhum ficheiro da linha passa do teto
```

**Contadores como DELTA desta onda:** nenhum de schema/registo/ADR/cena; `Cargo.lock` **+10** (a crate nova
`ph2d-code-format` e as arestas). Comandos da `TABLE`: **+1** (11 → 12): `CMD_FORMAT_ON_SAVE` «Turn Off/On
Format on Save». Chaves i18n novas em `ph2d-i18n/src/code_editor.rs` (+10 linhas).

**Foundational tocado:** nenhum (`ph2d-editor-core` e a shell intocados nesta onda). Toca `THIRD_PARTY_LICENSES.md`
(+18: os dois programas ajudantes, MIT e MPL-2.0, com onde está o fonte).

## 3. O que a onda fez (resumo; mecanismo na medição e no código)

- **Medição** (`956e14ddba`): oráculo VS Code com a extensão StyLua a usar o NOSSO binário e as NOSSAS regras.
  Leis: o cursor fica junto do mesmo carácter (muda de linha com ele); inserção NA posição ⇒ depois; dentro de
  branco apagado ⇒ uma ponta; âncora e cabeça seguem cada uma; a formatação é UM passo de undo (Ctrl+Z ⇒ sujo,
  refazer ⇒ limpo); já formatado ⇒ nenhum passo; erro de sintaxe grava sem formatar e sem aviso; o VS Code
  **segura a gravação até formatar** (a diferença declarada, §9).
- **`ph2d-code-format`** (crate nova): `stylua-pin.toml` (o manifesto nunca se chama `stylua.toml`: o StyLua
  procuraria config de fora) + `ph2d-format.toml` (regras; movido para dentro da crate), sempre `--config-path` e
  `--verify`. `run.rs`: `Program`/`Refusal` (sintaxe · `--verify` · falha) classificadas pela mensagem da versão
  fixada (2.5.2). `diff.rs`: `changes()` = diff mínimo (linhas comparadas sem brancos, depois carácter por par,
  cabeça e cauda comuns cortadas; 7 500 linhas re-recuadas **265 → 3,8 ms** — o diff de linhas cru era Myers sobre
  7 500 linhas todas diferentes). `worker.rs`: `Worker` = uma thread por pedido.
- **Lei do pino UMA** (`ph2d-code-lsp`): `ph2d_code_lsp::Pin` serve `LUAU_LSP` e `STYLUA`; `resolve.rs` reescrito
  (`ResolveError` mudou de forma, `binary_file_name()` devolve `String`); `scripts/pinned-helper-fetch.sh` novo,
  com `luau-lsp-fetch.sh` (agora invólucro; o git vê renomeação) e `stylua-fetch.sh` por cima; macOS por
  arquitectura.
- **Editor** (`ph2d-app-code/src/format.rs`, 189 linhas): o Ctrl+S aplica e dá o selo no MESMO quadro; a resposta
  entra como UM passo de undo (`EditKind::Other`, nunca se junta) só se a revisão do buffer é a do pedido e não há
  composição, e a aba salva-se OUTRA VEZ (cena + ficheiro) — sem ● depois; só o último Ctrl+S conta; comando
  «Turn Off/On Format on Save» (`CommandState.format_on_save`; por sessão); sem programa ⇒ grava e avisa uma vez;
  `--verify`/falha ⇒ nota no console (`Store::note`). `CodeDocs.format`, `CodeDoc.formatting/resave`.
  Quadro em que a formatação entra: **0,02 / 0,71 / 7,08 ms** (51 / 7 500 / 7 500 re-recuadas linhas; cabem nos
  16,7 ms de 60 Hz); loadavg 9–18 na medição.
- **O que o produto traz sai limpo do StyLua:** a 2.5.2 dobra a função VAZIA numa linha (`--preserve-block-newline-gaps
  Preserve` não impede — medido); o `NEW_SCRIPT_TEXT` ganhou um comentário em cada corpo; as cenas `code_smoke`/
  `script_smoke` e o espelho `bob_estrito.luau` passaram à forma do StyLua (gate em componentes). As fixturas
  `bob.luau` congeladas ficam.
- **`intel.rs`:** o ficheiro temporário das definições com nome único por escrita (a mesma corrida que o
  formatador tinha entre duas threads do mesmo processo).

## 4. O que um MERGE pode partir

- **`ph2d_code_lsp::ResolveError`** mudou de forma (`EnvPointsNowhere{pin,path}`, `NotFound{pin,tried}`;
  `PartialEq` à mão) e `binary_file_name()` devolve `String`: chamadores fora da linha quebram.
- **`scripts/luau-lsp-fetch.sh`** passou a invólucro de `scripts/pinned-helper-fetch.sh`: outra linha que o edite
  conflita (renomeação).
- **`commands::TABLE`** 11 → **12**; `CommandState` ganhou `format_on_save` (construtores fora da linha reprovam).
- **`CodeDocs.format`** e **`CodeDoc.formatting/resave`**: `docs.rs` a **696/700** (`wc -l`) — acrescentar ali
  estoura o tecto.
- **`NEW_SCRIPT_TEXT`** mudou (um comentário em cada corpo); **`script_smoke::BOB_LUAU`** e
  **`code_smoke::BOB_LUAU`** mudaram de texto (forma do StyLua; o `if` em 3 linhas desce as linhas seguintes 2) e o
  espelho `crates/ph2d-code-lsp/tests/it/fixtures/bob_estrito.luau` acompanhou: testes com números de linha desses
  scripts em outras linhas falham.
- **`Mesa`** dos testes do app-code liga `FormatServer::Off` por omissão (`com_formatador`).
- **Dev-dep nova** `ph2d-code-format` em `ph2d-app-components`.
- **Gates novos precisam do binário:** `bash scripts/stylua-fetch.sh` (como os do luau-lsp precisam do
  `luau-lsp-fetch.sh`). **O CI não foi visto a buscar nenhum dos dois** (procurado em `.github/workflows`; sem
  passo de fetch) — antes de shipar, confirmar como esses gates se comportam sem o binário.

## 5. Testes e mutação

- `crates/ph2d-app-code/tests/it/formatar.rs` **13**: `o_ctrl_s_aplica_no_quadro_e_a_formatacao_entra_num_passo_de_desfazer`
  · `o_que_se_escreve_antes_da_resposta_ganha_a_formatacao` · `o_comando_liga_e_desliga_em_todas_as_abas` ·
  `sem_o_programa_salva_e_avisa_uma_vez` · `a_recusa_do_formatador_vai_ao_console_e_o_texto_fica` ·
  `com_ficheiro_o_projecto_grava_se_outra_vez_formatado` ·
  `com_o_stylua_fixado_o_bob_sai_como_o_programa_e_o_cursor_fica_no_codigo` ·
  `o_modelo_do_script_novo_sai_limpo_do_stylua_fixado` · `com_o_stylua_fixado_um_texto_partido_salva_sem_formatar` ·
  `com_o_stylua_fixado_o_que_mudaria_o_codigo_nao_se_formata` · `medicao_do_quadro_em_que_a_formatacao_entra` ·
  `a_resposta_de_um_pedido_velho_nunca_entra_no_texto_novo` (cada pedido espera a SUA chave: a velha chega
  primeiro, com o pedido novo à espera) · `a_formatacao_nunca_se_junta_a_escrita_de_antes`.
- `ph2d-code-format`: **1** unitário (`diff::tests`) + **6** `it` (`o_stylua_resolvido_e_a_versao_fixada` ·
  `formata_como_o_programa_na_mao_e_e_idempotente` · `as_recusas_sao_as_da_versao_fixada` ·
  `a_config_de_fora_nunca_entra` · `o_trabalhador_devolve_as_edicoes_do_texto_enviado` ·
  `medicao_do_atraso_da_formatacao`). Fixturas `bob.formatado.luau`, `bob.sem_end.luau`.
- `ph2d-app-components/tests/it/os_scripts_da_cena_saem_limpos_do_formatador.rs` **1**.
- **Mutação: 16 mutações; 12 sangraram na 1.ª rodada.** Sobreviveram M2 (o filtro do bilhete: o teste dependia
  da ordem das threads ⇒ reescrito com uma chave por pedido, agora sangra), M10 (o caminho por pares desligado:
  o mesmo texto, 10 193 edições e 1 849 ms em vez de 7 500 e 3,6 ms ⇒ `medicao_do_atraso_da_formatacao` exige
  uma edição por linha re-recuada, sem relógio; agora sangra) e M3 (`EditKind::Typing` no passo da formatação:
  **equivalente** — `History::mark_saved` sela o grupo e a formatação só existe depois de um salvar; o teste
  `a_formatacao_nunca_se_junta_a_escrita_de_antes` fica como documento da lei). M14 (a ordem dos dois `if` do
  `classify`): equivalente (só difere com as duas mensagens no mesmo stderr). **Clippy: não corrido** (só no fecho).
- Vizinhos verdes por nome (18/10): app-code `it` 156/156 + lib 21; code-lsp 17 (+ `script_canonico` 3 depois do
  espelho); code-format; i18n; app-components lib 889 + `it` 5; editor-core `architecture_` 111; `cargo check -p
  ph2d-host-desktop` ok.
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/stylua-fetch.sh && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --profile smoke --test it -- formatar::`
  e `… cargo test -p ph2d-code-format --profile smoke`.

## 6. Portões da onda (18/10)

Só os vizinhos por nome do §5. **Não correram:** clippy, a suíte `nextest-impacted`, a mutação — tudo no fecho da
linha.

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.70s` (zero `Compiling`).

## 8. Smoke para o DONO

> Precisa do formatador instalado: `bash scripts/stylua-fetch.sh` (já está no cache desta máquina). ⚠️ Não fotografado.

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique no **Circle**, no cartão **Script**, em **Edit script**: abre a aba do `bob.luau`.
3. **Ctrl+End** (fim do script), **Enter**, escreva `local   x=1` (três espaços depois de `local`). Carregue **Ctrl+S**: o selo aparece na hora; logo a seguir a linha passa a `local x = 1` e o cursor fica no fim dela; não aparece o ●.
4. **Ctrl+Z** uma vez: volta `local   x=1` e aparece o ●. **Ctrl+Shift+Z**: volta `local x = 1`, sem ●.
5. **Ctrl+K**, escreva `format` e clique em **Turn Off Format on Save**; escreva `local   y=2` numa linha nova e **Ctrl+S**: fica como escreveu. **Ctrl+K** → **Turn On Format on Save** liga outra vez.
6. Apague um `end` e **Ctrl+S**: grava sem arrumar (o selo diz a linha do erro, como antes).
7. **Deu errado se:** o texto muda ANTES do Ctrl+S; é preciso mais de um Ctrl+Z para tirar a arrumação; o cursor salta para outro sítio; fica o ● depois do Ctrl+S; os comentários do script mudam sem ter mexido neles.

## 9. ABERTO

- As abas **FECHADAS** com ● gravadas pelo File ▸ Save não se formatam.
- O interruptor «Format on Save» vale **por sessão** (o app não tem armazém de preferências).
- Os avisos das deps MIT/Apache do binário quando o PH2D o levar ao lado do executável
  ([THIRD_PARTY_LICENSES.md](../../../THIRD_PARTY_LICENSES.md)).
- **Diferença declarada:** o VS Code segura a gravação até formatar; nós gravamos já, e uma edição antes da
  resposta larga a formatação (a revisão mudou).
- Escrever durante uma formatação **LENTA** não foi medido no oráculo (o StyLua em 4 linhas acaba antes da 1.ª tecla).
- O indicador `{ }` da barra do VS Code na cena 8 do oráculo não foi lido.
- O CI não foi visto a buscar o `stylua`/`luau-lsp` (§4).
- **Clippy por fazer** (só no fecho).
- Os herdados do [handoff anterior §9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I9_I12_I14.md#9-aberto)
  continuam.
- Integração: falta `clippy --workspace` e a suíte do fecho.

## 10. Próximo

**I15**: cena `PH2D_CODE_SMOKE=3` fotografada (escrever `ph2d.` e escolher; `ampiltude` → Alt+Enter; renomear e ver
no Inspector) · a bancada completa numa rodada com o LSP ligado, a lista E os balões abertos · solver antigo ×
novo em scripts reais, critério de morte: se o novo estourar, fica o antigo. A cena 3 mora em
`crates/ph2d-app-components/src/code_smoke.rs` (`CENAS = 2` lá e em `ph2d-app-code/src/smoke.rs`, conferidos em
teste). → acabamento (Ctrl+rato em mão + sublinhado; a lista de problemas) → smoke do dono e `/pd-linha-fechar`.
