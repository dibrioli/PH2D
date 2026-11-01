# E9 — o portão-mãe (a varredura F0 sobre o motor, nas duas leis × dois papéis)

> Leitor: a próxima LLM. Plano: [F4_2 §5 e §7 E9](F4_2_plano_wet_paint.md) · régua: [F0_varredura.md](F0_varredura.md)
> (§3.0 F0b, §4 limiares) · [ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md) §4 item 1.
> ⛔ Dono 31/10: o portão-mãe corre sobre o **papel texturizado PADRÃO** do app; o padrão é o que decide.

## 1. O que mudou

- **Setter do produto** `PainterTool::set_wet_lei_da_tinta(LeiDaTinta)` + leitor `wet_lei_da_tinta()`
  (`crates/ph2d-tool-painter/src/tool/paint/wetpaint.rs`, ao lado de `set_wet_relogio_fixo`). Cria a sessão se
  falta (`ensure_wet_session`) e escreve `engine.sim.tinta`; sobrevive a `wet_preset_aplicar` (a sonda afirma-o
  a cada passada: `assert_eq!(t.wet_lei_da_tinta(), Some(cena.lei))`).
  - É `pub(crate)`: não há consumidor fora da crate (sem UI nesta etapa). Quando a app o usar, passa a `pub` e
    entra em `macro_track/portas_internas.rs` (o gate `toda_porta_do_painter_tem_decisao` varre `    pub fn`).
  - ⚠️ **Aberto:** uma sessão NOVA (undo que mata a sessão, guarda do canvas, troca de grid ratio) nasce `Velha`.
    O gancho é uma linha em `ensure_wet_session` (`wetpaint/authored_actions.rs`, fora dos paths da E9):
    guardar a lei em `WetPaintState` e `engine.sim.set_tinta(..)` ao nascer. Na sonda não acontece (o assert passa).
- **Eixo PAPEL** na sonda (`sonda_varredura_oleo.rs`, `enum Papel { Padrao, Liso }`): `PH2D_SONDA_PAPEIS=padrao,liso`
  (omissão: os dois). O **padrão** é o do *New Image… → Textured Paper* (`PAPEL_TEXTURIZADO` em
  `crates/ph2d-tool-painter/src/tool/papel_texturizado.rs`: Cold Press `#E6D9CC`, relief 0,5, roughness 0,5,
  tooth 1), aplicado pelo MESMO caminho do produto: a sprite nasce `pixeis_do_papel_texturizado(W, H)` e abre como
  documento por `abre_a_sprite_nova` (o que `bind_document_corpo` chama) — que reconhece a sprite nascida, abre o
  fundo vazio e serve o papel. `liso` = a tela transparente da F0. O laço do papel fica por fora do das leis (as leis
  continuam o mais interno). TSV: coluna nova `papel` (`padrao|liso`, a última); a antiga coluna `papel` (Depth do
  `PH2D_SONDA_PAPEL` da F0b) chama-se agora `papel_depth`. PNG: sufixo `_<lei>_<papel>`.
  - O papel não entra no `canvas_rgba` (compõe-se só na saída, `compoe_sobre_o_papel`): a régua continua a ler a
    camada sobre o creme `236,226,206`. O papel entra pela física (o dente/relevo do papel no depósito e no filme).
- **O veredicto** (`fotos/f0_varredura/resumo_f0.py`; a tabela antiga «pior por pincel» fica em `--pior`): por
  célula e por lei, os limiares finais da F0 §4 (F0b) nas colunas `*_plano` — `manchas_azul_plano ≤ 1 %`,
  `km_p99_plano ≤ 8`, `km_max_plano ≤ 20`, `ferrugem_plano ≤ 0,05 %`, `furos = 0` só se a mesma célula a pressão 1
  tem `furos < 1 %`, `degraus_y_plano ≤ N−1`, `degraus_x_plano ≤ 2`. Depois a tabela «Velha × Pigmento: passam /
  total, por papel» e a lista das células que reprovam na Pigmento com a(s) métrica(s). Célula com `km_p99_plano`
  NaN (nada com alfa ≥ 0,9) conta «sem medida», nunca «passa». Nenhuma coluna `*_plano` faltava (`furos_pct` lê o
  alfa, não depende da luz).
