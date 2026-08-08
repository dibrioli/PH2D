# HANDOFF — `line/Vector`, a reforma dos TOKENS (2026-08-06)

> **Para quem assume a linha.** Faça a **FASE 0** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> **antes de abrir qualquer arquivo** — a janela abre na raiz (que é `main`) e os mesmos paths
> relativos existem nas duas árvores: editar a errada **compila e commita sem um único erro**.
> Módulo = `Vector`. Worktree: `Worktrees/line-Vector/`. HEAD deste handoff: **`85056908a`**.
>
> O plano-mãe é [`PLANO_UI_UX_padrao_figma.md`](PLANO_UI_UX_padrao_figma.md); este doc é o
> **estado** e a **ordem**, não a especificação.

---

## 1. O que já shipou (não reconstrua)

| Wave | O que é | Smoke |
|---|---|---|
| **W4b.1** — o ALIAS | Um token de cor **SEGUE** outro, no mesmo modo. Gesto de duas etapas no botão de corrente da row. Detecção de ciclo **na porta de escrita**. Viaja no arquivo. | ✅ `=59` |
| **W4b.2** — o CONTRASTE | A lei WCAG virou **DADO** (`ph2d_tokens::contrast::CONTRAST_PAIRS`): **uma lista, dois consumidores** (o gate de compilação e o painel). Bloco de aviso + marca nos **dois** lados do par. | ✅ `=59` |
| **W4c.1** — a CAMADA NUMÉRICA | A escala (`spacing.*`/`radius.*`/`stroke.*`) vira autorável, no molde exacto do de cor: chave `(modo, token)`, porta única, ciclo na porta, alias que atravessa famílias (px é px). Row com chip+elo+Reset no MESMO painel; viaja no arquivo. ⚠️ **`PROJECT_SCHEMA` 57→58** (PROVISÓRIO). | ✅ `=59` |
| **W4c.3** — MATH | `NumValue::Expr(String)` + a crate leaf **`ph2d-token-math`**: `{spacing.md} * 2` resolve VIVO. O parser é **injectado** por fn-pointers (a `ph2d-tokens` é folha de 44 widgets e `ph2d-expr-parse` arrasta o `ph2d-nodegraph`). ⚠️ **`PROJECT_SCHEMA` 58→59**. | ⏳ `=59` |
| **W4c.4** — ESCALA no DOCUMENTO | A espessura de um traço e o vão de um auto layout passam a SEGUIR um token numérico. ⚠️ A régua px↔mundo é a do PROJETO (`pixels_per_meter`) — sem ela o número erra por duas ordens de grandeza. ⚠️ **`PROJECT_SCHEMA` 59→60**. | ⏳ `=51` |
| **W4c.5** — DTCG | Import/export do grafo em `.tokens.json` (o W3C do Tokens Studio / Style Dictionary). O mapeamento é a **IDENTIDADE** — a chave já é o caminho DTCG. Crate leaf nova `ph2d-tokens-dtcg`. ⚠️ **Zero schema, zero ADR, nenhuma dep externa nova.** | ⏳ `=59` |
| **W4c.2** — A ESCALA VIVA | `Spacing::px()` passa a ser o valor **AUTORADO**: `num_runtime` projecta o grafo para uma tabela plana uma vez por quadro, e os ~1187 sítios de leitura ficam vivos **sem serem tocados**. Os 13 `const` viraram `fn` (o compilador enumerou-os, e achou 3 que a varredura não via). ⚠️ **PENDENTE DE SMOKE** — handoff [`HANDOFF_INTEGRACAO_line_Vector_W4c2_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c2_2026-08-06.md). | ⏳ `=59` |

**Aprovados pelo Enio em 2026-08-06** (*"Tudo perfeito"*), cena `PH2D_BUILD_SMOKE=59`.

⚠️ **A razão de a W4b.2 existir está num gate, não numa opinião:**
`the_compile_time_check_cannot_see_an_authored_break`. Um teste de unidade corre com a camada de
override **vazia**, logo afirma sempre a tabela de FÁBRICA; a cor que o artista escolhe move o
valor efetivo **em runtime**, onde nenhum teste está a olhar. Se esse gate um dia falhar, o
readout tornou-se redundante e a wave pode ser retirada.

**Também nesta jornada** (fora da fila de tokens): a row **Duplicate** da Hierarchy passou a
duplicar uma FORMA pela porta do painel (`3beeaadfb`), o roteiro de smoke ganhou a porta
`smoke_script` (`770249093`), e o log de seam morto cala para o color picker **por id**
(`fd14331c3`). Os três **pendentes de smoke**.

---

## 2. ⚠️ A MEDIÇÃO que reordena tudo o que falta

O plano tratava *math* (`{spacing.md} * 2`) e *os tokens de escala* como bloqueados pela fronteira
**`const fn`**, com a leitura implícita de que autorar `Spacing` custaria **runtime**.

**Medido em 2026-08-06 — a premissa está invertida.** `ColorToken::resolve` (`color.rs:510`), que
TODO widget chama para TODA cor em TODO frame, já paga por chamada:

1. um lookup **thread-local** na camada de override (`resolved_override`), e
2. **`lookup_color`: uma varredura LINEAR comparando STRINGS** sobre a tabela do tema (~350 folhas).

E o app entrega **60 fps** assim. Um lookup numérico para `Spacing` é **estritamente mais barato
que o que já shipa**.

⇒ **A parede nunca foi de performance. É de CONTEXTO DE COMPILAÇÃO:** `const PAD_Y: f32 =
Spacing::Sm.px();` não pode chamar uma fn não-const — e são **15 sítios `const`** assim
(`grep -rn "const .*Spacing::.*\.px()" crates shells`).

> ⚠️ **A W4c.2 mediu, e o número deste parágrafo era metade da história.** Os `const` são **13**
> (a varredura acima perde os que vivem dentro de um `const fn` — o compilador achou-os), e os
> sítios de LEITURA são **~1200**. Trocar cada um por `px_live(theme)` seria mil e duzentas edições
> para responder mil e duzentas vezes uma pergunta que o app responde **uma** vez por quadro. Por
> isso a resposta virou ao contrário: `px()` **é** o vivo, `factory_px()` é a fábrica, e a tabela
> plana do `num_runtime` é publicada uma vez por quadro pela ponte. Detalhe no `num_runtime.rs`.

**A arquitetura do padrão-ouro é a que o próprio plano enuncia** (§(b), Vol. 2 §4): *a tabela
achatada por modo é a forma de RUNTIME; o grafo de autoria vive no editor.* Portanto:

- **`px()` continua `const fn`** e continua a valer a **FÁBRICA** — os 15 `const` seguem legais, e
  um build de jogo que nunca autora nada fica **byte-idêntico**;
- nasce o acessor **VIVO** (a irmã que consulta o override) ao lado;
- o grafo (alias · math · ciclo · DTCG) **resolve PARA** a tabela plana. O jogo carrega o plano.

---

## 3. As waves que faltam, NESTA ordem

> Cada uma fecha com **UI na mesma wave** e **smoke próprio** — a lei desta linha
> ([[feedback_ship_the_ui_in_the_same_wave_not_later]]).

### ~~W4c.1 — A CAMADA NUMÉRICA~~ *(FEITA 2026-08-06, pendente de smoke)*
`ph2d-tokens::num` (a identidade `NumToken`) + `num_overrides` (a camada) + a row no painel + a
ponte + o arquivo. **A família é *o que se mede em PIXELS*** — Spacing+Radius+Stroke —, e é isso que
faz **um DEGRAU novo ser fiação**: uma escala nova é uma entrada na macro `num_tokens!`.

> ⚠️ **Esta frase dizia *"e é isso que faz a W4c.4 ser fiação"*, e a W4c.4 a corrigiu.** As duas
> coisas não são a mesma: acrescentar um **token** à tabela é de facto uma linha de macro;
> acrescentar um **ALVO** que o segue (uma propriedade do documento) esbarra numa fronteira que
> ninguém tinha medido — o token vale PIXELS e o documento mede MUNDO. Ver a W4c.4 abaixo.

⚠️ `Motion` (ms), `Density` (já é escolha) e `chrome.*` (sem identidade de token) ficaram FORA, cada
um com o motivo escrito no `num.rs`.
⚠️ **`PROJECT_SCHEMA` 57→58** (PROVISÓRIO) — o `SavedValue` ganhou `Number(f32)` e o numérico viaja
na **MESMA lista** `tokens`, roteado pela CHAVE.
Detalhe, colisões e o que smoke-testar: [`HANDOFF_INTEGRACAO_line_Vector_W4c1_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c1_2026-08-06.md).

