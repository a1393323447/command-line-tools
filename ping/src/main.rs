mod app;
mod icmp;
mod ip;

fn main() {
    let ping = app::PingApp::from_args();

    ping.run();
}
