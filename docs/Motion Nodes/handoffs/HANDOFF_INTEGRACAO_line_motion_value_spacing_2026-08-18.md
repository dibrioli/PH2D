# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, a wave do ESPAÇAMENTO (2026-08-18)

> **Este documento supersede** o `HANDOFF_INTEGRACAO_line_motion_value_wave_2026-08-18.md`
> apenas como *o que integrar agora* — ⚠️ **o mecanismo da wave do `motion.wave` continua LÁ**,
> e a cena `=57` dele foi **aprovada pelo Enio em 2026-08-18**.

---

## 1. O que esta wave é

A folha [06_animadores](../89_conferencia/06_animadores.md) **linha 46** marcava `P1` no
`motion.path` e chamava-o *"o caso mais forte da conferência: o mesmo app responde a mesma
pergunta com 6 controles num módulo e 3 no outro"* — o `pattern_along_path` do módulo Vector shipa
**Spacing · Start/End · Slide · Offset perpendicular · Side** desde 2026-07-23.

⚠️ **A célula é de 2026-08-10 e ENVELHECEU — a OITAVA desta conferência.** Medido antes de uma
linha ser escrita, **quatro dos cinco controles já são exprimíveis**, e a razão escrita na célula é
o que a derruba.

| Controle | Veredito MEDIDO |
|---|---|
| **Slide** | ⚠️ **JÁ EXISTE**: é o param `offset` deste nó, e a célula o listava como ausente |
| **Start/End** | COMPOSIÇÃO com o irmão `motion.spline_wrap` (`from`/`to`) |
| **Offset perpendicular** | COMPOSIÇÃO (`height_scale` sobre um `motion.move`) |
| **Side** | COMPOSIÇÃO (o SINAL do mesmo deslocamento) |
| **Spacing** | ⚠️ **vão REAL** — e é o que esta wave constrói |

⚠️ **A razão escrita na célula era falsa:** *"o deslocamento perpendicular precisaria da NORMAL,
que nada publica (`motion.move` é mundo)"* — o **`motion.spline_wrap` computa a normal**
(`un = [-ut.y, ut.x]`) e a expõe como `height_scale`, lendo a **MESMA curva desenhada** pelo mesmo
canal (`external::curve_of`) com a mesma `ph2d_arc_length::at` por baixo. O doc do próprio
`motion.path` já chamava o irmão de *"o segundo consumidor da mesma curva desenhada"*; ninguém
tinha perguntado a ele.

⇒ **`P1` → `P2`** para quatro: não falta capacidade, falta o **GESTO** (três nós e duas arestas à
mão). Placar da folha 06, **DERIVADO** pela ferramenta: **P1 8 → 7 · P2 16 → 17**.

---

## 2. O que foi construído

### 2.1 O `motion.path` conta por ESPAÇAMENTO

Dois params apendados ao `MANIFEST`: **`mode`** (`Number | Spacing`) e **`spacing`**.

⚠️ **Um MODO e não uma sentinela**, e a razão é o `ParamGate`: ele decide por **INTEIRO**, então
`spacing ∈ (0, 0.5)` pintaria os **dois** controles com só um no comando.

⚠️ **O espaçamento é uma distância de arco ABSOLUTA**, não a fração-da-largura-do-motivo que o
módulo Vector usa: este nó emite instâncias e **não conhece tamanho nenhum**.

⚠️ **A metade cara já estava paga:** `lut.last()` **É** o comprimento, e o `eval` já o calcula para
amostrar — a wave inteira é uma divisão sobre um número que já estava na mão. Nenhum canal novo,
nenhuma segunda travessia.

**A lei é o FLOOR** (`copies_that_fit`): o vão entregue **nunca é mais apertado que o pedido**, a
mesma lei do irmão do Vector. Gate com o discriminante: spacing 2,2 num comprimento 10 dá floor 4
e vão **2,50** ✔, contra o round 5 e vão 2,00 ✘.

⚠️ **`mode = Number` é BYTE-IDÊNTICO ao nó que shipava** (gate por `to_bits()`, com o `spacing`
**ARMADO** em 0,37 — senão ele seria verde sobre um param inerte).

⚠️ **Uma mutação SOBREVIVEU e o achado é dela:** o guard `n >= 1.0` do `copies_that_fit` é
**redundante através do `floor`** (o floor de qualquer coisa abaixo de 1 já é 0). Removido em vez
de gateado — *um gate que tivesse de mentir sobre o que prova é pior que a linha a menos*; o
`is_finite` fica, com a medição no doc-comment (o param é dirigível por fio, doc 58).

