═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR (também nas mensagens curtas entre ferramentas) — duas sessões já
   responderam em inglês, puxadas pelas notificações do arnês (project-memory/user_role.md).
⛔ REGRA DO DONO (CLAUDE.md §0.10): problemas equivalentes num ÚNICO bloco — um plano, um lote, UMA rodada
   de medição, um gate batched. E (05/10) comparar velocidade NÃO espera horas: docs/DevOps/MEDIR_VELOCIDADE.md.

FASE 0 (já, sem pedir confirmação):
1. cd Worktrees/line-motion-value && pwd && git branch --show-current   → line/motion-value
2. git log --oneline -6 && git status --short --ignored | grep -v target/
   → HEAD esperado: o commit deste handoff, sobre bfad80ea1. Árvore limpa (só `!! assets/sprites/`).
FASE 1:
3. git cherry main HEAD | grep -c '^+'   (esperado 5 — a linha tem commits NÃO integrados: a memória PT-BR, a
   sonda intercalada, a regra de medir, o índice DevOps, este handoff). >0 ⇒ `git range-diff main...HEAD`, depois
   `git rebase main 2>&1 | tee target/rebase.log` (conflito em project-memory/MEMORY.md ou reference_topic_*:
   resolva pelos ESTÁGIOS, as duas entradas sobrevivem e a contagem da família RECONTA-SE no ficheiro dela).
4. bash scripts/cargo-check-narrow.sh ph2d-shape-gpu && bash scripts/cargo-check-narrow.sh ph2d-app-motion
FASE 2 (leia DENTRO da worktree):
5. docs/Motion Nodes/121_as_formas_na_placa.md — §9.15 (plano, veredictos) e §9.16 (a régua intercalada e o que
   fechou) INTEIRAS. docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md §6 e §6.7 (o
   aberto). docs/DevOps/MEDIR_VELOCIDADE.md.
6. DIRETIVA_IMPLEMENTACAO.md · STACK_VERSOES.md · regras A–K do MODELO_ABERTURA_LINHA.md.
7. Reporte em UMA linha e siga.

O QUE JÁ EXISTE (NÃO reconstrua):
- O passe de formas na placa com as células por acumulação, a variante enxuta/completa, o tracejado no ecrã,
  o brilho com formas (tudo no main). O bloco §9.15, integrado em 04/10 e agora MEDIDO por inteiro (§9.16):
  A1a/A1b (o ajuste da contagem e o total do fechado no percurso), B1 (`cs_deposita` por aresta ESCRITA), B2
  (junta uma vez por troço), c2 (a capacidade medida no 1.º quadro de uma cena nova), D (`cs_varre` por
  subgrupo). `F` contra `base`: iGPU esticadas tracejadas −20 %, densas −16 % / −28 % (tracejadas), RTX
  nenhum arranjo pior que +10 % e −4…−20 % no F.
- A RÉGUA NOVA (§9.16): `sonda_intercalada` + `docs/Motion Nodes/ferramentas/mede_intercalado.sh` — 9 variantes ×
  6 cenas × 2 placas em ~37 s, sem espera de calma. Os pedaços são `override` do WGSL
  (`ShapePass::com_constantes`, `VarianteDoPasse`); `ph2d_gpu::pass_profiler::drain` dá os totais por bloco.
  ⛔ `__GL_SHADER_DISK_CACHE=0` só no processo da sonda (SIGSEGV de saída na RTX — medido).
- ⛔ RECUSAS MEDIDAS que não se reconstroem: o GRUPO por cópia tracejada (RTX −49 %, iGPU +6 %), D1/D2 do
  `cs_varre`, a variante esparsa, estreitar a célula, herdar a bissectriz, costurar as correntes, as pontas
  partilhadas (a ponta fecha cada traço).
- O 3D SAIU do app (ADR-0179): o Motion não depende dele; as menções em docs antigos são histórico.

