# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: Acabamento (19/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; I6/I11, I7, I8, I8b, I10, I9+I12+I14, [I13](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md)
> e [I15](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_I15.md) estão na linha; este vem por cima.
> Desenho e provas: [03_onda2.md](../03_onda2.md) §Próximo passo (item 4) · medições:
> [acabamento_problemas.md](../ferramentas/medicoes_onda2/acabamento_problemas.md) e
> [acabamento_bruto.json](../ferramentas/medicoes_onda2/acabamento_bruto.json) · oráculo:
> [oraculo_acabamento/](../ferramentas/oraculo_acabamento/). O mecanismo vive nesses; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` `100d34c8e0` · HEAD `2e988f0664` (antes do commit deste doc) · 69 commits da linha.
- Commits da onda (sobre `a9d0f32372`), **5**: `1a4126cc13` feat+perf (Ctrl+rato = mão e sublinhado;
  `problems()` guardado) · `76317df448` feat+docs (o nome do Ctrl+clique na cor do link + o oráculo) ·
  `f8bbb98db8` feat (a lista de problemas) · `72a0cd2909` fix (buracos dentro de buraco fundem as sessões) ·
  `2e988f0664` feat (cena `PH2D_CODE_SMOKE=4` conduzida e fotografada). 164 ficheiros, +4834 −219 (102 são
  PNG).

## 2. Superfície de colisão

`bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh` (linha inteira
contra o `main`):

```
merge-base 100d34c8e0 · 69 commit(s) · 447 arquivo(s)
PROJECT_SCHEMA 191 = base · tripla (191, 13, 23) = base · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20 = base
registo de componentes: ph2d-ecs 108 · render 109 · script 109 = base
contrato congelado (§6): intocado · ADR: nenhum (próximo livre 0189)
Cargo.lock: 2 pacotes novos na linha — ph2d-code-explain, ph2d-code-format (de ondas anteriores)
marcadores de conflito: nenhum
tectos de LOC (✗): ph2d-app-code/src/docs.rs 733/700 · ph2d-panel-code/src/console.rs 608/600 ·
  ph2d-panel-code/src/paint.rs 623/600 · shells/desktop/src/app_state.rs 925/600 · shells/desktop/src/main.rs 1063/600
