═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8).
⛔ ORDEM DO DONO (05/10): «integrar só quando não houver nada em aberto» e «resolver tudo numa ÚNICA onda». A 4.ª
   onda fechou quatro dos cinco itens; ESTE é o quinto: o contacto da CAIXA na placa. A linha continua sem integrar
   e sem push (CLAUDE.md §0.7): no fim você reporta e PARA.
⛔ REGRA DO DONO (§0.10): um plano, um lote de binários, UMA rodada intercalada, um gate batched; série só quando
   um passo muda o seguinte, e diga-o por escrito ANTES.
⛔⛔⛔ NUNCA mate um processo pelo PAI (o pai de um órfão é o `systemd --user`: LOGOUT do dono em 05/10).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -5 && git status
        --short --ignored | grep -v target/ ; git cherry main HEAD | grep -c '^+' ; se o main andou: range-diff e
        rebase (conflito fora dos seus ficheiros ⇒ PARE e reporte ao Enio).
FASE 1 (leia DENTRO da worktree): doc 121 §9.19 INTEIRO (`docs/Motion Nodes/121_as_formas_na_placa.md`, o fim) —
        a medição da `=114`, a cura do `O(n²)`, a PROVA do modelo (Jacobi refutado, Gauss–Seidel por cores aceite),
        a prova de custo dos despachos e a decisão; o handoff de integração de 05/10 §12; o doc 115 §9.5/§11/§14
        (a recusa antiga); DIRETIVA_IMPLEMENTACAO §5 (two-strikes).

O QUE JÁ EXISTE (NÃO reconstrua):
- A cura exacta da CPU: `ph2d_contact::impulsos` monta os pares pela grelha da separação (`impulso.rs`, gate
  `os_impulsos_pela_grelha_dao_os_bits_de_todos_os_pares`). `1 024` peças por taça a `60` fps.
- As composições alternativas dos impulsos, ATRÁS de portas de medição: `Leis::jacobi` (`impulso_jacobi.rs::
  uma_iteracao`, `PH2D_CONTACT_JACOBI=<it>`) e `Leis::cores` (`por_cores`, guloso pela ordem, `PH2D_CONTACT_CORES=1`).
- As sondas: `custo_do_tique_do_app_na_pilha` (o `advance_or_scrub_scoped` da ponte), `prova_dos_impulsos_da_placa`
  (vizinho mediano · velocidade · sobreposição na pilha da direita), `custo_de_um_despacho` (`1,6` µs por despacho
  num passe, as duas placas), `custo_do_cozimento_da_pilha`.
- As portas da régua do app: `PH2D_PILHA_LADO`/`PH2D_PILHA_COLIDE` (a taça cresce com a pilha),
  `mede_formas_na_placa.sh` com a `=114` (`COLIDE=`, `CELULAS="igpu:114:64 …"`). ⚠️ O roteiro de foto RECUSA um
  binário mais velho que qualquer `.rs` (testes incluídos): depois do build, `touch target/release/ph2d-host-desktop`
  quando o cargo confirmar que nada do binário mudou, e NÃO edite `.rs` durante a matriz.
- O kernel de DISCOS do `motion.collide` (`crates/ph2d-node-motion-collide/src/gpu.rs`, `GridSpec` com varreduras
  e a grelha reconstruída em `ph2d-gpu-cook/src/lib.rs`), o `sim.collide` no dispositivo (já tem `SC_BOX` por
  params), o estágio feito à mão de várias passadas (`ph2d-gpu-cook/src/voronoi.rs`, o molde).

