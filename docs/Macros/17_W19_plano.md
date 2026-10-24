# W19 — a família «mundo»: tudo pode ser macro (PLANO, antes do código)

> **Estado: FEITO — [handoff W19](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-24_W19.md).**

> Pedido: [PROMPT_W19](handoffs/PROMPT_W19_familia_mundo_2026-10-24.md). Base: `b48f8b9512` (main de 24/10) + W17/W18
> por integrar (HEAD `f54401e632`). Mapas lidos nesta sessão (3 exploradores, 24/10): o `WorldSnapshot`/undo, a
> física (ponte, toggle, bake), a sessão/portas/tocadores das macros e o `ph2d-preview-drive`.
> Lei de cima: [ADR-0190](../architecture/decisions/0190-a-macro-e-um-objeto-e-leva-tudo-o-que-usa.md) (o objecto
> Macro), [ADR-0186](../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md) (o
> formato), [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (física).

## 0. O pedido do dono (23/10, decisão 7, verbatim)

«a macro não será só imagem. Tudo na game engine pode criar macros. Por exemplo: posso começar com uma pintura de
um player e depois acrescentar um collider e depois simular a física. tudo pode ser macro e a macro pode conter
qualquer coisa. Logo a macro é um objeto completo e independente»

### 0.1 O que o código diz HOJE (conferido, não lido do plano W18)

| Facto | Onde | Consequência |
|---|---|---|
| A porta `mundo.*` **não existe** (o plano W18 §2.3 prometia `Door { "mundo.*", Refused { why: "W19" } }`) | `git grep '"mundo'` vazio; portas só do Painter em `ph2d-tool-painter/src/macro_track/portas.rs` | hoje pôr um collider a gravar é **omitido em silêncio** — a dor do Illustrator que o ADR-0186 proíbe |
| `aplicar::familia` trata qualquer família que não seja Painter como Motion | `ph2d-app-macros/src/aplicar.rs:63` | uma trilha desconhecida num ficheiro é aplicada ao Motion **em silêncio** — defeito, entra nesta onda |
| `one_per_family` nasce do campo único `painter: Option<Aberta>` / `motion: Option<Aberta>` | `sessao.rs:49-50`, recusa em `sessao.rs:283-289`; Painter `trilha_do_painter` = 1.ª da família (`ph2d-app-painter/src/macro_track/mod.rs:213`); Motion `truncate(1)` (`em_cena.rs:39`) | três sítios a mudar juntos |
| O mundo do undo é **uma linha por objecto**, chaveada pelo `StableId`, com `parent: Option<StableId>` e blobs postcard por `type_id` (blake3 do nome) | `ph2d-ecs/src/scene/save.rs:62-93` | a matéria da família mundo já existe; falta só a **diferença de linhas** como porta (hoje o diff é o `PartialEq` do `ProjectState`) |
| O `StableId` é um contador **por projecto** (sem UUID) | `stable_id.rs:86,112` | ids da macro têm de ser **locais** e remapeados ao aplicar |
| Copiar bytes de componente desconhecido + remapear referências entre objectos **já existe** | `ph2d_ecs::deep_copy_subtree` (`instantiate.rs:231`) + censo `RefKind::Object` → remapeador (`ph2d-app-components/src/instance_refs.rs`) | o aplicar da família mundo **reusa** esta porta — não escreve uma segunda |
| A física é runtime-truth: o `Transform` simulado é **pré-visualização** (`Driver::SolverPose`), o documento guarda a pose autorada | `ph2d-app-physics/src/bridge/dispatch.rs:60-115`, `ph2d-preview-drive/src/lib.rs:1-60` | «Simular» no editor **não gera passo de undo** — tem de se gravar por outra porta (§2.3) |
| A ponte pergunta à cena onde estão os corpos conduzidos por ela, tique a tique | `ph2d-physics-ecs/src/bridge/kinematic.rs:39` (`SceneAtTick`) | é por aqui que a reprodução da macro se declara à física global, sem dois escritores |
| O preview conduz factos de objectos que EXISTEM (`Visible(bool)`, poses…); não cria objectos | `ph2d-preview-drive/src/lib.rs:76-230` | um objecto que nasce e morre dentro da gravação não tem onde se mostrar (§2.4, recusa nomeada) |

## 1. Estado da arte (de conhecimento — nenhum destes instalado como oráculo)

| App | O que faz | O que foi TENTADO e abandonado / a dor | Lição para nós |
|---|---|---|---|
| Unreal *Transactions* | o undo regista a diferença das propriedades serializadas de cada `UObject` tocado | — | gravar o mundo = gravar o MESMO diff que o undo já faz, não um segundo modelo |
| Unity *Prefab overrides* / `Undo.RecordObject` | um override é uma lista de `PropertyModification` contra a base | os overrides «por campo» partem quando o tipo muda; Unity acabou a guardar componentes inteiros adicionados/removidos | blob inteiro por componente (é a nossa vtable: o `ph2d-ecs` já mediu que `(type, field)` não existe para 91 tipos) |
| Unreal *Simulate* + «Keep Simulation Changes» (K) | simular no editor não altera o nível; só o K copia as poses de volta | sem K tudo volta — o artista perde a pose que viu | a pose simulada é PREVIEW (a nossa lei); gravar a simulação como **passo que se volta a correr**, não as poses |
| Blender *Rigid Body cache* / «Bake to Keyframes» | cache por quadro; assar converte em chaves | o cache invalida-se ao mexer em qualquer coisa; assar perde a física (vira animação morta) | nada de poses por quadro na macro (é o pedido); cache só como optimização da reprodução, nunca como verdade |
| Houdini *DOP network* | a simulação é um NÓ da receita procedural, recozido de forma determinística ao reler | o *file cache* existe porque recozer longo é caro | o desenho escolhido: «Simular» é um passo da receita que recoze o rapier; medir o custo do recozer no scrub (§3) |
| Photoshop *Actions* | passo que não grava = «Insert Menu Item» / «Insert Stop» explícito | Illustrator **omite** ferramentas não graváveis em silêncio | cada componente declara-se (gravável / de outra família / recusado) e o painel diz o que não ficou |
| Godot *PackedScene* / Unity *Prefab* | objecto completo e independente, instanciado com ids novos | referências internas por id partem se não forem remapeadas | ids locais à macro + o remapeador `RefKind::Object` que a casa já tem |

**O que nos separa:** o objecto Macro é um prefab **com tempo** — a sua construção inteira (pintar, pôr um
collider, simular) toca de novo onde for aplicado.

## 2. O desenho — uma porta por pergunta

### 2.1 «O que muda entre dois passos?» — `ph2d_ecs::scene::linhas_que_mudam(antes, depois)` (NOVA, fundacional)
- Função pura sobre dois `WorldSnapshot`: devolve, por `StableId`, `Nasce(linha)` / `Morre` / `Muda { pai:
  Option<Option<StableId>>, componentes: Vec<(type_id, Option<bytes>)> }` (só os blobs que diferem; `None` = tirado).
  As linhas já vêm ordenadas por `StableId` ⇒ junção linear, `O(linhas)`, e os `Arc` iguais saltam-se por ponteiro
  (o caso «nada mudou nesta linha» não compara bytes).
- ⛔ Recusa: comparar o `ProjectState` inteiro (o `PartialEq` de hoje) — diz «mudou», não «o quê».

### 2.2 «Quem grava o mundo?» — a família `"mundo"` na sessão (`ph2d-app-macros/src/mundo/`)
- **A porta é o passo do undo.** Quando a shell empurra um passo (`push_linked`, `undo.rs:385`) e há sessão armada,
  chama `ph2d_app_macros::com(|m| m.passo_do_mundo(&pre.world, &agora.world, registo))` — **uma linha na shell**,
  o resto na família. É o MESMO grão que o Ctrl+Z do artista: um passo da macro = um passo que ele desfaria.
- Filtra a diferença à **sub-árvore do objecto Macro** (o objecto e os descendentes, no `antes` ∪ `depois`, pelo
  `parent` das linhas) e, por componente, pergunta à **tabela de portas do mundo** (§2.5):
  `Gravado` → entra; `DeOutraFamilia(painter|motion)` → não entra (a trilha dessa família já o tem — ex.: o
  `SpritePixels` de uma imagem que o Painter grava, o grafo do Motion, a própria `Macro`, o `MacroTempo`);
  `Recusado { porquê }` → não entra e **a sessão diz qual** (`panel.macros.refusal.world_component`, com o nome).
- **Ids locais:** `0` = o objecto Macro; os outros numeram-se pela 1.ª aparição na gravação. O `pai` e as
  referências `RefKind::Object` dentro dos blobs gravam-se locais (o remapeador do censo, aplicado ao gravar).
- **Recursos embutidos:** um `SpritePixels(AssetId)` gravado (imagem importada, não pintada pela macro) leva os
  bytes do asset para a tabela `recursos` da trilha, uma vez por `AssetId`. Sem bytes no `asset_db` ⇒ recusa nomeada.
- **Estado inicial** = as linhas da sub-árvore no instante do Gravar (ids locais), pelo mesmo filtro.

### 2.3 «Como se grava "Simular"?» — o passo `Simula`, lido da ponte
- Simular no editor é Physics armado + relógio a correr; o solver escreve pré-visualização e **não há passo de
  undo** (§0.1). Logo a porta é outra: a cada quadro, a sessão lê da ponte o intervalo de tiques que o solver correu
  (`last_stepped` antes/depois, e `simulate && playing`) e, se a sub-árvore tem um `RigidBody`, abre ou estende um
  evento `Simula { tiques: u32 }` no relógio da macro. Parar o relógio fecha-o.
- O evento leva a **pose que o artista viu no fim** de cada corpo (ids locais) como valor de CONFERÊNCIA, não como
  verdade: a reprodução recozida tem de dar o mesmo bit (determinismo, ADR-0131 D7); se não der, a macro **diz**
  que a física deste projecto deu outro resultado (a cena de destino tem outros obstáculos) — nunca corrige.
- ⛔ Recusa: gravar poses por quadro (o pedido; e seria «assar» — ADR-0131 D11 é opt-in do artista).
- **A entrada do jogador vai junto** (decisão 10, 24/10): o troço da fita (`tape`) dos tiques do `Simula`, para os
  corpos da sub-árvore, entra no evento; a reprodução dirige o rapier privado com ela (o mesmo laço de replay da
  ponte). A macro é independente da fita do projecto.
- **Obstáculos** (decisão 9, 24/10): o rapier privado recebe os corpos `Static` da cena de DESTINO (e os da macro).

### 2.4 «Como toca?» — escrita conduzida pela porta do preview (escolha a MEDIR, §3)
- O documento tem o **FIM** (como no Painter e no Motion desde a W18): aplicar «Novo» faz nascer todos os objectos
  da macro pela porta da cópia profunda (`deep_copy_subtree` sobre linhas locais → `StableId` novos → remapeador
  `RefKind::Object`), num passo de undo só.
- Enquanto o clipe toca (`MacroTempo` no objecto Macro, a mesma porta das outras famílias), o palco do mundo calcula
  `S(t)` = inicial + passos até `t`, e **conduz** o mundo pelo `PreviewDrive` (driver novo `MacroMundo`):
  - objecto que ainda não nasceu em `t` → `Visible(false)`;
  - componente que em `t` tem outro valor, ou ainda não foi posto → o blob de `S(t)` (ou ausente), escrito pelo
    registo; a captura repõe o autorado ⇒ **nada no undo nem no save enquanto toca** (lei do preview).
  - dentro de um `Simula`: o palco recoze um **rapier privado** (`PhysicsBridge::new` sobre um `SimWorld` com a
    sub-árvore em `S(início)` e os corpos `Static` da cena como obstáculos), passo fixo `1/60`, tiques = `t − início`;
    as poses escrevem-se como `MacroMundo` (pré-visualização). Depois do `Simula`, a pose fica a do último tique.
  - **A física global não escreve por cima:** enquanto o clipe toca, os corpos da macro respondem ao `SceneAtTick`
    da ponte (a pose da macro naquele tique) — são cena, não solver. Um escritor por facto.
- ⛔ Recusa: **mundo-palco renderizado** (um 2.º `SimWorld` desenhado no lugar da sub-árvore) — cria uma segunda fonte
  do que se pinta, que o `ph2d-preview-drive` recusa por lei; e o extract, o picking e a Luz 2D teriam de aprender
  outro mundo. (A medir mesmo assim, §3, para a recusa ter número.)
- ⚠️ Limite por desenho: um objecto que **nasce e é apagado** dentro da mesma gravação não existe no FIM ⇒ não tem
  onde se mostrar. Recusa nomeada ao Parar (`…refusal.born_and_deleted`), com o nome do objecto — nunca omissão.

### 2.5 «O que é gravável?» — a tabela de portas do mundo (`mundo::PORTAS`), com censo
- Uma entrada por **componente registado** (`ComponentRegistry::iter`): `Gravado` · `DeOutraFamilia(..)` ·
  `Recusado { why }`, e `como_portas_do_nucleo()` → `Door { "mundo.<Nome>", Recordability }` (o tipo do núcleo).
- Gate `todo_componente_registado_tem_porta_no_mundo`: varre o registo vivo (não o fonte) — um componente novo de
  outra linha sem decisão **reprova** aqui (molde de `toda_porta_do_painter_tem_decisao`).

### 2.6 «Várias imagens e vários Motion no mesmo objecto Macro» — tirar o `one_per_family`
- `Sessao`: `painter`/`motion: Option<Aberta>` → `Vec<Aberta>` chaveado pelo `alvo`; `segue_o_alvo` abre uma trilha
  nova em vez de recusar. Painter: `PalcosDoPainter` passa a palco por `(dono, índice da trilha)`; Motion: sai o
  `truncate(1)`. A chave `MACRO_RECUSA_UM_DE_CADA` sai (com a frase).

### 2.7 «Que bytes saem?» — formato
- Família `"mundo"`, `exige "mundo"`, **MINOR 4** só quando usada (um leitor 1.3 recusa em voz alta; gravações sem
  mundo continuam ao byte). Eventos: `Passo { linhas }` e `Simula { tiques, conferencia }`; `initial` = linhas
  locais + `recursos`. Tudo postcard dentro do mesmo codec etiquetado.
- `aplicar::familia` passa a `match` exaustivo; família desconhecida = `ErroAoAplicar::FamiliaDesconhecida(nome)`.
- `PROJECT_SCHEMA`: **0 degraus** (a macro viaja no blob do componente `Macro`, como na W18).

## 3. As medições (UMA rodada intercalada, antes de escolher — MEDIR_VELOCIDADE.md)

Sonda `mede_o_palco_do_mundo` (`--profile smoke`, as variantes no MESMO processo, intercaladas, resumo = MÍNIMO):

| Variante | O que mede | N |
|---|---|---|
| A — conduzida | escrever `S(t)` por quadro no mundo + `substitute`/`restore` da captura | 1, 10, 100 objectos |
| B — palco renderizado | `snapshot_to_world` da sub-árvore num 2.º `SimWorld` + extract dele | 1, 10, 100 |
| C — recozer `Simula` do zero (scrub para trás) | `PhysicsBridge::new` + k tiques | k = 60, 600, 3 600 · 1/10/100 corpos |
| C' — recozer com pontos de retoma a cada 60 tiques | o mesmo | idem |

**Kill-criterion (escrito antes):** se A > 1 ms/quadro a 100 objectos, ou C' > 4 ms num scrub a 10 s com 100 corpos,
a reprodução não é esta forma — volta ao plano. A tabela com os números entra aqui antes do código do palco.

**Medido (24/10).** 1.ª rodada (carga 150–245, tecto da linha): C a 600 tiques com 100 corpos = **58,9 ms** ⇒
REPROVOU o critério (o palco recozia do zero a cada scrub para trás). 2.ª rodada, com C' (`loadavg` 4,45 → 6,23;
`--profile smoke`; 5 blocos rodados; mínimo; o «pior scrub» cai em `ponto + passo − 1`, 8 scrubs descendentes
depois de tocar até 3 600):

