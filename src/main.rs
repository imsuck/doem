use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Adds a TODO
    #[arg(id = "title", short = 'a', long = "add")]
    add: Option<String>,
    /// TODO content
    #[arg(id = "content", short = 'c', long)]
    content: Option<String>,
    /// TODO urgency
    #[arg(id = "urgency", short = 'u', long)]
    urgency: Option<String>,
    /// Removes a TODO
    #[arg(id = "target", short = 'r', long = "remove")]
    remove: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut command: doem::Command = doem::Command::list();

    if args.add.is_some() && args.remove.is_none() {
        let content = match args.content {
            Some(val) => val,
            None => {
                eprintln!("Please provide TODO's content (--content)");
                return Ok(());
            }
        };
        let urgency = match args.urgency {
            Some(val) => val,
            None => {
                eprintln!("Please provide TODO's urgency (--urgency)");
                return Ok(());
            }
        };
        command = doem::Command::add(args.add.expect("???"), content, urgency).unwrap();
    } else if args.remove.is_some() && args.add.is_none() {
        command = doem::Command::remove(args.remove.expect("???"));
    } else if args.add.is_some() && args.remove.is_some() {
        eprintln!("Please don't use --add and --remove at the same time");
    }

    doem::run(command)?;

    Ok(())
}
