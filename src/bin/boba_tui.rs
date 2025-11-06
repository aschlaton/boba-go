use boba_go::tui::{run_start_page, StartAction};

fn main() {
    match run_start_page() {
        StartAction::NewLocalGame => {
            println!("Start local game - TODO");
        }
        StartAction::HowToPlay => {
            println!("How to play - TODO");
        }
        StartAction::Quit => {}
    }
}


