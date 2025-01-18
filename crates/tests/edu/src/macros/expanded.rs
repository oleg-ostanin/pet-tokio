// #![feature(prelude_import)]
// #[prelude_import]
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// mod basic {
//     mod hw {}
// }
// mod macros {
//     mod hw {
//         fn always_true() -> bool {
//             true
//         }
//         fn equals(x: i32, y: i32) -> bool {
//             x == y
//         }
//         fn expanded() {
//             let res = {
//                 let mut res = true;
//                 'block: {
//                     if (!always_true()) {
//                         res = false;
//                         break 'block;
//                     }
//                     if (!equals(2, 3)) {
//                         res = false;
//                         break 'block;
//                     }
//                     if (!equals(3, 3)) {
//                         res = false;
//                         break 'block;
//                     }
//                     if (!equals(3, 3)) {
//                         res = false;
//                         break 'block;
//                     }
//                 }
//                 res
//             };
//             {
//                 ::std::io::_print(format_args!("res: {0:?}\n", res));
//             };
//         }
//     }
//     mod expanded {}
// }
