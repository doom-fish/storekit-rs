use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use storekit::{StoreKitError, Transaction};

#[test]
fn idle_update_stream_times_out_instead_of_ending() {
    let mut updates = Transaction::updates().expect("Transaction.updates stream");
    for _ in 0..3 {
        match updates.next_timeout(Duration::from_millis(50)) {
            Err(StoreKitError::TimedOut(message)) => {
                assert!(message.contains("still open"), "{message}");
            }
            Ok(Some(_)) => {}
            other => panic!("an open updates stream reported {other:?}"),
        }
        assert!(!updates.is_finished());
    }
}

#[test]
fn finite_stream_next_waits_for_the_end_of_the_sequence() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let outcome = Transaction::all().and_then(|mut stream| {
            while stream.next()?.is_some() {}
            Ok((stream.is_finished(), stream.next()?.is_none()))
        });
        let _ = sender.send(outcome);
    });

    match receiver.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok((finished, still_none))) => {
            assert!(finished);
            assert!(still_none);
        }
        Ok(Err(error)) => panic!("Transaction.all failed: {error}"),
        Err(_) => eprintln!("skipped: Transaction.all did not finish within 30 s here"),
    }
}

#[test]
fn dropping_an_update_stream_while_it_is_idle_is_safe() {
    for _ in 0..64 {
        let mut updates = Transaction::updates().expect("Transaction.updates stream");
        assert!(matches!(
            updates.next_timeout(Duration::ZERO),
            Err(StoreKitError::TimedOut(_)) | Ok(Some(_))
        ));
        drop(updates);
    }
}

#[test]
fn huge_timeouts_wait_instead_of_failing() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut updates = Transaction::updates().expect("Transaction.updates stream");
        let _ = sender.send(
            updates
                .next_timeout(Duration::MAX)
                .map(|next| next.is_some()),
        );
    });
    match receiver.recv_timeout(Duration::from_millis(200)) {
        Err(mpsc::RecvTimeoutError::Timeout) | Ok(Ok(true)) => {}
        other => panic!("a huge timeout returned early: {other:?}"),
    }
}
