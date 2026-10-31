# F2b — A lei de MISTURA de cor: o que os apps fazem, o que é robusto, o que o ADR-0197 deve mudar (agente F2b, 31/10)

> Pedido do dono (31/10): *«o modo de mistura de cores que usamos é experimental […] veja como apps de sucesso como
> Procreate preferem a mescla de cor. Vamos usar o que traz resultados consistentes e robustos mais garantidos.»*
> Leitor: a próxima LLM (F3). Modelo regenerável: [`ferramentas/misturas.py`](ferramentas/misturas.py)
> (`python3 -I docs/Painter/Arquitetura/ferramentas/misturas.py`, numpy, semente `31102026`); números em
> [`fotos/f2b_misturas/metricas.tsv`](fotos/f2b_misturas/metricas.tsv).

## 0. Separar as DUAS leis que o ADR-0197 junta

| | (a) lei de CAMADA / cobertura | (b) lei de MISTURA de cor |
|---|---|---|
| pergunta | quanto do que está por baixo ainda se vê, dado o volume de tinta | duas tintas juntas dão QUE cor |
| ADR-0197 hoje | volumes conservados, Kubelka 1948 sobre $R_g$, cobertura derivada (eq. 2–4) | $\mathcal K=\sum v_iK_i$ a 3 bandas (eq. 1) — K-M de uma constante por canal |
| cura do dono_6 | **SIM** — as manchas (volume que se inverte) e os degraus (ramos) são de (a) | não: a ferrugem é ramo/inversa, não espaço de cor (F2 §0) |
| depende do espaço de cor? | não: só precisa do $R_\infty$ da tinta já misturada e do volume total | é a pergunta inteira |

(a) fica como está. Esta nota é sobre (b): **qual espaço** produz a cor $R_\infty$ da mistura que (a) depois deposita.

## 1. O que os apps fazem (pesquisa 31/10)

