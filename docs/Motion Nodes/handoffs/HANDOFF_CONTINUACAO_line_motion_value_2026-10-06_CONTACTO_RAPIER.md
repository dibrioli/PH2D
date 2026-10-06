═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ DECISÃO DO DONO (06/10): *«Usar o motor da casa»* — a colisão das formas do Motion (o contacto entre peças com
   `Collide` no cartão da forma) passa a ser resolvida pelo `rapier2d` (o motor do módulo de Física, ADR-0131), no
   lugar do solver PBD próprio (`ph2d-contact`). Motivo MEDIDO (doc 121 §9.19, o oráculo): a mesma pilha custa
   `0,40` ms por tique a `1 024` caixas no rapier e `~15` no nosso; `3,0` (`1,7` em paralelo) a `4 096` contra `48`–`93`.
⛔ Ordens que continuam: integrar só quando não houver nada em aberto; uma onda só; série só com o porquê escrito
   antes; NUNCA matar o pai de um processo (LOGOUT do dono em 05/10).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -5 && git status
        --short --ignored | grep -v target/ ; rebase no main se ele andou (conflito fora dos seus ficheiros ⇒ PARE).
FASE 1 (leia DENTRO da worktree): doc 121 §9.19 (5) INTEIRO, do «a medição» ao «o ORÁCULO»; o handoff de integração
        de 05/10 §12 (e o «06/10»); `crates/ph2d-node-sim-step/src/contact.rs` (o ponto de entrada de hoje:
        `separate` + `impulsos`); `crates/ph2d-contact/src/lib.rs` (o `Colisor`, `declarado`, `colisores`,
        `inv_inercias`, `materiais`); `crates/ph2d-physics` (como a casa já usa o rapier: `enhanced-determinism`,
        `glam`, a ponte ECS, ADR-0131); docs 109/111/115 do Motion (o que o dono aprovou na pilha: o colisor na
        forma, a rotação destravada e o `Lock Rotation`, o material — Friction/Bounce/Rolling —, o zumbido curado).

O QUE JÁ EXISTE (não reconstrua):
- O oráculo: `docs/Motion Nodes/ferramentas/oraculo_rapier_pilha/` (a pilha da `=114` no rapier, fora do repo).
- As réguas: `[motion-quadro]` (`PH2D_MOTION_RELOGIO=1`, janelas de `30` quadros — leia a PIOR, em `release`); a
  sonda `custo_do_tique_do_app_na_pilha` e a `prova_dos_impulsos_da_placa` (vizinho mediano · velocidade ·
  sobreposição) em `motion_state_pilha_demo_sondas.rs`; as portas `PH2D_PILHA_LADO`/`PH2D_PILHA_COLIDE` (a taça
  cresce com a pilha).
- Os gates da pilha (`motion_state_pilha_demo_tests.rs`: só a metade com Collide separa, as duas taças apanham, o
  cartão faz o que o anúncio diz, o `Rolling` acalma, as peças tombam salvo `Lock Rotation`, …) — são o
  COMPORTAMENTO aprovado pelo dono: ficam verdes com o rapier, ou o desvio vai ao dono no smoke com o porquê.

A TAREFA:
 (1) O DESENHO, escrito antes de construir (doc 121 §9.20): o rapier tem ESTADO entre tiques (os contactos
     persistentes e o aquecimento são o ganho) e o cozimento do Motion avança, recomeça (`Loop` da zona) e volta
     atrás (scrub). Decida e escreva: onde vive o mundo do rapier (por zona/sink, no estado do pump), como se
     sincroniza com o stream (posição, velocidade, `rot`, `spin` por id estável de elemento — nascimentos e mortes
     de peças), quem integra (o `sim.step` hoje integra semi-implícito com amortecimento; o rapier integra com
     contactos — uma integração só, nunca duas), o que acontece no scrub e no recomeço (reconstruir o mundo), e a
     taça (`sim.collide` Bowl: hoje um nó a seguir ao passo; dentro do mesmo mundo como colisor fixo é o que dá
     pilhas estáveis contra a parede — meça as duas). O mapeamento: caixa (com desvio e rotação) → `cuboid`; disco →
     `ball`; `inv_inertia = 0` / Lock Rotation → rotação travada; `inv_mass = 0` → corpo fixo; Friction/Bounce →
     os do rapier (a regra de combinação: a nossa é `√(a·b)` e `max`); **Rolling**: o rapier `0.35` não tem
     resistência ao rolamento? — confira no fonte da versão do Cargo.lock e, se não tiver, escreva a lei que o
     substitui (amortecimento angular por contacto) e meça-a contra o gate `the_rolling_on_the_card_calms_the_pile`.
 (2) PROVA antes do produto (two-strikes: é a 4.ª lei do contacto desta linha): uma sonda que corre a `=114`
     (`25` e `1 024` e `4 096` por taça) pelo novo caminho e pelo de hoje, no mesmo processo — tempo por tique e a
     régua da `prova_dos_impulsos_da_placa`. **Kill-criteria (escritos aqui, antes):** tique a `4 096` por taça
     `≤ 5` ms na CPU (`release`); a pilha de `25` e a de `1 024` dentro da banda da ordem (vizinho mediano `±5 %`,
     velocidade `≤ 1,5×` + `0,01` u/s, sobreposição mais funda `≤` a de hoje) OU a diferença mostrada ao dono no
     smoke e aprovada por ele; os gates da pilha verdes; determinismo: duas corridas iguais ao bit.
 (3) O PRODUTO: o `sim.step` passa a resolver o contacto pelo rapier; o `ph2d-contact` fica para quem ainda o usa
     (o `motion.collide` de discos na placa e o passe do sink — confira os chamadores; o que ficar sem chamador SAI,
     com o código, e o porquê no doc). As portas de prova de 05/10 (`PH2D_CONTACT_JACOBI`/`PH2D_CONTACT_CORES` e as
     composições `jacobi`/`cores` do `impulso.rs`) saem se nada as usar.
 (4) A MEDIÇÃO no app: `[motion-quadro]` em `release`, `=114` com `PH2D_PILHA_LADO` `32`·`64`·`128`, as duas placas —
     a pior janela por ciclo. Alvo: `60` fps a `4 096` por taça.
 (5) O item da placa (o contacto no dispositivo, prompt de 05/10 `…_CONTACTO_NA_PLACA.md`): reavalie com os números
     novos; se a CPU com o rapier chega a `16 384` com folga, a cerca da placa fica e o item fecha com a medição.
DoD: construído com a tabela antes/depois e os kill-criteria, ou recusado por medição; gate batched verde; mutações a
sangrar; doc 121 §9.20; handoff §13; memória; smoke ao dono FOTOGRAFADO em `--release` (a `=114` a cair, com a régua
`[motion-quadro]` ao lado), em passos numerados; NÃO integra, NÃO pusha. No relatório: ficou ALGUMA coisa em aberto?
A MÁQUINA É PARTILHADA: pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
