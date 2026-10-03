# HANDOFF (continuação, janela nova) — `line/Vector`: próximo trabalho = FORMAS VETORIAIS COM EFEITOS (2026-10-02)

> Para o agente que assume a linha numa janela NOVA (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`). Não é
> handoff de integração: o último de integração continua a ser o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). A janela anterior
> (F48–F49) está em [HANDOFF F48](HANDOFF_line_Vector_F48_O_FECHO_DA_IMAGEM_2026-10-02.md) e na
> [fila §F48–§F49](../01_a_fila.md).

> ✅ **ACTUALIZAÇÃO (mesma data, janela seguinte): o §2 FOI FEITO — F50** (`326589c2f` a lei,
> `436f97e63` a cena `=5`). O efeito coze-se no REPOUSO e dobra com a forma; `PH2D_SKIN_EFEITOS=0`
> volta à lei antiga. Mecanismo, medições, preço e os três abertos: [fila §F50](../01_a_fila.md).
> O §2 abaixo fica como estava, como o porquê. **03/10 (F50-d…g, pelos reports do dono):** campo do
> contorno cozido, ajuste que só aceita cúbicas que andam para a frente (curou também o gancho da
> F43 na origem), a união sem lascas e só dos fechados, a agulha do *Bloat*. Os abertos do §3 foram
> auditados contra o código: três eram notas envelhecidas (fila §F50).

## 0. Onde está

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector` |
| HEAD | o commit deste handoff (filho de `8f1c9ae35`); árvore limpa |
| base | `main` `1ad60a1ce` (a linha estava integrada; fez-se `git reset --keep main`) — **10 commits à frente** (com este), nenhum integrado |
| smoke | binário `--profile smoke` compilado na worktree |

Antes de ler: `cd` + `pwd` + `git branch --show-current` (= `line/Vector`). Se o `main` andou: FASE 1 do
modelo (`git cherry main HEAD` dá `>0` — os commits são seus, não integrados ⇒ `git rebase main`).

## 1. O que a janela anterior entregou (o dono aprovou cada smoke; o último: *«smoke OK»*)

1. **F49 — a imagem presa COSE o fio** entre membros que a corrente encosta
   (`ph2d_skeleton_live::skin_image_fecho::costura`): vãos `< 2` texels entre partes a `> 1,25` osso,
   pontas por bissecção, um lado só; `PH2D_SKIN_COSTURA=0` desliga. Custo: `+0,03 ms` sem nada a
   coser, `~0,7 ms` no quadro que cose (posa na CPU). Mutação `12/12`. Limite conhecido: a cúspide
   da ARTE onde uma tampa redonda encosta tangente (vive dentro das células da grelha).
2. **A ordem das faces** — o osso mais adiante na corrente pinta por cima (`ordena_pelo_osso`, uma vez
   por bind na gaveta `skin_bake_cache::desenhada_da_arte`, custo `0` por quadro).
3. **A F48 (a bola da silhueta sobre a borda da malha) foi RETIRADA** — custava até `58 ms` por
   imagem por quadro (*«queda de FPS»*) e saltava entre bico e arco. A `ph2d-vec-boolean` voltou ao
   `main` sem um byte. As recusas medidas estão na fila — não as reconstrua.
4. **A cena `PH2D_VEC_BONE_SMOKE=4`**: a imagem tem agora uma grelha de pontos (ordem do dono, para
   ver a deformação por dentro).
5. **O aberto «a placa não conhece as manchas nem a lei da junta»** do handoff de 01/10 estava
   DESACTUALIZADO (fechado na F9 W2, 20/09); medido: `0` px de diferença placa × CPU.

## 2. ⭐ O PRÓXIMO TRABALHO: formas vetoriais presas com EFEITOS (ordem do dono)

**O que o artista vê hoje.** Uma forma presa a ossos SEM efeito desenha-se pelo *desenho fiel*
(F37–F47: o bake segue o padrão-ouro, a silhueta da pele no contacto). Uma forma COM efeito sai dele
inteira e volta ao caminho antigo — a pele deforma os NÓS e o efeito corre depois sobre os nós
deformados. ⇒ na mesma pose, a forma com efeito dobra pior (facetada, sem a silhueta do contacto) do
que a mesma forma sem efeito.

**Onde está a decisão (fatos do código, conferidos):**
- `crates/ph2d-skeleton-live/src/skin_desenho.rs` — o cabeçalho, secção *«O que NÃO é assado»*:
  *«os efeitos correm sobre a contagem de nós (um Zig Zag sobre 45 nós é outro desenho)»*.
  `o_estilo_serve(viva)` = `viva.effects.is_empty() && viva.paints.iter().all(|e| e.dilate == 0.0)`;
  `os_nos_servem(fonte)` trata as quinas vivas (que JÁ entram: `cozido_para_o_bake` coze-as antes).
- O porteiro: `crates/ph2d-skeleton-live/src/skin_live.rs:412` (`.is_some_and(o_estilo_serve)`).
- Os efeitos: `ph2d_vec_scene::effect::PathEffect` — **dez**: `Trim`, `ZigZag`, `Repeat`, `Bloat`,
  `Warp`, `Falloff`, `Twist`, `Knot`, `Sketch`, `Hatch` (contados no enum em 2026-10-02). A 2.ª exclusão é o *offset de CAD numa camada* (`paints[].dilate`),
  indexado pelo id da fonte numa rota que a geometria viva não leva.
- Quem junta o desenho à cena: `skin_desenho::funde` em
  `shells/desktop/src/render_loop/fase_vector_live_geometry.rs` — ⚠️ a ORDEM ali é load-bearing (a
  forma presa cede a offset/padrão/largura viva e a booleana consome o que os operandos desenham).

**A pergunta a responder ANTES de codar (e medir, não supor):** cada efeito deve correr sobre a
forma de REPOUSO (antes da pele: o efeito é parte do desenho e dobra com ele, como a quina viva) ou
sobre o DESENHO deformado (depois: o efeito é «estilo» aplicado ao que se vê)? Provavelmente difere
por efeito — um *Zig Zag* ou um *Warp* desenhado em repouso deve dobrar com o braço; um *Trim*
(percentagem do comprimento) pode ter de ler o comprimento deformado. Há um oráculo? (Rive/Spine não
têm efeitos de caminho sobre ossos; o Illustrator/Affinity não têm ossos.) ⇒ provavelmente decisão
de PRODUTO por efeito: monte a cena, fotografe as duas leis lado a lado e leve-as ao dono.

**Cena de smoke:** não existe uma com efeito numa forma presa. O número da próxima cena conta-se no
roteador (`ph2d-app-vec/src/smoke_bone*.rs`; a `=4` é a do par). Siga o padrão do par: a mesma
forma com e sem efeito, os mesmos ossos.

## 3. Ainda em aberto (do dono — não decida por ele)

- O **vinco do cotovelo perto de `90°`** (o padrão-ouro tem-no; rotas C/D da pesquisa 04 paradas).
- **Ligar o bake no desenho** (`~135×` mais fiel por `5,7×` o relógio).
- O **`Strength` do envelope** inerte em arte preenchida.
- A **cúspide da arte** junto a uma tampa (limite da F49): só se o dono reparar — pede o fecho sobre o
  contorno da ARTE (pixels na `attach_skin_meshes`, que hoje não os vê).

## 4. Lições de processo desta janela (morderam)

- ⛔ **Restaurar uma mutação com `git checkout -- <f>` apaga o trabalho NÃO commitado nesse ficheiro**
  (aconteceu: uma cura apagada e reaplicada). ⇒ commit ANTES de mutar.
- ⛔ **O shell é fish/zsh: `$LISTA` não parte em palavras** — `git add -- $P` falhou com um caminho
  só. Listas vão em `bash -c 'P=(...); git add -- "${P[@]}"'`.
- ⚠️ **O `fotografa_cena.sh` COMPILA o binário** — nunca fotografe enquanto um agente de mutação
  mexe no código (a foto seria da mutação).
- ⚠️ **A máquina é partilhada e a carga salta** (`load 67` durante esta janela): tempos medidos sob
  carga só valem como razão lado a lado; refaça os absolutos com `uptime` ao lado.
- ⭐ **Uma régua de «coberto» geométrica não é o que o olho vê**: contar triângulo transparente como
  coberto inventou um «fio» que na tela era a baía inteira. A régua de TINTA (alfa) é a do olho.
- ⭐ **Uma lei feita para o vector (a bola) não serve a uma polilinha densa** — a medição de custo por
  pose (`diag_o_custo_da_malha_desenhada`) é o primeiro instrumento de qualquer lei por quadro.

## 5. Ao fechar a linha (quando o dono mandar integrar)

Gate batched 1× (`BASE=1ad60a1ce bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh` — na
última corrida `3 380/3 381` com a única falha, o tecto de LOC, já curada; clippy limpo; fmt limpo),
`agent-loop-profile.sh`, `rm -rf target/*/incremental`, handoff de INTEGRAÇÃO (F48 retirada + F49 +
ordem das faces + o que vier), trocar o link da linha Vector no `CLAUDE.md` §5.1, e o binário de smoke.

## 6. Smoke do que está na linha (aprovado)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=4 PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 cargo run -p ph2d-host-desktop --profile smoke`
— a imagem pontilhada de baixo sem fio cinzento onde o membro dobrado encosta no de baixo;
`PH2D_SKIN_COSTURA=0` mostra o antes.
