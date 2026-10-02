# ARQUIVO — CLAUDE.md (história, 27 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-1531,1559-1570` do original.
>
> ⚠️ **A única alteração ao corpo:** 6 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Flip** — animação 2D no idioma do Grease Pencil: tira de quadros, onion, tween v2 (correspondência por atribuição
  ótima + espiral logarítmica), **colorize LazyBrush**, multiplano 2.5D, airbrush, pressão, e o
  **motor novo de traço**, em que o traço deixa de ser rasterizado e passa a ser **PERCORRIDO**
  (`τ = ∫ f(dn) ds / pitch`, `α = 1 − exp(−τ)`) — a lei aditiva da tinta, que é o **limite** dos buffers de dab da
  indústria. Dois motores, **uma lei**: referência em CPU e o compute que shipa, unidos por gate de paridade cuja barra é
  **derivada do formato** (`rgba16float` ⇒ `2⁻¹¹`).
  ⚠️ `PH2D_FLIP_NEW_ENGINE=0` volta ao rasterizador antigo (vivo e testado, útil para bissecar).
  **Aberto:** ⏳ **W-Saída — o Flip sai do Flip** (Enio, 2026-08-23, **fim da fila**): assar um
  quadro em pixels destrava **três** features que já existem do outro lado — os 16 exportadores de
  imagem, o `Pack into Sheet`, e a camada do Painter. ⚠️ **É UM buraco, não três:** a entidade de um
  objeto Flip não tem `Sprite` nem pixels, e as três portas só sabem o que é um pixel; o primitivo
  de leitura **já existe** no `walk_gpu` (o harness de paridade usa-o). ⭐ E o T2 fecha o círculo
  com a §11 do Sprite — uma tira empacotada **é** o pool que uma `AnimationTag` percorre. Plano:
  [`01_plano_waves.md` §W-Saída](../../Flip/01_plano_waves.md) ·
  cache em **tiles de MUNDO** (sobreviver ao pan) · o **resíduo de quina** que a lei de área expôs
  (**13 px de 1115** — *não é regressão*) · cache **incremental** do ajuste · e **três itens que são decisão do Enio, já
  devolvidos com os números**: o resíduo de quina, **joins & caps** (⛔ a premissa de correção foi **refutada**; sobra
  pergunta de produto) e a **terceira lei** (o `Soft` do Krita — funciona exato, muda a borda em **+69%**).
  **Smokes:** `PH2D_FLIP_HARDNESS_SMOKE` (o mestre) · `PH2D_FLIP_COLORIZE_SMOKE` · `PH2D_FLIP_STRIP_SMOKE` ·
  `PH2D_FLIP_TIP_SMOKE` · `PH2D_FLIP_TWEEN_SMOKE` / `_PAIRS_` / `_PHASE_` / `_TORSION_` · `PH2D_FLIP_MULTIPLANE_SMOKE` ·
  `PH2D_FLIP_SELF_OVERLAP_SMOKE` · `PH2D_FLIP_AIRBRUSH_SMOKE` · `PH2D_FLIP_RESAMPLE_SMOKE` · `PH2D_FLIP_PRESSURE_SMOKE`.
  Diagnóstico: `PH2D_FLIP_STATS=1` · `PH2D_WALK_DUMP=<dir>`.
  ⚠️ **Rode a suíte em DEBUG e RELEASE** — um gate desta linha reprovou **só em debug** (um bar de relógio mede o perfil do build).
  **Ler:** [`docs/Flip/`](../../Flip) · [`BUGS_flip.md`](../../Flip/BUGS_flip.md) ·
  [`12_novo_motor_pesquisa.md`](../../Flip/12_novo_motor_pesquisa.md) · [handoffs](../../Flip/handoffs/README.md) ·
  [história](../estado-2026-08-18/flip.md)

