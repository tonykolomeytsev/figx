git tag -d v0.1.0
git push -d origin v0.1.0
git add .github/** 
git commit --amend --no-edit
git push --force
git tag v0.1.0
git push --tags

