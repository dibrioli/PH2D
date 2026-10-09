# 09 — PROMPT da janela de AUDITORIA da W1c: a borda pixelada e a raiz da gota

> Cole o bloco abaixo numa janela NOVA. É auto-contido. Origem: o smoke do dono de 09/10 sobre a W1c
> (`f216fd97c`): duas capturas ampliadas — [1_borda](fixtures/smoke_dono_w1c/1_borda.png) ·
> [2_raiz_da_gota](fixtures/smoke_dono_w1c/2_raiz_da_gota.png) — e a frase *«borda pixelada e artificial.
> veja o escorrimento. Que app pro faria isso?»*.

```
AUDITORIA — line/PainterWatercolor («Wet Painter»), depois da W1c: a borda de baixo PIXELADA e a raiz
ARTIFICIAL da gota, pelo smoke do dono (09/10). Esta janela MEDE e ACHA; não cura. Entrega: os achados com
mecanismo + reprodução + o portão que faltava, e o prompt da janela de cura (10_PROMPT_…).

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -6 && git status --short → a ponta é «docs(Painter 09/10): o prompt da auditoria da W1c…»
   (logo abaixo f216fd97c · 6abfef287 · b6d0b21e0 · 4ffbc7432); árvore limpa.
3. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh;
   um hook recusa todo texto de Bash que contenha «cargo test» fora da porta — inclusive dentro de heredoc:
   escreva os scripts num ficheiro (ferramenta Write) e corra `bash scripts/ph2d-run.sh bash <ficheiro>`.

O QUE O DONO VIU (abra as duas imagens com a ferramenta Read ANTES de tudo):
- 1_borda.png — a descida de um traço com a tela inclinada, ampliada no app: a orla de baixo é uma ESCADA de
  pixels duros (serrilhado), por cima dela ainda uma faixa azul escura e saturada de topo ondulado, e por
  baixo um halo de pontinhos soltos. O círculo cinzento é o cursor do pincel.
- 2_raiz_da_gota.png — onde a gota nasce: o fio é uma coluna RECTANGULAR colada à faixa, a faixa muda de
  altura de um lado para o outro do fio (um degrau na orla exactamente na raiz), e a junção é um canto vivo
  — nada do «funil» suave do Rebelle.
- O azul dele é mais claro e saturado que o da bancada (#1765b3): ele pinta com a cor DELE, o pincel e o
  tamanho dele, e olha AMPLIADO. Os portões da W1c medem o S a 1:1, a 30 s, com o azul da bancada.

LEIA (nesta ordem; só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md §3 (o TEMPLATE de auditoria por lente é obrigatório:
   LENTE · CLAIM · TRAÇO file:line · ASSERÇÃO-VERMELHA · NÃO-CHECADO-PELA-COMPILAÇÃO · LOC LIDAS).
2. docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md §4b — a W1b (5 rondas) e a W1c inteira: a
   cura (o canal da gota escoa a orla), a tabela, as RECUSAS MEDIDAS (não as reproponha), as mutações, o custo.
3. docs/Painter/NewFeaturesWetPaint/06_gotas_rebelle_x_ph2d.md e fixtures/rebelle_gotas/README.md (o oráculo
   do S a 1:1: s_inclinado.png, s_inclinado_1.png, s_sem_gotas.png, s_plano.png).
4. Código: crates/ph2d-wet-paint/src/drips.rs (DripLaw, drip_pass, spawn, feed), drips/edge.rs (Folha::por_fora
   — a linha de contacto e a orla num predicado —, pin_downhill, walk_edge, basin_reach), drips/trail.rs
   (advance, o pescoço/funil, settle, bead); solver/advect_jacobi.rs (onde o pin entra); sim.rs (Sim::gravity,
   drip_law, drip_px_per_cell); o composite do Wet Paint no tool
   (crates/ph2d-tool-painter/src/tool/paint/wetpaint/{composite.rs, grid_map.rs}); e o caminho do ECRÃ: como a
   tela do Painter é ampliada no viewport (procure o sampler/filtro de magnificação — ph2d-render e
   ph2d-app-painter; `FilterMode`, `mag_filter`). Os portões: crates/ph2d-tool-painter/tests/it/
   {borda_do_s.rs (a régua mede_borda/fios_do_s), gotas_pela_lei_do_rebelle.rs, sonda_lei_das_gotas.rs (as
   variantes da DripLaw no MESMO processo: PH2D_S_VARIANTES / PH2D_B_LEIS)}; a régua Python
   docs/Painter/NewFeaturesWetPaint/ferramentas/borda.py.

AS LENTES (agentes em paralelo — explorador para ler, verificador para correr; você decide e mede):

A. A BORDA PIXELADA. Separe as três causas possíveis e MEÇA qual é a do dono, uma por uma:
   A1. O motor: a linha de contacto é BINÁRIA por célula (papel seco por fora = fluxo zero), logo a orla é
       a fronteira de uma máscara de células — à razão de grade 1 é 1 px, mas à razão do app (qual é? leia o
       estado do tool: grid_ratio por omissão, o slider) cada célula pode ser 2–4 px de ecrã. A cobertura do
       pixel da orla é 0 ou 1: não há anti-serrilhado. Meça: o perfil TRANSVERSAL à orla (perpendicular, não
       vertical) em várias colunas — quantos px de transição entre papel e tinta (o Rebelle: compare na foto
       1:1, s_inclinado.png, com a mesma régua).
   A2. O ecrã: o viewport amplia a tela com filtro NEAREST? Então qualquer orla vira escada ao ampliar, no
       Rebelle também? Abra o oráculo AMPLIADO: gotas_inicia.sh / gotas_tela.sh (--forma s --size_px 100
       --agua 100, tela (0,1)) e fotografe a orla a 300–400 % pelo Navigator do Rebelle (o mesmo recorte que
       o dono). ⛔ NUNCA o fecha.sh do oráculo, nunca o arranque.sh inteiro, nunca DISPLAY=:0 nem spectacle
       (fotografa o ecrã REAL do dono); feche com gotas_fecha.sh. Se o Rebelle ampliado também mostra pixels
       duros, a escada do dono é do viewport e a pergunta muda (o que o Rebelle faz com a orla A 1:1 que a torna
       lisa ampliada?).
   A3. O contraste: uma faixa saturada contra papel branco amplifica a escada; a faixa ainda existe na cena do
       dono? A régua da W1c (faixa ≥ B0 ≤ 6 px) foi medida com o Wet Round a 100 px, água 1,0, o azul da
       bancada. Reproduza a cena do DONO (o tamanho e a água prováveis pela captura; a cor clara e saturada;
       faça a sonda com 2–3 hipóteses de pincel/tamanho/cor) e meça a faixa, o degrau e a transversal NELA.
   ⚠️ A régua da W1c mede a POSIÇÃO da orla (salto, rms de uma cúbica) — uma escada de 1 px perfeita passa
   «lisa» nela. Falta a régua da COBERTURA do pixel da orla (anti-serrilhado). Escreva-a (Python sobre as
   fotos e Rust para o portão) e corra-a sobre o oráculo e sobre o nosso, a 1:1 e na ampliação do app.

B. A RAIZ DA GOTA. Na captura 2: a orla muda de altura dos dois lados do fio, e a junção é um canto vivo.
   B1. O degrau na raiz: a bacia da gota é escoada (recede, drain_after) e a vizinha não — a orla de um lado
       recua e do outro fica? Ou o fio nasce na célula de orla e a linha de contacto abre ali (a gota muda o
       `film` da célula da orla, o predicado por_fora vira)? Meça a orla ±20 px em volta de cada raiz no S
       (mesma régua da posição), com e sem `drain_after`, com e sem `recede` baixo (sonda_lei_das_gotas).
   B2. O canto vivo: o fio é assentado em fileiras TRANSVERSAIS de largura inteira (settle: w células cheias +
       SOFT_EDGE 0,9) a começar na célula logo abaixo da orla, e o pescoço (NECK_W 3 px a afinar em 12 px)
       acrescenta tinta, não forma. O Rebelle: um funil que ALARGA para cima e se funde na orla (escuro ~2× o
       fio, 1–3 px de transição). Meça a largura do fio a 0, 2, 4, 8, 16 px abaixo da orla no Rebelle e no nosso
       (as fotos s_*.png a 1:1 e a ampliação do A2).
   B3. Pergunte também: o fio é vertical sempre (a gravidade é (0,1)); a orla é oblíqua — no Rebelle o fio
       nasce TANGENTE à orla e curva para baixo? (as fotos dizem).

C. O QUE O PORTÃO NÃO VIU (lente de régua — a família «como uma régua mente», project-memory):
   para cada portão da W1c (a_borda_do_s_e_a_do_rebelle, as_gotas_saem_de_toda_a_orla_do_s,
   a_grade_2_da_os_mesmos_fios_em_px, a bancada), diga o que ele mede, o que o dono viu que ele NÃO mede
   (cobertura do pixel, a ampliação, a cor/pincel do dono, a raiz), e a asserção que ficaria vermelha HOJE na
   cena do dono. Uma régua que pára a 1:1 aprova uma escada no ecrã ampliado.

REGRAS
- CLAUDE.md §0 inteiro. Só MEÇA: nenhuma mudança de produto. Sondas temporárias saem (ou ficam como sonda
  #[ignore] documentada, se servirem à janela de cura). Medição com a TABELA ao lado; variantes no MESMO
  processo (sonda_lei_das_gotas) — nunca uma const recompilada por variante; relógio acima de load ~5 não
  vale. Num bloco só (§0.10): todas as medições de A, B e C planeadas antes, uma rodada.
- «Não piorar a engine»: o que a cura vier a mexer fica no caminho com Create Drips LIGADO; as quatro cercas
  (cerca.sh · os_outros_meios_nao_mudam · sem_create_drips_e_o_escorrimento_natural_de_antes_ao_byte ·
  com_a_tela_direita_o_traco_e_o_de_antes_ao_byte) têm de continuar verdes — diga, para cada cura proposta, se
  ela pode tocar uma delas.
- NÃO REPROPOR (medido e recusado — 05 §4b): a faixa de transição que deixa passar fluxo para o papel seco
  (inunda: 106–127 mil px); a difusão do pigmento (inerte sobre o assente); cortar só a componente de descida
  (degraus e bases planas); baixar a força da lei (o corpo não clareia); assentamento na água parada (o corpo
  congela escuro); EdgeDarkening 100/200; gotas sem o canal; neck_dark ≥ 3 (brechas na orla recta); alcance
  fixo da bacia. Um anti-serrilhado que NÃO deixa passar água (só cobertura de pigmento na célula da orla, ou
  no composite/ecrã) é outra coisa — avalie-o.
- Pendente do dono (não decida): o custo a 4096² (pior tique das poças 3,9–4,3 ms > 2 ms depois da 2.ª tentativa
  — parado); Create Drips ligado ou não de fábrica.

ENTREGA
1. Os achados (A1–A3, B1–B3, C) no TEMPLATE da DIRETIVA §3, cada um com: mecanismo (file:line), a medição
   com a tabela (o nosso, o Rebelle, a cena do dono), a reprodução e o PORTÃO que faltava (a asserção que fica
   vermelha hoje). Achado sem medição não é achado.
2. Um parágrafo no 05 §4b («🔎 Auditoria da W1c, 09/10») com o essencial e o link do prompt seguinte.
3. docs/Painter/NewFeaturesWetPaint/10_PROMPT_janela_W1d_cura.md — o prompt auto-contido da janela de cura, no
   molde deste e do 08 (Fase 0, o que ler, as decisões do dono, os achados, os portões red-first, as recusas,
   as cercas, a régua do custo, a resposta ao dono).
4. Commit por caminho (--no-verify; ficheiro novo: git add antes); nunca push.
5. AO DONO: curto, sem jargão — o que está errado, de onde vem, e o que a próxima janela vai fazer; com uma
   imagem ampliada: o nosso contra o Rebelle, MESMA ampliação, a borda e a raiz da gota.
```
