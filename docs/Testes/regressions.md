# Wave 10 — Regressões descobertas no smoke final

Toda regressão encontrada pelo Enio durante o audit visual final vai aqui. Cada entry deve ter:

```
## [SHA do commit suspeito] — [Etapa] — [data]

**Sintoma:** o que vi de errado
**Onde:** janela/painel/tool
**Como reproduzir:** passos exatos
**Severidade:** Critical / High / Medium / Low
```

## 74b6d27 — Etapa 1.A/1.B smoke do Enio — 2026-05-24

### [R-1] BgRemoval: pickcolor + sliders ficam lentos (perf)

**Sintoma:** ao usar a ferramenta pickcolor (eyedropper) no painel BgRemoval, a ferramenta fica muito lenta e não é mais possível arrastar sliders suavemente.

**Onde:** `crates/ph2d-tool-bgremoval/src/tool.rs::add_extra_color` → `current_preview` → `run_canvas_preview` (pipeline de bg-removal em imagem de 512×512).

**Como reproduzir:**
1. Ativar bgremoval em um sprite.
2. Clicar no botão pickcolor no painel.
3. Clicar em um ponto do canvas para amostrar a cor.
4. Tentar arrastar os sliders Tolerance/Feather/Grow → drag travado.

**Causa identificada:** `add_extra_color` (chamado pelo eyedropper) seta `params_dirty = true`. O bridge runtime chama `current_preview` a cada frame; quando dirty, ele roda `run_canvas_preview` que executa o pipeline completo de bg-removal na imagem de 512×512 (~50ms em M-series). Slider drag emite ~60 events/sec, cada um marcando dirty → 60 × 50ms ocupam totalmente o frame budget.

**Severidade:** High — afeta UX interativo da feature principal.

**Fix path (Wave 11):** mover o cook para worker thread + double-buffer (cook em paralelo, swap quando pronto). Alternativa de curto prazo: reduzir preview a 256px durante drag ativo (debounce). Etapa 1.B EXPÔS o bug mas não o introduziu — o pipeline já era síncrono pré-RasterEditTool; agora só é mais óbvio que o cook está no caminho crítico do frame.

**Status:** 📝 ANOTADO — Wave 11 follow-up.

---

### [R-2] BgRemoval: máscara de proteção do preview ≠ commit final

**Sintoma:** o preview overlay mostra uma máscara de proteção limpa e regular acompanhando o contorno do personagem; após Apply, a imagem final tem a máscara muito mais grosseira (verde como blob irregular extending além do contorno).

**Onde:** `crates/ph2d-tool-bgremoval/src/algorithm/mod.rs::run_pipeline` chamado por:
- `run_canvas_preview` (preview, 512px max)
- `run_full_resolution` (commit, full source res)

**Causa identificada:** o pipeline tem **parâmetros em pixels absolutos** que não escalam proporcionalmente entre preview e full-res:
- `params.grow_px` (range `±GROW_FULL_SCALE`) — dilatação/erosão da matte em px. Em preview 512, 30px = 6% da largura; em full 1500, 30px = 2%. Efeito visual proporcional diferente.
- `params.min_island_pixels` (default ~50) — descarta ilhas pequenas como ruído. Em preview, mais ruído sobrevive como ilha grande (relativamente). Em full, mais ruído é descartado mas o ruído de borda fica.

O resultado: preview tem cleanup proporcionalmente mais forte (matte tighter, fewer artifacts); full tem cleanup proporcionalmente mais fraco (matte loose, artifacts surviving).

**Como reproduzir:**
1. Ativar bgremoval em um sprite com background levemente noisy.
2. Adicionar a cor BG via pickcolor.
3. Comparar overlay preview vs estado after-Apply.

**Severidade:** High — quebra contrato implícito "preview = what you'll get" do RasterEditTool. ADR-0041 §1 esperava que `current_preview` e `run_full` produzissem o MESMO resultado (modulo precisão de edge). Aqui produzem shapes substantialmente diferentes.

**Fix path (Wave 11):** escalar `grow_px` e `min_island_pixels` proporcionalmente à razão `source_dims / preview_dims` no `run_canvas_preview` para que o efeito visual seja idêntico em ambas as resoluções. Alternativa: rodar o pipeline em uma escala intermediária comum (e.g. sempre 1024px) e upscale o mask para a resolução final. Etapa 1.B EXPÔS o bug mas não o introduziu — o pipeline tinha o problema desde antes.

**Status:** 📝 ANOTADO — Wave 11 follow-up.

---

(Próximos achados vêm abaixo.)
