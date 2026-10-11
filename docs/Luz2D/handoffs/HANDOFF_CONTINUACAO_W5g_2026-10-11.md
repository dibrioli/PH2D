# Handoff de continuação — Luz 2D, W5g: o F5, o miolo das formas (2026-10-11)

> Regras R1–R9 da linha: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D`, base
> `main` `8f20c60de`; W5d–W5f por integrar. Esta janela: UM commit `wip` (dois portões antigos VERMELHOS —
> §3). Nada integrado; push e integração só por ordem do dono.

## 1. O que o dono via e o que era (a foto com setas, `fotos_W5f/dono_cena7_degraus_na_borda.png`)

Nos pontos TANGENTES dos discos, a fronteira acesa/escura do interior saía em degraus de ~4 px. Medido (sondas de
cor temporárias no `c3_fina`, fotografadas e retiradas), dois mecanismos na lei do interior (o texel de dentro lê
a luz do exterior mais próximo):

1. **O ponto de fronteira da busca é quantizado.** Fundo no interior, o candidato vem da semente do JFA da CÉLULA
   (`CELULA_PREENCHE` = 4 texels finos): o cruzamento rumo a ela salta ao longo da face, constante por célula ⇒
   os blocos de 4 px. ⛔ Projetar `P0` na reta AAEDT do texel NÃO serve longe da borda (o pé cai num texel cheio
   ou vazio: sonda = recusado em quase todo o miolo). ⛔ Secção áurea do ângulo que minimiza a distância: perto
   do centro a distância é chata e o ruído da borda escolhe (o terminador saiu torto — foto, recusado).
   ✅ **O pé** (`Config::pe_na_borda`, `c3_pe.wgsl`): o raio de P0 pela normal de fora até a saída, 6 passos,
   enquanto se aproxima. ⛔ Com a normal do Sobel (`c3_borda`) o pé salta ~ε·R (o Sobel treme ~2°): degraus de um
   lado só dos discos. ✅ **Normal SECANTE** (`normal_secante`): de um ponto 3 texels dentro, raios a ±atan(2/3)
   saem a ~2 texels de q; a corda é a tangente. Canto (q a > h/4 da corda) ⇒ zero, quem chama fica com o que tinha.
2. **A lâmpada vista do ponto rente** marchava raios RASANTES, ora livres ora tapados com a fase da grade.
   ✅ `Config::vis_na_face` (`visibilidade_na_face`): a face corta a abertura na tangente (conta exata), a marcha
   parte de `AFASTA_NA_FACE` (1,5) × max(texel da captura, texel fino) para fora. No canto, a marcha antiga.
   ⚠️ Sozinha ela PIORA (a normal do ponto da busca não é a da face — `face` na rodada); só vale com o pé.

## 2. A rodada (`medida_luz2d_w5g`, RTX 5060 Ti, smoke, `load` 3–5)

Um zoom por OITAVA (dentro dela a grade é a mesma no mundo: 0,8 = 1; 1,3 = 1,7 = 2,1 — medido). Polígonos da cena 5
× 3 oitavas × 3 fases × 7 anéis (0,35 · 0,5 · 0,65 · 0,8 · 0,9 · 0,97 · fora), o PIOR:

| | cruzamentos a mais | desvio do terminador | excesso de variação |
|---|---|---|---|
| W5f (busca) | ×2 | 4,2 tx | +1,63 |
| pé+face | **×0** | **2,3 tx** | **+0,58** |

Disco de IMAGEM, anel 0,97: zoom 1 ↔0,7 → 1,1 (TV +0,33 → +0,02); zoom 1,5 **×4 ↔10,8 → ×0 ↔1,5**; zoom 2,8 ×2
↔13,7 → ×2 ↔4,2 (a textura ampliada — o ABERTO 1 da W5d, outra causa). Custo da C3 (mínimo de 7, cenas 1–6):
1,72·1,05·1,09·2,83·5,53·3,39 → 1,66·1,36·1,21·2,65·4,99·3,18 ms (sem custo a mais). `pé` e `face` sozinhos: piores
que a W5f (tabela inteira no log da rodada; reproduz-se pelo cabeçalho do teste).

Fotos (preferências do dono: Baixa fixa, rebote ligado): [`fotos_W5g/cena7_discos_antes_em_cima_depois_em_baixo.png`](fotos_W5g/cena7_discos_antes_em_cima_depois_em_baixo.png)
— os degraus somem, a passagem sai em gradiente. `PH2D_LIGHT2D_PE=0 PH2D_LIGHT2D_FACE=0` volta ao antigo (comparar).

## 3. ⚠️ ABERTO — os dois portões vermelhos (o próximo passo da W5g)

A/B por flag (script `target/w5g/ab_portoes.sh` — troca os defaults do `Config::padrao` com `assert` e restaura):

| portão (teto) | pé só | face só | pé+face (o commit) |
|---|---|---|---|
| `o_interior_com_o_rebote_nao_sai_em_riscos` (1,05) | 1,322 · 1,342 | 1,132 | **1,069 · 1,071** |
| `as_formas_redondas_nao_saem_em_facetas` TV (0,40) | +1,08 | +0,49 | **+0,47** (zoom 2,5 · 3/8 · disco · 0,65) |
| `o_interior_do_opaco_nao_depende_da_posicao_na_grade` | verde | 2 texels (0,31) | ✅ verde depois da deteção de canto |

- Quem os move é o **pé**. ⛔ Não é o ruído do rebote ao longo da face: a mediana de 3 texels
  (`C0_AO_LONGO_DA_FACE_PADRAO` 2 → 3) deu 1,069 → 1,067 e o TV não mudou (o TV é SEM rebote). Hipótese a medir: o
  mapeamento agora contínuo projeta no interior o que antes os blocos da célula escondiam — a c0 (cascatas) varia
  de sonda para sonda ao longo da face, e o miolo era constante por bloco. Sonda: o TV do anel 0,65 com a c0
  zerada (só lâmpadas) × com ela; se for a c0, filtrar a c0 do PÉ pela distância ao longo da face na escala da
  célula das cascatas (não da fina).
- ⛔ Não subir os tetos: são medidos (W5d) e o controlo da W5f passa-os.
- Os controlos «W5c» do `as_formas_redondas` desligam agora também `pe_na_borda` e `vis_na_face`.

## 4. Outros abertos

- **Cena 8: buraco preto dentro da pilha** ([foto](fotos_W5g/cena8_pilha_buraco_preto_antes_e_depois.png): antes à
  esquerda/meio, depois à direita) — JÁ existia: os vãos FECHADOS entre peças da pilha são exterior que nenhuma luz
  alcança, e a lei põe-nos no interior vizinho. Pergunta de desenho (o vão fechado conta como exterior?).
- O novo portão `o_miolo_das_formas_nao_sai_em_degraus` leva 34–50 s em `dev` sob carga (R9: 10 s) — fixture menor.
- O serrilhado fino de 1–2 px junto ao terminador ainda se vê de perto na cena 7.
- Depois do F5: a abertura lenta (§3 da W5f).

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Olhe as quatro bolas em volta da lâmpada.
3. A passagem da parte clara para a escura de cada bola é uma faixa suave, sem degraus, nos dois lados.
4. Errado: degraus em escada onde a sombra nasce. Para comparar com o antigo, junte `PH2D_LIGHT2D_PE=0 PH2D_LIGHT2D_FACE=0`
   antes do `cargo`.
