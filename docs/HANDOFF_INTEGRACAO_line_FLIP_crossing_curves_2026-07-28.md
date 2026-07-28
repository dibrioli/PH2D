# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: o cruzamento e a curva (2026-07-28)

> **Missão do Enio** (com 5 screenshots): *"Nosso Flip desenha horrivelmente seus traços. Cria
> horrivelmente seus vértices. Nosso traço com hardness < 1 — veja onde se cruzam. Muito ruim."*
> Mais: ir ao fonte do Blender 5.2 e apps GP-like, e estudar os 13 pincéis para trazê-los.
>
> **5 commits, todos medidos, todos com mutação.** Estado: `line/FLIP`, tip `c0589697a`.

---

## 1. O que estava errado, e o que a medição disse

A pesquisa paralela (5 agentes: fonte do GP 5.2 · os 13 pincéis · apps GP-like · o estado da
arte fora do Blender · auditoria do nosso código) produziu **duas** causas, e nenhuma delas era
a que o comentário do nosso próprio shader apontava.

### ⛔ A hipótese que o shader me vendeu, e que a medição REFUTOU

O cabeçalho do `flip.wgsl` declarava *"teto conhecido: sobreposição com vizinhos i±2 e
auto-cruzamento NÃO-adjacente seguem first-wins (semântica do GP, **pinada em teste**)"*. Ela
mandou a investigação direto para "o cruzamento volta ao first-wins".

**As três partes eram falsas.** A união alcança i±k E o auto-cruzamento não-adjacente (é para
isso que existe `seg_extras`, o 4º binding do próprio shader), e **o teste citado nunca foi
escrito**. Medido: um laço macio que se auto-cruza sai com **0 pixels divergentes de 57.699**
contra a união analítica — a união já resolvia aquilo.

*Um comentário que contradiz o código shipado é pior que comentário nenhum.* Corrigido no
commit `2c953cff4`.

### ✅ A causa real nº 1 — a seleção de vizinhos era ESTRUTURALMENTE incapaz

```
alcance / passo  =  3r / 0,8r  =  3,75      ⇒  i±1 … i±4 SEMPRE na lista
```

As duas grandezas são proporcionais ao raio, então a razão é **constante** — o pincel some da
conta. Os ~6 vizinhos da própria fita entram **antes de existir cruzamento**, e — ordenados por
distância, que neles é ~0 — **ganham os 16 slots**. O segmento que de fato cruza cai fora,
aquele pixel volta ao first-wins, e a GPU pinta a cauda macia de uma passagem sobre o **núcleo**
de outra. É a foto do Enio.

⚠️ **A reamostragem que shipou em 27/07 criou essa razão.** Não é bug antigo: é a interação de
duas waves.

**Cura:** partição com **dois orçamentos**, e a definição se escreve sozinha — *vizinho da mesma
passagem = alcançável andando pela polilinha dentro do alcance*. É comprimento de **arco**, não
um `k` escolhido. (`5396f3ae5`)

### ✅ A causa real nº 2 — a parametrização da reamostragem cuspidava

`centered_tangent` era Catmull-Rom **uniforme**, com um comentário afirmando que *"uniforme
basta porque as quinas são tratadas à parte"*. O regime perigoso não é o giro **agudo**: é o
**espaçamento desigual com giro gentil** — e espaçamento desigual é o que o **RDP produz**. O
par RDP→reamostragem **fabricava** o caso ruim. (`4652c980b`)

---

## 2. Os números (todos de sonda, todos reprodutíveis)

| Wave | Antes | Depois |
|---|---|---|
| **W1a/b** hachura densificada vs união | **FAILED, desvio 178** (pintou 31, pede 209) | ok |
| **W1d** tinta do grampo sub-pixel `r=0,20` | `0,170` do esperado `0,352` | `0,352` |
| **W1d** idem `r=0,45` | `0,813` de `0,973` | `0,973` |
| **W2a** giro máx. da saída, espaçamento 50:1 | **158,5°** (cúspide) | **29,6°** |
| **W2a** desvio à polilinha, 50:1 | **5,55** (span de corda 2,83) | 1,51 |
| **W2a** inflação do arco, 50:1 | 1,081 | **1,002** |
| **W2b** pontos mudos numa reta | 3 entram, **49 saem, 46 mudos** | **0** (saem 3) |
| **W2b** arco de 90° (controle) | 33 | **33, intacto** |
| **self_overlap** erro vs lei de tinta | **43/255 (17%)** | ⛔ **ABERTO** |

---

## 3. Commits

| # | O quê |
|---|---|
| `5396f3ae5` | **W1a+b** — a fita local ganha orçamento PRÓPRIO (arco); cap **16**, MEDIDO |
| `89f928239` | **W1d** — o piso de largura mínima chega ao laço de extras |
| `4652c980b` | **W2a** — Catmull-Rom **CENTRÍPETA** (α=½) |
| `2c953cff4` | **docs** — o cabeçalho do shader para de mentir; o `self_overlap` ganha número |
| `c0589697a` | **W2b** — span já reto ganha ZERO pontos |

---

## 4. Superfície tocada

- **Nenhum schema** (`PROJECT_SCHEMA` 37 · `FLIP_SCHEMA` 12 — intocados).
- **Nenhum contrato congelado** (§6): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/
  `PanelEvent=4` intactos, conferido por grep.
- **Nenhum ADR novo, nenhuma crate nova, nenhuma dep nova, nenhum `Cargo.toml` tocado.**
- **Nenhum id/token/i18n.**
- ⚠️ **Uma assinatura interna mudou:** `flip_smooth::resample_smooth` 3→4 args (`flat_tol`).
  `pub(crate)`, **um único chamador** (`flip_draw::stroke_from_samples`).