### 2.2 O `spacing` declara que é uma DISTÂNCIA

`ParamUnit::Length`, o que o irmão `spline_wrap` declara para as coordenadas dele. ⚠️ O
`motion.path` **nunca declarara unidade nenhuma**, e uma wave que acrescenta uma distância sem
unidade é a omissão nova que o censo do doc 88 existe para achar. **Os outros três ficam sem
unidade e isso é DECISÃO**: `count` é contagem, `align` é interruptor, e o `offset` é FRAÇÃO — o
`spline_wrap` deixa as frações dele igualmente sem unidade, e divergir faria dois nós da mesma
família apresentarem a mesma grandeza de dois jeitos.

### 2.3 O gate que impede a célula de renascer

`ph2d-node-registry-init/tests/path_controls.rs` — três gates que medem as duas rotas sobre a
MESMA curva desenhada:

- o **CONTROLE** (as duas pousam na curva: sem ele, *"o composto recorta"* seria satisfeito por uma
  rota que simplesmente não pousa nela);
- o recorte por composição **e a metade oposta** (o vão do controle **não se move** varrendo o
  único knob que ele tem);
- o deslocamento perpendicular e o **LADO**, pela distância **COM SINAL** (uma régua sem sinal
  ficaria verde com os dois lados colapsados num só).

⚠️ **A barra é `> 0,3` e não `≈ 0,5`, e é geometria:** o lado côncavo do canto encolhe a distância
medida (o ponto mais próximo passa a ser o VÉRTICE) — o que é categórico é o **SINAL**.

Sonda irmã `measure_path_controls.rs` (`-- --ignored --nocapture`) com os números.

### 2.4 A cena

⚠️ **Ela vive no `PH2D_MOTION_NODE_PATH_SMOKE` (modo `=2`) e NÃO no roteador de
`PH2D_GPU_COOK_DEMO`, e o motivo é estrutural:** as cenas de conferência montam um `MotionDoc`
puro, e um nó que anda numa forma **DESENHADA** precisa do documento vetorial — a forma, a entidade
que a nomeia e o publisher, que é exactamente o roteiro de dois frames que aquele arquivo já
executa. Uma segunda encenação disso seria a segunda resposta a *"como uma curva chega ao grafo?"*.

O gate de env virou **MODO**, com o braço `_` mantendo `=1` na cena que o Enio já smokou.

**Quatro trilhas RETAS**, duas curtas (4) e duas longas (8):

| # | Trilha | Lei | Medido |
|---|---|---|---|
| 1 | Count Short | `count 9` | 9 peças, vão **0,444** |
| 2 | Count Long | `count 9` | 9 peças, vão **0,889** |
| 3 | Spacing Short | `spacing 0,50` | **8** peças, vão **0,500** |
| 4 | Spacing Long | `spacing 0,50` | **16** peças, vão **0,500** |

⚠️ **Retas de propósito:** a `=1` já responde *"o conjunto percorre uma curva?"*; esta responde
*"quantos, e a que distância?"*, e num segmento o vão é o número que o olho lê sem descontar
curvatura. ⚠️ **E quatro trilhas, não duas:** duas cadeias sobre a MESMA curva desenhariam as peças
umas por cima das outras, e o vão — que é a coisa que a cena existe para mostrar — ficaria
ilegível.

**Gates da cena, em DOIS lugares porque falham por motivos diferentes:**

- `spacing_scene.rs` (cook) — as duas leituras da mensagem, mais *as quatro fileiras não são a
  mesma fileira quatro vezes*;
- `the_spacing_scene_arms_what_its_message_promises.rs` (arch, shell) — ⚠️ **o gate de cook monta o
  grafo ele próprio, logo é CEGO à cena**: se o wiring esquecer o `mode`, as quatro fileiras caem
  na contagem por número e os três gates de lei seguem **verdes**. Mais o par *tabela impressa ×
  números produzidos*, que é exactamente o que apodrece quando alguém afina um e esquece o outro.

---

## 3. Superfície de colisão — MEDIDA, não auto-relatada

