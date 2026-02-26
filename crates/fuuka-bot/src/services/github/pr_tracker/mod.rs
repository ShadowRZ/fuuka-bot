use std::{borrow::Cow, collections::BTreeMap};

use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use octocrab::models::commits::GithubCommitStatus;
use regex::{Regex, RegexSet};

use crate::config::RepositoryParts;

pub(crate) mod streams;

pub struct PrTrackerRegexes {
    all_regexs: RegexSet,
    regex_map: Vec<(Regex, Vec<String>)>,
}

impl PrTrackerRegexes {
    fn next_branches(&self, branch: &str) -> Vec<String> {
        let Self {
            all_regexs,
            regex_map,
        } = self;

        all_regexs
            .matches(branch)
            .iter()
            .flat_map(move |item| {
                let (regex, target) = &regex_map[item];
                target.iter().map(|target| regex.replace(branch, target))
            })
            .map(Cow::into_owned)
            .collect()
    }

    fn all_branches(&self, branch: &str) -> Vec<String> {
        std::iter::once(String::from(branch))
            .chain(
                self.next_branches(branch).into_iter().flat_map(|item| {
                    std::iter::once(item.clone()).chain(self.next_branches(&item))
                }),
            )
            .collect()
    }
}

pub struct PrTrackerContext {
    pub branches: BTreeMap<RepositoryParts, PrTrackerRegexes>,
}

impl PrTrackerContext {
    pub fn next_branches(&self, repo: &RepositoryParts, branch: &str) -> Vec<String> {
        let Some(regexes) = self.branches.get(repo) else {
            return vec![];
        };

        regexes.next_branches(branch)
    }

    pub fn all_branches(&self, repo: &RepositoryParts, branch: &str) -> Vec<String> {
        let Some(regexes) = self.branches.get(repo) else {
            return vec![];
        };

        regexes.all_branches(branch)
    }

    pub fn new(
        targets: BTreeMap<RepositoryParts, BTreeMap<String, Vec<String>>>,
    ) -> Result<Self, regex::Error> {
        let mut branches = BTreeMap::new();
        for (key, value) in targets.into_iter() {
            let (regex, target): (Vec<String>, Vec<Vec<String>>) = value.into_iter().unzip();
            let mut res = Vec::new();
            for re in &regex {
                res.push(Regex::new(re)?);
            }
            let all_regexs = RegexSet::new(regex.clone())?;

            branches.insert(
                key,
                PrTrackerRegexes {
                    all_regexs,
                    regex_map: res.into_iter().zip(target.into_iter()).collect(),
                },
            );
        }

        Ok(Self { branches })
    }
}

