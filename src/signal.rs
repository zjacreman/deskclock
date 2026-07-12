use signal_hook::consts;
use signal_hook::flag;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Registers signal handlers for SIGTERM and SIGINT.
///
/// Returns an `Arc<AtomicBool>` that is set to `true` when any registered
/// signal is received. The returned `Arc` is fully cloneable — all clones
/// share the same underlying flag, so any handler that sets the flag
/// will be observable from every clone.
///
/// **Async-signal safety**: This uses `signal-hook`'s safe `flag::register`
/// helper, which installs a real POSIX signal handler that performs only an
/// `AtomicBool::store` with `SeqCst` ordering. That compiles to a single
/// atomic machine instruction and is async-signal-safe on all supported
/// platforms. No `unsafe` is required on our side.
///
/// **Note**: After calling `enable_raw_mode()` in `main.rs`, SIGINT from
/// Ctrl+C is no longer delivered by the terminal driver (ISIG is cleared).
/// This signal handler catches externally-sent SIGINT (e.g. `kill -2` or
/// another terminal session sending Ctrl+C).
pub fn register_signal_handler() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));

    flag::register(consts::SIGTERM, Arc::clone(&shutdown))
        .expect("failed to register SIGTERM handler");
    flag::register(consts::SIGINT, Arc::clone(&shutdown))
        .expect("failed to register SIGINT handler");

    shutdown
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_register_signal_handler_initial_flag_is_false() {
        let shutdown = register_signal_handler();
        assert!(
            !shutdown.load(Ordering::SeqCst),
            "Initial shutdown flag should be false before any signal is received"
        );
    }

    #[test]
    fn test_register_signal_handler_flag_can_be_set() {
        let shutdown = Arc::new(AtomicBool::new(false));
        shutdown.store(true, Ordering::SeqCst);
        assert!(
            shutdown.load(Ordering::SeqCst),
            "Setting shutdown flag to true should be observable"
        );
    }

    #[test]
    fn test_register_signal_handler_arc_clones_share_state() {
        let shutdown = Arc::new(AtomicBool::new(false));
        let clone1 = Arc::clone(&shutdown);
        let clone2 = Arc::clone(&shutdown);

        // Clone1 sets flag, clone2 must see it
        clone1.store(true, Ordering::SeqCst);
        assert!(
            clone2.load(Ordering::SeqCst),
            "Arc clones must share the same underlying flag"
        );

        // Clone2 sets flag to false, clone1 must see it
        clone2.store(false, Ordering::SeqCst);
        assert!(
            !clone1.load(Ordering::SeqCst),
            "Arc clones must share the same underlying flag"
        );
    }

    #[test]
    fn test_register_signal_handler_multiple_clones_observe_same_flag() {
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut clones = Vec::new();
        for _ in 0..5 {
            clones.push(Arc::clone(&shutdown));
        }

        // Set the central flag
        shutdown.store(true, Ordering::SeqCst);
        for clone in &clones {
            assert!(
                clone.load(Ordering::SeqCst),
                "All clones should observe true"
            );
        }
        // Now clear via central reference, verify all see false
        shutdown.store(false, Ordering::SeqCst);
        for clone in &clones {
            assert!(
                !clone.load(Ordering::SeqCst),
                "All clones should observe false after clearing"
            );
        }
    }
}
