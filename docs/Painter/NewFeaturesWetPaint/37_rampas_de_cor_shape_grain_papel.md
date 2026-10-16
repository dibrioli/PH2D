# 37 — Rampas de cor no Shape, no Grain e no Papel, e N secções de Grain: a avaliação (14/10)

> Pedido do dono (14/10, [prompt 36](36_PROMPT_janela_cor_do_shape_e_do_grain.md) Parte 2): *«Avalie a possibilidade
> e performance de termos color ramp em Shape Color, Grain Color e Paper Color (ainda não existe) para todos os
> modos de pintura (inclusive wet paint e water color). Avalie o custo de acrescentar novas seções Grain e Grain
> colors conforme o desejo do usuário.»* Sem código de produto. **A decisão é do dono** (lista 26, B8–B11).
>
> Réguas: `tests/custo_das_rampas.rs` (`custo_das_rampas_por_meio`, `custo_de_n_graos_por_texel`), perfil `smoke`,
> intercaladas ([MEDIR_VELOCIDADE](../../DevOps/MEDIR_VELOCIDADE.md)), `loadavg` 5,4 → 7,5. Oráculo:
> [`fixtures/krita_rampas/`](fixtures/krita_rampas/README.md) (Krita 6.0.4, corrido sob Xvfb, GPL — só oráculo).

## 1. Hoje: quem mostra e quem lê cada rampa

| Meio | Shape Color | Grain Color | Paper Color | De onde vem a cor |
|---|---|---|---|---|
| Digital | mostrada · **age** (rotas do carimbo, `stamp_route.rs`) | mostrada · age | não existe | cor do pincel; a rampa dona troca-a por texel |
| Impasto | mostrada · age (a cor; o relevo só lê o TOM B&W, `impasto.rs:242`) | mostrada · age | não existe | idem Digital; o corpo é outro plano |
| Composite | mostrada · age (cada camada passa pelas mesmas rotas — medido: o custo muda com a rampa) | mostrada · age | não existe | cor de cada camada |
| Watercolor | mostrada · age (`watercolor_accum_cor.rs::rampa_da_aguada`, indexada pelo peso da silhueta) | **oculta** | não existe | a aguada reconstrói-se da cor do pincel (`inercia::CorDaTexturaSemLeitor`) |
| Wet Paint | **oculta** (`snapshot.rs:60`) | **oculta** | não existe | o motor deposita a cor do pincel pela silhueta; o Shape só entra como TOM (B&W) |

O papel hoje é UMA cor (`paper_color: [f32; 3]`, `state.rs:433`), aplicada na composição (`papel.rs::compoe_sobre_o_papel`,
`cor_do_chao` para a aguada) — não há rampa nem leitura da altura do papel na cor.

⚠️ Achado lateral (não confirmado a fundo): o **Grain Color no Smear** aparece com o B&W travado, mas o
`grain_tone_lut` não tem chamador no Smear — candidato a controlo morto (C21).

## 2. O que o oráculo faz (corrido, não lido)

Krita 6.0.4 ([README](fixtures/krita_rampas/README.md)):

| Caso | Krita | PH2D (depois do prompt 36) |
|---|---|---|
| Ponta em *Gradient map* + textura Multiply | a textura **corta** a cobertura como numa ponta normal (α 0,936 vs 0,940); a cor segue a PONTA (Δ entre dois traços 0,000) | igual — é a lei que o prompt 36 pôs (`RampIndex::Silhouette`) |
| Ponta normal + textura em *Gradient map* | a cor vem da textura (corr. 1,000), **não** corta (α 1,000) | igual (rampa do Grain dona) |
| As duas em *Gradient map* | a TEXTURA ganha, a da ponta é ignorada (Krita tem UM gradiente, o da barra) | a do **Shape** ganha (`color_ramp_owner`) — diferença de desenho, B11 |
| Ancoragem da textura | sempre à TELA (não há «ancorar ao dab») | Tiled, View, Random, Stencil |
| Rampa do papel | **não existe**; o mais perto é a textura em *Lightness/Gradient map* | não existe |

Rebelle: não corrido nesta janela (a bancada não protege o ecrã do dono — D2); as receitas do P35 não expõem
nenhuma rampa. Procreate: só iPad, não corre aqui; o Brush Studio não tem *gradient map* por pincel (é um ajuste da
camada).

