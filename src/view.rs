use super::model::*;
use ansi_term::Color;
use regex::Regex;
use std::fmt::{self, Display, Formatter};

pub fn print(p: Prompt, c: Colors, bs: BranchSymbols, ss: StatusSymbols) -> String {
    let state = RepoStateView {
        model: p.repo.state,
        colors: c.clone(),
    };
    let repo = RepoStatusView {
        model: p.repo,
        colors: c.clone(),
    };
    let branch = BranchStatusView {
        model: p.branch,
        symbols: bs,
        colors: c.clone(),
    };
    let local = LocalStatusView {
        model: p.local,
        symbols: ss,
        colors: c,
    };
    let result = format!("{} {} {} {}", repo, state, branch, local);
    format!("{} ", respace(result.trim()))
}

fn respace(s: &str) -> String {
    format!("{}", Regex::new(r"\s+").unwrap().replace_all(s, " "))
}

#[cfg(test)]
mod print_tests {
    use super::*;

    #[test]
    fn respace_foo_bar() {
        assert_eq!(respace("foo   bar"), "foo bar");
    }

    #[test]
    fn prompt_is_respaced() {
        let p = Prompt {
            repo: RepoStatus {
                branch: Some(String::from("master")),
                state: git2::RepositoryState::Clean,
            },
            branch: Some(BranchStatus {
                ahead: 1,
                behind: 4,
            }),
            local: LOCAL_CLEAN,
        };
        let c = NO_COLORS.clone();
        let bs = BranchSymbols {
            ahead: "↑",
            behind: "↓",
        };
        let ss = StatusSymbols {
            nothing: "✓",
            staged: "s",
            unmerged: "m",
            unstaged: "u",
            untracked: ".",
        };
        assert_eq!(print(p, c, bs, ss), "master ↑1↓4 ✓ ");
    }

    #[test]
    fn prompt_is_trimmed() {
        let p = Prompt {
            repo: RepoStatus {
                branch: None,
                state: git2::RepositoryState::Clean,
            },
            branch: None,
            local: LocalStatus {
                staged: 1,
                unmerged: 0,
                unstaged: 0,
                untracked: 3,
            },
        };
        let c = NO_COLORS.clone();
        let bs = BranchSymbols {
            ahead: "↑",
            behind: "↓",
        };
        let ss = StatusSymbols {
            nothing: "✓",
            staged: "s",
            unmerged: "m",
            unstaged: "u",
            untracked: ".",
        };
        assert_eq!(print(p, c, bs, ss), "s1. ");
    }
}

#[derive(Clone)]
pub struct Colors {
    pub ok: Option<Color>,
    pub high: Option<Color>,
    pub normal: Option<Color>,
}

#[cfg(test)]
pub const NO_COLORS: Colors = Colors {
    ok: None,
    high: None,
    normal: None,
};

#[derive(Clone)]
pub struct StatusSymbols<'a> {
    pub nothing: &'a str,
    pub staged: &'a str,
    pub unmerged: &'a str,
    pub unstaged: &'a str,
    pub untracked: &'a str,
}

#[derive(Clone)]
pub struct BranchSymbols<'a> {
    pub ahead: &'a str,
    pub behind: &'a str,
}

pub struct RepoStateView {
    pub model: git2::RepositoryState,
    pub colors: Colors,
}

