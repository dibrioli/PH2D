# PLANO — a paralaxe, no melhor que a matemática dá

> Ordem do dono (2026-09-22): *«avalie o melhor possível independente do custo. busque o estado da
> arte e planeje»* — ou seja §0.6 (a melhor opção técnica vence o custo de construção) e §0.0 (o
> alvo é o extraordinário). A pesquisa medida está em [`23_pesquisa_paralaxe.md`](23_pesquisa_paralaxe.md).

---

## §1 — A avaliação: os dois modelos são O MESMO NÚMERO, e é isso que decide tudo

O estado da arte tem duas famílias, e toda a gente as trata como alternativas:

| | o artista escreve | quem o faz |
|---|---|---|
| **A · fracção de scroll** | *«esta camada anda a 40 %»* | Godot · Phaser · Construct · o nosso Flip |
| **B · câmara multiplano** | *«esta camada está a 3 metros»* | Disney · OpenToonz · After Effects · Cavalry |

**Elas não são alternativas. A segunda é a primeira mais uma linha.** Câmara pinhole, distância
focal `z₀` (o plano onde o artista desenha, que mapeia 1:1), camada a `z`:

```
projecção          u  = f·x / z
a câmara anda Δ    Δu = −f·Δ / z          no plano focal:  Δu₀ = −f·Δ / z₀
                   ──────────────────────────────────────────────────────
paralaxe           Δu / Δu₀   =  z₀ / z   ≡  k
tamanho aparente   (f·s/z) / (f·s/z₀)  =  z₀ / z   ≡  k     ← O MESMO NÚMERO
```

⭐⭐⭐ **A fracção de paralaxe e o factor de escala são a mesma grandeza `k = z₀/z`.** Logo:

- `k = 1` → objecto normal do mundo (está no plano focal)
- `k = 0` → infinitamente longe → **preso à vista** (é o HUD)
- `0 < k < 1` → fundo
- `k > 1` → primeiro plano, passa à frente

⇒ **não se escolhe entre A e B.** Guarda-se **um** número `k`; a distância é uma LEITURA dele
(`z = z₀/k`), e o modelo B é o que se ganha quando a câmara puder andar em profundidade.

## §2 — E o que NINGUÉM tem: o **DOLLY**

Toda câmara 2D tem **zoom** (mudar `f`) — que amplia tudo por igual. **Nenhum motor 2D tem
dolly** (andar em profundidade), que é o que faz o primeiro plano crescer mais que o fundo — a
razão pela qual a Disney construiu a câmara multiplano em 1937.

Com `k` guardado, o dolly `d` cai de graça:

```
z = z₀ / k₀                                (a distância implícita no número autorado)
k(d) = (z₀ − d) / (z − d)                  a fracção passa a depender do dolly
escala(d) = k(d) / k₀                      ⭐ e a escala é a MESMA razão
```

⚠️ **Com `d = 0` isto degenera EXACTAMENTE no modelo de hoje** — `k(0) = k₀` e `escala = 1`, sem um
bit de diferença. É a propriedade que esta casa exige de toda lei nova: *a omissão é byte-idêntica*.

⛔ **Degenerescências que têm de ser nomeadas, não descobertas:** `k₀ = 0` é `z = ∞` ⇒ `k(d) = 0` e
`escala ≡ 1` para todo `d` (o que está infinitamente longe nunca muda de tamanho — e a conta é
`0/0`, logo é **lei escrita**, não aritmética); e `z − d ≤ 0` é a câmara a **atravessar** a camada,
que se recusa em voz alta.

## §3 — Onde o número vive: **num componente de qualquer objecto**

| opção | veredito |
|---|---|
| um nó dedicado (`Parallax2D`) | ⛔ não é o nosso modelo, e a própria Godot já fugiu de um sítio pior por este motivo |
| propriedade de uma CAMADA (Construct) | ⛔ obriga a inventar «camadas» como coisa nova |
| **componente em QUALQUER objecto** (Phaser) | ⭐ compõe com sprite, vector, Flip, partículas e HUD |

⭐⭐ **E há uma razão que só existe nesta casa: o `UiCanvas` JÁ É `k = 0`.** Ele é conduzido para
acompanhar a vista (`Driver::CanvasPose`). Com o número num componente, **HUD, paralaxe e objecto
normal passam a ser uma lei com três valores** em vez de três sistemas. É o passo a seguir ao que a
Godot deu quando trocou o `CanvasLayer` pelo `Node2D`.

⚠️ **A vista lida é a PUBLICADA, nunca o alvo** (`CameraView.center`): a nossa câmara tem suavização
e zona morta, e ler o alvo faria as camadas tremerem contra o mundo em todo quadro em que a
suavização ainda não chegou.

⚠️ **E a pose escrita é CONDUZIDA** (`Driver::ParallaxPose`, como o `CanvasPose`), senão cada quadro
de um pan vira um passo de `Ctrl+Z`.

---

## §4 — O plano, em waves

Cada wave acaba num smoke e é byte-idêntica no ponto neutro.

### W1 — o número (`ScrollFactor`)

Componente em qualquer objecto: `k: [f32; 2]` (por eixo — a lei do alvo, medida, e o caso das nuvens
que correm em X e mal sobem). Lê `CameraView.center`, escreve a pose pelo ledger.

