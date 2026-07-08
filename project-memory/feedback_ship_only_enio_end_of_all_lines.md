---
name: feedback-ship-only-enio-end-of-all-lines
description: "Ship (ship.sh + push + CI) happens ONLY at the end of the whole multi-line round, and ONLY when Enio explicitly says — never at the end of your own line, and never offered proactively."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6d3039ad-668d-4133-a295-f69680a93752
---

Ship is **cross-line and Enio-only**. Do NOT run ship.sh/push, and do NOT even offer to ship, just because your own line's work is done and smoked. Ship happens **once**, at the end of the round of **all agents across all lines**, and **only Enio decides when** (Enio, 2026-07-07).

**Why:** in Modo L (workstation, parallel lines by worktree) each line accumulates local commits independently; a premature ship would push one line before the others integrate and before Enio has reviewed the whole round. Enio is the sole arbiter of ship timing — he had to correct me after I kept ending reports with "…or ship now".

**How to apply:** after finishing + smoking a feature on your line, report the local commits and STOP. Keep accumulating features/commits locally as Enio asks ("próximo"). Never append "or ship" to a report. Only when Enio explicitly says ship do you run [[feedback-fast-mode-ship]] (ship.sh + push + babysit). This refines [[feedback-fast-mode-ship]]: its "fim: ship" means end-of-**round**, Enio-triggered — not end-of-your-line.
