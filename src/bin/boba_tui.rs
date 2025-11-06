use boba_go::tui::{run_start_page, run_local_game, StartAction};

fn main() {
    match run_start_page() {
        StartAction::NewLocalGame => {
            if let Err(e) = run_local_game() {
                eprintln!("Game error: {}", e);
            }
        }
        StartAction::HowToPlay => {
            println!("How to play - TODO");
        }
        StartAction::Quit => {}
    }
}


