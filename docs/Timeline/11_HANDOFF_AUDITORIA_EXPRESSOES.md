# HANDOFF — AUDITORIA TOTAL do editor de Expressões

**Status:** FECHADO 2026-07-29 · no `main` em `2e4777da9` (o commit que trouxe este arquivo).

> ⚠️ **HISTÓRICO a partir de 2026-07-30** — a AUTORIA de expressões (o card + o catálogo de
> receitas) foi **retirada** por ordem do Enio; o MOTOR ficou. O que este doc mede sobre o
> catálogo segue válido, mas o código que ele descreve não existe mais no `main`. Registro
> completo: [`14_a_autoria_de_expressoes_foi_retirada.md`](14_a_autoria_de_expressoes_foi_retirada.md).

> **Para um agente NOVO.** A feature foi **reprovada pelo Enio** em 2026-07-29, depois de
> quatro rodadas de smoke. Este doc existe para você **auditar tudo nos mínimos detalhes**
> antes de qualquer reescrita — e para você **não acreditar em mim**. O plano de reescrita
> é o irmão deste: [`12_plano_reescrita_expressoes.md`](12_plano_reescrita_expressoes.md).
>
> Autor: o agente que implementou a feature e a levou a ser reprovada. Leia a §9 (as minhas
> falhas de método) antes da §3 — ela explica por que várias afirmações minhas nos
> doc-comments do código **estão erradas**, e por que a instrução central deste handoff é
> *meça você mesmo*.

---

## §0 — FASE 0, antes de abrir qualquer arquivo

