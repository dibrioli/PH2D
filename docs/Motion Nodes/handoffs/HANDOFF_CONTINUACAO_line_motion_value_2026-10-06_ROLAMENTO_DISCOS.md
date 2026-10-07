═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem do dono; uma onda só; série só com o porquê escrito antes; NUNCA matar
   o pai de um processo (LOGOUT do dono em 05/10). A medição do app não vale acima de `load ~5` (outras linhas correm
   testes pesados nesta máquina): confira `cat /proc/loadavg` antes de cada célula do app.
⛔ Decisão do dono (06/10): `16 384` por taça com a pilha formada fica a `~30` fps — o contacto no dispositivo NÃO
   entra nesta onda.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -5 && git status
        --short --ignored | grep -v target/ ; rebase no main se ele andou (conflito fora dos seus ficheiros ⇒ PARE).
FASE 1 (leia DENTRO da worktree): doc 121 §9.20 ponto 5, §9.21 (3) e §9.22; o handoff de integração de 05/10 §13–§15;
        `crates/ph2d-contact-world/src/rolar.rs` INTEIRO (o cabeçalho tem a tabela das seis leis recusadas e a das três
        variantes da trava) e o pedaço do `passo` em `lib.rs` que decide `presa`/`excessos`/`travao`;
        `crates/ph2d-app-motion/src/motion_state_pilha_demo_giro_diag.rs` (`perfil_com`, `probe_o_rolamento_na_pilha`).

O DEFEITO (o item 2 do «em aberto» de 06/10): uma pilha de DISCOS (a `=114` com `Collider Shape = Circle`) com
`Rolling` BAIXO (`0,1`) gira MAIS na queda do que sem o botão — rodopio na janela `120..180`: `104°` contra `10,7°`
sem `Rolling` (a `0,25` e a `0,75` acalma: `0°`). Depois da queda assenta como sem ele (`3,1°` contra `2,4°`). A leitura
escrita no §9.21 («um monte inclinado que desaba enquanto assenta») NÃO foi provada.

O QUE JÁ EXISTE (não reconstrua): a lei do rolamento em DUAS FASES (parada ⇒ rotação TRANCADA no solver até o binário
pedido passar a capacidade `Σ μr·λn·|braço|` em `EXCESSOS_PARA_SOLTAR = 2` passos seguidos; a rolar ⇒ binário constante
da capacidade); as sondas `probe_o_rolamento_na_pilha` (caixas e discos, `Rolling 0 · 0,1 · 0,25 · 0,75`, três janelas)
e `quanto_tempo_a_pilha_leva_a_assentar`; os gates `a_pile_of_discs_with_rolling_settles`,
`the_rolling_on_the_card_calms_the_pile`, `a_piece_rolling_on_a_piece_stops_as_it_does_on_the_bowl` (os tempos da taça
`±2 %`), `the_rolling_friction_locks_the_ball_on_the_ramp` (a `=115`, `< 0,035`); a prova de mutação
`docs/Motion Nodes/ferramentas/mutacao_a_onda_5_2026-10-06.py` (`16/16`).
⛔ AS RECUSAS MEDIDAS (não reconstrua sem ler a tabela do `rolar.rs`): cortar o `ω` depois do passo · binário com ganho
`1` e `½` · o amortecimento angular do rapier · o travão RELATIVO por par · o contacto ACHATADO pelo gancho
`modify_solver_contacts` (acelera a bola) · destrancar ao 1.º excesso · «só tranca se os contactos não pedem giro» ·
«espera 4 passos depois de destrancar».
⚠️ ARMADILHA medida: um gate/sonda que monta `sim_t = k·dt` em vez de acumular o relógio faz o mundo RENASCER a meio
(o travão lia `0,74×`). Marche pela porta do app (`advance_or_scrub_scoped` + `peek`) ou acumule `t += dt`.

A TAREFA:
 (1) PROVE OU REFUTE A LEITURA antes de mexer na lei (é a regra: o mecanismo antes da cura). Na pilha de discos
     `Rolling 0,1`, por disco e por tique, na janela `120..180`: a rotação líquida, o deslocamento ao longo da superfície
     (um disco a ROLAR desce `≈ Δθ·R`; um que GIRA no lugar não sai do sítio), quantas vezes tranca/destranca por segundo,
     e a inclinação do monte (o perfil de altura) com e sem o botão. Escreva no doc 121 §9.23 ANTES de construir:
     é avalanche (rolar a descer um monte mais inclinado — física, fica) ou é a trava a alternar (defeito, cura-se).
 (2) Se for defeito, o candidato a medir primeiro é o rolamento DENTRO do solver por uma junta do próprio rapier: um
     MOTOR angular com velocidade-alvo `0` e força máxima `capacidade / dt` (uma junta entre o disco e um corpo fixo,
     com os eixos lineares livres) — o solver resolve-o nas iterações dele, limitado, sem trancar nem destrancar. Meça o
     custo dele a `4 096` e `16 384` (as juntas a nascer e a morrer com os contactos) antes de o adoptar. Outras ideias
     entram na MESMA rodada, com o porquê escrito antes (CLAUDE.md §0.10: um bloco, uma rodada de medição).
 (3) Kill-criteria (escritos aqui, antes): discos `Rolling 0,1` na janela `120..180` `≤ 2×` o rodopio sem o botão
     (`≤ 21°`); e NENHUMA regressão — a tabela das caixas e dos discos a `0,25`/`0,75` (`rolar.rs`), a bola que pára nos
     tempos da taça `±2 %`, a rampa da `=115` `< 0,035`, os gates da pilha, `4 096` por taça a `60` fps na queda inteira
     (o `[motion-quadro]` em `release`), duas corridas iguais ao bit, o recuo seguido de Play ao bit.
DoD: construído com a tabela antes/depois e os kill-criteria, ou recusado por medição (e então a leitura do §9.21
provada, com o número); gate batched verde (`nextest-impacted`, check com warnings negados, clippy, fmt, censos);
mutações a sangrar (as `16` de hoje + as da lei nova); doc 121 §9.23; handoff §16; memória; smoke ao dono FOTOGRAFADO
em `--release` (a `=114` em discos, `Rolling 0,1`), em passos numerados; NÃO integra, NÃO pusha. No relatório: ficou
ALGUMA coisa em aberto?
A MÁQUINA É PARTILHADA: pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
