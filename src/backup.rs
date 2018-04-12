extern crate cursive;
use cursive::Cursive;
use cursive::views::{Dialog, TextView};

fn main() {
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    //siv.add_layer(TextView::new("Hello cursive! Press <q> to quit."));
    //siv.add_layer(Dialog::around(TextView::new("...")));

    siv.add_layer(Dialog::text("This is a survey!\nPress <Next> when you're ready.")
        .title("Important survey")
        .button("Next", show_next));

    siv.run();
}


fn show_next(s: &mut Cursive) {
    // Empty for now
    s.pop_layer();
    s.add_layer(Dialog::text("Did you do the thing?")
        .title("Question 1")
        .button("Yes!", |s| show_answer(s, "I knew it! Well done!"))
        .button("No!", |s| show_answer(s, "I knew you couldn't be trusted!"))
        .button("Uh?", |s| s.add_layer(Dialog::info("Try again!"))));
}

fn show_answer(s: &mut Cursive, msg: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(msg)
        .title("Results")
        .button("Finish", |s| s.quit()));
}