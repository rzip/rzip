use clap::ArgMatches;
// use ignore::WalkBuilder;
// use globset::{Glob, GlobSet, GlobSetBuilder};
// use std::path::PathBuf;
use std::ffi::OsString;

#[derive(Debug)]
pub struct CLIOptions {
    pub included_path_matches: Vec<OsString>,
    pub excluded_path_matches: Vec<OsString>,
    pub destination_folder: OsString,
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
        let included_path_matches = get_paths_from_clap(matches, "files");

        if included_path_matches.is_empty() {
            panic!("You need to provide a valid path to a compressed file.");
        }

        let excluded_path_matches = get_paths_from_clap(matches, "exclude_files");

        let destination_folder: OsString = matches
            .value_of_os("extract_directory")
            .map(OsString::from)
            .unwrap_or_else(|| OsString::from("./"));

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
            RZipOutput::File(self.destination_folder.clone())
        };

        let verbose = match (self.quiet, self.very_quiet) {
            (true, true) => RZipVerbose::None,
            (true, false) => RZipVerbose::Quiet,
            (false, _) => RZipVerbose::Normal,
        };

        let mut actions = vec![];

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
                overwrite: match (self.no_overwrite, self.overwrite_without_asking) {
                    (true, _) => OverwriteMode::Ignore,
                    (false, true) => OverwriteMode::Overwrite,
                    (false, false) => OverwriteMode::Ask,
                },
            };

            actions.push(RZipActions::Extract(extract_option))
        }

        RZipOptions {
            files: build_archive_list(&self.included_path_matches, &self.excluded_path_matches),
            actions,
            output,
            verbose,
        }
    }
}

#[derive(Debug)]
pub struct RZipOptions {
    pub files: Vec<OsString>, //TODO: Change to PathBuf
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
    File(OsString), //TODO: Change to PathBuf
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

fn get_paths_from_clap(matches: &ArgMatches, name: &str) -> Vec<OsString> {
    matches
        .values_of_os(name)
        .map(|paths| paths.map(OsString::from).collect())
        .unwrap_or_default()
}
//
//  Inclusions and exclusions is harder than it may look.
//  1) You need to identify what are globs and what are paths.
//  2) From the paths, you need to indetify what are files and what are folders
//  3) Ignore folders (I think?)
//  4) From the globs, find the main folder from where the glob starts
//      4.1) Example: './test/hola/**/*.zip' starts in './test/hola'
//  5) Walk the directories.
//  6) For each file, match the globs.
//  7) For every positively matched one, try to match the exclusions.
//
//  Code commented until I find the best way to do it.
//
//  Warning: The final return type for this function should be Vec<PathBuf>
fn build_archive_list(included_paths: &[OsString], _excluded_pats: &[OsString]) -> Vec<OsString> {
    /*let mut builder = WalkBuilder::new(&included_paths[0]);

    for path in included_paths.iter().skip(1) {
        builder.add(&path);
    }

    println!("{:?}", included_paths);
    println!("{:?}", excluded_pats);

    let exclusions = build_glob(excluded_pats);

    builder.standard_filters(false);

    let walker = builder.build();

    let mut paths: Vec<PathBuf> = vec![];

    for entry in walker {
        print!("{:?}", entry);
        match entry {
            Ok(ref entry) if entry.error().is_none() => {
                paths.push(entry.path().to_path_buf());
            }
            _ => {}
        }
    }*/

    included_paths.to_vec()
}

/*fn build_glob(globs: &[PathBuf]) -> GlobSet{
    let mut builder = GlobSetBuilder::new();

    for glob in globs {
        // Posible error in non utf8 filesystems
        let globstr = glob.to_str();
        if globstr.is_none() { continue; }

        let glob = Glob::new(globstr.unwrap());
        if glob.is_err() { continue; }

        builder.add(glob.unwrap());
    }

    builder.build().unwrap()
}*/
