---
name: feedback-perfection-no-deferrals
description: "Enio exige perfeição desde o início, sem adiamentos — gaps conhecidos viram trabalho na sessão atual, não follow-up de wave futura. Aplica em design, ADRs, contratos, implementação."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e21b615d-adfc-4968-8771-33f79640bfd7
---

**Regra:** zero "follow-up de wave futura" para gaps conhecidos. Quando uma auditoria (adversarial, código review, sense-check) identifica um problema, ele é endereçado **na sessão atual antes de ratificar/commitar/concluir**, não punted para v2/wave-N/futuro.

**Why:** Enio formulou explicitamente em 2026-05-26 durante auditoria da cascata W0 do Painter: "eu quero a perfeição desde o início, sem adiamentos. Coloque isso como regra". O mandato §0 do Painter ("padrão-ouro, sem gambiarras") é a manifestação técnica para Painter; esta é a regra operacional **geral, aplicável a todos os projetos do PH2D**.

**How to apply:**
- Auditorias adversariais que retornem findings críticos/altos/médios → **TODOS** resolvidos na sessão atual antes de fechar. Não use "v2/W-N/futuro" como bypass mecânico.
- "Deferral aceitável" como decisão de design é proibido se a tecnologia para fechar agora existe (não bloqueada por hardware externo, dependência terceira não-resolvida, etc.).
- Quando um auditor diz "isto vai morder em produção", a resposta é endereçar agora, não anotar como risco aceito.
- Caps numéricos que cobrem "Procreate-level" ao invés de "padrão-ouro absoluto" são sintoma — bumpe pro padrão-ouro.
- Se o escopo cresce além do orçamento de tokens da sessão, é melhor avisar e dividir em sessões sequenciais (continuar até erro-zero) do que aceitar "good enough".
- Contraponto importante: gaps **não-conhecidos** (que só emergem com uso real) ficam aceitáveis para iteração — mas gaps **conhecidos** (apontados em auditoria/spec/feedback) não. A regra não força adivinhação; força fechar o que já foi identificado.
- Em conflito: padrão-ouro vence pragmatismo cronograma.

**Escopo refinado 2026-05-27 (anti-inversão da própria regra):**

A regra aplica a gaps **dentro do escopo da decisão atual** (código que está sendo escrito, contrato que está sendo desenhado, feature da wave atual). NÃO aplica a gaps em **decisões adjacentes** (ADRs futuras, traits ainda não materializados em outra crate, runtimes vapor que vivem em outras áreas do projeto).