### ~~W4c.2 — OS 15 SÍTIOS~~ *(FEITA 2026-08-06, pendente de smoke)*
Virou **a tabela achatada** em vez das 1200 edições (ver a nota na §2). O `TOOL_RAIL_WIDTH_PX`
cascateou como previsto (dois `pub const RAIL_W` → `fn rail_w()`), e o **TETO foi MEDIDO**: com
`spacing.* = 1024 px` o *Reset This Mode* pousa em `y = 2206` numa viewport de 900 **e a rolagem
alcança-o**, em toda escala testada até `65536 px`. ⇒ **não há cap a escrever** — o penhasco
`y ≈ 158 + 2·px` é função da altura da JANELA, e um literal estaria errado para metade dos
monitores. Gate: `ph2d-panel-tokens/tests/scale_ceiling.rs`.

### ~~W4c.3 — MATH~~ ✅ FEITA (2026-08-06, `ae56c1abd` + `1754f14e2`)
`NumValue::Expr(String)` + a crate leaf **`ph2d-token-math`**. `{spacing.md} * 2` é o que o plano
pediu desde o início, e ele resolve VIVO (mudar o `spacing.md` move quem o lê).

⚠️ **O parser NÃO entrou na `ph2d-tokens`, e a razão é uma aresta MEDIDA:** `ph2d-expr-parse`
arrasta `ph2d-expr` → `ph2d-nodegraph`, e a `ph2d-tokens` é a folha de que 44 widgets dependem. Ele
é **injectado** (`num_expr::MathHost`, dois fn-pointers), o padrão do `LutSpec`/`set_ml_available`.

