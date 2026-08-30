# HANDOFF — `line/quadextract` (2026-08-30) — a RÉGUA LOCAL, e o `Follow Curvature` vivo

> Continuação de [29/08](HANDOFF_INTEGRACAO_line_quadextract_2026-08-29.md). A linha reabriu
> sobre o `main` integrado, com o stack subido (`wgpu` 29 · `vello` 0.10 · `rapier` 0.35) e o
> cache de compilação mudado de disco.
>
> ⚠️ **Método novo, por ordem do Enio (30/08):** *«não em micro passos; cada etapa termina num
> smoke; se tiver complexidade, é auditada antes de sugerir o smoke.»*

---

## §1 — O que a reabertura mediu, antes de tocar em código

| | |
|---|---|
| a linha no `main` | ⭐ tudo integrado; `line/quadextract` = `main` (`066b4f92e`), árvore limpa |
| stack novo | ⭐ workspace compila a frio em `2 min 31`; **4 078** testes do shell, `0` falhas |
| o botão na peça do artista | ⭐ `χ = 2` · `0` bordo · `0` não-manifold · `100 %` quads, e **4,7× mais rápido** (`27,8 s → 5,9 s`) |
| ⛔ `ph2d-quadbench/` | **não sobreviveu à mudança de disco** — nunca esteve no git (o oráculo é restrito). O corpus e as fases gravadas dele **não existem nesta máquina**; as sondas que o lêem falham **alto** (`expect`), não em silêncio |

---

## §2 — ⭐⭐⭐ A RÉGUA LOCAL (`ph2d_quadfill::local`)

O report de 29-30/08 não era visto por régua nenhuma: na mesma peça o botão relatava
`χ = 2` · `0` bordo · `0` não-manifold · `1` ilha · `100 %` quads · `>60° = 0`. **Toda régua verde.**

A `shape` mede os **cantos** de cada face e resume em percentis. A `local` mede o que os cantos
não podem ver, **e onde**:

| coluna | o que apanha |
|---|---|
| `warp_deg` | o quad **não-plano** (ângulo entre as normais das duas metades) |
| `kind` | **gravata** — o quad que se auto-intersecta (contagem de sinal sobre a normal de Newell) |
| `squareness` | a **lasca** (área ÷ aresta média²) |
| `radial` | **onde** — `0` é o centro, `1` é a ponta |

⚠️ **Cada gate traz o CONTROLO**: a mesma fixtura medida pela `shape`, a provar que ela fica
**verde**. Uma gravata mede `45°` de desvio de canto — abaixo da barra de `60°` — e aspecto `√2`.

### ⛔ Uma mutação sobreviveu e derrubou uma afirmação minha

O doc dizia que o `max` sobre as duas diagonais existe porque *«uma sela é plana ao longo de uma
e torcida ao longo da outra»*. **Falso** — quatro pontos ou são coplanares ou não são:

| fixtura | diagonal `0–2` | diagonal `1–3` |
|---|---|---|
| sela | `109,47` | `109,47` |
| canto levantado | `63,20` | `70,25` |
| assimétrico | `68,67` | `60,19` |

A razão certa é outra (*o número não pode depender da diagonal que o renderizador triangulou*), e
⚠️ **o gate precisou de DUAS fixturas, uma de cada ORDEM**: com só a assimétrica — onde `0–2` é a
maior — a mutação *«olha só a `0–2`»* sobrevivia. *Uma fixtura em que o máximo calha ser o
primeiro argumento não distingue `max` de «o primeiro».*

---

## §3 — ⛔⛔⛔ TRÊS hipóteses REFUTADAS pela medição

Medido nas três malhas que o artista mandou:

| malha | faces | defeitos | gravatas | na ponta |
|---|---|---|---|---|
| a escultura dele (entrada) | `15 275` | `0,47 %` | `4` | `2,83 %` |
| ⭐ **a nossa saída** | `15 426` | **`0,11 %`** | `1` | `0,15 %` |
| Blender / QRemeshify | `8 291` | `0,42 %` | `1` | `0,20 %` |

⇒ **a nossa malha é a mais limpa das três.** Torção, gravata e lasca **não** são o mecanismo.

E a quarta hipótese — a **orientação**, que este handoff nomeia como *«a que se lê literalmente
como buraco»* (face virada renderiza pelo lado de dentro e sai preta) — dá **`0` arestas viradas
nas três malhas**.

---

## §4 — ⭐⭐⭐ O QUE É: a densidade RADIAL, com alvo DERIVADO

| aresta-equivalente mediana | corpo | **ponta** | razão |
|---|---|---|---|
| Blender (o que ele aprovou) | `0,0439` | `0,0261` | ⭐ **`0,59`** |
| **nós** | `0,0306` | `0,0361` | ⛔ **`1,18`** |

E a contagem: a ponta dele tem **`674`** faces, a nossa **`370`** — metade, numa malha `1,9×` maior.
⇒ *«as pontas têm menos densidade de faces e perdem detalhes»* (Enio, 28/08) é **literalmente
verdade**, e o alvo passa a ser **derivado** (`0,59`), não escolhido.

⚠️ **Isto não reabre as recusas de 28/08 sem qualificação:** elas foram medidas com o
`relief_density` (expoente global de `aresta ∼ curvatura`) — uma correlação sobre a peça inteira,
instrumento muito mais fraco que a casca radial.