- **LOC:** `flip_smooth.rs` cruzaria 605 > 600 ⇒ `mod resample_measurement` foi para o irmão
  `flip_smooth_resample_tests.rs` por `#[path]` (segue FILHO, `use super::*` alcança privados).

### Mudanças de comportamento (o smoke decide)

1. **Toda curva reamostrada muda de forma** onde o espaçamento é desigual — é o fim da cúspide,
   mas é mudança visível. Espaçamento par: matematicamente idêntico (não *bit*-idêntico).
2. **Retas saem com menos vértices.** Subtrativo por construção (a tolerância é a do próprio RDP).
3. **Cruzamentos de pincel macio pintam certo** — o alvo da missão.

---

## 5. ⚠️ ABERTO, com o preço medido

### W1c — `self_overlap` conta DUAS vezes (**43/255, 17%**)

Cada face que passa o depth computa a **união GLOBAL**, então as `N` faces sobrepostas compõem
`1−(1−u)^N` com o MESMO `u` — a cobertura da passagem mais **próxima**, creditada `N` vezes.
`N` é o número de **QUADS**, não de **PASSAGENS**. A assinatura é exata: todo pixel sai em
`1−(1−OFF)²`, inclusive onde a 2ª passagem está a `dn = 0,82` e deveria contribuir ~nada.

⚠️ **O default é `self_overlap: false`** ⇒ isto **não** é o artefato que o Enio fotografou.

**Desenho da cura** (não construído): a lista de extras **particionada por passagem** — a fita
local já é separada desde o orçamento próprio; falta agrupar os extras de GRADE em runs de
índice consecutivo (marcador no bit alto de `seg_extras[].x`) e expor o `ribbon_count` — mais
**depth de volta a por-TRAÇO**, o que mata junto a colisão de `f32` do degrau por-segmento em
`sid` alto. Sonda pronta: `measure_the_self_overlap_double_count`.

### W3 — joins & caps (miter/bevel/round + butt/round/square)

Avaliado e deixado para **wave dedicada**: mexe na cobertura-união, a joia que custou uma semana
de bugs. O desenho que a pesquisa achou: **três tipos de quina = três métricas de distância** no
fragment (é o que o GP 5.2 faz), e o nosso `min` de cápsulas já é a estrutura certa para hospedá-las.

### W4 — os 13 pincéis (**a metade da missão que NÃO foi construída**)

A pesquisa está feita e não deve ser refeita. Achados que decidem o desenho:

- Os 13 são **assets** no 5.2, re-autorados à mão — **versionar não traduz**.
- `unprojected_radius` é o autoritativo (não `size`).
- **Marker Chisel não é uma elipse:** é pressão modulada pelo ALINHAMENTO com um eixo de 35°
  (`fac = 1 − |cos|`, `interpf(..., 0.3)`).
- **3 dos 13 usam `GP_MATERIAL_MODE_DOT`** — que nós já temos como `FlipStroke::tip`.
- O que falta de motor: **curva de pressão** (reusar `ph2d-curve`, já existe) e
  **pressão→opacidade** (hoje só largura).

### Menores

- `MAX_SUB_PER_SPAN = 24` segue sendo um cap de contagem, não de tolerância — com a regra de
  planeza ele quase nunca morde, mas não foi re-medido.
- A reamostragem roda a cada frame do drag (como o RDP); custo por-frame não medido.

---

## 6. Como rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP

# gates de unidade
cargo test -p ph2d-flip-render --release
cargo test -p ph2d-host-desktop --release --bins flip_smooth

# ⚠️ os gates GPU sao #[ignore] e PRECISAM de adapter (sem ele fazem skip, que nao e verde)
cargo test -p ph2d-flip-render --release --test gpu_render -- --ignored

# as sondas (todas imprimem tabela)
cargo test -p ph2d-flip-render --release --test gpu_render measure_ -- --ignored --nocapture
cargo test -p ph2d-host-desktop --release --bins measure_ -- --nocapture
```

### Smoke — o que o Enio tem de olhar

1. **O cruzamento** (a foto da missão): pincel **macio** (hardness < 1), traço que cruza a si
   mesmo e **hachura** que volta sobre si a meio raio. O núcleo de uma passagem não pode ser
   pintado pela cauda macia da outra.
2. **A curva** — desenhe devagar e depois rápido, com curvas fechadas e trechos retos no MESMO
   traço (é o que produz espaçamento desigual). Nenhuma barriga/bico onde a mão não fez.
3. **Zoom OUT bem longe** com hachura densa: a densidade da tinta não pode cair no encontro das
   passagens (W1d).
4. Retas continuam retas e nada ficou facetado (W2b não pode ter tirado densidade de curva).

---

## 7. Lições que valem além desta linha

- **Uma razão entre duas grandezas proporcionais ao mesmo parâmetro é CONSTANTE** — foi assim que
  o alcance/passo tornou os cruzamentos estruturalmente incapazes de caber. Procure essa forma.
- **Uma fixture pode não conter o fenômeno mesmo exercitando o código certo:** no W1d o **X**
  percorre o laço de extras e fica byte-idêntico; só o **quase-paralelo** enxerga o piso que
  falta. O X virou o CONTROLE justamente por isso.
- **Um `debug_assert` sobre uma constante não pode falhar** (clippy pegou o meu), e **uma
  constante que ninguém lê é um knob que não gira** — as duas saíram.
- **Um gate pode pinar o desperdício como contrato:** `rp.len() > 3` sobre uma RETA exigia os 46
  pontos mudos. A metade real dele sobreviveu; a asserção incidental morreu.
- **Meça no lugar certo:** eu quase reportei "sem ganho" por medir a razão de tinta num par de
  telas onde a cópia é ruído.