| Variante | 1 | 10 | 100 corpos/objectos |
|---|---|---|---|
| A — conduzir `S(t)` parado | 0,002 ms | 0,012 ms | **0,110 ms** |
| A — conduzir com troca (põe/tira a cada quadro) | 0,003 | 0,014 | 0,126 |
| A — quadro a andar 1 tique (ponte guardada) | 0,005 | 0,025 | 0,227 |
| B — palco renderizado | — | — | **recusada por LEI**, não por número: segunda fonte do que se pinta (§2.4 e ⛔ abaixo) |
| C — do zero, k = 60 / 600 / 3 600 | 0,48 / 2,45 / 7,91 | 0,74 / 5,70 / 19,9 | 3,64 / **40,2** / 157,6 ms |
| C' — pontos a cada **60**, pior scrub perto de 0 / 600 / 3 000 | 0,073 / 0,073 / 0,073 | 0,195 / 0,195 / 0,193 | **1,75 / 1,76 / 1,74 ms** |
| C' — pontos a cada 10, idem | 0,053 / 0,016 / 0,016 | 0,107 / 0,050 / 0,050 | 0,60 / 0,45 / 0,46 ms |

Memória dos pontos de uma corrida de 3 600 tiques com 100 corpos: passo 60 = 60 pontos, **8,0 MB**; passo 10 = 360
pontos, 48,2 MB (1,34 kB por corpo e ponto). **Escolha: C' a cada 60** (`palco_fisica::RETOMA`), 1,75 ms ≤ 4 ms no
pior scrub, a 1/6 da RAM do passo 10 — que se aproxima do tecto `TECTO_DOS_PONTOS` (64 MiB) já a 100 corpos × 60 s,
e acima dele o anel larga os pontos mais antigos e esse scrub volta a replayar do repouso (157 ms). A porta é do
anel da ponte (`PhysicsBridge::set_checkpoint_policy`, append-only); prova de bit:
`retomar_de_um_ponto_da_o_mesmo_bit_que_do_zero` e `o_scrub_para_tras_recoze_e_da_o_mesmo`.