| Item | Estado |
|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** — ⚠️ conferido nos **quatro** arquivos da família (`project.rs` · `project_schema.rs` · `project_schema_tests.rs` · `project_load.rs`), `git diff main...HEAD` **vazio** em todos. A `line/physics` PARTIU o `project.rs` em 15/08, e *um degrau escrito no arquivo antigo funde LIMPO e evapora* |
| Contrato congelado | **INTOCADO** (`git diff` vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs`) — os dois params vivem no `MANIFEST` do NÓ, e `ParamGate`/`ParamUiHint`/`ParamUnit` são side-metadata do registry |
| Registro do `ph2d-ecs` | **INTOCADO** ⇒ os três espelhos também |
| `Cargo.toml` / `Cargo.lock` | **ZERO** |
| ADR | **nenhum** ⇒ a linha fica **FORA de toda disputa de número** |
| Ids / scrollbar / i18n | **nenhum novo** |
| Cenas | ⚠️ **nenhum nível novo no roteador de `GPU_COOK_DEMO`** (segue em **57**, próxima livre **58**); a cena é um **modo** de uma env que já existia |
| Censo | **125 nós · 547 params · 528 com hint · 159 com unidade** — ⚠️ **RECONCILIA** com o do grupo P (`125 · 545 · 526 · 158`) exactamente pelos dois params desta wave, dos quais só o `spacing` carrega unidade |

**O ponto de merge sensível:** `shells/desktop/src/motion_node_path_smoke.rs` (165 → 320 LOC, cap
600) e `docs/Motion Nodes/89_conferencia/06_animadores.md` (a linha 46 e a Contagem). Nenhuma outra
linha viva toca os dois.

---

## 4. Gate de fechamento

| Gate | Resultado |
|---|---|
| `cargo fmt --all -- --check` | **EXIT 0** na árvore inteira |
| clippy `--all-targets -- -D warnings` nas 3 crates | **EXIT 0, zero warnings** |
| `ph2d-node-motion-path` | **11 gates**, verdes |
| `ph2d-node-registry-init` | **41 suítes ok**, zero falha |
| `every_demo_scene_ends_in_an_output_node` | ok |
| `file_loc_caps` (shell, 600) · `architecture_workspace_file_loc_cap` | ok |
| `no_tofu_glyphs` · `node_id_collisions` · `architecture_panel_wiring_parity` | ok |
| Mutações | **6 sangram** (4 no nó · 2 na cena) + **1 sobrevivente REMOVIDA** (guard redundante) |

⚠️ **E o commit `d51ea2baf` shipou fmt-VERMELHO**, corrigido em `edad2aa86`: o gate de fechamento
por `cargo test -p` **não inclui fmt**, que só aparece no `scripts/ship.sh` — *um vermelho que só o
ship vê é invisível entre integrações*, a mesma causa que a integração de 2026-08-16 achou em
quatro arquivos do `main`.

---

## 5. Smoke

```
env PH2D_MOTION_NODE_PATH_SMOKE=2 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime a tabela das quatro trilhas; se a lista não aparecer, PARE.** Ela **julga-se
PARADA** (não há relógio nisto).

**As duas leituras:**

1. as **de cima** têm o MESMO número de peças e vãos **diferentes** — a longa espalha o dobro;
2. as **de baixo** têm o MESMO vão e números **diferentes** — a longa cabe o dobro de peças.

⚠️ **Se as quatro fileiras tiverem a mesma contagem, o modo Spacing não chegou.**

E o **CONTROLE**: `env PH2D_MOTION_NODE_PATH_SMOKE=1` tem de continuar exactamente como estava (a
cena que o Enio já aprovou).

---

## 6. Aberto

- O **GESTO** dos quatro controles que a composição já exprime é `P2` — fechá-lo é **UI** (um
  preset de cadeia, ou o painel oferecer o irmão), não motor.
- ⚠️ A folha 06 tem **7 P1**; a folha 03 tem 6 e a 07 tem 3 (placar derivado por
  `ferramentas/placar_conferencia.py`, que **IMPRIME e sai vermelho** — `--write` não existe, e
  quem reconcilia a linha de `Contagem` é quem roda).
- ⚠️ **A primeira coisa de toda wave desta conferência é MEDIR se a composição já exprime o item.**
  Esta é a **oitava** célula a envelhecer: *o que se perde ao não reconferir não é tempo, é
  construir o que já existe.*

---

## 7. Integração

A linha **não integra e não pusha**. Ordem explícita do Enio ⇒ agente integrador dedicado
(DIRETRIZ §1.5.3).
