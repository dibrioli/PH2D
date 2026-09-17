---
name: project_motion_value_waits_for_the_next_integration_round
description: A `line/motion-value` fechou em 2026-09-06 e a integração daquele dia levou 5 das 6 linhas — esta ficou de fora; o Enio decidiu esperar a RODADA SEGUINTE, com mitigações em vigor.
metadata:
  type: project
---

✅ **RESOLVIDO — HISTÓRICO.** A linha entrou na rodada de **10/09**: o tip `bc02236d7` (o handoff de
integração, 2026-09-10) é ancestral do `main` (`git rev-list --count main..line/motion-value` = `0`,
medido 13/09), e o CLAUDE.md §5 Motion cita o handoff de 10/09. As mitigações abaixo **já não estão em
vigor**; ficam como registo do custo medido de esperar uma rodada.

**O facto (2026-09-06).** A linha fechou às **14:39** com 35 commits sobre `53832c884`. O
integrador começou às **15:01** com um commit que diz *«as lições das SEIS linhas, antes da
integração»*, fundiu **cinco** — `sculpt3d`, `Vector`, `components`, `3DModeling`, `UIUX` — e
fechou às **16:26** dizendo *«nenhuma das seis linhas os podia ver»*. **A `motion-value` é a que
faltou.** ⚠️ Ela **tinha** sido integrada na jornada anterior (`main@{11}`, 2026-09-04 15:58), e
é por isso que o L-System e o handoff de 03/09 estão no `main` — *confirmar «a linha foi
integrada» pelo nome no reflog dá a resposta errada; confirme pelo CONTEÚDO desta jornada*
(`git cat-file -e main:crates/ph2d-node-motion-spring/src/law.rs`).

**A decisão do Enio, 2026-09-06:** *«prefiro esperar»* a próxima rodada de integração. A linha
segue a implementar entretanto.

**Why:** o custo de esperar foi MEDIDO e não é zero. Na rodada que passou, as cinco linhas
tocaram **7 dos 130 ficheiros** desta linha (5%), e esses 7 custaram **três** defeitos que
nenhuma linha via sozinha: um corte `add/add` do mesmo ficheiro (`paint_socket.rs`, feito pelas
duas com as MESMAS três funções), o `mod` + `use` duplicados que daí saíram (o compilador
apanhou), e o `paint.rs` a **617** sobre um teto de **600** que nenhuma das duas estourava
sozinha. À data da decisão o encosto VIVO era **zero** (3DModeling · components · sculpt3d
tinham 2 · 2 · 5 ficheiros novos, nenhum meu; só a `quadextract` encostava, em `CLAUDE.md` e
`shells/desktop/src/main.rs`).

**How to apply — as mitigações em vigor (não as invente de novo):**

1. **Rebase no INÍCIO de cada sessão**, nunca só no fim: `git rebase main` na worktree, antes de
   tocar em código. A distância nunca passa de um dia, e o `rerere` (ligado, com a resolução do
   `paint_socket.rs` já em cache) replica sozinho o que já foi resolvido.
2. **Não tocar nos três ficheiros-hub** até a integração: `CLAUDE.md`,
   `shells/desktop/src/main.rs` e `crates/ph2d-panel-motion-graph/src/paint.rs`. A edição do §5
   que o protocolo exige **já foi feita**; uma segunda é conflito garantido.
3. **`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` no início da
   sessão também**, não só no fecho — o handoff é REFERÊNCIA e envelhece, a corrida é a evidência.
4. **A tabela de colisão do handoff é datada** e tem de ser re-corrida antes de fundir.
5. Marcos vivos: `linha-pronta-para-integrar-2026-09-06` (o tip verde) e `pre-rebase-2026-09-06`.

⚠️ **O risco que NÃO é conflito de texto:** esta linha carrega o **substrato do cartão** (os
params dentro do nó), e o **tutorial em PDF é o smoke do produto** — se outra linha mudar a UI do
cartão antes de esta entrar, o tutorial passa a **ensinar o que não acontece**, que é a família
registada em [[feedback_a_smoke_scene_that_teaches_the_opposite_is_worse_than_no_scene]]. A `UIUX`
(que acabou de passar o app inteiro para cartões) é a vizinha perigosa.

Ver [[feedback_two_lines_can_refactor_the_same_code_differently_and_both_survive_the_merge]] e
[[reference_topic_integration_discipline]].
