# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I6 + I11 (17/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado · medição:
> [I6_I11_definicao_e_cores.md](../ferramentas/medicoes_onda2/I6_I11_definicao_e_cores.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  base = `main` `ad5f8fe39c` (o `PROJECT_SCHEMA` lá é 191) · HEAD `3b536c7623` (antes do commit deste handoff).
- Commits sobre o `main`: `444bf8249c` (docs da reabertura) · `796f423015` feat I6+I11 · `2393431a55`
  test (Ctrl+clique/F12 e as cores, contra o servidor falso e o `luau-lsp` fixado) · `c7f74e65ee` test
  da composição (mata a M18) · `132c71b9bb` fmt · `3b536c7623` clippy. Total: **6 commits**, 36 ficheiros (+ os docs desta fecha).
- Sem o assistente o editor continua a editar (nada aqui depende dele).

## 2. Superfície de colisão (`bash <worktree>/scripts/collision-surface.sh`, 17/10)

```
merge-base ad5f8fe39c · 5 commits · 36 ficheiros
PROJECT_SCHEMA 191 (= base) · tripla (191, 13, 23) (= base) · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20
Registos de componentes 108/109/109 (= base) · contrato congelado: node.rs e tool.rs intocados
ADR: nenhum criado (próximo livre 0189) · Cargo.lock: nenhum pacote externo novo · conflitos: nenhum
TETOS DE LOC: nenhum ficheiro da linha passa do teto
```

## 3. O que a onda fez

**I6 — ir para a definição.**
- `ph2d-code-lsp`: o cliente guarda o resultado inteiro do `initialize` (`capabilities()`); declara
  `definition` e `semanticTokens` (full).
- `ph2d-code-intel`: `Req::Definition(usize)` (género 3) → `Answer::Definition(Vec<Target>)`;
  `lsp/sem.rs::definition` aceita `Location`, `Location[]` e `LocationLink[]`.
- `ph2d-code-syntax`: `Syntax::property_declarations` (nome + bytes de cada `ph2d.property`).
- `ph2d-app-code/src/goto.rs`: alvo neste script ⇒ cursor; `self.<nome>` de uma `ph2d.property` ⇒ a
  declaração NOSSA (o servidor não liga o literal ao campo); o resto (API, resposta `[]`) ⇒ `Hints::pin`:
  o balão do I5 preso ao nome (o rato não o leva). Tecla, clique ou edição largam o pedido
  (`CodeDoc::edited` faz `self.goto = None`, também na composição). Gatilhos: Ctrl/Cmd+clique em
  `pointer.rs::click_text`; F12 em `keys.rs`.

**I11 — tokens semânticos.**
- `ph2d-code-intel`: `Req::Tokens` (género 4, sem posição) → `Answer::Tokens(Vec<SemToken>)`;
  `lsp/sem.rs::Legend::of` (legenda do servidor) e `tokens` (deltas relativos em UTF-16 → bytes);
  `Sem::{Library, Parameter, Property, Function, Variable, Type}`.
- `ph2d-app-code/src/semantic.rs`: `SemHeld`; pausa `IDLE_NS` 1,5 s / `IDLE_ERRORS_NS` 0,5 s (= o
  `idle_parse_delay` / `_with_errors_found` do Godot 4.7.2), contada em `Mirror.changed_ns`; uma pergunta
  por (arranque, versão); ao abrir chegam sem pausa. Token tocado por uma edição larga-se; resposta em
  voo larga-se; resposta velha nunca pinta. Só recolore NOMES (`names_only`). O global inexistente pinta
  `Ink::Unresolved` a partir do diagnóstico `1001` seguro (o servidor não emite token para ele).
- `ph2d-panel-code`: `Ink::Unresolved` → `ColorToken::CodeUnresolved`. Sonda `ink_painted` /
  `semantic_count` em `docs_probe.rs`; `CodeDoc.painted` regista as cores do último quadro.

## 4. FOUNDATIONAL tocado (append-only) e contadores que SOMAM

| Onde | O quê | Aditivo porque |
|---|---|---|
| `ph2d-editor-core/src/doc_surface.rs` | `Key::F(u8)` (última variante); `key_of` em `surface_input.rs` | todos os `match` sobre `Key` têm curinga |
| `ph2d-editor-core/src/screens/hero/board_keys.rs` | `BoardKey::F(u8)` (última variante) | idem |
| `shells/desktop/src/input_dispatch/keyboard_board.rs` | traduz F1–F12 (+12 linhas) | shell **187 601 → ~187 613** contra o tecto 189 041 (a catraca `the_shell_only_shrinks`) |
| `ph2d-tokens` + `docs/design/tokens.json` | `ColorToken::CodeUnresolved` (`code-unresolved`) no fim do bloco `code-*`; forge `oklch(0.780 0.220 38)`, sunstone `oklch(0.500 0.220 38)`, blueprint `oklch(0.530 0.220 30)` (workshop herda; os modernos emprestam pelo prefixo `code-`) | token novo, nenhum existente muda |
| `ph2d-tokens/src/contrast.rs` | `CONTRAST_PAIRS` **+1** (`code_ink(CodeUnresolved)`) | par novo |

Números que SOMAM entre linhas (**o integrador RECONTA, não escolhe**):
- censo `quantas_entradas_tem_cada_painel`, painel tokens: **3 466 → 3 491** (+25 = 1 × `ROW_H_PX`);
- gate `the_code_inks_read_and_differ`: passou a **11 tintas**;
- `CONTRAST_PAIRS` +1;
- LOC da shell: +12 (acima).
Não toca: `PROJECT_SCHEMA`, ADR, registos de componentes, cenas de smoke, i18n.

## 5. Números medidos

[Medição completa](../ferramentas/medicoes_onda2/I6_I11_definicao_e_cores.md) (sonda
`../ferramentas/lspprobe_i6_i11.py`, ~3 s, `loadavg` 4,27). `luau-lsp` 1.70.1: tokens só para nomes
(local e global inexistente: nenhum); 116 tokens e 12 definições, iguais com e sem `--docs`. Definição:
local/parâmetro/`local function` = a declaração; `self.<propriedade>` e toda a API = `[]`. Pausa das
cores = a do Godot 4.7.2. Cor de alerta: `danger` reprovou (3,49–4,39:1 na linha do cursor em 6 temas;
ΔE 0,044 do `code-constant` em 6); a 1.ª cor `0.720 0.220 35` reprovou (PlumberRed 4,32, PlumberGreen
4,01, Candy 4,10). Ambas na tabela ⛔ Recusas MEDIDAS do doc.

## 6. Testes e mutação

- `definicao::` **7**: local → declaração (controlo sem Ctrl) · F12 na API abre o balão preso e o cursor
  fica · `self.amplitude` → a `ph2d.property` · tecla antes da resposta larga · composição antes da
  resposta larga · comentário não pergunta · definição e cores em voo não se cancelam.
- `cores::` **5**: ao abrir chegam sem pausa (biblioteca/parâmetro/comentário/alerta) · a escrever não
  pergunta, pergunta aos 0,5 s com erro e aos 1,5 s sem · seguem edições e largam o nome tocado ·
  resposta velha nunca pinta · a tecla nunca espera com o servidor a 0,2 s/mensagem.
- `oraculo_lsp::o_assistente_fixado_vai_a_definicao_e_pinta_as_cores` (`luau-lsp` 1.70.1 real) ·
  unitários `sem::` 2, `semantic::` 2.
- O servidor falso (`ph2d-app-code/tests/it/assistente.rs`) ganhou `Plano.definir`/`colorir`,
  `Visto.definicoes`/`cores`, e responde ao `initialize` com as capacidades gravadas do 1.70.1.
- **Mutação: 24/24 sangram.** Idle com erros ignorado · idle 0 · token tocado não larga · ticket não
  largado na edição · tokens não deslocam · sem `names_only` · sem o 1001 · sem `defaultLibrary` ·
  coluna absoluta · sem «uma por versão» · sem `changed_ns` · sem a propriedade · alvo ignorado · balão
  preso não preso · sem Ctrl+clique · F12 noutra tecla · sem `in_code` · edição sem largar o goto (M18) ·
  paint sem as cores · método `range` · `Tokens` no género da Definição · sem legenda · propriedade com
  as aspas · biblioteca → função. A M18 sobreviveu à 1.ª rodada (o teste largava o salto pelo teclado);
  cura = `uma_composicao_antes_da_resposta_larga_o_salto` (pedido em voo + `Compose::Commit`).
- Corrida (nomes): `bash scripts/cargo-test-narrow.sh ph2d-app-code definicao` · `… cores` ·
  `… o_assistente_fixado_vai_a_definicao`.

## 7. O que um MERGE pode partir

- **`Key::F(u8)` / `BoardKey::F(u8)`**: se outra linha também acrescentar uma variante no FIM do mesmo
  enum, o conflito é textual (duas últimas linhas) e a resolução é manter as duas; o `mergiraf` pode
  calá-lo — confira os dois enums e `key_of`.
- **Censo do tokens** (`quantas_entradas_tem_cada_painel`): outra linha que mexa em linhas do painel de
  tokens muda o 3 491; reconte, não some de cabeça.
- **`CONTRAST_PAIRS` / `the_code_inks_read_and_differ`**: tinta nova de outra linha ⇒ o gate pede ΔE ≥
  0,075 contra `code-unresolved` também, nos 12 temas.
- **Tecto da shell** (`the_shell_only_shrinks`): +12 linhas em `keyboard_board.rs`; outra linha que
  também suba a shell soma.
- Os tokens modernos emprestam `code-unresolved` pelo prefixo `code-`: um tema novo sem o prefixo reprova.

## 8. ABERTO

- O sublinhado de «link» com Ctrl+rato (`symbol_lookup_on_click` do Godot, VS Code) **não** foi feito.
- F13–F24 não são traduzidos (`Key::F` leva o número, a shell só emite F1–F12).
- A definição noutro script não existe (cada script é um ambiente, I0b §6 — nunca acontece).
- O servidor parado mantém as cores (seguem as edições até a próxima pergunta que não vem).
- A bancada com cores + balões + lista: I15, numa rodada só.

## 9. Próximo

**I7** (correcções nossas: globais `1001`, membros da API `1002`, `self.<campo>`) → I8 → I10 →
I9/I12/I14 → I13 → I15 (Ordem do [03_onda2.md](../03_onda2.md)).

## 10. Smoke para o DONO

> ⚠️ Não fotografado (a foto e a cena própria `PH2D_CODE_SMOKE=3` são o I15). Precisa do assistente
> instalado na máquina, como nos testes (`bash scripts/luau-lsp-fetch.sh`).

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Hierarchy clique em **Circle**; no Inspector, cartão **Script**, clique **Edit script**: abre a aba **bob.luau**.
3. **Cores:** espere um instante. Nas linhas `ph2d.set(self.id, "x", …)` e `ph2d.set(self.id, "y", …)`, as palavras `ph2d` e `set` passam a ter a mesma cor de `math` (a cor das coisas que já vêm com o programa); antes `ph2d` era branco.
4. **Ir para a definição de uma variável:** segure **Ctrl** e clique na palavra `brilho` do fim da linha `local d = s * self.amplitude * brilho`: o cursor salta para a linha de cima, `local brilho = …`.
5. **Ir para a definição de uma propriedade:** segure **Ctrl** e clique em `amplitude` de `self.amplitude` (na mesma linha): o cursor salta para o topo do script, para `ph2d.property("amplitude", …)`.
6. **A documentação de uma função do programa:** clique na palavra `set` de `ph2d.set(self.id, "x", …)` e carregue **F12**: aparece logo a caixinha com `function ph2d.set(…)` e a frase do que ela faz; o cursor não sai do sítio. Mexa o rato por cima de outras palavras: a caixinha fica. Carregue **Esc**: some.
7. **Um nome que não existe:** numa linha nova (clique no fim de `self.t = self.t + dt` e carregue Enter) escreva `pirnt(1)`: `pirnt` fica sublinhado E com as letras num vermelho-alaranjado. Corrija para `print(1)`: o sublinhado e a cor vermelha somem e, parando de escrever um segundo e meio, `print` fica com a cor de `math`.
8. Deu errado se: o Ctrl+clique não salta (ou salta para o sítio errado); o F12 em `set` não mostra nada ou mexe o cursor; a caixinha do F12 some ao mexer o rato; `ph2d` não muda de cor; `pirnt` fica sublinhado mas com letras normais; a escrita fica lenta; ou a barra de baixo diz «Code assistant not installed» (aí falta o assistente na máquina, não é defeito do editor).

## 11. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, HEAD `3b536c7623`, corrido 2×;
a 2.ª saída:

```
▸ linha line_codeeditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.41s
```

## 12. Portões da onda (17/10, `/pd-onda-fechar`)

- `cargo fmt --all` · clippy `-D warnings --all-targets` VERDE em ph2d-code-lsp, ph2d-code-intel,
  ph2d-code-syntax, ph2d-app-code, ph2d-panel-code, ph2d-tokens, ph2d-panel-registry-init,
  ph2d-editor-core, ph2d-host-desktop (a 1.ª rodada reprovou `single_match` no `goto.rs` e
  `type_complexity` nos planos do servidor falso: curados em `3b536c7623`, aliases `Sitios`/`Tokens`).
- Testes por nome verdes: `--test it` cores/definicao/dicas/assistente/completar/oraculo_lsp **48**;
  `semantic::` 2 · `sem::` 2 · `facts` 2 · tokens `the_code_inks_read_and_differ` +
  `every_token_derives…` 2 · tokens `--lib contrast` 9 · `quantas_entradas_tem_cada_painel` 15
  (`--all-features`) · editor-core `the_shell_only_shrinks`/`file_loc_caps`/`workspace_file_loc_cap` 3.
- Suíte (`PH2D_SUITE=onda nextest-impacted`, a cadência deixou correr — a fundação mudou):
  **17 832 / 17 834**, 2 TIMEOUT em `ph2d-tool-painter` (não tocado):
  `macro_track::tutorial::tests::pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` (600 s) e
  `presets_de_fabrica::a_largura_e_a_prometida` (360 s), com o `loadavg` a subir de 30 a 92–135.
  Sozinhos, 1× cada: **verdes** em 115,6 s e 195,5 s (`loadavg` 28 → 79). Lentos sob carga
  (família FLAKES_DE_CARGA), não desta onda; não promovidos.
- Tectos de LOC (`collision-surface.sh`): nenhum ficheiro da linha passa do tecto.