- ⛔ **`k = 1` não escreve nada** (nem entra no ledger) ⇒ uma cena sem paralaxe é byte-idêntica.
- **Gates:** a tabela do declive (`0 / 0.25 / 0.5 / 1 / 2 → 1 − k`, com os pontos que **discriminam**
  — o `0.5` não discrimina) · eixos separados · invariância ao zoom · **a rotação** (o deslocamento é
  `Δmundo·(1−k)` e não depende do ângulo — §4.4 da pesquisa) · o ledger não deixa passo de undo.
- **Schema:** `PROJECT_SCHEMA +1`, registo do `ph2d-ecs` `+1` e os **dois espelhos** `+1`.

### W2 — a repetição infinita (`ScrollRepeat`)

Porte da lei medida no alvo (MIT, com atribuição): **teleporta-se por um ladrilho INTEIRO**, nunca
por um resto — é isso que impede a costura de abrir e o erro de acumular.

- ⭐ **Onde superamos:** o alvo obriga a escrever `repeat_size` à mão. O nosso **deriva do conteúdo**
  (uma sprite sabe a largura dela; uma forma sabe a caixa) e o campo é só um *override*.
- **Gate:** varrer 10 000 unidades e exigir que a origem caia **exactamente** na mesma fase (o erro
  de `f32` não pode aparecer no décimo milésimo ladrilho).

### W3 — o confinamento (`ScrollLimits`)

A lei medida, com os dois joelhos: enquanto a **vista** couber na região, a paralaxe autorada; quando
a borda da vista alcança a borda da região, a camada **congela no ecrã**.

- ⭐ **Onde superamos:** a região **deriva da caixa do conteúdo** — uma caixa *«o fundo nunca mostra a
  borda»* em vez de quatro números.
- **Gate:** reproduzir a curva medida, **com os joelhos** em `(região − ecrã)/2`. ⚠️ Medir o declive
  MÉDIO aqui é o erro que refutou duas leis minhas: o gate amostra os pedaços, não a média.

### W4 — o movimento próprio, que **é função do RELÓGIO**

Nuvens que andam sozinhas. ⭐ **É aqui que ganhamos por desenho, não por afinação:** o do alvo não é
observável por nenhum dos quatro observáveis nem por um teste (§4.6 da pesquisa), porque vive no
caminho de desenho. O nosso é `offset = velocidade × playhead`:

- **puro** ⇒ sobrevive ao scrub e ao rebobinar sem uma linha de estado;
- **medível** ⇒ tem gate;
- **determinista** ⇒ entra no replay.

### W5 — a multiplano: a **escala** e o **dolly** (§2)

`GameCamera` ganha `focal_distance` (`z₀`) e `dolly` (`d`). A escala deriva (`k(d)/k₀`) e é
conduzida, como a pose.

- ⛔ **Com `dolly = 0` a saída é byte-idêntica à W1** — e há gate a exigi-lo.
- **Schema:** `PROJECT_SCHEMA +1` (campos novos no `GameCamera`).
- **Gate:** o teste da multiplano — dois planos a `k = 1` e `k = 0.25`, um dolly, e a razão dos
  tamanhos aparentes bate a fórmula ao `f32`; mais as duas degenerescências do §2.

### W6 — a unificação com o HUD

Uma porta só para *«a pose que a vista conduz»*, com **dois leitores**: o `UiCanvas` e o
`ScrollFactor`. O `UiCanvas` mantém o que é dele (o enquadramento, as âncoras, o letterbox); o que
passa a ser partilhado é a LEI da pose.

- **Gate:** `UiCanvas` e um objecto com `k = 0` produzem a **mesma** pose, ao bit.

### W7 — a superfície

Fileiras no Inspector (`Scroll Factor` em X/Y, com a **distância como leitura derivada**, nunca um
segundo campo guardado), i18n, e **duas cenas de smoke**: uma de fundo com três planos e repetição;
outra do dolly, que é a única que mostra o que nenhum outro motor faz.

---

## §5 — ⛔ Recusas MEDIDAS (não as reconstrua)

| recusa | porquê, com o número |
|---|---|
| **Os dois interruptores do alvo** (`follow_viewport` · `ignore_camera_scroll`) | medido: **4 combinações para 3 comportamentos**, com um estado duplicado e um sinal invertido atrás de nomes que não o dizem. Um `k` negativo e o `k = 0` exprimem o mesmo sem duas caixas |
| **O autoscroll do alvo** | não é observável (§4.6, com os dois controlos positivos) ⇒ portá-lo seria portar uma coisa que nem se pode gatear |
| **«Paralaxe nos eixos da VISTA»** | refutada pela física: um rolamento da câmara não produz paralaxe (todas as profundidades rodam por igual). Construí-lo seria construir um defeito |
| **Uma reescala nos limites** | não existe — era artefacto da minha régua (declive médio sobre uma curva com joelhos). Duas leis minhas caíram aqui |
| **Guardar a DISTÂNCIA ao lado da fracção** | são o mesmo número (§1). Dois campos que têm de concordar é o defeito que esta casa já pagou; a distância é uma leitura |

## §6 — O que fica por medir antes de a W5 abrir

1. **O valor de `z₀`** — hoje a nossa câmara tem `height_world` (um zoom), não uma distância focal.
   O default tem de sair de uma medição de enquadramento, não de um número escolhido (§0.0).
2. **O custo por quadro** com N camadas — o passe é `O(objectos com o componente)` e não foi medido.
3. **A composição com o `Transform` de um PAI** — uma camada de paralaxe filha de outro objecto: a
   ordem em que a pose conduzida e a hierarquia se aplicam tem de ser decidida com um gate.
