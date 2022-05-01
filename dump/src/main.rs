mod dump_app;

use dump_app::DumpApp;
use std::io::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    let app = DumpApp::from_args();
    app.run()?;
    Ok(())
}
