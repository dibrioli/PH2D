/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10_pastel_grao_e_mistura.md

Estado (10/11): linha `line/PainterWatercolor`, base `main` `f254e42bf1`, por integrar. O anterior:
`PROMPT_CONTINUACAO_2026-11-09_pastel_K1.md` (itens 1–4 FEITOS nesta janela).

FEITO (commits `3d8d15e0eb`, `956935388c`, `b8114c427b`):
- Pastel na receita K1 (`csp_secos.rs`) + `fill 0.5`; painel da ponta com «Grain softness», «Tip contact» (Flat · Rounded) e
  «Tooth fill» no FIM de `LINHAS_DA_PONTA` (29), só no modo Height.
- **O dente enche** (`GrainLaw::fill`, `texture/po.rs`, `tool/paint/po_do_papel.rs`): o Height lê o papel a
  `t + fill·pó·(1 − t)`, pó = alfa da camada no pen-down (o `before` do desfazer) desfocado por 3 caixas de raio 5.
  Medição, recusas (o desfoque de 1 px e o `covers` do relevo) e o custo (11 ms a 4K): LEIA a FASE 4 de
  `docs/Painter/handoffs/material_2026-11-09_pastel/README.md`. Testes: `o_dente_enche` (2, com mutações que sangram),
  `o_enchimento_sobe_o_vale_e_e_neutro_sem_po`, `seam_ponta`. Alarmes de pincel verdes (Pastel regerado).
- Binário de smoke compilado nesta worktree.

SMOKE DO DONO 10/11 (Pastel com o enchimento): **OK**, com quatro ordens — num BLOCO onde a família é a mesma:
1. **O grão do Pastel como o dele** (foto do cartão Grain): `Noise`, `Tiled`, Size **120×120**, Angle 0, Depth **1,00**,
   Contrast **0,55**, Brightness **0,55**, Detail **0,30**, Roughness **0,50**, Warp **0,50** — no K1 `params` era
   `[0.4, 0.5, 0.3, 0.5, 0.5, 0.5]` a 32 px; CONFIRA a ordem dos `params` do `Noise` contra os rótulos do cartão
   (`texture/patterns/specs.rs`) antes de escrever. O resto da lei K1 fica. Regerar o Pastel e rodar a sonda 1–5 passadas
   (`var_pastel_fill.tsv`) para mostrar ao dono se o enchimento ainda enche em 3–5 passadas com o grão grande.
2. **Chalk e Crayon também enchem o dente (dono: SIM)**: o Height + `fill` nas receitas deles (`csp_secos.rs`; hoje
   Multiply com `PAPEL_ASPERO`/`PAPEL_DE_DESENHO`) — medir antes de escolher a profundidade/maciez, folha lado a lado
   antes/depois ao dono; regerar e alarmes por nome.
   ⇒ 1 e 2 são a MESMA família (receitas dos secos): uma rodada de sonda intercalada, uma regeração, um par de alarmes.
3. **BUG provável da mistura de cor (Color mixing), foto do dono**: traço azul de fora PARA DENTRO do vermelho → o
   vermelho aparece ANTES do traço chegar à borda (puxado para fora); do vermelho para fora → o roxo fica só por cima do
   vermelho e o azul é puxado para dentro, sem levar vermelho para fora. O dono diz: a direção da mistura está invertida.
   Leitura da foto (a confirmar, NÃO acreditar): nos dois sentidos o roxo começa ~um lote antes da borda do vermelho ⇒
   suspeito o LOTE: a rota `stamp_dabs_mistura_opaca` (`tool/paint/stamp_route/mistura.rs`) lê a tela de antes do lote
   (`mistura_opaca.antes`, ladrilhos) e o reservatório pode estar a apanhar a cor sob os dabs SEGUINTES do lote — o tempo
   anda para trás dentro de um lote. Reproduzir por teste (traço azul a atravessar uma faixa vermelha nos dois sentidos,
   medir onde o roxo começa e se o vermelho é levado na direção do movimento), achar o mecanismo (`/pd-bug-causa`) e
   o gate que estava verde. Isto pode ter estragado a afinação da Color Mixing (memória
   `project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08.md`) — reavaliar depois do conserto.
4. **Color mixing num cartão PRÓPRIO** no painel, não como secção dentro de Basics (receita
   `docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`; o teste de costura acompanha).

Abertos herdados: o Tooth fill não age na ponta de píxel (L18), no Shape de camadas coloridas (`stamp_color`) e na máscara
de stamp em cache (`render_stamp_mask`, Grain View) — fechar ou esconder o controlo nessas rotas (controlo morto) ·
curva da pressão do K1 a p 0,5–0,65 abaixo da do Krita · o pálido no branco/blocos é a `mistura_opaca` (rever com o 3).

Fila depois (do prompt anterior): memória do 1.º traço · marca do Wet por camada · Wet Wash Mix · Splash ·
`SPRAY_FIXO_MAX` com Wet blend · painel a 245 px · dicas longas · papel de imagem sem dente com Grain.
Inegociáveis e fecho: os do prompt anterior (bateria por nome inclui `ph2d-painter-brush` `texture::` e
`ph2d-tool-painter` `o_dente_enche`).