- **Mosaico** (`mosaico_f0.py`): um PNG por pincel × papel × lei (`<slug>_<papel>_<lei>.png`), legenda pelo plano.
- **`corre_f0.sh`**: exporta `PH2D_SONDA_LEI=velha,pigmento` e `PH2D_SONDA_PAPEIS=padrao,liso` por omissão;
  `PH2D_F0_TIMEOUT` (omissão 14400 s por grupo; o 1500 antigo cortava a grelha a 564/810).
- **Teste** `portao_mae_f0_lei_pigmento_minimo` (`wetpaint_preset/portao_mae_f0.rs`, filho da sonda; os limiares
  em `const`, os MESMOS do `resumo_f0.py`): Thick Impasto + Watercolor/Fluid/Wet Round, P600/W0,5/pr1/N3, papel
  padrão, as duas leis; afirma os limiares na Pigmento (e que a célula cobre: `km_p99_plano` finito).
  `#[ignore = "E9: liga quando a E6b/E8 fecharem"]` — hoje VERMELHO (abaixo).

## 2. A corrida de prova (1 célula × 2 leis × 2 papéis)

`Oil/Flat/Thick Impasto` P600/W0,5/pr1/N3, pausa 300 (o filtro `sonda_varredura_oleo` também apanha o teste
mínimo, que correu junto por `--ignored`; 38 s de parede, 7–13 s por célula, a Pigmento ~35 % mais lenta):

| papel | lei | manchas_azul_plano | km_p99_plano | km_max_plano | ferrugem | furos | degraus y/x | veredicto |
|---|---|---|---|---|---|---|---|---|
| padrão | velha | 43,94 | 128,7 | 216,6 | 0 | 0 | 9/0 | reprova |
| padrão | pigmento | 41,59 | 224,6 | 235,0 | 0 | 0 | 1/0 | reprova (manchas, km) |
| liso | velha | 43,99 | 129,0 | 217,0 | 0 | 0 | 9/0 | reprova |
| liso | pigmento | 41,65 | 224,6 | 235,0 | 0 | 0 | 2/0 | reprova (manchas, km) |

Teste mínimo, Watercolor/Wet Round padrão: velha manchas 0,02 · km_p99 141 · ferrugem 1,12 · furos 0,91 · degraus 11;
pigmento manchas 15,58 · km_p99 168,8 · ferrugem 0 · furos 0 · degraus 2.

Os 4 PNGs (com luz):
- `velha_padrao`: azul em cima, orla verde-escura fina e serrilhada, amarelo com o relevo do empasto em cristas horizontais.
- `velha_liso`: igual ao padrão a olho (1 337 píxeis > 3 níveis diferentes, máx 118, nos grumos do relevo).
- `pigmento_padrao`: o azul passa a uma banda **branco-creme** (sem verde nenhum) antes do amarelo pálido — a mistura azul+amarelo não sai na curva K-M (km_p99 224).
- `pigmento_liso`: igual ao padrão a olho.

⚠️ **Achado — o «plano» não é plano nesta célula.** Os PNGs `_plano` mostram o MESMO relevo sombreado (cristas,
grumos) nas duas leis; na Pigmento o plano é igual à luz AO BIT (Δ 0), na Velha Δ ≤ 49–51. `render_plano` (Real Oil
`None` + Emboss 0) não tira a sombra que vai para o `canvas_rgba` com P600 (p99 altura 57–71). Até isto se curar, o
`km_*_plano` e `manchas_azul_plano` das células de empasto medem SOMBRA, e o portão reprova-as por isso. Quem tem
de olhar: a porta do relevo do render (`wp/render.rs:520-562`, o emboss do gradiente da massa e dos fios) e a da
Pigmento (`wp/render/tinta.rs`, E5) — fora dos paths da E9.