impl Display for RepoStateView {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match &self.model {
            git2::RepositoryState::Merge => "merge",
            git2::RepositoryState::Revert => "revert",
            git2::RepositoryState::RevertSequence => "revert…",
            git2::RepositoryState::CherryPick => "cherry-pick",
            git2::RepositoryState::CherryPickSequence => "cherry-pick…",
            git2::RepositoryState::Bisect => "bisect",
            git2::RepositoryState::Rebase => "rebase",
            git2::RepositoryState::RebaseInteractive => "rebase-i",
            git2::RepositoryState::RebaseMerge => "rebase-m",
            _ => "",
        };
        let s = View {
            text: s,
            color: self.colors.high,
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod repo_state_view {
    use super::*;

    #[test]
    fn empty() {
        let v = RepoStateView {
            model: git2::RepositoryState::Clean,
            colors: NO_COLORS.clone(),
        };
        assert_eq!(format!("{}", v), "");
    }

    #[test]
    fn rebase() {
        let v = RepoStateView {
            model: git2::RepositoryState::Rebase,
            colors: NO_COLORS.clone(),
        };
        assert_eq!(format!("{}", v), "rebase");
    }
}

pub struct RepoStatusView {
    pub model: RepoStatus,
    pub colors: Colors,
}

impl Display for RepoStatusView {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.model.branch.as_ref().map(|b| View {
            text: b,
            color: self.colors.normal,
        });
        if let Some(b) = b {
            write!(f, "{}", b)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod repo_status_view {
    use super::*;

    #[test]
    fn nothing() {
        let v = RepoStatusView {
            model: RepoStatus {
                branch: None,
                state: git2::RepositoryState::Clean,
            },
            colors: NO_COLORS.clone(),
        };
        assert_eq!(format!("{}", v), "");
    }

    #[test]
    fn branch_is_shown() {
        let v = RepoStatusView {
            model: RepoStatus {
                branch: Some("master".to_owned()),
                state: git2::RepositoryState::Clean,
            },
            colors: NO_COLORS.clone(),
        };
        assert_eq!(format!("{}", v), "master");
    }
}

pub struct BranchStatusView<'a> {
    pub model: Option<BranchStatus>,
    pub symbols: BranchSymbols<'a>,
    pub colors: Colors,
}

impl<'a> Display for BranchStatusView<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.model
            .as_ref()
            .map(|b| {
                if b.ahead == 0 && b.behind == 0 {
                    return Ok(());
                }
                let ahead = StatView {
                    symbol: self.symbols.ahead,
                    n: b.ahead,
                    color: self.colors.normal,
                };
                let behind = StatView {
                    symbol: self.symbols.behind,
                    n: b.behind,
                    color: self.colors.normal,
                };
                write!(f, "{}{}", ahead, behind)
            })
            .unwrap_or(Ok(()))
    }
}

#[cfg(test)]
mod branch_status_view {
    use super::*;

    fn given(m: Option<BranchStatus>) -> String {
        let v = BranchStatusView {
            model: m,
            symbols: BranchSymbols {
                ahead: "↑",
                behind: "↓",
            },
            colors: super::NO_COLORS.clone(),
        };
        format!("{}", v)
    }

    fn given_some(ahead: usize, behind: usize) -> String {
        given(Some(BranchStatus {
            ahead: ahead,
            behind: behind,
        }))
    }

    #[test]
    fn is_empty() {
        assert_eq!(given(None), "");
        assert_eq!(given_some(0, 0), "");
    }

    #[test]
    fn ahead() {
        assert_eq!(given_some(6, 0), "↑6");
    }

    #[test]
    fn behind() {
        assert_eq!(given_some(1, 3), "↑1↓3");
    }
}

const LOCAL_CLEAN: LocalStatus = LocalStatus {
    staged: 0,
    unmerged: 0,
    unstaged: 0,
    untracked: 0,
};

pub struct LocalStatusView<'a> {
    pub model: LocalStatus,
    pub symbols: StatusSymbols<'a>,
    pub colors: Colors,
}

impl<'a> Display for LocalStatusView<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if LOCAL_CLEAN == self.model {
            let v = View {
                text: self.symbols.nothing,
                color: self.colors.ok,
            };
            write!(f, "{}", v)
        } else {
            let unmerged = StatView {
                symbol: self.symbols.unmerged,
                n: self.model.unmerged,
                color: self.colors.high,
            };
            let unstaged = StatView {
                symbol: self.symbols.unstaged,
                n: self.model.unstaged,
                color: self.colors.normal,
            };
            let staged = StatView {
                symbol: self.symbols.staged,
                n: self.model.staged,
                color: self.colors.ok,
            };
            let untracked = View {
                text: if self.model.untracked == 0 {
                    ""
                } else {
                    self.symbols.untracked
                },
                color: None,
            };
            write!(f, "{}{}{}{}", unmerged, staged, unstaged, untracked)
        }
    }
}

