# 14 — A ordem de SUPERAR, e o que a medição respondeu antes da primeira linha

> **Ordem do dono, 2026-09-20:** *«faça tudo que for necessário para superar o objetivo. se for
> necessário reescrever o motor então faremos»* — precedida de *«faça uma pesquisa do que de melhor
> existe»* e de *«BATTLE for NEIGHBORVILLE é o alvo»*.

⚠️ **Este documento existe porque a ordem contém uma hipótese — *«reescrever o motor»* — e a §0.0
manda MEDIR antes de aceitar um limite ou um custo.** A medição respondeu, e a resposta não é a que
a ordem antecipava. Nada aqui foi afirmado de memória: cada linha traz o ficheiro onde se confere.

---

## §1 — ⛔⛔⛔ A resposta curta: NÃO é preciso reescrever o motor, e a evidência é estrutural

As leis do pipeline que sete waves construíram **não estão presas ao campo de distância**. São
crates-folha de **zero dependências**, cada uma com um gémeo em WGSL que devolve o próprio texto de
shader:

| crate | dependências | porta de shader |
|---|---|---|
| [`ph2d-material`](../../crates/ph2d-material/) (OpenPBR) | **nenhuma** | `src/wgsl.rs` |
| [`ph2d-style`](../../crates/ph2d-style/) (a camada de estilo) | **nenhuma** | `src/wgsl.rs` — `source()` |
| [`ph2d-bloom`](../../crates/ph2d-bloom/) (o brilho) | **nenhuma** | `src/wgsl.rs` |
| [`ph2d-view-transform`](../../crates/ph2d-view-transform/) (cor e exposição) | **nenhuma** | `src/wgsl.rs` — `SOURCE` |

⭐⭐⭐ **Uma lei que devolve o próprio código de shader serve QUALQUER geometria.** O
[`ph2d-field-gpu/src/paint.rs`](../../crates/ph2d-field-gpu/src/paint.rs) declara-o por escrito:
*«as três leis do pintor já atravessaram»*. Um consumidor novo — um rasterizador de triângulos —
inclui o mesmo `source()` e obtém a **mesma lei, com paridade por construção**, em vez de a
reimplementar.

⇒ *O que a ordem chama «reescrever o motor» é, medido, **ligar um consumidor novo às leis que já
existem**.* A reescrita seria a forma mais cara de ignorar a medição.

---

## §2 — ⛔⛔ O buraco REAL, que nenhum documento deste módulo nomeia

A frase que abriu esta obra é [`01`](01_o_alvo_decomposto.md): *«desejo isso para essa **game
engine**»*. Sete waves entregaram o pipeline — **dentro do modo Render do modelador de campos**.

Medido hoje, esta casa tem **TRÊS** motores de render, e o que sabe fazer PBR não é o que desenha o
jogo:

| motor | desenha | quem o usa | iluminação |
|---|---|---|---|
| [`ph2d-render`](../../crates/ph2d-render/) | **sprites 2D** | **o JOGO** (os componentes do TOP-20) | ⛔ nenhuma |
| [`ph2d-mesh-render`](../../crates/ph2d-mesh-render/) | malhas de triângulos | a escultura | ⛔ matcap + Blinn-Phong |
| [`ph2d-field-render`](../../crates/ph2d-field-render/) | campos de distância | o modelador | ✅ o pipeline inteiro |

⚠️ **A prova de que o jogo é 2D está no tipo**, não numa opinião:
[`RenderInstance`](../../crates/ph2d-render/src/sprite/instance.rs) tem `world_pos: [f32; 2]` e
`size: [f32; 2]`. E nenhuma crate de
[`ph2d-app-components`](../../crates/ph2d-app-components/) referencia `mesh_render` ou
`field_render`.

⇒ **O pipeline do alvo existe e não chega ao produto que o alvo nomeia.** Esse é o buraco, e ele é
de FIAÇÃO, não de motor.

---

## §3 — ⭐⭐⭐ A alavanca medida: as sondas declaram uma propriedade que o código não cumpre

O cabeçalho de [`probes.rs`](../../crates/ph2d-field-render/src/probes.rs) escreve a lei certa:

> *«as sondas são de MUNDO, logo só se refazem quando a cena ou a luz mudam, **nunca quando a
> câmera roda**»*

**O código faz o contrário**, e são três elos:

