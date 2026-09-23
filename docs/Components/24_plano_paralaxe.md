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
a conta é `0/0`, logo é **lei escrita**, não aritmética; e `z − d ≤ 0` é a câmara a **atravessar** a
camada, que se recusa em voz alta.

> ⛔⛔ **CORRECÇÃO (2026-09-22, a W5 a implementar): a 1.ª redacção deste parágrafo dizia `escala ≡ 1`
> e está REFUTADA pela fórmula acima.** ⚠️ Duas grandezas partilhavam o nome: o tamanho **ABSOLUTO**
> de uma camada infinitamente longe de facto não muda (era esse o parêntesis, e ele é verdade), e a
> `escala` desta lei é **RELATIVA ao plano focal** — que CRESCEU. Medido em aritmética exacta, o
> limite é **`1 − δ`** (`k = 1/10` → `0,5263`; `1/10⁴` → `0,50003`), e a forma fechada
> `escala = (1 − δ)/(1 − k·δ)` **já o contém** — não há um braço `if k == 0`, e é por isso que ela
> não pode divergir do limite. Gate:
> `o_ceu_encolhe_relativamente_ao_plano_focal_e_o_plano_dizia_o_contrario`.
>
> ⭐⭐ **E o `z₀` DESAPARECEU, o que fecha o bloqueador §6.1 sem uma decisão:** `escala` depende só de
> `k` e de `d/z₀` ⇒ o dolly exprime-se em **fracções da distância focal** e não há número para
> medir. *Um parâmetro adimensional não tem um default para escolher.*

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

### W6 — a unificação com o HUD — ⛔ **as DUAS metades da premissa foram REFUTADAS (2026-09-22)**

> A redacção original: *«uma porta só para «a pose que a vista conduz», com dois leitores: o
> `UiCanvas` e o `ScrollFactor`. O que passa a ser partilhado é a LEI da pose. **Gate:** `UiCanvas` e
> um objecto com `k = 0` produzem a mesma pose, ao bit.»*

⛔ **Metade 1 — «a POSE» não é partilhável; a TRANSLAÇÃO é.** Medido: a translação dos dois é
**idêntica ao bit** e a escala **nunca** o é. A do canvas responde *«quantos metros de mundo cabem
nesta janela?»* (ela existe para o HUD ser legível em qualquer resolução, e o `Fit` escolhe entre
confinar e esticar); a da paralaxe responde *«a que PROFUNDIDADE está esta camada?»* (W5), e com
`dolly = 0` **não existe de todo**. *Duas grandezas com o mesmo nome e perguntas diferentes.*

⛔ **Metade 2 — não há LEI duplicada para unificar.** A translação do canvas é, em `ph2d_hud::place`,
literalmente **`translate: view.center`** — uma ATRIBUIÇÃO. A da paralaxe é `autorada + centro·(1−k)`.
Elas coincidem em `k = 0` com a pose autorada em zero, e isso é um **FACTO sobre as duas leis**, não
uma cópia de uma delas. ⇒ chamar a lei da paralaxe de dentro do canvas acrescentaria uma dependência
para exprimir `centro`: **cerimónia, e não unificação**.

⭐ **O que a wave entrega:** o **gate que ATA as duas** — que é o que a unificação ia comprar, e tudo
o que ela ia comprar. `crates/ph2d-app-components/src/parallax_w6_tests.rs`. ⚠️ E a fixtura dele NÃO
pode ter a vista do tamanho da referência: ali as duas escalas são `1,0` e coincidem por acidente.

### W7 — a superfície — ✅ **FECHADA (2026-09-23)**

Fileiras no Inspector (`Scroll Factor` em X/Y, com a **distância como leitura derivada**, nunca um
segundo campo guardado), i18n, e **duas cenas de smoke**: uma de fundo com três planos e repetição;
outra do dolly, que é a única que mostra o que nenhum outro motor faz.

**O que ficou, e as decisões que a medição tomou:**

- ⭐⭐ **UMA secção `Parallax` para QUATRO componentes.** Quatro populações, um assunto: a secção
  existe com o `ScrollFactor` e o ladrilho, a deriva e a cerca são BLOCOS dentro dela, cada um só
  com o componente dele. Quatro secções dariam quatro cabeçalhos a dizer a mesma palavra.
- ⭐⭐ **O painel diz porque nada se mexe, e são DUAS razões pela ordem da recusa:** *não há câmera
  do jogo* (a lei não corre para ninguém) antes de *esta camada anda com o mundo* (`k = 1`, o valor
  de FÁBRICA — o artista anexa a paralaxe e nada muda). ⚠️ **O neutro viaja no instantâneo** porque
  a crate do painel vive abaixo do `ph2d-ecs` no DAG: quem responde é o construtor, pela mesma porta
  (`ScrollFactor::e_neutro`) que o passe consulta.
