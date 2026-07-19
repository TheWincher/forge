use forge_runtime::runtime::Runtime;

fn main() {
    let runtime = Runtime::new();
    runtime.run();
    runtime.shutdown();
}
