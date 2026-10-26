# W3 — as poderosas (25/10/2026)

> Plano: [`01_plano.md`](01_plano.md) §2.4, §5 (onda 3), §6 cena 5, §7 W3. Oráculo:
> [`fixtures/blender_5_2_3_constraints_2d.json`](fixtures/blender_5_2_3_constraints_2d.json) (casos `transformation`,
> `action`, `floor`, `pivot`, `armature`, `spline_ik`, `vertice`, corridos por
> [`ferramentas/oraculo_blender.py`](ferramentas/oraculo_blender.py) ANTES do código; os casos W1–W2 saem iguais ao bit).

## 0. Look At é a porta única de «olhar» (decisão do dono 25/10)

Spine e Rive usam uma IK de 1 osso para «olhar»; Blender, Godot e Unity separam Look At de IK; o PH2D restringe objectos
**e** ossos como estes ⇒ **Look At** olha (objecto ou osso, com desvio), **IK** alcança a partir de 2 ossos.

**Medido antes de mudar** (`ph2d-constraints-ecs/tests/it/olhar.rs`, build limpo — ver a armadilha do §4):

| Caso (osso, alvo em (−0,4; 2,7)) | Look At genérico (W1) | IK de 1 osso (W2) |
|---|---|---|
| pai rodado 30° · desvio 0,3 | o mesmo bit | — |
| **pai espelhado** (`sx = −1`) | a frente DESENHADA a **π** do alvo | aponta (mira da casa) |
| **limite de junta** [−0,35; 0,35] | ignora-o: 2,143 rad | 0,350 |
| escala desigual (2; 0,5) · força 50 % | 1–2 ulp de diferença | — |

O mesmo π valia para **objectos** sob pai espelhado (o gate da W1 nunca correu o pai espelhado, que o plano §5 pedia).

**A lei** (`ph2d_constraints_ecs::ik::olha`): o eixo +X DESENHADO aponta ao alvo pela mira da casa
(`ph2d_skeleton_live::bone::aim_rotation_under`, certa sob espelho e escala), + desvio, misturada pelo ângulo no local,
aparada pelo limite de junta. A IK chama-a em cada osso da corrente; com um osso só mira o próprio alvo. A lei pura
(`Kind::LookAt`, oráculo `apontar_3_tipos`) ganhou o eixo do espelho (`sx < 0 → π`) para «manter desvio» não saltar.

**No produto:** o botão *Look At* do painel do esqueleto nasce um `Kind::LookAt`; a IK `chain = 1` gravada (ficheiro,
colagem, o `Chain = 1` do painel) converte-se num Look At no quadro (`converter_ik_de_um_osso`, só ossos — num objecto a
IK é `NaoEOsso` e fica). O losango, o arrasto do alvo, o realce, «governado», apagar e espelho leem a **âncora do osso**
(`ancora_de`: IK ou Look At); o painel mostra ao Look At só *Mix* e *Aim Offset*.

## 1. Medido antes de desenhar

### ⛔ Recusas MEDIDAS no oráculo

| Caso | O Blender 5.2.3 faz | Porque recusamos | A lei do PH2D |
|---|---|---|---|
| `transformation.rot_loc.*` (Replace, só o canal y mapeado) | zera os OUTROS dois canais de posição (x autorado 1 → **0**) | o artista mapeia um canal e perde outro sem o pedir | um item mapeia UM canal para UM canal; os outros ficam |
| `armature.linear.0.5` (dois pais a 50 %, um rodado 90°) | a escala do dono **encolhe** a 0,707 | mistura de matrizes (LBS): o objecto murcha a meio da passagem | posição linear, rotação pelo arco curto, escala em proporção (`blend`) |
| `armature.volume.*` (*preserve volume*) | a posição faz **arco** ((1,414; 1,414) a 50 %) e a rotação não é linear no peso (26,18° a 30 %) | o dono sai da linha entre as duas poses-candidatas | idem |
| Spring | o Blender não tem mola; a Physics do Spine só corre com interface | sem oráculo corrível | a lei da casa (`ph2d-node-motion-spring::law::passo`); o gate é o determinismo |

