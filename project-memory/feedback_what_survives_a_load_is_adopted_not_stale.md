---
name: feedback-what-survives-a-load-is-adopted-not-stale
description: "Estado de sessão que sobrevive a um load não fica \"velho\" — é ADOTADO pelo documento novo (por id, bits ou NOME)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 55388e1a-541d-4d65-8237-d22637f7df4f
---

Trocar o documento (Ctrl+O, load de projeto) e deixar estado de sessão vivo **não produz um valor
obsoleto e inofensivo — produz um valor que o documento NOVO adota**:

- **por id / bits:** o `MotionState::install` já sabia disso (ids de nó são inteirinhos que o próximo
  documento reusa para nós diferentes; a seleção do painel passaria a editar outro nó).
- **por NOME (o pior):** a timeline reconecta binding órfã pelo hash do `Name`
  (`timeline_persist::upkeep`) — é o que faz delete+undo curar a animação. Nomes se repetem entre
  projetos ("Layer 1", "sprite_001"), então a animação do projeto A **passa a dirigir a pose dos
  objetos homônimos do projeto B**: uma animação que não está em arquivo nenhum, com a fila de undo já
  zerada pelo próprio load.
- **por relógio:** um playhead adiantado sobre um pump recém-instalado não é retomada — o pump SCRUBA
  até lá e abre a cena no meio.

**Why:** todo mecanismo de CURA (reconectar por nome, reusar id, retomar tempo) é, na troca de
documento, um mecanismo de CONTAMINAÇÃO. O mesmo código que salva o delete+undo é o que envenena o
load — a diferença é só se a identidade ainda significa a mesma coisa.

**How to apply:** ao carregar um documento, **liste o que a sessão precisa ESQUECER** (não o que
precisa restaurar) e trate cada item: relógio (rebobina **e pausa** — `rewind` preserva o play state),
undo (fila **e baseline** — o baseline sai do MUNDO depois de todas as mutações, não do arquivo, senão
o `post_frame_undo` do mesmo frame registra passo espúrio), documentos não-persistidos (timeline: zere,
é o degradado honesto), pins/seleções/ids keyados por entidade. Gate: um teste por item, dirigindo o
load REAL. [[feedback_try_to_build_the_harness_before_declaring_it_impossible]] ·
[[feedback_stale_comment_and_dead_code_lie]]