## 4. Onde encosta em contrato (§6) e schema

- `git grep 'ph2d_macro\|ph2d_physics_ecs\|preview_drive'` em `crates/ph2d-nodegraph`, `crates/ph2d-tool-registry`,
  `crates/ph2d-vector` ⇒ **0 ficheiros** (24/10). Nenhum contrato §6 tocado.
- Fundacional tocado (Modo L, append-only): `ph2d-ecs` (`linhas_que_mudam`), `ph2d-macro` (MINOR 4, `exige
  "mundo"`), `ph2d-preview-drive` (`Driver::MacroMundo`, variante no fim), `ph2d-physics-ecs` (nada — o
  `SceneAtTick` já existe; quem o implementa é a composição). ⇒ **suíte inteira na integração** (já era, W18).
- `PROJECT_SCHEMA` 0 degraus (`python3 scripts/schema-recount.py` no fecho).

## 5. A UI — as 4 condições, uma a uma

| Condição | O que existe / o que muda |
|---|---|
| O componente EXISTE | painel MACROS e linha «Macro steps» já existem; a família mundo aparece como trilha «World» na lista de famílias do objecto (`MacroDaBibliotecaView` já lista famílias por nome) |
| Pintado e registado | as etapas do mundo entram pela porta `EtapasDaFamilia` (impl `EtapasDoMundo`): uma etapa por passo, com o rótulo do que mudou («Collider added · Player», «Simulate · 2.0 s»), chaves novas em `ph2d-i18n/src/macros_mundo.rs` |
| O clique chega ao barramento | nenhum botão novo: gravar é o Gravar/Parar de hoje; pôr o collider é o «Add Physics Body» do Inspector; simular é Physics + Play de hoje — a macro OUVE os gestos que já existem |
| A SEQUÊNCIA leva a algum lugar | cena `=8` (§7): Add ▸ Macro → Gravar → imagem → pintar → Add Physics Body → Physics + Play → Parar → Guardar → apagar → abrir projecto vazio com chão → Aplicar ▸ Criar novo → tudo nasce e cai |

