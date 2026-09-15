#!/usr/bin/env bash
# Provas de mutação do REPORT DO DONO sobre o rewind dos projécteis (2026-09-15):
#   «o Rewind não está funcionando com os projéteis. Eles têm um comportamento diferente a cada
#    rewind»
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro: um `mv`/`cp` devolve mtime ANTIGO e o cargo serve o build DA
#    MUTAÇÃO ([[feedback_a_mutation_restore_by_mv_leaves_cargo_with_the_mutated_build]]).
# ⚠️ Cada `sed` traz `assert` de CONTAGEM — um `replace` que não casa é no-op silencioso.
set -u
cd "$(dirname "$0")/../../.." || exit 1

BRIDGE=crates/ph2d-physics-ecs/src/bridge
TMP="$(mktemp -d)"
FALHAS=0

restaura() { cp "$TMP/$(basename "$1")" "$1"; touch "$1"; }
guarda()   { cp "$1" "$TMP/$(basename "$1")"; }

conta() { # ficheiro padrão esperado
  local n; n=$(grep -c -- "$2" "$1")
  if [ "$n" != "$3" ]; then
    echo "  ⛔ ANCORA: '$2' aparece $n vezes em $1 (esperado $3) — a mutação NAO entrou"
    return 1
  fi
}

# ⚠️⚠️ **A prova tem CONTROLO sobre o próprio FILTRO**, e não é zelo: um filtro que casa ZERO
# testes faz o `cargo test` sair **verde**, e a 1.ª redacção deste script imprimiu
# «SOBREVIVEU» nas SETE mutações sobre um produto correcto
# ([[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]]). ⛔ `--exact` com um nome
# parcial é exactamente essa forma.
prova() { # nome  filtro-do-teste
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p ph2d-physics-ecs --all-targets -- "$2" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$2' nao casou teste nenhum; a prova nao mediu nada"
    FALHAS=$((FALHAS+1))
    return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$2' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

# ── 1. O laço de replay deixa de dirigir os projécteis ───────────────────────
guarda "$BRIDGE/controllers.rs"
conta "$BRIDGE/controllers.rs" "self.drive_projectiles(sim);" 1 || exit 1
sed -i 's|^        self.drive_projectiles(sim);$|        // MUTADO|' "$BRIDGE/controllers.rs"
prova "o replay dirige os PROJECTEIS" "um_scrub_para_o_meio_replaya_o_projectil"
restaura "$BRIDGE/controllers.rs"

# ── 2. …e o mover de vista de cima ───────────────────────────────────────────
guarda "$BRIDGE/controllers.rs"
sed -i 's|^        self.drive_topdown(sim);$|        // MUTADO|' "$BRIDGE/controllers.rs"
prova "o replay dirige o mover de VISTA DE CIMA" "um_scrub_para_o_meio_replaya_o_mover_de_vista_de_cima"
restaura "$BRIDGE/controllers.rs"

# ── 3. A memória de voo sobrevive ao Reset (o report, à letra) ───────────────
guarda "$BRIDGE/rewind.rs"
conta "$BRIDGE/rewind.rs" "self.projectile_state.clear();" 1 || exit 1
sed -i 's|^        self.projectile_state.clear();$|        // MUTADO|' "$BRIDGE/rewind.rs"
prova "o Reset esquece a memoria de VOO" "um_reset_devolve_o_mesmo_voo"
restaura "$BRIDGE/rewind.rs"

# ── 4. …e a do mover de vista de cima, pela mesma linha ──────────────────────
guarda "$BRIDGE/rewind.rs"
sed -i 's|^        self.topdown_state.clear();$|        // MUTADO|' "$BRIDGE/rewind.rs"
prova "o censo ve a memoria por esquecer" "reconstruir_do_repouso_esquece_as_tres_memorias_de_controlador"
restaura "$BRIDGE/rewind.rs"

# ── 5. O laço de replay volta a chamar UM controlador em vez da porta ───────
guarda "$BRIDGE/rewind.rs"
conta "$BRIDGE/rewind.rs" "self.drive_controllers(sim);" 1 || exit 1
sed -i 's|^            self.drive_controllers(sim);$|            self.drive_players(sim);|' "$BRIDGE/rewind.rs"
prova "o censo da PORTA UNICA ve o desvio" "os_dois_lacos_dirigem_os_controladores_pela_mesma_porta"
restaura "$BRIDGE/rewind.rs"

# ── 6. A porta fica ORFA: o replay deixa de a chamar ────────────────────────
guarda "$BRIDGE/rewind.rs"
sed -i 's|^            self.drive_controllers(sim);$|            // MUTADO|' "$BRIDGE/rewind.rs"
prova "o censo ve a porta sem chamador" "a_porta_unica_e_chamada_pelos_dois_lacos_que_andam_o_relogio"
restaura "$BRIDGE/rewind.rs"

# ── 7. O anúncio de morte volta a ser limpo por TIQUE ───────────────────────
guarda "$BRIDGE/projectile.rs"
conta "$BRIDGE/projectile.rs" "let mut planos: Vec<ProjectileMove>" 1 || exit 1
sed -i 's|^        let mut planos: Vec<ProjectileMove> = Vec::new();$|        self.projectile_done.clear();\n        let mut planos: Vec<ProjectileMove> = Vec::new();|' "$BRIDGE/projectile.rs"
prova "uma morte no MEIO da moldura chega a quem a le" "uma_morte_no_meio_da_moldura_chega_a_quem_a_le"
restaura "$BRIDGE/projectile.rs"

echo
if [ "$FALHAS" = 0 ]; then echo "TODAS as $((7)) mutacoes sangraram."; else echo "⛔ $FALHAS SOBREVIVERAM"; fi
git diff --stat -- "$BRIDGE" | tail -3
exit "$FALHAS"
