═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR (também nas mensagens curtas entre ferramentas).
⛔ REGRA DO DONO (CLAUDE.md §0.10): problemas equivalentes num ÚNICO bloco. Comparar velocidade NÃO espera horas:
   docs/DevOps/MEDIR_VELOCIDADE.md (a sonda intercalada; o controlo A/A é `PH2D_SONDA_AA=k`).
⛔⛔⛔ NUNCA mate um processo pelo PAI (`kill $PPID`, `pkill -P`, «o pai do órfão»): o pai de um órfão é o
   `systemd --user` e em 05/10 isso fez LOGOUT da sessão do dono. Termine só o que lançou, pelo PID dele; prefira o
   prazo (`timeout`, `PH2D_PRAZO`) e o `TaskStop` das tarefas de fundo.

FASE 0 (já, sem pedir confirmação):
1. cd Worktrees/line-motion-value && pwd && git branch --show-current   → line/motion-value
2. git log --oneline -8 && git status --short --ignored | grep -v target/   → HEAD = o commit «smoke da 2.ª onda aprovado» (o último da linha); árvore limpa (só `!! assets/sprites/`).
FASE 1:
3. git cherry main HEAD | grep -c '^+' (a linha NÃO está integrada: o handoff de integração de 05/10 cobre tudo).
   Se o main andou: `git range-diff main...HEAD` e `git rebase main 2>&1 | tee target/rebase.log`.
4. bash scripts/cargo-check-narrow.sh ph2d-shape-gpu && bash scripts/cargo-check-narrow.sh ph2d-app-motion
FASE 2 (leia DENTRO da worktree):
5. docs/Motion Nodes/121_as_formas_na_placa.md §9.17 e §9.18 INTEIRAS (o plano e os kill-criteria de (E) e (C) já
   estão escritos lá — NÃO os reescreva; acrescente a prova e os resultados por baixo). O handoff de integração
   docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md §6 e §10.
6. DIRETIVA_IMPLEMENTACAO.md (§5: a regra two-strikes) · regras A–K do MODELO_ABERTURA_LINHA.md.
7. Reporte em UMA linha e siga.

O QUE JÁ EXISTE (NÃO reconstrua):
- O passe de formas na placa com os pedaços do §9.15 DOBRADOS (§9.17); a sonda intercalada (`F` · `F-D` · `F-c2`, o
  A/A, `conta` e `escreve` à parte, as arestas e a imagem contra o `F` byte a byte); a porta `PH2D_FORMAS_CONSTANTES`
  e a lista vazia `CONSTANTES_DO_CONTORNO` à espera do próximo `override`.
- ⛔ RECUSAS MEDIDAS (não se reconstroem): L/H/P/G32 (§9.17 — arrumar o caminho de um pedaço não move o tempo: `0`
  derrames e `112` VGPRs, igual); o GRUPO por cópia (§9.14); D1/D2 do `cs_varre`; a esparsa; estreitar a célula; herdar a
  bissectriz; costurar as correntes; as pontas partilhadas.
- Fechado nesta jornada: bug #11 e a família (as cópias da lei numa porta: `ph2d-rig-kinematics`, `ph2d-motion-kit`,
  portão `architecture_a_lei_partilhada_dos_nos_vive_numa_porta`); (D) o contacto na placa — a recusa do doc 115
  RENOVADA até aos milhares com o número (`4 096` peças: `9,3 %` de um quadro na CPU), expira a `16 384`; (F) o
  SIGSEGV da RTX — causa medida (o perfilador segurava o dispositivo), cura no `pass_profiler` (`PassTimestamps` +
  `shutdown`), prova directa sob carga por fazer (§9.18 F diz porquê).