⚠️ **As chaves vêm entre `{}` por MEDIÇÃO:** o lexer partilhado só junta um `.` ao identificador
quando o que vem a seguir é uma LETRA, e **quatro** chaves têm um dígito (`spacing.2xl`, `3xl`,
`4xl`, `radius.2xl`). Alargar o lexer moveria o significado de `x.5` para dois consumidores que já
shipam.

⚠️ **A lei do ciclo ganhou FAN-OUT** (`alias_walk`): um alias tem 1 sucessor, uma expressão tem N ⇒
DFS com conjunto-visitado, que **subsumiu** a casa dos pombos.

Deixado ABERTO com o motivo, não esquecido: **a fórmula não é oferecida à COR** (seria um terceiro
`TokenValue` a responder a mesma pergunta — ver §4) · **um readout de dependências** (*quem lê este
token?*) não existe, e o preço de o não ter é o artista descobrir um laço pela recusa · **o campo
não faz auto-completar** de chaves.

### ~~W4c.4 — ESCALA~~ ✅ FEITA (2026-08-06, `ba37d0725`)
A espessura de um traço e o vão de um auto layout passam a poder **SEGUIR** um token numérico.

⚠️ **A frase desta linha estava ERRADA, e o (1) não tinha culpa.** Ela dizia *"cai de graça … se
custar mais que fiação, o (1) foi feito estreito demais"*; o custo estava noutro eixo, que ninguém
mediu: os três alvos vivem em **unidades de MUNDO** e um `NumToken` vale **PIXELS**. Ler o número
cru como mundo erra por duas ordens de grandeza — `stroke.default = 1,5` viraria **19% da altura**
de uma moldura de telefone, e `radius.full` daria 125 molduras.

