use moldyn_gui::visualizer_lite;

fn main() {
    pollster::block_on(visualizer_lite::visualizer_window());
}