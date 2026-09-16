# Auditoria do vaso — porque o render dele custa `76 ms`, e onde está a alavanca

> **Gatilho (Enio, 2026-09-15):** *«o render do vaso deveria ser mais rápido»*.
> **Resposta:** ele tem razão, e a causa não é a marcha, nem a placa, nem o material.
> **`68` dos `76 ms` são a TESSELAÇÃO DAS QUINAS ARREDONDADAS do contorno que ele desenhou** — e o
> botão *Resolution* dele **não pode ajudar**, porque já está no nível mais grosseiro.

Sondas: `preview::device_probes::audita_o_vaso` e `::audita_o_arredondamento_do_vaso`
(`#[ignore]`, pedem adaptador e máquina calma; correm pela porta com `PH2D_GPU=1`).

---

## §1 — Onde o quadro do vaso é gasto

⚠️ **Um quadro de `76 ms` pode ser `76` de marcha ou `70` de montagem e `6` de marcha, e a tabela
das cenas reais não distingue os dois.** A régua que separa é uma **varredura de resolução**: o
custo por pixel escala com a área, o custo fixo não. Com dois pontos, `fixo = (c₁·a₂ − c₂·a₁)/(a₂ − a₁)`,
e o terceiro ponto é o **controlo** que diz se o modelo de duas parcelas descreve a curva.

| cena | fita | vivos | guardados | `480×270` | `960×540` | `1920×1080` | FIXO | marcha | controlo |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`5` · o VASO** | `2 969` | `33` | `2 504` | `12,96` | `27,46` | **`77,13 ms`** | `8,68 ms` | `68,45 ms` (**89 %**) | erro `6,1 %` |
| `4` · cantoneira | `2 896` | `69` | `2 663` | `16,73` | `36,12` | `112,78` | `10,33` | `102,45` (`91 %`) | erro `0,5 %` |
| `2` · cubo | `28` | `6` | `21` | `2,16` | `2,84` | `5,69` | `1,93` | `3,77` (`66 %`) | erro `1,0 %` |

⇒ **a marcha é `89 %` do quadro do vaso.** O custo fixo (montar a fita, escalonar, emitir o WGSL) é
`8,7 ms` e escala com o tamanho da fita — o cubo paga `1,9`. Não é ele a alavanca.

⚠️ **E o material, o céu e as lâmpadas não aparecem nesta conta** — o pintor lê um G-buffer que a
marcha já produziu. *A última wave pôs o modo RENDER na placa e o tecto continua a ser o traçado.*

## §2 — A marcha é cara porque a FITA é grande, e a fita é grande por um motivo só

O WGSL emitido para o vaso tem **`2 504 lets` e `93 887 bytes`**: `select×555`, `max×280`,
`min×186`, aritmética `×1 849`, e apenas **`sqrt×2`**. ⛔ *Não são as transcendentais* — é volume.

E o volume tem uma origem: **o contorno de `12` pontos que o artista desenha vira `94` arestas.**
Medido, apagando as quinas uma a uma (a mesma peça, `1920×1080`):

| quinas redondas | arestas | fita (ops) | guardados | quadro | ms/aresta |
|---:|---:|---:|---:|---:|---:|
| `0` (quinas vivas) | **`12`** | `335` | `284` | **`8,67 ms`** | `0,723` |
| `2` | `30` | `911` | `770` | `20,24` | `0,675` |
| `5` | `54` | `1 690` | `1 424` | `36,15` | `0,669` |
| **`10` (o vaso real)** | **`94`** | `2 969` | `2 504` | **`76,49`** | `0,814` |

⭐⭐⭐ **`68` dos `76 ms` são o arredondamento das quinas.** Cada quina redonda custa **~`8,2`
arestas** e **~`260` operações de fita**, e o custo é **linear**: `0,67`–`0,81 ms` por aresta ao
longo de toda a faixa. *Isto concorda com a lei que a W56 já tinha medido noutro instrumento —
`0,95 ns` por ponto por aresta, linear perfeito.*

