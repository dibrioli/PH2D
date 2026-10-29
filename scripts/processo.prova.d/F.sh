# shellcheck shell=bash
# Item F (comissão 29/10): verificações baratas no commit — `scripts/commit-barato.sh` + o hook
# `.claude/hooks/commit-barato.sh`. Fixtura: um repo em `$tmp` com os gates REAIS copiados (o espelho
# lê deles os tectos e as listas congeladas), o script e o hook.
echo "— F: o commit barato apanha o que a suíte apanharia —"
_f="$tmp/f"
git init -q "$_f"
for _g in crates/ph2d-editor-core/tests/it/architecture_workspace_file_loc_cap.rs \
  crates/ph2d-editor-core/tests/it/architecture_panel_loc_cap.rs \
  crates/ph2d-editor-core/tests/it/architecture_widget_loc_cap.rs \
  crates/ph2d-editor-core/tests/it/hr15_no_hardcoded_ui_strings.rs \
  shells/desktop/tests/it/file_loc_caps.rs shells/desktop/tests/it/fn_loc_caps.rs \
  scripts/commit-barato.sh .claude/hooks/commit-barato.sh rustfmt.toml Cargo.toml; do
  mkdir -p "$_f/$(dirname "$_g")" && cp "$_g" "$_f/$_g"
done
git -C "$_f" add -A && git -C "$_f" -c user.email=p@p -c user.name=p commit -q -m base
mkdir -p "$_f/crates/ph2d-zz/src" "$_f/crates/ph2d-panel-zz/src" "$_f/shells/desktop/src"
_cb() { (cd "$_f" && bash scripts/commit-barato.sh "$@" 2>&1); }
_linhas() { for _ in $(seq "$1"); do echo "// linha"; done; }

_linhas 700 >"$_f/crates/ph2d-zz/src/novo.rs"
_s="$(_cb crates/ph2d-zz/src/novo.rs)"
igual "um .rs novo de 700 linhas passa" "$?" "0"
_linhas 701 >"$_f/crates/ph2d-zz/src/novo.rs"
_s="$(_cb crates/ph2d-zz/src/novo.rs)"; _r=$?
case "$_r:$_s" in 1:*"novo.rs:701 — 701 linhas, tecto 700"*) ok "um .rs novo de 701 linhas é recusado" ;;
  *) ko "701 linhas NÃO recusado (exit $_r): $_s" ;; esac

printf 'pub const C: u32 = 0xFF00FF;\n' >"$_f/crates/ph2d-panel-zz/src/paint.rs"
_s="$(_cb crates/ph2d-panel-zz/src/paint.rs)"; _r=$?
case "$_r:$_s" in 1:*"paint.rs:1 — cor literal \`0xFF00FF\`"*) ok "um hex de cor num painel é recusado" ;;
  *) ko "hex de cor NÃO recusado (exit $_r): $_s" ;; esac
printf 'pub fn f(x: f32) -> f32 {\n    x * 12.0\n}\n' >"$_f/crates/ph2d-panel-zz/src/n.rs"
case "$(_cb crates/ph2d-panel-zz/src/n.rs)" in *"número literal de UI \`12.0\`"*) ok "um f32 literal de UI é recusado" ;;
  *) ko "f32 literal de UI NÃO recusado" ;; esac
printf 'fn  torto( ) {}\n' >"$_f/crates/ph2d-zz/src/torto.rs"
case "$(_cb crates/ph2d-zz/src/torto.rs)" in *"torto.rs:1 — fora do \`cargo fmt\`"*) ok "um .rs fora do rustfmt é recusado" ;;
  *) ko ".rs fora do rustfmt NÃO recusado" ;; esac
{ echo "pub fn grande() {"; for _ in $(seq 200); do echo "    let _a = 1;"; done; echo "}"; } >"$_f/shells/desktop/src/grande.rs"
case "$(_cb shells/desktop/src/grande.rs)" in *"fn grande: 202 linhas, tecto 200"*) ok "uma fn de 202 linhas na shell é recusada" ;;
  *) ko "fn de 202 linhas NÃO recusada" ;; esac

# O hook: nega o `git commit -- <o de 701>`, e `PH2D_COMMIT_BARATO=0` deixa passar.
_h() { jq -n --arg c "$1" --arg d "$_f" '{tool_input:{command:$c},cwd:$d}' | bash .claude/hooks/commit-barato.sh; }
case "$(_h 'git commit --no-verify -m x -- crates/ph2d-zz/src/novo.rs')" in
  *'"permissionDecision": "deny"'*"701 linhas"*) ok "o hook nega o commit do ficheiro de 701" ;;
  *) ko "o hook NÃO negou o commit do ficheiro de 701" ;; esac
igual "PH2D_COMMIT_BARATO=0 desliga o hook" "$(_h 'PH2D_COMMIT_BARATO=0 git commit --no-verify -m x -- crates/ph2d-zz/src/novo.rs')" ""
igual "um comando sem git commit passa calado" "$(_h 'git status -- crates/ph2d-zz/src/novo.rs')" ""

# O espelho lê o tecto do GATE: um gate que muda de forma reprova alto, nunca aprova calado.
sed -i 's/const FILE_LOC_CAP: usize = 700;/const FILE_CAP: usize = 700;/' \
  "$_f/crates/ph2d-editor-core/tests/it/architecture_workspace_file_loc_cap.rs"
case "$(_cb crates/ph2d-zz/src/novo.rs)" in *"espelho partido"*) ok "gate que muda de forma = espelho partido" ;;
  *) ko "gate que mudou de forma NÃO partiu o espelho" ;; esac
