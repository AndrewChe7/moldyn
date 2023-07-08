use moldyn_gui::visualizer;

fn main() {
    pollster::block_on(visualizer::visualizer_window());
}