## §3 — ⛔ Por que o botão do artista não resolve

`ph2d_field::DEFAULT_PROFILE_RESOLUTION = 1`, e `tolerance_ratio_for(level) = TOLERANCE_RATIO/level`
⇒ **o nível `1` é o mais GROSSEIRO que existe**. O vaso já está lá. Subir o *Resolution*
**acrescenta** arestas; não há para onde descer.

⇒ *o artista não tem nenhum gesto que torne o vaso dele mais rápido.* É isso que faz disto um
defeito de produto e não uma preferência.

## §4 — ⭐⭐⭐ A alavanca: o perfil não sabe o que é um ARCO

```rust
pub struct Profile {
    contours: Vec<Vec<[f32; 2]>>,   // ⬅ polilinha PURA
    fill: FillRule,
    tolerance: f32,
}
```

Não existe primitiva de arco. Um raio de quina é **tesselado** em ~`8` segmentos rectos, e cada
segmento é ~`32` operações na fita que a marcha avalia **em cada passo de cada pixel**.

⭐ **Um arco exacto custa ~uma aresta e meia e é MAIS preciso que oito.** A distância 2D a um arco é
`|‖p−c‖ − r|` recortada pela cunha angular — da mesma ordem de grandeza da distância ponto-segmento,
e **sem erro de tesselação nenhum**.

Estimativa, a partir das contagens medidas (`32` ops por aresta recta; um arco ~`25` ops):

| | primitivas | fita (ops) | quadro estimado |
|---|---:|---:|---:|
| hoje (tesselado) | `94` | `2 969` | `76,5 ms` |
| com arco exacto | `12` rectas + `10` arcos | ~`630` | **~`15`–`25 ms`** |

⇒ **`3×` a `5×` no vaso, e a silhueta fica melhor, não pior.** ⚠️ **O intervalo é honesto:** o custo
exacto de uma primitiva de arco só se mede depois de ela existir; o que está medido é o preço do que
ela substitui.

⚠️ E o ganho **não é só do vaso**: toda peça desenhada com quinas vivas — a cantoneira da cena `4`
(`102` arestas, `112,78 ms`) — paga a mesma tesselação.

## §5 — ⛔ O que foi verificado e NÃO é a resposta

| hipótese | porque cai |
|---|---|
| «é o material / o céu / as lâmpadas» | o pintor lê um G-buffer pronto; a marcha é `89 %` |
| «são as transcendentais» | `sqrt×2` no vaso inteiro |
| «é o custo fixo de montar a fita» | `8,7` de `76,5 ms`, e escala com a fita (o cubo paga `1,9`) |
| «é a ocupação / os registos» | `vivos = 33` desde o escalonador (§43); o cubo tem `6` e a `4` tem `69` |
| «o *Resolution* resolve» | já está no nível mais grosseiro (§3) |
| «a especialização por região já poda isto» | o custo é **linear** nas arestas de `12` a `94`; se a poda mordesse, a curva seria sublinear |

## §6 — ⏳ O que fica

- ⏳ **A wave é o arco exacto no `Profile`** — um `enum` de primitiva (`Seg`/`Arc`) em vez de
  `Vec<[f32;2]>`, com `sd_profile_inner` a emitir a distância certa para cada uma. ⚠️ Ela atravessa
  o **cozimento** (`cook_path` tem de PRESERVAR o arco do Live Corner em vez de o achatar), a
  **regra de preenchimento** (o winding de um arco não é o de um segmento) e o **`ProfileIndex`**.
  Nada disto é contrato congelado;
- ⏳ **O `select×555` não foi auditado** — são ~`5,9` por aresta, e a hipótese é o teste de winding
  do preenchimento. Se metade dele for redundante com o `min` da distância, é uma segunda alavanca
  **na mesma fita**, e essa não precisa de modelo novo;
- ⏳ **A especialização por região não está a morder neste caso** e ninguém mediu porquê. A
  suspeita é a simetria do torno: um ladrilho do ecrã mapeia para uma faixa larga de `(r, y)`.
