mod icmp;
mod ip;
mod app;

fn main() {
    let ping = app::PingApp::from_args();
    
    ping.run();
}
