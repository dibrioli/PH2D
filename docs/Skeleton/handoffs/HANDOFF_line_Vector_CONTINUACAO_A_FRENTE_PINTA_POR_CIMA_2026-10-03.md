# HANDOFF (continuação, janela nova) — `line/Vector`: próximo = A PARTE DA FRENTE PINTA POR CIMA numa forma vectorial presa (2026-10-03)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). A janela anterior
> (F50…F50-j) está na [fila §F50](../01_a_fila.md) e no
> [handoff de 02/10](HANDOFF_line_Vector_CONTINUACAO_FORMAS_COM_EFEITOS_2026-10-02.md).

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

## 2. ⭐ O PRÓXIMO: numa dobra muito forte, a parte da FRENTE pinta por cima (como a imagem)

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

## 3. Os outros abertos (do dono — só se ele reparar)

- A **cúspide da ARTE** junto a uma tampa redonda (limite da F49, imagem presa): pede o fecho sobre o
  contorno da ARTE (pixels na `attach_skin_meshes`).
- No *Zig Zag* muito dobrado os dentes de dentro encavalam-se e fecham buraquinhos reais (a imagem
  também os mostra).

## 4. Lições desta janela (morderam)

- ⛔ **`git checkout -- <f>` para limpar uma SONDA levou um gate por commitar do mesmo ficheiro.**
  Commit antes de sondar, sempre.
- ⛔ **Duas fixturas frescas têm a MESMA entidade**, e a gaveta do desenho é `thread_local` e
  indexada pela entidade: uma referência «independente» deixava a resposta pronta na gaveta da
  outra. Referência numa thread própria.
- ⭐ **Uma régua sobre segmentos inteiros de um efeito forte mente por corda** (`0,025` em
  repouso): amostre o repouso já partido (`parte_nas_voltas`).
- ⭐ Separe as causas antes de curar: com `PH2D_SKIN_CONTACTO=0`, em repouso, e só o bake
  (`so_o_bake()` nos gates) — três dos reports do dono tinham causas diferentes do sintoma.

## 5. Ao fechar a linha (quando o dono mandar)

Gate batched 1× (`BASE=1ad60a1ce bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh`) —
⚠️ os testes da SHELL ainda não correram nesta janela (`ph2d-vec-skin`, `ph2d-vec-boolean` e
`ph2d-vec-scene` mudaram, e a shell consome-os); clippy `--all-targets`; `agent-loop-profile.sh`;
`rm -rf target/*/incremental`; handoff de INTEGRAÇÃO (F48 retirada + F49 + ordem das faces + F50…);
trocar o link da linha Vector no `CLAUDE.md` §5.1; e o binário de smoke.
