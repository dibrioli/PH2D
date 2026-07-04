---
name: no-industrial-claims-without-verification
description: "Em ADRs e docs canônicos: ZERO afirmação técnica/industrial sem verificação prévia executável (cargo search / WebFetch oficial / ls / grep / conta explícita). ADR-0055 KTX2/Basis caiu de 9.0 para 5.67/10 porque 4 das 12 CRITICALs foram alucinações factuais verificáveis em segundos."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 85df73a6-feb3-48ba-96a8-47365c0f1f69
---

**Regra:** ZERO afirmação técnica ou industrial em ADR/doc canônico/comentário Rust sem ter executado a
verificação correspondente nesta mesma sessão. Inferência por memória do treino é **insuficiente** —
o ecossistema muda mês a mês.

**Why:** Em 2026-05-26 escrevi ADR-0055 (KTX2/Basis pipeline) com 4 alucinações factuais detectáveis em
segundos por `cargo search` / WebFetch:

1. `basis-universal-rs >= 0.4` — **não existe.** Real: 0.3.1, dormente desde nov/2023. `cargo search
   basis-universal` resolveria em 5s.
2. "Unity 2022.3+ usa KTX2+Basis como texture format default" — **falso.** Unity 6 docs listam BC7/ASTC/
   ETC2; KTX2 é package opcional. WebFetch docs.unity3d.com resolveria em 30s.
3. "ADR-0009 Holographic Radiance Cascades" citada 4× — **não existe** no repo. `ls docs/architecture/
   decisions/0009-*.md` resolveria em 1s.
4. "VRAM -50% (BC7 vs RGBA8)" — **matemática errada.** BC7=8bpp ÷ RGBA8=32bpp = 0.25 → **-75%**. Conta
   explícita em 10s.

Auditoria 3-lente paralela pegou todas as 4 + 8 outras CRITICALs. Score 5.67/10 (Painter ratificado
mesmo dia: 9.0/10). Custou ~3h de sessão escrever ADR-0055 que precisou ser **deletado inteiro**.

**How to apply:** Antes de afirmar em ADR ou doc canônico:

| Tipo de afirmação | Verificação obrigatória | Custo |
|---|---|---|
| Versão de crate (`X = "Y.Z"`) | `cargo search X` + `cargo info X` | 5-15s |
| "Crate X está em maintenance ativa" | `git log` do repo upstream + check último release date | 30s |
| "Engine Y usa Z como default" | WebFetch docs oficiais da versão atual + quote da página | 30-60s |
| Adoption industrial generalizada | Pesquisa de ≥ 2 fontes independentes (docs oficiais, não blog posts) | 1-3min |
| Redução percentual / saving X% | Escrever a conta explícita NO TEXTO: `A bpp ÷ B bpp = ratio → saving` | 30s |
| Citação ADR-NNNN | `ls docs/architecture/decisions/NNNN-*.md` + ler header | 10s |
| Citação SKILL §X.Y | `grep "§X.Y" SKILL_Stack_PH2D_Definitiva.md` | 5s |
| Citação HR-N | Conferir spec EXATA no SKILL §HR-N (letra vs espírito) | 30s |
| Override de Hard Rule | Critério objetivo aplicável a futuras ADRs (não case-by-case) | escrita |
| Termo de indústria (e.g. "ACEScg") | Distinguir conceitos relacionados (tonemap operator ≠ working space ≠ gamut ≠ transfer function) antes de escrever | 1-2min |
| Tipo/cap em ADR sendo amendada | `grep "*_count_is_exact_N\|cap\|frozen"` no source + ler context | 1min |
| Tipo com nome comum (e.g. `ColorProfile`, `Asset`, `Format`) | grep cross-repo + verificar se há DUPLICATA com mesmo nome em outras ADRs | 1min |

**Vermelhinho-trigger (sinais que indicam: PARE e verifique antes de continuar):**

- Você acha que "lembra" da versão de um crate. → `cargo search` antes.
- Você está prestes a escrever "X engine usa Y como default/standard". → WebFetch antes.
- Você está propondo cap-bump em ADR-NNNN. → Conferir TODAS as ADRs ratificadas na última semana
  pra ver se alguma toca o mesmo tipo (especialmente nomes comuns: ColorProfile, ColorSpace, AssetId,
  Format, Variant).
- Você está sobrescrevendo precedente de outra ADR ("override de HR-1 / decisão #N do Enio em data Y").
  → Critério objetivo decisor ou não escreva.
- Você está afirmando ganho percentual sem mostrar a conta. → Escreva a conta no parágrafo.

**Bypass legítimo:** auto-referência ao codebase ATUAL (e.g. "ADR-0055 §2.1 linha 95" quando estou
escrevendo a §2.1 nesta sessão) — esse não exige WebFetch porque é estado próprio do documento.

**Pre-flight para ADR novo (≥ 5 min antes de começar a escrever):**

1. `ls docs/architecture/decisions/ | tail -10` — qual a última numbered?
2. `git log --since="48 hours ago" --oneline docs/architecture/decisions/` — ADRs ratificadas
   recentemente que podem conflitar
3. Para cada nome de tipo / crate / módulo que você vai mencionar: grep cross-repo (incluindo
   ADRs vizinhas)
4. Para cada dep externa: `cargo search` + último commit upstream + open issues count
5. Para cada afirmação industrial: ≥ 2 WebFetches docs oficiais
6. Conferir caps FROZEN nos arch-gates relacionados (`grep -rn "is_exact\|_cap"` no crate target)

Vide também [[feedback-perfection-no-deferrals]] (princípio que rege a régua de qualidade), [[ktx2-phase1-done-phase2-aborted-2026-05-26]] (caso de uso desta lição).
