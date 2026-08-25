---
name: feedback_an_instruction_doc_can_order_what_its_readers_own_fence_forbids
description: Auditar um documento de instruções é confrontá-lo com as CERCAS de quem o executa, não só com o conteúdo dele — a espec mandava o implementador entrar na pasta que o passo 0 dele proíbe
metadata:
  type: feedback
---

Duas vezes na mesma auditoria (R-pré do clean-room do quad remesh, 2026-08-24) o defeito não
estava no que o documento dizia, mas na **colisão entre o documento e a regra do leitor**:

- a espec dava, em dois sítios, o caminho do arnês do oráculo — e o **Passo 0 do bloco do
  implementador** cria um `deny` de leitura para exactamente aquela pasta. Um leitor obediente
  à espec violava o próprio passo 0; obediente ao passo 0, não conseguia executar a espec.
- o bloco de abertura de linha mandava `git rebase main` **todo começo de jornada**, enquanto
  o passo 4 do MESMO bloco (alterado por mim) fazia a worktree nascer de outra branch. As duas
  regras, cada uma correcta sozinha, produziam juntas o dano: arrastar 14 commits alheios para
  dentro da branch nova.

**Why:** um documento de instruções é auditado por leitura linear, e a leitura linear **valida
cada frase contra o assunto**, nunca contra o *regime* de quem vai executar. As cercas do leitor
vivem noutro arquivo (a skill, o bloco, o passo 0) — e é precisamente por isso que a contradição
sobrevive à revisão de quem escreveu. ⚠️ No clean-room isto é caro em dobro: o mecanismo que eu
verifiquei é que o arnês é consumidor **header-only**, e *um erro de compilação despeja o
cabeçalho alheio no terminal* — a exposição involuntária conta na mesma.

**How to apply:**
1. Ao rever qualquer briefing/espec/handoff, abra **lado a lado** o documento e as regras
   permanentes de quem o recebe, e pergunte por instrução: *«isto é alcançável sem quebrar o
   passo 0 / a regra D / a denylist dele?»*
2. ⚠️ **A cura vai no documento que a pessoa executa, nunca num aviso ao lado** — a regra que
   não está no caminho de quem a executa não existe ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).
3. ⛔ Cuidado com a herança de molde: uma regra colada de um template (a «D») continua a falar
   do mundo do template depois de você mudar a premissa dela.

Irmãs: [[feedback_a_tool_is_adopted_only_when_a_written_step_names_it]] ·
[[feedback_a_handoff_can_be_wrong_about_its_own_dirty_file]]
