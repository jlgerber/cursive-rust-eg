extern crate cursive;
//use cursive::event;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;
use cursive::align::{HAlign,VAlign, Align};
use cursive::menu::{MenuTree,MenuItem};
use std::rc::Rc;
static INPUT1: &'static str = "input1";
static INPUT2: &'static str = "input2";

pub fn s<I>(value: I) -> String where I: Into<String> {
    value.into()
}
/// Messages issues by Controller for Ui
pub enum UiMessage {
    UpdateOutput(String, String),
    Quit,
}

/// Messages issued by UI for controller
pub enum ControllerMessage {
    UpdatedInputAvailable(String, String),
    Quit,
    MenuItemSelected(String),
}

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}


impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>,
               ui_rx: mpsc::Receiver<UiMessage>
    ) -> Ui {
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

        ta.set_on_pre_event(Key::Esc, move |_s| {
            //let input = s.find_id::<TextArea>(INPUT1).unwrap();
            controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });
        let controller_tx_clone = ui.controller_tx.clone();

        ta.set_on_pre_event(Key::Enter, move |s| {
            let text;
            { // going from immutable to mutable borrow...
            let input = s.find_id::<TextArea>(INPUT1).unwrap();
            text = input.get_content().to_string();
            }
            let input = &mut s.find_id::<TextArea>(INPUT1).unwrap();
            input.set_content("");
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(INPUT1.to_string(), text))
                .unwrap();
        });

       let mut tb = OnEventView::new(TextArea::new()
                            .content("")
                            .with_id(INPUT2));
        let controller_tx_clone = ui.controller_tx.clone();
        tb.set_on_pre_event(Key::Esc, move |_s| {
            controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        tb.set_on_pre_event(Key::Enter, move |s| {
            let text;
            {
            let input = s.find_id::<TextArea>(INPUT2).unwrap();
            text = input.get_content().to_string();
            }
            let input =&mut s.find_id::<TextArea>(INPUT2).unwrap();

            input.set_content("");
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(INPUT2.to_string(), text))
                .unwrap();
        });

        //let mut lv = SelectView::new().h_align(HAlign::Center);//.v_align(VAlign::Center);
        //lv.add_item("test1",1);
        //lv.add_item("test2",2);



        let width = SizeConstraint::Fixed(50);
        let half_height = SizeConstraint::Fixed(20);
        let sp_ht = SizeConstraint::Fixed(2);

        let controller_tx_clonec = ui.controller_tx.clone();
        let input_pair = Panel::new(LinearLayout::vertical()
            .child(BoxView::new(width, half_height,ta))
            .child(BoxView::new(width,sp_ht, Button::new_raw("[PopupSelection]",  move | s| {

                let mut mt = MenuTree::new();
                let controller_tx_clone = controller_tx_clonec.clone();

                mt.add_leaf("one", move |_s| {
                    controller_tx_clone.send(
                        ControllerMessage::MenuItemSelected("one".to_string())
                    ).unwrap();
                });
                let controller_tx_clone = controller_tx_clonec.clone();

                mt.add_leaf("two", move |_s| {
                    controller_tx_clone.send(
                        ControllerMessage::MenuItemSelected("two".to_string())
                    ).unwrap();
                });

                let controller_tx_clone = controller_tx_clonec.clone();
                mt.add_leaf("three", move |_s| {
                    controller_tx_clone.send(
                        ControllerMessage::MenuItemSelected("three".to_string())
                    ).unwrap();
                });

                let mp = MenuPopup::new(Rc::new(mt));

                s.add_layer(mp)

            }) ))
            .child(BoxView::new(width,half_height, tb)));

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
                UiMessage::UpdateOutput(ctrl, text) => {
                    let mut output = self.cursive
                         .find_id::<TextView>("output")
                         .unwrap();
                         let newtext;
                         { // needs to be in its own scope or output.set_content doesn't work
                            let old = output.get_content();
                            let old_txt = (*old).source();
                            newtext = if old_txt.len() > 0 {format!("{}\n{}: {}", old_txt, ctrl, text)} else {format!("{}: {}",ctrl, text)};
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

/// Controller holds pointer to ui and channels
pub struct Controller {
    tx: mpsc::Sender<UiMessage>,
    rx: mpsc::Receiver<ControllerMessage>,
    ui: Ui,
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
            ui: Ui::new(c_tx, ui_rx), // removed .clone(). no reason to clone the channel here
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::UpdatedInputAvailable(ctrl,text) => {
                        self.tx
                            .send(UiMessage::UpdateOutput(ctrl, text))
                            .unwrap();
                    },
                    ControllerMessage::Quit => {
                        self.tx.send(UiMessage::Quit).unwrap();
                        break;
                    },
                    ControllerMessage::MenuItemSelected(item) => {
                        self.tx
                            .send(UiMessage::UpdateOutput(s("menu"), item))
                            .unwrap();
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