### ✔ Leis do oráculo que adoptamos (viram gate)

| Caso | Medido | Lei |
|---|---|---|
| `transformation.loc_rot.apara.*` | x do alvo [0; 2] → rotação [0°; 90°]: −1 → 0, 0,5 → 22,5°, 3 → 90° | linear, aparado nas bordas |
| `transformation.loc_rot.extrapola.*` | −1 → −45°, 3 → 135° | sem aparar |
| `transformation.loc_rot.soma.*` | autorado 10°: 0,5 → 32,5° | Add soma ao autorado |
| `transformation.rot_loc.*` | rotação [−90°; 90°] → y [0; 3]: −170 → 0, 45 → 2,25, 120 → 3 | idem, canal angular |
| `transformation.esc_esc.*` | sx [1; 2] → sy [1; 3]: 0,5 → 1, 1,5 → 2 | idem |
| `action.*` | rotação do alvo [0°; 90°] → quadro [1; 21] da acção (x 0→4, rot 0→90°, linear): 45° → (2; 45°), −45° → quadro 1, 135° → quadro 21 | linear, aparado; ⚠️ no Blender a faixa de rotação vai em GRAUS |
| `floor.*` | plano = alvo + desvio no eixo (+Y): abaixo → y 0,5; acima fica; tecto (`−Y`) → y ≤ 0,5; rodado 30° com `use_rotation` → projecção pela normal (0,067; 0,616) | idem |
| `pivot.*` | a rotação autorada acontece à volta do alvo: (2; 0) a 90° em torno de (1; 1) → (2; 2); −45° → (1; −0,414) | `p' = pivô + R(rot)(p − pivô)`, rotação igual |
| `spline_ik.*.NONE` | a raiz vai ao INÍCIO da curva; cada junta é o primeiro ponto da curva à distância (corda) = comprimento do osso; além do fim segue a tangente final (`curta`) | idem |
| `spline_ik.*.FIT_CURVE` | juntas na fracção do arco = comprimento acumulado ÷ total; cada osso estica até à corda (0,667 · 0,471 · 0,667 sobre o «L» de 2) | idem |
| `vertice` | Copy Location de um vértice de uma forma rodada 30° e escalada 2: (4,232; 0,866) | o afim da forma aplicado ao ponto |

### Composição que já existe (§5.0 do `CLAUDE.md`)

- **Multi-parent ponderado JÁ É a pilha:** *Child Of A* a 100 % + *Child Of B* à força `w_B` dá `blend(A, B, w_B)` — a
  média do *Multi-Parent* do Unity (posição linear, rotação pelo arco curto, sem encolher); com 3 pais, as forças
  normalizadas em sequência. Cada Child Of nasce sem salto. ⇒ **nenhum tipo novo**; gate de composição
  (`multi_parent_is_two_child_of_items`) contra a recusa do `armature.linear`.

- **Smart Bone JÁ É um Action para ossos** (`ph2d-app-skeleton/src/smart.rs:330` `drive`: o ângulo do osso escolhe o
  instante de um clip, `ph2d_skeleton::action_time`, `reach.rs:643`). ⇒ o Action da pilha GENERALIZA o canal (qualquer
  canal de qualquer alvo); medir na abertura do código se o Smart Bone passa a delegar nele (uma porta) ou se os dois
  são perguntas diferentes (o Smart Bone toca o clip na CENA; o Action toca-o no DONO).
- **Transformation:** expressão + *prop-link* da timeline (`frame_solve.rs`, `Nome.prop`) lê o canal de outro objecto, mas
  não tem faixa→faixa nem lugar na ordem da pilha ⇒ tipo novo.