## 3. A grelha inteira (o coordenador corre)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib --no-run   # o binário
PH2D_PRAZO=14400 bash scripts/ph2d-run.sh bash docs/Painter/Arquitetura/fotos/f0_varredura/corre_f0.sh /tmp/e9full
python3 docs/Painter/Arquitetura/fotos/f0_varredura/resumo_f0.py /tmp/e9full/varredura.tsv
python3 docs/Painter/Arquitetura/fotos/f0_varredura/mosaico_f0.py /tmp/e9full/varredura.tsv /tmp/e9full/celulas /tmp/e9full/mosaicos
```

Tamanho: 15 pincéis × 54 × 2 leis × 2 papéis = **3 240 células**. A F0 mediu ~22 células/min em 8 processos
(564 em 25 min); com a Pigmento ~35 % mais lenta, **≈ 2,5–3 h** de parede (o `PH2D_PRAZO` cobre 4 h). Pela fila:
`bash scripts/fila-pesada.sh pôr '<a 2.ª linha acima>'`. Só o papel padrão (o que decide) = metade.

## 4. Aberto

1. O teste mínimo está `#[ignore]` até a E6b/E8 fecharem; tirar o `ignore` é a ligação.
2. O «plano» com relevo (§2 ⚠️) — sem ele, nenhuma célula de empasto passa, em nenhuma lei.
3. Uma sessão nova nasce `Velha` (§1) — o gancho em `ensure_wet_session`.
4. `portao_ortogonalidade_no_motor` (linha E9 do F4_2) não foi pedido nesta etapa.

## 5. Diagnóstico da 1.ª corrida (31/10)

Medição: `diagnostico_e9_pigmento_por_ponto` (`wetpaint_preset/portao_mae_f0.rs`, `#[ignore]`, corre à mão com
`--ignored --nocapture`): a mesma célula (Thick Impasto P600/W0,5/pr1/N3, papel padrão e liso), coluna x = 180, e por
ponto o píxel composto, as somas da célula, o píxel com `relevo`/`altura` a 0 e o píxel ainda sem `Grid::oleo`.
Os dois papéis dão os mesmos números (o papel não entra no `canvas_rgba`).

### 5.1 Amarelo pálido e faixa creme na Pigmento — mecanismo: o MEIO por escrever (`m = 0` = água), nem (a) nem (b)

| ponto | somas (Oklab L,a,b ÷ v) | píxel sem relevo nem óleo | esperado (a cor da soma, opaca) |
|---|---|---|---|
| y 100 (miolo azul) | filme 2,43 µm + aberto 1,22 µm, ambos `0,518 −0,023 −0,210` (= o azul `40,90,220`: `0,517 −0,021 −0,205`) | `58,126,235` (L ≈ 0,60) | `40,90,220` (L 0,517) |
| y 128 (faixa) | filme 2,58 µm `0,759 −0,015 0,060` · aberto 0,94 µm `0,596 −0,020 −0,121` | `187,196,187` | ~`176,175,140` |
| y 154 (miolo amarelo) | filme 6,55 µm `0,847 −0,013 0,157` · aberto 0,96 µm `0,764 −0,015 0,065` | `240,219,108` | ~`240,203,55` |

- **(a) a lei escolhida — rejeitada como causa.** O ponto médio Oklab de `40,90,220` e `245,205,25` é
  `0,687 −0,017 −0,017` = sRGB `140,158,166` (cinza-azulado, L 0,687); sobre o creme `236,226,206` a 50 % de α seria
  mais escuro ainda. A faixa medida no PNG plano tinha L **0,88–0,92**, acima até do amarelo (0,858): clara demais para
  qualquer ponto do segmento. As SOMAS estão certas (miolo azul = o azul ao 3.º decimal; a faixa tem a mistura Oklab
  esperada); o que clareia é o ecrã. Que azul+amarelo não dê verde É a lei (F2b §4) — isso não é defeito.
