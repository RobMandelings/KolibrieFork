// struct Event {
//     id: u64,
// }
//
// struct Struct2<'a> {
//     slice: &'a [Event],
// }
//
// // Attempt: Struct1 owns events and also a Struct2 borrowing from them.
// struct Struct1<'a> {
//     events: Vec<Event>,
//     window: Struct2<'a>,
// }
//
// impl<'a> Struct1<'a> {
//     fn new(events: Vec<Event>) -> Self {
//         let s = Struct1 {
//             events,
//             // cannot do: window: Struct2 { slice: &s.events[..] },
//             window: Struct2 { slice: &s.events[..] },
//         };
//         s
//     }
// }
//
// fn main() {
//
// }

fn main() {}