# HANDOFF — linha `line/Vector` (2026-08-22)

**Status:** FECHADA, à espera de integração. O Enio deu a ordem explícita de integrar ao `main`.
**Para:** o **agente INTEGRADOR** e o próximo agente da linha.

> ⚠️ **A linha entrega um DEFEITO CONHECIDO e por diagnosticar** — os quatro chips do verbo
> por-forma não respondem ao clique (§6). O Enio viu-o no smoke, decidiu **integrar mesmo assim** e
> tratá-lo a seguir. **Isto não é um descuido a corrigir na integração: é a decisão dele, e está
> aqui escrito para que a integração não a re-litigue.**

> **Leia primeiro:** `CLAUDE.md` (inteiro) + `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch** | `line/Vector` (worktree `Worktrees/line-Vector/`) |
| **HEAD** | `ee45a7e5c` |
| **Base do fork** | `ee1432203` (merge-base com `main`) |
| **Commits à frente do `main`** | **19** |
| **Contratos congelados encostados** | **NENHUM** (§5) |
| **Gate de fecho** | ✅ `16.475/16.475` verdes · clippy `--workspace --all-targets` limpo · 14 índices de doc em dia |

### Os 3 commits desta sessão (do mais novo)

| commit | o quê |
|---|---|
| `ee45a7e5c` | **UM VERBO POR FORMA** na booleana viva — a receita mora na hierarquia |
| `f75b35831` | a **tinta do GRUPO** passa a ser a porta dos operandos absorvidos (o pick) |
| `098f3c96a` | **RETIRA o grafo** da booleana viva (veredito de produto) |

Os 16 anteriores vêm de sessões anteriores desta linha e já estavam fechados.

---

## §2 — ⚠️ NÚMEROS QUE SOMAM ENTRE LINHAS (leia antes do primeiro merge)

Esta é a parte da integração que passa **muda** quando corre mal (o git não sabe o que um número
significa). **Conte, nunca escolha um dos lados.**

| número | valor nesta linha | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **86 → 87** | [`project_schema.rs`](../../shells/desktop/src/project_schema.rs) + a escada ao lado + a **tripla** em [`project_schema_tests.rs`](../../shells/desktop/src/project_schema_tests.rs) — **três sítios** |
| registo de componentes (`ph2d-ecs`) | **58 → 59** | [`scene/registry.rs`](../../crates/ph2d-ecs/src/scene/registry.rs) |
| o mesmo registo, visto da `ph2d-render` | **59 → 60** | [`ph2d-render/src/registry.rs`](../../crates/ph2d-render/src/registry.rs) |
| o mesmo registo, visto da `ph2d-script` | **59 → 60** | [`ph2d-script/src/registry.rs`](../../crates/ph2d-script/src/registry.rs) |

> ✅ **INTEGRADO EM 2026-08-22 — E RECONTADO.** A `line/Sprite` entrou antes (v85/v86, +6 componentes),
> então os números acima eram os da linha contra o `main` do fork, não os do `main` combinado. O que
> ficou no `main`: `PROJECT_SCHEMA` **89** (os três degraus desta linha viraram **v87/v88/v89**; os
> dois do grafo revertido passaram por v89/v90 e saíram na mesma rebase) · `ph2d-ecs` **65** ·
> `ph2d-render` **66** · `ph2d-script` **66**. Cada degrau da escada diz de que número nasceu.
> ⚠️ **E houve uma colisão de MESMO SÍMBOLO que este handoff não previa** (§4, item 7): a
> `line/motion-value` também tinha curado a emenda do tracejado.

⚠️ **Os três contadores de registo são grandezas DIFERENTES** (`ecs`, `ecs+1 Sprite`, `ecs+1 Luau`).
Copiar o número do `ph2d-ecs` para os irmãos é **exactamente** o erro que os deixou vermelhos por um
commit inteiro nesta linha (`4fd3aad7d`, 21/08 — quem somou no ECS não somou nos dois irmãos, e
**nenhuma suite daquela tarefa os alcançava**). Se outra linha também registou um componente, o
valor certo é a **soma dos dois**, e não o de nenhum dos lados.

**Componente novo:** `ph2d_ecs::VecBoolOp` ([`vec_bool_op.rs`](../../crates/ph2d-ecs/src/vec_bool_op.rs)).

---

## §3 — O que a linha entregou

### 3.1 A tinta do GRUPO é a porta dos operandos absorvidos (`f75b35831`)

Report do Enio: *"depois de configurar só é possível selecionar e mover no canvas uma shape."*
Um operando consumido recebe lista **vazia** no mapa, e a lei do pick é *nada desenhado, nada pego*
— logo só a base era alcançável.

**Lei:** *onde a booleana desenha, cada operando absorvido é alcançável; onde ela não desenha, nada
é pego.* A lei do pick **não foi furada** — o que faltava era distinguir **absorção** de
**aniquilação**, e no mapa as duas são o mesmo `Some(vec![])`. Mecanismo e recusas:
[`26_*.md` §3](../26_plano_grafo_booleano_vivo.md).

### 3.2 Um verbo por forma (`ee45a7e5c`)

Desenho do Enio. As formas de um grupo booleano combinam-se **na ordem da hierarquia**, e cada uma
traz o verbo com que dobra sobre o resultado das anteriores. Padrão-ouro: o **compound shape vivo
do Illustrator**. Lei, decisões, discordância e recusas medidas:
[`27_um_verbo_por_forma.md`](../27_um_verbo_por_forma.md).

⚠️ **Sem nenhum override o mundo é byte-idêntico** — ausência do componente é **herança** do `op` do
grupo, e é isso que faz todo arquivo ≤ v86 desenhar igual.

---

## §4 — Riscos de INTEGRAÇÃO (o que eu declaro)

1. **`shells/desktop/src/render_loop/mod.rs`** — toquei em **cinco** sítios espalhados (a variável
   de pendência ~2611, o braço do clique ~2963, a chamada do selo ~2430, a publicação do `absorbed`
   ~7530 e o bloco de honra ~7789). É o arquivo mais disputado do repo: **espere conflito textual
   aqui** e resolva pelos ESTÁGIOS do índice, nunca pelos marcadores.
2. ⚠️ **`shells/desktop/tests/the_boolean_cooks_before_the_alignment.rs`** — o arch-gate da ORDEM
   dos produtores mudou de âncora (`"self.bool_live"` → a chamada do cozimento). **Se outra linha
   tocou o mesmo gate, funda com cuidado**: a âncora velha volta a ser ambígua no instante em que
   alguém lê `self.bool_live` mais cedo no frame — e um dos dois gates ficava **VERDE POR
   ACIDENTE** com ela.
3. **`crates/ph2d-editor-core/tests/architecture_panel_loc_cap.rs`** — a permissão de
   `paint_hierarchy_row` **desceu** 291 → 281 (a tabela de tom saiu para `badge_tone`). ⛔ Se um
   merge a fizer subir, é regressão: *"as permissões encolhem, nunca crescem"*.
4. **`crates/ph2d-panel-hierarchy/src/row.rs`** — o campo `badge` da linha, que era sempre `None`
   para formas vetoriais, passou a carregar o papel booleano. ⚠️ Ele é stampado **depois** do
   `primary_label` de propósito: o cabeçalho usa o badge como *tipo* da seleção, e stampar antes
   faria a barra de cima dizer `SUB` onde sempre disse `ENT`.
5. **Um arquivo RENOMEADO:** `measure_live_boolean_graph.rs` → `measure_live_boolean_chain.rs`.
6. ⛔ **Não integre `PROJECT_SCHEMA` escolhendo um lado** — §2.
7. ⚠️ **ACHADO NA INTEGRAÇÃO (2026-08-22) — a MESMA lei escrita duas vezes.** O commit
   `84d3b778d` (o tracejado que ENCAIXA no caminho) colidiu com a `line/motion-value`, que tinha
   curado o **mesmo** report do Enio no mesmo dia: `ph2d-vec-scene` ganhou `dash_fit::{fit,
   longest_contour, dash_lengths_for}` de um lado e `StrokeSpec::dash_lengths_fitted` +
   `stroke_plan::dash_for` do outro — **fórmula idêntica** (fechado `n = round(L/p)`; aberto `n`
   traços e `n−1` vãos), e as duas mudaram `kurbo_stroke` para a mesma assinatura. Ficou **UMA
   lei** (`dash_fit`) com **duas portas que só diferem em quem cozeu**: `dash_lengths_for` mede um
   caminho já cozido (o cache `tess.dash` do renderer, que a motion-value construiu) e `dash_for`
   coze e delega (a peça que chega da fonte — o Outline Stroke e a linha `Owned` do renderer).
   `dash_lengths_fitted` **saiu**. Prova: as **duas suítes** (`dash_fit_tests` + `stroke_plan_tests`,
   18 gates) verdes contra a lei única. ⛔ É o caso «mesmo símbolo» da DIRETRIZ §1.5.5 — a
   integração decidiu pelo critério mecânico (a sobrevivente passa os testes das duas) porque as
   leis eram iguais; **se o Enio preferir a outra porta como canônica, é troca de nome, não de lei.**

---

## §5 — Contratos congelados

**Nenhum encostado.** O `VectorOp`/`Vertex`/`Segment`/… do `ph2d-vector-doc` + `-traits`
(`CLAUDE.md` §6) não foi tocado; o motor novo `ph2d-vec-*` continua **não congelado**, e
`apply_chain_checked` é adição pura à superfície dele.

---

## §6 — ⚠️ O DEFEITO ABERTO: os quatro chips não respondem ao clique

**Report (Enio, 2026-08-22):** *"Os botões não funcionaram."*

**Repro:** cena `PH2D_BUILD_SMOKE=48` → ferramenta Vector → `Live = On` → selecionar o TRIO →
`Union` → clicar UMA forma → a row **This Shape** aparece → clicar num dos quatro modos → **nada
muda na tela**.

### 6.1 O que eu JÁ EXCLUÍ, com evidência

| suspeito | veredito |
|---|---|
| os ids não atravessam o barramento | ❌ excluído — estão em `forwards_plain_click` ([`event_clicks.rs`](../../crates/ph2d-panel-vector/src/event_clicks.rs)), o mesmo predicado do `Live Off/On`, que funciona |
| um braço `else if` anterior engole o id | ❌ excluído — o único candidato a catch-all (`fx_bridge_dispatch::classify_click`) é casamento EXACTO |
| o cozimento não vê o override | ❌ excluído — 8 gates + 5 mutantes mortos provam a cadeia e o memo |
| a triagem recusa a forma | ❌ excluído — 8 gates provam quem recebe o seletor |
| o motor ignora o verbo | ❌ excluído — 5 gates no motor |

### 6.2 A CAUSA-RAIZ do defeito ter shipado (esta é a lição, não o bug)

⚠️ **Eu gatei o modelo, o cozimento e a triagem — e NÃO gatei a costura do clique.** Não existe um
único gate no caminho `id do chip → pendência → componente escrito`. É literalmente a primeira das
quatro causas da `DIRETIVA_IMPLEMENTACAO` (**costura não-testada**), e o repo já a pagou aqui: o
comentário do `forwards_plain_click` conta que a simetria falhou o primeiro smoke exactamente
assim.

### 6.3 O que o próximo agente faz PRIMEIRO

1. **Gate red-first da costura**, antes de qualquer conserto: prove `id → pendência → `VecBoolOp`
   gravado`. Se ele nascer VERDE, o defeito é de outra natureza (ver 4) e o gate fica na mesma.
2. Suspeitos que restam, por ordem: **(a)** o bloco de honra (~7789) está dentro de uma guarda que
   pode não correr no estado em que o Enio clicou; **(b)** o `sel` ali é
   `self.vec_pen.selected_paths()` — se a forma foi selecionada por uma rota que não o popula, o
   escritor **reconfere e recusa em silêncio**; **(c)** o efeito só aparece no quadro SEGUINTE (o
   cozimento corre antes da escrita), o que num clique isolado é invisível mas num teste manual
   pode ler-se como "não fez nada"; **(d)** ambiguidade do report — **confirme com o Enio se
   "os botões" são os quatro chips novos ou os oito de sempre.**
3. ⚠️ **Não "conserte" às cegas.** O wiring está correcto em todos os pontos que a inspeção
   estática alcança; um remendo sem gate vermelho primeiro tem grande chance de mudar a coisa
   errada.

---

## §7 — Três harnesses de mutação MENTIRAM (leve isto para a próxima linha)

Vale mais que qualquer gate desta sessão, porque não é sobre booleanas:

| o que mentiu | por quê | o sinal |
|---|---|---|
| restaurar com `shutil.copy2` | repõe o **mtime original** ⇒ o cargo salta a reconstrução e a mutação **sobrevive** nas corridas seguintes | 7 mutantes com sangramento **IDÊNTICO** |
| filtro `bool_live_tests` | o módulo chama-se `bool_live::tests` ⇒ **zero** gates correram | 4 "sobreviventes" de uma vez |
| `finally` sozinho | **não corre em SIGTERM**, e um timeout mata por SIGTERM ⇒ a árvore ficou mutada | apanhado por `grep`, por sorte |

**As três curas são obrigatórias:** restaurar por `write_text` · um **controlo positivo** que exige
um mínimo de testes de facto executados · um **handler de sinal** que restaura.
*Um harness que não prova que rodou gates não mede mutação — mede a própria linha de comando.*

---

## §8 — A fila desta linha

1. **O defeito do §6** (o Enio quer isto a seguir).
2. ⏸️ Sincronizar **hover** entre a linha da hierarquia e a forma no canvas (eu recomendei, não
   está feito) — com o verbo por forma, apontar a linha certa passou a importar.
3. ⚠️ [`bool_live_tests.rs`](../../shells/desktop/src/bool_live_tests.rs) está em **586 LOC** de um
   teto de 600. **O próximo gate ali tem de orçar o split** (por assunto: a cadeia é irmã distinta
   da booleana básica).
4. ⛔ **O índice desta pasta é escrito à MÃO e estava 18 entradas atrasado** — parou no dia da
   própria arrumação (10/08), e 18 handoffs ficaram inalcançáveis a partir dele. Repus as linhas
   **derivando-as dos arquivos**, mas a cura durável é pôr `docs/*/handoffs/` sob o
   `scripts/doc-index.sh` (hoje cobre 14 diretórios e não estes). *Índice de diretório se GERA,
   não se escreve* — e esta tabela é a prova viva.
5. Os abertos anteriores da linha continuam onde estavam (`CLAUDE.md` §5, entrada **Vector**).

---

## §9 — ⚠️ MÁQUINA, não código: o `btrfs` ficou sem chunks livres

Mordeu **duas vezes** nesta sessão (um `ENOSPC` a meio de uma escrita, e antes disso suítes a
compilar pela metade **sem ficarem vermelhas**).

- **Não é disco cheio:** `df` reporta ~519 GB livres.
- **É `Device unallocated: 1,05 MiB`** — todos os chunks alocados, e **558 GiB de chunks de DADOS
  alocados e vazios** que o kernel não devolve sozinho. Os metadados não têm de onde crescer.
- **Cura, e precisa de root (decisão do Enio):** `sudo btrfs balance start -dusage=50 /home`.
- **Paliativo sem root:** `rm -rf target/*/incremental` **na própria worktree** (⛔ nunca na de
  outra linha — havia outras a compilar).
