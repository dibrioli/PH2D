# Handoff de continuação — GEF 08 Motion Input, onda 3 (16/11)

Linha `line/GameEngineFuncional` (a linha CONTINUA; não é fecho de linha nem integração). Ficha:
[08_motion_input.md](../componentes/08_motion_input.md). **Não integrado, não pushado. Smoke do dono da onda 3: PENDENTE.**
Base da onda: `1656e7f028`; 40 commits, o último `2ea82fa397`.

## Decisão do dono (16/11)

Sobre a maquete [08_onda3_maquete.png](../componentes/08_onda3_maquete.png): **1B** (cada entrada = objeto filho «Motion Input»),
**2A** (nó «Send signal» no grafo: When Pulse · Rises above · Falls below + Level; uma vez por passagem), **3A** (a origem
decide o par; sem seletor Type).

## O que a onda fez

- **MI-10** (`af790020b2`, cartão `9b51d1b354`): `MotionInput` só num filho (`pendura_entrada`; Add ▸ Motion Input e a paleta
  criam o filho e selecionam-no). `from = 0` = o pai (`ph2d_ecs::motion_input::dono`); escritor nomeado pelo pai; estado da
  ponte por bits do filho; a ponte renomeia o filho «Motion Input · <nome>» só quando muda. Inspector do filho: só cabeçalho +
  cartão Motion Input (sem Transform/Ordering/corpo: `3f765643c7`, `c69314500a`, `95f0e83f27`); sem gizmo nem arrasto
  (`35bd76f8a2`); ícone Gamepad + token `obj-motion-input` (`c4de65a78f`).
- **MI-9** (`b0251b2323`): `motion.get(nome)` — a 1.ª linha (quem escreve primeiro), espelho no início do passe de script
  (tique anterior), `nil` sem escritor; número, cor ou vec2.
- **MI-5** (`58addc13df`): `Valor::Vetor([f32;2])`; `value.input` ganha `xy` (Vec2, porta 6), `x`, `y` no fim; `out` = tamanho;
  fontes novas Velocity (corpo do pai) e Move (move_right−move_left, move_up−move_down das ações do Input Map; não há ação 2D
  no Input Map); Property de campo Vec2 manda o par; Remap/curva no tamanho, Smooth por componente; `motion.set(nome,
  ph2d.vec2(x,y))`. **PROJECT_SCHEMA 246 → 248** (247 filho, 248 fontes).
- **Property** (`e6ea70e412`): só oferece o que lê (filtro `canal_componente::resolver`; o gate varre todos os componentes);
  Transform · Position/Rotation/Scale pelo PropKind do Auto Key (`read_prop_kind`); saem linvel, force, offset, Skew e campos
  de variante ausente.
- **MI-8** (`aa36d90ac5`, `b073f8511c`, `e435e7e849`): crate nova `ph2d-node-send-signal` (`value.send_signal`), categoria
  Output; marchado como `pulse.signal` em `signal_nodes`; borda em `Envios::passo` (o nível exato só conta do lado da chegada;
  o 1.º tique e um salto não disparam; Play zera a base); publica só com `Playhead::crosses_forward` (ph2d-core; o relógio a
  andar para a frente, também a Timeline em Edit, como o pulse.signal). `SignalOrigin::Motion { source }` leva o objeto Motion
  dono do nó (regra «From: Myself» e Luau `on_signal(self, signal, from)` recebem-no); o censo de sinais conta-o; linha na
  folha de conferência `12_pulse.md`. Cartão «Motion»: «Sent X N times / once / not sent yet», só em Play, zera a cada Play,
  Stop guarda (`e2c6a75c57`).
- **Cook GPU** (`2d3867782e`): a fronteira CPU→GPU passou a (nó, porta) (`boundary_port_streams` em ph2d-eval-motion,
  `set_boundary_ports` e cache por (nó, porta) em ph2d-gpu-cook, regra de recuo removida de `plan_uniao.rs`, `BoundaryMismatch`
  se faltar porta): um nó GPU que lê `value.input.xy` ou `.r` fica na placa. Testes de placa
  `o_par_e_a_cor_chegam_a_placa_como_na_cpu` e `o_send_signal_pela_placa_cai_uma_vez_e_fala_pelo_objecto` verdes (RADV integrado).
