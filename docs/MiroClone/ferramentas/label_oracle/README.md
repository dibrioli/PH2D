# Oráculo do RÓTULO da seta — tldraw e Excalidraw, corridos por script, nunca lidos

O rótulo que vive AO LONGO da seta (posição ao longo da linha): onde fica para cada posição, o corte junto às
pontas, o arrasto pelo rato, o vão da linha atrás dele, quantos rótulos uma seta tem. Medido 19/10/2026 sobre
entradas NOSSAS, como caixa preta (só a API pública + o `DOCS.md` do pacote tldraw; nenhum fonte lido).

## Como correr

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone/docs/MiroClone/ferramentas/label_oracle
node sonda_tldraw_rotulo.mjs .        # ~2,5 min · tldraw 5.5.2 em ~/Apps/tldraw-oracle (SO=<prefixo> filtra casos)
node sonda_excalidraw_rotulo.mjs .    # ~7 min · as duas versões; VERSAO=next|0181 e SO=<prefixo> filtram
```

Pré-requisitos (já instalados nesta máquina, nada global): `~/Apps/tldraw-oracle` (página `www/`), e as páginas
W9 `www_w9/` de `~/Apps/excalidraw-oracle` (0.18.1) e `~/Apps/excalidraw-oracle-next` (next) — ver
`../excalidraw_oracle/README.md` (`instala.sh`, `instala_w9.sh`). O Chromium é o do Playwright 1.63.0 em
`~/.cache/ms-playwright`.

Saídas: `tldraw_rotulo.json` + `tldraw_<caso>.png`; `excalidraw_rotulo.json` + `excalidraw_<versão>_<caso>.png`
(e `_ponta40.png`: o ecrã a meio do gesto que move a ponta). Cada fixture leva `cabecalho` (versão, data, host,
`loadavg`, comando, sha256 do script, entrada, câmara, pedidos externos, nota «Caixa preta»).

Harness: o padrão de `../elbow_oracle/sonda_tldraw_segmento.mjs` (servidor estático local, `Math.random` e
`crypto.getRandomValues` semeados, Chromium headless 1280×800, câmara `Z/CX/CY`, arrastos reais do rato,
`window.oracle.editor` + `createShapeId`). O tldraw corre a `Z=1, CX=250, CY=200` (1 px = 1 un.). O Excalidraw
usa a página W9 (zoom 1, scroll 0 ⇒ ecrã = cena) e a seta nasce em (450, 300), à direita do painel de
propriedades que aparece com a seta seleccionada.

⚠️ **tldraw pede a letra ao CDN** (`cdn.tldraw.com/5.5.2/fonts/…`, traduções, ícones — lista em
`cabecalho.pedidos_externos`) e a 1.ª medida de um rótulo numa página nova sai com a letra de recurso (largura
39,61 em vez de 41,5 — medido). A sonda aquece a letra com um texto descartável antes de cada caso.

## Licenças (triadas pelo artefacto instalado em `node_modules`, 19/10)

`pacman -Qi` não se aplica (pacotes npm): lido o campo `license` do `package.json` e o ficheiro `LICENSE*` de
cada pacote instalado.

| artefacto | versão | licença | nota |
|---|---|---|---|
| `tldraw` · `@tldraw/editor` | 5.5.2 | «SEE LICENSE IN LICENSE.md» → *tldraw license* (fonte disponível, NÃO aberta; produção pede chave — a página mostra «Get a license for production») | só se CORRE como oráculo; nada do código vem para cá, só as leis medidas |
| `@excalidraw/excalidraw` | 0.18.1 | MIT (campo `license`; o pacote não traz ficheiro LICENSE) | |
| `@excalidraw/excalidraw` (next) | 0.18.0-143b5b6 | MIT (idem) | |
| `playwright` · `playwright-core` | 1.63.0 | Apache-2.0 (LICENSE) | nas três instalações |
| `react` · `react-dom` | 19.2.8 (tldraw) · 19.1.0 (excalidraw) | MIT (LICENSE) | |

## Leis MEDIDAS

Notação: `L` = comprimento de arco do corpo da seta; `s` = posição do centro do rótulo ao longo do corpo.
`t:` = chave em `tldraw_rotulo.json` (`casos[].caso`); `x:<versão>:` = chave em `excalidraw_rotulo.json`
(`versoes.<versão>.casos[].caso`).

| # | lei | tldraw 5.5.2 | Excalidraw next | Excalidraw 0.18.1 |
|---|---|---|---|---|
| 1 | **a fracção é de COMPRIMENTO DE ARCO** | `s = labelPosition·L` exacto (distância 0,00 no recta e no cotovelo, ≤ 0,08 na curva amostrada); o paramétrico por índice falha 6,75–35,6 px no cotovelo — `t:P_cotovelo_lp0.25/0.5/0.75`, `t:P_curva_lp*` | `labelPosition·L` exacto (resíduo 1,05 = re-medida da largura do texto) também em 3 e 4 pontos — `x:next:L_dobra3_assim_lp0.25` (550 contra 600 paramétrico), `x:next:L_quatro_lp0.5/0.75` | não usa `labelPosition` (ver 6) |
| 2 | **corte junto às pontas** | o centro fica a ≥ `e + p` de cada ponta do CORPO, `e` = distância do centro à borda da caixa do rótulo ao longo da direcção da linha, `p` = **20 / 23 / 26 / 56** para `size` s / m / l / xl; recta 0,8/0,6 «sim» 41,5×35,5 ⇒ 48,94; «a long label here» 185,5×35,5 ⇒ 52,58 (sai por cima); horizontal ⇒ meia largura + 23 (43,75 e 115,75). Corpo mais curto que `2(e+p)` ⇒ o rótulo vai ao MEIO (`L/2`). O corte é só no desenho: `labelPosition` 0 e 1 ficam gravados — `t:V_*` (varrimento 0..1 passo 0,02), `t:V_horizontal_sim_size_*`, `t:V_curta_recta_*` | **nenhum**: 0 ⇒ centro NA ponta inicial, 1 ⇒ centro NA ponta final (em cima da cabeça) — `x:next:L_*_lp0`, `x:next:L_*_lp1` | — |
| 3 | **arrastar além da ponta** | `labelPosition` satura em **0 / 1** (recta, cotovelo; 0,99984 na curva) e o desenho fica no corte da lei 2 — `t:D_*_b_alem_do_fim_200`, `t:D_*_b2_alem_do_inicio_200` | `labelPosition` satura em **0 / 1** e o centro fica EM CIMA da ponta (450,300 · 850,300 · 750,400) — `x:next:D_recta2_b_alem_do_fim_200`, `x:next:D_dobra3_assim_b*` | — (não arrasta, ver 6) |
| 4 | **arrastar na perpendicular** | projecta no ponto MAIS PERTO do corpo: recta e curva ficam em 0,30 (o rótulo não sai da linha); no cotovelo salta para o trecho mais perto (0,0593, depois cortado a 43,75) — `t:D_*_c_perpendicular_120` | projecta no ponto mais perto: 0,3 ⇒ **0,3** (alvo projectado 0,2974; o resto é a largura re-medida), o rótulo volta à linha — `x:next:D_*_c_perpendicular_120` | — |
| 5 | **arrastar ao longo** | segue o rato mas perde o primeiro passo do gesto (a zona morta antes de o arrasto começar): alvo 0,75 ⇒ 0,732 com passos de 6,5 px, 0,7365 com passos de 1,6 px — `t:D_*_a_ao_longo_3q`, `t:D_recta_a2_ao_longo_3q_100_passos` | segue o rato (0,3 ⇒ **0,7526** para o alvo 0,75; o +1 px é a largura re-medida) — `x:next:D_recta2_a_ao_longo_3q`; cotovelo 3 pontos: **0,75** exacto — `x:next:D_dobra3_assim_a_ao_longo_3q`. Nos D_* a seta NÃO mexe (`seta_mexeu: false`) | — |
| 6 | **a pega do meio GANHA ao rótulo** | com o rótulo a 0,5 o centro coincide com a pega do `bend`/`elbowMidPoint`: premir lá e arrastar CURVA a seta (bend 0 → 120) ou move o trecho do cotovelo; o rótulo não sai do sítio — `t:H_*_lp0.5_centro_perpendicular_120` | idem: no «meio» está a pega do ponto médio (2 pontos ⇒ arrastar INSERE um ponto) ou o vértice do meio (arrastar move-o) — `x:next:H_*` | idem, e como o rótulo da 0.18.1 está SEMPRE no meio, nunca se arrasta — `x:0181:H_*`, `x:0181:D_*` |
| 7 | **texto sempre horizontal** | caixa alinhada aos eixos em TODOS os casos (recta a 36,9°, curva, cotovelo; `alinhado_aos_eixos` em todas as linhas dos varrimentos) | `angle` 0 em todos | `angle` 0 em todos |
| 8 | **vão da linha atrás do rótulo** | a linha é CORTADA em toda a caixa do rótulo: «sim» na recta ⇒ sem vermelho de s 156,2 a 204,3 (48,1 px; a recta cruza a caixa em 51,9, menos a vizinhança 3×3 da amostra); o longo ⇒ 152,2..208,3 — `t:G_recta_sim_vao`, `t:S_recta_rotulo_longo`. Seta curta + rótulo longo: a linha some (0..39,4 de 46,5; só a cabeça fica) — `t:S_curta_rotulo_longo` | há vão (visto na captura `excalidraw_next_L_recta2_lp0.25_ponta40.png`); comprimento NÃO MEDIDO | NÃO MEDIDO |
| 9 | **largura do rótulo longo × comprimento da seta** | quebra linhas para caber: largura = `L − 55,5` (146,5 ⇒ 91; 186,5 ⇒ 131; 236,5 ⇒ 181), mínimo **72,5** (uma palavra por linha) e máximo a largura natural (185,5) — `t:W_largura_do_rotulo_x_comprimento` | NÃO MEDIDO | NÃO MEDIDO |
| 10 | **um rótulo só** | `richText` é UMA prop da seta; duplo clique no corpo (longe do rótulo) entra na edição DESSE rótulo (`getEditingShapeId` = a seta, `select.editing_shape`, nenhuma forma nova: 3 formas antes e depois) — `t:K_*` | duplo clique no corpo edita o MESMO texto `T`, sem elemento novo — `x:next:K_duplo_clique_no_corpo_com_rotulo`. O FORMATO aceita dois textos ligados (`boundElements` com os dois, ambos ficam) mas os dois caem no MESMO sítio, sobrepostos — `x:next:M_dois_textos_na_mesma_seta` | idem — `x:0181:K_*`, `x:0181:M_*` |
| 11 | **onde nasce o rótulo vazio** | duplo clique numa seta SEM texto recentra `labelPosition` em **0,5** (mesmo que estivesse a 0,3); no cotovelo põe-no no meio do trecho do meio (0,5115 aqui), não no meio do arco — `t:K_recta_duplo_clique_corpo_sem_rotulo_lp0.3`, `t:K_cotovelo_duplo_clique_corpo_sem_rotulo`. Com texto, não mexe (`t:K_cotovelo_duplo_clique_corpo_com_rotulo_lp0.3`) | NÃO MEDIDO | NÃO MEDIDO |
| 12 | **`labelPosition` ausente (null) no next = a lei VELHA do meio**, não 0,5 de arco: nº ímpar de pontos ⇒ o vértice do meio; par ⇒ o meio do trecho do meio — `x:next:L_dobra3_assim_lpnull` (750,300 ⇒ fracção 0,75), `x:next:L_quatro_lpnull` (550,350) | — | — |
| 13 | **0.18.1: o «meio» é sempre o do índice** (mesma regra da 12) e ignora `labelPosition` — `x:0181:L_*` (centro igual para 0/0,25/0,5/0,75/1). **Mas GUARDA o campo**: `restoreElements` devolve-o (0,25 ⇒ 0,25), o editor mantém-no e `serializeAsJSON` grava-o (`texto_restaurado_json`, `serializado_texto` em `x:0181:L_recta2_lp0.25`); ausente fica AUSENTE. O next, ao ler um ficheiro sem o campo, ESCREVE `labelPosition: null` (`x:next:L_recta2_lpnull`) | — | ✅ |
| 14 | **o x,y gravado do texto não é a verdade no next** | — | depois de mover uma ponta, o canvas desenha o rótulo no sítio novo mas o `x,y` do elemento fica o antigo (`x:next:L_recta2_lp0.25`, passo `ponta_final_arrastada_40_na_perpendicular`, e a captura `_ponta40.png`); `restoreElements(…, {refreshDimensions:true})` recoloca-o pela lei (`recolocado_por_restore`) | `refreshDimensions:false` deixa o texto onde o ficheiro diz (os dois) |
| 15 | **seta redonda (next)** | — | o rótulo fica na CURVA, perto da fracção de arco da polilinha (lp 0,25 ⇒ 4,4 px fora dela, 0,75 ⇒ 8,5) — `x:next:L_dobra3_assim_redonda_*`; a fracção sobre a curva verdadeira NÃO MEDIDA (precisa da curva dele) | no vértice do meio (passa por ele) |

## NÃO MEDIDO

- O vão da linha no Excalidraw (só visto numa captura) e o comprimento dele.
- A largura máxima do rótulo no Excalidraw (quebra de linha numa seta curta).
- A origem do `p` do tldraw (20/23/26 para s/m/l é `16 + 2·espessura`, mas o xl dá 56, não 36) — fica a tabela.
- O tldraw com `labelCenterSnapDistance` (10 px de ecrã, `DOCS.md`): o encaixe do rótulo no meio ao arrastar não
  foi isolado (nenhum alvo dos arrastos caiu a menos de 10 px do meio).
- Seta rodada (`rotation` ≠ 0) e zoom ≠ 1 no tldraw; texto de várias linhas escrito pelo artista.
