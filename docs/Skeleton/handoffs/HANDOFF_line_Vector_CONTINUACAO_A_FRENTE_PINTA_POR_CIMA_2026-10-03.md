# HANDOFF (continuação, janela nova) — `line/Vector`: próximo = o BOTÃO «antes/depois dos ossos» por efeito (ordem do dono), depois A PARTE DA FRENTE PINTA POR CIMA (2026-10-03)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). A janela anterior
> (F50…F50-k) está na [fila §F50](../01_a_fila.md) e no
> [handoff de 02/10](HANDOFF_line_Vector_CONTINUACAO_FORMAS_COM_EFEITOS_2026-10-02.md).

> ⛔⛔⛔ **SUBSTITUÍDO** pelo [handoff de 04/10 — os abertos](HANDOFF_line_Vector_CONTINUACAO_OS_ABERTOS_2026-10-04.md).
>
> ⛔⛔ **ACTUALIZADO no fim de 2026-10-03:** o §2 (o botão) foi FEITO e RETIRADO na mesma tarde — o
> dono escolheu simplificar: **o Bind coze os efeitos e uma forma presa não recebe efeitos** (fila
> **§F51**). O §2-b (o peso viaja com o desenho) continua por perguntar ao dono depois do smoke da
> F51 — com os efeitos cozidos no Bind, a pergunta passa a ser sobre o desenho COZIDO. O §3 segue.