- **Cena 28 «o herói manda»** (`crates/ph2d-physics-scene/src/cena_heroi_sinal.rs`) + grafo de demo 143
  (`crates/ph2d-app-motion/src/motion_state_heroi_demo.rs`): Hero (Character) com filhos «Motion Input · vida» (Health → barra
  de 24 peças) e «· rastro» (Velocity → estrelas pequenas empurradas por dois ventos, saídas x e y); 4 pedras de dano 20 (vida
  1 → 0.2, cruza 0.25 uma vez); Send signal `low_health` Falls below 0.25; Warning Lamp redonda (regra «When signal low_health →
  Show»); Script Meter (Luau: largura = `motion.get("vida")`, engorda em `on_signal`); o Hero corre à direita (quadros 420–455)
  e volta (470–525). Opções de foto `heroi_property`, `heroi_move`, `heroi_motion_sent`, `heroi_send_signal`.

## Números

- `PROJECT_SCHEMA` 246 → 248 (DELTA +2). Cenas de smoke do Game: 27 → 28. Demos do Motion: até 134…142 → 143. Crate nova: 1
  (`ph2d-node-send-signal`). `NodeOp`/`OpResolver`/`NodeManifest` não mudaram (gate `architecture_contract_surface`).
- LOC: `ph2d-app-components` ~80 834/80 869; `ph2d-app-motion` ~197 514/197 516; `ph2d-app-game` 2 869/2 869 (no tecto);
  `ph2d-game-smoke-fotos` no tecto; a shell não cresceu.
- **Mutação: 47 mutantes; 38 sangraram** (27 à primeira + os curados em `8a04501aad`, `e03cd1d72c`, `9bf023f5b6`, `2d8c9830cf`);
  3 só de placa (porta 0 sempre; sem BoundaryMismatch; costura sem portas), corridos à mão: os 3 sangraram em
  `o_par_e_a_cor_chegam_a_placa_como_na_cpu`; 2 equivalentes (filtro `resolver` redundante em `chaves_do`; Velocity do próprio
  objeto não corrido, palpite); grupo 4 só o nome do filho corrido; 1 mutante fraco do salto de tiques descartado.
- Fecho: `cargo fmt`; clippy `-D warnings` verde em 26 crates (curas `89f3599d38`, `dbd1a8a1a8`, `2ea82fa397`); testes da onda por
  nome todos verdes (editor-core 5, nodegraph 3, desktop 5, signal-census 1, app-motion 8, send-signal 10, panel-inspector 23,
  app-components 25, game-scenes 22, ecs 3, script 2, runtime 1, value-input 1, gpu-cook 2, physics-scene 2, app-vec 1, core 1)
  + os 2 de placa. Binário smoke compilado 2× (2.ª: `Finished smoke profile [optimized] target(s) in 0.27s`, zero Compiling).

## Superfície de colisão e o que um merge pode partir

- Foundational tocado: `ph2d-ecs` (`motion_input::dono`, `MotionInputs` com vetor, `SignalOrigin::Motion`), `ph2d-core`
  (`Playhead::crosses_forward`), `ph2d-eval-motion` e `ph2d-gpu-cook` (fronteira por porta), `PROJECT_SCHEMA` (3 sítios),
  `ph2d-runtime`/`ph2d-script` (`motion.get`, par em `motion.set`), `signal_nodes`.
- Um merge pode partir: o degrau do schema (**recontar** com `python3 scripts/schema-recount.py`); o registo de nós (crate nova);
  o censo de sinais e as cenas de smoke (28 e demo 143: contar, não escolher); catracas de LOC das crates acima.
- ⚠️ **PARA O INTEGRADOR:** para caber nas catracas, a onda cortou SÓ comentário `//` histórico (zero código) em: 17 ficheiros da
  `ph2d-app-components` (`831043380b`) + `asset_card_verbs.rs`, `instance_verbs.rs`, `motion_input_bridge/inspector/filho`, 4
  ficheiros de teste; na `ph2d-app-motion`: `motion_bridge_signals.rs`, `motion_state_pilha_demo.rs`,
  `motion_state_gpu_neighbour_demos.rs`, `motion_state_carimbo_demo.rs`, `motion_shape_gen.rs`, `motion_bridge_intents.rs`
  (344 linhas), `motion_scene_ctx.rs`; na `ph2d-app-game`: `smoke.rs`, `posse_do_jogo.rs`, `encaixe.rs`, `smoke_moedas.rs`;
  compactou `nomes.rs` da `ph2d-game-smoke-fotos`. Em conflito de comentário, vale o lado do `main` desde que o código case.
- Prova de fecho: os testes por nome acima + a mutação + o smoke compilado. A suíte completa e a paridade GPU×CPU ficam para o
  integrador.

## ABERTO

