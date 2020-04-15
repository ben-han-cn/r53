#![no_main]
use libfuzzer_sys::fuzz_target;
use r53::Message;

fuzz_target!(|data: &[u8]| {
    let _ = Message::from_wire(data);
});
