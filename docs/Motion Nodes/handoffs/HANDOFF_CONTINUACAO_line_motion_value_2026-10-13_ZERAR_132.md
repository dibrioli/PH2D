# CONTINUAÇÃO — `line/motion-value`: ZERAR as seis pendências do 131 (doc 132, P1–P6) — FECHADAS (13/10)

> Ordem do Enio: as pendências do [ZERAR_131](HANDOFF_CONTINUACAO_line_motion_value_2026-10-12_ZERAR_131.md) num
> ciclo (§0.10). Plano, leis, tabelas e recusas: [doc 132](../132_zerar_as_seis_pendencias.md) (§4.1, §7 = os
> números; a parte antiga do §7 em `archive/132_zerar_as_seis_pendencias/`).
> **Este ficheiro substitui o ZERAR_131 como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `b23e0156f9`, base = `main`
  `757afebd58` (`git log 757afebd58..HEAD`: 49 commits). **Nada integrado nem enviado.**
- `MAX_DEMO_LEVEL = 141` (`motion_state_demo_router.rs:34`): cenas novas `=138`…`=141` + `PH2D_MOTION_OBJ_SMOKE=18`.
  Cena nova que outra linha criar tem de recontar o número no roteador.

## O que fechou (o número decisivo; o resto no doc 132 §7)

| item | lei / cura | medido |
|---|---|---|
| **P5** aviso | quebra até 7 linhas, orçamento único (284 → 300 px), `ttl = máx(3 s, palavras·60/238)` (Brysbaert 2019); as barras descem pelas LINHAS. A cura do aviso nomeava `audio.bands` (1.º de 13 candidatos que aceitam a corrente pela CONTAGEM) com o nome cru: agora o `Attribute` pelo nome do menu (`7407808dd1`) | censo 432 frases/383 sítios: 244 cortavam → 0 |
| **P2** fita | `Historia::{Verdade, Dobrada, NaoGravada}`; faixa na régua SÓ da aba Arrange (`RulerClock::history`): o Motion lê o relógio do EDITOR, a régua da Keys sem pilha mexe o do CLIP — a faixa lá mentia. Tecto = 1/32 da RAM física, razão partilhado (fitas e anéis) | 8 GiB → 256 MiB (300 entradas: 5 min); 128 GB → 3,85 GiB (77 min) |
| **P2** anel | declarava 0 B por checkpoint leve e retinha ~1,9 KB: régua dhat do anel; o gate do cartão corria só com placa, agora CPU sempre; na placa arrastar antes do 1.º gravado esquecia a fita (`tique_das_entradas`) | declarado ÷ retido 1,00× |
| **P1** borrão | causa = custo por pixel (83 % da pilha); borrão por GRUPO (memória partilhada) é o único; vala POR LADO | Glow 4 096 discos `3,57 → 2,96 ms`, Drop Shadow `4,92 → 3,93`; Drop Shadow longa `6,53 → 3,38` (7 → 3 pág.) |
| **P1** `=138` no app | fazia 11 págs/11 ms: cada forma pagava uma célula a mais — a linha CALADA (quad 0) do buffer das sprites desde o W3 `584688eee8` (`001077631a`, `Variante::calada_vazia`). O resto é custo real (discos 18 px, Glow σ 9 ⇒ ~25 Mpx). O span `render.vello` somava a camada filtrada | placa 1,3–7,6×; 7 págs; `fx_cells.pilha` ~6–7 ms; `render.vello` 15 → 0,3 ms |
| **P3** `corrige` | 9 cursores desenrolados por `select`: mesma soma ao bit; gate que VÊ a ordem da soma (a mutação «fusão pela maior cabeça» sobrevivia) | nuvem 2 048 um grupo `33,93 → 19,26 ms`; `PECAS_PARA_UM_GRUPO` 1024 → 1536 |
| **P4** imagem em grupo | vai à PLACA; vigia e janela de 1–2 quadros eliminadas; mundo por baixo pousa com o modo (sem ler o backdrop); `MundoPartido` por segmentos; Each Shape com imagens filtra cada elemento nas duas rotas; textura assada na CPU; `Scene` fora da camada isolada fica na CPU por LEI | CPU 90,5 × placa 1,25 ms a 16k; janela 0 em 6 quadros; +0,12 ms/segmento; `Scene` por linha 23,8/97,5/312 × por média 2,0/2,4/4,9 ms (1k/4k/16k) |
| **P6** silhueta | dois defeitos: sampler do átlas na CPU ≠ do projecto; translação ímpar troca o quadrado das derivadas. Cena: `PH2D_MOTION_OBJ_SMOKE=18` (grafo sem objecto não tem sprites) | 0 texels (eram 61 e 75) |

## Mutações (todas RED sobre VERDE, árvore restaurada)

P2 7/7 + 6/6 (pontas) · P5 10/10 + a do desconto (M3 sobrevivia: o gate media-se na própria porta, agora no CARTÃO)
+ a das barras (gate novo) + cura 2/2 · P1/P3/P6 6/6 + a da ordem (gate novo) · P4 9/9 + 7/7 + 4/4 (gates dos dois
vermelhos) + 2/2 (calada).