- **(b) cobertura/α — rejeitada.** α = 255 em todo o eixo, nos dois papéis (o `liso` é tela transparente, logo o α é
  o da tinta). Volumes 1,2–7,5 µm contra 0,03–0,2 µm que a E6a mediu para cobrir 0,699 (C(3) do azul/amarelo): a
  camada está muito acima do esconder; o creme não se vê.
- **(c) um caminho clareia — confirmado, mas não o emboss nem o óleo:** tirar o relevo muda poucos níveis (y 100:
  `77,164,255 → 78,167,255`), tirar o óleo escurece um pouco (`→ 58,126,235`) e o azul continua claro. O resto é a
  ÓPTICA: `ecra_da_celula` (`wp/render/tinta.rs:16`) chama `s_fator(c.m, molhado)` com `c.m = 0` (decisão 4 do
  coordenador: o meio fica 0 até à E10) — e na lei de camada `m = 0` é **água**: `n = 1,33` contra o `N_OLEO = 1,48`
  onde o `R∞` da cor escolhida está normalizado, ⇒ `s_fator = ((1,75−1,33)/(1,75+1,33))² / ((1,75−1,48)/(1,75+1,48))²
  = 2,66` (e no fecho seco, `drying/tinta.rs:198`, `n = 1,25` ⇒ **3,98**). O S 2,66× maior divide o K/S: a tinta
  espessa tende a `R∞' = 1 + K/S' − √(K/S'² + 2K/S')` com `K/S' = K/S ÷ 2,66`. Previsto para o azul:
  `66,126,233`; medido `58,126,235`. Para o amarelo o canal B sobe de `25` para ~`108` (medido) — o «amarelo pálido» —
  e a faixa (mistura cinza-azulada) passa a creme-esbranquiçada.
- **Cura: NÃO feita nesta janela — é a E10 (escrever o meio).** O mecanismo está em `m` por escrever, não na lei de
  mistura nem no ecrã: com `m = 1` (linhaça) `s_fator = 1` ao bit e o ecrã devolve a cor escolhida. Mudar o NEUTRO
  para `m = 1` muda também a secagem (`drying/tinta.rs:183`: `m ≤ 0` não oxida; `τ(m) = 3600/m`) e o critério do
  `M_UM` (calibrado em água, `s_fator(0, true)` em `calibracao_de_m_um`: com `s_fator = 1` o branco a 600 cobriria
  menos, e o `M_UM` teria de baixar ~2,66×). É uma decisão do coordenador (decisão 4) ⇒ **recomendação**: a E10 entra
  ANTES da grelha inteira, com o depósito a escrever `m` pelo meio do pincel (óleo ⇒ 1, aguarela/água ⇒ 0, pesado
  pelo volume em `transfere`), e o `M_UM` re-calibrado no meio do pincel de fábrica.

### 5.2 O «plano» não tirava o relevo — mecanismo: o relevo e o óleo são da CÉLULA, a sonda desligava a CAIXA

`render_plano` fazia `set_wet_oleo_real(None)` + slider `emboss = 0`. Mas desde 28/10 (dono: «o Real Oil e o relevo são
do traço») o render lê `Grid::oleo` (o `oleo_do_composite` liga a óptica se ALGUMA camada tem o campo, nunca pela
caixa — só o `cfg(test)` `lei_da_caixa` a obedece) e o emboss de `Grid::relevo[i]`/`altura` (`render.rs` `emboss_k`),
nunca do slider. Nenhuma das duas escritas chegava ao render, **nas duas leis**. Na Pigmento o plano saía igual à luz
ao bit; na Velha a diferença de 49–51 níveis não era a luz: era o `Ecra` da CAIXA (`p.ecra`/`com_oleo`), que só o
ramo Velha lê (`cor_e_alfa`), e o ramo Pigmento não (por desenho: a cor é a das somas). O `pigment_row` está certo —
o defeito era da régua.
**Cura** (`sonda_varredura_oleo.rs` `render_plano`): tira da grade, só para esse composite, `relevo` (a zeros),
`altura` e `oleo` de cada camada, compõe, e repõe-nos (o motor que pinta a seguir é o mesmo).

