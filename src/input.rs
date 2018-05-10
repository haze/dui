
use termion::event;
use termion::event::Key::Char;
use types;

pub fn handle_input(arch: &mut types::Architecture, ev: &event::Key) {
    // h s-up
    // j s-down
    // k c-up
    // l c-down
    match ev {
        Char('h') => arch.server_up(),
        Char('j') => arch.server_down(),
        Char('k') => arch.channel_up(),
        Char('l') => arch.channel_down(),

        _ => {}
    }
}