```

**Contadores como DELTA desta onda:** schema/registo/ADR **0**; **cena de smoke +1** (`PH2D_CODE_SMOKE` 3 → 4;
contador só desta família); `Cargo.lock` **+1 linha** (dev-dep `ph2d-panel-code` em `ph2d-app-components`, sem
pacote novo); chaves i18n: **−6** `code.status.*` (`N errors · M warnings`), **+** `code.status.counter`,
`code.band.*`, `code.problems.*`; `Ink::ALL` 16 → 17 (`Ink::Link`); a shell cresceu **~12 linhas** (a catraca
`the_shell_only_shrinks` tem folga de composição; contar no fecho). Os cinco `✗` de LOC acima são do mapa: conferir
no fecho quais já vinham dos handoffs anteriores e quais esta onda empurrou.

**Foundational tocado (Modo L, append-only):** `ph2d-editor-core` — `DocumentSurface::pointer_look` (método com
**default `None`**: outras famílias não quebram) e o tipo `PointerLook`.

## 3. O que a onda fez (resumo; mecanismo nos docs linkados)

- **Ctrl+rato** (`1a4126cc13`, `reach.rs`): com Ctrl carregado, o ponteiro vira mão e a palavra sublinha **onde o
  Ctrl+clique alcança**. `Documents::surface_pointer_look` → a shell pergunta-o a cada quadro em `render_frame` e
  guarda o último `set_cursor` em `App.cursor_shown` (o backend Wayland do winit 0.30.13 reenvia cada `set_cursor`);
  `update_eyedropper_cursor` passou a `pub(crate)`.
- **`goto.rs` partido** em `goto_word` / `destination` / `documents`, com a regra do Ctrl+clique **CORRIGIDA**:
  palavras reservadas e números não se perguntam (`ph2d_code_syntax::is_identifier`), e o balão preso do
  Ctrl+clique/F12 só se mostra se `documents` (a resposta declara um nome: `function ` / `type ` / `local `, sem
  `*error-type*`). Sonda real `ferramentas/lspprobe_acabamento.py` → `acabamento_bruto.json`: o `luau-lsp` 1.70.1
  dá balão a `if`/`end`/`function` (a assinatura), `and`/`or`, `nil`/`true`, números, `pirnt` (`*error-type*`),
  `self.ampiltude` (`T`) e `self.id` (`number`) — daí a regra.
- **`problems()` guardado** (`1a4126cc13`): `Arc<[Problem]>` com chave (revisão, geração da árvore, `is_dirty`,
  `Held::generation`, `Lint::generation`, `saved_generation`) e `debug_assert` contra a recontagem. Medido
  `159,6 → 6,2 µs` a 7 500 linhas / 293 diag; pintura `51 → 7 500` linhas `+235 → +54 µs`
  (`medicoes_onda2/acabamento_problemas.md`, sonda `bancada::problemas_por_quadro`). **Fecha** a suspeita aberta
  no §9 do I15.
- **Oráculo** (`76317df448`, `ferramentas/oraculo_acabamento/`: `saida/amostras.txt`, `a11y_vscode.txt`,
  `a11y_godot.txt`): VS Code 1.142 e Godot 4.7.2. `Ink::Link = ColorToken::Accent` no traço e no nome
  (`reach::tint`). No Godot o accesskit não carrega.
- **Lista de problemas** (`f8bbb98db8`, `problem_list.rs`): separadores **Console / Problems** na faixa de baixo;
  contador ⊗/⚠ na barra (substituiu «N errors · M warnings»; `StatusLine.problems` → `StatusLine.counter`);
  **Ctrl+Shift+M** abre; **F8 / Shift+F8** navegam; ordem gravidade → linha → culpado → mensagem sobre índices;
  clique selecciona o culpado e o teclado fica no texto; nós AccessKit `List`/`ListItem`/`Tab`/`Button`.
- **Buracos fundidos** (`72a0cd2909`, `Tabstops::nests`/`merge`, oráculo `oraculo_snippet_aninhado/`): um item
  COM buracos dentro de um buraco funde as sessões como o VS Code — o buraco activo sai, entram os de dentro + o
  `$0` de dentro como paragem, seguem os de fora, acaba no `$0` de fora; Esc acaba tudo. **Fecha** o aberto do I15
  §9 («ainda substitui a sessão de fora»).
- **Cena 4** (`2e988f0664`, `PH2D_CODE_SMOKE=4`, `problemas.luau`): porta `PH2D_CODE_SMOKE_PASSO` 0..7 com
  gestos de rato / Ctrl / clique na lista / Tab por quadro; sonda `reach_settled_of`; fotos
  `ferramentas/smoke/cena4/passo_0..7.png`. As fotos precisam de `XDG_CACHE_HOME=$HOME/.cache`.

## 4. O que um MERGE pode partir

- **Contrato `DocumentSurface`** ganhou `pointer_look` (default `None`, não quebra); quem constrói **`CodeFrame`**
  fora da linha quebra (campo `link` novo). `Ink::ALL` 16 → 17. `StatusLine.problems` → `counter`.
- **Chaves i18n removidas** (6 `code.status.*`): quem as referir fora da linha perde o texto.
- **`CENAS` 3 → 4** em `ph2d-app-components/src/code_smoke.rs` e `ph2d-app-code/src/smoke.rs`. **CLAUDE.md §5.1**,
  a entrada do CodeEditor diz `PH2D_CODE_SMOKE=1|2` ⇒ tem de passar a `1..4` na integração (a linha não editou o
  CLAUDE.md).
- **Shell**: `render_frame` pergunta o `pointer_look` e guarda `App.cursor_shown`; `update_eyedropper_cursor`
  `pub(crate)`. Outra linha que mexa nesse trecho conflita.
- **Dev-dep nova** `ph2d-panel-code` em `ph2d-app-components` (`Cargo.toml` + `Cargo.lock`).
- **`goto.rs` partido** em três módulos; o teste interno de `problems.rs` ganhou o campo `generation`.
- **Ctrl+Shift+M** passa a ser tomado pela aba. Fora da aba, o **Ctrl+M / Ctrl+Shift+M ainda trocam o tema** da
  shell (§9).

## 5. Testes

- Gates novos: `alcance` (4, `reach`) · `tests/it/problemas_lista.rs` (8) · `esqueletos::um_item_com_buracos_dentro_de_um_buraco_funde_as_sessoes`
  (vermelho com a fusão desligada) · `a_cena_4_conduzida_passo_a_passo` (`ph2d-app-components`).
- Vizinhos por nome, **todos verdes**: `esqueletos` (11), `definicao`, `dicas`, `explicacoes`, `botoes`, `console`,
  `a11y`, `teclado`, `onda1`, `renomear`, `completar`, `cores` (7/7), `a_cena_3` / `a_cena_4`, o formatador da
  cena; em `ph2d-panel-code`: `a11y` (acertado: nó novo do contador), `estado`, `clique`, `sublinhado`,
  `virtualizacao`. `cargo check -p ph2d-host-desktop` verde.
- Sonda `#[ignore]`: `bancada::problemas_por_quadro` (`ph2d-app-code`).
- **Mutação: não corrida. Clippy: não corrido. Suíte `nextest-impacted`: não corrida** (só no fecho da linha).
- Corridas: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/luau-lsp-fetch.sh && bash scripts/cargo-test-narrow.sh ph2d-app-components a_cena_4` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-app-code problemas_lista` ·
  `… bash scripts/cargo-test-narrow.sh ph2d-app-code esqueletos` ·
  `… PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`.