A TAREFA — o contacto da caixa NA PLACA (4.ª topologia do dispositivo ⇒ two-strikes; os modelos de LEI e de CUSTO
estão provados; falta o do SEQUENCIADOR):
 (0) A RÉGUA DO APP já existe: `PH2D_MOTION_RELOGIO=1` imprime `[motion-quadro]` a cada `30` quadros (a parede, o
     Motion médio e máximo, os tiques por quadro). A `=114` cai sozinha no roteiro de foto (a demo força a ferramenta
     `motion`, que entra em Play); ⚠️ uma FOTO é um instante do ciclo (a queda recomeça a cada `3,6` s) e não prova
     que a cena parou — leia as janelas. O critério do quadro lê-se na PIOR janela, em `release`.
 (0b) PROVA do sequenciador: a `sim.zone` no dispositivo é passagem (inventário de 05/10) — meça se a
     `=114` SEM colisão corre os `8` sub-passos na placa (paridade por passo com a CPU nos gates `gpu_cpu_parity_sim`
     com `substeps = 8`). Se não corre, os sub-passos da zona no dispositivo são o 1.º bloco desta onda (sem eles o
     contacto não tem onde viver).
 (1) A LEI ÚNICA nas duas mídias: as cores por RONDAS com prioridade por par (`hash(lo, hi, ponto)`; uma restrição
     colore-se na ronda em que é o máximo entre as não coloridas que partilham uma peça) — determinística na CPU e
     na placa, logo o Gauss–Seidel por cores dá a mesma ordem nas duas. O produto da CPU passa a ela (a prova diz
     que fica na banda da ordem; re-meça com a `prova_dos_impulsos_da_placa` e os gates da pilha).
 (2) O estágio do contacto no dispositivo, logo a seguir ao `sim.step` em cada sub-passo: a grelha, `8` varreduras
     de separação (disco×disco, caixa×caixa pelo eixo separador e o trecho recortado, disco×caixa; o giro por
     varredura; Jacobi com média — o `par.rs`/`varredura.rs` termo a termo), a montagem das restrições (append
     atómico, `1`–`2` pontos), as cores por rondas, `8` iterações × as cores (normal, atrito, rolamento — o
     `resolve_um`), o spin. Tudo num passe por sub-passo (`1,6` µs por despacho).
 (3) As colunas do colisor pela rota da forma no dispositivo (`collider_box`, `collider_offset`, `inv_inertia`,
     `friction`/`bounce`/`rolling`, `rot`, `spin`) e o `sim.collide` a pousar a caixa pelo SUPORTE dela
     (`ponto_de_suporte`, a rotação) como a CPU.
 (4) As cercas de `motion_bridge_gpu_colisor.rs` caem SÓ para o que o kernel honra (caixa e disco declarados na
     forma; o colisor externo da membrana e o passe armado do sink continuam a cair para a CPU até terem kernel).
KILL-CRITERIA (escritos antes): paridade por PASSO contra a CPU (`1` tique, `8` sub-passos) — posições `≤ 2e-3` e
giro `≤ 0,1°` nas fixturas de caixas rodadas e discos (o `ε` do gate de discos); a pilha da `=114` (`25` e `1 024`)
dentro da banda da ordem da `prova_dos_impulsos_da_placa`; pior janela `[motion-quadro]` do app (`release`) com `Collide`:
`4 096` por taça `≤ 16,7` ms na iGPU e `16 384` `≤ 33` ms na RTX; os gates da pilha verdes nas duas rotas. Falha ⇒
a cerca fica, recusa medida com a tabela.
Leis que já custaram caro: a régua é a soma dos passes e o resumo o mínimo; um critério de semelhança precisa de
um A/A (a reordenação do Gauss–Seidel é o controlo); uma sonda de simulação tem de AVANÇAR (o `advance_tick`);
uma recusa medida sobre a metade barata de um passo renova-se em falso; a porta que cresce a população cresce o
recipiente. Os tectos de LOC (`impulso.rs` `~660`, `contorno_cpu.rs` `613`, `motion_shape_gen.rs` `672`): cresceu?
MOVA para um irmão.
DoD: construído com tabela antes/depois e os kill-criteria, ou recusado por medição; gate batched verde; mutações
a sangrar; doc 121 §9.20; handoff §13; memória; smoke ao dono FOTOGRAFADO (a `=114` com colisão em Play na placa:
`[motion-route]` a dizer dispositivo); NÃO integra, NÃO pusha. No relatório diga se ficou ALGUMA coisa em aberto.
A MÁQUINA É PARTILHADA: pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
