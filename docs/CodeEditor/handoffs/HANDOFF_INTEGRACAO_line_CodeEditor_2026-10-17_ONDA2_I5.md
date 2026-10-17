# HANDOFF DE INTEGRAÇÃO — line/CodeEditor, Onda 2 parcial: I0–I5 (17/10/2026)

> Leitor: o agente integrador (ordem do Enio de 17/10: integrar DEPOIS da `line/Vector`). Este
> handoff SOMA-SE ao da [Onda 0 + 1](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-15_ONDA1.md), que
> continua por integrar: a linha leva as três ondas juntas. Desenho e provas de cada item:
> [03_onda2.md](../03_onda2.md) §Estado. ⚠️ Leia o §3 e o §7 do handoff da Onda 1 também
> (ADR-0188, `PROJECT_SCHEMA`, `motores_do_quadro.rs` apagado, sítios do Board).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch
  `line/CodeEditor` · rebaseada a 17/10 sobre o `main` `db535d2a10` (Motion =139) · HEAD
  `d9d5459cb1` (antes deste handoff` · **77 commits** sobre o merge-base (as Ondas 0+1, os 10 da Onda 2 e a cura do fecho).
- Integrável: cada item da Onda 2 deixa o editor a editar sem o assistente (servidor ausente ou
  parado = editor sem sugestões e a barra diz porquê).
- Não integrada, não enviada; o smoke (§8) é do Enio.

## 2. Superfície de colisão (`bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh`, 17/10)

```
merge-base db535d2a10 · 77 commits · 393 ficheiros
PROJECT_SCHEMA 190 (base 189) · tripla (190, 13, 22) (base (189, 13, 22)) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 20 (= base)
Registos de componentes 108/109/109 (= base) · contrato congelado: node.rs e tool.rs intocados
ADR: cria 0188 (próximo livre no disco 0189) · Cargo.lock: 13 pacotes externos novos · marcadores de conflito: nenhum
TETOS DE LOC: ✗ 1060/600 shells/desktop/src/main.rs (vem da base; a linha troca 1 linha — Onda 1 §2)
```
Tudo o que é delta vem da Onda 1 (§3 dela); a Onda 2 só acrescenta 1 pacote externo (`fastrand`, dev).

## 3. Contadores como DELTA (a Onda 2 não acrescenta nenhum)

- `PROJECT_SCHEMA`, ADR, registos de componentes, cenas de smoke, tokens: **a Onda 2 não toca**
  (os deltas da linha são os do §3 da Onda 1: ADR-0188 provisório, schema +1, inspector 87 → 88).
- Chaves i18n novas (`ph2d-i18n/src/code_editor.rs`, só EN — os censos da família as contam):
  `code.status.assist_{starting,restarting,missing,stopped}`, `code.complete.property`,
  `code.a11y.completion`, `code.cmd.restart_assistant`.
- Comando novo na paleta: `CMD_RESTART_ASSIST` (`ph2d-app-code/src/commands.rs`, id da família).

## 4. O que mudou em FOUNDATIONAL — e porque é ADITIVO

| Crate | O quê | Porque é aditivo |
|---|---|---|
| `ph2d-code-lsp` | `request_async`/`cancel`, `connect` (servidor no processo), `Wake`, `did_open_at`/`did_change_at` com versão, `PublishedDiagnostics.version`, `snippetSupport: true`, `Completion.{filter, snippet}`; o leitor passou a `client/reader.rs` | API nova ao lado da antiga (`request`, `did_open`, `did_change` ficam); só a família de scripts a usa |
| `ph2d-code-intel` (**NOVA**) | o worker da gramática e a sessão assíncrona do `luau-lsp` (`LspSession`, `Req`/`Answer`, `Dirty`, conversões UTF-16) | crate nova, sem UI; dependentes: só `ph2d-app-code` |
| `ph2d-code-syntax` | `Syntax::edit_deferred`/`job`/`adopt`/`fresh`, `closer_tokens`, `declared_properties`, `in_code`; o `AutoEdit` só pede a árvore exacta no Enter/auto-par/recuo | funções novas; o `edit` síncrono continua (e conta em `sync_reparses`) |
| `ph2d-script` | `api_table::{luau_docs, luau_params}` (a base `--docs` do `luau-lsp`, gerada da tabela); dep `serde_json` | só leitura da tabela que já existia |
| `ph2d-script-doc` | `ScriptLibrary::revision()` (única no processo, fora do ficheiro e do `PartialEq`) | o formato gravado NÃO muda (gate `a_revisao_muda_com_o_conteudo_e_so_com_ele`) |
| `ph2d-i18n` | as chaves do §3 | só acrescenta |

**Contratos congelados (CLAUDE.md §6): nenhum tocado.** Prova: `git diff db535d2a10 HEAD --stat --
crates/ph2d-nodegraph crates/ph2d-tool-api crates/ph2d-vector crates/ph2d-imageio` → **vazio** (17/10).
Na Onda 2, `ph2d-editor-core` e `shells/desktop` **não** foram tocados (só a Onda 1 os toca).

### Crates e deps novas
`ph2d-code-intel` (nova, membro do workspace). Deps novas: `serde_json` em `ph2d-script` e em
`ph2d-app-code` (dev: o servidor LSP falso), `fastrand` em `ph2d-app-code` (dev: os traços ao acaso
do gate da tecla). `Cargo.lock`: regere.

## 5. Itens I0–I5 (commit · o teste que fica vermelho se a propriedade quebrar)

| # | Commits | O que entrega | Provas (mutação) |
|---|---|---|---|
| I0 | `f329e3d29c` | a rodada única de medição (reparse do topo, respostas reais do `luau-lsp` 1.70.1, 1 095 mutantes de erro, StyLua 2.5.2, as pausas do Godot) | as tabelas em `ferramentas/medicoes_onda2/` |
| I1 | `561567d0e6`, `381d62c92a` | o reparse do tree-sitter sai da tecla (worker; a árvore velha liberta-se no worker) | `gramatica::as_teclas_sem_arvore_nunca_reparsam_na_tecla` e 4 mais · **6/6** |
| I2 | `561567d0e6`, `c923f73340` | o `luau-lsp` vê o texto que o editor MOSTRA, de todos os scripts; a tecla nunca o espera; reinício automático; sem binário a barra diz porquê | `assistente::o_assistente_ve_o_texto_que_o_editor_mostra` (400 gestos) e 5 mais; `sessao::` 5 · **11/11** |
| I3 | `cc3f97fcc1` | os erros do assistente sublinhados enquanto se escreve, um por erro (fusão com o tree-sitter) | `problems::` 5, `assistente::os_sublinhados_seguram_se_entre_respostas` e 3 mais, `oraculo_lsp::o_assistente_fixado_sublinha_o_que_se_escreve` · **11/11** |
| I4 | `31abe5be51`, `2355b9d899` | a lista de completação (abre na pausa de 300 ms, filtro e ordem do Godot, uso no projeto, Tab/Enter, Ctrl+Espaço) | `completar::` 11, `oraculo_lsp::o_assistente_fixado_completa_a_api` · **15/15 + 3/3** |
| I5 | `251d865ea8`, `f20620a5c5` | o balão do rato (500 ms PARADO) e a ajuda de assinatura com o argumento marcado | `dicas::` 9, `oraculo_lsp::o_assistente_fixado_marca_o_argumento_e_mostra_o_balao` · **23/23** |

## 6. Prova de fecho (gate batched sobre o diff acumulado, 17/10)

- `PH2D_SUITE=fecho BASE=db535d2a10 bash scripts/nextest-impacted.sh` (743 s, `load` 130–240) →
  **18 970 / 18 974**: 2 vermelhos da linha — `arch_safe_clamp_only` (`popup.rs:62`) e
  `architecture_workspace_file_loc_cap` (`complete.rs` 858, `docs.rs` 839, `client.rs` 704) —,
  curados em `d9d5459cb1` (cortes para `complete_doc.rs`, `docs_probe.rs`, `client/hooks.rs`;
  nenhuma isenção) e corridos de novo por nome: verdes. Os outros 2 são **flakes de carga** fora do
  diff, confirmados sozinhos: `ph2d-physics-ecs it measure_player_budget::the_cost_of_a_player_is_linear_in_their_number`
  (3/3 verde a `load` 60–89) e o TIMEOUT de `ph2d-tool-painter wetpaint::achatamento_tests::o_escorrimento_do_s_nao_se_corta_em_rectangulos`
  (2/2 verde a `load` 33–57).
- Depois dos cortes: `PH2D_SUITE=fecho cargo test` das cinco crates da linha (`ph2d-app-code`,
  `ph2d-panel-code`, `ph2d-code-lsp`, `ph2d-code-intel`, `ph2d-code-syntax`) → **175/175**.
- Clippy `--all-targets -D warnings` nos 18 pacotes do diff → verde (curou 11 avisos, a maioria em
  testes de sonda que nunca tinham passado no clippy); `--all-features` em `ph2d-app-components`,
  `ph2d-app-registry-init`, `ph2d-panel-registry-init`, `ph2d-host-desktop` → verde.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` → 0 avisos · `cargo fmt --all
  --check` verde · `cargo machete`, `check-standalone-optional.sh` (11/11), `check-workflow-packages.sh`
  (32 contra 401) verdes · `the_shell_only_shrinks` e os `file_loc_caps` da shell verdes.
- **Auditoria (duas lentes, com a asserção vermelha de cada claim = as mutações do §5):**
  - *Correção assíncrona*: nenhuma resposta velha pinta no sítio errado — os diagnósticos só para a
    versão que o servidor tem (I2 `um_diagnostico_de_versao_velha_nunca_sai`), os segurados seguem as
    edições por enviar (I3 `uma_resposta_com_teclas_por_enviar_cai_no_sitio`), a lista/balão/assinatura
    só aceitam o bilhete que pediram, e um pedido novo do mesmo género cancela o velho (mutação I5
    M22: balão e assinatura no mesmo género ⇒ vermelho). Todo «não abre» dá tempo REAL a uma resposta.
  - *Costura e acessibilidade*: cada gesto entra pela porta real da família (`DocumentSurface::key`,
    `pointer`, `pointer_move`, `wheel`) e o efeito lê-se do que o quadro PINTOU (`*_painted`); a lista
    é `ListBox` + `active_descendant`, os balões `Tooltip` com o parâmetro no `value` (I4, I5).
  - Não verificado pela compilação nem pelos testes: o desenho real na placa (cores, cortes dos
    `Popover`) — fica para a foto do I15.

## 7. O que um merge pode partir

1. Tudo o do §7 do handoff da Onda 1 (continua a valer).
2. Os testes `oraculo_lsp::` precisam do `luau-lsp` FIXADO na máquina (`bash scripts/luau-lsp-fetch.sh`); sem ele reprovam com a frase que o diz (é de propósito).
3. `Os cortes do fecho moveram código DENTRO das crates: `ph2d-app-code/src/{complete_doc,docs_probe}.rs`
   e `ph2d-code-lsp/src/client/hooks.rs`. Uma linha que edite as funções movidas (`completion_shown`,
   `accept_completion`, as sondas `*_of`, `hooks_in_definitions`) cai num conflito modify/delete:
   reaplique no ficheiro novo.`

## 8. Smoke para o DONO (a cena da Onda 1, agora com o assistente)

> ⚠️ Os passos abaixo não foram fotografados (a foto e a cena própria `PH2D_CODE_SMOKE=3` são o I15).
> Precisam do assistente instalado na máquina, como nos testes (`bash scripts/luau-lsp-fetch.sh`).
> O binário já está compilado: a 2.ª compilação diz `Finished \`smoke\` profile [optimized] target(s) in 0.30s`, zero `Compiling`.

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Hierarchy clique em **Circle**; no Inspector, cartão **Script**, clique **Edit script**: abre a aba **bob.luau**.
3. **Balão:** pare o rato sobre a palavra `set` da última linha de código (`ph2d.set(self.id, "y", …)`) e espere meio segundo: aparece uma caixinha com `function ph2d.set(…)` e a frase «Writes a pose field…». Com o rato a andar não aparece nada; tirando o rato da palavra, ela some.
4. **Lista e ajuda dos argumentos:** clique no fim da linha `self.t = self.t + dt`, carregue Enter e escreva `ph2d.se`: num instante aparece a lista com `set` em cima. Carregue **Tab**: fica `ph2d.set()` com o cursor dentro dos parênteses e aparece uma caixinha com o que a função pede, com `entity` marcado. Escreva `self.id, ` (depois de `self.` a lista pode voltar a abrir: continue a escrever): a marca passa para `field`. Escreva `"x", 1` e saia com a seta → para fora dos parênteses: a caixinha some.
5. **Erro ao vivo:** noutra linha escreva `pirnt(1)` (sem salvar): `pirnt` fica sublinhado a vermelho logo; apague-o e o sublinhado some.
6. Deu errado se: a caixinha aparece com o rato a mexer; a lista ou a caixinha não aparecem; a marca não anda com as vírgulas; a escrita fica lenta ou trava; ou a barra de baixo diz «Code assistant not installed» (aí falta o assistente na máquina, não é defeito do editor).

## 9. Números MEDIDOS

- I1 (tecla fora do reparse, n = 1 000, réplicas intercaladas): a tecla, a pintura e a placa do
  editor não crescem com o ficheiro a 7 500 linhas; a árvore velha largada no quadro custava
  `0.33` ms p50 e passou a `0.001` ms ([I1](../ferramentas/medicoes_onda2/I1_tecla_fora_do_reparse.md)).
- I4 (lista aberta × fechada, `loadavg` 88): p50 `1.49/1.50` ms (51 linhas) e `1.46/1.44` ms
  (7 500); p99.9 `4.55` e `5.15` ms, com réplicas do controlo a distar até `0.95` ms sob essa carga
  ([I4](../ferramentas/medicoes_onda2/I4_lista_aberta.md)). ⚠️ A repetir no I15 numa rodada só,
  com a lista e os balões abertos.
- `luau-lsp` (I0b): hover < 1 ms, signatureHelp 0,4–1,1 ms, completion 1–7 ms no `bob.luau`.

## 10. ABERTO

- Onda 2: I6 (ir para a definição) e I11 (tokens semânticos) a seguir; depois I7 → I8 → I10 →
  I9/I12/I14 → I13 → I15 ([03_onda2.md](../03_onda2.md) §Próximo passo).
- Do I1: o Enter (e o `fresh` das dobras) ainda larga a árvore velha na thread do quadro
  (`0.33` ms p50 a 7 500 linhas, sobre o reparse síncrono que o Enter já paga).
- Do I2: o caminho `Missing` (binário não encontrado) não tem gate de ponta a ponta (só a frase).
- Do I3/I8: a mensagem é a CRUA do Luau; as explicações escritas à mão são o I8.
- Do I4/I12: os buracos do snippet (Tab entre argumentos).
- Do I5: sem acorde manual da assinatura; a mensagem do diagnóstico no balão é do I8.
- Do I15: a cena `PH2D_CODE_SMOKE=3`, fotografada, e a bancada completa com o LSP ligado.

## 11. `agent-loop-profile.sh`

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                139   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                398 : 116   alvo: <= 1,0  razao 3.4x (baseline: 4,3x)
  ✗ edicoes SEM conferir que casaram             2.7%   alvo: <= 2%  (23 de 849; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         272 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               65 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```