1. **3 testes de paridade GPU×CPU vermelhos JÁ ANTES desta onda** (medido revertendo o commit da fronteira):
   `gpu_cpu_parity::the_projection_angle_arm_matches_the_cpu_within_epsilon`,
   `gpu_cpu_parity_sim::the_emitter_burst_sim_matches_the_cpu`,
   `gpu_cpu_parity_strobe::the_shaped_strobe_matches_the_cpu_tick_by_tick` (ph2d-gpu-cook, `--ignored`, RADV integrado):
   o integrador confere contra o `main`.
2. Doc-comments velhos que dizem «só a porta 0» (campo `boundary_streams` e o dedup em `ph2d-eval-motion`; doc 120 §8.3.3) e a
   sonda manual `sonda_os_defeitos_latentes_da_entrega` (parte (a) obsoleta).
3. A foto `d` não mostra o nó aberto com When/Level (o editor de nós abre pequeno atrás da Timeline).
4. O rastro não segue o herói (estrelas nascem num ponto fixo; a legenda diz «empurradas pela velocidade»).
5. Sem «mira»/Move nem Grounded na cena 28 (Move está na foto `f`, pela opção `heroi_move`).
6. A regra «Send signal» publica também com a Timeline a tocar em Edit (como o `pulse.signal`).
7. Herdados: handoff GEF 07 > 15 KB (cortar com `python3 scripts/doc-split.py`), `oraculo_lsp` G5b do Ray (luau-lsp não
   instalado), a galáxia da cena 27 por cima do Hit Dummy.

## Próximo

Smoke do dono da onda 3 (abaixo). Depois: fecho da linha ou próximo item GEF — **perguntar ao Enio**, não decidir aqui.

## SMOKE DO DONO (onda 3) — o herói manda

Fotos em `fotos_08/`; a comparação com a maquete: [lado a lado](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_lado_a_lado.png).

1. **Abrir.** No terminal:
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GPU_COOK_DEMO=143 PH2D_GAME_SMOKE=28 ./target/smoke/ph2d-host-desktop`
   Errado se: o app não abre ou a cena vem vazia.
2. **Parado, olhe a Hierarquia.** Deve haver o **Hero** com dois filhos, **Motion Input · vida** e **Motion Input · rastro**, cada
   um com ícone de controle de jogo. Clique em «Motion Input · rastro»: o cartão mostra From **«Hero (parent)»**, 10 botões de
   Source e **«Reads Velocity here: (0.00, 0.00)»**. Foto: [parado](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_a_parado.png).
   Errado se: faltar um filho, o From disser outra coisa, ou o filho mostrar Transform/Ordering.
3. **Play.** Clique em Play. As pedras caem no herói, a barra de vida encolhe peça a peça, e quando a vida passa de 0.25 a
   **lâmpada acende**, aparece o aviso **«Signal: low_health»** e a barra amarela (Script Meter) engorda e muda. Fotos:
   [herói corre](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_b_play_corre.png) ·
   [pedras e lâmpada](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_c_play_pedras.png).
   Errado se: a lâmpada acende mais de uma vez, não acende, ou a barra não muda.
4. **O herói corre.** Logo depois ele corre à direita e volta; as estrelas voam empurradas pelos ventos. No cartão do rastro:
   **«Sends (x, y)»** com números que mudam. Errado se: as estrelas ficam paradas com o herói em movimento (o cartão continua
   a mudar? então é o aberto n.º 4, conhecido, não é defeito novo) ou os números ficam em 0.
5. **Clique no objeto Motion** (o da cena, não o filho). O cartão «Motion» diz **«Sent low_health once»**. Errado se: disser
   «not sent yet» depois de a lâmpada acender, ou «N times» com N > 1. Foto: [enviou](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_c2_motion_enviou.png).
6. **Pare (Stop)** e clique no filho **«vida»**. Troque **Source** para **Property**: abre uma lista com **«Transform · Position
   (x, y)»** entre as opções, e sem opções que não leem (nada de força ou velocidade linear). Foto:
   [Property](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_e_property.png).
   Errado se: a lista não abre, abre cortada ou não tem Position.
7. **Troque Source para Move.** O cartão diz **«Reads move_right · …»** (as ações do Input Map). Foto:
   [Move](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_f_move.png).
   Errado se: aparecer outro texto ou nenhum. (Desfaça com Ctrl+Z para voltar a Health, se quiser rever o Play.)
8. **Edit no Motion.** Selecione o objeto Motion, aperte **Tab**, abra o nó **Send signal** (na paleta, categoria Output): ele
   mostra **Signal**, **When** (Pulse · Rises above · Falls below) e **Level**. Foto da paleta:
   [Send signal](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_d_send_signal.png)
   (o nó aberto NÃO aparece nela). Errado se: o nó não existe na paleta ou When/Level não mudam ao clicar. Se o editor de nós
   abrir pequeno atrás da Timeline, é o aberto n.º 3.