**Why o refinamento:** descoberto em 2026-05-27 (KTX2 Fase 2, ADR-0055 v3) que aplicar a regra a *decisões adjacentes* causa exatamente o anti-pattern que ela deveria proteger contra. Round 1→Round 4 de auditoria adversarial trocaram classes de drift sem convergir porque cada round tentava "fechar tudo agora" inclusive `Plugin` trait inexistente em outra crate, `ph2d-i18n` runtime ainda não decidido em ADR própria, e `release-game` feature gate de outro escopo. Resultado: 13 vapor dependencies (E1..E13), 660 LOC de ADR com snippets de código que prometiam APIs que não existiam, e ciclo de polish infinito. Três LLMs externas convergiram em diagnóstico: perfeccionismo deslocado do código para o documento, com auditoria adversarial agindo sem oráculo (Goodhart's Law).

**Como diferenciar (litmus test):**
- Se o gap é fechável **modificando código dentro do escopo declarado da sessão/wave/PR atual** → fecha agora (regra clássica).
- Se o gap exige **abrir/modificar outro ADR, materializar trait em outra crate fora do escopo, decidir política de runtime que merece sua própria decisão** → vira owner-ADR slot reservado + entry em §Open Issues do plano vivo, NÃO trabalho-na-sessão.
- Em dúvida: se fechar agora obrigaria você a especular sobre o design de outra coisa que ainda não foi decidida, é gap adjacente. Defer com slot, não invente vapor.

**Consequência prática:** ADRs strategic-level (decisão arquitetural cross-cutting) catalogam dependências externas em §Open Issues / §Cross-ADR Dependencies como informação válida; isso NÃO é violação da regra — é honestidade epistêmica. Auditoria adversarial NÃO deve falhar ADR por "vapor adjacente" se a dependência tem owner identificado.

**Dimensão CUSTO explicitada 2026-05-28:** Enio formulou "minha decisão nesse projeto é: o melhor possível sem pensar em custos". Custo (tempo de build, complexidade/instalação no CI, número de crates, footprint de dependência, tempo de compile) **NÃO é razão válida** para escolher a opção técnica inferior. Quando há fork qualidade-vs-custo (ex: implementação de referência vendored vs wrapper mais leve; decode+encode completo vs decode-only para "economizar crates"; suportar HDR/wide-gamut de verdade vs rejeitar pra simplificar), o Coordenador escolhe **a mais correta/completa** e segue — custo sai da equação como fator de decisão. A escolha permanece técnica (qual é genuinamente melhor), não maximalista cega: ainda vale [[feedback-audit-scope-discipline]] (não invadir crate adjacente) e o litmus de gap-adjacente acima. Caso de aplicação: AVIF W3.T4 — virou Path C (libavif reference impl) + decode E encode + HDR real, em vez do Path A decode-only/reject-HDR que o handoff propunha pra economizar CI install.

**Dimensão SUPRESSÃO explicitada 2026-08-29 — «nada armengado»:** Enio reformulou a regra numa
atualização de stack: *«aqui buscamos o padrão ouro em tudo, o estado da arte. Havendo problemas,
corrigimos. Ao fim não quero nada armengado, quero a perfeição. Busque a perfeição sem olhar os
custos.»*

⚠️ **A faceta nova, e ela é contra-intuitiva: SILENCIAR um diagnóstico é armengo mesmo quando a
FERRAMENTA está genuinamente defeituosa.** O caso: o `clippy::manual_slice_fill` do Rust 1.98 emite
uma sugestão partida — `for slot in &mut x { *slot = v }` vira `&mut x.fill(v);`, com o `&mut` do
laço deixado para trás (**236 de 236** sugestões numa crate, e o `cargo clippy --fix` aplica-a e sai
`exit 0`). Eu pus `#![allow(...)]` com o mecanismo documentado, e **isso estava errado**.

*A justificativa era tecnicamente correta e respondia à pergunta errada.* «A regra está defeituosa,
logo desligo a regra» confunde **a transformação** (boa: `.fill()` é mais claro e pode virar
`memset`) com **o caminho automático até ela** (partido). O padrão-ouro é fazer a transformação —
à mão, nos 236 — e não carregar supressão nenhuma.

**How to apply:**
- Um `allow`/`ignore`/`skip`/`expect` novo é sempre suspeito. Pergunte: *estou a desligar um
  diagnóstico ERRADO sobre o meu código, ou a esconder trabalho que não quero fazer?* Só o primeiro
  é legítimo, e mesmo esse quer a cerca datada e o gatilho de reabertura.
- Quando a ferramenta automática está partida **de forma sistemática**, isso torna-a *tratável*, não
  desculpa: aplique, repare o defeito conhecido, e prove a completude com uma busca que não pode
  casar nada (aqui: nenhuma linha `^\s*&mut .*\.fill\(`). O compilador é o juiz dos casos em que a
  reparação seria errada.
- ⚠️ Meio-feito é o **pior** estado — pior que não começar e pior que terminar. Se metade de uma
  migração mecânica já entrou, terminá-la é a opção barata; deixá-la a meio obriga todo leitor futuro
  a descobrir sozinho qual metade é qual.
- Um teto imposto por dependência de TERCEIROS (o `vello` não suportar wgpu 30) é estado honesto, não
  armengo. Um teto imposto por código que **nós** vendorizamos e não atualizámos (o `ndarray ^0.15`
  do nosso `deep_filter`) é dívida nossa — classifique-os separadamente antes de os aceitar.

Liga com [[feedback-communication-style]] (pt-BR direto, opções concretas), [[feedback-communication-simplicity]] (não over-aski), [[feedback-audit-lens-diversity]] (rotação de lentes), [[no-industrial-claims-without-verification]] (verificações externas), [[feedback-audit-internal-state-grep]] (verificações internas) — quando aplico esta regra, executo direto, não pergunto "tem certeza?". E [[feedback-an-automatic-tools-exit-code-says-nothing-about-what-it-produced]] — foi a medição independente, não o `exit 0`, que revelou o defeito que motivou esta seção.
