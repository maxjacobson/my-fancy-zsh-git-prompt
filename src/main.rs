use std::env::current_dir;
use std::path::PathBuf;

use git2::{Repository, RepositoryState};

struct ZshOutput {
    is_bold: bool,
    color: Option<String>,
    text: String,
}

impl ZshOutput {
    fn new(text: &str) -> Self {
        ZshOutput {
            text: text.to_string(),
            is_bold: false,
            color: None,
        }
    }

    fn set_color(&mut self, color: &str) {
        self.color = Some(color.to_string());
    }

    fn make_bold(&mut self) {
        self.is_bold = true;
    }

    fn output(&self) -> String {
        let mut result = String::new();

        if self.is_bold {
            result.push_str("%B");
        }

        match self.color {
            Some(ref c) => {
                result.push_str("%F{");
                result.push_str(&format!("{}", c));
                result.push_str("%}");
            }
            None => {}
        }

        result.push_str(&self.text);

        match self.color {
            Some(ref _c) => result.push_str("%f"),
            None => {}
        }

        if self.is_bold {
            result.push_str("%b");
        }

        result
    }
}

struct DirectoryContext {
    path: PathBuf,
    repository: Option<Repository>,
}

impl DirectoryContext {
    fn current_directory_short_name(&self) -> Option<String> {
        self.directory_short_name(&self.path)
    }

    fn directory_short_name(&self, path: &PathBuf) -> Option<String> {
        if path.is_dir() {
            path.file_name()
                .map(|name_os_str| name_os_str.to_str().map(|name| name.to_string()))
                .unwrap_or(None)
        } else {
            None
        }
    }

    fn format_subdirectory_path(
        &self,
        repository_path: Option<&std::path::Path>,
        current_working_directory: &PathBuf,
    ) -> Option<String> {
        match repository_path {
            Some(repository_path) => {
                let repository_path_buf = repository_path.to_path_buf();

                match self.directory_short_name(&repository_path_buf) {
                    Some(short_name) => {
                        let mut result = String::new();
                        result.push_str(&short_name);
                        result.push_str("/");

                        let diff = current_working_directory.strip_prefix(repository_path_buf);

                        match diff {
                            Ok(diff_path) => match diff_path.to_str() {
                                Some(diff_path_str) => result.push_str(diff_path_str),
                                None => {}
                            },
                            Err(_) => {}
                        }

                        Some(result)
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    fn path_summary(&self) -> Option<String> {
        match self.repository {
            Some(ref repository) => {
                let repository_workdir = repository.workdir();
                if self.paths_match(repository_workdir, &self.path) {
                    self.current_directory_short_name()
                } else {
                    self.format_subdirectory_path(repository_workdir, &self.path)
                }
            }
            None => self.current_directory_short_name(),
        }
    }

    fn paths_match(
        &self,
        repository_path: Option<&std::path::Path>,
        current_working_directory: &PathBuf,
    ) -> bool {
        match repository_path {
            Some(repository_path) => repository_path == current_working_directory,
            None => false,
        }
    }
}

impl std::fmt::Display for DirectoryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.path_summary() {
            Some(name) => write!(f, "{}", name),
            None => Ok(()),
        }
    }
}

impl std::fmt::Display for ZshOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.output())
    }
}

fn any_files_changed(repository: &Repository) -> bool {
    repository
        .diff_index_to_workdir(None, None)
        .and_then(|diff| diff.stats())
        .and_then(|stats| Ok(stats.files_changed()))
        .map_or(false, |count| count > 0)
}

fn any_untracked_files(repository: &Repository) -> bool {
    repository.statuses(None).map_or(false, |statuses| {
        statuses.iter().any(|entry| entry.status().is_wt_new())
    })
}

fn summarize(repository: &Repository) -> ZshOutput {
    match repository.state() {
        RepositoryState::Clean => match repository.head() {
            Ok(head_reference) => {
                let branch_name = if head_reference.is_branch() {
                    head_reference
                        .shorthand()
                        .unwrap_or_else(|| "(unknown branch)")
                        .to_string()
                } else {
                    format!("{}", head_reference.target().unwrap())
                };

                if any_files_changed(repository) || any_untracked_files(repository) {
                    let text = format!("{}*", &branch_name);
                    let mut output = ZshOutput::new(&text);
                    output.set_color("red");
                    output
                } else {
                    let mut output = ZshOutput::new(&branch_name);
                    output.set_color("blue");
                    output
                }
            }
            Err(_) => {
                let mut output = ZshOutput::new("(no commits yet)");
                output.set_color("yellow");
                output
            }
        },
        RepositoryState::Merge => {
            let mut output = ZshOutput::new("(merging)");
            output.set_color("magenta");
            output
        }
        RepositoryState::Revert => {
            let mut output = ZshOutput::new("(reverting)");
            output.set_color("magenta");
            output
        }
        RepositoryState::RevertSequence => {
            let mut output = ZshOutput::new("(reverting)");
            output.set_color("magenta");
            output
        }
        RepositoryState::CherryPick => {
            let mut output = ZshOutput::new("(cherry-picking)");
            output.set_color("magenta");
            output
        }
        RepositoryState::CherryPickSequence => {
            let mut output = ZshOutput::new("(cherry-picking)");
            output.set_color("magenta");
            output
        }
        RepositoryState::Bisect => {
            let mut output = ZshOutput::new("(bisecting)");
            output.set_color("magenta");
            output
        }
        RepositoryState::Rebase => {
            let mut output = ZshOutput::new("(rebasing)");
            output.set_color("magenta");
            output
        }
        RepositoryState::RebaseInteractive => {
            let mut output = ZshOutput::new("(rebasing)");
            output.set_color("magenta");
            output
        }
        RepositoryState::RebaseMerge => {
            let mut output = ZshOutput::new("(rebasing)");
            output.set_color("magenta");
            output
        }
        RepositoryState::ApplyMailbox => {
            let mut output = ZshOutput::new("(mailbox-applying)");
            output.set_color("magenta");
            output
        }
        RepositoryState::ApplyMailboxOrRebase => {
            let mut output = ZshOutput::new("(mailbox-applying)");
            output.set_color("magenta");
            output
        }
    }
}

fn print_details(dir: DirectoryContext) {
    match dir.repository {
        Some(ref repository) => {
            println!("{} {} ", dir, summarize(repository));
        }
        None => {
            let mut output = ZshOutput::new("(not repo)");
            output.set_color("blue");
            output.make_bold();
            println!("{} {} ", dir, output);
        }
    }
}

fn main() {
    let dir = current_dir();
    if dir.is_err() {
        return;
    }

    let dir_path = dir.unwrap();
    let mut dir_context = DirectoryContext {
        path: dir_path.clone(),
        repository: None,
    };

    let repository = match Repository::discover(&dir_path) {
        Ok(r) => r,
        Err(_) => {
            print_details(dir_context);
            return;
        }
    };

    dir_context.repository = Some(repository);

    print_details(dir_context);
}
