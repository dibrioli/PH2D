#!/usr/bin/env bash
# Prova de mutação do RELEVO POR CAMADA na peça — W4 (`docs/3D/30` §15): a dobra
# única (2D e peça), a escrita só na camada activa, o desfazer por camada, a
# redobra quando a forma da dobra muda, a subida só do relevo, e o painel.
#
# Corre-se pela fatia da linha e com a placa (as corridas `gpu` pedem-na):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_relevo_por_camada.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa
# EXACTAMENTE uma vez, a mutação COMPILA, e a corrida corre N > 0 testes
# (`passed + failed`). Cada corrida tem o seu CONTROLO verde antes.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SRCS=(crates/ph2d-app-sculpt3d/src crates/ph2d-tool-painter/src crates/ph2d-mesh-render/src
      crates/ph2d-panel-painter-layers/src)
BK=$(mktemp -d)
for s in "${SRCS[@]}"; do mkdir -p "$BK/$s"; cp -r "$s/." "$BK/$s/"; done
restore() {
  for s in "${SRCS[@]}"; do rm -rf "$s"; mkdir -p "$s"; cp -r "$BK/$s/." "$s/"; find "$s" -name '*.rs' -exec touch {} +; done
}
# ⚠️ Também no INT/TERM: a fatia da linha (`ph2d-run.sh`) mata por prazo, e um
#    arnês morto a meio deixaria o src MUTADO.
trap restore EXIT
trap 'exit 130' INT TERM

# As corridas, por nome.
corre() {
  case "$1" in
    tool) cargo test -p ph2d-tool-painter --lib -- relief_fold piece_layers impasto 2>&1 ;;
    app) cargo test -p ph2d-app-sculpt3d --lib -- pilha_da_peca doc:: scenes::relevo_camadas tinta_da_peca 2>&1 ;;
    gpu) cargo test -p ph2d-app-sculpt3d --lib -- --ignored --test-threads=1 relevo_painel --skip diag_ 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in tool app gpu; do
    out=$(corre "$c"); rc=$?
    n=$(echo "$out" | contados)
    echo "CONTROLO [$c]: rc=$rc, $n testes"
    [ "$rc" -eq 0 ] && [ "$n" -gt 0 ] || { echo "ABORTO: o controlo [$c] não está verde"; exit 2; }
  done
fi

sangram=0; total=0
muta() { # ficheiro  agulha  substituto  nome  corrida
  local f="$1" agulha="$2" subst="$3" nome="$4" c="$5"
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a âncora casou $n vezes (esperado 1)"; return; fi
  python3 - "$f" "$agulha" "$subst" <<'PY'
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,'w').write(s.replace(a, b, 1))
PY
  touch "$f"
  local out rc corridos
  out=$(corre "$c"); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(echo "$out" | contados)
    if [ "$corridos" -eq 0 ]; then echo "  ABORTO [$nome]: zero testes correram"
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

A=crates/ph2d-app-sculpt3d/src
T=crates/ph2d-tool-painter/src
R=crates/ph2d-mesh-render/src
P=crates/ph2d-panel-painter-layers/src

# ── W4a: a dobra única (o 2D e a peça) ──────────────────────────────────────
muta $T/layers/relief_fold.rs '            h * (1.0 - c) + own * c' '            h + own * c' \
  'R1 o Level deixa de enterrar' app
muta $T/layers/relief_fold.rs '    let own = own * depth;' '    let (own, _) = (own, depth);' \
  'R2 a profundidade é ignorada' app
muta $T/layers/relief_fold.rs '        ids.retain(|&id| self.effectively_visible(id));' '' \
  'R3 a camada escondida entra na dobra' app
muta $T/layers/relief_fold.rs 'pub const RELIEF_FOLD_SEED: f32 = -0.0;' 'pub const RELIEF_FOLD_SEED: f32 = 0.0;' \
  'R4 a dobra começa em +0 (o -0 não atravessa o ficheiro)' app
muta $T/layers/relief_fold.rs '        let depth = norm.clamp(0.0, 1.0).mul_add(2.0, -1.0); // CLAMP-OK: 0..1 track → -1..1 domain' '        let depth = norm.clamp(0.0, 1.0); // CLAMP-OK: mutação' \
  'R5 o curso do arrasto deixa de ter o zero no meio' tool

