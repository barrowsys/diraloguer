#![feature(trace_macros)]
use diraloguer::diralogue;
use diraloguer::Toggle;

fn main() {
    // trace_macros!(true);
    diralogue!("Title", "Prompt" => (default = 1; confirmation = "Are you sure you want to quit?";) [
        "Title" => println!("You just selected the first item!");
    ]);
    diralogue!("Diralogue Test" => [
        "Sub Menu" => (confirmation = "are you sure you want to quit?";)[
            Toggle::new("Is this true?").true_text("yes").false_text("no");
            Toggle::new("Is this true? but 2").true_text("yes").false_text("no");
        ];
        "Second Sub Menu" => [
            Toggle::new("Another toggle");
            Toggle::new("Another another toggle");
            "Extra Sub Menu" => [
                Toggle::new("secret toggle!");
            ];
            "Print Cool Stuff" => println!("Cool stuff!!");
        ];
    ]);
}
