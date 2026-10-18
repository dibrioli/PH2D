# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I8 (17/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; o [I6/I11](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I6_I11.md) e o
> [I7](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I7.md) também estão na linha e este I8 vem por
> cima deles. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha I8) · medição:
> [I8_explicacoes.md](../ferramentas/medicoes_onda2/I8_explicacoes.md). O mecanismo vive nesses dois; aqui
> só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  base = `main` `ad5f8fe39c` (o `PROJECT_SCHEMA` lá é 191) · HEAD `e8a77c5bab` (antes do commit dos docs).
- Commits da onda I8 (sobre `e63f371186`): `4114f73a9d` feat (explicações por classe) · `8d2e51071c` test
  (a calha nunca pergunta ao assistente; o porquê no cartão) · `26840698d9` test (as duas mutações
  sobreviventes) · `0e69d30349` fix (rótulo «name(s)» que `ph2d-editor-core it
  no_row_label_carries_its_own_unit` acusou; o comentário do `script_quebrado.luau` diz o que o cartão
  mostra agora) · `e8a77c5bab` docs (medição). **5 commits.**
- **Decisão do dono, 17/10:** explicações só em **inglês** (o item dizia PT/EN; ordem de 19/09 «tudo em
  inglês»; `ph2d-i18n` só tem Ingles/Teste). Memória registada.
- Sem o assistente o editor continua a editar e os avisos NOSSOS (`self.<campo>`) explicam-se na mesma.

## 2. Superfície de colisão (`collision-surface.sh`, 17/10, contra o main)

```
merge-base ad5f8fe39c · 21 commit(s) · 117 arquivo(s)  (a linha toda, I6–I8 incluídos)
PROJECT_SCHEMA 191 (= base) · tripla (191, 13, 23) (= base) · VEC 23 · FLIP 13 · DOC_VERSION 20 (= base)
registos de componentes 108/109/109 (= base)
contrato congelado: intocado · ADR: nenhum (próximo livre 0189)
Cargo.lock: 1 pacote '+name' novo = "ph2d-code-explain" (INTERNO; nenhum pacote externo)
marcadores de conflito: nenhum · TETOS DE LOC: nenhum ficheiro passa do tecto
  (docs.rs 687/700 — perto; note.rs 221; code-explain lib.rs 434)
```

**FOUNDATIONAL tocado:**

- `ph2d-editor-core`: `InspectorScriptError` ganhou o campo **`why`**. Quem o construir por struct
  literal noutra linha deixa de compilar.
- `ph2d-panel-code`: `Problem` ganhou **`explanation: Option<Explanation{kind,title,why}>`**;
  `HoverView.text` virou **`sections: Vec<HoverSection{title: Option<&str>, text}>`**.
- Crate NOVA `ph2d-code-explain` ⇒ **`Cargo.lock` muda** (aresta nova; resolver na integração).

**Contadores que somam:** nenhum (sem degrau de schema, sem token de cor, sem linha de painel, sem registo
de componente, sem cena de smoke; a `PH2D_CODE_SMOKE=3` é do I15).

## 3. O que a onda fez (resumo; mecanismo no doc da onda)

- **Medição:** corpus do I0c = 54 classes estáticas (1 661 sítios) + 47 do motor (669) = 101 ⇒ **55 `Kind`**
  (53 do Luau/motor + `unknown_field`/`unknown_field_near`, nossas). O editor NÃO mostrava a mensagem em
  lado nenhum (só ponto na calha, onda, contagem) e não existe lista de problemas. Oráculos (Godot 4.7.2
  MIT, luau-lsp 1.70.1 MIT, rustc 1.95, Python 3.14.8, Lua 5.5): nenhum explica ao lado; forma adoptada =
  `rustc --explain` em pequeno (título de uma linha com o nome real · porquê · conserto).
- **`ph2d-code-explain` (NOVA, folha sobre `ph2d-i18n`):** `explain(msg) -> Option<Explained{kind, args}>`;
  `Kind::ALL/id/title_key/why_key/ours`; `Explained::unknown_field(name, near)`. `read.rs` normaliza
  espaços, tira a posição de chunk (`bob.luau:3: `), `TypeError:`/`SyntaxError:`, e um tipo enorme vira
  «a table»/«a function» (i18n `code.explain.type_table/_function`).
