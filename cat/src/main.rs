mod cat_app;

use cat_app::CatApp;

use std::io::Result;

fn main() -> Result<()> {
    let mut cat = CatApp::new();
    cat.get_args();
    cat.run()?;

    Ok(())
}