Siga [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
ao pé da letra. Resumo executável:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && pwd && git branch --show-current
```

* `pwd` **tem de** terminar em `/Worktrees/line-anim`
* a branch **tem de** ser `line/anim`

⚠️ **Todo comando de Bash começa com esse `cd`.** A cwd volta para a árvore primária
(`main`) entre chamadas, os mesmos paths relativos existem nas duas árvores, e editar a
errada **compila e commita sem um único erro**. Isto já aconteceu duas vezes nesta linha
esta semana — uma delas mandou cinco de oito edições para o `main` e eu quase reportei
"sem ganho" como achado.

Depois: `git log --oneline -8 && git status -sb`. A linha está **LIMPA** (4 commits novos,
árvore limpa, release verde em 1m29s). Os quatro commits desta jornada:

| sha | o que |
|---|---|
| `ac5cb95a1` | `Combine` por linha (Add/Multiply/Replace) — 29 de 55 receitas descartavam o valor acima |
| `c3426bbae` | `Follow` passa a ler um objeto que o artista só COLOCOU (o link lia a lista de bindings) |
| `8b99071d9` | 5 duplicatas EXATAS cortadas + `Jitter` virou por-objeto |
| `66e060123` | o card SEGUE a seleção da cena |

**Nada disto foi smokado pelo Enio.** O smoke que ele fez (os dois screenshots da §2) é
sobre o build **anterior** a estes commits mais o `Combine` (o formulário no screenshot
mostra os chips `+`, então ele rodou com pelo menos o 1º commit). **Confirme você em que
sha ele rodou antes de atribuir qualquer sintoma.**

---

## §1 — O que a feature é, e onde ela mora

Uma **expressão de propriedade** é uma fórmula em VEX-lite (`value + wiggle(2, 0.3)`) que
dirige um canal da timeline. Duas metades independentes:

**(A) O MOTOR** — ADR-0144 (o passe global) + ADR-0151 (por-clip) + ADR-0152 (prop-links).

| arquivo | papel |
|---|---|
| `crates/ph2d-expr/` | ⛔ **CONGELADO** (ADR-0039). O IR + o avaliador. **Não toque.** Só tem `Lt`/`Gt` — **não existe `<=`** |
| `crates/ph2d-expr-parse/` | o parser VEX-lite (crate NOVA, não congelada) — `wiggle` é açúcar que se desenrola AQUI |
| `crates/ph2d-timeline/src/frame_solve.rs` | o `LinkFrame` (nome→entidade, (entidade,prop)→valor), `eval_expr`, `resolve_link`, `build_names`, `seed_links`, `seed_unbound_links`, `topo_order` |
| `crates/ph2d-timeline/src/expr_pass.rs` | o passe pós-composição do driver **GLOBAL** (`binding.expr`) |
| `crates/ph2d-timeline/src/stack_eval.rs` | onde a expressão **POR-CLIP** é avaliada (dois sample sites) |
| `crates/ph2d-timeline/src/expr_live.rs` | o canal de PREVIEW ao vivo (thread_local, não serializado) |
| `crates/ph2d-timeline/src/apply.rs` · `apply_views.rs` | os 3 sítios que montam o `LinkFrame` e chamam o passe |

**(B) A AUTORIA** — plano [`10_plano_editor_de_expressoes.md`](10_plano_editor_de_expressoes.md).

| arquivo | papel |
|---|---|
| `crates/ph2d-expr-recipes/` | o CATÁLOGO (50 receitas hoje). Emite STRINGS; não parseia, não avalia |
| `crates/ph2d-expr-recipes/src/{recipe,stack,emit,knob,catalog,search,refusal}.rs` | modelo · fold · formatação · knobs · a lista · busca · as recusas |
| `crates/ph2d-expr-recipes/src/catalog/{life,wave,link,shape,time,logic,field,physics,raw}.rs` | as 9 famílias |
| `crates/ph2d-panel-timeline/src/expr_modal*.rs` | o CARD (modal arrastável): `expr_modal.rs` (estado+eventos), `_paint.rs` (layout), `_columns.rs` (galeria+planilha), `_preview.rs` (a fita) |
| `crates/ph2d-editor-core/src/ids/chrome/expr_modal.rs` | os ids (derivados por nome em runtime) |
| `shells/desktop/src/expr_smoke.rs` | a cena `PH2D_EXPR_SMOKE=1` |

**Gates existentes** (rode TODOS antes de mexer, e note quantos passam — é a sua baseline):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim
cargo test -p ph2d-expr-recipes -p ph2d-panel-timeline -p ph2d-timeline --no-fail-fast
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap --test architecture_panel_loc_cap --test no_magic_numeric --test arch_safe_clamp_only --test node_id_collisions
cargo test -p ph2d-host-desktop --test file_loc_caps
```

---

## §2 — Os reports do Enio, verbatim

**Rodada 1** (2026-07-29): *"De modo geral funcionando. Algumas expressões como Shake ao
mudar os parâmetros não mudava a animação / No lugar de sliders, melhor apenas caixas de
input numérico / Os valores padrão devem ser mais coerentes com o que se espera para uma
canvas de até 4k com 100px por metro. Alguns valores são tão altos que o objeto some do
canvas / Vamos nos desfazer do preview do objeto (sphere) no painel e vamos colocar o
preview fora do painel … Para o objeto do preview tente desenhar um fantasminha"*

**Rodada 2:** *"vamos tirar o fantasma. Vamos fazer o efeito correr no objeto selecionado
em tempo real mesmo que o clip esteja pausado, desde que o painel esteja aberto. A
velocidade em shake nunca foi velocidade, parece mais com um seed"*

**Rodada 3:** *"Funciona parcialmente! Vc corrigiu Shake e ficou bom. Muitos não estão
bons! Alguns não funcionam em nada, outros não produzem a curva do grafo de preview.
Vários problemas. Levante vários agentes para auditorar. Outra coisa: Neste app usamos o
olhinho para esconder algo. Por que usou um O?"*

**Rodada 4:** *"Jitter não funciona. Flicker não funciona! Dentro de Life muitas expressões
não passam de mais do mesmo. Exemplo: Sway e Breathe. Blink não funciona. Ramp loop pode
ser feito com Pulse. Expressões não podem ser somadas, multiplicadas, etc. Fallow e outros
da categoria ruins, não seguem o objeto referido. Se eu seleciono outro objeto na cena, o
painel de expressões não atualiza para o novo objeto."*

**Rodada 5 — a REPROVAÇÃO (com 2 screenshots):**

> *"veja o gráfico plano de flick / **Quase tudo em Time não funciona** / **Não vejo o
> menor sentido para artistas na seção logic** / **Mesmo deletando as expressões, elas
> ficam atuando** / **Layout absurdo, tudo apertado** / **Não tem scroll nem barra de
> scroll**"*

Os screenshots mostram, e cada um é um fato auditável:

1. **Fita do Flicker perfeitamente PLANA** (linha tracejada reta).
2. O card com a galeria numa coluna estreita (`< All` / Shake / Turbulence / Drift / Jitter
   / Breathe / Flicker), a planilha com 3 linhas e **`+1 more rows`** em vez de rolagem.
3. **`Turbulence · Detail = 0`** na caixa numérica — enquanto a fórmula na barra diz
   `wiggle(2, 4, 1, 0.5)`, ou seja **1**. A caixa e a fórmula discordam.
4. Título: `Expression — Translate X #7294` — **nenhum nome de objeto**.

---

## §3 — O que EU medi (e o que NÃO confirmei)

⚠️ **Trate cada linha como uma hipótese a reproduzir**, não como fato. Onde eu escrevi um
número, o comando que o produziu está no commit correspondente.

### 3.1 — CONFIRMADO por medição (repro no repo)

| # | fato | número |
|---|---|---|
| A | 29 de 55 receitas ignoravam `EmitCtx::inner` ⇒ empilhar descartava o de cima | Sway+Blink ⇒ `select(...)` só |
| B | `Jitter` era CONSTANTE | `value + noise(7)*0.2` = +0,197, excursão **0,0000** |
| C | `Flicker` é MULTIPLICATIVO ⇒ em `value = 0` dá **exatamente 0** | excursão 0,0000 em v=0; 0,3456 em v=1 |
| D | `Blink` ANIMA, mas SUBSTITUI o valor | excursão 1,0 (0→1) |
| E | 5 receitas eram identidade EXATA de outra com um knob mexido | pior delta 0,000000 |
| F | prop-link só lia propriedade com BINDING | fonte só-colocada **0,0000** (agora 7,0000) |
| G | `translation_x` (o `i18n_suffix`!) não era aceito ⇒ 0.0 silencioso | — |
| H | o `expr_pass` construía um **2º mapa de nomes** | derrotou o fix de (F) |
| I | o card nunca revisitava o alvo; o snapshot não tinha seleção | — |
| J | Sway ≠ Breathe (o Enio disse "mais do mesmo"; eles DIFEREM) | delta 0,075 |

### 3.2 — CONFIRMADO por leitura de código, **NÃO reproduzido** — comece por aqui

**⚠️ ASSIMETRIA DE ESCRITA DA EXPRESSÃO — o candidato nº 1 para *"mesmo deletando, ficam
atuando"*.**

* `snapshot.rs:588` — `row.expr = <per-clip do clip ativo>` **`.or_else(|| b.expr.clone())`**
  ou seja o card **LÊ** o per-clip **OU o GLOBAL**.
* `intent_apply.rs:316` — `SetBindingExpr` faz **só** `doc.set_clip_expr(active, target, expr)`
  ou seja o Apply **ESCREVE** só o per-clip.

⇒ **Um leitor, dois escritores, e o Apply só alcança um deles.** Se um `binding.expr`
GLOBAL existir, o card o mostra como uma linha `Custom Formula`, você apaga a linha, aperta
Apply, o per-clip (vazio) é removido — e o GLOBAL continua rodando no `expr_pass`, para
sempre, sem nenhuma UI que o alcance.

**Contra-evidência que você TEM de resolver:** varri o repo e **nenhum caminho de produto
escreve `binding.expr`** — só `morph_fade_smoke.rs:157` e testes. Então em uso normal essa
rota talvez não dispare. **Duas coisas a fazer, nesta ordem:** (1) confirme por grep se
alguma rota de produto (import de projeto, load, um intent antigo) escreve o global; (2)
independente disso, **a assimetria é um defeito por si** e tem de morrer.

**⚠️ CANDIDATO Nº 2 — a propriedade SEM KEYS congela onde a expressão a deixou.**

`solo_source_value` devolve `None` para track vazia (esparsidade deliberada: um binding
recém-criado não força valor default) ⇒ **nada escreve a propriedade**. Uma prop dirigida
por expressão PURA (sem keys) — que é exatamente o caso do objeto "Slider" do
`expr_smoke.rs:83` — **não volta a nada** quando você apaga a fórmula: ela fica no último
valor que a expressão escreveu. Da poltrona do artista isso É *"continua atuando"*.

O `expr_pass::take_restore` foi construído para devolver a pose, mas **só** para o
encerramento do PREVIEW, não para um Apply que limpa.

**⚠️ CANDIDATO Nº 3 — o canal de preview ao vivo.** `expr_live` é `thread_local` com um
`RESTORE` pendente. Vasculhe todas as rotas de morte do card (Apply · Cancel · X · painel
oculto · timeline fechada · retarget · undo) e prove que **cada uma** limpa. Eu adicionei
limpeza no `retarget` nesta jornada; não auditei as outras.

### 3.3 — NÃO investigado, reportado pelo Enio

**⚠️ "Quase tudo em Time não funciona".** O mecanismo que eu ACREDITO (não medi) é
**estrutural, não um bug de código**:

* uma linha `Time` reescreve o relógio **das linhas ABAIXO dela**. Sozinha, ou por último,
  ela **não faz nada** — e a galeria a oferece exatamente como oferece as outras;
* `Shake` e `Turbulence` declaram `ClockUse::Own` porque **`wiggle` constrói `time + __seed`
  DENTRO do parser** ⇒ nenhuma linha Time alcança as duas receitas mais óbvias de se
  combinar com tempo;
* logo: 7 receitas cuja utilidade depende de uma ordem que nada na tela ensina.

**Meça antes de projetar:** para cada uma das 7 (`stepped-time`, `delay`, `speed`,
`reverse-time`, `freeze-after`, `start-at`, `ping-pong-time`), monte `[Time, Sway]` e
`[Sway, Time]` e `[Time]` sozinha, e reporte a excursão. Eu **não** fiz isso.

**⚠️ "Logic sem sentido para artistas".** 7 receitas de condicional com limiar numérico
(`if-greater`, `if-less`, `if-near`, `switch`, `gate-and`, `gate-or`, `after-time`). É
programação, não animação. **Não defenda; a §4 do plano irmão propõe o corte.**

**⚠️ `Detail = 0` na caixa vs `1` na fórmula.** `Knob::lit("detail", …, 3.0, (1.0, 4.0))`.
`EmitCtx::lit` **clampa em silêncio** para emitir texto que o parser aceite (`wiggle`
recusa octaves < 1). Então a caixa mostra 0, a fórmula usa 1, **e o artista não tem como
saber qual vale**. Duas perguntas para a auditoria: (a) por que a caixa deixou passar de 1
para 0 — o clamp do widget usa o range do knob? (b) um clamp silencioso na emissão é
aceitável? (minha opinião: **não** — o widget tem de recusar, e aí a emissão não precisa
clampar).

**⚠️ A FITA PLANA do Flicker.** Eu sei o mecanismo aritmético (C: multiplicativo em v=0),
e **não sei** se é o único. Confira também: `expr_modal_preview.rs::extent` normaliza o
eixo vertical por min/max e tem um guard para `hi == lo`; e o `base` (o valor de repouso da
propriedade) é 0 para translação e 1 para escala. **Meça a fita de TODAS as 50 receitas** e
liste as que saem planas — foi o que o Enio pediu na rodada 3 e eu cobri por censo, não por
fita.

---

## §4 — O que eu mudei nesta jornada, e o que NÃO confio nisso

### 4.1 — `Combine` por linha (`ac5cb95a1`)
`Recipe.combine: Option<Combine>` — `Some` = SOURCE (emite só a contribuição, o fold
combina), `None` = MODIFIER (transforma o `inner`). Chip `+`/`×`/`=` na linha.
**Confio**: o catálogo é value-idêntico no default, provado contra tabela congelada
extraída compilando a crate do commit anterior (`tests/shared/pre_combine_table.rs`).
**Não confio**: nenhum smoke humano viu o chip. Ele é um botão de 22 px ao lado do olhinho,
num layout que o Enio chamou de apertado.

### 4.2 — `Follow` (`c3426bbae`)
`build_names` cobre a cena inteira; `seed_unbound_links` lê o mundo para fontes sem
binding; o `expr_pass` passou a usar UM mapa. `translation_x` aceito.
**Não confio**: nenhum smoke. E **não existe pick-whip** — o artista DIGITA `Ball.x` num
campo de texto, sem autocomplete, sem lista, e **um nome que não resolve é silenciosamente
0**. Um `Name` com ESPAÇO faz a fórmula inteira não parsear e o driver é descartado em
silêncio. Ou seja: **a família Link inteira (16→14 receitas) segue inusável na prática.**

### 4.3 — Catálogo (`8b99071d9`)
5 cortes com identidade EXATA medida; sobreviventes herdaram as buscas; `Jitter` virou
per-objeto via `__seed`. 55→50.
**Não confio**: cortei só o que é identidade EXATA. O Enio disse *"mais do mesmo"*, que é
uma queixa de **redundância percebida**, e ela é maior que identidade matemática (ver §3
do plano irmão).

### 4.4 — Seleção (`66e060123`)
`TimelineViewSnapshot.selected_entity` + `retarget`.
**Não confio**: o card só segue para um objeto que **tem binding** na mesma prop; senão
fica onde está e **nada na tela diz por quê** (o título mostra `#7294`, não um nome). Um
artista clica no objeto B, o card continua sobre A, e ele conclui — corretamente, do ponto
de vista dele — que não funciona.

---

## §5 — A auditoria que eu quero de você (checklist, com oráculo)

Ordem deliberada: **reproduzir → medir → só então ler meu código.**

### Bloco 1 — os sintomas do Enio, reproduzidos
1. **Rode o smoke** (`env PH2D_EXPR_SMOKE=1 cargo run -p ph2d-host-desktop --release`) e
   reproduza CADA um dos 6 itens da rodada 5. Para cada um: reproduz? em que gesto exato?
   Oráculo = a TELA, e um `eprintln` de diagnóstico se precisar.
2. **"Deletando, ficam atuando"**: isole qual dos 3 candidatos da §3.2 é. Meça o valor da
   propriedade antes/durante/depois do Apply-vazio, em prop COM keys e prop SEM keys.
3. **A fita**: censo de excursão da fita para as 50, com `base` de translação E de escala.
   Liste as planas e o mecanismo de cada.

### Bloco 2 — o motor, uma pergunta por vez
4. Enumere **todos os escritores e leitores** de expressão (global, per-clip, preview) numa
   tabela. Quantas portas escrevem? Quantas leem? Elas concordam?
5. `ClockUse` — meça as 7 de Time (§3.3). Quais são inertes e em que arranjo?
6. `wiggle` no `ph2d-expr-parse`: o que exatamente ele desenrola? Quantos `noise` por
   octave? O `Speed` é frequência ou seed? (O Enio reportou *"velocidade nunca foi
   velocidade, parece um seed"* e eu "corrigi" — **verifique o que eu fiz**.)
7. Determinismo: `__seed` vem de `target.get() * SEED_SPACING`. **Isso é estável entre
   sessões?** Adicionar/remover uma track re-rola o `Jitter` de todos os outros objetos?
   (Eu construí o `Jitter` novo em cima disso e **não** verifiquei.)

### Bloco 3 — o card, widget por widget
8. Meça o card: `card_w()`/`card_h()`, `BODY_SLOTS = 12`, `GALLERY_W = 190`,
   `SHEET_W = 320`, `ROW_BTN_W = 22`, `KNOB_LABEL_W = 84`, `KNOB_READOUT_W = 52`. Quantos
   px sobram para o nome de uma receita com o chip novo? (O screenshot sugere: quase nada.)
9. **Rolagem**: `expr_modal_paint.rs` diz, num doc-comment, *"Nothing here scrolls, and the
   geometry says why"* e escreve `+N more rows`. Isso era verdade para 9 famílias × ≤10
   receitas e **deixou de ser** com a planilha (1 + knobs por linha, 4 linhas de Turbulence
   já estouram). O app TEM primitivo de scroll (`ph2d-panel-wet-tuning` usa; scrollbar id
   837; `scrollable_panels_intercept_the_wheel` é gate). **Quanto custa usá-lo aqui?**
10. Varra o seam: para CADA widget do card, ele (a) existe, (b) é pintado E registrado,
    (c) o clique chega ao barramento, (d) a sequência leva a algum lugar. As 4 condições
    são independentes. O `MockPanelHost::new()` **pula o `populate`** — use
    `with_panel::<TimelinePanel>()` e `click_at` REAL, ou seu gate nasce verde sobre
    widget morto.
11. `Detail = 0` (§3.3): o clamp do widget e o clamp da emissão discordam. Confirme.

### Bloco 4 — o catálogo, receita por receita
12. Para cada uma das 50: excursão em v=0 e v=1 · fita plana? · lê o relógio? · precisa de
    linha acima? · precisa de link? · faz sentido para um ANIMADOR (uma frase). Isso é uma
    **tabela de 50 linhas** e é o insumo do corte do plano irmão.
13. Matriz de redundância: para cada PAR, existe ajuste de knob de A que reproduz B? Eu
    testei 8 pares à mão e achei 5. **Faça os 1225.** (É `O(n²)` sobre avaliação de
    fórmula: barato. Use busca em grade nos knobs.)

### Bloco 5 — os meus gates
14. Para cada gate que eu escrevi nesta jornada (19), pergunte: **o que ele afirma que
    poderia estar errado?** Mate os que não podem falhar pelo motivo que alegam. Eu
    documentei 3 sobreviventes de mutação; **assuma que há mais**.
15. `tests/shared/pre_combine_table.rs` é uma tabela congelada de 55 fórmulas. Confirme que
    ela foi extraída do commit ANTERIOR e não gerada pelo código sob teste.

---

## §6 — Contratos e limites que a auditoria NÃO pode violar

* ⛔ **`ph2d-expr` é CONGELADO** (ADR-0039, §6 do CLAUDE.md). Sem `<=`, sem `exp`, sem
  `atan2`. Mexer = ADR + ordem do Enio. `ph2d-expr-parse` NÃO é congelado.
* ⛔ `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` / `Tool=12` / `RasterEditTool=5` /
  `CanvasPaintTool=1` / `PanelEvent=4` — intactos hoje; confira por **grep**, não por
  auto-relato.
* `PROJECT_SCHEMA` = **37** no `main`. `DOC_VERSION` = 15. Nada nesta jornada bumpou. ⚠️ O
  número se **CONTA** a partir do `main` do dia — a `line/physics` e a `line/FLIP` já
  colidiram duas vezes nele.
* `ph2d-expr-recipes` é **leaf e dep-free** (`ph2d-expr`/`-parse` são **dev**-deps). Manter.
* HR-15: zero hex, zero literal de px de UI sem `LITERAL-PX-OK`, zero string hardcoded
  (i18n). HR-18: 700 LOC em `crates/`, 600 na shell, 600/200 em painel.
* O gate `architecture_panel_wiring_parity` só coleta `.register(ids::LITERAL` **direto** —
  registro em LAÇO é ponto cego dele (o buraco das 36 células do physics). Se você
  registrar knobs em laço, o seam é a única cobertura.

---

## §7 — Onde o histórico está escrito

* [`09_pesquisa_editor_de_expressoes.md`](09_pesquisa_editor_de_expressoes.md) — a pesquisa
  (AE/Cavalry/Motion/Blender/Rive convergiram num **catálogo com knobs**; um editor de
  texto melhor não fecha o vão).
* [`10_plano_editor_de_expressoes.md`](10_plano_editor_de_expressoes.md) — o plano que eu
  executei. **Leia com desconfiança**: ele é a fonte das decisões que o Enio reprovou (55
  receitas, 9 famílias, "nada rola"). A W4 (pick-whip) está lá e **nunca foi construída**.
* [`08_plano_expressoes_no_blend.md`](08_plano_expressoes_no_blend.md) — ADR-0152, o motor.
* ADRs: `0144` (expressões), `0145` (por-clip), `0146` (prop-links no blend), `0039`
  (congelamento do `ph2d-expr`).
* O CLAUDE.md §5, entrada **Timeline** → "AS JOIAS DA COROA" tem o parágrafo (C) sobre
  expressões. ⚠️ Ele afirma coisas que esta jornada corrigiu; **atualize-o quando fechar**.

---

## §8 — Estado dos smokes

| env | o que encena | último veredito |
|---|---|---|
| `PH2D_EXPR_SMOKE=1` | 3 objetos dirigidos por fórmula per-clip | **REPROVADO** (rodada 5) |
| `PH2D_SIGNAL_SMOKE=1` · `PH2D_EXTRAP_SMOKE=1` · `PH2D_TIMESCALE_SMOKE=1` · `PH2D_STAGGER_SMOKE=1` (Ctrl+drag) · `PH2D_BUFFER_SMOKE=1` | as outras waves das Joias | aprovados 2026-07-27 |

⚠️ A cena do `expr_smoke` **não** exercita o card: ela autora fórmulas por código. O Enio
abre o card pelo menu de contexto de uma track. **Uma cena que arma estado por baixo da
mesa pula exactamente a costura que ela deveria provar** — a cicatriz que o
`impasto_smoke` já pregava. O plano irmão troca isso.

---

## §9 — As minhas falhas de método (leia antes de confiar em qualquer prosa minha)

Estas são as que eu **peguei**. Assuma que há outras.

1. **Medi peça isolada em harness meu em vez do produto**, e reportei "funciona" três
   rodadas seguidas. O censo que eu rodei contava excursão de FÓRMULA; o Enio olha a TELA.
2. **Medi no lugar errado e concluí o oposto**: numa wave do Painter medi a modulação no
   EIXO do traço (onde tudo satura) e escrevi "invisível", quando o artefato vivia no
   ombro. Nesta feature o análogo é: `Flicker` tem excursão em v=1, e eu não perguntei qual
   `value` a propriedade real tem.
3. **Escrevi doc-comments que afirmam mais do que o código faz.** Achei três nesta jornada
   (`RecipeStack::recover` que nunca existiu, citado em 2 lugares; `ExprModal.title`
   prometendo `"Ball · Position Y"` quando o card nunca mostrou nome; `frame_solve`
   dizendo que o mapa é threaded EMPTY). **Todo doc-comment é uma afirmação a verificar.**
4. **Fixtures que não continham o fenômeno**, 3× nesta jornada: troquei de seleção com a
   planilha vazia (onde limpar e não limpar são indistinguíveis); supus o layout de
   `Entity::to_bits`; testei duplicata de nome sem separar "primeiro da varredura" de
   "menor bits".
5. **Uma mutação minha não sangrou porque a minha AFIRMAÇÃO estava errada**, não porque
   faltava gate (o guard do roteador do chip é higiene, não correctness).
6. **Tratei "o gate está verde" como "o produto funciona"** — a doença que o
   `DIRETIVA_IMPLEMENTACAO.md` chama de *"verde-de-compilação é velocidade; no audit vale
   ZERO"*, e eu a cometi com gates de unidade.
7. **Não perguntei o que o artista faz com a coisa.** Construí 55 receitas com uma
   taxonomia de implementação e só descobri que 7 delas (Logic) não têm sentido para um
   animador quando o Enio disse.
8. **A cwd do Bash escorregou para o `main` duas vezes.** Ver §0.

---

## §10 — O que fazer quando terminar a auditoria

Não conserte nada durante a auditoria. Entregue:

1. **A tabela de 50 linhas** (Bloco 4.12) e a **matriz de redundância** (4.13).
2. **A lista de defeitos**, cada um com: sintoma do Enio · mecanismo medido · arquivo:linha
   · o gate que faltava.
3. **A lista de gates meus que não provam o que alegam.**
4. **O veredito sobre o plano irmão**: [`12_plano_reescrita_expressoes.md`](12_plano_reescrita_expressoes.md)
   foi escrito por mim, **antes** da sua auditoria, a partir dos reports. Você tem os
   números; ele tem hipóteses. **Corrija-o** — e onde ele estiver errado, diga por que, com
   medição.

E **PARE**. A linha não integra nem pusha sem ordem explícita do Enio (§0.7 do CLAUDE.md).
