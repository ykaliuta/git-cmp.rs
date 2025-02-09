// SPDX-License-Identifier: Apache-2.0

use git2::{Commit, FileFavor, Index, IndexConflict, MergeOptions, Object, Oid, Repository};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

pub fn repo_open() -> Repository {
    Repository::open_from_env().unwrap()
}

fn revs_to_commits<'a>(repo: &'a Repository, refs: &[String]) -> Vec<Commit<'a>> {
    refs.iter()
        .filter_map(|ref_name| repo.revparse_single(ref_name).ok())
        .filter_map(|obj| obj.peel_to_commit().ok())
        .collect::<Vec<_>>()
}

pub fn cmp_commits(
    repo: &Repository,
    commit_ids: &[String],
    autofetch: bool,
) -> Result<(Oid, Oid), git2::Error> {
    let mut commit_ids = commit_ids.to_owned();
    if !autofetch && commit_ids.len() < 2 {
        commit_ids.push("HEAD".to_string());
    }

    let mut commits = revs_to_commits(repo, &commit_ids);

    if commits.len() != commit_ids.len() {
        return Err(git2::Error::from_str("Some commits were not found."));
    }

    // due to lifetime
    let mut new_commits;
    if autofetch {
        new_commits = commit_to_upstreams(repo, &commits[0]);
        commits.append(&mut new_commits);
    }

    let other = &commits[0];
    let our = &commits[1];
    let our_parent = our.parents().next().unwrap();

    let base = other.parents().next().unwrap();

    let merge = merge_commits_to_oid(repo, &base, &our_parent, other)?;
    let ours = commits.into_iter().skip(1).collect::<Vec<_>>();
    let squash = squash_commits(repo, &ours)?;
    Ok((merge, squash))
}

pub fn cmp_branches(repo: &Repository, commit_ids: &[String]) -> Result<(Oid, Oid), git2::Error> {
    let mut commit_ids = commit_ids.to_owned();
    if commit_ids.len() < 2 {
        commit_ids.push("main".to_string());
    }
    if commit_ids.len() < 3 {
        commit_ids.push("HEAD".to_string());
    }
    let commits = revs_to_commits(repo, &commit_ids);

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

    let merge = merge_commits_to_oid(repo, &their_base, &our_base, other)?;
    Ok((merge, our.id()))
}

fn conflict_path(c: &IndexConflict) -> Vec<u8> {
    c.ancestor
        .as_ref()
        .or(c.our.as_ref())
        .or(c.their.as_ref())
        .unwrap()
        .path
        .clone()
}

fn clear_conflict(idx: &mut Index, c: &IndexConflict) {
    let path = OsString::from_vec(conflict_path(c));
    for i in 1..=3 {
        let _ = idx.remove(path.as_ref(), i);
    }
}

fn merge_objects_to_index(
    repo: &Repository,
    base: &Object,
    our: &Object,
    their: &Object,
) -> Result<Index, git2::Error> {
    let base_tree = base.peel_to_tree()?;
    let our_tree = our.peel_to_tree()?;
    let their_tree = their.peel_to_tree()?;

    let mut opts = MergeOptions::new();
    opts.file_favor(FileFavor::Theirs);

    let mut index = repo.merge_trees(&base_tree, &our_tree, &their_tree, Some(&opts))?;
    if !index.has_conflicts() {
        return Ok(index);
    }

    let conflicts = index.conflicts()?;
    let entries = conflicts.filter_map(Result::ok).collect::<Vec<_>>();

    for conflict in entries.iter() {
        if let Some(ref their) = conflict.their {
            index.add(their)?;
        }
        clear_conflict(&mut index, conflict);
    }

    Ok(index)
}

fn merge_objects_to_oid(
    repo: &Repository,
    base: &Object,
    our: &Object,
    their: &Object,
) -> Result<Oid, git2::Error> {
    let mut index = merge_objects_to_index(repo, base, our, their)?;

    index.write_tree_to(repo)
}

fn merge_commits_to_oid(
    repo: &Repository,
    base: &Commit,
    our: &Commit,
    their: &Commit,
) -> Result<Oid, git2::Error> {
    merge_objects_to_oid(repo, base.as_object(), our.as_object(), their.as_object())
}

fn squash_commits(repo: &Repository, commits: &Vec<Commit>) -> Result<Oid, git2::Error> {
    let merge = Ok(commits[0].as_object().id());

    commits.iter().skip(1).fold(merge, |merge, cur| {
        merge.and_then(|merge| {
            //let merge = repo.find_object(merge, None)?;
            merge_objects_to_oid(
                repo,
                cur.parents().next().unwrap().as_object(),
                &repo.find_object(merge, None)?,
                cur.as_object(),
            )
        })
    })
}

// Handle ^commit <SHA1>
//        ^(cherry picked from commit <SHA1>)
fn line_to_upstream(line: &str) -> Option<&str> {
    let mut words = line.split_ascii_whitespace();
    let first = words.next();
    let second = words.next();

    match (first, second) {
        (Some("commit"), sec) => sec,
        (Some("(cherry"), Some("picked")) => {
            if let Some(sha1) = words.nth(2) {
                Some(&sha1[..sha1.len() - 1])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn commit_to_upstreams<'a>(repo: &'a Repository, commit: &Commit) -> Vec<Commit<'a>> {
    let message: std::str::Lines = commit.message().unwrap().lines();

    message.filter_map(line_to_upstream)
        .filter_map(|u| Oid::from_str(u).ok())
        .filter_map(|u| repo.find_commit(u).ok())
        .collect::<Vec<_>>()
}