## Aberto

- Na aba Keys o Motion não segue o relógio que o artista arrasta (desenho do módulo).
- `Scene` na placa por UM pipeline que desenhe os dois meios na ordem do buffer (escrito, não construído).
- Cadeia 16·4096 do `corrige`: mínimo pior, mediana melhor — não decidida.
- 5 sítios de aviso fora do censo (texto só em execução): `component_attach.rs:199`, `fase_physics_step.rs:113`,
  `fase_view_frame.rs:49`, `fase_sprite_pixel_smokes.rs:49/51`. O gate do censo custa ~16 s.
- 3 testes a ~150 s do tecto de 180 s sob carga: `where_the_drops_note_shows_up`, `que_numeros_o_size_tem`,
  `o_preco_das_imagens_de_um_grupo_cpu_contra_placa` (medida sem nome de sonda).
- Flake de carga de OUTRA linha (`line/PainterWatercolor`): `ph2d-tool-painter … a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`
  (PASS/FAIL/PASS a load 45, sem diff) — candidato à família do [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
- U2 continua (a unidade é do PROJECTO, não deste nó).

## Smoke ao dono (cenas fotografadas, 1600×944)

Comando base (cada passo muda só as variáveis):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && <VARS> cargo run -p ph2d-host-desktop --profile smoke`

1. `PH2D_GPU_COOK_DEMO=138 PH2D_FLUID_PROFILE=1` — milhares de discos a cair; o terminal mostra o custo (~7 páginas).
2. `PH2D_GPU_COOK_DEMO=139 PH2D_MOTION_FITA_BYTES=384k` — na aba **Keys** (a que abre; desde 13/10 — ADR-0115 R8
   emenda 3 — a régua dela mexe o relógio do Motion sem pilha) arraste a régua para trás: a faixa
   da história começa mais tarde e o cartão das entradas avisa ao passar do começo dela.
3. `PH2D_GPU_COOK_DEMO=140 PH2D_MOTION_ROUTE_LOG=1` — imagens e formas misturando num grupo; o terminal diz a rota.
4. `PH2D_GPU_COOK_DEMO=141` — o fio do `Oscillator` solto no CORPO do `Transform` não liga nem abre menu: o aviso longo inteiro, ~5 s. ⚠️ 1.º report do dono (13/10): «o aviso não some e o modal de nós se abre» — o automático repetia-se, e o gesto do roteiro não existia (o `card_port_menu` filtra por `same_type` ⇒ menu vazio ⇒ a paleta). Curado em `cc74ff8412`: `GraphIntent::DropRefused` (o corpo que não aceita diz porquê; a paleta só no vazio), uma porta para a frase (`motion_bridge_recusa.rs`), `PH2D_MOTION_AVISO_AUTO=<s>` dispara UMA vez (só para a foto). ✅ Smoke do dono OK (13/10): `=138`–`=141` e `PH2D_MOTION_OBJ_SMOKE=18`.
5. `PH2D_MOTION_OBJ_SMOKE=18` — cópias de uma sprite a cair com sombra em cada uma.

Deu errado se: aviso cortado com «…»; a faixa da fita AUSENTE na régua da aba Keys (sem pilha); a `=138` a pedir 10–11 páginas.

## Depois do smoke: a Keys mexe o Motion (13/10, `65a853a5e8`)

Pedido do dono: *«na aba Keys arrastar a régua e dar play tem de mexer a simulação»*. A linha P2 acima
(«faixa SÓ na Arrange») é a lei VELHA: o defeito era a Keys sem pilha dirigir o `clip_playhead`
(o `keys_mode` de 07-27 é o ESCOPO e levava o relógio junto). Agora `ph2d_timeline::ViewClock` —
o clip só sob pilha; sob pilha o editor segue o clip que toca UMA vez (`follow_clip_clock`), e onde
toca 0/2 vezes a barra diz `scene stays: …`. Lei, desvio e gates: ADR-0115 R8 emenda 3. Foto da `=139`
na Keys com a faixa: verificada (scratchpad da janela, `139_keys.png`).

## Fecho desta janela

HEAD `b23e0156f9`, load 37–47. `PH2D_SUITE=fecho` nextest-impacted **20 486/20 487** (a falha = o flake acima) ·
check warnings=deny ✓ · clippy forma do `ship.sh` ✓ · fmt/typos/machete ✓ · GPU não-sonda 288/295 (4
`smoke_fixture_renderable::w2..w5` `unimplemented!` pré-existentes + 3 timeouts que passam sozinhos em 146–152 s) ·
gpu-cook 250/250 · censos 117/117 + 12/12 · doc-index e adr-index ✓. Doc 132 cortado por `doc-split.py` (sha256) abaixo de 40 KB.

binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida):
`Finished smoke profile [optimized] target(s) in 0.24s` (depois de `cc74ff8412`) — zero `Compiling`.
