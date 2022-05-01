mod after_help;
mod dd_app;

use dd_app::DDApp;

fn main() {
    let mut app = DDApp::new();
    app.get_args();
}
