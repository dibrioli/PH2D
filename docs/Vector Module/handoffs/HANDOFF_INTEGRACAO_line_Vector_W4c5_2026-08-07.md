# HANDOFF DE INTEGRAÇÃO — `line/Vector`, **W4c.5: a tabela sai e entra em DTCG** (2026-08-07)

**Status:** FECHADO 2026-08-07 · no `main` em `82da03d34` (o commit que trouxe este arquivo).

> **Para o integrador.** HEAD desta wave: **`85056908a`** (1 commit).
> Worktree: `Worktrees/line-Vector/`. Branch: `line/Vector`.
> ⚠️ **PENDENTE DE SMOKE** — a linha fecha, entrega isto e **PARA**.

---

## 1. O que esta wave entrega

O **W9 do plano-mãe** (a última da fila W4c): **import/export do grafo de tokens** no formato
**DTCG** — o W3C que o Tokens Studio, o Style Dictionary e o Penpot falam.

Dois botões no topo do painel de Tokens (`Export DTCG...` / `Import DTCG...`), um codec numa crate
nova, e uma porta única de roteamento que o arquivo de projeto passou a partilhar.

---

## 2. As decisões que decidem o resto

### 2.1 ⭐ O mapeamento é a IDENTIDADE — e a fila avisou para conferir antes de inventar um

Um caminho DTCG é `grupo.token`; um alias é a mesma coisa entre chavetas: `{spacing.md}`.
**As nossas chaves já são exactamente isso**, e de propósito — o `num.rs` escreveu, ao pôr o ponto
na chave numérica, que *"o ponto é também a forma que o DTCG fala (W4c.5)"*.

| O que temos | O que o arquivo tem |
|---|---|
| `"accent"` (cor) | um token `accent` na raiz |
| `"spacing.md"` (px) | o token `md` dentro do grupo `spacing` |
| `TokenValue::Alias(accent)` | `"$value": "{accent}"` |
| `NumValue::Expr("{spacing.md} * 2")` | o texto, **verbatim**, no `$extensions` |

⚠️ **Nenhum prefixo é inventado.** Pôr as cores debaixo de um grupo `color` daria caminhos mais
bonitos (`color.accent`) e **quebraria a coincidência**: a fórmula que o artista escreve no painel
deixaria de ser o mesmo texto que o arquivo carrega, e o round-trip passaria a precisar de uma
tabela de tradução — exactamente o *mapeamento inventado* contra o qual a fila avisou.

### 2.2 ⚠️ A FORMA do `$value` foi MEDIDA na spec, não assumida

A spec **2025.10** exige objetos:

- **color**: `{"colorSpace":"srgb","components":[r,g,b],"alpha":a,"hex":"#rrggbb"}`
- **dimension**: `{"value":16,"unit":"px"}` — a `unit` é obrigatória **mesmo em zero**

A string `"#rrggbb"` e a string `"12px"` são a forma dos **rascunhos anteriores**, que metade do
ecossistema ainda emite (o Penpot tem issue aberta por só aceitar hex; o Style Dictionary aceita as
duas). ⇒ **escrevemos a da spec** (com o `hex`, que a própria spec chama de *fallback* para
ferramentas de gamut limitado) e **lemos as duas**: recusar a antiga faria o interop falhar
exactamente nos arquivos que há para importar hoje.

⚠️ **`rem` é RECUSADO e CONTADO**, nunca convertido — converter exige um tamanho de fonte-raiz que
este app não tem, e escrever `16` seria inventar um número que ninguém autorou.

### 2.3 ⭐ O export traz a TABELA INTEIRA; o import só autora o que DIFERE da fábrica

Exportar só o autorado daria, num projeto de fábrica, um arquivo **vazio** — inútil como interop, e
sem os tokens que os `{...}` referenciam.

⚠️ Isso **obriga** a outra metade, e ela é a lei da crate: **um valor que já é o de fábrica NÃO é
autorado**. Sem ela, reimportar um export de um projeto intocado autoraria os ~80 tokens de uma vez
— e a partir daí re-editar o `docs/design/tokens.json` deixaria de alcançar o app, **em silêncio**.
É a mesma frase que a porta de escrita já diz: *"escrever a cor de fábrica como override não é o
mesmo que soltar"*.