1. [`paint.rs:513`](../../crates/ph2d-field-gpu/src/paint.rs) — `let b_sondas = device.create_buffer(…)`,
   um buffer **novo a cada quadro**;
2. [`paint.rs:413`](../../crates/ph2d-field-gpu/src/paint.rs) — `let p_assa = (pintor.ao_rays > 0).then(…)`,
   o assar só corre com o ricochete;
3. [`gpu_frame.rs:317`](../../crates/ph2d-app-field3d/src/gpu_frame.rs) — `ao_rays: if assente && sonda.ricochete`,
   e o ricochete só corre no quadro **parado**.

⇒ **A luz indirecta é calculada e deitada fora, imagem após imagem, e não existe enquanto a mão
mexe.**

⚠️ *Isto não é um defeito de quem a construiu* — a wave que a trouxe tinha de provar a lei antes de
a tornar barata, e provou-a. É a dívida que sobra, e o cabeçalho dela já a nomeia.

### O que a indústria faz, medido nos cinco (2026-09-20)

| motor | como paga a luz indirecta |
|---|---|
| **Unreal** (Lumen) | traça contra *proxies* + *surface cache*; amortiza |
| **Godot** (SDFGI) | `frames_to_converge`, `frames_to_update_lights`, `gi/use_half_resolution`, TAA, FSR |
| **Unity** (HDRP) | volumes de sondas, SSGI, TAA, upscaling |
| **Blender** (EEVEE Next) | raios de ECRÃ + redutor espacial **e temporal**, TAA com reprojecção, `gi_diffuse_bounces = 3` |
| **Frostbite** (o ALVO) | ⭐ **Enlighten: transferência PRÉ-CALCULADA**, lightmaps assados por um traçador offline |
| **PH2D** | tudo, à resolução cheia, num quadro — **ou nada** |