A régua **já tinha dono**: `ProjectSettings::pixels_per_meter` (ADR-0131 D4). Não é o `px_to_world`
da câmera, que é px de TELA no zoom do momento — resolver por ele faria o valor SALVO depender de
onde o artista estava a olhar.

⚠️ **`CornerRadius` continua fora, e o motivo MUDOU:** o canal chegou (W4c.1); o que falta é um
controle **por-FORMA** — o raio é por-VÉRTICE e um binding é por-forma.

⚠️ **`PROJECT_SCHEMA` 59→60** (PROVISÓRIO) · `VEC_SCENE_SCHEMA` **intacto** (o binding é tabela
LATERAL no ECS) · **zero `Cargo.toml`**, **zero ADR**.

Detalhe, as 4 mutações que expuseram buraco de gate e o roteiro de smoke:
[`HANDOFF_INTEGRACAO_line_Vector_W4c4_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c4_2026-08-06.md).

### ~~W4c.5 — DTCG (o W9)~~ ✅ FEITA (2026-08-07, `85056908a`)
Import/export do grafo em `.tokens.json` — o formato W3C que o Tokens Studio / Style Dictionary /
Penpot falam.

⚠️ **O aviso desta linha estava certo, e a conferência fechou a questão:** o mapeamento é a
**IDENTIDADE**. Um caminho DTCG é `grupo.token` e um alias é `{spacing.md}` — as nossas chaves já
são exactamente isso, porque o `num.rs` pôs o ponto ali escrevendo que *"o ponto é também a forma
que o DTCG fala (W4c.5)"*. Pôr as cores debaixo de um grupo `color` daria caminhos mais bonitos e
**quebraria a coincidência**: a fórmula que o artista escreve no painel deixaria de ser o mesmo
texto que o arquivo carrega.

⚠️ **A forma do `$value` foi MEDIDA na spec, não assumida:** em `2025.10` cor e dimensão são
**objetos** (`{colorSpace, components, alpha, hex}` / `{value, unit}`); as strings `"#rrggbb"` e
`"12px"` são dos rascunhos anteriores, que metade do ecossistema ainda emite. **Escrevemos a da
spec** (com o `hex`, que ela própria chama de fallback) e **lemos as duas**.

⚠️ **A lei da wave:** o export traz a **tabela inteira** e o import **só autora o que DIFERE da
fábrica**. Sem essa segunda metade, reimportar um export de um projeto intocado autoraria os ~80
tokens e re-editar o `tokens.json` deixaria de alcançar o app **em silêncio**. Só vale para
literais — um alias e uma fórmula são estruturais.

**Crate nova `ph2d-tokens-dtcg`** (leaf) pela aresta que a `ph2d-tokens` declara no próprio
`Cargo.toml` (*"zero runtime deps"*) — o precedente da `ph2d-token-math`, e agora com **arch-gate**
a impedir a próxima linha de a furar. **Porta única nova `ph2d_tokens::route`** (*a chave decide a
família*), com o `project_tokens::install` a delegar.

⚠️ **Zero schema** (`PROJECT_SCHEMA` **60** e `VEC_SCENE_SCHEMA` **14** intactos), **zero ADR**,
**nenhuma dep externa nova** ⇒ esta wave fica **fora** de toda disputa de número.

Detalhe, as 13 mutações (uma sobreviveu e nomeou um buraco) e o roteiro de smoke:
[`HANDOFF_INTEGRACAO_line_Vector_W4c5_2026-08-07.md`](HANDOFF_INTEGRACAO_line_Vector_W4c5_2026-08-07.md).

---

## 3.1 ⭐ A FILA DE TOKENS FECHOU

As cinco waves do §3 estão feitas. O que resta desta linha é o §5 (fora da fila de tokens) e o que
a W4c.5 nomeou como **deliberadamente fora**:

- **Tipografia e motion em DTCG** — `Motion` mede-se em **ms** (outra régua) e `chrome.*` não tem
  identidade de token; um `$type: "duration"` precisa primeiro de o `Duration` ganhar `key()`/`ALL`.
- **SVG de entrada com hierarquia** e **exportação de sprites/atlas** — as outras duas metades do
  W9 no plano-mãe, com outros consumidores.
- **O `Resolver` do DTCG** (a spec separada que modela modos) não é lido nem escrito: um arquivo é
  um modo, que é o que o ecossistema faz.

---

## 4. ⛔ O que NÃO fazer (medido ou decidido — não re-litigue)

- **Math sobre COR como substituto do math numérico.** Seria um **terceiro** `TokenValue` a
  responder a mesma pergunta por outro caminho — a segunda porta que esta linha passou a jornada
  a colapsar. Se a cor derivada for pedida um dia, ela entra **depois** do `Expr`, reusando-o.
- **Silenciar `WidgetEvent::ValueChanged` como TIPO.** Aquele log é o **detector de seam morto**;
  a isenção é por **ID** (ver `forwarding.rs::expected_unhandled`, com gate das duas metades).
- **Converter os 15 `const` "por performance".** O motivo é **contexto de compilação**. Se alguém
  reabrir isto como assunto de custo, a medição do §2 é a resposta.
- **Uma segunda lista de pares WCAG.** `CONTRAST_PAIRS` é uma lista com dois consumidores; ela
  substituiu **quatro cópias** do mesmo laço.

---

## 5. Aberto FORA da fila de tokens

- **W7 — a metade de RUNTIME.** A máquina de estados é **PLANA**, não hierárquica. ⚠️ O **seletor
  de CURVA** fechou (**W7c, 2026-08-08** — handoff
  [`HANDOFF_INTEGRACAO_line_Vector_W7c_2026-08-08.md`](HANDOFF_INTEGRACAO_line_Vector_W7c_2026-08-08.md)):
  `set_easing` existia desde o v56 e **nenhum caminho de produto o chamava**. O que resta aqui é a
  **hierarquia** (um menu que abre com sub-estados) e a **MOLA**, que deixou de ser dispensável
  por ausência de regime e passou a ser decisão do Enio, com os números medidos.
- **W8a — o runtime dos jogos.** ⛔ **BLOQUEADO por ausência:** `ph2d-runtime` **não existe**
  (Front 2 não construída). Não é adiamento, é pré-requisito.
- **W2a — `VecTextParams.wrap_width`** + a quebra de linha real (o **parley** já está na árvore).
  ⚠️ Custa um **bump de `PROJECT_SCHEMA` global** — o número se **CONTA** contra o `main` do dia,
  nunca se escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

---

## 6. Duas armadilhas que MORDERAM nesta jornada

1. **A cwd do Bash escorrega para a árvore PRIMÁRIA.** Aconteceu duas vezes; uma delas um
   `python3` editou `tokens_smoke.rs` **no `main`**. Restaurado, mas: **prefixe TODO comando com o
   `cd` da worktree.**
2. **`git checkout` para desfazer uma mutação reverteu a feature inteira junto.** Desfaça mutação
   com **`cp` de um backup**, e `touch` depois (senão o cargo reusa o mutante).

⚠️ E uma que só o clippy pegou: uma inserção de teste **roubou o `#[test]` do gate vizinho**, que
parou de correr **em silêncio**. Depois de editar em massa um arquivo de testes, **conte os gates**.

---

## 7. Como rodar

```
# a cena dos TOKENS (o painel) — as duas waves aprovadas vivem aqui
env PH2D_BUILD_SMOKE=59 cargo run -p ph2d-host-desktop --release

# o roteiro impresso, para ler antes de escrever o próximo
cargo test -p ph2d-host-desktop --bins show_the_script -- --ignored --nocapture

# a bateria de fechamento desta linha
cargo test -p ph2d-tokens -p ph2d-panel-tokens -p ph2d-host-desktop
cargo clippy -p ph2d-host-desktop --all-targets
cargo test -p ph2d-editor-core --test no_tofu_glyphs
```

⚠️ **A linha NÃO integra e NÃO pusha sozinha** (CLAUDE.md §0.7): fecha, escreve o handoff de
integração (DIRETRIZ §1.5.9) e **PARA**, à espera de ordem explícita do Enio.
