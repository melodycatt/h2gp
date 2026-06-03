#!/usr/bin/env bash
set -e

PARENT_DIR="$(pwd)"
ORIGINALS=(interface hydro_brain hydro_receiver pca9685 icm42688 ina226 record_mac)
WORK_DIR="/tmp/merge_work_$$"

get_target() {
  case "$1" in
    hydro_brain)    echo "brain" ;;
    hydro_receiver) echo "receiver" ;;
    *)              echo "$1" ;;
  esac
}

force_remove() {
  local dir="$1"
  if [ -d "$dir" ]; then
    chmod -R 777 "$dir" 2>/dev/null || true
    rm -rf "$dir"
  fi
}

echo "=== Step 1: Commit and push all repos ==="
for repo in "${ORIGINALS[@]}"; do
  echo "--- $repo ---"
  cd "$PARENT_DIR/$repo"

  if ! git diff --quiet || ! git diff --cached --quiet || [ -n "$(git ls-files --others --exclude-standard)" ]; then
    git add -A
    git commit -m "chore: pre-merge commit"
  else
    echo "Nothing to commit in $repo"
  fi

  git push || echo "Push failed for $repo (no remote?), continuing..."

  cd "$PARENT_DIR"
done

echo ""
echo "=== Step 2: Install git-filter-repo ==="
pip install git-filter-repo -q

echo ""
echo "=== Step 3: Create clean backups (no gitignored files) ==="
mkdir -p "$PARENT_DIR/_backup"
for original in "${ORIGINALS[@]}"; do
  target=$(get_target "$original")
  if [ -d "$PARENT_DIR/_backup/$target" ]; then
    echo "Backup _backup/$target already exists, skipping"
  else
    echo "Backing up $original -> _backup/$target"
    git clone "$PARENT_DIR/$original" "$PARENT_DIR/_backup/$target"
  fi
done

echo ""
echo "=== Step 4: Make temp copies with rewritten history ==="
mkdir -p "$WORK_DIR"
for original in "${ORIGINALS[@]}"; do
  target=$(get_target "$original")
  if [ -d "$WORK_DIR/$target" ]; then
    echo "Temp copy $target already exists, skipping"
  else
    echo "Preparing $original -> $target"
    cp -r "$PARENT_DIR/$original" "$WORK_DIR/$target"
    git -C "$WORK_DIR/$target" filter-repo --to-subdirectory-filter "$target" --force
    # Sync working tree to match rewritten history (files now live under $target/)
    chmod -R 777 "$WORK_DIR/$target"
    git -C "$WORK_DIR/$target" clean -fdx
    git -C "$WORK_DIR/$target" checkout .
  fi
done

echo ""
echo "=== Step 5: Remove .git folders from originals ==="
# Backups and temp copies are already made — safe to do this now.
# Without this, 'git add -A' silently skips nested repos and the
# files never get tracked, causing the 'untracked files' merge error.
for original in "${ORIGINALS[@]}"; do
  echo "Removing .git from $original"
  force_remove "$PARENT_DIR/$original/.git"
done

echo ""
echo "=== Step 6: Init hydrogp as a git repo ==="
cd "$PARENT_DIR"
git init

echo ""
echo "=== Step 7: Create .gitignore ==="
cat > "$PARENT_DIR/.gitignore" << 'GITIGNORE'
# Compiled output
target/
build/

# Backup folder
_backup/
GITIGNORE

for original in "${ORIGINALS[@]}"; do
  target=$(get_target "$original")
  if [ -f "$PARENT_DIR/$original/.gitignore" ]; then
    echo "" >> "$PARENT_DIR/.gitignore"
    echo "# from $target" >> "$PARENT_DIR/.gitignore"
    sed "s|^|$target/|" "$PARENT_DIR/$original/.gitignore" >> "$PARENT_DIR/.gitignore"
  fi
done

echo ""
echo "=== Step 8: Commit everything in hydrogp ==="
git add -A
git commit -m "Initial commit: existing hydrogp files"

echo ""
echo "=== Step 9: Merge repos into hydrogp ==="
for original in "${ORIGINALS[@]}"; do
  target=$(get_target "$original")
  echo "--- Merging $target ---"
  git remote add "$target" "$WORK_DIR/$target"
  git fetch "$target"

  if git ls-remote --exit-code "$WORK_DIR/$target" main &>/dev/null; then
    BRANCH=main
  else
    BRANCH=master
  fi

  git merge --allow-unrelated-histories "$target/$BRANCH" -m "Merge $target into hydrogp" --strategy-option=theirs
  git remote remove "$target"
done

echo ""
echo "=== Step 10: Clean up temp work dir ==="
force_remove "$WORK_DIR"

echo ""
echo "=== Done! ==="
echo "hydrogp is now a git repo with all histories merged."
echo "Backups (no compiled files) are in hydrogp/_backup/ (gitignored)."
echo ""
git log --oneline | head -20
