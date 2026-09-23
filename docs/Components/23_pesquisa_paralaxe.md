# A PARALAXE — pesquisa do estado da arte, medida

> Ordem do dono (2026-09-22): *«vamos criar um sistema equivalente ao `Parallax2D` · `ParallaxLayer`.
> Faça pesquisa de sistemas altamente competentes, capazes e de fácil uso. Me relate o que
> descobriu. Busque o estado da arte»*.
>
> ⛔ **Isto é pesquisa, não um plano.** As decisões de desenho estão nomeadas no §7 e são do dono.

---

## §1 — O achado que vem antes de tudo: **a lei já existe nesta casa**

A §5.0 manda medir se a composição já exprime o item antes de o construir. Medido:

```rust
// crates/ph2d-flip-render/src/camera.rs — o multiplano 2.5D do Flip (ADR-0114)
pub fn parallax_model(model: &Xform, cam_center: [f32; 2], depth: f32) -> Xform {
    if depth == 1.0 { return *model; }            // plano da câmara: byte-idêntico
    let k = 1.0 - depth as f64;
    // …  e + (cam_center[0] - e) * k,  f + (cam_center[1] - f) * k
}
```

⇒ **a camada desloca-se `(1 − depth)` por unidade de câmara**, com `depth = 1` a seguir o mundo e
`depth = 0` a ficar presa ao ecrã. Tem gate, tem prova de mutação, e o `depth` sobrevive ao ficheiro.

**E é EXACTAMENTE a lei da Godot.** Medido no binário instalado (4.7.2, MIT), pelo declive
Δ(origem da camada) ÷ Δ(câmara):

| `scroll_scale` | declive medido | |
|---|---|---|
| `0.00` | **1.0000** | a camada anda COM a câmara ⇒ presa ao ecrã |
| `0.25` | 0.7500 | |
| `0.50` | 0.5000 | ⚠️ o único ponto que **não** discrimina (`1 − 0.5 = 0.5`) |
| `1.00` | **0.0000** | não compensa nada ⇒ objecto normal do mundo |
| `2.00` | **−1.0000** | anda ao contrário ⇒ passa à frente |

⇒ `declive = 1 − scroll_scale`, e **`scroll_scale` ≡ o nosso `depth`**, número por número.

⚠️ **A minha primeira sonda escreveu «DIVERGE» em quatro destas linhas** porque comparava o declive
com o `scroll_scale` em vez de com `1 − scroll_scale`. *A régua estava certa e a expectativa é que
estava errada* — os pontos que decidem são o `0`, o `1` e o `2`; a `0.5` as duas leituras coincidem.

---

## §2 — O estado da arte CONVERGIU, e não é no motor: é no NÚMERO

Quatro sistemas independentes chegaram à mesma parametrização — **um número por camada, `1` = mundo,
`0` = preso ao ecrã**:

| sistema | onde vive o número | nome |
|---|---|---|
| **Godot 4** (MIT, medido) | um nó `Parallax2D` na cena | `scroll_scale` |
| **Phaser** (levantamento) | **propriedade de TODO objecto** | `setScrollFactor()` |
| **Construct 3 / GDevelop** (levantamento) | propriedade da **camada** | *Parallax %* |
| **PH2D / Flip** (medido) | campo da camada do Flip | `depth` |

⭐ **A conclusão que isto permite: a parametrização está resolvida e não é onde se ganha.** Quem
quiser inovar não inventa um número novo — inventa um sítio melhor para ele viver.

⛔ **E há motores grandes que não têm NADA:** Unity, Unreal (Paper2D), Defold e Cocos não trazem
paralaxe — toda a gente escreve o mesmo script. *A ausência num motor de 20 anos é informação: isto
é barato de fazer e caro de fazer BEM.*

---

## §3 — A Godot já trocou o sistema dela uma vez, e o que ela trocou não foi o número

Os dois sistemas coexistem no binário, e a diferença é **estrutural** (medida no `ClassDB`):

| | vive sob | família |
|---|---|---|
| `ParallaxBackground` + `ParallaxLayer` (o antigo) | **`CanvasLayer`** | espaço de ECRÃ |
| `Parallax2D` (o novo) | **`Node2D`** | espaço do MUNDO |

