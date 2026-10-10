use std::io;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::fs;

use chrono::prelude::*;
use udev::MonitorBuilder;
use zbus::blocking::Connection;
use zmq;

const PAUSE_FILE: &str = "/var/lib/byeusb/monitoring.pause";

fn power_off() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::system()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;
    proxy.call_method("PowerOff", &(false,))?;
    println!("[POWER] Shutdown requested.");
    Ok(())
}

fn usb_monitor_loop(paused: Arc<AtomicBool>) -> io::Result<()> {
    println!("[INIT] Starting USB monitor...");

    let socket = MonitorBuilder::new()?
        .match_subsystem("usb")?
        .listen()?;

    println!("[INIT] USB monitor started.");
    println!("[INIT] Waiting for USB events...");

    let fd = socket.as_raw_fd();

    loop {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let result = unsafe { libc::poll(&mut pollfd, 1, 250) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        if result == 0 {
            continue;
        }

        if pollfd.revents & libc::POLLIN != 0 {
            let mut iter = socket.iter();
            while let Some(event) = iter.next() {
                let action = event.action();
                let devnode = event.devnode();
                let syspath = event.syspath();

                println!(
                    "[{}] USB EVENT: action={:?}, devnode={:?}, syspath={:?}",
                    Utc::now(),
                    action,
                    devnode,
                    syspath
                );

                if paused.load(Ordering::SeqCst) || fs::metadata(PAUSE_FILE).is_ok() {
                    continue;
                }

                if let Err(e) = power_off() {
                     eprintln!("[LOCK] failed: {}", e);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PULL)?;
    socket.bind("tcp://127.0.0.1:15641")?;

    let paused = Arc::new(AtomicBool::new(false));
    let paused_clone = Arc::clone(&paused);

    thread::spawn(move || {
        if let Err(e) = usb_monitor_loop(paused_clone) {
            eprintln!("[USB] monitor error: {}", e);
        }
    });

    loop {
        let msg = socket.recv_msg(0)?;
        let text = msg.as_str().unwrap_or("<invalid utf-8>").to_string();

        match text.as_str() {
            "start" => {
                paused.store(false, Ordering::SeqCst);
                let _ = fs::remove_file(PAUSE_FILE);
                println!("[MAIN] Monitor resumed.");
            }
            "stop" => {
                paused.store(true, Ordering::SeqCst);
                println!("[MAIN] Monitor paused.");
            }
            other => println!("[MAIN] Unknown command: {}", other),
        }
    }
}