- ⭐ **A fileira `Dolly` na secção `Camera`**, com a faixa a nomear o recurso de cada ponta: `0,9`
  em cima é o DOMÍNIO da lei (em `δ = 1` o plano do mundo tem tamanho aparente zero), `−1` em baixo
  é a saturação medida (o céu lê `1,79×` a `−1`, `2,42×` a `−2`, `2,94×` a `−3`).
- ⛔⛔ **E o gate do dolly apanhou um defeito PRÉ-EXISTENTE da secção Camera:** os números dela só se
  re-semeavam ao trocar de objecto, logo um `Ctrl+Z` deixava o valor velho no ecrã. Ela ganhou a
  ASSINATURA das irmãs (`sync_sections_camera_sig`).
- ⭐⭐ **As duas cenas** (`PH2D_PARALLAX_SMOKE=1|2`): a `=1` contrasta de propósito as duas leis — as
  árvores e o céu REPETEM, as colinas têm CERCA —, com os postes do chão como RÉGUA; a `=2` é o
  dolly sozinho, sem ninguém a andar. ⛔⛔ **A FOTO escolheu os números com os gates verdes:** a `7`
  e a `16` m a janela mostrava UMA árvore e UMA nuvem, e uma peça sozinha não se lê a andar mais
  devagar que outra.
- ⚠️ **O prólogo FECHA a régua do transporte** — a meia-vista da câmera é da JANELA, e com a régua
  aberta o céu e o chão saem do ecrã. Gate de texto, porque o prólogo não é alcançável de um teste.
- ⏳ **A distância como leitura derivada NÃO entrou** — ver o handoff §4.

---

## §5 — ⛔ Recusas MEDIDAS (não as reconstrua)

| recusa | porquê, com o número |
|---|---|
| **Os dois interruptores do alvo** (`follow_viewport` · `ignore_camera_scroll`) | medido: **4 combinações para 3 comportamentos**, com um estado duplicado e um sinal invertido atrás de nomes que não o dizem. Um `k` negativo e o `k = 0` exprimem o mesmo sem duas caixas |
| **O autoscroll do alvo** | não é observável (§4.6, com os dois controlos positivos) ⇒ portá-lo seria portar uma coisa que nem se pode gatear |
| **«Paralaxe nos eixos da VISTA»** | refutada pela física: um rolamento da câmara não produz paralaxe (todas as profundidades rodam por igual). Construí-lo seria construir um defeito |
| **Uma reescala nos limites** | não existe — era artefacto da minha régua (declive médio sobre uma curva com joelhos). Duas leis minhas caíram aqui |
| **Guardar a DISTÂNCIA ao lado da fracção** | são o mesmo número (§1). Dois campos que têm de concordar é o defeito que esta casa já pagou; a distância é uma leitura |

## §6 — O que ficava por medir antes de a W5 abrir — **as três FECHARAM (2026-09-22)**

1. ~~**O valor de `z₀`**~~ — ⭐⭐ **não existe número para medir.** A lei depende só de `k` e de
   `d/z₀`, logo o dolly é uma **fracção da distância focal** e o `z₀` desaparece do produto. *Um
   parâmetro adimensional não tem um default para escolher* (§2, com a correcção).
2. ~~**O custo por quadro**~~ — **MEDIDO** (`parallax_custo_tests.rs`, `--release`, tabela impressa
   por `custo_por_quadro_imprime_a_tabela`):

   | camadas | resto da cena | por quadro |
   |---:|---:|---:|
   | `1` | `0` | `0,59 µs` |
   | **`8`** | **`0`** | **`0,52 µs`** |
   | **`8`** | **`20 000`** | **`0,51 µs`** |
   | `100` | `0` | `3,59 µs` |
   | `1 000` | `0` | `41,0 µs` |
   | `10 000` | `0` | `758 µs` |

   ⭐ **O caso real (`3`–`8` camadas) custa `0,5 µs`** — `0,003 %` de um quadro de `16,7 ms` —, e
   `20 000` objectos SEM o componente não movem o número: o passe é `O(camadas)` e **cego à cena**.
   ⇒ **nenhum `MAX_*` é preciso**, e é a medição que o diz, não um palpite (§0.0).
3. ~~**A composição com o `Transform` de um PAI**~~ — **DECLARADA com gate**
   (`a_pose_conduzida_e_local_e_o_pai_compoe_por_cima`): a pose escrita é **LOCAL**, logo um pai
   rodado roda o deslocamento. ⛔ Desfazer o pai pediria a transformada de MUNDO, que é
   `O(profundidade)` por objecto e por quadro **e** só existe depois da propagação, que corre a
   seguir a esta fase. ⚠️ **No caso do artista (pai identidade) as duas saídas coincidem ao bit**, e
   é isso que torna a decisão barata.
