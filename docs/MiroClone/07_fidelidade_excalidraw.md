# 07 — A fidelidade do `.excalidraw` (importar e exportar) e o que o quadro aprende com o Excalidraw (W9)

> **Pedido do dono (smoke de 10/10, handoff W8 §12):** desenhou um quadro no excalidraw.com
> (`/home/enio/Downloads/bv.excalidraw`: elipse, rectângulo redondo, losango, três setas ligadas, `roughness 1`,
> `strokeWidth 1`), abriu-o no nosso (botão direito numa aba → «Open an Excalidraw file…») e comparou: (1) lá
> as setas são RECTAS, cá saíram em S; (2) a ponta dele é «inteligente», encolhe nas setas curtas; (3) lá a
> seta pára um pouco ANTES da forma, cá encosta — e a do losango para o rectângulo começa DENTRO do losango e
> acaba DENTRO do rectângulo; (4) o ficheiro é de um Excalidraw mais novo que o oráculo (ligações
> `mode:"orbit"`). Pediu: estudar o Excalidraw, tornar TODA a importação (e a exportação) mais fiel, e sugerir
> melhoramentos ao quadro inspirados nele.
>
> Método: CLAUDE.md §0.9 — o Excalidraw é um ORÁCULO que se CORRE (duas versões, 70 casos nossos, 220
> fixtures deterministas), nunca um fonte que se lê; o nosso lado medido por uma sonda descartável sobre as
> MESMAS entradas (apagada; a árvore ficou sem código mudado). Ferramentas e números:
> [`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md) §«W9».

## §0 — Veredito

- **Os quatro sintomas do dono têm UMA causa cada, todas no leitor/roteador, nenhuma no ficheiro:**
  1. **Curvas:** o leitor traduz `roundness {type:2}` → `Route::Curved`, e a nossa curva é DERIVADA (a do
     Miro: sai e entra perpendicular às faces) ⇒ um S de **41 a 83 px** de flecha nas três setas do dono. No
     Excalidraw uma seta «redonda» de **2 pontos é recta** (os controlos ficam a ⅓ e ⅔ da corda, medido nas
     duas versões, afastamento 0,00); só com 3+ pontos ele curva (Catmull-Rom pelos pontos).
  2. **Ponta:** a dele mede **min(25, último trecho / 2)** a 20°, sem depender da espessura; a nossa é FIXA
     (caixa 23,5) — e numa seta mais curta que ~24 **desaparece** (medido: L = 8, 15, 20 ⇒ zero pontas).
  3. **Vão e «entra na forma»:** o modo `orbit` é «aponta para o foco, pára antes»: a ponta fica na recta
     entre os dois focos, a **5 + strokeWidth/2 da forma** (5,5 no ficheiro do dono). O leitor traduz o
     `fixedPoint` para `Anchor::Fixed`, cuja lei é «a ponta É esse ponto» (o `inside` dele) ⇒ com um foco
     interior a ponta entra: a seta 3 do dono começa **45,4 px dentro** do losango (o foco dela é o centro) e
     acaba **36,3 px dentro** do rectângulo; a seta 2 acaba 15,6 px dentro.
  4. **Versão:** o `orbit` existe desde o build `0.18.0-c141960` (25/11/2025) do canal `next`; o npm `latest`
     ainda é o 0.18.1 (que IGNORA `orbit`). O leitor tem de falar as DUAS gramáticas e o escritor escrevê-las
     JUNTAS (medido: cada versão lê a sua e preserva a outra).
- **A lei do `orbit` reproduz o próprio Excalidraw ao centésimo:** recortando a recta entre os dois focos na
  faixa de 5 + sw/2 à volta do contorno (com os cantos redondos e a elipse), as seis pontas gravadas no
  ficheiro do dono saem com erro **≤ 0,04 px**. ⇒ a cura é uma âncora NOVA (`Anchor::Orbit`), não um remendo.
- **O formato inteiro:** fora das setas, o que perdemos ao importar é sobretudo ESTILO (o tipo de
  preenchimento, a rugosidade 2, a semente do tremor, a letra e o alinhamento do texto, a pressão constante da
  caneta nova) e três TIPOS (a nota `stickynote` do Excalidraw novo, a `line` fechada `polygon:true`, as pontas
  de cardinalidade). O que exportamos perde-se no 0.18.1 em dois sítios: as ligações (falamos só o legado) e
  nada mais grave — o `stickynote` NÃO pode sair para o 0.18.1 (ele o DESCARTA) ⇒ a nota continua a sair como
  `rectangle`.

## §1 — Fontes (versões, licenças, como se correu)

| oráculo | versão | licença (artefacto instalado) | onde |
|---|---|---|---|
| `@excalidraw/excalidraw` (npm `latest`) | **0.18.1** (20/04/2026) | MIT (`package.json`; sem ficheiro LICENSE) | `~/Apps/excalidraw-oracle` |
| `@excalidraw/excalidraw` (npm `next`, o do excalidraw.com) | **0.18.0-143b5b6** (08/10/2026) | MIT (idem; `element`/`common`/`math` MIT, `fractional-indexing` CC0-1.0) | `~/Apps/excalidraw-oracle-next` |
| `roughjs` 4.6.4 · `perfect-freehand` 1.2.0 · `points-on-curve` 1.0.1 · `laser-pointer` 1.3.1 | — | MIT (LICENSE no pacote) | idem |
| Chromium do Playwright 1.63.0 · React 19.1.0 · esbuild 0.28.2 | — | BSD-3 · MIT · MIT | idem |

- **Achar a versão:** `npm view @excalidraw/excalidraw dist-tags` ⇒ `latest 0.18.1`, `next 0.18.0-143b5b6`.
  Bissecção de 99 builds `0.18.0-<commit>` por `npm pack` + `grep` nos `.d.ts`: `BindMode = "inside" | "orbit" |
  "skip"` entre `7f66e1f` (13/11/2025) e **`c141960` (25/11/2025)**; `stickynote` entre `2b1c2a9` (07/09/2026) e
  **`afa3a65` (10/09/2026)**. O 0.18.1 é um remendo do ramo 0.18 e não tem nenhum dos dois.
- **Como correu:** `corre_fidelidade.mjs` (página `src/oracle_w9.js`, independente da versão) sobre
  `entradas/w9_casos.json` (70 casos gerados por `gera_w9_casos.py`) nas duas versões; passos `restore`
  (`restoreElements`, `repairBindings`), `montar` (editor 1280×800, zoom 1 ⇒ ecrã = cena), `empurrar` (selecciona
  as formas, `→` e `←`: o editor RE-LIGA as setas pela lei dele), `desenhar` (o rato do Playwright com a
  ferramenta), `serializar` (`serializeAsJSON`), `svg`. As tabelas saem de `mede_w9.py`. **Duas corridas =
  220/220 ficheiros idênticos** (relógio livre nos casos de rato, ids/sementes novas normalizadas).
- **O nosso lado:** uma sonda descartável (`ph2d-board-excalidraw/tests/…`, apagada) correu o `read` + a
  `RouteCache` + o `drawn` sobre as mesmas entradas (o ficheiro do dono e os `restore` de `B_*`, `P_*`, `F_*`).
- ⛔ **Não medido:** ver §6.4.

## §2 — As leis (lei → fonte → número)

### Setas: o fuste e a ponta (XA)

| lei | fonte | número |
|---|---|---|
| **XA1** seta/linha com `roundness` e **2 pontos** desenha-se RECTA; com **3+**, Catmull-Rom UNIFORME pelos pontos (pega de saída de Pᵢ = (Pᵢ₊₁ − Pᵢ₋₁)/6; nas pontas, (P₁ − P₀)/6); sem `roundness`, segmentos rectos | `F_fuste` (SVG, rugosidade 0): `M0 0 C33.33 13.33, 166.67 66.67, 200 80` para (0,0)→(200,80); `C16.67 16.67, 66.67 96.67, 100 100 C133.33 103.33, …` para 3 pontos | afastamento à corda **0,00** (2 pontos); 23,4 (3 pontos) |
| **XA2** o comprimento da ponta = **min(base, último trecho / 2)**, igual em qualquer espessura; base: `arrow` **25 a 20°** (meia-abertura), `triangle`/`triangle_outline` **15 a 25°**, `circle`/`circle_outline` **Ø 15**, `bar` **15** de cada lado, `diamond`/`diamond_outline` **24** de comprido (meia-largura 5,07); as de cardinalidade (next) pela mesma regra | `P_comprimento` (L 8…400 × sw 1/2/4), `P_tipos_comprimento` (7 tipos × L 10…100), `P_ultimo_trecho` | `arrow`: L 8 → 4,00 · 20 → 10,00 · 40 → 20,00 · ≥ 50 → **25,00**; 20,0° em todas; sw 1/2/4 iguais; 3 pontos com o último trecho 20 ⇒ 10 |
| **XA3** a ponta segue a direcção do fim do caminho (numa curva, a tangente perto do fim) | `F_fuste` | — |

### Setas: a ligação (XB)

| lei | fonte | número |
|---|---|---|
| **XB1** ⭐ **`orbit`**: a ponta fica na recta entre os DOIS FOCOS (o `fixedPoint` de cada ponta presa, no mundo; a ponta solta é o seu próprio foco), no 1.º ponto que entra na faixa de **g = 5 + strokeWidth(da forma)/2** à volta do contorno; a faixa segue o contorno real (cantos redondos do rectângulo `type 3` = min(S/4, 32), o corte de ¼ do losango `type 2`); na elipse, a elipse de semi-eixos + g; se a recta chega ao foco sem entrar na faixa (foco FORA), a ponta é o foco | `B_*` (empurrar), `B_seta_sw*_forma_sw*`, `B_orbit_fora`, `B_duas_pontas`; e os pontos gravados pelo excalidraw.com no ficheiro do dono | g = **5,5 · 6 · 7** para a forma de sw 1 · 2 · 4 (seta sw 4 em forma sw 1 ⇒ 5,5); foco a −0,25 ⇒ a ponta a **50** fora; ficheiro do dono: erro ≤ **0,04 px** nas 6 pontas |
| **XB2** **`inside`**: a ponta É o foco (dentro da forma, sem vão) | `B_inside_*` | rect 0,75/0,3 ⇒ ponta (650, 136) exacta, 36 dentro |
| **XB3** `skip` comporta-se como `orbit` na geometria | `B_skip_dentro` | iguais a `B_orbit_dentro` ao centésimo |
| **XB4** o losango tem o vão medido contra o losango DELE (vértices `⌊w/2⌋+1`, a assimetria recusada em W1) | `B_orbit_centro_diag` | aresta: **6,00**; no vértice esquerdo 5,67 (5,16 com sw 1) |
| **XB5** desenhar (next): largar DENTRO de uma forma (2…40 px, centro) ⇒ `inside` no ponto exacto; FORA até ~10 px ⇒ `orbit` com o foco ENCAIXADO no contorno (o meio do lado: `[0, 0.5001]`) e a ponta a g; a 20 px ⇒ solta; `Ctrl` ⇒ nunca liga; o estado nasce `bindMode "orbit"`, `isMidpointSnappingEnabled true` | `M_*`, `M2_*` (rato, relógio livre) | dentro ⇒ `inside` em **23/23** sondas (profundidade 2…40, tempo parado 0…1500 ms, Alt, Shift, um salto, devagar, fundo-e-volta, arrastar a ponta) |
| **XB6** desenhar (0.18.1): liga se a ponta cai na faixa (dentro até ≥ 24, fora até ≥ 20; a 40 dentro não liga), grava `{focus, gap = distância}` e no re-ligar põe a ponta FORA a esse `gap` (largada 16 dentro ⇒ fica 16 fora) | `M2_*` (0181) | — |
| **XB7** a ponta gravada é a VERDADE ao abrir: nenhuma das versões re-liga ao carregar (`restore` e montar não mexem); só mexer uma forma ligada re-calcula | `dono_bv`, `B_*` (passos montar × empurrar) | o ficheiro do dono no next: 0,00 de mudança depois de empurrar |

### O ficheiro nas duas versões (XV)

| lei | fonte | número |
|---|---|---|
| **XV1** o **0.18.1 ignora `mode`/`fixedPoint`** numa seta não-cotovelo: preserva as chaves, acrescenta `focus: 0` e, sem `gap`, re-liga ao CENTRO a encostar | `B_orbit_*` (0181) | vão 0,00 (rect/elipse), −0,86 no losango |
| **XV2** o **next converte `{focus, gap}`** em `{mode:"orbit", fixedPoint}` (o foco tirado da ponta gravada: `focus 0` ⇒ `[0.5001, 0.5001]`) e **ignora o `gap`** gravado (usa XB1) | `B_legado_*` (next) | `gap 10` ⇒ 6,00 |
| **XV3** com as DUAS gramáticas na mesma ligação (`{mode, fixedPoint, focus, gap}`), cada versão usa a sua e o next deixa cair `focus`/`gap` | `B_ambos_orbit_e_legado` | 0181: vão 3,01 (o `gap`); next: 6,00 e a ponta na recta do foco (0,00) |
| **XV4** **`stickynote`** (next, ferramenta `N`): arrastar dá um QUADRADO 200×200, `backgroundColor #ffdf6b`, `roundness {type:2}`, `baseHeight 200`, sem traço (SVG: caminho tremido com raio 8 + sombra); o texto preso tem `containerId` e `baseFontSize`. O **0.18.1 DESCARTA-O** (tipo desconhecido; o texto fica solto) | `M_nota`, `T_formato` | — |
| **XV5** next renomeia ao ler: `dot` → `circle`, `crowfoot_one/many/one_or_many` → `cardinality_one/many/one_or_many`; acrescenta `created`, `baseFontSize`, `labelPosition` (texto) e `strokeOptions {variability, streamline 0.5}` (caneta: `"variable"` ao ler um ficheiro antigo; a ferramenta nova grava `"constant"` e **sw 1**); larga `lastCommittedPoint` | `P_tipos`, `T_formato`, `M_caneta` | — |
| **XV6** inalterado entre as versões: o fuste (XA1), a ponta `arrow`/`bar`/`circle`/`triangle`/`diamond` (XA2), `polygon`, `fixedSegments` do cotovelo, `customData`, `link`, `locked`, `angle`, `opacity`, `roundness.value`; as ferramentas Rectângulo (`type 3`) e Linha (`type 2`, sw 2) | `T_formato`, `M_*` | — |
| **XV7** ⭐ **o legado `{focus, gap}` como o 0.18.1 o LÊ** (W10). Ao re-ligar (a forma mexeu) a ponta fica na recta do ponto VIZINHO da seta (`points[1]` / `points[n-2]`) que passa por \|focus\|·K, K o canto da CAIXA (rectângulo E elipse) ou o vértice do losango do lado do sinal ⇔ **focus = d/h(n)**, d a distância com sinal do centro à recta, h o suporte da caixa (hw\|nx\|+hh\|ny\|) ou do losango (max(hw\|nx\|, hh\|ny\|)), no referencial da forma (rotação incluída). Foco além de ±1 é aceite sem corte. A ponta é o 1.º ponto a **`gap`**: rectângulo alargado de cantos vivos (Chebyshev; a espessura da forma não entra), elipse de semi-eixos + gap (também «redonda» e rodada); `gap` negativo entra na forma. **`gap 0` é especial**: a ponta vai para DENTRO, para o ponto do foco \|f\|·K; 0,01 já é normal. **Rectas paralelas aos eixos** (fora do centro): dois cantos empatam e ele escolhe o do outro lado — a ponta sai espelhada (até com gap positivo). Uma recta cujo foco não toca a forma + gap não é re-ligada (a ponta fica onde estava). Losango e rectângulo de cantos redondos: o vão dele NÃO é uma distância constante — lei não medida (§3) | `saidas/w10_F_*.0181.json` (34 casos, ~3 600 pontas re-ligadas; tabelas por `mede_w10.py`) | \|Δfocus\| **0,00000** em todos os casos (0,00003 no losango redondo); gap: rectângulo 0,0000 (`F_rectangle_alem_g15` ⇒ foco além de ±1), elipse 0,0000, rect −10 e elipse −10/−40/−47 entram; `F_*_g0` ⇒ dentro; `F_*_eixos_*` ⇒ 48 pontas espelhadas; losango g=6: vértice x a 4,43, y a 5,39, resíduo até 3,7 ao longo da aresta; rect redondo g=6: 5,09 no lado, 3,17 em cima |

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| copiar a ponta invertida | ⛔ medido (`P_ultimo_trecho`, as duas versões): seta REDONDA com o último trecho curto e colinear (200 + 20) desenha as riscas À FRENTE da ponta (229,4 contra a ponta em 220) — defeito dele; a nossa direcção vem do caminho e fica |
| o losango de vértices `⌊w/2⌋+1` só para acertar o vão XB4 | ⛔ já recusado (README do oráculo, W1): o nosso losango é simétrico; o vão difere ≤ 0,7 px no vértice |
| o vão do losango e do rectângulo redondo no 0.18.1 (XV7) | ⛔ fica SEM lei: o escritor grava o nosso g por fora e menos a distância ao contorno por dentro; resíduo medido no nosso ficheiro após o 0.18.1 re-ligar **3,875** (losango) e **3,513** (rect redondo) em órbita/centro, **4,70** (losango) e **85,6** (rect redondo, do lado de lá) num ponto fixo dentro — contra até **106 px** antes da W10. O dono usa o excalidraw.com (next), onde é exacto (XB1) |
| um ponto fixo (`inside`) DENTRO, do lado de LÁ da recta, no 0.18.1 (XV7; auditoria W10, lente 1) | ⛔ NÃO se exprime: ele pára na ENTRADA do nível `gap` (o 1.º ponto da recta). O escritor escreve, de dois pares, o que a lei (`legacy_focus::landing`, exacta a ≤ 0,044 px no rect/elipse) põe mais perto: a entrada no nível, ou o ponto do foco \|f\|·K do `gap 0`. Medido no nosso ficheiro: 68,4 e 70,1 (rect), 50,9 e 24,7 (elipse), 9,8 (elipse funda junto ao eixo maior: o nível teria semi-eixo < 1 px e recusa-se — a elipse degenerada dele errou 9,8). A âncora `Fixed` só nasce do `inside` do next, onde é exacta |
| o foco do GESTO na faixa por fora = o CURSOR (next) | ⛔ aberto: o next guarda o próprio cursor como foco (`fixedPoint [-0.015, 0.25]` com o cursor 3 px fora, `w10_R_ponta_banda_diag.next`); o nosso `anchor_for` projecta o foco no contorno ⇒ a ponta difere ~**4,3 px** numa aproximação diagonal (494,0 134,49 no next contra ~138,8 no nosso). Fica a lei da W9 (smoke do dono OK 10/10) |
| o realce do arrasto de uma ponta marcar o FOCO | ⛔ o next NÃO marca o foco a meio do arrasto (`w10_R_*`): só o contorno azul e a seta já parada a g. Os pontos que ele mostra (cinza, azul ao encaixar) são o encaixe no meio do lado (`isMidpointSnappingEnabled`), que não temos; o nosso ponto no foco saiu (W10) |
| ler o `gap` gravado no legado `{focus, gap}` | ⛔ o Excalidraw de hoje (next) DEITA-O FORA (XV2) e o ficheiro dele passa a orbit; guardar um vão por ligação seria um campo que o produto de referência já não tem |
| imitar o 0.18.1 a ignorar `orbit` (XV1) | ⛔ é a versão velha a não perceber a nova; o leitor percebe as duas |
| `inside` × `orbit` por TEMPORIZADOR (o `bindModeHandler` do next) | ⛔ não reproduzido: em 23 sondas de rato a ponta largada dentro saiu SEMPRE `inside`; a regra observável é a geográfica (XB5). Não se copia um temporizador que não se consegue medir |
| exportar a nota como `stickynote` | ⛔ o 0.18.1 (o npm `latest`) DESCARTA-A (XV4): a nota continua `rectangle` cheio + texto; o `customData.ph2d` já a devolve igual na volta (X2) |
| o «hachure» como preenchimento de NASCENÇA do rascunho | ⛔ continua recusado (W4, foto da cena 5): só o elemento IMPORTADO com `fillStyle hachure/cross-hatch` o traz (§5 M7) |
| pontos editados no Excalidraw numa seta curva ou em cotovelo | ⛔ continua (estudo 06 §3): a curva e o cotovelo NOSSOS são derivados. A seta `Spline` (§5 M3) é outra coisa: a curva DELE pelos pontos dele |
| o círculo da ponta CENTRADO no bico (o dele: `P_tipos_comprimento`, centro = bico, Ø min(15, L/2)) | ⛔ fecho W9 (auditoria, lente 1): fica o do catálogo, TANGENTE ao bico — a ponta não entra no vão de 5 + sw/2 que acabámos de abrir (centrado, metade do círculo tapava-o); o TAMANHO é o dele (XA2, gate `xa2_every_head_type…`), a posição desvia `tamanho/2` (7,5 no cheio) |
| o miolo da forma ⇒ `Orbit` no ponto largado (§5 M4, §6.1) | ⛔ fecho W9: o miolo continua `Center` — a face do Miro (a curva de nascença sai perpendicular pelo lado que olha a outra forma, e as paralelas afastam-se, W2), AGORA com o vão (`center_exit`); numa recta o centro É a órbita do foco `[0.5, 0.5]`. A faixa junto ao contorno é `Orbit` (o encaixe XB5) |

## §4 — O formato inteiro: Excalidraw · nós · lacuna · cura

Legenda do «nós»: **L** = o leitor (`read.rs`), **E** = o escritor (`write.rs`), **D** = o desenho
(`ph2d-board-route`/`-render`). ✅ fiel · ⚠️ aproximado · ❌ perdido.

| elemento / propriedade | Excalidraw (medido) | nós hoje | lacuna | cura |
|---|---|---|---|---|
| seta `roundness` + 2 pontos | RECTA (XA1) | L ⇒ `Curved` (S de 41–83 px) ❌ | o sintoma 1 do dono | L: 2 pontos ⇒ `Straight` |
| seta `roundness` + 3+ pontos | Catmull-Rom 1/6 pelos pontos (XA1) | L ⇒ `Curved` com `waypoints` (a curva do Miro, braço 0,45/0,4) ⚠️ | outra curva pelos mesmos pontos | `Route::Spline` (variante NOVA no fim do enum: a curva dele pelos pontos dele) |
| seta sem `roundness` | segmentos | L ⇒ `Straight` + `waypoints` ✅ | — | — |
| `elbowed` + `fixedSegments` | cotovelo do editor; os troços fixados guardados | L ⇒ `Elbow` (rota NOSSA, `fixedSegments` perdido) ⚠️ | recusa 06 §3 | — (fica) |
| ligação `orbit` (next) | XB1: recta entre focos, vão 5 + sw/2 | L ⇒ `Anchor::Fixed` = a ponta NO foco ❌ (entra até 45 px) | sintomas 3 e 4 | `Anchor::Orbit([u,v])` (variante nova no fim) + XB1 no roteador |
| ligação `inside` (next) | a ponta no foco (XB2) | L ⇒ `Fixed` ✅ (por acaso: é a mesma lei) | — | manter, explícito |
| ligação `skip` | geometria de orbit (XB3) | ⇒ `Fixed` ❌ | — | ⇒ `Orbit` |
| ligação legado `{focus, gap}` (0.18.1) | 0.18.1: foco desviado por `focus`, a `gap`; next: XV2 | L ⇒ `Anchor::Center` (sem `fixedPoint`) ⚠️ (rota NOSSA a sair pela face) | — | ⇒ `Orbit` com o foco tirado da ponta gravada (como o next: a recta da outra ponta pela ponta gravada até ao interior) |
| E: as ligações que escrevemos | — | ~~`{focus:0, gap:1}`~~ ⇒ **W10: o par da XV7** (`legacy_focus.rs`; + `fixedPoint` só no cotovelo) ✅ | (antes) no next viram orbit ao CENTRO (XV2): uma âncora `Fixed` nossa perde o ponto quando o dono mexe a forma lá | E: as DUAS gramáticas (XV3): `{elementId, mode, fixedPoint, focus, gap}`; `Fixed` ⇒ `inside`, `Orbit`/`Center` ⇒ `orbit` |
| E: as pontas gravadas | XB7: são a verdade ao abrir | encostam (vão 0) ⚠️ | lá ficam encostadas até mexer | E: as pontas com o vão XB1 (o desenho igual ao dele à chegada) |
| ponta `arrow` | min(25, trecho/2) a 20° (XA2) | caixa FIXA 23,5 (26,6°); some abaixo de ~24 ❌ | sintoma 2 | D: `head_len = min(base, trecho/2)` |
| `triangle_outline` | triângulo vazado | L ⇒ `Triangle` (cheio) ⚠️ | — | `Head::TriangleOpen` (no fim do enum) |
| `dot` | = `circle` (XV5) | ⇒ `Circle` ✅ | — | — |
| `crowfoot_*` / `cardinality_*` (6) | pés-de-galinha (ER) | ⇒ `Head::None` ❌ | a notação ER some | `Head::One/Many/OneOrMany/ExactlyOne/ZeroOrOne/ZeroOrMany` (onda própria; pedem um gesto na barra) |
| rótulo da seta (texto com `containerId` = seta) | a meio (`labelPosition` null = 0,5) | ⇒ `label` ✅ | `labelPosition` ≠ 0,5 perdido (next) | L/E: posição do rótulo (campo novo; baixo impacto) |
| rectangle/ellipse/diamond | — | ✅ (caixa, ângulo, `roundness` ⇒ `round`) | `roundness.value` ignorado | — (raio é lei; o `value` não aparece na UI dele) |
| `stickynote` (next) | XV4 | SALTA com aviso ❌ | as notas de quem usa o Excalidraw de hoje | L: ⇒ `ShapeType::Sticky` (cor, texto, tamanho); E: continua `rectangle` (recusa §3) |
| `text` solto | `textAlign` (left de nascença), `verticalAlign`, `fontFamily` (8 famílias), `lineHeight` | ⇒ rectângulo sem traço com o texto CENTRADO ⚠️ | o texto alinhado à esquerda muda de sítio; a letra de máquina (Helvetica/Cascadia/Nunito…) vira a nossa | modelo: alinhamento do texto da forma (esq./centro/dir., topo/meio) + `fontFamily` 1/5/8 ⇒ rascunho, 2/6/9 ⇒ sem serifa, 3 ⇒ mono (se/quando houver) |
| texto num contentor | centrado | ✅ | `fontFamily` | idem |
| `line` aberta | — | ⇒ seta sem pontas ✅ | — | — |
| `line` `polygon:true` (fechada, cheia) | polígono cheio | ⇒ seta sem pontas, sem o preenchimento ❌ | a forma livre fechada | forma `ShapeType::Polygon` com os pontos (onda própria) |
| `freedraw` `pressures` + `simulatePressure` | — | ✅ | — | — |
| `freedraw` `strokeOptions.variability "constant"` (next, a ferramenta nova) | largura CONSTANTE | ⇒ pressão simulada (afina) ⚠️ | o traço do dono afinaria cá | L: `constant` ⇒ sem pressão |
| `frame` / `magicframe` (`name`, `frameId`) | — | ✅ | — | — |
| `groupIds`, `locked`, `opacity`, `angle`, `strokeStyle`, cores | — | ✅ | — | — |
| `fillStyle` hachure / cross-hatch / zigzag | riscas | ⇒ cheio ❌ (`FillStyle::Solid` sempre) | o desenho importado muda de cara | `Style.fill_style` por elemento (o porto `ph2d-board-rough` já tem `Hachure`/`CrossHatch`; `zigzag` por portar) |
| `roughness` 0 / 1 / 2 | 2 = «cartoonist» | ⇒ booleano (`sketch`) ⚠️ | 2 perde-se | `sketch` → nível 0/1/2 |
| `seed` | a semente do tremor | ⇒ tirada do NOSSO id ❌ | o mesmo rascunho treme diferente cá | guardar a semente importada (no `Org`, raro) |
| `link` (URL) | URL ou ligação a elemento (`isElementLink`) | ignorado ❌ | — | o nosso `link` é a elemento; URL = campo novo (baixo) |
| `customData` de terceiros | preservado | ignorado ⚠️ | some na volta | E: devolver o que não é `ph2d` (baixo) |
| `image`, `embeddable`, `iframe` | — | SALTAM com aviso (X3) ⚠️ | recusa 06 §3 (sem elemento imagem) | onda própria |
| `index` / ordem, `isDeleted`, `version` | — | ✅ (X4) | — | — |

## §5 — Melhoramentos inspirados no Excalidraw (por impacto para o dono)

| # | o quê | como se mediu | o valor | recomendação |
|---|---|---|---|---|
| **M1** | ⭐ **A seta que pára ANTES da forma** (`Anchor::Orbit` + o vão) | XB1 em 20 casos `B_*` e nos pontos do ficheiro do dono | vão **5 + sw/2** (5,5 com o traço fino de nascença); erro ≤ 0,04 px | **FAZER JÁ** — ✅ **DECIDIDO pelo dono (10/10): o quadro INTEIRO adopta o vão, como o Excalidraw** (*«like Excalidraw»*): as setas que o artista desenha no nosso quadro também param 5 + sw/2 antes da forma, não só as importadas. O Miro encosta; recusado por ordem do dono |
| **M2** | ⭐ **A ponta que encolhe** | XA2 (`P_comprimento` 39 setas, `P_tipos_comprimento` 63) | **min(base, trecho/2)**, sem espessura | **FAZER JÁ** em todo o quadro: hoje uma seta curta fica SEM ponta (L ≤ 20 ⇒ zero pontas) — defeito, não estilo |
| **M3** | ⭐ **A «redonda» de 2 pontos é recta**; a de 3+ é a curva DELE | XA1 (`F_fuste`) | afastamento 0,00 / Catmull-Rom 1/6 | FAZER JÁ **só na importação** (sintoma 1); `Route::Spline` para 3+. ⛔ **Decisão do dono (10/10): a curva do NOSSO quadro não muda** (*«não quero que a seta redonda de dois pontos fique reta, quero apenas que a importação seja fiel»*): o `Route::Curve` desenhado no quadro continua a curva do Miro; só a seta que VEM de um `.excalidraw` se desenha como lá. E a volta guarda a forma dele: a «redonda» de 2 pontos lida como recta volta a sair `roundness {type:2}` (o `customData.ph2d` leva o `roundness` original) |
| **M4** | O gesto de ligar do next: largar DENTRO ⇒ ponto exacto (`inside`); perto por FORA ⇒ órbita encaixada no meio do lado; `Ctrl` desliga | XB5 (rato, 23 sondas) | 23/23 dentro ⇒ `inside`; por fora liga a 6 e 10, não a 20 | o nosso `anchor_for` já é parecido (centro × ponto fixo numa faixa); trocar «miolo ⇒ Centro» por «miolo ⇒ Orbit no ponto» quando M1 entrar; manter o `Ctrl` |
| **M5** | Ler o `stickynote` | XV4 | 200×200, `#ffdf6b`, raio 8 | ler ⇒ a nossa nota; escrever continua `rectangle` |
| **M6** | Escrever as DUAS gramáticas de ligação | XV3 | — | FAZER com M1: o ficheiro nosso fica certo no excalidraw.com e no 0.18.1 |
| **M7** | Estilo importado: `fillStyle` riscas, `roughness 2`, a semente | `T_formato` + o código (`hand_options` fixa `Solid`; `seed()` vem do id) | — | médio: um desenho do Excalidraw abre «com a cara dele»; os portos já existem |
| **M8** | Alinhamento e família da letra do texto | `T_formato` | 8 famílias; `left` de nascença | médio: o texto solto alinhado à esquerda é o caso COMUM no Excalidraw e cá muda de sítio |
| **M9** | Pontas de cardinalidade (pé-de-galinha) e o triângulo vazado | `P_tipos`, `P_tipos_comprimento` | base 15, mesma regra XA2 | baixo-médio (diagramas ER) |
| **M10** | Caneta de largura CONSTANTE (a nova do Excalidraw, sw 1) | `M_caneta` (next) | `variability "constant"` | baixo: ler `constant` ⇒ sem pressão |
| **W10** | ✅ **As pontas da W9 no 0.18.1** | XV7 em 34 casos `F_*` (0.18.1) e 9 `R_*` (as duas versões) | o escritor grava o legado pela XV7 (`ph2d-board-excalidraw/src/legacy_focus.rs`: \|gap\| ≥ 0,01; \|focus\|·(1+1e-6) desfaz o empate dos eixos); gates `xv7_the_0181_moves_no_tip_when_it_rebinds_our_arrows` (rect/elipse ≤ **0,044** px medido; losango/redondo ≤ 4) e `xv7_the_legacy_law_reproduces_the_0181_sweep`; no next o mesmo ficheiro: rect/elipse ≤ 0,05 salvo a curva do Miro (15 pontos) a 0,278 px e o losango 1,43 (a recusa W1 do ⌊w/2⌋+1) | feito. A letra importada (`Letter::Hand/Sans`) volta a Auto quando o botão «Rascunho ↔ Final» alcança o elemento (decisão do dono 10/10). O realce do arrasto sem o ponto no foco (§3) |

## §6 — Plano da próxima onda

### §6.1 — Ficheiros

- `crates/ph2d-board-model/src/connector.rs`: `Anchor::Orbit([f64; 2])` e `Route::Spline` **no FIM** dos enums
  (postcard é posicional); `Head::TriangleOpen` no fim de `Head`. Doc-comments com a lei e o link para este estudo.
- `crates/ph2d-board-geom`: a interseção da recta com o contorno AFASTADO de `g` (rectângulo com os cantos
  redondos da regra de W1, elipse de semi-eixos + g, losango/polígono pelo offset) — `ray_exit_offset(el, from,
  to, g)`.
- `crates/ph2d-board-route/src/lib.rs`: `exit()` para `Orbit` (XB1: a recta entre os FOCOS das duas pontas,
  recortada no vão; foco fora ⇒ o foco); `ORBIT_GAP = 5.0` (+ sw da forma / 2) com a tabela §2 ao lado;
  `Spline` (Catmull-Rom 1/6, XA1); o tamanho da ponta `min(base, último trecho / 2)` em `drawn_at`/`stems`
  (`HEAD_BASE` por tipo: 25 · 15 · 15 · 24 · 15).
- `crates/ph2d-board-excalidraw/src/read.rs` (⛔ SÓ a importação — o `Route::Curve` do quadro não muda, §5 M3): 2 pontos redondos ⇒ `Straight` com o `roundness` dele guardado para a volta; 3+ ⇒ `Spline`; `mode` ⇒
  `Orbit`/`Fixed`; legado ⇒ `Orbit` com o foco da ponta gravada; `stickynote` ⇒ `Sticky`; `triangle_outline`;
  `strokeOptions.variability`.
- `crates/ph2d-board-excalidraw/src/write.rs`: ligações com as duas gramáticas; as pontas com o vão; `Spline` ⇒
  `roundness {type:2}` com os pontos dela (não achatada).
- `docs/MiroClone/ferramentas/excalidraw_oracle/`: os gates leem as fixtures `saidas/w9_*` (nada a re-correr).

- `crates/ph2d-board-edit` (`anchor_for` e o gesto de ligar): ⭐ por decisão do dono (§5 M1) a seta DESENHADA
  no quadro nasce `Orbit` (o vão de XB1) quando a ponta cai numa forma — o gesto de M4: largar dentro ⇒ o
  ponto exacto em órbita, `Ctrl` desliga a ligação. As setas que já existem nos ficheiros (`Fixed`/`Centre`)
  continuam a ler-se como estão.

### §6.2 — Os testes que têm de nascer VERMELHOS (e ver-se vermelhos antes da cura)

1. **`read` do ficheiro do dono** (`entradas/w9_dono_bv.excalidraw`): as três setas `Route::Straight`, `Anchor::Orbit`
   nas seis pontas, e as pontas ROTEADAS iguais às gravadas pelo Excalidraw a **≤ 0,05 px** (hoje: S de 41–83 px
   e pontas até 45 px dentro).
2. **Oráculo XB1** (`ph2d-board-route::oracle_tests`): para cada `w9_B_*.next.json` (rect e elipse; o losango com
   tolerância de 1 px, §3), a ponta depois de `empurrar` — sw 1/2/4, seta ≠ forma, foco fora, cantos redondos,
   duas pontas presas.
3. **Oráculo XA2**: `w9_P_comprimento.next.json` — 39 setas, a ponta `arrow` com `min(25, L/2)` a ±0,05 (hoje
   L 8/15/20 sem ponta); `w9_P_tipos_comprimento` para triângulo, círculo, barra, losango.
4. **Oráculo XA1**: `w9_F_fuste` — 2 pontos redondos ⇒ recta; 3 pontos ⇒ os controlos do SVG dele (rugosidade 0) a 0,01.
5. **Escrita XV3**: o nosso `.excalidraw` com uma `Fixed` e uma `Orbit` passa pelo oráculo (next e 0.18.1):
   `restore` + `empurrar` deixa as pontas onde as escrevemos (≤ 0,05) — um caso novo em `w9_casos.json`
   gerado a partir da saída do nosso escritor.
6. **Ida e volta** (X2 continua): nosso → `.excalidraw` → nosso = IGUAL com `Orbit` e `Spline`.
7. Prova de mutação nas leis novas (o vão, o `min` da ponta, a escolha 2 pontos ⇒ recta).

### §6.3 — O que a cena/smoke deve mostrar ao dono

Uma cena nova (a 11.ª do quadro, `PH2D_BOARD_SMOKE=11`) que abre o ficheiro do PRÓPRIO dono
(`docs/MiroClone/bv.excalidraw`, cópia de `entradas/w9_dono_bv.excalidraw`; a foto `docs/MiroClone/bv_excalidraw.png`) ao lado da
fotografia do Excalidraw (`saidas/w9_dono_bv.next.svg` rasterizado): as três setas RECTAS, a parar 5,5 antes
das formas, a seta curta do losango com a ponta pequena; e uma fila de setas curtas (10…60) a mostrar a ponta a
encolher. Fotografada antes de mandar; passos numerados para o dono (abrir o ficheiro pelo menu da aba,
comparar com a foto).

### §6.4 — ⛔ O que NÃO se conseguiu medir

- **Como o excalidraw.com gravou `orbit` com focos INTERIORES** (`[0.86, 0.69]` no rectângulo do dono): no next
  de 08/10 toda a ponta largada dentro de uma forma saiu `inside` (23 sondas: profundidade 2…40, parado
  0…1500 ms, Alt/Shift, um salto, devagar, arrastar a ponta de uma seta existente). Ou o site corre um build
  diferente do `next` npm, ou é um gesto não sondado. Não muda a cura: o LEITOR trata os dois modos pela
  geometria medida (XB1/XB2), e o ficheiro do dono confirma XB1 a 0,04 px.
- A seta por CLIQUES (clique, clique): a sonda `M2_prof16_cliques` não criou seta nas duas versões (o gesto
  não foi reproduzido pelo Playwright); o auto-ajuste do texto da nota (`baseFontSize`); o `zigzag`; o
  desenho do cotovelo com `fixedSegments` depois de mexer; imagens e embeddables; o nosso rascunho de um
  elemento importado contra o dele ao píxel (a semente difere, §4).