#[cfg(test)]
mod local_status_view {
    use super::*;

    fn given(m: LocalStatus) -> String {
        let v = LocalStatusView {
            model: m,
            symbols: StatusSymbols {
                nothing: "✔",
                staged: ".",
                unmerged: "x",
                unstaged: "+",
                untracked: "…",
            },
            colors: NO_COLORS.clone(),
        };
        format!("{}", v)
    }

    #[test]
    fn clean() {
        let v = given(LocalStatus {
            staged: 0,
            unmerged: 0,
            unstaged: 0,
            untracked: 0,
        });
        assert_eq!(v, "✔");
    }

    #[test]
    fn zeroes_are_omitted() {
        let v = given(LocalStatus {
            staged: 1,
            unmerged: 0,
            unstaged: 0,
            untracked: 4,
        });
        assert_eq!(v, ".1…");
    }

    #[test]
    fn not_clean() {
        let v = given(LocalStatus {
            staged: 1,
            unmerged: 2,
            unstaged: 3,
            untracked: 4,
        });
        assert_eq!(v, "x2.1+3…");
    }
}

pub struct View<'a> {
    pub text: &'a str,
    pub color: Option<Color>,
}

impl<'a> Display for View<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.text {
            "" => Ok(()),
            t => match self.color {
                Some(c) => write!(f, "{}", c.paint(t)),
                None => write!(f, "{}", t),
            },
        }
    }
}

#[cfg(test)]
mod simple_view_tests {
    use super::View;
    use ansi_term::Color;

    fn given(text: &str, c: Option<Color>) -> String {
        let v = View {
            text: text,
            color: c,
        };
        format!("{}", v)
    }

    #[test]
    fn empty() {
        assert_eq!(given("", None), "");
        assert_eq!(given("", Some(Color::Red)), "");
    }

    #[test]
    fn correct_text() {
        assert_eq!(given("foo", None), "foo");
        assert_eq!(given("bar", None), "bar");
    }

    #[test]
    fn correct_color() {
        assert_eq!(
            given("foo", Some(Color::Fixed(1))),
            "\u{1b}[38;5;1mfoo\u{1b}[0m"
        );
        assert_eq!(
            given("foo", Some(Color::Fixed(2))),
            "\u{1b}[38;5;2mfoo\u{1b}[0m"
        );
    }
}

pub struct StatView<'a> {
    pub symbol: &'a str,
    pub n: usize,
    pub color: Option<Color>,
}

impl<'a> Display for StatView<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.n {
            0 => Ok(()),
            n => match self.color {
                Some(c) => write!(f, "{}{}", c.paint(self.symbol), n),
                None => write!(f, "{}{}", self.symbol, n),
            },
        }
    }
}

#[cfg(test)]
mod stat_view_tests {
    use super::StatView;
    use ansi_term::Color;

    fn given(prefix: &str, n: usize, c: Option<Color>) -> String {
        let v = StatView {
            symbol: prefix,
            n: n,
            color: c,
        };
        format!("{}", v)
    }

    #[test]
    fn no_text() {
        assert_eq!(given("foo", 0, None), "");
        assert_eq!(given("bar", 0, None), "");
        assert_eq!(given("foo", 0, Some(Color::Red)), "");
    }

    #[test]
    fn text() {
        assert_eq!(given("foo", 1, None), "foo1");
        assert_eq!(given("bar", 1, None), "bar1");
    }

    #[test]
    fn number() {
        assert_eq!(given("foo", 1, None), "foo1");
        assert_eq!(given("foo", 2, None), "foo2");
        assert_eq!(given("foo", 3, None), "foo3");
    }

    #[test]
    fn color() {
        let colors = vec![1, 2, 3];
        for c in colors {
            assert_eq!(
                given("foo", 1, Some(Color::Fixed(c))),
                format!("{}1", Color::Fixed(c).paint("foo"))
            );
        }
    }
}
