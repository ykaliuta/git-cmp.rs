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

With `--autofetch` switch the process is automated and the program can
fetch commits to squash from the commit message. It requires some
discipline on backporting and putting upstream commit references in
the message in predefined formats. 2 formats are supported
- git cherry-pick -x, `(cherry picked ...)` at the beginning of the line:
```
Author: Christian Brauner <brauner@kernel.org>
Date:   Mon Jan 20 12:56:10 2025 +0100

    samples/vfs: fix build warnings

    Fix build warnings reported from linux-next.

    Reported-by: Stephen Rothwell <sfr@canb.auug.org.au>
    Link: https://lore.kernel.org/r/20250120192504.4a1965a0@canb.auug.org.au
    Signed-off-by: Christian Brauner <brauner@kernel.org>
    (cherry picked from commit 68e6b7d98bc64bbf1a54d963ca85111432f3a0b4)

```
- git show, `commit <ID>` at the beginning of the line:
```
Author: Yauheni Kaliuta <y.kaliuta@gmail.com>
Date:   Fri Jan 24 10:10:42 2025 +0200

    samples/vfs: fix build warnings

    commit 68e6b7d98bc64bbf1a54d963ca85111432f3a0b4
    Author: Christian Brauner <brauner@kernel.org>
    Date:   Mon Jan 20 12:56:10 2025 +0100

        samples/vfs: fix build warnings

        Fix build warnings reported from linux-next.

        Reported-by: Stephen Rothwell <sfr@canb.auug.org.au>
        Link: https://lore.kernel.org/r/20250120192504.4a1965a0@canb.auug.org.au
        Signed-off-by: Christian Brauner <brauner@kernel.org>

    Signed-off-by: Yauheni Kaliuta <y.kaliuta@gmail.com>
```

The commits are squashed from top to bottom. If extra commits are
provided in the command line, they are squashed first.

Since it all is implemented via merging, the same logic is applied to
another usecase -- comparing rebased branches. The usecase is to
compare changes on the feature branch which is actively developed and
rebased agaist its upstream branch since plain `git diff` between 2
versions will show changes between bases as well.

The program does not use working tree or default index, so safe to run
with any state of the repository.

## Usage

```
git-cmp commit <backported commit> [--autofetch] [upstream commits...]
```

where upstream commit is `HEAD` by default.

```
git-cmp branch <old branch> [<common upstream> [<current branch>]]
```
where `common upstream` by default is `main` and `current branch` is `HEAD`.

