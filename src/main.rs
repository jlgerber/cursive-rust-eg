extern crate cursive;
use cursive::event;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;

static INPUT1: &'static str = "input";
static INPUT2: &'static str = "input2";

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
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
            ui_rx: ui_rx,
            controller_tx: controller_tx,
        };


        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let mut ta = OnEventView::new(TextArea::new()
                            .content("")
                            .with_id(INPUT1));
        let controller_tx_clone = ui.controller_tx.clone();

        ta.set_on_pre_event(Key::Esc, move |s| {
            let input = s.find_id::<TextArea>(INPUT1).unwrap();
            controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });
        let controller_tx_clone = ui.controller_tx.clone();

        ta.set_on_pre_event(Key::Enter, move |s| {
            let input = &mut s.find_id::<TextArea>(INPUT1).unwrap();
            let text = format!("input1: {}",input.get_content());
            input.set_content("");
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(text))
                .unwrap();
        });

       let mut tb = OnEventView::new(TextArea::new()
                            .content("")
                            .with_id(INPUT2));
        let controller_tx_clone = ui.controller_tx.clone();
        tb.set_on_pre_event(Key::Esc, move |s| {
            controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        tb.set_on_pre_event(Key::Enter, move |s| {
            let input = &mut s.find_id::<TextArea>(INPUT2).unwrap();
            let text = format!("input2: {}",input.get_content());

            input.set_content("");
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(text))
                .unwrap();
        });


        let width = SizeConstraint::Fixed(50);
        let half_height = SizeConstraint::Fixed(20);
        let sp_ht = SizeConstraint::Fixed(2);
        let input_pair = LinearLayout::vertical()
            .child(BoxView::new(width, half_height,ta))
            .child(BoxView::new(width, sp_ht, TextView::new("")))
            .child(BoxView::new(width,half_height, tb));

        ui.cursive.add_layer(LinearLayout::horizontal()
            .child(BoxView::new(SizeConstraint::Fixed(50),
                                SizeConstraint::Fixed(110),
                        input_pair
                        ))
            .child(BoxView::new(SizeConstraint::Fixed(50),
                                SizeConstraint::Fixed(110),
                                Panel::new(TextView::new("")
                                    .with_id("output")))));


        // Configure a callback
        // let controller_tx_clone = ui.controller_tx.clone();
        // ui.cursive.add_global_callback(Key::Tab, move |c| {
        //     // When the user presses Tab, send an
        //     // UpdatedInputAvailable message to the controller.
        //     let input = c.find_id::<TextArea>(INPUT1).unwrap();
        //     let text = input.get_content().to_owned();
        //     controller_tx_clone.send(
        //         ControllerMessage::UpdatedInputAvailable(text))
        //         .unwrap();
        // });

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
                         let newtext;
                         { // needs to be in its own scope or output.set_content doesn't work
                            let old = output.get_content();
                            let old_txt = (*old).source();
                            newtext = if old_txt.len() > 0 {format!("{}\n{}", old_txt, text)} else {text};
                         }
                    output.set_content(newtext);
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