O aviso do que não ficou gravado: `MacroPanelInfo.recusa` já junta as recusas da sessão — as do mundo entram aí.

## 6. Gates, red-first, e a fixture

**Fixture `mundo_com_chao`:** um objecto Macro, uma sprite filha 64×64 (pintada), um chão `Static` FORA do objecto
Macro (a cena), `ppm` 100. Passos: pôr `RigidBody`+`Collider` (o «Add Physics Body» real), simular 120 tiques.

| # | Gate | Vermelho quando |
|---|---|---|
| 1 | `linhas_que_mudam_da_so_o_que_mudou` (ph2d-ecs) | pôr 1 componente em 1 de 1 000 linhas devolve ≠ 1 `Muda` com 1 blob |
| 2 | `o_collider_posto_a_gravar_entra_na_macro` | o passo do undo dentro do objecto Macro não vira `Passo` |
| 3 | `o_que_o_painter_grava_nao_se_grava_duas_vezes` | o `SpritePixels` da imagem pintada aparece na trilha mundo |
| 4 | `todo_componente_registado_tem_porta_no_mundo` | um registo sem decisão passa |
| 5 | `um_componente_recusado_diz_se_pelo_nome` | a recusa não chega ao `MacroPanelInfo.recusa` |
| 6 | `simular_grava_um_passo_e_nenhuma_pose` | o evento leva poses por tique, ou não nasce |
| 7 | `recozer_o_simula_da_o_bit_que_o_artista_viu` | a pose recozida ≠ a de conferência (mesmo projecto) |
| 8 | `aplicar_novo_faz_nascer_com_ids_novos_e_juntas_remapeadas` | um `PhysicsJoint` aplicado aponta para o original |
| 9 | `a_tocar_nada_entra_no_undo_nem_no_save` | `ProjectState::capture` durante a reprodução ≠ o FIM |
| 10 | `antes_do_collider_o_corpo_nao_cai` | em `t` < passo do collider, a física (privada ou global) move o corpo |
| 11 | `a_fisica_global_nao_escreve_por_cima_da_macro` | com Physics armado, a pose de um corpo da macro difere da do palco |
| 12 | `duas_imagens_no_mesmo_objeto_macro_tocam_as_duas` | a 2.ª trilha do Painter não toca |
| 13 | `um_leitor_1_3_recusa_o_mundo_em_voz_alta` | `CodecError::Exige` não nasce |
| 14 | `familia_desconhecida_nao_se_aplica_como_motion` | o `aplicar` aceita `"xpto"` |
| 15 | `nasce_e_apagado_na_gravacao_diz_se` | o Parar não nomeia o objecto |
| 16 | `a_cena_8_percorre_o_caminho_inteiro` (sonda) | os números do §7 mudam |

