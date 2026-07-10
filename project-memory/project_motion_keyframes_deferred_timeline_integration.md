---
name: project-motion-keyframes-deferred-timeline-integration
description: M2.W1 keyframes do Motion ADIADO (2026-07-09) pro fim do módulo — timeline nasce em outra linha; achados da pesquisa pré-implementação preservados aqui
metadata: 
  node_type: memory
  type: project
  originSessionId: 227796b3-59fd-4ed2-9058-2c1bb85a6059
---

**Decisão (Enio, 2026-07-09):** M2.W1 (parâmetros animáveis/keyframes + timeline do Motion,
handoff completo recebido) foi **ADIADO para o FIM do módulo Motion**. Uma linha paralela
está construindo a timeline; keyframes do Motion integrarão com a timeline **pronta**, não
construirão uma própria. A linha `line-MotionNodes` ficou intocada (HEAD `e9457228`, 5
commits à frente da main, pesquisa foi só-leitura).

**Achados da pesquisa (§2 do handoff) — não re-pagar este custo na retomada:**

1. **Referência MiniCavalryV2 lida** (`/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2`):
   - Precedência de param efetivo: **socket (promotion) > keyframe sample > literal** —
     `pull-evaluator.js:133-148`; o literal NUNCA é destruído (é o fallback). Keyframes
     amostram SEMPRE em timeline-time, não no `time` do eval (Phase 25).
   - Modelo: `node.keyframes[paramName] = { track: [{t, v, easing}] }`, track sorted por t;
     easing NOMEADO (família Penner) armazenado no **keyframe A do segmento** (outgoing).
   - `sampleKeyframeTrack` (`helpers.js:285`): hold antes do 1º e depois do último kf,
     binary search do par, `lerp(a.v, b.v, easing(k))`.
   - Auto-record: **Play desliga auto-record** (snapshot `_autoRecordBeforePlay`, Stop
     restaura; toggle manual invalida o snapshot) — Phase 26.
   - Serialização (`io.js`): salva duration/fps/inPoint/outPoint/loop/pingpong; **nunca**
     playing/currentTime/autoRecord (session-only); re-sort defensivo das tracks no load.

2. **Risco de arquitetura PRA LINHA DA TIMELINE (relevante JÁ):** `ph2d-timeline` liga
   keyframes a entidades ECS via `TargetBinding { target: AnimTarget, entity: u64, prop:
   PropKind }` — `PropKind` é **enum fechado com repr numérico serializado**
   (TranslationX=0..Opacity=5). Um param de Motion é `(NodeId, nome_string_arbitrário)`.
   Se a linha da timeline não projetar o binding com **ponto de extensão append-only**
   (ou target genérico), a integração final do Motion força redesign do binding.

3. **Divergência de unidade a decidir pelo Enio:** `ph2d-timeline::PropKind::Rotation`
   documenta **radianos** (`prop.rs:26`), mas o app padronizou **graus** como unidade
   autorada (Motion migrou em `978aa57c`; `rot` do stream é graus, rad só na borda da
   basis em `ph2d-eval-motion`). Investigar/alinhar quando a timeline integrar.

4. **Substrato Motion já mapeado:** `EvalCtx::param` = override do Graph senão default do
   manifest (panic em nome não-declarado); `Cook` memoiza por fingerprint que INCLUI hash
   FNV dos overrides (`params_fingerprint`) — logo "working graph derivado com amostras
   aplicadas via `set_param`" invalida o memo naturalmente por-frame; `MotionTransport` =
   `tick × fixed_dt` determinístico; `MotionHistory` = snapshot do doc inteiro.
   Gate a manter verde: `every_row_range_contains_its_value_for_every_node_and_param`.

Handoff original completo (escopo W1, aceitação, lições §7) está na conversa de
2026-07-09; o escopo em si segue válido pra retomada — só a parte "construir timeline"
sai (vira integração com [[project-multiagent-modo-l-2026-07-05]] a linha da timeline).
O Motion segue em desenvolvimento SEM timeline (Enio, mesma data): próximo trabalho na
linha é o que não depende de keyframes (ex.: param promotion socket>literal, mais nós).
