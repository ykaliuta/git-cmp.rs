The primary goal of this program is to compare cherry-picked commit
with the original one to check conflicts resolution during
cherry-picking.

It is implemented by cherry-picking one commit to the parent of
another with automatic resolution of conflicts with "their" policy and
running `git diff` on the resulting tree with another commit's tree.

This basic logic is extented to check squashed commit as well. In this
scenario the backported commit consist of several upstream
commits. Instead of one upstream commit, the program accepts list of
the those squashed commits, squashes all upstream commits to the first
one (with the same policy `theirs`), then cherry-picks backported to
the parent of the first upstream and runs diff between that two trees.

Since it all is implemented via merging, the same logic is applied to
another usecase -- comparing rebased branches. The usecase is to
compare changes on the feature branch which is actively developed and
rebased agaist its upstream branch since plain `git diff` between 2
versions will show changes between bases as well.

The program does not use working tree or default index, so safe to run
with any state of the repository.

## Usage

```
git-cmp commit <backported commit> [upstream commits...]
```

where upstream commit is `HEAD` by default.

```
git-cmp branch <old branch> [<common upstream> [<current branch>]]
```
where `common upstream` by default is `main` and `current branch` is `HEAD`.