Provas de mutação (no fecho): o filtro da sub-árvore, a tabela de portas, o `SceneAtTick` da macro, o remapeador.

## 7. A cena de smoke `PH2D_MACRO_SMOKE=8`

O exemplo do dono, sozinha, cada passo diz no ecrã o que é: **+ ▸ Macro** → **Gravar** → imagem nova 128×128 dentro
dele → pintar um boneco (três pinceladas) → **Add Physics Body** no Inspector (collider à medida da imagem) →
**Physics** + **Play** 2 s (cai sobre um chão que a cena tem) → **Parar** → **Guardar na biblioteca** → **apagar** →
**projecto novo** com um chão noutra altura → **Aplicar ▸ Criar novo**: a imagem nasce, pinta-se, ganha o collider e
cai — sobre o chão DESTE projecto. `CENAS_DO_SMOKE` 7 → 8.
Os números (altura onde assenta, tiques, texels) saem da sonda `a_cena_8_percorre_o_caminho_inteiro` **antes** de a
mensagem ao dono ser escrita, e a cena é fotografada (`import -window`, nunca `spectacle`) antes de ir a ele.

**Medido (24/10, sonda `a_cena_8_percorre_o_caminho_inteiro`, banca sem placa, `ppm` 100, chão 900×24 px):**