A TAREFA — UM BLOCO: «fechar o aberto do passe de formas e atacar a próxima alavanca do tracejado»
Itens (inventário primeiro; plano + kill-criteria na nova §9.17 do doc 121 ANTES de construir; comite o plano sozinho):
 a) Provas de mutação dos pedaços do §9.15 (NUNCA correram). Arnês novo em docs/Motion Nodes/ferramentas/, no
    padrão do `mutacao_o_bloco_do_9_14_2026-10-04.py` (pré-voo das âncoras, corrida LIMPA verde, restauro + touch).
    As previstas: o ajuste da contagem a `1` · o traço adiado do fechado nunca emitido · o prefixo das escritas
    deslocado de um bloco · a reserva sem as pontas (`pecas · 4 + junta`) · o c2 desligado e a cena nova sem
    `medir_ja` · a correção da fronteira da célula no subgrupo (SÓ sangra na iGPU, subgrupo 64 — corra lá).
    Gates: `ph2d-shape-gpu --test it -- --ignored` (14) nas duas placas.
 b) Os overrides dos pedaços: decida com número — ficam como portas de medição (custo zero em execução) ou
    dobram-se (os ramos `false` são código velho vivo). Se dobrar, o `variantes()` da sonda intercalada perde os
    pedaços e o §9.16 diz porquê.
 c) A medição no APP: `=127` densa contínua e tracejada (`PH2D_TRACO_ESTICADO_TRACEJADO=1`), as duas placas, rota e
    `[formas]` lidos, e a MEMÓRIA DAS ARESTAS (o B2 corta a reserva; ninguém a mediu no app — acrescente uma linha
    `[formas] arestas: capacidade …` ao `relata` do `garante` em contorno.rs). ⚠️ `mede_formas_na_placa.sh` ainda
    tem a espera de calma (`calma()`, `SEGUIDAS=3`): tire-a pela regra nova (o app mede-se pela parede de
    janelas de 120 quadros; carga alheia vai ao lado como contexto).
 d) A PRÓXIMA ALAVANCA das tracejadas: a ablação `E1` (§9.15) mostrou que a EMISSÃO por peça é `0,19` dos `0,40` ms
    de `conta + escreve` (as voltas e a escrita na memória já não pesam). Inventário de `emite_pedaco` /
    `bissectriz_ate` / `emite_junta` (contorno.wgsl) e das peças que vão às células; candidatos construídos já como
    `override` e medidos pela `sonda_intercalada` (critério do proxy de telemóvel = iGPU; RTX ≤ +10 %).
    ⛔ não reconstrua a topologia por troço.
 e) Os registos (`registos_dos_shaders.sh`) do `base` e do `F`.
COMO: inventário (agente `explorador`) → plano/kill-criteria §9.17 → construir → UMA medição intercalada (minutos)
→ gate batched 1× → mutações SOZINHAS na árvore → handoff de integração NOVO
(`HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md`; o de 03/10 já está no main) → smoke ao dono em passos
numerados, fotografado (FORMAS + SIMULAÇÃO; a `=127` serve) → `rm -rf target/*/incremental` e o smoke compilado 2×.
Leis que já custaram caro nesta linha:
 - A régua é a SOMA dos passes (o relógio por passe mente na fronteira); o resumo é o MÍNIMO das rodadas.
 - O 1.º quadro de uma cena nova é outro evento: meça-o à parte (a média fria escondia 76 ms).
 - Um despacho dimensionado pela RESERVA paga fios mortos: meça reservado × escrito antes de culpar a geometria.
 - ⛔ `contorno.rs` está em 695/700 LOC: cresceu? MOVA para um irmão (há `contorno_capacidade.rs`, `contorno_sondas.rs`).
 - Script de várias etapas: NUNCA encadeie com `;` depois de um passo que pode falhar (em 05/10 um `git add`
   aceitou ficheiros com marcadores porque o resolvedor falhou antes) — use `&&` ou pare no erro.
 - A placa é partilhada: PH2D_GPU=1 bash scripts/ph2d-run.sh …; NUNCA force. Mutação nunca em paralelo com builds.
 - A memória do Claude aponta para o project-memory do checkout PRINCIPAL: escreva-a em
   Worktrees/line-motion-value/project-memory/ e comite-a na linha.
FORA DE ESCOPO: bug #11 (fk.rs duplicado) · o contacto dos colisores na placa (Física) · a mordida do traço rente.
DoD: cada item com tabela antes/depois e o kill-criterion escrito antes; recusa medida também fecha · gate batched ·
handoff novo · smoke fotografado · NÃO integra, NÃO pusha (CLAUDE.md §0.7).
A MÁQUINA É PARTILHADA (regra K): pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