E na mesma régua (`docs/Components/ferramentas/godot_parallax_velho_probe.gd`):

| valor | ANTIGO `motion_scale` → declive | NOVO `scroll_scale` → declive |
|---|---|---|
| `0.0` | 0.0000 | 1.0000 |
| `0.5` | −0.5000 | 0.5000 |
| `1.0` | −1.0000 | 0.0000 |
| `2.0` | −2.0000 | −1.0000 |

Os sinais e os valores são diferentes **porque os espaços são diferentes** — e o efeito VISTO pelo
artista é o mesmo com o mesmo número (`0` = preso, `1` = mundo).

⭐⭐ **A lição inteira está aqui: eles não mudaram o que o artista escreve; mudaram ONDE o nó vive.**
Num `CanvasLayer` a camada não pode ser filha de nada do mundo, não herda transformada de um pai e
não compõe com o zoom; num `Node2D` compõe com tudo. *O valor de um sistema de paralaxe é a
composição, não a fórmula.*

---

## §4 — O que separa «funciona» de «CAPAZ» — sete coisas, medidas

### 4.1 ✅ Repetição infinita, e o mecanismo que a torna exacta

Com `repeat_size = 256` e a câmara a varrer `0 → 768` (`scroll_scale 0.5`):

```
cam.x 0    origem.x −436.0
cam.x 128  origem.x −372.0   salto  64.0     ← 0.5 × 128 ✓
cam.x 256  origem.x −308.0   salto  64.0
cam.x 384  origem.x   12.0   salto 320.0     ← 64 + 256 = UM ladrilho inteiro
cam.x 512  origem.x   76.0   salto  64.0
cam.x 768  origem.x  204.0   salto  64.0
```

⭐ **A camada teleporta-se por um múltiplo EXACTO do ladrilho.** É isso que faz a costura não poder
abrir: o salto é invisível porque a imagem a seguir ao salto é a mesma, e o erro de vírgula
flutuante **não acumula** porque nunca se soma um resto.

### 4.2 ✅ Eixos separados

`scroll_scale = (0.2, 0.8)` → declive `x = 0.8` com a câmara em X, `y = 0.2` com a câmara em Y.
(Nuvens que correm em X e mal se mexem em Y.)

### 4.3 ✅ Zoom — o declive em MUNDO não muda

`zoom 1.0 / 2.0 / 0.5` → declive `0.5000` nos três. O deslocamento é do mundo, e a vista trata-o
como trata tudo o resto. *É a resposta certa, e é a que um sistema escrito em espaço de ecrã erra.*

### 4.4 ⚠️ Rotação da câmara — ela IGNORA

`rot 0 / 0.5 / 1.57 rad` → declive `x = 0.5`, `y = 0` nos três. A paralaxe corre nos **eixos do
mundo**, não nos da vista. Com a câmara rodada, «para o lado» no ecrã deixa de ser o eixo em que a
camada fica para trás. É uma **limitação declarada** do alvo, e uma porta aberta para nós.

### 4.5 ⛔ Limites — MEDIDOS e NÃO EXPLICADOS

Com a região `−400..400` e `scroll_scale 0.5`:

```
cam.x −1200  origem.x −1360      declive 1.00 (fora da região)
cam.x  −400  origem.x  −560
cam.x     0  origem.x  −180      declive 0.95 (DENTRO — e não os 0.50 autorados!)
cam.x   400  origem.x   200
cam.x  1200  origem.x  1000      declive 1.00 (fora)
```

Fora da região a camada passa a andar 1:1 com a câmara (congela no ecrã); **dentro dela o declive
lido é `0.95`, não o `0.5` que foi escrito.** Não tenho mecanismo para isto. ⇒ **fica NOMEADO como
por medir**; copiar o comportamento sem o entender seria copiar um defeito.

### 4.6 ⛔ Autoscroll — INCONCLUSIVO, e é o controlo que o diz

`autoscroll = 60 px/s`, câmara parada, 30 quadros: **andou `0.000`**. E o **controlo** (`autoscroll =
0`) andou **`0.000` também** ⇒ a sonda não distingue *«não funciona»* de *«não observo»* (o efeito
pode viver num deslocamento interno que não passa pela transformada do nó, ou não correr sem
desenho). *Uma célula sem controlo positivo não é um resultado.*

