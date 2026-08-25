---
name: feedback-a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another
description: Uma recusa MEDIDA vale para a pergunta que ela mediu; com outra pergunta ou outro substrato, ela tem de ser re-medida antes de ser honrada
metadata:
  type: feedback
---

Uma recusa medida é uma resposta a **uma** pergunta, sobre **um** substrato. Antes de a honrar,
confira as duas coisas — a recusa pode estar certa e irrelevante.

**O caso (2026-08-24, `target/` em RAM).** A nota de 22/08 aposentava o tmpfs com dois argumentos:
(1) *mecanismo* — 30 dos 33 GB acabavam no zram, que é RAM, e o swap esgotava; (2) *ganho* —
*«o preço acima está medido; o ganho nunca esteve»*. As duas verdadeiras. E as duas caíram:

- **O SUBSTRATO mudou.** A causa tem nome — `/dev/shm` é um tmpfs **swappável**. O kernel 6.4
  trouxe a montagem **`noswap`** e a máquina corre 6.18. Medido: um tmpfs `noswap` cheio a 100%
  moveu o swap em **zero bytes**. O modo de falha deixou de existir; a nota não sabia disso porque
  a opção não existia quando ela foi escrita.
- **A PERGUNTA era outra.** «O ganho nunca esteve» media **velocidade** — e continua certo
  (28→24 s é ruído). Ninguém tinha medido **escrita**, porque ninguém perguntava pelo desgaste do
  SSD. Na moeda nova: **9,09 → 2,42 GB, −73%**.

⚠️ **E a mesma medição refutou a MINHA intuição de alvo:** pus primeiro só o `incremental/` na RAM
(54% dos bytes, reescrito a cada edição — parecia óbvio). Vale **15%**. O que escreve é `deps/`,
porque o cargo nunca coleta o artefato velho. *A pasta com mais churn não é a pasta com mais
escrita* — meça as duas antes de escolher.

**Como aplicar:** ao encontrar «medido e rejeitado», leia **o que foi medido** e **contra o quê**.
Se a sua pergunta é outra (escrita vs velocidade, memória vs tempo), ou se o mecanismo nomeado tem
uma cura que não existia, a recusa não se aplica — e reconferi-la é obrigação de quem move o número
(CLAUDE.md §0.0). Deixe a emenda **no caminho de quem lê a recusa**, não num doc novo: aqui foram
`hw-profile.sh` (1º comando de todo agente), o script retirado, e o §2 do doc da política.

Relacionado: [[feedback-documented-decision-chesterton-fence]] ·
[[feedback-a-deferral-notes-bar-may-exceed-the-projects-policy]] ·
[[feedback-the-ceiling-is-the-hardwares-never-the-fallbacks]] ·
[[feedback-a-correct-mechanism-can-prescribe-the-wrong-cure]]