⚠️ **Só vale para LITERAIS.** Um alias e uma fórmula são **estruturais** — o artista autorou o
*vínculo*, e o número que ele por acaso dá hoje não o desfaz.

⚠️ **Preço honesto, nomeado:** um literal autorado que por acaso *é* a cor de fábrica volta como
não-autorado. **A aparência do app é idêntica** (o `resolve` dá o mesmo valor pelos dois caminhos);
o que muda é a contagem do readout. A alternativa — autorar os ~80 — é muito pior.

### 2.4 UM ARQUIVO É UM MODO, porque o DTCG não tem modos

O nosso override é do par `(modo, token)`; o formato W3C não tem esse eixo (modos são um conceito de
*resolver*, uma spec separada que quase nenhuma ferramenta lê). Enfiar os quatro em `$extensions`
seria ilegível para toda outra ferramenta, o que anula o motivo de exportar.

⚠️ **O import escreve no modo VIGENTE**, nunca no que o arquivo diz: o artista vê um modo de cada
vez e a primeira linha do painel nomeia-o — a mesma lei do *Reset This Mode*. O modo do arquivo
viaja no `$description`, **para uma pessoa ler, não para uma máquina obedecer**.

### 2.5 A math não existe no formato

O `$value` de uma fórmula é o **número que ela dá** (o que todo leitor DTCG consome) e o **texto**
vai no `$extensions`, sob `dev.ph2d.tokens`. Um round-trip por aqui recupera a fórmula; um por outra
ferramenta recupera o número — a degradação honesta.

---

## 3. As duas peças estruturais

### 3.1 Porta única nova: `ph2d_tokens::route` — *a chave decide a família*

Duas coisas de fora chegam com um par `(chave, valor)`: o **arquivo de projeto**
(`project_tokens::install`, postcard) e o **import DTCG**. A lei é a mesma, e o próprio doc do
`project_tokens.rs` já avisava que *"o import/export DTCG (W4c.5) teria de as juntar de novo"*.

`project_tokens::install` **delega** — o corpo dele encolheu de ~55 linhas para ~20 e a suíte dele
(342 linhas de gates) ficou verde sem uma linha tocada, que é a prova de que foi *pure code motion*.

### 3.2 Crate nova `ph2d-tokens-dtcg` (leaf: `ph2d-tokens` + `serde_json`)

⚠️ **Ela é própria por uma aresta que a `ph2d-tokens` DECLARA no próprio `Cargo.toml`:**
*"design-data puro — zero runtime deps"*. Ela é a folha de que **44 widgets e todo painel**
dependem, e `serde_json` ali poria um parser de JSON no caminho de compilação de cada um deles para
uma feature que corre **duas vezes na vida de um projeto**.

É o precedente exacto da **`ph2d-token-math`** (que ficou de fora porque `ph2d-expr-parse` arrasta o
`ph2d-nodegraph`). A diferença: a math é **chamada de dentro** da `ph2d-tokens` (e por isso é
injectada por fn-pointers) e o codec **não é** — ele corre na fronteira, então uma dependência
normal na direcção certa basta.

⚠️ **E agora a frase tem gate:** `crates/ph2d-tokens/tests/the_leaf_stays_dep_free.rs` afirma que a
secção `[dependencies]` dela está **vazia**, com controle positivo. Um comentário não impede a linha
seguinte.

⚠️ **`ph2d-token-math` entrou nas `[dev-dependencies]` da crate nova** (o `src/` não a toca ⇒
machete-safe, o padrão da `ph2d-painter-brush` na `ph2d-flip-render`). A razão é uma propriedade do
PRODUTO: a porta de escrita **recusa** uma fórmula sem `MathHost` instalado, então um gate de
round-trip sem o parser não conseguiria sequer **criar** a fórmula que quer ver voltar.

---

## 4. A UI, na MESMA wave

- **`Export DTCG...` / `Import DTCG...`**, lado a lado, no topo do painel, ao lado do
  *Reset This Mode* — mesmo escopo (o modo inteiro, as duas famílias).
