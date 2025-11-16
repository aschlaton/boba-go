use boba_go::tui::{run_start_page, run_local_game, run_host_game, run_join_game, StartAction};

#[tokio::main]
async fn main() {
    match run_start_page() {
        StartAction::NewLocalGame => {
            if let Err(e) = run_local_game() {
                eprintln!("Game error: {}", e);
            }
        }
        StartAction::HostNetworkGame => {
            if let Err(e) = run_host_game().await {
                eprintln!("Network error: {}", e);
            }
        }
        StartAction::JoinNetworkGame => {
            if let Err(e) = run_join_game().await {
                eprintln!("Network error: {}", e);
            }
        }
        StartAction::HowToPlay => {
            println!("How to play - Coming soon!");
        }
        StartAction::Quit => {}
    }
}