| Número | Valor |
|---|---|
| trilha mundo | 2 eventos: o passo do collider em `9,48 s`, UM `Simula` em `13,00 s`, tiques `0..100` (a sonda corre 2 tiques em cada 3 quadros de render); o passo que o Pause deixa sai inteiro |
| 1.º chão (centro `−2,50 m`, topo `−2,38 m`) | a gravação assenta em `y = −1,7400 m` (topo + meio lado `0,64`) |
| 2.º chão (centro `−4,00 m`, topo `−3,88 m`) | a reprodução do aplicado assenta em `y = −3,2400 m` |
| texels pintados no aplicado | `4 871` de `16 384` (128²) |
| o palco na cena da gravação × a conferência | o bit (`[1,6432526e−4, −1,7400224, 1,4798611e−7]`), com e sem retoma — na 1.ª corrida (antes da cura do palco de 24/10) `x`/rotação vinham com o SINAL trocado |

**No app (fotos `handoffs/smoke_2026-10-24_W19/`, depois das curas de 24/10):** os 11 passos ensinam o que
acontece — o painel lista 6 etapas (3 pinceladas, «Canvas appears», «RigidBody, Collider added», «Simulate · 2.48 s»),
sem recusas nem «Transform changed»; no projecto limpo o boneco nasce, pinta-se e assenta no chão NOVO (`−400 px`).
A 1.ª corrida da cena (antes das curas) achou quatro defeitos, curados com gates: (a) o `Simula` partia-se num quadro
de render sem tique — `tres_quadros_de_render_por_tique_dao_um_simula`; (b) o Pause deixava a pose do solver como
autoria (a captura do undo chega a vir um tique atrasada: tira-se toda pose que a corrida PERCORREU) —
`o_pause_nao_grava_a_pose_do_solver_como_autoria`, e o FIM no destino recoze-se lá —
`aplicado_noutro_chao_o_fim_e_o_chao_novo` (`y = −3,2400 m`); (c) sair do Paint recusava a macro —
`sair_do_paint_e_voltar_a_gravar_nao_recusa`; (d) a conferência acendia noutro chão (`Simula.obstaculos`, blake3) —
`a_conferencia_so_se_faz_sobre_o_mesmo_chao`.

