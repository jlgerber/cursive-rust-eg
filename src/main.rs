#[macro_use] extern crate lazy_static;
extern crate cursive;
use cursive::{ Cursive, event::Key, align::HAlign,
               menu::MenuTree, view::*, views::* };

use std::{rc::Rc, sync::mpsc, collections::HashMap };

lazy_static! {
    static ref GROUP_PROJECTS: HashMap<String, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("2d".to_string(), vec!["nukestartup","nukemenus","openandupdate","indiaspecial"]);
        m.insert("layout".to_string(), vec!["layouttools","layoutpipeline","layouteffort"]);
        m.insert("transfer".to_string(), vec!["baz", "bla","barg"]);
        m
    };
}
//
//   static labels
//
static INPUT1: &'static str = "input1";
static INPUT2: &'static str = "input2";
static MSGV:   &'static str = "message";
static OUTPUT: &'static str = "output";
static BTN1:   &'static str = "button1";
static GROUP:  &'static str = "group";
static PROJECT:&'static str = "project";

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
    UpdateProjectForGroup(String),
}

/// Messages issued by UI for controller
pub enum ControllerMessage {
    UpdatedInputAvailable(String, String),
    Quit,
    MenuItemSelected(String),
    UpdatedMsg(String),
    PopupPressed(String),
    GroupSelected(String),
    ProjectSelected(String),
    GroupProjOkSelected,
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
            .child( Button::new("Popup", move |_s| {
                controller_tx_clone.send(
                    ControllerMessage::PopupPressed("Main Popup".to_string())
                ).unwrap();
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

        //let controller_tx_clonec = self.get_out_chan();
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
        self
    }

    // Set the message
    pub fn message<'a>(&mut self, msg: &'a str) {
        self.cursive
            .find_id::<TextView>(MSGV)
            .unwrap()
            .set_content(msg);
    }

    // displays a dialog
    fn display_dialog(&mut self, title: String) {
        let controller_tx_clone = self.get_out_chan();

        let mut input1 = SelectView::new();
        let mut input2: SelectView = SelectView::new();

        // Populate the first input
        for key in GROUP_PROJECTS.keys() {
            let keystr = key.as_str();
            input1.add_item(keystr,keystr);
        }

        // set callback for when the user selects it
        input1.set_on_submit(  move | _s, item:&str| {
            // send message indicating that the group has been selected
            controller_tx_clone.send(
                ControllerMessage::GroupSelected(item.to_string())
            ).unwrap();
        });

        let controller_tx_clone = self.get_out_chan();

        input2.set_on_submit( move | _s, item:&str| {
            controller_tx_clone.send(
                ControllerMessage::ProjectSelected(item.to_string())
            ).unwrap();
        });
        let input1 = LinearLayout::vertical()
            .child(TextView::new("Group"))
            .child(TextView::new("-----"))
            .child(IdView::new(GROUP, input1));
        let input2 = LinearLayout::vertical()
            .child(TextView::new("Project"))
            .child(TextView::new("-------"))
            .child(IdView::new(PROJECT, input2));

        let content = Panel::new(LinearLayout::horizontal()
            .child(BoxView::new(SizeConstraint::AtLeast(15), SizeConstraint::Full, input1))
            .child(BoxView::new(SizeConstraint::AtLeast(15), SizeConstraint::Full,input2)));
        let wrapped_content = LinearLayout::vertical()
            .child(TextView::new(title).h_align(HAlign::Center))
            .child(content);
        let controller_tx_clone = self.get_out_chan();

        let dialog = Dialog::new()
            .content(wrapped_content)
            .dismiss_button("Cancel")
            .button("Ok",   move |s| {
                controller_tx_clone.send(
                    ControllerMessage::GroupProjOkSelected
                ).unwrap();
                s.pop_layer();
            });

        let result = BoxView::new(SizeConstraint::AtLeast(40), SizeConstraint::Fixed(20), dialog);
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
                },
                UiMessage::UpdateProjectForGroup(group) => {
                   // let project =  self.cursive.find_id::<SelectView>(PROJECT).unwrap();
                   //let grpref = &group.as_str();
                    let results = &GROUP_PROJECTS[&group];
                    // find the project
                    let mut project_selectview = self.cursive.find_id::<SelectView>(PROJECT).unwrap();

                    // clear the project. Surprisingly, there isn't a convenience function
                    // to do this.
                    loop {
                        let len = project_selectview.len();
                        if len == 0 {
                            break;
                        }
                        project_selectview.remove_item(len-1);
                    }

                    // Now add the new projects matcing the current group
                    for key in results {
                        project_selectview.add_item(*key, (*key).to_string());
                    }
                },
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
    group: Option<String>,
    project: Option<String>,
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
            group: None,
            project: None,
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
                        self.tx
                            .send(UiMessage::Quit)
                            .unwrap();
                        break;
                    },

                    ControllerMessage::MenuItemSelected(item) => {
                        self.tx
                            .send(UiMessage::UpdateOutput(s("menu"), item))
                            .unwrap();
                    },

                    ControllerMessage::UpdatedMsg(message) => {
                        self.tx.send(UiMessage::Msg(message)).unwrap();
                    },

                    ControllerMessage::PopupPressed(message) => {
                        self.tx
                            .send(UiMessage::DisplayDialog(message))
                            .unwrap();
                    },

                    ControllerMessage::GroupSelected(group) => {
                        self.tx.send(UiMessage::Msg(format!("Group {} selected",group))).unwrap();
                        self.tx.send(UiMessage::UpdateProjectForGroup(group.clone())).unwrap();

                        self.group = Some(group);
                        self.project = None;
                    },

                    ControllerMessage::ProjectSelected(project) =>{
                        self.tx.send(UiMessage::Msg(format!("Project {} selected",project))).unwrap();
                        self.project = Some(project);
                    },

                    ControllerMessage::GroupProjOkSelected => {
                        let mut problem= false;
                        if self.group.is_none() {
                            self.tx.send(UiMessage::Msg(s("WARNING - group is not set."))).unwrap();
                            problem = true;
                        }
                        if self.project.is_none() {
                            self.tx.send(UiMessage::Msg(s("WARNING - project is not set."))).unwrap();
                            problem = true;
                        }
                        if !problem {
                            self.tx
                                .send(UiMessage::UpdateOutput(s("Group"), self.group.clone().unwrap()))
                                .unwrap();
                            self.tx
                                .send(UiMessage::UpdateOutput(s("Project"), self.project.clone().unwrap()))
                                .unwrap();
                        }
                    }
                // End of pattern match
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