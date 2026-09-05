# 04 — ⛔ Recusas MEDIDAS: o que foi construído, medido e rejeitado

> **Leia esta página ANTES de propor qualquer mudança de desenho no quad remesh.** Cada linha é
> trabalho que já foi pago: alguém a construiu inteira, mediu, e o número disse não. *Arquivar
> sem indexar as recusas seria apagá-las* (`CLAUDE.md` §5.0).
>
> ⚠️ **E a cerca de Chesterton tem a metade que morde:** *uma recusa medida responde UMA
> pergunta.* Duas destas já dissolveram porque a cadeia por baixo mudou — as duas estão
> marcadas com ⟳ e dizem o que mudou. Antes de herdar uma recusa, confira se a premissa dela
> ainda está de pé.

## §1 — Na fase zero (F1)

| o que foi tentado | medição | veredito |
|---|---|---|
| **O F1 seguir o alvo do quad** (`PH2D_F1_TARGET=1`) | `χ = 1` · `4` bordo · **`123` dobras** contra `χ = 2` · `0` · `21` | ⛔ *uma malha de trabalho mais fina não é mais informação — é onde a topologia se perde* |
| **Calota mais fina** (`0,75 h` e `0,5 h`) | a fase zero fica verde no bico (`0,84` e `0,55`) e a jusante devolve `7`–`48` arestas de bordo | ⛔ o que a jusante não digere é a **inflação** |
| **Banda simétrica** no campo de tamanho | a mutação que a apagava **sobreviveu** aos dois gates: com a renormalização por cima, o tecto deixa de ser observável | ⛔ revertida |
| **Canonicalizar a pose** da peça | `−77 %` · `−105 %` de alcance | ⛔ destrói |
| **Campo de tamanho contínuo** (em vez do `min` das 27 células) | dispersão `4,4 → 1,4 %` e a ponta de `0/4` para `1/4`, com uma célula a `−40,8 %` | ⛔ *a nitidez da ponta vive exactamente do que se estava a suavizar* |
| **Construir o campo UMA vez da referência** | quebra a realimentação de facto (a grelha é refeita a cada ronda sobre a malha que o laço modifica): `−48,6 %` | ⛔ |
| **`ADAPT_RATIO = 64`** (a folga grande) | melhor em `7` de `8` células — e **a fixtura estava errada**: ela media a peça com a pose assada, e o botão vê sempre a peça **recentrada** | ⛔ revertida no mesmo dia por veredito do dono |

## §2 — No campo e no traçado (F2/F3)

| o que foi tentado | medição | veredito |
|---|---|---|
| ⟳ **Reforçar o alinhamento na CALOTA** (`PH2D_TIP_ALIGN=5`) | sem a calota: as cinco réguas do campo ficam verdes e a **extracção não fecha** (um laço de `14` arestas a `1,1 h` do bico). **COM** a calota (a célula `(1,1)`, corrida em 04/09): `3` de `7` candidatas com furos (`40` e `26` arestas), `p90 3,00` em **todas** | ⛔ fica **instrumento**, não cura — e agora com as duas metades medidas juntas |
| **`k ≥ 10`** no mesmo reforço | furos fora dos espinhos, `>60` a subir; `k = 30` destrói | ⛔ |
| **Podar patches** (menos patches, leques maiores) | empata a orelha com o oráculo e **colapsa a geometria** (`18° → 38°`) | ⛔ *o oráculo usa MAIS leques e sai mais quadrado* |
| **Achatamento conforme** (LSCM) e mais três achatamentos | o conforme dá o **pior**: erro conforme a `1,01` e enviesamento a `28°` | ⛔ *«mais conforme ⇒ mais quadrado» é FALSO* |
| **Ponto fixo** na relaxação | contrai exactamente ½ por ronda e **não endireita nada** | ⛔ *convergir e acertar são coisas diferentes* |
| **Subdivisão local** | fechada por medição | ⛔ |
| **A restrição do arco como 2.ª camada** (fora do `ClosureSystem`) | `100 %` de recusas — os cantos **são** as incógnitas livres da costura | ⛔ a cura entra DENTRO do sistema |

## §3 — Na extracção e no mapa