## 8. A ordem do trabalho

1. Medição §3 (sonda) → a tabela entra no §3 → decide A (esperado) ou volta ao plano.
2. `linhas_que_mudam` (#1) · formato MINOR 4 (#13) · `aplicar::familia` (#14).
3. Gravar: hook do undo, filtro, portas + censo, ids locais, recursos (#2–#5, #15).
4. `Simula` (#6, #7). 5. Aplicar novo/por cima do mundo (#8). 6. Palco conduzido + `SceneAtTick` (#9–#11).
7. `one_per_family` fora (#12). 8. Etapas/i18n/painel. 9. Cena `=8` + sonda + fotos. 10. Fecho (§1.5.9) e handoff.

## ⛔ Recusas MEDIDAS / decididas

| Recusa | Porquê |
|---|---|
| gravar poses por quadro da simulação | pedido do prompt; seria assar (ADR-0131 D11 é opt-in) |
| mundo-palco renderizado (2.º `SimWorld` desenhado) | segunda fonte do que se pinta (lei do `ph2d-preview-drive`); extract/picking/Luz teriam outro mundo — número no §3 |
| comparar o `ProjectState` inteiro para achar o passo | diz «mudou», não «o quê» |
| `field_id` por campo dentro do componente | não existe para os 91 tipos (medido em `ph2d-ecs/src/instantiate.rs`); blob inteiro |
| um 2.º remapeador de referências para a macro | o censo `RefKind::Object` já liga cada campo ao seu; duas portas divergem |
| reproduzir o `Simula` lendo a fita do PROJECTO | a macro aplicada noutro projecto não a tem — o troço vai dentro do evento (decisão 10) |
