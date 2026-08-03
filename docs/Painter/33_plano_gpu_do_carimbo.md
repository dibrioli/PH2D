# 33 — O carimbo na GPU

> Ordem do Enio, 2026-08-03: *"vamos ao GPU"*, depois do log de smoke que atribuiu o custo.
> Pré-requisito lido: [ADR-0146](../architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)
> (a Fase 1 mede a FRONTEIRA e decide as outras) e a §5.15 do [doc 28](28_otimizacoes_o_que_funcionou.md).

## 1. O que o produto MEDIU, e por que a CPU acabou aqui

Log do smoke (`PH2D_IMPASTO_SMOKE=1 PH2D_PAINT_PERF=1`, canvas 4096², elipse viva em Digital):

```
re-stamp por entrega: restore 1.34ms | relevo 0.00ms | save 1.48ms | CARIMBO 18.30ms (x47)
deposito: 47 lotes em BANDA x 0 serial(is), 8342 dabs, 813.04 M visitas (1.2 ns/visita)
```

| fato | número | como se sabe |
|---|---|---|
| a rota em banda é tomada | **47 de 47** | `deposito`, e o gate `the_artists_default_brush_takes_the_banded_road_on_a_live_figure` |
| o carimbo É o custo | **18,30 de 21,12 ms = 87%** | as quatro fases, medidas no código que shipa |
| o kernel está NA velocidade | **1,06 ns/visita** | contra **1,69** da sonda pós-wave: o produto é mais RÁPIDO por texel |
| o trabalho | **17,3 M visitas/entrega** | num canvas que tem **16,7 M pixels** |
| o pincel do artista | **raio ≈ 155 px** | derivado: `17,3 M ÷ 177 dabs = 97.460 texels/dab ⇒ lado 312 = 2r+2` |
| a redundância | **10,1×** | espaçamento é 10% do diâmetro ⇒ cada texel é blendado ~10 vezes |

⚠️ **Duas coisas que a medição derrubou e que não devem ser re-propostas:**

1. **O `save_region` NÃO é o gargalo.** Ele aloca um `Vec` do tamanho da figura por quadro e eu o
   nomeei como o próximo alvo; medido, **1,48 ms — 7%**. Restore + save juntos são 13%.
2. **Não há kernel lento nem rota errada.** `área × 10 × 1 ns` **é** os 18 ms. A CPU está no teto
   desta configuração, e é por isso que a alavanca seguinte é o dispositivo — não uma micro-otimização.

⚠️ **E o raio NUNCA se assume.** Eu derivei um `28 ns/visita` de um raio de 40 (o que o smoke arma)
antes de o log publicar as visitas; o artista usava ~155, o custo é **quadrático**, e a atribuição
estava 15× errada. O log publica **visitas de texel** por isso — a soma das pegadas É o trabalho.

## 2. A lei: UM modelo, dois consumidores

O risco desta wave tem nome — [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]. Um
kernel de WGSL que re-derivasse a silhueta do dab seria uma **segunda resposta** a *"que forma tem
este dab?"*, divergindo no único lugar onde ninguém lê um número: uma screenshot.