- ⚠️ **Oferecidos SEMPRE**, ao contrário do vizinho, e a assimetria é a decisão: um *Reset* de um
  modo de fábrica é um clique que não faz nada; um EXPORT de um modo de fábrica é o design system
  inteiro. **O gate tem o *Reset* como CONTROLE** — sem ele não distingue *"o par é sempre
  oferecido"* de *"tudo é sempre oferecido"*.
- Ids `TOKENS_DTCG_EXPORT` / `TOKENS_DTCG_IMPORT` (hash de string) + 2 chaves i18n.
- ⚠️ As reticências são **ASCII** (`...`), como o `Import Font...` do painel de vetor: a fonte cobre
  o `\u{2026}` mas o `no_tofu_glyphs` não o vigia, e este painel não tem outro botão de diálogo com
  quem ser consistente.
- O diálogo é **nativo** (`rfd`), na shell. ⚠️ O caminho **não** é fixo: um `PH2D_TOKENS_PATH` é a
  tentação barata e shipa uma feature que só o autor sabe usar.

⚠️ **A POLÍTICA foi cortada da I/O:** um diálogo bloqueia e precisa de janela, então tudo o que
decide (*que modo recebe, o que se mantém dos outros, por que porta se escreve, o que o artista lê*)
vive na `install`, que um gate dirige sem tocar num arquivo. Sem esse corte a wave seria "provada"
por um gate que só afirma que o codec funciona — com a costura de fora, que é onde as waves desta
linha falham.

---

## 5. Colisão de integração — o que conferir

| Item | Valor | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | **60**, INTOCADO | o `project.rs` **não é tocado** — `git diff` vazio |
| `VEC_SCENE_SCHEMA_VERSION` | **14**, INTOCADO | |
| ADR novo | **nenhum** | ⇒ fora da disputa de número desta janela |
| Contrato congelado | **intacto** | `architecture_contract_surface` 3/3 · `..._tool_...` 4/4 · `..._vector_...` 11/11, RODADOS |
| Registro do `ph2d-ecs` | **intocado** | nenhum componente novo |
| `Cargo.toml` centrais | **nenhum** | a crate nova entra pelo glob `crates/*` |
| **Dep externa nova** | **NENHUMA** | `serde_json` já está na árvore (`ph2d-mcp`, `ph2d-asset`, e dev-dep da própria `ph2d-tokens`) |
| Crate nova | `ph2d-tokens-dtcg` | + 2 arestas de path (`shells/desktop` → ela; ela → `ph2d-tokens`) |
| ids novos | 2 (`TOKENS_DTCG_EXPORT`/`_IMPORT`) | hash de string, no `node_id_collisions` |

⚠️ **O `node_id_collisions` ganhou a FAMÍLIA `TOKENS_*` inteira, não só o par novo:** os três consts
do W6 (`TOKENS_PANEL`/`_CLOSE`/`_RESET_ALL`) **nunca estiveram** naquela lista, e acrescentar só os
dois de hoje deixaria a lacuna aberta com a aparência de fechada.

**Ponto de merge sensível:** `crates/ph2d-editor-core/tests/node_id_collisions.rs` — a lista é
escrita à mão e outra linha pode ter acrescentado entradas no fim. **Só ADICIONE**; o gate falha
alto se duas entradas colidirem.

---

## 6. Gates e mutações

**Novos:** 9 no `route` · 26 na crate do codec (export + import + round-trip) · 5 na política da
shell · 3 no seam do painel · 4 no arch-gate da shell · 1 no arch-gate da folha.

**13 mutações, 13 sangram.** As que valem ser lidas:

| # | Mutação | Sangra |
|---|---|---|
| M2 | a `factory` de cor passa a consultar a camada de override | `the_factory_does_not_see_the_override_layer` |
| M3 | ⭐ o filtro de fábrica some do import (autora a tabela toda) | os DOIS gates de round-trip + o da lei |
| M4 | o alias é achatado no export | o do alias + o round-trip |
| M6 | duas casas decimais no componente de cor | `every_byte_level_survives_the_round_trip` |
| M7 | o `rem` é aceite como px | o das formas de dimensão |
| M9 | ⚠️ **SOBREVIVEU na 1ª rodada** | ver abaixo |