⭐⭐⭐ **Nenhum deles calcula a luz toda em cada imagem, e o ALVO menos que todos.** O
[SIGGRAPH 2026](https://advances.realtimerendering.com/s2026/index.html) confirma a direcção: uma
*memória de radiância* (EA), *orçamento variável de raios por pixel* (Activision) e *upscaling por
aprendizagem* (Sony).

⇒ *O alvo do dono é alcançado com luz que PERSISTE, não com luz recalculada.* As sondas de mundo
**são** o nosso lightmap — só lhes falta sobreviver ao quadro.

---

## §4 — ⭐⭐ A vantagem que nenhum dos cinco tem, e porque ela é estrutural

- A **Unreal** traça contra um campo de distância **pré-cozido por malha**, com o erro disso.
- O **Godot** traça contra **voxels**.
- O **EEVEE** traça contra o **ECRÃ** — não vê o que está fora do enquadramento (limitação
  declarada no manual dele).
- O **Frostbite** usa transferência **pré-calculada**: a cena tem de ser estática.

⭐ **Nós traçamos contra a forma verdadeira**, exacta e analítica, e sobre geometria que o artista
está a mudar naquele instante. ⇒ *amortizada como eles amortizam, a nossa luz indirecta fica mais
correcta que a deles, porque a fonte não é uma aproximação.*

---

## §5 — ⛔ A decisão que é do DONO, e que muda a ordem das fases

A ordem *«superar»* tem duas leituras, e elas pedem obras diferentes. **Não decido isto:**

| leitura | o que significa | o que custa |
|---|---|---|
| **(A)** o **modo Render do modelador** deve bater os cinco em qualidade e ser utilizável em movimento | afinar e amortizar o que existe | as fases `F1`–`F2` abaixo |
| **(B)** o **PH2D deve poder FAZER um jogo** como o BfN | o jogo passa a ter um caminho 3D com PBR | `F1`–`F2` **e** `F3`–`F5` |

⚠️ **A frase original do dono diz *«game engine»*, e o TOP-20 de componentes que outra linha
constrói é de JOGO** — o que aponta para **(B)**. Mas o jogo é 2D hoje, e passá-lo a 3D é uma
decisão de PRODUTO, não de engenharia.

⭐ **As fases `F1` e `F2` servem as DUAS leituras**, e é por isso que começam já: nenhuma linha
delas é desperdiçada seja qual for a resposta.

---

## §6 — O plano, por ordem de dependência

### `F1` — ⭐⭐⭐ A luz sobrevive ao movimento *(serve as duas leituras)*

O que a `W9` deixou nomeado, agora com a causa identificada no §3.

1. **As sondas persistem entre quadros.** Buffer próprio, reassado quando a **cena** ou a **luz**
   mudam — nunca quando a câmera roda. *A lei já está escrita no cabeçalho delas; falta cumpri-la.*
2. **A oclusão a meia resolução.** ⭐ Já medida e **não construída**, com a nota escrita em
   [`occlusion.rs`](../../crates/ph2d-field-render/src/occlusion.rs): *«cabe em meia resolução com
   reconstrução guiada pela normal — `4×` mais barata, o que poria `96` cones abaixo do preço dos
   `16` de ontem»*. Traz uma classe de artefacto própria (halo na descontinuidade de profundidade),
   que é o que a wave tem de medir.
3. **Reprojecção temporal.** O quadro anterior é informação; hoje é deitado fora.

- **Régua:** a luz indirecta e a sombra **ligadas** com a câmera a mexer, dentro do orçamento de
  `16,7 ms` a `1920×1080`.
- **Kill-criterion (§5 da DIRETIVA, declarado ANTES do build):** se depois dos passos `1` e `2` o
  quadro de movimento com luz ligada não descer abaixo de `16,7 ms` nas cenas que hoje já são
  nítidas, **a `F1` não existe nesta forma** e o problema muda de classe (passa a ser de
  resolução/upscaling, não de amortização).
- ⚠️ **Antes de tocar em código:** re-medir a linha de partida com a máquina abaixo de `load 10`,
  em `--release`, com a ociosidade real ao lado — as duas armadilhas que o
  [handoff de 2026-09-20 §10.3](../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-20.md)
  já pagou.

### `F2` — O material deixa de ser uma cor *(serve as duas leituras)*

- **Texturas.** Hoje um objecto tem **uma cor só** — medido: a
  [`ph2d-material`](../../crates/ph2d-material/src/lib.rs) não tem uma única entrada de textura ou
  de UV. É a diferença mais visível numa comparação lado a lado com qualquer dos cinco.
- **Os campos que faltam do OpenPBR: `21` de `41`** (contados no `struct`). Ficam de fora
  `transmission_*` (vidro), `fuzz_*` (tecido), `thin_film_*` (iridescência), a anisotropia e
  `geometry_opacity`.

### `F3`–`F5` — só se a resposta ao §5 for **(B)**

3. **O jogo ganha um caminho 3D.** Um `RenderInstance` com pose 3D, e o
   [`ph2d-mesh-render`](../../crates/ph2d-mesh-render/) — que **já rasteriza triângulos com wgpu** —
   a servir a cena de jogo.
4. **Esse caminho inclui as leis do §1.** É aqui que se vê que não era preciso reescrever: o
   `mesh.wgsl` deixa de fazer matcap e passa a incluir o `ph2d_material::wgsl`.
5. **Sombras e animação esqueletal** na cena de jogo.

---

## §7 — ⛔ O que este plano NÃO faz, e porquê

- **Não persegue GI dinâmica a 60 Hz.** O alvo não a tem (§3): o Frostbite pré-calcula. Persegui-la
  seria gastar no sítio que o próprio alvo evitou.
- **Não constrói Nanite nem streaming.** [`02` §4](02_o_estado_da_arte.md) já os recusou por
  medição, e o alvo não os usa.
- **Não compra o jogo para medir, ainda.** A pergunta técnica que a compra responderia
  (*como é que o alvo ilumina?*) já está respondida — Enlighten, pré-calculado. O que a compra
  compraria é **calibração fina**, e calibrar antes da `F1` seria afinar um motor que ainda vai
  mudar de forma. ⚠️ Ele também corre mal em Linux (ProtonDB **prata**, a tender para bronze).

---

## ⛔ Recusas MEDIDAS desta página

| recusa | mecanismo | onde |
|---|---|---|
| **reescrever o motor** | as leis são crates-folha de zero dependências com gémeo em WGSL; falta um consumidor, não um motor | §1 |
| **levar o ray-march de SDF para o jogo** | o custo é `instruções × passos × píxeis`, e cada objecto acrescenta instruções que TODO pixel corre — `308` para `21` objectos | §2 |
| **GI dinâmica a 60 Hz como alvo** | o alvo (Frostbite/Enlighten) **pré-calcula**; a nossa persistência é o equivalente | §3 |
| **comprar o BfN agora** | a pergunta técnica já está respondida de graça; a calibração vem depois da `F1` | §7 |
