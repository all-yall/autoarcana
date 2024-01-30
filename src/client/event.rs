use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::{Duration},
};
use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

/// Terminal client events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Mouse click/scroll
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Terminal event handler
pub struct EventHandler {
    /// Event sender channel
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    /// Event receiver channel
    receiver: mpsc::Receiver<Event>,
    /// Thread polls crossterm for user input and sends them along the channel
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new() -> Self {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        let (sender, receiver) = mpsc::channel();

        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                loop {

                    if event::poll(timeout).expect("unable to poll for event") {
                        let event = event::read().expect("unable to read event");
                        match event {
                            CrosstermEvent::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    // ignore key release
                                    Ok(())
                                }
                            }
                            CrosstermEvent::Mouse(e) => {
                                sender.send(Event::Mouse(e))
                            }
                            CrosstermEvent::Resize(w, h) => {
                                sender.send(Event::Resize(w, h))
                            }
                            _ => unimplemented!(),
                        }
                        .expect("failed to send terminal event")
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self, timeout: Duration) -> Result<Option<Event>> {
        let result = self.receiver.recv_timeout(timeout);
        match result {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(e) => Err(e.into())
        }
    }

}