## 6. Portões da onda (19/10)

Só os vizinhos por nome do §5. **Não correram:** clippy, a suíte `nextest-impacted`, a mutação — tudo no fecho da
linha (`/pd-linha-fechar`). A mutação deve cobrir `reach`, `goto::documents`, a chave do cache de `problems()` e
`Tabstops::merge`.

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:
`Finished \`smoke\` profile [optimized] target(s) in 0.22s` (zero `Compiling`).

## 8. Smoke para o DONO

(por fazer)

## 9. ABERTO

- **Pré-existente:** `A11Y_RENAME == A11Y_HOVER` (`0xC000_0000`) em `ph2d-panel-code`; não colidem só porque
  nunca aparecem juntos.
- **Ctrl+M / Ctrl+Shift+M fora da aba** trocam o tema: a shell casa `KeyCode::KeyM` sem olhar modificadores
  (defeito pré-existente; antes da onda o Ctrl+Shift+M dentro da aba também caía lá).
- **Leitor de ecrã:** o programa **não liga** a árvore AccessKit ao leitor do sistema (nenhum adapter na shell,
  `sync_a11y` não é chamado) — vale para o editor todo; os nós novos só são provados por teste.
- **Cena 4, passo 1:** o balão do `ph2d.set` aparece com o Ctrl carregado; o Godot fecha-o ao carregar Ctrl —
  por medir se o VS Code o faz.
- **Herdados do I15 §9** não fechados: a secção Script do Inspector por fotografar; a sonda do solver sem
  `--docs`; a regra de Fatin nas caudas (ruído do par 0,6–1,0 ms). Os herdados do
  [I13 §9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md#9-aberto) continuam.
- Integração: falta `clippy --workspace` e a suíte do fecho; CLAUDE.md §5.1 `1|2` → `1..4`.

## 10. Próximo

Smoke do dono da cena 4 → `/pd-linha-fechar` (clippy + suíte + mutação sobre `reach` / `goto::documents` / cache
de `problems()` / `Tabstops::merge`). Com o smoke OK, a fila do plano acaba; o que resta é o §9.
