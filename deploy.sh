#!/bin/bash
BRANCH_NAME=`git rev-parse --abbrev-ref HEAD`
WS_FILES=`git status --porcelain | wc -l`

if [[ $WS_FILES -eq 1 ]]; then
    echo "Dirty workspace! Commit all files and try again."
    exit 1
fi

git checkout deploy
git reset --hard $BRANCH_NAME
trunk build
cp dist/* .
git add .
git commit --amend --no-edit
git push origin deploy -f
git checkout $BRANCH_NAME