**A cura é a do `ImpastoLightPass`** (*"o LUT especular SOBE pronto ⇒ o `powf`, o único transcendental
do modelo, nunca roda no device"*): a CPU avalia o falloff com a função que **já shipa** e manda a
TABELA; o device só amostra.

⚠️ **E isso é ESTRUTURAL, não disciplinar:** a crate `ph2d-paint-gpu` **não depende de
`ph2d-painter-brush`**. Ela não alcança o `falloff_weight`, logo não CONSEGUE ter opinião sobre a
lei. O que ela recebe é dado simples — uma tabela e uma lista de discos.

## 3. O que a GPU pega, e o que ela NÃO pega

O despacho já tem a forma certa: o ramo em banda só existe para o pincel de falloff nu
(`!textured && !shape_active && !accumulate_cap && image.is_none()`). A GPU **estreita mais**:

| entra | fica na CPU | porquê |
|---|---|---|
| falloff via LUT | Shape / Grain / imagem | o ramo em banda já os exclui |
| blend **Normal** | os outros 23 modos | 24 leis é uma tradução própria, e o caso quente é um |
| `preserve_alpha` | cap de Accumulate | o cap LÊ e ESCREVE máscara compartilhada (serial por semântica) |
| footprint deform | film AA (Smooth Edges) | o AA custa 9 amostras/texel e só existe com impasto |

**Todo caso fora da lista cai na rota em banda de hoje**, que é testada e fica. O modo de falha de um
caso novo é *lento*, nunca *errado*.

## 4. A aritmética que o WGSL tem de reproduzir AO BIT

Lida do produto, não inventada (`dab/bands.rs`):

```
dx = (px + 0.5) − cx           dy = (py + 0.5) − cy      // centro de pixel
wv = footprint.apply([dx/r, dy/r])      t = |wv|
w  = LUT[t]                                     // a tabela, nunca a lei
out_a = da·(1−m) + sa·m
mix(b,s) = (b·da·(1−m) + s·sa·m) / out_a        // lerp PREmultiplicado, saída straight
encode(v) = (clamp(v,0,1) · 255.0 + 0.5) as u8  // round-to-nearest
```

⚠️ **O risco de paridade é a CONTRAÇÃO FMA**, não a fórmula: o driver pode fundir `a·b + c`. A
política do repo já é essa e não é "bit-a-bit" (o compositor declara que runtime não é bit-idêntico
entre backends) — o template é o do `ImpastoLightPass`: **literais exatos no gate CPU-only + épsilon
documentado no gate `#[ignore]` contra o kernel canônico**, e ⚠️ **um limite de MAGNITUDE sozinho não
basta** (tirar o `+0.5` do `quantise` moveu 2375 bytes por UM nível e passava sob um limite de 2) ⇒
o gate conta TAMBÉM **quantos** bytes diferem.

## 5. As fatias

**S1 — o kernel + a paridade** (correção; **não precisa de relógio**).
Crate `ph2d-paint-gpu`: `StampPass::run(base, w, h, region, lut, dabs) -> Vec<u8>`. Gate: a MESMA
lista de dabs pela rota em banda da CPU e pela GPU, comparando *pior delta* **e** *quantos bytes
diferem*.

**S2 — a FRONTEIRA medida** (é ela que decide as outras — a Fase 1 do ADR-0146).
A v1 **não muda posse**: `canvas_rgba` continua autoritativo, e o passe sobe a REGIÃO da figura,
computa e lê de volta. Estimativa a conferir: bbox ~1,7 M px × 4 B = **6,8 MB por sentido**, ~1,4 ms
o par contra os 18,3 ms do carimbo. ⚠️ **Estimativa é minha; o número é do produto** — esta linha já
errou por aritmética três vezes nesta sessão.

**S3 — a fiação.** O tool não tem device (e não deve ter): ele publica o lote + a região, e o SHELL
roda o passe, no molde do `denoise_ml_with_progress` (callback puro, a ponte é do shell).

**S4 — só se o S2 disser que a fronteira dói:** a tela fica RESIDENTE no device durante o gesto e o
readback acontece uma vez, no commit. ⚠️ Isso torna o `canvas_rgba` **stale durante o arrasto** e
exige a porta `bring_home()` para os ~25 leitores — exatamente o padrão que o worker da sim do Wet
Paint já construiu nesta crate. **Não fazer antes de o S2 pedir.**

## 6. O que NÃO é a resposta

⛔ **Aumentar o espaçamento no preview.** Corta a redundância linearmente (10×→5× = metade do
trabalho) e **muda o LOOK** — a lei é produto-por-dab, e a §13.10 já mediu o que acontece quando se
mexe na acumulação (a lei do envelope matou o endurecimento e produziu CONTAS; reprovada na tela).
Fica registrado para ninguém a redescobrir como ideia nova.
