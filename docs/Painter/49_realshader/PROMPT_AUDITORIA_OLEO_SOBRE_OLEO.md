JANELA NOVA — AUDITORIA SÉRIA do ÓLEO SOBRE ÓLEO (dono 31/10, smoke do build `fe53e7ad4f`+). Coordenador autónomo:
sem perguntas, sem pedir permissão, sem smoke ao dono até a cura estar fotografada no APP REAL.

FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
(line/PainterWatercolor). Leia SÓ: este ficheiro, `HANDOFF_SUPERAR_REBELLE.md` §8 e `ESTADO_REBELLE_MEDIDO.md` (a tabela).
A janela principal COORDENA; agentes leem, medem e constroem (≤ 3 em paralelo, 45 min cada; briefing com a linha ⛔
«cd X && … · bash -c/sh -c · rm -rf com variável/glob · kill de PID não gravado · escrever em .git/.claude/ ·
AskUserQuestion · push · tocar fora dos seus paths · spectacle · NVIDIA (PH2D_WET_GPU=1 / testes ph2d-wet-gpu)»).
⛔ A RTX travou o PC duas vezes em 31/10 (Xid 109): nada na NVIDIA nesta auditoria.

## A queixa (verbatim) e as capturas
«qualquer tipo de óleo sobre óleo faz encaroçar. o branco ocorre no óleo sobre óleo.»
Capturas do dono em `docs/Painter/49_realshader/smoke_31_10_oleo/` (LEIA as três):
- `dono_3.png` — óleo azul pintado em várias passadas: o miolo sobreposto fica GRUMOSO e escuro, e há dezenas de
  FUROS pequenos cor de papel espalhados por toda a mancha (a tinta abre buracos até ao papel).
- `dono_4.png` — a mesma mancha depois de mais passadas: o centro vira um campo de CAROÇOS em relevo (bolinhas
  isoladas), as estrias somem.
- `dono_5.png` — amarelo sobre azul (óleo sobre óleo): faixas de PONTILHADO BRANCO/claro com pontos verdes na zona de
  mistura e na orla, e o azul por baixo todo encaroçado.

## O que JÁ foi tentado em 31/10 (não repetir às cegas — e suspeitar de cada um)
- `611591d423` espalha: no 1.º toque de um traço numa célula com tinta, a espessura de baixo cai a metade.
- `3570226c1e` ladeira do óleo pesa pela fracção assente da fotografia (o traço já não derrete longe do cruzamento).
- `a6796fd570` piso 0,25 do ganho do céu (pretos nas fendas, Estúdio/Interior/Pátio).
- `4279ba26eb` sub-passo natural da cápsula (tinta a mais a Size 48–64).
- `a1d8b75767`/`e5efa2d206` Knob::Opaque + mistura K-M na célula opaca (só guache/acrílico; o óleo NÃO é opaco nela).
- Sondas existentes (crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/): `sonda_oleo_grumos.rs`,
  `sonda_oleo_derrete.rs`, `sonda_oleo_estria.rs`, `sonda_oleo_pretos`, `borda_branca_tests.rs`; driver do APP real
  (compositor virtual, eventos um a um): scratchpad antigo `oleo_app_smoke.rs.txt` + `mod_rs.patch` (recriar se sumiu —
  ver como o agente de 31/10 o fez: kwin virtual + `import -window`, NUNCA spectacle).
⛔ LIÇÃO de 31/10: as sondas de motor passavam raio 50/64 (o dono usa Size 64 = RAIO 32) e uma pincelada; viam tudo
limpo enquanto o app mostrava o defeito. Nesta auditoria a régua é o APP REAL na cena do dono, e cada sonda tem de
reproduzir primeiro a captura antes de valer.

## Método (pd-auditoria: lentes independentes + varredura de seam)
1. REPRODUZIR no app real (agente A): Painter › Wet Paint, os óleos de fábrica (Thick Impasto, Filbert, Flat Bristle,
   Express Oil, Knife*, Dabber, Metallic Flat, Tilted Shade), Size 48/64/96, cor azul e depois amarelo por cima,
   10–30 passadas sobrepostas em vaivém/cruzadas sobre o papel com relevo creme, com o tempo real a correr.
   Fotografar e MEDIR: furos até ao papel (px cor de papel cercados de tinta), caroços (picos 5×5 com proeminência
   por área), px claros/brancos que nenhuma mistura das duas tintas explica (o critério da `borda_branca`),
   anisotropia/estria. Tabela por pincel × nº de passadas.
2. LENTES (agentes B e C, sem ver a conclusão um do outro), cada uma lê o código do caminho óleo-sobre-óleo e procura
   o MECANISMO com ficheiro:linha:
   - B (massa/altura): transfer.rs + espalha.rs + espessura + oleo_campo.rs + lamina/espátula + pickup/redeposição +
     ladeira — quem cria FUROS (massa/espessura que vai a 0 numa célula cercada: levanta/pickup que leva tudo,
     espalha que corta por célula sob pegada ruidosa, o assentar que zera), quem cria CAROÇOS (soma por célula sem
     difusão lateral, ruído dos fios que soma em fase, saturação que não aplana).
   - C (cor/ecrã): assenta/ponto.rs (tapar extra `a_e`, `c_eff` diluída em BRANCO, caixa por canal), com_oleo /
     PINCEL_DE_OLEO (o toggle Real Oil da sessão salta as células de óleo — `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`
     vermelho desde 776a9ff173), oleo_vista/luz (brilho especular que lê como branco?), o render do relevo — quem
     pinta BRANCO/claro no óleo sobre óleo (o pontilhado de `dono_5.png`).
3. SEAM: para cada mecanismo, a régua que o deixou passar (porquê os portões estavam verdes) e o portão novo que o
   prende NA CENA DO DONO (raio 32, várias passadas, curva).
4. CURA num ÚNICO bloco (CLAUDE.md §0.10): uma lei física coerente para óleo molhado sobre óleo molhado — a tinta nova
   ARRASTA e mistura a de baixo pela pegada (difusão lateral), o relevo satura e aplana sob pressão, nunca abre furo
   até ao papel onde havia tinta, nunca clareia além das duas tintas (mistura subtractiva K-M, como a guache opaca),
   as estrias das cerdas da ÚLTIMA passada ficam limpas. Uma pincelada única muda o mínimo (o dono aprovou-a).
   Teste vermelho primeiro por mecanismo; verde; portões de óleo por nome em `--profile smoke`.
5. Fotografar no APP REAL antes/depois (montagem), regerar as miniaturas de óleo que o portão
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` acusar (só a janela principal regera:
   `PH2D_PRAZO=36000 PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="<nomes>" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it gera_os_presets_de_fabrica -- --ignored --nocapture`),
   handoff técnico `HANDOFF_AUDITORIA_OLEO_SOBRE_OLEO.md`, e smoke ao dono em passos numerados (o comando:
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`).
NÃO integrar no main nem push.