| o que foi tentado | medição | veredito |
|---|---|---|
| **Penalizar a costura** (`SEAM_WEIGHT`) em vez de eliminar a variável | resíduo de costura `1,00`; `30`–`78` arestas de bordo; `χ` `−4`..`−13` | ⛔ substituído pela **eliminação de variável**: resíduo `0,000`, bordo `0`, `χ = +2`, e `3`–`4 ×` mais rápido |
| **Dois subsistemas** para os fechos de ciclo | o par realimenta-se e diverge (esfera a `NaN`, toro a `6,4e17`); amortecer **não** cura | ⛔ um sistema linear só |
| **Portar a biblioteca MPL-2.0** da extracção | obrigaria a publicar arquivos no subsistema mais valioso, e a extracção dela **não termina** na nossa escala | ⛔ ADR-0167: ela fica como **oráculo**, fora da árvore |
| **O memo das fitas** (cache do assador) | `0` de `237` quadros evitam uma reconstrução, e a cura óbvia reabre o `wgpu OOM` | ⛔ |

## §4 — No acabamento

| o que foi tentado | medição | veredito |
|---|---|---|
| ⟳ **Puxar o vértice mais avançado até ao ápice** | o vértice estava a `0,6198` de mundo — **`23` células** — e o deslocamento levava o aspecto a `12,11` e o enviesamento a `85°` | ⛔ **naquela pergunta**: a grade do bico estava a `3,85 ×` o alvo. Com a calota a grade tem resolução e o deslocamento é de **meia célula** ⇒ é o [`snap_tips`] de 04/09 |
| **Shrinkwrap da região da ponta** | a malha é **destruída** (aspecto `1,9 · 10⁸`) e a régua **mal se move** (`p90 2,03 → 1,55`) | ⛔ *mover vértices `76 ×` não cura: o que falta são CÉLULAS* |
| **Rematar acima da barra** | trocaria um bico chato por um **espeto de uma célula**, e apagaria do selector um defeito de células | ⛔ gate |
| **Rematar um bico-LASCA** (leque de `8` a `15°`) | as faces péssimas vão de `8` para `16` | ⛔ gate — na malha do produto o mesmo remate deixa o censo **intacto** |
| **A suavização do campo de direcções** (`HINT_SMOOTH_ROUNDS`) | construída para curar a lei alinhada e **não curou** | ⛔ a hipótese do ruído por face está refutada |
| **`SQUARE_ROUNDS = 16`** (o ajuste de quadrado cego) | aspecto máximo `122,7 → 30,3`, enviesamento mediano `27° → 26°`, pagando **`3,4 ×`** as dobras | ⛔ *se mover vértices `16 ×` não move a mediana, o defeito está na CONECTIVIDADE* |

## §5 — No selector e no produto

| o que foi tentado | medição | veredito |
|---|---|---|
| **Reordenar as chaves** (pôr a ponta à frente dos furos) | — | ⛔ decisão de produto que o dono já tomou **três vezes** no sentido contrário |
| **O «segundo sorteio»** (re-correr a cadeia com a peça escalada) | na `sculpt_antes` o espinho principal cai nas `18` candidatas de dois sorteios | ⛔ não é rede |
| **Os níveis de exportação mandarem na densidade dos quads** | o `Max` custou **`27 min 29 s`** para sair com `316` arestas de bordo e `6` não-manifold | ⛔ revertido — *o limite da cadeia não é o tempo, é a TOPOLOGIA* |
| **A grade fina para a cadeia de quads** (`line/3DModeling`) | **`107 ×`** o preço para a mesma resposta, e **pior** fidelidade | ⛔ |
| ⟳ **`Follow Curvature` a nascer em `0`** | *«pede-se `400 %` e a saída move-se `7 %`»*, `−15 %` de contagem, o dobro das faces `>60°` | ⟳ **dissolvida em 04/09**: com a fase zero graduada, a calota e o remate, o `1` fica melhor em todas as colunas e mais rápido — ver [`03_as_curas.md`](03_as_curas.md) §9 |

## §6 — O que a cadeia **não** faz, por decisão

- **Feature lines autoradas** — o traçado não recebe arestas marcadas pelo artista.
- **O solver injectivo** do mapa (o que impediria as dobras que sobram) — é wave com espec
  própria, e a recusa é de **âmbito**, não de mérito.
- **O factor de escala conforme por construção** (`Δ log h` contra a curvatura de Gauss) — a
  cura publicada para a adaptação que o G3 projecta fora. Continua por fazer; o
  `Follow Curvature` de hoje é a metade que a renormalização torna barata.
