extern crate cursive;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;
use cursive::align::{HAlign,VAlign, Align};
use cursive::menu::{MenuTree,MenuItem};
use std::rc::Rc;

//
//   static labels
//
static INPUT1: &'static str = "input1";
static INPUT2: &'static str = "input2";
static MSGV:   &'static str = "message";
static OUTPUT: &'static str = "output";
static BTN1:   &'static str = "button1";
//
//  Helper Functions
//
fn s<I>(value: I) -> String
    where I: Into<String> {
    value.into()
}

//
//  Messages
//
/// Messages issues by Controller for Ui
pub enum UiMessage {
    UpdateOutput(String, String),
    Quit,
    Msg(String),
    DisplayDialog(String),
}

/// Messages issued by UI for controller
pub enum ControllerMessage {
    UpdatedInputAvailable(String, String),
    Quit,
    MenuItemSelected(String),
    UpdatedMsg(String),
    PopupPressed(String),
}

//
//  View
//
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
         Ui {
            cursive: Cursive::new(),
            ui_rx: ui_rx,
            controller_tx: controller_tx,
        }
    }

    /// Get a clone of the outgoing channel
    fn get_out_chan(&mut self) -> mpsc::Sender<ControllerMessage>  {
        self.controller_tx.clone()
    }

    fn build_eventview(&mut self, title: &'static str) -> OnEventView<IdView<TextArea>> {
        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let mut ta = OnEventView::new(TextArea::new()
                            .content("")
                            .with_id(title));
        let controller_tx_clone = self.get_out_chan();

        ta.set_on_pre_event(Key::Esc, move |_s| {
            controller_tx_clone.send(
                ControllerMessage::Quit)
                .unwrap();
        });
        let controller_tx_clone = self.get_out_chan();

        ta.set_on_pre_event(Key::Enter, move |s| {
            let text;
            { // going from immutable to mutable borrow...
            let input = s.find_id::<TextArea>(title).unwrap();
            text = input.get_content().to_string();
            }
            let input = &mut s.find_id::<TextArea>(title).unwrap();
            input.set_content("");
            controller_tx_clone.send(
                ControllerMessage::UpdatedInputAvailable(title.to_string(), text))
                .unwrap();
            controller_tx_clone.send(
                ControllerMessage::UpdatedMsg(format!("updated textarea {}",title))
            ).unwrap();
        });
        ta
    }

    // build a button which pops up a menu and communicates the choice
    fn build_pushbutton(&mut self) -> IdView<Button> {
        let controller_tx_clonec = self.get_out_chan();

        let b1 = Button::new_raw("PopupSelection", move | s| {
            let mut mt = MenuTree::new();
            let controller_tx_clone = controller_tx_clonec.clone();

            mt.add_leaf("one", move |_s| {
                controller_tx_clone.send(
                    ControllerMessage::MenuItemSelected("one".to_string())
                ).unwrap();
                controller_tx_clone.send(
                    ControllerMessage::UpdatedMsg("selected menu item - one".to_string())
                ).unwrap();
            });
            let controller_tx_clone = controller_tx_clonec.clone();

            mt.add_leaf("two", move |_s| {
                controller_tx_clone.send(
                    ControllerMessage::MenuItemSelected("two".to_string())
                ).unwrap();
               controller_tx_clone.send(
                    ControllerMessage::UpdatedMsg("selected menu item - two".to_string())
                ).unwrap();
            });

            let controller_tx_clone = controller_tx_clonec.clone();
            mt.add_leaf("three", move |_s| {
                controller_tx_clone.send(
                    ControllerMessage::MenuItemSelected("three".to_string())
                ).unwrap();
               controller_tx_clone.send(
                    ControllerMessage::UpdatedMsg("selected menu item - three".to_string())
                ).unwrap();
            });

            let mp = MenuPopup::new(Rc::new(mt));
            s.add_layer(mp)
        }).with_id("foo");
        b1
    }

    fn build_rightview(&mut self) -> LinearLayout {
        let controller_tx_clone = self.get_out_chan();
        LinearLayout::vertical()
            .child(BoxView::new( SizeConstraint::Full, SizeConstraint::Full, TextView::new("").with_id(OUTPUT)))
            .child( Button::new("Popup", move |mut s| {
                controller_tx_clone.send(
                    ControllerMessage::PopupPressed("Main Popup".to_string())
                );
            }).with_id(BTN1))
    }
    /// build the ui
    pub fn build(mut self) -> Self {
        //
        let width       = SizeConstraint::Fixed(50);
        let height      = SizeConstraint::Fixed(30);
        let half_height = SizeConstraint::Fixed(14);
        let sp_ht       = SizeConstraint::Fixed(1);

        let label_width = SizeConstraint::Fixed(10);
        let msg_ht      = SizeConstraint::Fixed(1);
        let msg_width   = SizeConstraint::Fixed(80);

        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let ta = self.build_eventview(INPUT1);
        let tb = self.build_eventview(INPUT2);

        let controller_tx_clonec = self.get_out_chan();
        let b1 = self.build_pushbutton();

        let left_side = Panel::new(LinearLayout::vertical()
            .child(BoxView::new(width, half_height, ta))
            .child(Panel::new(BoxView::new(width,sp_ht, b1)))
            .child(BoxView::new(width,half_height, tb)));

        let right_side = Panel::new(self.build_rightview());

        let main = LinearLayout::horizontal()
            .child(BoxView::new(width, height, left_side))
            .child(BoxView::new(width, height, right_side));

        let message = Panel::new(LinearLayout::horizontal()
            .child(BoxView::new(label_width, msg_ht, TextView::new("MESSAGE: ")))
            .child(BoxView::new(msg_width, msg_ht, TextView::new("").with_id(MSGV))));

        let main_w_msg = LinearLayout::vertical()
            .child(main)
            .child(message);


        self.cursive.add_layer(main_w_msg);

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

        self
    }

    pub fn message<'a>(&mut self, msg: &'a str) {
        self.cursive
            .find_id::<TextView>(MSGV)
            .unwrap()
            .set_content(msg);
    }

    // displays a dialog
    fn display_dialog(&mut self, title: String) {
        let input1 = LinearLayout::horizontal()
            .child(TextView::new("Group:   "))
            .child( BoxView::new( SizeConstraint::Fixed(30), SizeConstraint::Fixed(1),
                TextArea::new().with_id("group"))
            );
        let input2 = LinearLayout::horizontal()
            .child(TextView::new("Project: "))
            .child( BoxView::new(SizeConstraint::Fixed(30), SizeConstraint::Fixed(1),
                TextArea::new().with_id("project"))
            );
        let content = LinearLayout::vertical()
            .child(TextView::new(title))
            .child(input1)
            .child(input2);

        let dialog = Dialog::new()
            .content(content)
            .dismiss_button("Cancel")
            .button("Ok", |s| s.quit());
        let result = BoxView::new(SizeConstraint::AtLeast(40), SizeConstraint::Fixed(10), dialog);
        self.cursive.add_layer(result);
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
                },
                UiMessage::Msg(message) => {
                    self.message(&message.as_str());
                },
                UiMessage::DisplayDialog(title) => {
                    self.display_dialog(title);
                }
            }
        }

        // Step the UI
        self.cursive.step();

        true
    }
}

//
//  Controller
//
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
        let (c_tx, c_rx)  = mpsc::channel::<ControllerMessage>();
        let (ui_tx,ui_rx) = mpsc::channel::<UiMessage>();
        let mut ui = Ui::new(c_tx, ui_rx).build();
        ui.message("Startup Successful");
        Ok(Controller {
            tx: ui_tx,
            rx: c_rx,
            ui: ui,
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
                    },
                    ControllerMessage::UpdatedMsg(message) => {
                        self.tx.send(UiMessage::Msg(message));
                    },
                    ControllerMessage::PopupPressed(message) => {
                        self.tx.send(UiMessage::DisplayDialog(message));
                    },
                };
            }
        }
    }
}

//
//  Main
//

fn main() {
    // Launch the controller and UI
    let controller = Controller::new();
    match controller {
        Ok(mut controller) => controller.run(),
        Err(e) => println!("Error: {}", e),
    };
}