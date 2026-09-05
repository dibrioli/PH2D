#!/usr/bin/env bash
# Reconstrói docs/UI_New_and_Simple/referencias/ numa máquina nova.
# O PAYLOAD é gitignorado (bulk de terceiros, licenças alheias); ESTE script é versionado.
# Cada alvo traz a licença ao lado — a triagem de licença vem ANTES de qualquer leitura de fonte.
set -u
cd "$(dirname "$0")"
mkdir -p referencias && cd referencias

clone() { # nome url licenca [subdir...]
  local nome="$1" url="$2" lic="$3"; shift 3
  [ -d "$nome" ] && { echo "· $nome já existe — pulo"; return 0; }
  echo "↓ $nome  ($lic)"
  if [ "$#" -gt 0 ]; then
    git clone --depth 1 --filter=blob:none --sparse "$url" "$nome" -q 2>&1 | tail -2 || { echo "  ✗ falhou"; return 1; }
    ( cd "$nome" && git sparse-checkout set "$@" )
  else
    git clone --depth 1 "$url" "$nome" -q 2>&1 | tail -2 || { echo "  ✗ falhou"; return 1; }
  fi
  echo "$lic" > "$nome/.LICENCA-PH2D"
  du -sh "$nome" | sed 's/^/  /'
}

# ── HIG completos, licença permissiva de DOCUMENTAÇÃO ───────────────────
clone blender-developer-docs https://projects.blender.org/blender/blender-developer-docs.git \
      "CC-BY-SA 4.0 (DOCS; o CÓDIGO do Blender é GPL — não é lido aqui)" \
      docs/features/interface
# ⭐ O MANUAL do Blender — a fonte de Workspaces (layouts) e Modes (per-objecto).
# ⚠️ O HIG (acima) NAO cobre isto: o `modal_interfaces.md` dele e' um esboco vazio.
clone blender-manual https://projects.blender.org/blender/blender-manual.git \
      "CC-BY-SA 4.0 (DOCS)" \
      manual/interface manual/scene_layout manual/editors
clone godot-contributing-docs https://github.com/godotengine/godot-contributing-docs.git "CC-BY 4.0"
clone godot-docs https://github.com/godotengine/godot-docs.git "CC-BY 4.0" \
      tutorials/ui getting_started/introduction contributing/development/editor
clone godot-proposals https://github.com/godotengine/godot-proposals.git "MIT"

# ── Design systems com TOKENS publicados ────────────────────────────────
clone spectrum-design-data https://github.com/adobe/spectrum-design-data.git "Apache-2.0"
clone gnome-hig https://gitlab.gnome.org/GNOME/gnome-devel-docs.git "CC-BY-SA 4.0" hig

# ── Motor de UI do Godot (MIT — LEGÍVEL e PORTÁVEL) ─────────────────────
clone godot-editor-src https://github.com/godotengine/godot.git "MIT" \
      editor/docks editor/themes scene/gui scene/main/window.cpp

# ── MODELOS COM CÓDIGO COMPLETO para o redesenho (pesquisa/08, 2026-09-04) ──
# ⭐ O tema «Modern» do Godot 4.6 (= o godot-minimal-theme portado nativo) já vem em
# godot-editor-src/editor/themes/theme_modern.cpp (MIT). Estes são os OUTROS.
clone godot-minimal-theme https://github.com/passivestar/godot-minimal-theme.git "MIT"
# Pixelorama: uma ferramenta de ARTE feita em Godot — 9 temas DERIVADOS de (base, accent, contrast).
# ⚠️ O sparse-checkout é em modo CONE: aceita DIRECTÓRIOS, nunca ficheiros (um ficheiro dá
#    `fatal: … is not a directory`, e a clonagem fica só com a raiz). Os ficheiros que interessam
#    estão nomeados no comentário de cada linha.
clone pixelorama https://github.com/Orama-Interactive/Pixelorama.git "MIT" \
      assets src/Autoload src/Classes            # assets/theme.tres · Autoload/Themes.gd · Classes/ThemeUtils.gd
# Material Maker: editor de nós em Godot, MIT.
clone material-maker https://github.com/RodZill4/material-maker.git "MIT" material_maker/theme
# Graphite: editor 2D vetorial/raster em Rust (Apache-2.0) — o chrome plano de referência.
clone graphite https://github.com/GraphiteEditor/Graphite.git "Apache-2.0" frontend/src
# Os três toolkits Rust cuja TEORIA de tema é portável para o ph2d-tokens:
clone iced https://github.com/iced-rs/iced.git "MIT" core/src/theme                # palette.rs
clone egui https://github.com/emilk/egui.git "Apache-2.0 OR MIT" crates/egui/src   # style.rs
clone xilem https://github.com/linebender/xilem.git "Apache-2.0" masonry/src       # theme.rs
# Dear ImGui: o `ImGuiStyle` canónico das UIs de ferramenta (MIT). Os ficheiros que interessam
# (`imgui.h`, `imgui_draw.cpp`) estão na RAIZ, que o modo cone traz sempre; o sub-dir é só para
# a clonagem ser esparsa.
clone imgui https://github.com/ocornut/imgui.git "MIT" misc/cpp

echo; echo "── total ──"; du -sh . 2>/dev/null
