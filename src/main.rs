use tokio::runtime::Builder;

fn main() {
    env_logger::init();
    let runtime = Builder::new_current_thread().build().unwrap();
    runtime.block_on(tutorial12_camera::run());
}