A TAREFA — DOIS ITENS do §9.18, por esta ordem (o (E) primeiro: é velocidade no proxy de telemóvel):
 (E) A EMISSÃO TRACEJADA POR PEÇA, em paralelo. ⚠️ 3.ª topologia da escrita ⇒ two-strikes: PROVAR O MODELO ANTES de
     construir o produto. A prova (§9.18 E): o passeio de cada cópia só pelos TROÇOS + a tabela por troço + o prefixo
     das peças + um passe com UM FIO POR PEÇA a fazer só o `pedaco` (sem arestas), como `override`/ablação medida
     pela sonda intercalada: `escreve` das esticadas tracejadas da iGPU `≤ 0,13` ms ⇒ o modelo vale e constrói-se a
     emissão; acima ⇒ (E) fecha por recusa medida. Kill-criterion do produto: `escreve ≤ 0,15` (de `0,244`), soma
     `≤ 0,86` (de `0,94`), nenhuma cena pior `+5 %` iGPU / `+10 %` RTX, imagem IGUAL ao `F` (o controlo da sonda),
     os `14` gates, a reserva igual. Mutações dos pedaços aceites. Não use memória de grupo nem fase em série de um
     fio (as razões medidas por que o GRUPO perdeu).
 (C) A MORDIDA do traço rente: a placa é a lei; a rota Vello do Motion usa o traçador do Vello (`stroke_uniform`,
     `crates/ph2d-vec-render/src/stroke_uniform.rs` — PARTILHADO com o módulo Vector, outra linha: NÃO o mude). A cura
     é no `encode` do Motion (`crates/ph2d-app-motion/src/motion_shape_gen.rs`, `build_shape_path` ~374–432): o
     contorno TRACEJADO expande-se na CPU pela MESMA lei da placa (peças por pedaço, faixas nas bissectrizes, juntas,
     pontas — como sub-caminhos com o mesmo sentido, preenchidos `nonzero`) e vai ao Vello como FILL. Antes de
     escrever: há na casa uma porta CPU dessa lei? (`ph2d-shape-gpu/src/eixo.rs` constrói o EIXO; a emissão está só
     no WGSL). Uma lei, uma porta: se escrever a expansão em Rust, os gates comparam-na com a placa ao bit-ou-alfa
     `≤ 1`. Kill-criterion (§9.18 C): família nova com pedaços rentes depois de uma quina e PONTA REDONDA no arnês
     do tracejado contra o Vello (`motion_shape_placa_gpu_tracejado_tests.rs`); rota Vello do Motion = placa
     (alfa `≤ 1`); as famílias de hoje não pioram; o `encode` da `=127` tracejada `≤ +10 %` na CPU.
COMO: inventário (agente `explorador`) → prova do modelo (E) → construir → UMA medição intercalada → gate batched 1×
→ mutações SOZINHAS na árvore → acrescente uma §11 ao handoff de integração de 05/10 (a linha ainda não foi
integrada) → smoke ao dono em passos numerados, fotografado (FORMAS + SIMULAÇÃO; a `=127` tracejada) →
`rm -rf target/*/incremental` e o smoke compilado 2×.
Leis que já custaram caro nesta linha:
 - A régua é a SOMA dos passes; o resumo é o MÍNIMO das rodadas; mínimo e mediana longe ⇒ a régua não decide.
 - Um critério «nenhuma cena melhora sem ele» é cego à TROCA entre cenas — escreva a troca aceitável.
 - Com poucas cópias (`72` = duas ondas) registos/ocupação NÃO são o custo — o caminho por fio é.
 - ⛔ `contorno.rs` está em `624/700`; cresceu? MOVA para um irmão (`contorno_capacidade.rs`, `contorno_sondas.rs`).
 - Script de várias etapas: `&&`, nunca `;` depois de um passo que pode falhar. zsh não parte `$var` em palavras:
   listas de pacotes vão por `bash -c`.
 - A placa é partilhada: PH2D_GPU=1 bash scripts/ph2d-run.sh …; NUNCA force. Mutação nunca em paralelo com builds.
 - A memória do Claude aponta para o project-memory do checkout PRINCIPAL: escreva-a em
   Worktrees/line-motion-value/project-memory/ e comite-a na linha.
FORA DE ESCOPO: o contacto da caixa na placa (recusa renovada, §9.18 D) · o `stroke_uniform` do Vector.
DoD: cada item com tabela antes/depois e o kill-criterion escrito antes; recusa medida também fecha · gate batched ·
handoff · smoke fotografado · NÃO integra, NÃO pusha (CLAUDE.md §0.7).
A MÁQUINA É PARTILHADA (regra K): pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
