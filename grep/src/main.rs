mod grep_app;

use grep_app::GrepApp;
use std::io::Result;

fn main() -> Result<()> {
    let mut app = GrepApp::new();
    app.get_args();
    app.run()?;

    Ok(())
}