- **`ph2d-i18n`:** `src/code_explain.rs` (`code.explain.<id>` e `.why`); `code.problem.unknown_field[_near]`
  saíram de `code_editor.rs` (agora `code.explain.unknown_field[_near]`).
- **`ph2d-panel-code`:** `Problem.explanation`; `HoverView.sections` (título marcado); sonda
  `CodeView::problem_dot(line)`.
- **`ph2d-app-code`:** `note.rs` (novo): rato parado 0,5 s no ponto da CALHA ⇒ as explicações da linha
  (não pergunta ao assistente: `Hovering::asks`); sobre o culpado sublinhado ⇒ a mensagem CRUA por cima
  da resposta do assistente; `Hovering.note/ripe`, `Note::{Gutter(line), Culprit}`; `Hints::close()` mantém
  o balão de problema quando o assistente pára; `CodeDocs::notes()` por quadro. Registo local `Seen` →
  `code-explanations-seen.tsv` (`PH2D_CODE_SEEN_FILE` · `$XDG_DATA_HOME/ph2d` · `~/.local/share/ph2d` ·
  `%APPDATA%\ph2d`): uma linha `<segundos>\t<classe>` por explicação que um balão DA CALHA abriu;
  `set_seen_file(None)` na `Mesa` dos testes. Sondas `seen_kinds()`, `problem_dot()`. `problems.rs`
  preenche a explicação das linhas com mensagem do Luau; `lint.rs` usa `Explained::unknown_field`.
- **Cartão:** `InspectorScriptError.why`; `script_inspector.rs::no_cartao` põe o título da explicação no
  lugar da frase do Luau (o chip do símbolo E7 fica); `script_avisos.rs::porque` pinta o porquê por
  baixo (Text2). O cartão não grava no registo.

## 4. O que um MERGE pode partir

- **`InspectorScriptError`** (ph2d-editor-core): campo `why` novo ⇒ struct literal noutro lado não compila.
- **`Problem`** (ph2d-panel-code): campo `explanation` novo; **`HoverView.text` não existe mais**
  (`sections`): qualquer linha que leia/construa o `HoverView` quebra.
- **`Cargo.lock`:** resolver aceitando o pacote interno novo; a crate entra no workspace (`members`) —
  conferir se outra linha também mexeu na lista.
- **i18n:** `code.problem.unknown_field[_near]` já não existem em `code_editor.rs`; quem os referir cai
  no id novo `code.explain.unknown_field[_near]`. `ph2d-app-code/src/docs.rs` está a 687/700: outra linha que
  acrescente ali estoura o tecto.
- **Testes:** os `script_inspector_tests` e o texto do `script_quebrado.luau` esperam a explicação, não a
  frase crua do Luau. `Mesa` dos testes põe `set_seen_file(None)` (sem isso o teste grava no disco).
- **`it no_row_label_carries_its_own_unit`** (ph2d-editor-core): nenhum título de explicação leva a
  unidade no texto («name(s)» reprovou).

## 5. Testes e mutação

- `ph2d-code-explain it censo::` **6**: o corpus inteiro (≥ 2 400 mensagens) tem explicação; cada linha
  das duas TSV; cada `Kind` com título + porquê sem marcador por encher e título ≤ 90 car.; nenhuma `Kind`
  não-nossa morta; o `1002` de 567 car. → «There is no “proeprty” here»; o `=` do smoke 2, e o mesmo com
  prefixo de chunk.
- `ph2d-app-code it explicacoes::` **3** (calha explica / culpado mostra o cru / nada antes da pausa · a
  calha nunca pergunta — o falso só conta balões com `pairar` · só a explicação conta como vista · sem
  assistente o aviso nosso explica-se · com o assistente PARADO — espera `assist_reason` «não volta»,
  escrevendo letras até o falso morrer — a calha ainda explica). `note::tests` **1** (registo).
  `ph2d-panel-inspector it o_erro_explicado_pinta_o_porque_por_baixo`; os testes do cartão de
  `script_inspector_tests` actualizados.
- **Mutação (agente `mutacao`): 19/19 sangram.** 1.ª rodada: 16/18 válidas sangraram, M16 inválida (a
  guarda tornava o `match` não exaustivo). Sobreviventes: **M9** (`close()` mantém o balão de problema:
  o teste do assistente parado nunca chegava a «parado» — `assist_text` também tem texto a arrancar) e
  **M10** (`&& h.asks()`: o falso só conta pedidos de balão com `pairar`). 2.ª rodada: M9, M10, M16b
  sangram.
