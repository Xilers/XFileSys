use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

pub fn register_sig_handler<F>(f: F)
where
    F: Send + 'static + Fn(),
{
    let mut signals =
        Signals::new(&[SIGINT]).expect("Failed to register signal handler: SIGINT register failed");
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            f();
        }
    });
}