- **Flip «troca desenho por ângulo»:** a escolha é `FlipLayer::drawing_at(frame)` (`ph2d-flip/src/layer.rs:118`), sem
  `PropKind` de desenho ⇒ um Action sobre um Flip não troca nada hoje; a peça precisa do canal (medir antes).
- **Ponto de forma:** `world_polyline` percorre a forma COZIDA (o índice não é o do vértice autorado); o `VecVertex` não tem
  id estável até à W4. **Ponto de simulação:** sem id estável por ponto confirmado (`ph2d-eval-motion` `MotionCookPump`).

## 2. O desenho (a fazer — ordem de trabalho)

| Peça | Forma | Lei / porta | Gate red-first (fixture) |
|---|---|---|---|
| **Pivot** | `Kind::Pivot { offset: [f64; 2] }` (alvo opcional: sem alvo, pivô = dono + offset) | pura, `ph2d-constraints` | `pivot_turns_the_owner_around_the_target` (`pivot.*`) |
| **Floor** | `Kind::Floor { eixo: ±Y/±X, desvio, usa_rotacao }` | pura: plano = alvo + desvio; projecção pela normal | `floor_keeps_the_owner_on_its_side` (`floor.*`) |
| **Transformation** | `Kind::Transformation { de, de_faixa, para, para_faixa, extrapola, mix, space }` — UM canal → UM canal | pura; rotação em radianos no documento, graus no cartão | `transformation_maps_one_channel_range_to_another` (`transformation.*`, + a recusa do Replace) |
| **Multi-parent** | nenhum tipo: Child Of + Child Of à força | `blend` | `multi_parent_is_two_child_of_items` (`armature.*` como recusa) |
| **Spline IK** | `Kind::SplineIk { chain, ajuste: Nenhum/Esticar }`, alvo = forma (a poligonal do `Caminhos`, como o Follow Path) | corrente como a IK (`Inerte::Corrente`, `ordem()`): junta seguinte = 1.º ponto da poligonal à CORDA = comprimento; além do fim, a tangente final; Esticar = fracção do arco + `sx = corda/repouso` | `spline_ik_lays_the_chain_on_the_curve` (`spline_ik.*`) |
| **Action** | `Kind::Action { canal, faixa, clip, de_t, ate_t }` | a amostragem do clip é da TIMELINE (`ph2d_timeline::pose_at`/`solo_source_value`), que depende da `constraints-ecs` (feature `skeleton`) ⇒ a fase recebe-a por um recurso-porta que a `ph2d-app-constraints` instala | `action_picks_the_clip_time_from_the_target_channel` (`action.*`) |
| **Spring** | `Kind::Spring { canais, duracao, ressalto }` (os knobs do nó `motion.spring`, `physics_of(Time, …)`) | `ph2d_node_motion_spring::law::passo` por canal, `dt = 1/fps`; estado por `(entidade, item)`; checkpoints a cada `STRIDE = 10` quadros (o da física, `ph2d-physics/src/world/checkpoint.rs:195`); salto de tempo ⇒ volta ao checkpoint ≤ f e re-simula com a ENTRADA de cada quadro, reposta pela «cena no quadro» (o padrão `SceneAtTick` da física: `apply_from_doc` em `t_k`, `ph2d-app-physics/src/bake.rs:148`) | `a_spring_rewinds_to_the_same_bit` (scrub 0→40→12→40 = play) |
| **Prender a ponto** | Copy Location com sub-alvo = vértice AUTORADO de uma forma (índice agora; o id estável da W4 migra-o) | o afim da forma no ponto | `a_point_of_a_shape_is_a_target` (`vertice`) |

- `quadro` (a porta da família) passa a receber o **tempo** (quadro, fps, a tocar ou não) e a «cena no quadro» — hoje recebe
  `(sim, cena, preview)` e não sabe o tempo (`ph2d-app-constraints/src/lib.rs:34`).