---

## §5 — ⭐⭐ A TRANSFERÊNCIA, medida com UMA lei nos dois lados

`ph2d_quadfill::tip_body_ratio` tem **dois consumidores de propósito**: o **pedido** (um valor por
vértice da malha de trabalho) e a **entrega** (a raiz da área por face da saída). Domínios
diferentes, lei igual — *medi-los com duas funções daria dois números que ninguém pode dividir.*

Na peça do artista, `Detail 0,85`, `Follow Curvature = 1`:

| | razão ponta/corpo |
|---|---|
| o campo **PEDE** | ⭐ `0,486` — *melhor que o `0,59` do oráculo aprovado* |
| a cadeia **ENTREGA** | ⛔ `1,144` |

⇒ **o pedido está certo; a cadeia descarta-o e ainda inverte o sinal.**

---

## §6 — ⛔ A cura por ALISAMENTO: construída, medida, REFUTADA — e o que ela revelou

**Hipótese:** o G3 resolve `min ‖∇f − R/h‖²`, cuja condição de óptimo é `Δf = ∇·(R/h)` — um
**passa-baixo**. Um `h` de alta frequência sairia lavado. ⇒ alisar o pedido em log.

⛔ **Refutada, e pela razão certa:** `48` rondas movem o pedido de `0,486` para `0,502` — **`3 %`**.
*O pedido nunca foi de alta frequência*, e alisar não é o que move a densidade.

⚠️ **A 1.ª versão desta varredura não podia dizer isso:** a sonda do `PEDE` corria **antes** de
`smooth_in_log`, e as quatro corridas imprimiram o mesmo `0,486`. *Uma sonda posta antes do passo
que ela devia medir mede o passo anterior.*

⭐⭐ **Mas a varredura devolveu outra coisa: o alisamento compra a FORMA.** As faces com canto pior
que `60°` caem de `8` para `0` — melhor que a linha de base. *A adaptação passa a ser de graça em
qualidade.* ⇒ `SIZING_SMOOTH_ROUNDS = 8`, com a tabela no doc.

⚠️ **Um gate que já existia apanhou uma regressão minha:** a 1.ª versão alisava **depois** de a
contagem prevista estar calculada, e `a_densidade_segue_a_curvatura_sem_mudar_a_contagem` reprovou
com `−3,1 %` (barra `2 %`). *Normalizar por um número medido sobre um campo que já não existe é
normalizar para nada.* ⇒ o alisamento corre **antes** da contagem.

---

## §7 — ⭐⭐⭐ O RESULTADO: o `Follow Curvature` deixa de ser um knob morto

Peça do artista, `Detail 0,85`:

| | quads | razão ponta/corpo | faces na ponta | `>60°` | `χ` / bordo / não-manif. |
|---|---|---|---|---|---|
| knob **desligado** (hoje) | `9 188` | ⛔ `1,533` | `82` | `2` | `2 / 0 / 0` |
| ⭐ knob **ligado** | `8 257` | ⭐ **`1,062`** | ⭐ **`142`** | ⭐ **`0`** | `2 / 0 / 0` |

**A ponta passa de `53 %` mais grossa para `6 %`, com `+73 %` de faces lá, zero faces péssimas, e
a topologia intacta.** Preço: `−10 %` de quads (o slider redistribui, não cria).

### ⛔⛔ E porque ele NASCE DESLIGADO

Na fixtura sintética de espinhos (`espinhos:6`) o mesmo knob **parte a topologia**:

| | quads | razão | `>60°` | `χ` | bordo | não-manif. |
|---|---|---|---|---|---|---|
| desligado | `9 469` | `1,597` | ⭐ `0` | `2` | ⭐ `0` | ⭐ `0` |
| ligado | `8 122` | `1,064` | ⛔ `9` | `2` | ⛔ `4` | ⛔ `1` |

⇒ *uma fase medida sozinha pode melhorar e piorar o produto* — a lei que esta linha já pagou três
vezes. **O knob fica alcançável e o default fica em `0`**; a decisão é do dono do produto, com a
tabela na mão.

---

## §8 — ⏳ ABERTO

- ⛔ **A transferência continua rota** (`0,486` pedido → `1,062` entregue). O alisamento foi
  ilibado; o mecanismo é o do §8-quater de 28/08 (a projecção de mínimos quadrados), e a cura
  publicada é o **factor de escala conforme por construção** (`Δ log h` contra a curvatura de
  Gauss). ⚠️ **Agora ela tem régua para ser julgada** — `tip_body_ratio` nos dois lados.
- ⏳ **Porque o knob parte a topologia na fixtura sintética e não na peça do artista** — não
  medido. É o gatilho que decide se ele pode nascer ligado.
- ⏳ o `ph2d-quadbench` **não existe nesta máquina** (§1); toda comparação fase-a-fase com o
  oráculo está indisponível até ele voltar.
- ⏳ o motor **`Fast`** do menu continua a um clique, com a saída pior (herdado).

## §9 — Ponto cego novo na ferramenta do laço interno

`scripts/cargo-check-narrow.sh` corre `cargo check -p` **sem `--all-targets`** ⇒ não compila
`#[cfg(test)]` nenhum, e imprimiu *«compila»* sobre um `use` que não resolvia. É a **quarta**
variante da mesma cegueira (memória actualizada). ⇒ ao mexer em código de teste, `--all-targets`.
