# HANDOFF DE CONTINUAÇÃO — `line/motion-value` · **o PLANO e as tarefas em aberto**

**Data:** 2026-08-19 · **Para:** o próximo agente desta linha · **Worktree:**
`/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`

> ⚠️ **Isto não é um handoff de integração** (aquele é o
> [`…_FECHO_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_FECHO_2026-08-18.md), já
> integrado). Isto é o que a próxima janela precisa para **continuar a implementar**: onde
> parar de reconstruir, o que fazer a seguir, em que ordem, e as leis que esta linha pagou
> para aprender.

---

## §0 — Os primeiros três comandos, antes de abrir arquivo nenhum

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && pwd && \
  git branch --show-current && git log --oneline -1
```

⚠️ **A janela abre na raiz (= `main`) e o MESMO caminho relativo existe nas duas árvores** —
editar a errada compila e commita **sem erro**. E a `cwd` do Bash **volta ao primário entre
turnos**: prefixe **todo** comando com o `cd`
([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

Depois:

```bash
bash scripts/hw-profile.sh          # tier → MODO (aqui: workstation ⇒ Modo L)
cargo check --workspace | tail -3   # a base compila?
python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"   # o placar VIVO
```

---

## §1 — Onde a linha está

| | |
|---|---|
| base | `main` `ee1432203` — a linha foi **reaberta por fast-forward** depois de a integração ter entrado |
| commits desta janela | **1** (`15fbce95c`, o espaço do campo) |
| estado | **verde** — `fmt` 0, `clippy` 0, suítes das crates tocadas 0 falhas |
| smoke pendente | **`PH2D_GPU_COOK_DEMO=60`** — nunca smokado |

⚠️ **Dois smokes de ontem também nunca foram vistos pelo Enio** e já estão no `main`:
`=58` (re-smoke depois da correção do relógio que expirava) e `=59` (a porta de tempo).
Se ele reportar algo sobre eles, o mecanismo está no handoff do FECHO.

---

## §2 — O que NÃO se reconstrói (feito e integrado)

- **A porta de tempo** em `oscillator`/`noise`/`wiggle` — porta VALUE opcional, índice **1**,
  desligada ⇒ `ctx.playhead()` bit-a-bit, ligada ⇒ **um relógio por elemento**. CPU + GPU.
- **`TimeMode::Curve`** (índice 5) no `ph2d-nodegraph`, com a janela a **REPETIR** — ele é o
  superset cíclico do `Loop`/`PingPong`.
- **`motion.drive`**: canais `Size X` (10) e `Size Y` (11).
- **`value.attribute`**: os chips `Position X` · `Position Y` · `Radius` · `Angle`.
- **`motion.noise`**: o **espaço do campo** — `rotation` + `uniform`/`scale_y`. ⚠️ O *offset*
  e o *scale uniforme* **não são params de propósito**: saem da composição e do próprio
  `scale` (medido em `measure_noise_space`).
- **A folha 06 FECHOU** — 0 P0, 0 P1, 12 ✅, 18 P2.

---

## §3 — O PLANO, em quatro grupos, nesta ordem

> A regra de cadência é do Enio: **implementar em GRUPOS de nós, e a cada grupo UMA cena de
> smoke**. A próxima cena livre é a **`=61`** — ⚠️ e esse número se **CONTA lendo o `match`**
> do [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs),
> nunca esta linha (ela envelhece no primeiro grupo).

### Grupo S — os DEFEITOS, antes de qualquer knob

Um defeito silencioso vale mais que uma feature, e há **dois** nomeados e medidos:

1. ⛔ **Um SEGUNDO `fx.glow` é silenciosamente INERTE** (folha 11) — `from_graph` faz
   `.find(…)` e o segundo nó nunca corre. O artista empilha dois glows, vê um, e conclui que
   o parâmetro não funciona.
2. ⛔ **O diagnóstico de nome do `value.attribute` não olha o MODO** — `unresolved_reads`
   recebe só os **nomes** (`columns::names_at`), então uma coluna `Vec2` digitada à mão no
   campo livre **resolve** para o diagnóstico e **lê zeros** para o cook, sem badge. Os `Vec2`
   que a tabela conhece já têm chip (é a cura do caso real); **o caso geral fica**. Preço: o
   callback e o `ph2d-motion-diagnose` teriam de carregar a **dimensão** da coluna.

⚠️ Estes dois não precisam de cena nova — o smoke deles é *"empilhe dois glows"* e *"digite
`vel` no campo livre"*. Se quiser uma cena, faça-a mostrar o **antes/depois** lado a lado.

### Grupo T — as folhas que FECHAM (3 células, 2 folhas)

Fechar uma folha é um marco que o Enio lê, e estas duas custam pouco:

| folha | P1 | o pedido |
|---|---|---|
| **17_zero_param_debug** | 1 | `motion.integrate`: **sub-steps / o timestep exposto** (Blender GN *Simulation Zone* dá **Delta Time** como input do nó) |
| **15_value** | 2 | `value.unary`: **Ceil · Round · Truncate** (Blender *Float to Integer*) · `value.switch`: **N entradas** (Blender *Index Switch*) |

⚠️ **O `value.switch` de N entradas mexe no MANIFESTO** (a lista de portas cresce) — leia a
lei das portas apendadas no §4 antes de desenhar.

### Grupo U — `source.shape`, e o item mais pedido do catálogo

A folha 14 tem **7 P1** e quatro são do `source.shape`. O **TRIM / dash** é o que a própria
folha marca como *"o item mais pedido"* (Cavalry *Trim Path*, AE *Trim Paths*). Os outros:
`fill_rule` não exposto · sweep/start/inner (pizza, rosquinha, anel parcial) · raio POR CANTO
e *corner smoothing* (squircle) · e o estrutural *"`size` é GEOMETRIA, não coluna"*.

⚠️ Este grupo é o único que provavelmente **encosta no módulo Vector** (o trim de um path
desenhado). Se encostar num arquivo fora do Motion, é caso de **parar e reportar ao Enio**
(CLAUDE.md §0.2) — não renegoceie com outra linha.

### Grupo V — as folhas grandes, por ORDEM DE DEFEITO

`08_stream_utilidade` (8) · `14_source` (o resto) · `01_distribuicao` (6) · `04_deformers`
(6) · `10_field` (6) · `11_fx_raster` (o resto) · `02_force` (5) · `05_transform` (4) ·
`03_simulacao` (3) · `07_tempo` (3) · `09_cor` (3).

⚠️ **Não ataque por tamanho.** Dentro de cada folha, o que vem primeiro é o que a célula
descreve como **comportamento errado** (o `fx.glow` inerte, o `motion.duplicator` que perde a
escala do ponto, o `motion.step` com limitação auto-declarada), e só depois o que é knob
ausente.

---

## §4 — As LEIS que esta linha pagou para aprender

⚠️ **Cada uma destas custou um gate vermelho, um smoke reprovado ou uma medição** — elas não
são estilo.

1. **TRAP 1 SEMPRE, e ele vale para a FOUNDATION também.** Dez células da folha 06
   envelheceram — a última **em metade**: o *scale uniforme* do campo do ruído já era o param
   `scale`, bit-a-bit. E na porta de tempo o orçamento listava **três saídas caras** porque o
   seletor de variante só vê params — e o canal certo (`ColumnAccess::ReadBroadcast` + o
   `const HAS_<porta>_<col>` do codegen) **já existia**. *Meça se o substrato já exprime,
   antes de orçar um mecanismo novo.*
2. **Um ✅ de MECANISMO não é um ✅ de ARTISTA.** A folha 15 marcava as lanes de uma `Vec2`
   como fechadas porque o degrau existia — e não havia gesto que chegasse lá. *Um degrau sem
   chip é inalcançável.*
3. **Uma fixture só prova o que ela CONTÉM.** A fileira de teste do `motion.noise` tem
   `y = 0` em toda peça, e um gate de `scale_y` reprovou sobre código correcto. ⚠️ E a
   rotação **mostra-se** numa fileira, que é o que esconde o buraco de quem só olha para um
   dos dois eixos.
4. **A régua tem de ser a coisa REAL.** O oráculo da cena `=60` subtraiu a *média* para tirar
   a grade; a grade varre 4,48 de mundo em Y e a razão do controle deu **0,21** em vez de ~1.
5. **A DIREÇÃO de um knob pode ser contra-intuitiva — meça-a.** Escala maior num eixo =
   feição **menor** nele (`dx/dy` cai de 0,976 para 0,341). O rótulo tem de dizer o que o
   artista vê, não o que o número sugere.
6. **Nenhum controle pode EXPIRAR.** O `TimeMode::Curve` clampava a janela e a sub-árvore
   congelava para sempre; os gates mediam **dentro** da janela e ficavam verdes sobre produto
   morto. Mesma classe do `fade` do oscilador. *Um gate que só olha para dentro da janela não
   pode ver uma janela que não repete.*
7. **Uma exceção por NÚMERO DE LINHA quebra em silêncio.** A tabela `HAND` do
   `placar_conferencia.py` era chaveada por `(arquivo, nº)`; acrescentar uma linha desalinhou
   tudo e o placar imprimiu **um ✅ a menos**. Hoje a chave é um TRECHO e cada uma tem de
   casar **exactamente uma** linha, senão a ferramenta sai vermelha.
8. ⛔ **OLHE o arquivo antes de escrever nele.** Nesta janela eu sobrescrevi a cena `=51`
   inteira ao criar um módulo com um nome que já existia (`…_demos_space.rs`). Recuperou-se
   com `git checkout --`, mas só porque a árvore estava limpa. *Um `ls` antes do `Write`
   custa nada.*
9. ⚠️ **A suíte inteira é um relógio.** Duas corridas marcaram falhas que eram **carga**
   (`the_cost_of_depth_is_linear_not_explosive` e
   `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`), com `load average` em 14,8.
   Sozinhas passam. *Nada desta workstation vale acima de `load ~5`.*
10. **A porta de tempo é uma COLUNA, não um escopo** — ela não herda a recusa
    `CookError::SequentialInTimeScope`. Se acrescentar uma porta a outro nó, o gate
    `the_time_port_is_a_column_not_a_cook_scope` é o molde.

---

## §5 — O ritual de cada célula (o que fazer, na ordem)

1. **Leia a célula inteira**, inclusive a coluna *"exprimível?"* — ela costuma trazer o
   mecanismo, e é onde as dez que envelheceram estavam erradas.
2. **Escreva uma SONDA `measure_*`** em `crates/ph2d-node-registry-init/tests/` que tenta as
   rotas de composição e **IMPRIME** (`#[ignore]`, `--nocapture`). Se ela mostrar que o
   catálogo já dá, a célula **envelheceu** — reescreva o veredito com o número e siga.
3. **Só então** escreva o param, com o default que **reduz** ao mundo de antes, e um gate que
   peça `==` sobre isso.
4. **CPU e GPU juntos**, com paridade. Se o nó tem kernel, o corpo WGSL é port linha-a-linha e
   a paridade é quem guarda a igualdade das duas cópias.
5. **Prova de mutação** — RED só conta sobre algo visto VERDE antes.
6. **Uma cena** por grupo, com **CONTROLE** dentro dela. Números que a mensagem cita vivem em
   `const` presos por um gate que lê o fonte da narração.
7. **Reconcilie a `Contagem`** da folha rodando o placar (ele **imprime e sai vermelho**;
   `--write` não existe).
8. **`CLAUDE.md §5` recebe UMA LINHA** — a narrativa vai no handoff.

---

## §6 — Comandos que esta linha usa

```bash
# inner loop
bash scripts/cargo-check-narrow.sh ph2d-node-motion-<nó>

# a suíte de uma crate (exit 0 verde · 1 teste vermelho · 2 não compilou)
bash scripts/cargo-test-narrow.sh ph2d-node-motion-<nó>

# a sonda de uma célula
CARGO_INCREMENTAL=0 cargo test -p ph2d-node-registry-init --test measure_<x> -- --ignored --nocapture

# paridade CPU×GPU (⚠️ skip gracioso NÃO é verde — confirme que o adapter apareceu)
CARGO_INCREMENTAL=0 cargo test -p ph2d-gpu-cook --test gpu_cpu_parity -- --ignored --test-threads=1 <filtro>

# o gate batched, 1× no fim do grupo
CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast

# a superfície de colisão, antes de fechar
bash scripts/collision-surface.sh main
```

---

## §7 — O smoke que está pendente

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=60 cargo run -p ph2d-host-desktop --release
```

Quatro blocos, o **mesmo** ruído; muda só o espaço. Julga-se **PARADO**.
1. controle · 2. rodado 45° · 3. comprimido no Y (listras deitadas) · 4. os dois, nessa ordem.
⚠️ Se um bloco parecer **mais agitado** que os outros, a cena perdeu o controle — o que muda
é ONDE o campo é amostrado, nunca quanto ele vale.

---

## §8 — Onde ler

- **Estado do módulo:** `CLAUDE.md §5` (roteador, não história).
- **A conferência:** [`89_conferencia/README.md`](../89_conferencia/README.md) — 17 folhas; o
  placar é **derivado**.
- **O mecanismo das waves desta linha:** [`handoffs/README.md`](README.md) — o índice
  cronológico (⚠️ ele estava **oito** atrás em 18/08; se acrescentar um handoff, reconcilie a
  contagem lendo a pasta).
- **Processo:** DIRETRIZ §1.5 (Modo L) · §1.5.9 (fechar a linha).