- **Lições (memória a registar pela janela principal):** (a) um contador do falso que só conta com o
  plano ligado lê 0 e aprova um «nunca»; (b) «o assistente parou» prova-se pela RAZÃO da falha, não por
  haver texto na barra.
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --test it -- explicacoes::`

## 6. Portões da onda (17/10)

- `cargo fmt --all` sem mudanças · clippy `-D warnings --all-targets` verde nas 7 crates (code-explain,
  app-code, panel-code, i18n, editor-core, app-components, panel-inspector).
- Testes por nome verdes: app-code `it` 70/70 · panel-code `it` 16 (+2 ignorados) · panel-inspector
  `script` 15/15 · i18n 16/16 · code-explain `censo::` 6. `oraculo_lsp::`/`onda1::`/`code_smoke`
  re-corridos verdes depois do comentário do script.
- Suíte `nextest-impacted` com `PH2D_SUITE="motivo: crate nova + deps em Cargo.toml"`: **18 889 / 18 890**.
  A 1 vermelha = `ph2d-editor-core it no_row_label_carries_its_own_unit` (real; curada em `0e69d30349`,
  re-corrida por nome: verde). 25 lentos, todos passaram (ph2d-tool-painter, app-macros, app-light2d,
  app-motion); `loadavg` no fim 17,51/38,21/31,00 ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª saída:

```
▸ linha line_codeeditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

## 8. Smoke para o DONO

> ⚠️ Não fotografado (a foto e a cena `PH2D_CODE_SMOKE=3` são do I15). O passo 4 mostra mais com o assistente instalado (`bash scripts/luau-lsp-fetch.sh`).

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke`
2. Abre sozinha a aba **script_quebrado.luau**. Na linha 3 (`local x = = 1`) há um **ponto vermelho** à esquerda, ao lado do número da linha, e o segundo `=` está sublinhado.
3. **A explicação:** pare o rato em cima do ponto vermelho da linha 3 e espere meio segundo. Abre um balão: em cima, destacado, **A value is missing before “=”**; por baixo, a frase que diz o porquê e o que fazer («Luau wanted a value here … Look for a doubled sign, like = =, …»).
4. **A mensagem original:** pare o rato em cima do `=` sublinhado. O balão agora mostra a mensagem do Luau tal como ele a escreve, em inglês técnico, com **Expected identifier when parsing expression, got '='**, sem o texto destacado.
5. **O cartão:** na Hierarchy clique no **triângulo**. No Inspector, cartão **Script**, o aviso vermelho diz **line 3** e **A value is missing before “=”**, com o `=` num quadradinho, e por baixo a mesma frase do porquê.
6. **O aviso nosso:** feche e corra o nível 1: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`. Clique em **Circle**, cartão **Script**, **Edit script**. Na linha com `self.amplitude`, troque `amplitude` por `ampiltude` e espere um segundo e meio. Pare o rato no ponto amarelo dessa linha: o balão diz **self.ampiltude is never set — did you mean amplitude?** e, por baixo, que o **Alt+Enter** corrige.
7. Deu errado se: o balão abre antes de parar o rato; o ponto vermelho não abre balão; o balão do ponto mostra a mensagem técnica («Expected identifier…») em vez da explicação; o balão do `=` mostra a explicação em vez da mensagem original; o cartão ainda diz «Expected identifier when parsing expression, got»; ou a escrita fica lenta.

## 9. ABERTO

- **Lista de problemas (painel)** não existe — decisão do dono; a explicação vive só no balão da calha, no
  balão do culpado e no cartão.
- **O cartão não grava no registo** das explicações vistas (só o balão da calha grava).
- **Ctrl+rato em mão** (do I6) continua para o **I15**, com a foto e a cena `PH2D_CODE_SMOKE=3`.
- Só inglês (decisão do dono); outro idioma entra quando o app inteiro o tiver.
- `ph2d-app-code/src/docs.rs` a 687/700: a próxima onda que escreva ali move código para um módulo
  irmão primeiro (como o `note.rs` fez).

## 10. Próximo

**I10** (rename e referências + marcador do undo global) → I9/I12/I14 → I13 → I15 (ordem do
[03_onda2.md](../03_onda2.md)).