## 3. Desempenho, medido

### 3.1 Um traço (49 movimentos, raio 24, Shape Checker View, Grain Noise), ms — mínimo (mediana) de 7×2

| Meio | Grain | Off | Shape B&W | Shape Cor | Grain B&W | Grain Cor |
|---|---|---|---|---|---|---|
| Digital | Tiled | 2,7 (2,7) | 2,9 (3,0) | 3,2 (3,2) | 3,2 (3,2) | 3,2 (3,2) |
| Digital | View | 1,3 (1,3) | 1,3 (1,3) | 1,8 (1,8) | 1,8 (1,8) | 1,8 (1,8) |
| Impasto | Tiled | 6,6 (7,2) | 7,1 (7,4) | 7,0 (7,2) | 7,0 (7,2) | 7,1 (7,2) |
| Impasto | View | 5,1 (5,3) | 5,3 (5,4) | 5,5 (5,7) | 5,2 (5,7) | 5,2 (5,7) |
| Composite (2 camadas) | Tiled | 7,2 (7,8) | 7,7 (8,3) | 8,4 (9,0) | 8,4 (9,0) | 8,5 (9,1) |
| Composite (2 camadas) | View | 4,5 (5,2) | 4,4 (5,2) | 5,6 (6,6) | 5,6 (6,4) | 5,6 (6,3) |
| Watercolor | Tiled | 24,8 (26,4) | 25,2 (26,5) | 25,7 (26,8) | 24,7 (26,0) | 24,9 (26,0) |
| Watercolor | View | 24,6 (25,5) | 25,0 (26,3) | 25,8 (26,6) | 24,7 (26,0) | 25,4 (26,2) |
| Wet Paint | Tiled | 36,8 (42,7) | 37,2 (42,9) | 39,1 (42,6) | 36,6 (43,1) | 37,1 (42,5) |
| Wet Paint | View | 37,2 (43,1) | 38,0 (43,1) | 37,1 (42,6) | 36,9 (42,6) | 38,2 (42,9) |

Leitura: a rampa em cor custa **+0,5 ms por traço** no Digital (Tiled +19 %: o Grain sai do cache da tela para o núcleo
por pixel; View +38 %: o carimbo cinzento vira o carimbo colorido) e **+1,1 ms** no Composite (por camada). Watercolor
e Wet Paint: dentro do ruído (no Wet Paint a rampa nem é lida). Nenhum meio passa de 1 ms a mais por traço.

### 3.2 N grãos, por texel (µs por dab de raio 48 ≈ 7 238 texels) — mínimo (mediana) de 9×5

| Grain | Mapping | N=0 | N=1 | N=2 | N=4 |
|---|---|---|---|---|---|
| Noise | Tiled | 5 | 401 | 792 | 1 600 |
| Noise | View | 5 | 434 | 860 | 1 739 |
| Grain | Tiled | 5 | 404 | 798 | 1 611 |
| Voronoi | View | 5 | 384 | 930 | 1 808 |

**~55 ns por amostra por texel**, linear em N. Onde isso pesa: só nas rotas POR PIXEL (Grain Tiled com rampa, Rake,
Random, Accumulate desligado, Continuous). As rotas em cache pagam o Grain uma vez: o View assa-o no carimbo
(custo zero por dab, qualquer N); o Tiled guarda-o num cache da tela por traço — **um cache por grão**, `W×H` bytes
cada (16 MB a 4096²).

## 4. O que cada item pede

**Shape Color no Wet Paint.** O motor recebe UMA cor por dab (a do pincel) e deposita-a pela silhueta; a rampa
pede uma cor POR TEXEL no depósito do pigmento (`ph2d-wet-paint`, a mistura e a secagem leem a cor depositada). É
mudança no motor, não no painel — o preço real só se sabe medindo o depósito com cor por texel. Sem isso, mostrar a
secção seria um controlo mudo.

**Grain Color na Watercolor e no Wet Paint.** O mesmo problema mais um: nos dois o Grain é o PAPEL que segura o
pigmento (granulação; a água), não um carimbo — colorir por ele mudaria a física, não a cor do pincel.