pub(crate) async fn track(
    client: matrix_sdk::Client,
    context: super::Context,
    repository: RepositoryParts,
    room_id: OwnedRoomId,
    pr_number: i32,
    base: String,
    head: String,
) {
    let Some(ref cron) = context.cron else {
        return;
    };

    loop {
        cron.wait_for_next_tick().await;

        let compare = context
            .octocrab
            .commits(&repository.owner, &repository.repo)
            .compare(&base, &head)
            .per_page(1)
            .send()
            .await;
        match compare {
            Ok(compare) => {
                let in_branch = matches!(
                    compare.status,
                    GithubCommitStatus::Behind | GithubCommitStatus::Identical
                );
                if in_branch {
                    let head = head.clone();
                    if let Some(room) = client.get_room(&room_id)
                        && let Err(error) = room
                            .send_queue()
                            .send(
                                RoomMessageEventContent::text_plain(format!(
                                    "PR #{pr_number} is now in branch {base}!"
                                ))
                                .into(),
                            )
                            .await
                    {
                        tracing::warn!(
                            %room_id,
                            "Failed to queue in branch info to send to room {room_id}: {error}"
                        );
                    }

                    let next_branches = context.pr_tracker.next_branches(&repository, &base);
                    for branch in next_branches {
                        let client = client.clone();
                        let context = context.clone();
                        let head = head.clone();
                        let room_id = room_id.clone();
                        let repository = repository.clone();
                        tokio::task::spawn_local(async move {
                            track(
                                client, context, repository, room_id, pr_number, branch, head,
                            )
                            .await;
                        });
                    }
                }
            }
            Err(error) => tracing::warn!(
                "Failed to compare {owner}/{repo}/{base}...{head}: {error}",
                owner = &repository.owner,
                repo = &repository.repo
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    static CONTEXT: LazyLock<PrTrackerContext> = LazyLock::new(|| {
        let mut branches = BTreeMap::new();
        let mut targets = BTreeMap::new();
        targets.insert(r"\Astaging\z".to_string(), vec!["staging-next".to_string()]);
        targets.insert(r"\Astaging-next\z".to_string(), vec!["master".to_string()]);
        targets.insert(
            r"\Astaging-next-([\d.]+)\z".to_string(),
            vec!["release-$1".to_string()],
        );
        targets.insert(
            r"\Ahaskell-updates\z".to_string(),
            vec!["staging".to_string()],
        );
        targets.insert(
            r"\Amaster\z".to_string(),
            vec![
                "nixpkgs-unstable".to_string(),
                "nixos-unstable-small".to_string(),
            ],
        );
        targets.insert(
            r"\Anixos-(.*)-small\z".to_string(),
            vec!["nixos-$1".to_string()],
        );
        targets.insert(
            r"\Arelease-([\d.]+)\z".to_string(),
            vec![
                "nixpkgs-$1-darwin".to_string(),
                "nixos-$1-small".to_string(),
            ],
        );
        targets.insert(
            r"\Astaging-((1.|20)\.\d{2})\z".to_string(),
            vec!["release-$1".to_string()],
        );
        targets.insert(
            r"\Astaging-((2[1-9]|[3-90].)\.\d{2})\z".to_string(),
            vec!["staging-next-$1".to_string()],
        );
        targets.insert(r"\Astaging-nixos\z".to_string(), vec!["master".to_string()]);

        branches.insert(
            RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            targets,
        );

        PrTrackerContext::new(branches).unwrap()
    });

    #[test]
    fn nixpkgs_staging_next() {
        let branch = "staging-next";
        let targets = CONTEXT.next_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(targets, vec!["master"]);
    }

    #[test]
    fn nixpkgs_master() {
        let branch = "master";
        let targets = CONTEXT.next_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(targets, vec!["nixpkgs-unstable", "nixos-unstable-small"]);
    }

    #[test]
    fn nixpkgs_master_all_branches() {
        let branch = "master";
        let targets = CONTEXT.all_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(
            targets,
            vec![
                "master",
                "nixpkgs-unstable",
                "nixos-unstable-small",
                "nixos-unstable"
            ]
        );
    }

    #[test]
    fn nixpkgs_staging_26_05_all_branches() {
        let branch = "staging-26.05";
        let targets = CONTEXT.all_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(
            targets,
            vec!["staging-26.05", "staging-next-26.05", "release-26.05"]
        );
    }

    #[test]
    fn nixpkgs_staging_25_11_all_branches() {
        let branch = "staging-25.11";
        let targets = CONTEXT.all_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(
            targets,
            vec!["staging-25.11", "staging-next-25.11", "release-25.11"]
        );
    }

    #[test]
    fn nixpkgs_release_25_11_all_branches() {
        let branch = "release-25.11";
        let targets = CONTEXT.all_branches(
            &RepositoryParts {
                owner: "NixOS".to_string(),
                repo: "nixpkgs".to_string(),
            },
            branch,
        );
        pretty_assertions::assert_eq!(
            targets,
            vec![
                "release-25.11",
                "nixpkgs-25.11-darwin",
                "nixos-25.11-small",
                "nixos-25.11"
            ]
        );
    }
}
