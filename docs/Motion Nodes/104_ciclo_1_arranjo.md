# 104 — CICLO 1 · ARRANJO: pôr muitos objectos na tela

> O primeiro ciclo da [dinâmica](103_dinamica_dos_ciclos.md). **Tutorial:** *«Do primeiro
> objecto ao milhão»*. ⚠️ Este é o único ciclo que carrega o **substrato** (os params passam a
> viver no cartão e o painel lateral sai); os ciclos 2+ só pagam o grupo deles.

## §1 — O grupo (10 nós), e por que estes

Os nós que **põem objectos na tela e decidem onde eles ficam** — a primeira coisa que um artista
faz, e por isso o primeiro tutorial. Contagens do registry em 2026-09-05.

| nó | nome na tela | params | categoria hoje |
|---|---|---:|---|
| `motion.grid` | Grid | 4 | Source |
| `motion.scatter` | Scatter | 4 | Source |
| `motion.distribute_radial` | Radial Array | 7 | Source |
| `motion.fibonacci` | Fibonacci Spiral | 3 | Source |
| `motion.lattice` | Lattice | 4 | Source |
| `motion.voronoi` | Voronoi | 5 | Source |
| `motion.distribute_poisson` | Poisson Disk | 4 | Distribute |
| `motion.distribute_curve` | Curve Points | 12 | Source |
| `motion.path` | Path | 8 | Distribute |
| `motion.clone` | Clone | 10 | Distribute |

⚠️ **Primeiro achado, e é de facilidade de uso:** para o artista estes dez são **uma** família
(*«onde é que as coisas ficam»*), e a paleta parte-os em **duas** categorias — e mete os sete de
`Source` no mesmo saco que o `Emitter`, o `Boids`, o `Soft Body` e o `Verlet Rope`, que são
**simulações**, e que o `Shape`/`Object`/`Text`/`Table`, que são **entradas**. As duas
categorias sobrecarregadas (`Transform` 43 e `Utility` 43) já têm sub-clusters
(`palette_subgroups`); `Source` (19) não tem. ⇒ item do ciclo.

## §2 — Estado do substrato (o que já ficou pronto)

| passo | estado |
|---|---|
| medir o custo de uma row e de um cartão | ✅ [doc 103 §7](103_dinamica_dos_ciclos.md) — **11,7 µs/cartão · 2,8 µs/row** |
| `CardParam` + a shell a preencher pela porta única | ✅ `stamp_card_params` ← `shown_params` |
| a faixa na geometria (`param_band_top` · `param_row_rect` · `readout_top`) | ✅ 6 gates, 2 provas de mutação |
| o pintor da row (faixa inteira, valor à direita, nível como preenchimento) | ✅ `paint_card_params.rs` |
| LOD por legibilidade (`11 px × zoom ≥ 9 px`) | ✅ |
| **arrastar** a row para editar (hit-test + intent) | ⏳ |
| **secções dobráveis** no cartão (`ParamGroup`) | ⏳ |
| **os editores ricos** (curva, gradiente, paleta, texto, ficheiro, cor) | ⏳ |
| **o painel lateral sai** | ⏳ |
| auditoria + upgrade dos 10 nós | ⏳ |
| o tutorial em PDF | ⏳ |

## §3 — A auditoria do grupo (passo 2 do ciclo)

⏳ A escrever. Perguntas que ela tem de responder por nó: *que params a referência expõe e nós
não* · *corre no device?* · *quantos objectos por ms?* · *o cartão dele cabe?*
