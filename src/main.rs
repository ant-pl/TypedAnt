use clap::Parser;

mod ant_arg;

fn main() {
    let arg = ant_arg::Args::parse();

    if let Some(filepath) = arg.file {
        file_runner::run_file(&filepath);
        return;
    }

    repl::repl();
}