# ── W4b: a pilha da peça ─────────────────────────────────────────────────────
muta $A/pilha_da_peca_relevo.rs '        corpo = corpo.max(c);' '        corpo = c;' \
  'R6 o corpo é o da última camada, não o máximo' app
muta $A/pilha_da_peca_relevo.rs $'            .relief_layers_bottom_up()\n            .into_iter()\n            .filter_map(|id| {\n                let c = self.pilha.get(id)?;\n                let r' $'            .relief_layers_bottom_up()\n            .into_iter()\n            .rev()\n            .filter_map(|id| {\n                let c = self.pilha.get(id)?;\n                let r' \
  'R7 a peça dobra de cima para baixo' app
muta $A/pilha_da_peca_traco.rs '        self.relevo_nas(&ord, peca);' '' \
  'R8 o incremental esquece o relevo' app
muta $A/pilha_da_peca_traco.rs $'            && let Some(plano) = self.planos.get_mut(&id)\n        {\n            let alvo = plano.relevo' $'            && let Some(plano) = self.base().and_then(|b| self.planos.get_mut(&b))\n        {\n            let alvo = plano.relevo' \
  'R9 o traço escreve o relevo na base' app
muta $A/pilha_da_peca_traco.rs '        w.com_relevo(plano.relevo.clone());' '' \
  'R10 a cópia de trabalho leva o relevo composto' app
muta $A/pilha_da_peca_relevo.rs '            c.has_relief = tem;' '            c.has_relief = false && tem;' \
  'R11 o painel nunca sabe que a camada tem relevo' app
muta $A/pilha_da_peca_relevo.rs $'        let plano = self.planos.get_mut(&id)?;\n        let alvo = plano.relevo.get_or_insert_with' $'        let plano = self.base().and_then(|b| self.planos.get_mut(&b))?;\n        let alvo = plano.relevo.get_or_insert_with' \
  'R12 o desfazer do relevo vai à base' app
muta $A/pilha_da_peca_relevo.rs '                Some((id, c.impasto_depth.to_bits(), c.impasto_composite))' '                Some((id, 0, c.impasto_composite))' \
  'R13 a assinatura não vê a profundidade' app
muta $A/pilha_da_peca.rs $'        if let Some(p) = self.planos.get(&id).cloned() {\n            self.planos.insert(copia, p);' $'        if let Some(mut p) = self.planos.get(&id).cloned() {\n            p.relevo = None;\n            self.planos.insert(copia, p);' \
  'R14 a cópia perde o relevo' app
muta $A/pilha_da_peca_porta.rs '                    && x.is_reference == y.is_reference' '                    && x.is_reference == y.is_reference
                    && x.impasto_depth.to_bits() == y.impasto_depth.to_bits()' \
  'R15 a porta do metadado recusa a profundidade' app

# ── W4b: a placa ─────────────────────────────────────────────────────────────
muta $A/tinta_da_peca_pilha.rs '    *relevo_sujo |= pilha.redobra_o_relevo(peca);' '    let _ = pilha.redobra_o_relevo(peca);' \
  'R16 a redobra não pede a subida do relevo' gpu
muta $R/tinta_gpu_relevo.rs '        queue.write_buffer(&g.alturas, 0, bytemuck::cast_slice(alt));' '' \
  'R17 a subida só do relevo não escreve as alturas' gpu
muta $R/tinta_gpu_relevo.rs $'            queue.write_buffer(&g.inclinacoes, de as u64, &gb[de..ate]);\n        }\n        true' $'            let _ = (de, ate, &gb);\n        }\n        true' \
  'R18 a subida só do relevo não refaz as inclinações' gpu

# ── W4c: o painel ────────────────────────────────────────────────────────────
muta $T/tool/piece_layers.rs '                m.set_impasto_depth_norm(l, v);' '                let _ = (l, v);' \
  'R19 o espelho da peça ignora o arrasto da profundidade' tool
muta $P/paint_rows.rs '    if layer.has_relief {' '    if layer.has_relief && !crate::peca::on_piece() {' \
  'R20 a linha da profundidade não se pinta na peça' gpu

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
else
  echo "RESULTADO: $sangram de $total sangram"
fi
