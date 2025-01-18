use crate::channel::mpmc::channel::Channel;

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn send() {
        let (tx, rx) = Channel::new::<i32>();
        tx.send(42).unwrap()
    }

    #[test]
    fn receive() {
        let (tx, mut rx) = Channel::new::<i32>();
        tx.send(42).unwrap();
        tx.send(43).unwrap();
        let mut rx_cloned_0 = rx.clone();
        let mut rx_cloned_1 = rx.clone();

        let handle_0 = thread::spawn( move || {
            assert_eq!(42, rx_cloned_0.receive().unwrap());
            assert_eq!(43, rx_cloned_0.receive().unwrap());
        });

        let handle_1 = thread::spawn( move || {
            assert_eq!(42, rx_cloned_1.receive().unwrap());
            assert_eq!(43, rx_cloned_1.receive().unwrap());
        });

        handle_0.join().unwrap();
        handle_1.join().unwrap();
    }

    #[test]
    fn receive_before_send() {
        let (tx, mut rx) = Channel::new::<i32>();

        let mut rx_cloned_0 = rx.clone();
        let mut rx_cloned_1 = rx.clone();

        let handle_0 = thread::spawn( move || {
            assert_eq!(42, rx_cloned_0.receive().unwrap());
            assert_eq!(43, rx_cloned_0.receive().unwrap());
        });

        let handle_1 = thread::spawn( move || {
            assert_eq!(42, rx_cloned_1.receive().unwrap());
            assert_eq!(43, rx_cloned_1.receive().unwrap());
        });

        tx.send(42).unwrap();
        tx.send(43).unwrap();

        handle_0.join().unwrap();
        handle_1.join().unwrap();
    }
}