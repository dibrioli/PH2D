---
name: feedback-fast-mode-ship
description: "Workflow oficial PH2D — de dia commit --no-verify sem push/CI; no fim do dia ./scripts/ship.sh + fix loop + push + babysit (modo observa-e-corrige, entrega sem falta)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f0521d79-636c-4f46-a1fa-5c161bcf6d2e
---

Enio (2026-05-20): quase todo o tempo estava sendo gasto em commit/push/CI, não em implementar — a validação rodava a cada commit. Decisão: **separar implementar de entregar**.

**Why:** o pre-commit hook (~5min) rodava em todo commit, e o CI ainda falhava em coisas que o hook não cobre (clippy `--all-targets`, machete, deny, audit) → ciclos de fix repetidos. Insustentável.

**How to apply:**
- **De dia:** `git commit --no-verify` (instantâneo, sem hook) pros checkpoints; `cargo check -p <crate>` quando quiser; **NUNCA push/CI no meio do dia**.
- **Fim do dia / quando Enio mandar "commit"/"push"/"ship"/"fim do dia":** entrar em **modo observa-e-corrige** e entregar commits+push+CI verdes SEM FALTA:
  1. `./scripts/ship.sh` — paridade EXATA com a job lint+test do `spike.yml` (fmt, clippy `--all-targets --features ph2d-spike/bevy_ecs`, cargo machete, cargo deny, cargo audit, nextest --workspace). NÃO usar `--all-features` (liga path flecs do spike que o CI nem linta).
  2. Corrigir TODO `✗`, re-rodar até 100% verde. **NÃO pushar antes disso.**
  3. Squash dos checkpoints `--no-verify` em commits limpos.
  4. Push → babysit CI até verde (corrigir+re-push em vermelho; escalar só após 3 falhas do mesmo job).

Oficializado no repo (vale pros agentes que eu levanto, não só nesta sessão): `scripts/ship.sh`, `DIRETRIZ.md` §7.0 (+ §7.2 troca a matriz manual incompleta pelo ship.sh), `CLAUDE.md` seção "Fluxo de trabalho: fast mode / ship". Raiz documentada em [[feedback_precommit_arch_gates]]. `git commit` sempre em background (hook estoura timeout de 2min do foreground).

**REFORÇO (Enio 2026-07-02): "ship só sob meu comando no fim da implementação."** NÃO auto-shippar — nem em "milestones", nem quando o feature "parece pronto", nem depois de gates verdes locais. Commitar local (`--no-verify`) a cada etapa e **PARAR**; só rodar `ship.sh`+push+babysit quando o Enio disser explicitamente "ship"/"push"/"fim". Padrão dele é **smoke manual entre etapas** (ele testa a UI, reporta bugs, aí corrige) — pushar no meio atropela esse loop + arrisca colisão com o trabalho paralelo dele na MESMA branch (ex.: commits "PH2D Wet Paint" intercalados com os meus em 2026-07-02). Ao pushar, `git push` leva TODA a ancestralidade — inclui commits alheios não-pushados; por isso: não pushe sem comando.