- Cartão: `MAX_NUMEROS = 2` e `MAX_ESCOLHAS = 2` (`ph2d-editor-core/src/constraint_edits.rs:30`) não cabem o
  Transformation (4 números, 3 escolhas) — sobem por append (ids novos anotados para a integração); canais
  `canal_constraint` em append (gate `o_vocabulario_das_constraints_nao_renumera`).
- Cena 5 (plano §6): antena com Spring sobre um corpo animado; sonda headless play × scrub = 0, foto antes da mensagem.

## 3. Feito (W3.2) — o que o código decidiu e porquê

| Peça | Onde | Gate (nome) | Decisão medida / desvio do §2 |
|---|---|---|---|
| Pivot · Floor · Transformation | `ph2d-constraints/src/poderosas.rs` | `pivot_turns_the_owner_around_the_target` · `floor_keeps_the_owner_on_its_side` · `transformation_maps_one_channel_range_to_another` · `transformation_replace_leaves_the_unmapped_channels_alone` (`tests/it/oraculo_w3.rs`, vistos 5/5 vermelhos com o tipo a devolver o dono) | o Pivot gira à volta do alvo pela rotação MUNDIAL do dono; sem alvo, o pivô é o dono + desvio (mostra o «Pick» mas não o exige) |
| Multi-parent | composição | `multi_parent_is_two_child_of_items` (verde de saída: é composição — a prova é a mutação) | nenhum tipo novo, como decidido |
| Spline IK | lei `lay_on_curve` (pura) + fase `ph2d-constraints-ecs/src/curva.rs` | `spline_ik_lays_the_chain_on_the_curve` (puro: recta · curta · meio círculo Bézier, os 3 modos; ECS: a corrente `curta` sob um afim da forma, e tirar a pilha devolve o autorado) | a raiz MOVE-SE ao início da curva (como no Blender); o esticar de cada osso divide pelo herdado do pai (`scale.x` herda-se na casa TRS) |
| Copy Point | `Kind::PontoDeForma { indice }` + recurso `Pontos` (os `verts[i].anchor` AUTORADOS, cheios pela família) | `a_point_of_a_shape_is_a_target` (puro + ECS; índice inexistente ⇒ `SemAlvo`) | ⚠️ tipo próprio, NÃO um sub-alvo no `Item`: o componente grava-se em postcard (sem nomes de campo) — um campo novo no `Item` partia os ficheiros da linha; uma variante em append não |
| Action | `Kind::Action { canal, faixa, clip, de_t, ate_t }`; o tempo é `ph2d_skeleton::action_time` (a lei do Smart Bone: UMA lei); a amostra é o recurso-porta `Accoes` (`Arc<dyn Fn(t) → canais locais>`), cheio por `ph2d-app-constraints/src/relogio.rs` com as `Track` CLONADAS do clip | `action_picks_the_clip_time_from_the_target_channel` (ECS, os 5 casos `action.*`; sem clip ⇒ `Estado::SemClip`) | o Smart Bone CONTINUA (toca o clip na CENA, escrevendo); o Action toca-o no DONO, sem escrever — perguntas diferentes, lei do tempo partilhada. `clip` = a POSIÇÃO na animação do dono (`clips_of`): o clip não tem id estável (o Smart Bone guarda o NOME) — ⚠️ apagar um clip anterior desloca-o |
| Spring | `Kind::Spring { canais, duracao, ressalto }`; `Simulador` em `ph2d-constraints-ecs/src/relogio.rs` (`law::passo` do `motion.spring`, `STRIDE = 10`, cada checkpoint guarda a ENTRADA e cai se ela mudou) | `a_spring_rewinds_to_the_same_bit` (0→40→12→40→7→8→9→33 = tocar, ao bit; editar a animação invalida) · `a_spring_adds_its_lag_at_its_place_in_the_stack` · `a_cena_5_toca_e_arrasta_igual` (a sonda da cena 5) | ⭐ a mola persegue a pose ANIMADA (`world_pose_at` — a «cena no quadro», como o `SceneAtTick` da física põe a timeline) e devolve o ATRASO, que a fase soma na posição do item. Assim tocar e arrastar leem a MESMA entrada. A entrada lê-se com a cena AUTORADA (`com_autorado`): sem isso a mola lia o que ela própria escreveu no quadro passado |
| O tempo no quadro | `quadro(sim, cena, preview, Option<Relogio>)`; `Relogio::da_vista` escolhe o transporte da vista (`ViewClock`) | — | a shell passa os três transportes; a abertura da cena de smoke mudou-se para a família (`smoke::abrir_escolhido`) — shell −2 linhas |
| Cartão | `TIPOS` 11→18; `MAX_NUMEROS`/`MAX_ESCOLHAS`/`MAX_OPCOES` → 5 (Action: 5 números; Transformation: 5 escolhas; um canal tem 5 opções); canais 19–37 em append; `Unidade::Segundos` | os gates do cartão (verificador) | ids novos para a integração: `insp_constraint_tipo_11..17`, `numero_2..4_*`, `opcao_*_4_*` e `opcao_2..4_*` |
| Cena 5 | `smoke.rs` `antena` + `smoke_antena.rs` | `a_cena_5_toca_e_arrasta_igual` · `a_ponta_anda_a_cada_imagem_do_ecra` · `a_haste_chega_a_ponta` | Corpo (curva da timeline, cúbica entrada-saída) · Ponta (Spring + Limit Distance Surface à Base: a antena DOBRA, não estica) · Haste (Look At à ponta), em laço de 3 s |

