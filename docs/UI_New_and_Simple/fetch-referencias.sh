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

echo; echo "── total ──"; du -sh . 2>/dev/null
