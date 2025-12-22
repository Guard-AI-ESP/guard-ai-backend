use glob::glob;
use std::fs;

use guard_ai_backend::models::event::EventV1;

#[test]
fn examples_should_deserialize() {
    // Les exemples doivent être accessibles via submodule: contracts/events/v1/examples
    let pattern = "contracts/guard-ai-contracts/events/v1/examples/*.json";
    let mut count = 0;

    for entry in glob(pattern).expect("invalid glob pattern") {
        let path = entry.expect("glob entry");
        let content = fs::read_to_string(&path).expect("read example file");
        let _evt: EventV1 = serde_json::from_str(&content).expect("deserialize EventV1");
        count += 1;
    }

    assert!(count >= 3, "expected at least 3 examples, got {count}");
}