**Smoke do dono 25/10 — «o movimento da ponta não é fluido»** (medido antes de mexer): o ecrã desenha a 60 Hz e a curva
lê o instante CONTÍNUO, mas a mola andava por QUADRO (24 q/s) — o atraso ficava parado em **56 de 120** imagens e saltava
no quadro seguinte (gate `a_ponta_anda_a_cada_imagem_do_ecra`, visto vermelho). Cura na lei: `Simulador::atraso_em(q)` mistura
os quadros `⌊q⌋` e `⌊q⌋+1` (o `previo` guarda a outra ponta; continua função de `q` só — o gate compara instantes do meio
por ordem salteada, ao bit). Na cena: o corpo mudava de sentido de uma vez (chaves lineares) e a haste esticava como
borracha ⇒ chaves cúbicas e comprimento fixo. A haste escura (0,2) era invisível no fundo do palco: amarela, 0,12 m.

**Medido antes de abrir (o que ficou fora):** Flip «troca desenho por ângulo» — o desenho é uma escolha discreta
por *hold* (`FlipLayer::drawing_at`), sem canal animável: um Action sobre um Flip não tem o que tocar (fica para quando
o Flip tiver o canal). Ponto de simulação — o `MotionCookPump` não tem id estável por ponto (só a ordem dos sinks):
prender a um ponto de simulação fica para quando houver o id.

**Aberto, por medir:** a entrada da mola é o `world_pose_at` (o clip ACTIVO): sob a pilha do Arrange ou o Autoplay de
um dono, a pose que o app mostra sai do `apply_from_doc` e pode diferir. A mola não reage a arrastar o corpo com o gizmo
parado (a entrada é a animação).

## 4. Armadilhas

- ⛔ **A prova de mutação da W2 deixou a `ph2d-skeleton-live` compilada com a mira MUTADA em `target/debug`**, e o cargo
  achou-a fresca (o `mtime` do ficheiro restaurado não a invalidou): os testes mediram a lei mutada e o compilador avisou
  `unused variable: a, b` em `bone.rs:118`. Cura: `cargo clean -p ph2d-skeleton-live`. Depois de uma ronda de mutação,
  um aviso de variável não usada numa lei é o sinal.