### 5.3 A régua da mistura da lei activa — `mix_*`

`mix_p99`/`mix_max` (e `_plano`): distância (níveis sRGB) do píxel coberto (α ≥ 0,9, já sobre o creme) ao SEGMENTO
Oklab `40,90,220`→`245,205,25` (201 pontos, `oklab`/`srgb_de_oklab` na sonda, as matrizes do Ottosson), cada ponto
composto sobre o creme com o α do píxel. Colunas novas no fim do TSV (`mix_p99 mix_max mix_p99_plano
mix_max_plano`); `km_*` fica como diagnóstico. Veredicto: Pigmento por `mix_p99_plano ≤ 8`, `mix_max_plano ≤ 20`
(`resumo_f0.py` `mistura(r)`; TSV antigo sem `mix_*` cai na km); Velha continua pela km. O mesmo no teste
`portao_mae_f0_lei_pigmento_minimo` (`reprova(.., lei)`; a célula «cobre» = `mix_p99_plano` finito).

### 5.4 A lei sobrevive à sessão e não fica velha

- `WetPaintState::lei_da_tinta` (`wetpaint.rs`) guarda a lei; `set_wet_lei_da_tinta` escreve-a e
  `ensure_wet_session` (`wetpaint/authored_actions.rs`) passa-a ao motor novo — a sessão que nasce depois de
  desfazer/guarda/grid ratio herda-a. Teste `a_sessao_nova_herda_a_lei_da_tinta` (verde).
- `Engine::set_lei_da_tinta` (`wp/painter/lei_da_tinta.rs`, novo; `painter.rs` só ganha o `mod`): ao SAIR de
  `Pigmento` esvazia `Grid::tinta` em todas as camadas; ao voltar o `garante` migra da massa. Teste
  `voltar_a_lei_velha_e_regressar_nao_le_somas_velhas` (`wp/painter/lei_da_tinta_tests.rs`: Pigmento → Velha (campo
  vazio) → traço amarelo + passo da água na lei velha → Pigmento; toda célula com tinta tem `filme·M_UM == susp` e
  `aberto·M_UM == sett`, rel. 1e-3) — verde.

### 5.5 A célula re-corrida (antes × depois desta secção)

Mesmo comando da §2 com `PH2D_SONDA_LEI=velha,pigmento PH2D_SONDA_PAPEIS=padrao,liso`, P600/W0,5/pr1/N3; a «luz»
não mudou ao bit (km 216,6/128,7 e 235,0/224,6 iguais — a régua plana repõe a grade). Plano:

| papel | lei | manchas_azul_plano | km_p99 / km_max plano | mix_p99 / mix_max plano | degraus y/x | veredicto |
|---|---|---|---|---|---|---|
| padrão | velha | 43,94 → **0,00** | 128,7/216,6 → **28,4/41,5** | — → 172,1/185,0 (não é a régua dela) | 9/0 | reprova (km, degraus) |
| padrão | pigmento | 41,59 → **10,28** | 224,6/235,0 → 165,9/170,0 (diagnóstico) | — → **64,6/157,2** | 1/0 | reprova (manchas, mix) |
| liso | velha | 43,99 → **0,00** | 129,0/217,0 → **28,6/41,4** | — | 9/0 | reprova (km, degraus) |
| liso | pigmento | 41,65 → **10,37** | 224,6/235,0 → 165,9/169,9 | — → **64,1/157,2** | 1/0 | reprova (manchas, mix) |

