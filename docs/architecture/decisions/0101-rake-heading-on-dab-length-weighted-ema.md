# ADR-0101 — Rake: heading como propriedade do Dab (EMA length-weighted no motor)

**Status:** Accepted (implementado 2026-06-26; aguarda smoke manual do Enio com caneta para fechar como Done).
**Contexto/decisor:** Enio, 2026-06-26 ("temos um rake que nunca funciona" → rip-out + rewrite limpo, handoff `docs/Painter/HANDOFF_rake_rewrite.md`).
**Substitui/relaciona:** revoga as duas tentativas de Rake desta sessão (v1 lerp por-dab `1cba06cc`; v2 `advance_rake` long-baseline `c6f56f56`). Estende o brush clean-room; o brush **não** é contract-gateado ([project-painter-brush-came-back-cleanroom]), então esta ADR registra a **decisão arquitetural**, não um contrato congelado. Reusa, ortogonalmente, o slot Shape do [ADR-0100](0100-dual-texture-slots-shape-grain.md) (o ponto 5 daquela ADR — "Rake reusa `advance_rake`" — é **superado** aqui).
**Doc de detalhe:** [`docs/Painter/rake_rewrite_design.md`](../../Painter/rake_rewrite_design.md).

## Contexto

O **Rake** ("a rotação da textura segue a direção do traço") foi remendado duas vezes e nunca funcionou.
A math de `texture::dab_basis` sempre esteve correta (`u = normalize_or(dab_dir, angle)`); o bug era a
**fonte** de `dab_dir`: o tool **reconstruía** a direção a jusante, da corda entre centros de dabs
consecutivos (`brush_settings::dab_tangent` → `paint/rake.rs::advance_rake`). Mas os dabs ficam a ~3 px,
sobre um spline suavizado pelo estabilizador — a *direção* dessa corda é **ruído**. Suavizar a saída não
recupera entrada corrompida (v1 anárquico; v2 dependia da densidade de dab). E havia **dois** rakes
paralelos (Shape + Grain, 4 campos de estado em `PaintState`).

## Decisão

1. **A heading do traço é propriedade de primeira-classe do `Dab`** (`pub dir: [f32; 2]`, unitário;
   `[0,0]` = indefinida), computada **uma vez no motor** (`ph2d-painter-brush`), onde a tangente do
   caminho é limpa. `dab_basis` só **lê** `d.dir`. Acaba toda a engenharia-reversa a jusante.
2. **Filtro = EMA da tangente unitária, length-weighted, em espaço de VETOR** (módulo novo `heading.rs`):
   `α = step_len/(step_len + L)`; `heading ← normalize(heading + α·(t − heading))`; `L = max(½·diâmetro,
   8px)`. Vetor (não ângulo) ⇒ **wrap-safe** (reversão ~180° dá snap, não spin). Length-weighted ⇒
   independente da densidade de dab/spacing. **HR-5 transcendental-free** (só `sqrt`; sem RNG ⇒
   determinístico). Modelo validado contra MyPaint (EMA length-weighted) + Krita (escala ∝ tamanho do
   pincel) + Blender (segura a última heading no parado).
3. **Unifica Shape + Grain.** `d.dir` é do caminho, não do slot ⇒ os **dois** `dab_basis` o leem. Apagados
   os 4 campos (`rake_dir`/`rake_accum`/`shape_rake_dir`/`shape_rake_accum`), 4 inits, 4 resets, 8
   writebacks, 4 chamadas `advance_rake`, o módulo `rake.rs` inteiro e o `dab_tangent`.
4. **Update no motor:** `Stroke` carrega `heading`, atualizado em `walk_space` (onde `dir` do spline e
   `to_next` de comprimento-de-arco existem) e carimbado por `dab_at` em **todo** dab. Reset em `begin`.
   `anchored_dab` (não passa por `dab_at`) recebe a heading explícita = direção do arraste. `Line`
   salva/restaura `heading` no preview; `Curve/Circle/Polygon` resetam-na no início do fill.
5. **Eixo e cache inalterados.** Convenção `u = ao-longo-do-traço` mantida (cosmético; flip "atravessa" é
   follow-up de 1 linha). `is_cacheable`/`is_canvas_cacheable` continuam exigindo `!rake`.
6. **Off = byte-idêntico.** Com Rake off, `dab_basis` ignora `d.dir` ⇒ um brush não-Rake é bit-a-bit igual
   ao baseline. Provado por teste.

## Alternativas rejeitadas

- **Carimbar o `dir` cru de `walk_space`** (sem EMA): é a sub-corda de ~3px — mesma classe de ruído.
  Provado: o teste de arco falha (oscila) com a corda crua.
- **Continuar reconstruindo no tool** (v1/v2): a entrada (centros de dab) já está corrompida; insolúvel a
  jusante. Dependia da densidade de dab.
- **EMA em espaço de ÂNGULO:** precisa de caso especial de wrap (±180°) e de `atan2` (viola HR-5).
- **Manter dois rakes (Shape+Grain) separados:** a heading é do caminho, não do slot; dois estados eram
  acoplamento redundante.

## Consequências

- **+** Rake estável que segue o traço em curvas, suave, independente de spacing/tamanho; Shape e Grain
  ganham de graça; código muito menor (4 campos + 1 módulo + 1 fn a menos).
- **+** `Dab` ganha um campo (`dir`); `dab_basis` inalterado na assinatura (já recebia `dab_dir`).
- **−** O `Dab` não é mais `Default`-trivial em testes legados que o constroem por literal (3 sites
  atualizados com `dir: [0,0]`); construções por `..*d` herdam o campo de graça (ex.: `tiling.rs`).
- **Risco residual:** validação final é **visual/caneta** (Enio) — os testes provam tangente+monotonia+
  spacing-independence headless, mas o "feel" da constante `L = ½·diâmetro` pode querer ajuste (é um knob
  fixo bem-calibrado; expor como slider é follow-up).
