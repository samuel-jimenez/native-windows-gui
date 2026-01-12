#!/bin/sh
# pre-commit.sh

#stash changes

STASH_NAME="pre-commit-$(date +%s)"
git stash save --quiet --keep-index --include-untracked $STASH_NAME



# format
cargo +nightly fmt -- --config imports_granularity="Crate",group_imports="StdExternalCrate"
#re add
git add .

# Test prospective commit
./run_tests.sh
RESULT=$?

STASHES=$(git stash list)
if [[ $STASHES == *"$STASH_NAME"* ]]; then
  git stash pop --quiet
fi

[ $RESULT -ne 0 ] && exit 1
exit 0