`portao_mae_f0_lei_pigmento_minimo` (`--ignored --exact`): continua VERMELHO, agora pelas métricas certas —
Thick Impasto pigmento `manchas_azul 10,28 · mix_p99 64,6 · mix_max 157,2`; Wet Round pigmento `manchas_azul 15,58 ·
mix_p99 87,6 · mix_max 106,6`; ferrugem 0, furos 0, degraus 1–2 nas duas. Foto:
[`depois_do_diagnostico.png`](fotos/e9/depois_do_diagnostico.png) (cima Pigmento, baixo Velha; padrão | liso, com luz).
No plano (sem relevo) a Pigmento mostra a causa da §5.1 a olho: azul `~70,130,235` e amarelo pálido; o `mix_p99`
de 64 é a distância desse ecrã claro ao segmento, e as `manchas_azul` 10 % são PROVAVELMENTE o azul molhado×seco com `s_fator`
diferente (2,66 molhado × 3,98 fechado seco) — não medido célula a célula. Orla fina de amarelo/ciano saturado na Pigmento: não investigada.

**Recomendação:** NÃO correr a grelha inteira agora. Com o meio por escrever, toda célula Pigmento mede o `s_fator`
da água (a §5.1), não a tinta: 3 h de máquina para confirmar um defeito já medido. Primeiro a E10 (escrever `m` no
depósito pelo meio do pincel, re-calibrar `M_UM`), re-correr esta célula (≈ 1 min), e só com ela perto do limiar a grelha.

### 5.6 Aberto

1. O meio (§5.1) — E10, decisão do coordenador (decisão 4).
2. A orla saturada da Pigmento no plano.
3. `diagnostico_e9_pigmento_por_ponto` fica `#[ignore]` como instrumento (corre-o depois da E10: o píxel sem relevo
   nem óleo deve dar a cor das somas).

## 6. Depois da lei do meio (31/10): a cor escolhida é o masstone SECO

**A lei** (coordenador, F4_1 §5): `𝒦 = v·((S·s(m, seco))·q(R∞))`, `𝒮 = s(m, molhado)·(v·S)` — `ph2d-paint-tinta`
`lei.rs` `optica_de_rinf`/`optica(soma, m, molhado)`, `transfere.rs` `optica_celula(.., m, molhado)`/`fecha(rg, aberto, m)`
(cura seca); chamadores `wp/render/tinta.rs` (`ecra_da_celula`), `wp/drying/tinta.rs` (`fecha_celula`). Seca em todo `m`
o masstone é a cor (0,000 níveis); molhada a aquarela escurece; `m = 1` igual ao bit à forma antiga. `M_UM = 1374` fica
(E6a: o branco tem `𝒦 = 0`). Oráculo e fixture mudaram igual; paridade verde (max |ΔR| 1,41e-6).

### 6.1 A célula re-corrida (antes = §5.5, depois = esta lei)

Mesmo comando (`PH2D_SONDA_OUT=<scratchpad>/e9c PH2D_SONDA_PINCEIS="Oil/Flat/Thick Impasto" PH2D_SONDA_PIGMENTOS=600
PH2D_SONDA_AGUAS=0.5 PH2D_SONDA_PRESSOES=1 PH2D_SONDA_NS=3 PH2D_SONDA_LEI=velha,pigmento PH2D_SONDA_PAPEIS=padrao,liso
… --lib sonda_varredura_oleo -- --ignored --nocapture`, 49 s). Plano; a Velha não mudou ao bit (28,4/41,5 · 9/0).

| papel | lei | manchas_azul_plano | mix_p99 / mix_max plano | km_p99 / km_max plano (diag.) | ferrugem | furos | degraus y/x | veredicto |
|---|---|---|---|---|---|---|---|---|
| padrão | pigmento | 10,28 → **9,18** | 64,6/157,2 → **51,1/157,2** | 165,9/170,0 → 114,9/166,4 | 0 | 0 | 1/0 → 2/0 | reprova (manchas, mix) |
| liso | pigmento | 10,37 → **9,25** | 64,1/157,2 → **51,3/157,2** | 165,9/169,9 → 114,9/166,4 | 0 | 0 | 1/0 → 1/0 | reprova (manchas, mix) |
| padrão | Wet Round pigmento (teste mínimo) | 15,58 → **16,15** | 87,6/106,6 → **84,6/111,2** | — | 0 | 0 | 1–2 | reprova |

