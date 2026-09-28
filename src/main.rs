use ariadne::{Color, Label, Report, ReportKind, Source};
use color_eyre::{Result, eyre::Context};

mod cli;

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(cfg!(debug_assertions))
        .display_env_section(cfg!(debug_assertions))
        .install()?;

    let args = cli::parse();

    let source = std::fs::read_to_string(&args.file).context("Failed to read file")?;
    let ast = miffed::parse::parse_program(&source, args.delay_slot);

    let path_str = args.file.display().to_string();
    for error in ast.errors() {
        let span = (path_str.as_str(), error.span().into_range());

        let _ = Report::build(ReportKind::Error, span.clone())
            .with_message(error.reason())
            .with_label(
                Label::new(span.clone())
                    .with_message("here")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((path_str.as_str(), Source::from(source.as_str())));
    }

    let Some(ast) = ast.into_output() else {
        return Ok(());
    };

    dbg!(&ast);

    let machine = miffed::exec::Machine::try_from(ast)?;
    let mut buf = [0; 4];

    machine
        .memory
        .read(machine.memory.last_executable_address + 1, &mut buf)?;
    dbg!(buf);

    Ok(())
}
