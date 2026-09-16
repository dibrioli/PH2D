# 112 — CICLO 7: APARÊNCIA, a cor e o rasto

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«A cor e o rasto»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_aparencia_probe.rs`](../../crates/ph2d-app-motion/src/motion_aparencia_probe.rs); o
> mecanismo de cada wave vai para o handoff dela.

---

## §1 — O grupo, DERIVADO da paleta

O grupo é **a categoria `Fx` da paleta** (o cabeçalho magenta que o artista vê), pedida ao
registry e sem as fixturas — e **não** um prefixo: a família mistura `fx.*` e `motion.*`, e uma
lista escrita à mão aqui envelhecia em silêncio no dia em que um nó `Fx` nascesse.

| família | nós |
|---|---|
| `fx.*` | 3 — `drop_shadow` · `glow` · `rgb_split` |
| `motion.*` | 7 — `color_array` · `color_ramp` · `slit_scan` · `strobe` · `sub_uv` · `tint` · `trail` |
| **total** | **10** |

⚠️ Coincide com a linha do doc 103 §5, e isso é uma **verificação**, não a fonte. Gate
`the_fx_group_is_derived_and_not_empty` (piso de `10` e as **duas** metades da família — um filtro
que só apanhasse os `fx.*` leria `3`).

---

## §2 — O RETRATO (sonda `audit_the_fx_group`, 2026-09-16)

```text
  nó                        | params | no cartão | device | portas | efeito
  --------------------------|--------|-----------|--------|--------|--------
  fx.drop_shadow            |      8 |         5 |  NAO   | 1->1   | Pure
  fx.glow                   |     15 |        13 |  NAO   | 1->1   | Pure
  fx.rgb_split              |      8 |         4 |  NAO   | 1->1   | Pure
  motion.color_array        |      0 |         1 |  sim   | 2->1   | Pure
  motion.color_ramp         |      0 |         1 |  sim   | 2->1   | Pure
  motion.slit_scan          |      1 |         1 |  NAO   | 2->1   | Pure
  motion.strobe             |     10 |         8 |  NAO   | 3->1   | Pure
  motion.sub_uv             |      5 |         6 |  sim   | 2->1   | Temporal
  motion.tint               |     10 |         3 |  sim   | 1->1   | Pure
  motion.trail              |     11 |        11 |  NAO   | 2->1   | Pure
```

⚠️ Esta tabela é a fotografia **antes** da W1a e fica assim de propósito — reescrevê-la apagaria o
retrato de que o §3 é a resposta. *Corra a sonda antes de citar a coluna.*

**O que ele já diz, sem uma linha de código:**

1. ⛔ **Seis de dez fora do dispositivo** (`device = NAO`).
2. ✅ **O vocabulário está LIMPO** — as três perguntas da sonda `the_fx_vocabulary_the_artist_reads`
   (um rótulo com várias chaves · uma chave com vários rótulos · palavras de enum que se leem como
   variantes) devolvem **zero** linhas.
3. ⚠️ **As diferenças `params`/`no cartão` são GATES DE MODO, não controlos perdidos** — o `tint`
   mostra `3` de `10` porque a segunda cor só existe em `Gradient`; o `rgb_split` mostra `4` de `8`
   porque o centro, a força e o início só existem em `Aberration`; os `r/g/b/a` de toda cor são UMA
   linha (`Color`). ⏳ A confirmar pelo censo do alcance (W2), não por esta leitura.
4. ⚠️ **O `motion.slit_scan` tem UM botão** (`Lag`). Um slit-scan tem muito mais a dizer (direcção,
   o campo que decide o atraso de cada elemento, a interpolação) — é o candidato óbvio da W3.
5. ⚠️ **Os `motion.color_array`/`color_ramp` declaram `0` params e mostram `1`**: a linha é o editor
   rico (paleta/gradiente), que vive num param de TEXTO. Não é defeito.

---

## §3 — ⛔⛔⛔ O ACHADO QUE DECIDE O CICLO: a aparência é o ÚLTIMO nó, e o último nó decide a rota

A lei 1 do protocolo manda que todo nó do grupo diga onde corre. ⚠️ **Um `NAO` no retrato não é um
preço** (a lição do ciclo 6, doc 110 §11.2): ele só custa se o nó estiver no caminho do OBJECTO, e o
preço é quantos elementos sobem na costura. ⇒ sonda `probe_does_an_fx_chain_stay_on_the_device`:
cada nó no meio de uma cadeia que **já** está no dispositivo.

```text
  grid 320² -> scale -> X -> output  | onde corre  | stages | costura            | elementos que SOBEM
  -----------------------------------|-------------|--------|--------------------|--------------------
  (sem X — o controlo)               | dispositivo |      3 | —                  | — (nada sobe)
  fx.drop_shadow                     | ⛔ CPU       |      1 | fx.drop_shadow:0   | 204 800
  fx.glow                            | ⛔ CPU       |      1 | fx.glow:0          | 102 400
  fx.rgb_split                       | ⛔ CPU       |      1 | fx.rgb_split:0     | 102 400
  motion.color_array                 | dispositivo |      4 | —                  | —
  motion.color_ramp                  | dispositivo |      4 | —                  | —
  motion.slit_scan                   | ⛔ CPU       |      1 | motion.slit_scan:0 | 102 400
  motion.strobe                      | ⛔ CPU       |      1 | motion.strobe:0    | 102 400
  motion.sub_uv                      | dispositivo |      4 | —                  | —
  motion.tint                        | dispositivo |      4 | —                  | —
  motion.trail                       | ⛔ CPU       |      1 | motion.trail:0     | 102 400
  motion.strobe + pulse.beat         | ⛔ CPU       |      1 | motion.strobe:0    | 102 400