`portao_mae_f0_lei_pigmento_minimo` (`--ignored`): **continua VERMELHO** — Thick Impasto `manchas_azul_plano 9,18 ·
mix_p99 51,1 · mix_max 157,2`; Wet Round `16,15 · 84,6 · 111,2`. Foto: [`depois_da_lei_seca.png`](fotos/e9/depois_da_lei_seca.png)
(cima Pigmento, baixo Velha; padrão | liso, com luz). No plano o azul é o azul e o amarelo deixou de ser creme: é um
amarelo-ocre (a mistura Oklab com o azul que o amarelo arrasta, §5.1 — a lei, não defeito).

### 6.2 O que ainda reprova — mecanismo medido: a sonda lê a tinta MOLHADA

`diagnostico_e9_pigmento_por_ponto` (coluna x = 180, papel padrão): **`film > 0` nos 8 pontos** (1,5–5,7) — a régua
fotografa o traço molhado, e a lei nova escurece-o DE PROPÓSITO (m = 0 ⇒ `s(0, molhado) = 2,66 < s(0, seco) = 3,98`).
A cor SECA das mesmas somas (Oklab ÷ v → sRGB) contra o píxel medido «sem óleo», distância ao segmento azul→amarelo:

| y | seca (das somas) | d_seg | molhada (píxel sem óleo) | d_seg |
|---|---|---|---|---|
| 100 | 36,90,223 | 5,4 | 28,76,216 | 18,9 |
| 112 | 42,92,218 | 0,7 | 33,78,211 | 16,6 |
| 124 | 107,141,187 | 1,4 | 92,126,175 | 18,8 |
| 128 | 156,167,155 | 1,3 | 141,153,141 | 22,3 |
| 132 | 188,181,129 | 1,3 | 176,169,114 | 21,5 |
| 140 | 218,194,97 | 3,0 | 210,183,83 | 16,5 |
| 154 | 231,200,73 | 1,5 | 226,189,61 | 14,0 |
| 170 | 228,199,78 | 2,0 | 223,188,66 | 14,3 |

(Script: `seg.py` no scratchpad da sessão; ignora α e o creme.) **Seca, cada ponto fica a ≤ 5,4 níveis do segmento**
(o limiar é `mix_p99 ≤ 8`); molhada, 14–22. O `mix_p99 51` mede o escurecimento do molhado (+ a orla, `mix_max 157`
igual antes e depois: é a orla saturada da §5.6 item 2, não a lei).

- **Thick Impasto é ÓLEO** e a célula tem `m = 0` (o meio por escrever, decisão 4 até à E10): com `m = 1`, `s(1, molhado)
  = s(1, seco) = 1` e o molhado é o seco ao bit — este pincel deixava de escurecer molhado. A cura dele é a E10.
- **`manchas_azul` 9–16 %: NÃO resolvido.** Mede o azul SOZINHO logo após a passada (`le_azul`, `canvas_rgba`), também
  molhado. A hipótese «molhado × seco entre vizinhas» (o `molhado` é binário em `film > 0`, logo a fronteira de
  evaporação vira um degrau de ~5 níveis de luminância no azul) não foi medida célula a célula: a sonda vive em
  `ph2d-tool-painter` (fora destes paths). Um instrumento no motor (`Engine` com o pincel por omissão) cobriu 29 células e
  secou-as todas em 300 passos — não representativo, retirado.
- **Proposta (do coordenador):** (1) a E10 escreve `m` pelo meio do pincel (óleo ⇒ 1) ANTES da grelha; (2) o portão lê
  a cor da lei depois de SECO — na sonda, um passe `film ← 0` só para a régua (como o `render_plano` tira relevo/óleo e
  repõe) ou correr a secagem até `film = 0`; (3) para separar as manchas, gravar em `le_azul` o par (luminância, `film > 0`)
  por píxel e contar as manchas só nas molhadas, só nas secas e na fronteira. Sem isto, nenhuma célula de aquarela passa
  o `mix_p99 ≤ 8` molhada — por desenho da lei.
