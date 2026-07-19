use forge_runtime::runtime::{run, shutdown, start};

fn main() {
    start();
    run();
    shutdown();
}
