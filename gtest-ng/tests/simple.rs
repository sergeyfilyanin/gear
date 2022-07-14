use gtest_ng::System;

#[test]
fn minimal() {
    let sys = System::new();

    let prog = sys
        .submit_code("../target/wasm32-unkown-unknown/release/demo_ping.wasm")
        .init();

    let user = sys.user();
    let result = user.send(prog.id(), "PING");
    assert!(result.is_ok());
}