### 4.7 ⚠️ Os dois interruptores — 4 combinações, **3** comportamentos

| `follow_viewport` | `ignore_camera_scroll` | declive |
|---|---|---|
| `true` | `false` | **0.5** — a paralaxe normal |
| `false` | `false` | **−0.5** — inverte (a camada vai contra a câmara) |
| `true` | `true` | **0.0** — ignora a câmara (para se rolar à mão) |
| `false` | `true` | **0.0** — *o mesmo que o anterior* |

⇒ **um estado duplicado e um sinal invertido** atrás de duas caixas cujos nomes não dizem isso.
É aqui que o alvo falha o *«fácil de usar»* — e é barato ganhar-lhe.

---

## §5 — O OUTRO modelo: a câmara multiplano (a ponta «capaz»)

O modelo acima é *o artista escreve a fracção*. Existe um segundo, mais antigo e mais forte:
**a camada tem uma DISTÂNCIA, e a fracção e a ESCALA derivam-se dela** — a multiplano da Disney,
que o **OpenToonz** (⭐ **BSD-3, porta aberta**) implementa como profundidade de coluna.

A diferença que decide: com uma fracção, aproximar a câmara de um fundo dá paralaxe mas **não** o
faz crescer; com uma distância, o fundo cresce sozinho e na proporção certa. Um é um efeito de
scroll; o outro é uma câmara.

⚠️ **Não medido:** o OpenToonz **não tem porta de consola** (arsenal §, medido: `--help` abre a GUI
e bloqueia). O fonte é permissivo e pode ser lido e portado — mas não está nesta máquina, e nada
neste documento sobre ele é medição.

⭐ E o nosso `depth` do Flip **já se chama profundidade** e já está ancorado na origem do objecto —
ou seja, a casa já escolheu o vocabulário do modelo forte sem ainda ter a segunda metade (a escala).

---

## §6 — A vantagem que só nós temos: **o HUD já é a paralaxe no `0`**

O `UiCanvas` é conduzido para acompanhar a vista da câmara (`Driver::CanvasPose`) — ou seja,
**um HUD é exactamente uma camada com fracção `0`**. E a câmara publica a vista FINAL:

```rust
pub struct CameraView { pub center: [f32; 2], pub height_world: f32, pub cull_mask: u32 }
```

⚠️ **Isto é load-bearing:** a nossa câmara tem suavização e zona morta, logo a paralaxe tem de ler
o **centro publicado**, nunca o alvo perseguido — senão as camadas tremem contra o mundo em todo
quadro em que a suavização ainda não chegou.

⇒ se o número viver num **componente de qualquer objecto** (o modelo do Phaser), então
`UiCanvas`, paralaxe e objecto normal passam a ser **uma lei com três valores**, em vez de três
sistemas. É a composição que a Godot foi buscar quando trocou o `CanvasLayer` pelo `Node2D`, levada
um passo à frente.

---

## §7 — As perguntas que são DECISÃO DO DONO

1. **Onde vive o número** — componente em qualquer objecto (Phaser, compõe com tudo, subsume o HUD)
   · propriedade de uma camada (Construct, mais simples de autorar um fundo) · as duas.
2. **Fracção ou distância** (§5) — e se for distância, a escala deriva ou não.
3. **Rotação da câmara** (§4.4) — seguir o alvo (eixos do mundo) ou superar (eixos da vista).
4. **A repetição** é do mesmo componente ou de um irmão.
5. Os **limites** (§4.5) ficam por medir antes de qualquer decisão.

---

## Instrumentos (versionados, reprodutíveis)

- [`ferramentas/godot_parallax_probe.gd`](ferramentas/godot_parallax_probe.gd) — a lei, zoom,
  rotação, eixos, repetição, autoscroll, limites, interruptores (por DECLIVE, com controlo)
- [`ferramentas/godot_parallax_velho_probe.gd`](ferramentas/godot_parallax_velho_probe.gd) — o
  sistema antigo na mesma régua
- corrida: `godot --headless --quit-after 900 --script <sonda> -- <saída>`
  ⚠️ **não passe `--quit`**: ele mata no 1.º quadro e a sonda espera dezenas