| App | Espaço de mistura por omissão | Mistura «pigmento» | Fonte (consultada 31/10/2026) |
|---|---|---|---|
| **Procreate** | RGB do documento (sRGB codificado; o manual não nomeia espaço) — Wet Mix = Dilution/Charge/Attack/Pull/Grade/Blur: **quantidade** e arrasto de tinta, nunca o espaço. Pintar uma cor sobre outra não mistura; a mistura vem do Smudge/Wet Mix. | **não tem** | [Brush Studio Settings](https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings) · [Paint, Smudge, Erase](https://help.procreate.com/procreate/handbook/5.1/brushes/paint-smudge-erase) — o espaço é INFERIDO (nenhuma fonte o documenta) |
| **Photoshop Mixer Brush** | RGB do documento (Wet = quanto apanha do canvas, Mix = razão canvas/reservatório) | não (pedido aberto na comunidade: «Mixbox true pigment mixing») | [Adobe: Mixer Brush](https://helpx.adobe.com/photoshop/using/painting-mixer-brush.html) · [pedido Mixbox](https://community.adobe.com/t5/photoshop-ecosystem-ideas/feature-request-mixbox-true-pigment-mixing/idc-p/15273424) |
| **Adobe Fresco** (Live Oils/Watercolor) | não documentado; azul sobre amarelo dá verde no tutorial | não declarado; um expert da comunidade: «digital colour mixing will always be a bit different» | [Fresco watercolor](https://helpx.adobe.com/ee/fresco/how-to/realistic-watercolor-painting.html) · [pedido Mixbox no Fresco](https://community.adobe.com/questions-646/feature-request-pigment-mixbox-in-fresco-308147) |
| **Clip Studio Paint** | RGB; modos Blend / Running color / Smear; a 2.0 acrescentou opção **Perceptual** (corrige o brilho da mistura) | não | [CSP Blending tools](https://help.clip-studio.com/en-us/manual_en/240_brushes/Blending_tools.htm) · [tips 8217](https://tips.clip-studio.com/en-us/articles/8217) |
| **Krita** | RGB do perfil do documento (linear se o perfil for linear); Color Smudge reescrito na 5.0/5.2 | modo **Spectral** = porte do spectral.js (MR em rascunho / builds de teste) | [cgchannel 5.2](https://www.cgchannel.com/2023/10/krita-foundation-releases-krita-5/) · [MR 1997](https://invent.kde.org/graphics/krita/-/merge_requests/1997) · [krita-artists K-M](https://krita-artists.org/t/paint-like-color-mixing-kubelka-munk/78156) |
| **Corel Painter** (Thick Paint, Real Watercolor) | não documentado («simulate blending pigments with water») | não declarado | [Corel Real watercolor](https://product.corel.com/help/Painter/540111162/Corel-Painter-en/Corel-Painter-Real-Watercolor-Brushes.html) |
| **Rebelle** | RGB; **Pigments** (Pro, desde a 5) = **Mixbox licenciado** | Mixbox (K-M num espaço latente de 4 pigmentos + resíduo) | [créditos Rebelle](https://escapemotions.com/products/rebelle/manual/copyright-credits) · [Rebelle 5](https://escapemotions.com/blog/rebelle-5-released-today-a-whole-new-level-of-painting-experience) |
| **ArtRage** | RGB; **Real Color Blending** OPCIONAL, **desligado por omissão** («far more intensive calculations»); Vitae: perfil de mistura | sim, opt-in; utilizadores: «às vezes ACRESCENTA saturação» | [ArtRage (Wikipedia + manual)](https://en.wikipedia.org/wiki/ArtRage) · [blending brushes](https://www.artrage.com/manuals/natural-media-tools/blending/blending-brushes/) |
| **MyPaint 2.0** (02/2020) | **Pigment** por omissão = média geométrica pesada de 10 bandas (código em `libmypaint`, **ISC**); compatibilidade 1.x na preferência | sim (WGM 10b) | [omgubuntu 2.0](https://www.omgubuntu.co.uk/2020/02/mypaint-2-0-released) · [linuxuprising alpha](https://www.linuxuprising.com/2019/01/free-drawing-and-painting-program.html) · `pacman -Qi libmypaint` → ISC |
| **Infinite Painter** | nada encontrado | nada encontrado | — (pesquisa sem resultado; não afirmar) |

**Leitura:** os apps de maior sucesso comercial (Procreate, Photoshop, CSP, Fresco) **misturam em RGB** e vendem a
sensação de tinta pela QUANTIDADE (carga, diluição, arrasto, wetness) — não pelo espaço. Onde existe mistura
subtractiva ela é **opcional** (ArtRage desligado por omissão, Krita experimental, Rebelle só no Pro, CSP só um
«Perceptual»). A única excepção por omissão é o MyPaint 2. O artigo do Mixbox diz-o explicitamente: «in today's
painting software, [blue and yellow] mix toward gray» ([Sochorová & Jamriška 2021, ACM TOG 40(6)](https://dcgi.fel.cvut.cz/wp-content/wpallimport-dist/publications/pdf/publications-2021-sochorova-tog-pigments-paper.pdf)).

## 2. As opções e a licença (por artefacto)

| Opção | Princípio | Licença | Utilizável num app comercial? |
|---|---|---|---|
| sRGB (gama) | lerp dos valores codificados | — | sim |
| RGB linear | lerp da luz | — | sim |
| **Oklab** | lerp no Oklab (Ottosson 2020; matrizes publicadas) | domínio público / MIT ([bottosson.github.io/posts/oklab](https://bottosson.github.io/posts/oklab/)) | sim |
| **Mixbox** | sRGB → latente $z=(c_1..c_4, r_R,r_G,r_B)$: 4 pigmentos Golden (PB15:4, PY73, PR122, PW6, base Berns 2016, 36 bandas) + resíduo aditivo; lerp no latente; LUT 256³ de 48 MB para o *unmix* (L-BFGS-B pré-calculado) | **CC BY-NC 4.0**; comercial só por contrato (`mixbox@scrtwpns.com`) — [github.com/scrtwpns/mixbox](https://github.com/scrtwpns/mixbox) | **NÃO** sem licença paga. Nem como oráculo no produto. O MÉTODO está publicado (artigo), mas a LUT e os coeficientes são o produto deles |
| **spectral.js** (R. van Wijnen) | sRGB → reflectância 38 bandas (380–750 nm) por 7 primárias (W,C,M,Y,R,G,B; LHTSS de Burns); K-M de UMA constante; concentração $c=f^2T^2L$ (L = luminância); volta por CMF + D65 | **MIT** («MIT © 2025 Ronald van Wijnen») — [github.com/rvanwijnen/spectral.js](https://github.com/rvanwijnen/spectral.js) | sim (com o aviso MIT); reimplementável do princípio |
| **MyPaint espectral** (WGM 10b) | 3 primárias espectrais (SR,SG,SB) pesadas pelo RGB → média geométrica pesada por banda → matriz T | `libmypaint` **ISC** (já copiado com aviso em `crates/ph2d-painter-brush/src/espectral.rs`) | sim |
| **K-M 3 bandas** (ADR-0197, `ph2d-paint-tinta`) | $K/S=(1-R)^2/2R$ por canal RGB linear, soma pesada, volta analítica | nosso | sim |

## 3. Medição (`misturas.py`, 31/10)

13 pares (azul+amarelo, cobalto+hansa, vermelho+azul, branco+vermelho, preto+amarelo, dois complementares, as quatro
primárias do Mixbox — ftalo, quinacridona, hansa, branco titânio —, o par do dono_6 *aproximado* (azul pálido
`120,160,220` + amarelo `240,200,40`), preto+branco, quase-iguais). Sete leis: as seis da §2 implementáveis (K-M a 10
bandas sobre as bandas do libmypaint, peso $t$, e com o peso $f^2L$ do spectral.js — é o PRINCÍPIO do spectral.js
sobre outra base espectral, não o spectral.js; a base de 7 primárias dele não foi reproduzida nesta janela).

| métrica | sRGB | RGB linear | **Oklab** | K-M 3b | WGM 10b | K-M 10b | K-M 10b L |
|---|---|---|---|---|---|---|---|
| amostras fora da gama sRGB (de 663) | 0 | 0 | 7 (máx 0,002) | 0 | 34 (0,046) | 130 (0,21) | 92 (0,21) |
| L abaixo da mais escura das duas (×100, pior par) | 9,4 (magenta+verde) | **0** | **0** | **38,9** | 7,9 | 16,5 | 16,5 |
| pares com L não monótona (de 13) | 3 | **0** | **0** | 9 | 3 | 6 | 6 |
| «lama»: croma no meio / média dos extremos (pior par) | 0,12 | 0,13 | 0,10 | 0,30 | 0,09 | 0,20 | 0,04 |
| azul+amarelo 50/50 (sRGB 8-bit) | 126,122,66 oliva-cinza | 185,156,96 bege | 118,131,123 cinza | **1,48,1 quase preto** | 48,104,83 verde-azulado | 0,69,67 petróleo | **31,154,58 verde** |
| ruído ±1/255 na entrada: ganho máx (ΔE saída/entrada) | 6,1 | 1,1 | **1,0** | **20,2** | 4,4 | 3,7 | 3,8 |
| idem, ΔE×100 máx da saída | 0,40 | 0,41 | 3,4 (preto+branco, perto do 0 do cbrt) | 2,7 | 1,5 | 1,7 | **13,3** (preto some) |
| identidade mix(c,c) — ΔE×100 máx | 0 | 0 | 0 | 4,6 (piso de R) | 0,39 (ida-volta da base) | 1,21 | 1,21 |
| deriva 20× mix(c,c) em 8 bits — ΔE×100 | 0 | 0 | 0 | 0 | 0 | 1,0 | 0,8 |
| vidrado 20×10 % (8 bits): escurece além das duas (×100) | 9,5 | **0** | **0** | 39,3 | 7,8 | 16,6 | 16,4 |
| custo numpy, ns/mistura (CPU, indicativo) | 6,6 | 40 | 69 | 49 | 159 | 192 | 229 |
| operações por mistura | 3 FMA | 6 pow | 6 cbrt + 2 matrizes 3×3 ×2 + cubo | 3 div + 3 sqrt + 6 pow | 10 log + 10 exp + 10×3 + 30×3 | 10 div + 10 sqrt + 30+30 | idem + 2 lum |
| inversa analítica (sem Newton) | sim | sim | **sim** (cubo) | sim | sim (WGM) | sim | sim |

Pior par por lei (diagnóstico em `scratchpad`, regenerável pela mesma função):
- **K-M 3b** com cores escolhidas no seletor (canais em 0) é a lei MENOS robusta: azul+amarelo → `1,48,1`,
  magenta+verde → `1,1,80`, preto+branco → `1,1,1` (o preto de $R\approx0$ tem $K/S\approx5000$ e engole tudo). Pôr
  um piso em $R$ não cura, troca o preto por lama: piso 0,03 → azul+amarelo `67,67,65`, preto+branco `67,67,67`.
  Com pigmentos de $R_\infty$ moderado (os do `lei_unica.py`) o efeito é menor, mas o seletor do artista dá canais 0.
- **K-M 10b L** (princípio spectral.js) dá o verde mais vivo (`31,154,58`), mas o peso por luminância faz o PRETO
  desaparecer (preto+branco → `252,247,255`) e é a lei mais sensível ao ruído junto do preto (ΔE 13). Laranja+azul
  também vira VERDE (`46,130,50`), o que tinta real (castanho-cinza) não faz.
- **WGM 10b** (MyPaint): o compromisso «pigmento» mais calmo (verde-azulado `48,104,83`, sai pouco da gama), mas
  não é idempotente sem o atalho `base == tinta` que o `espectral.rs` já tem — atalho = descontinuidade a 1/255.
- **Oklab**: zero escurecimento, zero não-monotonia, identidade exacta, ganho de ruído 1 por construção, deriva 0;
  o preço é o azul+amarelo dar CINZA (croma 0,018) — complementares atravessam o neutro.
- **RGB linear**: igualmente monótona e estável, mas clareia o meio (preto+branco → `188`), lido como «lavado».
- **sRGB**: o que Procreate/Photoshop dão; meio preto+branco `128`; escurece 9 % no magenta+verde.

Figuras:
- [`gradientes.png`](fotos/f2b_misturas/gradientes.png) — cada par 0..1 (esquerda = 1.ª cor), uma faixa por lei.
- [`meio_50_50.png`](fotos/f2b_misturas/meio_50_50.png) — a mistura 50/50 de seis pares por lei, com as duas cores nas pontas.
- [`vidrado_20x.png`](fotos/f2b_misturas/vidrado_20x.png) — L Oklab ao longo de 20 passadas de 10 % de B sobre A
  (8 bits): Oklab e RGB linear vão de A a B sem descer abaixo da mais escura; as K-M descem.

## 4. Recomendação

1. **UMA lei de mistura por omissão: Oklab, média PESADA PELO VOLUME** ($\bar o = \sum v_i o_i / \sum v_i$, $o$ =
   Oklab da cor escolhida). É a única medida que tem, juntas, as propriedades que o dono pediu: monotonia de L (0
   pares violam), nunca mais escura que a mais escura (0), identidade exacta, ganho de ruído 1, deriva 0 em N
   passadas, ida-volta analítica, custo trivial na GPU. Com o peso pelo VOLUME é associativa: misturar em N passadas
   = misturar de uma vez (a conservação do ADR continua a valer, agora sobre $(\sum v_i,\ \sum v_i o_i)$).
   O que o artista espera de um Procreate (meios perceptualmente a meio, sem lama escura nem lavado) é o que o Oklab
   dá; é mais robusto que o sRGB do Procreate (que escurece complementares 9 %) e mais «tinta» que o RGB linear
   (que lava o meio).
2. **azul+amarelo = verde NÃO vale o risco como lei por omissão.** Toda lei que o dá (K-M 3b, K-M 10b L, WGM) paga em
   robustez medida: escurece abaixo das duas tintas (8–39 %), sai da gama (até 0,21), amplifica ruído (4–20×), não é
   idempotente, ou faz o preto sumir. Os apps que o oferecem põem-no OPT-IN. Se o dono o quiser, a via é um
   **controlo opcional do pincel «Pigment mix» (0..1) que interpola Oklab → WGM 10b** (libmypaint, ISC, já no repo),
   como o slider Pigment do MyPaint — nunca K-M 3b e nunca Mixbox (CC BY-NC).
3. **A lei de camada (a) não muda:** a cor misturada dá $R_\infty$ por canal (Oklab → RGB linear, *clamp* `[ε,1]` —
   7 de 663 amostras tocaram 0,002 fora), $K/S=(1-R_\infty)^2/2R_\infty$ com o $S$ da transparência do pigmento, e a
   eq. (2) de Kubelka 1948 sobre $R_g$ faz a cobertura com o volume TOTAL. Os Lemas 2–3 (camada finita, diluir)
   ficam iguais; o Lema 1 passa a valer em Oklab (convexidade ⇒ $L$ entre as tintas), medido acima.

### O que o ADR-0197 deve mudar

- **Eq. (1):** trocar $\mathcal K_\lambda=\sum_i v_iK_{i,\lambda}$ (mistura K-M a 3 bandas) por: estado =
  $(V=\sum v_i,\ \bar o=\sum v_i o_i / V)$; $\mathcal K,\mathcal S$ derivam de $R_\infty(\bar o)$ e de $V$. A mistura
  deixa de ser K-M; a camada continua K-M.
- **Lema 1 / Teorema:** reescrever «por banda» como «em $L$ Oklab» para a mistura; o teorema de camada fica por banda.
- **§Alternativas / «Bandas: 3 bastam; 8 bandas ou latentes a medir»:** fechar — 3 bandas K-M é a pior lei medida
  (§3); latentes = Mixbox, licença NC ⇒ recusa. Mover para a tabela `⛔ Recusas MEDIDAS`:
  K-M 3b como mistura (escurece 39 %, ganho 20×), Mixbox (licença), K-M espectral com peso por luminância (preto some).
- **«5 leis de mistura → 1»:** as cinco (RGB, K-M `ph2d-pigment`, espectral, RYB, pré-multiplicada) colapsam em Oklab
  pesado por volume + o opcional WGM; a pré-multiplicada sobrevive só como composição de camadas (não é mistura).
- **`ph2d-paint-tinta`:** a paridade com `lei_unica.py` (fixture) tem de ser re-gravada quando a eq. (1) mudar;
  o dono_6 continua curado pela camada (as provas da F2 §3.4 são de (a)).
- **Ajuda ilustrada:** o controlo opcional, se entrar, chama-se pelo que faz («Pigment mix: blue + yellow → green»),
  com a figura `meio_50_50.png` como ilustração.

## ⛔ Recusas MEDIDAS (desta nota)

| Recusado | Porquê (número) |
|---|---|
| K-M 3 bandas como mistura por omissão | azul+amarelo `1,48,1`; L 38,9 abaixo da mais escura; ganho de ruído 20×; piso em R vira lama cinza |
| Mixbox | CC BY-NC 4.0 — comercial só por contrato |
| K-M espectral com concentração ∝ luminância (spectral.js) como omissão | preto+branco → branco; ΔE 13 a ±1/255 junto do preto; laranja+azul → verde |
| RGB linear como mistura | preto+branco → 188 (meio lavado) |
| sRGB (o do Procreate) | escurece 9 % em complementares; inferior ao Oklab em todas as métricas de robustez excepto custo |

Limites desta medição: o par do dono_6 é aproximado (cores lidas a olho do smoke, não do ficheiro); o spectral.js foi
reimplementado pelo PRINCÍPIO sobre a base de 10 bandas do libmypaint, não com a base LHTSS de 7 primárias dele —
correr o spectral.js real (MIT, `npm`) como oráculo é o passo seguinte se o opcional «Pigment mix» for aprovado;
o espaço do Procreate é inferido (não documentado).