**Paper Color (nova), em todos os meios.** A forma barata e verdadeira: a cor do papel deixa de ser uma cor e passa a
ser `rampa[altura do papel]` — o papel com vales mais escuros (ou de outra cor), como papel tingido. Corre na
COMPOSIÇÃO (`compoe_sobre_o_papel` / `cor_do_chao`), uma consulta de LUT por píxel visível, **não por carimbo**: o
custo não depende do traço. Precisa do mapa de altura do papel na composição (existe quando o Relief está ligado).
Na Watercolor/Wet Paint o chão já entra no modelo óptico pela `cor_do_chao` — passa a ler a cor por texel.

**N secções de Grain (o artista acrescenta).**
- Dados: `BrushSpec` é `Copy`; `TextureSettings` tem ~64 B (4 hoje: texture, shape, edge_flow, paper). Com teto fixo
  (`[TextureSettings; 4]` + rampas) continua `Copy` (+~200 B por pincel); `Vec` quebraria o `Copy` em toda a crate.
- Formato: o preset (`ph2d-brush-presets`, `SCHEMA = 1`) leva `texture` em `CAMPOS_DO_PINCEL`; os grãos extra entram
  como campo novo com omissão vazia. As rampas hoje NÃO entram no preset (vivem no estado da ferramenta) — ou passam a
  entrar (degrau de formato), ou um grão extra salvo perde a cor. Projeto: `PROJECT_SCHEMA` (postcard posicional) só se
  o estado do pincel for persistido no `.ph2dproj` (não confirmado).
- Painel: `SECCOES_MOVEIS` é `[NodeId; 8]` fixo; uma secção REPETÍVEL pede ids por índice, i18n, `populate_sections`,
  `inercia::Dependente` (por grão), e os censos `as_seccoes_arrastam_e_tem_tema`, `cada_nome_deste_painel_cabe_na_coluna_da_seccao`,
  `censo_dos_controlos`, `quantas_entradas_tem_cada_painel`.
- Motor: cada rota que hoje amostra UM Grain amostra N (por pixel, o carimbo assado, o cache da tela, a máscara,
  Smear/Blur/Clone, o relevo do Impasto, a granulação da Watercolor, a água do Wet Paint). Custo: §3.2.
- Preço estimado: ~1 200–1 800 LOC (motor ~400, rotas ~300, painel ~400, formato e macros ~200, portões ~400) e
  ~10 portões novos (um por rota, N = 2 contra N = 1 com o 2.º neutro ao byte, o formato ida e volta).

## 5. Decisão (do dono)

| # | Item | Faz? | Quanto custa | O que se perde se não fizer |
|---|---|---|---|---|
| B8 | Paper Color (rampa pela altura do papel), os 4 meios | **recomendo: fazer** | baixo: uma LUT por píxel na composição; ~300 LOC + portões | papel tingido/com vales coloridos só por camada pintada à mão |
| B9 | Shape Color no Wet Paint | **feito (prompt 41, 16/10)** — medido primeiro: a cor por célula num plano da janela, pousada com a sujidade da ponta | medido: ×1,06–1,11 o custo do traço; a rampa sobrevive à água nos 12 pincéis de fábrica | o Wet Paint fica sem pontas coloridas (hoje: oculta, honesto) |
| B10 | Grain Color na Watercolor e no Wet Paint | recomendo: **não fazer** | alto e muda a física do papel/água | nada que o Rebelle tenha |
| B11 | N secções de Grain | recomendo: **2 grãos com teto fixo**, se o dono quiser o efeito | ~1 500 LOC, +0,4 ms por dab de raio 48 por grão nas rotas por pixel; zero nas assadas | o 2.º grão hoje faz-se com a camada de Textura |
| B12 | As duas rampas em cor ao mesmo tempo | hoje o Shape ganha; o Krita dá a vitória à textura | trocar é uma linha (`color_ramp_owner`) | — (só a escolha da lei) |

**Recomendação:** o B8 (Paper Color) é o único que entrega cor nova em TODOS os meios sem tocar no custo do
carimbo. O B11 só vale com um teto pequeno e fixo. O B9 abre-se com uma régua, não com código. O B10 não.