⚠️ **A M9 nomeou um buraco meu:** apagar o filtro de modo do lado **NUMÉRICO** do `install` passava
por todos os gates — o de *"os outros modos sobrevivem"* cobria só a família de **COR**. Duas
famílias são **duas camadas**, e camadas em série querem um gate cada
([[feedback_layered_defenses_need_per_layer_gates]]). O gate ganhou a segunda metade e a mutação
sangra.

⚠️ **E uma mutação foi INVÁLIDA, não um buraco:** a 1ª versão da M2 fazia a `factory` chamar o
`resolve` — que chama a `factory` — ⇒ recursão infinita. Ela sangrava por estourar a pilha, **não
pelo gate**. Refeita para consultar a camada *uma vez*, ela sangra o gate certo.

**Bateria de fechamento:** `nextest-impacted` **8948/8948** · `clippy --workspace --all-targets`
limpo · `fmt --check` limpo · `machete` limpo · os dois caps de LOC verdes · `no_tofu_glyphs` verde.

⚠️ **Duas flakes PRÉ-EXISTENTES apareceram na varredura e passam isoladas** — as duas já estão
documentadas no CLAUDE.md §5 como gates de RAZÃO sensíveis a carga:
`ph2d-tool-painter … the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` e
`ph2d-timeline::nesting_clock the_cost_of_depth_is_linear_not_explosive`. Esta linha **toca zero
arquivos** nas duas crates. Re-rode-as sozinhas antes de suspeitar de um merge.

---

## 7. O SMOKE

```
env PH2D_BUILD_SMOKE=59 cargo run -p ph2d-host-desktop --release
```

O **passo 12** é a wave (o roteiro imprime-se no terminal ao abrir):

1. **Export**: autore uma cor, um elo e uma fórmula → `Export DTCG...` → escolha um lugar.
2. **Abra o arquivo num editor de texto** — ele é para ser LIDO: cada token com `$type` e `$value`,
   o elo como `{accent}`, e a fórmula no `$extensions` (o `$value` dela é o número que ela deu).
3. **Import**: *Reset This Mode* → `Import DTCG...` do mesmo arquivo → **os três voltam**, e a
   fórmula volta como **FÓRMULA**, não como o número.
4. ⚠️ **A pergunta que decide a wave:** import de um arquivo de **fábrica** tem de autorar **ZERO**
   (o toast diz *"already at factory"*). **Se ele autorar ~80, PARE** — a tabela de fábrica ficou
   inalcançável e re-editar o `tokens.json` deixa de chegar ao app.
5. Aperte **M** antes do Import: ele re-veste o modo que está **na tela**, não o que o arquivo diz.

**Interop de verdade (opcional, e é o que a wave existe para):** abra o `.json` exportado no Tokens
Studio / num pipeline de Style Dictionary.

---

## 8. Aberto, nomeado

- **A tipografia e o motion ficam de fora**, e o motivo está escrito no `num.rs`: `Motion` mede-se
  em **ms** (outra régua) e `chrome.*` não tem identidade de token. Um `$type: "duration"` é uma
  wave própria, e ela precisa primeiro de o `Duration` ganhar `key()`/`ALL`.
- **`$type` não é consultado no import** — a CHAVE já decide a família, e perguntá-lo daria uma
  segunda resposta à mesma pergunta, com o modo de falha de o arquivo cair por uma *anotação*
  (`"number"` em vez de `"dimension"`, que várias ferramentas emitem) em vez de por um valor.
- **A herança de `$type` do grupo** existe na spec e **não é escrita** (todo token carrega o seu):
  escrevê-la faria o arquivo depender de o leitor a implementar.
- **O `Resolver` do DTCG** (a spec separada que modela modos) não é lido nem escrito.
- ⚠️ **O SVG de entrada com hierarquia** e a **exportação de sprites/atlas** — as outras duas metades
  do W9 no plano-mãe — **não** entraram: são outros assuntos com outros consumidores.

---

## 9. Como rodar a bateria desta wave

```
cargo test -p ph2d-tokens -p ph2d-tokens-dtcg -p ph2d-panel-tokens
cargo test -p ph2d-host-desktop --bins tokens_bridge_dtcg
cargo test -p ph2d-host-desktop --test the_dtcg_interop_asks_for_the_file
cargo test -p ph2d-editor-core --test node_id_collisions
bash scripts/nextest-impacted.sh
```
