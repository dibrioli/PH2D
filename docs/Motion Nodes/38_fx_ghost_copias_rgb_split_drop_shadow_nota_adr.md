# 38 — M4 FX por-instância: `fx.rgb_split` + `fx.drop_shadow` (e por que `fx.mirror` foi CANCELADO)

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **M4 — FX por-instância**
**Status:** implementado, testado (5 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (`NodeManifest`/`NodeOp`/`OpResolver` provados **8/2/1** depois)
**Foundational tocado:** **nenhum** — 2 drop-crates + a demo + os testes do shell

---

## 0. `fx.mirror` — CANCELADO pela cerca de Chesterton

O plano (§3, ETAPA B) pedia **três** nós: `fx.mirror` · `fx.rgb_split` · `fx.drop_shadow`.

**`motion.mirror` JÁ EXISTE** e faz **exatamente** o que o `fx.mirror` faria: reflete cada elemento pelo
eixo através do **centroide** e **duplica** (`count → 2·count`, doc 25). Criar um segundo nó com o mesmo
comportamento e um prefixo diferente seria uma pegadinha pro artista ("qual dos dois?") e código morto.

**Cancelado.** É a mesma decisão que matou o `motion.distribute_poisson` (o `motion.scatter` já ratifica
Mitchell best-candidate). *A regra: antes de criar o nó que o plano pediu, procure o nó que já o faz.*

---

## 1. Os dois nós são **o mesmo primitivo**: cópias-fantasma

`rgb_split` e `drop_shadow` são, mecanicamente, a MESMA operação — **duplicar o stream, deslocar a cópia,
recolorir, e desenhá-la ATRÁS**. Por isso saíram na mesma fatia e compartilham o leaf `copies.rs`
(`positions` · `tints` · `falloff_at` · `tile`), copiado nas duas crates (leaf de 60 linhas > foundational
novo pra 2 consumidores — [[project_brush_along_path_satellite_not_node]]).

```text
out = [ fantasma(s), em BLOCO ] ++ [ os elementos, verbatim ]
          (atrás)                        (por cima)
```

**A ordem do stream É a ordem de desenho** (o `lower_to_instances` percorre as linhas em ordem; todo
instance de Motion tem `z_order = 0`). Precedente: o `motion.trail` já depende disso ("carried rows draw
FIRST so the live head paints on top").

**Bloco, não intercalado.** Todos os fantasmas antes de todos os elementos = a sombra **da camada** (o layer
style do Photoshop). Intercalar (sombra, elemento, sombra, elemento…) deixaria a sombra de um cair **em cima**
do vizinho — lê como sujeira, não como profundidade.

---

## 2. `fx.rgb_split` — aberração cromática

**Pesquisa.** Cavalry tem *RGB Split*; AE monta com *Shift Channels* + offset; Unity/Unreal/Godot têm
*Chromatic Aberration* no post-process. **São dois looks diferentes**, e o nó oferece os dois (`mode`):

| Modo | Deslocamento | O que é |
|---|---|---|
| **Split** | uniforme `(x, y)` | a estilização glitch/datamosh — os canais escorregam todos juntos |
| **Aberration** | **radial**, `strength × (P − centroide)` | a **física**: aberração cromática *lateral* é uma **magnificação dependente do comprimento de onda** → a franja é **zero no eixo óptico** e cresce **linearmente** com a distância dele. É o que o post-process de jogo realmente faz (amostra R/B em UVs escalados a partir do centro da tela) |

### Por que os TRÊS canais do shader viram DOIS fantasmas aqui

Um FX de passe separa a imagem **aditivamente** em R, G e B e os desliza; os três **somam** de volta pra
branco onde se sobrepõem. Este nó roda no **stream de instâncias**, e o renderer faz **alpha-blend**
(`premultiplied = 0`, *over* padrão): três cópias opacas empilhadas mostrariam **só a de cima**, não a soma
— o miolo sairia azul sólido em vez de intacto.

A forma honesta por-instância é o **par complementar ATRÁS do elemento intacto**:

```text
out = [ fantasma R (+off) ] ++ [ fantasma G+B (−off) ] ++ [ o elemento, verbatim ]
```

Num elemento opaco isso **reproduz exatamente** as bordas que o split aditivo produz — o lado `+` mostra R
com G,B faltando (franja **vermelha**) e o lado `−` mostra G,B sem R (franja **ciano**) — enquanto o corpo
fica na cor que o artista autorou.

### O insight que faz o nó ser correto e não um truque

**Isolar canal é MULTIPLICAR.** A coluna `tint` multiplica na cor do elemento — então mascará-la com
`[1,0,0,·]` **é** o canal vermelho *daquele elemento*, seja ele qual for. Um elemento azul **não joga franja
vermelha**, porque não tem vermelho pra jogar. Nada no nó assume branco. (O ingênuo — "pinta um fantasma de
vermelho e o outro de ciano" — dá franja vermelha a um corpo puro-azul, do nada. É o mutante #1, provado
vermelho.)

---

## 3. `fx.drop_shadow` — a sombra

**Referência: o layer effect do Photoshop / AE** (Angle, Distance, Opacity, Color) — e os **defaults são os
deles**: preto a **35 %**, jogado pra baixo-e-direita.

- **`Direction` é a direção pra onde a sombra CAI** (o nome e o sentido do AE). O Photoshop chama o mesmo
  dial de `Angle` e aponta pra **luz** — o vetor oposto. O label diz qual é, então não há o que decorar.
  Graus (a única unidade de ângulo autorada do app), ccw a partir de `+x`, no mundo **y-up** → o default
  **315°** cai pra baixo-e-direita.
- **Sombra é uma COR, não uma cópia escurecida.** O RGB vem do swatch; só o **alpha** é herdado
  (`swatch.a × elemento.a × falloff`) — elemento meio-transparente projeta sombra meio-transparente, e o
  `falloff` decide **quais** elementos projetam. (Uma cópia escurecida por `tint` carregaria a MATIZ do
  elemento e daria sombra **vermelha** a uma bola vermelha. Mutante #4, provado vermelho.)

### O que deliberadamente NÃO está aqui: o BLUR

`Size` (e o `Spread` que estrangula a matte borrada) são operações **raster** — pertencem ao **compositor
HDR** (`fx.blur`/`fx.glow`), que é decisão cross-module (o handoff manda **PARAR e reportar** antes de
encostar nisso). Então esta sombra é **hard-edged**, honestamente o que ela é: o look flat-design /
long-shadow. **Não** fabriquei maciez falsa com uma pilha de fantasmas.

**Nenhum dos dois nós toca a coluna `size`** — ver §6 (o achado do `motion.scale`).

---

## 4. Segurança do stream (os dois)

- **Teto duro** `MAX_INSTANCES = 65_536`. `3 × count` (ou `2 ×`) é uma alocação dirigida por um upstream
  **não-confiável**. Estourou → o FX **se desliga sozinho** (devolve a entrada **verbatim**), nunca
  meio-desenha: uma cena sem um terço das franjas lê como **bug**; uma cena sem aberração lê como "o efeito
  está desligado".
- **Param-lixo (NaN/∞)** conta como desligado — senão envenenaria o alpha de toda franja/sombra.
- **`id` é duplicado** (um fantasma compartilha a identidade da fonte), então os dois nós vão **DEPOIS** de
  qualquer coisa que pareia estado por id (`motion.integrate`, `motion.spring`) — convencionalmente logo
  antes do Output. Mesma regra do `motion.trail`, mesma nota no doc de cada crate.
- **HR-5:** tudo aritmética; o `drop_shadow` usa o leaf parabólico `trig.rs` (ciclos) — zero transcendental.

---

## 5. As guardas — 5 mutantes provados VERMELHOS

Verde-de-compilação vale zero. Cada guarda foi **falsificada de propósito** e voltou vermelha:

| # | Mutante | Guarda que pegou |
|---|---|---|
| 1 | fantasma R pintado de vermelho puro (`1.0`) em vez de mascarar o `tint` | `the_channels_are_isolated_out_of_the_elements_own_colour` |
| 2 | eixo radial = origem do mundo em vez do **centroide** | `aberration_is_zero_at_the_axis_and_grows_outward` |
| 3 | elementos ANTES das sombras (ordem de bloco invertida) | `the_shadows_are_one_block_behind_the_untouched_elements` |
| 4 | sombra = cópia escurecida em vez da cor do swatch | `the_shadow_is_a_colour_and_inherits_only_the_transparency` |
| 5 | **o FX ligado na entrada mas NÃO na saída** (nó pendurado na demo) | `every_element_casts_a_shadow_that_tracks_it` → **`left: 16, right: 32`** |

O #5 é **a** cicatriz da linha (a costura não-testada, DIRETIVA §1): o grafo **valida**, o nó **cozinha**, e
mesmo assim o efeito não chega à tela. Só um `assert_eq!` na CONTAGEM que sai do `motion.output` pega isso.

E um 6º: trocar o `mode` da demo pro uniforme derruba a guarda radial — o teste sabe a diferença entre os
dois looks, não só que "algo se moveu".

---

## 6. Achado incidental (NÃO é desta fatia — reportado, não corrigido)

**`motion.scale` no seu DEFAULT (`amount = 1.0`) não é a identidade.** Verificado lendo os três sites:

1. `motion.grid` **não emite** a coluna `size`.
2. `motion.scale` materializa `size = [1,1] × amount` quando a coluna está ausente (`lib.rs:66-68`).
3. O lowering usa a coluna quando ela existe, senão o `default_size` do shell — que é **`[0.4, 0.4]`**
   (`motion_state.rs:88`).

→ Soltar um `motion.scale` **no default** sobre um grid faz cada quad pular de `0.4` pra `1.0`: **2,5×
maior**, sem o artista ter mexido em nada.

A causa é semântica: `size` é **absoluta**, mas o fallback do lowering é `0.4` — então **qualquer** nó que
materialize `size` muda o tamanho renderizado. O conserto certo (unificar a unidade: fallback `[1,1]` e a
demo escalando explicitamente) muda o render de **todo** documento sem coluna `size` — é fatia própria, com
ADR. **Os nós desta fatia são imunes: nenhum dos dois toca `size`.**

---

## 7. Superfície nova (pro integrador)

| Item | Valor |
|---|---|
| Crates novas | `ph2d-node-fx-rgb-split` · `ph2d-node-fx-drop-shadow` (as 1ªs da família `fx.*`) |
| Node ids | `fx.rgb_split` · `fx.drop_shadow` — `NodeUiCategory::Fx`, display "RGB Split" / "Drop Shadow" |
| Codegen | `ph2d-node-registry-init` regenerado (`cargo run -p ph2d-node-sync`) — **73** crates-nó |
| Contrato | **intacto** (8/2/1) · **zero** foundational |

## 8. Aberto (M4)

`fx.*` de PASSE (`glow`/`bloom`/`blur` dual-Kawase/`vignette`/`levels`/`hue_shift`) = **compositor HDR**,
cross-module → **PARE e reporte ao Enio** (o handoff é explícito). Rig (`rig.*`) exige a decisão M4.N3.
E o `motion.scale` do §6.