```

⭐⭐⭐ **A costura não cai no nó: cai na CADEIA.** O `grid → scale` que estava no dispositivo passa a
cozinhar na CPU para alimentar o nó, e o stream inteiro sobe a cada quadro. E um nó de aparência é,
por natureza, **o último de um grafo** — a cor, o brilho e a sombra aplicam-se depois de tudo o
resto. ⇒ ***todo grafo que usa brilho, sombra, separação RGB, rasto, estroboscópio ou slit-scan
corre inteiro na CPU***, e isso é o `50,9×` do [doc 98](98_auditoria_de_performance_2026-09-01.md).

⛔⛔ **E o pior dos seis era o mais barato de curar: o `fx.glow` é um PASSA-TUDO.** O `eval` dele é
`input.clone()` — os parâmetros são lidos pelo RENDERIZADOR (`from_graph`, no `present_fx`), nunca
pelo cozimento. Ele derrubava a cadeia inteira e subia `102 400` elementos **para lhes não mudar um
byte**.

⚠️ **E há um segundo achado dentro da tabela:** o `fx.rgb_split` subiu `102 400` e não `307 200`,
apesar de triplicar as linhas. A razão é o `MAX_INSTANCES = 262 144` — um tecto **medido no caminho
de CPU** (`~10–15 ns` por linha emitida, `~3 ms` no tecto), que **desliga o efeito em silêncio**
acima dele: a `102 400` objectos a separação RGB **não faz nada**. Enquanto o nó vive na CPU o
tecto está certo; ⚠️ *no dia em que ele for para o dispositivo, o recurso muda e o número tem de ser
re-medido* (`CLAUDE.md` §0.0: **nunca deixe o fallback definir o produto**).

---

## §4 — ✅ W1a FEITA: o brilho deixa de levar o grafo para a CPU

Uma linha: `reg.register_gpu_kernel(MANIFEST.id, GpuKernel::PASSTHROUGH)` — o molde que o
`pulse.signal` já usava (*«um nó que derrubasse a cadeia inteira para a CPU seria a pior espécie de
sonda: a que muda o programa que mede»*). Perante um kernel sem corpo e sem bindings o sequenciador
**não emite passe nenhum** e a corrente atravessa-o, que é literalmente a lei da CPU.

```text
  grid 320² -> scale -> fx.glow -> output | antes         | depois
  ----------------------------------------|---------------|------------------------
  onde corre                              | ⛔ CPU          | dispositivo
  estágios no dispositivo                 | 1             | 4 (o passa-tudo não emite passe)
  elementos que sobem por quadro          | 102 400       | 0
```

⚠️ **O efeito continua a ser desenhado:** o passe do brilho lê o grafo
(`ph2d_node_fx_glow::from_graph(&motion.doc.graph)` no `present_fx` e no `fase_vector_fx_recook`),
e nenhum dos dois pergunta por onde o cozimento passou.

⭐ **A catraca** `the_fx_group_route_only_improves` — um nó do grupo no caminho do objecto não pode
levar a cadeia para a CPU sem estar **nomeado** na lista `NA_CPU`, com a razão; e as **duas
metades**: um nó da lista que passe a ficar no dispositivo reprova até a linha dele ser apagada
(*uma catraca sem censo de obsolescência vira licença*). Mutação (tirar o `PASSTHROUGH`): **RED** —
*«`fx.glow` leva a cadeia para a CPU»*.

---

## §5 — A fila do ciclo

1. ✅ **W1a — o brilho passa-tudo** (§4).
2. ⏳ **W1b — os que MULTIPLICAM as linhas** (`fx.rgb_split` ×3 · `fx.drop_shadow` ×2). O
   substrato tem metade: a `CountLawCtx` já dá a contagem de cada entrada, logo `n × k` exprime-se;
   falta o corpo ler a linha-fonte `i mod n` e o índice da cópia `i / n`. ⚠️ **E o `MAX_INSTANCES`
   tem de ser re-medido no dispositivo** (§3) — hoje ele apaga a separação RGB a `102 400` objectos.
3. ⏳ **W1c — os que têm ESTADO** (`motion.strobe` · `motion.slit_scan` · `motion.trail`). O
   `motion.strobe` é o consumidor que o ciclo 6 deixou a apontar para aqui (doc 110 §8.6: os nove
   `pulse.*` estão no dispositivo **sem consumidor**). ⚠️ O `trail` em `Resampled` re-cozinha a
   própria entrada em N instantes (ADR-0163) e é CPU **por desenho**; o modo `Remembered` não.
4. ⏳ **W2 — o cartão e o alcance** — o censo do ciclo 6 sobre o grupo; confirmar o §2.3.
5. ⏳ **W3 — o poder que falta** — as folhas 06 (animadores), 09 (cor) e 11 (fx raster) da
   conferência; o `slit_scan` de um botão só (§2.4); o P2 aberto da folha 11 (a *dirt texture*).
6. ⏳ **W4 — a MEDIÇÃO** — residência e relógio do grupo.
7. ⏳ **W5 — a cena e o TUTORIAL** *«A cor e o rasto»* — o smoke do dono.

⚠️ **A ordem W1 → W2 não é preferência: é a lei 1 do protocolo.** Um grupo cujo uso normal leva o
grafo inteiro para a CPU não fecha um ciclo com «tem mais botões».