## 0. Onde está

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector`, base `main` `1ad60a1ce`, `~31` commits à frente, nenhum integrado |
| smoke | binário `--profile smoke` compilado na worktree |
| o dono | aprovou o smoke de cada passo da F50 (o último: *«smoke OK»* do *Hatch*) e mandou **não fechar** a linha e seguir pelos abertos |

## 1. O que a janela anterior entregou (F50, todas aprovadas em smoke)

Formas vectoriais presas COM EFEITO: o efeito coze-se no REPOUSO e dobra com a forma (F50); campo
do domínio sobre o contorno COZIDO (F50-d); o ajuste só aceita cúbicas que andam para a frente —
curou na origem o gancho da F43 (F50-e); a união sem lascas e só dos fechados (F50-f); a agulha do
*Bloat* (F50-g); as voltas apertadas viram nós (F50-h); o orçamento de amostras é o do contorno
(F50-i); o solver do campo numa thread (F50-j). Cena **`PH2D_VEC_BONE_SMOKE=5`** (seis barras).
Os abertos de 01/10 foram auditados contra o código: três eram notas envelhecidas (fila §F50).

## 2. ⭐⭐⭐⭐ O PRÓXIMO (ordem do dono, 2026-10-03): um BOTÃO por efeito — «Antes dos ossos | Depois dos ossos»

**A conversa que o pediu:** o dono perguntou *«os efeitos são aplicados ao desenho vetorial após
sua deformação causada pelo osso?»* — não: desde a F50 o efeito coze em REPOUSO e o desenho dobra
(«antes»). Explicadas as duas ordens (antes: o efeito gruda e dobra, o *Hatch* curva com a barra;
depois: o efeito é refeito sobre a forma já dobrada, um *Twist* fica sempre bem formado), a ordem
dele: *«vamos colocar um botão de seleção para a ordem em que o efeito entra, se antes ou
depois»*. É o modelo do Blender (a posição do modificador em relação ao `Armature`, por efeito).

**Mapa (conferido por explorador nesta janela, file:line da worktree):**
- **Dado:** `FxEntry { effect, enabled }` em `crates/ph2d-vec-scene/src/effect.rs:148`. Postcard
  POSICIONAL ⇒ campo novo no FIM, `VEC_SCENE_SCHEMA_VERSION` (`schema.rs:80`, hoje `22`) `+1` e
  `PROJECT_SCHEMA` (`shells/desktop/src/project_schema.rs:343`, hoje `178`) `+1` — conte o degrau com
  `python3 scripts/schema-recount.py` (CLAUDE.md §1). Precedente exacto: o próprio `enabled`
  (commit `c80ad95`, `VEC_SCENE 9→10`, `PROJECT_SCHEMA 19→20`) — leia o diff dele antes.
- **UI (copie a cadeia do «olho» `enabled`):** pintura+hit em `crates/ph2d-panel-vector/src/paint_effects.rs:203–245`
  (id `vector_fx_hide_id(row)` em `ph2d-editor-core/src/ids/chrome/vector.rs`) → classificação em
  `crates/ph2d-app-vec/src/fx_bridge_dispatch.rs:62` → aplicação `:109` → mutação
  `fx_bridge.rs:162` (`toggle_enabled`) → estado `ph2d-panel-vector/src/state_effects.rs:45`
  (`FxRowView`) → undo provado por `shells/desktop/src/fx_undo_smoke.rs:75`. Texto em
  `crates/ph2d-i18n/src/vector.rs` (`panel.vector.fx.*`). ⚠️ DIRETIVA §2: as 7 pontas no mesmo
  passo, com o teste de costura que DIRIGE o clique. O botão só faz sentido numa forma PRESA — se
  ela não está presa, ele não muda nada: mostre-o desligado com a razão, ou esconda-o (régua §5.0
  «controlo morto»).
- **Lei «depois»** (`crates/ph2d-skeleton-live/src/skin_desenho.rs`): a pilha parte-se em duas pela
  ordem de cada entrada — as «antes» cozem em repouso como hoje (`geometria_cozida`,
  `cozido_com_efeitos`, campo do cozido); o bake dobra; as «depois» correm sobre o DESENHADO com o
  `FxCtx` do REPOUSO (um `run_stack` com contexto dado — hoje `effect::run_stack` calcula-o da
  entrada, `effect.rs:469`) para o tamanho do efeito não depender da pose. ⚠️ Um *Falloff* modula o
  efeito SEGUINTE — vai na fase dele. ⚠️ Com «depois», a união do contacto corre ANTES do efeito
  (sobre a forma dobrada) e o efeito depois — meça a ordem com fotos.
- **Cena:** a `=5` ganha o par lado a lado (o mesmo *Twist* forte antes e depois) — o número da
  cena conta-se no roteador `ph2d-app-vec/src/smoke_bone.rs` (`NIVEIS = 5`).

## 2-b. ⏸️ EM ESPERA: o peso VIAJA COM O DESENHO — cada pedaço segue o osso de onde VEIO

⚠️ O dono escolheu esta lei ANTES de saber que podia pôr o efeito «depois»; com o botão, o *Twist*
forte em «depois» fica sempre bem formado. **Pergunte-lhe de novo depois do smoke do botão** — esta
secção pode já não ser precisa.

**O report:** *«quanto mais veloz se arrasta o valor de twist mais deformações bizarras… com alguns
valores após parar fica bom, com outros fica ruim»*. O arrasto foi curado (F50-k: mostra-se sempre
o último par pilha+campo exacto). O resto é a LEI: o campo da F50-d é ESPACIAL (o domínio é o
contorno cozido, à *Puppet* do After Effects) e, com um *Twist* forte, as pontas enrolam-se perto do
osso do MEIO e passam a ser dele. Medido (nós do cozido dominados por osso, barra da fixtura):
*Twist* `60°` `[55, 22, 55]` · `120°` `[33, 66, 33]` · `150°` `[40, 53, 39]` — a forma muda de dono
conforme o valor. **O dono escolheu a lei MATERIAL** (pergunta directa, com a opção espacial ao
lado): *«O osso de onde veio»* — é a do Blender (o grupo de vértices é do PONTO; um modificador
antes do `Armature` move o ponto e o peso vai com ele; oráculo da F50 em
`docs/Skeleton/oraculo/oraculo_ordem_dos_efeitos.py`).

**Desenho proposto (NÃO medido):** para os efeitos que movem pontos (*Twist*, *Warp*, *Bloat*,
*Falloff*), DEFORMAR A MALHA DO CAMPO DA FONTE pelo próprio efeito — os vértices da malha passam
pelo efeito com o `FxCtx` da forma (como âncoras de alças recolhidas num contorno de marcadores;
os deformadores mapeiam âncoras ponto a ponto) e os PESOS ficam os mesmos. O campo resultante é
material por construção (sem solver: mais barato que o da F50-d) e a leitura baricêntrica dos nós
do cozido fica igual. ⚠️ A medir: dobras da malha onde o efeito sobrepõe a forma a si mesma
(*Twist* forte) — a leitura acha o primeiro triângulo; os geradores (*Hatch*, *Repeat*, *Zig Zag*,
*Sketch*, *Trim*, *Knot*) não são mapas de pontos — fique com o campo do cozido (F50-d) para eles,
e meça se a pilha mista (deformador + gerador) precisa das duas leis em sequência. Gate sugerido:
os nós dominados por osso no cozido = os da fonte, em toda a varredura do *Twist* (`0°`…`360°`).

## 3. ⭐ DEPOIS: numa dobra muito forte, a parte da FRENTE pinta por cima (como a imagem)

**O que o artista vê hoje** (FOTOGRAFADO a `100°`–`110°` na `=5`): o contorno já se une (o «V»
limpo), mas o que é ABERTO ou fica DENTRO — as riscas de um *Hatch*, os dentes de um *Zig Zag* —
da parte de TRÁS aparece por cima da parte da frente, onde os membros se sobrepõem. A imagem presa
resolve isto pela ORDEM DAS FACES: o osso mais adiante na corrente pinta por cima (F48-c,
`ordena_pelo_osso`, chave `Σwⱼ·j/Σwⱼ`).

**Desenho proposto (NÃO medido — meça antes):**
1. Partir a forma cozida em REPOUSO em pedaços por osso: cada triângulo da malha do campo
   (`CozidoFx.campo`) vai para o osso da chave dele; a região do pedaço é a forma ∩ a união dos
   triângulos (booleana, uma vez por pilha — cabe na gaveta, como o campo).
2. O desenho de uma forma passa a ser uma LISTA (`LiveGeometry` já leva `Vec<VecPath>` por id):
   por chave crescente, o preenchimento do pedaço (sem traço), o traço do contorno ORIGINAL que cai
   nele (aberto) e as riscas recortadas nele.
3. ⚠️ Os riscos medidos noutras mídias: a COSTURA entre pedaços adjacentes (a imagem pagou-a na
   F48/F49 — fio de fundo entre membros); o preço (o bake por pedaço multiplica o relógio — a régua
   é `diag_o_preco_do_efeito_por_quadro`, hoje `0,9`–`2,7 ms`); e o recorte de caminhos ABERTOS
   (`ph2d-vec-boolean::cut`).
4. Perguntar primeiro se a COMPOSIÇÃO já o exprime (§5.0): talvez baste recortar só os caminhos
   abertos da parte de trás pela região da parte da frente deformada (sem partir o preenchimento).

## 4. Os outros abertos (do dono — só se ele reparar)

- A **cúspide da ARTE** junto a uma tampa redonda (limite da F49, imagem presa): pede o fecho sobre o
  contorno da ARTE (pixels na `attach_skin_meshes`).
- No *Zig Zag* muito dobrado os dentes de dentro encavalam-se e fecham buraquinhos reais (a imagem
  também os mostra).

## 5. Lições desta janela (morderam)

- ⛔ **`git checkout -- <f>` para limpar uma SONDA levou um gate por commitar do mesmo ficheiro.**
  Commit antes de sondar, sempre.
- ⛔ **Duas fixturas frescas têm a MESMA entidade**, e a gaveta do desenho é `thread_local` e
  indexada pela entidade: uma referência «independente» deixava a resposta pronta na gaveta da
  outra. Referência numa thread própria.
- ⭐ **Uma régua sobre segmentos inteiros de um efeito forte mente por corda** (`0,025` em
  repouso): amostre o repouso já partido (`parte_nas_voltas`).
- ⭐ Separe as causas antes de curar: com `PH2D_SKIN_CONTACTO=0`, em repouso, e só o bake
  (`so_o_bake()` nos gates) — três dos reports do dono tinham causas diferentes do sintoma.

## 6. Ao fechar a linha (quando o dono mandar)

Gate batched 1× (`BASE=1ad60a1ce bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh`) —
⚠️ os testes da SHELL ainda não correram nesta janela (`ph2d-vec-skin`, `ph2d-vec-boolean` e
`ph2d-vec-scene` mudaram, e a shell consome-os); clippy `--all-targets`; `agent-loop-profile.sh`;
`rm -rf target/*/incremental`; handoff de INTEGRAÇÃO (F48 retirada + F49 + ordem das faces + F50…);
trocar o link da linha Vector no `CLAUDE.md` §5.1; e o binário de smoke.
