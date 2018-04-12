extern crate cursive;
use cursive::event;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    //ui_tx: mpsc::Sender<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}

pub enum UiMessage {
    UpdateOutput(String),
    Quit,
}

impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>,
               ui_rx: mpsc::Receiver<UiMessage>
    ) -> Ui {
        //let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let mut ui = Ui {
            cursive: Cursive::new(),
            //ui_tx: ui_tx,
            ui_rx: ui_rx,
            controller_tx: controller_tx,
        };


        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let mut ta = OnEventView::new(TextArea::new()
                            .content("")
                            .with_id("input"));
       // ta.set_on_pre_event(Key::Esc, |s| s.quit() );
        let controller_tx_clone = ui.controller_tx.clone();
        ta.set_on_pre_event(Key::Esc, move |s| {
        let input = s.find_id::<TextArea>("input").unwrap();
        let text = input.get_content().to_owned();
        controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });

        ui.cursive.add_layer(LinearLayout::horizontal()
            .child(BoxView::new(SizeConstraint::Fixed(50),
                                SizeConstraint::Fixed(110),
                        ta
                        ))
            .child(BoxView::new(SizeConstraint::Fixed(50),
                                SizeConstraint::Fixed(110),
                                Panel::new(TextView::new("")
                                    .with_id("output")))));


        // Configure a callback
        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_global_callback(Key::Tab, move |c| {
            // When the user presses Tab, send an
            // UpdatedInputAvailable message to the controller.
            let input = c.find_id::<TextArea>("input").unwrap();
            let text = input.get_content().to_owned();
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(text))
                .unwrap();
        });

        ui
    }

    /// Step the UI by calling into Cursive's step function, then
    /// processing any UI messages.
    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        // Process any pending UI messages
        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::UpdateOutput(text) => {
                    let mut output = self.cursive
                        .find_id::<TextView>("output")
                        .unwrap();
                    output.set_content(text);
                },
                UiMessage::Quit => {
                    self.cursive.quit();
                    return false;
                }
            }
        }

        // Step the UI
        self.cursive.step();

        true
    }
}

pub struct Controller {
    tx: mpsc::Sender<UiMessage>,
    rx: mpsc::Receiver<ControllerMessage>,
    ui: Ui,
}

pub enum ControllerMessage {
    UpdatedInputAvailable(String),
    Quit
}

impl Controller {
    /// Create a new controller
    pub fn new(
    ) -> Result<Controller, String> {
        let (c_tx, c_rx) = mpsc::channel::<ControllerMessage>();
        let (ui_tx,ui_rx) = mpsc::channel::<UiMessage>();
        Ok(Controller {
            tx: ui_tx,
            rx: c_rx,
            ui: Ui::new(c_tx, ui_rx), // removed .clone(). no reason to clone the channel
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::UpdatedInputAvailable(text) => {
                        self.tx
                            .send(UiMessage::UpdateOutput(text))
                            .unwrap();
                    },
                    ControllerMessage::Quit => {
                         //self.ui.cursive.quit();
                        self.tx.send(UiMessage::Quit).unwrap();
                        break;
                    }
                };
            }
        }
    }
}

fn main() {
    // Launch the controller and UI
    let controller = Controller::new();
    match controller {
        Ok(mut controller) => controller.run(),
        Err(e) => println!("Error: {}", e),
    };
}