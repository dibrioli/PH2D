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
