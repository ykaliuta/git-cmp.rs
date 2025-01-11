use git2::{Commit, Index, Oid, Repository};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

pub fn repo_open() -> Repository {
    Repository::open_from_env().unwrap()
}

fn revs_to_commits<'a, 'b>(repo: &'a Repository, refs: &'b Vec<String>) -> Vec<Commit<'a>> {
    refs.into_iter()
        .filter_map(|ref_name| repo.revparse_single(ref_name).ok())
        .filter_map(|obj| obj.peel_to_commit().ok())
        .collect::<Vec<_>>()
}

pub fn cmp_commits(repo: &Repository, commit_ids: &Vec<String>) -> Result<(Oid, Oid), git2::Error> {
    let commit_ids = &mut commit_ids.clone();
    if commit_ids.len() < 2 {
        commit_ids.push("HEAD".to_string());
    }
    let commits = revs_to_commits(repo, commit_ids);

    if commits.len() != commit_ids.len() {
        return Err(git2::Error::from_str("Some commits were not found."));
    }

    let other = &commits[0];
    let our = &commits[1];
    let our_parent = our.parents().next().unwrap();

    let base = other.parents().next().unwrap();

    let merge = merge_trees(repo, &base, &our_parent, other)?;
    Ok((merge, our.id()))
}

pub fn cmp_branches(
    repo: &Repository,
    commit_ids: &Vec<String>,
) -> Result<(Oid, Oid), git2::Error> {
    let commit_ids = &mut commit_ids.clone();
    if commit_ids.len() < 2 {
        commit_ids.push("main".to_string());
    }
    if commit_ids.len() < 3 {
        commit_ids.push("HEAD".to_string());
    }
    let commits = revs_to_commits(repo, commit_ids);

    if commits.len() != commit_ids.len() {
        return Err(git2::Error::from_str("Some commits were not found."));
    }

    let other = &commits[0];
    let our = &commits[1];
    let upstream = &commits[2];

    let our_base_oid = repo.merge_base(our.id(), upstream.id())?;
    let their_base_oid = repo.merge_base(other.id(), our_base_oid)?;

    let our_base = repo.find_commit(our_base_oid)?;
    let their_base = repo.find_commit(their_base_oid)?;

    let merge = merge_trees(repo, &their_base, &our_base, other)?;
    Ok((merge, our.id()))
}

fn merge_trees_theirs(
    repo: &Repository,
    base: &Commit,
    our: &Commit,
    their: &Commit,
) -> Result<Index, git2::Error> {
    let base_tree = base.tree()?;
    let our_tree = our.tree()?;
    let their_tree = their.tree()?;

    let mut index = repo.merge_trees(&base_tree, &our_tree, &their_tree, None)?;

    if !index.has_conflicts() {
        return Ok(index);
    }

    let conflicts = index.conflicts()?;
    let entries = conflicts.filter_map(Result::ok).collect::<Vec<_>>();

    for conflict in entries.iter() {
        if let Some(ref their) = conflict.their {
            index.add(&their)?;

            let path = OsString::from_vec(conflict.their.as_ref().unwrap().path.clone());
            for i in 1..=3 {
                let _ = index.remove(path.as_ref(), i);
            }
        }
    }

    Ok(index)
}

fn merge_trees(
    repo: &Repository,
    base: &Commit,
    our: &Commit,
    their: &Commit,
) -> Result<Oid, git2::Error> {
    let mut index = merge_trees_theirs(repo, base, our, their)?;

    index.write_tree_to(repo)
}
