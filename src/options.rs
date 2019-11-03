use clap::ArgMatches;
use ignore::WalkBuilder;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CLIOptions {
    pub included_path_matches: Vec<PathBuf>,
    pub excluded_path_matches: Vec<PathBuf>,
    pub destination_folder: PathBuf,
    pub list_only_short: bool,
    pub output_to_pipe: bool,
    pub test_files_only: bool,
    pub display_filename_only: bool,
    pub no_overwrite: bool,
    pub overwrite_without_asking: bool,
    pub quiet: bool,
    pub very_quiet: bool,
}

impl CLIOptions {
    pub fn from_clap_matches(matches: &ArgMatches) -> CLIOptions {
        let included_path_matches = get_paths_from_clap(
            matches,
            "files",
            "You need to provide a valid path to a compressed file.",
        );
        let excluded_path_matches = get_paths_from_clap(
            matches,
            "files",
            "You need to provide a valid path to a compressed file.",
        );
        let destination_folder = matches
            .value_of_os("extract_directory")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./"));

        CLIOptions {
            included_path_matches,
            excluded_path_matches,
            destination_folder,
            list_only_short: matches.is_present("list_archives"),
            output_to_pipe: matches.is_present("extrat_to_pipe"),
            test_files_only: matches.is_present("test_files"),
            display_filename_only: matches.is_present("display_only"),
            no_overwrite: matches.is_present("no_overwrite"),
            overwrite_without_asking: matches.is_present("overwrite_no_prompt"),
            quiet: matches.is_present("quiet"),
            very_quiet: matches.occurrences_of("quiet") == 2,
        }
    }

    pub fn process_options(&self) -> RZipOptions {
        let output = if self.output_to_pipe {
            RZipOutput::StdOut
        } else {
            RZipOutput::File
        };
        let verbose = if self.quiet && self.very_quiet {
            RZipVerbose::None
        } else if self.quiet {
            RZipVerbose::Quiet
        } else {
            RZipVerbose::Normal
        };

        let mut actions: Vec<RZipActions> = vec![];

        let mut extract = true;

        if self.list_only_short {
            extract = false;
            actions.push(RZipActions::List)
        }

        if self.test_files_only {
            extract = false;
            actions.push(RZipActions::Test)
        }

        if self.display_filename_only {
            extract = false;
            actions.push(RZipActions::DisplayFilename)
        }

        if extract {
            let extract_option = ExtractionOptions {
                overwrite: if self.no_overwrite {
                    OverwriteMode::Ignore
                } else if self.overwrite_without_asking {
                    OverwriteMode::Overwrite
                } else {
                    OverwriteMode::Ask
                },
            };

            actions.push(RZipActions::Extract(extract_option))
        }

        RZipOptions {
            files: build_archive_list(&self.excluded_path_matches, &self.excluded_path_matches),
            actions,
            output,
            verbose,
        }
    }
}

#[derive(Debug)]
pub struct RZipOptions {
    pub files: Vec<PathBuf>,
    pub actions: Vec<RZipActions>,
    pub output: RZipOutput,
    pub verbose: RZipVerbose,
}

#[derive(Debug)]
pub enum RZipVerbose {
    Normal,
    Quiet,
    None,
}

#[derive(Debug)]
pub enum RZipOutput {
    File,
    StdOut,
}

#[derive(Debug)]
pub enum RZipActions {
    List,
    Test,
    DisplayFilename,
    Extract(ExtractionOptions),
}

#[derive(Debug)]
pub struct ExtractionOptions {
    overwrite: OverwriteMode,
}

#[derive(Debug)]
pub enum OverwriteMode {
    Ask,
    Overwrite,
    Ignore,
}

fn get_paths_from_clap(matches: &ArgMatches, name: &str, error: &str) -> Vec<PathBuf> {
    matches
        .values_of_os(name)
        .map(|paths| paths.map(PathBuf::from).collect())
        .expect(error)
}

fn build_archive_list(included_paths: &[PathBuf], excluded_pats: &[PathBuf]) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(&included_paths[0]);

    for path in included_paths.iter().skip(1) {
        builder.add(&path);
    }

    for path in excluded_pats.iter().skip(1) {
        let mut glob = OsString::from("!");
        glob.push(path);
        builder.add(glob);
    }

    builder.standard_filters(false);

    let walker = builder.build();

    let mut paths: Vec<PathBuf> = vec![];

    for entry in walker {
        match entry {
            Ok(ref entry) if entry.error().is_none() => {
                paths.push(entry.path().to_path_buf());
            }
            _ => {}
        }
    }

    paths
}

// Using ignore crate as it is easier to use and seems to be the one
// that will be maintained in the future: https://github.com/rust-lang-nursery/glob/issues/59
